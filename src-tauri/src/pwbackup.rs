//! Copies of the KeePassXC passwords file, and restoring one.
//!
//! KeePassXC keeps the passwords in its own encrypted file (`data/passwords.kdbx`,
//! see `keepassxc::find_database`), protected by KeePassXC's master password.
//! Cipherleaf never opens it. This module only copies it, and the only thing it
//! reads of the contents is the 8-byte KeePassXC file signature, so it can tell
//! a database from a half-written or foreign file. It can't tell whether a file
//! is the right database or opens with the owner's password.
//!
//! Backups: on every Cipherleaf lock the file is copied to
//! `data/backups/passwords-<ms>.kdbx` if it differs from the newest copy; the
//! newest 10 are kept. Only files named exactly like that are listed or deleted.
//!
//! Restore: check first, never delete. The chosen file is copied to
//! `data/passwords-restore-check.tmp` and the signature of that copy is checked.
//! Then the live file is renamed to `data/passwords-undo-<ms>.kdbx` and the copy
//! renamed into its place. The app never deletes or rotates `passwords-undo-*`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::keepassxc::find_database;
use crate::portable::Paths;
use crate::restore::{self, BackupInfo};

/// How many copies of the passwords file are kept.
pub const KEEP: usize = 10;
/// Largest passwords file that is copied or restored.
const MAX_BYTES: u64 = 512 * 1024 * 1024;
/// The first 8 bytes of every KDBX file (two little-endian 32-bit signatures).
const SIGNATURE: [u8; 8] = [0x03, 0xD9, 0xA2, 0x9A, 0x67, 0xFB, 0x4B, 0xB5];
/// A copy being made on lock, before it is checked and renamed into place.
const BACKUP_TEMP: &str = "passwords-backup.tmp";
/// The chosen file, copied next to the live one before it is installed.
const RESTORE_TEMP: &str = "passwords-restore-check.tmp";
/// The old passwords file, kept when a restore replaces it (lowercase; see
/// `keepassxc::is_database`, which never takes such a file for the database in use).
pub const UNDO_PREFIX: &str = "passwords-undo-";

const NETWORK: &str = "Backups on a network or device path can't be used. Copy the file to this PC first.";
const DIALOG: &str = "Choose the backup file with the file dialog.";
const NOT_KDBX: &str = "That file doesn't look like a KeePassXC database, so it wasn't used.";
const IN_USE: &str = "The passwords file is in use by another program. Close KeePassXC and any program that may be using it (antivirus or a file-sync tool, for example) and try again.";

// ---------------------------------------------------------------- names

/// The digits of `passwords-<digits>.kdbx`, or `None` for any other name.
fn digits_of(name: &str) -> Option<&str> {
    let digits = name.strip_prefix("passwords-")?.strip_suffix(".kdbx")?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then_some(digits)
}

/// The automatic copies look like `passwords-<ms>.kdbx`.
pub fn is_backup_name(name: &str) -> bool {
    digits_of(name).is_some()
}

/// The number in a copy's name, as a sort key. It is the wall-clock time the copy
/// was made, so it is only the tie-breaker after the file's modification time.
fn age_key(name: &str) -> (usize, String) {
    let digits = digits_of(name).unwrap_or("").trim_start_matches('0');
    (digits.len(), digits.to_string())
}

/// The names of the copies in the backups folder, oldest first: by modification
/// time (the same order as [`list`]), equal times by the number in the name.
/// Regular files only.
fn copies(paths: &Paths) -> io::Result<Vec<String>> {
    let entries = match fs::read_dir(&paths.backups) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut found: Vec<(u64, String)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if !is_backup_name(&name) {
                return None;
            }
            let meta = e.metadata().ok().filter(|m| m.is_file())?;
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Some((modified_ms, name))
        })
        .collect();
    found.sort_by(|(ta, na), (tb, nb)| ta.cmp(tb).then_with(|| age_key(na).cmp(&age_key(nb))));
    Ok(found.into_iter().map(|(_, name)| name).collect())
}

/// What a file looked like at one moment: size and modification time.
type Stamp = (u64, Option<SystemTime>);

fn stamp_of(path: &Path) -> io::Result<Stamp> {
    let meta = fs::metadata(path)?;
    Ok((meta.len(), meta.modified().ok()))
}

/// Only one copy runs at a time (the temp file and the rotation are shared).
static BACKUP_LOCK: Mutex<()> = Mutex::new(());

/// The copies for the restore list, newest first. Reads only names, times and sizes.
pub fn list(paths: &Paths) -> Result<Vec<BackupInfo>> {
    restore::list_in(&paths.backups, is_backup_name)
}

// ---------------------------------------------------------------- file helpers

/// True if the file starts with the KDBX signature (and so is not empty).
fn has_signature(path: &Path) -> io::Result<bool> {
    let mut head = [0u8; 8];
    match File::open(path)?.read_exact(&mut head) {
        Ok(()) => Ok(head == SIGNATURE),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(e) => Err(e),
    }
}

/// Byte-for-byte comparison.
fn files_equal(a: &Path, b: &Path) -> io::Result<bool> {
    let (mut fa, mut fb) = (File::open(a)?, File::open(b)?);
    let len = fa.metadata()?.len();
    if len != fb.metadata()?.len() {
        return Ok(false);
    }
    let (mut buf_a, mut buf_b) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    let mut left = len;
    while left > 0 {
        let n = left.min(buf_a.len() as u64) as usize;
        fa.read_exact(&mut buf_a[..n])?;
        fb.read_exact(&mut buf_b[..n])?;
        if buf_a[..n] != buf_b[..n] {
            return Ok(false);
        }
        left -= n as u64;
    }
    Ok(true)
}

/// Copies `from` to the NEW file `to` and syncs it. Returns the number of bytes
/// copied; copying stops one byte past `max`, so a huge file isn't read to the end.
fn copy_new(from: &Path, to: &Path, max: u64) -> io::Result<u64> {
    let mut src = File::open(from)?;
    let mut dst = OpenOptions::new().write(true).create_new(true).open(to)?;
    let copied = io::copy(&mut (&mut src).take(max + 1), &mut dst)?;
    dst.sync_all()?;
    Ok(copied)
}

// ---------------------------------------------------------------- automatic copies

/// Called when Cipherleaf locks. Best effort: an error is printed and nothing
/// else happens, so it can never block or fail the lock.
pub fn backup_on_lock(paths: &Paths) {
    if let Err(e) = backup_if_changed(paths, crate::model::now_ms().max(0) as u64) {
        eprintln!("passwords backup failed: {e}");
    }
}

/// Copies the passwords file to `data/backups/passwords-<now_ms>.kdbx` unless it
/// is identical to the newest copy. The copy is made under a temporary name,
/// checked (signature, not empty) and only then renamed, so a file that
/// KeePassXC is in the middle of saving never becomes a "backup". Returns the
/// new copy's name, or `None` if nothing was copied.
pub fn backup_if_changed(paths: &Paths, now_ms: u64) -> Result<Option<String>> {
    // No file (for example the drive was removed): nothing to do, and nothing is created.
    let Some(db) = find_database(paths) else {
        return Ok(None);
    };
    // Two locks at once must not delete each other's temporary file or rotate together.
    let _one_at_a_time = BACKUP_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let before = stamp_of(&db)?;
    if before.0 == 0 || before.0 > MAX_BYTES {
        return Ok(None);
    }
    fs::create_dir_all(&paths.backups)?;
    let existing = copies(paths)?;
    if let Some(newest) = existing.last()
        && files_equal(&db, &paths.backups.join(newest)).unwrap_or(false)
    {
        return Ok(None);
    }

    let tmp = paths.backups.join(BACKUP_TEMP);
    let _ = fs::remove_file(&tmp);
    let copied = copy_new(&db, &tmp, MAX_BYTES);
    // If the file changed while it was being copied (KeePassXC saving), the copy may be torn.
    if copied.is_ok() && stamp_of(&db).ok() != Some(before) {
        let _ = fs::remove_file(&tmp);
        eprintln!("passwords backup skipped: the file changed while it was being copied");
        return Ok(None);
    }
    let checked = copied.and_then(|n| Ok(n > 0 && n <= MAX_BYTES && has_signature(&tmp)?));
    match checked {
        Ok(true) => {}
        Ok(false) => {
            let _ = fs::remove_file(&tmp);
            eprintln!("passwords backup skipped: the file is empty or is not a KeePassXC database (maybe mid-save)");
            return Ok(None);
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
    }

    // `rename` replaces an existing file on Windows, so take a name that is free.
    let mut ms = now_ms;
    let dest = loop {
        let p = paths.backups.join(format!("passwords-{ms}.kdbx"));
        if !p.try_exists()? {
            break p;
        }
        ms += 1;
    };
    // About a second in all: a lock must not be held up for long by a stubborn file.
    if let Err(e) = restore::rename_retry(&tmp, &dest, crate::backup::RENAME_TRIES, crate::backup::RENAME_PAUSE) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    let name = dest.file_name().and_then(|n| n.to_str()).map(str::to_string);
    if let Some(name) = &name {
        rotate(paths, name);
    }
    Ok(name)
}

/// Deletes the oldest copies (by modification time) beyond [`KEEP`], never
/// `just_made`, never any other file. Only names exactly like
/// `passwords-<digits>.kdbx` count: a file put in the backups folder by hand with
/// such a name is rotated like the automatic ones.
fn rotate(paths: &Paths, just_made: &str) {
    let Ok(mut names) = copies(paths) else {
        return;
    };
    names.retain(|n| n != just_made);
    // `just_made` is one of the kept ones.
    while names.len() + 1 > KEEP {
        let old = names.remove(0);
        if let Err(e) = fs::remove_file(paths.backups.join(&old)) {
            eprintln!("passwords backup: could not remove {old}: {e}");
        }
    }
}

// ---------------------------------------------------------------- restore

fn temp_path(paths: &Paths) -> PathBuf {
    paths.data.join(RESTORE_TEMP)
}

/// Removes the checked copy. Errors are ignored.
pub fn remove_temp(paths: &Paths) {
    let _ = fs::remove_file(temp_path(paths));
}

fn plain_io(e: Error) -> Error {
    restore::plain_io_with(e, IN_USE)
}

/// Turns what the UI sent into a file that may be restored from, or says why not.
///
/// A bare name (no `/` or `\`) must be an automatic copy and is looked up in the
/// backups folder. Anything else must be an absolute local path that is the file
/// most recently chosen in the file dialog (`picked`, canonical).
pub fn resolve_source(paths: &Paths, source: &str, picked: Option<&Path>) -> Result<PathBuf> {
    resolve_source_with(paths, source, picked, MAX_BYTES)
}

fn resolve_source_with(paths: &Paths, source: &str, picked: Option<&Path>, max_bytes: u64) -> Result<PathBuf> {
    if source.trim().is_empty() {
        return Err(Error::invalid("Choose a passwords backup to restore."));
    }
    let path = if !source.contains(['/', '\\']) {
        if source.contains("..") || source.contains(':') || !is_backup_name(source) {
            return Err(Error::invalid("That is not the name of a passwords backup."));
        }
        paths.backups.join(source)
    } else {
        // `\\server\share`, `\\?\...` and `\\.\...` (devices) all start like this.
        if source.starts_with("\\\\") || source.starts_with("//") {
            return Err(Error::invalid(NETWORK));
        }
        let path = PathBuf::from(source);
        if !path.is_absolute() || !restore::is_picked(&path, picked) {
            return Err(Error::invalid(DIALOG));
        }
        path
    };

    let is_kdbx = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("kdbx"));
    if !is_kdbx {
        return Err(Error::invalid("A passwords backup is a file ending in .kdbx."));
    }
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(Error::invalid("That backup file wasn't found."));
        }
        Err(_) => return Err(Error::invalid("That backup file can't be read.")),
    };
    if !meta.is_file() {
        return Err(Error::invalid("That is a folder, not a backup file."));
    }
    if meta.len() == 0 {
        return Err(Error::invalid("That backup file is empty."));
    }
    if meta.len() > max_bytes {
        return Err(Error::invalid("That file is too large to be a passwords backup."));
    }

    // A link could still lead somewhere else, so look at where the file really is.
    let real = fs::canonicalize(&path).map_err(|_| Error::invalid("That backup file can't be read."))?;
    if real.to_string_lossy().starts_with(r"\\?\UNC\") {
        return Err(Error::invalid(NETWORK));
    }
    if find_database(paths).and_then(|live| fs::canonicalize(live).ok()).is_some_and(|live| live == real) {
        return Err(Error::invalid("That is the passwords file in use now, not a backup."));
    }
    // The old file that an earlier restore kept. (The temporary copy can't get here:
    // the .kdbx rule above refuses its name.)
    let is_undo = |p: &Path| p.file_name().is_some_and(|n| n.to_string_lossy().to_ascii_lowercase().starts_with(UNDO_PREFIX));
    if is_undo(&path) || is_undo(&real) {
        return Err(Error::invalid(
            "That file is a safety copy kept by an earlier restore. Copy it under another name to use it as a backup.",
        ));
    }
    Ok(path)
}

/// Copies `source` to `data/passwords-restore-check.tmp` and checks the copy
/// (not empty, KeePassXC signature), then installs it. Returns the name of the
/// kept old file in the data folder, or `None` if there was none. On any
/// failure the copy is removed and the live file is as it was.
pub fn stage_and_install(paths: &Paths, source: &Path, now_ms: u64) -> Result<Option<String>> {
    stage_and_install_with(paths, source, now_ms, MAX_BYTES, restore::RENAME_ATTEMPTS, restore::RENAME_DELAY).map_err(plain_io)
}

fn stage_and_install_with(
    paths: &Paths,
    source: &Path,
    now_ms: u64,
    max_bytes: u64,
    attempts: u32,
    delay: Duration,
) -> Result<Option<String>> {
    let result = stage(paths, source, max_bytes).and_then(|()| install(paths, now_ms, attempts, delay));
    if result.is_err() {
        remove_temp(paths);
    }
    result
}

fn stage(paths: &Paths, source: &Path, max_bytes: u64) -> Result<()> {
    fs::create_dir_all(&paths.data)?;
    remove_temp(paths); // a leftover from an earlier attempt
    let tmp = temp_path(paths);
    let copied = copy_new(source, &tmp, max_bytes)?;
    if copied == 0 {
        return Err(Error::invalid("That backup file is empty."));
    }
    if copied > max_bytes {
        return Err(Error::invalid("That file is too large to be a passwords backup."));
    }
    if !has_signature(&tmp)? {
        return Err(Error::invalid(NOT_KDBX));
    }
    Ok(())
}

/// Puts the checked copy where the passwords file is: the SAME path the current
/// database has, or `data/passwords.kdbx` if there is none. The current file is
/// renamed to `data/passwords-undo-<ms>.kdbx` first (a name that is free: an
/// earlier undo file is never overwritten) and its name is returned. If the
/// second rename fails, the old file is put back. Nothing is ever deleted.
fn install(paths: &Paths, now_ms: u64, attempts: u32, delay: Duration) -> Result<Option<String>> {
    let tmp = temp_path(paths);
    let live = find_database(paths);
    let target = live.clone().unwrap_or_else(|| paths.data.join("passwords.kdbx"));

    let mut undo: Option<PathBuf> = None;
    if let Some(live) = &live {
        // `rename` replaces an existing file on Windows, so pick a name that is free.
        let mut ms = now_ms;
        let base = loop {
            let p = paths.data.join(format!("{UNDO_PREFIX}{ms}.kdbx"));
            if !p.try_exists()? {
                break p;
            }
            ms += 1;
        };
        restore::rename_retry(live, &base, attempts, delay)?;
        undo = Some(base);
    }

    if let Err(e) = restore::rename_retry(&tmp, &target, attempts, delay) {
        return Err(match &undo {
            Some(base) => restore::install_failure(e, &roll_back(base, &target, attempts, delay)),
            None => Error::Io(e),
        });
    }
    Ok(undo.and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string)))
}

/// Puts the old file back. Returns plain sentences for what could not be put
/// back (empty if all went well).
fn roll_back(base: &Path, target: &Path, attempts: u32, delay: Duration) -> Vec<String> {
    match restore::rename_retry(base, target, attempts, delay) {
        Ok(()) => Vec::new(),
        Err(e) => {
            eprintln!("restore passwords: could not put the passwords file back: {e}");
            let name = base.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            vec![format!("Your previous passwords file could not be put back; it is saved as {name} in the data folder.")]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::new_id;
    use std::time::SystemTime;

    fn temp_paths() -> Paths {
        let root = std::env::temp_dir().join(format!("cipherleaf-pwbackup-{}", new_id()));
        std::fs::create_dir_all(&root).unwrap();
        let paths = Paths::at(&root, root.join("kp"));
        std::fs::create_dir_all(&paths.backups).unwrap();
        paths
    }

    /// A file that starts like a KeePassXC database, followed by `body`.
    fn kdbx(body: &str) -> Vec<u8> {
        let mut v = SIGNATURE.to_vec();
        v.extend_from_slice(body.as_bytes());
        v
    }

    fn live(paths: &Paths) -> PathBuf {
        paths.data.join("passwords.kdbx")
    }

    fn write(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }

    fn set_age(path: &Path, secs_ago: u64) {
        let t = SystemTime::now() - Duration::from_secs(secs_ago);
        OpenOptions::new().write(true).open(path).unwrap().set_modified(t).unwrap();
    }

    // ------------------------------------------------------------ automatic copies

    #[test]
    fn only_exact_backup_names_count() {
        for ok in ["passwords-1.kdbx", "passwords-1700000000000.kdbx", "passwords-007.kdbx"] {
            assert!(is_backup_name(ok), "{ok}");
        }
        for bad in [
            "passwords-.kdbx",
            "passwords-abc.kdbx",
            "passwords-12.kdbx.bak",
            "passwords-12.KDBX",
            "passwords-undo-12.kdbx",
            "passwords.kdbx",
            "xpasswords-12.kdbx",
            "passwords-1 2.kdbx",
            "passwords-12.kdbx ",
            "vault-12.clf",
            "passwords-backup.tmp",
        ] {
            assert!(!is_backup_name(bad), "{bad}");
        }
    }

    #[test]
    fn nothing_happens_without_a_passwords_file() {
        let paths = temp_paths();
        std::fs::remove_dir_all(&paths.backups).unwrap();
        assert_eq!(backup_if_changed(&paths, 1).unwrap(), None);
        assert!(!paths.backups.exists(), "no folder is created for nothing");
        // An empty live file is not copied either.
        write(&live(&paths), b"");
        assert_eq!(backup_if_changed(&paths, 1).unwrap(), None);
    }

    #[test]
    fn backs_up_when_there_is_no_backup_yet() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("one"));
        let name = backup_if_changed(&paths, 1000).unwrap().expect("a copy");
        assert_eq!(name, "passwords-1000.kdbx");
        assert_eq!(std::fs::read(paths.backups.join(&name)).unwrap(), kdbx("one"));
        assert_eq!(names_in(&paths.backups), ["passwords-1000.kdbx"], "no temporary file is left");
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("one"), "the live file is untouched");
    }

    #[test]
    fn backs_up_a_database_that_is_not_called_passwords_kdbx() {
        let paths = temp_paths();
        write(&paths.data.join("family.kdbx"), &kdbx("fam"));
        assert!(backup_if_changed(&paths, 5).unwrap().is_some());
    }

    #[test]
    fn does_not_copy_an_unchanged_file() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("same"));
        assert!(backup_if_changed(&paths, 1000).unwrap().is_some());
        assert_eq!(backup_if_changed(&paths, 2000).unwrap(), None);
        assert_eq!(backup_if_changed(&paths, 3000).unwrap(), None);
        assert_eq!(names_in(&paths.backups), ["passwords-1000.kdbx"]);
    }

    #[test]
    fn copies_again_when_the_file_changed() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("v1"));
        backup_if_changed(&paths, 1000).unwrap();
        write(&live(&paths), &kdbx("v2"));
        assert_eq!(backup_if_changed(&paths, 2000).unwrap().as_deref(), Some("passwords-2000.kdbx"));
        assert_eq!(std::fs::read(paths.backups.join("passwords-1000.kdbx")).unwrap(), kdbx("v1"));
        assert_eq!(std::fs::read(paths.backups.join("passwords-2000.kdbx")).unwrap(), kdbx("v2"));
        // Changed back to the older content: it differs from the NEWEST copy, so it is copied.
        write(&live(&paths), &kdbx("v1"));
        assert!(backup_if_changed(&paths, 3000).unwrap().is_some());
        // Same size, one byte different.
        write(&live(&paths), &kdbx("v3"));
        assert!(backup_if_changed(&paths, 4000).unwrap().is_some());
        assert_eq!(names_in(&paths.backups).len(), 4);
    }

    #[test]
    fn two_copies_in_the_same_millisecond_get_different_names() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("a"));
        backup_if_changed(&paths, 500).unwrap();
        write(&live(&paths), &kdbx("b"));
        assert_eq!(backup_if_changed(&paths, 500).unwrap().as_deref(), Some("passwords-501.kdbx"));
        assert_eq!(std::fs::read(paths.backups.join("passwords-500.kdbx")).unwrap(), kdbx("a"));
    }

    #[test]
    fn skips_a_file_with_a_bad_signature_and_leaves_no_temp_file() {
        let paths = temp_paths();
        write(&live(&paths), b"this is a text file, or a half-written one");
        assert_eq!(backup_if_changed(&paths, 1).unwrap(), None);
        // Shorter than the signature, and a near miss.
        write(&live(&paths), &SIGNATURE[..5]);
        assert_eq!(backup_if_changed(&paths, 2).unwrap(), None);
        let mut near = kdbx("x");
        near[7] ^= 1;
        write(&live(&paths), &near);
        assert_eq!(backup_if_changed(&paths, 3).unwrap(), None);
        assert!(names_in(&paths.backups).is_empty(), "{:?}", names_in(&paths.backups));
        // A good copy exists, then the file is caught mid-save: the good copy stays the newest.
        write(&live(&paths), &kdbx("good"));
        backup_if_changed(&paths, 10).unwrap();
        write(&live(&paths), b"partial");
        assert_eq!(backup_if_changed(&paths, 20).unwrap(), None);
        assert_eq!(names_in(&paths.backups), ["passwords-10.kdbx"]);
    }

    #[test]
    fn an_old_leftover_temp_file_does_not_block_a_copy() {
        let paths = temp_paths();
        write(&paths.backups.join(BACKUP_TEMP), b"stale");
        write(&live(&paths), &kdbx("x"));
        assert!(backup_if_changed(&paths, 9).unwrap().is_some());
        assert_eq!(names_in(&paths.backups), ["passwords-9.kdbx"]);
    }

    #[test]
    fn keeps_only_the_newest_10_and_never_touches_other_names() {
        let paths = temp_paths();
        let b = &paths.backups;
        for i in 0..12u64 {
            write(&b.join(format!("passwords-{}.kdbx", 1000 + i)), &kdbx(&format!("old{i}")));
        }
        // Names that look similar, and must be left alone.
        let others = [
            "vault-1.clf",
            "vault-2.clf",
            "passwords-undo-5.kdbx",
            "passwords-abc.kdbx",
            "passwords-.kdbx",
            "passwords-12.kdbx.bak",
            "notes.txt",
        ];
        for o in others {
            write(&b.join(o), b"keep me");
        }
        std::fs::create_dir(b.join("passwords-77.kdbx")).unwrap(); // a folder with a matching name
        write(&live(&paths), &kdbx("changed"));

        let made = backup_if_changed(&paths, 5000).unwrap().unwrap();
        assert_eq!(made, "passwords-5000.kdbx");
        let names = names_in(b);
        let copies_left: Vec<&String> = names.iter().filter(|n| is_backup_name(n) && b.join(n).is_file()).collect();
        assert_eq!(copies_left.len(), KEEP, "{names:?}");
        // The 3 oldest of the 13 went; the newest 9 old ones and the new one stay.
        for gone in ["passwords-1000.kdbx", "passwords-1001.kdbx", "passwords-1002.kdbx"] {
            assert!(!names.contains(&gone.to_string()), "{gone} should be deleted");
        }
        for kept in ["passwords-1003.kdbx", "passwords-1011.kdbx", "passwords-5000.kdbx"] {
            assert!(names.contains(&kept.to_string()), "{kept} should stay");
        }
        for o in others {
            assert_eq!(std::fs::read(b.join(o)).unwrap(), b"keep me", "{o}");
        }
        assert!(b.join("passwords-77.kdbx").is_dir());
    }

    #[test]
    fn rotation_never_deletes_the_copy_just_made_even_if_the_clock_went_back() {
        let paths = temp_paths();
        for i in 0..KEEP as u64 {
            write(&paths.backups.join(format!("passwords-{}.kdbx", 9000 + i)), &kdbx(&format!("o{i}")));
        }
        write(&live(&paths), &kdbx("new"));
        let made = backup_if_changed(&paths, 5).unwrap().unwrap(); // older than all the others
        assert!(paths.backups.join(&made).is_file());
        assert_eq!(copies(&paths).unwrap().len(), KEEP);
        assert!(!paths.backups.join("passwords-9000.kdbx").exists(), "an old one went instead");
    }

    #[test]
    fn equal_times_are_ordered_by_the_number_in_the_name() {
        let paths = temp_paths();
        for n in ["passwords-99.kdbx", "passwords-100.kdbx", "passwords-0005.kdbx"] {
            write(&paths.backups.join(n), &kdbx(n));
            OpenOptions::new().write(true).open(paths.backups.join(n)).unwrap().set_modified(UNIX_EPOCH + Duration::from_secs(1_000_000)).unwrap();
        }
        assert_eq!(copies(&paths).unwrap(), ["passwords-0005.kdbx", "passwords-99.kdbx", "passwords-100.kdbx"]);
    }

    #[test]
    fn a_future_named_old_copy_does_not_cost_the_real_newest_copy() {
        let paths = temp_paths();
        let b = &paths.backups;
        // One old copy whose name carries a time far in the future (the clock was wrong),
        // and nine more recent ones with small numbers.
        write(&b.join("passwords-99999999999999.kdbx"), &kdbx("stale"));
        set_age(&b.join("passwords-99999999999999.kdbx"), 10_000);
        for i in 0..9u64 {
            let p = b.join(format!("passwords-{}.kdbx", 100 + i));
            write(&p, &kdbx(&format!("c{i}")));
            set_age(&p, 5_000 - i * 100);
        }
        write(&live(&paths), &kdbx("fresh"));
        let made = backup_if_changed(&paths, 5000).unwrap().unwrap(); // 11 copies now
        let left = copies(&paths).unwrap();
        assert_eq!(left.len(), KEEP);
        assert!(left.contains(&made), "the copy just made stays");
        assert!(!left.contains(&"passwords-99999999999999.kdbx".to_string()), "the oldest by time goes, whatever its name says");
        assert!(left.contains(&"passwords-100.kdbx".to_string()), "{left:?}");
        assert_eq!(left.last().unwrap(), &made, "newest by time is last");
    }

    #[test]
    fn unchanged_is_judged_against_the_newest_copy_by_time() {
        let paths = temp_paths();
        let b = &paths.backups;
        // The copy with the huge number is older; the real newest holds the live content.
        write(&b.join("passwords-99999999999999.kdbx"), &kdbx("old"));
        set_age(&b.join("passwords-99999999999999.kdbx"), 1_000);
        write(&b.join("passwords-7.kdbx"), &kdbx("live"));
        set_age(&b.join("passwords-7.kdbx"), 10);
        write(&live(&paths), &kdbx("live"));
        assert_eq!(backup_if_changed(&paths, 1).unwrap(), None);
    }

    #[test]
    fn a_file_that_changed_while_copying_is_noticed() {
        let paths = temp_paths();
        let f = paths.data.join("x.bin");
        write(&f, b"one");
        let a = stamp_of(&f).unwrap();
        assert_eq!(a, stamp_of(&f).unwrap(), "untouched file: same stamp");
        write(&f, b"three"); // size changed
        assert_ne!(a, stamp_of(&f).unwrap());
        let b = stamp_of(&f).unwrap();
        OpenOptions::new().write(true).open(&f).unwrap().set_modified(SystemTime::now() - Duration::from_secs(60)).unwrap();
        assert_ne!(b, stamp_of(&f).unwrap(), "same size, other time");
    }

    #[test]
    fn simultaneous_backups_do_not_lose_each_others_temp_file() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("shared"));
        let results: Vec<_> = (0..4)
            .map(|i| {
                let p = paths.clone();
                std::thread::spawn(move || backup_if_changed(&p, 100 + i))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        assert!(results.iter().all(|r| r.is_ok()), "{results:?}");
        // One copy of the same content, no leftovers.
        assert_eq!(names_in(&paths.backups).len(), 1, "{:?}", names_in(&paths.backups));
    }

    #[test]
    fn the_vault_backups_are_not_affected() {
        // The vault rotation (state.rs) only looks at `vault-*.clf`; this module only at
        // `passwords-<digits>.kdbx`. Neither pattern matches the other's files.
        assert!(!is_backup_name("vault-1.clf"));
        let paths = temp_paths();
        for i in 0..12 {
            write(&paths.backups.join(format!("vault-{i}.clf")), b"v");
        }
        write(&live(&paths), &kdbx("p"));
        backup_if_changed(&paths, 1).unwrap();
        assert_eq!(restore::list_backups(&paths).unwrap().len(), 12);
        assert_eq!(list(&paths).unwrap().len(), 1);
    }

    #[test]
    fn list_shows_only_copies_newest_first() {
        let paths = temp_paths();
        let b = &paths.backups;
        write(&b.join("passwords-100.kdbx"), b"old");
        write(&b.join("passwords-300.kdbx"), b"newest!");
        write(&b.join("passwords-200.kdbx"), b"mid");
        write(&b.join("passwords-undo-1.kdbx"), b"x");
        write(&b.join("vault-1.clf"), b"x");
        write(&b.join("passwords-abc.kdbx"), b"x");
        std::fs::create_dir(b.join("passwords-9.kdbx")).unwrap();
        set_age(&b.join("passwords-100.kdbx"), 300);
        set_age(&b.join("passwords-200.kdbx"), 200);
        set_age(&b.join("passwords-300.kdbx"), 100);

        let l = list(&paths).unwrap();
        let names: Vec<&str> = l.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["passwords-300.kdbx", "passwords-200.kdbx", "passwords-100.kdbx"]);
        assert_eq!(l[0].size, 7);
        assert!(l[0].modified_ms > l[1].modified_ms && l[1].modified_ms > l[2].modified_ms);
        std::fs::remove_dir_all(b).unwrap();
        assert!(list(&paths).unwrap().is_empty());
    }

    // ------------------------------------------------------------ restore: choosing the file

    fn rejected(paths: &Paths, source: &str, reason: &str) {
        let picked = fs::canonicalize(source).ok();
        match resolve_source(paths, source, picked.as_deref()) {
            Err(Error::Invalid(m)) => assert_eq!(m, reason, "wrong reason for {source:?}"),
            other => panic!("{source:?} should be rejected with {reason:?}, got {other:?}"),
        }
    }

    #[test]
    fn a_bare_name_resolves_inside_the_backups_folder() {
        let paths = temp_paths();
        write(&paths.backups.join("passwords-42.kdbx"), &kdbx("x"));
        assert_eq!(resolve_source(&paths, "passwords-42.kdbx", None).unwrap(), paths.backups.join("passwords-42.kdbx"));
    }

    #[test]
    fn an_absolute_path_must_be_the_one_the_dialog_returned() {
        let paths = temp_paths();
        let outside = paths.data.join("incoming"); // not the data folder itself, which holds the live database
        std::fs::create_dir(&outside).unwrap();
        let a = outside.join("my copy.KDBX");
        let b = outside.join("other.kdbx");
        write(&a, &kdbx("a"));
        write(&b, &kdbx("b"));
        let text = a.display().to_string();
        let pick_a = fs::canonicalize(&a).unwrap();
        let pick_b = fs::canonicalize(&b).unwrap();
        for picked in [None, Some(pick_b.as_path())] {
            match resolve_source(&paths, &text, picked) {
                Err(Error::Invalid(m)) => assert_eq!(m, DIALOG),
                other => panic!("expected refusal, got {other:?}"),
            }
        }
        assert_eq!(resolve_source(&paths, &text, Some(&pick_a)).unwrap(), a);
        // Spelled differently (with `..`) it is still the picked file.
        let odd = outside.join("..").join("incoming").join("my copy.KDBX").display().to_string();
        assert!(resolve_source(&paths, &odd, Some(&pick_a)).is_ok());
        // A relative path gets the same message; a bare name needs no pick.
        assert!(matches!(resolve_source(&paths, "my copy.kdbx", Some(&pick_a)), Err(Error::Invalid(_))));
    }

    #[test]
    fn bad_sources_are_rejected_for_the_right_reason() {
        let paths = temp_paths();
        let b = &paths.backups;
        let not_a_name = "That is not the name of a passwords backup.";
        write(&b.join("passwords-1.kdbx"), &kdbx("x"));
        write(&b.join("passwords-2.txt"), &kdbx("x"));
        write(&b.join("notes.kdbx"), &kdbx("x"));
        write(&b.join("passwords-empty.kdbx"), b"");
        write(&b.join("passwords-0.kdbx"), b"");
        write(&b.join("passwords-ads:x.kdbx"), &kdbx("x"));
        write(&b.join("passwords-dots..kdbx"), &kdbx("x"));
        std::fs::create_dir(b.join("passwords-7.kdbx")).unwrap();
        write(&live(&paths), &kdbx("live"));
        write(&paths.data.join(RESTORE_TEMP), &kdbx("temp"));
        write(&paths.data.join("passwords-undo-9.kdbx"), &kdbx("undo"));
        let abs = |p: PathBuf| p.display().to_string();

        rejected(&paths, "", "Choose a passwords backup to restore.");
        rejected(&paths, "  ", "Choose a passwords backup to restore.");
        // Bare names: wrong shape, even when a file of that name exists.
        rejected(&paths, "passwords-2.txt", not_a_name);
        rejected(&paths, "notes.kdbx", not_a_name);
        rejected(&paths, "passwords-dots..kdbx", not_a_name);
        rejected(&paths, "passwords-ads:x.kdbx", not_a_name);
        rejected(&paths, "passwords-undo-9.kdbx", not_a_name);
        rejected(&paths, "passwords.kdbx", not_a_name);
        // Right shape, but not usable.
        rejected(&paths, "passwords-99.kdbx", "That backup file wasn't found.");
        rejected(&paths, "passwords-7.kdbx", "That is a folder, not a backup file.");
        rejected(&paths, "passwords-0.kdbx", "That backup file is empty.");
        // Relative paths and parts of paths.
        rejected(&paths, "..\\passwords-1.kdbx", DIALOG);
        rejected(&paths, "../passwords-1.kdbx", DIALOG);
        rejected(&paths, "backups\\passwords-1.kdbx", DIALOG);
        rejected(&paths, "\\passwords-1.kdbx", DIALOG);
        // Network and device paths.
        rejected(&paths, r"\\server\share\x.kdbx", NETWORK);
        rejected(&paths, r"\\?\C:\x.kdbx", NETWORK);
        rejected(&paths, r"\\.\C:\x.kdbx", NETWORK);
        rejected(&paths, "//server/share/x.kdbx", NETWORK);
        // The live database, spelled plainly and with `..` inside.
        let in_use = "That is the passwords file in use now, not a backup.";
        rejected(&paths, &abs(live(&paths)), in_use);
        rejected(&paths, &abs(b.join("..").join("passwords.kdbx")), in_use);
        // Wrong extension, the temporary copy, a folder, a missing file (never picked).
        let ext = "A passwords backup is a file ending in .kdbx.";
        rejected(&paths, &abs(b.join("passwords-2.txt")), ext);
        rejected(&paths, &abs(paths.data.join(RESTORE_TEMP)), ext);
        rejected(&paths, &abs(b.join("passwords-7.kdbx")), "That is a folder, not a backup file.");
        rejected(&paths, &abs(b.join("nothing.kdbx")), DIALOG);
        // Safety copies from an earlier restore are not taken as backups.
        rejected(
            &paths,
            &abs(paths.data.join("passwords-undo-9.kdbx")),
            "That file is a safety copy kept by an earlier restore. Copy it under another name to use it as a backup.",
        );
        // And the good one still works.
        assert!(resolve_source(&paths, "passwords-1.kdbx", None).is_ok());
    }

    #[test]
    fn the_live_database_is_refused_when_it_has_another_name_too() {
        let paths = temp_paths();
        let db = paths.data.join("family.kdbx");
        write(&db, &kdbx("live"));
        let text = db.display().to_string();
        let picked = fs::canonicalize(&db).unwrap();
        match resolve_source(&paths, &text, Some(&picked)) {
            Err(Error::Invalid(m)) => assert_eq!(m, "That is the passwords file in use now, not a backup."),
            other => panic!("expected refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_file_over_the_size_limit_is_refused() {
        let paths = temp_paths();
        write(&paths.backups.join("passwords-1.kdbx"), &kdbx("0123456789")); // 18 bytes
        match resolve_source_with(&paths, "passwords-1.kdbx", None, 17) {
            Err(Error::Invalid(m)) => assert_eq!(m, "That file is too large to be a passwords backup."),
            other => panic!("expected refusal, got {other:?}"),
        }
        assert!(resolve_source_with(&paths, "passwords-1.kdbx", None, 18).is_ok(), "exactly at the limit");
    }

    // ------------------------------------------------------------ restore: installing

    #[test]
    fn installs_and_keeps_the_old_file_as_an_undo_file() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("OLD"));
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, &kdbx("NEW"));

        let undo = stage_and_install(&paths, &src, 1234).unwrap().expect("an undo file");
        assert_eq!(undo, "passwords-undo-1234.kdbx");
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("NEW"));
        assert_eq!(std::fs::read(paths.data.join(&undo)).unwrap(), kdbx("OLD"));
        assert!(!temp_path(&paths).exists());
        assert_eq!(std::fs::read(&src).unwrap(), kdbx("NEW"), "the backup itself is untouched");
        // The undo file is not mistaken for the database in use.
        assert_eq!(find_database(&paths).unwrap(), live(&paths));
    }

    #[test]
    fn installs_over_the_same_path_when_the_database_has_another_name() {
        let paths = temp_paths();
        let db = paths.data.join("zeta.kdbx");
        write(&db, &kdbx("OLD"));
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, &kdbx("NEW"));
        let undo = stage_and_install(&paths, &src, 8).unwrap().unwrap();
        assert_eq!(std::fs::read(&db).unwrap(), kdbx("NEW"));
        assert!(!live(&paths).exists(), "no second database is created");
        assert_eq!(std::fs::read(paths.data.join(undo)).unwrap(), kdbx("OLD"));
        // The next start still opens the same database, not the undo file.
        assert_eq!(find_database(&paths).unwrap(), db);
    }

    #[test]
    fn works_when_no_database_existed() {
        let paths = temp_paths();
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, &kdbx("NEW"));
        assert_eq!(stage_and_install(&paths, &src, 1).unwrap(), None);
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("NEW"));
        assert!(!temp_path(&paths).exists());
        assert!(names_in(&paths.data).iter().all(|n| !n.starts_with(UNDO_PREFIX)));
    }

    #[test]
    fn never_overwrites_an_earlier_undo_file() {
        let paths = temp_paths();
        write(&paths.data.join("passwords-undo-50.kdbx"), &kdbx("EARLIER"));
        write(&paths.data.join("passwords-undo-51.kdbx"), &kdbx("EARLIER2"));
        write(&live(&paths), &kdbx("OLD"));
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, &kdbx("NEW"));
        let undo = stage_and_install(&paths, &src, 50).unwrap().unwrap();
        assert_eq!(undo, "passwords-undo-52.kdbx");
        assert_eq!(std::fs::read(paths.data.join("passwords-undo-50.kdbx")).unwrap(), kdbx("EARLIER"));
        assert_eq!(std::fs::read(paths.data.join("passwords-undo-51.kdbx")).unwrap(), kdbx("EARLIER2"));
        assert_eq!(std::fs::read(paths.data.join(undo)).unwrap(), kdbx("OLD"));
        // And a second restore keeps this one too.
        let undo2 = stage_and_install(&paths, &src, 52).unwrap().unwrap();
        assert_eq!(undo2, "passwords-undo-53.kdbx");
        assert_eq!(std::fs::read(paths.data.join("passwords-undo-52.kdbx")).unwrap(), kdbx("OLD"));
    }

    #[test]
    fn a_failed_second_rename_puts_the_old_file_back() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("OLD"));
        // No checked copy exists, so the second rename fails.
        let r = install(&paths, 7, 1, Duration::ZERO);
        assert!(r.is_err());
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("OLD"));
        assert!(!paths.data.join("passwords-undo-7.kdbx").exists());
    }

    #[test]
    fn a_failed_rollback_is_named_in_the_error() {
        let paths = temp_paths();
        // The undo file is gone, so putting it back fails too.
        let base = paths.data.join("passwords-undo-3.kdbx");
        let notes = roll_back(&base, &live(&paths), 1, Duration::ZERO);
        assert_eq!(notes.len(), 1);
        let text = restore::install_failure(io::Error::other("raw"), &notes).to_string();
        assert!(text.contains("could not be put back") && text.contains("passwords-undo-3.kdbx"), "{text}");
        assert!(!text.contains("raw"));
    }

    #[test]
    fn a_file_without_the_signature_is_refused_and_nothing_changes() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("OLD"));
        let src = paths.backups.join("passwords-1.kdbx");
        for bad in [b"just some text, not a database".to_vec(), SIGNATURE[..4].to_vec()] {
            write(&src, &bad);
            match stage_and_install(&paths, &src, 1) {
                Err(Error::Invalid(m)) => assert_eq!(m, NOT_KDBX),
                other => panic!("expected refusal, got {other:?}"),
            }
            assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("OLD"));
            assert!(!temp_path(&paths).exists(), "the copy is removed");
            assert!(names_in(&paths.data).iter().all(|n| !n.starts_with(UNDO_PREFIX)));
        }
    }

    #[test]
    fn an_empty_or_huge_file_is_refused_while_copying() {
        let paths = temp_paths();
        write(&live(&paths), &kdbx("OLD"));
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, b"");
        let r = stage_and_install_with(&paths, &src, 1, MAX_BYTES, 1, Duration::ZERO);
        assert!(matches!(&r, Err(Error::Invalid(m)) if m == "That backup file is empty."), "{r:?}");
        write(&src, &kdbx("0123456789"));
        let r = stage_and_install_with(&paths, &src, 1, 12, 1, Duration::ZERO);
        assert!(matches!(&r, Err(Error::Invalid(m)) if m.contains("too large")), "{r:?}");
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("OLD"));
        assert!(!temp_path(&paths).exists());
    }

    #[test]
    fn a_leftover_temp_file_is_replaced() {
        let paths = temp_paths();
        write(&temp_path(&paths), b"stale");
        let src = paths.backups.join("passwords-1.kdbx");
        write(&src, &kdbx("NEW"));
        stage_and_install(&paths, &src, 1).unwrap();
        assert_eq!(std::fs::read(live(&paths)).unwrap(), kdbx("NEW"));
    }

    #[test]
    fn raw_file_errors_become_one_plain_message() {
        for raw in [io::Error::from_raw_os_error(32), io::Error::from_raw_os_error(5), io::Error::new(io::ErrorKind::PermissionDenied, "x")] {
            match plain_io(Error::Io(raw)) {
                Error::Other(m) => assert_eq!(m, IN_USE),
                other => panic!("expected the plain message, got {other:?}"),
            }
        }
        assert!(matches!(plain_io(Error::Io(io::Error::from_raw_os_error(112))), Error::Other(m) if m.contains("free space")));
        assert!(matches!(plain_io(Error::invalid("x")), Error::Invalid(_)));
        assert!(!IN_USE.contains("os error"));
    }
}
