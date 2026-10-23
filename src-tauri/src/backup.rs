//! Backups of the vault (the logic that doesn't need a window).
//!
//! The rules: a backup is written to a temporary file next to its final place,
//! checked with the session's key (every record and attachment must authenticate
//! and the header must match), flushed to disk, and only then renamed to its final
//! name. A failed or half-written copy never exists under a final name, so it can
//! never be counted, listed for a restore or push a good copy out. Rotation keeps
//! the newest copies by modified time (the number in the name breaks a tie), never
//! deletes the file just written, and ignores anything it does not clearly own.
//!
//! There are two separate rotations in the backups folder. The automatic backups are
//! `vault-<digits>.clf` (10 kept). The safety backups made just before a notebook or section
//! is deleted are `vault-predelete-<digits>.clf` (5 kept). Each rotation counts and deletes
//! only its own files, so a run of deletes can't push out the ordinary backups, and the
//! ordinary ones can't push out the copy that still holds what was just deleted.

use std::cmp::Ordering;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::model::now_ms;
use crate::portable::Paths;
use crate::restore::rename_retry;
use crate::vault::Session;

/// How many automatic backups are kept.
pub const MAX_BACKUPS: usize = 10;
/// How many safety backups (made before a notebook or section is deleted) are kept.
pub const MAX_SAFETY_BACKUPS: usize = 5;

/// The file-name start of the automatic backups, and of the safety backups.
const AUTOMATIC: &str = "vault-";
const PREDELETE: &str = "vault-predelete-";

/// A rename that fails is tried again: 5 tries, 250 ms apart (about 1 s in all), because an
/// antivirus or sync tool may still hold a file that was closed a moment ago. Short enough
/// that a lock is not held up by a stubborn file.
pub const RENAME_TRIES: u32 = 5;
pub const RENAME_PAUSE: Duration = Duration::from_millis(250);

/// A modified time more than this far ahead of the PC's clock can't be real (the clock was
/// wrong when the file was written), so such a file does not count as a recent backup.
const FUTURE_SLACK_MS: u64 = 24 * 60 * 60 * 1000;

/// Lets only one backup run at a time, so cleaning up stray temporary files can
/// never remove the file another backup is writing.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// `vault-<digits>.<ext>` gives the digits (`ext` includes the dot). Anything else is not ours
/// (a safety backup, `vault-predelete-<digits>`, is not an automatic one).
#[cfg(test)]
fn number_in_name(name: &str, ext: &str) -> Option<u64> {
    number_with_prefix(name, AUTOMATIC, ext)
}

/// `<prefix><digits><ext>` gives the digits.
fn number_with_prefix(name: &str, prefix: &str, ext: &str) -> Option<u64> {
    let digits = name.strip_prefix(prefix)?.strip_suffix(ext)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// One automatic backup as rotation sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate {
    name: String,
    modified_ms: u64,
    number: u64,
}

/// Which names to delete: the oldest candidates beyond `keep`, counting `just_made`
/// (which is never deleted, whatever its time or number). Newest means the latest
/// modified time; equal times fall back to the number in the name, then the name.
///
/// A file whose modified time is more than a day later than `clock_ms` (the PC's clock) was
/// written while the clock was wrong. It must not outrank the real backups, or a PC with a
/// wrong clock could leave only one real backup: such files rank below every real one,
/// among themselves by the number in the name.
fn plan_rotation(mut others: Vec<Candidate>, just_made: &str, keep: usize, clock_ms: u64) -> Vec<String> {
    let limit = clock_ms.saturating_add(FUTURE_SLACK_MS);
    others.retain(|c| c.name != just_made);
    others.sort_by(|a, b| {
        let (future_a, future_b) = (a.modified_ms > limit, b.modified_ms > limit);
        future_a
            .cmp(&future_b)
            .then_with(|| if future_a { Ordering::Equal } else { b.modified_ms.cmp(&a.modified_ms) })
            .then_with(|| b.number.cmp(&a.number))
            .then_with(|| b.name.cmp(&a.name))
    });
    let room = keep.saturating_sub(1); // `just_made` takes one place
    others.into_iter().skip(room).map(|c| c.name).collect()
}

/// The PC's clock in milliseconds since 1970 (0 if it can't be read).
fn clock_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Rotates `dir` after `just_made` was written. Only files named exactly
/// `<prefix><digits>.clf` are looked at (the other kind of backup is never touched). A file that is empty counts for nothing (it can
/// only be junk from an older version that wrote straight to the final name) and is
/// removed; a file whose details can't be read is left alone and not counted.
fn rotate(dir: &Path, prefix: &str, just_made: &str, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut others = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Some(number) = number_with_prefix(&name, prefix, ".clf") else {
            continue;
        };
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() || name == just_made {
            continue;
        }
        if meta.len() == 0 {
            if let Err(e) = fs::remove_file(entry.path()) {
                eprintln!("backup: could not remove the empty file {name}: {e}");
            }
            continue;
        }
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        others.push(Candidate { name, modified_ms, number });
    }
    for old in plan_rotation(others, just_made, keep, clock_ms()) {
        if let Err(e) = fs::remove_file(dir.join(&old)) {
            eprintln!("backup: could not remove {old}: {e}");
        }
    }
}

/// Removes leftovers of a backup that was cut short (`vault-<digits>.tmp`, or
/// `vault-predelete-<digits>.tmp` for a safety backup).
fn clean_strays(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let is_stray = entry
            .file_name()
            .to_str()
            .is_some_and(|n| number_with_prefix(n, AUTOMATIC, ".tmp").or(number_with_prefix(n, PREDELETE, ".tmp")).is_some());
        if is_stray && entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Writes `session` to `tmp`, checks it, flushes it to disk and renames it to `dest`.
/// On an error `tmp` may be left behind; the caller removes it. `dest` is replaced
/// only by the last step, and only by a copy that passed the check.
fn write_checked(session: &Session, tmp: &Path, dest: &Path) -> Result<()> {
    session.backup_to(tmp)?;
    session.verify_copy(tmp)?;
    OpenOptions::new().write(true).open(tmp)?.sync_all()?;
    // About a second in all: a lock must not be held up for long by a stubborn file.
    rename_retry(tmp, dest, RENAME_TRIES, RENAME_PAUSE)?;
    Ok(())
}

/// Makes an automatic backup `vault-<ms>.clf` in the backups folder and rotates. Returns
/// the file name. Nothing exists under the final name unless the copy passed its check.
pub fn make_backup(paths: &Paths, session: &Session) -> Result<String> {
    make_backup_in(&paths.backups, session, now_ms().max(0) as u64, MAX_BACKUPS)
}

/// The safety backup before a notebook or section is deleted: `vault-predelete-<ms>.clf`,
/// checked like any backup, in a rotation of its own (the newest `MAX_SAFETY_BACKUPS` are
/// kept). It never counts toward, and never deletes, the automatic backups, and the
/// automatic rotation never touches it. Returns the file name.
pub fn make_safety_backup(paths: &Paths, session: &Session) -> Result<String> {
    make_in(&paths.backups, session, now_ms().max(0) as u64, PREDELETE, MAX_SAFETY_BACKUPS)
}

fn make_backup_in(dir: &Path, session: &Session, now: u64, keep: usize) -> Result<String> {
    make_in(dir, session, now, AUTOMATIC, keep)
}

fn make_in(dir: &Path, session: &Session, now: u64, prefix: &str, keep: usize) -> Result<String> {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    fs::create_dir_all(dir)?;
    clean_strays(dir);
    // `rename` replaces an existing file on Windows, so take a name that is free.
    let mut ms = now;
    let (tmp, dest, name) = loop {
        let name = format!("{prefix}{ms}.clf");
        let dest = dir.join(&name);
        let tmp = dir.join(format!("{prefix}{ms}.tmp"));
        if !dest.try_exists()? && !tmp.try_exists()? {
            break (tmp, dest, name);
        }
        ms += 1;
    };
    if let Err(e) = write_checked(session, &tmp, &dest) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    rotate(dir, prefix, &name, keep);
    Ok(name)
}

/// The manual "Save a backup copy": the same steps, with the copy written next to `dest`.
/// An existing `dest` is replaced only after the new copy passed its check, and the live
/// vault is refused as a destination.
pub fn save_copy(session: &Session, dest: &Path, live_vault: &Path) -> Result<()> {
    if same_file(dest, live_vault) {
        return Err(Error::invalid(
            "That is the file Cipherleaf is using for your notes right now. Choose a different name or folder for the backup copy.",
        ));
    }
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let file = dest.file_name().and_then(|n| n.to_str()).ok_or_else(|| Error::invalid("Unsupported backup path."))?;
    let tmp = dest.with_file_name(format!("{file}.{}.tmp", now_ms()));
    if let Err(e) = write_checked(session, &tmp, dest) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// The path with symbolic links and `..` resolved; a file that doesn't exist yet is
/// resolved through its folder.
fn resolve(p: &Path) -> PathBuf {
    fs::canonicalize(p)
        .or_else(|_| {
            let dir = p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
            let name = p.file_name().ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            fs::canonicalize(dir).map(|d| d.join(name))
        })
        .unwrap_or_else(|_| p.to_path_buf())
}

fn same_file(a: &Path, b: &Path) -> bool {
    resolve(a) == resolve(b)
}

/// Plain words for why a backup failed (no raw error codes where a plain one exists).
pub fn reason(e: &Error) -> String {
    let full = match e {
        Error::Io(io_err) => {
            // 112 = disk full, 39 = handle disk full.
            io_err.kind() == io::ErrorKind::StorageFull || matches!(io_err.raw_os_error(), Some(112 | 39))
        }
        Error::Db(rusqlite::Error::SqliteFailure(f, _)) => f.code == rusqlite::ErrorCode::DiskFull,
        _ => false,
    };
    if full {
        "the drive is full".to_string()
    } else {
        e.to_string()
    }
}

/// The message shown when an automatic backup could not be made (lock, quit, hourly).
///
/// What it says about the notes depends on the cause, and never promises more than is known:
/// - the vault file is gone (`vault_present` false, for example the drive was removed): no
///   reassurance, and a hint to check the drive;
/// - the new copy failed its integrity check (`Error::Corrupt`): the vault file or the drive
///   may be damaged, so the person is told not to rely on the vault file alone;
/// - anything else (a full drive, a locked folder): the vault file is untouched by a failed
///   copy, so it is still saved.
pub fn failure_notice(e: &Error, vault_present: bool) -> String {
    let because = reason(e);
    if !vault_present {
        format!(
            "Cipherleaf could not make a backup copy of your notes ({because}). The vault file data\\vault.clf cannot be found, so check that the drive is connected."
        )
    } else if matches!(e, Error::Corrupt) {
        "Cipherleaf could not make a backup copy of your notes: the copy failed its integrity check, so the vault file data\\vault.clf or the drive it is on may be damaged. Do not rely on it alone. Keep your older backups, and if your notes look wrong, restore one of them.".to_string()
    } else {
        format!(
            "Cipherleaf could not make a backup copy of your notes ({because}). Your notes are still saved in data\\vault.clf."
        )
    }
}

/// The error returned when the safety backup before a bulk delete failed.
pub fn safety_failure(e: &Error) -> Error {
    Error::Other(format!("Cipherleaf could not make a safety backup first, so nothing was deleted: {}", reason(e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::new_id;
    use crate::vault::Session;
    use rusqlite::{params, Connection};
    use std::time::SystemTime;

    /// A clock that is later than every modified time the plan tests use.
    const CLOCK: u64 = 1_000_000;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cipherleaf-backup-{}", new_id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn session_in(dir: &Path) -> Session {
        let mut s = Session::create_open(&dir.join("vault.clf")).unwrap();
        s.put_attachment("image/png", b"pixels").unwrap();
        s
    }

    fn set_time(path: &Path, t: SystemTime) {
        OpenOptions::new().write(true).open(path).unwrap().set_modified(t).unwrap();
    }

    fn secs_ago(path: &Path, secs: u64) {
        set_time(path, SystemTime::now() - Duration::from_secs(secs));
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, bytes).unwrap();
        p
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> =
            fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
        v.sort();
        v
    }

    fn cand(name: &str, modified_ms: u64) -> Candidate {
        Candidate { name: name.into(), modified_ms, number: number_in_name(name, ".clf").unwrap_or(0) }
    }

    /// Damages a record in the live file, so a copy of it fails the check.
    fn damage_live_vault(dir: &Path) {
        let conn = Connection::open(dir.join("vault.clf")).unwrap();
        let (id, mut data): (Vec<u8>, Vec<u8>) =
            conn.query_row("SELECT id, data FROM records LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        data[40] ^= 0xff;
        conn.execute("UPDATE records SET data = ?1 WHERE id = ?2", params![data, id]).unwrap();
    }

    #[test]
    fn only_exact_names_are_ours() {
        assert_eq!(number_in_name("vault-123.clf", ".clf"), Some(123));
        assert_eq!(number_in_name("vault-123.tmp", ".tmp"), Some(123));
        for bad in
            ["vault-.clf", "vault-1a.clf", "vault-old.clf", "other-vault-1.clf", "vault-1.clf.bak", "vault-1.tmp"]
        {
            assert_eq!(number_in_name(bad, ".clf"), None, "{bad}");
        }
        assert_eq!(number_in_name("vault-99999999999999999999999.clf", ".clf"), None, "too long for a number");
    }

    #[test]
    fn the_plan_keeps_the_newest_by_modified_time_not_by_name() {
        // Names say 1..=4, times say the opposite for 1 and 4.
        let others = vec![
            cand("vault-1.clf", 400),
            cand("vault-2.clf", 200),
            cand("vault-3.clf", 300),
            cand("vault-4.clf", 100),
        ];
        let gone = plan_rotation(others, "vault-9.clf", 3, CLOCK); // just_made + 2 more
        assert_eq!(gone, ["vault-2.clf", "vault-4.clf"]);
    }

    #[test]
    fn equal_times_fall_back_to_the_number_in_the_name() {
        let others = vec![cand("vault-5.clf", 100), cand("vault-7.clf", 100), cand("vault-6.clf", 100)];
        assert_eq!(plan_rotation(others, "vault-9.clf", 3, CLOCK), ["vault-5.clf"]);
    }

    #[test]
    fn the_plan_never_deletes_the_file_just_made() {
        // Even when its time is the oldest, and even with keep = 1.
        let others = vec![cand("vault-1.clf", 1), cand("vault-2.clf", 5000), cand("vault-3.clf", 6000)];
        let gone = plan_rotation(others, "vault-1.clf", 1, CLOCK);
        assert_eq!(gone, ["vault-3.clf", "vault-2.clf"]);
        assert!(!gone.contains(&"vault-1.clf".to_string()));
        assert!(plan_rotation(vec![], "vault-1.clf", 10, CLOCK).is_empty());
    }

    #[test]
    fn a_backup_is_checked_and_complete_and_leaves_no_temp_file() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        let name = make_backup_in(&backups, &s, 1_700_000_000_000, MAX_BACKUPS).unwrap();
        assert_eq!(name, "vault-1700000000000.clf");
        assert_eq!(names(&backups), [name.clone()]);
        s.verify_copy(&backups.join(&name)).unwrap();
        // Two backups in the same millisecond get two names.
        let second = make_backup_in(&backups, &s, 1_700_000_000_000, MAX_BACKUPS).unwrap();
        assert_ne!(second, name);
        assert_eq!(names(&backups).len(), 2);
    }

    #[test]
    fn a_wrong_clock_cannot_push_out_good_backups() {
        // The audit's repro: ten files named from 1790000000000, then a new one made with a
        // clock that says 1704067200000. The new file must survive and the oldest must go.
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        for i in 0..10u64 {
            let p = write(&backups, &format!("vault-{}.clf", 1_790_000_000_000 + i), b"old backup");
            secs_ago(&p, 10_000 - i * 100); // i = 0 is the oldest
        }
        let name = make_backup_in(&backups, &s, 1_704_067_200_000, MAX_BACKUPS).unwrap();
        let left = names(&backups);
        assert_eq!(left.len(), MAX_BACKUPS);
        assert!(left.contains(&name), "the new backup must be kept: {left:?}");
        assert!(!left.contains(&"vault-1790000000000.clf".to_string()), "the oldest by time goes");
        assert!(left.contains(&"vault-1790000000009.clf".to_string()));
    }

    #[test]
    fn a_future_dated_file_ranks_below_every_real_backup() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        let future = write(&backups, "vault-1.clf", b"from the future");
        set_time(&future, SystemTime::now() + Duration::from_secs(10 * 365 * 86_400));
        for i in 0..9u64 {
            let p = write(&backups, &format!("vault-{}.clf", 100 + i), b"backup");
            secs_ago(&p, 5_000 - i * 100);
        }
        let name = make_backup_in(&backups, &s, 5, MAX_BACKUPS).unwrap();
        let left = names(&backups);
        assert_eq!(left.len(), MAX_BACKUPS);
        assert!(left.contains(&name));
        assert!(!left.contains(&"vault-1.clf".to_string()), "the file with the impossible time goes first");
        for i in 0..9u64 {
            assert!(left.contains(&format!("vault-{}.clf", 100 + i)), "real backup {i} stays: {left:?}");
        }
    }

    #[test]
    fn the_plan_puts_future_dated_files_below_real_ones() {
        let day = FUTURE_SLACK_MS;
        let others = vec![
            cand("vault-50.clf", CLOCK + 5 * day), // future, bigger number
            cand("vault-40.clf", CLOCK + 9 * day), // future, later time but smaller number
            cand("vault-10.clf", 100),
            cand("vault-20.clf", 200),
            cand("vault-30.clf", CLOCK + day), // exactly one day ahead is still allowed (a real time)
        ];
        // Keep 3 (just_made + 2): the two newest real ones win (30, 20); 10 is the oldest real;
        // the future ones rank below it, larger number first.
        let gone = plan_rotation(others, "vault-99.clf", 3, CLOCK);
        assert_eq!(gone, ["vault-10.clf", "vault-50.clf", "vault-40.clf"]);
    }

    #[test]
    fn nine_future_dated_files_cannot_crowd_out_new_backups() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        for i in 0..9u64 {
            let p = write(&backups, &format!("vault-{}.clf", 1_790_000_000_000 + i), b"wrong clock");
            set_time(&p, SystemTime::now() + Duration::from_secs((30 + i) * 86_400));
        }
        let mut made = Vec::new();
        for i in 0..5u64 {
            made.push(make_backup_in(&backups, &s, 1_000 + i, MAX_BACKUPS).unwrap());
            let left = names(&backups);
            assert_eq!(left.len(), 10usize.min(9 + made.len()), "{left:?}");
            for m in &made {
                assert!(left.contains(m), "{m} must stay after backup {i}: {left:?}");
            }
        }
        // Keep going: the wrong-clock files all leave before any real backup does.
        for i in 5..12u64 {
            made.push(make_backup_in(&backups, &s, 1_000 + i, MAX_BACKUPS).unwrap());
        }
        let left = names(&backups);
        assert_eq!(left.len(), MAX_BACKUPS);
        assert!(left.iter().all(|n| !n.starts_with("vault-1790")), "{left:?}");
        for m in &made[made.len() - MAX_BACKUPS..] {
            assert!(left.contains(m), "{m}: {left:?}");
        }
    }

    #[test]
    fn empty_and_unrelated_files_do_not_count_and_nothing_good_is_removed() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        for i in 0..9u64 {
            let p = write(&backups, &format!("vault-{}.clf", 100 + i), b"backup");
            secs_ago(&p, 5_000 - i * 100);
        }
        // Junk from the old logic: 0-byte files under the final name.
        write(&backups, "vault-50.clf", b"");
        write(&backups, "vault-51.clf", b"");
        // Not ours: never counted, never touched (even empty).
        write(&backups, "vault-old.clf", b"");
        write(&backups, "vault-keep-me.clf", b"maybe a good vault");
        write(&backups, "notes.txt", b"x");
        write(&backups, "passwords-5.kdbx", b"x");
        fs::create_dir(backups.join("vault-777.clf.dir")).unwrap();
        let name = make_backup_in(&backups, &s, 10_000, MAX_BACKUPS).unwrap();
        let left = names(&backups);
        // 9 good + the new one = 10: no good backup was removed because of the junk.
        for i in 0..9u64 {
            assert!(left.contains(&format!("vault-{}.clf", 100 + i)), "{left:?}");
        }
        assert!(left.contains(&name));
        assert!(!left.contains(&"vault-50.clf".to_string()) && !left.contains(&"vault-51.clf".to_string()));
        for kept in ["vault-old.clf", "vault-keep-me.clf", "notes.txt", "passwords-5.kdbx"] {
            assert!(left.contains(&kept.to_string()), "{kept} must stay");
        }
    }

    #[test]
    fn stray_temp_files_are_cleaned_but_other_temp_files_are_not() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        write(&backups, "vault-5.tmp", b"half");
        write(&backups, "other.tmp", b"mine");
        make_backup_in(&backups, &s, 10, MAX_BACKUPS).unwrap();
        let left = names(&backups);
        assert!(!left.contains(&"vault-5.tmp".to_string()));
        assert!(left.contains(&"other.tmp".to_string()));
    }

    #[test]
    fn a_backup_that_fails_its_check_leaves_nothing_and_removes_no_good_backup() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        let first = make_backup_in(&backups, &s, 100, MAX_BACKUPS).unwrap();
        let mut keep = vec![first];
        for i in 0..9u64 {
            let p = write(&backups, &format!("vault-{}.clf", 200 + i), b"backup");
            secs_ago(&p, 5_000 - i * 100);
            keep.push(format!("vault-{}.clf", 200 + i));
        }
        keep.sort();
        damage_live_vault(&dir);
        let err = make_backup_in(&backups, &s, 999, MAX_BACKUPS);
        assert!(err.is_err(), "a copy that does not authenticate is a failure");
        assert_eq!(names(&backups), keep, "no temp file, no half file, nothing rotated out");
    }

    #[test]
    fn a_backup_into_an_unusable_folder_fails_cleanly() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let blocker = write(&dir, "backups", b"a file where the folder should be");
        assert!(make_backup_in(&blocker, &s, 1, MAX_BACKUPS).is_err());
        assert_eq!(fs::read(&blocker).unwrap(), b"a file where the folder should be");
    }

    #[test]
    fn a_manual_copy_replaces_an_old_file_only_after_the_new_one_is_good() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let live = dir.join("vault.clf");
        let dest = write(&dir, "mine.clf", b"OLD BACKUP");
        // A failing copy leaves the old file untouched and no temp file.
        damage_live_vault(&dir);
        assert!(save_copy(&s, &dest, &live).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"OLD BACKUP");
        assert_eq!(names(&dir), ["mine.clf", "vault.clf"]);
        // A good copy replaces it.
        let dir2 = temp_dir();
        let s2 = session_in(&dir2);
        let dest2 = write(&dir2, "mine.clf", b"OLD BACKUP");
        save_copy(&s2, &dest2, &dir2.join("vault.clf")).unwrap();
        s2.verify_copy(&dest2).unwrap();
        assert_eq!(names(&dir2), ["mine.clf", "vault.clf"], "no temp file left");
    }

    #[test]
    fn a_manual_copy_refuses_the_live_vault_and_leaves_it_alone() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let live = dir.join("vault.clf");
        let before = fs::read(&live).unwrap();
        assert!(save_copy(&s, &live, &live).is_err());
        // The same file spelled another way.
        let sneaky = dir.join(".").join("sub").join("..").join("vault.clf");
        fs::create_dir_all(dir.join("sub")).unwrap();
        assert!(save_copy(&s, &sneaky, &live).is_err());
        assert_eq!(fs::read(&live).unwrap(), before);
        assert_eq!(names(&dir), ["sub", "vault.clf"]);
        // A new file that does not exist yet is fine.
        save_copy(&s, &dir.join("new.clf"), &live).unwrap();
    }

    #[test]
    fn a_manual_copy_onto_a_folder_fails_and_leaves_no_temp_file() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let target = dir.join("is-a-folder.clf");
        fs::create_dir(&target).unwrap();
        assert!(save_copy(&s, &target, &dir.join("vault.clf")).is_err());
        assert_eq!(names(&dir), ["is-a-folder.clf", "vault.clf"]);
    }

    #[test]
    fn the_messages_are_plain() {
        let full = Error::Io(io::Error::from(io::ErrorKind::StorageFull));
        assert_eq!(reason(&full), "the drive is full");
        assert_eq!(
            failure_notice(&full, true),
            "Cipherleaf could not make a backup copy of your notes (the drive is full). Your notes are still saved in data\\vault.clf."
        );
        assert_eq!(
            safety_failure(&full).to_string(),
            "Cipherleaf could not make a safety backup first, so nothing was deleted: the drive is full"
        );
        assert_eq!(reason(&Error::Corrupt), "The vault file is damaged or has been tampered with.");
    }

    #[test]
    fn the_notice_does_not_promise_what_is_not_known() {
        let full = Error::Io(io::Error::from(io::ErrorKind::StorageFull));
        let gone = Error::Io(io::Error::from(io::ErrorKind::NotFound));

        // Vault file missing (drive removed): no reassurance, a hint about the drive.
        for e in [&full, &gone, &Error::Corrupt] {
            let text = failure_notice(e, false);
            assert!(!text.contains("still saved"), "{text}");
            assert!(text.contains("cannot be found") && text.contains("drive is connected"), "{text}");
        }

        // The copy failed its integrity check: not reassuring, points at older backups.
        let text = failure_notice(&Error::Corrupt, true);
        assert!(!text.contains("still saved"), "{text}");
        assert!(text.contains("integrity check") && text.contains("Do not rely on it alone"), "{text}");
        assert!(text.contains("older backups") && text.contains("restore"), "{text}");

        // Ordinary causes with the vault in place keep the reassurance.
        for e in [&full, &gone] {
            let text = failure_notice(e, true);
            assert!(text.ends_with("Your notes are still saved in data\\vault.clf."), "{text}");
        }
    }

    const _: () = assert!(RENAME_TRIES >= 5);

    #[test]
    fn the_rename_retry_is_about_a_second_in_all() {
        let total = RENAME_PAUSE * (RENAME_TRIES - 1);
        assert!(total >= Duration::from_millis(900) && total <= Duration::from_millis(1_200), "{total:?}");
    }

    #[test]
    fn safety_backups_have_their_own_names_and_rotation() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        // Ten ordinary backups, oldest first.
        let mut ordinary = Vec::new();
        for i in 0..10u64 {
            ordinary.push(make_backup_in(&backups, &s, 1_000 + i, MAX_BACKUPS).unwrap());
        }
        assert_eq!(names(&backups).len(), 10);
        // Eight safety backups: only the newest 5 stay, and no ordinary one is touched.
        let mut safety = Vec::new();
        for i in 0..8u64 {
            let name = make_in(&backups, &s, 2_000 + i, PREDELETE, MAX_SAFETY_BACKUPS).unwrap();
            assert_eq!(name, format!("vault-predelete-{}.clf", 2_000 + i));
            s.verify_copy(&backups.join(&name)).unwrap();
            safety.push(name);
        }
        let left = names(&backups);
        for o in &ordinary {
            assert!(left.contains(o), "{o} must stay: {left:?}");
        }
        for n in &safety[..3] {
            assert!(!left.contains(n), "{n} is the oldest safety backup and goes");
        }
        for n in &safety[3..] {
            assert!(left.contains(n), "{n}: {left:?}");
        }
        assert_eq!(left.len(), 15);
        // More ordinary backups rotate only the ordinary ones.
        let newer = make_backup_in(&backups, &s, 3_000, MAX_BACKUPS).unwrap();
        let left = names(&backups);
        assert_eq!(left.len(), 15, "{left:?}");
        assert!(left.contains(&newer) && !left.contains(&ordinary[0]));
        for n in &safety[3..] {
            assert!(left.contains(n), "the ordinary rotation must not touch {n}");
        }
    }

    #[test]
    fn stray_safety_temp_files_are_cleaned_and_junk_is_handled_like_for_ordinary_backups() {
        let dir = temp_dir();
        let s = session_in(&dir);
        let backups = dir.join("backups");
        fs::create_dir_all(&backups).unwrap();
        write(&backups, "vault-predelete-5.tmp", b"half");
        write(&backups, "vault-predelete-x.tmp", b"not ours");
        write(&backups, "vault-predelete-6.clf", b""); // junk: removed by the safety rotation
        write(&backups, "vault-predelete-old.clf", b""); // not ours: stays
        make_in(&backups, &s, 10, PREDELETE, MAX_SAFETY_BACKUPS).unwrap();
        let left = names(&backups);
        assert!(!left.contains(&"vault-predelete-5.tmp".to_string()));
        assert!(!left.contains(&"vault-predelete-6.clf".to_string()));
        assert!(left.contains(&"vault-predelete-x.tmp".to_string()));
        assert!(left.contains(&"vault-predelete-old.clf".to_string()));
        assert!(left.contains(&"vault-predelete-10.clf".to_string()));
    }

    #[test]
    fn a_safety_name_is_not_an_ordinary_one_and_the_other_way_round() {
        assert_eq!(number_in_name("vault-predelete-123.clf", ".clf"), None);
        assert_eq!(number_with_prefix("vault-predelete-123.clf", PREDELETE, ".clf"), Some(123));
        assert_eq!(number_with_prefix("vault-123.clf", PREDELETE, ".clf"), None);
        assert_eq!(number_with_prefix("vault-predelete-.clf", PREDELETE, ".clf"), None);
    }
}
