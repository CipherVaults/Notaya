//! What the exports need to know about a picture from the vault: its kind and size in pixels,
//! and, for the PDF, its pixels in a form a PDF can hold.
//!
//! Only the four kinds the vault accepts exist (PNG, JPEG, GIF, WebP; see `add_attachment`).
//! Markdown and Word files carry the original bytes. A PDF can carry JPEG as it is, and PNG
//! after it is decoded here. GIF and WebP are not decoded: the PDF shows a short note instead.

use super::deflate;
use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Png,
    Jpeg,
    Gif,
    Webp,
}

impl Kind {
    pub fn from_mime(mime: &str) -> Option<Kind> {
        match mime {
            "image/png" => Some(Kind::Png),
            "image/jpeg" => Some(Kind::Jpeg),
            "image/gif" => Some(Kind::Gif),
            "image/webp" => Some(Kind::Webp),
            _ => None,
        }
    }
    pub fn ext(self) -> &'static str {
        match self {
            Kind::Png => "png",
            Kind::Jpeg => "jpg",
            Kind::Gif => "gif",
            Kind::Webp => "webp",
        }
    }
    pub fn mime(self) -> &'static str {
        match self {
            Kind::Png => "image/png",
            Kind::Jpeg => "image/jpeg",
            Kind::Gif => "image/gif",
            Kind::Webp => "image/webp",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::Png => "PNG",
            Kind::Jpeg => "JPEG",
            Kind::Gif => "GIF",
            Kind::Webp => "WebP",
        }
    }
}

/// Width and height in pixels, read from the file header. `None` if the header is not understood.
pub fn dimensions(kind: Kind, b: &[u8]) -> Option<(u32, u32)> {
    let be32 = |i: usize| b.get(i..i + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]));
    let dims = match kind {
        Kind::Png => (be32(16)?, be32(20)?),
        Kind::Gif => {
            let s = b.get(6..10)?;
            (u16::from_le_bytes([s[0], s[1]]) as u32, u16::from_le_bytes([s[2], s[3]]) as u32)
        }
        Kind::Jpeg => {
            let f = jpeg_frame(b)?;
            (f.width, f.height)
        }
        Kind::Webp => webp_dimensions(b)?,
    };
    (dims.0 > 0 && dims.1 > 0).then_some(dims)
}

fn webp_dimensions(b: &[u8]) -> Option<(u32, u32)> {
    let chunk = b.get(12..16)?;
    let le24 = |i: usize| b.get(i..i + 3).map(|s| s[0] as u32 | (s[1] as u32) << 8 | (s[2] as u32) << 16);
    match chunk {
        b"VP8X" => Some((le24(24)? + 1, le24(27)? + 1)),
        b"VP8 " => {
            let s = b.get(26..30)?;
            Some(((u16::from_le_bytes([s[0], s[1]]) & 0x3FFF) as u32, (u16::from_le_bytes([s[2], s[3]]) & 0x3FFF) as u32))
        }
        b"VP8L" => {
            let s = b.get(21..25)?;
            let v = u32::from_le_bytes([s[0], s[1], s[2], s[3]]);
            Some(((v & 0x3FFF) + 1, ((v >> 14) & 0x3FFF) + 1))
        }
        _ => None,
    }
}

pub struct JpegFrame {
    pub width: u32,
    pub height: u32,
    pub components: u8,
    /// Adobe's marker: CMYK pictures saved by Photoshop store inverted values.
    pub adobe: bool,
}

/// Reads the frame header (SOFn) of a JPEG file.
pub fn jpeg_frame(b: &[u8]) -> Option<JpegFrame> {
    if !b.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut i = 2;
    let mut adobe = false;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return None;
        }
        let marker = b[i + 1];
        if marker == 0xFF {
            i += 1;
            continue;
        }
        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        if len < 2 {
            return None;
        }
        if marker == 0xEE && b.get(i + 4..i + 9) == Some(b"Adobe") {
            adobe = true;
        }
        let is_sof = (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if is_sof {
            let s = b.get(i + 4..i + 10)?;
            return Some(JpegFrame {
                height: u16::from_be_bytes([s[1], s[2]]) as u32,
                width: u16::from_be_bytes([s[3], s[4]]) as u32,
                components: s[5],
                adobe,
            });
        }
        if marker == 0xDA {
            return None;
        }
        i += 2 + len;
    }
    None
}

/// A PNG's header fields and its chunks that matter here.
pub struct PngParts {
    pub width: u32,
    pub height: u32,
    pub depth: u8,
    pub color: u8,
    pub interlaced: bool,
    pub palette: Vec<u8>,
    pub trns: Vec<u8>,
    /// The zlib stream (all IDAT chunks joined).
    pub idat: Vec<u8>,
}

pub fn png_parts(b: &[u8]) -> Result<PngParts> {
    let bad = || Error::invalid("A PNG picture could not be read.");
    if !b.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Err(bad());
    }
    let mut p = PngParts { width: 0, height: 0, depth: 0, color: 0, interlaced: false, palette: vec![], trns: vec![], idat: vec![] };
    let mut i = 8;
    while i + 8 <= b.len() {
        let len = u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
        let kind = &b[i + 4..i + 8];
        let data = b.get(i + 8..i + 8 + len).ok_or_else(bad)?;
        match kind {
            b"IHDR" if len >= 13 => {
                p.width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                p.height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                p.depth = data[8];
                p.color = data[9];
                p.interlaced = data[12] != 0;
            }
            b"PLTE" => p.palette = data.to_vec(),
            b"tRNS" => p.trns = data.to_vec(),
            b"IDAT" => p.idat.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        i += 12 + len;
    }
    let valid_depth = match p.color {
        0 => matches!(p.depth, 1 | 2 | 4 | 8 | 16),
        3 => matches!(p.depth, 1 | 2 | 4 | 8),
        2 | 4 | 6 => matches!(p.depth, 8 | 16),
        _ => false,
    };
    if p.width == 0 || p.height == 0 || !valid_depth || p.idat.is_empty() || (p.color == 3 && p.palette.len() < 3) {
        return Err(bad());
    }
    Ok(p)
}

impl PngParts {
    pub fn channels(&self) -> usize {
        match self.color {
            2 => 3,
            4 => 2,
            6 => 4,
            _ => 1,
        }
    }

    /// Bytes in one row of pixels (without the filter byte).
    pub fn row_bytes(&self) -> usize {
        (self.width as usize * self.channels() * self.depth as usize).div_ceil(8)
    }

    /// True when a PDF can use the compressed data as it is (FlateDecode with the PNG
    /// predictors): no transparency, no interlacing, at most 8 bits per sample.
    pub fn pdf_passthrough(&self) -> bool {
        !self.interlaced && self.trns.is_empty() && matches!((self.color, self.depth), (0, 1..=8) | (3, _) | (2, 8))
    }

    /// Decodes the picture to 8-bit RGB and, when any pixel is not fully opaque, an alpha plane.
    pub fn decode_rgb(&self) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let bad = || Error::invalid("A PNG picture could not be read.");
        if self.interlaced {
            return Err(Error::invalid("Interlaced PNG pictures are not supported in PDF exports."));
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let rb = self.row_bytes();
        let expected = h.checked_mul(rb + 1).ok_or_else(bad)?;
        // A picture in the vault is at most 25 MB, but it could still describe a huge canvas.
        if w.saturating_mul(h) > 40_000_000 {
            return Err(Error::invalid("A picture is too large to put in a PDF."));
        }
        let raw = deflate::zlib_decompress(&self.idat, expected)?;
        if raw.len() < expected {
            return Err(bad());
        }
        let bpp = (self.channels() * self.depth as usize).div_ceil(8).max(1);
        let mut rows = vec![0u8; h * rb];
        for y in 0..h {
            let filter = raw[y * (rb + 1)];
            let src = &raw[y * (rb + 1) + 1..(y + 1) * (rb + 1)];
            let (done, rest) = rows.split_at_mut(y * rb);
            let prior: &[u8] = if y == 0 { &[] } else { &done[(y - 1) * rb..] };
            let cur = &mut rest[..rb];
            for x in 0..rb {
                let a = if x >= bpp { cur[x - bpp] as i32 } else { 0 };
                let b = if y > 0 { prior[x] as i32 } else { 0 };
                let c = if y > 0 && x >= bpp { prior[x - bpp] as i32 } else { 0 };
                let pred = match filter {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => (a + b) / 2,
                    4 => {
                        let p = a + b - c;
                        let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                        if pa <= pb && pa <= pc {
                            a
                        } else if pb <= pc {
                            b
                        } else {
                            c
                        }
                    }
                    _ => return Err(bad()),
                };
                cur[x] = (src[x] as i32 + pred) as u8;
            }
        }

        let mut rgb = Vec::with_capacity(w * h * 3);
        let mut alpha = Vec::with_capacity(w * h);
        let depth = self.depth as usize;
        // One sample (channel `ch` of pixel `x` in row `row`), scaled to 8 bits.
        let sample = |row: &[u8], x: usize, ch: usize| -> (u8, u16) {
            let idx = x * self.channels() + ch;
            match depth {
                16 => {
                    let v = u16::from_be_bytes([row[idx * 2], row[idx * 2 + 1]]);
                    ((v >> 8) as u8, v)
                }
                8 => (row[idx], row[idx] as u16),
                _ => {
                    let bit = idx * depth;
                    let v = (row[bit / 8] >> (8 - depth - bit % 8)) & ((1 << depth) - 1);
                    let scaled = if self.color == 3 { v } else { (v as u32 * 255 / ((1 << depth) - 1)) as u8 };
                    (scaled, v as u16)
                }
            }
        };
        let trns16 = |i: usize| self.trns.get(i * 2..i * 2 + 2).map(|s| u16::from_be_bytes([s[0], s[1]]));
        for y in 0..h {
            let row = &rows[y * rb..(y + 1) * rb];
            for x in 0..w {
                let (px, a) = match self.color {
                    0 => {
                        let (g, raw) = sample(row, x, 0);
                        ([g, g, g], if trns16(0) == Some(raw) { 0 } else { 255 })
                    }
                    2 => {
                        let (r, rr) = sample(row, x, 0);
                        let (g, rg) = sample(row, x, 1);
                        let (b, rbv) = sample(row, x, 2);
                        let transparent = trns16(0) == Some(rr) && trns16(1) == Some(rg) && trns16(2) == Some(rbv);
                        ([r, g, b], if transparent { 0 } else { 255 })
                    }
                    3 => {
                        let (i, _) = sample(row, x, 0);
                        let i = i as usize;
                        let c = self.palette.get(i * 3..i * 3 + 3).ok_or_else(bad)?;
                        ([c[0], c[1], c[2]], self.trns.get(i).copied().unwrap_or(255))
                    }
                    4 => {
                        let (g, _) = sample(row, x, 0);
                        ([g, g, g], sample(row, x, 1).0)
                    }
                    _ => ([sample(row, x, 0).0, sample(row, x, 1).0, sample(row, x, 2).0], sample(row, x, 3).0),
                };
                rgb.extend_from_slice(&px);
                alpha.push(a);
            }
        }
        let alpha = alpha.iter().any(|&a| a != 255).then_some(alpha);
        Ok((rgb, alpha))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Builds a PNG file from unfiltered rows (filter byte 0 is added to each row).
    pub fn make_png(w: u32, h: u32, depth: u8, color: u8, rows: &[u8], plte: &[u8], trns: &[u8]) -> Vec<u8> {
        let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let mut chunk = |kind: &[u8], data: &[u8]| {
            out.extend((data.len() as u32).to_be_bytes());
            let mut body = kind.to_vec();
            body.extend_from_slice(data);
            out.extend_from_slice(&body);
            out.extend(deflate::crc32(&body).to_be_bytes());
        };
        let mut ihdr = Vec::new();
        ihdr.extend(w.to_be_bytes());
        ihdr.extend(h.to_be_bytes());
        ihdr.extend([depth, color, 0, 0, 0]);
        chunk(b"IHDR", &ihdr);
        if !plte.is_empty() {
            chunk(b"PLTE", plte);
        }
        if !trns.is_empty() {
            chunk(b"tRNS", trns);
        }
        let rb = rows.len() / h as usize;
        let mut filtered = Vec::new();
        for r in rows.chunks(rb) {
            filtered.push(0);
            filtered.extend_from_slice(r);
        }
        chunk(b"IDAT", &deflate::zlib_compress(&filtered));
        chunk(b"IEND", &[]);
        out
    }

    /// A 16 by 12 RGBA picture saved by Pillow (zlib level 9, dynamic Huffman codes, PNG filters).
    pub const PILLOW_RGBA: [u8; 183] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x0c, 0x08,
        0x06, 0x00, 0x00, 0x00, 0x6b, 0xe7, 0x3d, 0x81, 0x00, 0x00, 0x00, 0x7e, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x9d, 0xd0, 0xa1, 0x0a, 0xc3, 0x30, 0x10,
        0x80, 0xe1, 0x7f, 0x30, 0xa8, 0x1b, 0x15, 0xab, 0xab, 0x9b, 0x9b, 0x9b, 0x3b, 0x17, 0x17, 0x97, 0x27, 0x88, 0xab, 0xcb, 0x63, 0xf4, 0xc9, 0x3b, 0x13,
        0xc2, 0x71, 0x24, 0x69, 0x5a, 0xf1, 0xc1, 0x35, 0x09, 0x3f, 0x47, 0x1f, 0xc0, 0x31, 0x03, 0x1d, 0x7b, 0xef, 0xfe, 0xc9, 0x1b, 0x60, 0xba, 0x4d, 0x05,
        0x5e, 0xb7, 0x98, 0xc0, 0x72, 0x59, 0x25, 0xb0, 0x0e, 0x88, 0x65, 0x6e, 0x04, 0x3e, 0x15, 0xbe, 0x7a, 0xde, 0x09, 0x7c, 0x33, 0x51, 0xb3, 0x95, 0xce,
        0x02, 0xbf, 0x86, 0x58, 0xe6, 0x4e, 0x40, 0xf2, 0x23, 0x51, 0x82, 0xf9, 0x96, 0x91, 0x0d, 0x04, 0x70, 0xf9, 0x1f, 0x38, 0x65, 0x03, 0x5c, 0x2b, 0xe0,
        0x4d, 0xc0, 0x1b, 0xb1, 0xcc, 0x03, 0x1b, 0x04, 0xb5, 0x41, 0x30, 0x12, 0x7f, 0xd9, 0x38, 0x1a, 0x1e, 0x5a, 0x65, 0xdb, 0xfa, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    /// The smallest JPEG header with a frame: SOI, an APP0 segment, SOF0 (3 by 2, three components).
    pub fn tiny_jpeg_header() -> Vec<u8> {
        let mut b = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00];
        b.extend([0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x03, 0x03]);
        b.extend([0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00]);
        b.extend([0xFF, 0xD9]);
        b
    }

    #[test]
    fn reads_the_size_of_every_kind() {
        assert_eq!(dimensions(Kind::Png, &PILLOW_RGBA), Some((16, 12)));
        assert_eq!(dimensions(Kind::Jpeg, &tiny_jpeg_header()), Some((3, 2)));
        let gif = [b'G', b'I', b'F', b'8', b'9', b'a', 0x40, 0x01, 0xF0, 0x00, 0, 0, 0];
        assert_eq!(dimensions(Kind::Gif, &gif), Some((320, 240)));
        let mut webp = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        webp.extend([0u8; 8]);
        webp.extend([0x1F, 0x00, 0x00, 0x0F, 0x00, 0x00]); // 32 by 16
        assert_eq!(dimensions(Kind::Webp, &webp), Some((32, 16)));
        assert_eq!(dimensions(Kind::Png, b"\x89PNG"), None);
    }

    #[test]
    fn decodes_a_png_written_by_another_program() {
        let p = png_parts(&PILLOW_RGBA).unwrap();
        assert!(!p.pdf_passthrough(), "RGBA needs decoding");
        let (rgb, alpha) = p.decode_rgb().unwrap();
        assert_eq!(rgb.len(), 16 * 12 * 3);
        let at = |x: usize, y: usize| (y * 16 + x) * 3;
        // Pillow reports (48, 100, 105, 255) at (3, 5) and (192, 140, 76, 128) at (12, 7).
        assert_eq!(&rgb[at(3, 5)..at(3, 5) + 3], &[48, 100, 105]);
        assert_eq!(&rgb[at(12, 7)..at(12, 7) + 3], &[192, 140, 76]);
        let alpha = alpha.expect("half the picture is see-through");
        assert_eq!(alpha[5 * 16 + 3], 255);
        assert_eq!(alpha[7 * 16 + 12], 128);
    }

    #[test]
    fn decodes_palette_gray_and_transparency() {
        // 2-bit palette, 3 by 1 pixels: indexes 0, 1, 2; index 1 is see-through.
        let png = make_png(3, 1, 2, 3, &[0b0001_1000], &[255, 0, 0, 0, 255, 0, 0, 0, 255], &[255, 0]);
        let p = png_parts(&png).unwrap();
        assert!(!p.pdf_passthrough(), "tRNS needs decoding");
        let (rgb, alpha) = p.decode_rgb().unwrap();
        assert_eq!(rgb, vec![255, 0, 0, 0, 255, 0, 0, 0, 255]);
        assert_eq!(alpha, Some(vec![255, 0, 255]));

        let gray = make_png(2, 2, 8, 0, &[0, 255, 128, 64], &[], &[]);
        let p = png_parts(&gray).unwrap();
        assert!(p.pdf_passthrough());
        let (rgb, alpha) = p.decode_rgb().unwrap();
        assert_eq!(rgb, vec![0, 0, 0, 255, 255, 255, 128, 128, 128, 64, 64, 64]);
        assert_eq!(alpha, None, "opaque pictures get no alpha plane");
    }

    #[test]
    fn refuses_broken_pngs() {
        assert!(png_parts(b"not a png").is_err());
        let mut png = make_png(2, 2, 8, 2, &[0; 12], &[], &[]);
        // Cut the picture data short.
        let at = png.windows(4).position(|w| w == b"IDAT").unwrap();
        png[at + 6] ^= 0xFF;
        assert!(png_parts(&png).and_then(|p| p.decode_rgb()).is_err());
    }
}
