//! PDF documents, written directly (PDF 1.4, A4). The text uses the PDF standard fonts
//! (Helvetica and Courier), which every PDF reader has, so no font is embedded. Those fonts
//! cover Western European letters (Windows-1252); other characters (for example Arabic, Greek,
//! Cyrillic, Chinese or emoji) cannot be shown and are replaced with "?" (emoji are left out),
//! and the export reports how many. Word export keeps them.
//!
//! Layout: each block becomes horizontal strips (a line of text, a picture, a table row) that
//! are then placed on pages; a strip that does not fit starts a new page. Backgrounds of
//! quotes, callouts and code blocks belong to every strip inside them, so they continue across
//! a page break. Pictures: JPEG as it is, PNG decoded (or as it is when the PDF can read it
//! directly); GIF and WebP are not decoded, a short note stands in their place.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::deflate::zlib_compress;
use super::doc::{Align, Block, Cell, Image, Inline, ListKind, Marks, Rgb};
use super::image::{self, Kind};
use super::pdf_fonts::{fallback, win_ansi, HELVETICA, HELVETICA_BOLD};
use super::zip::civil;
use super::{table_grid, ExportDoc, Part, Slot};

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const LEFT: f32 = 56.0;
const TOP: f32 = 56.0;
const BOTTOM: f32 = PAGE_H - 64.0;
const WIDTH: f32 = PAGE_W - 2.0 * LEFT;
const BODY: f32 = 10.5;

const INK: Rgb = Rgb(0x1F, 0x23, 0x2A);
const MUTED: Rgb = Rgb(0x6B, 0x72, 0x80);
const LINK: Rgb = Rgb(0x25, 0x63, 0xEB);
const LINE: Rgb = Rgb(0xC9, 0xCE, 0xD6);
const CODE_BG: Rgb = Rgb(0xF3, 0xF4, 0xF6);
const HEADER_BG: Rgb = Rgb(0xF1, 0xF3, 0xF5);

pub struct Output {
    pub bytes: Vec<u8>,
    /// Pictures that could not be put in the PDF (GIF, WebP, interlaced or broken PNG).
    pub images_left_out: usize,
    /// Characters the fonts do not have.
    pub chars_replaced: usize,
}

pub fn render(doc: &ExportDoc) -> Output {
    render_at(doc, crate::model::now_ms())
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Font {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
    MonoBold,
}

impl Font {
    const ALL: [(Font, &'static str); 6] = [
        (Font::Regular, "Helvetica"),
        (Font::Bold, "Helvetica-Bold"),
        (Font::Italic, "Helvetica-Oblique"),
        (Font::BoldItalic, "Helvetica-BoldOblique"),
        (Font::Mono, "Courier"),
        (Font::MonoBold, "Courier-Bold"),
    ];
    fn pick(bold: bool, italic: bool, mono: bool) -> Font {
        match (mono, bold, italic) {
            (true, true, _) => Font::MonoBold,
            (true, false, _) => Font::Mono,
            (false, true, true) => Font::BoldItalic,
            (false, true, false) => Font::Bold,
            (false, false, true) => Font::Italic,
            (false, false, false) => Font::Regular,
        }
    }
    fn index(self) -> usize {
        Font::ALL.iter().position(|(f, _)| *f == self).unwrap_or(0) + 1
    }
    /// Width of WinAnsi text in points.
    fn width(self, bytes: &[u8], size: f32) -> f32 {
        let units: u32 = bytes
            .iter()
            .map(|&b| match self {
                Font::Mono | Font::MonoBold => 600,
                Font::Bold | Font::BoldItalic => HELVETICA_BOLD[(b.max(32) - 32) as usize] as u32,
                _ => HELVETICA[(b.max(32) - 32) as usize] as u32,
            })
            .sum();
        units as f32 * size / 1000.0
    }
}

/// Something to draw, in strip coordinates (x from the page's left edge, y down from the
/// strip's top; text at its baseline).
#[derive(Clone, Debug)]
enum Op {
    Text { x: f32, y: f32, font: Font, size: f32, color: Rgb, bytes: Vec<u8> },
    Fill { x: f32, y: f32, w: f32, h: f32, color: Rgb },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Rgb },
    Image { x: f32, y: f32, w: f32, h: f32, index: usize },
    Link { x: f32, y: f32, w: f32, h: f32, uri: String },
}

impl Op {
    fn moved(&self, dx: f32, dy: f32) -> Op {
        match self.clone() {
            Op::Text { x, y, font, size, color, bytes } => Op::Text { x: x + dx, y: y + dy, font, size, color, bytes },
            Op::Fill { x, y, w, h, color } => Op::Fill { x: x + dx, y: y + dy, w, h, color },
            Op::Line { x1, y1, x2, y2, width, color } => Op::Line { x1: x1 + dx, y1: y1 + dy, x2: x2 + dx, y2: y2 + dy, width, color },
            Op::Image { x, y, w, h, index } => Op::Image { x: x + dx, y: y + dy, w, h, index },
            Op::Link { x, y, w, h, uri } => Op::Link { x: x + dx, y: y + dy, w, h, uri },
        }
    }
}

/// A background or a bar at the left that runs the full height of every strip of a block.
#[derive(Clone, Debug)]
struct Decor {
    x: f32,
    w: f32,
    fill: Option<Rgb>,
    bar: Option<Rgb>,
}

#[derive(Clone, Debug, Default)]
struct Strip {
    h: f32,
    ops: Vec<Op>,
    decor: Vec<Decor>,
    /// Keep on the same page as the next strip (headings).
    keep_next: bool,
    break_before: bool,
    /// Only spacing: dropped at the top of a page.
    space: bool,
    /// The first line's baseline (where a list marker goes).
    baseline: f32,
    bookmark: Option<String>,
}

#[derive(Clone)]
struct Ctx {
    x: f32,
    w: f32,
    decor: Vec<Decor>,
    shift: u8,
    color: Rgb,
    italic: bool,
    bold: bool,
    list_depth: usize,
}

impl Ctx {
    fn root(shift: u8) -> Ctx {
        Ctx { x: LEFT, w: WIDTH, decor: Vec::new(), shift, color: INK, italic: false, bold: false, list_depth: 0 }
    }
    fn inset(&self, left: f32, right: f32) -> Ctx {
        Ctx { x: self.x + left, w: (self.w - left - right).max(40.0), ..self.clone() }
    }
}

/// How a picture is held in the PDF.
enum PdfImage {
    Jpeg { w: u32, h: u32, components: u8, invert: bool, data: Vec<u8> },
    /// FlateDecode data with PNG predictors (a PNG's own compressed data).
    PngDirect { w: u32, h: u32, depth: u8, colors: u8, palette: Option<Vec<u8>>, data: Vec<u8> },
    Rgb { w: u32, h: u32, rgb: Vec<u8>, alpha: Option<Vec<u8>> },
}

impl PdfImage {
    fn size(&self) -> (u32, u32) {
        match self {
            PdfImage::Jpeg { w, h, .. } | PdfImage::PngDirect { w, h, .. } | PdfImage::Rgb { w, h, .. } => (*w, *h),
        }
    }

    fn build(kind: Kind, bytes: &[u8]) -> Option<PdfImage> {
        match kind {
            Kind::Jpeg => {
                let f = image::jpeg_frame(bytes)?;
                matches!(f.components, 1 | 3 | 4).then(|| PdfImage::Jpeg { w: f.width, h: f.height, components: f.components, invert: f.components == 4 && f.adobe, data: bytes.to_vec() })
            }
            Kind::Png => {
                let p = image::png_parts(bytes).ok()?;
                if p.pdf_passthrough() {
                    let palette = (p.color == 3).then(|| p.palette.clone());
                    Some(PdfImage::PngDirect { w: p.width, h: p.height, depth: p.depth, colors: p.channels() as u8, palette, data: p.idat })
                } else {
                    let (rgb, alpha) = p.decode_rgb().ok()?;
                    Some(PdfImage::Rgb { w: p.width, h: p.height, rgb, alpha })
                }
            }
            Kind::Gif | Kind::Webp => None,
        }
    }
}

struct Layout<'a> {
    doc: &'a ExportDoc,
    images: Vec<PdfImage>,
    image_index: HashMap<String, Option<usize>>,
    images_left_out: usize,
    chars_replaced: usize,
}

/// A piece of text in one style, already in the font's encoding.
#[derive(Clone)]
struct Piece {
    bytes: Vec<u8>,
    font: Font,
    size: f32,
    color: Rgb,
    background: Option<Rgb>,
    underline: bool,
    strike: bool,
    link: Option<String>,
    space: bool,
    width: f32,
}

#[derive(Clone, Copy)]
struct TextStyle {
    size: f32,
    bold: bool,
    italic: bool,
    mono: bool,
    color: Rgb,
    align: Align,
    leading: f32,
}

impl TextStyle {
    fn body(ctx: &Ctx) -> TextStyle {
        TextStyle { size: BODY, bold: ctx.bold, italic: ctx.italic, mono: false, color: ctx.color, align: Align::Left, leading: 1.4 }
    }
}

fn heading_size(level: u8) -> f32 {
    [22.0, 17.0, 14.0, 12.5, 11.5, 11.0][(level.clamp(1, 6) - 1) as usize]
}

impl<'a> Layout<'a> {
    /// Text in the fonts' encoding. Characters the fonts lack become "?" (emoji and invisible
    /// joiners are left out) and are counted.
    fn encode(&mut self, s: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(s.len());
        for c in s.chars() {
            if let Some(b) = win_ansi(c) {
                out.push(b);
            } else if c == '\t' {
                out.extend_from_slice(b"    ");
            } else if let Some(f) = fallback(c) {
                out.extend_from_slice(f.as_bytes());
            } else if matches!(c as u32, 0x200B..=0x200D | 0xFE00..=0xFE0F | 0x0000..=0x001F) {
                // Invisible characters: nothing to show.
            } else if matches!(c as u32, 0x1F000..=0x1FFFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF) {
                self.chars_replaced += 1;
            } else {
                self.chars_replaced += 1;
                out.push(b'?');
            }
        }
        out
    }

    fn pieces(&mut self, content: &[Inline], st: TextStyle) -> Vec<Vec<Piece>> {
        // Lines (split at hard breaks) of pieces; spaces are pieces of their own.
        let mut lines: Vec<Vec<Piece>> = vec![Vec::new()];
        for item in content {
            let (text, marks, math) = match item {
                Inline::Break => {
                    lines.push(Vec::new());
                    continue;
                }
                Inline::Text(t, m) => (t.as_str(), m.clone(), false),
                Inline::Math(m) => (m.as_str(), Marks::default(), true),
            };
            let size = marks.size.map(|s| s * st.size / BODY).unwrap_or(st.size);
            let font = Font::pick(st.bold || marks.bold, st.italic || marks.italic || math, st.mono || marks.code);
            let color = if marks.link.is_some() { LINK } else { marks.color.unwrap_or(st.color) };
            let background = marks.highlight.or(if marks.code && !st.mono { Some(CODE_BG) } else { None });
            let link = marks.link.clone().filter(|u| safe_link(u));
            let mut word = String::new();
            let flush = |word: &mut String, space: bool, lines: &mut Vec<Vec<Piece>>, this: &mut Self| {
                if word.is_empty() {
                    return;
                }
                let bytes = this.encode(word);
                word.clear();
                if bytes.is_empty() {
                    return;
                }
                let width = font.width(&bytes, size);
                lines.last_mut().unwrap().push(Piece {
                    bytes,
                    font,
                    size,
                    color,
                    background,
                    underline: marks.underline || link.is_some(),
                    strike: marks.strike,
                    link: link.clone(),
                    space,
                    width,
                });
            };
            for c in text.chars() {
                if c == '\n' {
                    flush(&mut word, false, &mut lines, self);
                    lines.push(Vec::new());
                } else if c == ' ' {
                    flush(&mut word, false, &mut lines, self);
                    word.push(' ');
                    flush(&mut word, true, &mut lines, self);
                } else {
                    word.push(c);
                }
            }
            flush(&mut word, false, &mut lines, self);
        }
        lines
    }

    /// Wraps inline content to the width and returns one strip per line.
    fn text(&mut self, content: &[Inline], ctx: &Ctx, st: TextStyle) -> Vec<Strip> {
        let mut strips = Vec::new();
        for hard_line in self.pieces(content, st) {
            // Words: runs of pieces without a space between them.
            let mut words: Vec<(Vec<Piece>, Option<Piece>)> = Vec::new();
            let mut current: Vec<Piece> = Vec::new();
            for p in hard_line {
                if p.space {
                    words.push((std::mem::take(&mut current), Some(p)));
                } else {
                    current.push(p);
                }
            }
            words.push((current, None));
            let mut line: Vec<Piece> = Vec::new();
            let mut line_w = 0.0;
            let mut pending: Option<Piece> = None;
            for (word, space) in words {
                let ww: f32 = word.iter().map(|p| p.width).sum();
                let sw = pending.as_ref().map_or(0.0, |p| p.width);
                if !line.is_empty() && line_w + sw + ww > ctx.w {
                    strips.push(self.line_strip(std::mem::take(&mut line), ctx, st));
                    line_w = 0.0;
                    pending = None;
                }
                if ww > ctx.w {
                    // A word longer than the line: cut it.
                    for p in word {
                        let mut rest = p.bytes.clone();
                        while !rest.is_empty() {
                            let room = ctx.w - line_w;
                            // The longest prefix that fits (at least one character), measured in one forward
                            // pass: widths are per character, so a very long word stays linear.
                            let mut n = 0;
                            let mut used = 0.0;
                            while n < rest.len() {
                                let w = p.font.width(&rest[n..n + 1], p.size);
                                if n >= 1 && used + w > room {
                                    break;
                                }
                                used += w;
                                n += 1;
                            }
                            if used > room && !line.is_empty() {
                                strips.push(self.line_strip(std::mem::take(&mut line), ctx, st));
                                line_w = 0.0;
                                continue;
                            }
                            let part: Vec<u8> = rest.drain(..n).collect();
                            let width = p.font.width(&part, p.size);
                            line.push(Piece { bytes: part, width, ..p.clone() });
                            line_w += width;
                            if !rest.is_empty() {
                                strips.push(self.line_strip(std::mem::take(&mut line), ctx, st));
                                line_w = 0.0;
                            }
                        }
                    }
                } else {
                    if let Some(s) = pending.take().filter(|_| !line.is_empty()) {
                        line_w += s.width;
                        line.push(s);
                    }
                    line_w += ww;
                    line.extend(word);
                }
                pending = space;
            }
            strips.push(self.line_strip(line, ctx, st));
        }
        strips
    }

    fn line_strip(&mut self, mut line: Vec<Piece>, ctx: &Ctx, st: TextStyle) -> Strip {
        while line.last().is_some_and(|p| p.space) {
            line.pop();
        }
        let size = line.iter().map(|p| p.size).fold(st.size, f32::max);
        let h = size * st.leading;
        let baseline = (h - size) / 2.0 + size * 0.8;
        let line_w: f32 = line.iter().map(|p| p.width).sum();
        let mut x = ctx.x
            + match st.align {
                Align::Center => (ctx.w - line_w).max(0.0) / 2.0,
                Align::Right => (ctx.w - line_w).max(0.0),
                _ => 0.0,
            };
        let mut ops = Vec::new();
        let mut texts = Vec::new();
        for p in line {
            if let Some(bg) = p.background {
                ops.push(Op::Fill { x, y: baseline - p.size * 0.8, w: p.width, h: p.size * 1.05, color: bg });
            }
            texts.push(Op::Text { x, y: baseline, font: p.font, size: p.size, color: p.color, bytes: p.bytes.clone() });
            if p.underline {
                texts.push(Op::Line { x1: x, y1: baseline + p.size * 0.12, x2: x + p.width, y2: baseline + p.size * 0.12, width: p.size * 0.06, color: p.color });
            }
            if p.strike {
                texts.push(Op::Line { x1: x, y1: baseline - p.size * 0.28, x2: x + p.width, y2: baseline - p.size * 0.28, width: p.size * 0.06, color: p.color });
            }
            if let Some(uri) = &p.link {
                texts.push(Op::Link { x, y: baseline - p.size * 0.85, w: p.width, h: p.size * 1.1, uri: uri.clone() });
            }
            x += p.width;
        }
        ops.extend(texts);
        Strip { h, ops, decor: ctx.decor.clone(), baseline, ..Strip::default() }
    }

    fn space(&self, h: f32, ctx: &Ctx) -> Strip {
        Strip { h, decor: ctx.decor.clone(), space: true, ..Strip::default() }
    }

    fn heading(&mut self, level: u8, content: &[Inline], align: Align, ctx: &Ctx, out: &mut Vec<Strip>) {
        let size = heading_size(level);
        if !out.is_empty() {
            out.push(self.space(size * 0.6, ctx));
        }
        let st = TextStyle { size, bold: true, italic: false, mono: false, color: Rgb(0x11, 0x18, 0x27), align, leading: 1.25 };
        let mut lines = self.text(content, ctx, st);
        for l in &mut lines {
            l.keep_next = true;
        }
        out.extend(lines);
        let mut gap = self.space(size * 0.3, ctx);
        gap.keep_next = true;
        out.push(gap);
    }

    fn blocks(&mut self, blocks: &[Block], ctx: &Ctx, out: &mut Vec<Strip>) {
        for b in blocks {
            self.block(b, ctx, out);
        }
    }

    fn block(&mut self, b: &Block, ctx: &Ctx, out: &mut Vec<Strip>) {
        match b {
            Block::Paragraph { align, content } => {
                let st = TextStyle { align: *align, ..TextStyle::body(ctx) };
                out.extend(self.text(content, ctx, st));
                out.push(self.space(5.0, ctx));
            }
            Block::Heading { level, align, content } => self.heading((level + ctx.shift).min(6), content, *align, ctx, out),
            Block::List { kind, items } => {
                let indent = 18.0;
                let inner = Ctx { list_depth: ctx.list_depth + 1, ..ctx.inset(indent, 0.0) };
                for (n, item) in items.iter().enumerate() {
                    let start = out.len();
                    let mut item_strips = Vec::new();
                    self.blocks(&item.blocks, &inner, &mut item_strips);
                    // The space after the item's last paragraph is smaller inside a list.
                    if item_strips.last().is_some_and(|s| s.space) {
                        item_strips.pop();
                    }
                    if item_strips.is_empty() {
                        item_strips.push(Strip { h: BODY * 1.4, baseline: BODY * 1.2, decor: ctx.decor.clone(), ..Strip::default() });
                    }
                    out.extend(item_strips);
                    out.push(self.space(2.5, ctx));
                    // The strips of an item that holds only spacing (an empty callout, say) get a plain line first,
                    // so there is always a strip to carry the bullet or number.
                    if !out.iter().skip(start).any(|s| !s.space) {
                        let at = start.min(out.len());
                        out.insert(at, Strip { h: BODY * 1.4, baseline: BODY * 1.2, decor: ctx.decor.clone(), ..Strip::default() });
                    }
                    let Some(first) = out.iter_mut().skip(start).find(|s| !s.space) else { continue };
                    let base = if first.baseline > 0.0 { first.baseline } else { BODY };
                    let mx = ctx.x;
                    match kind {
                        ListKind::Bullet => {
                            let mark: &[u8] = [&b"\x95"[..], b"\x96", b"\xB7"][ctx.list_depth % 3];
                            first.ops.push(Op::Text { x: mx + 5.0, y: base, font: Font::Regular, size: BODY, color: ctx.color, bytes: mark.to_vec() });
                        }
                        ListKind::Ordered(s) => {
                            let label = format!("{}.", *s as usize + n).into_bytes();
                            let w = Font::Regular.width(&label, BODY);
                            first.ops.push(Op::Text { x: mx + indent - 4.0 - w, y: base, font: Font::Regular, size: BODY, color: ctx.color, bytes: label });
                        }
                        ListKind::Task => {
                            let (bx, by, s) = (mx + 2.0, base - 8.0, 8.5);
                            for (x1, y1, x2, y2) in [(bx, by, bx + s, by), (bx + s, by, bx + s, by + s), (bx + s, by + s, bx, by + s), (bx, by + s, bx, by)] {
                                first.ops.push(Op::Line { x1, y1, x2, y2, width: 0.8, color: MUTED });
                            }
                            if item.checked == Some(true) {
                                first.ops.push(Op::Line { x1: bx + 1.8, y1: by + 4.4, x2: bx + 3.6, y2: by + 6.6, width: 1.2, color: INK });
                                first.ops.push(Op::Line { x1: bx + 3.6, y1: by + 6.6, x2: bx + 7.0, y2: by + 1.8, width: 1.2, color: INK });
                            }
                        }
                    }
                }
                out.push(self.space(3.0, ctx));
            }
            Block::Quote(inner) => {
                let mut c = ctx.inset(14.0, 0.0);
                c.decor.push(Decor { x: ctx.x + 2.0, w: 3.0, fill: None, bar: Some(LINE) });
                c.color = Rgb(0x4B, 0x55, 0x63);
                self.blocks(inner, &c, out);
            }
            Block::Callout { blocks, .. } => {
                let mut c = ctx.inset(14.0, 10.0);
                c.decor.push(Decor { x: ctx.x, w: ctx.w, fill: Some(Rgb(0xFB, 0xF6, 0xE9)), bar: Some(Rgb(0xE0, 0xA0, 0x30)) });
                out.push(self.space(6.0, &c));
                self.blocks(blocks, &c, out);
                if out.last().is_some_and(|s| s.space) {
                    out.pop();
                }
                out.push(self.space(6.0, &c));
                out.push(self.space(6.0, ctx));
            }
            Block::Toggle(inner) | Block::Shelf(inner) => {
                if let Some((first, rest)) = inner.split_first() {
                    match first {
                        Block::Paragraph { align, content } | Block::Heading { align, content, .. } => {
                            let mut c = vec![Inline::Text("› ".into(), Marks::default())];
                            c.extend(content.iter().cloned());
                            let b = match first {
                                Block::Heading { level, .. } => Block::Heading { level: *level, align: *align, content: c },
                                _ => Block::Paragraph { align: *align, content: c },
                            };
                            self.block(&b, ctx, out);
                        }
                        other => self.block(other, ctx, out),
                    }
                    self.blocks(rest, &ctx.inset(14.0, 0.0), out);
                }
            }
            Block::Columns(cols) => cols.iter().for_each(|c| self.blocks(c, ctx, out)),
            Block::Code { text, .. } => {
                let mut c = ctx.inset(8.0, 8.0);
                c.decor.push(Decor { x: ctx.x, w: ctx.w, fill: Some(CODE_BG), bar: None });
                let st = TextStyle { size: 9.0, bold: false, italic: false, mono: true, color: INK, align: Align::Left, leading: 1.35 };
                out.push(self.space(6.0, &c));
                for line in text.split('\n') {
                    let line = line.replace('\t', "    ");
                    // Keep runs of spaces: every space is its own piece and the line is not trimmed at the start.
                    let content = vec![Inline::Text(if line.is_empty() { " ".into() } else { line.replace(' ', "\u{A0}") }, Marks::default())];
                    out.extend(self.text(&content, &c, st));
                }
                out.push(self.space(6.0, &c));
                out.push(self.space(6.0, ctx));
            }
            Block::Math(latex) => {
                if !latex.trim().is_empty() {
                    let st = TextStyle { italic: true, align: Align::Center, color: Rgb(0x37, 0x41, 0x51), ..TextStyle::body(ctx) };
                    out.extend(self.text(&[Inline::Text(latex.trim().to_string(), Marks::default())], ctx, st));
                    out.push(self.space(6.0, ctx));
                }
            }
            Block::Image(img) => self.image(img, ctx, out),
            Block::Table(rows) => self.table(rows, ctx, out),
            Block::Rule => out.push(Strip {
                h: 14.0,
                ops: vec![Op::Line { x1: ctx.x, y1: 7.0, x2: ctx.x + ctx.w, y2: 7.0, width: 0.7, color: LINE }],
                decor: ctx.decor.clone(),
                ..Strip::default()
            }),
            // A PDF keeps the title only, also when the page is part of the export (no links between PDF files).
            Block::PageLink { title, .. } => {
                let st = TextStyle { italic: true, color: MUTED, ..TextStyle::body(ctx) };
                out.extend(self.text(&[Inline::Text(format!("→ {title}"), Marks::default())], ctx, st));
                out.push(self.space(5.0, ctx));
            }
        }
    }

    fn note(&mut self, text: &str, ctx: &Ctx, out: &mut Vec<Strip>) {
        let st = TextStyle { italic: true, color: MUTED, size: 9.0, ..TextStyle::body(ctx) };
        out.extend(self.text(&[Inline::Text(text.to_string(), Marks::default())], ctx, st));
        out.push(self.space(5.0, ctx));
    }

    fn picture(&mut self, id: &str) -> Option<usize> {
        if let Some(i) = self.image_index.get(id) {
            return *i;
        }
        let built = self.doc.images.get(id).and_then(|p| PdfImage::build(p.kind, &p.bytes));
        let index = built.map(|img| {
            self.images.push(img);
            self.images.len() - 1
        });
        self.image_index.insert(id.to_string(), index);
        index
    }

    fn image(&mut self, img: &Image, ctx: &Ctx, out: &mut Vec<Strip>) {
        let Some(id) = img.attachment.as_ref().filter(|id| self.doc.images.contains_key(*id)) else {
            self.note("(picture not available)", ctx, out);
            return;
        };
        let Some(index) = self.picture(id) else {
            self.images_left_out += 1;
            let kind = self.doc.images[id].kind.name();
            self.note(&format!("[A {kind} picture is not included in the PDF. Export to Word or Markdown to keep it.]"), ctx, out);
            return;
        };
        let (pw, ph) = self.images[index].size();
        let natural = pw as f32 * 0.75; // 96 pixels per inch
        let mut w = match img.width_pct {
            Some(pct) => ctx.w * pct / 100.0,
            None => natural.min(ctx.w),
        };
        let mut h = w * ph as f32 / pw.max(1) as f32;
        let max_h = BOTTOM - TOP - 40.0;
        if h > max_h {
            w *= max_h / h;
            h = max_h;
        }
        let x = ctx.x
            + match img.align {
                Align::Center => (ctx.w - w) / 2.0,
                Align::Right => ctx.w - w,
                _ => 0.0,
            };
        out.push(Strip { h: h + 4.0, ops: vec![Op::Image { x, y: 2.0, w, h, index }], decor: ctx.decor.clone(), ..Strip::default() });
        if !img.caption.trim().is_empty() {
            let st = TextStyle { italic: true, color: MUTED, size: 9.0, align: img.align, ..TextStyle::body(ctx) };
            out.extend(self.text(&[Inline::Text(img.caption.trim().to_string(), Marks::default())], ctx, st));
        }
        out.push(self.space(6.0, ctx));
    }

    fn table(&mut self, rows: &[Vec<Cell>], ctx: &Ctx, out: &mut Vec<Strip>) {
        let grid = table_grid(rows);
        let cols = grid.first().map_or(0, Vec::len);
        if cols == 0 {
            return;
        }
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
        let widths: Vec<f32> = if px.len() == cols && px.iter().all(|w| *w > 0.0) {
            let total: f32 = px.iter().sum();
            px.iter().map(|w| w / total * ctx.w).collect()
        } else {
            vec![ctx.w / cols as f32; cols]
        };
        let xs: Vec<f32> = std::iter::once(0.0).chain(widths.iter().scan(0.0, |acc, w| {
            *acc += w;
            Some(*acc)
        })).collect();
        let pad = 4.0;
        for (r, row) in grid.iter().enumerate() {
            // Lay out each cell that starts in this row, then the row is as tall as its tallest cell.
            let mut cells: Vec<(usize, usize, Vec<Op>, f32, bool)> = Vec::new();
            for (c, slot) in row.iter().enumerate() {
                if let Slot::Cell(cr, ci) = *slot {
                    let cell = &rows[cr][ci];
                    let span = cell.colspan.max(1).min(cols - c);
                    let cw = xs[c + span] - xs[c];
                    let inner = Ctx { x: 0.0, w: (cw - 2.0 * pad).max(10.0), decor: Vec::new(), bold: cell.header, ..ctx.clone() };
                    let mut strips = Vec::new();
                    self.blocks(&cell.blocks, &inner, &mut strips);
                    while strips.last().is_some_and(|s| s.space) {
                        strips.pop();
                    }
                    let (ops, h) = stack(&strips);
                    cells.push((c, span, ops, h, cell.header));
                }
            }
            let row_h = cells.iter().map(|c| c.3).fold(BODY * 1.4, f32::max) + 2.0 * pad;
            let mut ops = Vec::new();
            for (c, span, _, _, header) in &cells {
                if *header {
                    ops.push(Op::Fill { x: ctx.x + xs[*c], y: 0.0, w: xs[c + span] - xs[*c], h: row_h, color: HEADER_BG });
                }
            }
            for (c, _, cell_ops, _, _) in &cells {
                ops.extend(cell_ops.iter().map(|o| o.moved(ctx.x + xs[*c] + pad, pad)));
            }
            // Borders: every cell gets its sides; a cell merged downwards has no line between its rows.
            let below_is_down = |c: usize| grid.get(r + 1).is_some_and(|next| matches!(next[c], Slot::Down(_)));
            for (c, slot) in row.iter().enumerate() {
                let x0 = ctx.x + xs[c];
                let x1 = ctx.x + xs[c + 1];
                let line = |x1_: f32, y1: f32, x2: f32, y2: f32| Op::Line { x1: x1_, y1, x2, y2, width: 0.6, color: LINE };
                if !matches!(slot, Slot::Down(_)) {
                    ops.push(line(x0, 0.0, x1, 0.0));
                }
                if !below_is_down(c) {
                    ops.push(line(x0, row_h, x1, row_h));
                }
                if !matches!(slot, Slot::Right) {
                    ops.push(line(x0, 0.0, x0, row_h));
                }
                if c + 1 == cols {
                    ops.push(line(x1, 0.0, x1, row_h));
                }
            }
            let header_row = rows[r].iter().all(|c| c.header) && !rows[r].is_empty();
            out.push(Strip { h: row_h, ops, decor: ctx.decor.clone(), keep_next: header_row && r == 0, baseline: pad + BODY, ..Strip::default() });
        }
        out.push(self.space(8.0, ctx));
    }
}

/// Puts strips under one another into one list of drawing operations (used inside table cells,
/// which do not break across pages). Returns the operations and the total height.
fn stack(strips: &[Strip]) -> (Vec<Op>, f32) {
    let mut ops = Vec::new();
    let mut y = 0.0;
    for s in strips {
        for d in &s.decor {
            if let Some(fill) = d.fill {
                ops.push(Op::Fill { x: d.x, y, w: d.w, h: s.h, color: fill });
            }
            if let Some(bar) = d.bar {
                ops.push(Op::Fill { x: d.x, y, w: 3.0, h: s.h, color: bar });
            }
        }
        ops.extend(s.ops.iter().map(|o| o.moved(0.0, y)));
        y += s.h;
    }
    (ops, y)
}

fn safe_link(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    (u.starts_with("http://") || u.starts_with("https://") || u.starts_with("mailto:")) && url.is_ascii()
}

struct Placed {
    pages: Vec<Vec<Op>>,
    /// (page, y from the top, title) for the PDF's bookmarks.
    outline: Vec<(usize, f32, String)>,
}

/// Puts the strips on pages.
fn paginate(strips: Vec<Strip>) -> Placed {
    let mut pages: Vec<Vec<Op>> = vec![Vec::new()];
    let mut outline = Vec::new();
    let mut y = TOP;
    for i in 0..strips.len() {
        let s = &strips[i];
        let fresh = y <= TOP + 0.01;
        if s.break_before && !fresh {
            pages.push(Vec::new());
            y = TOP;
        }
        if s.space && y <= TOP + 0.01 {
            continue;
        }
        // A heading stays with what follows it (up to a few strips).
        let mut need = s.h;
        let mut j = i;
        while strips[j].keep_next && j + 1 < strips.len() && j < i + 4 {
            j += 1;
            need += strips[j].h;
        }
        if y + need > BOTTOM && y > TOP + 0.01 {
            pages.push(Vec::new());
            y = TOP;
        }
        let page = pages.last_mut().unwrap();
        for d in &s.decor {
            if let Some(fill) = d.fill {
                page.push(Op::Fill { x: d.x, y, w: d.w, h: s.h, color: fill });
            }
            if let Some(bar) = d.bar {
                page.push(Op::Fill { x: d.x, y, w: 3.0, h: s.h, color: bar });
            }
        }
        page.extend(s.ops.iter().map(|o| o.moved(0.0, y)));
        if let Some(title) = &s.bookmark {
            outline.push((pages.len() - 1, y, title.clone()));
        }
        y += s.h;
    }
    Placed { pages, outline }
}

fn num(v: f32) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

fn color_op(c: Rgb, stroke: bool) -> String {
    format!("{} {} {} {}", num(c.0 as f32 / 255.0), num(c.1 as f32 / 255.0), num(c.2 as f32 / 255.0), if stroke { "RG" } else { "rg" })
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    s.push('<');
    for b in bytes {
        let _ = write!(s, "{b:02X}");
    }
    s.push('>');
    s
}

/// A PDF text string: plain when it is printable ASCII, otherwise UTF-16 with a byte order mark.
fn pdf_string(s: &str) -> String {
    if s.chars().all(|c| (' '..='~').contains(&c)) {
        let escaped = s.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
        format!("({escaped})")
    } else {
        let mut b = vec![0xFE, 0xFF];
        for u in s.encode_utf16() {
            b.extend(u.to_be_bytes());
        }
        hex(&b)
    }
}

pub fn render_at(doc: &ExportDoc, now_ms: i64) -> Output {
    let mut lay = Layout { doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0 };
    let mut strips: Vec<Strip> = Vec::new();
    let mut first = true;
    let mut after_heading = false;
    for part in &doc.parts {
        let (level, title, blocks) = match part {
            Part::Heading { level, text } => (*level, text, None),
            Part::Page { level, title, blocks } => (*level, title, Some(blocks)),
        };
        let start = strips.len();
        let ctx = Ctx::root(level);
        // Headings at the top of a new page need no space above them.
        let mut tmp = Vec::new();
        lay.heading(level, &[Inline::Text(title.clone(), Marks::default())], Align::Left, &ctx, &mut tmp);
        strips.extend(tmp);
        if let Some(s) = strips.get_mut(start) {
            s.break_before = !first && !after_heading;
            s.bookmark = Some(title.clone());
        }
        if let Some(blocks) = blocks {
            lay.blocks(blocks, &ctx, &mut strips);
            after_heading = false;
        } else {
            after_heading = true;
        }
        first = false;
    }
    let placed = paginate(strips);
    let bytes = write_pdf(doc, &lay.images, &placed, now_ms);
    Output { bytes, images_left_out: lay.images_left_out, chars_replaced: lay.chars_replaced }
}

fn write_pdf(doc: &ExportDoc, images: &[PdfImage], placed: &Placed, now_ms: i64) -> Vec<u8> {
    // Objects are collected first (number = index + 1) and written with the cross-reference table at the end.
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let reserve = |objects: &mut Vec<Vec<u8>>| {
        objects.push(Vec::new());
        objects.len()
    };
    let catalog = reserve(&mut objects);
    let pages_obj = reserve(&mut objects);
    let info = reserve(&mut objects);
    let mut font_refs = String::new();
    for (i, (_, name)) in Font::ALL.iter().enumerate() {
        let n = reserve(&mut objects);
        objects[n - 1] = format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>").into_bytes();
        let _ = write!(font_refs, "/F{} {n} 0 R ", i + 1);
    }
    let stream = |dict: &str, data: &[u8]| -> Vec<u8> {
        let mut v = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
        v.extend_from_slice(data);
        v.extend_from_slice(b"\nendstream");
        v
    };
    let mut image_refs = String::new();
    for (i, img) in images.iter().enumerate() {
        let n = reserve(&mut objects);
        let body = match img {
            PdfImage::Jpeg { w, h, components, invert, data } => {
                let cs = match components {
                    1 => "/DeviceGray",
                    4 => "/DeviceCMYK",
                    _ => "/DeviceRGB",
                };
                let decode = if *invert { " /Decode [1 0 1 0 1 0 1 0]" } else { "" };
                stream(&format!("/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace {cs} /BitsPerComponent 8{decode} /Filter /DCTDecode"), data)
            }
            PdfImage::PngDirect { w, h, depth, colors, palette, data } => {
                let cs = match (palette, colors) {
                    (Some(p), _) => format!("[/Indexed /DeviceRGB {} {}]", (p.len() / 3).max(1) - 1, hex(&p[..p.len() / 3 * 3])),
                    (None, 1) => "/DeviceGray".into(),
                    _ => "/DeviceRGB".into(),
                };
                stream(
                    &format!(
                        "/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace {cs} /BitsPerComponent {depth} /Filter /FlateDecode /DecodeParms << /Predictor 15 /Colors {colors} /BitsPerComponent {depth} /Columns {w} >>"
                    ),
                    data,
                )
            }
            PdfImage::Rgb { w, h, rgb, alpha } => {
                let smask = alpha.as_ref().map(|a| {
                    let m = reserve(&mut objects);
                    objects[m - 1] = stream(&format!("/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode"), &zlib_compress(a));
                    m
                });
                let sm = smask.map(|m| format!(" /SMask {m} 0 R")).unwrap_or_default();
                stream(&format!("/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8{sm} /Filter /FlateDecode"), &zlib_compress(rgb))
            }
        };
        objects[n - 1] = body;
        let _ = write!(image_refs, "/Im{} {n} 0 R ", i + 1);
    }
    let resources = format!("<< /Font << {font_refs}>> /XObject << {image_refs}>> >>");

    let total = placed.pages.len();
    let mut page_objs = Vec::new();
    for (pi, ops) in placed.pages.iter().enumerate() {
        let page_n = reserve(&mut objects);
        let content_n = reserve(&mut objects);
        page_objs.push(page_n);
        let mut c = String::new();
        let mut annots = Vec::new();
        let y = |top: f32| num(PAGE_H - top);
        for op in ops {
            match op {
                Op::Fill { x, y: t, w, h, color } => {
                    let _ = writeln!(c, "{} {} {} {} {} re f", color_op(*color, false), num(*x), y(t + h), num(*w), num(*h));
                }
                Op::Line { x1, y1, x2, y2, width, color } => {
                    let _ = writeln!(c, "{} {} w {} {} m {} {} l S", color_op(*color, true), num(*width), num(*x1), y(*y1), num(*x2), y(*y2));
                }
                Op::Text { x, y: t, font, size, color, bytes } => {
                    let _ = writeln!(c, "BT /F{} {} Tf {} {} {} Td {} Tj ET", font.index(), num(*size), color_op(*color, false), num(*x), y(*t), hex(bytes));
                }
                Op::Image { x, y: t, w, h, index } => {
                    let _ = writeln!(c, "q {} 0 0 {} {} {} cm /Im{} Do Q", num(*w), num(*h), num(*x), y(t + h), index + 1);
                }
                Op::Link { x, y: t, w, h, uri } => annots.push((*x, *t, *w, *h, uri.clone())),
            }
        }
        // Page number at the bottom.
        let label = format!("{} / {}", pi + 1, total).into_bytes();
        let lw = Font::Regular.width(&label, 8.0);
        let _ = writeln!(c, "BT /F1 8 Tf {} {} {} Td {} Tj ET", color_op(MUTED, false), num((PAGE_W - lw) / 2.0), num(30.0), hex(&label));

        let mut annot_refs = Vec::new();
        for (x, t, w, h, uri) in annots {
            let a = reserve(&mut objects);
            objects[a - 1] = format!(
                "<< /Type /Annot /Subtype /Link /Rect [{} {} {} {}] /Border [0 0 0] /A << /S /URI /URI {} >> >>",
                num(x),
                y(t + h),
                num(x + w),
                y(t),
                pdf_string(&uri)
            )
            .into_bytes();
            annot_refs.push(format!("{a} 0 R"));
        }
        let annots = if annot_refs.is_empty() { String::new() } else { format!(" /Annots [{}]", annot_refs.join(" ")) };
        objects[content_n - 1] = stream("/Filter /FlateDecode", &zlib_compress(c.as_bytes()));
        objects[page_n - 1] = format!(
            "<< /Type /Page /Parent {pages_obj} 0 R /MediaBox [0 0 {} {}] /Resources {resources} /Contents {content_n} 0 R{annots} >>",
            num(PAGE_W),
            num(PAGE_H)
        )
        .into_bytes();
    }
    let kids: Vec<String> = page_objs.iter().map(|n| format!("{n} 0 R")).collect();
    objects[pages_obj - 1] = format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), page_objs.len()).into_bytes();

    // Bookmarks: one per page title and section heading, in a flat list.
    let mut outlines = String::new();
    if !placed.outline.is_empty() {
        let root = reserve(&mut objects);
        let first = objects.len() + 1;
        let n = placed.outline.len();
        for (i, (page, top, title)) in placed.outline.iter().enumerate() {
            let me = first + i;
            let prev = if i > 0 { format!(" /Prev {} 0 R", me - 1) } else { String::new() };
            let next = if i + 1 < n { format!(" /Next {} 0 R", me + 1) } else { String::new() };
            let item = format!(
                "<< /Title {} /Parent {root} 0 R{prev}{next} /Dest [{} 0 R /XYZ 0 {} null] >>",
                pdf_string(title),
                page_objs[*page],
                num(PAGE_H - top + 4.0)
            );
            objects.push(item.into_bytes());
        }
        objects[root - 1] = format!("<< /Type /Outlines /First {first} 0 R /Last {} 0 R /Count {n} >>", first + n - 1).into_bytes();
        outlines = format!(" /Outlines {root} 0 R /PageMode /UseOutlines");
    }
    objects[catalog - 1] = format!("<< /Type /Catalog /Pages {pages_obj} 0 R{outlines} >>").into_bytes();
    let (yy, mo, d, h, mi, s) = civil(now_ms);
    objects[info - 1] = format!("<< /Title {} /Producer (Notaya) /CreationDate (D:{yy:04}{mo:02}{d:02}{h:02}{mi:02}{s:02}Z) >>", pdf_string(&doc.title)).into_bytes();

    let mut out = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", i + 1).into_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).into_bytes());
    for o in offsets {
        out.extend(format!("{o:010} 00000 n \n").into_bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root {catalog} 0 R /Info {info} 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).into_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::super::deflate::zlib_decompress;
    use super::super::doc::parse;
    use super::super::doc::tests::{every_node_json, titles};
    use super::super::tests::sample_doc;
    use super::*;

    /// Checks the file structure (header, every cross-reference offset, trailer) and returns
    /// each object's text by number, with streams decompressed.
    fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
        hay[from..].windows(needle.len()).position(|w| w == needle).map(|p| p + from)
    }

    fn objects(pdf: &[u8]) -> Vec<String> {
        assert!(pdf.starts_with(b"%PDF-1.4\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let sx = pdf.windows(10).rposition(|w| w == b"startxref\n").unwrap();
        let tail = String::from_utf8(pdf[sx..].to_vec()).unwrap();
        let xref: usize = tail.lines().nth(1).unwrap().parse().unwrap();
        let table = String::from_utf8(pdf[xref..].to_vec()).unwrap();
        assert!(table.starts_with("xref\n0 "), "startxref points at the table");
        let mut lines = table.lines().skip(1);
        let count: usize = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
        assert!(table.contains(&format!("/Size {count}")));
        lines.next(); // the free entry
        let mut out = Vec::new();
        for n in 1..count {
            let entry = lines.next().unwrap();
            assert_eq!(entry.len(), 19, "entries are 20 bytes with the newline");
            let off: usize = entry[..10].parse().unwrap();
            let head = format!("{n} 0 obj\n");
            assert_eq!(&pdf[off..off + head.len()], head.as_bytes(), "object {n} is where the table says");
            let end = find(pdf, b"\nendobj\n", off).unwrap();
            let body = &pdf[off + head.len()..end];
            let s = String::from_utf8_lossy(body).into_owned();
            if let (Some(a), true) = (s.find("stream\n"), s.contains("/FlateDecode") && !s.contains("/Predictor")) {
                let data = &body[a + 7..body.len() - "\nendstream".len()];
                let len: usize = s.split("/Length ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
                assert_eq!(data.len(), len, "stream length of object {n}");
                let plain = zlib_decompress(data, 50_000_000).unwrap();
                out.push(format!("{}\n{}", &s[..a], String::from_utf8_lossy(&plain)));
            } else {
                out.push(s);
            }
        }
        out
    }

    /// The hex digits of some text (found inside a longer text string too).
    fn hex_of(s: &str) -> String {
        hex(s.as_bytes()).trim_matches(['<', '>']).to_string()
    }

    #[test]
    fn a_pdf_has_a_valid_structure_and_all_the_text() {
        let doc = sample_doc(parse(&every_node_json(), &titles));
        let out = render_at(&doc, 1_793_190_896_000);
        let objs = objects(&out.bytes);
        let all = objs.join("\n");
        assert!(all.contains("/Type /Catalog"));
        assert!(all.contains("/BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding"));
        assert!(all.contains("/Subtype /Image"));
        assert!(all.contains("/SMask"), "the half see-through PNG keeps its transparency");
        assert!(all.contains("/S /URI /URI (https://example.com/a?b=1&c=2)"));
        assert!(all.contains("/Outlines") && all.contains("/Title (Sample page)"));
        assert!(all.contains("/CreationDate (D:20261028123456Z)"));
        for word in ["Sample", "Big", "heading", "bold", "italic", "struck", "Bullet", "Third", "Done", "quote.", "callout.", "Toggle", "Hidden", "column", "main()", "Name", "alpha", "spans", "Unknown", "Child", "caption"] {
            assert!(all.contains(&hex_of(word)), "the text {word:?} is drawn");
        }
        // café, the euro sign and curly quotes are in the fonts; the bullet is WinAnsi 0x95.
        assert!(all.contains(&hex(&[b'c', b'a', b'f', 0xE9])));
        assert!(all.contains(&hex(&[0x80])));
        assert!(all.contains("<95> Tj"));
        assert_eq!(out.images_left_out, 0);
        assert_eq!(out.chars_replaced, 0, "every character of the sample is in the fonts (the callout icon is not drawn)");
        let pages = objs.iter().filter(|o| o.contains("/Type /Page ")).count();
        assert!(objs.iter().any(|o| o.contains(&format!("/Count {pages}"))));
    }

    #[test]
    fn long_documents_break_into_pages_and_other_scripts_are_counted() {
        let mut content = Vec::new();
        for i in 0..120 {
            content.push(serde_json::json!({ "type": "paragraph", "content": [{ "type": "text", "text": format!("Paragraph {i} with enough words to fill a line or two of the page, again and again and again.") }] }));
        }
        content.push(serde_json::json!({ "type": "paragraph", "content": [{ "type": "text", "text": "مرحبا" }] }));
        let json = serde_json::json!({ "type": "doc", "content": content }).to_string();
        let out = render_at(&sample_doc(parse(&json, &titles)), 0);
        let objs = objects(&out.bytes);
        let pages = objs.iter().filter(|o| o.contains("/Type /Page ")).count();
        assert!(pages >= 4, "120 paragraphs need several pages, got {pages}");
        assert!(objs.join("\n").contains(&hex_of(&format!("{pages} / {pages}"))), "page numbers");
        assert_eq!(out.chars_replaced, 5);
    }

    #[test]
    fn gif_and_webp_pictures_are_reported() {
        let mut doc = sample_doc(parse(&every_node_json(), &titles));
        doc.images.values_mut().for_each(|p| p.kind = Kind::Gif);
        let out = render_at(&doc, 0);
        assert_eq!(out.images_left_out, 1);
        assert!(!objects(&out.bytes).join("\n").contains("/Subtype /Image"));
    }

    #[test]
    fn jpeg_and_plain_png_pictures_go_in_as_they_are() {
        let jpeg = image::tests::tiny_jpeg_header();
        let img = PdfImage::build(Kind::Jpeg, &jpeg).unwrap();
        assert!(matches!(img, PdfImage::Jpeg { w: 3, h: 2, components: 3, .. }));
        let png = image::tests::make_png(2, 1, 8, 2, &[1, 2, 3, 4, 5, 6], &[], &[]);
        assert!(matches!(PdfImage::build(Kind::Png, &png).unwrap(), PdfImage::PngDirect { colors: 3, depth: 8, .. }));
        assert!(PdfImage::build(Kind::Webp, b"RIFF").is_none());
    }

    #[test]
    fn text_strings_are_encoded() {
        assert_eq!(pdf_string("a(b)\\"), "(a\\(b\\)\\\\)");
        assert_eq!(pdf_string("é"), "<FEFF00E9>");
        assert_eq!(num(1.0), "1");
        assert_eq!(num(0.5), "0.5");
        assert_eq!(num(-0.001), "0");
    }
}
