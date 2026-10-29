//! Word documents (.docx: a ZIP of XML) to the import model.
//!
//! Read: `word/document.xml` (paragraphs, runs, tables, hyperlinks, pictures), `styles.xml`
//! (which styles are headings, the title and quotes), `numbering.xml` (bullet or numbered
//! lists, nested by level), the relationships (link targets and media) and the title in
//! `docProps/core.xml`. Kept: headings 1 to 4 (5 to 9 become 4), bold, italic, underline,
//! strike, highlight, text color, lists, tables (with merged columns), links (http, https,
//! mailto), embedded pictures (PNG, JPEG, GIF, WebP) and line breaks. Dropped: tracked
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
use super::{safe_link, Ctx};

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
        "yellow" => 3,
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
/// highlight: the color blended onto white) as the nearest Notaya highlight. Fills that are
/// not a highlight are left out: `auto`, white, grays and near-grays (the light gray Notaya uses
/// for inline code, Word's light shading), and dark fills. `None` for those.
fn shading_highlight(fill: &str) -> Option<String> {
    if fill.len() != 6 || !fill.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(fill, 16).ok()?;
    let rgb = [((v >> 16) & 0xFF) as f64, ((v >> 8) & 0xFF) as f64, (v & 0xFF) as f64];
    let (max, min) = (rgb.iter().cloned().fold(0.0, f64::max), rgb.iter().cloned().fold(255.0, f64::min));
    // Near-gray or white: no color to speak of. Dark: not a highlighter.
    if max - min < 24.0 || min < 150.0 {
        return None;
    }
    let mut best: Option<(f64, usize)> = None;
    for (i, h) in HIGHLIGHTS.iter().enumerate() {
        let nums: Vec<f64> = h.trim_start_matches("rgba(").trim_end_matches(')').split(',').filter_map(|x| x.trim().parse().ok()).collect();
        let [r, g, b, a] = nums[..] else { continue };
        let blended = [255.0 + (r - 255.0) * a, 255.0 + (g - 255.0) * a, 255.0 + (b - 255.0) * a];
        let d: f64 = blended.iter().zip(rgb).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt();
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i));
        }
    }
    best.filter(|(d, _)| *d <= 80.0).map(|(_, i)| HIGHLIGHTS[i].to_string())
}

#[derive(Default)]
struct Para {
    style: Option<String>,
    num: Option<(String, u32)>,
    outline: Option<u8>,
    inl: Vec<Inline>,
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
                            .and_then(|(t, _)| safe_link(t))
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
                    "tab" if !in_ppr && run.is_some() => {
                        if let (Some(p), Some(m)) = (paras.last_mut(), run.as_ref()) {
                            p.inl.push(Inline::Text(" ".into(), m.clone()));
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
                        if let Some(p) = paras.pop() {
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
                        if let Some(t) = tables.pop() {
                            if !t.rows.is_empty() {
                                push_item(Item::Block(Block::Table(t.rows)), &mut cells, &mut body);
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

/// Paragraph records to blocks: headings, quotes and nested lists.
fn finish(items: Vec<Item>, styles: &HashMap<String, Style>, numbering: &HashMap<String, Levels>, title: &mut Option<String>) -> Vec<Block> {
    let mut out = Vec::new();
    let mut list: Vec<(u32, bool, Vec<Inline>)> = Vec::new();
    let flush_list = |list: &mut Vec<(u32, bool, Vec<Inline>)>, out: &mut Vec<Block>| {
        if !list.is_empty() {
            let base = list.iter().map(|x| x.0).min().unwrap_or(0);
            out.extend(build_list(list, base));
            list.clear();
        }
    };
    for item in items {
        match item {
            Item::Block(b) => {
                flush_list(&mut list, &mut out);
                out.push(b);
            }
            Item::Para(p) => {
                if let Some((id, lvl)) = p.num.as_ref().filter(|(id, _)| id != "0" && !id.is_empty()) {
                    let ordered = numbering.get(id).and_then(|l| l.get(*lvl as usize)).copied().flatten().unwrap_or(false);
                    if list.len() < 10_000 {
                        list.push((*lvl, ordered, p.inl));
                    }
                    continue;
                }
                flush_list(&mut list, &mut out);
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
        assert!(s.contains("#c00000") && s.contains(HIGHLIGHTS[3]));
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
        for w in ["code", "auto", "white", "dark", "paragraph", "cell"] {
            assert_eq!(marks(w).unwrap().highlight, None, "{w}");
        }
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
            Block::Quote(i) | Block::Toggle(i) | Block::Callout(_, i) => i.iter().map(depth).max().unwrap_or(0),
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
