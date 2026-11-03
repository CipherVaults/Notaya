//! Word documents (.docx: a ZIP of XML) to the import model.
//!
//! Read: `word/document.xml` (paragraphs, runs, tables, hyperlinks, pictures), `styles.xml`
//! (which styles are headings, the title and quotes), `numbering.xml` (bullet or numbered
//! lists, nested by level), the relationships (link targets and media) and the title in
//! `docProps/core.xml`. Kept: headings 1 to 4 (5 to 9 become 4), bold, italic, underline,
//! strike, highlight, text color, lists, tables (with merged columns), links (http, https,
//! mailto), embedded pictures (PNG, JPEG, GIF, WebP) and line breaks. What Notaya's own Word
//! export writes comes back too: code blocks (paragraphs in a code style; the language from
//! Notaya's hidden bookmark), task lists (paragraphs that start with ☐ or ☑), picture captions
//! (a "Caption" paragraph after the picture), shelves (a one-cell table in the "NotayaShelf"
//! style) and every palette highlight as itself (the run shading is matched exactly first; see
//! `palette`). Dropped: tracked
//! deletions, field codes (their shown result stays), comments, footnotes, headers and
//! footers, text boxes' position, fonts and sizes, and pictures in other formats (EMF, WMF).
//!
//! Limits (the file is hostile): tables nested deeper than [`MAX_TABLE_DEPTH`] are read flat (their
//! text stays, as paragraphs of the cell around them), paragraphs and hyperlinks nest at most
//! [`MAX_NEST`] deep, a document keeps at most [`doc::MAX_PAGE_CELLS`] table cells, and what is
//! inflated counts against the import's shared budget (`Vfs::budget`).

use std::cell::Cell as Counter;
use std::collections::HashMap;

use super::doc::{self, Block, Cell, Inline, Marks, HIGHLIGHTS};
use super::markup::{attr, local, Token, Tokenizer};
use super::zip::Archive;
use super::{palette, safe_link, Ctx};
use crate::export::doc::Dir;
use crate::export::docx::{CODE_BOOKMARK, SHELF_STYLE};

const MAX_XML: u64 = 64 * 1024 * 1024;
/// Tables inside tables inside cells: deeper ones are read flat.
const MAX_TABLE_DEPTH: usize = 8;
/// Paragraphs inside paragraphs (text boxes) and hyperlinks inside hyperlinks.
const MAX_NEST: usize = 32;
const MAX_ENTRIES: usize = 100_000;

pub struct DocxDoc {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

#[derive(Default, Clone)]
struct Style {
    name: String,
    outline: Option<u8>,
    based_on: Option<String>,
}

fn xml_of(a: &Archive, name: &str, budget: &Counter<u64>) -> Option<String> {
    let i = a.find(name)?;
    a.read(i, MAX_XML, budget).ok().map(|b| super::decode_text(&b))
}

fn styles(xml: &str) -> HashMap<String, Style> {
    let mut map = HashMap::new();
    let mut cur: Option<(String, Style)> = None;
    for t in Tokenizer::xml(xml) {
        match t {
            Token::Start { name, attrs, .. } => match local(&name) {
                "style" => cur = attr(&attrs, "w:styleId").filter(|_| map.len() < MAX_ENTRIES).map(|id| (id.to_string(), Style::default())),
                "name" => {
                    if let Some((_, s)) = cur.as_mut() {
                        s.name = attr(&attrs, "w:val").unwrap_or("").to_ascii_lowercase();
                    }
                }
                "outlineLvl" => {
                    if let Some((_, s)) = cur.as_mut() {
                        s.outline = attr(&attrs, "w:val").and_then(|v| v.parse().ok());
                    }
                }
                "basedOn" => {
                    if let Some((_, s)) = cur.as_mut() {
                        s.based_on = attr(&attrs, "w:val").map(str::to_string);
                    }
                }
                _ => {}
            },
            Token::End { name } if local(&name) == "style" => {
                if let Some((id, s)) = cur.take() {
                    map.insert(id, s);
                }
            }
            _ => {}
        }
    }
    map
}

/// Levels 0 to 8 of a list: ordered (`Some(true)`), bullets (`Some(false)`) or not defined.
type Levels = [Option<bool>; 9];

/// numId -> levels.
fn numbering(xml: &str) -> HashMap<String, Levels> {
    let mut abstracts: HashMap<String, Levels> = HashMap::new();
    let mut nums: HashMap<String, String> = HashMap::new();
    let mut cur_abs: Option<String> = None;
    let mut cur_lvl: Option<u32> = None;
    let mut cur_num: Option<String> = None;
    for t in Tokenizer::xml(xml) {
        match t {
            Token::Start { name, attrs, .. } => match local(&name) {
                "abstractNum" => cur_abs = attr(&attrs, "w:abstractNumId").map(str::to_string),
                "lvl" => cur_lvl = attr(&attrs, "w:ilvl").and_then(|v| v.parse().ok()),
                "numFmt" => {
                    if let (Some(a), Some(l)) = (&cur_abs, cur_lvl) {
                        let ordered = attr(&attrs, "w:val").is_some_and(|v| v != "bullet" && v != "none");
                        if (l as usize) < 9 && (abstracts.len() < MAX_ENTRIES || abstracts.contains_key(a)) {
                            abstracts.entry(a.clone()).or_default()[l as usize] = Some(ordered);
                        }
                    }
                }
                "num" => cur_num = attr(&attrs, "w:numId").map(str::to_string),
                "abstractNumId" => {
                    if let (Some(n), Some(v)) = (&cur_num, attr(&attrs, "w:val")) {
                        if nums.len() < MAX_ENTRIES {
                            nums.insert(n.clone(), v.to_string());
                        }
                    }
                }
                _ => {}
            },
            Token::End { name } => match local(&name) {
                "abstractNum" => cur_abs = None,
                "lvl" => cur_lvl = None,
                "num" => cur_num = None,
                _ => {}
            },
            _ => {}
        }
    }
    nums.into_iter().filter_map(|(n, a)| abstracts.get(&a).map(|l| (n, *l))).collect()
}

/// Relationship id -> (target, external?).
fn rels(xml: &str) -> HashMap<String, (String, bool)> {
    let mut map = HashMap::new();
    for t in Tokenizer::xml(xml) {
        if let Token::Start { name, attrs, .. } = t {
            if local(&name) == "Relationship" && map.len() < MAX_ENTRIES {
                if let (Some(id), Some(target)) = (attr(&attrs, "Id"), attr(&attrs, "Target")) {
                    let external = attr(&attrs, "TargetMode").is_some_and(|m| m.eq_ignore_ascii_case("external"));
                    map.insert(id.to_string(), (target.to_string(), external));
                }
            }
        }
    }
    map
}

fn on(attrs: &[(String, String)]) -> bool {
    !matches!(attr(attrs, "w:val"), Some("0" | "false" | "off" | "none"))
}

/// Word's highlight names, as the nearest Notaya highlight color.
fn word_highlight(v: &str) -> Option<String> {
    let i = match v {
        "none" => return None,
        "lightGray" | "darkGray" | "black" => 0,
        "darkYellow" => 1,
        // Word's yellow is Butter, the palette's yellow (not Honey, which reads as orange-brown).
        "yellow" => return Some(palette::HIGHLIGHTS[9].1.to_string()),
        "green" | "darkGreen" => 4,
        "cyan" | "blue" | "darkBlue" | "darkCyan" => 5,
        "darkMagenta" => 6,
        "magenta" => 7,
        "red" | "darkRed" => 8,
        _ => return Some(String::new()),
    };
    Some(HIGHLIGHTS[i].to_string())
}

/// A run's shading fill (`<w:shd w:fill="RRGGBB"/>`, how Notaya's own Word export writes a
/// highlight: the palette color blended onto white) as a Notaya highlight: the exact palette entry
/// for a value Notaya wrote, otherwise the nearest one. Fills that are not a highlight are left
/// out (`auto`, white, the light gray of inline code and Word's light shading, dark fills); see
/// `palette::highlight`.
fn shading_highlight(fill: &str) -> Option<String> {
    if fill.len() != 6 {
        return None;
    }
    palette::parse(fill).and_then(palette::highlight)
}

/// Paragraph styles that hold a code block (Notaya's own, its older name, and Word's usual ones).
fn is_code_style(p: &Para, styles: &HashMap<String, Style>) -> bool {
    style_is(p, styles, &["notayacode", "notaya code", "code", "source code", "html preformatted"])
}

fn is_caption(p: &Para, styles: &HashMap<String, Style>) -> bool {
    style_is(p, styles, &["caption"])
}

/// The paragraph's style id or style name (lower case) is one of `names`.
fn style_is(p: &Para, styles: &HashMap<String, Style>, names: &[&str]) -> bool {
    let Some(id) = &p.style else { return false };
    let id_low = id.to_ascii_lowercase();
    names.contains(&id_low.as_str()) || styles.get(id).is_some_and(|s| names.contains(&s.name.as_str()))
}

/// A paragraph that starts with a check box (☐ open, ☑ or ☒ done; how Notaya's export and Word's
/// check box content control write a task): whether it is ticked, and the text after the box.
fn task_item(inl: &[Inline]) -> Option<(bool, Vec<Inline>)> {
    let first = inl.iter().position(|i| !matches!(i, Inline::Text(t, _) if t.trim().is_empty()))?;
    let Inline::Text(t, m) = &inl[first] else { return None };
    let t = t.trim_start();
    let c = t.chars().next().filter(|c| matches!(c, '☐' | '☑' | '☒'))?;
    let rest = t[c.len_utf8()..].trim_start_matches([' ', '\u{a0}']);
    let mut out = Vec::with_capacity(inl.len());
    if !rest.is_empty() {
        out.push(Inline::Text(rest.to_string(), m.clone()));
    }
    out.extend(inl[first + 1..].iter().cloned());
    // The text after the box may start in the next run.
    if let Some(Inline::Text(t, _)) = out.first_mut() {
        *t = t.trim_start_matches([' ', '\u{a0}']).to_string();
    }
    Some((c != '☐', out))
}

/// The text of a code paragraph: line breaks are new lines, tabs stay tabs.
fn code_text(inl: &[Inline]) -> String {
    let mut s = String::new();
    for i in inl {
        match i {
            Inline::Text(t, _) => s.push_str(t),
            Inline::Break => s.push('\n'),
            _ => {}
        }
    }
    s
}

#[derive(Default)]
struct Para {
    style: Option<String>,
    num: Option<(String, u32)>,
    outline: Option<u8>,
    inl: Vec<Inline>,
    /// A code block starts here (Notaya's hidden bookmark), with this language.
    code_start: Option<String>,
    /// A right-to-left paragraph (`w:bidi`).
    bidi: bool,
}

enum Item {
    Para(Para),
    Block(Block),
}

#[derive(Default)]
struct TableState {
    rows: Vec<Vec<Cell>>,
    row: Vec<Cell>,
    header_row: bool,
    row_count: usize,
    /// The table style (`NotayaShelf` marks a shelf).
    style: Option<String>,
    /// A table drawn right-to-left (`w:bidiVisual`).
    bidi: bool,
}

struct CellState {
    items: Vec<Item>,
    colspan: u32,
}

pub fn parse(bytes: Vec<u8>, ctx: &mut Ctx, key: &str) -> Result<DocxDoc, String> {
    let a = Archive::open(bytes).map_err(|_| "This is not a Word document (.docx), or it is damaged. Older .doc files are not supported; save them as .docx first.".to_string())?;
    // One allowance for the whole import, shared with every other file in it.
    let budget = ctx.vfs.budget();
    let document = xml_of(&a, "word/document.xml", budget).ok_or("This is not a Word document (.docx), or it is damaged.")?;
    let styles = xml_of(&a, "word/styles.xml", budget).map(|x| styles(&x)).unwrap_or_default();
    let numbering = xml_of(&a, "word/numbering.xml", budget).map(|x| numbering(&x)).unwrap_or_default();
    let rels = xml_of(&a, "word/_rels/document.xml.rels", budget).map(|x| rels(&x)).unwrap_or_default();
    let core_title = xml_of(&a, "docProps/core.xml", budget).and_then(|x| {
        let mut in_title = false;
        let mut t = String::new();
        for tok in Tokenizer::xml(&x) {
            match tok {
                Token::Start { name, .. } if local(&name) == "title" => in_title = true,
                Token::End { name } if local(&name) == "title" => in_title = false,
                Token::Text(s) if in_title => t.push_str(&s),
                _ => {}
            }
        }
        let t = doc::collapse_ws(&t);
        (!t.is_empty()).then_some(t)
    });

    let mut body: Vec<Item> = Vec::new();
    let mut tables: Vec<TableState> = Vec::new();
    let mut cells: Vec<CellState> = Vec::new();
    let mut paras: Vec<Para> = Vec::new();
    // The link in force at each hyperlink level (a level with no link of its own keeps the one
    // around it, so a run finds its link in one step however deep the hyperlinks nest).
    let mut links: Vec<Option<String>> = Vec::new();
    let mut link_over = 0usize;
    let mut para_over = 0usize;
    // Tables deeper than MAX_TABLE_DEPTH: their rows and cells are not structure, only their text counts.
    let mut flat = 0usize;
    let mut cell_count = 0usize;
    let mut cells_cut = false;
    let mut pics: HashMap<String, Inline> = HashMap::new();
    let mut run: Option<Marks> = None;
    let (mut in_t, mut in_ppr, mut in_rpr) = (false, false, false);
    // Inside a tracked deletion or another part whose text is not shown.
    let mut skip = 0usize;
    // Complex fields: the instruction between `begin` and `separate` is not shown.
    let mut field_instr = 0usize;

    fn push_item(item: Item, cells: &mut [CellState], body: &mut Vec<Item>) {
        match cells.last_mut() {
            Some(c) => c.items.push(item),
            None => body.push(item),
        }
    }

    for tok in Tokenizer::xml(&document) {
        if ctx.ctl.stopped() {
            break;
        }
        match tok {
            Token::Start { name, attrs, self_closing } => {
                let n = local(&name);
                if skip > 0 {
                    if !self_closing {
                        skip += 1;
                    }
                    continue;
                }
                match n {
                    "del" | "moveFrom" | "instrText" | "delText" | "footnoteReference" | "endnoteReference" | "commentReference" | "sdtPr" | "rPrChange" | "pPrChange" => {
                        if !self_closing {
                            skip = 1;
                        }
                    }
                    "p" if paras.len() >= MAX_NEST => para_over += 1,
                    "p" => paras.push(Para::default()),
                    "pPr" => in_ppr = !self_closing,
                    "rPr" => in_rpr = !self_closing,
                    "pStyle" if in_ppr => {
                        if let Some(p) = paras.last_mut() {
                            p.style = attr(&attrs, "w:val").map(str::to_string);
                        }
                    }
                    "bidi" if in_ppr => {
                        if let Some(p) = paras.last_mut() {
                            p.bidi = on(&attrs);
                        }
                    }
                    "bidiVisual" if flat == 0 => {
                        if let Some(t) = tables.last_mut() {
                            t.bidi = on(&attrs);
                        }
                    }
                    "outlineLvl" if in_ppr => {
                        if let Some(p) = paras.last_mut() {
                            p.outline = attr(&attrs, "w:val").and_then(|v| v.parse().ok());
                        }
                    }
                    "ilvl" if in_ppr => {
                        if let Some(p) = paras.last_mut() {
                            let lvl = attr(&attrs, "w:val").and_then(|v| v.parse().ok()).unwrap_or(0u32).min(8);
                            let id = p.num.take().map(|(id, _)| id).unwrap_or_default();
                            p.num = Some((id, lvl));
                        }
                    }
                    "numId" if in_ppr => {
                        if let Some(p) = paras.last_mut() {
                            let id = attr(&attrs, "w:val").unwrap_or("0").to_string();
                            let lvl = p.num.take().map(|(_, l)| l).unwrap_or(0);
                            p.num = Some((id, lvl));
                        }
                    }
                    "hyperlink" if links.len() >= MAX_NEST => link_over += 1,
                    "hyperlink" => {
                        let target = attr(&attrs, "r:id")
                            .and_then(|id| rels.get(id))
                            .filter(|(_, external)| *external)
                            .and_then(|(t, _)| safe_link(t).or_else(|| ctx.page_href(t)))
                            .or_else(|| links.last().cloned().flatten());
                        links.push(target);
                    }
                    "r" if !in_ppr => {
                        run = Some(Marks { link: links.last().cloned().flatten(), ..Marks::default() });
                    }
                    "b" | "bCs" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            m.bold = on(&attrs);
                        }
                    }
                    "i" | "iCs" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            m.italic = on(&attrs);
                        }
                    }
                    "u" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            m.underline = on(&attrs);
                        }
                    }
                    "strike" | "dstrike" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            m.strike = on(&attrs);
                        }
                    }
                    // Run shading only (inside run properties); paragraph and cell shading are not read.
                    "shd" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            if m.highlight.is_none() {
                                m.highlight = attr(&attrs, "w:fill").and_then(shading_highlight);
                            }
                        }
                    }
                    "highlight" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            m.highlight = attr(&attrs, "w:val").and_then(word_highlight);
                        }
                    }
                    "color" if in_rpr => {
                        if let Some(m) = run.as_mut() {
                            let v = attr(&attrs, "w:val").unwrap_or("auto");
                            if v.len() == 6 && v.chars().all(|c| c.is_ascii_hexdigit()) && v != "000000" {
                                m.color = Some(format!("#{}", v.to_ascii_lowercase()));
                            }
                        }
                    }
                    "t" if !self_closing => in_t = true,
                    // A tab: a space in text (the page writer turns it into one), a tab in code.
                    "tab" if !in_ppr && run.is_some() => {
                        if let (Some(p), Some(m)) = (paras.last_mut(), run.as_ref()) {
                            p.inl.push(Inline::Text("\t".into(), m.clone()));
                        }
                    }
                    // Notaya's hidden bookmark at the start of a code block: `_NotayaCode<n>_<language>`.
                    "bookmarkStart" => {
                        if let (Some(p), Some(rest)) = (paras.last_mut(), attr(&attrs, "w:name").and_then(|n| n.strip_prefix(CODE_BOOKMARK))) {
                            let lang = rest.split_once('_').map_or("", |(_, l)| l);
                            // `_` stands for `-` (bookmark names cannot hold a hyphen).
                            p.code_start = Some(lang.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').map(|c| if c == '_' { '-' } else { c }).take(32).collect());
                        }
                    }
                    "tblStyle" if flat == 0 => {
                        if let Some(t) = tables.last_mut() {
                            t.style = attr(&attrs, "w:val").map(|v| v.chars().take(64).collect());
                        }
                    }
                    "br" | "cr" if run.is_some() => {
                        let page = attr(&attrs, "w:type").is_some_and(|t| t == "page" || t == "column");
                        if let (Some(p), false) = (paras.last_mut(), page) {
                            p.inl.push(Inline::Break);
                        }
                    }
                    "fldChar" => match attr(&attrs, "w:fldCharType") {
                        Some("begin") => field_instr += 1,
                        Some("separate") | Some("end") => field_instr = field_instr.saturating_sub(1),
                        _ => {}
                    },
                    "blip" | "imagedata" => {
                        let id = attr(&attrs, "r:embed").or_else(|| attr(&attrs, "r:id"));
                        if let Some((target, false)) = id.and_then(|id| rels.get(id)).cloned() {
                            let path = super::resolve_rel("word", &target).unwrap_or_default();
                            let label = target.rsplit('/').next().unwrap_or("picture").to_string();
                            // A picture used again and again is read (and reported) once.
                            let inl = match pics.get(&path) {
                                Some(known) => known.clone(),
                                None => {
                                    let made = match a.find(&path).map(|i| a.read(i, super::MAX_IMAGE, budget)) {
                                        Some(Ok(bytes)) => match ctx.images.bytes(&format!("{key}:{path}"), bytes, &label) {
                                            Some(i) => Inline::Image(i),
                                            None => {
                                                ctx.problem(&label, "Only PNG, JPEG, GIF and WebP pictures are imported (this one is another format or too large).");
                                                Inline::Text(format!("[Picture not imported: {label}]"), Marks { italic: true, ..Marks::default() })
                                            }
                                        },
                                        _ => {
                                            ctx.problem(&label, "The picture could not be read from the document.");
                                            Inline::Text(format!("[Picture not imported: {label}]"), Marks { italic: true, ..Marks::default() })
                                        }
                                    };
                                    if pics.len() < MAX_ENTRIES {
                                        pics.insert(path.clone(), made.clone());
                                    }
                                    made
                                }
                            };
                            if let Some(p) = paras.last_mut() {
                                p.inl.push(inl);
                            }
                        }
                    }
                    "tbl" if flat > 0 || tables.len() >= MAX_TABLE_DEPTH => flat += 1,
                    "tbl" => tables.push(TableState::default()),
                    "tr" | "tc" | "tblHeader" | "gridSpan" if flat > 0 => {}
                    "tr" => {
                        if let Some(t) = tables.last_mut() {
                            t.row.clear();
                            t.header_row = false;
                        }
                    }
                    "tblHeader" => {
                        if let Some(t) = tables.last_mut() {
                            t.header_row = on(&attrs);
                        }
                    }
                    "tc" => cells.push(CellState { items: Vec::new(), colspan: 1 }),
                    "gridSpan" => {
                        if let Some(c) = cells.last_mut() {
                            c.colspan = attr(&attrs, "w:val").and_then(|v| v.parse().ok()).unwrap_or(1u32).clamp(1, 50);
                        }
                    }
                    _ => {}
                }
            }
            Token::Text(s) => {
                if skip == 0 && in_t && field_instr == 0 {
                    if let (Some(p), Some(m)) = (paras.last_mut(), run.as_ref()) {
                        p.inl.push(Inline::Text(s, m.clone()));
                    }
                }
            }
            Token::End { name } => {
                let n = local(&name);
                if skip > 0 {
                    skip -= 1;
                    continue;
                }
                match n {
                    "t" => in_t = false,
                    "pPr" => in_ppr = false,
                    "rPr" => in_rpr = false,
                    "r" => run = None,
                    "hyperlink" if link_over > 0 => link_over -= 1,
                    "hyperlink" => {
                        links.pop();
                    }
                    "p" if para_over > 0 => para_over -= 1,
                    "p" => {
                        if let Some(mut p) = paras.pop() {
                            // A right-to-left paragraph: its text runs carry the direction to the page writer.
                            if p.bidi {
                                for i in p.inl.iter_mut() {
                                    if let Inline::Text(_, m) = i {
                                        m.dir = Dir::Rtl;
                                    }
                                }
                            }
                            push_item(Item::Para(p), &mut cells, &mut body);
                        }
                    }
                    "tc" | "tr" if flat > 0 => {}
                    "tc" => {
                        if let (Some(c), Some(t)) = (cells.pop(), tables.last_mut()) {
                            let header = t.header_row;
                            if cell_count >= doc::MAX_PAGE_CELLS {
                                cells_cut = true;
                            } else if t.row.len() < 100 {
                                cell_count += 1;
                                let blocks = finish(c.items, &styles, &numbering, &mut None);
                                t.row.push(Cell { header, colspan: c.colspan, blocks });
                            }
                        }
                    }
                    "tr" => {
                        if let Some(t) = tables.last_mut() {
                            let row = std::mem::take(&mut t.row);
                            if !row.is_empty() && t.rows.len() < 2_000 {
                                t.rows.push(row);
                            }
                            t.row_count += 1;
                        }
                    }
                    "tbl" if flat > 0 => flat -= 1,
                    "tbl" => {
                        if let Some(mut t) = tables.pop() {
                            // A one-cell table in Notaya's shelf style is a shelf.
                            let shelf = t.style.as_deref() == Some(SHELF_STYLE) && t.row_count == 1 && t.rows.len() == 1 && t.rows[0].len() == 1;
                            if shelf {
                                let cell = t.rows.remove(0).remove(0);
                                push_item(Item::Block(Block::Shelf(cell.blocks)), &mut cells, &mut body);
                            } else if !t.rows.is_empty() {
                                let mut table = vec![Block::Table(t.rows)];
                                if t.bidi {
                                    doc::force_first_rtl(&mut table);
                                }
                                if let Some(table) = table.pop() {
                                    push_item(Item::Block(table), &mut cells, &mut body);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if cells_cut {
        ctx.problem("Tables", "This document has more table cells than are imported; the rest were left out.");
    }
    let mut title = None;
    let blocks = finish(body, &styles, &numbering, &mut title);
    Ok(DocxDoc { title: title.or(core_title), blocks })
}

/// The heading level, or 0 for "title", from the paragraph's style.
fn heading_level(p: &Para, styles: &HashMap<String, Style>) -> Option<u8> {
    if let Some(o) = p.outline.filter(|o| *o < 9) {
        return Some(o + 1);
    }
    let mut id = p.style.clone()?;
    for _ in 0..8 {
        let st = styles.get(&id).cloned().unwrap_or_default();
        let name = if st.name.is_empty() { id.to_ascii_lowercase() } else { st.name.clone() };
        if name == "title" {
            return Some(0);
        }
        if let Some(rest) = name.strip_prefix("heading").map(str::trim) {
            if let Ok(n) = rest.parse::<u8>() {
                return Some(n.clamp(1, 9));
            }
        }
        if let Some(o) = st.outline.filter(|o| *o < 9) {
            return Some(o + 1);
        }
        id = st.based_on.clone()?;
    }
    None
}

fn is_quote(p: &Para, styles: &HashMap<String, Style>) -> bool {
    let Some(id) = &p.style else { return false };
    let name = styles.get(id).map(|s| s.name.clone()).unwrap_or_else(|| id.to_ascii_lowercase());
    name == "quote" || name == "intense quote"
}

/// Paragraph records to blocks: headings, quotes, nested lists, code blocks (consecutive code
/// paragraphs, a new block at each of Notaya's code bookmarks), task lists (paragraphs that
/// start with a check box) and picture captions (a "Caption" paragraph right after a picture).
fn finish(items: Vec<Item>, styles: &HashMap<String, Style>, numbering: &HashMap<String, Levels>, title: &mut Option<String>) -> Vec<Block> {
    let mut out = Vec::new();
    let mut list: Vec<(u32, bool, Vec<Inline>)> = Vec::new();
    // The code block and the task list being collected.
    let mut code: Option<(String, Vec<String>)> = None;
    let mut tasks: Vec<(bool, Vec<Block>)> = Vec::new();
    fn flush_list(list: &mut Vec<(u32, bool, Vec<Inline>)>, out: &mut Vec<Block>) {
        if !list.is_empty() {
            let base = list.iter().map(|x| x.0).min().unwrap_or(0);
            out.extend(build_list(list, base));
            list.clear();
        }
    }
    fn flush_code(code: &mut Option<(String, Vec<String>)>, out: &mut Vec<Block>) {
        if let Some((lang, lines)) = code.take() {
            out.push(Block::Code(lang, lines.join("\n")));
        }
    }
    fn flush_tasks(tasks: &mut Vec<(bool, Vec<Block>)>, out: &mut Vec<Block>) {
        if !tasks.is_empty() {
            out.push(Block::Tasks(std::mem::take(tasks)));
        }
    }
    for item in items {
        match item {
            Item::Block(b) => {
                flush_list(&mut list, &mut out);
                flush_code(&mut code, &mut out);
                flush_tasks(&mut tasks, &mut out);
                out.push(b);
            }
            Item::Para(p) => {
                if is_code_style(&p, styles) {
                    flush_list(&mut list, &mut out);
                    flush_tasks(&mut tasks, &mut out);
                    let text = code_text(&p.inl);
                    if code.is_some() && p.code_start.is_none() {
                        if let Some((_, lines)) = code.as_mut().filter(|(_, l)| l.len() < 100_000) {
                            lines.push(text);
                        }
                    } else {
                        flush_code(&mut code, &mut out);
                        code = Some((p.code_start.unwrap_or_default(), vec![text]));
                    }
                    continue;
                }
                flush_code(&mut code, &mut out);
                if let Some((id, lvl)) = p.num.as_ref().filter(|(id, _)| id != "0" && !id.is_empty()) {
                    flush_tasks(&mut tasks, &mut out);
                    let ordered = numbering.get(id).and_then(|l| l.get(*lvl as usize)).copied().flatten().unwrap_or(false);
                    if list.len() < 10_000 {
                        list.push((*lvl, ordered, p.inl));
                    }
                    continue;
                }
                flush_list(&mut list, &mut out);
                if heading_level(&p, styles).is_none() {
                    if let Some((checked, inl)) = task_item(&p.inl) {
                        tasks.push((checked, vec![Block::Paragraph(inl)]));
                        continue;
                    }
                }
                flush_tasks(&mut tasks, &mut out);
                if is_caption(&p, styles) {
                    // The caption of the picture just before it.
                    if let Some(Block::Paragraph(prev)) = out.last() {
                        let mut seen = prev.iter().filter(|i| !matches!(i, Inline::Text(t, _) if t.trim().is_empty()));
                        if let (Some(Inline::Image(idx)), None) = (seen.next(), seen.next()) {
                            let idx = *idx;
                            *out.last_mut().unwrap() = Block::Image(idx, doc::plain(&p.inl));
                            continue;
                        }
                    }
                }
                match heading_level(&p, styles) {
                    Some(0) if title.is_none() && out.is_empty() => *title = Some(doc::plain(&p.inl)).filter(|t| !t.is_empty()),
                    Some(0) => out.push(Block::Heading(1, p.inl)),
                    Some(level) => out.push(Block::Heading(level.min(4), p.inl)),
                    None if is_quote(&p, styles) => out.push(Block::Quote(vec![Block::Paragraph(p.inl)])),
                    None => out.push(Block::Paragraph(p.inl)),
                }
            }
        }
    }
    flush_list(&mut list, &mut out);
    flush_code(&mut code, &mut out);
    flush_tasks(&mut tasks, &mut out);
    out
}

/// Builds nested lists from (level, ordered, text) lines.
fn build_list(lines: &[(u32, bool, Vec<Inline>)], base: u32) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut items: Vec<Vec<Block>> = Vec::new();
    let mut kind: Option<bool> = None;
    let mut i = 0;
    let push = |out: &mut Vec<Block>, items: &mut Vec<Vec<Block>>, kind: Option<bool>| {
        if !items.is_empty() {
            let it = std::mem::take(items);
            out.push(if kind == Some(true) { Block::Numbered(1, it) } else { Block::Bullets(it) });
        }
    };
    while i < lines.len() {
        let (lvl, ordered, inl) = &lines[i];
        if *lvl <= base || items.is_empty() {
            if kind.is_some_and(|k| k != *ordered) {
                push(&mut out, &mut items, kind);
            }
            kind = Some(*ordered);
            items.push(vec![Block::Paragraph(inl.clone())]);
            i += 1;
        } else {
            // Deeper lines belong to the last item.
            let start = i;
            while i < lines.len() && lines[i].0 > base {
                i += 1;
            }
            let child_base = lines[start..i].iter().map(|x| x.0).min().unwrap_or(base + 1);
            let nested = build_list(&lines[start..i], child_base);
            if let Some(last) = items.last_mut() {
                last.extend(nested);
            }
        }
    }
    push(&mut out, &mut items, kind);
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::importers::html::tests::{json, with_ctx};
    use crate::importers::zip;

    const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>
<w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr><w:r><w:t>Quarterly report</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Summary</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r><w:r><w:t xml:space="preserve"> and </w:t></w:r><w:r><w:rPr><w:i/><w:u w:val="single"/><w:strike/><w:color w:val="C00000"/><w:highlight w:val="yellow"/></w:rPr><w:t>styled</w:t></w:r><w:r><w:rPr><w:b w:val="0"/></w:rPr><w:t xml:space="preserve"> plain</w:t></w:r><w:r><w:br/><w:t>next line</w:t></w:r></w:p>
<w:p><w:hyperlink r:id="rId5"><w:r><w:t>a link</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> </w:t></w:r><w:hyperlink r:id="rId6"><w:r><w:t>bad link</w:t></w:r></w:hyperlink></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>first</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>nested</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>second</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="2"/></w:numPr></w:pPr><w:r><w:t>step one</w:t></w:r></w:p>
<w:tbl><w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>Wide</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
<w:p><w:r><w:drawing><wp:inline><a:graphic><a:graphicData><pic:pic><pic:blipFill><a:blip r:embed="rId7"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>
<w:p><w:r><w:drawing><a:blip r:embed="rId8"/></w:drawing></w:r></w:p>
<w:p><w:del><w:r><w:delText>deleted</w:delText></w:r></w:del><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>HYPERLINK "x"</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>shown</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Quote"/></w:pPr><w:r><w:t>Said</w:t></w:r></w:p>
</w:body></w:document>"#;

    const STYLES: &str = r#"<w:styles xmlns:w="w"><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/></w:style><w:style w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:styleId="Quote"><w:name w:val="Quote"/></w:style></w:styles>"#;
    const NUMBERING: &str = r#"<w:numbering xmlns:w="w"><w:abstractNum w:abstractNumId="10"><w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl><w:lvl w:ilvl="1"><w:numFmt w:val="bullet"/></w:lvl></w:abstractNum><w:abstractNum w:abstractNumId="11"><w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="10"/></w:num><w:num w:numId="2"><w:abstractNumId w:val="11"/></w:num></w:numbering>"#;
    const RELS: &str = r#"<Relationships><Relationship Id="rId5" Type="hyperlink" Target="https://example.com/" TargetMode="External"/><Relationship Id="rId6" Type="hyperlink" Target="javascript:alert(1)" TargetMode="External"/><Relationship Id="rId7" Type="image" Target="media/image1.png"/><Relationship Id="rId8" Type="image" Target="media/image2.emf"/></Relationships>"#;

    pub fn sample_docx() -> Vec<u8> {
        zip::build(
            &[
                ("[Content_Types].xml", b"<Types/>"),
                ("word/document.xml", DOC.as_bytes()),
                ("word/styles.xml", STYLES.as_bytes()),
                ("word/numbering.xml", NUMBERING.as_bytes()),
                ("word/_rels/document.xml.rels", RELS.as_bytes()),
                ("word/media/image1.png", b"\x89PNG\r\n\x1a\nimage"),
                ("word/media/image2.emf", b"\x01\0\0\0emf"),
                ("docProps/core.xml", b"<cp:coreProperties><dc:title>Old core title</dc:title></cp:coreProperties>"),
            ],
            true,
        )
    }

    #[test]
    fn word_right_to_left_paragraphs_and_tables_keep_their_direction() {
        use crate::importers::html::tests::dirs_of;
        let body = r#"
            <w:p><w:pPr><w:bidi/></w:pPr><w:r><w:t>English bidi</w:t></w:r></w:p>
            <w:p><w:pPr><w:bidi/><w:jc w:val="left"/></w:pPr><w:r><w:rPr><w:rtl/></w:rPr><w:t>مرحبا</w:t></w:r></w:p>
            <w:p><w:pPr><w:bidi w:val="0"/></w:pPr><w:r><w:t>Not bidi</w:t></w:r></w:p>
            <w:p><w:r><w:t>مرحبا بلا bidi</w:t></w:r></w:p>
            <w:p><w:pPr><w:bidi/></w:pPr><w:r><w:t>123</w:t></w:r></w:p>
            <w:tbl><w:tblPr><w:bidiVisual/></w:tblPr><w:tr><w:tc><w:p><w:r><w:t>First cell</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Second</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let (d, _, problems) = with_ctx(&[], |ctx| parse(docx_with_body(body), ctx, "bidi.docx").unwrap());
        assert!(problems.is_empty(), "{problems:?}");
        let got = dirs_of(&json(&d.blocks));
        let find = |t: &str| got.iter().find(|(x, _)| x == t).unwrap_or_else(|| panic!("{t} in {got:?}")).1.clone();
        // A right-to-left paragraph with English text keeps its direction; Arabic gets it from the text.
        assert_eq!(find("English bidi").as_deref(), Some("rtl"));
        assert_eq!(find("مرحبا"), None);
        assert_eq!(find("Not bidi"), None);
        assert_eq!(find("مرحبا بلا bidi"), None, "no bidi in Word: shown by its text, so it is right-aligned here");
        assert_eq!(find("123").as_deref(), Some("rtl"));
        // A table drawn right-to-left: its first text is made right-to-left so the columns run from the right.
        assert_eq!(find("First cell").as_deref(), Some("rtl"));
        assert_eq!(find("Second"), None);
    }

    #[test]
    fn a_word_document_keeps_its_structure() {
        let (d, images, problems) = with_ctx(&[], |ctx| parse(sample_docx(), ctx, "r.docx").unwrap());
        assert_eq!(d.title.as_deref(), Some("Quarterly report"), "the Title style wins over core.xml");
        assert_eq!(images.list.len(), 1);
        assert_eq!(problems.len(), 1, "the EMF picture is reported");
        let v = json(&d.blocks);
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["heading", "paragraph", "paragraph", "bulletList", "orderedList", "table", "image", "paragraph", "paragraph", "blockquote"]);
        let s = c[1].to_string();
        for m in ["bold", "italic", "underline", "strike", "highlight", "textStyle", "hardBreak"] {
            assert!(s.contains(m), "{m}");
        }
        assert!(s.contains("#c00000") && s.contains(palette::HIGHLIGHTS[9].1), "Word's yellow is Butter");
        assert!(c[2].to_string().contains("https://example.com/") && !c[2].to_string().contains("javascript"));
        assert_eq!(c[3]["content"].as_array().unwrap().len(), 2);
        assert_eq!(c[3]["content"][0]["content"][1]["type"], "bulletList", "level 1 nests under the first item");
        assert_eq!(c[5]["content"][0]["content"][0]["type"], "tableHeader");
        assert_eq!(c[5]["content"][0]["content"][0]["attrs"]["colspan"], 2);
        assert!(c[7].to_string().contains("Picture not imported"));
        assert_eq!(c[8]["content"][0]["text"], "shown");
        assert!(!v.to_string().contains("deleted") && !v.to_string().contains("HYPERLINK"));
    }

    #[test]
    fn run_shading_from_notaya_s_own_export_is_a_highlight() {
        let body = concat!(
            r#"<w:p><w:r><w:rPr><w:shd w:val="clear" w:color="auto" w:fill="F8E6AD"/></w:rPr><w:t>marked</w:t></w:r>"#,
            r#"<w:r><w:rPr><w:shd w:val="clear" w:color="auto" w:fill="F1F3F5"/></w:rPr><w:t>code</w:t></w:r>"#,
            r#"<w:r><w:rPr><w:shd w:fill="D9D9D8"/></w:rPr><w:t>gray</w:t></w:r>"#,
            r#"<w:r><w:rPr><w:shd w:fill="auto"/></w:rPr><w:t>auto</w:t></w:r>"#,
            r#"<w:r><w:rPr><w:shd w:fill="FFFFFF"/></w:rPr><w:t>white</w:t></w:r>"#,
            r#"<w:r><w:rPr><w:shd w:fill="1F2937"/></w:rPr><w:t>dark</w:t></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:shd w:val="clear" w:fill="F8E6AD"/></w:pPr><w:r><w:t>paragraph</w:t></w:r></w:p>"#,
            r#"<w:tbl><w:tr><w:tc><w:tcPr><w:shd w:fill="F8E6AD"/></w:tcPr><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
        );
        let (d, _, _) = with_ctx(&[], |ctx| parse(docx_with_body(body), ctx, "a.docx").unwrap());
        let marks = |word: &str| -> Option<Marks> {
            fn find(inl: &[Inline], word: &str) -> Option<Marks> {
                inl.iter().find_map(|i| if let Inline::Text(t, m) = i { (t == word).then(|| m.clone()) } else { None })
            }
            d.blocks.iter().find_map(|b| match b {
                Block::Paragraph(i) => find(i, word),
                Block::Table(rows) => rows.iter().flatten().flat_map(|c| &c.blocks).find_map(|b| if let Block::Paragraph(i) = b { find(i, word) } else { None }),
                _ => None,
            })
        };
        let hl = marks("marked").unwrap().highlight;
        assert!(hl.as_deref().is_some_and(|h| HIGHLIGHTS.contains(&h)), "{hl:?}");
        // The light gray Notaya's own gray highlights are written as comes back as a gray highlight.
        let gray = marks("gray").unwrap().highlight;
        assert!(gray.as_deref().is_some_and(|h| HIGHLIGHTS.contains(&h)), "{gray:?}");
        for w in ["code", "auto", "white", "dark", "paragraph", "cell"] {
            assert_eq!(marks(w).unwrap().highlight, None, "{w}");
        }
    }

    /// A page through Notaya's own Word export and back: code blocks, task lists, captions,
    /// shelves and every palette color come back as they were.
    #[test]
    fn notaya_s_own_word_export_comes_back() {
        use crate::export::doc::{parse as parse_page, tests::IMG_ID};
        use crate::importers::html::tests::colors_of;
        use serde_json::json;
        let mut colored = Vec::new();
        for (i, (_, c)) in palette::TEXT_COLORS.iter().enumerate() {
            colored.push(json!({ "type": "text", "text": format!("tc{i}"), "marks": [{ "type": "textStyle", "attrs": { "color": c } }] }));
            colored.push(json!({ "type": "text", "text": " " }));
        }
        for (i, (_, h)) in palette::HIGHLIGHTS.iter().enumerate() {
            colored.push(json!({ "type": "text", "text": format!("hl{i}"), "marks": [{ "type": "highlight", "attrs": { "color": h } }] }));
            colored.push(json!({ "type": "text", "text": " " }));
        }
        colored.push(json!({ "type": "text", "text": "dflt", "marks": [{ "type": "highlight", "attrs": { "color": null } }] }));
        colored.push(json!({ "type": "text", "text": " and " }));
        colored.push(json!({ "type": "text", "text": "inlinecode", "marks": [{ "type": "code" }] }));
        let page = json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": colored },
            { "type": "codeBlock", "attrs": { "language": "rust" }, "content": [{ "type": "text", "text": "fn main() {\n    let x = 1;\n\tx\n}" }] },
            { "type": "codeBlock", "attrs": { "language": "php-template" }, "content": [{ "type": "text", "text": "second block" }] },
            { "type": "codeBlock", "attrs": { "language": null }, "content": [{ "type": "text", "text": "third block" }] },
            { "type": "taskList", "content": [
                { "type": "taskItem", "attrs": { "checked": true }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Done task" }] }] },
                { "type": "taskItem", "attrs": { "checked": false }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Open task" }] }] }
            ] },
            { "type": "image", "attrs": { "src": format!("http://clatt.localhost/{IMG_ID}"), "caption": "A caption" } },
            { "type": "paragraph", "content": [{ "type": "text", "text": "After the picture" }] },
            { "type": "toggle", "attrs": { "open": true, "shelf": true }, "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "Shelf title" }] },
                { "type": "paragraph", "content": [{ "type": "text", "text": "Inside the shelf" }] }
            ] },
            { "type": "toggle", "attrs": { "open": true }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Plain toggle" }] }] }
        ] });
        let doc = crate::export::tests::sample_doc(parse_page(&page.to_string(), &|_| None));
        let bytes = crate::export::docx::render_at(&doc, 0).unwrap();
        let (d, images, problems) = with_ctx(&[], |ctx| parse(bytes, ctx, "page.docx").unwrap());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(images.list.len(), 1);
        let v = json(&d.blocks);
        for (i, (name, c)) in palette::TEXT_COLORS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("tc{i}")).0.as_deref(), Some(*c), "text color {name}");
        }
        for (i, (name, h)) in palette::HIGHLIGHTS.iter().enumerate() {
            assert_eq!(colors_of(&v, &format!("hl{i}")).1.as_deref(), Some(*h), "highlight {name}");
        }
        assert_eq!(colors_of(&v, "dflt").1.as_deref(), Some(""), "the default highlight");
        assert_eq!(colors_of(&v, "inlinecode"), (None, None), "inline code's gray is no highlight");
        let c = v["content"].as_array().unwrap();
        let types: Vec<&str> = c.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["heading", "paragraph", "codeBlock", "codeBlock", "codeBlock", "taskList", "image", "paragraph", "toggle", "paragraph"], "{v}");
        assert_eq!(c[2]["attrs"]["language"], "rust");
        assert_eq!(c[2]["content"][0]["text"], "fn main() {\n    let x = 1;\n\tx\n}");
        assert_eq!(c[3]["attrs"]["language"], "php-template");
        assert_eq!(c[3]["content"][0]["text"], "second block", "code blocks next to each other stay apart");
        assert_eq!(c[4]["attrs"]["language"], serde_json::Value::Null);
        assert_eq!(c[4]["content"][0]["text"], "third block");
        assert_eq!(c[5]["content"][0]["attrs"]["checked"], true);
        assert_eq!(c[5]["content"][0]["content"][0]["content"][0]["text"], "Done task");
        assert_eq!(c[5]["content"][1]["attrs"]["checked"], false);
        assert_eq!(c[5]["content"][1]["content"][0]["content"][0]["text"], "Open task");
        assert_eq!(c[6]["attrs"]["caption"], "A caption");
        assert_eq!(c[8]["attrs"]["shelf"], true);
        assert_eq!(c[8]["content"][0]["content"][0]["text"], "Shelf title");
        assert_eq!(c[8]["content"][1]["content"][0]["text"], "Inside the shelf");
        assert!(c[9].to_string().contains("Plain toggle"), "a plain toggle stays a line of text in Word");
    }

    #[test]
    fn code_tasks_and_captions_from_other_word_files() {
        let body = concat!(
            // Word's own code styles and a user's extra line (no bookmark: it joins the block above).
            r#"<w:p><w:pPr><w:pStyle w:val="HTMLPreformatted"/></w:pPr><w:r><w:t xml:space="preserve">line 1</w:t></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:pStyle w:val="HTMLPreformatted"/></w:pPr><w:r><w:t xml:space="preserve">  line 2</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>between</w:t></w:r></w:p>"#,
            // A check box content control (its glyph in a run of its own) and a typed box.
            r#"<w:p><w:sdt><w:sdtPr><w14:checkbox/></w:sdtPr><w:sdtContent><w:r><w:t>☒</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t xml:space="preserve"> Ticked</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>☐Not yet</w:t></w:r></w:p>"#,
            // A heading that starts with a box is still a heading; a caption with no picture stays text.
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>☐ Heading</w:t></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:pStyle w:val="Caption"/></w:pPr><w:r><w:t>Lonely caption</w:t></w:r></w:p>"#,
            // Bookmarks that are not Notaya's do nothing.
            r#"<w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/><w:r><w:t>toc</w:t></w:r></w:p>"#
        );
        let styles = r#"<w:styles><w:style w:styleId="HTMLPreformatted"><w:name w:val="HTML Preformatted"/></w:style><w:style w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:styleId="Caption"><w:name w:val="caption"/></w:style></w:styles>"#;
        let xml = format!(r#"<w:document xmlns:w="w"><w:body>{body}</w:body></w:document>"#);
        let bytes = zip::build(&[("word/document.xml", xml.as_bytes()), ("word/styles.xml", styles.as_bytes())], false);
        let (d, _, _) = with_ctx(&[], |ctx| parse(bytes, ctx, "a.docx").unwrap());
        assert_eq!(d.blocks[0], Block::Code(String::new(), "line 1\n  line 2".into()));
        assert!(matches!(&d.blocks[1], Block::Paragraph(_)));
        let Block::Tasks(items) = &d.blocks[2] else { panic!("{:?}", d.blocks) };
        assert_eq!(items.iter().map(|(c, b)| (*c, text_of(b).contains(if *c { "Ticked" } else { "Not yet" }))).collect::<Vec<_>>(), [(true, true), (false, true)]);
        assert!(!text_of(&d.blocks[2..3]).contains('☒'), "the box itself is not kept as text");
        assert!(matches!(&d.blocks[3], Block::Heading(1, _)));
        assert!(matches!(&d.blocks[4], Block::Paragraph(_)));
        assert!(matches!(&d.blocks[5], Block::Paragraph(_)));
    }

    #[test]
    fn a_shelf_needs_notaya_s_style_and_one_cell() {
        let cell = |t: &str| format!("<w:tc><w:p><w:r><w:t>{t}</w:t></w:r></w:p></w:tc>");
        let body = format!(
            "<w:tbl><w:tblPr><w:tblStyle w:val=\"NotayaShelf\"/></w:tblPr><w:tr>{}</w:tr></w:tbl>\
             <w:tbl><w:tblPr><w:tblStyle w:val=\"NotayaShelf\"/></w:tblPr><w:tr>{}{}</w:tr></w:tbl>\
             <w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr><w:tr>{}</w:tr></w:tbl>",
            cell("shelf"),
            cell("a"),
            cell("b"),
            cell("plain")
        );
        let (d, _, _) = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
        assert!(matches!(&d.blocks[0], Block::Shelf(b) if text_of(b).contains("shelf")), "{:?}", d.blocks);
        assert!(matches!(&d.blocks[1], Block::Table(_)), "two cells: a table");
        assert!(matches!(&d.blocks[2], Block::Table(_)), "another style: a table");
    }

    #[test]
    fn not_a_docx() {
        let (r, _, _) = with_ctx(&[], |ctx| parse(b"not a zip".to_vec(), ctx, "x").err());
        assert!(r.unwrap().contains("not a Word document"));
        let (r, _, _) = with_ctx(&[], |ctx| parse(zip::build(&[("a.txt", b"x")], false), ctx, "x").err());
        assert!(r.is_some());
    }

    fn depth(b: &Block) -> usize {
        1 + match b {
            Block::Bullets(items) | Block::Numbered(_, items) => items.iter().flatten().map(depth).max().unwrap_or(0),
            Block::Tasks(items) => items.iter().flat_map(|(_, i)| i).map(depth).max().unwrap_or(0),
            Block::Quote(i) | Block::Toggle(i) | Block::Shelf(i) | Block::Callout(_, i) => i.iter().map(depth).max().unwrap_or(0),
            Block::Table(rows) => rows.iter().flatten().flat_map(|c| &c.blocks).map(depth).max().unwrap_or(0),
            Block::Columns(cols) => cols.iter().flatten().map(depth).max().unwrap_or(0),
            _ => 0,
        }
    }

    fn docx_with_body(body: &str) -> Vec<u8> {
        let xml = format!(r#"<w:document xmlns:w="w" xmlns:r="r"><w:body>{body}</w:body></w:document>"#);
        zip::build(&[("word/document.xml", xml.as_bytes())], false)
    }

    fn text_of(blocks: &[Block]) -> String {
        json(blocks).to_string()
    }

    #[test]
    fn deeply_nested_tables_are_read_flat_beyond_eight_levels() {
        let _heavy = crate::importers::heavy();
        let start = std::time::Instant::now();
        // A million nested <w:tbl> (the reviewer's probe was millions): nothing recursive,
        // nothing deeper than the cap, and the text inside survives.
        let n = 500_000;
        let body = format!("{}<w:p><w:r><w:t>deep text</w:t></w:r></w:p>{}", "<w:tbl><w:tr><w:tc>".repeat(n / 5), "</w:tc></w:tr></w:tbl>".repeat(n / 5));
        let (d, _, _) = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
        let max = d.blocks.iter().map(depth).max().unwrap_or(0);
        assert!(max <= 2 * MAX_TABLE_DEPTH + 2, "{max}");
        assert!(text_of(&d.blocks).contains("deep text"));
        let body = format!("{}<w:p><w:r><w:t>x</w:t></w:r></w:p>{}", "<w:tbl>".repeat(n), "</w:tbl>".repeat(n));
        let _ = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
        // Paragraphs, hyperlinks, structured tags and simple fields nested without end.
        let link = r#"<w:hyperlink r:id="nope">"#;
        for open_close in [("<w:p>", "</w:p>"), (link, "</w:hyperlink>"), ("<w:sdt><w:sdtContent>", "</w:sdtContent></w:sdt>"), ("<w:fldSimple w:instr=\"x\">", "</w:fldSimple>")] {
            let body = format!("<w:p>{}<w:r><w:t>kept</w:t></w:r>{}</w:p>", open_close.0.repeat(100_000), open_close.1.repeat(100_000));
            let (d, _, _) = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
            assert!(text_of(&d.blocks).contains("kept"), "{}", open_close.0);
        }
        // Many runs inside many hyperlinks that have no link of their own: linear, not quadratic.
        let body = format!("<w:p>{}{}</w:p>", link.repeat(30), "<w:r><w:t>a</w:t></w:r>".repeat(100_000));
        let _ = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
        assert!(start.elapsed().as_secs() < 180, "{:?}", start.elapsed());
    }

    #[test]
    fn a_word_document_keeps_at_most_the_page_limit_of_table_cells() {
        let row = format!("<w:tr>{}</w:tr>", "<w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc>".repeat(100));
        let body = format!("<w:tbl>{}</w:tbl>", row.repeat(1_500));
        let (d, _, problems) = with_ctx(&[], |ctx| parse(docx_with_body(&body), ctx, "a.docx").unwrap());
        let cells: usize = d.blocks.iter().map(|b| if let Block::Table(rows) = b { rows.iter().map(Vec::len).sum() } else { 0 }).sum();
        assert!(cells <= doc::MAX_PAGE_CELLS, "{cells}");
        assert!(problems.iter().any(|p| p.item.starts_with("Tables")), "{problems:?}");
    }

    #[test]
    fn word_files_draw_on_the_shared_import_budget() {
        let ((spent, left), _, _) = with_ctx(&[], |ctx| {
            let before = ctx.vfs.budget().get();
            parse(sample_docx(), ctx, "a.docx").unwrap();
            (before - ctx.vfs.budget().get(), ctx.vfs.budget().get())
        });
        assert!(spent > 0 && left > 0);
        // With the allowance used up the document cannot be read at all.
        let (r, _, _) = with_ctx(&[], |ctx| {
            ctx.vfs.budget().set(10);
            parse(sample_docx(), ctx, "a.docx").err()
        });
        assert!(r.unwrap().contains("not a Word document"));
    }

    #[test]
    fn a_stopped_import_stops_the_word_converter() {
        let body = "<w:p><w:r><w:t>a</w:t></w:r></w:p>".repeat(100_000);
        let vfs = crate::importers::source::Vfs::from_zip_bytes(zip::build(&[("a.md", b"x")], false)).unwrap();
        let ctl = crate::importers::Control::expired();
        let mut images = crate::importers::Images::default();
        let mut problems = Vec::new();
        let mut ctx = Ctx { vfs: &vfs, ctl: &ctl, dir: String::new(), images: &mut images, problems: &mut problems, name: "t".into() };
        let d = parse(docx_with_body(&body), &mut ctx, "a.docx").unwrap();
        assert!(d.blocks.len() < 100_000, "{}", d.blocks.len());
    }
}
