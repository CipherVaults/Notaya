//! A ZIP reader for imports (Notion exports and Word files), written for hostile input.
//!
//! It reads the central directory, never writes anything to disk, and inflates an entry
//! only when it is asked for, with the inflate code of the exporter. Limits: the number of entries, the size of
//! each entry, the total of everything inflated during one import (a shared budget), and the
//! compression ratio of large entries (zip bombs). Entry names are cleaned: `..`, absolute
//! paths, drive letters, `:` and control characters are refused (no zip-slip, even though
//! nothing is ever extracted to disk). Encrypted entries and methods other than stored and
//! deflate are refused with a reason.

use std::borrow::Cow;
use std::cell::Cell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

pub const MAX_ENTRIES: usize = 50_000;
/// Largest single payload entry, as declared and as inflated.
pub const MAX_ENTRY: u64 = 2 * 1024 * 1024 * 1024;
/// Deflated entries above this size must not be compressed more than `MAX_RATIO` to 1;
/// sub-megabyte entries (256 KiB–1 MiB) also enforce `MAX_SMALL_RATIO` to 1, and a single
/// archive's accepted entries are capped at `MAX_ARCHIVE_UNPACKED` declared bytes.
const RATIO_FLOOR: u64 = 1024 * 1024;
const SMALL_RATIO_FLOOR: u64 = 256 * 1024;
const MAX_RATIO: u64 = 100;
const MAX_SMALL_RATIO: u64 = 100;
const MAX_ARCHIVE_UNPACKED: u64 = 4 * 1024 * 1024 * 1024;
const MAX_CENTRAL_DIRECTORY: u64 = 64 * 1024 * 1024;

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
    source: Source,
    directory_offset: u64,
    pub(super) directory_size: u64,
    pub entries: Vec<Entry>,
    /// Entries that were refused, with the reason.
    pub rejected: Vec<(String, String)>,
}

enum Backing {
    Memory(Vec<u8>),
    Disk(Mutex<File>),
}

/// A checked byte range in a picked archive. Stored parts can share the original handle.
struct Source {
    backing: Arc<Backing>,
    offset: u64,
    len: u64,
}

impl Source {
    fn subrange(&self, offset: u64, len: u64) -> Option<Self> {
        if offset.checked_add(len)? > self.len {
            return None;
        }
        Some(Self { backing: self.backing.clone(), offset: self.offset.checked_add(offset)?, len })
    }

    fn read_exact(&self, offset: u64, bytes: &mut [u8]) -> Option<()> {
        if offset.checked_add(bytes.len() as u64)? > self.len {
            return None;
        }
        let start = self.offset.checked_add(offset)?;
        match self.backing.as_ref() {
            Backing::Memory(data) => {
                let start = usize::try_from(start).ok()?;
                bytes.copy_from_slice(data.get(start..start.checked_add(bytes.len())?)?);
            }
            Backing::Disk(file) => {
                let mut file = file.lock().ok()?;
                file.seek(SeekFrom::Start(start)).ok()?;
                file.read_exact(bytes).ok()?;
            }
        }
        Some(())
    }

    fn read(&self, offset: u64, len: u64) -> Option<Vec<u8>> {
        if offset.checked_add(len)? > self.len {
            return None;
        }
        let len = usize::try_from(len).ok()?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(len).ok()?;
        bytes.resize(len, 0);
        self.read_exact(offset, &mut bytes)?;
        Some(bytes)
    }

    fn bytes(&self) -> Option<Cow<'_, [u8]>> {
        match self.backing.as_ref() {
            Backing::Memory(data) => {
                let start = usize::try_from(self.offset).ok()?;
                let end = usize::try_from(self.offset.checked_add(self.len)?).ok()?;
                data.get(start..end).map(Cow::Borrowed)
            }
            Backing::Disk(_) => self.read(0, self.len).map(Cow::Owned),
        }
    }
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at.checked_add(2)?)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at.checked_add(8)?)?.try_into().ok()?))
}

const NOT_ZIP: &str = "This is not a ZIP file, or it is damaged.";
const SPLIT_ZIP: &str = "ZIP files split over several disks are not supported.";
const DAMAGED_ENTRY: &str = "Damaged entry.";

/// Notion wraps exports as `Part-N.zip` or `Export-…-Part-N.zip`.
pub(super) fn is_export_part(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some((stem, extension)) = name.rsplit_once('.') else { return false };
    if !extension.eq_ignore_ascii_case("zip") {
        return false;
    }
    let stem = stem.to_ascii_lowercase();
    let Some(number) = stem.strip_prefix("part-").or_else(|| {
        stem.strip_prefix("export-").and_then(|rest| {
            rest.strip_prefix("part-").or_else(|| rest.rsplit_once("-part-").map(|(_, n)| n))
        })
    }) else { return false };
    number.parse::<u32>().is_ok_and(|n| n > 0) && number.bytes().all(|b| b.is_ascii_digit())
}

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
        let len = data.len() as u64;
        Self::open_source(Source { backing: Arc::new(Backing::Memory(data)), offset: 0, len })
    }

    pub(super) fn open_file(file: File, len: u64) -> Result<Archive, String> {
        Self::open_source(Source { backing: Arc::new(Backing::Disk(Mutex::new(file))), offset: 0, len })
    }

    fn open_source(source: Source) -> Result<Archive, String> {
        if source.len < 22 {
            return Err(NOT_ZIP.into());
        }
        // End of central directory: the last 22 bytes plus at most a 64 KiB comment.
        let tail_len = source.len.min(22 + 65_535);
        let tail_offset = source.len - tail_len;
        let tail = source.read(tail_offset, tail_len).ok_or(NOT_ZIP)?;
        let eocd = (0..=tail.len() - 22)
            .rev()
            .find(|&i| {
                u32_at(&tail, i) == Some(0x0605_4b50)
                    && u16_at(&tail, i + 20).is_some_and(|len| i + 22 + usize::from(len) == tail.len())
            })
            .ok_or(NOT_ZIP)?;
        let end = &tail[eocd..];
        if u16_at(end, 4) != Some(0) || u16_at(end, 6) != Some(0) {
            return Err(SPLIT_ZIP.into());
        }
        let mut count = u64::from(u16_at(end, 10).ok_or(NOT_ZIP)?);
        let mut cd_size = u64::from(u32_at(end, 12).ok_or(NOT_ZIP)?);
        let mut cd_offset = u64::from(u32_at(end, 16).ok_or(NOT_ZIP)?);
        let mut directory_limit = tail_offset + eocd as u64;
        if count == 0xFFFF || cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF {
            // ZIP64: the locator sits right before the end record.
            let locator_offset = directory_limit.checked_sub(20).ok_or(NOT_ZIP)?;
            let mut locator = [0; 20];
            source.read_exact(locator_offset, &mut locator).ok_or(NOT_ZIP)?;
            if u32_at(&locator, 0) != Some(0x0706_4b50) {
                return Err(NOT_ZIP.into());
            }
            if u32_at(&locator, 4) != Some(0) || u32_at(&locator, 16) != Some(1) {
                return Err(SPLIT_ZIP.into());
            }
            let record_offset = u64_at(&locator, 8).ok_or(NOT_ZIP)?;
            let mut record = [0; 56];
            source.read_exact(record_offset, &mut record).ok_or(NOT_ZIP)?;
            let record_len = u64_at(&record, 4).ok_or(NOT_ZIP)?;
            if u32_at(&record, 0) != Some(0x0606_4b50)
                || record_len < 44
                || record_offset.checked_add(12).and_then(|at| at.checked_add(record_len)) != Some(locator_offset)
            {
                return Err(NOT_ZIP.into());
            }
            count = u64_at(&record, 32).ok_or(NOT_ZIP)?;
            if u32_at(&record, 16) != Some(0) || u32_at(&record, 20) != Some(0) || u64_at(&record, 24) != Some(count) {
                return Err(SPLIT_ZIP.into());
            }
            cd_size = u64_at(&record, 40).ok_or(NOT_ZIP)?;
            cd_offset = u64_at(&record, 48).ok_or(NOT_ZIP)?;
            directory_limit = record_offset;
        } else if u16_at(end, 8).map(u64::from) != Some(count) {
            return Err(SPLIT_ZIP.into());
        }
        if count > MAX_ENTRIES as u64 {
            return Err(format!("This ZIP file has more than {MAX_ENTRIES} entries."));
        }
        if cd_size > MAX_CENTRAL_DIRECTORY {
            return Err("The ZIP file's directory is larger than 64 MB.".into());
        }
        if cd_offset.checked_add(cd_size).ok_or(NOT_ZIP)? > directory_limit {
            return Err(NOT_ZIP.into());
        }
        let directory = source.read(cd_offset, cd_size).ok_or(NOT_ZIP)?;
        let b = directory.as_slice();
        let mut entries = Vec::new();
        let mut rejected = Vec::new();
        let mut declared_total: u64 = 0;
        let mut at = 0;
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
            let mut local_offset = u64::from(at.checked_add(42).and_then(|p| u32_at(b, p)).ok_or(NOT_ZIP)?);
            let name_at = at.checked_add(46).ok_or(NOT_ZIP)?;
            let extra_at = name_at.checked_add(name_len).ok_or(NOT_ZIP)?;
            let name_bytes = b.get(name_at..extra_at).ok_or(NOT_ZIP)?;
            let extra = b.get(extra_at..extra_at.checked_add(extra_len).ok_or(NOT_ZIP)?).ok_or(NOT_ZIP)?;
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
            at = extra_at.checked_add(extra_len).and_then(|p| p.checked_add(comment_len)).ok_or(NOT_ZIP)?;
            if at > b.len() {
                return Err(NOT_ZIP.into());
            }

            let raw = String::from_utf8_lossy(name_bytes);
            if raw.ends_with('/') || raw.ends_with('\\') {
                continue; // a folder
            }
            let Some(name) = clean_name(&raw) else {
                rejected.push((raw.into_owned(), "Unsafe name in the ZIP file (refused).".into()));
                continue;
            };
            let reason = if flags & 1 != 0 {
                Some("Encrypted entries are not supported.")
            } else if method != 0 && method != 8 {
                Some("This compression method is not supported.")
            } else if size > MAX_ENTRY && !(method == 0 && is_export_part(&name) && size <= MAX_ARCHIVE_UNPACKED) {
                Some("Too large.")
            } else if method == 8
                && ((size > RATIO_FLOOR && size / comp_size.max(1) > MAX_RATIO)
                    || (size > SMALL_RATIO_FLOOR && size / comp_size.max(1) > MAX_SMALL_RATIO))
            {
                Some("Compressed suspiciously well (possible ZIP bomb); refused.")
            } else if method == 0 && comp_size != size {
                Some("Damaged entry.")
            } else if declared_total.saturating_add(size) > MAX_ARCHIVE_UNPACKED {
                Some("The archive declares more than 4 GB of content; refused.")
            } else {
                None
            };
            if let Some(r) = reason {
                rejected.push((name, r.into()));
                continue;
            }
            declared_total = declared_total.saturating_add(size);
            entries.push(Entry { name, method, crc, comp_size, size, local_offset });
        }
        Ok(Archive { source, directory_offset: cd_offset, directory_size: cd_size, entries, rejected })
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
            return Err("The import is too large in total (more than 4 GB of content).".into());
        }
        // Valid compressed data can be slightly larger than its output. Bound the allocation
        // independently of the output limit in case the directory declares an absurd input size.
        if e.comp_size > e.size.saturating_add(e.size / 8).saturating_add(64 * 1024) {
            return Err("Damaged entry (invalid compressed size).".into());
        }
        let source = self.entry_source(e)?;
        let out = if e.method == 0 {
            source.read(0, source.len).ok_or(DAMAGED_ENTRY)?
        } else {
            // The declared size is the limit, so a lying header cannot inflate further.
            let limit = usize::try_from(e.size).map_err(|_| "Too large.")?;
            let raw = source.bytes().ok_or(DAMAGED_ENTRY)?;
            crate::export::deflate::inflate_raw(&raw, limit).map_err(|_| "Damaged or oversized entry.".to_string())?
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

    fn entry_source(&self, entry: &Entry) -> Result<Source, String> {
        let mut header = [0; 30];
        self.source.read_exact(entry.local_offset, &mut header).ok_or(DAMAGED_ENTRY)?;
        if u32_at(&header, 0) != Some(0x0403_4b50)
            || u16_at(&header, 6).is_some_and(|flags| flags & 1 != 0)
            || u16_at(&header, 8) != Some(entry.method)
        {
            return Err(DAMAGED_ENTRY.into());
        }
        let name_len = u64::from(u16_at(&header, 26).ok_or(DAMAGED_ENTRY)?);
        let extra_len = u64::from(u16_at(&header, 28).ok_or(DAMAGED_ENTRY)?);
        let start = entry.local_offset.checked_add(30 + name_len + extra_len).ok_or(DAMAGED_ENTRY)?;
        if start.checked_add(entry.comp_size).ok_or(DAMAGED_ENTRY)? > self.directory_offset {
            return Err(DAMAGED_ENTRY.into());
        }
        self.source.subrange(start, entry.comp_size).ok_or_else(|| DAMAGED_ENTRY.into())
    }

    /// Containers do not spend the payload budget. Stored wrappers share their parent's backing,
    /// and the caller separately bounds all wrapper I/O and the memory held by deflated wrappers.
    pub(super) fn open_part(&self, index: usize, max_in_memory: u64) -> Result<(Archive, u64), String> {
        let e = self.entries.get(index).ok_or("Missing entry.")?;
        if e.method != 0 {
            if e.size > max_in_memory {
                return Err("Compressed export parts exceed the memory allowance; choose the part ZIPs directly.".into());
            }
            let bytes = self.read(index, max_in_memory, &Cell::new(max_in_memory))?;
            return Ok((Self::open(bytes)?, e.size));
        }
        let source = self.entry_source(e)?;
        let mut chunk = vec![0; 64 * 1024];
        let mut offset = 0;
        let mut crc = u32::MAX;
        while offset < source.len {
            let n = (source.len - offset).min(chunk.len() as u64) as usize;
            source.read_exact(offset, &mut chunk[..n]).ok_or(DAMAGED_ENTRY)?;
            crc = update_crc32(crc, &chunk[..n]);
            offset += n as u64;
        }
        if !crc != e.crc {
            return Err("Damaged entry (checksum mismatch).".into());
        }
        Ok((Self::open_source(source)?, 0))
    }
}

/// CRC-32 (IEEE), as used by ZIP.
pub fn crc32(data: &[u8]) -> u32 {
    !update_crc32(u32::MAX, data)
}

fn update_crc32(mut crc: u32, data: &[u8]) -> u32 {
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
    for &byte in data {
        crc = table[((crc ^ u32::from(byte)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc
}

/// Builds a ZIP in memory (tests only). `deflate`: compress with the exporter.
#[cfg(test)]
pub fn build(files: &[(&str, &[u8])], deflate: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut cd = Vec::new();
    for (name, data) in files {
        let (method, body) = if deflate {
            (8u16, crate::export::deflate::deflate_raw(data))
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
    fn file_backed_stored_parts_share_the_archive_range_and_check_crc() {
        let inner = build(&[("Page.md", b"# Example")], true);
        let mut outer = build(&[("Export-Part-1.zip", &inner)], false);
        let path = std::env::temp_dir().join(format!("notaya-zip-{}.zip", crate::model::new_id()));
        std::fs::write(&path, &outer).unwrap();
        let archive = Archive::open_file(File::open(&path).unwrap(), outer.len() as u64).unwrap();
        let (part, memory) = archive.open_part(0, 0).unwrap();
        assert_eq!(memory, 0);
        assert!(Arc::ptr_eq(&archive.source.backing, &part.source.backing));
        assert_eq!(part.read(0, 100, &Cell::new(100)).unwrap(), b"# Example");
        drop((archive, part));
        std::fs::remove_file(path).unwrap();

        let start = 30 + "Export-Part-1.zip".len();
        outer[start] ^= 0xff;
        let archive = Archive::open(outer).unwrap();
        assert!(archive.open_part(0, 0).err().unwrap().contains("checksum"));
    }

    #[test]
    fn zip64_directory_records_are_read_from_their_declared_offsets() {
        let mut zip = build(&[("Page.md", b"# Example")], true);
        let end = zip.len() - 22;
        let mut legacy = zip.split_off(end);
        let cd_size = u32_at(&legacy, 12).unwrap();
        let cd_offset = u32_at(&legacy, 16).unwrap();
        legacy[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        zip.extend_from_slice(&0x0606_4b50u32.to_le_bytes());
        zip.extend_from_slice(&44u64.to_le_bytes());
        zip.extend_from_slice(&[45, 0, 45, 0]);
        zip.extend_from_slice(&[0; 8]);
        zip.extend_from_slice(&1u64.to_le_bytes());
        zip.extend_from_slice(&1u64.to_le_bytes());
        zip.extend_from_slice(&u64::from(cd_size).to_le_bytes());
        zip.extend_from_slice(&u64::from(cd_offset).to_le_bytes());
        zip.extend_from_slice(&0x0706_4b50u32.to_le_bytes());
        zip.extend_from_slice(&0u32.to_le_bytes());
        zip.extend_from_slice(&(end as u64).to_le_bytes());
        zip.extend_from_slice(&1u32.to_le_bytes());
        zip.extend_from_slice(&legacy);
        let archive = Archive::open(zip).unwrap();
        assert_eq!(archive.read(0, 100, &Cell::new(100)).unwrap(), b"# Example");
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

    #[test]
    fn hostile_offsets_and_sizes_never_overflow() {
        let base = build(&[("a.md", b"hello world, hello world"), ("b/c.md", b"second")], true);
        let eocd = base.windows(4).rposition(|w| w == 0x0605_4b50u32.to_le_bytes()).unwrap();
        // A ZIP64 locator that points at the very end of the address space.
        for rec in [u64::MAX, u64::MAX - 8, u64::MAX - 40, 0xFFFF_FFFF_FFFF_FFF0, i64::MAX as u64, 1 << 40] {
            let mut z = base[..eocd].to_vec();
            z.extend_from_slice(&0x0706_4b50u32.to_le_bytes());
            z.extend_from_slice(&0u32.to_le_bytes());
            z.extend_from_slice(&rec.to_le_bytes());
            z.extend_from_slice(&1u32.to_le_bytes());
            let mut end = base[eocd..].to_vec();
            end[10..12].copy_from_slice(&0xFFFFu16.to_le_bytes());
            z.extend_from_slice(&end);
            assert!(Archive::open(z).is_err(), "{rec:#x}");
        }
        // Every byte of the central directory and the end record, set to a few extreme values.
        let cd = base.windows(4).position(|w| w == 0x0201_4b50u32.to_le_bytes()).unwrap();
        for i in cd..base.len() {
            for v in [0x00u8, 0x7F, 0x80, 0xFF] {
                let mut z = base.clone();
                z[i] = v;
                if let Ok(a) = Archive::open(z) {
                    for k in 0..a.entries.len() {
                        let _ = a.read(k, 1 << 20, &Cell::new(1 << 20));
                    }
                }
            }
        }
        // A local-header offset and sizes at their largest.
        let mut z = base.clone();
        z[cd + 42..cd + 46].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        let a = Archive::open(z).unwrap();
        assert!(a.read(0, 1 << 20, &Cell::new(1 << 20)).is_err());
        let mut z = base.clone();
        z[cd + 20..cd + 24].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
        if let Ok(a) = Archive::open(z) {
            let _ = a.read(0, u64::MAX, &Cell::new(u64::MAX));
        }
    }

    #[test]
    fn refuses_sub_megabyte_zip_bomb_ratios() {
        let zeros = vec![0u8; 512 * 1024];
        let z = build(&[("small_bomb.md", &zeros)], true);
        let a = Archive::open(z).unwrap();
        assert!(a.entries.is_empty(), "512 KB of zeros compresses >100:1 and must be refused");
        assert!(a.rejected.iter().any(|(n, r)| n == "small_bomb.md" && r.contains("ZIP bomb")));
    }
}
