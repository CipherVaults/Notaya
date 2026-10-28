//! Markdown (CommonMark and GitHub's extensions, the common subset) to the import model.
//!
//! Blocks: ATX and Setext headings, paragraphs, bullet, numbered and task lists (nested by
//! indentation), quotes, fenced and indented code, rules, pipe tables, HTML blocks (passed to
//! the HTML converter, which sanitises them; Notion writes callouts as `<aside>`), front
//! matter (its `title:` is used, the rest dropped) and link reference definitions.
//! Inline: bold, italic, strike, `==highlight==`, code, links, autolinks, bare URLs, pictures,
//! hard breaks, entities, `$math$`, and a few inline HTML tags.
//!
//! Hostile input: nesting is capped, the search for a closing delimiter is bounded and a
//! failed search is remembered, so the work stays roughly linear.

use std::collections::HashMap;

use super::doc::{self, Block, Cell, Inline, Marks};
use super::markup::decode_one;
use super::{html, safe_link, Ctx};

const MAX_DEPTH: usize = 16;
/// Paragraphs longer than this are taken as plain text (no inline parsing).
const MAX_INLINE: usize = 200_000;
/// How far to look for a closing delimiter or bracket.
const SCAN: usize = 4_000;

pub struct MdDoc {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

pub fn parse(src: &str, ctx: &mut Ctx) -> MdDoc {
    let src = src.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = src.lines().map(expand_tabs).collect();
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

    let mut p = Parser { ctx, refs };
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
}

impl Parser<'_, '_> {
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
                    out.push(Block::Paragraph(self.inline(&text)));
                }
            };
        }
        while i < n {
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
                let mut html_lines = Vec::new();
                while i < n && !blank(&lines[i]) {
                    html_lines.push(lines[i].as_str());
                    i += 1;
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
                    let next = lines[*i..].iter().position(|x| !blank(x)).map(|p| *i + p);
                    match next {
                        Some(j) if indent(&lines[j]) >= content_col => {
                            item.push(String::new());
                            *i += 1;
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
        let mut rows = vec![head.iter().map(|c| Cell { header: true, colspan: 1, blocks: vec![Block::Paragraph(self.inline(c))] }).collect::<Vec<_>>()];
        while *i < lines.len() && !blank(&lines[*i]) && lines[*i].contains('|') && rows.len() < 5_000 {
            let mut cells = split_row(&lines[*i]);
            cells.resize(head.len(), String::new());
            rows.push(cells.iter().map(|c| Cell { header: false, colspan: 1, blocks: vec![Block::Paragraph(self.inline(c))] }).collect());
            *i += 1;
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
        let mut st = InlineState { failed: HashMap::new() };
        self.inline_into(&chars, &Marks::default(), &mut out, 0, &mut st);
        out
    }

    fn inline_into(&mut self, s: &[char], m: &Marks, out: &mut Vec<Inline>, depth: usize, st: &mut InlineState) {
        let mut buf = String::new();
        let flush = |buf: &mut String, out: &mut Vec<Inline>| {
            if !buf.is_empty() {
                out.push(Inline::Text(std::mem::take(buf), m.clone()));
            }
        };
        let n = s.len();
        let mut i = 0;
        while i < n {
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
                        lm.link = safe_link(&dest);
                        if depth < MAX_DEPTH {
                            self.inline_into(&label, &lm, out, depth + 1, st);
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
                    let end = (i + 1..n.min(i + SCAN)).find(|&j| s[j] == '$' && s[j - 1] != ' ' && s[j - 1] != '\\' && s[j] != '\n');
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
                            self.inline_into(&s[i + run..end], &im, out, depth + 1, st);
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
                '[' => depth += 1,
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
                    '(' => parens += 1,
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
        let mark: Option<fn(&mut Marks)> = match name.as_str() {
            "b" | "strong" => Some(|m| m.bold = true),
            "i" | "em" => Some(|m| m.italic = true),
            "u" | "ins" => Some(|m| m.underline = true),
            "s" | "del" | "strike" => Some(|m| m.strike = true),
            "mark" => Some(|m| m.highlight = Some(String::new())),
            "code" => Some(|m| m.code = true),
            _ => None,
        };
        if lower.starts_with('/') {
            // A closing tag without its opening one: dropped.
            return Some((Vec::new(), close + 1));
        }
        if let Some(f) = mark {
            let end_tag: Vec<char> = format!("</{name}>").chars().collect();
            let lim = s.len().min(close + 1 + SCAN);
            if let Some(e) = (close + 1..lim).find(|&j| s[j..].iter().take(end_tag.len()).map(|c| c.to_ascii_lowercase()).eq(end_tag.iter().copied())) {
                let mut im = m.clone();
                f(&mut im);
                let mut out = Vec::new();
                if depth < MAX_DEPTH {
                    self.inline_into(&s[close + 1..e], &im, &mut out, depth + 1, st);
                }
                return Some((out, e + end_tag.len()));
            }
        }
        // Any other tag is dropped; the text around it stays.
        Some((Vec::new(), close + 1))
    }
}

struct InlineState {
    /// Searches that already failed: (delimiter, position). For a closer of a given kind the
    /// key is (char, run length as position), since no closer after one place means none after
    /// any later place either.
    failed: HashMap<(char, usize), ()>,
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
    if st.failed.contains_key(&key) {
        return None;
    }
    let lim = s.len().min(from + SCAN);
    let mut j = from;
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
    if lim == s.len() {
        st.failed.insert(key, ());
    }
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
    fn hostile_markdown_stays_fast_and_safe() {
        let many = "*a ".repeat(50_000) + &"[".repeat(50_000) + &"`".repeat(10_000) + &"_".repeat(50_000);
        let start = std::time::Instant::now();
        let _ = md(&many);
        let deep = ">".repeat(5_000) + " deep\n" + &"- ".repeat(3_000) + "x";
        let v = md(&deep);
        assert!(v.to_string().contains("deep"));
        assert!(start.elapsed().as_secs() < 20, "took {:?}", start.elapsed());
        let v = md("<script>alert(1)</script>\n\n<div onclick=\"x()\">ok</div>\n\n[x](  javascript:alert(1) )");
        let s = v.to_string();
        assert!(!s.contains("alert") && !s.contains("onclick") && s.contains("ok"));
    }
}
