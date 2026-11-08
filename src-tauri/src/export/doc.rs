//! A page's TipTap (ProseMirror) JSON turned into a small document model that the four
//! writers (HTML, Markdown, Word, PDF) share. Every node type the editor can make has a place here
//! (see `src/features/notes/editor/`); a node type this code does not know keeps its text, so
//! nothing a person wrote is dropped silently.

use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

impl Align {
    fn parse(v: Option<&Value>) -> Align {
        match v.and_then(Value::as_str) {
            Some("center") => Align::Center,
            Some("right") => Align::Right,
            Some("justify") => Align::Justify,
            _ => Align::Left,
        }
    }
}

/// The reading direction of a text block. `Auto` (the default, and what every page saved before
/// this was added has) means "decide from the text": see [`first_strong`]. `Ltr` and `Rtl` are a
/// person's explicit choice, stored on the block as `dir` in the page JSON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dir {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

impl Dir {
    /// From the page JSON's `dir` attribute: anything but "ltr" and "rtl" is automatic.
    pub fn parse(v: Option<&Value>) -> Dir {
        match v.and_then(Value::as_str) {
            Some("ltr") => Dir::Ltr,
            Some("rtl") => Dir::Rtl,
            _ => Dir::Auto,
        }
    }

    /// The value of an HTML `dir` attribute.
    pub fn html(self) -> &'static str {
        match self {
            Dir::Auto => "auto",
            Dir::Ltr => "ltr",
            Dir::Rtl => "rtl",
        }
    }
}

/// True for the letters of the right-to-left scripts: Hebrew, Arabic (and Persian, Urdu), Syriac,
/// Thaana, N'Ko, Samaritan, Mandaic and their presentation forms. Digits and punctuation of those
/// scripts are not letters, so they are not strong either.
fn is_rtl_letter(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF | 0x10800..=0x10FFF | 0x1E800..=0x1EFFF) && c.is_alphabetic()
}

/// The direction of the first strong character of `text`: a letter (right-to-left for the scripts
/// above, left-to-right for every other script) or an explicit direction mark (U+200F, U+200E).
/// `None` when there is none (digits, punctuation, emoji, nothing). This is the "automatic" rule;
/// the editor uses the same one (`src/features/notes/editor/direction.ts`).
pub fn first_strong(text: &str) -> Option<Dir> {
    for c in text.chars() {
        match c {
            '\u{200F}' => return Some(Dir::Rtl),
            '\u{200E}' => return Some(Dir::Ltr),
            c if is_rtl_letter(c) => return Some(Dir::Rtl),
            c if c.is_alphabetic() => return Some(Dir::Ltr),
            _ => {}
        }
    }
    None
}

/// The explicit direction a text block carries. It is kept on the text runs' [`Marks`] (so the
/// shape of [`Block`] does not change): the first run that has one decides.
pub fn explicit_dir(content: &[Inline]) -> Dir {
    content
        .iter()
        .find_map(|i| match i {
            Inline::Text(_, m) if m.dir != Dir::Auto => Some(m.dir),
            _ => None,
        })
        .unwrap_or_default()
}

/// The direction a text block is shown in: its explicit one, else by its text (left-to-right when
/// the text has no strong character).
pub fn content_dir(content: &[Inline]) -> Dir {
    match explicit_dir(content) {
        Dir::Auto => first_strong(&inline_text(content)).unwrap_or(Dir::Ltr),
        d => d,
    }
}

/// The direction of the first text block inside `blocks` (what a list, quote, box or table takes
/// as its own), or `None` when no text block has an explicit direction or a strong character.
pub fn blocks_dir(blocks: &[Block]) -> Option<Dir> {
    for b in blocks {
        let found = match b {
            Block::Paragraph { content, .. } | Block::Heading { content, .. } => match explicit_dir(content) {
                Dir::Auto => first_strong(&inline_text(content)),
                d => Some(d),
            },
            Block::List { items, .. } => items.iter().find_map(|i| blocks_dir(&i.blocks)),
            Block::Quote(x) | Block::Toggle(x) | Block::Shelf(x) | Block::Callout { blocks: x, .. } => blocks_dir(x),
            Block::Columns(cols) => cols.iter().find_map(|c| blocks_dir(c)),
            Block::Table(rows) => rows.iter().flatten().find_map(|c| blocks_dir(&c.blocks)),
            Block::Database { table: Some(t), .. } => first_strong(&t.title).or_else(|| t.rows.iter().flatten().find_map(|c| blocks_dir(&c.blocks))),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// A color as red, green and blue (0 to 255). Translucent colors are blended onto white,
/// because a page on paper (or in Word) has a white background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn parse(s: &str) -> Option<Rgb> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix('#') {
            let h = |i: usize, n: usize| u8::from_str_radix(hex.get(i..i + n)?, 16).ok();
            return match hex.len() {
                3 => Some(Rgb(h(0, 1)? * 17, h(1, 1)? * 17, h(2, 1)? * 17)),
                6 | 8 => Some(Rgb(h(0, 2)?, h(2, 2)?, h(4, 2)?)),
                _ => None,
            };
        }
        let inner = s.strip_prefix("rgba(").or_else(|| s.strip_prefix("rgb("))?.strip_suffix(')')?;
        let parts: Vec<f32> = inner.split(',').map(|p| p.trim().parse::<f32>()).collect::<Result<_, _>>().ok()?;
        if parts.len() < 3 {
            return None;
        }
        let a = parts.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        let blend = |c: f32| (255.0 - a * (255.0 - c.clamp(0.0, 255.0))).round() as u8;
        Some(Rgb(blend(parts[0]), blend(parts[1]), blend(parts[2])))
    }

    pub fn hex(self) -> String {
        format!("{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

/// The highlight color when the highlight mark has none (the editor's default yellow).
pub const DEFAULT_HIGHLIGHT: Rgb = Rgb(0xFE, 0xF0, 0x8A);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
    pub link: Option<String>,
    pub color: Option<Rgb>,
    pub highlight: Option<Rgb>,
    /// Font size in points (from the editor's "16px" style), when set.
    pub size: Option<f32>,
    /// A link to another page (`notaya:page/<id>`) whose page is part of the same export, as the
    /// exporter resolved it; `link` is then `None`. A link to a page outside the export has
    /// neither, so its text is written as plain text.
    pub page: Option<PageRef>,
    /// The text block's explicit direction, repeated on each of its runs (see [`explicit_dir`]).
    /// `Auto` for every page saved before the direction setting existed.
    pub dir: Dir,
}

/// Where a link to another page of the same export goes. Only the exporter makes these, from its
/// own table of page files: nothing from the page content is copied into them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PageRef {
    /// Another file of the same ZIP: its path relative to the linking file, every segment
    /// percent-encoded (see [`url_segment`]) and `..` to go up, so it can be neither a scheme
    /// (`javascript:`, `file:`) nor an absolute path.
    File(String),
    /// The n-th page (from 1) of the same document; see [`page_anchor`].
    Anchor(usize),
}

impl PageRef {
    /// The address to write in an HTML or Markdown link.
    pub fn href(&self) -> String {
        match self {
            PageRef::File(p) => p.clone(),
            PageRef::Anchor(n) => format!("#{}", page_anchor(*n)),
        }
    }
}

/// The name of the place where the n-th page of a document starts: an HTML `id` and a Word
/// bookmark (letters, digits and `_` only, so it is valid for both).
pub fn page_anchor(n: usize) -> String {
    format!("notaya_page_{n}")
}

/// Percent-encodes one path segment of a relative link: everything except `A-Z a-z 0-9 - . _ ~`.
pub fn url_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text(String, Marks),
    Break,
    Math(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListKind {
    Bullet,
    Ordered(u32),
    Task,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    /// For task lists: ticked or not.
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub header: bool,
    pub colspan: usize,
    pub rowspan: usize,
    /// Column widths in pixels set by dragging in the editor (empty = not set).
    pub widths: Vec<f32>,
    pub blocks: Vec<Block>,
}

/// A database's saved view, resolved in the Rust core before a writer sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct DatabaseTable {
    pub title: String,
    /// Header followed by at most 1,000 data rows, in the saved view's order.
    pub rows: Vec<Vec<Cell>>,
    pub more: usize,
}

/// Reuses the ordinary text and table renderers, including their right-to-left handling.
pub fn database_blocks(table: Option<&DatabaseTable>) -> Vec<Block> {
    let para = |text, bold| Block::Paragraph { align: Align::Left, content: vec![Inline::Text(text, Marks { bold, ..Marks::default() })] };
    match table {
        Some(t) => {
            let mut blocks = vec![para(t.title.clone(), true), Block::Table(t.rows.clone())];
            if t.more > 0 {
                blocks.push(para(format!("and {} more rows", t.more), false));
            }
            blocks
        }
        None => vec![para("(database not available)".into(), false)],
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    /// The attachment id inside the vault, when the picture is one of its own.
    pub attachment: Option<String>,
    pub caption: String,
    /// Width as a percentage of the text column, when set.
    pub width_pct: Option<f32>,
    pub align: Align,
    /// The picture was cropped to this shape (width / height), when set (V1.5.1). HTML keeps the shape; Word and
    /// PDF show the whole picture.
    pub ratio: Option<f32>,
    /// The text wraps around it, at the side `align` says (V1.5.1). HTML keeps it; Word, PDF and Markdown put
    /// the picture in the text. A picture placed freely on the page (`free`, `x`, `y`, `layer`) is exported in the
    /// text, at the block the editor kept it next to, so reading order follows its height.
    pub float: bool,
}

/// "16:9" as 16 / 9. Anything else (no ratio, junk, zero) is none.
fn parse_ratio(v: Option<&Value>) -> Option<f32> {
    let (w, h) = v?.as_str()?.split_once(':')?;
    let (w, h): (f32, f32) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 && w <= 100.0 && h <= 100.0).then_some(w / h)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading { level: u8, align: Align, content: Vec<Inline> },
    Paragraph { align: Align, content: Vec<Inline> },
    List { kind: ListKind, items: Vec<ListItem> },
    Quote(Vec<Block>),
    Callout { icon: String, blocks: Vec<Block> },
    /// A collapsible block; the first block is the line that is always shown.
    Toggle(Vec<Block>),
    /// A shelf: a toggle the editor draws as a shaded box (its `shelf` attribute is set).
    Shelf(Vec<Block>),
    Columns(Vec<Vec<Block>>),
    Code { language: String, text: String },
    Math(String),
    Image(Image),
    Table(Vec<Vec<Cell>>),
    /// The page stores only the database and view ids. `table` is filled by the exporter;
    /// absent, binned or unresolved databases keep the unavailable placeholder.
    Database { id: String, view: String, table: Option<DatabaseTable> },
    Rule,
    /// A link to another page (the title is looked up when the page is exported). `target` is
    /// set when that page is part of the same export.
    PageLink { title: String, target: Option<PageRef> },
}

/// Parses a page's stored content. Empty or unreadable content gives an empty document. Links
/// to other pages are left as plain text (see [`parse_with`]).
pub fn parse(content: &str, page_title: &dyn Fn(&str) -> Option<String>) -> Vec<Block> {
    parse_with(content, page_title, &|_| None)
}

#[cfg(test)]
#[test]
fn block_ids_are_metadata_for_every_export_node() {
    let original = tests::every_node_json();
    let mut with_ids: Value = serde_json::from_str(&original).unwrap();
    crate::blocks::ensure_ids(&mut with_ids);
    assert_eq!(parse(&original, &tests::titles), parse(&with_ids.to_string(), &tests::titles));
}

/// Like [`parse`]; `page_ref` says, by page id, where a link to another page goes when that page
/// is part of the same export, so the writers can make it a real link.
pub fn parse_with(content: &str, page_title: &dyn Fn(&str) -> Option<String>, page_ref: &dyn Fn(&str) -> Option<PageRef>) -> Vec<Block> {
    if content.trim().is_empty() {
        return Vec::new();
    }
    let p = Pages { title: page_title, link: page_ref };
    match serde_json::from_str::<Value>(content) {
        Ok(v) => blocks(children(&v), &p),
        Err(_) => vec![Block::Paragraph { align: Align::Left, content: vec![Inline::Text(content.to_string(), Marks::default())] }],
    }
}

/// What the parser asks the exporter about other pages: their titles, and where links to them go.
struct Pages<'a> {
    title: &'a dyn Fn(&str) -> Option<String>,
    link: &'a dyn Fn(&str) -> Option<PageRef>,
}

/// The page id of an internal link (`notaya:page/<id>` or `notaya:page/<id>#<anchor>`), the
/// form the editor writes (see `internalLinks.ts`).
pub fn internal_page_id(href: &str) -> Option<&str> {
    let rest = href.trim().strip_prefix("notaya:page/")?;
    let id = rest.split('#').next().unwrap_or("");
    (!id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')).then_some(id)
}

fn children(v: &Value) -> &[Value] {
    v.get("content").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn attr<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
    v.get("attrs").and_then(|a| a.get(name)).filter(|x| !x.is_null())
}

fn attr_str(v: &Value, name: &str) -> String {
    attr(v, name).and_then(Value::as_str).unwrap_or("").to_string()
}

fn node_type(v: &Value) -> &str {
    v.get("type").and_then(Value::as_str).unwrap_or("")
}

fn blocks(nodes: &[Value], p: &Pages) -> Vec<Block> {
    let mut out = Vec::new();
    for n in nodes {
        block(n, p, &mut out);
    }
    out
}

fn block(n: &Value, p: &Pages, out: &mut Vec<Block>) {
    let kids = children(n);
    match node_type(n) {
        "paragraph" => out.push(Block::Paragraph { align: Align::parse(attr(n, "textAlign")), content: dir_inlines(n, kids, p) }),
        "heading" => {
            let level = attr(n, "level").and_then(Value::as_u64).unwrap_or(1).clamp(1, 6) as u8;
            out.push(Block::Heading { level, align: Align::parse(attr(n, "textAlign")), content: dir_inlines(n, kids, p) });
        }
        "bulletList" => out.push(Block::List { kind: ListKind::Bullet, items: list_items(kids, p) }),
        "orderedList" => {
            let start = attr(n, "start").and_then(Value::as_u64).unwrap_or(1).min(1_000_000) as u32;
            out.push(Block::List { kind: ListKind::Ordered(start), items: list_items(kids, p) });
        }
        "taskList" => out.push(Block::List { kind: ListKind::Task, items: list_items(kids, p) }),
        "blockquote" => out.push(Block::Quote(blocks(kids, p))),
        "callout" => out.push(Block::Callout { icon: attr_str(n, "icon"), blocks: blocks(kids, p) }),
        "toggle" if attr(n, "shelf").and_then(Value::as_bool) == Some(true) => out.push(Block::Shelf(blocks(kids, p))),
        "toggle" => out.push(Block::Toggle(blocks(kids, p))),
        "columns" => out.push(Block::Columns(kids.iter().map(|c| blocks(children(c), p)).collect())),
        // A column outside its group (not made by the editor): just its blocks.
        "column" => out.extend(blocks(kids, p)),
        "codeBlock" => out.push(Block::Code { language: attr_str(n, "language"), text: plain_text(n) }),
        "mathBlock" => out.push(Block::Math(attr_str(n, "latex"))),
        // A Mermaid diagram is drawn by the editor only; exports carry its code, as a code block with a caption.
        "mermaidBlock" => {
            let code = attr_str(n, "code");
            if !code.trim().is_empty() {
                let caption = Marks { italic: true, ..Marks::default() };
                out.push(Block::Paragraph { align: Align::Left, content: vec![Inline::Text("Mermaid diagram (code)".into(), caption)] });
                out.push(Block::Code { language: "mermaid".into(), text: code });
            }
        }
        "image" => out.push(Block::Image(Image {
            attachment: attachment_id(&attr_str(n, "src")),
            caption: attr_str(n, "caption"),
            width_pct: attr(n, "width").and_then(Value::as_f64).filter(|w| *w > 0.0).map(|w| w.min(100.0) as f32),
            align: Align::parse(attr(n, "align")),
            ratio: parse_ratio(attr(n, "ratio")),
            float: attr(n, "float").and_then(Value::as_bool) == Some(true),
        })),
        "table" => out.push(Block::Table(kids.iter().map(|row| children(row).iter().map(|c| cell(c, p)).collect()).collect())),
        "databaseBlock" => out.push(Block::Database { id: attr_str(n, "databaseId"), view: attr_str(n, "viewId"), table: None }),
        "horizontalRule" => out.push(Block::Rule),
        "pageLink" => {
            let id = attr_str(n, "pageId");
            match (p.title)(&id) {
                Some(title) => out.push(Block::PageLink { title, target: (p.link)(&id) }),
                None => out.push(Block::PageLink { title: "Page not found".into(), target: None }),
            }
        }
        // Inline content where a block is expected (not made by the editor): wrap it.
        "text" | "hardBreak" | "mathInline" => {
            out.push(Block::Paragraph { align: Align::Left, content: inlines(std::slice::from_ref(n), p) });
        }
        _ => {
            // Unknown block: keep what it contains, or at least its text.
            if kids.iter().any(|k| node_type(k) == "text") {
                out.push(Block::Paragraph { align: Align::Left, content: inlines(kids, p) });
            } else if !kids.is_empty() {
                out.extend(blocks(kids, p));
            }
        }
    }
}

fn list_items(items: &[Value], p: &Pages) -> Vec<ListItem> {
    items
        .iter()
        .map(|it| ListItem {
            checked: (node_type(it) == "taskItem").then(|| attr(it, "checked").and_then(Value::as_bool).unwrap_or(false)),
            blocks: blocks(children(it), p),
        })
        .collect()
}

fn cell(c: &Value, p: &Pages) -> Cell {
    let num = |name: &str| attr(c, name).and_then(Value::as_u64).unwrap_or(1).clamp(1, 50) as usize;
    let widths = attr(c, "colwidth")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_f64).map(|w| w as f32).filter(|w| w.is_finite()).collect())
        .unwrap_or_default();
    Cell { header: node_type(c) == "tableHeader", colspan: num("colspan"), rowspan: num("rowspan"), widths, blocks: blocks(children(c), p) }
}

/// The vault attachment id in an image address such as `http://clatt.localhost/<32 hex>`.
pub fn attachment_id(src: &str) -> Option<String> {
    let s = src.trim();
    if !(s.starts_with("http://clatt.localhost/") || s.starts_with("clatt://") || s.starts_with("https://clatt.localhost/")) {
        return None;
    }
    let last = s.trim_end_matches('/').rsplit('/').next()?;
    let id = last.split(['?', '#']).next()?;
    (id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit())).then(|| id.to_ascii_lowercase())
}

fn inlines(nodes: &[Value], p: &Pages) -> Vec<Inline> {
    let mut out = Vec::new();
    for n in nodes {
        match node_type(n) {
            "text" => {
                let text = n.get("text").and_then(Value::as_str).unwrap_or("");
                if !text.is_empty() {
                    out.push(Inline::Text(text.to_string(), marks(n, p)));
                }
            }
            "hardBreak" => out.push(Inline::Break),
            "mathInline" => out.push(Inline::Math(attr_str(n, "latex"))),
            "image" => {} // An inline picture is not made by the editor (pictures are blocks).
            _ => out.extend(inlines(children(n), p)),
        }
    }
    out
}

/// The inline content of a paragraph or heading, with the node's `dir` attribute put on its runs.
fn dir_inlines(n: &Value, kids: &[Value], p: &Pages) -> Vec<Inline> {
    let mut content = inlines(kids, p);
    let dir = Dir::parse(attr(n, "dir"));
    if dir != Dir::Auto {
        for i in &mut content {
            if let Inline::Text(_, m) = i {
                m.dir = dir;
            }
        }
    }
    content
}

fn marks(n: &Value, p: &Pages) -> Marks {
    let mut m = Marks::default();
    for mark in n.get("marks").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        match node_type(mark) {
            "bold" => m.bold = true,
            "italic" => m.italic = true,
            "underline" => m.underline = true,
            "strike" => m.strike = true,
            "code" => m.code = true,
            "link" => {
                let href = attr_str(mark, "href");
                if href.trim_start().to_ascii_lowercase().starts_with("notaya:") {
                    // Our own scheme never leaves the app: a link to a page of this export goes to
                    // its file or place; any other one is plain text.
                    m.link = None;
                    m.page = internal_page_id(&href).and_then(|id| (p.link)(id));
                } else {
                    m.link = Some(href).filter(|h| !h.is_empty());
                }
            }
            "highlight" => m.highlight = Some(Rgb::parse(&attr_str(mark, "color")).unwrap_or(DEFAULT_HIGHLIGHT)),
            "textStyle" => {
                if let Some(c) = Rgb::parse(&attr_str(mark, "color")) {
                    m.color = Some(c);
                }
                let size = attr_str(mark, "fontSize");
                if let Some(px) = size.strip_suffix("px").and_then(|v| v.trim().parse::<f32>().ok()).filter(|v| v.is_finite()) {
                    m.size = Some((px * 0.75).clamp(6.0, 72.0));
                } else if let Some(pt) = size.strip_suffix("pt").and_then(|v| v.trim().parse::<f32>().ok()).filter(|v| v.is_finite()) {
                    m.size = Some(pt.clamp(6.0, 72.0));
                }
            }
            _ => {} // comment marks and others: the text stays, the mark is not exported
        }
    }
    m
}

/// All the text inside a node (for code blocks).
fn plain_text(n: &Value) -> String {
    let mut s = String::new();
    fn walk(v: &Value, s: &mut String) {
        match node_type(v) {
            "text" => s.push_str(v.get("text").and_then(Value::as_str).unwrap_or("")),
            "hardBreak" => s.push('\n'),
            _ => children(v).iter().for_each(|c| walk(c, s)),
        }
    }
    walk(n, &mut s);
    s
}

/// The text of inline content, without formatting.
pub fn inline_text(content: &[Inline]) -> String {
    let mut s = String::new();
    for i in content {
        match i {
            Inline::Text(t, _) => s.push_str(t),
            Inline::Break => s.push(' '),
            Inline::Math(m) => s.push_str(m),
        }
    }
    s
}

/// Every vault attachment the blocks show, in order, each once.
pub fn attachments(blocks: &[Block], out: &mut Vec<String>) {
    for b in blocks {
        match b {
            Block::Image(Image { attachment: Some(id), .. }) => {
                if !out.contains(id) {
                    out.push(id.clone());
                }
            }
            Block::List { items, .. } => items.iter().for_each(|i| attachments(&i.blocks, out)),
            Block::Quote(x) | Block::Toggle(x) | Block::Shelf(x) | Block::Callout { blocks: x, .. } => attachments(x, out),
            Block::Columns(cols) => cols.iter().for_each(|c| attachments(c, out)),
            Block::Table(rows) => rows.iter().flatten().for_each(|c| attachments(&c.blocks, out)),
            _ => {}
        }
    }
}

/// The pages of the same document that links in the blocks point at ([`PageRef::Anchor`]), so
/// the writers mark only those places.
pub fn anchor_targets(blocks: &[Block], out: &mut std::collections::HashSet<usize>) {
    let inl = |content: &[Inline], out: &mut std::collections::HashSet<usize>| {
        for i in content {
            if let Inline::Text(_, Marks { page: Some(PageRef::Anchor(n)), .. }) = i {
                out.insert(*n);
            }
        }
    };
    for b in blocks {
        match b {
            Block::Paragraph { content, .. } | Block::Heading { content, .. } => inl(content, out),
            Block::PageLink { target: Some(PageRef::Anchor(n)), .. } => {
                out.insert(*n);
            }
            Block::List { items, .. } => items.iter().for_each(|i| anchor_targets(&i.blocks, out)),
            Block::Quote(x) | Block::Toggle(x) | Block::Shelf(x) | Block::Callout { blocks: x, .. } => anchor_targets(x, out),
            Block::Columns(cols) => cols.iter().for_each(|c| anchor_targets(c, out)),
            Block::Table(rows) => rows.iter().flatten().for_each(|c| anchor_targets(&c.blocks, out)),
            _ => {}
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const IMG_ID: &str = "0123456789abcdef0123456789abcdef";

    /// A page with every node and mark the editor can make.
    pub fn every_node_json() -> String {
        serde_json::json!({
            "type": "doc",
            "content": [
                { "type": "heading", "attrs": { "level": 1, "textAlign": "center" }, "content": [{ "type": "text", "text": "Big heading" }] },
                { "type": "heading", "attrs": { "level": 4 }, "content": [{ "type": "text", "text": "Small heading" }] },
                { "type": "paragraph", "content": [
                    { "type": "text", "text": "Plain, " },
                    { "type": "text", "text": "bold", "marks": [{ "type": "bold" }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "italic", "marks": [{ "type": "italic" }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "under", "marks": [{ "type": "underline" }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "struck", "marks": [{ "type": "strike" }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "x = 1", "marks": [{ "type": "code" }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "a link", "marks": [{ "type": "link", "attrs": { "href": "https://example.com/a?b=1&c=2" } }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "red", "marks": [{ "type": "textStyle", "attrs": { "color": "#e5484d", "fontSize": "20px" } }] },
                    { "type": "text", "text": ", " },
                    { "type": "text", "text": "marked", "marks": [{ "type": "highlight", "attrs": { "color": "rgba(236, 189, 40, 0.38)" } }] },
                    { "type": "text", "text": " and <special> *chars* & café € “quotes”." },
                    { "type": "hardBreak" },
                    { "type": "text", "text": "After a break, with " },
                    { "type": "mathInline", "attrs": { "latex": "E = mc^2" } },
                    { "type": "text", "text": "commented", "marks": [{ "type": "comment", "attrs": { "text": "a note" } }] }
                ] },
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "Bullet one" }] },
                        { "type": "bulletList", "content": [{ "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Nested bullet" }] }] }] }
                    ] },
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Bullet two" }] }] }
                ] },
                { "type": "orderedList", "attrs": { "start": 3 }, "content": [
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Third" }] }] },
                    { "type": "listItem", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Fourth" }] }] }
                ] },
                { "type": "taskList", "content": [
                    { "type": "taskItem", "attrs": { "checked": true }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Done task" }] }] },
                    { "type": "taskItem", "attrs": { "checked": false }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Open task" }] }] }
                ] },
                { "type": "blockquote", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "A quote." }] }] },
                { "type": "callout", "attrs": { "icon": "💡" }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "A callout." }] }] },
                { "type": "toggle", "attrs": { "open": false }, "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "Toggle line" }] },
                    { "type": "paragraph", "content": [{ "type": "text", "text": "Hidden text." }] }
                ] },
                { "type": "columns", "content": [
                    { "type": "column", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Left column" }] }] },
                    { "type": "column", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Right column" }] }] }
                ] },
                { "type": "codeBlock", "attrs": { "language": "rust" }, "content": [{ "type": "text", "text": "fn main() {\n    println!(\"```\");\n}" }] },
                { "type": "mathBlock", "attrs": { "latex": "x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}" } },
                { "type": "image", "attrs": { "src": format!("http://clatt.localhost/{IMG_ID}"), "width": 50, "align": "center", "caption": "A caption" } },
                { "type": "image", "attrs": { "src": "https://example.com/remote.png" } },
                { "type": "table", "content": [
                    { "type": "tableRow", "content": [
                        { "type": "tableHeader", "attrs": { "colspan": 1, "rowspan": 1, "colwidth": [120] }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Name" }] }] },
                        { "type": "tableHeader", "attrs": { "colspan": 1, "rowspan": 1, "colwidth": null }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Value | pipe" }] }] }
                    ] },
                    { "type": "tableRow", "content": [
                        { "type": "tableCell", "attrs": { "colspan": 1, "rowspan": 1 }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "alpha" }] }] },
                        { "type": "tableCell", "attrs": { "colspan": 1, "rowspan": 1 }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "1", "marks": [{ "type": "bold" }] }] }] }
                    ] },
                    { "type": "tableRow", "content": [
                        { "type": "tableCell", "attrs": { "colspan": 2, "rowspan": 1 }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "spans both" }] }] }
                    ] }
                ] },
                { "type": "horizontalRule" },
                { "type": "pageLink", "attrs": { "pageId": "child-page" } },
                { "type": "someFutureBlock", "content": [{ "type": "text", "text": "Unknown block text" }] },
                { "type": "paragraph" }
            ]
        })
        .to_string()
    }

    pub fn titles(id: &str) -> Option<String> {
        (id == "child-page").then(|| "Child page".to_string())
    }

    #[test]
    fn a_mermaid_diagram_is_exported_as_captioned_code() {
        let json = serde_json::json!({ "type": "doc", "content": [
            { "type": "mermaidBlock", "attrs": { "code": "graph TD\n  A --> B", "showCode": false } },
            { "type": "mermaidBlock", "attrs": { "code": "  " } }
        ] })
        .to_string();
        let blocks = parse(&json, &titles);
        assert_eq!(blocks.len(), 2, "an empty diagram is left out: {blocks:?}");
        assert!(matches!(&blocks[0], Block::Paragraph { content, .. } if matches!(&content[0], Inline::Text(t, m) if t == "Mermaid diagram (code)" && m.italic)));
        assert_eq!(blocks[1], Block::Code { language: "mermaid".into(), text: "graph TD\n  A --> B".into() });
    }

    #[test]
    fn pictures_read_the_new_placement_attributes_and_ignore_the_rest() {
        let src = format!("http://clatt.localhost/{IMG_ID}");
        let json = |attrs: Value| serde_json::json!({ "type": "doc", "content": [{ "type": "image", "attrs": attrs }] }).to_string();
        let one = |attrs: Value| match parse(&json(attrs), &titles).remove(0) {
            Block::Image(i) => i,
            _ => panic!("expected a picture"),
        };
        let img = one(serde_json::json!({ "src": src, "width": 40, "align": "right", "float": true, "ratio": "16:9", "free": true, "x": 12.5, "y": 300, "layer": "back" }));
        assert_eq!(img.attachment.as_deref(), Some(IMG_ID));
        assert!(img.float && img.align == Align::Right);
        assert!((img.ratio.unwrap() - 16.0 / 9.0).abs() < 1e-6);
        assert_eq!(img.width_pct, Some(40.0), "a free picture is exported in the text, with its width");
        // Old pages have none of them; junk is ignored.
        let old = one(serde_json::json!({ "src": src }));
        assert!(!old.float && old.ratio.is_none());
        let junk = one(serde_json::json!({ "src": src, "float": "yes", "ratio": "0:5", "x": "left", "layer": 7 }));
        assert!(!junk.float && junk.ratio.is_none());
        assert!(one(serde_json::json!({ "src": src, "ratio": "999:1" })).ratio.is_none());
    }

    #[test]
    fn parses_every_node_type() {
        let doc = parse(&every_node_json(), &titles);
        let kinds: Vec<&str> = doc
            .iter()
            .map(|b| match b {
                Block::Heading { .. } => "heading",
                Block::Paragraph { .. } => "paragraph",
                Block::List { kind: ListKind::Bullet, .. } => "bullets",
                Block::List { kind: ListKind::Ordered(_), .. } => "ordered",
                Block::List { kind: ListKind::Task, .. } => "tasks",
                Block::Quote(_) => "quote",
                Block::Callout { .. } => "callout",
                Block::Toggle(_) => "toggle",
                Block::Shelf(_) => "shelf",
                Block::Columns(_) => "columns",
                Block::Code { .. } => "code",
                Block::Math(_) => "math",
                Block::Image(_) => "image",
                Block::Table(_) => "table",
                Block::Database { .. } => "database",
                Block::Rule => "rule",
                Block::PageLink { .. } => "pagelink",
            })
            .collect();
        assert_eq!(
            kinds,
            [
                "heading", "heading", "paragraph", "bullets", "ordered", "tasks", "quote", "callout", "toggle", "columns", "code", "math", "image", "image", "table",
                "rule", "pagelink", "paragraph", "paragraph"
            ]
        );
        let Block::Paragraph { content, .. } = &doc[2] else { panic!() };
        let Inline::Text(_, red) = &content[13] else { panic!() };
        assert_eq!(red.color, Some(Rgb(0xE5, 0x48, 0x4D)));
        assert_eq!(red.size, Some(15.0));
        let Inline::Text(_, marked) = &content[15] else { panic!() };
        assert_eq!(marked.highlight, Some(Rgb(248, 230, 173)), "translucent highlight blended onto white");
        assert!(content.contains(&Inline::Break));
        assert!(content.contains(&Inline::Math("E = mc^2".into())));
        let Block::Image(img) = &doc[12] else { panic!() };
        assert_eq!(img.attachment.as_deref(), Some(IMG_ID));
        assert_eq!(img.width_pct, Some(50.0));
        let Block::Image(remote) = &doc[13] else { panic!() };
        assert_eq!(remote.attachment, None, "only the vault's own pictures are exported");
        assert_eq!(doc[16], Block::PageLink { title: "Child page".into(), target: None });
        let mut ids = Vec::new();
        attachments(&doc, &mut ids);
        assert_eq!(ids, [IMG_ID]);
    }

    #[test]
    fn links_to_pages_of_the_export_are_resolved_and_others_are_plain_text() {
        let json = serde_json::json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [
                { "type": "text", "text": "in", "marks": [{ "type": "link", "attrs": { "href": "notaya:page/child-page#h1abc" } }] },
                { "type": "text", "text": "out", "marks": [{ "type": "link", "attrs": { "href": "notaya:page/elsewhere" } }] },
                { "type": "text", "text": "odd", "marks": [{ "type": "link", "attrs": { "href": " NOTAYA:page/../../x" } }] },
                { "type": "text", "text": "web", "marks": [{ "type": "link", "attrs": { "href": "https://example.com" } }] }
            ] },
            { "type": "pageLink", "attrs": { "pageId": "child-page" } },
            { "type": "pageLink", "attrs": { "pageId": "elsewhere" } },
            { "type": "bulletList", "content": [{ "type": "listItem", "content": [{ "type": "pageLink", "attrs": { "pageId": "child-page" } }] }] }
        ]})
        .to_string();
        let resolve = |id: &str| (id == "child-page").then(|| PageRef::Anchor(2));
        let titles = |id: &str| match id {
            "child-page" => Some("Child page".to_string()),
            "elsewhere" => Some("Elsewhere".to_string()),
            _ => None,
        };
        let doc = parse_with(&json, &titles, &resolve);
        let Block::Paragraph { content, .. } = &doc[0] else { panic!() };
        let marks: Vec<&Marks> = content.iter().map(|i| if let Inline::Text(_, m) = i { m } else { panic!() }).collect();
        assert_eq!((marks[0].page.clone(), marks[0].link.clone()), (Some(PageRef::Anchor(2)), None), "a link into the export");
        assert_eq!((marks[1].page.clone(), marks[1].link.clone()), (None, None), "a page outside the export: plain text, never a notaya: address");
        assert_eq!((marks[2].page.clone(), marks[2].link.clone()), (None, None), "our own scheme in any form never survives");
        assert_eq!(marks[3].link.as_deref(), Some("https://example.com"));
        assert_eq!(doc[1], Block::PageLink { title: "Child page".into(), target: Some(PageRef::Anchor(2)) });
        assert_eq!(doc[2], Block::PageLink { title: "Elsewhere".into(), target: None });
        let mut used = std::collections::HashSet::new();
        anchor_targets(&doc, &mut used);
        assert_eq!(used.into_iter().collect::<Vec<_>>(), [2]);
        // Plain `parse` resolves nothing.
        let Block::Paragraph { content, .. } = &parse(&json, &titles)[0] else { panic!() };
        assert!(content.iter().all(|i| matches!(i, Inline::Text(_, m) if m.page.is_none())));
        assert_eq!(internal_page_id("notaya:page/abc_D-1#t:x"), Some("abc_D-1"));
        for bad in ["notaya:page/", "notaya:page/a b", "notaya:page/../x", "https://x"] {
            assert_eq!(internal_page_id(bad), None, "{bad}");
        }
        assert_eq!(url_segment("A b%#?&é.md"), "A%20b%25%23%3F%26%C3%A9.md");
        assert_eq!(PageRef::Anchor(3).href(), "#notaya_page_3");
    }

    #[test]
    fn empty_or_broken_content_is_handled() {
        assert!(parse("", &titles).is_empty());
        assert_eq!(parse("not json", &titles).len(), 1, "the raw text is kept");
        assert!(parse(r#"{"type":"doc"}"#, &titles).is_empty());
    }

    #[test]
    fn colors_and_attachment_addresses() {
        assert_eq!(Rgb::parse("#abc"), Some(Rgb(0xAA, 0xBB, 0xCC)));
        assert_eq!(Rgb::parse("rgb(1, 2, 3)"), Some(Rgb(1, 2, 3)));
        assert_eq!(Rgb::parse("var(--x)"), None);
        assert_eq!(Rgb::parse("#12"), None);
        assert_eq!(attachment_id(&format!("http://clatt.localhost/{}", IMG_ID.to_uppercase())).as_deref(), Some(IMG_ID));
        assert_eq!(attachment_id(&format!("clatt://localhost/{IMG_ID}")).as_deref(), Some(IMG_ID));
        assert_eq!(attachment_id(&format!("https://evil.example/{IMG_ID}")), None);
        assert_eq!(attachment_id("http://clatt.localhost/../etc"), None);
    }
    #[test]
    fn links_inside_a_shelf_count_as_link_targets() {
        let link = Block::PageLink { title: "Inside".into(), target: Some(PageRef::Anchor(7)) };
        let mut found = std::collections::HashSet::new();
        anchor_targets(&[Block::Shelf(vec![link.clone()]), Block::Toggle(vec![Block::Quote(vec![Block::PageLink { title: "Deep".into(), target: Some(PageRef::Anchor(9)) }])])], &mut found);
        assert!(found.contains(&7) && found.contains(&9), "{found:?}");
    }

    #[test]
    fn the_first_strong_letter_decides_the_automatic_direction() {
        assert_eq!(first_strong("Hello"), Some(Dir::Ltr));
        assert_eq!(first_strong("مرحبا بالعالم"), Some(Dir::Rtl));
        assert_eq!(first_strong("שלום"), Some(Dir::Rtl));
        assert_eq!(first_strong("سلام"), Some(Dir::Rtl), "Persian and Urdu share the Arabic block");
        // Digits, punctuation, emoji and spaces are not strong: the first letter after them decides.
        assert_eq!(first_strong("123 - مرحبا"), Some(Dir::Rtl));
        assert_eq!(first_strong("\u{661}\u{662}\u{663} abc"), Some(Dir::Ltr), "Arabic-Indic digits are not letters");
        assert_eq!(first_strong("(!) 😀 hello"), Some(Dir::Ltr));
        assert_eq!(first_strong("Hello مرحبا"), Some(Dir::Ltr));
        assert_eq!(first_strong("\u{200F}abc"), Some(Dir::Rtl), "an explicit right-to-left mark");
        assert_eq!(first_strong("12 + 3"), None);
        assert_eq!(first_strong(""), None);
    }

    #[test]
    fn database_block_parses_ids_without_trusting_embedded_data() {
        let blocks = parse(r#"{"type":"doc","content":[
            {"type":"databaseBlock","attrs":{"databaseId":"db","viewId":"view","title":"untrusted","rows":["untrusted"]}},
            {"type":"databaseBlock"}
        ]}"#, &titles);
        assert_eq!(blocks, vec![
            Block::Database { id: "db".into(), view: "view".into(), table: None },
            Block::Database { id: String::new(), view: String::new(), table: None },
        ]);
        let fallback = database_blocks(None);
        assert!(matches!(&fallback[0], Block::Paragraph { content, .. } if inline_text(content) == "(database not available)"));
    }

    #[test]
    fn a_page_saved_before_the_direction_setting_loads_as_automatic() {
        // No `dir` anywhere (what every older version wrote): every block is automatic, and Arabic text
        // is shown right-to-left by its first letter, with no migration.
        let old = r#"{"type":"doc","content":[
            {"type":"heading","attrs":{"level":2,"textAlign":null},"content":[{"type":"text","text":"عنوان"}]},
            {"type":"paragraph","content":[{"type":"text","text":"مرحبا بالعالم"}]},
            {"type":"paragraph","attrs":{"textAlign":"left"},"content":[{"type":"text","text":"English"}]},
            {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"بند"}]}]}]}]}"#;
        let blocks = parse(old, &titles);
        for b in &blocks {
            if let Block::Paragraph { content, .. } | Block::Heading { content, .. } = b {
                assert_eq!(explicit_dir(content), Dir::Auto);
            }
        }
        let dirs: Vec<Dir> = blocks
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph { content, .. } | Block::Heading { content, .. } => Some(content_dir(content)),
                _ => None,
            })
            .collect();
        assert_eq!(dirs, vec![Dir::Rtl, Dir::Rtl, Dir::Ltr]);
        assert_eq!(blocks_dir(&blocks[3..]), Some(Dir::Rtl), "a list takes the direction of its first line");
    }

    #[test]
    fn a_stored_direction_overrides_the_text() {
        let json = r#"{"type":"doc","content":[
            {"type":"paragraph","attrs":{"dir":"rtl"},"content":[{"type":"text","text":"English"},{"type":"text","text":" bold","marks":[{"type":"bold"}]}]},
            {"type":"paragraph","attrs":{"dir":"ltr"},"content":[{"type":"text","text":"مرحبا"}]},
            {"type":"paragraph","attrs":{"dir":null},"content":[{"type":"text","text":"مرحبا"}]},
            {"type":"paragraph","attrs":{"dir":"sideways"},"content":[{"type":"text","text":"x"}]}]}"#;
        let dirs: Vec<(Dir, Dir)> = parse(json, &titles)
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph { content, .. } => Some((explicit_dir(content), content_dir(content))),
                _ => None,
            })
            .collect();
        assert_eq!(dirs, vec![(Dir::Rtl, Dir::Rtl), (Dir::Ltr, Dir::Ltr), (Dir::Auto, Dir::Rtl), (Dir::Auto, Dir::Ltr)]);
    }
}
