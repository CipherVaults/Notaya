//! A small TrueType reader and subsetter, written for the PDF export (no font library is
//! linked). It reads fonts with `glyf` outlines (single fonts and `.ttc` collections), looks up
//! characters through the `cmap` (formats 4 and 12), gives advances and bounds, and writes a
//! subset that keeps only the glyphs the PDF uses. Glyph numbers are kept (unused glyphs are
//! empty and the tables are cut after the last used one), so the PDF can use Identity-H with
//! CIDToGIDMap Identity. Hinting instructions are dropped. Fonts with PostScript (CFF)
//! outlines, and fonts whose licence bits forbid embedding or subsetting, are refused.

use std::borrow::Cow;
use std::collections::BTreeSet;

/// Bounds-checked big-endian reads.
fn u16_at(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes(d.get(o..o.checked_add(2)?)?.try_into().ok()?))
}
fn i16_at(d: &[u8], o: usize) -> Option<i16> {
    u16_at(d, o).map(|v| v as i16)
}
fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(d.get(o..o.checked_add(4)?)?.try_into().ok()?))
}

#[derive(Clone, Copy, Default)]
struct Table {
    off: usize,
    len: usize,
}

pub struct Face {
    data: Cow<'static, [u8]>,
    tables: Vec<([u8; 4], Table)>,
    pub units: u16,
    pub num_glyphs: u16,
    long_loca: bool,
    num_hmetrics: u16,
    /// Ascent, descent, cap height and the bounding box, in 1/1000 of the font size.
    pub ascent: i32,
    pub descent: i32,
    pub cap_height: i32,
    pub bbox: [i32; 4],
    pub italic_angle: f32,
    /// Offset of the chosen cmap subtable in the file, and its format.
    sub: (usize, u16),
}

impl Face {
    /// Reads font number `index` of a font file. `None` when the file is not a usable TrueType
    /// font (CFF outlines, broken tables, or a licence that forbids embedding).
    pub fn parse(data: impl Into<Cow<'static, [u8]>>, index: u32) -> Option<Face> {
        let data: Cow<'static, [u8]> = data.into();
        let d: &[u8] = &data;
        let base = if d.get(0..4)? == b"ttcf" {
            if index >= u32_at(d, 8)? {
                return None;
            }
            u32_at(d, 12 + 4 * index as usize)? as usize
        } else {
            0
        };
        if !matches!(u32_at(d, base)?, 0x0001_0000 | 0x7472_7565) {
            return None; // 'OTTO' (CFF), 'wOFF' and the rest are not read
        }
        let n = u16_at(d, base + 4)? as usize;
        let mut tables = Vec::with_capacity(n);
        for i in 0..n {
            let e = base + 12 + 16 * i;
            let tag: [u8; 4] = d.get(e..e + 4)?.try_into().ok()?;
            let off = u32_at(d, e + 8)? as usize;
            let len = u32_at(d, e + 12)? as usize;
            if off.checked_add(len)? > d.len() {
                return None;
            }
            tables.push((tag, Table { off, len }));
        }
        let find = |tag: &[u8; 4]| tables.iter().find(|(t, _)| t == tag).map(|(_, t)| *t);
        let head = find(b"head")?;
        let hhea = find(b"hhea")?;
        let maxp = find(b"maxp")?;
        find(b"glyf")?;
        find(b"loca")?;
        find(b"hmtx")?;
        let cmap = find(b"cmap");
        if head.len < 54 || hhea.len < 36 || maxp.len < 6 {
            return None;
        }
        let units = u16_at(d, head.off + 18)?;
        if units < 16 {
            return None;
        }
        let scale = |v: i32| (v as f32 * 1000.0 / units as f32).round() as i32;
        let mut face = Face {
            units,
            num_glyphs: u16_at(d, maxp.off + 4)?,
            long_loca: i16_at(d, head.off + 50)? != 0,
            num_hmetrics: u16_at(d, hhea.off + 34)?,
            ascent: scale(i16_at(d, hhea.off + 4)? as i32),
            descent: scale(i16_at(d, hhea.off + 6)? as i32),
            cap_height: 0,
            bbox: [scale(i16_at(d, head.off + 36)? as i32), scale(i16_at(d, head.off + 38)? as i32), scale(i16_at(d, head.off + 40)? as i32), scale(i16_at(d, head.off + 42)? as i32)],
            italic_angle: 0.0,
            sub: (0, 0),
            data: Cow::Borrowed(&[]),
            tables: Vec::new(),
        };
        face.cap_height = face.ascent * 7 / 10;
        if let Some(os2) = find(b"OS/2") {
            if os2.len >= 10 {
                let fs = u16_at(d, os2.off + 8)?;
                // Bit 1: restricted licence; bit 8: no subsetting; bit 9: bitmaps only.
                if fs & 0x0002 != 0 || fs & 0x0100 != 0 || fs & 0x0200 != 0 {
                    return None;
                }
            }
            if os2.len >= 90 && u16_at(d, os2.off)? >= 2 {
                let cap = i16_at(d, os2.off + 88)? as i32;
                if cap > 0 {
                    face.cap_height = scale(cap);
                }
            }
        }
        if let Some(post) = find(b"post") {
            if post.len >= 8 {
                face.italic_angle = i16_at(d, post.off + 4)? as f32;
            }
        }
        // The best character table: Unicode full range, then Unicode BMP (a subset has none).
        if let Some(cmap) = cmap {
            let cm = cmap.off;
            let count = u16_at(d, cm + 2)? as usize;
            let mut best: Option<(u32, usize, u16)> = None;
            for i in 0..count {
                let (plat, enc) = (u16_at(d, cm + 4 + 8 * i)?, u16_at(d, cm + 6 + 8 * i)?);
                let off = cm + u32_at(d, cm + 8 + 8 * i)? as usize;
                let fmt = u16_at(d, off)?;
                let rank = match (plat, enc, fmt) {
                    (3, 10, 12) => 4,
                    (0, _, 12) => 3,
                    (3, 1, 4) => 2,
                    (0, _, 4) => 1,
                    _ => 0,
                };
                if rank > best.map_or(0, |b| b.0) {
                    best = Some((rank, off, fmt));
                }
            }
            if let Some((_, off, fmt)) = best {
                face.sub = (off, fmt);
            }
        }
        face.data = data;
        face.tables = tables;
        Some(face)
    }

    fn table(&self, tag: &[u8; 4]) -> Option<Table> {
        self.tables.iter().find(|(t, _)| t == tag).map(|(_, t)| *t)
    }

    /// The glyph for a character, or 0 when the font has none.
    pub fn glyph(&self, c: char) -> u16 {
        self.lookup(c as u32).unwrap_or(0)
    }

    fn lookup(&self, c: u32) -> Option<u16> {
        let d: &[u8] = &self.data;
        let (off, fmt) = self.sub;
        match fmt {
            12 => {
                let n = u32_at(d, off + 12)? as usize;
                let (mut lo, mut hi) = (0, n);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    let g = off + 16 + 12 * mid;
                    let (start, end) = (u32_at(d, g)?, u32_at(d, g + 4)?);
                    if c < start {
                        hi = mid;
                    } else if c > end {
                        lo = mid + 1;
                    } else {
                        let gid = u32_at(d, g + 8)? + (c - start);
                        return u16::try_from(gid).ok();
                    }
                }
                None
            }
            4 => {
                if c > 0xFFFF {
                    return None;
                }
                let segs = (u16_at(d, off + 6)? / 2) as usize;
                let (ends, starts, deltas, ranges) = (off + 14, off + 16 + 2 * segs, off + 16 + 4 * segs, off + 16 + 6 * segs);
                let (mut lo, mut hi) = (0, segs);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    let end = u16_at(d, ends + 2 * mid)? as u32;
                    if c > end {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo >= segs {
                    return None;
                }
                let start = u16_at(d, starts + 2 * lo)? as u32;
                if c < start {
                    return None;
                }
                let delta = u16_at(d, deltas + 2 * lo)?;
                let range = u16_at(d, ranges + 2 * lo)? as usize;
                if range == 0 {
                    return Some((c as u16).wrapping_add(delta));
                }
                let g = u16_at(d, ranges + 2 * lo + range + 2 * (c - start) as usize)?;
                (g != 0).then_some(g.wrapping_add(delta))
            }
            _ => None,
        }
    }

    /// Advance width of a glyph in 1/1000 of the font size.
    pub fn advance(&self, gid: u16) -> u32 {
        let units = self.advance_units(gid) as u32;
        (units * 1000 + self.units as u32 / 2) / self.units as u32
    }

    fn advance_units(&self, gid: u16) -> u16 {
        let (Some(t), d) = (self.table(b"hmtx"), &*self.data) else { return 0 };
        let i = gid.min(self.num_hmetrics.saturating_sub(1)) as usize;
        u16_at(d, t.off + 4 * i).unwrap_or(0)
    }

    fn lsb(&self, gid: u16) -> i16 {
        let (Some(t), d) = (self.table(b"hmtx"), &*self.data) else { return 0 };
        let n = self.num_hmetrics as usize;
        let g = gid as usize;
        if g < n { i16_at(d, t.off + 4 * g + 2) } else { i16_at(d, t.off + 4 * n + 2 * (g - n)) }.unwrap_or(0)
    }

    fn glyph_data(&self, gid: u16) -> &[u8] {
        let (Some(loca), Some(glyf)) = (self.table(b"loca"), self.table(b"glyf")) else { return &[] };
        let d: &[u8] = &self.data;
        let g = gid as usize;
        let (a, b) = if self.long_loca {
            (u32_at(d, loca.off + 4 * g), u32_at(d, loca.off + 4 * g + 4))
        } else {
            (u16_at(d, loca.off + 2 * g).map(|v| v as u32 * 2), u16_at(d, loca.off + 2 * g + 2).map(|v| v as u32 * 2))
        };
        match (a, b) {
            (Some(a), Some(b)) if b > a && (b as usize) <= glyf.len => &d[glyf.off + a as usize..glyf.off + b as usize],
            _ => &[],
        }
    }

    /// Horizontal extent of a glyph's outline (xMin, xMax) in 1/1000 of the font size, if it has one.
    pub fn bounds(&self, gid: u16) -> Option<(i32, i32)> {
        let g = self.glyph_data(gid);
        let s = |v: i16| (v as i32 * 1000) / self.units as i32;
        Some((s(i16_at(g, 2)?), s(i16_at(g, 6)?)))
    }

    /// The PostScript name (name ID 6), reduced to letters, digits and hyphens.
    pub fn postscript_name(&self) -> Option<String> {
        let t = self.table(b"name")?;
        let d: &[u8] = &self.data;
        let count = u16_at(d, t.off + 2)? as usize;
        let strings = t.off + u16_at(d, t.off + 4)? as usize;
        for i in 0..count {
            let r = t.off + 6 + 12 * i;
            if u16_at(d, r + 6)? != 6 {
                continue;
            }
            let (plat, len, off) = (u16_at(d, r)?, u16_at(d, r + 8)? as usize, u16_at(d, r + 10)? as usize);
            let raw = d.get(strings + off..strings + off + len)?;
            let name: String = if plat == 3 || plat == 0 {
                raw.chunks(2).filter(|c| c.len() == 2 && c[0] == 0).map(|c| c[1] as char).collect()
            } else {
                raw.iter().map(|&b| b as char).collect()
            };
            let clean: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
        None
    }

    /// The glyphs a composite glyph is made of.
    fn components(g: &[u8]) -> Option<Vec<(usize, u16)>> {
        // (offset of the component record, glyph index)
        let mut out = Vec::new();
        if i16_at(g, 0)? >= 0 {
            return Some(out);
        }
        let mut p = 10;
        loop {
            let flags = u16_at(g, p)?;
            out.push((p, u16_at(g, p + 2)?));
            p += 4 + if flags & 0x0001 != 0 { 4 } else { 2 };
            p += if flags & 0x0008 != 0 { 2 } else if flags & 0x0040 != 0 { 4 } else if flags & 0x0080 != 0 { 8 } else { 0 };
            if flags & 0x0020 == 0 {
                return Some(out);
            }
        }
    }

    /// A glyph without its hinting instructions.
    fn unhinted(g: &[u8]) -> Vec<u8> {
        let strip = || -> Option<Vec<u8>> {
            let nc = i16_at(g, 0)?;
            if nc >= 0 {
                let head = 10 + 2 * nc as usize;
                let len = u16_at(g, head)? as usize;
                let mut v = g.get(..head)?.to_vec();
                v.extend([0, 0]);
                v.extend_from_slice(g.get(head + 2 + len..)?);
                Some(v)
            } else {
                let comps = Face::components(g)?;
                let (last, _) = *comps.last()?;
                let mut v = g.to_vec();
                for (p, _) in &comps {
                    let f = u16_at(g, *p)? & !0x0100;
                    v[*p..*p + 2].copy_from_slice(&f.to_be_bytes());
                }
                // The instructions (if any) follow the last component.
                let flags = u16_at(g, last)?;
                let mut end = last + 4 + if flags & 0x0001 != 0 { 4 } else { 2 };
                end += if flags & 0x0008 != 0 { 2 } else if flags & 0x0040 != 0 { 4 } else if flags & 0x0080 != 0 { 8 } else { 0 };
                v.truncate(end);
                Some(v)
            }
        };
        strip().unwrap_or_default()
    }

    /// A font file with only these glyphs (plus glyph 0 and what composite glyphs need). The
    /// glyph numbers stay the same.
    pub fn subset(&self, gids: &BTreeSet<u16>) -> Option<Vec<u8>> {
        let mut keep: BTreeSet<u16> = gids.iter().copied().filter(|g| *g < self.num_glyphs).collect();
        keep.insert(0);
        let mut todo: Vec<u16> = keep.iter().copied().collect();
        while let Some(g) = todo.pop() {
            for (_, c) in Face::components(self.glyph_data(g)).unwrap_or_default() {
                if c < self.num_glyphs && keep.insert(c) {
                    todo.push(c);
                }
            }
        }
        let count = *keep.iter().next_back()? as usize + 1;
        let (mut glyf, mut loca, mut hmtx) = (Vec::new(), Vec::with_capacity(4 * (count + 1)), Vec::with_capacity(4 * count));
        for g in 0..count as u16 {
            loca.extend((glyf.len() as u32).to_be_bytes());
            if keep.contains(&g) {
                let raw = self.glyph_data(g);
                if !raw.is_empty() {
                    glyf.extend(Face::unhinted(raw));
                    while glyf.len() % 4 != 0 {
                        glyf.push(0);
                    }
                }
                hmtx.extend(self.advance_units(g).to_be_bytes());
                hmtx.extend(self.lsb(g).to_be_bytes());
            } else {
                hmtx.extend([0, 0, 0, 0]);
            }
        }
        loca.extend((glyf.len() as u32).to_be_bytes());
        let d: &[u8] = &self.data;
        let copy = |tag: &[u8; 4], len: usize| -> Option<Vec<u8>> {
            let t = self.table(tag)?;
            Some(d.get(t.off..t.off + len.min(t.len))?.to_vec())
        };
        let mut head = copy(b"head", 54)?;
        head[8..12].fill(0); // checkSumAdjustment, filled in below
        head[50..52].copy_from_slice(&1i16.to_be_bytes());
        let mut hhea = copy(b"hhea", 36)?;
        hhea[34..36].copy_from_slice(&(count as u16).to_be_bytes());
        let mut maxp = copy(b"maxp", 32)?;
        maxp[4..6].copy_from_slice(&(count as u16).to_be_bytes());
        let parts: Vec<(&[u8; 4], Vec<u8>)> = vec![(b"glyf", glyf), (b"head", head), (b"hhea", hhea), (b"hmtx", hmtx), (b"loca", loca), (b"maxp", maxp)];
        // Table directory.
        let n = parts.len();
        let mut out = Vec::new();
        out.extend(0x0001_0000u32.to_be_bytes());
        out.extend((n as u16).to_be_bytes());
        let mut pow = 1u16;
        let mut log = 0u16;
        while pow * 2 <= n as u16 {
            pow *= 2;
            log += 1;
        }
        out.extend((pow * 16).to_be_bytes());
        out.extend(log.to_be_bytes());
        out.extend((n as u16 * 16 - pow * 16).to_be_bytes());
        let mut offset = 12 + 16 * n;
        let mut head_at = 0;
        for (tag, body) in &parts {
            let sum = checksum(body);
            out.extend_from_slice(*tag);
            out.extend(sum.to_be_bytes());
            out.extend((offset as u32).to_be_bytes());
            out.extend((body.len() as u32).to_be_bytes());
            if *tag == b"head" {
                head_at = offset;
            }
            offset += body.len().div_ceil(4) * 4;
        }
        for (_, body) in &parts {
            out.extend_from_slice(body);
            while out.len() % 4 != 0 {
                out.push(0);
            }
        }
        let adjust = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        out[head_at + 8..head_at + 12].copy_from_slice(&adjust.to_be_bytes());
        Some(out)
    }
}

fn checksum(d: &[u8]) -> u32 {
    d.chunks(4).fold(0u32, |s, c| {
        let mut w = [0u8; 4];
        w[..c.len()].copy_from_slice(c);
        s.wrapping_add(u32::from_be_bytes(w))
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn be16(v: &mut Vec<u8>, x: u16) {
        v.extend(x.to_be_bytes());
    }

    /// A simple glyph: one contour of on-curve points.
    fn simple(points: &[(i16, i16)], instructions: &[u8]) -> Vec<u8> {
        let (xmin, xmax) = (points.iter().map(|p| p.0).min().unwrap(), points.iter().map(|p| p.0).max().unwrap());
        let (ymin, ymax) = (points.iter().map(|p| p.1).min().unwrap(), points.iter().map(|p| p.1).max().unwrap());
        let mut g = Vec::new();
        for v in [1i16, xmin, ymin, xmax, ymax, points.len() as i16 - 1] {
            be16(&mut g, v as u16);
        }
        be16(&mut g, instructions.len() as u16);
        g.extend_from_slice(instructions);
        g.extend(std::iter::repeat_n(1u8, points.len())); // on-curve, 16-bit deltas
        let mut prev = 0;
        for p in points {
            be16(&mut g, (p.0 - prev) as u16);
            prev = p.0;
        }
        prev = 0;
        for p in points {
            be16(&mut g, (p.1 - prev) as u16);
            prev = p.1;
        }
        g
    }

    /// A composite glyph: glyph `base` moved by (dx, dy), with instructions after it.
    fn composite(base: u16, dx: i16, dy: i16) -> Vec<u8> {
        let mut g = Vec::new();
        for v in [-1i16, 0, 0, 1000, 1000] {
            be16(&mut g, v as u16);
        }
        be16(&mut g, 0x0001 | 0x0002 | 0x0100); // words, xy values, instructions follow
        be16(&mut g, base);
        be16(&mut g, dx as u16);
        be16(&mut g, dy as u16);
        be16(&mut g, 2);
        g.extend([0xB0, 0x00]);
        g
    }

    /// A tiny font with 1000 units per em. Glyphs: 0 box, 1 "A" (with instructions), 2 "B"
    /// (composite of 1), 3 U+4E2D, 4 U+3042, 5 U+AC00 (all squares), 6 punctuation (U+3001, U+3002, U+FF08). fsType is given.
    pub fn test_font(fs_type: u16) -> Vec<u8> {
        build_font(fs_type, true)
    }

    /// `format12` false gives a font whose only character table is the BMP one (format 4).
    pub fn build_font(fs_type: u16, format12: bool) -> Vec<u8> {
        let square = |s: i16| simple(&[(100, 0), (100 + s, 0), (100 + s, s), (100, s)], &[0xB0, 0x01]);
        let glyphs = vec![square(500), simple(&[(50, 0), (350, 0), (200, 700)], &[0xB0, 0x07, 0x2B, 1, 2, 3, 4]), composite(1, 400, 0), square(900), square(800), square(700), square(300)];
        let mut glyf = Vec::new();
        let mut loca = Vec::new();
        for g in &glyphs {
            be16(&mut loca, (glyf.len() / 2) as u16);
            glyf.extend_from_slice(g);
            if glyf.len() % 2 != 0 {
                glyf.push(0);
            }
        }
        be16(&mut loca, (glyf.len() / 2) as u16);
        let advances = [600u16, 400, 800, 1000, 1000, 1000, 1000];
        let mut hmtx = Vec::new();
        for a in advances {
            be16(&mut hmtx, a);
            be16(&mut hmtx, 50);
        }
        let mut head = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0x5F, 0x0F, 0x3C, 0xF5, 0, 0];
        be16(&mut head, 1000); // unitsPerEm at 18
        head.extend([0u8; 16]); // created, modified
        for v in [0i16, -200, 1000, 900] {
            be16(&mut head, v as u16);
        }
        head.extend([0u8; 6]); // macStyle, lowestRecPPEM, fontDirectionHint
        be16(&mut head, 0); // indexToLocFormat: short
        be16(&mut head, 0);
        let mut hhea = vec![0, 1, 0, 0];
        for v in [800i16, -200, 0] {
            be16(&mut hhea, v as u16);
        }
        hhea.extend([0u8; 24]);
        be16(&mut hhea, 7);
        let mut maxp = vec![0, 1, 0, 0];
        be16(&mut maxp, 7);
        maxp.extend([0u8; 26]);
        let mut os2 = vec![0u8; 96];
        os2[1] = 2; // version 2
        os2[8..10].copy_from_slice(&fs_type.to_be_bytes());
        os2[88..90].copy_from_slice(&700u16.to_be_bytes());
        // cmap: format 4 for 'A', 'B' (and the end mark) and format 12 for the rest.
        let mut f4 = Vec::new();
        for v in [4u16, 0, 0, 4, 4, 1, 0, 0x42, 0xFFFF, 0, 0x41, 0xFFFF, 0xFFC0, 1, 0, 0] {
            be16(&mut f4, v);
        }
        let len = f4.len() as u16;
        f4[2..4].copy_from_slice(&len.to_be_bytes());
        let mut f12 = Vec::new();
        f12.extend([0, 12, 0, 0]);
        f12.extend((16u32 + 84).to_be_bytes());
        f12.extend(0u32.to_be_bytes());
        f12.extend(7u32.to_be_bytes());
        for (s, e, g) in [(0x41u32, 0x42u32, 1u32), (0x3001, 0x3001, 6), (0x3002, 0x3002, 6), (0x3042, 0x3042, 4), (0x4E2D, 0x4E2D, 3), (0xAC00, 0xAC00, 5), (0xFF08, 0xFF08, 6)] {
            f12.extend(s.to_be_bytes());
            f12.extend(e.to_be_bytes());
            f12.extend(g.to_be_bytes());
        }
        let mut cmap = if format12 { vec![0, 0, 0, 2, 0, 3, 0, 1, 0, 0, 0, 20, 0, 3, 0, 10] } else { vec![0, 0, 0, 1, 0, 3, 0, 1, 0, 0, 0, 12] };
        if format12 {
            cmap.extend((20 + f4.len() as u32).to_be_bytes());
        }
        cmap.extend(f4);
        if format12 {
            cmap.extend(f12);
        }
        let mut name = vec![0, 0, 0, 1, 0, 18, 0, 3, 0, 1, 4, 9, 0, 6, 0, 18, 0, 0];
        for ch in "Test-Font".chars() {
            name.extend([0, ch as u8]);
        }
        let tables: Vec<(&[u8; 4], Vec<u8>)> = vec![(b"OS/2", os2), (b"cmap", cmap), (b"glyf", glyf), (b"head", head), (b"hhea", hhea), (b"hmtx", hmtx), (b"loca", loca), (b"maxp", maxp), (b"name", name)];
        let mut out = vec![0, 1, 0, 0];
        be16(&mut out, tables.len() as u16);
        out.extend([0u8; 6]);
        let mut offset = 12 + 16 * tables.len();
        for (tag, body) in &tables {
            out.extend_from_slice(*tag);
            out.extend(checksum(body).to_be_bytes());
            out.extend((offset as u32).to_be_bytes());
            out.extend((body.len() as u32).to_be_bytes());
            offset += body.len().div_ceil(4) * 4;
        }
        for (_, body) in &tables {
            out.extend_from_slice(body);
            while out.len() % 4 != 0 {
                out.push(0);
            }
        }
        out
    }

    #[test]
    fn reads_the_test_font() {
        let f = Face::parse(test_font(0), 0).unwrap();
        assert_eq!((f.units, f.num_glyphs), (1000, 7));
        assert_eq!((f.glyph('A'), f.glyph('B'), f.glyph('C')), (1, 2, 0));
        assert_eq!((f.glyph('中'), f.glyph('あ'), f.glyph('가'), f.glyph('\u{1F600}')), (3, 4, 5, 0));
        let bmp = Face::parse(build_font(0, false), 0).unwrap();
        assert_eq!((bmp.glyph('A'), bmp.glyph('B'), bmp.glyph('C'), bmp.glyph('中')), (1, 2, 0, 0), "format 4 alone");
        assert_eq!((f.advance(1), f.advance(3), f.advance(6)), (400, 1000, 1000));
        assert_eq!(f.bounds(3), Some((100, 1000)));
        assert_eq!((f.ascent, f.descent, f.cap_height), (800, -200, 700));
        assert_eq!(f.postscript_name().as_deref(), Some("Test-Font"));
    }

    #[test]
    fn fonts_that_forbid_embedding_or_subsetting_are_refused() {
        assert!(Face::parse(test_font(0x0008), 0).is_some(), "editable embedding is allowed");
        assert!(Face::parse(test_font(0x0004), 0).is_some(), "print and preview is allowed");
        assert!(Face::parse(test_font(0x0002), 0).is_none(), "restricted licence");
        assert!(Face::parse(test_font(0x0100), 0).is_none(), "no subsetting");
        assert!(Face::parse(test_font(0x0200), 0).is_none(), "bitmap embedding only");
        let mut cff = test_font(0);
        cff[0..4].copy_from_slice(b"OTTO");
        assert!(Face::parse(cff, 0).is_none(), "CFF outlines are not read");
        assert!(Face::parse(vec![1, 2, 3], 0).is_none());
    }

    #[test]
    fn a_subset_keeps_the_used_glyphs_and_their_components() {
        let f = Face::parse(test_font(0), 0).unwrap();
        let sub = f.subset(&BTreeSet::from([2u16, 3])).unwrap();
        let s = Face::parse(sub.clone(), 0).unwrap();
        assert_eq!(s.num_glyphs, 4, "cut after the last used glyph");
        assert!(s.glyph_data(0).len() > 0 && s.glyph_data(1).len() > 0, "glyph 0 and the component of the composite stay");
        assert!(s.glyph_data(2).len() > 0 && s.glyph_data(3).len() > 0);
        assert_eq!(s.advance(2), 800);
        assert_eq!(s.advance(3), 1000);
        // Hinting is gone: the simple glyph has no instructions and is shorter.
        assert!(s.glyph_data(1).len() < f.glyph_data(1).len());
        assert_eq!(u16_at(s.glyph_data(1), 10 + 2), Some(0));
        // The composite no longer says that instructions follow.
        let comp = s.glyph_data(2);
        assert_eq!(u16_at(comp, 10).unwrap() & 0x0100, 0);
        assert!(comp.len() <= 20, "the instructions after the component are cut off");
        // Table directory and checksums: the whole file sums to the magic number.
        assert_eq!(checksum(&sub), 0xB1B0_AFBA);
        let unused = f.subset(&BTreeSet::from([6u16])).unwrap();
        let u = Face::parse(unused, 0).unwrap();
        assert_eq!(u.num_glyphs, 7);
        assert!(u.glyph_data(5).is_empty(), "glyphs that are not used are empty");
    }

    #[test]
    fn collections_are_read_by_index() {
        let font = test_font(0);
        let mut ttc = b"ttcf".to_vec();
        ttc.extend([0, 1, 0, 0]);
        ttc.extend(2u32.to_be_bytes());
        let first = 20u32;
        ttc.extend(first.to_be_bytes());
        ttc.extend(first.to_be_bytes());
        // The table offsets of the font are relative to the file start, so shift them.
        let mut shifted = font.clone();
        let n = u16_at(&font, 4).unwrap() as usize;
        for i in 0..n {
            let o = 12 + 16 * i + 8;
            let v = u32_at(&font, o).unwrap() + first;
            shifted[o..o + 4].copy_from_slice(&v.to_be_bytes());
        }
        ttc.extend(shifted);
        assert!(Face::parse(ttc.clone(), 1).is_some());
        assert!(Face::parse(ttc, 2).is_none());
    }
}
