//! Word documents (.docx, Office Open XML). A .docx file is a ZIP of XML parts; this writes the
//! few parts Word needs: the document, its styles, list numbering, the pictures and links, and
//! the file properties. Headings use Word's own Heading 1 to 6 styles, so the navigation pane
//! and a table of contents work. Equations are written as their TeX text (Word's equation
//! format is not produced). Right-to-left text (Arabic, Hebrew: a paragraph set to right-to-left, or one
//! whose first strong letter is) gets Word's right-to-left paragraph (`w:bidi`, which makes
//! Word's "start" the right side, so it is right-aligned), its runs get `w:rtl`, and a table
//! whose first text is right-to-left is drawn right-to-left (`w:bidiVisual`).
//! So that Notaya's own Word importer brings a page back as it was: a code block is a paragraph in
//! the "Notaya Code" style opened by a hidden bookmark naming its language, a task item starts with
//! ☐ or ☑, a picture's caption is a "Caption" paragraph right after it, a highlight is the run's
//! shading in the exact (blended) palette color, and a shelf is a one-cell table in the "Notaya
//! Shelf" table style.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::doc::{blocks_dir, content_dir, first_strong, page_anchor, Align, Block, Dir, Image, Inline, ListKind, Marks, PageRef, Rgb};
use super::zip::{civil, ZipWriter};
use super::{table_grid, ExportDoc, Part, Slot};
use crate::error::Result;

/// A4 with 2.54 cm margins, in twentieths of a point.
const PAGE_W: u32 = 11906;
const PAGE_H: u32 = 16838;
const MARGIN: u32 = 1440;
const TEXT_W: u32 = PAGE_W - 2 * MARGIN;
const EMU_PER_TWIP: u64 = 635;

pub fn render(doc: &ExportDoc) -> Result<Vec<u8>> {
    render_at(doc, crate::model::now_ms())
}

pub fn render_at(doc: &ExportDoc, now_ms: i64) -> Result<Vec<u8>> {
    let mut w = Writer { doc, body: String::new(), rels: Vec::new(), images: HashMap::new(), links: HashMap::new(), nums: Vec::new(), drawing_id: 0, bookmarks: 0, page_break: false, media: Vec::new() };
    let mut first = true;
    let mut after_heading = false;
    let targets = doc.anchor_targets();
    let mut page_no = 0;
    for part in &doc.parts {
        // Each page starts on a new page, except right after its notebook or section heading.
        w.page_break = !first && !after_heading;
        match part {
            Part::Heading { level, text } => {
                w.heading(*level, &[Inline::Text(text.clone(), Marks::default())], Align::Left, &Ctx::default());
                after_heading = true;
            }
            Part::Page { level, title, blocks } => {
                page_no += 1;
                w.page_title(*level, title, targets.contains(&page_no).then_some(page_no));
                w.blocks(blocks, &Ctx { shift: *level, ..Ctx::default() });
                after_heading = false;
            }
        }
        first = false;
    }

    let mut zip = ZipWriter::new(now_ms);
    zip.add("[Content_Types].xml", content_types().as_bytes(), true)?;
    zip.add("_rels/.rels", ROOT_RELS.as_bytes(), true)?;
    zip.add("docProps/core.xml", core_props(&doc.title, now_ms).as_bytes(), true)?;
    zip.add("docProps/app.xml", APP_PROPS.as_bytes(), true)?;
    zip.add("word/document.xml", w.document().as_bytes(), true)?;
    zip.add("word/styles.xml", STYLES.as_bytes(), true)?;
    zip.add("word/numbering.xml", w.numbering().as_bytes(), true)?;
    zip.add("word/_rels/document.xml.rels", w.document_rels().as_bytes(), true)?;
    for (name, id) in &w.media {
        if let Some(p) = doc.images.get(id) {
            zip.add(&format!("word/media/{name}"), &p.bytes, false)?;
        }
    }
    zip.finish()
}

/// Where a paragraph sits: inside lists, quotes, callouts, and how far headings move down.
#[derive(Clone, Default)]
struct Ctx {
    /// Left indent in twips.
    indent: u32,
    /// Heading levels inside a page move down by this much (the page title is above them).
    shift: u8,
    quote: bool,
    callout: bool,
    /// How many lists this paragraph is inside.
    list_depth: u32,
    /// Text in table header cells is bold.
    bold: bool,
}

struct Writer<'a> {
    doc: &'a ExportDoc,
    body: String,
    /// Relationships of the document part: (id, type, target, external).
    rels: Vec<(String, &'static str, String, bool)>,
    /// Attachment id to its relationship id and size in pixels.
    images: HashMap<String, (String, Option<(u32, u32)>)>,
    links: HashMap<String, String>,
    /// List numbering instances: (abstract numbering, level, start).
    nums: Vec<(u32, u32, u32)>,
    drawing_id: u32,
    /// Bookmarks written so far (each code block opens with one).
    bookmarks: u32,
    page_break: bool,
    /// Picture files: (name in word/media, attachment id).
    media: Vec<(String, String)>,
}

const REL_IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const REL_LINK: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

impl<'a> Writer<'a> {
    fn rel(&mut self, kind: &'static str, target: String, external: bool) -> String {
        let id = format!("rId{}", self.rels.len() + 10);
        self.rels.push((id.clone(), kind, target, external));
        id
    }

    fn take_break(&mut self) -> &'static str {
        if std::mem::take(&mut self.page_break) { "<w:pageBreakBefore/>" } else { "" }
    }

    /// Writes one paragraph. `ppr_extra` goes in the paragraph properties after the style
    /// (numbering, borders and so on, already in schema order); `runs` is its content.
    fn para(&mut self, style: Option<&str>, ctx: &Ctx, align: Align, numbering: Option<(u32, u32)>, runs: &str, rtl: bool) {
        let mut ppr = String::new();
        if let Some(s) = style {
            let _ = write!(ppr, "<w:pStyle w:val=\"{s}\"/>");
        }
        ppr.push_str(self.take_break());
        if let Some((num, lvl)) = numbering {
            let _ = write!(ppr, "<w:numPr><w:ilvl w:val=\"{lvl}\"/><w:numId w:val=\"{num}\"/></w:numPr>");
        }
        if ctx.quote || ctx.callout {
            let color = if ctx.callout { "E0A030" } else { "C4C9D2" };
            let _ = write!(ppr, "<w:pBdr><w:left w:val=\"single\" w:sz=\"18\" w:space=\"8\" w:color=\"{color}\"/></w:pBdr>");
        }
        if ctx.callout {
            ppr.push_str("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"FBF6E9\"/>");
        }
        if rtl {
            ppr.push_str("<w:bidi/>");
        }
        if numbering.is_none() && ctx.indent > 0 {
            let _ = write!(ppr, "<w:ind w:left=\"{}\"/>", ctx.indent);
        }
        let jc = match (align, rtl) {
            (Align::Center, _) => Some("center"),
            (Align::Right, false) | (Align::Left, true) => Some("right"),
            (Align::Justify, _) => Some("both"),
            _ => None,
        };
        // In a right-to-left paragraph Word's "start" is the right side already.
        let jc = if rtl && align == Align::Left { None } else { jc };
        if let Some(j) = jc {
            let _ = write!(ppr, "<w:jc w:val=\"{j}\"/>");
        }
        let _ = write!(self.body, "<w:p>{}{runs}</w:p>", if ppr.is_empty() { String::new() } else { format!("<w:pPr>{ppr}</w:pPr>") });
    }

    fn heading(&mut self, level: u8, content: &[Inline], align: Align, ctx: &Ctx) {
        let level = level.clamp(1, 6);
        let runs = self.runs(content, ctx.bold);
        let rtl = is_rtl(content);
        self.para(Some(&format!("Heading{level}")), &Ctx { indent: ctx.indent, ..Ctx::default() }, align, None, &runs, rtl);
    }

    fn blocks(&mut self, blocks: &[Block], ctx: &Ctx) {
        for b in blocks {
            self.block(b, ctx, None);
        }
    }

    /// `numbering`: the list numbering for the first paragraph of a list item.
    fn block(&mut self, b: &Block, ctx: &Ctx, numbering: Option<(u32, u32)>) {
        match b {
            Block::Paragraph { align, content } => {
                let runs = self.runs(content, ctx.bold);
                let style = if ctx.quote { Some("Quote") } else { None };
                self.para(style, ctx, *align, numbering, &runs, is_rtl(content));
            }
            Block::Heading { level, align, content } => self.heading((level + ctx.shift).min(6), content, *align, ctx),
            Block::List { kind, items } => self.list(*kind, items, ctx),
            Block::Quote(inner) => self.blocks(inner, &Ctx { quote: true, indent: ctx.indent + 240, ..ctx.clone() }),
            Block::Callout { icon, blocks } => {
                let inner = Ctx { callout: true, indent: ctx.indent + 240, ..ctx.clone() };
                match blocks.split_first() {
                    Some((Block::Paragraph { align, content }, rest)) => {
                        let mut c = vec![Inline::Text(format!("{icon} "), Marks::default())];
                        c.extend(content.iter().cloned());
                        self.block(&Block::Paragraph { align: *align, content: c }, &inner, None);
                        self.blocks(rest, &inner);
                    }
                    _ => {
                        self.block(&Block::Paragraph { align: Align::Left, content: vec![Inline::Text(icon.clone(), Marks::default())] }, &inner, None);
                        self.blocks(blocks, &inner);
                    }
                }
            }
            Block::Toggle(inner) => {
                if let Some((first, rest)) = inner.split_first() {
                    match first {
                        Block::Paragraph { align, content } | Block::Heading { align, content, .. } => {
                            let mut c = vec![Inline::Text("▸ ".into(), Marks::default())];
                            c.extend(content.iter().cloned());
                            let b = match first {
                                Block::Heading { level, .. } => Block::Heading { level: *level, align: *align, content: c },
                                _ => Block::Paragraph { align: *align, content: c },
                            };
                            self.block(&b, ctx, None);
                        }
                        other => self.block(other, ctx, None),
                    }
                    self.blocks(rest, &Ctx { indent: ctx.indent + 360, ..ctx.clone() });
                }
            }
            Block::Shelf(inner) => self.shelf(inner, ctx),
            Block::Columns(cols) => cols.iter().for_each(|c| self.blocks(c, ctx)),
            Block::Code { language, text } => {
                // One paragraph in the "Notaya Code" style (monospace), opened by a hidden bookmark
                // that marks where the block starts and names its language, so Notaya's importer
                // brings it back as the same code block.
                self.bookmarks += 1;
                let id = self.bookmarks;
                // A bookmark name holds only letters, digits and `_`: a `-` (as in `php-template`) is written as `_`.
                let lang: String = language.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').map(|c| if c == '-' { '_' } else { c }).take(20).collect();
                let mut runs = format!("<w:bookmarkStart w:id=\"{id}\" w:name=\"{CODE_BOOKMARK}{id}_{lang}\"/><w:bookmarkEnd w:id=\"{id}\"/>");
                for (i, line) in text.split('\n').enumerate() {
                    if i > 0 {
                        runs.push_str("<w:r><w:br/></w:r>");
                    }
                    runs.push_str(&run_xml(line, "", false));
                }
                self.para(Some("NotayaCode"), ctx, Align::Left, numbering, &runs, false);
            }
            Block::Math(latex) => {
                if !latex.trim().is_empty() {
                    let runs = run_xml(latex.trim(), MATH_RPR, false);
                    self.para(None, ctx, Align::Center, numbering, &runs, false);
                }
            }
            Block::Image(img) => self.image(img, ctx, numbering),
            Block::Table(rows) => self.table(rows, ctx),
            Block::Rule => {
                let brk = self.take_break();
                let _ = write!(
                    self.body,
                    "<w:p><w:pPr>{brk}<w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"BFBFBF\"/></w:pBdr></w:pPr></w:p>"
                );
            }
            Block::PageLink { title, target } => self.page_link_row(title, target.as_ref(), ctx, numbering),
        }
    }

    fn list(&mut self, kind: ListKind, items: &[super::doc::ListItem], ctx: &Ctx) {
        let lvl = ctx.list_depth.min(8);
        let num = match kind {
            ListKind::Bullet => Some(self.num(0, lvl, 1)),
            ListKind::Ordered(start) => Some(self.num(1, lvl, start.max(1))),
            ListKind::Task => None,
        };
        let inner = Ctx { indent: ctx.indent.max(720 + 360 * lvl), list_depth: lvl + 1, ..ctx.clone() };
        for item in items {
            let mut rest: &[Block] = &item.blocks;
            match (num, item.blocks.first()) {
                (Some(n), Some(first @ (Block::Paragraph { .. } | Block::Code { .. } | Block::Math(_) | Block::Image(_) | Block::PageLink { .. }))) => {
                    self.block(first, &inner, Some((n, lvl)));
                    rest = &item.blocks[1..];
                }
                (Some(n), _) => self.para(None, &inner, Align::Left, Some((n, lvl)), "", false),
                (None, first) => {
                    let mark = if item.checked == Some(true) { "☑ " } else { "☐ " };
                    let content = match first {
                        Some(Block::Paragraph { content, .. }) => {
                            rest = &item.blocks[1..];
                            content.clone()
                        }
                        _ => Vec::new(),
                    };
                    let mut c = vec![Inline::Text(mark.into(), Marks::default())];
                    c.extend(content);
                    let runs = self.runs(&c, ctx.bold);
                    let rtl = is_rtl(&c);
                    self.para(None, &Ctx { indent: ctx.indent.max(360 + 360 * lvl), ..ctx.clone() }, Align::Left, None, &runs, rtl);
                }
            }
            self.blocks(rest, &inner);
        }
    }

    /// A numbering instance for one list, so every list counts from its own start.
    fn num(&mut self, abstract_id: u32, lvl: u32, start: u32) -> u32 {
        if abstract_id == 0 {
            if let Some(i) = self.nums.iter().position(|n| n.0 == 0) {
                return i as u32 + 1;
            }
        }
        self.nums.push((abstract_id, lvl, start));
        self.nums.len() as u32
    }

    fn image(&mut self, img: &Image, ctx: &Ctx, numbering: Option<(u32, u32)>) {
        let Some(id) = img.attachment.as_ref().filter(|id| self.doc.images.contains_key(*id)) else {
            let runs = run_xml("(picture not available)", "<w:i/><w:color w:val=\"6B7280\"/>", false);
            self.para(None, ctx, Align::Left, numbering, &runs, false);
            return;
        };
        if !self.images.contains_key(id) {
            let p = &self.doc.images[id];
            let name = format!("image{}.{}", self.media.len() + 1, p.kind.ext());
            let rid = self.rel(REL_IMAGE, format!("media/{name}"), false);
            self.media.push((name, id.clone()));
            self.images.insert(id.clone(), (rid, super::image::dimensions(p.kind, &p.bytes)));
        }
        let (rid, dims) = self.images[id].clone();
        let avail = TEXT_W.saturating_sub(ctx.indent).max(1440) as u64 * EMU_PER_TWIP;
        let (pw, ph) = dims.unwrap_or((640, 480));
        let natural = pw as u64 * 9525; // 96 pixels per inch
        let mut cx = match img.width_pct {
            Some(pct) => (avail as f64 * pct as f64 / 100.0) as u64,
            None => natural.min(avail),
        };
        let mut cy = cx * ph as u64 / pw.max(1) as u64;
        // Never taller than most of a page.
        let max_h = (PAGE_H - 2 * MARGIN - 1440) as u64 * EMU_PER_TWIP;
        if cy > max_h {
            cx = cx * max_h / cy;
            cy = max_h;
        }
        self.drawing_id += 1;
        let n = self.drawing_id;
        let descr = xml(&img.caption);
        let drawing = format!(
            "<w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/>\
             <wp:docPr id=\"{n}\" name=\"Picture {n}\" descr=\"{descr}\"/>\
             <wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>\
             <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic>\
             <pic:nvPicPr><pic:cNvPr id=\"{n}\" name=\"Picture {n}\"/><pic:cNvPicPr/></pic:nvPicPr>\
             <pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
             <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>\
             </pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
        );
        self.para(None, ctx, img.align, numbering, &drawing, false);
        if !img.caption.trim().is_empty() {
            let runs = run_xml(img.caption.trim(), "", is_rtl_text(&img.caption));
            self.para(Some("Caption"), ctx, img.align, None, &runs, is_rtl_text(&img.caption));
        }
    }

    /// A shelf as a shaded box: a table of one cell in the "Notaya Shelf" table style, holding the
    /// shelf's title line and content. The style name is what Notaya's importer looks for.
    fn shelf(&mut self, inner: &[Block], ctx: &Ctx) {
        let w = TEXT_W.saturating_sub(ctx.indent).max(2000);
        let pre_break = self.take_break();
        if !pre_break.is_empty() {
            let _ = write!(self.body, "<w:p><w:pPr>{pre_break}</w:pPr></w:p>");
        }
        let ind = if ctx.indent > 0 { format!("<w:tblInd w:w=\"{}\" w:type=\"dxa\"/>", ctx.indent) } else { String::new() };
        let _ = write!(
            self.body,
            "<w:tbl><w:tblPr><w:tblStyle w:val=\"{SHELF_STYLE}\"/><w:tblW w:w=\"{w}\" w:type=\"dxa\"/>{ind}<w:tblLayout w:type=\"fixed\"/><w:tblLook w:val=\"0000\" w:firstRow=\"0\" w:lastRow=\"0\" w:firstColumn=\"0\" w:lastColumn=\"0\" w:noHBand=\"1\" w:noVBand=\"1\"/></w:tblPr>\
             <w:tblGrid><w:gridCol w:w=\"{w}\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"{w}\" w:type=\"dxa\"/><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F3F4F6\"/></w:tcPr>"
        );
        let before = self.body.len();
        self.blocks(inner, &Ctx { shift: ctx.shift, ..Ctx::default() });
        if self.body.len() == before || self.body.ends_with("</w:tbl>") {
            self.body.push_str("<w:p/>");
        }
        self.body.push_str("</w:tc></w:tr></w:tbl><w:p><w:pPr><w:spacing w:after=\"0\"/></w:pPr></w:p>");
    }

    fn table(&mut self, rows: &[Vec<super::doc::Cell>], ctx: &Ctx) {
        let grid = table_grid(rows);
        let cols = grid.first().map_or(0, Vec::len);
        if cols == 0 {
            return;
        }
        let avail = TEXT_W.saturating_sub(ctx.indent).max(2000);
        // Column widths dragged in the editor (pixels), when the first row has them all.
        let mut px: Vec<f32> = Vec::new();
        if let Some(first) = rows.first() {
            for c in first {
                if c.widths.len() == c.colspan.max(1) {
                    px.extend(&c.widths);
                } else {
                    px.clear();
                    break;
                }
            }
        }
        let widths: Vec<u32> = if px.len() == cols && px.iter().all(|w| *w > 0.0) {
            let total: f32 = px.iter().sum();
            px.iter().map(|w| (w / total * avail as f32) as u32).collect()
        } else {
            vec![avail / cols as u32; cols]
        };
        let pre_break = self.take_break();
        if !pre_break.is_empty() {
            let _ = write!(self.body, "<w:p><w:pPr>{pre_break}</w:pPr></w:p>");
        }
        let _ = write!(
            self.body,
            "<w:tbl><w:tblPr><w:tblStyle w:val=\"NotayaTable\"/>{}<w:tblW w:w=\"{}\" w:type=\"dxa\"/>{}<w:tblLayout w:type=\"fixed\"/><w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"0\" w:lastColumn=\"0\" w:noHBand=\"1\" w:noVBand=\"1\"/></w:tblPr><w:tblGrid>",
            // A table whose first text runs right-to-left is drawn from the right (Word's "bidiVisual").
            if rows.iter().flatten().find_map(|c| blocks_dir(&c.blocks)) == Some(Dir::Rtl) { "<w:bidiVisual/>" } else { "" },
            widths.iter().sum::<u32>(),
            if ctx.indent > 0 { format!("<w:tblInd w:w=\"{}\" w:type=\"dxa\"/>", ctx.indent) } else { String::new() }
        );
        for w in &widths {
            let _ = write!(self.body, "<w:gridCol w:w=\"{w}\"/>");
        }
        self.body.push_str("</w:tblGrid>");
        for (r, row) in grid.iter().enumerate() {
            let header_row = rows[r].iter().all(|c| c.header) && !rows[r].is_empty();
            self.body.push_str("<w:tr>");
            if header_row && r == 0 {
                self.body.push_str("<w:trPr><w:tblHeader/></w:trPr>");
            }
            let mut c = 0;
            while c < cols {
                match row[c] {
                    Slot::Cell(cr, ci) => {
                        let cell = &rows[cr][ci];
                        let span = cell.colspan.max(1).min(cols - c);
                        let w: u32 = widths[c..c + span].iter().sum();
                        let mut tcpr = format!("<w:tcW w:w=\"{w}\" w:type=\"dxa\"/>");
                        if span > 1 {
                            let _ = write!(tcpr, "<w:gridSpan w:val=\"{span}\"/>");
                        }
                        if cell.rowspan > 1 {
                            tcpr.push_str("<w:vMerge w:val=\"restart\"/>");
                        }
                        if cell.header {
                            tcpr.push_str("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F1F3F5\"/>");
                        }
                        let _ = write!(self.body, "<w:tc><w:tcPr>{tcpr}</w:tcPr>");
                        let before = self.body.len();
                        self.blocks(&cell.blocks, &Ctx { bold: cell.header, ..Ctx::default() });
                        // A cell must end with a paragraph.
                        if self.body.len() == before || self.body.ends_with("</w:tbl>") {
                            self.body.push_str("<w:p/>");
                        }
                        self.body.push_str("</w:tc>");
                        c += span;
                    }
                    Slot::Down(span) => {
                        let span = span.max(1).min(cols - c);
                        let w: u32 = widths[c..c + span].iter().sum();
                        let gs = if span > 1 { format!("<w:gridSpan w:val=\"{span}\"/>") } else { String::new() };
                        let _ = write!(self.body, "<w:tc><w:tcPr><w:tcW w:w=\"{w}\" w:type=\"dxa\"/>{gs}<w:vMerge/></w:tcPr><w:p/></w:tc>");
                        c += span;
                    }
                    Slot::Right => c += 1,
                    Slot::Empty => {
                        let _ = write!(self.body, "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc>", widths[c]);
                        c += 1;
                    }
                }
            }
            self.body.push_str("</w:tr>");
        }
        self.body.push_str("</w:tbl>");
        // Word needs a paragraph between two tables, and after a table at the end of a cell.
        self.body.push_str("<w:p><w:pPr><w:spacing w:after=\"0\"/></w:pPr></w:p>");
    }

    /// Runs (and hyperlinks) for inline content.
    fn runs(&mut self, content: &[Inline], bold: bool) -> String {
        let mut out = String::new();
        for item in content {
            match item {
                Inline::Text(text, m) => {
                    let mut rpr = String::new();
                    if m.link.is_some() || m.page.is_some() {
                        rpr.push_str("<w:rStyle w:val=\"Hyperlink\"/>");
                    }
                    if m.code {
                        rpr.push_str("<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/>");
                    }
                    if m.bold || bold {
                        rpr.push_str("<w:b/><w:bCs/>");
                    }
                    if m.italic {
                        rpr.push_str("<w:i/><w:iCs/>");
                    }
                    if m.strike {
                        rpr.push_str("<w:strike/>");
                    }
                    if let Some(c) = m.color {
                        let _ = write!(rpr, "<w:color w:val=\"{}\"/>", c.hex());
                    }
                    if let Some(size) = m.size {
                        let hp = (size * 2.0).round() as u32;
                        let _ = write!(rpr, "<w:sz w:val=\"{hp}\"/><w:szCs w:val=\"{hp}\"/>");
                    }
                    if m.underline {
                        rpr.push_str("<w:u w:val=\"single\"/>");
                    }
                    let shade = m.highlight.or(if m.code { Some(Rgb(0xF1, 0xF3, 0xF5)) } else { None });
                    if let Some(h) = shade {
                        let _ = write!(rpr, "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{}\"/>", h.hex());
                    }
                    let run = run_xml(text, &rpr, is_rtl_text(text));
                    match m.link.as_deref().filter(|u| safe_link(u)) {
                        Some(url) => {
                            let rid = match self.links.get(url) {
                                Some(r) => r.clone(),
                                None => {
                                    let target = clean_link_target(url);
                                    let r = self.rel(REL_LINK, target, true);
                                    self.links.insert(url.to_string(), r.clone());
                                    r
                                }
                            };
                            let _ = write!(out, "<w:hyperlink r:id=\"{rid}\" w:history=\"1\">{run}</w:hyperlink>");
                        }
                        None => match &m.page {
                            Some(r) => out.push_str(&self.page_hyperlink(r, &run)),
                            None => out.push_str(&run),
                        },
                    }
                }
                Inline::Break => out.push_str("<w:r><w:br/></w:r>"),
                Inline::Math(latex) => {
                    if !latex.trim().is_empty() {
                        out.push_str(&run_xml(latex.trim(), MATH_RPR, false));
                    }
                }
            }
        }
        out
    }

    fn document(&self) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
             xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
             xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
             xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
             xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><w:body>{}\
             <w:sectPr><w:pgSz w:w=\"{PAGE_W}\" w:h=\"{PAGE_H}\"/><w:pgMar w:top=\"{MARGIN}\" w:right=\"{MARGIN}\" w:bottom=\"{MARGIN}\" w:left=\"{MARGIN}\" w:header=\"708\" w:footer=\"708\" w:gutter=\"0\"/></w:sectPr>\
             </w:body></w:document>",
            self.body
        )
    }

    fn document_rels(&self) -> String {
        let mut s = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>\
             <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" Target=\"numbering.xml\"/>",
        );
        for (id, kind, target, external) in &self.rels {
            let mode = if *external { " TargetMode=\"External\"" } else { "" };
            let _ = write!(s, "<Relationship Id=\"{id}\" Type=\"{kind}\" Target=\"{}\"{mode}/>", xml(target));
        }
        s.push_str("</Relationships>");
        s
    }

    fn numbering(&self) -> String {
        let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">");
        for (abs, fmt) in [(0, "bullet"), (1, "decimal")] {
            let _ = write!(s, "<w:abstractNum w:abstractNumId=\"{abs}\"><w:multiLevelType w:val=\"hybridMultilevel\"/>");
            for lvl in 0..9u32 {
                let text = if fmt == "bullet" { ["•", "◦", "▪"][lvl as usize % 3].to_string() } else { format!("%{}.", lvl + 1) };
                let left = 720 + 360 * lvl;
                let font = if fmt == "bullet" { "<w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/></w:rPr>" } else { "" };
                let _ = write!(
                    s,
                    "<w:lvl w:ilvl=\"{lvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{fmt}\"/><w:lvlText w:val=\"{text}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr>{font}</w:lvl>"
                );
            }
            s.push_str("</w:abstractNum>");
        }
        for (i, (abs, lvl, start)) in self.nums.iter().enumerate() {
            let _ = write!(s, "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"{abs}\"/>", i + 1);
            if *abs == 1 {
                let _ = write!(s, "<w:lvlOverride w:ilvl=\"{lvl}\"><w:startOverride w:val=\"{start}\"/></w:lvlOverride>");
            }
            s.push_str("</w:num>");
        }
        s.push_str("</w:numbering>");
        s
    }
}

/// Links to other pages of the same export.
impl Writer<'_> {
    /// A page's title heading; `bookmark` (the page's number) marks it as the place that links
    /// in this document go to.
    fn page_title(&mut self, level: u8, title: &str, bookmark: Option<usize>) {
        let content = [Inline::Text(title.to_string(), Marks::default())];
        match bookmark {
            None => self.heading(level, &content, Align::Left, &Ctx::default()),
            Some(n) => {
                // The id comes from the same counter as the code-block bookmarks (ids must be unique in the file).
                self.bookmarks += 1;
                let id = self.bookmarks;
                let runs = format!("<w:bookmarkStart w:id=\"{id}\" w:name=\"{}\"/>{}<w:bookmarkEnd w:id=\"{id}\"/>", page_anchor(n), self.runs(&content, false));
                self.para(Some(&format!("Heading{}", level.clamp(1, 6))), &Ctx::default(), Align::Left, None, &runs, is_rtl_text(title));
            }
        }
    }

    /// `run` as a link to another page: a bookmark of this document, or (in a ZIP) the other
    /// .docx file by its relative path, which the exporter made from its own file names.
    fn page_hyperlink(&mut self, target: &PageRef, run: &str) -> String {
        match target {
            PageRef::Anchor(n) => format!("<w:hyperlink w:anchor=\"{}\" w:history=\"1\">{run}</w:hyperlink>", page_anchor(*n)),
            PageRef::File(path) => {
                let rid = match self.links.get(path) {
                    Some(r) => r.clone(),
                    None => {
                        let r = self.rel(REL_LINK, clean_link_target(path), true);
                        self.links.insert(path.clone(), r.clone());
                        r
                    }
                };
                format!("<w:hyperlink r:id=\"{rid}\" w:history=\"1\">{run}</w:hyperlink>")
            }
        }
    }

    /// The "Link to page" row: an arrow and the page's title, linked when the page is part of the export.
    fn page_link_row(&mut self, title: &str, target: Option<&PageRef>, ctx: &Ctx, numbering: Option<(u32, u32)>) {
        const RPR: &str = "<w:i/>";
        let runs = match target {
            None => run_xml(&format!("→ {title}"), RPR, is_rtl_text(title)),
            Some(r) => {
                let link = run_xml(title, "<w:rStyle w:val=\"Hyperlink\"/><w:i/>", is_rtl_text(title));
                format!("{}{}", run_xml("→ ", RPR, false), self.page_hyperlink(r, &link))
            }
        };
        self.para(None, ctx, Align::Left, numbering, &runs, false);
    }
}

/// The start of the hidden bookmark (`_NotayaCode<n>_<language>`) that opens a code block, and the
/// table style of a shelf. Notaya's Word importer reads both (keep them in step with importers/docx.rs).
pub const CODE_BOOKMARK: &str = "_NotayaCode";
pub const SHELF_STYLE: &str = "NotayaShelf";

const MATH_RPR: &str ="<w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/><w:i/>";

/// Only web and mail links become clickable; anything else stays plain text.
/// A link target as a plain ASCII address for the relationships file: trimmed, control characters
/// removed, and spaces and non-ASCII characters percent-encoded.
fn clean_link_target(url: &str) -> String {
    let mut out = String::new();
    for c in url.trim().chars().filter(|c| !c.is_control()) {
        if c.is_ascii() && c != ' ' {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

fn safe_link(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("http://") || u.starts_with("https://") || u.starts_with("mailto:")
}

/// One run of text. Tabs and line breaks become Word's own tab and break elements.
fn run_xml(text: &str, rpr: &str, rtl: bool) -> String {
    let rpr = if rtl { format!("{rpr}<w:rtl/>") } else { rpr.to_string() };
    let mut s = String::from("<w:r>");
    if !rpr.is_empty() {
        let _ = write!(s, "<w:rPr>{rpr}</w:rPr>");
    }
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            s.push_str("<w:br/>");
        }
        for (k, piece) in line.split('\t').enumerate() {
            if k > 0 {
                s.push_str("<w:tab/>");
            }
            if !piece.is_empty() {
                let _ = write!(s, "<w:t xml:space=\"preserve\">{}</w:t>", xml(piece));
            }
        }
    }
    s.push_str("</w:r>");
    s
}

/// Text for XML, with the characters XML does not allow left out.
pub fn xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => out.push(c),
        }
    }
    out
}

/// True when the first strong character of the text belongs to a right-to-left script (Arabic, Hebrew).
fn is_rtl_text(s: &str) -> bool {
    first_strong(s) == Some(Dir::Rtl)
}

/// True when a paragraph runs right-to-left: set so by its direction, else by its first strong letter.
fn is_rtl(content: &[Inline]) -> bool {
    content_dir(content) == Dir::Rtl
}

fn content_types() -> String {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>",
    );
    for kind in [super::image::Kind::Png, super::image::Kind::Jpeg, super::image::Kind::Gif, super::image::Kind::Webp] {
        let _ = write!(s, "<Default Extension=\"{}\" ContentType=\"{}\"/>", kind.ext(), kind.mime());
    }
    s.push_str(
        "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
         <Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
         <Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>\
         <Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>\
         <Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>\
         </Types>",
    );
    s
}

const ROOT_RELS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>\
<Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties\" Target=\"docProps/app.xml\"/>\
</Relationships>";

const APP_PROPS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\"><Application>Notaya</Application></Properties>";

fn core_props(title: &str, now_ms: i64) -> String {
    let (y, mo, d, h, mi, s) = civil(now_ms);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\
         <dc:title>{}</dc:title><dcterms:created xsi:type=\"dcterms:W3CDTF\">{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z</dcterms:created></cp:coreProperties>",
        xml(title)
    )
}

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Arial"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault>
<w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val="111827"/><w:sz w:val="36"/><w:szCs w:val="36"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="300" w:after="100"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val="111827"/><w:sz w:val="30"/><w:szCs w:val="30"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val="1F2937"/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="60"/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val="1F2937"/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="160" w:after="60"/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val="374151"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="160" w:after="60"/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:b/><w:bCs/><w:i/><w:iCs/><w:color w:val="374151"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:qFormat/><w:rPr><w:i/><w:iCs/><w:color w:val="4B5563"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="NotayaCode"><w:name w:val="Notaya Code"/><w:basedOn w:val="Normal"/><w:pPr><w:shd w:val="clear" w:color="auto" w:fill="F3F4F6"/><w:spacing w:after="160" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/><w:sz w:val="19"/><w:szCs w:val="19"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Caption"><w:name w:val="caption"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:rPr><w:i/><w:iCs/><w:color w:val="6B7280"/><w:sz w:val="18"/><w:szCs w:val="18"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="2563EB"/><w:u w:val="single"/></w:rPr></w:style>
<w:style w:type="table" w:styleId="NotayaTable"><w:name w:val="Notaya Table"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/><w:left w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/><w:right w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="C9CED6"/></w:tblBorders><w:tblCellMar><w:top w:w="60" w:type="dxa"/><w:left w:w="100" w:type="dxa"/><w:bottom w:w="60" w:type="dxa"/><w:right w:w="100" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>
<w:style w:type="table" w:styleId="NotayaShelf"><w:name w:val="Notaya Shelf"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="D9DDE3"/><w:left w:val="single" w:sz="4" w:space="0" w:color="D9DDE3"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="D9DDE3"/><w:right w:val="single" w:sz="4" w:space="0" w:color="D9DDE3"/></w:tblBorders><w:tblCellMar><w:top w:w="100" w:type="dxa"/><w:left w:w="160" w:type="dxa"/><w:bottom w:w="60" w:type="dxa"/><w:right w:w="160" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>
</w:styles>"#;

#[cfg(test)]
mod tests {
    use super::super::doc::parse;
    use super::super::doc::tests::{every_node_json, titles};
    use super::super::tests::sample_doc;
    use super::super::zip::tests::read_zip;
    use super::*;

    /// A small well-formedness check: every tag is closed in order, attributes are quoted,
    /// and only the five XML entities are used.
    fn assert_well_formed(name: &str, x: &str) {
        let body = x.strip_prefix("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap_or(x);
        let mut stack: Vec<String> = Vec::new();
        let mut rest = body;
        while let Some(start) = rest.find('<') {
            let text = &rest[..start];
            for (i, _) in text.match_indices('&') {
                let ent = &text[i..];
                assert!(["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"].iter().any(|e| ent.starts_with(e)), "{name}: bad entity near {}", &ent[..ent.len().min(12)]);
            }
            assert!(!text.contains('>'), "{name}: stray >");
            let end = rest[start..].find('>').expect("tag closed") + start;
            let tag = &rest[start + 1..end];
            if let Some(closing) = tag.strip_prefix('/') {
                assert_eq!(stack.pop().as_deref(), Some(closing), "{name}: closing tag");
            } else if !tag.ends_with('/') {
                stack.push(tag.split_whitespace().next().unwrap().to_string());
            }
            assert_eq!(tag.matches('"').count() % 2, 0, "{name}: unbalanced quotes in <{tag}>");
            rest = &rest[end + 1..];
        }
        assert!(stack.is_empty(), "{name}: unclosed {stack:?}");
    }

    #[test]
    fn a_docx_has_the_parts_word_needs_and_well_formed_xml() {
        let doc = sample_doc(parse(&every_node_json(), &titles));
        let bytes = render_at(&doc, 1_793_190_896_000).unwrap();
        assert!(bytes.starts_with(b"PK\x03\x04"));
        let entries = read_zip(&bytes);
        let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
        for part in ["[Content_Types].xml", "_rels/.rels", "word/document.xml", "word/styles.xml", "word/numbering.xml", "word/_rels/document.xml.rels", "docProps/core.xml", "word/media/image1.png"] {
            assert!(names.contains(&part), "missing {part}: {names:?}");
        }
        assert_eq!(names[0], "[Content_Types].xml", "the content types come first");
        for (name, data) in &entries {
            if name.ends_with(".xml") || name.ends_with(".rels") {
                assert_well_formed(name, std::str::from_utf8(data).unwrap());
            }
        }
        let get = |n: &str| String::from_utf8(entries.iter().find(|e| e.0 == n).unwrap().1.clone()).unwrap();
        let document = get("word/document.xml");
        for expected in [
            "<w:pStyle w:val=\"Heading1\"/></w:pPr><w:r><w:t xml:space=\"preserve\">Sample page</w:t>",
            "<w:pStyle w:val=\"Heading2\"/>",
            "<w:pStyle w:val=\"Heading5\"/>",
            "<w:b/><w:bCs/></w:rPr><w:t xml:space=\"preserve\">bold</w:t>",
            "<w:i/><w:iCs/></w:rPr><w:t xml:space=\"preserve\">italic</w:t>",
            "<w:u w:val=\"single\"/>",
            "<w:strike/>",
            "<w:color w:val=\"E5484D\"/><w:sz w:val=\"30\"/>",
            "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F8E6AD\"/>",
            "&lt;special&gt; *chars* &amp; café € “quotes”.",
            "<w:hyperlink r:id=\"rId",
            "<w:numPr><w:ilvl w:val=\"0\"/>",
            "<w:numPr><w:ilvl w:val=\"1\"/>",
            "☑ ",
            "☐ ",
            "<w:pStyle w:val=\"Quote\"/>",
            "💡 ",
            "▸ ",
            "<w:pStyle w:val=\"NotayaCode\"/></w:pPr><w:bookmarkStart w:id=\"1\" w:name=\"_NotayaCode1_rust\"/><w:bookmarkEnd w:id=\"1\"/>",
            "println!(&quot;```&quot;);",
            "\\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}",
            "<a:blip r:embed=\"rId",
            "<w:pStyle w:val=\"Caption\"/>",
            "(picture not available)",
            "<w:tbl>",
            "<w:gridSpan w:val=\"2\"/>",
            "<w:tblHeader/>",
            "→ Child page",
            "Unknown block text",
        ] {
            assert!(document.contains(expected), "document.xml lacks {expected:?}");
        }
        let rels = get("word/_rels/document.xml.rels");
        assert!(rels.contains("Target=\"https://example.com/a?b=1&amp;c=2\" TargetMode=\"External\""));
        assert!(rels.contains("Target=\"media/image1.png\""));
        let numbering = get("word/numbering.xml");
        assert!(numbering.contains("<w:startOverride w:val=\"3\"/>"), "the ordered list starts at 3");
        assert!(get("docProps/core.xml").contains("<dc:title>Sample page</dc:title>"));
        assert!(get("docProps/core.xml").contains("2026-10-28T12:34:56Z"));
    }

    #[test]
    fn shelves_code_blocks_and_their_markers_are_well_formed() {
        let json = r#"{"type":"doc","content":[
            {"type":"toggle","attrs":{"open":true,"shelf":true},"content":[{"type":"paragraph","content":[{"type":"text","text":"Shelf"}]},{"type":"toggle","attrs":{"shelf":true},"content":[{"type":"paragraph","content":[{"type":"text","text":"Inner"}]}]}]},
            {"type":"codeBlock","attrs":{"language":"c++\"/><!"},"content":[{"type":"text","text":"a"}]},
            {"type":"codeBlock","attrs":{"language":"averyveryveryveryverylonglanguagename"},"content":[{"type":"text","text":"b"}]}]}"#;
        let entries = read_zip(&render_at(&sample_doc(parse(json, &titles)), 0).unwrap());
        for (name, data) in &entries {
            if name.ends_with(".xml") {
                assert_well_formed(name, std::str::from_utf8(data).unwrap());
            }
        }
        let document = String::from_utf8(entries.iter().find(|e| e.0 == "word/document.xml").unwrap().1.clone()).unwrap();
        assert_eq!(document.matches("<w:tblStyle w:val=\"NotayaShelf\"/>").count(), 2, "a shelf inside a shelf");
        assert!(document.contains("w:name=\"_NotayaCode1_c\"/>"), "only letters and digits of the language");
        assert!(!document.contains("<!--"));
        let long = document.split("w:name=\"").filter_map(|s| s.split('"').next()).find(|n| n.starts_with("_NotayaCode2_")).unwrap();
        assert!(long.len() <= 40, "Word's bookmark names are at most 40 characters: {long}");
        let styles = String::from_utf8(entries.iter().find(|e| e.0 == "word/styles.xml").unwrap().1.clone()).unwrap();
        assert!(styles.contains("w:styleId=\"NotayaShelf\"") && styles.contains("w:styleId=\"NotayaCode\""));
    }

    #[test]
    fn every_bookmark_in_a_document_has_its_own_id() {
        let mut doc = sample_doc(Vec::new());
        doc.parts = vec![
            Part::Page { level: 1, title: "One".into(), blocks: vec![Block::Code { language: "rust".into(), text: "fn a() {}".into() }] },
            Part::Page { level: 1, title: "Two".into(), blocks: vec![Block::PageLink { title: "One".into(), target: Some(PageRef::Anchor(1)) }, Block::Code { language: "c".into(), text: "x".into() }] },
        ];
        let entries = read_zip(&render_at(&doc, 0).unwrap());
        let document = String::from_utf8(entries.iter().find(|e| e.0 == "word/document.xml").unwrap().1.clone()).unwrap();
        let ids: Vec<&str> = document.split("<w:bookmarkStart w:id=\"").skip(1).filter_map(|s| s.split('"').next()).collect();
        assert!(ids.len() >= 3, "{ids:?}");
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "bookmark ids must be unique: {ids:?}");
    }

    #[test]
    fn right_to_left_text_gets_a_right_to_left_paragraph() {
        let doc = sample_doc(parse(r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"مرحبا بالعالم"}]}]}"#, &titles));
        let entries = read_zip(&render_at(&doc, 0).unwrap());
        let document = String::from_utf8(entries.iter().find(|e| e.0 == "word/document.xml").unwrap().1.clone()).unwrap();
        assert!(document.contains("<w:bidi/>"));
        assert!(document.contains("<w:rtl/></w:rPr><w:t xml:space=\"preserve\">مرحبا بالعالم</w:t>"));
    }

    fn document_xml(json: &str) -> String {
        let doc = sample_doc(parse(json, &titles));
        let entries = read_zip(&render_at(&doc, 0).unwrap());
        String::from_utf8(entries.iter().find(|e| e.0 == "word/document.xml").unwrap().1.clone()).unwrap()
    }

    fn p(attrs: &str, text: &str) -> String {
        format!(r#"{{"type":"paragraph","attrs":{{{attrs}}},"content":[{{"type":"text","text":"{text}"}}]}}"#)
    }

    #[test]
    fn a_chosen_direction_decides_the_word_paragraph() {
        // English text set to right-to-left: a right-to-left paragraph (right-aligned), runs stay as they are.
        let x = document_xml(&format!(r#"{{"type":"doc","content":[{}]}}"#, p(r#""dir":"rtl""#, "English forced")));
        assert!(x.contains("<w:pPr><w:bidi/></w:pPr>"), "{x}");
        assert!(!x.contains("<w:jc "), "under w:bidi, Word's start is the right side already: no jc");
        assert!(!x.contains("<w:rtl/>"));
        // Arabic text set to left-to-right: no bidi, and the Arabic run is still marked right-to-left for Word's shaping.
        let x = document_xml(&format!(r#"{{"type":"doc","content":[{}]}}"#, p(r#""dir":"ltr""#, "مرحبا")));
        assert!(!x.contains("<w:bidi/>"), "{x}");
        assert!(x.contains("<w:rtl/>"));
        // Automatic: by the first strong letter, block by block.
        let x = document_xml(&format!(r#"{{"type":"doc","content":[{},{},{}]}}"#, p(r#""dir":null"#, "مرحبا"), p("", "Hello"), p(r#""dir":null"#, "שלום")));
        assert_eq!(x.matches("<w:bidi/>").count(), 2, "{x}");
    }

    #[test]
    fn explicit_right_alignment_stays_right_in_both_directions() {
        // Under w:bidi no jc is the right side; an English paragraph aligned right gets jc "right".
        let x = document_xml(&format!(r#"{{"type":"doc","content":[{},{}]}}"#, p(r#""textAlign":"right""#, "مرحبا"), p(r#""textAlign":"right""#, "Hello")));
        assert!(x.contains("<w:pPr><w:bidi/></w:pPr>"), "{x}");
        assert!(x.contains("<w:pPr><w:jc w:val=\"right\"/></w:pPr>"), "{x}");
        let x = document_xml(&format!(r#"{{"type":"doc","content":[{}]}}"#, p(r#""textAlign":"center""#, "مرحبا")));
        assert!(x.contains("<w:bidi/><w:jc w:val=\"center\"/>"), "{x}");
    }

    #[test]
    fn arabic_task_items_lists_and_tables_run_right_to_left() {
        let json = r#"{"type":"doc","content":[
            {"type":"taskList","content":[{"type":"taskItem","attrs":{"checked":true},"content":[{"type":"paragraph","content":[{"type":"text","text":"مهمة"}]}]}]},
            {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"بند"}]}]}]},
            {"type":"table","content":[{"type":"tableRow","content":[{"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"الاسم"}]}]}]}]},
            {"type":"table","content":[{"type":"tableRow","content":[{"type":"tableHeader","content":[{"type":"paragraph","content":[{"type":"text","text":"Name"}]}]}]}]}
        ]}"#;
        let x = document_xml(json);
        // task item, bullet item, the Arabic table's cell: three right-to-left paragraphs.
        assert_eq!(x.matches("<w:bidi/>").count(), 3, "{x}");
        assert_eq!(x.matches("<w:bidiVisual/>").count(), 1, "only the Arabic table is drawn right-to-left");
        // bidiVisual comes right after the table style (schema order).
        assert!(x.contains("<w:tblStyle w:val=\"NotayaTable\"/><w:bidiVisual/><w:tblW"), "{x}");
        assert_well_formed("rtl", &x);
    }

    #[test]
    fn xml_text_is_escaped_and_cleaned() {
        assert_eq!(xml("a<b>&\"c\u{1}\u{FFFF}"), "a&lt;b&gt;&amp;&quot;c");
        assert!(!safe_link("javascript:alert(1)"));
        assert!(safe_link("HTTPS://x.y"));
    }
}
