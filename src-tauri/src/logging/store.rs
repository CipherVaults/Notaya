//! The log files on disk: append, rotate, the "running" flag, and the flood limiter.
//!
//! Nothing here panics and nothing here reports a failure to the person: a log that cannot be
//! written is simply not written.

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const LOG_FILE: &str = "notaya-log.txt";
pub const OLD_LOG_FILE: &str = "notaya-log.1.txt";
pub const FLAG_FILE: &str = "running.flag";
/// When the current file is bigger than this before a write, it becomes the `.1` file.
pub const ROTATE_AT: u64 = 256 * 1024;
/// One entry (a panic with a backtrace is the longest) is cut to this many bytes, so the two files
/// together stay a little over 512 KB at most.
pub const ENTRY_MAX: usize = 4000;

/// Writes `entry` (one event, possibly several lines) to the end of the log in `dir`, starting a new
/// file first if the current one is over `rotate_at` bytes. Returns whether it was written.
pub fn append(dir: &Path, rotate_at: u64, entry: &str) -> bool {
    let mut text: String = if entry.len() > ENTRY_MAX {
        let mut end = ENTRY_MAX;
        while !entry.is_char_boundary(end) {
            end -= 1;
        }
        entry[..end].to_string()
    } else {
        entry.to_string()
    };
    text.push('\n');
    (|| -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        let file = dir.join(LOG_FILE);
        if fs::metadata(&file).map(|m| m.len()).unwrap_or(0) > rotate_at {
            let old = dir.join(OLD_LOG_FILE);
            let _ = fs::remove_file(&old);
            fs::rename(&file, &old)?;
        }
        let mut f = OpenOptions::new().create(true).append(true).open(&file)?;
        f.write_all(text.as_bytes())?;
        f.flush()
    })()
    .is_ok()
}

/// Seconds since 1970 from the system clock (0 if the clock is before 1970).
pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// `2026-10-29T07:45:00Z` for a number of seconds since 1970 (UTC, civil calendar).
pub fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

pub fn now_utc() -> String {
    format_utc(now_secs())
}

/// A time stamp exactly as `format_utc` writes it, else nothing.
fn time_only(s: &str) -> String {
    entry_time(s).filter(|t| t.len() == s.len()).unwrap_or_default()
}

/// A version number (digits and dots), else nothing.
fn version_only(s: &str) -> String {
    if !s.is_empty() && s.len() <= 20 && s.chars().all(|c| c.is_ascii_digit() || c == '.') { s.to_string() } else { String::new() }
}

/// What the "running" flag from an earlier run says.
#[derive(Debug, PartialEq, Eq)]
pub struct Flag {
    pub started: String,
    pub version: String,
}

pub fn read_flag(dir: &Path) -> Option<Flag> {
    let text = fs::read_to_string(dir.join(FLAG_FILE)).ok()?;
    let mut flag = Flag { started: String::new(), version: String::new() };
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("started=") {
            flag.started = time_only(v.trim());
        } else if let Some(v) = line.strip_prefix("version=") {
            flag.version = version_only(v.trim());
        }
    }
    Some(flag)
}

pub fn write_flag(dir: &Path, started: &str, version: &str) -> bool {
    (|| -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join(FLAG_FILE), format!("started={started}\nversion={version}\n"))
    })()
    .is_ok()
}

pub fn remove_flag(dir: &Path) {
    let _ = fs::remove_file(dir.join(FLAG_FILE));
}

/// Size in bytes of the log files together.
pub fn total_bytes(dir: &Path) -> u64 {
    [LOG_FILE, OLD_LOG_FILE].iter().filter_map(|n| fs::metadata(dir.join(n)).ok()).map(|m| m.len()).sum()
}

/// The time stamp of the newest entry (the start of the last line that begins with one), read from the
/// end of the newest file that has one.
pub fn last_entry_time(dir: &Path) -> Option<String> {
    for name in [LOG_FILE, OLD_LOG_FILE] {
        let Ok(mut f) = fs::File::open(dir.join(name)) else { continue };
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        let from = len.saturating_sub(16 * 1024);
        if f.seek(SeekFrom::Start(from)).is_err() {
            continue;
        }
        let mut buf = Vec::new();
        if f.take(16 * 1024).read_to_end(&mut buf).is_err() {
            continue;
        }
        let text = String::from_utf8_lossy(&buf);
        if let Some(t) = text.lines().rev().filter_map(entry_time).next() {
            return Some(t);
        }
    }
    None
}

/// The time stamp at the start of a log line, if it has one (`2026-10-29T07:45:00Z ...`).
fn entry_time(line: &str) -> Option<String> {
    let head = line.get(..20)?;
    let b = head.as_bytes();
    let shape = b"dddd-dd-ddTdd:dd:ddZ";
    let ok = b.iter().zip(shape).all(|(c, s)| if *s == b'd' { c.is_ascii_digit() } else { c == s });
    ok.then(|| head.to_string())
}

/// Limits how many entries of one kind are written: at most `per_minute` in any one minute and
/// `per_run` since the program started, so a UI stuck in an error loop cannot fill the disk or slow
/// the app down.
pub struct Limiter {
    per_minute: u32,
    per_run: u32,
    window_start: u64,
    in_window: u32,
    total: u32,
    told_window: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    /// Not written. `first` is true for the first refusal of this minute or of the run, when a
    /// single "too many errors" line is written instead.
    Refuse { first: bool },
}

impl Limiter {
    pub const fn new(per_minute: u32, per_run: u32) -> Limiter {
        Limiter { per_minute, per_run, window_start: 0, in_window: 0, total: 0, told_window: false }
    }

    pub fn check(&mut self, now: u64) -> Verdict {
        if self.total >= self.per_run {
            let first = self.total == self.per_run;
            self.total = self.total.saturating_add(1);
            return Verdict::Refuse { first };
        }
        if now.saturating_sub(self.window_start) >= 60 {
            self.window_start = now;
            self.in_window = 0;
            self.told_window = false;
        }
        if self.in_window >= self.per_minute {
            let first = !self.told_window;
            self.told_window = true;
            return Verdict::Refuse { first };
        }
        self.in_window += 1;
        self.total += 1;
        Verdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("notaya-log-{}", crate::model::new_id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn utc_time_stamps_are_right() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_793_259_900), "2026-10-29T07:45:00Z");
        assert_eq!(format_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }

    #[test]
    fn entries_are_appended_one_per_line() {
        let dir = temp_dir();
        assert!(append(&dir, ROTATE_AT, "first"));
        assert!(append(&dir, ROTATE_AT, "second"));
        assert_eq!(fs::read_to_string(dir.join(LOG_FILE)).unwrap(), "first\nsecond\n");
    }

    #[test]
    fn a_full_file_becomes_the_old_file_and_the_old_old_file_is_replaced() {
        let dir = temp_dir();
        let line = "x".repeat(99);
        // 100 bytes per entry, rotate above 1000.
        for _ in 0..11 {
            append(&dir, 1000, &line);
        }
        assert!(!dir.join(OLD_LOG_FILE).exists(), "1100 bytes written, but the check is before the write");
        append(&dir, 1000, "after");
        assert!(dir.join(OLD_LOG_FILE).exists());
        assert_eq!(fs::read_to_string(dir.join(LOG_FILE)).unwrap(), "after\n");
        let first_old = fs::metadata(dir.join(OLD_LOG_FILE)).unwrap().len();
        assert_eq!(first_old, 1100);
        // A second rotation replaces the .1 file: the oldest entries are gone.
        for _ in 0..11 {
            append(&dir, 1000, "y");
        }
        for _ in 0..600 {
            append(&dir, 1000, &line);
        }
        let old = fs::read_to_string(dir.join(OLD_LOG_FILE)).unwrap();
        assert!(!old.contains("after"), "the first .1 file was replaced");
    }

    #[test]
    fn the_total_size_stays_under_the_cap() {
        let dir = temp_dir();
        let big = "z".repeat(3000);
        for _ in 0..400 {
            append(&dir, ROTATE_AT, &big);
        }
        let total = total_bytes(&dir);
        assert!(total <= 2 * ROTATE_AT + 2 * 3001, "{total}");
        assert!(total > ROTATE_AT, "{total}");
        let files: Vec<_> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(files.len(), 2, "{files:?}");
    }

    #[test]
    fn one_entry_is_cut_to_the_entry_cap() {
        let dir = temp_dir();
        append(&dir, ROTATE_AT, &"é".repeat(10_000));
        let len = fs::metadata(dir.join(LOG_FILE)).unwrap().len() as usize;
        assert!(len <= ENTRY_MAX + 1, "{len}");
    }

    #[test]
    fn a_failed_write_does_not_panic() {
        // A file where the folder should be: the folder cannot be made.
        let dir = temp_dir();
        let blocked = dir.join("logs");
        fs::write(&blocked, b"a file").unwrap();
        assert!(!append(&blocked, ROTATE_AT, "lost"));
        assert!(!write_flag(&blocked, "t", "v"));
        remove_flag(&blocked);
        assert_eq!(read_flag(&blocked), None);
        assert_eq!(total_bytes(&blocked), 0);
        assert_eq!(last_entry_time(&blocked), None);
        // A folder where the log file should be: opening it fails.
        let dir2 = temp_dir();
        fs::create_dir(dir2.join(LOG_FILE)).unwrap();
        assert!(!append(&dir2, ROTATE_AT, "lost"));
    }

    #[test]
    fn the_flag_round_trips_and_is_removed() {
        let dir = temp_dir();
        assert_eq!(read_flag(&dir), None);
        assert!(write_flag(&dir, "2026-10-29T07:45:00Z", "1.4.0"));
        assert_eq!(read_flag(&dir), Some(Flag { started: "2026-10-29T07:45:00Z".into(), version: "1.4.0".into() }));
        remove_flag(&dir);
        assert_eq!(read_flag(&dir), None);
        // Anything odd in the file is dropped: only a time stamp and a version number are accepted.
        fs::write(dir.join(FLAG_FILE), "started=Dear diary\nFAKE ERROR\nversion=<b>1.0</b>\n").unwrap();
        assert_eq!(read_flag(&dir), Some(Flag { started: String::new(), version: String::new() }));
    }

    #[test]
    fn the_last_entry_time_is_read_from_the_end() {
        let dir = temp_dir();
        assert_eq!(last_entry_time(&dir), None);
        append(&dir, ROTATE_AT, "2026-10-29T07:45:00Z INFO one");
        append(&dir, ROTATE_AT, "2026-10-30T08:00:01Z ERROR two\n    continuation line");
        assert_eq!(last_entry_time(&dir).as_deref(), Some("2026-10-30T08:00:01Z"));
    }

    #[test]
    fn the_limiter_allows_20_a_minute_and_200_a_run() {
        let mut l = Limiter::new(20, 200);
        for i in 0..20 {
            assert_eq!(l.check(1000 + i), Verdict::Allow);
        }
        assert_eq!(l.check(1030), Verdict::Refuse { first: true });
        assert_eq!(l.check(1031), Verdict::Refuse { first: false });
        // A minute later it allows again.
        assert_eq!(l.check(1061), Verdict::Allow);
        // 200 per run in total.
        let mut l = Limiter::new(20, 200);
        let mut allowed = 0;
        let mut first_refusals = 0;
        for minute in 0..30u64 {
            for _ in 0..50 {
                match l.check(minute * 61) {
                    Verdict::Allow => allowed += 1,
                    Verdict::Refuse { first: true } => first_refusals += 1,
                    Verdict::Refuse { .. } => {}
                }
            }
        }
        assert_eq!(allowed, 200);
        assert!(first_refusals >= 1);
        assert_eq!(l.check(10_000_000), Verdict::Refuse { first: false });
    }
}
