//! The small document model every importer produces, and its conversion to the editor's
//! TipTap (ProseMirror) JSON plus the plain-text copy kept for search.
//!
//! The renderer is the only place that writes editor JSON, so it alone has to follow the
//! editor's schema (see `src/features/notes/editor/extensions.ts`): list items start with a
//! paragraph, containers are never empty, images are blocks, text nodes are never empty and
//! code text carries no other mark.

use serde_json::{json, Map, Value};

/// Text formatting. `link` is already checked (http, https or mailto only).
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
    /// Background color: `Some("")` is the editor's default highlight, otherwise one of
    /// [`HIGHLIGHTS`]. `None` = no highlight.
    pub highlight: Option<String>,
    /// Text color, one of [`TEXT_COLORS`].
    pub color: Option<String>,
    /// Font family, one of the editor's choices ([`FONT_SERIF`], [`FONT_MONO`]).
    pub font: Option<String>,
    pub link: Option<String>,
}

/// Color names used in Notion's export classes, matched by position to the Notaya colors below.
pub const NOTION_CLASS_NAMES: [&str; 9] = ["gray", "brown", "orange", "yellow", "green", "blue", "purple", "pink", "red"];

/// The Notaya palette colors an import writes (second row of TEXT_COLORS and HIGHLIGHTS in
/// src/lib/palette.ts), by name.
pub const COLOR_NAMES: [&str; 9] = ["Ash", "Cocoa", "Amber", "Honey", "Moss", "Denim", "Plum", "Orchid", "Cherry"];

/// Their text colors, exactly as the editor's text color menu writes them (so the swatch
/// shows as selected).
pub const TEXT_COLORS: [&str; 9] = ["#787774", "#9f6b53", "#d9730d", "#cb912f", "#448361", "#337ea9", "#9065b0", "#c14c8a", "#d44c47"];

/// Their highlight colors, exactly as the editor's highlight menu writes them.
pub const HIGHLIGHTS: [&str; 9] = [
    "rgba(120, 119, 116, 0.28)",
    "rgba(159, 107, 83, 0.28)",
    "rgba(217, 115, 13, 0.28)",
    "rgba(203, 145, 47, 0.30)",
    "rgba(68, 131, 97, 0.28)",
    "rgba(51, 126, 169, 0.28)",
    "rgba(144, 101, 176, 0.28)",
    "rgba(193, 76, 138, 0.28)",
    "rgba(212, 76, 71, 0.28)",
];

/// The editor's Serif and Mono font choices (FONT_FAMILIES in BubbleToolbar.tsx).
pub const FONT_SERIF: &str = "Georgia, Cambria, serif";
pub const FONT_MONO: &str = "\"JetBrains Mono Variable\", Consolas, monospace";

/// The Notaya color for a color name in Notion's export (`red`, `red_background`, `gray`…), as
/// `(is_background, exact value)`: red is Cherry, gray is Ash, and so on. `default` and unknown
/// names give `None`.
pub fn notion_color(name: &str) -> Option<(bool, &'static str)> {
    let (base, background) = match name.strip_suffix("_background") {
        Some(b) => (b, true),
        None => (name, false),
    };
    let i = NOTION_CLASS_NAMES.iter().position(|c| *c == base)?;
    Some(if background { (true, HIGHLIGHTS[i]) } else { (false, TEXT_COLORS[i]) })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text(String, Marks),
    Break,
    /// A picture that sat inside text: the index into the import's image list. The renderer
    /// moves it out into its own block (pictures are blocks in the editor).
    Image(usize),
    /// An inline equation (TeX).
    Math(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub header: bool,
    pub colspan: u32,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    Bullets(Vec<Vec<Block>>),
    Numbered(u32, Vec<Vec<Block>>),
    Tasks(Vec<(bool, Vec<Block>)>),
    Quote(Vec<Block>),
    /// A highlighted box with an emoji (Notion callouts).
    Callout(String, Vec<Block>),
    /// A collapsible block; its first block is the line that stays visible.
    Toggle(Vec<Block>),
    Code(String, String),
    Rule,
    Table(Vec<Vec<Cell>>),
    /// A picture (index into the import's image list) with a caption.
    Image(usize, String),
    /// A block equation (TeX).
    Math(String),
    /// Two to five columns side by side (other counts are flattened).
    Columns(Vec<Vec<Block>>),
    /// A link to another imported page: the page's source path and the text to show when
    /// that page is not part of the import.
    PageLink(String, String),
}

/// Containers nested deeper than this are flattened into their parent.
const MAX_DEPTH: usize = 24;

/// What a stored picture turned into when the page was written.
pub enum ImageOut {
    /// Shown from the vault at this URL.
    Url(String),
    /// Could not be stored: shown as this text instead.
    Text(String),
}

/// Writes the blocks as an editor document. `image` says what each image index became, and
/// `page` gives the id of an imported page from its source path.
pub fn render(blocks: &[Block], image: &dyn Fn(usize) -> ImageOut, page: &dyn Fn(&str) -> Option<String>) -> (String, String) {
    let mut r = Renderer { image, page, text: Vec::new() };
    let mut content = r.blocks(blocks, 0);
    if content.is_empty() {
        content.push(json!({ "type": "paragraph" }));
    }
    let doc = json!({ "type": "doc", "content": content });
    (doc.to_string(), r.text.join("\n\n"))
}

struct Renderer<'a> {
    image: &'a dyn Fn(usize) -> ImageOut,
    page: &'a dyn Fn(&str) -> Option<String>,
    /// Plain text of every text block, in order (joined like the editor's getText()).
    text: Vec<String>,
}

impl Renderer<'_> {
    fn blocks(&mut self, blocks: &[Block], depth: usize) -> Vec<Value> {
        let mut out = Vec::new();
        for b in blocks {
            self.block(b, depth, &mut out);
        }
        out
    }

    /// Blocks for a container that must hold at least one block.
    fn container(&mut self, blocks: &[Block], depth: usize) -> Vec<Value> {
        let mut v = self.blocks(blocks, depth + 1);
        if v.is_empty() {
            v.push(json!({ "type": "paragraph" }));
        }
        v
    }

    /// Content of a list item: a paragraph first, then any blocks.
    fn item(&mut self, blocks: &[Block], depth: usize) -> Vec<Value> {
        let mut v = self.blocks(blocks, depth + 1);
        if v.first().and_then(|n| n.get("type")).and_then(Value::as_str) != Some("paragraph") {
            v.insert(0, json!({ "type": "paragraph" }));
        }
        v
    }

    fn block(&mut self, b: &Block, depth: usize, out: &mut Vec<Value>) {
        let deep = depth >= MAX_DEPTH;
        match b {
            Block::Paragraph(inl) => self.textblock(json!({ "type": "paragraph" }), inl, out),
            Block::Heading(level, inl) => {
                let level = (*level).clamp(1, 4);
                self.textblock(json!({ "type": "heading", "attrs": { "level": level, "textAlign": null } }), inl, out)
            }
            Block::Bullets(items) | Block::Numbered(_, items) if deep => {
                for i in items {
                    out.extend(self.blocks(i, depth));
                }
            }
            Block::Bullets(items) => {
                if items.is_empty() {
                    return;
                }
                let content: Vec<Value> =
                    items.iter().map(|i| json!({ "type": "listItem", "content": self.item(i, depth) })).collect();
                out.push(json!({ "type": "bulletList", "content": content }));
            }
            Block::Numbered(start, items) => {
                if items.is_empty() {
                    return;
                }
                let content: Vec<Value> =
                    items.iter().map(|i| json!({ "type": "listItem", "content": self.item(i, depth) })).collect();
                out.push(json!({ "type": "orderedList", "attrs": { "start": (*start).max(1), "type": null }, "content": content }));
            }
            Block::Tasks(items) => {
                if deep {
                    for (_, i) in items {
                        out.extend(self.blocks(i, depth));
                    }
                    return;
                }
                if items.is_empty() {
                    return;
                }
                let content: Vec<Value> = items
                    .iter()
                    .map(|(checked, i)| json!({ "type": "taskItem", "attrs": { "checked": checked }, "content": self.item(i, depth) }))
                    .collect();
                out.push(json!({ "type": "taskList", "content": content }));
            }
            Block::Quote(inner) | Block::Toggle(inner) | Block::Callout(_, inner) if deep => out.extend(self.blocks(inner, depth)),
            Block::Quote(inner) => out.push(json!({ "type": "blockquote", "content": self.container(inner, depth) })),
            Block::Toggle(inner) => out.push(json!({ "type": "toggle", "attrs": { "open": true }, "content": self.container(inner, depth) })),
            Block::Callout(icon, inner) => {
                let icon = if icon.is_empty() || icon.chars().count() > 8 { "💡" } else { icon.as_str() };
                out.push(json!({ "type": "callout", "attrs": { "icon": icon }, "content": self.container(inner, depth) }))
            }
            Block::Code(lang, code) => {
                let code = code.trim_end_matches('\n');
                let lang: String = lang.chars().filter(|c| c.is_ascii_alphanumeric() || "+#-_".contains(*c)).take(32).collect();
                let mut node = json!({ "type": "codeBlock", "attrs": { "language": if lang.is_empty() { Value::Null } else { Value::String(lang.to_ascii_lowercase()) } } });
                if !code.is_empty() {
                    node["content"] = json!([{ "type": "text", "text": code }]);
                    self.text.push(code.to_string());
                }
                out.push(node);
            }
            Block::Rule => out.push(json!({ "type": "horizontalRule" })),
            Block::Table(rows) => {
                if deep {
                    for row in rows {
                        for c in row {
                            out.extend(self.blocks(&c.blocks, depth));
                        }
                    }
                    return;
                }
                let width = rows.iter().map(|r| r.iter().map(|c| c.colspan.max(1) as usize).sum::<usize>()).max().unwrap_or(0);
                if width == 0 {
                    return;
                }
                let mut trs = Vec::new();
                for row in rows {
                    let mut cells = Vec::new();
                    let mut used = 0;
                    for c in row {
                        let span = (c.colspan.max(1) as usize).min(width - used);
                        if span == 0 {
                            break;
                        }
                        used += span;
                        cells.push(json!({
                            "type": if c.header { "tableHeader" } else { "tableCell" },
                            "attrs": { "colspan": span, "rowspan": 1, "colwidth": null },
                            "content": self.container(&c.blocks, depth),
                        }));
                    }
                    // Every row gets the same number of columns.
                    while used < width {
                        used += 1;
                        cells.push(json!({ "type": "tableCell", "attrs": { "colspan": 1, "rowspan": 1, "colwidth": null }, "content": [{ "type": "paragraph" }] }));
                    }
                    trs.push(json!({ "type": "tableRow", "content": cells }));
                }
                out.push(json!({ "type": "table", "content": trs }));
            }
            Block::Image(index, caption) => self.image(*index, caption, out),
            Block::Math(latex) => {
                let latex: String = latex.trim().chars().take(20_000).collect();
                if !latex.is_empty() {
                    self.text.push(latex.clone());
                    out.push(json!({ "type": "mathBlock", "attrs": { "latex": latex } }));
                }
            }
            Block::Columns(cols) => {
                if deep || cols.len() < 2 || cols.len() > 5 {
                    for c in cols {
                        out.extend(self.blocks(c, depth));
                    }
                    return;
                }
                let content: Vec<Value> = cols.iter().map(|c| json!({ "type": "column", "content": self.container(c, depth) })).collect();
                out.push(json!({ "type": "columns", "content": content }));
            }
            Block::PageLink(path, label) => match (self.page)(path) {
                Some(id) => out.push(json!({ "type": "pageLink", "attrs": { "pageId": id } })),
                None => self.textblock(json!({ "type": "paragraph" }), &[t(label.clone())], out),
            },
        }
    }

    fn image(&mut self, index: usize, caption: &str, out: &mut Vec<Value>) {
        match (self.image)(index) {
            ImageOut::Url(src) => {
                let caption: String = caption.chars().take(500).collect();
                out.push(json!({
                    "type": "image",
                    "attrs": { "src": src, "alt": null, "title": null, "width": null, "align": "center", "caption": caption },
                }));
            }
            ImageOut::Text(t) => {
                let t = if caption.is_empty() { t } else { format!("{t} {caption}") };
                self.text.push(t.clone());
                out.push(json!({ "type": "paragraph", "content": [{ "type": "text", "text": t, "marks": [{ "type": "italic" }] }] }));
            }
        }
    }

    /// A paragraph or heading. Pictures inside it are moved out into their own blocks, and
    /// one with no text at all is dropped.
    fn textblock(&mut self, node: Value, inl: &[Inline], out: &mut Vec<Value>) {
        let mut run: Vec<&Inline> = Vec::new();
        let mut pending_images = Vec::new();
        let is_heading = node["type"] == "heading";
        for i in inl {
            match i {
                Inline::Image(idx) => {
                    if is_heading {
                        pending_images.push(*idx);
                    } else {
                        self.flush_run(&node, &run, out);
                        run.clear();
                        self.image(*idx, "", out);
                    }
                }
                other => run.push(other),
            }
        }
        self.flush_run(&node, &run, out);
        for idx in pending_images {
            self.image(idx, "", out);
        }
    }

    fn flush_run(&mut self, node: &Value, run: &[&Inline], out: &mut Vec<Value>) {
        let (content, text) = inline_nodes(run);
        if content.is_empty() {
            return;
        }
        let mut n = node.clone();
        n["content"] = Value::Array(content);
        self.text.push(text);
        out.push(n);
    }
}

fn mark_list(m: &Marks) -> Vec<Value> {
    let mut v = Vec::new();
    if m.code {
        // The editor's code mark excludes every other mark.
        v.push(json!({ "type": "code" }));
        return v;
    }
    if let Some(href) = &m.link {
        v.push(json!({ "type": "link", "attrs": { "href": href, "target": null, "rel": "noopener noreferrer nofollow", "class": null } }));
    }
    if m.bold {
        v.push(json!({ "type": "bold" }));
    }
    if m.italic {
        v.push(json!({ "type": "italic" }));
    }
    if m.underline {
        v.push(json!({ "type": "underline" }));
    }
    if m.strike {
        v.push(json!({ "type": "strike" }));
    }
    if let Some(c) = &m.highlight {
        let color = if c.is_empty() { Value::Null } else { json!(c) };
        v.push(json!({ "type": "highlight", "attrs": { "color": color } }));
    }
    if m.color.is_some() || m.font.is_some() {
        v.push(json!({ "type": "textStyle", "attrs": { "color": m.color, "fontFamily": m.font, "fontSize": null } }));
    }
    v
}

/// Text nodes for a run of inline content: whitespace at the edges trimmed, neighbours with
/// the same marks merged, never an empty text node. Returns the nodes and their plain text.
fn inline_nodes(run: &[&Inline]) -> (Vec<Value>, String) {
    // Merge first, so trimming works on whole strings.
    // (None, "") is a line break, (None, latex) an inline equation.
    let mut parts: Vec<(Option<Marks>, String)> = Vec::new();
    for i in run {
        match i {
            Inline::Math(latex) => {
                let latex: String = latex.trim().chars().take(5_000).collect();
                if !latex.is_empty() {
                    parts.push((None, latex));
                }
            }
            Inline::Text(t, m) => {
                if t.is_empty() {
                    continue;
                }
                let clean: String = t.chars().filter(|c| !c.is_control() || *c == '\t').map(|c| if c == '\t' { ' ' } else { c }).collect();
                match parts.last_mut() {
                    Some((Some(pm), pt)) if pm == m => pt.push_str(&clean),
                    _ => parts.push((Some(m.clone()), clean)),
                }
            }
            Inline::Break => parts.push((None, String::new())),
            Inline::Image(_) => {}
        }
    }
    // Trim the start and end of the whole run, and drop breaks at the edges.
    while matches!(parts.first(), Some((None, t)) if t.is_empty()) {
        parts.remove(0);
    }
    while matches!(parts.last(), Some((None, t)) if t.is_empty()) {
        parts.pop();
    }
    if let Some((Some(m), t)) = parts.first_mut() {
        if !m.code {
            *t = t.trim_start().to_string();
        }
    }
    if let Some((Some(m), t)) = parts.last_mut() {
        if !m.code {
            *t = t.trim_end().to_string();
        }
    }
    let mut nodes = Vec::new();
    let mut text = String::new();
    for (m, t) in parts {
        match m {
            None if t.is_empty() => {
                nodes.push(json!({ "type": "hardBreak" }));
                text.push('\n');
            }
            None => {
                text.push_str(&t);
                nodes.push(json!({ "type": "mathInline", "attrs": { "latex": t } }));
            }
            Some(m) => {
                if t.is_empty() {
                    continue;
                }
                let mut n = Map::new();
                n.insert("type".into(), json!("text"));
                n.insert("text".into(), json!(t));
                let marks = mark_list(&m);
                if !marks.is_empty() {
                    n.insert("marks".into(), Value::Array(marks));
                }
                text.push_str(&t);
                nodes.push(Value::Object(n));
            }
        }
    }
    // A run of nothing but breaks and spaces is empty.
    if nodes.iter().all(|n| n["type"] == "hardBreak") {
        return (Vec::new(), String::new());
    }
    (nodes, text)
}

/// Plain text helper used by converters (for titles and captions).
pub fn plain(inl: &[Inline]) -> String {
    let mut s = String::new();
    for i in inl {
        match i {
            Inline::Text(t, _) => s.push_str(t),
            Inline::Break => s.push(' '),
            Inline::Math(m) => s.push_str(m),
            Inline::Image(_) => {}
        }
    }
    collapse_ws(&s)
}

/// Collapses runs of whitespace into one space and trims the ends.
pub fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Shorthand for plain text with no marks.
pub fn t(s: impl Into<String>) -> Inline {
    Inline::Text(s.into(), Marks::default())
}

/// True when the blocks hold no text and no picture.
pub fn is_empty(blocks: &[Block]) -> bool {
    blocks.iter().all(|b| match b {
        Block::Paragraph(i) | Block::Heading(_, i) => i.iter().all(|x| match x {
            Inline::Text(t, _) => t.trim().is_empty(),
            Inline::Break => true,
            Inline::Image(_) | Inline::Math(_) => false,
        }),
        Block::Rule => true,
        Block::Code(_, c) => c.trim().is_empty(),
        Block::Bullets(items) | Block::Numbered(_, items) => items.iter().all(|i| is_empty(i)),
        Block::Tasks(items) => items.iter().all(|(_, i)| is_empty(i)),
        Block::Quote(b) | Block::Toggle(b) | Block::Callout(_, b) => is_empty(b),
        Block::Table(rows) => rows.iter().all(|r| r.iter().all(|c| is_empty(&c.blocks))),
        Block::Columns(cols) => cols.iter().all(|c| is_empty(c)),
        Block::Image(..) | Block::Math(_) | Block::PageLink(..) => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_images(_: usize) -> ImageOut {
        ImageOut::Text("[picture]".into())
    }

    fn bold(s: &str) -> Inline {
        Inline::Text(s.into(), Marks { bold: true, ..Marks::default() })
    }

    #[test]
    fn renders_valid_shapes() {
        let blocks = vec![
            Block::Heading(6, vec![t("Title")]),
            Block::Paragraph(vec![t("  a "), bold("b"), t(" c  ")]),
            Block::Paragraph(vec![t("   ")]),
            Block::Bullets(vec![vec![], vec![Block::Code(String::new(), "x".into())]]),
            Block::Quote(vec![]),
            Block::Code("Rust<script>".into(), "fn main() {}\n".into()),
            Block::Table(vec![vec![Cell { header: true, colspan: 2, blocks: vec![] }], vec![Cell { header: false, colspan: 1, blocks: vec![Block::Paragraph(vec![t("1")])] }]]),
        ];
        let (json, text) = render(&blocks, &no_images, &|_| None);
        let v: Value = serde_json::from_str(&json).unwrap();
        let c = v["content"].as_array().unwrap();
        assert_eq!(c[0]["attrs"]["level"], 4, "levels above 4 are clamped");
        assert_eq!(c[1]["content"][0]["text"], "a ");
        assert_eq!(c[1]["content"][1]["marks"][0]["type"], "bold");
        assert_eq!(c[1]["content"][2]["text"], " c");
        assert_eq!(c[2]["type"], "bulletList", "an empty paragraph is dropped");
        assert_eq!(c[2]["content"][0]["content"][0]["type"], "paragraph");
        assert_eq!(c[2]["content"][1]["content"][0]["type"], "paragraph", "a list item starts with a paragraph");
        assert_eq!(c[3]["content"][0]["type"], "paragraph", "containers are never empty");
        assert_eq!(c[4]["attrs"]["language"], "rustscript");
        assert_eq!(c[5]["content"][1]["content"].as_array().unwrap().len(), 2, "rows are padded");
        assert!(text.contains("Title\n\na b c"));
        assert!(!json.contains("\"text\":\"\""), "no empty text nodes");
    }

    #[test]
    fn images_move_out_of_text_and_code_drops_other_marks() {
        let blocks = vec![
            Block::Paragraph(vec![t("before"), Inline::Image(0), t("after")]),
            Block::Paragraph(vec![Inline::Text("x".into(), Marks { code: true, bold: true, link: Some("https://a".into()), ..Marks::default() })]),
        ];
        let (json, _) = render(&blocks, &|_| ImageOut::Url("http://clatt.localhost/abc".into()), &|_| None);
        let v: Value = serde_json::from_str(&json).unwrap();
        let c = v["content"].as_array().unwrap();
        assert_eq!(c.len(), 4);
        assert_eq!(c[1]["type"], "image");
        assert_eq!(c[1]["attrs"]["src"], "http://clatt.localhost/abc");
        assert_eq!(c[3]["content"][0]["marks"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn colors_are_written_like_the_editor_writes_them() {
        assert_eq!(notion_color("red"), Some((false, "#d44c47")));
        assert_eq!(notion_color("gray_background"), Some((true, "rgba(120, 119, 116, 0.28)")));
        assert_eq!(notion_color("default"), None);
        let m = Marks { color: Some("#d44c47".into()), highlight: Some(HIGHLIGHTS[3].into()), font: Some(FONT_SERIF.into()), ..Marks::default() };
        let (json, _) = render(&[Block::Paragraph(vec![Inline::Text("x".into(), m)])], &no_images, &|_| None);
        let v: Value = serde_json::from_str(&json).unwrap();
        let marks = &v["content"][0]["content"][0]["marks"];
        assert_eq!(marks[0], json!({ "type": "highlight", "attrs": { "color": "rgba(203, 145, 47, 0.30)" } }));
        assert_eq!(marks[1], json!({ "type": "textStyle", "attrs": { "color": "#d44c47", "fontFamily": "Georgia, Cambria, serif", "fontSize": null } }));
    }

    #[test]
    fn deep_nesting_is_flattened() {
        let mut b = Block::Paragraph(vec![t("deep")]);
        for _ in 0..200 {
            b = Block::Quote(vec![b]);
        }
        let (json, text) = render(&[b], &no_images, &|_| None);
        assert!(json.matches("blockquote").count() <= MAX_DEPTH + 1);
        assert_eq!(text, "deep");
    }
}
