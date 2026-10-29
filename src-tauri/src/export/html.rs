//! HTML: one self-contained HTML5 file. Pictures are embedded as `data:` addresses, the few
//! styles are inline, and there are no scripts and no outside resources: a strict
//! `Content-Security-Policy` meta tag says so too, so a browser would refuse anything else.
//!
//! The markup is plain and conventional (headings, `p`, `ul`/`ol`, `blockquote`, `pre`, `table`,
//! `details`, `mark`, `span style="color:…"`), so Notaya's own HTML importer and other tools read
//! it back. Every piece of text, every attribute and every address is escaped; colors and sizes
//! are written only from values the document model has already checked. Links stay live only for
//! `http`, `https` and `mailto`; any other address is written as plain text.

use std::collections::HashMap;

use super::doc::{Align, Block, Cell, Image, Inline, ListKind, Marks};
use super::{ExportDoc, Part, Slot};
use crate::error::{Error, Result};

/// The most picture data one HTML file may carry (before it is encoded, about a third more).
pub const MAX_PICTURE_BYTES: u64 = 256 * 1024 * 1024;

const CSP: &str = "default-src 'none'; img-src data:; style-src 'unsafe-inline'";

const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#656d76;--line:#d0d7de;--soft:#f6f8fa;--link:#0969da}\
@media (prefers-color-scheme:dark){:root{--bg:#16181d;--fg:#e6e8eb;--muted:#9aa3ad;--line:#3a3f47;--soft:#20242b;--link:#6cb0ff}}\
body{margin:0 auto;max-width:46rem;padding:2rem 1.25rem 4rem;background:var(--bg);color:var(--fg);font:16px/1.6 system-ui,-apple-system,\"Segoe UI\",sans-serif;overflow-wrap:anywhere}\
h1,h2,h3,h4,h5,h6{line-height:1.25;margin:1.6em 0 .5em}\
a{color:var(--link)}\
mark{color:#1f2328;border-radius:.2em;padding:0 .1em}\
code{font-family:ui-monospace,\"Cascadia Mono\",Consolas,monospace;font-size:.92em;background:var(--soft);border-radius:.25em;padding:.1em .3em}\
pre{background:var(--soft);border:1px solid var(--line);border-radius:.5em;padding:.8em 1em;overflow-x:auto}\
pre code{background:none;padding:0}\
blockquote{margin:1em 0;padding:.1em 1em;border-inline-start:4px solid var(--line);color:var(--muted)}\
blockquote.callout{background:var(--soft);border-radius:.5em;color:var(--fg);border-inline-start-color:var(--link)}\
blockquote.callout>p.icon{float:inline-start;margin:.5em .6em 0 0}\
ul.tasks{list-style:none;padding-inline-start:.4em}\
ul.tasks input{margin-inline-end:.5em}\
table{border-collapse:collapse;max-width:100%;margin:1em 0}\
th,td{border:1px solid var(--line);padding:.35em .65em;vertical-align:top}\
th{background:var(--soft)}\
details{margin:.8em 0}summary{cursor:pointer;font-weight:600}\
figure{margin:1em 0}figure img{max-width:100%;height:auto}figcaption{color:var(--muted);font-size:.9em;margin-top:.3em}\
.columns{display:flex;gap:1.5rem;flex-wrap:wrap}.columns>div{flex:1 1 14rem;min-width:0}\
.math{font-family:ui-monospace,Consolas,monospace}\
hr{border:0;border-top:1px solid var(--line);margin:1.5em 0}";

pub fn render(doc: &ExportDoc) -> Result<String> {
    // The size is checked before anything is encoded.
    let mut total = 0u64;
    for id in doc.image_order() {
        if let Some(p) = doc.images.get(&id) {
            total += p.bytes.len() as u64;
        }
    }
    if total > MAX_PICTURE_BYTES {
        return Err(Error::invalid(format!(
            "The pictures are too large for one HTML file ({} MB; the limit is {} MB). Export as Markdown, or as separate files in a ZIP, or choose fewer pages.",
            total / (1024 * 1024),
            MAX_PICTURE_BYTES / (1024 * 1024)
        )));
    }
    let mut pictures: HashMap<String, String> = HashMap::new();
    for id in doc.image_order() {
        if let Some(p) = doc.images.get(&id) {
            // The type comes from the bytes themselves, never from what was stored with them.
            if let Some(mime) = crate::commands::notes::sniff_image(&p.bytes) {
                pictures.insert(id, format!("data:{mime};base64,{}", base64(&p.bytes)));
            }
        }
    }
    let r = Renderer { pictures };
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str(&format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n", esc(CSP)));
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta name=\"generator\" content=\"Notaya\">\n");
    out.push_str(&format!("<title>{}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n", esc(&doc.title)));
    for part in &doc.parts {
        match part {
            Part::Heading { level, text } => {
                let l = (*level).clamp(1, 6);
                out.push_str(&format!("<h{l} dir=\"auto\">{}</h{l}>\n", esc(text)));
            }
            Part::Page { level, title, blocks } => {
                let l = (*level).clamp(1, 6);
                out.push_str(&format!("<h{l} dir=\"auto\">{}</h{l}>\n", esc(title)));
                for line in r.blocks(blocks, *level) {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
    }
    out.push_str("</body>\n</html>\n");
    Ok(out)
}

/// Escapes text for element content and for double-quoted attribute values.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\0' => out.push('\u{FFFD}'),
            c => out.push(c),
        }
    }
    out
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

struct Renderer {
    /// Attachment id to its `data:` address.
    pictures: HashMap<String, String>,
}

fn align_style(a: Align) -> &'static str {
    match a {
        Align::Left => "",
        Align::Center => " style=\"text-align:center\"",
        Align::Right => " style=\"text-align:right\"",
        Align::Justify => " style=\"text-align:justify\"",
    }
}

impl Renderer {
    /// The blocks as lines of HTML. `shift` is added to heading levels (as in the other formats).
    fn blocks(&self, blocks: &[Block], shift: u8) -> Vec<String> {
        blocks.iter().filter_map(|b| self.block(b, shift)).collect()
    }

    fn joined(&self, blocks: &[Block], shift: u8) -> String {
        self.blocks(blocks, shift).join("\n")
    }

    /// The content of a list item or table cell: a first paragraph is written without its `p`.
    fn compact(&self, blocks: &[Block], shift: u8) -> String {
        match blocks.split_first() {
            Some((Block::Paragraph { content, .. }, rest)) => {
                let mut s = inline(content);
                for line in self.blocks(rest, shift) {
                    s.push('\n');
                    s.push_str(&line);
                }
                s
            }
            _ => self.joined(blocks, shift),
        }
    }

    fn block(&self, b: &Block, shift: u8) -> Option<String> {
        Some(match b {
            Block::Paragraph { align, content } => {
                let text = inline(content);
                if text.trim().is_empty() {
                    return None;
                }
                format!("<p dir=\"auto\"{}>{text}</p>", align_style(*align))
            }
            Block::Heading { level, align, content } => {
                let l = level.saturating_add(shift).clamp(1, 6);
                format!("<h{l} dir=\"auto\"{}>{}</h{l}>", align_style(*align), inline(content))
            }
            Block::List { kind, items } => {
                let (open, close) = match kind {
                    ListKind::Bullet => ("<ul dir=\"auto\">".to_string(), "</ul>"),
                    ListKind::Ordered(1) => ("<ol dir=\"auto\">".to_string(), "</ol>"),
                    ListKind::Ordered(n) => (format!("<ol dir=\"auto\" start=\"{n}\">"), "</ol>"),
                    ListKind::Task => ("<ul dir=\"auto\" class=\"tasks\">".to_string(), "</ul>"),
                };
                let mut out = vec![open];
                for item in items {
                    let body = self.compact(&item.blocks, shift);
                    let check = match (kind, item.checked) {
                        (ListKind::Task, Some(true)) => "<input type=\"checkbox\" disabled checked> ",
                        (ListKind::Task, _) => "<input type=\"checkbox\" disabled> ",
                        _ => "",
                    };
                    out.push(format!("<li dir=\"auto\">{check}{body}</li>"));
                }
                out.push(close.to_string());
                out.join("\n")
            }
            Block::Quote(blocks) => format!("<blockquote dir=\"auto\">\n{}\n</blockquote>", self.joined(blocks, shift)),
            Block::Callout { icon, blocks } => {
                let mut s = String::from("<blockquote class=\"callout\" dir=\"auto\">\n");
                if !icon.trim().is_empty() {
                    s.push_str(&format!("<p class=\"icon\">{}</p>\n", esc(icon)));
                }
                s.push_str(&self.joined(blocks, shift));
                s.push_str("\n</blockquote>");
                s
            }
            Block::Toggle(blocks) => {
                let (summary, rest) = match blocks.split_first() {
                    Some((Block::Paragraph { content, .. } | Block::Heading { content, .. }, rest)) => (inline(content), rest),
                    _ => (String::new(), &blocks[..]),
                };
                format!("<details open>\n<summary dir=\"auto\">{summary}</summary>\n{}\n</details>", self.joined(rest, shift))
            }
            Block::Columns(cols) => {
                let mut s = String::from("<div class=\"columns\">\n");
                for c in cols {
                    s.push_str(&format!("<div>\n{}\n</div>\n", self.joined(c, shift)));
                }
                s.push_str("</div>");
                s
            }
            Block::Code { language, text } => {
                let lang: String = language.chars().filter(|c| c.is_ascii_alphanumeric() || "+-#._".contains(*c)).collect();
                let class = if lang.is_empty() { String::new() } else { format!(" class=\"language-{}\"", esc(&lang)) };
                // A browser drops one line break right after <pre>, so a leading one is doubled.
                let lead = if text.starts_with('\n') { "\n" } else { "" };
                format!("<pre dir=\"ltr\"><code{class}>{lead}{}</code></pre>", esc(text))
            }
            Block::Math(latex) => {
                if latex.trim().is_empty() {
                    return None;
                }
                format!("<p class=\"math\" dir=\"ltr\">$${}$$</p>", esc(latex.trim()))
            }
            Block::Image(img) => self.image(img),
            Block::Table(rows) => self.table(rows, shift),
            Block::Rule => "<hr>".to_string(),
            Block::PageLink { title } => format!("<p dir=\"auto\">\u{2192} {}</p>", esc(title)),
        })
    }

    fn image(&self, img: &Image) -> String {
        let Some(src) = img.attachment.as_ref().and_then(|id| self.pictures.get(id)) else {
            return "<p><em>(picture not available)</em></p>".to_string();
        };
        let mut style = String::new();
        if let Some(w) = img.width_pct.filter(|w| w.is_finite() && *w > 0.0 && *w <= 100.0) {
            style.push_str(&format!("width:{}%;", (w * 10.0).round() / 10.0));
        }
        match img.align {
            Align::Center => style.push_str("display:block;margin-inline:auto;"),
            Align::Right => style.push_str("display:block;margin-inline-start:auto;"),
            _ => {}
        }
        let style = if style.is_empty() { String::new() } else { format!(" style=\"{style}\"") };
        let caption = img.caption.trim();
        let tag = format!("<img src=\"{src}\" alt=\"{}\"{style}>", esc(caption));
        if caption.is_empty() {
            format!("<figure>{tag}</figure>")
        } else {
            format!("<figure>{tag}<figcaption dir=\"auto\">{}</figcaption></figure>", esc(caption))
        }
    }

    fn table(&self, rows: &[Vec<Cell>], shift: u8) -> String {
        let grid = super::table_grid(rows);
        if grid.first().is_none_or(Vec::is_empty) {
            return String::new();
        }
        let width = grid[0].len();
        let mut out = vec!["<table>".to_string()];
        for (r, row) in grid.iter().enumerate() {
            let mut line = String::from("<tr>");
            for (c, slot) in row.iter().enumerate() {
                let Slot::Cell(rr, ci) = slot else { continue };
                let cell = &rows[*rr][*ci];
                let tag = if cell.header { "th" } else { "td" };
                // Spans never reach past the grid (the grid has at most 64 columns).
                let cs = cell.colspan.clamp(1, width - c);
                let rs = cell.rowspan.clamp(1, grid.len() - r);
                let mut attrs = String::new();
                if cs > 1 {
                    attrs.push_str(&format!(" colspan=\"{cs}\""));
                }
                if rs > 1 {
                    attrs.push_str(&format!(" rowspan=\"{rs}\""));
                }
                line.push_str(&format!("<{tag} dir=\"auto\"{attrs}>{}</{tag}>", self.compact(&cell.blocks, shift)));
            }
            line.push_str("</tr>");
            out.push(line);
        }
        out.push("</table>".to_string());
        out.join("\n")
    }
}

/// One mark that is open around some text.
#[derive(Clone, PartialEq)]
enum Mark {
    Link(String),
    Style { color: Option<String>, size: Option<String> },
    Highlight(String),
    Bold,
    Italic,
    Strike,
    Underline,
}

impl Mark {
    fn open(&self) -> String {
        match self {
            Mark::Link(url) => format!("<a href=\"{}\" rel=\"noopener noreferrer\">", esc(url)),
            Mark::Style { color, size } => {
                let mut s = String::new();
                if let Some(c) = color {
                    s.push_str(&format!("color:#{c};"));
                }
                if let Some(z) = size {
                    s.push_str(&format!("font-size:{z}pt;"));
                }
                format!("<span style=\"{s}\">")
            }
            Mark::Highlight(c) => format!("<mark style=\"background:#{c}\">"),
            Mark::Bold => "<strong>".into(),
            Mark::Italic => "<em>".into(),
            Mark::Strike => "<s>".into(),
            Mark::Underline => "<u>".into(),
        }
    }
    fn close(&self) -> &'static str {
        match self {
            Mark::Link(_) => "</a>",
            Mark::Style { .. } => "</span>",
            Mark::Highlight(_) => "</mark>",
            Mark::Bold => "</strong>",
            Mark::Italic => "</em>",
            Mark::Strike => "</s>",
            Mark::Underline => "</u>",
        }
    }
}

/// Only web and mail links stay live; anything else (`javascript:`, `file:`, `data:`) is plain text.
fn safe_href(url: &str) -> Option<String> {
    let u: String = url.trim().chars().filter(|c| !c.is_control()).collect();
    let lower = u.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")).then_some(u)
}

/// The marks of a run, outermost first.
fn mark_list(m: &Marks) -> Vec<Mark> {
    let mut v = Vec::new();
    if let Some(url) = m.link.as_deref().and_then(safe_href) {
        v.push(Mark::Link(url));
    }
    let size = m.size.filter(|z| z.is_finite()).map(|z| format!("{}", (z.clamp(6.0, 72.0) * 10.0).round() / 10.0));
    if m.color.is_some() || size.is_some() {
        v.push(Mark::Style { color: m.color.map(|c| c.hex()), size });
    }
    if let Some(c) = m.highlight {
        v.push(Mark::Highlight(c.hex()));
    }
    if m.bold {
        v.push(Mark::Bold);
    }
    if m.italic {
        v.push(Mark::Italic);
    }
    if m.strike {
        v.push(Mark::Strike);
    }
    if m.underline {
        v.push(Mark::Underline);
    }
    v
}

/// Inline content as HTML. Marks that continue over several runs stay open, so the tags are
/// always properly nested.
pub fn inline(content: &[Inline]) -> String {
    let mut out = String::new();
    let mut stack: Vec<Mark> = Vec::new();
    let close_to = |out: &mut String, stack: &mut Vec<Mark>, keep: usize| {
        while stack.len() > keep {
            if let Some(m) = stack.pop() {
                out.push_str(m.close());
            }
        }
    };
    for item in content {
        match item {
            Inline::Text(text, marks) => {
                let want = mark_list(marks);
                let keep = stack.iter().zip(&want).take_while(|(a, b)| a == b).count();
                close_to(&mut out, &mut stack, keep);
                for m in &want[keep..] {
                    out.push_str(&m.open());
                    stack.push(m.clone());
                }
                if marks.code {
                    out.push_str(&format!("<code>{}</code>", esc(text)));
                } else {
                    out.push_str(&esc(text));
                }
            }
            Inline::Break => out.push_str("<br>"),
            Inline::Math(latex) => {
                if !latex.trim().is_empty() {
                    out.push_str(&format!("<span class=\"math\" dir=\"ltr\">${}$</span>", esc(latex.trim())));
                }
            }
        }
    }
    close_to(&mut out, &mut stack, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::super::doc::{self, Rgb};
    use super::super::tests::sample_doc;
    use super::super::{Picture, image::Kind};
    use super::*;
    use crate::importers::markup::{Token, Tokenizer};

    fn page_html(blocks: Vec<Block>) -> String {
        render(&sample_doc(blocks)).unwrap()
    }

    fn para(text: &str, marks: Marks) -> Block {
        Block::Paragraph { align: Align::Left, content: vec![Inline::Text(text.into(), marks)] }
    }

    /// Checks that every start tag has its end tag in the right order (void tags excepted).
    fn assert_well_formed(html: &str) {
        const VOID: [&str; 6] = ["meta", "br", "hr", "img", "input", "link"];
        let mut stack: Vec<String> = Vec::new();
        for t in Tokenizer::html(html) {
            match t {
                Token::Start { name, self_closing, .. } => {
                    if !VOID.contains(&name.as_str()) && !self_closing {
                        stack.push(name);
                    }
                }
                Token::End { name } => {
                    assert_eq!(stack.pop().as_deref(), Some(name.as_str()), "tag </{name}> closes the wrong element");
                }
                Token::Text(_) => {}
            }
        }
        assert!(stack.is_empty(), "unclosed tags: {stack:?}");
    }

    /// Every start tag in the file.
    fn tags(html: &str) -> Vec<(String, Vec<(String, String)>)> {
        Tokenizer::html(html)
            .filter_map(|t| match t {
                Token::Start { name, attrs, .. } => Some((name, attrs)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_page_is_a_valid_self_contained_file() {
        let html = page_html(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        assert!(html.starts_with("<!DOCTYPE html>\n<html>"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("http-equiv=\"Content-Security-Policy\" content=\"default-src &#39;none&#39;; img-src data:; style-src &#39;unsafe-inline&#39;\""));
        assert!(html.contains("prefers-color-scheme:dark"));
        assert_well_formed(&html);
        let all = tags(&html);
        for forbidden in ["script", "iframe", "object", "embed", "link", "form", "base"] {
            assert!(all.iter().all(|(n, _)| n != forbidden), "<{forbidden}> must not appear");
        }
        // No outside resources: the only addresses are data: pictures and the (http) link of the text.
        for (name, attrs) in &all {
            for (k, v) in attrs {
                assert!(!k.starts_with("on"), "event handler {k} on <{name}>");
                if k == "src" {
                    assert!(v.starts_with("data:image/png;base64,"), "{v}");
                }
                if k == "href" {
                    assert_eq!(v, "https://example.com/a?b=1&c=2");
                }
            }
        }
        assert!(html.contains("rel=\"noopener noreferrer\""));
        // The remote picture is not fetched or linked.
        assert!(!html.contains("remote.png"));
        assert!(html.contains("(picture not available)"));
    }

    #[test]
    fn every_kind_of_content_has_its_tag() {
        let html = page_html(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        for needle in [
            "<h2 dir=\"auto\" style=\"text-align:center\">Big heading</h2>",
            "<h5 dir=\"auto\">Small heading</h5>",
            "<strong>bold</strong>",
            "<em>italic</em>",
            "<u>under</u>",
            "<s>struck</s>",
            "<code>x = 1</code>",
            "<span style=\"color:#E5484D;font-size:15pt;\">red</span>",
            "<mark style=\"background:#F8E6AD\">marked</mark>",
            "and &lt;special&gt; *chars* &amp; café € “quotes”.",
            "<br>",
            "<span class=\"math\" dir=\"ltr\">$E = mc^2$</span>",
            "<ul dir=\"auto\">",
            "<ol dir=\"auto\" start=\"3\">",
            "<input type=\"checkbox\" disabled checked> Done task",
            "<input type=\"checkbox\" disabled> Open task",
            "<blockquote dir=\"auto\">",
            "<blockquote class=\"callout\" dir=\"auto\">",
            "<details open>",
            "<summary dir=\"auto\">Toggle line</summary>",
            "<div class=\"columns\">",
            "<pre dir=\"ltr\"><code class=\"language-rust\">",
            "<p class=\"math\" dir=\"ltr\">$$x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}$$</p>",
            "<figure><img src=\"data:image/png;base64,",
            "alt=\"A caption\"",
            "width:50%;",
            "<figcaption dir=\"auto\">A caption</figcaption>",
            "<table>",
            "<th dir=\"auto\">Name</th>",
            "colspan=\"2\"",
            "<hr>",
            "\u{2192} Child page",
            "Unknown block text",
        ] {
            assert!(html.contains(needle), "missing {needle:?} in:\n{html}");
        }
        assert_well_formed(&html);
    }

    #[test]
    fn text_and_attributes_are_escaped() {
        let evil = "<script>alert(1)</script> \"onerror=\"x\" </style><img src=x onerror=alert(1)> &amp; ' \0";
        let blocks = vec![
            para(evil, Marks::default()),
            para(evil, Marks { code: true, ..Marks::default() }),
            para("link", Marks { link: Some(format!("https://a.example/\"onmouseover=\"alert(1)\"{evil}")), ..Marks::default() }),
            Block::Code { language: "rust\"><script>".into(), text: evil.into() },
            Block::Math(evil.into()),
            Block::Image(Image { attachment: Some(doc::tests::IMG_ID.into()), caption: evil.into(), width_pct: Some(50.0), align: Align::Center }),
            Block::Callout { icon: evil.into(), blocks: vec![] },
            Block::PageLink { title: evil.into() },
            Block::Toggle(vec![para(evil, Marks::default())]),
        ];
        let mut d = sample_doc(blocks);
        d.title = evil.into();
        d.parts.push(Part::Heading { level: 2, text: evil.into() });
        let html = render(&d).unwrap();
        // The page itself has no script and no style tag besides its own, and no tag made by the text.
        assert_well_formed(&html);
        let all = tags(&html);
        assert!(all.iter().all(|(n, _)| n != "script"), "{html}");
        assert_eq!(all.iter().filter(|(n, _)| n == "style").count(), 1);
        for (n, attrs) in &all {
            assert!(attrs.iter().all(|(k, _)| !k.starts_with("on")), "<{n}> {attrs:?}");
        }
        assert_eq!(all.iter().filter(|(n, _)| n == "img").count(), 1, "the picture only");
        assert!(!html.contains("<script"));
        assert!(!html.contains('\0'));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&lt;/style&gt;"));
        // The link address is one attribute value, quotes included.
        let link = all.iter().find(|(n, _)| n == "a").unwrap();
        assert_eq!(link.1.iter().filter(|(k, _)| k == "href").count(), 1);
        assert!(link.1.iter().all(|(k, _)| k == "href" || k == "rel"));
        // The language cannot leave its class.
        assert!(html.contains("class=\"language-rustscript\""));
    }

    #[test]
    fn only_web_and_mail_links_stay_live() {
        for (href, live) in [
            ("https://example.com", true),
            ("HTTP://example.com/x", true),
            ("mailto:me@example.com", true),
            ("  https://example.com ", true),
            ("javascript:alert(1)", false),
            ("JaVaScRiPt:alert(1)", false),
            ("java\nscript:alert(1)", false),
            ("data:text/html,<script>alert(1)</script>", false),
            ("file:///C:/secret.txt", false),
            ("//evil.example/x", false),
            ("vbscript:x", false),
            ("", false),
        ] {
            let html = page_html(vec![para("text", Marks { link: Some(href.into()), ..Marks::default() })]);
            let has_a = tags(&html).iter().any(|(n, _)| n == "a");
            assert_eq!(has_a, live, "{href:?}");
            assert!(html.contains("text"), "the text stays");
            assert_well_formed(&html);
        }
    }

    #[test]
    fn colors_are_kept_and_only_from_validated_values() {
        let marks = Marks { color: Some(Rgb(0x11, 0x22, 0x33)), highlight: Some(Rgb(0xFE, 0xF0, 0x8A)), bold: true, ..Marks::default() };
        let html = page_html(vec![para("x", marks)]);
        assert!(html.contains("<span style=\"color:#112233;\"><mark style=\"background:#FEF08A\"><strong>x</strong></mark></span>"), "{html}");
        // A color the model could not read never reaches the file.
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"y","marks":[{"type":"textStyle","attrs":{"color":"red;background:url(http://evil.example)"}},{"type":"highlight","attrs":{"color":"expression(alert(1))"}}]}]}]}"#;
        let html = page_html(doc::parse(json, &|_| None));
        assert!(!html.contains("evil.example") && !html.contains("expression"));
        assert!(html.contains("<mark style=\"background:#FEF08A\">y</mark>"), "the default highlight");
    }

    #[test]
    fn marks_continue_across_runs_and_stay_nested() {
        let bold = Marks { bold: true, ..Marks::default() };
        let both = Marks { bold: true, italic: true, ..Marks::default() };
        let content = vec![
            Inline::Text("a ".into(), bold.clone()),
            Inline::Text("b".into(), both),
            Inline::Text(" c".into(), bold),
            Inline::Text(" plain".into(), Marks::default()),
        ];
        assert_eq!(inline(&content), "<strong>a <em>b</em> c</strong> plain");
    }

    #[test]
    fn right_to_left_text_gets_dir_auto_everywhere() {
        let html = page_html(vec![
            para("مرحبا بالعالم", Marks::default()),
            Block::List { kind: ListKind::Bullet, items: vec![doc::ListItem { checked: None, blocks: vec![para("שלום", Marks::default())] }] },
            Block::Quote(vec![para("نص", Marks::default())]),
        ]);
        assert!(html.contains("<p dir=\"auto\">مرحبا بالعالم</p>"));
        assert!(html.contains("<ul dir=\"auto\">\n<li dir=\"auto\">שלום</li>"));
        assert!(html.contains("<blockquote dir=\"auto\">"));
        assert!(html.contains("<h1 dir=\"auto\">Sample page</h1>"));
    }

    #[test]
    fn tables_keep_spans_and_cap_columns() {
        let c = |t: &str, cs, rs| Cell { header: false, colspan: cs, rowspan: rs, widths: vec![], blocks: vec![para(t, Marks::default())] };
        let rows = vec![vec![c("a", 2, 1), c("b", 1, 2)], vec![c("c", 1, 1), c("d", 1, 1)]];
        let html = page_html(vec![Block::Table(rows)]);
        assert!(html.contains("<tr><td dir=\"auto\" colspan=\"2\">a</td><td dir=\"auto\" rowspan=\"2\">b</td></tr>"), "{html}");
        assert!(html.contains("<tr><td dir=\"auto\">c</td><td dir=\"auto\">d</td></tr>"));
        // A very wide row is cut at 64 columns.
        let wide = vec![(0..200).map(|i| c(&i.to_string(), 1, 1)).collect::<Vec<_>>()];
        let html = page_html(vec![Block::Table(wide)]);
        assert_eq!(html.matches("<td ").count(), 64);
        assert_well_formed(&html);
    }

    #[test]
    fn pictures_are_typed_by_their_bytes() {
        let mut d = sample_doc(vec![Block::Image(Image { attachment: Some(doc::tests::IMG_ID.into()), caption: String::new(), width_pct: None, align: Align::Left })]);
        // Stored as a PNG, but the bytes are a GIF: the file says GIF.
        d.images.insert(doc::tests::IMG_ID.into(), Picture { kind: Kind::Png, bytes: b"GIF89a\x01\0\x01\0\x80\0\0\0\0\0\xff\xff\xff,\0\0\0\0\x01\0\x01\0\0\x02\x02D\x01\0;".to_vec() });
        let html = render(&d).unwrap();
        assert!(html.contains("src=\"data:image/gif;base64,R0lGODlh"), "{html}");
        // Bytes that are no picture at all (for example an SVG with a script) are never embedded.
        d.images.insert(doc::tests::IMG_ID.into(), Picture { kind: Kind::Png, bytes: b"<svg onload=alert(1)></svg>".to_vec() });
        let html = render(&d).unwrap();
        assert!(!html.contains("<svg") && !html.contains("data:image") && html.contains("(picture not available)"));
    }

    #[test]
    fn base64_matches_the_standard_examples() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(crate::importers::base64(&base64(&all)), Some(all), "round trip through the importers' decoder");
    }

    #[test]
    fn too_many_picture_bytes_are_refused_with_a_message() {
        let mut d = sample_doc(vec![Block::Image(Image { attachment: Some(doc::tests::IMG_ID.into()), caption: String::new(), width_pct: None, align: Align::Left })]);
        // A picture just over the limit (zeros after a PNG signature; nothing is encoded).
        let mut big = vec![0u8; MAX_PICTURE_BYTES as usize + 1];
        big[..8].copy_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        d.images.insert(doc::tests::IMG_ID.into(), Picture { kind: Kind::Png, bytes: big });
        let err = render(&d).unwrap_err().to_string();
        assert!(err.contains("too large for one HTML file") && err.contains("256 MB"), "{err}");
    }

    #[test]
    fn text_starting_with_a_line_break_in_code_survives() {
        let html = page_html(vec![Block::Code { language: String::new(), text: "\nx".into() }]);
        assert!(html.contains("<pre dir=\"ltr\"><code>\n\nx</code></pre>"));
    }
}
