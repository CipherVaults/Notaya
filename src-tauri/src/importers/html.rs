//! HTML (and Notion's HTML export) to the import document model.
//!
//! The HTML is first built into a small tree (a forgiving builder: implied end tags for `p`,
//! `li`, table cells; at most 100 levels deep; at most a million tokens), then walked. The
//! walk keeps only known blocks and marks: scripts, styles, frames, forms, SVG and every
//! attribute except a few (href, src, alt, class, start, colspan, checked, style for bold,
//! italic, underline and strike, and for the text and background color of `span`, `mark` and
//! `font`) are dropped, so nothing active can reach the editor. A color is never copied: it is
//! read as a number and written as the nearest Notaya palette color (`palette`), so Notaya's own
//! exports come back with the same colors. A `details` with the class `notaya-shelf` is a shelf.
//! Links keep only http, https and mailto. Notion's classes are mapped: text and background colors
//! (`highlight-*`, `block-color-*`, written as the matching Notaya colors Ash to Cherry), callouts, toggles, to-do lists, code languages,
//! equations, bookmarks, columns, links to other pages, the page icon, the cover picture and
//! the page font (`page serif`, `page mono`).

use super::doc::{self, Block, Cell, Inline, Marks, FONT_MONO, FONT_SERIF};
use crate::export::doc::Dir;
use super::markup::{attr, Token, Tokenizer};
use super::{palette, Control, Ctx};

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
const MAX_TREE_DEPTH: usize = 100;

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

/// A Notion icon element: the emoji is its text (older exports) or its `data-emoji` attribute (newer exports leave the text empty).
fn icon_text(e: &El) -> Option<String> {
    let t = e.text().trim().to_string();
    let t = if t.is_empty() { e.attr("data-emoji").unwrap_or("").trim().to_string() } else { t };
    (!t.is_empty() && t.chars().count() <= 8).then_some(t)
}

/// The icon of a Notion callout (`<aside>` or `<figure class="callout">`): its icon element, else `data-notion-callout-icon`.
fn callout_icon(e: &El) -> Option<String> {
    e.find(&|x| x.has_class("icon"))
        .and_then(icon_text)
        .or_else(|| e.attr("data-notion-callout-icon").map(|t| t.trim().to_string()).filter(|t| !t.is_empty() && t.chars().count() <= 8))
}

/// Notion's built-in covers are pictures on notion.so, which are never fetched. The gradients and plain colors become the nearest of
/// Notaya's own covers (approximate: the pictures are not looked at); photographs and anything unknown stay without a cover.
fn notion_builtin_cover(src: &str) -> Option<String> {
    const GRADIENTS: [&str; 11] = ["dusk", "ocean", "forest", "ember", "rose", "aurora", "peach", "lagoon", "grape", "sunrise", "midnight"];
    let path = src.split(['?', '#']).next()?;
    let rest = path.strip_prefix("https://www.notion.so/images/page-cover/").or_else(|| path.strip_prefix("https://notion.so/images/page-cover/"))?;
    let stem = rest.rsplit_once('.').map_or(rest, |(s, _)| s);
    if let Some(n) = stem.strip_prefix("gradients_").and_then(|n| n.parse::<usize>().ok()).filter(|n| *n >= 1) {
        return Some(format!("gradient:{}", GRADIENTS[(n - 1) % GRADIENTS.len()]));
    }
    let color = match stem {
        "solid_red" => "#e5484d",
        "solid_yellow" => "#e9b949",
        "solid_blue" => "#3b5bdb",
        "solid_beige" => "#f6e1c3",
        _ => return None,
    };
    Some(format!("solid:{color}"))
}

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

/// Builds the tree. Never fails; stops early (with what it has) when the import is cancelled or
/// out of time.
pub fn build(src: &str, ctl: &Control) -> El {
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
        if count > MAX_TOKENS || ctl.stopped() {
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
    /// A structured Notion page, eligible for a plain database description after conversion.
    pub database_header: bool,
    /// The page's emoji (Notion's page icon).
    pub icon: Option<String>,
    /// The cover picture (Notion's page cover), as an image index.
    pub cover: Option<usize>,
    /// Notion's built-in cover as one of Notaya's own (`gradient:<name>` or `solid:#rrggbb`), when it is not a picture to fetch.
    pub cover_look: Option<String>,
}

pub fn parse_document(src: &str, ctx: &mut Ctx) -> HtmlDoc {
    parse_document_with_view(src, ctx, false)
}

/// A CSV supplies the database view; keep the surrounding document through the same sanitizer.
pub fn parse_database_document(src: &str, ctx: &mut Ctx) -> HtmlDoc {
    parse_document_with_view(src, ctx, true)
}

fn parse_document_with_view(src: &str, ctx: &mut Ctx, database_view: bool) -> HtmlDoc {
    let root = build(src, ctx.ctl);
    let mut c = Conv { ctx, doc: HtmlDoc::default(), depth: 0, cells: 0, database_view };
    c.doc.database_header = database_view && root.find(&|e| e.name == "article" && e.has_class("page")).is_some_and(|page| {
        page.elements().any(|e| e.name == "header" && e.elements().any(|h| h.name == "h1" && h.has_class("page-title")))
            && page.elements().any(|e| e.name == "div" && e.has_class("page-body"))
    });
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
    let root = build(src, ctx.ctl);
    let mut c = Conv { ctx, doc: HtmlDoc::default(), depth: 0, cells: 0, database_view: false };
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
    /// Table cells made so far on this page (the renderer keeps at most `doc::MAX_PAGE_CELLS`).
    cells: usize,
    database_view: bool,
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

/// Elements that carry text of their own (a paragraph, heading, list item, table cell, summary, caption):
/// a `dir="ltr"` on one of these is the author's statement about that text. Anywhere else (the
/// page, `div`, `ul`, `table`...) `ltr` is only a default, so it does not hold Arabic or Hebrew
/// text on the left; `rtl` is kept wherever it is written, and passes down to what is inside.
const TEXT_ELEMENTS: &[&str] = &["p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "td", "th", "summary", "caption", "figcaption", "dt", "dd", "legend"];

/// Inline elements: a direction written on one does not make the paragraph around it run that way.
const INLINE_ELEMENTS: &[&str] = &[
    "span", "bdi", "bdo", "a", "b", "i", "em", "strong", "u", "s", "strike", "del", "ins", "code", "kbd", "samp", "tt", "var", "cite", "dfn", "font",
    "mark", "sub", "sup", "small", "big", "abbr", "q", "label", "time",
];

/// Languages written right to left (the primary language subtag of `lang`).
const RTL_LANGS: &[&str] = &["ar", "he", "iw", "fa", "ur", "ps", "sd", "ug", "yi", "dv", "ckb", "ku"];

fn rtl_lang(lang: &str) -> bool {
    let primary = lang.trim().split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
    RTL_LANGS.contains(&primary.as_str())
}

/// The reading direction an element sets for its text: its `dir` attribute, else CSS `direction`
/// in its `style`, else a right-to-left `lang`. `None` when it says nothing (the direction around
/// it carries on). `ltr` and `auto` both give `Auto` except on text elements (see [`TEXT_ELEMENTS`]).
fn element_dir(e: &El) -> Option<Dir> {
    if INLINE_ELEMENTS.contains(&e.name.as_str()) {
        return None;
    }
    let from_css = || {
        e.attr("style")?.split(';').find_map(|decl| {
            let (k, v) = decl.split_once(':')?;
            (k.trim().eq_ignore_ascii_case("direction")).then(|| v.trim().to_ascii_lowercase())
        })
    };
    let word = e.attr("dir").map(|v| v.trim().to_ascii_lowercase()).or_else(from_css);
    let own = TEXT_ELEMENTS.contains(&e.name.as_str());
    match word.as_deref() {
        Some("rtl") => Some(Dir::Rtl),
        Some("ltr") if own => Some(Dir::Ltr),
        Some("ltr" | "auto") => Some(Dir::Auto),
        _ => e.attr("lang").or_else(|| e.attr("xml:lang")).filter(|l| rtl_lang(l)).map(|_| Dir::Rtl),
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

/// Text and background colors written inline on `span`, `mark` and `font` (Notaya's own HTML and
/// Markdown exports, Google Docs, Word's HTML), as the nearest Notaya palette colors (see
/// `palette`). A Notion color class on the same element wins. Black text and white or
/// transparent backgrounds are no color.
fn inline_colors(e: &El, parent: &Marks, m: &mut Marks) {
    let (mut color, highlight) = e.attr("style").map(palette::from_style).unwrap_or_default();
    if e.name == "font" && color.is_none() {
        color = e.attr("color").and_then(palette::parse).and_then(palette::text_color);
    }
    if m.color == parent.color && color.is_some() {
        m.color = color;
    }
    if m.highlight == parent.highlight && highlight.is_some() {
        m.highlight = highlight;
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
            if self.ctx.ctl.stopped() {
                return;
            }
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
        // Notion collection tables (and plain view tables in older exports) are supplied by CSV.
        // An explicitly marked simple-table is ordinary page content and is kept.
        if self.database_view && name == "table" && (e.has_class("collection-content") || e.attr("class").unwrap_or("").trim().is_empty()) {
            return;
        }
        let mut m = parent.clone();
        color_marks(e, &mut m);
        if let Some(d) = element_dir(e) {
            m.dir = d;
        }
        if let Some(st) = e.attr("style") {
            style_marks(st, &mut m);
        }
        if matches!(name, "span" | "mark" | "font") {
            inline_colors(e, parent, &mut m);
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
                    if let Some(look) = notion_builtin_cover(src) {
                        self.doc.cover_look = Some(look);
                    } else if let Inline::Image(i) = self.ctx.image(src, alt) {
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
                if let Some(href) = e.attr("href").and_then(|h| super::safe_link(h).or_else(|| self.ctx.page_href(h))) {
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
                out.block(Block::Callout(callout_icon(e).unwrap_or_else(|| "💡".into()), bl));
            }
            "details" => {
                let mut inner = Vec::new();
                let summary = e.elements().find(|x| x.name == "summary");
                match summary {
                    Some(s) => {
                        let mut sm = m.clone();
                        if let Some(d) = element_dir(s) {
                            sm.dir = d;
                        }
                        inner.push(Block::Paragraph(self.inline_of(&s.children, &sm)))
                    }
                    None => inner.push(Block::Paragraph(Vec::new())),
                }
                let rest: Vec<&Node> = e.children.iter().filter(|n| !matches!(n, Node::El(x) if x.name == "summary")).collect();
                let mut o = Out::default();
                for n in rest {
                    self.walk(std::slice::from_ref(n), &m, &mut o);
                }
                inner.extend(o.finish());
                // Notaya's own exports mark a shelf with this class (the editor's `shelf` attribute).
                if e.has_class("notaya-shelf") {
                    out.block(Block::Shelf(inner));
                } else {
                    out.block(Block::Toggle(inner));
                }
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
                // Older exports write the emoji as the text of the icon; newer ones write `<span class="icon" data-emoji="🏠"></span>`.
                if let Some(t) = icon_text(e).or_else(|| e.find(&|x| x.has_class("icon")).and_then(icon_text)) {
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
                    if let Some(d) = element_dir(li) {
                        lm.dir = d;
                    }
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
                if self.cells >= doc::MAX_PAGE_CELLS {
                    if self.cells == doc::MAX_PAGE_CELLS {
                        self.cells += 1;
                        self.ctx.problem("Tables", "Some table cells beyond the limit for one page were left out.");
                    }
                    break;
                }
                self.cells += 1;
                let colspan = td.attr("colspan").and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(1).clamp(1, doc::MAX_TABLE_COLS as u32);
                let mut cm = m.clone();
                color_marks(td, &mut cm);
                if let Some(d) = element_dir(td) {
                    cm.dir = d;
                }
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
            let bl = self.blocks(&e.children, m);
            out.block(Block::Callout(callout_icon(e).unwrap_or_else(|| "💡".into()), bl));
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
    use crate::importers::{zip, Control, Images};
    use serde_json::Value;

    pub fn with_ctx<T>(files: &[(&str, &[u8])], f: impl FnOnce(&mut Ctx) -> T) -> (T, Images, Vec<crate::importers::Skipped>) {
        let vfs = Vfs::from_zip_bytes(zip::build(files, false)).unwrap();
        let mut images = Images::default();
        let mut problems = Vec::new();
        let r = {
            let ctl = Control::standard();
            let mut ctx = Ctx { vfs: &vfs, ctl: &ctl, dir: String::new(), images: &mut images, problems: &mut problems, name: "test".into() };
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
        for bad in ["evil", "javascript", "cookie", "onclick", "onerror", "iframe", "swf", "file:", "C:", "server"] {
            assert!(!v.contains(bad), "{bad} leaked: {v}");
        }
        assert!(v.contains("https://ok.example/x"));
        assert!(v.contains("Picture: pixel.png") && v.contains("https://tracker.example/pixel.png"), "a remote picture is a link, not fetched");
        assert!(images.list.is_empty());
        assert_eq!(problems.len(), 2, "the file: and UNC pictures are reported, not read");
    }

    #[test]
    fn a_notion_mermaid_code_block_becomes_a_diagram_block() {
        let src = r#"<pre class="code"><code class="language-Mermaid">graph TD
  A --&gt; B</code></pre><pre><code class="language-js">x</code></pre>"#;
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(src, ctx));
        let v = json(&blocks);
        let c = v["content"].as_array().unwrap();
        assert_eq!(c[0]["type"], "mermaidBlock", "{v}");
        assert_eq!(c[0]["attrs"]["code"], "graph TD\n  A --> B");
        assert_eq!(c[1]["type"], "codeBlock");
    }

    #[test]
    fn notion_page_fonts_map_to_serif_mono_or_the_default() {
        for (class, expect) in [("page serif", Some("Georgia, Cambria, serif")), ("page mono", Some("\"JetBrains Mono Variable\", Consolas, monospace")), ("page sans", None), ("page", None)] {
            let src = format!("<html><body><article class=\"{class}\"><p>text</p></article></body></html>");
            let (d, _, _) = with_ctx(&[], |ctx| parse_document(&src, ctx));
            let v = json(&d.blocks);
            let font = &v["content"][0]["content"][0]["marks"][0]["attrs"]["fontFamily"];
            match expect {
                Some(f) => assert_eq!(font, f, "{class}"),
                None => assert!(v["content"][0]["content"][0].get("marks").is_none(), "{class}: the default font sets nothing: {v}"),
            }
        }
    }

    #[test]
    fn deep_and_broken_html_does_not_crash() {
        let deep = "<div>".repeat(100_000) + "x" + &"</div>".repeat(10);
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(&deep, ctx));
        let _ = json(&blocks); // very deep text may be dropped, but nothing crashes or hangs
        let nested = "<div>".repeat(50) + "x" + &"</div>".repeat(50);
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(&nested, ctx));
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

    /// Newer Notion exports write the page icon and the callout icon as `data-emoji` attributes with no text, and the cover as a built-in picture on notion.so.
    #[test]
    fn notion_icons_in_data_emoji_attributes_and_builtin_covers() {
        let page = |cover: &str| {
            format!(
                r#"<html><body><article class="page sans" data-notion-page-icon="🏠"><header><img class="page-cover-image" src="{cover}"/><div class="page-header-icon page-header-icon-with-cover"><span class="icon" data-emoji="🏠"></span></div><h1 class="page-title">Hubs</h1></header><div class="page-body"><aside class="block-color-yellow_background callout" data-notion-callout-icon="⚠️"><div><span class="icon" data-emoji="⚠️"></span></div><div><p>Watch out</p></div></aside><figure class="callout"><div><span class="icon" data-emoji="🔥"></span></div><div><p>Hot</p></div></figure></div></article></body></html>"#
            )
        };
        let (d, images, problems) = with_ctx(&[], |ctx| parse_document(&page("https://www.notion.so/images/page-cover/gradients_8.png"), ctx));
        assert_eq!(d.title.as_deref(), Some("Hubs"));
        assert_eq!(d.icon.as_deref(), Some("🏠"), "the page icon comes from data-emoji");
        assert!(d.cover.is_none() && images.list.is_empty(), "a cover on notion.so is never fetched");
        assert_eq!(d.cover_look.as_deref(), Some("gradient:lagoon"), "a built-in gradient becomes one of Notaya's own");
        assert!(problems.is_empty(), "{problems:?}");
        let v = json(&d.blocks);
        let icons: Vec<&str> = v["content"].as_array().unwrap().iter().filter(|b| b["type"] == "callout").map(|b| b["attrs"]["icon"].as_str().unwrap()).collect();
        assert_eq!(icons, ["⚠️", "🔥"], "callout icons come from data-emoji, not the default");
        for (src, want) in [
            ("https://www.notion.so/images/page-cover/gradients_2.png", Some("gradient:ocean")),
            ("https://www.notion.so/images/page-cover/gradients_12.png", Some("gradient:dusk")),
            ("https://www.notion.so/images/page-cover/solid_red.png", Some("solid:#e5484d")),
            ("https://www.notion.so/images/page-cover/nasa_earth.jpg", None),
            ("https://www.notion.so/images/page-cover/gradients_x.png", None),
            ("https://www.notion.so/images/page-cover/gradients_0.png", None),
            ("https://evil.example/images/page-cover/gradients_2.png", None),
        ] {
            let (d, images, _) = with_ctx(&[], |ctx| parse_document(&page(src), ctx));
            assert_eq!(d.cover_look.as_deref(), want, "{src}");
            assert!(d.cover.is_none() && images.list.is_empty(), "{src}");
        }
    }

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
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert_eq!(problems[0].item, "Link to Other page 0123456789abcdef0123456789abcdef.html (test)");
        assert_eq!(problems[0].reason, "The linked page file was not found in this import.");
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
        assert!(!s.contains("teal"), "class names are never written to the page");
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
        assert!(s.contains("internal"), "the missing link label stays");
        assert!(!s.contains("Other%20page") && !s.contains("notaya-src:"), "the missing link has no address");
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

    /// The text color and highlight of the text node that holds the word `word`.
    pub fn colors_of(v: &Value, word: &str) -> (Option<String>, Option<String>) {
        fn find<'v>(v: &'v Value, word: &str) -> Option<&'v Value> {
            if v["type"] == "text" && v["text"].as_str().is_some_and(|t| t.split_whitespace().any(|w| w == word)) {
                return Some(v);
            }
            v["content"].as_array()?.iter().find_map(|c| find(c, word))
        }
        let node = find(v, word).unwrap_or_else(|| panic!("no text {word:?} in {v}"));
        let mut out = (None, None);
        for m in node["marks"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            if m["type"] == "textStyle" {
                out.0 = m["attrs"]["color"].as_str().map(str::to_string);
            }
            if m["type"] == "highlight" {
                out.1 = Some(m["attrs"]["color"].as_str().unwrap_or("").to_string());
            }
        }
        out
    }

    #[test]
    fn inline_colors_become_notaya_palette_colors() {
        use crate::importers::palette::{HIGHLIGHTS, TEXT_COLORS};
        // As Notaya's HTML export writes them: the text color as is, the highlight blended onto white.
        let mut src = String::from("<p>");
        for (i, (_, v)) in TEXT_COLORS.iter().enumerate() {
            src.push_str(&format!("<span style=\"color:{};font-size:12pt;\">tc{i}</span> ", v.to_uppercase()));
        }
        for (i, (_, v)) in HIGHLIGHTS.iter().enumerate() {
            let c = crate::export::doc::Rgb::parse(v).unwrap();
            src.push_str(&format!("<mark style=\"background:#{}\">hl{i}</mark> ", c.hex()));
        }
        // Other writers: Google Docs (black on transparent is no color), <font color>, a Notion class
        // (it wins over a style on the same element), and values that are not colors.
        src.push_str(concat!(
            r#"<span style="color:#000000;background-color:transparent">gdocs</span> "#,
            r##"<font color="#d44c47">font</font> "##,
            r#"<mark class="highlight-red_background" style="background:#EEE5C4">notion</mark> "#,
            r#"<span style="color:red;background:url(https://evil.example/x.png)">bad</span> "#,
            r#"<mark>plain</mark> <span style="background-color:rgba(203, 145, 47, 0.30)">rgba</span></p>"#
        ));
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(&src, ctx));
        let v = json(&blocks);
        for (i, (name, c)) in TEXT_COLORS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("tc{i}")).0.as_deref(), Some(*c), "text color {name}");
        }
        for (i, (name, h)) in HIGHLIGHTS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("hl{i}")).1.as_deref(), Some(*h), "highlight {name}");
        }
        assert_eq!(colors_of(&v, "gdocs"), (None, None));
        assert_eq!(colors_of(&v, "font").0.as_deref(), Some("#d44c47"));
        assert_eq!(colors_of(&v, "notion").1.as_deref(), Some(doc::HIGHLIGHTS[8]), "the Notion class wins");
        assert_eq!(colors_of(&v, "bad"), (None, None));
        assert!(!v.to_string().contains("evil"));
        assert_eq!(colors_of(&v, "plain").1.as_deref(), Some(""), "a plain <mark> is the default highlight");
        assert_eq!(colors_of(&v, "rgba").1.as_deref(), Some(HIGHLIGHTS[2].1));
    }

    /// (first text, stored `dir`) of every paragraph and heading in an editor document, in order.
    pub fn dirs_of(v: &Value) -> Vec<(String, Option<String>)> {
        fn go(v: &Value, out: &mut Vec<(String, Option<String>)>) {
            if v["type"] == "paragraph" || v["type"] == "heading" {
                let text = v["content"][0]["text"].as_str().unwrap_or("").to_string();
                out.push((text, v["attrs"]["dir"].as_str().map(str::to_string)));
            }
            for c in v["content"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
                go(c, out);
            }
        }
        let mut out = Vec::new();
        go(v, &mut out);
        out
    }

    #[test]
    fn html_directions_are_kept_where_the_text_would_not_give_them() {
        let src = r#"<div dir="rtl"><p>English in rtl</p><p dir="auto">auto</p><p dir="ltr">ltr back</p></div>
            <p lang="ar-EG">123</p><p style="color:red; direction: rtl">456</p><p dir="rtl">مرحبا</p>
            <div dir="ltr"><p>مرحبا من div</p></div><p dir="ltr">مرحبا ltr</p>
            <ul dir="rtl"><li>one</li><li dir="auto">two</li></ul>
            <table dir="rtl"><tr><td>cell</td></tr></table>
            <p>Plain <span dir="rtl">inline</span></p><h2 dir="rtl">Heading</h2>"#;
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(src, ctx));
        let d: Vec<(String, Option<String>)> = dirs_of(&json(&blocks));
        let got = |text: &str| d.iter().find(|(t, _)| t == text).unwrap_or_else(|| panic!("{text} in {d:?}")).1.clone();
        // rtl is inherited from an ancestor, and kept when the text is English.
        assert_eq!(got("English in rtl").as_deref(), Some("rtl"));
        // auto and ltr (that match the text) store nothing: the editor decides by the text.
        assert_eq!(got("auto"), None);
        assert_eq!(got("ltr back"), None);
        // lang and CSS direction count, and they matter when the text has no letters.
        assert_eq!(got("123").as_deref(), Some("rtl"));
        assert_eq!(got("456").as_deref(), Some("rtl"));
        // rtl on Arabic text is what the text gives anyway.
        assert_eq!(got("مرحبا"), None);
        // ltr on a wrapper is only a default: Arabic text inside still runs right-to-left (by the text).
        assert_eq!(got("مرحبا من div"), None);
        // ltr on the paragraph itself is the author's choice and is kept.
        assert_eq!(got("مرحبا ltr").as_deref(), Some("ltr"));
        // A list and a table under dir="rtl"; an item that says auto goes back to the text.
        assert_eq!(got("one").as_deref(), Some("rtl"));
        assert_eq!(got("two"), None);
        assert_eq!(got("cell").as_deref(), Some("rtl"));
        // An inline dir does not turn the paragraph around it.
        assert_eq!(got("Plain inline"), None);
        assert_eq!(got("Heading").as_deref(), Some("rtl"));
    }

    #[test]
    fn html_without_any_direction_stays_automatic() {
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment("<h1>مرحبا</h1><p>مرحبا بالعالم</p><p>Hello</p><ul><li>بند</li></ul>", ctx));
        let v = json(&blocks);
        assert!(dirs_of(&v).iter().all(|(_, d)| d.is_none()), "{v}");
        assert!(!v.to_string().contains("\"dir\""), "no dir attribute is written");
    }

    #[test]
    fn a_shelf_comes_back_as_a_shelf() {
        let src = "<details open class=\"notaya-shelf\"><summary>Shelf title</summary><p>Inside</p></details><details open><summary>Toggle</summary><p>x</p></details><details class=\"shelf\"><summary>Other</summary></details>";
        let (blocks, _, _) = with_ctx(&[], |ctx| parse_fragment(src, ctx));
        let v = json(&blocks);
        let c = v["content"].as_array().unwrap();
        assert_eq!(c.len(), 3);
        assert_eq!(c[0]["type"], "toggle");
        let bid = c[0]["attrs"]["bid"].as_str().expect("the shelf has a block id");
        assert!(bid.len() == 16 && bid.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(c[0]["attrs"], serde_json::json!({ "bid": bid, "open": true, "shelf": true }), "the editor's own attribute");
        assert_eq!(c[0]["content"][0]["content"][0]["text"], "Shelf title");
        assert_eq!(c[0]["content"][1]["content"][0]["text"], "Inside");
        assert!(c[1]["attrs"].get("shelf").is_none() && c[2]["attrs"].get("shelf").is_none(), "only Notaya's class makes a shelf");
    }

    #[test]
    fn hostile_tables_and_many_tags_stay_small_and_fast() {
        let _heavy = crate::importers::heavy();
        let start = std::time::Instant::now();
        // The reviewer's probe: 100 cells of colspan 50 in each of 2,000 rows.
        let mut h = String::from("<table>");
        for _ in 0..2_000 {
            h.push_str("<tr>");
            for _ in 0..100 {
                h.push_str("<td colspan=50>x</td>");
            }
            h.push_str("</tr>");
        }
        let (d, _, problems) = with_ctx(&[], |ctx| parse_document(&h, ctx));
        let r = doc::render_limited(&d.blocks, &|_| doc::ImageOut::Text("p".into()), &|_| None, doc::MAX_RENDER_BYTES).unwrap();
        let v: Value = serde_json::from_str(&r.content).unwrap();
        let cells = v.to_string().matches("\"type\":\"tableCell\"").count();
        assert!(cells <= doc::MAX_TABLE_CELLS, "{cells}");
        assert!(r.cut_rows > 0);
        assert!(problems.iter().any(|p| p.item.starts_with("Tables")), "{problems:?}");
        // 40,000 empty <title> and <script> elements (the tokenizer lower-cased the whole rest each time).
        for unit in ["<title></title>", "<script></script>", "<style></style>", "<textarea></textarea>"] {
            let h = unit.repeat(40_000) + "<p>end</p>";
            let (d, _, _) = with_ctx(&[], |ctx| parse_document(&h, ctx));
            assert!(!d.blocks.is_empty(), "{unit}");
        }
        // Deep nesting with no boundary elements, then many block starts.
        let h = "<span>".repeat(150) + &"<div>".repeat(100_000);
        let _ = with_ctx(&[], |ctx| parse_document(&h, ctx));
        assert!(start.elapsed().as_secs() < 180, "{:?}", start.elapsed());
    }

    #[test]
    fn a_stopped_import_stops_the_html_converter() {
        let h = "<p>a</p>".repeat(100_000);
        let vfs = Vfs::from_zip_bytes(zip::build(&[("a.md", b"x")], false)).unwrap();
        let ctl = Control::expired();
        let mut images = Images::default();
        let mut problems = Vec::new();
        let mut ctx = Ctx { vfs: &vfs, ctl: &ctl, dir: String::new(), images: &mut images, problems: &mut problems, name: "t".into() };
        let d = parse_document(&h, &mut ctx);
        assert!(d.blocks.len() < 100_000, "it stopped early: {}", d.blocks.len());
    }
}
