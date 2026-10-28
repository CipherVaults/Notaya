//! HTML (and Notion's HTML export) to the import document model.
//!
//! The HTML is first built into a small tree (a forgiving builder: implied end tags for `p`,
//! `li`, table cells; at most 200 levels deep; at most a million tokens), then walked. The
//! walk keeps only known blocks and marks: scripts, styles, frames, forms, SVG and every
//! attribute except a few (href, src, alt, class, start, colspan, checked, style for bold,
//! italic, underline and strike) are dropped, so nothing active can reach the editor. Links
//! keep only http, https and mailto. Notion's classes are mapped: text and background colors
//! (`highlight-*`, `block-color-*`, written as the matching Notaya colors Ash to Cherry), callouts, toggles, to-do lists, code languages,
//! equations, bookmarks, columns, links to other pages, the page icon, the cover picture and
//! the page font (`page serif`, `page mono`).

use super::doc::{self, Block, Cell, Inline, Marks, FONT_MONO, FONT_SERIF};
use super::markup::{attr, Token, Tokenizer};
use super::Ctx;

pub struct El {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

pub enum Node {
    El(El),
    Text(String),
}

const MAX_TOKENS: usize = 1_000_000;
const MAX_TREE_DEPTH: usize = 200;

const VOID: &[&str] = &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"];

/// Elements whose start closes an open `<p>`.
const CLOSES_P: &[&str] = &[
    "address", "article", "aside", "blockquote", "details", "div", "dl", "fieldset", "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4",
    "h5", "h6", "header", "hr", "main", "nav", "ol", "p", "pre", "section", "table", "ul",
];

/// Dropped with everything inside.
const DROP: &[&str] = &[
    "script", "style", "noscript", "template", "iframe", "object", "embed", "svg", "canvas", "select", "option", "button", "textarea", "video", "audio",
    "source", "track", "map", "area", "meta", "link", "base", "input", "frame", "frameset", "applet", "dialog",
];

impl El {
    fn new(name: &str, attrs: Vec<(String, String)>) -> El {
        El { name: name.to_string(), attrs, children: Vec::new() }
    }

    fn attr(&self, name: &str) -> Option<&str> {
        attr(&self.attrs, name)
    }

    fn classes(&self) -> impl Iterator<Item = &str> {
        self.attr("class").unwrap_or("").split_whitespace()
    }

    fn has_class(&self, c: &str) -> bool {
        self.classes().any(|x| x == c)
    }

    fn elements(&self) -> impl Iterator<Item = &El> {
        self.children.iter().filter_map(|n| if let Node::El(e) = n { Some(e) } else { None })
    }

    /// First element below this one (depth first) that matches.
    fn find(&self, pred: &dyn Fn(&El) -> bool) -> Option<&El> {
        fn go<'e>(e: &'e El, pred: &dyn Fn(&El) -> bool, depth: usize) -> Option<&'e El> {
            if depth > 64 {
                return None;
            }
            for c in e.elements() {
                if pred(c) {
                    return Some(c);
                }
                if let Some(f) = go(c, pred, depth + 1) {
                    return Some(f);
                }
            }
            None
        }
        go(self, pred, 0)
    }

    /// All the text inside, scripts and styles excluded.
    pub fn text(&self) -> String {
        fn go(e: &El, out: &mut String, depth: usize) {
            if depth > 64 || DROP.contains(&e.name.as_str()) {
                return;
            }
            for c in &e.children {
                match c {
                    Node::Text(t) => out.push_str(t),
                    Node::El(x) if x.name == "br" => out.push('\n'),
                    Node::El(x) => go(x, out, depth + 1),
                }
            }
        }
        let mut s = String::new();
        go(self, &mut s, 0);
        s
    }
}

/// Builds the tree. Never fails.
pub fn build(src: &str) -> El {
    let mut stack = vec![El::new("#root", Vec::new())];
    fn pop(stack: &mut Vec<El>) {
        if stack.len() > 1 {
            let e = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(Node::El(e));
        }
    }
    /// Closes the innermost open element named in `names`, unless a `boundary` comes first.
    fn close(stack: &mut Vec<El>, names: &[&str], boundary: &[&str]) {
        for i in (1..stack.len()).rev() {
            let n = stack[i].name.as_str();
            if names.contains(&n) {
                while stack.len() > i {
                    pop(stack);
                }
                return;
            }
            if boundary.contains(&n) {
                return;
            }
        }
    }
    for (count, tok) in Tokenizer::html(src).enumerate() {
        if count > MAX_TOKENS {
            break;
        }
        match tok {
            Token::Text(t) => stack.last_mut().unwrap().children.push(Node::Text(t)),
            Token::Start { name, attrs, self_closing } => {
                let n = name.as_str();
                if CLOSES_P.contains(&n) {
                    close(&mut stack, &["p"], &["div", "li", "td", "th", "blockquote", "table", "details", "figure", "body"]);
                }
                match n {
                    "li" => close(&mut stack, &["li"], &["ul", "ol"]),
                    "td" | "th" => close(&mut stack, &["td", "th"], &["tr", "table"]),
                    "tr" => close(&mut stack, &["tr"], &["table"]),
                    "thead" | "tbody" | "tfoot" => close(&mut stack, &["thead", "tbody", "tfoot"], &["table"]),
                    "dt" | "dd" => close(&mut stack, &["dt", "dd"], &["dl"]),
                    _ => {}
                }
                let el = El::new(n, attrs);
                if VOID.contains(&n) || self_closing || stack.len() >= MAX_TREE_DEPTH {
                    stack.last_mut().unwrap().children.push(Node::El(el));
                } else {
                    stack.push(el);
                }
            }
            Token::End { name } => {
                if name == "br" {
                    stack.last_mut().unwrap().children.push(Node::El(El::new("br", Vec::new())));
                    continue;
                }
                if let Some(pos) = stack.iter().rposition(|e| e.name == name) {
                    if pos > 0 {
                        while stack.len() > pos {
                            pop(&mut stack);
                        }
                    }
                }
            }
        }
    }
    while stack.len() > 1 {
        pop(&mut stack);
    }
    stack.pop().unwrap()
}

/// What a whole HTML page gave.
#[derive(Default)]
pub struct HtmlDoc {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
    /// The page's emoji (Notion's page icon).
    pub icon: Option<String>,
    /// The cover picture (Notion's page cover), as an image index.
    pub cover: Option<usize>,
}

pub fn parse_document(src: &str, ctx: &mut Ctx) -> HtmlDoc {
    let root = build(src);
    let mut c = Conv { ctx, doc: HtmlDoc::default(), depth: 0 };
    if let Some(t) = root.find(&|e| e.name == "title") {
        let t = doc::collapse_ws(&t.text());
        if !t.is_empty() {
            c.doc.title = Some(t);
        }
    }
    let mut out = Out::default();
    c.walk(&root.children, &Marks::default(), &mut out);
    c.doc.blocks = out.finish();
    c.doc
}

/// HTML inside a Markdown file (Notion's `<aside>` callouts, `<img>`, `<details>`...).
pub fn parse_fragment(src: &str, ctx: &mut Ctx) -> Vec<Block> {
    let root = build(src);
    let mut c = Conv { ctx, doc: HtmlDoc::default(), depth: 0 };
    let mut out = Out::default();
    c.walk(&root.children, &Marks::default(), &mut out);
    out.finish()
}

/// Blocks being collected, plus the paragraph being filled.
#[derive(Default)]
struct Out {
    blocks: Vec<Block>,
    inl: Vec<Inline>,
}

impl Out {
    fn flush(&mut self) {
        if self.inl.is_empty() {
            return;
        }
        let inl = std::mem::take(&mut self.inl);
        let only_space = inl.iter().all(|i| matches!(i, Inline::Text(t, _) if t.trim().is_empty()) || matches!(i, Inline::Break));
        if !only_space {
            self.blocks.push(Block::Paragraph(inl));
        }
    }

    fn block(&mut self, b: Block) {
        self.flush();
        self.blocks.push(b);
    }

    fn text(&mut self, t: &str, m: &Marks, pre: bool) {
        if pre {
            self.inl.push(Inline::Text(t.to_string(), m.clone()));
            return;
        }
        let mut s = String::with_capacity(t.len());
        let mut last_space = match self.inl.last() {
            None => true,
            Some(Inline::Text(p, _)) => p.ends_with(' '),
            Some(Inline::Break) => true,
            _ => false,
        };
        for ch in t.chars() {
            if ch.is_whitespace() && ch != '\u{a0}' {
                if !last_space {
                    s.push(' ');
                    last_space = true;
                }
            } else {
                s.push(ch);
                last_space = false;
            }
        }
        if !s.is_empty() {
            self.inl.push(Inline::Text(s, m.clone()));
        }
    }

    fn finish(mut self) -> Vec<Block> {
        self.flush();
        self.blocks
    }
}

struct Conv<'c, 'a> {
    ctx: &'c mut Ctx<'a>,
    doc: HtmlDoc,
    depth: usize,
}

/// Bold, italic, underline and strike from a `style` attribute (Google Docs, Word HTML, Notion).
fn style_marks(style: &str, m: &mut Marks) {
    for decl in style.split(';') {
        let Some((k, v)) = decl.split_once(':') else { continue };
        let k = k.trim().to_ascii_lowercase();
        let v = v.trim().to_ascii_lowercase();
        match k.as_str() {
            "font-weight" if v == "bold" || v == "bolder" || v.parse::<u32>().is_ok_and(|n| n >= 600) => m.bold = true,
            "font-style" if v.contains("italic") => m.italic = true,
            "text-decoration" | "text-decoration-line" => {
                if v.contains("underline") {
                    m.underline = true;
                }
                if v.contains("line-through") {
                    m.strike = true;
                }
            }
            // Notion draws underline as a bottom border.
            "border-bottom" if v.contains("solid") => m.underline = true,
            _ => {}
        }
    }
}

/// Notion's color classes (`highlight-red`, `highlight-red_background`, `block-color-blue`...),
/// written as the matching Notaya palette colors (Ash, Cocoa, Amber, Honey, Moss, Denim, Plum,
/// Orchid, Cherry) with their exact values.
fn color_marks(e: &El, m: &mut Marks) {
    for c in e.classes() {
        let name = c.strip_prefix("highlight-").or_else(|| c.strip_prefix("block-color-")).or_else(|| c.strip_prefix("color-"));
        if let Some((background, value)) = name.and_then(doc::notion_color) {
            if background {
                m.highlight = Some(value.to_string());
            } else {
                m.color = Some(value.to_string());
            }
        }
    }
}

/// Code language from `language-xxx` classes (Notion writes `language-JavaScript`).
fn code_language(e: &El) -> String {
    let lang = e
        .classes()
        .chain(e.find(&|x| x.name == "code").map(|c| c.classes().collect::<Vec<_>>()).unwrap_or_default())
        .find_map(|c| c.strip_prefix("language-"))
        .unwrap_or("")
        .to_ascii_lowercase();
    match lang.as_str() {
        "plain" | "text" | "plaintext" => String::new(),
        "c++" => "cpp".into(),
        "c#" => "csharp".into(),
        "shell" | "sh" => "bash".into(),
        "js" => "javascript".into(),
        "ts" => "typescript".into(),
        _ => lang,
    }
}

/// The TeX source of a KaTeX or MathML equation.
fn tex_of(e: &El) -> Option<String> {
    let a = if e.name == "annotation" { Some(e) } else { e.find(&|x| x.name == "annotation" && x.attr("encoding").is_some_and(|v| v.contains("tex"))) }?;
    let t = a.text().trim().to_string();
    (!t.is_empty()).then_some(t)
}

impl Conv<'_, '_> {
    fn walk(&mut self, nodes: &[Node], m: &Marks, out: &mut Out) {
        for n in nodes {
            match n {
                Node::Text(t) => out.text(t, m, false),
                Node::El(e) => {
                    self.depth += 1;
                    if self.depth < 120 {
                        self.element(e, m, out);
                    } else {
                        // Too deep: keep the text only.
                        out.text(&e.text(), m, false);
                    }
                    self.depth -= 1;
                }
            }
        }
    }

    /// Children as blocks of their own (for containers).
    fn blocks(&mut self, nodes: &[Node], m: &Marks) -> Vec<Block> {
        let mut o = Out::default();
        self.walk(nodes, m, &mut o);
        o.finish()
    }

    fn element(&mut self, e: &El, parent: &Marks, out: &mut Out) {
        let name = e.name.as_str();
        if DROP.contains(&name) || name == "head" || name == "title" {
            return;
        }
        let mut m = parent.clone();
        color_marks(e, &mut m);
        if let Some(st) = e.attr("style") {
            style_marks(st, &mut m);
        }
        match name {
            "br" => out.inl.push(Inline::Break),
            "hr" => out.block(Block::Rule),
            "p" | "dt" | "caption" | "figcaption" | "summary" | "legend" => {
                out.flush();
                let bl = self.blocks(&e.children, &m);
                out.blocks.extend(bl);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if e.has_class("page-title") {
                    let t = doc::collapse_ws(&e.text());
                    if !t.is_empty() {
                        self.doc.title = Some(t);
                    }
                    return;
                }
                let level = name[1..].parse::<u8>().unwrap_or(1);
                out.flush();
                for b in self.blocks(&e.children, &m) {
                    out.blocks.push(match b {
                        Block::Paragraph(i) => Block::Heading(level, i),
                        other => other,
                    });
                }
            }
            "img" => {
                let src = e.attr("src").unwrap_or("");
                let alt = e.attr("alt").unwrap_or("");
                if e.has_class("page-cover-image") {
                    if let Inline::Image(i) = self.ctx.image(src, alt) {
                        self.doc.cover = Some(i);
                    }
                    return;
                }
                if e.has_class("icon") || e.has_class("bookmark-icon") || e.has_class("bookmark-image") || src.is_empty() {
                    return;
                }
                let inl = self.ctx.image(src, alt);
                out.inl.push(inl);
            }
            "a" => {
                if let Some(href) = e.attr("href").and_then(super::safe_link) {
                    m.link = Some(href);
                }
                self.walk(&e.children, &m, out);
            }
            "strong" | "b" => {
                m.bold = true;
                self.walk(&e.children, &m, out);
            }
            "em" | "i" | "cite" | "dfn" | "var" => {
                m.italic = true;
                self.walk(&e.children, &m, out);
            }
            "u" | "ins" => {
                m.underline = true;
                self.walk(&e.children, &m, out);
            }
            "s" | "strike" | "del" => {
                m.strike = true;
                self.walk(&e.children, &m, out);
            }
            "code" | "kbd" | "samp" | "tt" => {
                m.code = true;
                self.walk(&e.children, &m, out);
            }
            "mark" => {
                if m.highlight.is_none() && m.color == parent.color {
                    // A plain <mark> without a Notion color class.
                    m.highlight = Some(String::new());
                }
                self.walk(&e.children, &m, out);
            }
            "math" | "annotation" => {
                if let Some(t) = tex_of(e) {
                    out.inl.push(Inline::Math(t));
                }
            }
            "span" if e.has_class("notion-text-equation-token") || e.has_class("katex") || e.has_class("katex-display") => {
                if let Some(t) = tex_of(e) {
                    out.inl.push(Inline::Math(t));
                }
            }
            "span" if e.has_class("icon") => {}
            "ul" | "ol" => {
                out.flush();
                if e.has_class("toggle") {
                    // Notion wraps each toggle in a list item of its own.
                    for li in &e.children {
                        if let Node::El(x) = li {
                            let bl = self.blocks(&x.children, &m);
                            out.blocks.extend(bl);
                        }
                    }
                    return;
                }
                self.list(e, &m, out);
            }
            "blockquote" => {
                let bl = self.blocks(&e.children, &m);
                out.block(Block::Quote(bl));
            }
            "aside" => {
                let bl = self.blocks(&e.children, &m);
                out.block(Block::Callout("💡".into(), bl));
            }
            "details" => {
                let mut inner = Vec::new();
                let summary = e.elements().find(|x| x.name == "summary");
                match summary {
                    Some(s) => inner.push(Block::Paragraph(self.inline_of(&s.children, &m))),
                    None => inner.push(Block::Paragraph(Vec::new())),
                }
                let rest: Vec<&Node> = e.children.iter().filter(|n| !matches!(n, Node::El(x) if x.name == "summary")).collect();
                let mut o = Out::default();
                for n in rest {
                    self.walk(std::slice::from_ref(n), &m, &mut o);
                }
                inner.extend(o.finish());
                out.block(Block::Toggle(inner));
            }
            "pre" => {
                let code = e.text();
                out.block(Block::Code(code_language(e), code));
            }
            "table" => self.table(e, &m, out),
            "figure" => self.figure(e, &m, out),
            "div" if e.has_class("column-list") => {
                let cols: Vec<Vec<Block>> = e.elements().map(|c| self.blocks(&c.children, &m)).collect();
                out.block(Block::Columns(cols));
            }
            "div" if e.has_class("page-header-icon") => {
                let t = e.text().trim().to_string();
                if !t.is_empty() && t.chars().count() <= 8 {
                    self.doc.icon = Some(t);
                }
            }
            "article" | "body" => {
                // Notion's page font: <article class="page serif"> or "page mono".
                if e.has_class("serif") {
                    m.font = Some(FONT_SERIF.into());
                } else if e.has_class("mono") {
                    m.font = Some(FONT_MONO.into());
                }
                out.flush();
                self.walk(&e.children, &m, out);
                out.flush();
            }
            "div" | "section" | "main" | "header" | "footer" | "nav" | "html" | "center" | "form" | "fieldset" | "address" | "dl" | "dd" | "hgroup"
            | "li" | "tr" | "td" | "th" | "tbody" | "thead" | "tfoot" => {
                out.flush();
                self.walk(&e.children, &m, out);
                out.flush();
            }
            _ => self.walk(&e.children, &m, out),
        }
    }

    /// Inline content only (block structure inside is flattened).
    fn inline_of(&mut self, nodes: &[Node], m: &Marks) -> Vec<Inline> {
        let mut inl = Vec::new();
        for b in self.blocks(nodes, m) {
            if let Block::Paragraph(i) | Block::Heading(_, i) = b {
                if !inl.is_empty() {
                    inl.push(Inline::Break);
                }
                inl.extend(i);
            }
        }
        inl
    }

    fn list(&mut self, e: &El, m: &Marks, out: &mut Out) {
        let todo_list = e.has_class("to-do-list");
        let mut items: Vec<(Option<bool>, Vec<Block>)> = Vec::new();
        for n in &e.children {
            match n {
                Node::Text(t) if t.trim().is_empty() => {}
                Node::Text(t) => items.push((None, vec![Block::Paragraph(vec![doc::t(t.trim())])])),
                Node::El(li) => {
                    let task = li
                        .find(&|x| (x.name == "input" && x.attr("type").is_some_and(|t| t.eq_ignore_ascii_case("checkbox"))) || x.has_class("checkbox"))
                        .map(|x| x.attrs.iter().any(|(k, _)| k == "checked") || x.has_class("checkbox-on"));
                    let task = task.or(if todo_list { Some(false) } else { None });
                    let mut lm = m.clone();
                    color_marks(li, &mut lm);
                    items.push((task, self.blocks(&li.children, &lm)));
                }
            }
        }
        if items.is_empty() {
            return;
        }
        if items.iter().any(|(t, _)| t.is_some()) {
            out.block(Block::Tasks(items.into_iter().map(|(t, b)| (t.unwrap_or(false), b)).collect()));
        } else if e.name == "ol" {
            let start = e.attr("start").and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(1).clamp(1, 1_000_000);
            out.block(Block::Numbered(start, items.into_iter().map(|(_, b)| b).collect()));
        } else {
            out.block(Block::Bullets(items.into_iter().map(|(_, b)| b).collect()));
        }
    }

    fn table(&mut self, e: &El, m: &Marks, out: &mut Out) {
        fn rows<'e>(e: &'e El, in_head: bool, acc: &mut Vec<(&'e El, bool)>) {
            for c in e.elements() {
                match c.name.as_str() {
                    "tr" => acc.push((c, in_head)),
                    "thead" => rows(c, true, acc),
                    "tbody" | "tfoot" => rows(c, false, acc),
                    _ => {}
                }
            }
        }
        if let Some(cap) = e.elements().find(|x| x.name == "caption") {
            let bl = self.blocks(&cap.children, m);
            out.flush();
            out.blocks.extend(bl);
        }
        let mut acc = Vec::new();
        rows(e, false, &mut acc);
        let mut table = Vec::new();
        for (tr, head) in acc.into_iter().take(2_000) {
            let mut row = Vec::new();
            for td in tr.elements().filter(|x| x.name == "td" || x.name == "th").take(100) {
                let colspan = td.attr("colspan").and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(1).clamp(1, 50);
                let mut cm = m.clone();
                color_marks(td, &mut cm);
                row.push(Cell { header: head || td.name == "th", colspan, blocks: self.blocks(&td.children, &cm) });
            }
            if !row.is_empty() {
                table.push(row);
            }
        }
        if !table.is_empty() {
            out.block(Block::Table(table));
        }
    }

    fn figure(&mut self, e: &El, m: &Marks, out: &mut Out) {
        if e.has_class("equation") {
            if let Some(t) = tex_of(e) {
                out.block(Block::Math(t));
            }
            return;
        }
        if e.has_class("callout") {
            let icon = e.find(&|x| x.has_class("icon")).map(|x| x.text().trim().to_string()).filter(|t| !t.is_empty() && t.chars().count() <= 8);
            let bl = self.blocks(&e.children, m);
            out.block(Block::Callout(icon.unwrap_or_else(|| "💡".into()), bl));
            return;
        }
        if e.has_class("link-to-page") {
            if let Some(a) = e.find(&|x| x.name == "a") {
                let label = {
                    let mut s = String::new();
                    for c in &a.children {
                        match c {
                            Node::Text(t) => s.push_str(t),
                            Node::El(x) if !x.has_class("icon") => s.push_str(&x.text()),
                            _ => {}
                        }
                    }
                    doc::collapse_ws(&s)
                };
                let path = a.attr("href").and_then(|h| super::resolve_rel(&self.ctx.dir, h)).and_then(|p| self.ctx.vfs.find(&p));
                match path {
                    Some(p) => out.block(Block::PageLink(p, label)),
                    None => out.block(Block::Paragraph(vec![doc::t(label)])),
                }
            }
            return;
        }
        if e.has_class("bookmark") {
            if let Some(a) = e.find(&|x| x.name == "a") {
                let href = a.attr("href").and_then(super::safe_link);
                let title = a.find(&|x| x.has_class("bookmark-title")).map(|x| doc::collapse_ws(&x.text())).filter(|t| !t.is_empty());
                let label = title.or_else(|| href.clone()).unwrap_or_default();
                let mut lm = m.clone();
                lm.link = href;
                out.block(Block::Paragraph(vec![Inline::Text(format!("🔖 {label}"), lm)]));
            }
            return;
        }
        // A picture with its caption, or any other figure.
        let caption = e.elements().find(|x| x.name == "figcaption").map(|c| doc::collapse_ws(&c.text())).unwrap_or_default();
        let rest: Vec<&Node> = e.children.iter().filter(|n| !matches!(n, Node::El(x) if x.name == "figcaption")).collect();
        let mut o = Out::default();
        for n in rest {
            self.walk(std::slice::from_ref(n), m, &mut o);
        }
        let mut blocks = o.finish();
        let mut used = caption.is_empty();
        for b in blocks.iter_mut() {
            if !used {
                if let Block::Paragraph(i) = b {
                    if let [Inline::Image(idx)] = i.as_slice() {
                        *b = Block::Image(*idx, caption.clone());
                        used = true;
                    }
                }
            }
        }
        out.flush();
        out.blocks.extend(blocks);
        if !used {
            out.blocks.push(Block::Paragraph(vec![Inline::Text(caption, Marks { italic: true, ..m.clone() })]));
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::importers::source::Vfs;
    use crate::importers::{zip, Images};
    use serde_json::Value;

    pub fn with_ctx<T>(files: &[(&str, &[u8])], f: impl FnOnce(&mut Ctx) -> T) -> (T, Images, Vec<crate::importers::Skipped>) {
        let vfs = Vfs::from_zip_bytes(zip::build(files, false)).unwrap();
        let mut images = Images::default();
        let mut problems = Vec::new();
        let r = {
            let mut ctx = Ctx { vfs: &vfs, dir: String::new(), images: &mut images, problems: &mut problems, name: "test".into() };
            f(&mut ctx)
        };
        (r, images, problems)
    }

    pub fn json(blocks: &[Block]) -> Value {
        let (j, _) = doc::render(blocks, &|i| doc::ImageOut::Url(format!("http://clatt.localhost/img{i}")), &|p| Some(format!("id-of-{p}")));
        serde_json::from_str(&j).unwrap()
    }

    #[test]
    fn hostile_html_is_neutralised() {
        let src = r#"<p onclick="evil()">Hi <a href="javascript:alert(1)">bad</a> <a href=" https://ok.example/x ">good</a>
            <script>document.cookie</script><style>p{}</style><iframe src="https://x"></iframe>
            <img src="https://tracker.example/pixel.png" onerror="evil()"><img src="file:///C:/Windows/win.ini"><img src="\\\\server\\share\\x.png">
            <svg><script>evil()</script></svg><object data="x.swf"></object></p>"#;
        let (blocks, images, problems) = with_ctx(&[], |ctx| parse_fragment(src, ctx));
        let v = json(&blocks).to_string();
        for bad in ["evil", "javascript", "cookie", "onclick", "onerror", "iframe", "swf", "win.ini", "server"] {
            assert!(!v.contains(bad), "{bad} leaked: {v}");
        }
        assert!(v.contains("https://ok.example/x"));
        assert!(v.contains("Picture: pixel.png") && v.contains("https://tracker.example/pixel.png"), "a remote picture is a link, not fetched");
        assert!(images.list.is_empty());
        assert_eq!(problems.len(), 2, "the file: and UNC pictures are reported, not read");
    }

    #[test]
    fn deep_and_broken_html_does_not_crash() {
        let deep = "<div>".repeat(100_000) + "x" + &"</div>".repeat(10);
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(&deep, ctx));
        assert!(json(&blocks).to_string().contains('x'));
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment("<ul><li>a<li>b</ul><table><tr><td>1<td>2<tr><td>3</table><p>one<p>two", ctx));
        let v = json(&blocks);
        assert_eq!(v["content"][0]["content"].as_array().unwrap().len(), 2);
        assert_eq!(v["content"][1]["content"][0]["content"].as_array().unwrap().len(), 2);
        assert_eq!(v["content"][3]["content"][0]["text"], "two");
    }

    /// A page as Notion's HTML export writes it, with every color class.
    pub const NOTION_PAGE: &str = r#"<html><head><meta http-equiv="Content-Type" content="text/html; charset=utf-8"/><title>Project plan</title><style>/* Notion CSS */ .highlight-red{color:rgba(212,76,71,1)}</style></head><body><article id="1" class="page serif"><header><img class="page-cover-image" src="Project%20plan%200123456789abcdef0123456789abcdef/cover.png" style="object-fit:cover"/><div class="page-header-icon page-header-icon-with-cover"><span class="icon">🚀</span></div><h1 class="page-title">Project plan</h1><p class="page-description"></p></header><div class="page-body">
<p id="a" class="">Colors: <mark class="highlight-gray">gray</mark> <mark class="highlight-brown">brown</mark> <mark class="highlight-orange">orange</mark> <mark class="highlight-yellow">yellow</mark> <mark class="highlight-teal">green</mark> <mark class="highlight-green">green2</mark> <mark class="highlight-blue">blue</mark> <mark class="highlight-purple">purple</mark> <mark class="highlight-pink">pink</mark> <mark class="highlight-red">red</mark></p>
<p id="b" class="">Backgrounds: <mark class="highlight-gray_background">g</mark> <mark class="highlight-brown_background">b</mark> <mark class="highlight-orange_background">o</mark> <mark class="highlight-yellow_background">y</mark> <mark class="highlight-green_background">gr</mark> <mark class="highlight-blue_background">bl</mark> <mark class="highlight-purple_background">pu</mark> <mark class="highlight-pink_background">pi</mark> <mark class="highlight-red_background">r</mark></p>
<p id="c" class="block-color-blue_background">A whole blue block</p>
<h2 id="d" class="block-color-red">Red heading</h2>
<p><strong>bold</strong> <em>italic</em> <span style="border-bottom:0.05em solid">under</span> <del>gone</del> <code>code</code> <a href="https://notion.so">link</a> <a href="Other%20page%200123456789abcdef0123456789abcdef.html">internal</a></p>
<figure class="block-color-yellow_background callout" style="white-space:pre-wrap;display:flex" id="e"><div style="font-size:1.5em"><span class="icon">⚠️</span></div><div style="width:100%">Watch out</div></figure>
<ul id="f" class="toggle"><li><details open=""><summary>More</summary><p>Hidden part</p></details></li></ul>
<ul id="g" class="to-do-list"><li><div class="checkbox checkbox-on"></div> <span class="to-do-children-checked">Done</span></li><li><div class="checkbox checkbox-off"></div> <span class="to-do-children-unchecked">Open</span></li></ul>
<ul id="h" class="bulleted-list"><li style="list-style-type:disc">One</li></ul><ul class="bulleted-list"><li style="list-style-type:disc">Two</li></ul>
<ol type="1" class="numbered-list" start="3"><li>Third</li></ol>
<blockquote id="i" class="">Quoted</blockquote>
<pre id="j" class="code"><code class="language-JavaScript">let x = 1;
x++;</code></pre>
<figure id="k" class="equation"><style>@import url('https://cdnjs.cloudflare.com/ajax/libs/KaTeX/0.16.9/katex.min.css')</style><div class="equation-container"><span class="katex-display"><span class="katex"><span class="katex-mathml"><math><semantics><mrow><mi>E</mi></mrow><annotation encoding="application/x-tex">E = mc^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true">E=mc2</span></span></span></div></figure>
<p>Inline <span class="notion-text-equation-token" contenteditable="false"><span class="katex"><span class="katex-mathml"><math><semantics><annotation encoding="application/x-tex">a^2</annotation></semantics></math></span><span class="katex-html" aria-hidden="true">a2</span></span></span> math</p>
<figure id="l" class="bookmark source"><a href="https://example.com/article"><div class="bookmark-info"><div class="bookmark-text"><div class="bookmark-title">An article</div></div></div><img src="https://example.com/favicon.ico" class="bookmark-icon"/></a></figure>
<div id="m" class="column-list"><div id="n" style="width:50%" class="column"><p>Left</p></div><div id="o" style="width:50%" class="column"><p>Right</p></div></div>
<figure id="p" class="link-to-page"><a href="Project%20plan%200123456789abcdef0123456789abcdef/Sub%20page%200123456789abcdef0123456789abcdef.html"><span class="icon">📄</span>Sub page</a></figure>
<figure id="q" class="image"><a href="Project%20plan%200123456789abcdef0123456789abcdef/photo.png"><img style="width:400px" src="Project%20plan%200123456789abcdef0123456789abcdef/photo.png"/></a><figcaption>A photo</figcaption></figure>
<table id="r" class="simple-table"><thead class="simple-table-header"><tr><th class="simple-table-header-color simple-table-header">Name</th><th class="block-color-green_background">Score</th></tr></thead><tbody><tr><td>Ann</td><td class="block-color-red">9</td></tr></tbody></table>
<hr id="s"/>
</div></article><span class="sans" style="font-size:14px;padding-top:2em"></span></body></html>"#;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

    #[test]
    fn a_notion_html_page_keeps_its_formatting() {
        let files: &[(&str, &[u8])] = &[
            ("Project plan 0123456789abcdef0123456789abcdef/cover.png", PNG),
            ("Project plan 0123456789abcdef0123456789abcdef/photo.png", PNG),
            ("Project plan 0123456789abcdef0123456789abcdef/Sub page 0123456789abcdef0123456789abcdef.html", b"<p>sub</p>"),
        ];
        let (d, images, problems) = with_ctx(files, |ctx| parse_document(NOTION_PAGE, ctx));
        assert_eq!(d.title.as_deref(), Some("Project plan"));
        assert_eq!(d.icon.as_deref(), Some("🚀"));
        assert_eq!(d.cover, Some(0));
        assert_eq!(images.list.len(), 2);
        assert!(problems.is_empty(), "{problems:?}");
        let v = json(&d.blocks);
        let s = v.to_string();
        let c = v["content"].as_array().unwrap();

        // Every Notion color class becomes its Notaya palette color, with the exact value.
        for (i, name) in doc::NOTION_CLASS_NAMES.iter().enumerate() {
            let text = format!(r#""color":"{}","fontFamily""#, doc::TEXT_COLORS[i]);
            assert!(s.contains(&text), "text color {name} -> {}", doc::COLOR_NAMES[i]);
            let highlight = format!(r#""color":"{}"}},"type":"highlight""#, doc::HIGHLIGHTS[i]);
            assert!(s.contains(&highlight), "background {name} -> {}", doc::COLOR_NAMES[i]);
        }
        assert!(!s.contains("teal"), "an unknown Notion color is dropped, the text kept");
        // The page font (serif) is kept on the text.
        assert!(s.contains(r#""fontFamily":"Georgia, Cambria, serif""#));
        // A block background colors the text inside.
        let blue_block = c.iter().find(|b| b.to_string().contains("A whole blue block")).unwrap();
        assert!(blue_block.to_string().contains(doc::HIGHLIGHTS[5]));
        let red_heading = c.iter().find(|b| b["type"] == "heading").unwrap();
        assert!(red_heading.to_string().contains(doc::TEXT_COLORS[8]));
        // Formatting.
        for mark in ["bold", "italic", "underline", "strike", "code", "link"] {
            assert!(s.contains(&format!(r#""type":"{mark}""#)), "{mark}");
        }
        assert!(s.contains("https://notion.so"));
        assert!(!s.contains("Other%20page"), "relative links lose the link but keep the text");
        // Blocks.
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        for t in ["callout", "toggle", "taskList", "bulletList", "orderedList", "blockquote", "codeBlock", "mathBlock", "columns", "pageLink", "image", "table", "horizontalRule"] {
            assert!(types.contains(&t), "{t} missing from {types:?}");
        }
        let callout = c.iter().find(|b| b["type"] == "callout").unwrap();
        assert_eq!(callout["attrs"]["icon"], "⚠️");
        assert!(callout.to_string().contains(doc::HIGHLIGHTS[3]), "the callout background colors its text");
        let tasks = c.iter().find(|b| b["type"] == "taskList").unwrap();
        assert_eq!(tasks["content"][0]["attrs"]["checked"], true);
        assert_eq!(tasks["content"][1]["attrs"]["checked"], false);
        assert_eq!(c.iter().find(|b| b["type"] == "orderedList").unwrap()["attrs"]["start"], 3);
        assert_eq!(c.iter().find(|b| b["type"] == "codeBlock").unwrap()["attrs"]["language"], "javascript");
        assert_eq!(c.iter().find(|b| b["type"] == "mathBlock").unwrap()["attrs"]["latex"], "E = mc^2");
        assert!(s.contains(r#"{"attrs":{"latex":"a^2"},"type":"mathInline"}"#) || s.contains(r#""latex":"a^2""#));
        assert!(!s.contains("E=mc2") && !s.contains("cdnjs"), "KaTeX's drawn copy and its CSS are not imported");
        assert!(s.contains("An article") && s.contains("https://example.com/article"));
        assert!(!s.contains("favicon"), "bookmark icons are never loaded");
        let link = c.iter().find(|b| b["type"] == "pageLink").unwrap();
        assert_eq!(link["attrs"]["pageId"], "id-of-Project plan 0123456789abcdef0123456789abcdef/Sub page 0123456789abcdef0123456789abcdef.html");
        let img = c.iter().find(|b| b["type"] == "image").unwrap();
        assert_eq!(img["attrs"]["caption"], "A photo");
        let table = c.iter().find(|b| b["type"] == "table").unwrap();
        assert_eq!(table["content"][0]["content"][0]["type"], "tableHeader");
        assert!(table.to_string().contains(doc::HIGHLIGHTS[4]) && table.to_string().contains(doc::TEXT_COLORS[8]), "cell colors");
    }
}
