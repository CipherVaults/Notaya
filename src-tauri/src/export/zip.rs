//! A minimal ZIP writer (enough for a .docx file): DEFLATE-compressed or stored entries,
//! UTF-8 names, no ZIP64 (an export larger than 4 GB is refused).

use super::deflate::{crc32, deflate_raw};
use crate::error::{Error, Result};

pub struct ZipWriter {
    out: Vec<u8>,
    central: Vec<u8>,
    count: u16,
    time: u16,
    date: u16,
}

impl ZipWriter {
    /// `ms`: the time stored for every entry (milliseconds since 1970, UTC).
    pub fn new(ms: i64) -> ZipWriter {
        let (time, date) = dos_time(ms);
        ZipWriter { out: Vec::new(), central: Vec::new(), count: 0, time, date }
    }

    /// Adds a file. Pictures, which are compressed already, are stored as they are.
    pub fn add(&mut self, name: &str, data: &[u8], compress: bool) -> Result<()> {
        let too_big = || Error::invalid("The export is too large for a single file.");
        let crc = crc32(data);
        let packed;
        let (method, body): (u16, &[u8]) = if compress {
            packed = deflate_raw(data);
            if packed.len() < data.len() { (8, &packed) } else { (0, data) }
        } else {
            (0, data)
        };
        let offset = u32::try_from(self.out.len()).map_err(|_| too_big())?;
        let size = u32::try_from(body.len()).map_err(|_| too_big())?;
        let raw_size = u32::try_from(data.len()).map_err(|_| too_big())?;
        let name_bytes = name.as_bytes();
        let flags: u16 = 1 << 11; // the name is UTF-8

        let mut local = Vec::with_capacity(30 + name_bytes.len());
        local.extend(0x0403_4B50u32.to_le_bytes());
        local.extend(20u16.to_le_bytes()); // version needed
        local.extend(flags.to_le_bytes());
        local.extend(method.to_le_bytes());
        local.extend(self.time.to_le_bytes());
        local.extend(self.date.to_le_bytes());
        local.extend(crc.to_le_bytes());
        local.extend(size.to_le_bytes());
        local.extend(raw_size.to_le_bytes());
        local.extend((name_bytes.len() as u16).to_le_bytes());
        local.extend(0u16.to_le_bytes()); // extra field
        local.extend_from_slice(name_bytes);
        self.out.extend(local);
        self.out.extend_from_slice(body);

        let c = &mut self.central;
        c.extend(0x0201_4B50u32.to_le_bytes());
        c.extend(20u16.to_le_bytes()); // made by
        c.extend(20u16.to_le_bytes()); // version needed
        c.extend(flags.to_le_bytes());
        c.extend(method.to_le_bytes());
        c.extend(self.time.to_le_bytes());
        c.extend(self.date.to_le_bytes());
        c.extend(crc.to_le_bytes());
        c.extend(size.to_le_bytes());
        c.extend(raw_size.to_le_bytes());
        c.extend((name_bytes.len() as u16).to_le_bytes());
        c.extend([0u8; 8]); // extra, comment, disk number, internal attributes
        c.extend(0u32.to_le_bytes()); // external attributes
        c.extend(offset.to_le_bytes());
        c.extend_from_slice(name_bytes);
        self.count = self.count.checked_add(1).ok_or_else(too_big)?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<Vec<u8>> {
        let too_big = || Error::invalid("The export is too large for a single file.");
        let cd_offset = u32::try_from(self.out.len()).map_err(|_| too_big())?;
        let cd_size = u32::try_from(self.central.len()).map_err(|_| too_big())?;
        self.out.extend_from_slice(&self.central);
        self.out.extend(0x0605_4B50u32.to_le_bytes());
        self.out.extend([0u8; 4]); // disk numbers
        self.out.extend(self.count.to_le_bytes());
        self.out.extend(self.count.to_le_bytes());
        self.out.extend(cd_size.to_le_bytes());
        self.out.extend(cd_offset.to_le_bytes());
        self.out.extend(0u16.to_le_bytes()); // comment
        Ok(self.out)
    }
}

/// MS-DOS time and date (UTC; ZIP has no time zone), at least 1980-01-01.
fn dos_time(ms: i64) -> (u16, u16) {
    let (year, month, day, h, m, s) = civil(ms.max(315_532_800_000));
    let year = year.clamp(1980, 2107);
    let time = (h << 11 | m << 5 | s / 2) as u16;
    let date = (((year - 1980) << 9) | (month << 5) | day) as u16;
    (time, date)
}

/// A UTC date and time: (year, month, day, hour, minute, second).
pub fn civil(ms: i64) -> (i64, i64, i64, i64, i64, i64) {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Days since 1970 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day, rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
pub mod tests {
    use super::super::deflate::inflate_raw;
    use super::*;

    /// Reads a ZIP made by `ZipWriter` back: (name, contents) for each entry, checking every
    /// header, size and checksum on the way.
    pub fn read_zip(zip: &[u8]) -> Vec<(String, Vec<u8>)> {
        let u16_at = |i: usize| u16::from_le_bytes([zip[i], zip[i + 1]]) as usize;
        let u32_at = |i: usize| u32::from_le_bytes([zip[i], zip[i + 1], zip[i + 2], zip[i + 3]]) as usize;
        let eocd = zip.len() - 22;
        assert_eq!(u32_at(eocd), 0x0605_4B50, "end of central directory");
        let count = u16_at(eocd + 10);
        let mut at = u32_at(eocd + 16);
        assert_eq!(at + u32_at(eocd + 12), eocd, "central directory size");
        let mut out = Vec::new();
        for _ in 0..count {
            assert_eq!(u32_at(at), 0x0201_4B50);
            let method = u16_at(at + 10);
            let crc = u32_at(at + 16) as u32;
            let size = u32_at(at + 20);
            let raw = u32_at(at + 24);
            let name_len = u16_at(at + 28);
            let local = u32_at(at + 42);
            let name = String::from_utf8(zip[at + 46..at + 46 + name_len].to_vec()).unwrap();
            assert_eq!(u32_at(local), 0x0403_4B50);
            assert_eq!(&zip[local + 30..local + 30 + name_len], name.as_bytes());
            let start = local + 30 + name_len + u16_at(local + 28);
            let body = &zip[start..start + size];
            let data = if method == 8 { inflate_raw(body, raw).unwrap() } else { body.to_vec() };
            assert_eq!(data.len(), raw);
            assert_eq!(crc32(&data), crc, "checksum of {name}");
            out.push((name, data));
            at += 46 + name_len;
        }
        out
    }

    #[test]
    fn entries_round_trip() {
        let mut z = ZipWriter::new(1_790_000_000_000);
        z.add("a.xml", "<a>héllo</a>".repeat(50).as_bytes(), true).unwrap();
        z.add("media/b.png", &[1, 2, 3], false).unwrap();
        let bytes = z.finish().unwrap();
        let entries = read_zip(&bytes);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "a.xml");
        assert_eq!(entries[0].1, "<a>héllo</a>".repeat(50).as_bytes());
        assert_eq!(entries[1], ("media/b.png".to_string(), vec![1, 2, 3]));
    }

    #[test]
    fn dos_dates() {
        // 2026-10-28 12:34:56 UTC
        let (time, date) = dos_time(1_793_190_896_000);
        assert_eq!(date >> 9, 2026 - 1980);
        assert_eq!((date >> 5) & 0xF, 10);
        assert_eq!(date & 0x1F, 28);
        assert_eq!(time >> 11, 12);
        assert_eq!((time >> 5) & 0x3F, 34);
        assert_eq!(dos_time(0).1, 1 << 5 | 1, "before 1980 becomes 1980-01-01");
    }
}
