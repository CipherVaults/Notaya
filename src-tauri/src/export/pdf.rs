//! PDF documents, written directly (PDF 1.4, A4). Western text uses the PDF standard fonts
//! (Helvetica and Courier), which every PDF reader has, so no font is embedded. Those fonts
//! cover Western European letters (Windows-1252). Arabic and Hebrew are drawn with two small
//! fonts bundled in the program, and Chinese, Japanese and Korean with the fonts Windows has
//! (see `pdf_unicode`); only the glyphs used are embedded. Arabic letters are joined
//! (`arabic`), lines are ordered with the Unicode bidirectional algorithm (`bidi`) after they
//! are wrapped, and right-to-left paragraphs are right-aligned. Characters no font has (for
//! example Greek, Cyrillic, or Chinese on a PC without a Chinese font) are replaced with "?"
//! (emoji are left out), and the export reports how many. Word export keeps them.
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
use super::pdf_unicode::{is_cjk, script, GlyphInfo, Hint, Pool};
use super::{arabic, bidi};
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
    /// Characters no font has (shown as "?", or left out for emoji).
    pub chars_replaced: usize,
    /// Of those, Chinese, Japanese or Korean characters for which the PC has no usable font.
    pub chars_no_font: usize,
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
    /// Standard-font text. `actual` is the text as it is read, for pieces drawn out of reading order.
    Text { x: f32, y: f32, font: Font, size: f32, color: Rgb, bytes: Vec<u8>, actual: Option<String> },
    /// Text in an embedded font: the glyphs with their offsets from (x, y) in points (up is positive).
    Glyphs { x: f32, y: f32, w: f32, face: usize, size: f32, color: Rgb, glyphs: Vec<(u16, f32, f32)>, actual: Option<String>, bold: bool, italic: bool },
    Fill { x: f32, y: f32, w: f32, h: f32, color: Rgb },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Rgb },
    Image { x: f32, y: f32, w: f32, h: f32, index: usize },
    Link { x: f32, y: f32, w: f32, h: f32, uri: String },
}

impl Op {
    fn moved(&self, dx: f32, dy: f32) -> Op {
        match self.clone() {
            Op::Text { x, y, font, size, color, bytes, actual } => Op::Text { x: x + dx, y: y + dy, font, size, color, bytes, actual },
            Op::Glyphs { x, y, w, face, size, color, glyphs, actual, bold, italic } => Op::Glyphs { x: x + dx, y: y + dy, w, face, size, color, glyphs, actual, bold, italic },
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
    chars_no_font: usize,
    pool: Pool,
}

/// A glyph of an embedded font with the characters it stands for.
#[derive(Clone)]
struct PGlyph {
    info: GlyphInfo,
    text: String,
}

/// A piece of text in one style, already in the font's encoding (standard fonts) or as glyphs
/// of an embedded font (`bytes` is empty then).
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
    glyphs: Vec<PGlyph>,
    /// Synthetic bold and italic for embedded fonts (they come in one weight).
    bold: bool,
    italic: bool,
    /// Embedding level from the bidirectional algorithm (0 for plain left-to-right text), and
    /// whether the paragraph runs right to left.
    level: u8,
    rtl: bool,
    /// A line may break after this piece although no space follows (Chinese, Japanese, Korean).
    brk: bool,
    /// The characters of a standard-font piece at an odd level whose drawing differs from its
    /// reading (mirrored brackets, reversed runs), for text extraction.
    logical: String,
}

/// The style of one run of inline text.
#[derive(Clone)]
struct Run {
    font: Font,
    size: f32,
    color: Rgb,
    background: Option<Rgb>,
    underline: bool,
    strike: bool,
    link: Option<String>,
    bold: bool,
    italic: bool,
}

impl Run {
    fn piece(&self, bytes: Vec<u8>, space: bool, width: f32) -> Piece {
        Piece {
            bytes,
            font: self.font,
            size: self.size,
            color: self.color,
            background: self.background,
            underline: self.underline,
            strike: self.strike,
            link: self.link.clone(),
            space,
            width,
            glyphs: Vec::new(),
            bold: self.bold,
            italic: self.italic,
            level: 0,
            rtl: false,
            brk: false,
            logical: String::new(),
        }
    }
}

impl Piece {
    /// How many parts the piece can be cut into (characters of a standard-font piece, glyphs of an embedded one).
    fn units(&self) -> usize {
        if self.glyphs.is_empty() { self.bytes.len() } else { self.glyphs.len() }
    }

    fn unit_width(&self, n: usize) -> f32 {
        if self.glyphs.is_empty() { self.font.width(&self.bytes[n..n + 1], self.size) } else { self.glyphs[n].info.adv * self.size }
    }

    /// Cuts the first `n` units off.
    fn split(mut self, n: usize) -> (Piece, Piece) {
        let mut head = self.clone();
        if self.glyphs.is_empty() {
            head.bytes = self.bytes.drain(..n).collect();
            head.width = head.font.width(&head.bytes, head.size);
            self.width = self.font.width(&self.bytes, self.size);
        } else {
            head.glyphs = self.glyphs.drain(..n).collect();
            head.width = head.glyphs.iter().map(|g| g.info.adv * head.size).sum();
            self.width = self.glyphs.iter().map(|g| g.info.adv * self.size).sum();
        }
        head.logical.clear();
        self.logical.clear();
        head.brk = false;
        (head, self)
    }
}

/// Characters that do not start a line, and characters that do not end one (a simple kinsoku).
fn no_line_start(c: char) -> bool {
    "、。，．：；！？）］｝〕〉》」』】〙〗〟’”ぁぃぅぇぉっゃゅょゎゕゖゝゞァィゥェォッャュョヮヵヶ・ーヽヾ々〜,.:;!?)]}%".contains(c)
}

fn no_line_end(c: char) -> bool {
    "（［｛〔〈《「『【〘〖〝‘“([{".contains(c)
}

/// Characters the plain path cannot handle: right-to-left scripts, Chinese, Japanese and Korean,
/// and the invisible direction marks.
fn needs_unicode(c: char) -> bool {
    (c as u32 >= 0x0590 && matches!(bidi::class(c), bidi::Bc::R | bidi::Bc::AL | bidi::Bc::AN)) || is_cjk(c) || matches!(c as u32, 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069)
}

fn invisible(c: char) -> bool {
    matches!(c as u32, 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2069 | 0xFEFF | 0xFE00..=0xFE0F)
}

/// The byte of a mirrored bracket in the standard fonts' encoding.
fn mirror_byte(b: u8) -> u8 {
    match b {
        b'(' => b')',
        b')' => b'(',
        b'[' => b']',
        b']' => b'[',
        b'{' => b'}',
        b'}' => b'{',
        b'<' => b'>',
        b'>' => b'<',
        0xAB => 0xBB,
        0xBB => 0xAB,
        _ => b,
    }
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
    /// The base direction the person chose for the block (0 = left-to-right, 1 = right-to-left); `None` = by the text.
    base: Option<u8>,
}

impl TextStyle {
    fn body(ctx: &Ctx) -> TextStyle {
        TextStyle { size: BODY, bold: ctx.bold, italic: ctx.italic, mono: false, color: ctx.color, align: Align::Left, leading: 1.4, base: None }
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
        let mut runs: Vec<Run> = Vec::new();
        let mut hard: Vec<Vec<(usize, String)>> = vec![Vec::new()];
        for item in content {
            let (text, marks, math) = match item {
                Inline::Break => {
                    hard.push(Vec::new());
                    continue;
                }
                Inline::Text(t, m) => (t.as_str(), m.clone(), false),
                Inline::Math(m) => (m.as_str(), Marks::default(), true),
            };
            let size = marks.size.map(|s| s * st.size / BODY).unwrap_or(st.size);
            let bold = st.bold || marks.bold;
            let italic = st.italic || marks.italic || math;
            let font = Font::pick(bold, italic, st.mono || marks.code);
            let link = marks.link.clone().filter(|u| safe_link(u));
            runs.push(Run {
                font,
                size,
                color: if marks.link.is_some() { LINK } else { marks.color.unwrap_or(st.color) },
                background: marks.highlight.or(if marks.code && !st.mono { Some(CODE_BG) } else { None }),
                underline: marks.underline || link.is_some(),
                strike: marks.strike,
                link,
                bold,
                italic,
            });
            for (k, segment) in text.split('\n').enumerate() {
                if k > 0 {
                    hard.push(Vec::new());
                }
                if !segment.is_empty() {
                    hard.last_mut().unwrap().push((runs.len() - 1, segment.to_string()));
                }
            }
        }
        hard.into_iter()
            .map(|items| {
                if items.iter().any(|(_, t)| t.chars().any(needs_unicode)) {
                    self.complex_line(&items, &runs, st)
                } else {
                    self.simple_line(&items, &runs)
                }
            })
            .collect()
    }

    /// A line of standard-font text: words and spaces.
    fn simple_line(&mut self, items: &[(usize, String)], runs: &[Run]) -> Vec<Piece> {
        let mut line: Vec<Piece> = Vec::new();
        for (ri, text) in items {
            let run = &runs[*ri];
            let mut word = String::new();
            let flush = |word: &mut String, space: bool, line: &mut Vec<Piece>, this: &mut Self| {
                if word.is_empty() {
                    return;
                }
                let bytes = this.encode(word);
                word.clear();
                if bytes.is_empty() {
                    return;
                }
                let width = run.font.width(&bytes, run.size);
                line.push(run.piece(bytes, space, width));
            };
            for c in text.chars() {
                if c == ' ' {
                    flush(&mut word, false, &mut line, self);
                    word.push(' ');
                    flush(&mut word, true, &mut line, self);
                } else {
                    word.push(c);
                }
            }
            flush(&mut word, false, &mut line, self);
        }
        line
    }

    /// A line with right-to-left or Chinese, Japanese or Korean text: levels from the bidi
    /// algorithm, Arabic letters shaped, each script in its own font. The pieces stay in
    /// reading order; `line_strip` puts them in drawing order once the line is wrapped.
    fn complex_line(&mut self, items: &[(usize, String)], runs: &[Run], st: TextStyle) -> Vec<Piece> {
        let mut chars: Vec<char> = Vec::new();
        let mut owner: Vec<usize> = Vec::new();
        for (ri, text) in items {
            for c in text.chars() {
                chars.push(c);
                owner.push(*ri);
            }
        }
        let hint = if chars.iter().any(|c| matches!(*c as u32, 0x3040..=0x30FF)) {
            Hint::Kana
        } else if chars.iter().any(|c| matches!(*c as u32, 0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7FF)) {
            Hint::Hangul
        } else {
            Hint::Han
        };
        let resolved = bidi::resolve(&chars, if st.mono { Some(0) } else { st.base });
        let rtl = resolved.base == 1;
        // What each shaped character is drawn with: a space, standard-font text, or a glyph.
        enum Kind {
            Space,
            Ansi,
            Glyph(GlyphInfo),
        }
        struct Item {
            run: usize,
            level: u8,
            kind: Kind,
            ch: char,
            text: String,
        }
        let mut seq: Vec<Item> = Vec::new();
        for s in arabic::shape(&chars) {
            let first = chars[s.from];
            let c = s.ch;
            if invisible(c) || (c as u32) < 0x20 && c != '\t' {
                continue;
            }
            let text: String = chars[s.from..s.to].iter().collect();
            let kind = if c == ' ' {
                Kind::Space
            } else if win_ansi(c).is_some() || fallback(c).is_some() || c == '\t' {
                Kind::Ansi
            } else if script(c).is_some() {
                match self.pool.glyph(c, hint, &text).or_else(|| if c != first { self.pool.glyph(first, hint, &text) } else { None }) {
                    Some(g) => Kind::Glyph(g),
                    None => {
                        if is_cjk(c) && !self.pool.has_cjk_font() {
                            self.chars_no_font += 1;
                        }
                        Kind::Ansi
                    }
                }
            } else {
                Kind::Ansi
            };
            seq.push(Item { run: owner[s.from], level: resolved.levels[s.from], kind, ch: c, text });
        }
        // Atoms: neighbours in the same style, level and font. A space or a CJK character stands alone.
        let mut line: Vec<Piece> = Vec::new();
        let mut first_last: Vec<(char, char)> = Vec::new();
        let mut i = 0;
        while i < seq.len() {
            let mut j = i + 1;
            let alone = matches!(seq[i].kind, Kind::Space) || is_cjk(seq[i].ch);
            while !alone && j < seq.len() && seq[j].run == seq[i].run && seq[j].level == seq[i].level && !is_cjk(seq[j].ch) {
                let same = match (&seq[i].kind, &seq[j].kind) {
                    (Kind::Ansi, Kind::Ansi) => true,
                    (Kind::Glyph(a), Kind::Glyph(b)) => a.face == b.face,
                    _ => false,
                };
                if !same {
                    break;
                }
                j += 1;
            }
            let run = &runs[seq[i].run];
            let level = seq[i].level;
            let mut piece = match &seq[i].kind {
                Kind::Space => run.piece(vec![b' '], true, run.font.width(b" ", run.size)),
                Kind::Ansi => {
                    let s: String = seq[i..j].iter().map(|it| it.ch).collect();
                    let bytes = self.encode(&s);
                    let width = run.font.width(&bytes, run.size);
                    let mut p = run.piece(bytes, false, width);
                    if level % 2 == 1 && (s.chars().count() > 1 || s.chars().any(|c| bidi::mirror(c) != c)) {
                        p.logical = s;
                    }
                    p
                }
                Kind::Glyph(_) => {
                    let glyphs: Vec<PGlyph> = seq[i..j].iter().filter_map(|it| if let Kind::Glyph(g) = &it.kind { Some(PGlyph { info: *g, text: it.text.clone() }) } else { None }).collect();
                    let width = glyphs.iter().map(|g| g.info.adv * run.size).sum();
                    let mut p = run.piece(Vec::new(), false, width);
                    p.glyphs = glyphs;
                    p
                }
            };
            if piece.units() == 0 {
                i = j;
                continue;
            }
            piece.level = level;
            piece.rtl = rtl;
            first_last.push((seq[i].ch, seq[j - 1].ch));
            line.push(piece);
            i = j;
        }
        // Where a line may break without a space: around Chinese, Japanese and Korean characters.
        for k in 0..line.len().saturating_sub(1) {
            let (_, last) = first_last[k];
            let (next, _) = first_last[k + 1];
            if !line[k].space && !line[k + 1].space && (is_cjk(last) || is_cjk(next)) && !no_line_start(next) && !no_line_end(last) {
                line[k].brk = true;
            }
        }
        line
    }

    /// Wraps inline content to the width and returns one strip per line.
    fn text(&mut self, content: &[Inline], ctx: &Ctx, st: TextStyle) -> Vec<Strip> {
        // A direction chosen with the editor's direction button wins over the text's own.
        let st = match (st.base, super::doc::explicit_dir(content)) {
            (None, super::doc::Dir::Ltr) => TextStyle { base: Some(0), ..st },
            (None, super::doc::Dir::Rtl) => TextStyle { base: Some(1), ..st },
            _ => st,
        };
        let mut strips = Vec::new();
        for hard_line in self.pieces(content, st) {
            // Words: runs of pieces without a space between them.
            let mut words: Vec<(Vec<Piece>, Option<Piece>)> = Vec::new();
            let mut current: Vec<Piece> = Vec::new();
            for p in hard_line {
                if p.space {
                    words.push((std::mem::take(&mut current), Some(p)));
                } else {
                    let end = p.brk;
                    current.push(p);
                    if end {
                        words.push((std::mem::take(&mut current), None));
                    }
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
                        let mut rest = p;
                        while rest.units() > 0 {
                            let room = ctx.w - line_w;
                            // The longest prefix that fits (at least one character), measured in one forward
                            // pass: widths are per character, so a very long word stays linear.
                            let mut n = 0;
                            let mut used = 0.0;
                            while n < rest.units() {
                                let w = rest.unit_width(n);
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
                            let (part, tail) = rest.split(n);
                            line_w += part.width;
                            line.push(part);
                            rest = tail;
                            if rest.units() > 0 {
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
        // A right-to-left paragraph is aligned to the right unless it is centred.
        // A direction the person chose decides this even when the line has only plain letters (English set to right-to-left).
        let rtl = st.base.map_or_else(|| line.first().is_some_and(|p| p.rtl), |b| b == 1);
        let align = match st.align {
            Align::Left | Align::Justify if rtl => Align::Right,
            a => a,
        };
        let start = ctx.x
            + match align {
                Align::Center => (ctx.w - line_w).max(0.0) / 2.0,
                Align::Right => (ctx.w - line_w).max(0.0),
                _ => 0.0,
            };
        // Where each piece goes: the line is ordered visually (the pieces are in reading order).
        let levels: Vec<u8> = line.iter().map(|p| p.level).collect();
        let order: Vec<usize> = if levels.iter().any(|l| l % 2 == 1) { bidi::visual_order(&levels) } else { (0..line.len()).collect() };
        let mut xs = vec![0.0f32; line.len()];
        let mut at = start;
        for &k in &order {
            xs[k] = at;
            at += line[k].width;
        }
        let mut ops = Vec::new();
        let mut texts = Vec::new();
        for (k, p) in line.into_iter().enumerate() {
            let x = xs[k];
            if let Some(bg) = p.background {
                ops.push(Op::Fill { x, y: baseline - p.size * 0.8, w: p.width, h: p.size * 1.05, color: bg });
            }
            let odd = p.level % 2 == 1;
            if p.glyphs.is_empty() {
                let (bytes, actual) = if odd { (p.bytes.iter().rev().map(|&b| mirror_byte(b)).collect(), Some(p.logical.clone()).filter(|l| !l.is_empty())) } else { (p.bytes.clone(), None) };
                texts.push(Op::Text { x, y: baseline, font: p.font, size: p.size, color: p.color, bytes, actual });
            } else {
                texts.push(self.glyph_op(&p, x, baseline));
            }
            if p.underline {
                texts.push(Op::Line { x1: x, y1: baseline + p.size * 0.12, x2: x + p.width, y2: baseline + p.size * 0.12, width: p.size * 0.06, color: p.color });
            }
            if p.strike {
                texts.push(Op::Line { x1: x, y1: baseline - p.size * 0.28, x2: x + p.width, y2: baseline - p.size * 0.28, width: p.size * 0.06, color: p.color });
            }
            if let Some(uri) = &p.link {
                texts.push(Op::Link { x, y: baseline - p.size * 0.85, w: p.width, h: p.size * 1.1, uri: uri.clone() });
            }
        }
        ops.extend(texts);
        Strip { h, ops, decor: ctx.decor.clone(), baseline, ..Strip::default() }
    }

    /// The drawing operation for a piece of embedded-font glyphs. Marks (vowel signs and the like)
    /// are centred over the letter before them, in reading order.
    fn glyph_op(&self, p: &Piece, x: f32, baseline: f32) -> Op {
        let n = p.glyphs.len();
        let odd = p.level % 2 == 1;
        let visual: Vec<usize> = if odd { (0..n).rev().collect() } else { (0..n).collect() };
        let mut pos = vec![0.0f32; n];
        let mut pen = 0.0;
        for &k in &visual {
            let g = &p.glyphs[k].info;
            if !g.mark {
                pos[k] = pen;
                pen += g.adv;
            }
        }
        let mut base: Option<usize> = None;
        let mut stacked = [0u32; 3];
        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            let g = &p.glyphs[k].info;
            let (dx, dy) = if g.mark {
                let (lo, hi) = g.bounds.unwrap_or((0.0, 0.0));
                let mc = p.glyphs[k].text.chars().next().unwrap_or(' ');
                let below = arabic::is_below(mc);
                // Where on the letter the mark sits (shin and sin dots at the sides, holam at the left).
                let at = match mc as u32 {
                    0x05C1 => 0.78,
                    0x05C2 => 0.22,
                    0x05B9 | 0x05BA => 0.3,
                    _ => 0.5,
                };
                let (bx, bw) = match base {
                    Some(b) => (pos[b], p.glyphs[b].info.adv),
                    None => (pos.get(k + 1).copied().unwrap_or(0.0), 0.0),
                };
                // The dagesh sits inside the letter and does not push other marks up.
                let side = if mc == '\u{05BC}' { 2 } else { below as usize };
                let lift = if side == 2 { 0.0 } else { stacked[side] as f32 * 0.3 };
                stacked[side] += 1;
                (bx + bw * at - (lo + hi) / 2.0, if below { -lift } else { lift })
            } else {
                base = Some(k);
                stacked = [0; 3];
                (pos[k], 0.0)
            };
            out.push((g.gid, dx * p.size, dy * p.size));
        }
        if odd {
            out.reverse();
        }
        let actual = odd.then(|| p.glyphs.iter().map(|g| g.text.as_str()).collect::<String>());
        Op::Glyphs { x, y: baseline, w: p.width, face: p.glyphs[0].info.face, size: p.size, color: p.color, glyphs: out, actual, bold: p.bold, italic: p.italic }
    }

    fn space(&self, h: f32, ctx: &Ctx) -> Strip {
        Strip { h, decor: ctx.decor.clone(), space: true, ..Strip::default() }
    }

    fn heading(&mut self, level: u8, content: &[Inline], align: Align, ctx: &Ctx, out: &mut Vec<Strip>) {
        let size = heading_size(level);
        if !out.is_empty() {
            out.push(self.space(size * 0.6, ctx));
        }
        let st = TextStyle { size, bold: true, italic: false, mono: false, color: Rgb(0x11, 0x18, 0x27), align, leading: 1.25, base: None };
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
                for (n, item) in items.iter().enumerate() {
                    // An item that starts with right-to-left text has its marker on the right.
                    let rtl = blocks_rtl(&item.blocks);
                    let inner = Ctx { list_depth: ctx.list_depth + 1, ..if rtl { ctx.inset(0.0, indent) } else { ctx.inset(indent, 0.0) } };
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
                            let x = if rtl { ctx.x + ctx.w - indent + 5.0 } else { mx + 5.0 };
                            first.ops.push(Op::Text { x, y: base, font: Font::Regular, size: BODY, color: ctx.color, bytes: mark.to_vec(), actual: None });
                        }
                        ListKind::Ordered(s) => {
                            let number = *s as usize + n;
                            // In a right-to-left item the full stop comes after the number, so it is drawn to its left.
                            let label = if rtl { format!(".{number}") } else { format!("{number}.") }.into_bytes();
                            let w = Font::Regular.width(&label, BODY);
                            let x = if rtl { ctx.x + ctx.w - indent + 4.0 } else { mx + indent - 4.0 - w };
                            let actual = rtl.then(|| format!("{number}."));
                            first.ops.push(Op::Text { x, y: base, font: Font::Regular, size: BODY, color: ctx.color, bytes: label, actual });
                        }
                        ListKind::Task => {
                            let (bx, by, s) = (if rtl { ctx.x + ctx.w - indent + 4.0 } else { mx + 2.0 }, base - 8.0, 8.5);
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
                let st = TextStyle { size: 9.0, bold: false, italic: false, mono: true, color: INK, align: Align::Left, leading: 1.35, base: None };
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

/// Whether a list item starts with right-to-left text (by its first text block).
fn blocks_rtl(blocks: &[Block]) -> bool {
    let text = |content: &[Inline]| -> Vec<char> {
        content
            .iter()
            .filter_map(|i| match i {
                Inline::Text(t, _) | Inline::Math(t) => Some(t.chars()),
                Inline::Break => None,
            })
            .flatten()
            .collect()
    };
    match blocks.first() {
        Some(Block::Paragraph { content, .. } | Block::Heading { content, .. }) => bidi::base_level(&text(content)) == 1,
        Some(Block::Quote(inner) | Block::Toggle(inner) | Block::Shelf(inner)) => blocks_rtl(inner),
        _ => false,
    }
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
        utf16_string(s)
    }
}

/// A PDF text string as UTF-16 with a byte order mark, always in hexadecimal.
fn utf16_string(s: &str) -> String {
    let mut b = vec![0xFE, 0xFF];
    for u in s.encode_utf16() {
        b.extend(u.to_be_bytes());
    }
    hex(&b)
}

pub fn render_at(doc: &ExportDoc, now_ms: i64) -> Output {
    render_with(doc, now_ms, Pool::system())
}

/// `render_at` with a given pool of fonts (tests use their own font folder).
fn render_with(doc: &ExportDoc, now_ms: i64, pool: Pool) -> Output {
    let mut lay = Layout { doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0, chars_no_font: 0, pool };
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
    let bytes = write_pdf(doc, &lay.images, &lay.pool, &placed, now_ms);
    Output { bytes, images_left_out: lay.images_left_out, chars_replaced: lay.chars_replaced, chars_no_font: lay.chars_no_font }
}

fn write_pdf(doc: &ExportDoc, images: &[PdfImage], pool: &Pool, placed: &Placed, now_ms: i64) -> Vec<u8> {
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
    // Embedded fonts (Arabic, Hebrew, Chinese, Japanese, Korean): five objects each.
    for slot in pool.used() {
        let first = objects.len() + 1;
        if let Some(parts) = pool.objects(slot, first) {
            objects.extend(parts);
            let _ = write!(font_refs, "/FA{slot} {first} 0 R ");
        }
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
                Op::Text { x, y: t, font, size, color, bytes, actual } => {
                    if let Some(a) = actual {
                        let _ = writeln!(c, "/Span << /ActualText {} >> BDC", utf16_string(a));
                    }
                    let _ = writeln!(c, "BT /F{} {} Tf {} {} {} Td {} Tj ET", font.index(), num(*size), color_op(*color, false), num(*x), y(*t), hex(bytes));
                    if actual.is_some() {
                        c.push_str("EMC\n");
                    }
                }
                Op::Glyphs { x, y: t, face, size, color, glyphs, actual, bold, italic, .. } => {
                    if let Some(a) = actual {
                        let _ = writeln!(c, "/Span << /ActualText {} >> BDC", utf16_string(a));
                    }
                    let _ = write!(c, "q {} ", color_op(*color, false));
                    if *bold {
                        let _ = write!(c, "{} 2 Tr {} w ", color_op(*color, true), num(size * 0.03));
                    }
                    let _ = write!(c, "BT /FA{face} {} Tf ", num(*size));
                    let shear = if *italic { "0.2" } else { "0" };
                    for (gid, dx, dy) in glyphs {
                        let _ = write!(c, "1 0 {shear} 1 {} {} Tm {} Tj ", num(x + dx), num(PAGE_H - t + dy), hex(&gid.to_be_bytes()));
                    }
                    c.push_str("ET Q\n");
                    if actual.is_some() {
                        c.push_str("EMC\n");
                    }
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
        content.push(serde_json::json!({ "type": "paragraph", "content": [{ "type": "text", "text": "Привет" }] }));
        let json = serde_json::json!({ "type": "doc", "content": content }).to_string();
        let out = render_at(&sample_doc(parse(&json, &titles)), 0);
        let objs = objects(&out.bytes);
        let pages = objs.iter().filter(|o| o.contains("/Type /Page ")).count();
        assert!(pages >= 4, "120 paragraphs need several pages, got {pages}");
        assert!(objs.join("\n").contains(&hex_of(&format!("{pages} / {pages}"))), "page numbers");
        assert_eq!(out.chars_replaced, 6, "Cyrillic is not in any of the fonts");
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

    // ---- Arabic, Hebrew, Chinese, Japanese and Korean ----

    use super::super::ttf::tests::test_font;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    fn para(text: &str) -> Value {
        json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] })
    }

    fn doc_of(blocks: Vec<Value>) -> ExportDoc {
        sample_doc(parse(&json!({ "type": "doc", "content": blocks }).to_string(), &titles))
    }

    /// A folder with the synthetic test font standing in for the Chinese, Japanese and Korean fonts of Windows.
    fn font_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("notaya-pdf-test-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["msyh.ttc", "YuGothR.ttc", "malgun.ttf"] {
            std::fs::write(dir.join(f), test_font(0)).unwrap();
        }
        dir
    }

    /// The strips of one paragraph laid out at the page width (or a given width).
    fn strips_of(text: &str, width: Option<f32>, st: impl Fn(&Ctx) -> TextStyle, pool: Pool) -> (Vec<Strip>, usize) {
        let doc = sample_doc(Vec::new());
        let mut lay = Layout { doc: &doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0, chars_no_font: 0, pool };
        let mut ctx = Ctx::root(0);
        if let Some(w) = width {
            ctx.w = w;
        }
        let style = st(&ctx);
        let strips = lay.text(&[Inline::Text(text.into(), Marks::default())], &ctx, style);
        (strips, lay.chars_no_font)
    }

    fn body(ctx: &Ctx) -> TextStyle {
        TextStyle::body(ctx)
    }

    /// (x, width) of the drawn pieces of a strip, standard-font text included.
    fn spans(strip: &Strip) -> Vec<(f32, f32)> {
        strip
            .ops
            .iter()
            .filter_map(|o| match o {
                Op::Glyphs { x, w, .. } => Some((*x, *w)),
                Op::Text { x, font, size, bytes, .. } => Some((*x, font.width(bytes, *size))),
                _ => None,
            })
            .collect()
    }

    /// The text a viewer would extract from the content streams: the ActualText of a span when
    /// there is one, otherwise the characters of the glyphs through the font's ToUnicode map.
    fn extract(pdf: &[u8]) -> String {
        let objs = objects(pdf);
        let parse_hex = |s: &str| -> Vec<u8> { (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect() };
        let utf16 = |b: &[u8]| -> String { String::from_utf16(&b.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect::<Vec<_>>()).unwrap() };
        // font resource -> (gid -> text)
        let mut maps: HashMap<String, HashMap<u16, String>> = HashMap::new();
        for o in &objs {
            if o.contains("/Type /Page ") {
                for part in o.split("/FA").skip(1) {
                    let (n, rest) = part.split_once(' ').unwrap();
                    let type0: usize = rest.split(' ').next().unwrap().parse().unwrap();
                    let tu: usize = objs[type0 - 1].split("/ToUnicode ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
                    let mut m = HashMap::new();
                    for line in objs[tu - 1].lines().filter(|l| l.starts_with('<') && l.contains("> <")) {
                        let (a, b) = line.split_once("> <").unwrap();
                        m.insert(u16::from_str_radix(a.trim_start_matches('<'), 16).unwrap(), utf16(&parse_hex(b.trim_end_matches('>'))));
                    }
                    maps.insert(format!("FA{n}"), m);
                }
            }
        }
        let mut out = String::new();
        for o in &objs {
            if !(o.contains(" Tf ") && o.contains(" Tj")) {
                continue;
            }
            let mut actual: Option<String> = None;
            let mut spent = false;
            for line in o.lines() {
                if let Some(rest) = line.strip_prefix("/Span << /ActualText <") {
                    actual = Some(utf16(&parse_hex(rest.split('>').next().unwrap())[2..]));
                    spent = false;
                } else if line == "EMC" {
                    actual = None;
                } else if line.contains(" Tj") {
                    // Every text object of the line: font, then the strings shown.
                    for tok in line.split("BT ").skip(1) {
                        let font = tok.split(' ').next().unwrap().trim_start_matches('/').to_string();
                        let mut shown = String::new();
                        for part in tok.split(" Tj").filter(|p| p.contains('<')) {
                            let hexs = part.rsplit('<').next().unwrap().trim_end_matches('>').to_string();
                            let bytes = parse_hex(&hexs);
                            if let Some(m) = maps.get(&font) {
                                for pair in bytes.chunks(2) {
                                    shown.push_str(&m[&u16::from_be_bytes([pair[0], pair[1]])]);
                                }
                            } else {
                                shown.extend(bytes.iter().map(|&b| b as char));
                            }
                        }
                        match &actual {
                            Some(a) if !spent => {
                                out.push_str(a);
                                spent = true;
                            }
                            Some(_) => {}
                            None => out.push_str(&shown),
                        }
                    }
                }
            }
        }
        // The page title is drawn first and the page number last.
        out.strip_prefix("Sample page").and_then(|s| s.strip_suffix("1 / 1")).unwrap_or(&out).to_string()
    }

    const ARABIC_SAMPLE: &str = "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ، لا إله إلا الله";

    #[test]
    fn arabic_hebrew_and_cjk_are_embedded_and_read_back_as_written() {
        let texts = [ARABIC_SAMPLE, "שָׁלוֹם עולם!", "中中あ、中、가", "اشتريت 3 كتب من Amazon بسعر 25.50 دولار (حوالي 100 ريال).", "plain English", "گچپژ ٠١٢٣"];
        let doc = doc_of(texts.iter().map(|t| para(t)).collect());
        let out = render_with(&doc, 0, Pool::new(Some(font_dir("read-back"))));
        assert_eq!((out.chars_replaced, out.chars_no_font), (0, 0));
        let objs = objects(&out.bytes);
        let all = objs.join("\n");
        assert!(all.contains("/Subtype /Type0") && all.contains("/Encoding /Identity-H") && all.contains("/CIDToGIDMap /Identity") && all.contains("/FontFile2"));
        assert!(all.contains("/Subtype /CIDFontType2") && all.contains("/W [") && all.contains("/ToUnicode"));
        let fonts = all.matches("/Subtype /Type0").count();
        assert_eq!(fonts, 4, "Arabic, Hebrew, and two of the three Chinese/Japanese/Korean fonts (kana and Han come from the Japanese one, Hangul from the Korean one)");
        assert_eq!(extract(&out.bytes), texts.concat());
    }

    #[test]
    fn embedded_fonts_are_valid_subsets_with_widths() {
        let doc = doc_of(vec![para(ARABIC_SAMPLE), para("שלום")]);
        let out = render_at(&doc, 0);
        // Find the font files and read them back.
        let pdf = &out.bytes;
        let mut faces = 0;
        let mut at = 0;
        while let Some(p) = find(pdf, b"/Length1 ", at) {
            let a = find(pdf, b"stream\n", p).unwrap() + 7;
            let b = find(pdf, b"\nendstream", a).unwrap();
            let file = zlib_decompress(&pdf[a..b], 10_000_000).unwrap();
            let len1: usize = std::str::from_utf8(&pdf[p + 9..p + 9 + 7]).unwrap().split(' ').next().unwrap().parse().unwrap();
            assert_eq!(file.len(), len1);
            let face = super::super::ttf::Face::parse(file, 0).expect("the subset is a TrueType font");
            assert!(face.num_glyphs > 5 && face.num_glyphs < 600);
            faces += 1;
            at = b;
        }
        assert_eq!(faces, 2);
        let all = objects(pdf).join("\n");
        assert!(all.contains("/FontName /"), "subset tag and name");
        let tag = all.split("/FontName /").nth(1).unwrap().split('+').next().unwrap();
        assert!(tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase()));
        // The file is smaller than the whole bundled font: only the glyphs used are in it.
        assert!(pdf.len() < 60_000, "{} bytes", pdf.len());
    }

    #[test]
    fn documents_without_other_scripts_embed_nothing_and_keep_the_standard_fonts() {
        let out = render_at(&sample_doc(parse(&every_node_json(), &titles)), 0);
        let all = objects(&out.bytes).join("\n");
        assert!(!all.contains("/Type0") && !all.contains("/FA") && !all.contains("ActualText") && !all.contains(" Tm "));
    }

    #[test]
    fn right_to_left_paragraphs_are_right_aligned_and_left_to_right_ones_left() {
        let right = LEFT + WIDTH;
        for text in ["مرحبا بالعالم", "שלום עולם", "123 مرحبا"] {
            let (strips, _) = strips_of(text, None, body, Pool::new(None));
            let line = spans(&strips[0]);
            let end = line.iter().map(|(x, w)| x + w).fold(0.0, f32::max);
            assert!((end - right).abs() < 0.01, "{text}: ends at {end}, not at {right}");
        }
        for text in ["Hello world", "Hello مرحبا", "你好"] {
            let (strips, _) = strips_of(text, None, body, Pool::new(None));
            let start = spans(&strips[0]).iter().map(|(x, _)| *x).fold(f32::MAX, f32::min);
            assert!((start - LEFT).abs() < 0.01, "{text}: starts at {start}");
        }
        // Centred stays centred.
        let (strips, _) = strips_of("مرحبا", None, |c| TextStyle { align: Align::Center, ..TextStyle::body(c) }, Pool::new(None));
        let line = spans(&strips[0]);
        let mid = (line.iter().map(|(x, _)| *x).fold(f32::MAX, f32::min) + line.iter().map(|(x, w)| x + w).fold(0.0, f32::max)) / 2.0;
        assert!((mid - (LEFT + WIDTH / 2.0)).abs() < 0.01);
    }

    #[test]
    fn a_direction_chosen_by_the_person_wins_over_the_text() {
        // Arabic text set to left-to-right starts at the left edge; by itself it ends at the right edge.
        let laid = |dir: super::super::doc::Dir| {
            let doc = sample_doc(Vec::new());
            let mut lay = Layout { doc: &doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0, chars_no_font: 0, pool: Pool::new(None) };
            let ctx = Ctx::root(0);
            let style = TextStyle::body(&ctx);
            let strips = lay.text(&[Inline::Text("مرحبا".into(), Marks { dir, ..Marks::default() })], &ctx, style);
            spans(&strips[0])
        };
        let auto = laid(super::super::doc::Dir::Auto);
        let ltr = laid(super::super::doc::Dir::Ltr);
        let auto_start = auto.iter().map(|(x, _)| *x).fold(f32::MAX, f32::min);
        let ltr_start = ltr.iter().map(|(x, _)| *x).fold(f32::MAX, f32::min);
        assert!(auto_start > LEFT + 1.0, "automatic: right-aligned, starts at {auto_start}");
        assert!((ltr_start - LEFT).abs() < 0.01, "left-to-right: starts at the left edge, not {ltr_start}");
        // English text set to right-to-left ends at the right edge.
        let doc = sample_doc(Vec::new());
        let mut lay = Layout { doc: &doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0, chars_no_font: 0, pool: Pool::new(None) };
        let ctx = Ctx::root(0);
        let style = TextStyle::body(&ctx);
        let strips = lay.text(&[Inline::Text("Hello world".into(), Marks { dir: super::super::doc::Dir::Rtl, ..Marks::default() })], &ctx, style);
        let end = spans(&strips[0]).iter().map(|(x, w)| x + w).fold(0.0, f32::max);
        assert!((end - (LEFT + WIDTH)).abs() < 0.01, "English set to right-to-left: ends at {end}");
    }

    #[test]
    fn mixed_lines_are_ordered_visually() {
        // Hebrew paragraph with an English word in the middle: [עולם] [hello] [שלום] from the left.
        let (strips, _) = strips_of("שלום hello עולם", None, body, Pool::new(None));
        let xs: Vec<f32> = strips[0].ops.iter().filter_map(|o| match o { Op::Glyphs { x, .. } | Op::Text { x, .. } => Some(*x), _ => None }).collect();
        // Reading order of the ops: שלום, space, hello, space, עולם.
        assert_eq!(xs.len(), 5);
        assert!(xs[4] < xs[2] && xs[2] < xs[0], "{xs:?}");
        // Arabic with a number: [ريال] [123] [سعر].
        let (strips, _) = strips_of("سعر 123 ريال", None, body, Pool::new(None));
        let xs: Vec<f32> = strips[0].ops.iter().filter_map(|o| match o { Op::Glyphs { x, .. } | Op::Text { x, .. } => Some(*x), _ => None }).collect();
        assert!(xs[4] < xs[2] && xs[2] < xs[0], "{xs:?}");
        // English paragraph with an Arabic word: Hello then the Arabic, in the usual order.
        let (strips, _) = strips_of("Hello مرحبا world", None, body, Pool::new(None));
        let xs: Vec<f32> = strips[0].ops.iter().filter_map(|o| match o { Op::Glyphs { x, .. } | Op::Text { x, .. } => Some(*x), _ => None }).collect();
        assert!(xs[0] < xs[2] && xs[2] < xs[4], "{xs:?}");
        // Brackets around English in a Hebrew paragraph are mirrored and keep their pairing.
        let (strips, _) = strips_of("שלום (hello)", None, body, Pool::new(None));
        let texts: Vec<(f32, Vec<u8>, Option<String>)> = strips[0].ops.iter().filter_map(|o| match o { Op::Text { x, bytes, actual, .. } => Some((*x, bytes.clone(), actual.clone())), _ => None }).collect();
        let open = texts.iter().find(|t| t.2.as_deref() == Some("(")).expect("the opening bracket carries its reading text");
        let close = texts.iter().find(|t| t.2.as_deref() == Some(")")).unwrap();
        assert_eq!(open.1, b")", "drawn mirrored");
        assert_eq!(close.1, b"(");
        assert!(close.0 < open.0, "the closing bracket is on the left in a right-to-left line");
    }

    #[test]
    fn long_paragraphs_wrap_logically_then_reorder_per_line() {
        let text = "هذه فقرة عربية طويلة تحتوي على كلمات English مختلطة ورقم 12345 وأخرى (بين قوسين) لاختبار التفاف الأسطر وإعادة ترتيبها بصرياً سطراً بسطر";
        let (strips, _) = strips_of(text, Some(180.0), body, Pool::new(None));
        assert!(strips.len() >= 4, "{} lines", strips.len());
        for s in &strips {
            let line = spans(s);
            assert!(!line.is_empty());
            for (x, w) in &line {
                assert!(*x >= LEFT - 0.01 && x + w <= LEFT + 180.0 + 0.01, "inside the column: {x} {w}");
            }
            let end = line.iter().map(|(x, w)| x + w).fold(0.0, f32::max);
            assert!((end - (LEFT + 180.0)).abs() < 0.01, "every line of the paragraph is right-aligned");
        }
        // Reading order is kept in the content: the ops of all lines, in order, spell the paragraph.
        let doc = doc_of(vec![para(text)]);
        let out = render_at(&doc, 0);
        assert_eq!(extract(&out.bytes).replace(' ', ""), text.replace(' ', ""));
    }

    #[test]
    fn chinese_japanese_and_korean_wrap_at_any_character_with_a_simple_kinsoku() {
        let pool = || Pool::new(Some(font_dir("wrap")));
        // 10.5 points per character, 100 points wide: nine fit.
        let (strips, none) = strips_of(&"中".repeat(30), Some(100.0), body, pool());
        assert_eq!(none, 0);
        let counts: Vec<usize> = strips.iter().map(|s| s.ops.iter().filter(|o| matches!(o, Op::Glyphs { .. })).count()).collect();
        assert_eq!(counts, vec![9, 9, 9, 3]);
        // A full stop never starts a line: the last character before it moves down with it.
        let (strips, _) = strips_of(&format!("{}。{}", "中".repeat(9), "中".repeat(5)), Some(100.0), body, pool());
        let gids = |s: &Strip| -> Vec<u16> { s.ops.iter().filter_map(|o| if let Op::Glyphs { glyphs, .. } = o { Some(glyphs[0].0) } else { None }).collect() };
        assert_eq!(gids(&strips[0]).len(), 8);
        assert_eq!(&gids(&strips[1])[..2], &[3, 6]);
        // An opening bracket never ends a line.
        let (strips, _) = strips_of(&format!("{}（中中", "中".repeat(8)), Some(100.0), body, pool());
        assert_eq!(gids(&strips[0]).len(), 8);
        assert_eq!(&gids(&strips[1])[..2], &[6, 3]);
        // Latin words stay whole next to Chinese text.
        let (strips, _) = strips_of("中中中中中中中中hello中", Some(100.0), body, pool());
        assert_eq!(strips.len(), 2);
        // Left to right, left aligned.
        assert!((spans(&strips[0])[0].0 - LEFT).abs() < 0.01);
    }

    #[test]
    fn without_a_cjk_font_the_characters_are_question_marks_and_reported() {
        let doc = doc_of(vec![para("中文 and 한국어 ok")]);
        let out = render_with(&doc, 0, Pool::new(None));
        assert_eq!((out.chars_replaced, out.chars_no_font), (5, 5));
        let all = objects(&out.bytes).join("\n");
        assert!(!all.contains("/Type0") && all.contains(&hex_of("?")));
        // A font folder without usable fonts is the same.
        let empty = std::env::temp_dir().join(format!("notaya-pdf-test-empty-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        let out = render_with(&doc, 0, Pool::new(Some(empty.clone())));
        assert_eq!((out.chars_replaced, out.chars_no_font), (5, 5));
        std::fs::remove_dir_all(empty).ok();
        // Characters that the font lacks, when there is a font, are replaced but not "missing font".
        let out = render_with(&doc_of(vec![para("中文")]), 0, Pool::new(Some(font_dir("lacks"))));
        assert_eq!((out.chars_replaced, out.chars_no_font), (1, 0));
    }

    #[test]
    fn arabic_bold_and_italic_use_synthetic_styles_and_links_and_marks_are_kept() {
        let json = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
            { "type": "text", "text": "غامق", "marks": [{ "type": "bold" }] },
            { "type": "text", "text": " ", "marks": [] },
            { "type": "text", "text": "مائل", "marks": [{ "type": "italic" }, { "type": "underline" }] },
        ] }] });
        let out = render_at(&sample_doc(parse(&json.to_string(), &titles)), 0);
        let all = objects(&out.bytes).join("\n");
        assert!(all.contains(" 2 Tr "), "bold is drawn as fill and stroke");
        assert!(all.contains("1 0 0.2 1 "), "italic is slanted");
        assert_eq!(extract(&out.bytes), "غامق مائل");
    }

    #[test]
    fn list_items_in_arabic_have_their_marker_on_the_right() {
        let doc = doc_of(vec![
            json!({ "type": "bulletList", "content": [
                { "type": "listItem", "content": [para("عنصر أول")] },
                { "type": "listItem", "content": [para("item")] },
            ] }),
            json!({ "type": "orderedList", "content": [{ "type": "listItem", "content": [para("עברית")] }] }),
        ]);
        let ctx = Ctx::root(0);
        let mut lay = Layout { doc: &doc, images: Vec::new(), image_index: HashMap::new(), images_left_out: 0, chars_replaced: 0, chars_no_font: 0, pool: Pool::new(None) };
        let mut strips = Vec::new();
        lay.blocks(match &doc.parts[0] { Part::Page { blocks, .. } => blocks, _ => unreachable!() }, &ctx, &mut strips);
        let lines: Vec<&Strip> = strips.iter().filter(|s| !s.space).collect();
        let marker = |s: &Strip| s.ops.iter().rev().find_map(|o| if let Op::Text { x, bytes, .. } = o { Some((*x, bytes.clone())) } else { None });
        // Arabic item: marker near the right edge, text ending left of it.
        let (mx, mb) = marker(lines[0]).unwrap();
        assert_eq!(mb, b"\x95");
        assert!((mx - (LEFT + WIDTH - 18.0 + 5.0)).abs() < 0.01);
        let text_end = spans(lines[0]).iter().filter(|(x, _)| *x != mx).map(|(x, w)| x + w).fold(0.0, f32::max);
        assert!((text_end - (LEFT + WIDTH - 18.0)).abs() < 0.01, "{text_end}");
        // English item: marker at the left.
        assert!((marker(lines[1]).unwrap().0 - (LEFT + 5.0)).abs() < 0.01);
        // Hebrew numbered item: the label is ".1" drawn at the right, read as "1.".
        let (nx, nb) = marker(lines[2]).unwrap();
        assert_eq!(nb, b".1");
        assert!(nx > LEFT + WIDTH - 18.0);
    }

    #[test]
    fn code_blocks_keep_their_direction_but_still_show_the_letters() {
        let doc = doc_of(vec![json!({ "type": "codeBlock", "content": [{ "type": "text", "text": "x = \"مرحبا\"" }] })]);
        let out = render_at(&doc, 0);
        assert_eq!(out.chars_replaced, 0);
        let (strips, _) = strips_of("x = a", None, |c| TextStyle { mono: true, size: 9.0, ..TextStyle::body(c) }, Pool::new(None));
        assert!((spans(&strips[0])[0].0 - LEFT).abs() < 0.01);
    }

    #[test]
    fn tables_headings_and_empty_lines_with_other_scripts_do_not_break_the_layout() {
        let doc = doc_of(vec![
            json!({ "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "عنوان" }] }),
            json!({ "type": "table", "content": [{ "type": "tableRow", "content": [
                { "type": "tableHeader", "content": [para("اسم")] },
                { "type": "tableHeader", "content": [para("Name")] },
            ] }, { "type": "tableRow", "content": [
                { "type": "tableCell", "content": [para("أحمد")] },
                { "type": "tableCell", "content": [para("中中")] },
            ] }] }),
            json!({ "type": "paragraph", "content": [{ "type": "text", "text": "سطر" }, { "type": "hardBreak" }, { "type": "text", "text": "line two" }, { "type": "hardBreak" }, { "type": "hardBreak" }, { "type": "text", "text": "\u{200F}" }] }),
        ]);
        let out = render_with(&doc, 0, Pool::new(Some(font_dir("table"))));
        assert_eq!((out.chars_replaced, out.chars_no_font), (0, 0));
        objects(&out.bytes);
        assert_eq!(extract(&out.bytes), "عنواناسمNameأحمد中中سطرline two");
    }

    /// Writes sample PDFs for looking at (not run by default): `NOTAYA_PDF_SAMPLES` is the folder
    /// to write to and `NOTAYA_PDF_CJK_FONTS` a folder with `msyh.ttc`, `YuGothR.ttc` and `malgun.ttf`.
    #[test]
    #[ignore]
    fn write_sample_pdfs() {
        let Some(dir) = std::env::var_os("NOTAYA_PDF_SAMPLES").map(PathBuf::from) else { return };
        std::fs::create_dir_all(&dir).unwrap();
        let fonts = std::env::var_os("NOTAYA_PDF_CJK_FONTS").map(PathBuf::from);
        let styled = |text: &str, marks: Vec<Value>| json!({ "type": "text", "text": text, "marks": marks });
        let blocks = vec![
            json!({ "type": "heading", "attrs": { "level": 1 }, "content": [{ "type": "text", "text": "Notaya PDF: Unicode samples" }] }),
            para("English: The quick brown fox jumps over the lazy dog. Plain Western text still uses the standard PDF fonts."),
            json!({ "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "العربية" }] }),
            para("مرحبا بكم في نوتايا، تطبيق الملاحظات المحلي. هذه فقرة عربية طويلة لاختبار التفاف الأسطر والحروف المتصلة، والكتابة من اليمين إلى اليسار، مع محاذاة النص إلى اليمين."),
            para("بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ — لا إله إلا الله — الحمد لله رب العالمين"),
            para("اشتريت 3 كتب من Amazon بسعر 25.50 دولار (حوالي 100 ريال) في 2026/11/03، والرقم ٢٠٢٦ بالأرقام العربية."),
            para("Mixed line: the word مرحبا means hello and شكراً means thanks (شكراً جزيلاً)."),
            json!({ "type": "paragraph", "content": [styled("نص غامق", vec![json!({ "type": "bold" })]), styled(" ونص مائل ", vec![json!({ "type": "italic" })]), styled("ورابط", vec![json!({ "type": "link", "attrs": { "href": "https://example.com" } })])] }),
            para("گچپژ — فارسی: سلام دنیا، این یک متن فارسی است."),
            json!({ "type": "bulletList", "content": [{ "type": "listItem", "content": [para("عنصر أول في القائمة")] }, { "type": "listItem", "content": [para("English item")] }] }),
            json!({ "type": "orderedList", "content": [{ "type": "listItem", "content": [para("البند الأول")] }, { "type": "listItem", "content": [para("البند الثاني")] }] }),
            json!({ "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "עברית" }] }),
            para("שלום עולם! זוהי פסקה בעברית עם מספרים 123 ו-English בתוכה (בסוגריים), ועוד טקסט ארוך כדי לבדוק את גלישת השורות מימין לשמאל."),
            para("בְּרֵאשִׁית בָּרָא אֱלֹהִים אֵת הַשָּׁמַיִם וְאֵת הָאָרֶץ"),
            json!({ "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "中文 日本語 한국어" }] }),
            para("你好，世界！这是一个中文段落，用来测试在任意字符处换行。文字从左到右排列，标点符号不会出现在行首。Notaya 是一个本地笔记应用，支持中文、English 和数字 123。"),
            para("こんにちは世界。これは日本語の段落です。ひらがな、カタカナ、漢字が混ざっています。「かぎ括弧」の中も正しく表示されます。"),
            para("안녕하세요 세계! 이것은 한국어 단락입니다. 한글은 공백으로 단어를 나누어 줄을 바꿉니다. 숫자 123 과 English 도 섞여 있습니다."),
            json!({ "type": "table", "content": [{ "type": "tableRow", "content": [{ "type": "tableHeader", "content": [para("الاسم")] }, { "type": "tableHeader", "content": [para("Name")] }, { "type": "tableHeader", "content": [para("שם")] }] },
                { "type": "tableRow", "content": [{ "type": "tableCell", "content": [para("أحمد محمد")] }, { "type": "tableCell", "content": [para("Ahmed")] }, { "type": "tableCell", "content": [para("אחמד")] }] }] }),
            json!({ "type": "codeBlock", "content": [{ "type": "text", "text": "print(\"مرحبا\")  # code stays left to right" }] }),
        ];
        let doc = doc_of(blocks);
        let with = render_with(&doc, 1_793_190_896_000, Pool::new(fonts));
        std::fs::write(dir.join("unicode-samples.pdf"), &with.bytes).unwrap();
        std::fs::write(dir.join("unicode-samples.report.txt"), format!("chars_replaced={} chars_no_font={}\n", with.chars_replaced, with.chars_no_font)).unwrap();
        let without = render_with(&doc, 1_793_190_896_000, Pool::new(None));
        std::fs::write(dir.join("unicode-samples-no-cjk-font.pdf"), &without.bytes).unwrap();
        std::fs::write(dir.join("unicode-samples-no-cjk-font.report.txt"), format!("chars_replaced={} chars_no_font={}\n", without.chars_replaced, without.chars_no_font)).unwrap();
    }
}
