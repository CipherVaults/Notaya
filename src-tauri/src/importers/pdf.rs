//! Text from PDF files, with headings guessed from the font size.
//!
//! A small reader written for hostile input: it finds the objects by scanning the file (so a
//! broken cross-reference table does not matter), reads compressed object streams, follows
//! the page tree, and runs each page's content stream (and the forms it draws) only to
//! collect text and its position. Text is decoded with the font's ToUnicode map, its
//! encoding (WinAnsi, MacRoman, Standard, Differences) or Latin-1. Lines become paragraphs;
//! lines in a clearly larger font become headings. Inline Unicode replacement text
//! (ActualText) on a span supplies the logical text for that span.
//!
//! What it cannot do, and says so: encrypted PDFs, scanned or image-only PDFs (no text to
//! find; recognising text in pictures is not supported), and fonts without any character map
//! (the text cannot be decoded). Pictures, tables, columns and links in a PDF are not kept:
//! unmarked text comes out in the order the PDF draws it.
//!
//! Limits: decoded bytes per stream and in total, operators per file, pages, nesting.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::doc::{Block, Inline};
use super::Control;

const MAX_STREAM: usize = 64 * 1024 * 1024;
const MAX_DECODED_TOTAL: u64 = 512 * 1024 * 1024;
const MAX_OPS: u64 = 30_000_000;
const MAX_PAGES: usize = 3_000;
const MAX_DEPTH: usize = 32;
/// Longest string or hex string kept.
const MAX_STRING: usize = 1024 * 1024;
/// Longest ToUnicode destination (bytes) and the most bytes of destinations one map may hold.
const MAX_CMAP_DEST: usize = 64;
const MAX_CMAP_BYTES: usize = 8 * 1024 * 1024;
const MAX_CMAP_ENTRIES: usize = 100_000;
/// Most form XObjects one file may draw.
const MAX_FORMS: u32 = 20_000;
/// Most bytes of shown text and logical replacements one PDF may be asked to decode.
const MAX_TEXT_BYTES: usize = 32 * 1024 * 1024;
/// Most objects kept from one file, and most bytes of object text (not streams) the scanner
/// may read: a dictionary or array costs far more memory than the bytes it is written in.
const MAX_OBJECTS: usize = 1_000_000;
const MAX_OBJECT_TEXT: usize = 48 * 1024 * 1024;
/// Most total AST/value nodes lexed into memory across scan + compressed object streams.
const MAX_OBJECT_NODES: usize = 2_000_000;
const MAX_TRAILERS: usize = 256;
const MAX_FILTERS: usize = 8;
const MAX_FONT_CACHE: usize = 4_096;
const MAX_TOTAL_FONT_BYTES: usize = 32 * 1024 * 1024;

type Dict = Vec<(Vec<u8>, Obj)>;

#[derive(Clone, Debug)]
enum Obj {
    Null,
    #[allow(dead_code)]
    Bool(bool),
    Num(f64),
    Name(Vec<u8>),
    Str(Vec<u8>),
    Arr(Vec<Obj>),
    Dict(Dict),
    Ref(u32),
    /// A stream: its dictionary and the byte range of its data in the file.
    Stream(Dict, usize, usize),
    Op(Vec<u8>),
}

impl Obj {
    fn num(&self) -> Option<f64> {
        if let Obj::Num(n) = self { Some(*n) } else { None }
    }
    fn name(&self) -> Option<&[u8]> {
        if let Obj::Name(n) = self { Some(n) } else { None }
    }
    fn dict(&self) -> Option<&Dict> {
        match self {
            Obj::Dict(d) | Obj::Stream(d, _, _) => Some(d),
            _ => None,
        }
    }
}

fn lookup<'d>(d: &'d Dict, key: &[u8]) -> Option<&'d Obj> {
    d.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn is_ws(c: u8) -> bool {
    matches!(c, 0 | 9 | 10 | 12 | 13 | 32)
}

fn is_delim(c: u8) -> bool {
    matches!(c, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

struct Lex<'a> {
    b: &'a [u8],
    pos: usize,
    nodes: usize,
    max_nodes: usize,
}

impl<'a> Lex<'a> {
    fn new(b: &'a [u8], pos: usize) -> Self {
        Lex { b, pos: pos.min(b.len()), nodes: 0, max_nodes: usize::MAX }
    }

    fn bounded(b: &'a [u8], pos: usize, max_nodes: usize) -> Self {
        Lex { b, pos: pos.min(b.len()), nodes: 0, max_nodes }
    }

    fn skip_ws(&mut self) {
        loop {
            while self.pos < self.b.len() && is_ws(self.b[self.pos]) {
                self.pos += 1;
            }
            if self.pos < self.b.len() && self.b[self.pos] == b'%' {
                while self.pos < self.b.len() && self.b[self.pos] != b'\n' && self.b[self.pos] != b'\r' {
                    self.pos += 1;
                }
            } else {
                return;
            }
        }
    }

    fn regular(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self.pos < self.b.len() && !is_ws(self.b[self.pos]) && !is_delim(self.b[self.pos]) {
            self.pos += 1;
        }
        &self.b[start..self.pos]
    }

    fn number(word: &[u8]) -> Option<f64> {
        if word.is_empty() || !word.iter().all(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.')) || !word.iter().any(u8::is_ascii_digit) {
            return None;
        }
        let s = std::str::from_utf8(word).ok()?;
        // PDF allows "-.5" and "5." but not exponents; a stray second sign is tolerated.
        s.trim_start_matches('+').parse::<f64>().ok().or_else(|| s.replace("--", "-").parse().ok()).filter(|n: &f64| n.is_finite())
    }

    /// Reads one object (or an operator keyword). `None` at the end or when the node budget ends.
    fn obj(&mut self, depth: usize) -> Option<Obj> {
        if self.nodes >= self.max_nodes {
            return None;
        }
        self.nodes += 1;
        self.skip_ws();
        let b = self.b;
        let c = *b.get(self.pos)?;
        if depth > MAX_DEPTH {
            self.pos += 1;
            return Some(Obj::Null);
        }
        match c {
            b'/' => {
                self.pos += 1;
                let raw = self.regular();
                let mut name = Vec::with_capacity(raw.len());
                let mut i = 0;
                while i < raw.len() {
                    if raw[i] == b'#' && i + 2 < raw.len() + 0 && i + 2 <= raw.len() - 1 {
                        if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&raw[i + 1..i + 3]).unwrap_or("zz"), 16) {
                            name.push(v);
                            i += 3;
                            continue;
                        }
                    }
                    name.push(raw[i]);
                    i += 1;
                }
                Some(Obj::Name(name))
            }
            b'(' => Some(Obj::Str(self.literal())),
            b'<' if b.get(self.pos + 1) == Some(&b'<') => {
                self.pos += 2;
                let mut d = Vec::new();
                loop {
                    self.skip_ws();
                    if self.pos >= b.len() {
                        break;
                    }
                    if b.get(self.pos..).is_some_and(|r| r.starts_with(b">>")) {
                        self.pos += 2;
                        break;
                    }
                    match self.obj(depth + 1) {
                        Some(Obj::Name(k)) => {
                            let v = self.obj(depth + 1).unwrap_or(Obj::Null);
                            if d.len() < 4096 {
                                d.push((k, v));
                            }
                        }
                        Some(_) => {}
                        None => break,
                    }
                }
                Some(Obj::Dict(d))
            }
            b'<' => {
                self.pos += 1;
                let mut out = Vec::new();
                let mut half: Option<u8> = None;
                while self.pos < b.len() && b[self.pos] != b'>' && out.len() < MAX_STRING {
                    if let Some(v) = (b[self.pos] as char).to_digit(16) {
                        match half.take() {
                            Some(h) => out.push(h << 4 | v as u8),
                            None => half = Some(v as u8),
                        }
                    }
                    self.pos += 1;
                }
                if let Some(h) = half {
                    out.push(h << 4);
                }
                // The closing `>`, if there is one (an unclosed string ends at the end of the data).
                self.pos = (self.pos + 1).min(b.len());
                Some(Obj::Str(out))
            }
            b'[' => {
                self.pos += 1;
                let mut a = Vec::new();
                loop {
                    self.skip_ws();
                    if self.pos >= b.len() {
                        break;
                    }
                    if b[self.pos] == b']' {
                        self.pos += 1;
                        break;
                    }
                    match self.obj(depth + 1) {
                        Some(o) => {
                            if a.len() < 200_000 {
                                a.push(o);
                            }
                        }
                        None => break,
                    }
                }
                Some(Obj::Arr(a))
            }
            b')' | b'>' | b']' | b'{' | b'}' => {
                self.pos += 1;
                Some(Obj::Op(vec![c]))
            }
            _ => {
                let word = self.regular();
                if word.is_empty() {
                    self.pos += 1;
                    return Some(Obj::Op(vec![c]));
                }
                if let Some(n) = Self::number(word) {
                    // "12 0 R" is a reference.
                    if n >= 0.0 && n.fract() == 0.0 && word.iter().all(u8::is_ascii_digit) {
                        let save = self.pos;
                        self.skip_ws();
                        let g = self.regular();
                        if !g.is_empty() && g.iter().all(u8::is_ascii_digit) {
                            self.skip_ws();
                            if b.get(self.pos) == Some(&b'R') && b.get(self.pos + 1).is_none_or(|x| is_ws(*x) || is_delim(*x)) {
                                self.pos += 1;
                                return Some(Obj::Ref(n.min(u32::MAX as f64) as u32));
                            }
                        }
                        self.pos = save;
                    }
                    return Some(Obj::Num(n));
                }
                Some(match word {
                    b"true" => Obj::Bool(true),
                    b"false" => Obj::Bool(false),
                    b"null" => Obj::Null,
                    w => Obj::Op(w.to_vec()),
                })
            }
        }
    }

    fn literal(&mut self) -> Vec<u8> {
        let b = self.b;
        self.pos += 1;
        let mut out = Vec::new();
        let mut depth = 1;
        while self.pos < b.len() {
            let c = b[self.pos];
            self.pos += 1;
            match c {
                b'\\' => {
                    let Some(&e) = b.get(self.pos) else { break };
                    self.pos += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'\r' => {
                            if b.get(self.pos) == Some(&b'\n') {
                                self.pos += 1;
                            }
                        }
                        b'\n' => {}
                        b'0'..=b'7' => {
                            let mut v = u32::from(e - b'0');
                            for _ in 0..2 {
                                match b.get(self.pos) {
                                    Some(d @ b'0'..=b'7') => {
                                        v = v * 8 + u32::from(d - b'0');
                                        self.pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            out.push(v as u8);
                        }
                        other => out.push(other),
                    }
                }
                b'(' => {
                    depth += 1;
                    out.push(c);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(c);
                }
                _ => out.push(c),
            }
            if out.len() > MAX_STRING {
                break;
            }
        }
        out
    }
}

fn find(hay: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= hay.len() {
        return None;
    }
    hay[from..].windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

pub struct PdfDoc {
    pub blocks: Vec<Block>,
    pub pages: usize,
    /// Pages where no text was found.
    pub empty_pages: usize,
}

struct Pdf<'a> {
    data: &'a [u8],
    objs: HashMap<u32, Obj>,
    /// Decoded bytes this PDF may still use, and the whole import's (shared) allowance.
    budget: Cell<u64>,
    shared: &'a Cell<u64>,
    ctl: &'a Control,
    forms: Cell<u32>,
    text: Cell<usize>,
    ops: Cell<u64>,
    fonts: std::cell::RefCell<HashMap<u32, Rc<Font>>>,
    font_bytes: Cell<usize>,
}

impl<'a> Pdf<'a> {
    fn get<'s>(&'s self, o: &'s Obj) -> &'s Obj {
        let mut cur = o;
        for _ in 0..16 {
            match cur {
                Obj::Ref(n) => match self.objs.get(n) {
                    Some(x) => cur = x,
                    None => return &Obj::Null,
                },
                _ => return cur,
            }
        }
        &Obj::Null
    }

    fn key<'s>(&'s self, d: &'s Dict, k: &[u8]) -> &'s Obj {
        lookup(d, k).map_or(&Obj::Null, |v| self.get(v))
    }

    fn stream(&self, o: &Obj) -> Option<Vec<u8>> {
        if self.ctl.stopped() {
            return None;
        }
        let Obj::Stream(d, start, end) = self.get(o) else { return None };
        let raw = self.data.get(*start..*end)?;
        if raw.len() > MAX_STREAM {
            return None;
        }
        let filters: Vec<Vec<u8>> = match self.key(d, b"Filter") {
            Obj::Null => Vec::new(),
            Obj::Name(n) => vec![n.clone()],
            Obj::Arr(a) => {
                if a.len() > MAX_FILTERS {
                    return None;
                }
                let mut out = Vec::with_capacity(a.len());
                for x in a {
                    out.push(self.get(x).name()?.to_vec());
                }
                out
            }
            _ => return None,
        };
        if filters.is_empty() {
            if (raw.len() as u64) > self.budget.get().min(self.shared.get()) {
                return None;
            }
            self.budget.set(self.budget.get().saturating_sub(raw.len() as u64));
            self.shared.set(self.shared.get().saturating_sub(raw.len() as u64));
            return Some(raw.to_vec());
        }
        let mut buf = raw.to_vec();
        for f in filters {
            if self.ctl.stopped() {
                return None;
            }
            let limit = MAX_STREAM.min(self.budget.get() as usize).min(self.shared.get() as usize);
            if limit == 0 {
                return None;
            }
            let next = match f.as_slice() {
                b"FlateDecode" | b"Fl" => inflate(&buf, limit),
                b"ASCIIHexDecode" | b"AHx" => Some(ascii_hex(&buf, limit)),
                b"ASCII85Decode" | b"A85" => Some(ascii85(&buf, limit)),
                b"LZWDecode" | b"LZW" => Some(lzw(&buf, limit)),
                b"RunLengthDecode" | b"RL" => Some(run_length(&buf, limit)),
                _ => return None,
            };
            let Some(next) = next else {
                // Debit attempted limit so repeated failing filter-chain probes cannot bypass budget.
                self.budget.set(self.budget.get().saturating_sub(limit as u64));
                self.shared.set(self.shared.get().saturating_sub(limit as u64));
                return None;
            };
            self.budget.set(self.budget.get().saturating_sub(next.len() as u64));
            self.shared.set(self.shared.get().saturating_sub(next.len() as u64));
            if next.len() >= limit && limit < MAX_STREAM {
                return None;
            }
            buf = next;
        }
        Some(buf)
    }
}

fn inflate(data: &[u8], limit: usize) -> Option<Vec<u8>> {
    use crate::export::deflate::{inflate_raw, zlib_decompress};
    // A zlib stream; if its checksum or header is damaged, try the bare data after the 2-byte header,
    // then the data as it is.
    zlib_decompress(data, limit)
        .ok()
        .or_else(|| inflate_raw(data.get(2..)?, limit).ok())
        .or_else(|| inflate_raw(data, limit).ok())
}

fn ascii_hex(data: &[u8], limit: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut half: Option<u8> = None;
    for &c in data {
        if c == b'>' || out.len() >= limit {
            break;
        }
        if let Some(v) = (c as char).to_digit(16) {
            match half.take() {
                Some(h) => out.push(h << 4 | v as u8),
                None => half = Some(v as u8),
            }
        }
    }
    if let Some(h) = half {
        out.push(h << 4);
    }
    out
}

fn ascii85(data: &[u8], limit: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut group = [0u32; 5];
    let mut n = 0;
    let data = data.strip_prefix(b"<~").unwrap_or(data);
    for &c in data {
        if out.len() >= limit {
            break;
        }
        match c {
            b'~' => break,
            b'z' if n == 0 => out.extend_from_slice(&[0; 4]),
            b'!'..=b'u' => {
                group[n] = u32::from(c - b'!');
                n += 1;
                if n == 5 {
                    let v = group.iter().fold(0u64, |a, d| a * 85 + u64::from(*d)) as u32;
                    out.extend_from_slice(&v.to_be_bytes());
                    n = 0;
                }
            }
            _ => {}
        }
    }
    if n > 1 {
        for g in group.iter_mut().skip(n) {
            *g = 84;
        }
        let v = group.iter().fold(0u64, |a, d| a * 85 + u64::from(*d)) as u32;
        out.extend_from_slice(&v.to_be_bytes()[..n - 1]);
    }
    out
}

fn run_length(data: &[u8], limit: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() && out.len() < limit {
        let len = data[i];
        i += 1;
        match len {
            128 => break,
            0..=127 => {
                let n = usize::from(len) + 1;
                out.extend_from_slice(&data[i..(i + n).min(data.len())]);
                i += n;
            }
            _ => {
                if let Some(&b) = data.get(i) {
                    out.extend(std::iter::repeat_n(b, 257 - usize::from(len)));
                }
                i += 1;
            }
        }
    }
    out
}

fn lzw(data: &[u8], limit: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut table: Vec<Vec<u8>> = (0..256u16).map(|i| vec![i as u8]).collect();
    table.push(Vec::new());
    table.push(Vec::new());
    let mut width = 9;
    let mut bits: u32 = 0;
    let mut nbits = 0;
    let mut prev: Option<Vec<u8>> = None;
    for &byte in data {
        bits = (bits << 8) | u32::from(byte);
        nbits += 8;
        while nbits >= width {
            let code = ((bits >> (nbits - width)) & ((1 << width) - 1)) as usize;
            nbits -= width;
            if code == 256 {
                table.truncate(258);
                width = 9;
                prev = None;
                continue;
            }
            if code == 257 {
                return out;
            }
            let entry = if code < table.len() {
                table[code].clone()
            } else if let Some(p) = &prev {
                let mut e = p.clone();
                e.push(p[0]);
                e
            } else {
                return out;
            };
            out.extend_from_slice(&entry);
            if out.len() > limit {
                return out;
            }
            if let Some(p) = prev.take() {
                if table.len() < 4096 {
                    let mut e = p;
                    e.push(entry[0]);
                    table.push(e);
                }
            }
            prev = Some(entry);
            if table.len() + 1 >= (1 << width) && width < 12 {
                width += 1;
            }
        }
    }
    out
}

// ---------------- fonts ----------------

struct Font {
    /// From the ToUnicode map.
    map: HashMap<u32, String>,
    /// Code lengths in bytes allowed by the map's code space.
    lens: Vec<usize>,
    /// Composite font: two-byte codes unless the map says otherwise.
    composite: bool,
    /// Single-byte encoding (simple fonts).
    enc: [Option<char>; 256],
    /// Glyph widths (thousandths of the font size) by code.
    widths: HashMap<u32, f64>,
    default_width: f64,
}

impl Font {
    fn empty() -> Self {
        Font { map: HashMap::new(), lens: Vec::new(), composite: false, enc: [None; 256], widths: HashMap::new(), default_width: 500.0 }
    }

    fn approx_bytes(&self) -> usize {
        2048usize
            .saturating_add(self.map.len().saturating_mul(64))
            .saturating_add(self.map.values().map(String::len).sum())
            .saturating_add(self.widths.len().saturating_mul(24))
    }

    /// Splits the bytes into codes and decodes them: (code, text, unmapped?).
    fn decode(&self, bytes: &[u8]) -> Vec<(u32, String, bool)> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            let mut len = if self.composite { 2 } else { 1 };
            if !self.lens.is_empty() {
                if self.lens.len() == 1 {
                    len = self.lens[0];
                } else {
                    // Longest code that the map knows, else the shortest allowed.
                    len = *self.lens.iter().min().unwrap_or(&1);
                    for &l in self.lens.iter().rev() {
                        if i + l <= bytes.len() && self.map.contains_key(&be(&bytes[i..i + l])) {
                            len = l;
                            break;
                        }
                    }
                }
            }
            let len = len.clamp(1, 4).min(bytes.len() - i);
            let code = be(&bytes[i..i + len]);
            i += len;
            if let Some(s) = self.map.get(&code) {
                out.push((code, s.clone(), false));
            } else if !self.composite && code < 256 {
                match self.enc[code as usize] {
                    Some(c) => out.push((code, c.to_string(), false)),
                    None => out.push((code, String::new(), code >= 32)),
                }
            } else {
                out.push((code, "\u{FFFD}".into(), true));
            }
        }
        out
    }

    fn width(&self, code: u32) -> f64 {
        self.widths.get(&code).copied().unwrap_or(self.default_width)
    }
}

fn be(b: &[u8]) -> u32 {
    b.iter().fold(0u32, |a, x| (a << 8) | u32::from(*x))
}

fn utf16be(b: &[u8]) -> String {
    let units: Vec<u16> = b.chunks(2).map(|c| if c.len() == 2 { u16::from_be_bytes([c[0], c[1]]) } else { u16::from(c[0]) }).collect();
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
fn parse_cmap(data: &[u8]) -> (HashMap<u32, String>, Vec<usize>) {
    parse_cmap_with(data, &Control::standard())
}

fn parse_cmap_with(data: &[u8], ctl: &Control) -> (HashMap<u32, String>, Vec<usize>) {
    let mut map = HashMap::new();
    let mut lens: Vec<usize> = Vec::new();
    let mut lex = Lex::bounded(data, 0, MAX_OBJECT_NODES);
    let mut mode = 0u8; // 1 codespace, 2 bfchar, 3 bfrange
    let mut pending: Vec<Obj> = Vec::new();
    let mut guard = 0;
    let mut bytes = 0usize;
    while let Some(o) = lex.obj(0) {
        guard += 1;
        if guard > 2_000_000 || map.len() > MAX_CMAP_ENTRIES || bytes >= MAX_CMAP_BYTES || ctl.stopped() {
            break;
        }
        match &o {
            Obj::Op(op) => {
                mode = match op.as_slice() {
                    b"begincodespacerange" => 1,
                    b"beginbfchar" => 2,
                    b"beginbfrange" => 3,
                    _ => 0,
                };
                pending.clear();
                continue;
            }
            _ => pending.push(o),
        }
        match mode {
            1 if pending.len() == 2 => {
                if let Obj::Str(lo) = &pending[0] {
                    if (1..=4).contains(&lo.len()) && !lens.contains(&lo.len()) {
                        lens.push(lo.len());
                    }
                }
                pending.clear();
            }
            2 if pending.len() == 2 => {
                if let Obj::Str(src) = &pending[0] {
                    let dst = match &pending[1] {
                        Obj::Str(d) if d.len() <= MAX_CMAP_DEST => Some(utf16be(d)),
                        Obj::Name(n) => glyph_char(&String::from_utf8_lossy(n)).map(|c| c.to_string()),
                        _ => None,
                    };
                    if let (Some(d), true) = (dst, src.len() <= 4) {
                        bytes += d.len();
                        map.insert(be(src), d);
                    }
                }
                pending.clear();
            }
            3 if pending.len() == 3 => {
                if let (Obj::Str(lo), Obj::Str(hi)) = (&pending[0], &pending[1]) {
                    if lo.len() <= 4 && hi.len() <= 4 {
                        let (lo, hi) = (be(lo), be(hi));
                        if hi >= lo && hi - lo <= 65_535 {
                            match &pending[2] {
                                Obj::Str(d) if d.len() >= 2 && d.len() <= MAX_CMAP_DEST => {
                                    let mut units: Vec<u16> = d.chunks(2).map(|c| u16::from_be_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect();
                                    let last = units.len() - 1;
                                    let base = units[last];
                                    for (k, code) in (lo..=hi).enumerate() {
                                        // The caps are checked inside the loop: one range can name 65,536 codes.
                                        if map.len() >= MAX_CMAP_ENTRIES || bytes >= MAX_CMAP_BYTES {
                                            break;
                                        }
                                        units[last] = base.wrapping_add(k as u16);
                                        let text = String::from_utf16_lossy(&units);
                                        bytes += text.len();
                                        map.insert(code, text);
                                    }
                                }
                                Obj::Arr(a) => {
                                    for (k, d) in a.iter().enumerate() {
                                        if map.len() >= MAX_CMAP_ENTRIES || bytes >= MAX_CMAP_BYTES || (k % 1024 == 0 && ctl.stopped()) {
                                            break;
                                        }
                                        if let Obj::Str(d) = d {
                                            if d.len() <= MAX_CMAP_DEST {
                                                let text = utf16be(d);
                                                bytes += text.len();
                                                map.insert(lo.saturating_add(k as u32), text);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                pending.clear();
            }
            0 => pending.clear(),
            _ => {}
        }
    }
    lens.sort_unstable();
    (map, lens)
}

/// Windows-1252 (the WinAnsi encoding of PDF fonts).
pub fn win_ansi(b: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž', '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™',
        'š', '›', 'œ', '\u{9d}', 'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&b) { HIGH[usize::from(b - 0x80)] } else { char::from(b) }
}

const MAC_ROMAN_HIGH: &str = "ÄÅÇÉÑÖÜáàâäãåçéèêëíìîïñóòôöõúùûü†°¢£§•¶ß®©™´¨≠ÆØ∞±≤≥¥µ∂∑∏π∫ªºΩæø¿¡¬√ƒ≈∆«»…\u{a0}ÀÃÕŒœ–—“”‘’÷◊ÿŸ⁄€‹›ﬁﬂ‡·‚„‰ÂÊÁËÈÍÎÏÌÓÔ\u{f8ff}ÒÚÛÙıˆ˜¯˘˙˚¸˝˛ˇ";

/// Glyph names (Adobe's list, the common part) to characters.
fn glyph_char(name: &str) -> Option<char> {
    let name = name.split('.').next().unwrap_or(name);
    let name = name.split('_').next().unwrap_or(name);
    if name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() {
        return name.chars().next();
    }
    if let Some(hex) = name.strip_prefix("uni") {
        return u32::from_str_radix(hex.get(..4)?, 16).ok().and_then(char::from_u32);
    }
    if let Some(hex) = name.strip_prefix('u') {
        if (4..=6).contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
        }
    }
    const ASCII: [&str; 95] = [
        "space", "exclam", "quotedbl", "numbersign", "dollar", "percent", "ampersand", "quotesingle", "parenleft", "parenright", "asterisk", "plus", "comma",
        "hyphen", "period", "slash", "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "colon", "semicolon", "less", "equal",
        "greater", "question", "at", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
        "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", "underscore", "grave", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k",
        "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "braceleft", "bar", "braceright", "asciitilde",
    ];
    if let Some(i) = ASCII.iter().position(|n| *n == name) {
        return char::from_u32(0x20 + i as u32);
    }
    const LATIN1: [&str; 96] = [
        "nbspace", "exclamdown", "cent", "sterling", "currency", "yen", "brokenbar", "section", "dieresis", "copyright", "ordfeminine", "guillemotleft",
        "logicalnot", "sfthyphen", "registered", "macron", "degree", "plusminus", "twosuperior", "threesuperior", "acute", "mu", "paragraph",
        "periodcentered", "cedilla", "onesuperior", "ordmasculine", "guillemotright", "onequarter", "onehalf", "threequarters", "questiondown", "Agrave",
        "Aacute", "Acircumflex", "Atilde", "Adieresis", "Aring", "AE", "Ccedilla", "Egrave", "Eacute", "Ecircumflex", "Edieresis", "Igrave", "Iacute",
        "Icircumflex", "Idieresis", "Eth", "Ntilde", "Ograve", "Oacute", "Ocircumflex", "Otilde", "Odieresis", "multiply", "Oslash", "Ugrave", "Uacute",
        "Ucircumflex", "Udieresis", "Yacute", "Thorn", "germandbls", "agrave", "aacute", "acircumflex", "atilde", "adieresis", "aring", "ae", "ccedilla",
        "egrave", "eacute", "ecircumflex", "edieresis", "igrave", "iacute", "icircumflex", "idieresis", "eth", "ntilde", "ograve", "oacute", "ocircumflex",
        "otilde", "odieresis", "divide", "oslash", "ugrave", "uacute", "ucircumflex", "udieresis", "yacute", "thorn", "ydieresis",
    ];
    if let Some(i) = LATIN1.iter().position(|n| *n == name) {
        return char::from_u32(0xA0 + i as u32);
    }
    Some(match name {
        "Euro" => '€',
        "quotesinglbase" => '‚',
        "florin" => 'ƒ',
        "quotedblbase" => '„',
        "ellipsis" => '…',
        "dagger" => '†',
        "daggerdbl" => '‡',
        "circumflex" => 'ˆ',
        "perthousand" => '‰',
        "Scaron" => 'Š',
        "guilsinglleft" => '‹',
        "OE" => 'Œ',
        "Zcaron" => 'Ž',
        "quoteleft" => '‘',
        "quoteright" => '’',
        "quotedblleft" => '“',
        "quotedblright" => '”',
        "bullet" => '•',
        "endash" => '–',
        "emdash" => '—',
        "tilde" => '˜',
        "trademark" => '™',
        "scaron" => 'š',
        "guilsinglright" => '›',
        "oe" => 'œ',
        "zcaron" => 'ž',
        "Ydieresis" => 'Ÿ',
        "fi" => 'ﬁ',
        "fl" => 'ﬂ',
        "ff" => 'ﬀ',
        "ffi" => 'ﬃ',
        "ffl" => 'ﬄ',
        "minus" => '−',
        "fraction" => '⁄',
        "dotlessi" => 'ı',
        "Lslash" => 'Ł',
        "lslash" => 'ł',
        "hyphensoft" => '\u{ad}',
        "nonbreakingspace" => '\u{a0}',
        _ => return None,
    })
}

impl Pdf<'_> {
    fn font(&self, o: &Obj) -> Rc<Font> {
        if let Obj::Ref(n) = o {
            if let Some(f) = self.fonts.borrow().get(n) {
                return f.clone();
            }
        }
        if self.fonts.borrow().len() >= MAX_FONT_CACHE || self.font_bytes.get() >= MAX_TOTAL_FONT_BYTES {
            return Rc::new(Font::empty());
        }
        let built = self.build_font(self.get(o));
        let cost = built.approx_bytes();
        self.font_bytes.set(self.font_bytes.get().saturating_add(cost));
        let f = Rc::new(built);
        if let Obj::Ref(n) = o {
            if self.fonts.borrow().len() < MAX_FONT_CACHE && self.font_bytes.get() <= MAX_TOTAL_FONT_BYTES {
                self.fonts.borrow_mut().insert(*n, f.clone());
            }
        }
        f
    }

    fn build_font(&self, o: &Obj) -> Font {
        let empty = Vec::new();
        let d = o.dict().unwrap_or(&empty);
        let composite = self.key(d, b"Subtype").name() == Some(b"Type0");
        let (map, mut lens) = self.stream(lookup(d, b"ToUnicode").unwrap_or(&Obj::Null)).map(|s| parse_cmap_with(&s, self.ctl)).unwrap_or_default();
        if composite && lens.is_empty() {
            if let Some(s) = self.stream(lookup(d, b"Encoding").unwrap_or(&Obj::Null)) {
                lens = parse_cmap_with(&s, self.ctl).1;
            }
        }
        // Simple fonts: base encoding, then Differences.
        let mut enc: [Option<char>; 256] = [None; 256];
        let base = |name: &[u8], enc: &mut [Option<char>; 256]| {
            for (i, slot) in enc.iter_mut().enumerate() {
                let b = i as u8;
                *slot = match name {
                    b"MacRomanEncoding" if b >= 0x80 => MAC_ROMAN_HIGH.chars().nth(i - 0x80),
                    b"StandardEncoding" if b == 0x27 => Some('’'),
                    b"StandardEncoding" if b == 0x60 => Some('‘'),
                    b"StandardEncoding" if b >= 0x80 => None,
                    _ if b < 0x20 => None,
                    _ => Some(win_ansi(b)),
                };
            }
        };
        base(b"WinAnsiEncoding", &mut enc);
        match self.key(d, b"Encoding") {
            Obj::Name(n) => base(n, &mut enc),
            Obj::Dict(ed) => {
                if let Some(n) = self.key(ed, b"BaseEncoding").name() {
                    base(n, &mut enc);
                }
                if let Obj::Arr(diffs) = self.key(ed, b"Differences") {
                    let mut code = 0usize;
                    for x in diffs {
                        match self.get(x) {
                            Obj::Num(n) => code = (*n).clamp(0.0, 255.0) as usize,
                            Obj::Name(g) => {
                                if code < 256 {
                                    enc[code] = glyph_char(&String::from_utf8_lossy(g));
                                }
                                code += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        // Widths.
        let mut widths = HashMap::new();
        let mut default_width = 500.0;
        if composite {
            default_width = 1000.0;
            if let Obj::Arr(desc) = self.key(d, b"DescendantFonts") {
                if let Some(dd) = desc.first().map(|x| self.get(x)).and_then(Obj::dict) {
                    if let Some(dw) = self.key(dd, b"DW").num() {
                        default_width = dw;
                    }
                    if let Obj::Arr(w) = self.key(dd, b"W") {
                        let mut i = 0;
                        while i < w.len() && widths.len() < 100_000 {
                            let first = self.get(&w[i]).num().unwrap_or(0.0).max(0.0) as u32;
                            match w.get(i + 1).map(|x| self.get(x)) {
                                Some(Obj::Arr(list)) => {
                                    for (k, v) in list.iter().enumerate() {
                                        if let Some(v) = self.get(v).num() {
                                            widths.insert(first.saturating_add(k as u32), v);
                                        }
                                    }
                                    i += 2;
                                }
                                Some(Obj::Num(last)) => {
                                    let last = (*last).max(0.0) as u32;
                                    let v = w.get(i + 2).and_then(|x| self.get(x).num()).unwrap_or(default_width);
                                    for c in first..=last.min(first.saturating_add(65_535)) {
                                        widths.insert(c, v);
                                    }
                                    i += 3;
                                }
                                _ => break,
                            }
                        }
                    }
                }
            }
        } else if let Obj::Arr(w) = self.key(d, b"Widths") {
            let first = self.key(d, b"FirstChar").num().unwrap_or(0.0).max(0.0) as u32;
            for (k, v) in w.iter().enumerate().take(256) {
                if let Some(v) = self.get(v).num() {
                    widths.insert(first.saturating_add(k as u32), v);
                }
            }
            if let Some(m) = self.key(d, b"MissingWidth").num() {
                default_width = m;
            }
        }
        Font { map, lens, composite, enc, widths, default_width }
    }
}

// ---------------- page content ----------------

type M = [f64; 6];
const ID: M = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn mul(a: &M, b: &M) -> M {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

struct Line {
    text: String,
    y: f64,
    size: f64,
}

#[derive(Clone, Copy)]
struct TextBounds {
    x: f64,
    y: f64,
    size: f64,
    end: f64,
}

struct Drawing {
    text: String,
    bounds: TextBounds,
    chars: usize,
    unmapped: usize,
}

struct Replacement {
    text: String,
    drawings: Vec<Drawing>,
    bytes: usize,
}

fn replacement_text(args: &[Obj]) -> Option<String> {
    let [Obj::Name(tag), Obj::Dict(properties)] = args else {
        return None;
    };
    if tag.as_slice() != b"Span" {
        return None;
    }
    let Some(Obj::Str(bytes)) = lookup(properties, b"ActualText") else {
        return None;
    };
    let text = bytes.strip_prefix(&[0xfe, 0xff])?;
    if text.is_empty() || text.len() % 2 != 0 {
        return None;
    }
    let units = text.chunks_exact(2).map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
    let mut decoded = String::new();
    for character in char::decode_utf16(units) {
        decoded.extend(imported_character(character.ok()?));
    }
    (!decoded.is_empty()).then_some(decoded)
}

fn imported_character(character: char) -> Option<char> {
    match character {
        '\t' | '\u{a0}' => Some(' '),
        c if c.is_control() => None,
        c => Some(c),
    }
}

#[derive(Default)]
struct Collector {
    lines: Vec<Line>,
    cur: String,
    y: f64,
    size: f64,
    x_end: f64,
    open: bool,
    chars: usize,
    unmapped: usize,
    replacement: Option<Replacement>,
    last_was_replacement: bool,
}

impl Collector {
    fn draw(&mut self, drawing: Drawing) {
        if let Some(replacement) = &mut self.replacement {
            // Keep a bounded glyph fallback until the span closes. Large or unfinished
            // spans still have the normal glyph decoding available.
            replacement.bytes += std::mem::size_of::<Drawing>() + drawing.text.capacity();
            if replacement.bytes <= MAX_STRING {
                replacement.drawings.push(drawing);
                return;
            }
            self.finish_replacement(false);
        }
        self.add_drawing(drawing);
    }

    fn add_drawing(&mut self, drawing: Drawing) {
        self.chars += drawing.chars;
        self.unmapped += drawing.unmapped;
        self.add(&drawing.text, drawing.bounds, false);
    }

    fn finish_replacement(&mut self, closed: bool) {
        let Some(replacement) = self.replacement.take() else {
            return;
        };
        if closed {
            let mut drawn = replacement.drawings.iter().filter(|drawing| drawing.chars != 0);
            if let Some(first) = drawn.next() {
                let mut bounds = first.bounds;
                for drawing in drawn {
                    bounds.x = bounds.x.min(drawing.bounds.x);
                    bounds.end = bounds.end.max(drawing.bounds.end);
                    bounds.size = bounds.size.max(drawing.bounds.size);
                }
                self.chars += replacement.text.chars().count();
                self.add(&replacement.text, bounds, true);
                return;
            }
        }
        for drawing in replacement.drawings {
            self.add_drawing(drawing);
        }
    }
    fn end_line(&mut self) {
        if self.open {
            let t = self.cur.split_whitespace().collect::<Vec<_>>().join(" ");
            if !t.is_empty() && self.lines.len() < 200_000 {
                self.lines.push(Line { text: t, y: self.y, size: self.size });
            }
            self.cur.clear();
            self.open = false;
        }
    }

    fn add(&mut self, text: &str, bounds: TextBounds, replacement: bool) {
        let TextBounds { x, y, size, end } = bounds;
        if text.is_empty() {
            self.x_end = end;
            return;
        }
        if self.open {
            let tol = 0.5 * size.max(self.size).max(0.1);
            if (y - self.y).abs() > tol {
                self.end_line();
            // Right-to-left replacements can jump left within one logical word.
            } else if (x > self.x_end + 0.15 * size
                || (!replacement && !self.last_was_replacement && x < self.x_end - 3.0 * size))
                && !self.cur.ends_with(' ')
                && !text.starts_with(' ')
            {
                self.cur.push(' ');
            }
        }
        if !self.open {
            self.open = true;
            self.y = y;
            self.size = size;
        }
        self.size = self.size.max(size);
        self.cur.push_str(text);
        self.x_end = end;
        self.last_was_replacement = replacement;
    }
}

struct TextState {
    font: Option<Rc<Font>>,
    size: f64,
    tm: M,
    tlm: M,
    leading: f64,
    char_spacing: f64,
    word_spacing: f64,
    scale: f64,
}

impl Pdf<'_> {
    fn resources_fonts(&self, res: &Obj) -> HashMap<Vec<u8>, Obj> {
        let mut out = HashMap::new();
        if let Some(r) = self.get(res).dict() {
            if let Some(f) = self.key(r, b"Font").dict() {
                for (k, v) in f {
                    out.insert(k.clone(), v.clone());
                }
            }
        }
        out
    }

    fn run(&self, content: &[u8], res: &Obj, ctm0: M, col: &mut Collector, depth: usize, seen: &mut HashSet<u32>) {
        let fonts = self.resources_fonts(res);
        let mut font_cache: HashMap<Vec<u8>, Rc<Font>> = HashMap::new();
        let mut ctm = ctm0;
        let mut stack: Vec<M> = Vec::new();
        let mut ts = TextState { font: None, size: 12.0, tm: ID, tlm: ID, leading: 0.0, char_spacing: 0.0, word_spacing: 0.0, scale: 1.0 };
        let mut lex = Lex::new(content, 0);
        let mut args: Vec<Obj> = Vec::new();
        let mut tokens = 0u32;
        // Track local nesting, while an outer replacement can cover text drawn by a form.
        let mut marked_depth = 0;
        let mut replacement_depth = None;
        while let Some(o) = lex.obj(0) {
            tokens = tokens.wrapping_add(1);
            if tokens % 65_536 == 0 && self.ctl.stopped() {
                self.ops.set(MAX_OPS + 1);
                break;
            }
            let Obj::Op(op) = o else {
                if args.len() < 64 {
                    args.push(o);
                }
                continue;
            };
            let n = self.ops.get() + 1;
            self.ops.set(n);
            if n > MAX_OPS {
                break;
            }
            if n % 1024 == 0 && self.ctl.stopped() {
                self.ops.set(MAX_OPS + 1);
                break;
            }
            let num = |i: usize| args.get(i).and_then(Obj::num).unwrap_or(0.0);
            match op.as_slice() {
                b"q" => {
                    if stack.len() < 64 {
                        stack.push(ctm);
                    }
                }
                b"Q" => ctm = stack.pop().unwrap_or(ctm0),
                b"cm" if args.len() >= 6 => ctm = mul(&[num(0), num(1), num(2), num(3), num(4), num(5)], &ctm),
                b"BT" => {
                    ts.tm = ID;
                    ts.tlm = ID;
                }
                b"ET" => {}
                b"BMC" | b"BDC" => {
                    marked_depth += 1;
                    if op.as_slice() == b"BDC" && replacement_depth.is_none() && col.replacement.is_none() {
                        if let Some(text) = replacement_text(&args) {
                            if !self.charge_text(text.len()) {
                                break;
                            }
                            col.replacement = Some(Replacement { text, drawings: Vec::new(), bytes: 0 });
                            replacement_depth = Some(marked_depth);
                        }
                    }
                }
                b"EMC" if marked_depth > 0 => {
                    if replacement_depth == Some(marked_depth) {
                        col.finish_replacement(true);
                        replacement_depth = None;
                    }
                    marked_depth -= 1;
                }
                b"Tf" => {
                    if let Some(Obj::Name(name)) = args.first() {
                        // A font written inline in the resources is built once per content stream.
                        ts.font = match font_cache.get(name) {
                            Some(f) => Some(f.clone()),
                            None => fonts.get(name).map(|f| {
                                let f = self.font(f);
                                if font_cache.len() < 256 {
                                    font_cache.insert(name.clone(), f.clone());
                                }
                                f
                            }),
                        };
                    }
                    ts.size = num(1);
                }
                b"Td" => {
                    ts.tlm = mul(&[1.0, 0.0, 0.0, 1.0, num(0), num(1)], &ts.tlm);
                    ts.tm = ts.tlm;
                }
                b"TD" => {
                    ts.leading = -num(1);
                    ts.tlm = mul(&[1.0, 0.0, 0.0, 1.0, num(0), num(1)], &ts.tlm);
                    ts.tm = ts.tlm;
                }
                b"Tm" if args.len() >= 6 => {
                    ts.tlm = [num(0), num(1), num(2), num(3), num(4), num(5)];
                    ts.tm = ts.tlm;
                }
                b"T*" => {
                    ts.tlm = mul(&[1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading], &ts.tlm);
                    ts.tm = ts.tlm;
                }
                b"TL" => ts.leading = num(0),
                b"Tc" => ts.char_spacing = num(0),
                b"Tw" => ts.word_spacing = num(0),
                b"Tz" => ts.scale = num(0) / 100.0,
                b"Tj" => {
                    if let Some(Obj::Str(s)) = args.first() {
                        self.show(s, &mut ts, &ctm, col);
                    }
                }
                b"'" => {
                    ts.tlm = mul(&[1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading], &ts.tlm);
                    ts.tm = ts.tlm;
                    if let Some(Obj::Str(s)) = args.first() {
                        self.show(s, &mut ts, &ctm, col);
                    }
                }
                b"\"" => {
                    ts.word_spacing = num(0);
                    ts.char_spacing = num(1);
                    ts.tlm = mul(&[1.0, 0.0, 0.0, 1.0, 0.0, -ts.leading], &ts.tlm);
                    ts.tm = ts.tlm;
                    if let Some(Obj::Str(s)) = args.get(2) {
                        self.show(s, &mut ts, &ctm, col);
                    }
                }
                b"TJ" => {
                    if let Some(Obj::Arr(a)) = args.first() {
                        for x in a {
                            match x {
                                Obj::Str(s) => self.show(s, &mut ts, &ctm, col),
                                Obj::Num(k) => {
                                    let tx = -k / 1000.0 * ts.size * ts.scale;
                                    ts.tm = mul(&[1.0, 0.0, 0.0, 1.0, tx, 0.0], &ts.tm);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                b"Do" if depth < 8 => {
                    if let Some(Obj::Name(name)) = args.first() {
                        self.form(name, res, &ctm, col, depth, seen);
                    }
                }
                b"BI" => {
                    // Inline picture: skip its data, which can contain anything.
                    match find(content, lex.pos, b"ID") {
                        None => lex.pos = content.len(),
                        Some(id) => {
                        let mut p = id + 3;
                        loop {
                            match find(content, p, b"EI") {
                                Some(e) if e > 0 && is_ws(content[e - 1]) && content.get(e + 2).is_none_or(|c| is_ws(*c)) => {
                                    lex.pos = e + 2;
                                    break;
                                }
                                Some(e) => p = e + 2,
                                None => {
                                    lex.pos = content.len();
                                    break;
                                }
                            }
                        }
                        }
                    }
                }
                _ => {}
            }
            args.clear();
        }
        if replacement_depth.is_some() {
            col.finish_replacement(false);
        }
    }

    fn form(&self, name: &[u8], res: &Obj, ctm: &M, col: &mut Collector, depth: usize, seen: &mut HashSet<u32>) {
        if self.forms.get() >= MAX_FORMS || self.ops.get() > MAX_OPS {
            return;
        }
        let Some(r) = self.get(res).dict() else { return };
        let Some(xo) = self.key(r, b"XObject").dict() else { return };
        let Some(entry) = lookup(xo, name) else { return };
        if let Obj::Ref(n) = entry {
            if !seen.insert(*n) {
                return;
            }
        }
        let obj = self.get(entry);
        let Some(d) = obj.dict() else { return };
        if self.key(d, b"Subtype").name() != Some(b"Form") {
            return;
        }
        self.forms.set(self.forms.get() + 1);
        if self.ctl.stopped() {
            self.ops.set(MAX_OPS + 1);
            return;
        }
        let Some(data) = self.stream(entry) else { return };
        let matrix = match self.key(d, b"Matrix") {
            Obj::Arr(a) if a.len() == 6 => {
                let v: Vec<f64> = a.iter().map(|x| self.get(x).num().unwrap_or(0.0)).collect();
                [v[0], v[1], v[2], v[3], v[4], v[5]]
            }
            _ => ID,
        };
        let form_res = lookup(d, b"Resources").cloned().unwrap_or_else(|| res.clone());
        self.run(&data, &form_res, mul(&matrix, ctm), col, depth + 1, seen);
        if let Obj::Ref(n) = entry {
            seen.remove(n);
        }
    }

    fn charge_text(&self, bytes: usize) -> bool {
        let shown = self.text.get().saturating_add(bytes);
        self.text.set(shown);
        if shown > MAX_TEXT_BYTES {
            self.ops.set(MAX_OPS + 1);
            return false;
        }
        true
    }

    fn show(&self, s: &[u8], ts: &mut TextState, ctm: &M, col: &mut Collector) {
        let Some(font) = ts.font.clone() else { return };
        // All text shown is counted, however often a form is drawn again.
        if !self.charge_text(s.len()) {
            return;
        }
        let m = mul(&ts.tm, ctm);
        let (x, y) = (m[4], m[5]);
        let v_scale = (m[2] * m[2] + m[3] * m[3]).sqrt();
        let size = (ts.size.abs() * if v_scale > 0.0 { v_scale } else { 1.0 }).max(0.1);
        let mut text = String::new();
        let mut advance = 0.0;
        let mut chars = 0;
        let mut unmapped = 0;
        for (code, glyph, missing) in font.decode(s) {
            let space = if glyph == " " && !font.composite { ts.word_spacing } else { 0.0 };
            advance += (font.width(code) / 1000.0 * ts.size + ts.char_spacing + space) * ts.scale;
            chars += 1;
            unmapped += usize::from(missing);
            text.extend(glyph.chars().filter_map(imported_character));
        }
        ts.tm = mul(&[1.0, 0.0, 0.0, 1.0, advance, 0.0], &ts.tm);
        let end = mul(&ts.tm, ctm)[4];
        col.draw(Drawing { text, bounds: TextBounds { x, y, size, end }, chars, unmapped });
    }
}

// ---------------- reading the file ----------------

struct ScanState {
    lexed: usize,
    nodes: usize,
}

impl ScanState {
    fn new() -> Self {
        ScanState { lexed: 0, nodes: 0 }
    }
    fn nodes_left(&self) -> usize {
        MAX_OBJECT_NODES.saturating_sub(self.nodes)
    }
    fn exhausted(&self) -> bool {
        self.lexed > MAX_OBJECT_TEXT || self.nodes >= MAX_OBJECT_NODES
    }
}

/// Finds the objects by scanning for `N G obj`. Each object is read once at most; the total
/// bytes read as object text and total AST nodes are budgeted, so a file that is nothing but
/// unclosed strings, arrays, or huge flat `[0 0 ...]` lists cannot exhaust CPU or heap.
fn scan(data: &[u8], ctl: &Control) -> (HashMap<u32, Obj>, Vec<Dict>, ScanState) {
    let mut objs = HashMap::new();
    let mut trailers = Vec::new();
    let mut budget = ScanState::new();
    let soft_budget = data.len().saturating_mul(4).max(1 << 20);
    let mut i = 0;
    while let Some(off) = find(data, i, b"obj") {
        i = off + 3;
        if ctl.stopped() || objs.len() >= MAX_OBJECTS || budget.exhausted() {
            break;
        }
        if data.get(off + 3).is_some_and(|c| !is_ws(*c) && !is_delim(*c)) {
            continue;
        }
        // Back over "<num> <gen> ".
        let mut j = off;
        let back_ws = |j: &mut usize| {
            let s = *j;
            while *j > 0 && is_ws(data[*j - 1]) {
                *j -= 1;
            }
            *j < s
        };
        let back_digits = |j: &mut usize| {
            let s = *j;
            while *j > 0 && data[*j - 1].is_ascii_digit() && s - *j < 10 {
                *j -= 1;
            }
            *j < s
        };
        if !back_ws(&mut j) || !back_digits(&mut j) || !back_ws(&mut j) {
            continue;
        }
        let num_end = j;
        if !back_digits(&mut j) || (j > 0 && !is_ws(data[j - 1]) && !is_delim(data[j - 1])) {
            continue;
        }
        let Some(num) = std::str::from_utf8(&data[j..num_end]).ok().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let mut lex = Lex::bounded(data, off + 3, budget.nodes_left());
        let Some(mut o) = lex.obj(0) else { break };
        budget.nodes = budget.nodes.saturating_add(lex.nodes);
        budget.lexed = budget.lexed.saturating_add(lex.pos.saturating_sub(off + 3));
        if budget.lexed > soft_budget {
            // Too much object text already: what an object swallowed is not searched again.
            i = i.max(lex.pos);
        }
        if let Obj::Dict(d) = &o {
            lex.skip_ws();
            if data.get(lex.pos..).is_some_and(|r| r.starts_with(b"stream")) {
                let mut start = lex.pos + 6;
                if data.get(start) == Some(&b'\r') {
                    start += 1;
                }
                if data.get(start) == Some(&b'\n') {
                    start += 1;
                }
                let declared = lookup(d, b"Length").and_then(Obj::num).map(|n| n.max(0.0) as usize);
                let by_length = declared.and_then(|len| {
                    let end = start.checked_add(len)?;
                    let mut k = end;
                    while k < data.len() && is_ws(data[k]) {
                        k += 1;
                    }
                    data.get(k..)?.starts_with(b"endstream").then_some(end)
                });
                let end = by_length.or_else(|| {
                    find(data, start, b"endstream").map(|e| {
                        let mut e = e;
                        if e > start && data[e - 1] == b'\n' {
                            e -= 1;
                        }
                        if e > start && data[e - 1] == b'\r' {
                            e -= 1;
                        }
                        e
                    })
                });
                let end = end.unwrap_or(data.len()).max(start.min(data.len()));
                o = Obj::Stream(d.clone(), start, end);
                i = i.max(end);
            }
        }
        objs.insert(num, o);
    }
    let mut t = 0;
    while let Some(off) = find(data, t, b"trailer") {
        t = off + 7;
        if ctl.stopped() || trailers.len() >= MAX_TRAILERS || budget.exhausted() {
            break;
        }
        let mut lex = Lex::bounded(data, off + 7, budget.nodes_left());
        let o = lex.obj(0);
        budget.nodes = budget.nodes.saturating_add(lex.nodes);
        budget.lexed = budget.lexed.saturating_add(lex.pos.saturating_sub(off + 7));
        if budget.lexed > soft_budget {
            t = t.max(lex.pos);
        }
        if let Some(Obj::Dict(d)) = o {
            trailers.push(d);
        }
    }
    (objs, trailers, budget)
}

/// A PDF read on its own (tests): the normal time limit and a private decoding allowance.
#[cfg(test)]
pub fn extract(data: &[u8]) -> Result<PdfDoc, String> {
    extract_with(data, &Control::standard(), &Cell::new(MAX_DECODED_TOTAL))
}

/// `shared` is what the whole import may still decode (all PDFs, Word files and ZIPs together).
pub fn extract_with(data: &[u8], ctl: &Control, shared: &Cell<u64>) -> Result<PdfDoc, String> {
    const STOPPED: &str = "The import was stopped before this PDF was read completely.";
    if find(&data[..data.len().min(1024)], 0, b"%PDF-").is_none() {
        return Err("This is not a PDF file, or it is damaged.".into());
    }
    let (objs, mut trailers, mut scan_budget) = scan(data, ctl);
    if ctl.stop_now().is_some() {
        return Err(STOPPED.into());
    }
    let mut pdf = Pdf { data, objs, budget: Cell::new(MAX_DECODED_TOTAL), shared, ctl, forms: Cell::new(0), text: Cell::new(0), ops: Cell::new(0), fonts: Default::default(), font_bytes: Cell::new(0) };
    // Cross-reference streams carry the trailer keys too.
    for o in pdf.objs.values() {
        if let Obj::Stream(d, _, _) = o {
            if lookup(d, b"Type").and_then(Obj::name) == Some(b"XRef") {
                trailers.push(d.clone());
            }
        }
    }
    if trailers.iter().any(|t| lookup(t, b"Encrypt").is_some_and(|e| !matches!(e, Obj::Null))) {
        return Err("This PDF is encrypted (protected with a password or restricted), so its text cannot be read. Remove the protection, for example by printing it to a new PDF, and import that.".into());
    }
    // Objects inside compressed object streams share the single unified object/text/node budget.
    let objstms: Vec<u32> = pdf
        .objs
        .iter()
        .filter(|(_, o)| matches!(o, Obj::Stream(d, _, _) if lookup(d, b"Type").and_then(Obj::name) == Some(b"ObjStm")))
        .map(|(n, _)| *n)
        .collect();
    let mut extra = Vec::new();
    for n in objstms {
        if ctl.stopped() || pdf.objs.len() + extra.len() >= MAX_OBJECTS || scan_budget.exhausted() {
            break;
        }
        let r = Obj::Ref(n);
        let Some(d) = pdf.get(&r).dict().cloned() else { continue };
        let Some(buf) = pdf.stream(&r) else { continue };
        let count = pdf.key(&d, b"N").num().unwrap_or(0.0).clamp(0.0, 100_000.0) as usize;
        let first = pdf.key(&d, b"First").num().unwrap_or(0.0).max(0.0) as usize;
        let mut lex = Lex::bounded(&buf, 0, scan_budget.nodes_left());
        let mut heads = Vec::new();
        for k in 0..count {
            if (k % 1024 == 0 && ctl.stopped()) || scan_budget.exhausted() {
                break;
            }
            match (lex.obj(0), lex.obj(0)) {
                (Some(Obj::Num(num)), Some(Obj::Num(off))) => heads.push((num as u32, off.max(0.0) as usize)),
                _ => break,
            }
        }
        scan_budget.nodes = scan_budget.nodes.saturating_add(lex.nodes);
        scan_budget.lexed = scan_budget.lexed.saturating_add(lex.pos);
        for (num, off) in heads {
            // Many entries can point at the same array or string: total objects, text and nodes share one budget.
            if ctl.stopped() || pdf.objs.len() + extra.len() >= MAX_OBJECTS || scan_budget.exhausted() {
                break;
            }
            let at = first.saturating_add(off).min(buf.len());
            let mut l = Lex::bounded(&buf, at, scan_budget.nodes_left());
            let parsed = l.obj(0);
            scan_budget.nodes = scan_budget.nodes.saturating_add(l.nodes);
            scan_budget.lexed = scan_budget.lexed.saturating_add(l.pos.saturating_sub(at));
            if let Some(o) = parsed {
                extra.push((num, o));
            }
        }
    }
    for (num, o) in extra {
        pdf.objs.entry(num).or_insert(o);
    }

    // The pages, in order.
    let root = trailers.iter().rev().find_map(|t| lookup(t, b"Root").cloned());
    let mut pages: Vec<(Obj, Obj)> = Vec::new();
    let catalog = root.or_else(|| pdf.objs.iter().find(|(_, o)| o.dict().is_some_and(|d| lookup(d, b"Type").and_then(Obj::name) == Some(b"Catalog"))).map(|(n, _)| Obj::Ref(*n)));
    if let Some(cat) = catalog.as_ref().and_then(|c| pdf.get(c).dict()) {
        if let Some(p) = lookup(cat, b"Pages") {
            let mut seen = HashSet::new();
            walk_pages(&pdf, p, &Obj::Null, &mut pages, &mut seen, 0);
        }
    }
    if pages.is_empty() {
        let mut nums: Vec<u32> = pdf.objs.iter().filter(|(_, o)| o.dict().is_some_and(|d| lookup(d, b"Type").and_then(Obj::name) == Some(b"Page"))).map(|(n, _)| *n).collect();
        nums.sort_unstable();
        for n in nums.into_iter().take(MAX_PAGES) {
            let d = pdf.get(&Obj::Ref(n)).dict().cloned().unwrap_or_default();
            let res = lookup(&d, b"Resources").cloned().unwrap_or(Obj::Null);
            pages.push((Obj::Ref(n), res));
        }
    }
    if pages.is_empty() {
        return Err("No pages were found in this PDF; it may be damaged.".into());
    }

    let mut all_lines: Vec<Vec<Line>> = Vec::new();
    let mut chars = 0;
    let mut unmapped = 0;
    let mut text_bytes = 0usize;
    for (page, res) in &pages {
        if ctl.stopped() {
            break;
        }
        let Some(d) = pdf.get(page).dict() else { continue };
        let contents: Vec<Obj> = match lookup(d, b"Contents") {
            Some(Obj::Arr(a)) => a.clone(),
            Some(Obj::Ref(r)) => match pdf.get(&Obj::Ref(*r)) {
                Obj::Arr(a) => a.clone(),
                _ => vec![Obj::Ref(*r)],
            },
            _ => Vec::new(),
        };
        let mut content = Vec::new();
        for c in &contents {
            if ctl.stopped() || content.len() > MAX_STREAM {
                break;
            }
            if let Some(s) = pdf.stream(c) {
                content.extend_from_slice(&s);
                content.push(b'\n');
            }
        }
        let mut col = Collector::default();
        pdf.run(&content, res, ID, &mut col, 0, &mut HashSet::new());
        col.end_line();
        chars += col.chars;
        unmapped += col.unmapped;
        text_bytes += col.lines.iter().map(|l| l.text.len()).sum::<usize>();
        all_lines.push(col.lines);
        if pdf.ops.get() > MAX_OPS || text_bytes > MAX_TEXT_BYTES {
            break;
        }
    }
    if ctl.stop_now().is_some() {
        return Err(STOPPED.into());
    }
    let empty_pages = all_lines.iter().filter(|l| l.is_empty()).count();
    let visible: usize = all_lines.iter().flatten().map(|l| l.text.chars().filter(|c| c.is_alphanumeric()).count()).sum();
    if chars > 0 && unmapped * 10 > chars * 3 {
        return Err("The text in this PDF uses fonts without a character map, so it cannot be read reliably.".into());
    }
    if visible < 3 {
        return Err("No text could be found in this PDF. It may be a scanned or image-only document; recognising text in pictures (OCR) is not supported.".into());
    }
    Ok(PdfDoc { blocks: paragraphs(&all_lines), pages: pages.len(), empty_pages })
}

fn walk_pages(pdf: &Pdf, node: &Obj, inherited: &Obj, out: &mut Vec<(Obj, Obj)>, seen: &mut HashSet<u32>, depth: usize) {
    if depth > 64 || out.len() >= MAX_PAGES {
        return;
    }
    if let Obj::Ref(n) = node {
        if !seen.insert(*n) {
            return;
        }
    }
    let Some(d) = pdf.get(node).dict() else { return };
    let res = lookup(d, b"Resources").cloned().unwrap_or_else(|| inherited.clone());
    match pdf.key(d, b"Kids") {
        Obj::Arr(kids) => {
            for k in kids {
                walk_pages(pdf, k, &res, out, seen, depth + 1);
            }
        }
        _ => out.push((node.clone(), res)),
    }
}

/// Lines to paragraphs and headings. The body size is the size most text has.
fn paragraphs(pages: &[Vec<Line>]) -> Vec<Block> {
    let mut weight: HashMap<i64, usize> = HashMap::new();
    for l in pages.iter().flatten() {
        *weight.entry((l.size * 2.0).round() as i64).or_default() += l.text.chars().count();
    }
    let body = weight.into_iter().max_by_key(|(s, w)| (*w, -*s)).map(|(s, _)| s as f64 / 2.0).unwrap_or(12.0).max(1.0);
    let mut out = Vec::new();
    for lines in pages {
        let mut para = String::new();
        let mut para_size = 0.0;
        let mut last_y = 0.0;
        let flush = |para: &mut String, out: &mut Vec<Block>| {
            if !para.trim().is_empty() {
                out.push(Block::Paragraph(vec![super::doc::t(para.trim().to_string())]));
            }
            para.clear();
        };
        for l in lines {
            let ratio = l.size / body;
            let heading = ratio >= 1.2 && l.text.chars().count() <= 200 && l.text.chars().any(char::is_alphabetic);
            let gap = last_y - l.y;
            let new_para = para.is_empty() || heading || (l.size - para_size).abs() > 0.15 * body || gap > 1.7 * l.size.max(para_size) || gap < -0.5 * l.size;
            if heading {
                flush(&mut para, &mut out);
                let level = if ratio >= 1.8 { 1 } else if ratio >= 1.4 { 2 } else { 3 };
                // A heading split over two lines joins the previous heading.
                if let Some(Block::Heading(lv, inl)) = out.last_mut() {
                    if *lv == level && (last_y - l.y) < 1.6 * l.size && para_size == l.size {
                        inl.push(Inline::Text(format!(" {}", l.text), Default::default()));
                        last_y = l.y;
                        continue;
                    }
                }
                out.push(Block::Heading(level, vec![super::doc::t(l.text.clone())]));
                para_size = l.size;
                last_y = l.y;
                continue;
            }
            if new_para {
                flush(&mut para, &mut out);
                para.push_str(&l.text);
            } else if para.ends_with('-') && para.chars().rev().nth(1).is_some_and(char::is_alphabetic) && l.text.chars().next().is_some_and(char::is_lowercase) {
                para.pop();
                para.push_str(&l.text);
            } else {
                para.push(' ');
                para.push_str(&l.text);
            }
            para_size = l.size;
            last_y = l.y;
        }
        flush(&mut para, &mut out);
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Builds a small PDF from object bodies (object 1 is the catalog).
    pub fn build_pdf(objects: &[String]) -> Vec<u8> {
        let mut out = "%PDF-1.4\n%\u{e2}\u{e3}\u{cf}\u{d3}\n".as_bytes().to_vec();
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, body).as_bytes());
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for o in offsets {
            out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
        out
    }

    fn stream(content: &str, flate: bool) -> String {
        if flate {
            let z = crate::export::deflate::zlib_compress(content.as_bytes());
            // Binary bytes as Latin-1 chars round-trip through the byte buffer below.
            let body: String = z.iter().map(|b| char::from(*b)).collect();
            format!("<< /Length {} /Filter /FlateDecode >>\nstream\n{}\nendstream", z.len(), body)
        } else {
            format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content)
        }
    }

    /// Like build_pdf, but object bodies may hold Latin-1 encoded binary.
    pub fn latin1(objects: &[String]) -> Vec<u8> {
        build_pdf(objects).iter().map(|b| *b).collect::<Vec<u8>>().into_iter().collect::<Vec<u8>>()
    }

    fn pdf_bytes(objects: &[String]) -> Vec<u8> {
        // Re-encode chars < 256 as single bytes.
        let text = String::from_utf8(build_pdf(objects)).unwrap_or_default();
        text.chars().map(|c| c as u32 as u8).collect()
    }

    pub fn sample_pdf() -> Vec<u8> {
        let content = "BT /F1 24 Tf 72 720 Td (Annual Report) Tj ET\n\
                       BT /F1 12 Tf 72 690 Td (This is the first line of a para-) Tj 0 -14 Td (graph that continues here.) Tj ET\n\
                       BT /F1 12 Tf 72 640 Td [(Kerned) -300 (words)] TJ ( and) Tj ET\n\
                       BT /F2 12 Tf 72 610 Td <00410042> Tj ET\n\
                       BI /W 4 /H 4 /BPC 8 /CS /G ID \x00EI\x01\x02 EI\n\
                       BT /F3 12 Tf 72 580 Td (\x01\x02) Tj ET";
        let cmap = "/CIDInit /ProcSet findresource begin 12 dict begin begincmap 1 begincodespacerange <0000> <FFFF> endcodespacerange 2 beginbfchar <0041> <00C9> <0042> <0074> endbfchar endcmap";
        pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 /Resources << /Font << /F1 5 0 R /F2 6 0 R /F3 9 0 R >> >> >>".into(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>".into(),
            stream(content, true),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
            "<< /Type /Font /Subtype /Type0 /BaseFont /X /Encoding /Identity-H /ToUnicode 7 0 R /DescendantFonts [8 0 R] >>".into(),
            stream(cmap, false),
            "<< /Type /Font /Subtype /CIDFontType2 /DW 500 >>".into(),
            "<< /Type /Font /Subtype /Type1 /Encoding << /Differences [1 /Euro /fi] >> >>".into(),
        ])
    }

    #[test]
    fn text_headings_and_fonts() {
        let d = extract(&sample_pdf()).unwrap();
        let text: Vec<String> = d.blocks.iter().map(|b| match b {
            Block::Heading(l, i) => format!("H{l}:{}", super::super::doc::plain(i)),
            Block::Paragraph(i) => super::super::doc::plain(i),
            _ => String::new(),
        }).collect();
        assert_eq!(text[0], "H1:Annual Report");
        assert_eq!(text[1], "This is the first line of a paragraph that continues here.");
        assert_eq!(text[2], "Kerned words and");
        assert_eq!(text[3], "Ét", "ToUnicode map of a composite font");
        assert_eq!(text[4], "€ﬁ", "Differences");
        assert_eq!(d.pages, 1);
    }

    fn extracted_text(bytes: &[u8]) -> String {
        extract(bytes)
            .unwrap()
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::Heading(_, inlines) | Block::Paragraph(inlines) => Some(super::super::doc::plain(inlines)),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn reimports_notayas_pdf_text_in_logical_order() {
        use crate::export::{doc, pdf, tests::sample_doc};
        use serde_json::json;

        let lines = ["English control 42", "مرحبا بالعالم", "שלום עולם", "Prefix مرحبا suffix 19", "שלום(somethinglong)"];
        let paragraphs: Vec<_> = lines
            .iter()
            .map(|line| json!({"type": "paragraph", "content": [{"type": "text", "text": line}]}))
            .collect();
        let doc = sample_doc(doc::parse(&json!({"type": "doc", "content": paragraphs}).to_string(), &|_| None));
        let exported = pdf::render_at(&doc, 0);
        assert_eq!(exported.chars_replaced, 0);
        let actual = extracted_text(&exported.bytes);
        for line in lines {
            assert_eq!(actual.matches(line).count(), 1, "{line:?} in {actual:?}");
        }
    }

    #[test]
    fn unmarked_logical_order_uses_the_font_map() {
        let cmap = "1 begincodespacerange <0000> <FFFF> endcodespacerange \
                    4 beginbfchar <0001> <05E9> <0002> <05DC> <0003> <05D5> <0004> <05DD> endbfchar";
        let bytes = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
            "<< /Type /Page /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".into(),
            stream("BT /F1 12 Tf 72 700 Td <0001000200030004> Tj ET", false),
            "<< /Type /Font /Subtype /Type0 /Encoding /Identity-H /ToUnicode 6 0 R >>".into(),
            stream(cmap, false),
        ]);
        assert_eq!(extracted_text(&bytes), "שלום");
    }

    #[test]
    fn marked_text_replaces_a_span_once_without_dropping_adjacent_text() {
        let content = "BT /F1 12 Tf 72 700 Td (Before ) Tj \
                       /Span << /ActualText <FEFF05E905DC05D505DD> >> BDC \
                       (12) Tj /Span BMC [(34) -50 (56)] TJ EMC (78) Tj EMC \
                       ( after) Tj ET \
                       /Span << /ActualText <FEFF0041> >> BDC \
                       BT /F1 12 Tf 72 680 Td (unfinished span stays readable) Tj ET";
        let bytes = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
            "<< /Type /Page /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".into(),
            stream(content, false),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
        ]);
        assert_eq!(extracted_text(&bytes), "Before שלום after unfinished span stays readable");
    }

    #[test]
    fn marked_form_text_retains_the_space_to_a_separate_column() {
        let content = "/Span << /ActualText <FEFF05E905DC05D505DD> >> BDC /Word Do EMC \
                       BT /F1 12 Tf 160 700 Td (42) Tj ET";
        let form = stream("BT /F1 12 Tf 72 700 Td (word) Tj ET", false)
            .replacen("<<", "<< /Type /XObject /Subtype /Form /BBox [0 0 612 792]", 1);
        let bytes = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
            "<< /Type /Page /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> /XObject << /Word 6 0 R >> >> >>".into(),
            stream(content, false),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
            form,
        ]);
        assert_eq!(extracted_text(&bytes), "שלום 42");
    }

    #[test]
    fn honest_failures() {
        let enc = String::from_utf8_lossy(&pdf_bytes(&["<< /Type /Catalog /Pages 2 0 R >>".into(), "<< /Type /Pages /Kids [] >>".into()]))
            .replace("/Root 1 0 R", "/Root 1 0 R /Encrypt << /Filter /Standard >>");
        assert!(extract(enc.as_bytes()).err().unwrap().contains("encrypted"));

        let scanned = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] >>".into(),
            "<< /Type /Page /Contents 4 0 R >>".into(),
            stream("q 612 0 0 792 0 0 cm /Im1 Do Q", false),
        ]);
        assert!(extract(&scanned).err().unwrap().contains("scanned"));

        let no_map = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] /Resources << /Font << /F1 5 0 R >> >> >>".into(),
            "<< /Type /Page /Contents 4 0 R >>".into(),
            stream("BT /F1 12 Tf 72 700 Td <0012003400560078009a00bc> Tj ET", false),
            "<< /Type /Font /Subtype /Type0 /Encoding /Identity-H >>".into(),
        ]);
        assert!(extract(&no_map).err().unwrap().contains("character map"));
        assert!(extract(b"hello").err().unwrap().contains("not a PDF"));
    }

    #[test]
    fn hostile_pdfs_do_not_hang_or_crash() {
        let _heavy = crate::importers::heavy();
        let mut cases: Vec<Vec<u8>> = vec![
            b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 1 0 R >> endobj trailer << /Root 1 0 R >>".to_vec(),
            b"%PDF-1.7\n1 0 obj [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[ endobj".to_vec(),
            b"%PDF-1.7\n1 0 obj << /Length 999999999 >> stream\nxx".to_vec(),
            b"%PDF-1.7\n1 0 obj (unterminated \\".to_vec(),
            b"%PDF-1.7\n 0 obj 99999999999 0 obj <</#zz 1>> endobj".to_vec(),
        ];
        // A form that draws itself.
        cases.push(pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] >>".into(),
            "<< /Type /Page /Contents 4 0 R /Resources << /XObject << /X 4 0 R >> >> >>".into(),
            "<< /Type /XObject /Subtype /Form /Length 6 /Resources << /XObject << /X 4 0 R >> >> >>\nstream\n/X Do \nendstream".into(),
        ]));
        // A page tree that loops.
        cases.push(pdf_bytes(&["<< /Type /Catalog /Pages 2 0 R >>".into(), "<< /Type /Pages /Kids [2 0 R 2 0 R] >>".into()]));
        let start = std::time::Instant::now();
        for c in cases {
            let _ = extract(&c);
        }
        assert!(start.elapsed().as_secs() < 60);
        assert_eq!(ascii85(b"<~87cURD]i,\"Ebo80~>", 100), b"Hello World!");
        assert_eq!(run_length(&[2, b'a', b'b', b'c', 254, b'x', 128], 100), b"abcxxx");
        assert_eq!(MAC_ROMAN_HIGH.chars().count(), 128);
        let _ = latin1(&[]);
    }

    use crate::importers::within;

    #[test]
    fn an_unclosed_hex_string_does_not_panic() {
        // The reviewer's 25-byte probe: `<41` with no `>` used to slice past the end.
        for case in [&b"%PDF-1.4\n1 0 obj <</A <41"[..], b"%PDF-1.4\n1 0 obj <", b"%PDF-1.4\n1 0 obj <4", b"%PDF-1.4\n1 0 obj (", b"%PDF-1.4\ntrailer <</A <41", b"%PDF-1.4\n1 0 obj <</A <<41"] {
            let _ = extract(case);
        }
        let mut lex = Lex::new(b"<41", 0);
        assert!(matches!(lex.obj(0), Some(Obj::Str(s)) if s == [0x41]));
        assert_eq!(lex.pos, 3);
        assert!(lex.obj(0).is_none());
    }

    /// Small deterministic generator for the fuzz tests.
    fn rng(seed: u64) -> impl FnMut() -> u64 {
        let mut s = seed;
        move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        }
    }

    const SOUP: &[&str] = &[
        "obj", "endobj", "stream", "endstream", "<<", ">>", "[", "]", "(", ")", "<", ">", "/Type", "/Page", "/Pages", "/Kids", "/Contents", "/Length", "/Filter",
        "/FlateDecode", "/ASCIIHexDecode", "/ASCII85Decode", "/LZWDecode", "/RunLengthDecode", "/Font", "/F1", "/XObject", "/Subtype", "/Form", "/Type0", "/ToUnicode",
        "/DescendantFonts", "/W", "/DW", "/Widths", "/Encoding", "/Differences", "/Resources", "/Catalog", "/Root", "/Encrypt", "/N", "/First", "/ObjStm", "1 0 R",
        "2 0 R", "3 0 R", "1", "2", "0", "65535", "-7", "99999999999", "4294967296", "1e9", ".", "-", "<41>", "<0041>", "<>", "(abc)", "(\\", "\\", "BT", "ET", "Tf", "Td",
        "Tj", "TJ", "Tm", "cm", "q", "Q", "Do", "BI", "ID", "EI", "'", "\"", "begincodespacerange", "endcodespacerange", "beginbfchar", "endbfchar", "beginbfrange",
        "endbfrange", "<0000>", "<FFFF>", "<00000000>", "<FFFFFFFF>", "xref", "trailer", "startxref", "%", "\n", " ", "\r", "\t", "~>", "<~", "z", "!", "u",
    ];

    fn soup(next: &mut impl FnMut() -> u64, tokens: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for _ in 0..tokens {
            out.extend_from_slice(SOUP[(next() % SOUP.len() as u64) as usize].as_bytes());
            out.push(if next() % 4 == 0 { b'\n' } else { b' ' });
        }
        out
    }

    #[test]
    fn token_soup_never_panics_or_hangs() {
        let _heavy = crate::importers::heavy();
        // 50,000 random files made of PDF tokens: whole files, and the skeleton of a real one with
        // soup as the page content, as a font map, as an object stream and as a width list.
        within(400, "50,000 soup cases", || {
            let mut next = rng(0x9e37_79b9_7f4a_7c15);
            for case in 0..50_000u32 {
                let n = 3 + (next() % 40) as usize;
                let body = soup(&mut next, n);
                let text = String::from_utf8_lossy(&body).to_string();
                let data = match case % 5 {
                    0 => [b"%PDF-1.4\n".to_vec(), body].concat(),
                    1 => pdf_bytes(&[
                        "<< /Type /Catalog /Pages 2 0 R >>".into(),
                        "<< /Type /Pages /Kids [3 0 R] /Resources << /Font << /F1 5 0 R >> /XObject << /X 4 0 R >> >> >>".into(),
                        "<< /Type /Page /Contents 4 0 R >>".into(),
                        stream(&format!("BT /F1 12 Tf {text} ET /X Do"), false),
                        "<< /Type /Font /Subtype /Type1 >>".into(),
                    ]),
                    2 => pdf_bytes(&[
                        "<< /Type /Catalog /Pages 2 0 R >>".into(),
                        "<< /Type /Pages /Kids [3 0 R] /Resources << /Font << /F1 5 0 R >> >> >>".into(),
                        "<< /Type /Page /Contents 4 0 R >>".into(),
                        stream("BT /F1 12 Tf <00410042> Tj (abc) Tj ET", false),
                        "<< /Type /Font /Subtype /Type0 /ToUnicode 6 0 R /DescendantFonts [7 0 R] >>".into(),
                        stream(&format!("1 begincodespacerange <0000> <FFFF> endcodespacerange {text}"), false),
                        format!("<< /Type /Font /W [{text}] >>"),
                    ]),
                    3 => pdf_bytes(&[
                        "<< /Type /Catalog /Pages 2 0 R >>".into(),
                        "<< /Type /Pages /Kids [3 0 R] >>".into(),
                        "<< /Type /Page /Contents 4 0 R >>".into(),
                        stream("BT (x) Tj ET", false),
                        format!("<< /Type /ObjStm /N 5 /First 10 /Length {} >>\nstream\n{}\nendstream", text.len(), text),
                    ]),
                    _ => {
                        let mut d = sample_pdf();
                        for _ in 0..6 {
                            let i = (next() % d.len() as u64) as usize;
                            d[i] = next() as u8;
                        }
                        d
                    }
                };
                let _ = extract(&data);
            }
        });
    }

    #[test]
    fn a_long_bfrange_and_huge_cmaps_are_capped() {
        let _heavy = crate::importers::heavy();
        // A 16 KB file: ranges of 65,536 codes, each to a long destination, must not build
        // gigabytes of strings.
        let dest: String = (0..32).map(|_| "0041").collect();
        let mut cmap = String::from("1 begincodespacerange <0000> <FFFF> endcodespacerange\n");
        for _ in 0..400 {
            cmap.push_str(&format!("1 beginbfrange <0000> <FFFF> <{dest}> endbfrange\n"));
        }
        let (map, _) = within(120, "bfrange", || parse_cmap(cmap.as_bytes()));
        assert!(map.len() <= MAX_CMAP_ENTRIES + 1, "{}", map.len());
        let bytes: usize = map.values().map(String::len).sum();
        assert!(bytes <= MAX_CMAP_BYTES + 4096, "{bytes}");
        // A range whose destination is an array of long strings.
        let mut cmap = String::from("1 beginbfrange <0000> <FFFF> [");
        for _ in 0..5000 {
            cmap.push_str(&format!("<{dest}>"));
        }
        cmap.push_str("] endbfrange");
        let (map, _) = within(120, "bfrange array", || parse_cmap(cmap.as_bytes()));
        assert!(map.len() <= MAX_CMAP_ENTRIES + 1);
        // Hex and ASCII85 decoders stop at the limit.
        assert_eq!(ascii_hex(&vec![b'4'; 10_000], 100).len(), 100);
        assert!(ascii85(&vec![b'u'; 10_000], 100).len() <= 104);
    }

    #[test]
    fn an_object_scan_with_endless_objects_is_not_quadratic() {
        let _heavy = crate::importers::heavy();
        // `1 0 obj [` over and over (the reviewer used 45 to 180 KB), and the same with strings.
        for unit in ["1 0 obj [ ", "1 0 obj ( ", "1 0 obj << /A ", "1 0 obj <4 ", "trailer ( "] {
            let data = format!("%PDF-1.4\n{}", unit.repeat(300_000 / unit.len()));
            let r = within(120, unit, || extract(data.as_bytes()));
            assert!(r.is_err());
        }
        // Many entries of an object stream pointing at one unclosed array.
        let entries: String = (1..=50_000).map(|n| format!("{n} 0 ")).collect();
        let body = format!("{entries}[ [ [ [ {}", "x ".repeat(100_000));
        let objstm = format!("<< /Type /ObjStm /N 50000 /First {} /Length {} >>\nstream\n{}\nendstream", entries.len(), body.len(), body);
        let data = pdf_bytes(&["<< /Type /Catalog /Pages 2 0 R >>".into(), "<< /Type /Pages /Kids [] >>".into(), objstm]);
        let _ = within(120, "objstm heads", || extract(&data));
    }

    #[test]
    fn inline_picture_markers_and_forms_are_bounded() {
        let _heavy = crate::importers::heavy();
        // Thousands of `BI` with no `ID`, and forms that draw ten forms each, ten levels deep.
        let content = "BI ".repeat(200_000);
        let data = pdf_bytes(&[
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            "<< /Type /Pages /Kids [3 0 R] >>".into(),
            "<< /Type /Page /Contents 4 0 R >>".into(),
            stream(&content, false),
        ]);
        let _ = within(120, "BI", || extract(&data));
        let xobjects: String = (0..10).map(|i| format!("/X{i} {} 0 R ", 5 + i)).collect();
        let draws: String = (0..10).map(|i| format!("/X{i} Do ")).collect();
        let mut objs = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] >>".to_string(),
            format!("<< /Type /Page /Contents 4 0 R /Resources << /XObject << {xobjects} >> >> >>"),
            stream(&draws, false),
        ];
        for _ in 0..10 {
            objs.push(format!("<< /Type /XObject /Subtype /Form /Resources << /XObject << {xobjects} >> >> /Length {} >>\nstream\n{draws}\nendstream", draws.len()));
        }
        let data = pdf_bytes(&objs);
        let _ = within(120, "form explosion", || extract(&data));
    }

    #[test]
    fn a_stopped_import_stops_the_pdf_reader() {
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let ctl = Control::new(std::time::Duration::from_secs(60), cancel);
        let r = extract_with(&sample_pdf(), &ctl, &Cell::new(MAX_DECODED_TOTAL));
        assert!(r.err().unwrap().contains("stopped"));
        let r = extract_with(&sample_pdf(), &Control::expired(), &Cell::new(MAX_DECODED_TOTAL));
        assert!(r.err().unwrap().contains("stopped"));
    }

    #[test]
    fn the_shared_allowance_limits_what_a_pdf_may_inflate() {
        assert!(extract_with(&sample_pdf(), &Control::standard(), &Cell::new(MAX_DECODED_TOTAL)).is_ok());
        // With nothing left in the shared allowance the compressed page cannot be read.
        assert!(extract_with(&sample_pdf(), &Control::standard(), &Cell::new(0)).is_err());
        // A good read spends from it.
        let shared = Cell::new(MAX_DECODED_TOTAL);
        extract_with(&sample_pdf(), &Control::standard(), &shared).unwrap();
        assert!(shared.get() < MAX_DECODED_TOTAL);
    }

    #[test]
    fn hostile_pdf_node_amplification_filter_chains_and_fonts_are_bounded() {
        let _heavy = crate::importers::heavy();
        // 1. Flat `[0 0 ...]` amplification across `scan` AND `ObjStm` stops at MAX_OBJECT_NODES.
        let many_zeros = "0 ".repeat(100_000);
        let arr = format!("[{many_zeros}]");
        let objstm_body = format!("10 0 {arr}");
        let objstm = format!("<< /Type /ObjStm /N 1 /First 5 /Length {} >>\nstream\n{objstm_body}\nendstream", objstm_body.len());
        let objs: Vec<String> = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [] >>".to_string(),
            objstm,
        ]
        .into_iter()
        .chain((0..25).map(|_| arr.clone()))
        .collect();
        let data = pdf_bytes(&objs);
        let err = within(500, "flat zero arrays + objstm", || extract(&data)).err().unwrap();
        assert!(err.contains("No pages") || err.contains("stopped"), "{err}");

        // 2. Filter-chain / invalid-Filter / uncompressed-over-budget checks in Pdf::stream.
        let chain: String = std::iter::repeat("/FlateDecode ").take(12).collect();
        let raw_payload = b"hello uncompressed stream payload";
        let shared = Cell::new(100u64);
        let mut map = HashMap::new();
        map.insert(1, Obj::Stream(vec![(b"Filter".to_vec(), Obj::Arr(vec![Obj::Name(b"FlateDecode".to_vec()); 12]))], 0, 4));
        map.insert(2, Obj::Stream(vec![(b"Filter".to_vec(), Obj::Arr(vec![Obj::Num(123.0)]))], 0, 4));
        map.insert(3, Obj::Stream(Vec::new(), 0, raw_payload.len()));
        map.insert(4, Obj::Stream(vec![(b"ToUnicode".to_vec(), Obj::Ref(5))], 0, 0));
        let cmap = b"1 begincodespacerange <00> <FF> endcodespacerange\n1 beginbfchar <41> <005A> endbfchar\n";
        let font_dict = Obj::Dict(vec![
            (b"Subtype".to_vec(), Obj::Name(b"Type0".to_vec())),
            (b"ToUnicode".to_vec(), Obj::Ref(5)),
        ]);
        let mut combined = raw_payload.to_vec();
        let cmap_start = combined.len();
        combined.extend_from_slice(cmap);
        map.insert(5, Obj::Stream(Vec::new(), cmap_start, combined.len()));
        map.insert(10, font_dict);
        let _ = chain;
        let pdf = Pdf {
            data: &combined,
            objs: map,
            budget: Cell::new(MAX_DECODED_TOTAL),
            shared: &shared,
            ctl: &Control::standard(),
            forms: Cell::new(0),
            text: Cell::new(0),
            ops: Cell::new(0),
            fonts: Default::default(),
            font_bytes: Cell::new(0),
        };
        assert!(pdf.stream(&Obj::Ref(1)).is_none(), "deep filter chain must fail");
        assert!(pdf.stream(&Obj::Ref(2)).is_none(), "non-Name /Filter element must fail instead of falling back to uncompressed");
        assert!(pdf.stream(&Obj::Ref(3)).is_some(), "uncompressed stream within budget reads and spends");
        assert!(pdf.stream(&Obj::Ref(3)).is_some());
        assert!(pdf.stream(&Obj::Ref(3)).is_some());
        assert!(pdf.stream(&Obj::Ref(3)).is_none(), "uncompressed stream beyond remaining shared budget must fail");

        // 3. Cached font budget stops unbounded font allocations once MAX_TOTAL_FONT_BYTES is reached.
        pdf.shared.set(MAX_DECODED_TOTAL);
        let f_ok = pdf.font(&Obj::Ref(10));
        assert_eq!(f_ok.map.get(&0x41).map(String::as_str), Some("Z"));
        assert!(pdf.font_bytes.get() >= 2048);
        pdf.font_bytes.set(MAX_TOTAL_FONT_BYTES);
        let mut objs2 = pdf.objs.clone();
        objs2.insert(11, objs2.get(&10).unwrap().clone());
        let pdf2 = Pdf { data: pdf.data, objs: objs2, budget: Cell::new(MAX_DECODED_TOTAL), shared: &shared, ctl: pdf.ctl, forms: Cell::new(0), text: Cell::new(0), ops: Cell::new(0), fonts: Default::default(), font_bytes: Cell::new(MAX_TOTAL_FONT_BYTES) };
        let f_capped = pdf2.font(&Obj::Ref(11));
        assert!(f_capped.map.is_empty() && f_capped.widths.is_empty(), "real populated font bypassed when font budget exhausted");
    }
}
