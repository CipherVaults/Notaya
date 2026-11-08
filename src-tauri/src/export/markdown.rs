//! Markdown (CommonMark with the GitHub table and task-list extensions). Underline, text colors
//! and highlights have no Markdown form, so they are written as small HTML tags, which most
//! Markdown viewers show. Toggles become `<details>`, equations `$…$` and `$$…$$`.
//! Pictures are written as files in a folder next to the `.md` file and linked from it.

use std::collections::HashMap;

use super::doc::{inline_text, Align, Block, Cell, Inline, ListKind, Marks, PageRef};
use super::{ExportDoc, Part, Slot};

pub struct Output {
    pub text: String,
    /// The picture files to write into the pictures folder: (file name, bytes).
    pub images: Vec<(String, Vec<u8>)>,
}

/// `assets_dir` is the name of the pictures folder next to the `.md` file.
pub fn render(doc: &ExportDoc, assets_dir: &str) -> Output {
    let mut names: HashMap<String, String> = HashMap::new();
    let mut images = Vec::new();
    for id in doc.image_order() {
        if let Some(p) = doc.images.get(&id) {
            let name = format!("image-{:03}.{}", images.len() + 1, p.kind.ext());
            names.insert(id.clone(), format!("{}/{}", url_path(assets_dir), name));
            images.push((name, p.bytes.clone()));
        }
    }
    let r = Renderer { names };
    let mut chunks: Vec<String> = Vec::new();
    for part in &doc.parts {
        match part {
            Part::Heading { level, text } => chunks.push(format!("{} {}", "#".repeat(*level as usize), escape_text(text))),
            Part::Page { level, title, blocks } => {
                chunks.push(format!("{} {}", "#".repeat(*level as usize), escape_text(title)));
                let body = r.blocks(blocks, *level).join("\n");
                if !body.trim().is_empty() {
                    chunks.push(body);
                }
            }
        }
    }
    let mut text = chunks.join("\n\n");
    text.push('\n');
    Output { text, images }
}

/// Percent-encodes a relative path for a Markdown link (spaces and other unsafe characters).
fn url_path(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

struct Renderer {
    /// Attachment id to its relative link.
    names: HashMap<String, String>,
}

impl Renderer {
    /// The blocks as lines, with a blank line between blocks. `shift` is added to heading levels.
    fn blocks(&self, blocks: &[Block], shift: u8) -> Vec<String> {
        self.blocks_in(blocks, shift, false)
    }

    /// `in_item`: the blocks of one list item, where a nested list follows its line directly.
    fn blocks_in(&self, blocks: &[Block], shift: u8, in_item: bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (i, b) in blocks.iter().enumerate() {
            let lines = self.block(b, shift);
            if lines.is_empty() {
                continue;
            }
            let tight = in_item && i > 0 && matches!(b, Block::List { .. }) && matches!(blocks[i - 1], Block::Paragraph { .. });
            if !out.is_empty() && !tight {
                out.push(String::new());
            }
            out.extend(lines);
        }
        out
    }

    fn block(&self, b: &Block, shift: u8) -> Vec<String> {
        match b {
            Block::Paragraph { content, .. } => {
                let text = inline(content, false);
                if text.trim().is_empty() {
                    return vec![];
                }
                text.split('\n').map(|l| escape_line_start(l)).collect()
            }
            Block::Heading { level, content, .. } => {
                let level = (level + shift).min(6) as usize;
                vec![format!("{} {}", "#".repeat(level), inline(content, true))]
            }
            Block::List { kind, items } => {
                let mut out = Vec::new();
                for (n, item) in items.iter().enumerate() {
                    let marker = match kind {
                        ListKind::Bullet => "- ".to_string(),
                        ListKind::Ordered(start) => format!("{}. ", *start as usize + n),
                        ListKind::Task => if item.checked == Some(true) { "- [x] " } else { "- [ ] " }.to_string(),
                    };
                    let body = self.blocks_in(&item.blocks, shift, true);
                    let pad = " ".repeat(if matches!(kind, ListKind::Task) { 2 } else { marker.len() });
                    if body.is_empty() {
                        out.push(marker.trim_end().to_string());
                    }
                    for (k, line) in body.iter().enumerate() {
                        out.push(if k == 0 {
                            format!("{marker}{line}")
                        } else if line.is_empty() {
                            String::new()
                        } else {
                            format!("{pad}{line}")
                        });
                    }
                }
                out
            }
            Block::Quote(blocks) => quote(self.blocks(blocks, shift)),
            Block::Callout { icon, blocks } => {
                let mut lines = self.blocks(blocks, shift);
                match lines.first_mut() {
                    Some(first) if !icon.is_empty() => *first = format!("{} {first}", escape_text(icon)),
                    None => lines.push(escape_text(icon)),
                    _ => {}
                }
                quote(lines)
            }
            Block::Toggle(blocks) | Block::Shelf(blocks) => {
                let (summary, rest) = match blocks.split_first() {
                    Some((Block::Paragraph { content, .. } | Block::Heading { content, .. }, rest)) => (inline_text(content), rest),
                    _ => (String::new(), &blocks[..]),
                };
                // A shelf is marked with a fixed class, so Notaya's importer brings it back as one.
                let open = if matches!(b, Block::Shelf(_)) { "<details open class=\"notaya-shelf\">" } else { "<details open>" };
                let mut out = vec![open.to_string(), format!("<summary>{}</summary>", html_escape(&summary))];
                let body = self.blocks(rest, shift);
                if !body.is_empty() {
                    out.push(String::new());
                    out.extend(body);
                    out.push(String::new());
                }
                out.push("</details>".to_string());
                out
            }
            Block::Columns(cols) => {
                let all: Vec<Block> = cols.iter().flatten().cloned().collect();
                self.blocks(&all, shift)
            }
            Block::Code { language, text } => {
                let longest = longest_run(text, '`');
                let fence = "`".repeat(longest.max(2) + 1);
                let lang: String = language.chars().filter(|c| c.is_ascii_alphanumeric() || "+-#._".contains(*c)).collect();
                let mut out = vec![format!("{fence}{lang}")];
                out.extend(text.split('\n').map(str::to_string));
                out.push(fence);
                out
            }
            Block::Math(latex) => {
                if latex.trim().is_empty() {
                    return vec![];
                }
                let mut out = vec!["$$".to_string()];
                out.extend(latex.trim().split('\n').map(str::to_string));
                out.push("$$".to_string());
                out
            }
            Block::Image(img) => {
                let alt = escape_text(&img.caption);
                let mut out = match img.attachment.as_ref().and_then(|id| self.names.get(id)) {
                    Some(path) => vec![format!("![{alt}]({path})")],
                    None => vec!["*(picture not available)*".to_string()],
                };
                if !img.caption.trim().is_empty() {
                    out.push(String::new());
                    out.push(format!("*{}*", escape_text(img.caption.trim())));
                }
                if img.align == Align::Center {
                    // Markdown has no alignment; the picture is left as it is.
                }
                out
            }
            Block::Table(rows) => table(rows),
            Block::Database { id, table, .. } => {
                let mut out = self.blocks(&super::doc::database_blocks(table.as_ref()), shift);
                if let Some(t) = table {
                    out.push(String::new());
                    out.push(database_reference(id, &t.title));
                }
                out
            }
            Block::Rule => vec!["---".to_string()],
            Block::PageLink { title, target } => vec![page_link_row(title, target.as_ref())],
        }
    }
}

fn database_reference(id: &str, title: &str) -> String {
    // One encoded path segment: even a malformed stored id cannot inject a second link.
    format!("[database: {}](notaya://db/{})", escape_text(title), super::doc::url_segment(id))
}

fn quote(lines: Vec<String>) -> Vec<String> {
    lines.into_iter().map(|l| if l.is_empty() { ">".to_string() } else { format!("> {l}") }).collect()
}

/// A GitHub-style table. Merged cells are written as the cell plus empty cells, because
/// Markdown tables cannot merge cells.
fn table(rows: &[Vec<Cell>]) -> Vec<String> {
    let grid = super::table_grid(rows);
    let cols = grid.first().map_or(0, |r| r.len());
    if cols == 0 {
        return vec![];
    }
    let text_of = |slot: &Slot| -> String {
        match slot {
            Slot::Cell(r, c) => cell_text(&rows[*r][*c]),
            _ => String::new(),
        }
    };
    let line = |cells: Vec<String>| format!("| {} |", cells.join(" | "));
    let mut out = Vec::new();
    for (i, row) in grid.iter().enumerate() {
        out.push(line(row.iter().map(text_of).collect()));
        if i == 0 {
            out.push(line(vec!["---".to_string(); cols]));
        }
    }
    out
}

fn cell_text(cell: &Cell) -> String {
    let mut parts = Vec::new();
    fn walk(blocks: &[Block], parts: &mut Vec<String>) {
        for b in blocks {
            match b {
                Block::Paragraph { content, .. } | Block::Heading { content, .. } => parts.push(inline(content, true)),
                Block::List { items, kind } => {
                    for item in items {
                        let mark = match (kind, item.checked) {
                            (ListKind::Task, Some(true)) => "☑ ",
                            (ListKind::Task, _) => "☐ ",
                            _ => "• ",
                        };
                        let mut inner = Vec::new();
                        walk(&item.blocks, &mut inner);
                        parts.push(format!("{mark}{}", inner.join(" ")));
                    }
                }
                Block::Code { text, .. } => parts.push(format!("`{}`", text.replace('\n', " ").replace('|', "\\|"))),
                Block::Math(m) => parts.push(format!("${m}$")),
                Block::Image(i) => parts.push(if i.caption.is_empty() { "(picture)".into() } else { format!("(picture: {})", escape_text(&i.caption)) }),
                Block::PageLink { title, target } => parts.push(page_link_row(title, target.as_ref())),
                Block::Database { id, table, .. } => {
                    walk(&super::doc::database_blocks(table.as_ref()), parts);
                    if let Some(t) = table {
                        // A nested table is flattened to text inside a Markdown cell.
                        parts.extend(t.rows.iter().flatten().map(cell_text));
                        parts.push(database_reference(id, &t.title));
                    }
                }
                Block::Quote(x) | Block::Toggle(x) | Block::Shelf(x) | Block::Callout { blocks: x, .. } => walk(x, parts),
                Block::Columns(c) => c.iter().for_each(|x| walk(x, parts)),
                Block::Table(_) | Block::Rule => {}
            }
        }
    }
    walk(&cell.blocks, &mut parts);
    parts.retain(|p| !p.trim().is_empty());
    parts.join("<br>")
}

#[derive(Clone, PartialEq)]
enum Mark {
    Link(String),
    Bold,
    Italic,
    Strike,
    Underline,
    Highlight(String),
    Color(String),
}

impl Mark {
    fn open(&self) -> String {
        match self {
            Mark::Link(_) => "[".into(),
            Mark::Bold => "**".into(),
            Mark::Italic => "*".into(),
            Mark::Strike => "~~".into(),
            Mark::Underline => "<u>".into(),
            Mark::Highlight(c) => format!("<mark style=\"background:#{c}\">"),
            Mark::Color(c) => format!("<span style=\"color:#{c}\">"),
        }
    }
    fn close(&self) -> String {
        match self {
            Mark::Link(url) => format!("]({})", link_target(url)),
            Mark::Bold => "**".into(),
            Mark::Italic => "*".into(),
            Mark::Strike => "~~".into(),
            Mark::Underline => "</u>".into(),
            Mark::Highlight(_) => "</mark>".into(),
            Mark::Color(_) => "</span>".into(),
        }
    }
}

/// The marks of a run, outermost first.
fn mark_list(m: &Marks) -> Vec<Mark> {
    let mut v = Vec::new();
    // Only web and mail links stay live; anything else (javascript:, file:, data:) is written as plain text.
    if let Some(url) = m.link.as_ref().filter(|u| {
        let u = u.trim().to_ascii_lowercase();
        u.starts_with("http://") || u.starts_with("https://") || u.starts_with("mailto:")
    }) {
        v.push(Mark::Link(url.clone()));
    } else if let Some(PageRef::File(path)) = &m.page {
        // Another page of the same ZIP: its file, by the path the exporter made.
        v.push(Mark::Link(path.clone()));
    }
    if let Some(c) = m.color {
        v.push(Mark::Color(c.hex()));
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

/// The "Link to page" row: `→ [Title](Other%20page.md)` when the page is another file of the
/// same ZIP, else `→ Title`. (A single Markdown file keeps the title only: heading anchors
/// differ from viewer to viewer.)
fn page_link_row(title: &str, target: Option<&PageRef>) -> String {
    match target {
        Some(PageRef::File(path)) => format!("→ [{}]({})", escape_text(title), link_target(path)),
        _ => format!("→ {}", escape_text(title)),
    }
}

fn link_target(url: &str) -> String {
    let u: String = url.chars().filter(|c| !c.is_control()).collect();
    if u.contains([' ', '(', ')', '<', '>']) {
        format!("<{}>", u.replace('<', "%3C").replace('>', "%3E"))
    } else {
        u
    }
}

/// Inline content as Markdown. Marks that continue over several runs stay open, so bold text
/// with an italic word inside comes out as `**a *b* c**`. `one_line` turns line breaks into
/// `<br>` (for headings and table cells).
pub fn inline(content: &[Inline], one_line: bool) -> String {
    let mut out = String::new();
    let mut stack: Vec<Mark> = Vec::new();
    // Delimiters must touch the text, so whitespace at the edge moves outside them.
    fn close_to(out: &mut String, stack: &mut Vec<Mark>, keep: usize) {
        if stack.len() <= keep {
            return;
        }
        let trimmed = out.trim_end_matches(' ').len();
        let spaces = out.len() - trimmed;
        out.truncate(trimmed);
        while stack.len() > keep {
            let m = stack.pop().unwrap();
            out.push_str(&m.close());
        }
        out.push_str(&" ".repeat(spaces));
    }
    for item in content {
        match item {
            Inline::Text(text, marks) => {
                let want = mark_list(marks);
                let keep = stack.iter().zip(&want).take_while(|(a, b)| a == b).count();
                close_to(&mut out, &mut stack, keep);
                let body = if marks.code { code_span(text) } else { escape_text(text) };
                let lead = body.len() - body.trim_start_matches(' ').len();
                out.push_str(&body[..lead]);
                for m in &want[keep..] {
                    out.push_str(&m.open());
                    stack.push(m.clone());
                }
                out.push_str(&body[lead..]);
            }
            Inline::Break => {
                close_to(&mut out, &mut stack, 0);
                out.push_str(if one_line { "<br>" } else { "\\\n" });
            }
            Inline::Math(latex) => {
                close_to(&mut out, &mut stack, 0);
                if !latex.trim().is_empty() {
                    out.push('$');
                    out.push_str(latex.trim());
                    out.push('$');
                }
            }
        }
    }
    close_to(&mut out, &mut stack, 0);
    if one_line {
        out = out.replace('\n', " ");
    }
    out
}

fn code_span(text: &str) -> String {
    let ticks = "`".repeat(longest_run(text, '`') + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') { " " } else { "" };
    format!("{ticks}{pad}{}{pad}{ticks}", text.replace('\n', " "))
}

fn longest_run(s: &str, ch: char) -> usize {
    let (mut best, mut cur) = (0, 0);
    for c in s.chars() {
        if c == ch {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 0;
        }
    }
    best
}

/// Escapes characters that Markdown would read as formatting.
pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '~' | '$' => {
                out.push('\\');
                out.push(c);
            }
            '&' if chars.get(i + 1).is_some_and(|n| n.is_ascii_alphanumeric() || *n == '#') => out.push_str("\\&"),
            '\n' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// A paragraph line that starts like a heading, list item or rule is escaped.
fn escape_line_start(line: &str) -> String {
    let t = line.trim_start();
    let starts_block = t.starts_with('#')
        || t.starts_with("- ")
        || t.starts_with("+ ")
        || t == "-"
        || t.starts_with('=')
        || t.starts_with("---")
        || {
            let digits = t.chars().take_while(char::is_ascii_digit).count();
            digits > 0 && matches!(t[digits..].chars().next(), Some('.') | Some(')'))
        };
    if !starts_block {
        return line.to_string();
    }
    let lead = line.len() - t.len();
    let digits = t.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 {
        format!("{}{}\\{}", &line[..lead], &t[..digits], &t[digits..])
    } else {
        format!("{}\\{}", &line[..lead], t)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::doc::tests::{every_node_json, titles, IMG_ID};
    use super::super::doc::{parse, Rgb};
    use super::super::tests::sample_doc;
    use super::*;

    #[test]
    fn inline_database_exports_rows_in_logical_order_with_a_reference() {
        let md = render(&super::super::tests::inline_database_doc(), "assets").text;
        assert!(md.contains("[database: مهام](notaya://db/database1)"));
        assert_eq!(md.lines().filter(|l| l.starts_with('|')).count(), 4, "header, delimiter, two live rows");
        assert!(md.contains("| أول | مرحبا 123 عالم | 12.5 | 25 |"));
        assert!(md.contains("| ثان | שלום | 25 | 50 |"));
        assert!(!md.contains("HIDDEN_CELL") && !md.contains("TRASHED_CELL"));
        assert!(md.find("Before").unwrap() < md.find("| الاسم").unwrap());
        assert!(md.find("notaya://db/database1").unwrap() < md.find("After").unwrap());
    }

    #[test]
    fn database_reference_escapes_the_title_and_encodes_only_one_id_segment() {
        assert_eq!(database_reference("id/)[x](javascript:evil)", "a] <b>\nnext"), "[database: a\\] \\<b\\> next](notaya://db/id%2F%29%5Bx%5D%28javascript%3Aevil%29)");
    }

    #[test]
    fn every_node_type_has_a_markdown_form() {
        let doc = sample_doc(parse(&every_node_json(), &titles));
        let out = render(&doc, "My notes.assets");
        let md = &out.text;
        for expected in [
            "# Sample page",
            "## Big heading",
            "##### Small heading",
            "**bold**",
            "*italic*",
            "<u>under</u>",
            "~~struck~~",
            "`x = 1`",
            "[a link](https://example.com/a?b=1&c=2)",
            "<span style=\"color:#E5484D\">red</span>",
            "<mark style=\"background:#F8E6AD\">marked</mark>",
            "\\<special\\> \\*chars\\* & café € “quotes”.\\",
            "After a break, with $E = mc^2$commented",
            "- Bullet one\n  - Nested bullet\n- Bullet two",
            "3. Third\n4. Fourth",
            "- [x] Done task\n- [ ] Open task",
            "> A quote.",
            "> 💡 A callout.",
            "<details open>\n<summary>Toggle line</summary>\n\nHidden text.\n\n</details>",
            "Left column\n\nRight column",
            "````rust\nfn main() {\n    println!(\"```\");\n}\n````",
            "$$\nx = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}\n$$",
            "![A caption](My%20notes.assets/image-001.png)\n\n*A caption*",
            "*(picture not available)*",
            "| Name | Value \\| pipe |\n| --- | --- |\n| alpha | **1** |\n| spans both |  |",
            "---",
            "→ Child page",
            "Unknown block text",
        ] {
            assert!(md.contains(expected), "missing {expected:?} in:\n{md}");
        }
        assert_eq!(out.images.len(), 1);
        assert_eq!(out.images[0].0, "image-001.png");
        assert!(doc.images.contains_key(IMG_ID));
    }

    #[test]
    fn overlapping_marks_nest_and_spaces_stay_outside() {
        let bold = Marks { bold: true, ..Marks::default() };
        let bold_italic = Marks { bold: true, italic: true, ..Marks::default() };
        let content = vec![
            Inline::Text("a ".into(), bold.clone()),
            Inline::Text("b".into(), bold_italic),
            Inline::Text(" c".into(), bold),
        ];
        assert_eq!(inline(&content, false), "**a *b* c**");
        let colored = Marks { color: Some(Rgb(1, 2, 3)), ..Marks::default() };
        assert_eq!(inline(&[Inline::Text("x ".into(), colored), Inline::Text("y".into(), Marks::default())], false), "<span style=\"color:#010203\">x</span> y");
    }

    #[test]
    fn a_shelf_is_a_marked_details_and_colors_are_inline_html() {
        let json = r##"{"type":"doc","content":[{"type":"toggle","attrs":{"open":true,"shelf":true},"content":[
            {"type":"paragraph","content":[{"type":"text","text":"Shelf <title>"}]},
            {"type":"paragraph","content":[{"type":"text","text":"Inside","marks":[{"type":"textStyle","attrs":{"color":"#d04f4f"}},{"type":"highlight","attrs":{"color":"rgba(206, 178, 80, 0.34)"}}]}]}]},
            {"type":"toggle","attrs":{"open":true,"shelf":false},"content":[{"type":"paragraph","content":[{"type":"text","text":"Plain"}]}]}]}"##;
        let doc = sample_doc(parse(json, &titles));
        let md = render(&doc, "a").text;
        assert!(md.contains("<details open class=\"notaya-shelf\">\n<summary>Shelf &lt;title&gt;</summary>"), "{md}");
        assert!(md.contains("<span style=\"color:#D04F4F\"><mark style=\"background:#EEE5C4\">Inside</mark></span>"), "{md}");
        assert!(md.contains("<details open>\n<summary>Plain</summary>"), "{md}");
        // A color the model could not read is never written.
        let bad = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"textStyle","attrs":{"color":"red\" onmouseover=\"x"}}]}]}]}"#;
        let md = render(&sample_doc(parse(bad, &titles)), "a").text;
        assert!(!md.contains("span") && !md.contains("onmouseover"), "{md}");
    }

    #[test]
    fn line_starts_and_code_are_escaped() {
        assert_eq!(escape_line_start("# not a heading"), "\\# not a heading");
        assert_eq!(escape_line_start("12. not a list"), "12\\. not a list");
        assert_eq!(escape_line_start("- dash"), "\\- dash");
        assert_eq!(escape_line_start("fine"), "fine");
        assert_eq!(code_span("a`b"), "``a`b``");
        assert_eq!(escape_text("Tom & Jerry &amp;"), "Tom & Jerry \\&amp;");
    }
}
