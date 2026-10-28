//! A ZIP reader for imports (Notion exports and Word files), written for hostile input.
//!
//! It reads the central directory, never writes anything to disk, and inflates an entry
//! only when it is asked for, with `miniz_oxide`. Limits: the number of entries, the size of
//! each entry, the total of everything inflated during one import (a shared budget), and the
//! compression ratio of large entries (zip bombs). Entry names are cleaned: `..`, absolute
//! paths, drive letters, `:` and control characters are refused (no zip-slip, even though
//! nothing is ever extracted to disk). Encrypted entries and methods other than stored and
//! deflate are refused with a reason.

use std::cell::Cell;

pub const MAX_ENTRIES: usize = 50_000;
/// Largest single entry, as declared and as inflated.
pub const MAX_ENTRY: u64 = 512 * 1024 * 1024;
/// Entries above this size must not be compressed more than `MAX_RATIO` to 1.
const RATIO_FLOOR: u64 = 1024 * 1024;
const MAX_RATIO: u64 = 100;

pub struct Entry {
    /// Cleaned path, `/` separated, no empty, `.` or `..` parts.
    pub name: String,
    method: u16,
    crc: u32,
    comp_size: u64,
    pub size: u64,
    local_offset: u64,
}

pub struct Archive {
    data: Vec<u8>,
    pub entries: Vec<Entry>,
    /// Entries that were refused, with the reason.
    pub rejected: Vec<(String, String)>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

const NOT_ZIP: &str = "This is not a ZIP file, or it is damaged.";

/// Cleans an entry name. `None` if it must be refused.
pub fn clean_name(raw: &str) -> Option<String> {
    let s = raw.replace('\\', "/");
    if s.starts_with('/') || s.chars().any(|c| c.is_control() || c == ':') {
        return None;
    }
    let mut parts = Vec::new();
    for part in s.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            p => parts.push(p),
        }
    }
    if parts.is_empty() { None } else { Some(parts.join("/")) }
}

impl Archive {
    pub fn open(data: Vec<u8>) -> Result<Archive, String> {
        let b = &data;
        if b.len() < 22 {
            return Err(NOT_ZIP.into());
        }
        // End of central directory: the last 22 bytes plus at most a 64 KiB comment.
        let lowest = b.len().saturating_sub(22 + 65_535);
        let eocd = (lowest..=b.len() - 22)
            .rev()
            .find(|&i| u32_at(b, i) == Some(0x0605_4b50))
            .ok_or(NOT_ZIP)?;
        if u16_at(b, eocd + 4) != Some(0) || u16_at(b, eocd + 6) != Some(0) {
            return Err("ZIP files split over several parts are not supported.".into());
        }
        let mut count = u64::from(u16_at(b, eocd + 10).ok_or(NOT_ZIP)?);
        let mut cd_size = u64::from(u32_at(b, eocd + 12).ok_or(NOT_ZIP)?);
        let mut cd_offset = u64::from(u32_at(b, eocd + 16).ok_or(NOT_ZIP)?);
        if count == 0xFFFF || cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF {
            // ZIP64: the locator sits right before the end record.
            let loc = eocd.checked_sub(20).ok_or(NOT_ZIP)?;
            if u32_at(b, loc) != Some(0x0706_4b50) {
                return Err(NOT_ZIP.into());
            }
            let rec = usize::try_from(u64_at(b, loc + 8).ok_or(NOT_ZIP)?).map_err(|_| NOT_ZIP)?;
            if u32_at(b, rec) != Some(0x0606_4b50) {
                return Err(NOT_ZIP.into());
            }
            count = u64_at(b, rec + 32).ok_or(NOT_ZIP)?;
            cd_size = u64_at(b, rec + 40).ok_or(NOT_ZIP)?;
            cd_offset = u64_at(b, rec + 48).ok_or(NOT_ZIP)?;
        }
        if count > MAX_ENTRIES as u64 {
            return Err(format!("This ZIP file has more than {MAX_ENTRIES} entries."));
        }
        let start = usize::try_from(cd_offset).map_err(|_| NOT_ZIP)?;
        let end = start.checked_add(usize::try_from(cd_size).map_err(|_| NOT_ZIP)?).ok_or(NOT_ZIP)?;
        if end > b.len() {
            return Err(NOT_ZIP.into());
        }
        let mut entries = Vec::new();
        let mut rejected = Vec::new();
        let mut at = start;
        for _ in 0..count {
            if u32_at(b, at) != Some(0x0201_4b50) {
                return Err(NOT_ZIP.into());
            }
            let flags = u16_at(b, at + 8).ok_or(NOT_ZIP)?;
            let method = u16_at(b, at + 10).ok_or(NOT_ZIP)?;
            let crc = u32_at(b, at + 16).ok_or(NOT_ZIP)?;
            let mut comp_size = u64::from(u32_at(b, at + 20).ok_or(NOT_ZIP)?);
            let mut size = u64::from(u32_at(b, at + 24).ok_or(NOT_ZIP)?);
            let name_len = usize::from(u16_at(b, at + 28).ok_or(NOT_ZIP)?);
            let extra_len = usize::from(u16_at(b, at + 30).ok_or(NOT_ZIP)?);
            let comment_len = usize::from(u16_at(b, at + 32).ok_or(NOT_ZIP)?);
            let mut local_offset = u64::from(u32_at(b, at + 42).ok_or(NOT_ZIP)?);
            let name_bytes = b.get(at + 46..at + 46 + name_len).ok_or(NOT_ZIP)?;
            let extra = b.get(at + 46 + name_len..at + 46 + name_len + extra_len).ok_or(NOT_ZIP)?;
            // ZIP64 sizes and offset, in that order, only for the fields that overflowed.
            let mut e = 0;
            while e + 4 <= extra.len() {
                let id = u16_at(extra, e).unwrap_or(0);
                let len = usize::from(u16_at(extra, e + 2).unwrap_or(0));
                if id == 0x0001 {
                    let mut f = e + 4;
                    let lim = (e + 4 + len).min(extra.len());
                    for field in [&mut size, &mut comp_size, &mut local_offset] {
                        if *field == 0xFFFF_FFFF && f + 8 <= lim {
                            *field = u64_at(extra, f).unwrap_or(*field);
                            f += 8;
                        }
                    }
                }
                e += 4 + len;
            }
            at += 46 + name_len + extra_len + comment_len;

            let raw = String::from_utf8(name_bytes.to_vec()).unwrap_or_else(|_| String::from_utf8_lossy(name_bytes).into_owned());
            if raw.ends_with('/') || raw.ends_with('\\') {
                continue; // a folder
            }
            let Some(name) = clean_name(&raw) else {
                rejected.push((raw, "Unsafe name in the ZIP file (refused).".into()));
                continue;
            };
            let reason = if flags & 1 != 0 {
                Some("Encrypted entries are not supported.")
            } else if method != 0 && method != 8 {
                Some("This compression method is not supported.")
            } else if size > MAX_ENTRY {
                Some("Too large.")
            } else if method == 8 && size > RATIO_FLOOR && size / comp_size.max(1) > MAX_RATIO {
                Some("Compressed suspiciously well (possible ZIP bomb); refused.")
            } else if method == 0 && comp_size != size {
                Some("Damaged entry.")
            } else {
                None
            };
            if let Some(r) = reason {
                rejected.push((name, r.into()));
                continue;
            }
            entries.push(Entry { name, method, crc, comp_size, size, local_offset });
        }
        Ok(Archive { data, entries, rejected })
    }

    /// Finds an entry by its cleaned name.
    pub fn find(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.name == name)
    }

    /// Inflates one entry. Refused if it is larger than `max`, or if the shared `budget`
    /// (bytes still allowed for this import) would run out.
    pub fn read(&self, index: usize, max: u64, budget: &Cell<u64>) -> Result<Vec<u8>, String> {
        let e = self.entries.get(index).ok_or("Missing entry.")?;
        if e.size > max {
            return Err("Too large.".into());
        }
        if e.size > budget.get() {
            return Err("The import is too large in total (more than 2 GB of content).".into());
        }
        let b = &self.data;
        let lo = usize::try_from(e.local_offset).map_err(|_| "Damaged entry.")?;
        if u32_at(b, lo) != Some(0x0403_4b50) {
            return Err("Damaged entry.".into());
        }
        let name_len = usize::from(u16_at(b, lo + 26).ok_or("Damaged entry.")?);
        let extra_len = usize::from(u16_at(b, lo + 28).ok_or("Damaged entry.")?);
        let start = lo + 30 + name_len + extra_len;
        let len = usize::try_from(e.comp_size).map_err(|_| "Damaged entry.")?;
        let raw = b.get(start..start.checked_add(len).ok_or("Damaged entry.")?).ok_or("Damaged entry.")?;
        let out = if e.method == 0 {
            raw.to_vec()
        } else {
            // The declared size is the limit, so a lying header cannot inflate further.
            let limit = usize::try_from(e.size).map_err(|_| "Too large.")?;
            miniz_oxide::inflate::decompress_to_vec_with_limit(raw, limit).map_err(|_| "Damaged or oversized entry.".to_string())?
        };
        if out.len() as u64 != e.size {
            return Err("Damaged entry (wrong size).".into());
        }
        if crc32(&out) != e.crc {
            return Err("Damaged entry (checksum mismatch).".into());
        }
        budget.set(budget.get() - e.size);
        Ok(out)
    }
}

/// CRC-32 (IEEE), as used by ZIP.
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
    let mut c = 0xFFFF_FFFFu32;
    for &byte in data {
        c = table[((c ^ u32::from(byte)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

/// Builds a ZIP in memory (tests only). `deflate`: compress with miniz_oxide.
#[cfg(test)]
pub fn build(files: &[(&str, &[u8])], deflate: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut cd = Vec::new();
    for (name, data) in files {
        let (method, body) = if deflate {
            (8u16, miniz_oxide::deflate::compress_to_vec(data, 6))
        } else {
            (0u16, data.to_vec())
        };
        let offset = out.len() as u32;
        let crc = crc32(data);
        let mut h = Vec::new();
        h.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&0x0800u16.to_le_bytes());
        h.extend_from_slice(&method.to_le_bytes());
        h.extend_from_slice(&[0; 4]);
        h.extend_from_slice(&crc.to_le_bytes());
        h.extend_from_slice(&(body.len() as u32).to_le_bytes());
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h.extend_from_slice(&(name.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&h);
        out.extend_from_slice(&body);
        let mut c = Vec::new();
        c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&0x0800u16.to_le_bytes());
        c.extend_from_slice(&method.to_le_bytes());
        c.extend_from_slice(&[0; 4]);
        c.extend_from_slice(&crc.to_le_bytes());
        c.extend_from_slice(&(body.len() as u32).to_le_bytes());
        c.extend_from_slice(&(data.len() as u32).to_le_bytes());
        c.extend_from_slice(&(name.len() as u16).to_le_bytes());
        c.extend_from_slice(&[0; 12]);
        c.extend_from_slice(&offset.to_le_bytes());
        c.extend_from_slice(name.as_bytes());
        cd.extend_from_slice(&c);
    }
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&cd);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(cd.len() as u32).to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_stored_and_deflated_entries() {
        for deflate in [false, true] {
            let z = build(&[("a/b.md", b"# Hi"), ("dir/", b""), ("c.txt", b"hello hello hello")], deflate);
            let a = Archive::open(z).unwrap();
            assert_eq!(a.entries.len(), 2, "folders are not entries");
            let budget = Cell::new(1000);
            assert_eq!(a.read(a.find("a/b.md").unwrap(), 100, &budget).unwrap(), b"# Hi");
            assert_eq!(budget.get(), 996, "the budget is spent");
            assert_eq!(a.read(a.find("c.txt").unwrap(), 5, &budget).unwrap_err(), "Too large.");
        }
    }

    #[test]
    fn refuses_zip_slip_names() {
        for bad in ["../evil.md", "a/../../evil.md", "/etc/passwd", "C:\\Windows\\x.md", "a\\..\\..\\x", "file.md:stream", "a\u{0}b"] {
            assert_eq!(clean_name(bad), None, "{bad}");
        }
        assert_eq!(clean_name("./a//b\\c.md").as_deref(), Some("a/b/c.md"));
        let z = build(&[("../evil.md", b"x"), ("ok.md", b"y")], false);
        let a = Archive::open(z).unwrap();
        assert_eq!(a.entries.len(), 1);
        assert_eq!(a.rejected.len(), 1);
    }

    #[test]
    fn refuses_a_zip_bomb_and_respects_the_budget() {
        let zeros = vec![0u8; 20 * 1024 * 1024];
        let z = build(&[("bomb.md", &zeros)], true);
        let a = Archive::open(z).unwrap();
        assert!(a.entries.is_empty(), "a 20 MB file of zeros compresses about 1000:1 and is refused");
        assert!(a.rejected[0].1.contains("ZIP bomb"));

        let z = build(&[("a.md", &[b'x'; 600]), ("b.md", &[b'y'; 600])], false);
        let a = Archive::open(z).unwrap();
        let budget = Cell::new(1000);
        assert!(a.read(0, 10_000, &budget).is_ok());
        assert!(a.read(1, 10_000, &budget).unwrap_err().contains("too large in total"));
    }

    #[test]
    fn a_lying_size_or_checksum_is_caught() {
        let mut z = build(&[("a.md", b"hello world, hello world")], true);
        // Corrupt the CRC in the central directory.
        let cd = z.windows(4).rposition(|w| w == 0x0201_4b50u32.to_le_bytes()).unwrap();
        z[cd + 16] ^= 0xFF;
        let a = Archive::open(z).unwrap();
        assert!(a.read(0, 1000, &Cell::new(1000)).unwrap_err().contains("checksum"));
    }

    #[test]
    fn garbage_is_not_a_zip() {
        assert!(Archive::open(b"PK\x05\x06garbage".to_vec()).is_err());
        assert!(Archive::open(vec![0; 10]).is_err());
        let mut z = build(&[("a.md", b"x")], false);
        let n = z.len();
        z[n - 6..n - 2].copy_from_slice(&0xFFFF_FF00u32.to_le_bytes()); // central directory offset out of range
        assert!(Archive::open(z).is_err());
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
