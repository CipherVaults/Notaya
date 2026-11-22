//! Markdown (CommonMark and GitHub's extensions, the common subset) to the import model.
//!
//! Blocks: ATX and Setext headings, paragraphs, bullet, numbered and task lists (nested by
//! indentation), quotes, fenced and indented code, rules, pipe tables, HTML blocks (passed to
//! the HTML converter, which sanitises them; Notion writes callouts as `<aside>`), front
//! matter (its `title:` is used, the rest dropped) and link reference definitions.
//! Inline: bold, italic, strike, `==highlight==`, code, links, autolinks, bare URLs, pictures,
//! hard breaks, entities, `$math$`, and a few inline HTML tags (`b`, `i`, `u`, `s`, `code`,
//! `mark`, `br`, `img`, and `span`/`mark`/`font` with a text or background color, which becomes
//! the nearest Notaya palette color; see `palette`).
//!
//! Hostile input: nesting is capped, the search for a closing delimiter is bounded and a
//! failed search is remembered (per piece of text it was made in), so the work stays roughly
//! linear. Table cells are capped per table and per page, and the import's deadline and
//! cancel button are checked in the line and character loops.

use std::collections::HashMap;

use super::doc::{self, Block, Cell, Inline, Marks};
use super::markup::{attr, decode_one, Token, Tokenizer};
use super::{html, palette, resolve_rel, safe_link, Ctx};

const MAX_DEPTH: usize = 16;
/// Paragraphs longer than this are taken as plain text (no inline parsing).
const MAX_INLINE: usize = 200_000;
/// How far to look for a closing delimiter or bracket.
const SCAN: usize = 4_000;
/// Brackets or parentheses nested deeper than this inside a link are not a link.
const MAX_BRACKETS: i32 = 32;
/// Lines read from one file (a file of nothing but line breaks must not need gigabytes).
const MAX_LINES: usize = 3_000_000;

pub struct MdDoc {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

pub fn parse(src: &str, ctx: &mut Ctx) -> MdDoc {
    let src = src.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = src.lines().take(MAX_LINES + 1).map(expand_tabs).collect();
    if lines.len() > MAX_LINES {
        lines.truncate(MAX_LINES);
        ctx.problem("Long file", "The file has more than 3 million lines; only the first 3 million were read.");
    }
    let mut title = None;

    // Front matter.
    if lines.first().is_some_and(|l| l.trim_end() == "---") {
        if let Some(end) = lines.iter().take(300).skip(1).position(|l| l.trim_end() == "---" || l.trim_end() == "...") {
            for l in &lines[1..end + 1] {
                if let Some(v) = l.strip_prefix("title:") {
                    let v = v.trim().trim_matches(['"', '\'']).trim();
                    if !v.is_empty() {
                        title = Some(v.to_string());
                    }
                }
            }
            lines.drain(..end + 2);
        }
    }

    // Link reference definitions (outside code fences).
    let mut refs = HashMap::new();
    let mut in_fence = false;
    lines.retain(|l| {
        if fence(l).is_some() {
            in_fence = !in_fence;
        }
        if in_fence || indent(l) > 3 {
            return true;
        }
        let t = l.trim_start();
        if let Some(rest) = t.strip_prefix('[') {
            if let Some((label, after)) = rest.split_once("]:") {
                if !label.is_empty() && !label.contains(']') && refs.len() < 10_000 {
                    let url = after.trim().split_whitespace().next().unwrap_or("").trim_matches(['<', '>']);
                    if !url.is_empty() {
                        refs.insert(label.to_lowercase(), url.to_string());
                        return false;
                    }
                }
            }
        }
        true
    });

    let mut p = Parser { ctx, refs, cells: 0 };
    let blocks = p.blocks(&lines, 0);
    MdDoc { title, blocks }
}

fn expand_tabs(l: &str) -> String {
    if !l.contains('\t') {
        return l.to_string();
    }
    let mut out = String::new();
    let mut col = 0;
    for c in l.chars() {
        if c == '\t' {
            let n = 4 - col % 4;
            out.push_str(&" ".repeat(n));
            col += n;
        } else {
            out.push(c);
            col += 1;
        }
    }
    out
}

fn indent(l: &str) -> usize {
    l.len() - l.trim_start_matches(' ').len()
}

fn blank(l: &str) -> bool {
    l.trim().is_empty()
}

/// An opening or closing code fence: (char, length, indent, info string).
fn fence(l: &str) -> Option<(char, usize, usize, String)> {
    let ind = indent(l);
    if ind > 3 {
        return None;
    }
    let t = &l[ind..];
    let c = t.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let n = t.chars().take_while(|x| *x == c).count();
    if n < 3 {
        return None;
    }
    let info = t[n..].trim().to_string();
    if c == '`' && info.contains('`') {
        return None;
    }
    Some((c, n, ind, info))
}

fn atx(l: &str) -> Option<(u8, String)> {
    let ind = indent(l);
    if ind > 3 {
        return None;
    }
    let t = &l[ind..];
    let n = t.chars().take_while(|c| *c == '#').count();
    if n == 0 || n > 6 {
        return None;
    }
    let rest = &t[n..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let mut text = rest.trim();
    // Closing #s.
    let stripped = text.trim_end_matches('#');
    if stripped.is_empty() || stripped.ends_with(' ') {
        text = stripped.trim_end();
    }
    Some((n as u8, text.to_string()))
}

fn rule(l: &str) -> bool {
    if indent(l) > 3 {
        return false;
    }
    let t: String = l.chars().filter(|c| !c.is_whitespace()).collect();
    t.len() >= 3 && (t.chars().all(|c| c == '*') || t.chars().all(|c| c == '-') || t.chars().all(|c| c == '_'))
}

fn setext(l: &str) -> Option<u8> {
    if indent(l) > 3 {
        return None;
    }
    let t = l.trim();
    if !t.is_empty() && t.chars().all(|c| c == '=') {
        Some(1)
    } else if !t.is_empty() && t.chars().all(|c| c == '-') {
        Some(2)
    } else {
        None
    }
}

struct Marker {
    ordered: bool,
    /// The bullet character, or the delimiter after the number.
    kind: char,
    start: u32,
    indent: usize,
    /// Column where the item's content starts.
    content: usize,
    empty: bool,
}

fn marker(l: &str) -> Option<Marker> {
    let ind = indent(l);
    let t = &l[ind..];
    let (ordered, kind, start, len) = if let Some(c) = t.chars().next().filter(|c| matches!(c, '-' | '*' | '+')) {
        (false, c, 0, 1)
    } else {
        let digits = t.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 || digits > 9 {
            return None;
        }
        let delim = t[digits..].chars().next()?;
        if delim != '.' && delim != ')' {
            return None;
        }
        (true, delim, t[..digits].parse().ok()?, digits + 1)
    };
    let after = &t[len..];
    if after.is_empty() {
        return Some(Marker { ordered, kind, start, indent: ind, content: ind + len + 1, empty: true });
    }
    if !after.starts_with(' ') {
        return None;
    }
    let spaces = after.len() - after.trim_start_matches(' ').len();
    let pad = if spaces > 4 || after.trim().is_empty() { 1 } else { spaces };
    Some(Marker { ordered, kind, start, indent: ind, content: ind + len + pad, empty: after.trim().is_empty() })
}

fn quote_line(l: &str) -> Option<String> {
    let ind = indent(l);
    if ind > 3 {
        return None;
    }
    let rest = l[ind..].strip_prefix('>')?;
    Some(rest.strip_prefix(' ').unwrap_or(rest).to_string())
}

fn delimiter_row(l: &str) -> bool {
    let t = l.trim();
    if !t.contains('-') {
        return false;
    }
    let t = t.trim_matches('|');
    t.split('|').all(|c| {
        let c = c.trim();
        !c.is_empty() && c.trim_matches(':').chars().all(|x| x == '-') && c.contains('-')
    })
}

const HTML_BLOCK_TAGS: &[&str] = &[
    "address", "article", "aside", "blockquote", "details", "div", "dl", "figure", "figcaption", "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header",
    "hr", "main", "nav", "ol", "p", "pre", "section", "summary", "table", "ul", "img", "picture", "center", "iframe", "script", "style", "video", "audio",
];

fn html_block(l: &str) -> bool {
    if indent(l) > 3 {
        return false;
    }
    let t = l.trim_start();
    if t.starts_with("<!--") {
        return true;
    }
    let Some(rest) = t.strip_prefix('<') else { return false };
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
    let next = rest[name.len()..].chars().next();
    HTML_BLOCK_TAGS.contains(&name.as_str()) && matches!(next, None | Some(' ' | '>' | '/' | '\t'))
}

/// Starts a block that may interrupt a paragraph.
fn interrupts(l: &str) -> bool {
    atx(l).is_some() || fence(l).is_some() || rule(l) || quote_line(l).is_some() || html_block(l) || marker(l).is_some_and(|m| !m.empty && (!m.ordered || m.start == 1))
}

struct Parser<'c, 'a> {
    ctx: &'c mut Ctx<'a>,
    refs: HashMap<String, String>,
    /// Table cells made so far on this page.
    cells: usize,
}

impl Parser<'_, '_> {
    /// A paragraph that is only `[Title](Other%20page%20<id>.md)`, a link to another Markdown or
    /// HTML file of the same import (how Notion writes a link to a page), becomes a page link.
    fn page_link(&mut self, text: &str) -> Option<Block> {
        let t = text.trim();
        if t.contains('\n') || t.len() > 2_000 {
            return None;
        }
        let (label, dest) = t.strip_prefix('[')?.strip_suffix(')')?.rsplit_once("](")?;
        if label.contains("](") || dest.contains(char::is_whitespace) || dest.contains(['(', ')', '<', '>']) {
            return None;
        }
        let lower = dest.to_ascii_lowercase();
        let lower = lower.split(['?', '#']).next().unwrap_or("");
        if !(lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".html") || lower.ends_with(".htm")) {
            return None;
        }
        let path = resolve_rel(&self.ctx.dir, dest).and_then(|p| self.ctx.vfs.find(&p))?;
        Some(Block::PageLink(path, doc::collapse_ws(label)))
    }

    fn blocks(&mut self, lines: &[String], depth: usize) -> Vec<Block> {
        let mut out = Vec::new();
        let mut para: Vec<&str> = Vec::new();
        let n = lines.len();
        let mut i = 0;
        macro_rules! flush {
            () => {
                if !para.is_empty() {
                    let text = para.join("\n");
                    para.clear();
                    match self.page_link(&text) {
                        Some(b) => out.push(b),
                        None => {
                            let inl = self.inline(&text);
                            // A line of italic text right after a picture is its caption (how
                            // Notaya's Markdown export writes one, and a common convention).
                            let t = text.trim();
                            let emphasis = t.len() > 2 && ((t.starts_with('*') && t.ends_with('*')) || (t.starts_with('_') && t.ends_with('_')));
                            match (picture_only(out.last()), italic_only(&inl).filter(|_| emphasis)) {
                                (Some(idx), Some(caption)) => *out.last_mut().unwrap() = Block::Image(idx, caption),
                                _ => out.push(Block::Paragraph(inl)),
                            }
                        }
                    }
                }
            };
        }
        while i < n {
            if self.ctx.ctl.stopped() {
                break;
            }
            let l = lines[i].as_str();
            if blank(l) {
                flush!();
                i += 1;
                continue;
            }
            if !para.is_empty() {
                if let Some(level) = setext(l) {
                    let text = para.join("\n");
                    para.clear();
                    out.push(Block::Heading(level, self.inline(&text)));
                    i += 1;
                    continue;
                }
            }
            if para.is_empty() && indent(l) >= 4 {
                let mut code = Vec::new();
                while i < n && (blank(&lines[i]) || indent(&lines[i]) >= 4) {
                    code.push(lines[i].get(4..).unwrap_or("").to_string());
                    i += 1;
                }
                while code.last().is_some_and(|c| c.trim().is_empty()) {
                    code.pop();
                }
                out.push(Block::Code(String::new(), code.join("\n")));
                continue;
            }
            if let Some((c, len, ind, info)) = fence(l) {
                flush!();
                i += 1;
                let mut code = Vec::new();
                while i < n {
                    if let Some((c2, len2, _, info2)) = fence(&lines[i]) {
                        if c2 == c && len2 >= len && info2.is_empty() {
                            i += 1;
                            break;
                        }
                    }
                    let line = &lines[i];
                    let strip = indent(line).min(ind);
                    code.push(line[strip..].to_string());
                    i += 1;
                }
                let lang = info.split_whitespace().next().unwrap_or("").to_string();
                if lang == "math" || lang == "latex" && false {
                    out.push(Block::Math(code.join("\n")));
                } else {
                    out.push(Block::Code(lang, code.join("\n")));
                }
                continue;
            }
            if let Some((level, text)) = atx(l) {
                flush!();
                out.push(Block::Heading(level, self.inline(&text)));
                i += 1;
                continue;
            }
            if rule(l) {
                flush!();
                out.push(Block::Rule);
                i += 1;
                continue;
            }
            if l.trim() == "$$" && para.is_empty() {
                // A display equation on its own lines.
                if let Some(end) = lines[i + 1..].iter().take(500).position(|x| x.trim() == "$$") {
                    let tex = lines[i + 1..i + 1 + end].join("\n");
                    out.push(Block::Math(tex));
                    i += end + 2;
                    continue;
                }
            }
            if quote_line(l).is_some() {
                flush!();
                let mut inner = Vec::new();
                while i < n {
                    if let Some(q) = quote_line(&lines[i]) {
                        inner.push(q);
                    } else if !blank(&lines[i]) && !interrupts(&lines[i]) && inner.last().is_some_and(|x: &String| !x.trim().is_empty()) {
                        inner.push(lines[i].clone()); // lazy continuation
                    } else {
                        break;
                    }
                    i += 1;
                }
                let blocks = if depth < MAX_DEPTH { self.blocks(&inner, depth + 1) } else { vec![Block::Paragraph(vec![doc::t(inner.join(" "))])] };
                // GitHub's alerts (> [!NOTE]) become callouts.
                out.push(callout_of(blocks));
                continue;
            }
            if let Some(m) = marker(l) {
                if para.is_empty() || (!m.empty && (!m.ordered || m.start == 1)) {
                    flush!();
                    let blocks = self.list(lines, &mut i, depth);
                    out.extend(blocks);
                    continue;
                }
            }
            if para.is_empty() && l.contains('|') && i + 1 < n && delimiter_row(&lines[i + 1]) {
                out.push(self.table(lines, &mut i));
                continue;
            }
            if para.is_empty() && html_block(l) {
                // A closed comment on its own line must leave the next Markdown block intact.
                let standalone_comment = l.trim_start()
                    .strip_prefix("<!--")
                    .and_then(|comment| comment.split_once("-->"))
                    .is_some_and(|(_, rest)| rest.trim().is_empty());
                if standalone_comment {
                    i += 1;
                    continue;
                }
                let mut html_lines = Vec::new();
                // A toggle (<details>) goes on past blank lines until it is closed, so its
                // content stays inside it (Notion writes toggles this way).
                // A callout (<aside>) does the same when it is closed further down, so its text after a blank line stays in it.
                let toggle = l.trim_start().to_ascii_lowercase().starts_with("<details");
                let aside = !toggle && l.trim_start().to_ascii_lowercase().starts_with("<aside") && lines[i..].iter().take(20_000).any(|x| x.to_ascii_lowercase().contains("</aside>"));
                let mut open = 0i32;
                while i < n && html_lines.len() < 20_000 {
                    if blank(&lines[i]) && !((toggle || aside) && open > 0) {
                        break;
                    }
                    if toggle || aside {
                        let low = lines[i].to_ascii_lowercase();
                        let (o, c) = if toggle { ("<details", "</details>") } else { ("<aside", "</aside>") };
                        open += low.matches(o).count() as i32 - low.matches(c).count() as i32;
                    }
                    html_lines.push(lines[i].as_str());
                    i += 1;
                }
                // Notaya's own export (and many tools) write a toggle or shelf as `<details>`, a
                // `<summary>` line, then the body as Markdown, then `</details>`: the body is read as
                // Markdown (page-link rows, lists, code...), not as HTML text.
                if toggle {
                    if let Some(block) = self.details_with_markdown_body(&html_lines, depth) {
                        out.push(block);
                        continue;
                    }
                }
                if aside {
                    if let Some(block) = self.aside_with_markdown_body(&html_lines, depth) {
                        out.push(block);
                        continue;
                    }
                }
                let blocks = html::parse_fragment(&html_lines.join("\n"), self.ctx);
                out.extend(blocks);
                continue;
            }
            if !para.is_empty() && interrupts(l) {
                flush!();
                continue;
            }
            para.push(l.trim_start());
            i += 1;
        }
        flush!();
        out
    }

    /// Notion's Markdown callout: `<aside>`, an emoji line, the text as Markdown (blank lines included), `</aside>`. The emoji is the icon
    /// (the lightbulb when there is none) and the rest is read as Markdown. `None` for anything else (it is read as HTML as before).
    fn aside_with_markdown_body(&mut self, html_lines: &[&str], depth: usize) -> Option<Block> {
        let n = html_lines.len();
        if n < 3 || depth >= MAX_DEPTH {
            return None;
        }
        let low: Vec<String> = html_lines.iter().map(|l| l.trim().to_ascii_lowercase()).collect();
        // The tags are alone on the first and last line, and no second callout sits at the top level.
        if !low[0].starts_with("<aside") || !low[0].ends_with('>') || low[0].matches('<').count() != 1 || low[n - 1] != "</aside>" {
            return None;
        }
        let mut open = 0i32;
        for (k, l) in low.iter().enumerate() {
            open += l.matches("<aside").count() as i32 - l.matches("</aside>").count() as i32;
            if (k + 1 < n && open <= 0) || (k + 1 == n && open != 0) {
                return None;
            }
        }
        let mut body: Vec<String> = html_lines[1..n - 1].iter().map(|l| l.to_string()).collect();
        let first = body.iter().position(|l| !blank(l))?;
        let lead = body[first].trim();
        let is_icon = !lead.is_ascii() && lead.chars().count() <= 8 && !lead.chars().any(|c| c.is_alphanumeric());
        let icon = if is_icon { lead.to_string() } else { "💡".to_string() };
        if is_icon {
            body.remove(first);
        }
        Some(Block::Callout(icon, self.blocks(&body, depth + 1)))
    }

    /// `<details ...>`, a summary within its first lines, a Markdown body and a closing `</details>`
    /// as the last line of the one block (no sibling `<details>` in between): the toggle or shelf
    /// with its body parsed as Markdown. `None` for anything else (it is read as HTML as before).
    fn details_with_markdown_body(&mut self, html_lines: &[&str], depth: usize) -> Option<Block> {
        let n = html_lines.len();
        if n < 3 || depth >= MAX_DEPTH {
            return None;
        }
        let low: Vec<String> = html_lines.iter().map(|l| l.trim().to_ascii_lowercase()).collect();
        if !low[0].starts_with("<details") || low[n - 1] != "</details>" {
            return None;
        }
        // The summary ends on one of the first three lines.
        let s_end = low.iter().take(3).position(|l| l.contains("</summary>"))?;
        if low[..=s_end].iter().any(|l| l.matches("<details").count() > 1 || l.contains("</details>")) {
            return None;
        }
        // Only one `<details>` at the top: the depth never returns to zero before the last line.
        let mut open = 0i32;
        for (k, l) in low.iter().enumerate() {
            open += l.matches("<details").count() as i32 - l.matches("</details>").count() as i32;
            if (k + 1 < n && open <= 0) || (k + 1 == n && open != 0) {
                return None;
            }
        }
        let head = format!("{}\n</details>", html_lines[..=s_end].join("\n"));
        let mut parsed = html::parse_fragment(&head, self.ctx);
        if parsed.len() != 1 {
            return None;
        }
        let body: Vec<String> = html_lines[s_end + 1..n - 1].iter().map(|l| l.to_string()).collect();
        let more = self.blocks(&body, depth + 1);
        match parsed.pop()? {
            Block::Toggle(mut inner) => {
                inner.extend(more);
                Some(Block::Toggle(inner))
            }
            Block::Shelf(mut inner) => {
                inner.extend(more);
                Some(Block::Shelf(inner))
            }
            _ => None,
        }
    }

    fn list(&mut self, lines: &[String], i: &mut usize, depth: usize) -> Vec<Block> {
        let n = lines.len();
        let first = marker(&lines[*i]).expect("a list starts with a marker");
        let mut items: Vec<(Option<bool>, Vec<String>)> = Vec::new();
        while *i < n {
            let Some(m) = marker(&lines[*i]) else { break };
            if m.ordered != first.ordered || m.kind != first.kind || m.indent >= first.content {
                break;
            }
            let content_col = m.content;
            let first_line = lines[*i].get(content_col.min(lines[*i].len())..).unwrap_or("").to_string();
            let mut item = vec![first_line];
            *i += 1;
            while *i < n {
                let l = &lines[*i];
                if blank(l) {
                    // The whole run of blank lines is looked at once (looking from every blank
                    // line to the end of the run was quadratic).
                    let next = lines[*i..].iter().position(|x| !blank(x)).map(|p| *i + p);
                    match next {
                        Some(j) if indent(&lines[j]) >= content_col => {
                            item.extend(std::iter::repeat_n(String::new(), j - *i));
                            *i = j;
                            continue;
                        }
                        _ => break,
                    }
                }
                if indent(l) >= content_col {
                    item.push(l[content_col..].to_string());
                    *i += 1;
                    continue;
                }
                if marker(l).is_some() || atx(l).is_some() || fence(l).is_some() || rule(l) || quote_line(l).is_some() {
                    break;
                }
                if item.last().is_some_and(|x| !x.trim().is_empty()) {
                    item.push(l.trim_start().to_string());
                    *i += 1;
                    continue;
                }
                break;
            }
            // Task item?
            let mut task = None;
            let head = item[0].clone();
            for (pre, checked) in [("[ ]", false), ("[x]", true), ("[X]", true)] {
                if let Some(rest) = head.strip_prefix(pre) {
                    if rest.is_empty() || rest.starts_with(' ') {
                        task = Some(checked);
                        item[0] = rest.trim_start().to_string();
                    }
                }
            }
            items.push((task, item));
            // Blank lines between items of the same list.
            let mut j = *i;
            while j < n && blank(&lines[j]) {
                j += 1;
            }
            if j < n && j > *i && marker(&lines[j]).is_some_and(|m| m.ordered == first.ordered && m.kind == first.kind && m.indent < first.content) {
                *i = j;
            }
        }
        // Consecutive task items form a task list, the others a normal list.
        let mut out = Vec::new();
        let mut run: Vec<(Option<bool>, Vec<Block>)> = Vec::new();
        let mut start = first.start;
        let mut emitted = 0;
        let push_run = |run: &mut Vec<(Option<bool>, Vec<Block>)>, out: &mut Vec<Block>, start: u32| {
            if run.is_empty() {
                return;
            }
            let items = std::mem::take(run);
            if items[0].0.is_some() {
                out.push(Block::Tasks(items.into_iter().map(|(t, b)| (t.unwrap_or(false), b)).collect()));
            } else if first.ordered {
                out.push(Block::Numbered(start, items.into_iter().map(|(_, b)| b).collect()));
            } else {
                out.push(Block::Bullets(items.into_iter().map(|(_, b)| b).collect()));
            }
        };
        for (task, lines) in items {
            let blocks = if depth < MAX_DEPTH { self.blocks(&lines, depth + 1) } else { vec![Block::Paragraph(vec![doc::t(lines.join(" "))])] };
            if run.first().is_some_and(|(t, _)| t.is_some() != task.is_some()) {
                let len = run.len();
                push_run(&mut run, &mut out, start + emitted);
                emitted += len as u32;
            }
            run.push((task, blocks));
            let _ = &mut start;
        }
        push_run(&mut run, &mut out, start + emitted);
        out
    }

    fn table(&mut self, lines: &[String], i: &mut usize) -> Block {
        let head = split_row(&lines[*i]);
        *i += 2;
        // At most MAX_TABLE_CELLS cells in a table and MAX_PAGE_CELLS on a page; the rows after
        // that are read past (not parsed) and the import says so.
        let max_rows = (doc::MAX_TABLE_CELLS / head.len().max(1)).clamp(1, 5_000);
        let mut rows = Vec::new();
        let mut cut = false;
        if self.cells + head.len() <= doc::MAX_PAGE_CELLS {
            self.cells += head.len();
            rows.push(head.iter().map(|c| Cell { header: true, colspan: 1, blocks: vec![Block::Paragraph(self.inline(c))] }).collect::<Vec<_>>());
        } else {
            cut = true;
        }
        while *i < lines.len() && !blank(&lines[*i]) && lines[*i].contains('|') {
            if self.ctx.ctl.stopped() {
                break;
            }
            if rows.len() < max_rows && self.cells + head.len() <= doc::MAX_PAGE_CELLS {
                let mut cells = split_row(&lines[*i]);
                cells.resize(head.len(), String::new());
                self.cells += cells.len();
                rows.push(cells.iter().map(|c| Cell { header: false, colspan: 1, blocks: vec![Block::Paragraph(self.inline(c))] }).collect());
            } else {
                cut = true;
            }
            *i += 1;
        }
        if cut {
            self.ctx.problem("Table", "A table has more cells than are imported (20,000 per table, 100,000 per page); the rest of its rows were left out.");
        }
        Block::Table(rows)
    }

    // ---------------- inline ----------------

    fn inline(&mut self, text: &str) -> Vec<Inline> {
        let chars: Vec<char> = text.chars().collect();
        let mut out = Vec::new();
        if chars.len() > MAX_INLINE {
            out.push(doc::t(text.replace('\n', " ")));
            return out;
        }
        self.inline_into(&chars, &Marks::default(), &mut out, 0);
        out
    }

    /// Inline content of `s`. The remembered failed searches (`st`) belong to this piece of text
    /// alone: positions and the end of the text mean something different in a label or an
    /// emphasis that is parsed on its own, so each piece starts with an empty memory.
    fn inline_into(&mut self, s: &[char], m: &Marks, out: &mut Vec<Inline>, depth: usize) {
        let mut st = InlineState::default();
        let st = &mut st;
        let mut buf = String::new();
        let flush = |buf: &mut String, out: &mut Vec<Inline>| {
            if !buf.is_empty() {
                out.push(Inline::Text(std::mem::take(buf), m.clone()));
            }
        };
        let n = s.len();
        let mut i = 0;
        while i < n {
            if i % 512 == 0 && self.ctx.ctl.stopped() {
                break;
            }
            let c = s[i];
            match c {
                '\\' if i + 1 < n && s[i + 1] == '\n' => {
                    flush(&mut buf, out);
                    out.push(Inline::Break);
                    i += 2;
                }
                '\\' if i + 1 < n && s[i + 1].is_ascii_punctuation() => {
                    buf.push(s[i + 1]);
                    i += 2;
                }
                '\n' => {
                    if buf.ends_with("  ") {
                        let t = buf.trim_end().to_string();
                        buf = t;
                        flush(&mut buf, out);
                        out.push(Inline::Break);
                    } else {
                        let t = buf.trim_end().to_string();
                        buf = t;
                        buf.push(' ');
                    }
                    i += 1;
                }
                '`' => {
                    let run = s[i..].iter().take_while(|x| **x == '`').count();
                    match find_backticks(s, i + run, run, st) {
                        Some(end) => {
                            let mut code: String = s[i + run..end].iter().collect::<String>().replace('\n', " ");
                            if code.len() > 2 && code.starts_with(' ') && code.ends_with(' ') {
                                code = code[1..code.len() - 1].to_string();
                            }
                            flush(&mut buf, out);
                            out.push(Inline::Text(code, Marks { code: true, ..m.clone() }));
                            i = end + run;
                        }
                        None => {
                            buf.extend(std::iter::repeat_n('`', run));
                            i += run;
                        }
                    }
                }
                '!' if i + 1 < n && s[i + 1] == '[' => match self.link_at(s, i + 1, st) {
                    Some((label, dest, end)) => {
                        flush(&mut buf, out);
                        let alt: String = label.iter().collect();
                        out.push(self.ctx.image(&dest, &alt));
                        i = end;
                    }
                    None => {
                        buf.push('!');
                        i += 1;
                    }
                },
                '[' => match self.link_at(s, i, st) {
                    Some((label, dest, end)) => {
                        flush(&mut buf, out);
                        let mut lm = m.clone();
                        lm.link = safe_link(&dest).or_else(|| self.ctx.page_href(&dest));
                        if depth < MAX_DEPTH {
                            self.inline_into(&label, &lm, out, depth + 1);
                        } else {
                            out.push(Inline::Text(label.iter().collect(), lm));
                        }
                        i = end;
                    }
                    None => {
                        buf.push('[');
                        i += 1;
                    }
                },
                '<' => {
                    if let Some((inl, end)) = self.angle(s, i, m, depth, st) {
                        flush(&mut buf, out);
                        out.extend(inl);
                        i = end;
                    } else {
                        buf.push('<');
                        i += 1;
                    }
                }
                '&' => {
                    let rest: String = s[i..n.min(i + 40)].iter().collect();
                    match decode_one(&rest) {
                        Some((t, used)) => {
                            buf.push_str(&t);
                            i += rest[..used].chars().count();
                        }
                        None => {
                            buf.push('&');
                            i += 1;
                        }
                    }
                }
                '$' if i + 1 < n && s[i + 1] != ' ' && s[i + 1] != '$' => {
                    // $math$, only when it closes on the same line and is not money ("$5 and $6").
                    let end = scan_memo(st, ('$', 0), i + 1, n, |j| s[j] == '$' && s[j - 1] != ' ' && s[j - 1] != '\\');
                    let lineless = end.is_some_and(|e| !s[i + 1..e].contains(&'\n'));
                    match end {
                        Some(e) if lineless && !(e + 1 < n && s[e + 1].is_ascii_digit()) && !s[i + 1].is_ascii_digit() => {
                            flush(&mut buf, out);
                            out.push(Inline::Math(s[i + 1..e].iter().collect()));
                            i = e + 1;
                        }
                        _ => {
                            buf.push('$');
                            i += 1;
                        }
                    }
                }
                '*' | '_' | '~' | '=' => {
                    let run = s[i..].iter().take_while(|x| **x == c).count();
                    let prev = if i == 0 { ' ' } else { s[i - 1] };
                    let next = s.get(i + run).copied().unwrap_or(' ');
                    let can_open = !next.is_whitespace() && !(c == '_' && prev.is_alphanumeric());
                    let want = match c {
                        '~' | '=' if run >= 2 => 2,
                        '~' | '=' => 0,
                        _ if run >= 3 => 3,
                        _ if run == 2 => 2,
                        _ => 1,
                    };
                    let found = if can_open && want > 0 && depth < MAX_DEPTH { find_closer(s, i + want, c, want, st) } else { None };
                    match found {
                        Some(end) => {
                            flush(&mut buf, out);
                            let mut im = m.clone();
                            match (c, want) {
                                ('~', _) => im.strike = true,
                                ('=', _) => im.highlight = Some(String::new()),
                                (_, 1) => im.italic = true,
                                (_, 2) => im.bold = true,
                                _ => {
                                    im.bold = true;
                                    im.italic = true;
                                }
                            }
                            // Extra delimiters beyond the ones used stay as text.
                            let extra = run - want;
                            if extra > 0 {
                                buf.extend(std::iter::repeat_n(c, extra));
                                flush(&mut buf, out);
                            }
                            self.inline_into(&s[i + run..end], &im, out, depth + 1);
                            i = end + want;
                        }
                        None => {
                            buf.extend(std::iter::repeat_n(c, run));
                            i += run;
                        }
                    }
                }
                'h' if (i == 0 || !s[i - 1].is_alphanumeric()) && starts_with(s, i, "http") => {
                    let (url, end) = bare_url(s, i);
                    match url.as_deref().and_then(safe_link) {
                        Some(href) if end > i => {
                            flush(&mut buf, out);
                            out.push(Inline::Text(url.unwrap_or_default(), Marks { link: Some(href), ..m.clone() }));
                            i = end;
                        }
                        _ => {
                            buf.push('h');
                            i += 1;
                        }
                    }
                }
                _ => {
                    buf.push(c);
                    i += 1;
                }
            }
        }
        flush(&mut buf, out);
    }

    /// `[label](dest "title")`, `[label][ref]` or `[label]` (a known reference) at `open`.
    /// Returns the label, the destination and the index after the construct.
    fn link_at(&self, s: &[char], open: usize, st: &mut InlineState) -> Option<(Vec<char>, String, usize)> {
        if st.failed.contains_key(&('[', open)) {
            return None;
        }
        let mut depth = 0;
        let mut j = open;
        let lim = s.len().min(open + SCAN);
        let close = loop {
            if j >= lim {
                st.failed.insert(('[', open), ());
                return None;
            }
            match s[j] {
                '\\' => j += 1,
                '[' => {
                    depth += 1;
                    if depth > MAX_BRACKETS {
                        st.failed.insert(('[', open), ());
                        return None;
                    }
                }
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        break j;
                    }
                }
                _ => {}
            }
            j += 1;
        };
        let label = s[open + 1..close].to_vec();
        if s.get(close + 1) == Some(&'(') {
            let mut k = close + 2;
            let mut parens = 1;
            let lim = s.len().min(close + 2 + SCAN);
            while k < lim {
                match s[k] {
                    '\\' => k += 1,
                    '(' => {
                        parens += 1;
                        if parens > MAX_BRACKETS {
                            return None;
                        }
                    }
                    ')' => {
                        parens -= 1;
                        if parens == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                k += 1;
            }
            if k >= lim {
                return None;
            }
            let inside: String = s[close + 2..k].iter().collect();
            let inside = inside.trim();
            let dest = if let Some(rest) = inside.strip_prefix('<') {
                rest.split('>').next().unwrap_or("").to_string()
            } else {
                // The destination ends at the first space (a title may follow).
                inside.split_whitespace().next().unwrap_or("").to_string()
            };
            return Some((label, dest, k + 1));
        }
        let (key, end) = if s.get(close + 1) == Some(&'[') {
            let e = (close + 2..s.len().min(close + 2 + 1000)).find(|&x| s[x] == ']')?;
            let k: String = s[close + 2..e].iter().collect();
            (if k.is_empty() { label.iter().collect() } else { k }, e + 1)
        } else {
            (label.iter().collect::<String>(), close + 1)
        };
        let dest = self.refs.get(&key.to_lowercase())?.clone();
        Some((label, dest, end))
    }

    /// `<https://…>` autolinks, `<a@b.c>`, and inline HTML tags.
    fn angle(&mut self, s: &[char], i: usize, m: &Marks, depth: usize, st: &mut InlineState) -> Option<(Vec<Inline>, usize)> {
        let lim = s.len().min(i + SCAN);
        let close = (i + 1..lim).find(|&j| s[j] == '>' || s[j] == '<' || s[j] == '\n')?;
        if s[close] != '>' {
            return None;
        }
        let inner: String = s[i + 1..close].iter().collect();
        if !inner.contains(' ') && (inner.contains("://") || inner.contains('@')) {
            let href = if inner.contains("://") { inner.clone() } else { format!("mailto:{inner}") };
            let link = safe_link(&href)?;
            return Some((vec![Inline::Text(inner, Marks { link: Some(link), ..m.clone() })], close + 1));
        }
        let lower = inner.to_ascii_lowercase();
        let name: String = lower.trim_start_matches('/').chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        if name.is_empty() || !lower.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '/') {
            return None;
        }
        if name == "br" {
            return Some((vec![Inline::Break], close + 1));
        }
        if name == "img" {
            let frag: String = s[i..close + 1].iter().collect();
            let blocks = html::parse_fragment(&frag, self.ctx);
            let inl = blocks.into_iter().flat_map(|b| if let Block::Paragraph(x) = b { x } else { Vec::new() }).collect();
            return Some((inl, close + 1));
        }
        if lower.starts_with('/') {
            // A closing tag without its opening one: dropped.
            return Some((Vec::new(), close + 1));
        }
        // The text and background color of `<span style="color:…">`, `<mark style="background:…">`
        // (how Notaya's Markdown export writes colors) and `<font color>`, read as the nearest
        // Notaya palette colors. No other attribute is looked at.
        let (color, highlight) = if matches!(name.as_str(), "span" | "mark" | "font") {
            let tag: String = s[i..close + 1].iter().collect();
            Tokenizer::html(&tag)
                .find_map(|t| if let Token::Start { attrs, .. } = t { Some(attrs) } else { None })
                .map(|attrs| {
                    let (mut c, h) = attr(&attrs, "style").map(palette::from_style).unwrap_or_default();
                    if name == "font" && c.is_none() {
                        c = attr(&attrs, "color").and_then(palette::parse).and_then(palette::text_color);
                    }
                    (c, h)
                })
                .unwrap_or_default()
        } else {
            (None, None)
        };
        let mut im = m.clone();
        let known = match name.as_str() {
            "b" | "strong" => {
                im.bold = true;
                true
            }
            "i" | "em" => {
                im.italic = true;
                true
            }
            "u" | "ins" => {
                im.underline = true;
                true
            }
            "s" | "del" | "strike" => {
                im.strike = true;
                true
            }
            "code" => {
                im.code = true;
                true
            }
            // A plain <mark> is the default highlight.
            "mark" => {
                im.highlight = Some(highlight.unwrap_or_default());
                true
            }
            "span" | "font" if color.is_some() || highlight.is_some() => {
                if highlight.is_some() {
                    im.highlight = highlight;
                }
                true
            }
            _ => false,
        };
        if color.is_some() {
            im.color = color;
        }
        if known {
            let end_tag: Vec<char> = format!("</{name}>").chars().collect();
            let key = ('<', name.bytes().fold(0usize, |a, b| a.wrapping_mul(31).wrapping_add(usize::from(b))));
            if let Some(e) = scan_memo(st, key, close + 1, s.len(), |j| s[j..].iter().take(end_tag.len()).map(|c| c.to_ascii_lowercase()).eq(end_tag.iter().copied())) {
                let mut out = Vec::new();
                if depth < MAX_DEPTH {
                    self.inline_into(&s[close + 1..e], &im, &mut out, depth + 1);
                }
                return Some((out, e + end_tag.len()));
            }
        }
        // Any other tag is dropped; the text around it stays.
        Some((Vec::new(), close + 1))
    }
}

/// What the searches in one piece of text have already learned.
#[derive(Default)]
struct InlineState {
    /// Searches that failed outright: (delimiter, position or kind).
    failed: HashMap<(char, usize), ()>,
    /// For a kind of closer: the span `[from, to)` known to hold none. A later search that
    /// starts inside it continues at `to`, so each stretch is looked through once, not once per
    /// opener (a text of `*a *a *a` ... would otherwise cost 4,000 steps per opener).
    none_in: HashMap<(char, usize), (usize, usize)>,
}

/// The first `j` in `from..len` (looking at most SCAN ahead) for which `hit(j)` is true, using
/// and updating what earlier searches of the same `key` learned.
fn scan_memo(st: &mut InlineState, key: (char, usize), from: usize, len: usize, hit: impl Fn(usize) -> bool) -> Option<usize> {
    let lim = len.min(from.saturating_add(SCAN));
    let (mut start, mut known_from) = (from, from);
    if let Some(&(a, b)) = st.none_in.get(&key) {
        if a <= from && from < b {
            start = b;
            known_from = a;
        }
    }
    for j in start..lim {
        if hit(j) {
            return Some(j);
        }
    }
    st.none_in.insert(key, (known_from, lim.max(start)));
    None
}

fn starts_with(s: &[char], i: usize, p: &str) -> bool {
    p.chars().enumerate().all(|(k, c)| s.get(i + k) == Some(&c))
}

fn find_backticks(s: &[char], from: usize, run: usize, st: &mut InlineState) -> Option<usize> {
    let key = ('`', 1_000_000 + run);
    if let Some(&()) = st.failed.get(&key) {
        return None;
    }
    let mut j = from;
    while j < s.len() {
        if s[j] == '`' {
            let r = s[j..].iter().take_while(|x| **x == '`').count();
            if r == run {
                return Some(j);
            }
            j += r;
        } else {
            j += 1;
        }
    }
    st.failed.insert(key, ());
    None
}

/// A closing run of `want` `c` characters after `from`, not preceded by whitespace.
fn find_closer(s: &[char], from: usize, c: char, want: usize, st: &mut InlineState) -> Option<usize> {
    let key = (c, 2_000_000 + want);
    let lim = s.len().min(from + SCAN);
    // Skip what an earlier search already looked through without finding a closer.
    let (mut j, mut known_from) = (from, from);
    if let Some(&(a, b)) = st.none_in.get(&key) {
        if a <= from && from < b {
            j = b;
            known_from = a;
        }
    }
    while j < lim {
        let x = s[j];
        if x == '`' {
            let r = s[j..].iter().take_while(|y| **y == '`').count();
            j += r;
            continue;
        }
        if x == c {
            let r = s[j..].iter().take_while(|y| **y == c).count();
            let prev = s[j - 1];
            let next = s.get(j + r).copied().unwrap_or(' ');
            let ok_side = !prev.is_whitespace() && !(c == '_' && next.is_alphanumeric());
            if ok_side && j > from && (r == want || (want < 3 && r == 3) || (want == 1 && r > 1 && r != 2)) {
                return Some(j + r - want);
            }
            j += r;
            continue;
        }
        j += 1;
    }
    st.none_in.insert(key, (known_from, lim.max(j)));
    None
}

/// A URL written in the text: up to the first whitespace or `<`, without trailing punctuation.
fn bare_url(s: &[char], i: usize) -> (Option<String>, usize) {
    let mut j = i;
    while j < s.len() && !s[j].is_whitespace() && s[j] != '<' && j - i < 2048 {
        j += 1;
    }
    while j > i && matches!(s[j - 1], '.' | ',' | ':' | ';' | '!' | '?' | '"' | '\'' | '*' | '_' | '~') {
        j -= 1;
    }
    let mut url: String = s[i..j].iter().collect();
    if url.ends_with(')') && url.matches('(').count() < url.matches(')').count() {
        url.pop();
        j -= 1;
    }
    if url.starts_with("http://") || url.starts_with("https://") { (Some(url), j) } else { (None, i) }
}

fn split_row(l: &str) -> Vec<String> {
    let t = l.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').filter(|x| !x.ends_with('\\')).unwrap_or(t);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut in_code = false;
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cur.push('|');
                chars.next();
            }
            '`' => {
                in_code = !in_code;
                cur.push(c);
            }
            '|' if !in_code => cells.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_string());
    cells.truncate(100);
    cells
}

/// The picture of a paragraph that holds one picture and nothing else.
fn picture_only(b: Option<&Block>) -> Option<usize> {
    let Some(Block::Paragraph(inl)) = b else { return None };
    let mut seen = inl.iter().filter(|i| !matches!(i, Inline::Text(t, _) if t.trim().is_empty()));
    match (seen.next(), seen.next()) {
        (Some(Inline::Image(idx)), None) => Some(*idx),
        _ => None,
    }
}

/// The text of a paragraph that is all italic and nothing else (no link, no other mark).
fn italic_only(inl: &[Inline]) -> Option<String> {
    let only = Marks { italic: true, ..Marks::default() };
    let all = !inl.is_empty() && inl.iter().all(|i| matches!(i, Inline::Text(t, m) if *m == only || t.trim().is_empty()));
    let text = doc::plain(inl);
    (all && !text.is_empty() && inl.iter().any(|i| matches!(i, Inline::Text(_, m) if *m == only))).then_some(text)
}

/// `> [!NOTE]` (GitHub's alerts) as a callout; any other quote stays a quote.
fn callout_of(mut blocks: Vec<Block>) -> Block {
    if let Some(Block::Paragraph(first)) = blocks.first_mut() {
        if let Some(Inline::Text(t, _)) = first.first_mut() {
            for (tag, icon) in [("[!NOTE]", "ℹ️"), ("[!TIP]", "💡"), ("[!IMPORTANT]", "❗"), ("[!WARNING]", "⚠️"), ("[!CAUTION]", "🔥")] {
                if t.to_ascii_uppercase().starts_with(tag) {
                    *t = t[tag.len()..].trim_start().to_string();
                    if t.is_empty() {
                        first.remove(0);
                        if matches!(first.first(), Some(Inline::Break)) {
                            first.remove(0);
                        }
                    }
                    return Block::Callout(icon.into(), blocks);
                }
            }
        }
    }
    Block::Quote(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::html::tests::{json, with_ctx};
    use crate::importers::source::Vfs;
    use crate::importers::{zip, Control, Images};

    fn md(src: &str) -> serde_json::Value {
        let (d, _, _) = with_ctx(&[], |ctx| parse(src, ctx));
        json(&d.blocks)
    }

    #[test]
    fn blocks() {
        let v = md("---\ntitle: \"Front\"\ntags: [a]\n---\n# Head #\n\nPara one\nline two  \nbreak\n\nSetext\n===\n\n- a\n- b\n  - nested\n\n1. one\n2. two\n\n- [ ] todo\n- [x] done\n\n> quote\n> more\n\n```rust\nfn x() {}\n```\n\n    indented\n\n---\n\n| A | B |\n|---|:-:|\n| 1 | 2 \\| 3 |\n| only |\n");
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["heading", "paragraph", "heading", "bulletList", "orderedList", "taskList", "blockquote", "codeBlock", "codeBlock", "horizontalRule", "table"]);
        assert_eq!(c[0]["content"][0]["text"], "Head");
        assert_eq!(c[1]["content"][0]["text"], "Para one line two");
        assert_eq!(c[1]["content"][1]["type"], "hardBreak");
        assert_eq!(c[3]["content"][1]["content"][1]["type"], "bulletList", "nested list");
        assert_eq!(c[5]["content"][1]["attrs"]["checked"], true);
        assert_eq!(c[7]["attrs"]["language"], "rust");
        assert_eq!(c[8]["content"][0]["text"], "indented");
        assert_eq!(c[10]["content"][1]["content"][1]["content"][0]["content"][0]["text"], "2 | 3");
        assert_eq!(c[10]["content"][2]["content"].as_array().unwrap().len(), 2, "short rows are padded");
        let (d, _, _) = with_ctx(&[], |ctx| parse("---\ntitle: \"Front\"\n---\nx", ctx));
        assert_eq!(d.title.as_deref(), Some("Front"));
    }

    #[test]
    fn standalone_comment_leaves_the_next_markdown_block_intact() {
        let titled = md("<!-- export ownership -->\n# Original title\n\nPage body.");
        let content = titled["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["type"], "heading");
        assert_eq!(content[0]["attrs"]["level"], 1);
        assert_eq!(content[0]["content"][0]["text"], "Original title");
        assert_eq!(content[1]["content"][0]["text"], "Page body.");

        let trailing = md("<!-- comment --> Keep this -->\n\n# Heading");
        assert!(trailing.to_string().contains("Keep this -->"));

        let literal = md("    <!-- code example -->\n\n# Heading");
        assert_eq!(literal["content"][0]["type"], "codeBlock");
        assert_eq!(literal["content"][0]["content"][0]["text"], "<!-- code example -->");
    }

    #[test]
    fn inline_marks_and_links() {
        let v = md("**b** *i* _i2_ ***bi*** ~~s~~ ==h== `c*not*` [l](https://a.b/c \"t\") [bad](javascript:alert(1)) <https://auto.link> see https://bare.example/x). snake_case_word 2*3*4 $x^2$ costs $5 and $6 a<br>b <u>u</u> &copy; [r][1]\n\n[1]: https://ref.example");
        let s = v.to_string();
        for mark in ["bold", "italic", "strike", "highlight", "code", "link", "underline", "hardBreak", "mathInline"] {
            assert!(s.contains(&format!("\"{mark}\"")), "{mark} in {s}");
        }
        assert!(s.contains("c*not*"), "no emphasis inside code");
        assert!(!s.contains("javascript"));
        assert!(s.contains("https://auto.link") && s.contains("https://bare.example/x\"") && s.contains("https://ref.example"));
        assert!(s.contains("snake_case_word"), "intraword underscores are kept");
        assert!(s.contains("costs $5 and $6"), "money is not math");
        assert!(s.contains("©"));
    }

    #[test]
    fn pictures_and_notion_markdown() {
        let png: &[u8] = b"\x89PNG\r\n\x1a\nxxxx";
        let src = "# Page\n\n![shot](Page%200123456789abcdef0123456789abcdef/shot.png)\n\n![remote](https://img.example/a.png)\n\n![missing](nope.png)\n\n<aside>\n💡 A Notion callout\n\n</aside>\n\n> [!WARNING]\n> Careful\n";
        let ((d, v), images, problems) = with_ctx(&[("Page 0123456789abcdef0123456789abcdef/shot.png", png)], |ctx| {
            let d = parse(src, ctx);
            let v = json(&d.blocks);
            (d, v)
        });
        let _ = d;
        let c = v["content"].as_array().unwrap();
        assert_eq!(images.list.len(), 1);
        assert_eq!(c[1]["type"], "image");
        assert!(c[2].to_string().contains("https://img.example/a.png"), "remote picture kept as a link");
        assert!(c[3].to_string().contains("Picture not imported"));
        assert_eq!(problems.len(), 1);
        assert_eq!(c[4]["type"], "callout");
        assert_eq!(c[5]["type"], "callout");
        assert_eq!(c[5]["attrs"]["icon"], "⚠️");
    }

    #[test]
    fn a_notion_callout_keeps_its_text_after_a_blank_line() {
        let src = "<aside>\n\u{1F3E0}\n\nStart here: **bold** text\n\n- one\n- two\n\n</aside>\n\nAfter\n\n<aside>\nno emoji line\n\nsecond paragraph\n\n</aside>\n\n<aside>\nnever closed\n\nStays a paragraph";
        let (blocks, _, _) = with_ctx(&[], |ctx| parse(src, ctx).blocks);
        let v = json(&blocks);
        let c = v["content"].as_array().unwrap();
        assert_eq!(c[0]["type"], "callout");
        assert_eq!(c[0]["attrs"]["icon"], "\u{1F3E0}", "the emoji line is the icon");
        let inner = c[0]["content"].to_string();
        assert!(inner.contains("Start here") && inner.contains("bold") && inner.contains("bulletList") && inner.contains("two"), "the text after the blank line is inside the callout: {inner}");
        assert!(!inner.contains("\u{1F3E0}"), "the icon is not repeated in the text: {inner}");
        assert_eq!(c[1]["type"], "paragraph");
        assert!(c[1].to_string().contains("After"), "text after the callout stays outside it");
        assert_eq!(c[2]["type"], "callout");
        assert_eq!(c[2]["attrs"]["icon"], "\u{1F4A1}", "no emoji line keeps the default icon");
        assert!(c[2]["content"].to_string().contains("second paragraph"), "{}", c[2]);
        assert!(c[2]["content"].to_string().contains("no emoji line"), "{}", c[2]);
        assert!(c.len() >= 4 && c.iter().skip(3).all(|b| b["type"] != "callout" || !b.to_string().contains("Stays a paragraph")), "a callout that is never closed does not swallow the rest: {v}");
        assert!(c.iter().any(|b| b.to_string().contains("Stays a paragraph")));
    }

    #[test]
    fn a_paragraph_that_is_a_link_to_another_page_becomes_a_page_link() {
        let files: &[(&str, &[u8])] = &[("Sub page 0123456789abcdef0123456789abcdef.md", b"# Sub"), ("notes.txt", b"x")];
        let src = "[Sub page](Sub%20page%200123456789abcdef0123456789abcdef.md)\n\n[Gone](Gone.md)\n\n[Text](notes.txt)\n\nSee [Sub page](Sub%20page%200123456789abcdef0123456789abcdef.md) here.\n\n[Up](../Sub%20page%200123456789abcdef0123456789abcdef.md)";
        let (blocks, _, _) = with_ctx(files, |ctx| parse(src, ctx).blocks);
        assert_eq!(blocks[0], Block::PageLink("Sub page 0123456789abcdef0123456789abcdef.md".into(), "Sub page".into()));
        assert!(matches!(blocks[1], Block::Paragraph(_)), "a missing page stays text");
        assert!(matches!(blocks[2], Block::Paragraph(_)), "only Markdown and HTML pages");
        assert!(matches!(blocks[3], Block::Paragraph(_)), "a link inside a sentence stays a link");
        assert!(matches!(blocks[4], Block::Paragraph(_)), "never outside the import");
    }

    #[test]
    fn a_shelf_or_toggle_body_is_read_as_markdown_with_page_link_rows_and_lists() {
        let files: &[(&str, &[u8])] = &[("Plain page.md", b"# Plain"), ("Beta page.md", b"# Beta")];
        let src = "<details open class=\"notaya-shelf\">\n<summary>Shelf</summary>\n\nShelf body\n\n\u{2192} [Plain page](Plain%20page.md)\n\n\u{2192} [Beta page](Beta%20page.md)\n\n- one\n- two\n\n<details open>\n<summary>Inner</summary>\n\nDeep\n\n</details>\n\n</details>\n\nAfter";
        let (blocks, _, _) = with_ctx(files, |ctx| parse(src, ctx).blocks);
        assert_eq!(blocks.len(), 2, "{blocks:?}");
        let Block::Shelf(inner) = &blocks[0] else { panic!("a shelf: {blocks:?}") };
        assert!(matches!(inner[0], Block::Paragraph(_)), "the summary line");
        // The arrow rows are still paragraphs here; the renderer turns "→ [Page](file)" lines into page-link rows.
        for (k, want) in [(2, "notaya-src:Plain page.md"), (3, "notaya-src:Beta page.md")] {
            let Block::Paragraph(inl) = &inner[k] else { panic!("{inner:?}") };
            assert!(inl.iter().any(|i| matches!(i, Inline::Text(_, m) if m.link.as_deref() == Some(want))), "{inner:?}");
        }
        assert!(matches!(inner[4], Block::Bullets(_)), "a list: {inner:?}");
        assert!(matches!(&inner[5], Block::Toggle(t) if t.len() == 2), "a toggle inside: {inner:?}");
        assert!(matches!(blocks[1], Block::Paragraph(_)));
        // Two details side by side, or a block that is not closed, keep the old reading (as HTML).
        let (b2, _, _) = with_ctx(files, |ctx| parse("<details>\n<summary>A</summary>\n\nx\n\n</details>\n<details>\n<summary>B</summary>\n\ny\n\n</details>", ctx).blocks);
        assert_eq!(b2.len(), 2, "{b2:?}");
    }

    #[test]
    fn fenced_mermaid_becomes_a_diagram_block() {
        let src = "Intro\n\n```mermaid\nflowchart TD\n  A --> B\n```\n\n```Mermaid\n```\n\n```js\nlet x = 1;\n```\n";
        let v = md(src);
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        // An empty fence stays an (empty) code block; other languages stay code.
        assert_eq!(types, ["paragraph", "mermaidBlock", "codeBlock", "codeBlock"], "{v}");
        assert_eq!(c[1]["attrs"]["code"], "flowchart TD\n  A --> B");
        assert_eq!(c[1]["attrs"]["showCode"], false);
    }

    #[test]
    fn notion_markdown_toggles_equations_todos_and_bookmarks() {
        let src = "- [x] done\n- [ ] open\n\n<details>\n<summary>More</summary>\n\nHidden part\n\n</details>\n\n$$\nE = mc^2\n$$\n\n[An article](https://example.com/a)\n\n> quoted\n\n1. one\n2. two\n";
        let v = md(src);
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["taskList", "toggle", "mathBlock", "paragraph", "blockquote", "orderedList"], "{v}");
        assert_eq!(c[0]["content"][0]["attrs"]["checked"], true);
        assert_eq!(c[2]["attrs"]["latex"], "E = mc^2");
        assert!(c[3].to_string().contains("https://example.com/a"));
        assert!(c[1].to_string().contains("Hidden part"));
    }

    #[test]
    fn inline_color_html_becomes_palette_colors() {
        use crate::importers::html::tests::colors_of;
        use crate::importers::palette::{HIGHLIGHTS, TEXT_COLORS};
        // As Notaya's Markdown export writes them.
        let mut src = String::new();
        for (i, (_, v)) in TEXT_COLORS.iter().enumerate() {
            src.push_str(&format!("<span style=\"color:{}\">tc{i}</span> ", v.to_uppercase()));
        }
        for (i, (_, v)) in HIGHLIGHTS.iter().enumerate() {
            src.push_str(&format!("<mark style=\"background:#{}\">hl{i}</mark> ", crate::export::doc::Rgb::parse(v).unwrap().hex()));
        }
        src.push_str("<span style=\"color:#D04F4F\"><mark style=\"background:#EEE5C4\">**both**</mark></span> <mark>plain</mark> <span style=\"font-weight:bold\">nostyle</span> <span style=\"color:url(x)\">bad</span> <font color=\"#d44c47\">font</font> <span style=\"color:#D04F4F\">open");
        let v = md(&src);
        for (i, (name, c)) in TEXT_COLORS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("tc{i}")).0.as_deref(), Some(*c), "text color {name}");
        }
        for (i, (name, h)) in HIGHLIGHTS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("hl{i}")).1.as_deref(), Some(*h), "highlight {name}");
        }
        assert_eq!(colors_of(&v, "both"), (Some("#d04f4f".into()), Some(HIGHLIGHTS[9].1.into())));
        assert!(v.to_string().contains("\"bold\""), "Markdown inside the tags is still read");
        assert_eq!(colors_of(&v, "plain"), (None, Some(String::new())));
        assert_eq!(colors_of(&v, "nostyle"), (None, None), "other spans are dropped, their text stays");
        assert_eq!(colors_of(&v, "bad"), (None, None));
        assert_eq!(colors_of(&v, "font").0.as_deref(), Some("#d44c47"));
        assert!(v.to_string().contains("open"), "a span that is never closed keeps its text");
        assert!(!v.to_string().contains("span") && !v.to_string().contains("url("));
    }

    #[test]
    fn shelves_and_picture_captions_come_back() {
        let png: &[u8] = b"\x89PNG\r\n\x1a\nxxxx";
        let src = "<details open class=\"notaya-shelf\">\n<summary>Shelf title</summary>\n\nInside the shelf\n\n</details>\n\n<details open>\n<summary>Toggle</summary>\n\nx\n\n</details>\n\n![A caption](a.png)\n\n*A caption*\n\n![b](a.png)\n\nNot *a caption*\n\n![c](a.png)\n\n![missing](nope.png)";
        let (v, _, _) = with_ctx(&[("a.png", png)], |ctx| json(&parse(src, ctx).blocks));
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["toggle", "toggle", "image", "image", "paragraph", "image", "paragraph"], "{v}");
        assert_eq!(c[0]["attrs"]["shelf"], true);
        assert!(c[0].to_string().contains("Inside the shelf"));
        assert!(c[1]["attrs"].get("shelf").is_none());
        assert_eq!(c[2]["attrs"]["caption"], "A caption");
        assert_eq!(c[3]["attrs"]["caption"], "", "text that is not all italic is no caption");
        assert_eq!(c[5]["attrs"]["caption"], "", "a missing picture's note is no caption");
    }

    #[test]
    fn hostile_markdown_stays_fast_and_safe() {
        let _heavy = crate::importers::heavy();
        let many = "*a ".repeat(50_000) + &"[".repeat(50_000) + &"`".repeat(10_000) + &"_".repeat(50_000);
        let start = std::time::Instant::now();
        let _ = md(&many);
        let deep = ">".repeat(5_000) + " deep\n" + &"- ".repeat(3_000) + "x";
        let v = md(&deep);
        assert!(v.to_string().contains("deep"));
        assert!(start.elapsed().as_secs() < 90, "took {:?}", start.elapsed());
        let v = md("<script>alert(1)</script>\n\n<div onclick=\"x()\">ok</div>\n\n[x](  javascript:alert(1) )");
        let s = v.to_string();
        assert!(!s.contains("alert") && !s.contains("onclick") && s.contains("ok"));
    }

    fn plain_of(blocks: &[Block]) -> String {
        blocks.iter().map(|b| if let Block::Paragraph(i) = b { doc::plain(i) } else { String::new() }).collect::<Vec<_>>().join("|")
    }

    #[test]
    fn a_failed_search_in_one_piece_does_not_spoil_the_rest() {
        // `**a *b**` is bold "a *b"; the lone `*` inside fails to find a partner, and that must
        // not make the later `*c*` a literal.
        let (d, _, _) = with_ctx(&[], |ctx| parse("**a *b** then *c*", ctx));
        let Block::Paragraph(inl) = &d.blocks[0] else { panic!("{:?}", d.blocks) };
        let flags: Vec<(String, bool, bool)> = inl
            .iter()
            .filter_map(|i| if let Inline::Text(t, m) = i { Some((t.clone(), m.bold, m.italic)) } else { None })
            .collect();
        assert!(flags.contains(&("c".to_string(), false, true)), "{flags:?}");
        assert!(flags.iter().any(|(t, b, _)| t.contains("a *b") && *b), "{flags:?}");
        // The same for backticks and `$`.
        let (d, _, _) = with_ctx(&[], |ctx| parse("**x ` y** and `z` and $a$", ctx));
        let Block::Paragraph(inl) = &d.blocks[0] else { panic!() };
        assert!(inl.iter().any(|i| matches!(i, Inline::Text(t, m) if t == "z" && m.code)), "{inl:?}");
        assert!(inl.iter().any(|i| matches!(i, Inline::Math(m) if m == "a")), "{inl:?}");
    }

    #[test]
    fn long_runs_of_blank_lines_and_openers_are_linear() {
        let _heavy = crate::importers::heavy();
        let start = std::time::Instant::now();
        // The reviewer's probe: a list item, 40,000 blank lines, then an indented continuation.
        let src = format!("- a\n{}  b\n", "\n".repeat(40_000));
        let (d, _, _) = with_ctx(&[], |ctx| parse(&src, ctx));
        assert!(matches!(d.blocks[0], Block::Bullets(_)));
        // Millions of blank lines in a row (a 20 MB file of line breaks) must not need gigabytes.
        let src = format!("- a\n{}  b\n", "\n".repeat(1_000_000));
        let _ = with_ctx(&[], |ctx| parse(&src, ctx));
        // Openers that never close: each used to search 4,000 characters ahead.
        for unit in ["*a ", "_a ", "~~a ", "==a ", "$a ", "[a](b ", "[a ", "`a ", "<b>a ", "![a](", "[[[[", "*[a](b) ", "<span style=\"color:#d04f4f\">a ", "<mark style=\"background:#EEE5C4;color:#d04f4f\">"] {
            let src = unit.repeat(60_000);
            let (d, _, _) = with_ctx(&[], |ctx| parse(&src, ctx));
            assert!(!d.blocks.is_empty(), "{unit}");
        }
        // Deep nesting.
        let src = "> ".repeat(10_000) + "x";
        let _ = with_ctx(&[], |ctx| parse(&src, ctx));
        let src = "- ".repeat(10_000) + "x";
        let _ = with_ctx(&[], |ctx| parse(&src, ctx));
        assert!(start.elapsed().as_secs() < 180, "{:?}", start.elapsed());
    }

    #[test]
    fn pipe_tables_are_capped() {
        let mut src = String::from("| a | b | c |\n|---|---|---|\n");
        for _ in 0..30_000 {
            src.push_str("| 1 | 2 | 3 |\n");
        }
        let (d, _, problems) = with_ctx(&[], |ctx| parse(&src, ctx));
        let Block::Table(rows) = &d.blocks[0] else { panic!() };
        assert!(rows.len() * 3 <= doc::MAX_TABLE_CELLS, "{}", rows.len());
        assert!(problems.iter().any(|p| p.item.starts_with("Table")), "{problems:?}");
        assert_eq!(d.blocks.len(), 1, "the rest of the rows were read past, not turned into paragraphs");
        // A hundred columns on 3,000 rows.
        let head = format!("|{}\n|{}\n", "a|".repeat(100), "-|".repeat(100));
        let row = format!("|{}\n", "x|".repeat(100));
        let src = head + &row.repeat(3_000);
        let (d, _, _) = with_ctx(&[], |ctx| parse(&src, ctx));
        let Block::Table(rows) = &d.blocks[0] else { panic!() };
        assert!(rows.len() * 100 <= doc::MAX_TABLE_CELLS);
    }

    #[test]
    fn a_stopped_import_stops_the_markdown_converter() {
        let src = "paragraph\n\n".repeat(200_000);
        let vfs = Vfs::from_zip_bytes(zip::build(&[("a.md", b"x")], false)).unwrap();
        let ctl = Control::expired();
        let mut images = Images::default();
        let mut problems = Vec::new();
        let mut ctx = Ctx { vfs: &vfs, ctl: &ctl, dir: String::new(), images: &mut images, problems: &mut problems, name: "t".into() };
        let d = parse(&src, &mut ctx);
        assert!(d.blocks.len() < 200_000, "{}", d.blocks.len());
        let _ = plain_of(&d.blocks);
    }
}
