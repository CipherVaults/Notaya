//! Small, dependency-free compression helpers for the exports: CRC-32 (ZIP), Adler-32 (zlib),
//! a DEFLATE decoder (to read PNG pictures for the PDF) and a simple DEFLATE encoder (LZ77 with
//! the fixed Huffman codes of RFC 1951, used for the PDF streams and the DOCX parts).
//!
//! The encoder favours simple, obviously correct code over the best ratio. Its output is checked
//! against the decoder in the tests, and by Python's zlib and the PDF tools when samples are made.

use crate::error::{Error, Result};

/// CRC-32 (IEEE 802.3), as used by ZIP.
pub fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, slot) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *slot = c;
        }
        t
    });
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// Adler-32, the checksum at the end of a zlib stream.
pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

/* ---------- Encoder ---------- */

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    fn new(capacity: usize) -> Self {
        BitWriter { out: Vec::with_capacity(capacity), acc: 0, n: 0 }
    }
    /// Writes the low `count` bits of `value`, least significant bit first.
    fn bits(&mut self, value: u32, count: u32) {
        self.acc |= (value as u64) << self.n;
        self.n += count;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }
    /// Writes a Huffman code, which DEFLATE stores most significant bit first.
    fn code(&mut self, code: u32, len: u32) {
        let reversed = code.reverse_bits() >> (32 - len);
        self.bits(reversed, len);
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push(self.acc as u8);
        }
        self.out
    }
}

fn fixed_literal(w: &mut BitWriter, sym: u32) {
    match sym {
        0..=143 => w.code(0x30 + sym, 8),
        144..=255 => w.code(0x190 + (sym - 144), 9),
        256..=279 => w.code(sym - 256, 7),
        _ => w.code(0xC0 + (sym - 280), 8),
    }
}

fn write_match(w: &mut BitWriter, len: usize, dist: usize) {
    let li = LEN_BASE.iter().rposition(|&b| b as usize <= len).unwrap_or(0);
    fixed_literal(w, 257 + li as u32);
    w.bits((len - LEN_BASE[li] as usize) as u32, LEN_EXTRA[li] as u32);
    let di = DIST_BASE.iter().rposition(|&b| b as usize <= dist).unwrap_or(0);
    w.code(di as u32, 5);
    w.bits((dist - DIST_BASE[di] as usize) as u32, DIST_EXTRA[di] as u32);
}

/// Raw DEFLATE (RFC 1951): one block with the fixed codes, LZ77 matches found with a hash chain.
pub fn deflate_raw(data: &[u8]) -> Vec<u8> {
    const WINDOW: usize = 32 * 1024;
    const HASH_BITS: u32 = 15;
    const MAX_CHAIN: usize = 48;
    let mut w = BitWriter::new(data.len() / 2 + 64);
    w.bits(1, 1); // last block
    w.bits(1, 2); // fixed Huffman codes
    let mut head = vec![usize::MAX; 1 << HASH_BITS];
    let mut prev = vec![usize::MAX; WINDOW];
    let hash = |i: usize| -> usize {
        let v = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8 | data[i + 2] as u32;
        (v.wrapping_mul(2_654_435_761) >> (32 - HASH_BITS)) as usize
    };
    let insert = |i: usize, head: &mut Vec<usize>, prev: &mut Vec<usize>| {
        if i + 3 <= data.len() {
            let h = hash(i);
            prev[i % WINDOW] = head[h];
            head[h] = i;
        }
    };
    let mut i = 0;
    while i < data.len() {
        let mut best_len = 0;
        let mut best_dist = 0;
        if i + 3 <= data.len() {
            let h = hash(i);
            let mut cand = head[h];
            let max_len = (data.len() - i).min(258);
            let mut chain = 0;
            while cand != usize::MAX && i - cand <= WINDOW && chain < MAX_CHAIN {
                if cand < i {
                    let mut l = 0;
                    while l < max_len && data[cand + l] == data[i + l] {
                        l += 1;
                    }
                    if l > best_len {
                        best_len = l;
                        best_dist = i - cand;
                        if l == max_len {
                            break;
                        }
                    }
                }
                let next = prev[cand % WINDOW];
                if next == usize::MAX || next >= cand {
                    break;
                }
                cand = next;
                chain += 1;
            }
        }
        if best_len >= 3 {
            write_match(&mut w, best_len, best_dist);
            for k in i..i + best_len {
                insert(k, &mut head, &mut prev);
            }
            i += best_len;
        } else {
            fixed_literal(&mut w, data[i] as u32);
            insert(i, &mut head, &mut prev);
            i += 1;
        }
    }
    fixed_literal(&mut w, 256);
    w.finish()
}

/// A zlib stream (RFC 1950), as PDF's FlateDecode expects.
pub fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    out.extend(deflate_raw(data));
    out.extend(adler32(data).to_be_bytes());
    out
}

/* ---------- Decoder ---------- */

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    acc: u32,
    n: u32,
}

impl<'a> BitReader<'a> {
    fn bits(&mut self, count: u32) -> Result<u32> {
        while self.n < count {
            let b = *self.data.get(self.pos).ok_or_else(bad)?;
            self.pos += 1;
            self.acc |= (b as u32) << self.n;
            self.n += 8;
        }
        let v = if count == 0 { 0 } else { self.acc & ((1u32 << count) - 1) };
        self.acc = if count >= 32 { 0 } else { self.acc >> count };
        self.n -= count;
        Ok(v)
    }
    fn align(&mut self) {
        self.acc = 0;
        self.n = 0;
    }
}

fn bad() -> Error {
    Error::invalid("A picture could not be read (damaged compressed data).")
}

/// A canonical Huffman code: how many codes of each length, and the symbols in code order.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Huffman> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut offs = [0u16; 16];
        for len in 1..16 {
            offs[len] = offs[len - 1] + counts[len - 1];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, r: &mut BitReader) -> Result<u16> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= r.bits(1)? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                return self.symbols.get((index + (code - first)) as usize).copied().ok_or_else(bad);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(bad())
    }
}

/// Decodes raw DEFLATE data. Refuses to produce more than `max_out` bytes (a guard against
/// pictures that would expand without limit).
pub fn inflate_raw(data: &[u8], max_out: usize) -> Result<Vec<u8>> {
    let mut r = BitReader { data, pos: 0, acc: 0, n: 0 };
    let mut out: Vec<u8> = Vec::new();
    loop {
        let last = r.bits(1)?;
        match r.bits(2)? {
            0 => {
                r.align();
                let p = r.pos;
                let hdr = data.get(p..p + 4).ok_or_else(bad)?;
                let len = u16::from_le_bytes([hdr[0], hdr[1]]) as usize;
                let nlen = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;
                if len != (!nlen & 0xFFFF) {
                    return Err(bad());
                }
                let body = data.get(p + 4..p + 4 + len).ok_or_else(bad)?;
                if out.len() + len > max_out {
                    return Err(bad());
                }
                out.extend_from_slice(body);
                r.pos = p + 4 + len;
            }
            1 => {
                let mut lengths = [0u8; 288];
                lengths[..144].fill(8);
                lengths[144..256].fill(9);
                lengths[256..280].fill(7);
                lengths[280..].fill(8);
                let lit = Huffman::new(&lengths)?;
                let dist = Huffman::new(&[5u8; 30])?;
                inflate_block(&mut r, &mut out, &lit, &dist, max_out)?;
            }
            2 => {
                let hlit = r.bits(5)? as usize + 257;
                let hdist = r.bits(5)? as usize + 1;
                let hclen = r.bits(4)? as usize + 4;
                const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                let mut cl = [0u8; 19];
                for &o in ORDER.iter().take(hclen) {
                    cl[o] = r.bits(3)? as u8;
                }
                let clh = Huffman::new(&cl)?;
                let mut lengths = vec![0u8; hlit + hdist];
                let mut i = 0;
                while i < hlit + hdist {
                    let sym = clh.decode(&mut r)?;
                    let (val, rep) = match sym {
                        0..=15 => (sym as u8, 1),
                        16 => (*lengths.get(i.wrapping_sub(1)).ok_or_else(bad)?, 3 + r.bits(2)? as usize),
                        17 => (0, 3 + r.bits(3)? as usize),
                        _ => (0, 11 + r.bits(7)? as usize),
                    };
                    if i + rep > lengths.len() {
                        return Err(bad());
                    }
                    lengths[i..i + rep].fill(val);
                    i += rep;
                }
                let lit = Huffman::new(&lengths[..hlit])?;
                let dist = Huffman::new(&lengths[hlit..])?;
                inflate_block(&mut r, &mut out, &lit, &dist, max_out)?;
            }
            _ => return Err(bad()),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

fn inflate_block(r: &mut BitReader, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman, max_out: usize) -> Result<()> {
    loop {
        let sym = lit.decode(r)? as usize;
        if sym < 256 {
            if out.len() >= max_out {
                return Err(bad());
            }
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let li = sym - 257;
            if li >= 29 {
                return Err(bad());
            }
            let len = LEN_BASE[li] as usize + r.bits(LEN_EXTRA[li] as u32)? as usize;
            let di = dist.decode(r)? as usize;
            if di >= 30 {
                return Err(bad());
            }
            let d = DIST_BASE[di] as usize + r.bits(DIST_EXTRA[di] as u32)? as usize;
            if d > out.len() || out.len() + len > max_out {
                return Err(bad());
            }
            let start = out.len() - d;
            for k in 0..len {
                let b = out[start + k];
                out.push(b);
            }
        }
    }
}

/// Decodes a zlib stream (the format inside PNG pictures) and checks its checksum.
pub fn zlib_decompress(data: &[u8], max_out: usize) -> Result<Vec<u8>> {
    if data.len() < 6 {
        return Err(bad());
    }
    let (cmf, flg) = (data[0], data[1]);
    if cmf & 0x0F != 8 || ((cmf as u16) << 8 | flg as u16) % 31 != 0 || flg & 0x20 != 0 {
        return Err(bad());
    }
    let out = inflate_raw(&data[2..], max_out)?;
    // The checksum follows the compressed data; its exact position is not tracked here, so the
    // last four bytes are used (PNG's IDAT data ends with it).
    let tail = &data[data.len() - 4..];
    if u32::from_be_bytes([tail[0], tail[1], tail[2], tail[3]]) != adler32(&out) {
        return Err(bad());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_match_known_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn deflate_round_trips_through_the_decoder() {
        let mut samples: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"a".to_vec(),
            b"abcabcabcabcabcabcabcabc hello hello hello".to_vec(),
            vec![0u8; 100_000],
        ];
        // Pseudo-random bytes with some repeats, longer than the window.
        let mut x = 12345u32;
        let mut noisy = Vec::new();
        for i in 0..80_000u32 {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            noisy.push(if i % 7 == 0 { (x >> 16) as u8 } else { (i % 251) as u8 });
        }
        samples.push(noisy);
        for s in samples {
            let z = zlib_compress(&s);
            assert_eq!(zlib_decompress(&z, s.len() + 1).unwrap(), s);
            if s.len() > 1000 {
                assert!(z.len() < s.len(), "repetitive data gets smaller");
            }
        }
    }

    #[test]
    fn decodes_zlib_streams_made_by_other_encoders() {
        // zlib.compress(b"hello hello hello hello", 9) and a stored block, made with Python's zlib.
        // (A block with dynamic codes is covered by the PNG test in `image.rs`.)
        let fixed = [0x78, 0xda, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x27, 0x01, 0x68, 0x03, 0x08, 0xb1];
        assert_eq!(zlib_decompress(&fixed, 100).unwrap(), b"hello hello hello hello");
        let stored = [0x78, 0x01, 0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c', 0x02, 0x4d, 0x01, 0x27];
        assert_eq!(zlib_decompress(&stored, 100).unwrap(), b"abc");
    }

    #[test]
    fn the_decoder_refuses_damaged_or_oversized_data() {
        let z = zlib_compress(&vec![7u8; 10_000]);
        assert!(zlib_decompress(&z, 9_999).is_err(), "more than the limit");
        let mut broken = z.clone();
        let n = broken.len();
        broken[n - 1] ^= 0xFF;
        assert!(zlib_decompress(&broken, 20_000).is_err(), "wrong checksum");
        assert!(zlib_decompress(&[0x78, 0x01, 0xFF], 10).is_err());
    }
}
