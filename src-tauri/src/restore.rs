//! Restoring the vault from a backup file (the logic that doesn't need a window).
//!
//! The rules: check first, never delete. The chosen file is copied to
//! `data/restore-check.tmp` and that copy is opened with the password (so the
//! exact bytes that will be installed are the ones that were verified). Only
//! then is the live vault renamed to `data/restore-undo-<ms>.clf` and the copy
//! renamed to `data/vault.clf`. The app never deletes or rotates `restore-undo-*`
//! files; the backup rotation only touches `vault-*.clf` in `data/backups`.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::crypto::KdfParams;
use crate::portable::Paths;
use crate::vault::Session;

/// Largest backup file accepted.
const MAX_BACKUP_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// A backup's password-stretching settings must stay within these limits, which
/// are tighter than what `KdfParams::validate` accepts when opening the live
/// vault: a restore must not be made to allocate several GiB by a crafted file.
const MAX_RESTORE_M_KIB: u32 = 1_048_576; // 1 GiB
const MAX_RESTORE_T: u32 = 10;
const MAX_RESTORE_P: u32 = 16;
/// A rename that fails is tried again (Windows may still hold a file that was
/// closed a moment ago).
pub(crate) const RENAME_ATTEMPTS: u32 = 6;
pub(crate) const RENAME_DELAY: Duration = Duration::from_millis(300);
/// The verified copy, before it is renamed to `vault.clf`.
const TEMP_NAME: &str = "restore-check.tmp";
/// The old vault, kept when a restore replaces it.
const UNDO_PREFIX: &str = "restore-undo-";

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub name: String,
    pub modified_ms: u64,
    pub size: u64,
}

/// The automatic backups look like `vault-<ms>.clf` (same rule as the rotation).
fn is_backup_name(name: &str) -> bool {
    name.starts_with("vault-") && name.ends_with(".clf")
}

/// `path` with `suffix` added to the file name (`vault.clf` -> `vault.clf-journal`).
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.file_name().map(OsString::from).unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

fn temp_path(paths: &Paths) -> PathBuf {
    paths.data.join(TEMP_NAME)
}

/// The regular files `vault-*.clf` in the backups folder, newest first. Reads
/// only names, times and sizes, never contents, so it works while locked.
pub fn list_backups(paths: &Paths) -> Result<Vec<BackupInfo>> {
    list_in(&paths.backups, is_backup_name)
}

/// The regular files in `dir` whose names satisfy `is_name`, newest first (shared
/// with the passwords-file backups). A missing folder is an empty list.
pub(crate) fn list_in(dir: &Path, is_name: fn(&str) -> bool) -> Result<Vec<BackupInfo>> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !is_name(&name) {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        out.push(BackupInfo { name, modified_ms, size: meta.len() });
    }
    // Newest first; equal (or unknown) times fall back to the name, which holds the time.
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms).then_with(|| b.name.cmp(&a.name)));
    Ok(out)
}

/// Turns what the UI sent into a file that may be restored from, or says why not.
///
/// A bare name (no `/` or `\`) must be an automatic backup and is looked up in
/// the backups folder. Anything else must be an absolute local path that is the
/// file most recently chosen in the file dialog (`picked`, canonical).
pub fn resolve_source(paths: &Paths, source: &str, picked: Option<&Path>) -> Result<PathBuf> {
    if source.trim().is_empty() {
        return Err(Error::invalid("Choose a backup to restore."));
    }
    let path = if !source.contains(['/', '\\']) {
        if source.contains("..") || source.contains(':') || !is_backup_name(source) {
            return Err(Error::invalid("That is not the name of a Cipherleaf backup."));
        }
        paths.backups.join(source)
    } else {
        // `\\server\share`, `\\?\...` and `\\.\...` (devices) all start like this.
        if source.starts_with("\\\\") || source.starts_with("//") {
            return Err(Error::invalid(
                "Backups on a network or device path can't be used. Copy the file to this PC first.",
            ));
        }
        let path = PathBuf::from(source);
        if !path.is_absolute() {
            return Err(Error::invalid("Choose the backup file with the file dialog."));
        }
        if !is_picked(&path, picked) {
            return Err(Error::invalid("Choose the backup file with the file dialog."));
        }
        path
    };

    let is_clf = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("clf"));
    if !is_clf {
        return Err(Error::invalid("A Cipherleaf backup is a file ending in .clf."));
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
    if meta.len() > MAX_BACKUP_BYTES {
        return Err(Error::invalid("That file is too large to be a Cipherleaf backup."));
    }

    // A link could still lead somewhere else, so look at where the file really is.
    let real = fs::canonicalize(&path).map_err(|_| Error::invalid("That backup file can't be read."))?;
    if real.to_string_lossy().starts_with(r"\\?\UNC\") {
        return Err(Error::invalid(
            "Backups on a network or device path can't be used. Copy the file to this PC first.",
        ));
    }
    if fs::canonicalize(&paths.vault).is_ok_and(|live| live == real) {
        return Err(Error::invalid("That is the vault in use now, not a backup."));
    }
    // (`restore-check.tmp` can't get here: the .clf extension rule above refuses it.)
    Ok(path)
}

/// True if `path` is the file the dialog returned last (compared canonicalized).
pub(crate) fn is_picked(path: &Path, picked: Option<&Path>) -> bool {
    picked.is_some_and(|p| fs::canonicalize(path).is_ok_and(|c| c == p))
}

/// Copies `from` to the new file `to` and syncs it. A fresh file, not
/// `fs::copy`, so a read-only source doesn't make a read-only vault.
fn copy_file(from: &Path, to: &Path) -> Result<()> {
    let mut src = File::open(from)?;
    let mut dst = OpenOptions::new().write(true).create_new(true).open(to)?;
    let copied = io::copy(&mut (&mut src).take(MAX_BACKUP_BYTES + 1), &mut dst)?;
    if copied == 0 {
        return Err(Error::invalid("That backup file is empty."));
    }
    if copied > MAX_BACKUP_BYTES {
        return Err(Error::invalid("That file is too large to be a Cipherleaf backup."));
    }
    dst.sync_all()?;
    Ok(())
}

/// Removes the checked copy and any SQLite journal next to it. Errors are ignored.
pub fn remove_temp(paths: &Paths) {
    let tmp = temp_path(paths);
    let _ = fs::remove_file(with_suffix(&tmp, "-journal"));
    let _ = fs::remove_file(&tmp);
}

/// Copies `source` to `data/restore-check.tmp` and opens THAT COPY with the
/// credentials: the header, every record and every attachment must authenticate.
/// On success the copy is closed and synced, ready for [`install`]. On any
/// failure the copy is removed and the live vault is untouched.
pub fn verify_and_stage(paths: &Paths, source: &Path, password: &str, keyfile: Option<&[u8; 32]>) -> Result<()> {
    let result = stage(paths, source, password, keyfile).map_err(plain_io);
    if result.is_err() {
        remove_temp(paths);
    }
    result
}

fn stage(paths: &Paths, source: &Path, password: &str, keyfile: Option<&[u8; 32]>) -> Result<()> {
    fs::create_dir_all(&paths.data)?;
    remove_temp(paths); // a leftover from an earlier attempt
    let tmp = temp_path(paths);
    copy_file(source, &tmp)?;
    // Look at the settings before spending memory on them (unlocking runs Argon2).
    let header = match Session::read_header(&tmp) {
        Ok(h) => h,
        Err(Error::Db(_)) => return Err(Error::Corrupt),
        Err(e) => return Err(e),
    };
    check_restore_kdf(&header.kdf)?;
    {
        let session = match Session::unlock(&tmp, password, keyfile) {
            Ok(s) => s,
            // Not a SQLite file at all, or tables missing: a damaged file.
            Err(Error::Db(_)) => return Err(Error::Corrupt),
            Err(e) => return Err(e),
        };
        match session.verify_attachments() {
            Err(Error::Db(_)) => return Err(Error::Corrupt),
            other => other?,
        }
        // Dropped here: no SQLite handle may stay open on the file before it is renamed.
    }
    // Needs a write handle on Windows (FlushFileBuffers); nothing is written.
    OpenOptions::new().write(true).open(&tmp)?.sync_all()?;
    Ok(())
}

/// Refuses password-stretching settings that would need too much memory or time.
fn check_restore_kdf(kdf: &KdfParams) -> Result<()> {
    if kdf.m_kib > MAX_RESTORE_M_KIB || kdf.t > MAX_RESTORE_T || kdf.p > MAX_RESTORE_P {
        return Err(Error::invalid("This backup uses unusual password settings and can't be restored here."));
    }
    Ok(())
}

/// `fs::rename`, tried again a few times if it fails. A missing source is final.
pub(crate) fn rename_retry(from: &Path, to: &Path, attempts: u32, delay: Duration) -> io::Result<()> {
    let mut tries = 1;
    loop {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) if tries >= attempts.max(1) || e.kind() == io::ErrorKind::NotFound => return Err(e),
            Err(_) => {
                tries += 1;
                std::thread::sleep(delay);
            }
        }
    }
}

const IN_USE: &str = "The vault file is in use by another program. Close programs that may be using it (antivirus or a file-sync tool, for example) and try again.";
const NO_SPACE: &str = "There is not enough free space on the drive to restore this backup.";

/// Turns a raw file error from the copy or rename steps into one plain message
/// (the original goes to stderr). Other errors pass through unchanged.
fn plain_io(e: Error) -> Error {
    plain_io_with(e, IN_USE)
}

/// [`plain_io`] with the "in use" sentence of the file being restored.
pub(crate) fn plain_io_with(e: Error, in_use: &str) -> Error {
    let Error::Io(io_err) = e else {
        return e;
    };
    eprintln!("restore: file error: {io_err}");
    // 112 = disk full, 39 = handle disk full.
    let full = io_err.kind() == io::ErrorKind::StorageFull || matches!(io_err.raw_os_error(), Some(112 | 39));
    Error::Other(if full { NO_SPACE } else { in_use }.to_string())
}

/// The error for a failed install. If putting the old files back failed too,
/// the text says so and where they are, instead of hiding it. `rollback_notes`
/// are plain sentences from [`roll_back`].
pub(crate) fn install_failure(e: io::Error, rollback_notes: &[String]) -> Error {
    if rollback_notes.is_empty() {
        return Error::Io(e);
    }
    eprintln!("restore: install failed: {e}");
    Error::Other(format!("The backup could not be installed. {}", rollback_notes.join(" ")))
}

/// Replaces the live vault with the checked copy. An existing vault is renamed
/// to `data/restore-undo-<now_ms>.clf` (with its `-journal` file, if any) and
/// its name is returned; if there was no vault, returns `None` (a stray journal
/// is still moved aside, so SQLite can't apply it to the new file). If a rename
/// fails, the old files are put back. Nothing is ever deleted.
pub fn install(paths: &Paths, now_ms: u64) -> Result<Option<String>> {
    install_with(paths, now_ms, RENAME_ATTEMPTS, RENAME_DELAY).map_err(plain_io)
}

fn install_with(paths: &Paths, now_ms: u64, attempts: u32, delay: Duration) -> Result<Option<String>> {
    let tmp = temp_path(paths);
    let journal = with_suffix(&paths.vault, "-journal");
    // An error here (not just "no") stops the restore: don't guess.
    let has_vault = paths.vault.try_exists()?;
    let has_journal = journal.try_exists()?;

    // The undo file's path, if anything is moved aside.
    let mut undo: Option<PathBuf> = None;
    let mut vault_moved = false;
    let mut journal_moved = false;
    if has_vault || has_journal {
        // `rename` replaces an existing file on Windows, so pick a name that is free.
        let mut ms = now_ms;
        let base = loop {
            let p = paths.data.join(format!("{UNDO_PREFIX}{ms}.clf"));
            if !p.try_exists()? && !with_suffix(&p, "-journal").try_exists()? {
                break p;
            }
            ms += 1;
        };
        if has_vault {
            rename_retry(&paths.vault, &base, attempts, delay)?;
            vault_moved = true;
        }
        undo = Some(base);
    }

    if has_journal {
        let base = undo.as_deref().expect("set when a journal exists");
        match rename_retry(&journal, &with_suffix(base, "-journal"), attempts, delay) {
            Ok(()) => journal_moved = true,
            Err(e) => {
                let moved = Moved { vault: vault_moved, journal: false };
                return Err(install_failure(e, &roll_back(paths, base, moved, attempts, delay)));
            }
        }
    }

    if let Err(e) = rename_retry(&tmp, &paths.vault, attempts, delay) {
        return Err(match &undo {
            Some(base) => {
                let moved = Moved { vault: vault_moved, journal: journal_moved };
                install_failure(e, &roll_back(paths, base, moved, attempts, delay))
            }
            None => Error::Io(e),
        });
    }
    let kept_vault = if vault_moved { undo } else { None };
    Ok(kept_vault.and_then(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string)))
}

/// Which of the live files were moved aside to the undo name.
struct Moved {
    vault: bool,
    journal: bool,
}

/// Puts the moved files back. Returns plain sentences for what could not be put
/// back (empty if all went well). The journal is put back only if the vault was:
/// a journal next to the wrong file could be applied to it.
fn roll_back(paths: &Paths, base: &Path, moved: Moved, attempts: u32, delay: Duration) -> Vec<String> {
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let journal_name = name(&with_suffix(base, "-journal"));
    let mut notes = Vec::new();
    let mut vault_back = true;
    if moved.vault
        && let Err(e) = rename_retry(base, &paths.vault, attempts, delay)
    {
        eprintln!("restore: could not put the vault back: {e}");
        notes.push(format!(
            "Your previous vault could not be put back; it is saved as {} in the data folder.",
            name(base)
        ));
        vault_back = false;
    }
    if moved.journal {
        let journal = with_suffix(&paths.vault, "-journal");
        if !vault_back {
            notes.push(format!("Its journal file was left as {journal_name}."));
        } else if let Err(e) = rename_retry(&with_suffix(base, "-journal"), &journal, attempts, delay) {
            eprintln!("restore: could not put the journal back: {e}");
            notes.push(format!("The journal file of your previous vault is saved as {journal_name}."));
        }
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    use crate::model::{new_id, now_ms, Item, Sticky};
    use std::time::{Duration, SystemTime};

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    fn temp_paths() -> Paths {
        let root = std::env::temp_dir().join(format!("cipherleaf-restore-{}", new_id()));
        std::fs::create_dir_all(&root).unwrap();
        let paths = Paths::at(&root, root.join("kp"));
        std::fs::create_dir_all(&paths.backups).unwrap();
        paths
    }

    fn sticky(text: &str) -> Item {
        Item::Sticky(Sticky {
            id: new_id(),
            content: String::new(),
            text: text.into(),
            color: "yellow".into(),
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 320.0,
            on_top: true,
            open: false,
            created: now_ms(),
            updated: now_ms(),
        })
    }

    fn write(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }

    fn set_age(path: &Path, secs_ago: u64) {
        let t = SystemTime::now() - Duration::from_secs(secs_ago);
        OpenOptions::new().write(true).open(path).unwrap().set_modified(t).unwrap();
    }

    const NOT_A_NAME: &str = "That is not the name of a Cipherleaf backup.";
    const DIALOG: &str = "Choose the backup file with the file dialog.";
    const NETWORK: &str = "Backups on a network or device path can't be used. Copy the file to this PC first.";
    const NOT_CLF: &str = "A Cipherleaf backup is a file ending in .clf.";

    /// `source` must be refused with exactly `reason`. For an absolute path that
    /// exists, the dialog is taken to have returned it, so the refusal comes from
    /// the path itself and not from the missing pick.
    fn rejected(paths: &Paths, source: &str, reason: &str) {
        let picked = fs::canonicalize(source).ok();
        match resolve_source(paths, source, picked.as_deref()) {
            Err(Error::Invalid(m)) => assert_eq!(m, reason, "wrong reason for {source:?}"),
            other => panic!("{source:?} should be rejected with {reason:?}, got {other:?}"),
        }
    }

    #[test]
    fn list_shows_only_backup_files_newest_first() {
        let paths = temp_paths();
        let b = &paths.backups;
        write(&b.join("vault-100.clf"), b"old");
        write(&b.join("vault-300.clf"), b"newest!");
        write(&b.join("vault-200.clf"), b"mid");
        write(&b.join("notes.clf"), b"x");
        write(&b.join("vault-1.txt"), b"x");
        write(&b.join("other-vault-1.clf"), b"x");
        std::fs::create_dir(b.join("vault-dir.clf")).unwrap();
        set_age(&b.join("vault-100.clf"), 300);
        set_age(&b.join("vault-200.clf"), 200);
        set_age(&b.join("vault-300.clf"), 100);

        let list = list_backups(&paths).unwrap();
        let names: Vec<&str> = list.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["vault-300.clf", "vault-200.clf", "vault-100.clf"]);
        assert_eq!(list[0].size, 7);
        assert!(list[0].modified_ms > list[1].modified_ms && list[1].modified_ms > list[2].modified_ms);
    }

    #[test]
    fn list_of_a_missing_folder_is_empty() {
        let paths = temp_paths();
        std::fs::remove_dir_all(&paths.backups).unwrap();
        assert!(list_backups(&paths).unwrap().is_empty());
    }

    #[test]
    fn list_uses_camel_case_json() {
        let json = serde_json::to_string(&BackupInfo { name: "vault-1.clf".into(), modified_ms: 5, size: 7 }).unwrap();
        assert_eq!(json, r#"{"name":"vault-1.clf","modifiedMs":5,"size":7}"#);
    }

    #[test]
    fn a_bare_name_resolves_inside_the_backups_folder() {
        let paths = temp_paths();
        write(&paths.backups.join("vault-42.clf"), b"data");
        assert_eq!(resolve_source(&paths, "vault-42.clf", None).unwrap(), paths.backups.join("vault-42.clf"));
    }

    #[test]
    fn an_absolute_path_with_any_clf_name_resolves() {
        let paths = temp_paths();
        let f = paths.data.join("cipherleaf-backup-1.CLF");
        write(&f, b"data");
        let picked = fs::canonicalize(&f).unwrap();
        assert_eq!(resolve_source(&paths, &f.display().to_string(), Some(&picked)).unwrap(), f);
    }

    #[test]
    fn an_absolute_path_must_be_the_one_the_dialog_returned() {
        let paths = temp_paths();
        let a = paths.data.join("a.clf");
        let b = paths.data.join("b.clf");
        write(&a, b"data");
        write(&b, b"data");
        let a_text = a.display().to_string();
        let pick_a = fs::canonicalize(&a).unwrap();
        let pick_b = fs::canonicalize(&b).unwrap();
        let msg = "Choose the backup file with the file dialog.";
        // Nothing picked, or another file picked: refused with the plain message.
        for picked in [None, Some(pick_b.as_path())] {
            match resolve_source(&paths, &a_text, picked) {
                Err(Error::Invalid(m)) => assert_eq!(m, msg),
                other => panic!("expected refusal, got {other:?}"),
            }
        }
        // The picked one passes, also when spelled differently (with `..`).
        let odd = paths.backups.join("..").join("a.clf").display().to_string();
        assert!(resolve_source(&paths, &a_text, Some(&pick_a)).is_ok());
        assert!(resolve_source(&paths, &odd, Some(&pick_a)).is_ok());
        // A relative path gets the same message, and a bare name needs no pick.
        assert!(matches!(resolve_source(&paths, "a.clf", Some(&pick_a)), Err(Error::Invalid(_))));
        write(&paths.backups.join("vault-5.clf"), b"data");
        assert!(resolve_source(&paths, "vault-5.clf", None).is_ok());
        assert!(is_picked(&a, Some(&pick_a)) && !is_picked(&a, Some(&pick_b)) && !is_picked(&a, None));
    }

    #[test]
    fn bad_sources_are_rejected_for_the_right_reason() {
        let paths = temp_paths();
        let b = &paths.backups;
        write(&b.join("vault-1.clf"), b"data");
        write(&b.join("vault-2.txt"), b"data");
        write(&b.join("notes.clf"), b"data");
        write(&b.join("vault-empty.clf"), b"");
        // Real files whose names look like an attack: dots, and an NTFS stream
        // (`vault-ads` with a stream called `x.clf`, which opens as a plain file).
        write(&b.join("vault-dots..clf"), b"data");
        write(&b.join("vault-ads:x.clf"), b"data");
        assert!(b.join("vault-ads:x.clf").is_file() && b.join("vault-dots..clf").is_file());
        std::fs::create_dir(b.join("vault-dir.clf")).unwrap();
        write(&paths.vault, b"live");
        write(&paths.data.join(TEMP_NAME), b"temp");
        let abs = |p: PathBuf| p.display().to_string();

        rejected(&paths, "", "Choose a backup to restore.");
        rejected(&paths, "   ", "Choose a backup to restore.");
        // Bare names: wrong shape, even when a file of that name exists.
        rejected(&paths, "vault-2.txt", NOT_A_NAME);
        rejected(&paths, "notes.clf", NOT_A_NAME);
        rejected(&paths, "vault-dots..clf", NOT_A_NAME);
        rejected(&paths, "vault-ads:x.clf", NOT_A_NAME);
        // Right shape, but not usable.
        rejected(&paths, "vault-missing.clf", "That backup file wasn't found.");
        rejected(&paths, "vault-dir.clf", "That is a folder, not a backup file.");
        rejected(&paths, "vault-empty.clf", "That backup file is empty.");
        // Relative paths and parts of paths.
        rejected(&paths, "..\\vault-1.clf", DIALOG);
        rejected(&paths, "../vault-1.clf", DIALOG);
        rejected(&paths, "backups\\vault-1.clf", DIALOG);
        rejected(&paths, "\\vault-1.clf", DIALOG);
        // Network and device paths.
        rejected(&paths, r"\\server\share\x.clf", NETWORK);
        rejected(&paths, r"\\?\C:\x.clf", NETWORK);
        rejected(&paths, r"\\.\C:\x.clf", NETWORK);
        rejected(&paths, "//server/share/x.clf", NETWORK);
        // The live vault, spelled plainly and spelled with `..` inside.
        let live = "That is the vault in use now, not a backup.";
        rejected(&paths, &abs(paths.vault.clone()), live);
        rejected(&paths, &abs(b.join("..").join("vault.clf")), live);
        // The temporary copy has the wrong extension, so it never gets further.
        rejected(&paths, &abs(paths.data.join(TEMP_NAME)), NOT_CLF);
        // Absolute paths: wrong extension, a folder, and a missing file (never picked).
        rejected(&paths, &abs(b.join("vault-2.txt")), NOT_CLF);
        rejected(&paths, &abs(b.join("vault-dir.clf")), "That is a folder, not a backup file.");
        rejected(&paths, &abs(b.join("nothing.clf")), DIALOG);
        // And the good one still works.
        assert!(resolve_source(&paths, "vault-1.clf", None).is_ok());
    }

    #[test]
    fn install_keeps_the_old_vault_as_an_undo_file() {
        let paths = temp_paths();
        write(&paths.vault, b"OLD");
        write(&with_suffix(&paths.vault, "-journal"), b"OLDJ");
        write(&temp_path(&paths), b"NEW");

        let undo = install(&paths, 1234).unwrap().expect("an undo file");
        assert_eq!(undo, "restore-undo-1234.clf");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"NEW");
        assert_eq!(std::fs::read(paths.data.join(&undo)).unwrap(), b"OLD");
        assert_eq!(std::fs::read(paths.data.join("restore-undo-1234.clf-journal")).unwrap(), b"OLDJ");
        assert!(!temp_path(&paths).exists());
        assert!(!with_suffix(&paths.vault, "-journal").exists());
    }

    #[test]
    fn install_never_overwrites_an_earlier_undo_file() {
        let paths = temp_paths();
        write(&paths.data.join("restore-undo-50.clf"), b"EARLIER");
        write(&paths.vault, b"OLD");
        write(&temp_path(&paths), b"NEW");
        let undo = install(&paths, 50).unwrap().unwrap();
        assert_eq!(undo, "restore-undo-51.clf");
        assert_eq!(std::fs::read(paths.data.join("restore-undo-50.clf")).unwrap(), b"EARLIER");
        assert_eq!(std::fs::read(paths.data.join(undo)).unwrap(), b"OLD");
    }

    #[test]
    fn install_without_a_vault_makes_no_undo_file() {
        let paths = temp_paths();
        write(&temp_path(&paths), b"NEW");
        assert_eq!(install(&paths, 1).unwrap(), None);
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"NEW");
        assert!(!temp_path(&paths).exists());
        let undo_files = std::fs::read_dir(&paths.data)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(UNDO_PREFIX))
            .count();
        assert_eq!(undo_files, 0);
    }

    #[test]
    fn a_failed_install_puts_the_old_vault_back() {
        let paths = temp_paths();
        write(&paths.vault, b"OLD");
        write(&with_suffix(&paths.vault, "-journal"), b"OLDJ");
        // No temp file: the second rename fails.
        assert!(install(&paths, 7).is_err());
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"OLD");
        assert_eq!(std::fs::read(with_suffix(&paths.vault, "-journal")).unwrap(), b"OLDJ");
        assert!(!paths.data.join("restore-undo-7.clf").exists());
        assert!(!paths.data.join("restore-undo-7.clf-journal").exists());
    }

    /// A vault with `password` and one sticky note, saved as a backup in the backups folder.
    fn make_backup(paths: &Paths, name: &str, password: &str, keyfile: Option<&[u8; 32]>, text: &str) -> (PathBuf, String) {
        let src = paths.data.join(format!("source-{}.clf", new_id()));
        let mut s = Session::create(&src, password, keyfile, FAST).unwrap();
        let item = sticky(text);
        let id = item.id().to_string();
        s.put(item).unwrap();
        s.put_attachment("image/png", b"PIXELS").unwrap();
        let dest = paths.backups.join(name);
        s.backup_to(&dest).unwrap();
        (dest, id)
    }

    #[test]
    fn restore_end_to_end_with_the_right_password() {
        let paths = temp_paths();
        let (_, item_id) = make_backup(&paths, "vault-1.clf", "password-A", None, "from the backup");
        // A different live vault.
        let live = Session::create(&paths.vault, "password-B", None, FAST).unwrap();
        drop(live);
        let live_bytes = std::fs::read(&paths.vault).unwrap();

        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        verify_and_stage(&paths, &source, "password-A", None).unwrap();
        assert!(temp_path(&paths).exists());
        let undo = install(&paths, 99).unwrap().expect("the old vault is kept");

        let s = Session::unlock(&paths.vault, "password-A", None).unwrap();
        assert!(matches!(s.get(&item_id), Some(Item::Sticky(st)) if st.text == "from the backup"));
        assert!(Session::unlock(&paths.vault, "password-B", None).is_err());
        assert!(!temp_path(&paths).exists());
        // The old vault is intact under its undo name and still opens with its own password.
        assert_eq!(std::fs::read(paths.data.join(&undo)).unwrap(), live_bytes);
        assert!(Session::unlock(&paths.data.join(&undo), "password-B", None).is_ok());
        // The backup itself is untouched.
        assert!(Session::unlock(&paths.backups.join("vault-1.clf"), "password-A", None).is_ok());
    }

    #[test]
    fn restore_works_on_a_first_run_with_a_keyfile() {
        let paths = temp_paths();
        let kf = crate::crypto::random_bytes::<32>();
        let (_, item_id) = make_backup(&paths, "vault-1.clf", "password-A", Some(&kf), "kf");
        assert!(!paths.vault.exists());
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        // Without the keyfile it fails and installs nothing.
        assert!(matches!(verify_and_stage(&paths, &source, "password-A", None), Err(Error::BadCredentials)));
        assert!(!paths.vault.exists() && !temp_path(&paths).exists());
        verify_and_stage(&paths, &source, "password-A", Some(&kf)).unwrap();
        assert_eq!(install(&paths, 1).unwrap(), None);
        let s = Session::unlock(&paths.vault, "password-A", Some(&kf)).unwrap();
        assert!(s.get(&item_id).is_some());
    }

    #[test]
    fn a_wrong_password_changes_nothing() {
        let paths = temp_paths();
        make_backup(&paths, "vault-1.clf", "password-A", None, "secret");
        drop(Session::create(&paths.vault, "password-B", None, FAST).unwrap());
        let before = std::fs::read(&paths.vault).unwrap();
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();

        let r = verify_and_stage(&paths, &source, "not-the-password", None);
        assert!(matches!(r, Err(Error::BadCredentials)), "got {r:?}");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), before, "live vault must be byte-for-byte unchanged");
        assert!(!temp_path(&paths).exists(), "the checked copy must be removed");
        assert!(!with_suffix(&temp_path(&paths), "-journal").exists());
    }

    #[test]
    fn a_damaged_file_is_refused_and_leaves_no_temp_file() {
        let paths = temp_paths();
        write(&paths.vault, b"live");
        // Not a database at all.
        write(&paths.backups.join("vault-1.clf"), b"this is not a vault, just some text");
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        let r = verify_and_stage(&paths, &source, "pw", None);
        assert!(matches!(r, Err(Error::Corrupt)), "got {r:?}");
        // A real vault with a flipped byte in a record.
        let (backup, _) = make_backup(&paths, "vault-2.clf", "pw", None, "x");
        let conn = rusqlite::Connection::open(&backup).unwrap();
        let (id, mut data): (Vec<u8>, Vec<u8>) = conn
            .query_row("SELECT id, data FROM records LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        data[40] ^= 0xff;
        conn.execute("UPDATE records SET data = ?1 WHERE id = ?2", rusqlite::params![data, id]).unwrap();
        drop(conn);
        let source = resolve_source(&paths, "vault-2.clf", None).unwrap();
        let r = verify_and_stage(&paths, &source, "pw", None);
        assert!(matches!(r, Err(Error::Corrupt)), "got {r:?}");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"live");
        assert!(!temp_path(&paths).exists());
    }

    #[test]
    fn a_leftover_temp_file_is_replaced() {
        let paths = temp_paths();
        make_backup(&paths, "vault-1.clf", "pw", None, "x");
        write(&temp_path(&paths), b"stale");
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        verify_and_stage(&paths, &source, "pw", None).unwrap();
        install(&paths, 1).unwrap();
        assert!(Session::unlock(&paths.vault, "pw", None).is_ok());
    }

    #[test]
    fn a_corrupted_attachment_is_refused() {
        let paths = temp_paths();
        write(&paths.vault, b"live");
        let (backup, _) = make_backup(&paths, "vault-1.clf", "pw", None, "x");
        let conn = rusqlite::Connection::open(&backup).unwrap();
        let (id, mut data): (Vec<u8>, Vec<u8>) = conn
            .query_row("SELECT id, data FROM attachments LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        data[30] ^= 0xff;
        conn.execute("UPDATE attachments SET data = ?1 WHERE id = ?2", rusqlite::params![data, id]).unwrap();
        drop(conn);
        // The records are fine, so a plain unlock still works: only the attachment check catches it.
        assert!(Session::unlock(&backup, "pw", None).is_ok());
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        let r = verify_and_stage(&paths, &source, "pw", None);
        assert!(matches!(r, Err(Error::Corrupt)), "got {r:?}");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"live");
        assert!(!temp_path(&paths).exists());
    }

    #[test]
    fn unusual_password_settings_are_refused() {
        let ok = |m_kib, t, p| check_restore_kdf(&KdfParams { m_kib, t, p });
        assert!(ok(256 * 1024, 3, 4).is_ok(), "the app's own settings");
        assert!(ok(1_048_576, 10, 16).is_ok(), "exactly at the limits");
        for (m_kib, t, p) in [(1_048_577, 3, 4), (256 * 1024, 11, 4), (256 * 1024, 3, 17), (4 * 1024 * 1024, 64, 64)] {
            match ok(m_kib, t, p) {
                Err(Error::Invalid(m)) => {
                    assert_eq!(m, "This backup uses unusual password settings and can't be restored here.")
                }
                other => panic!("({m_kib}, {t}, {p}) should be refused, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_stray_journal_is_moved_aside_when_there_is_no_vault() {
        let paths = temp_paths();
        write(&with_suffix(&paths.vault, "-journal"), b"STRAY");
        write(&temp_path(&paths), b"NEW");
        assert_eq!(install(&paths, 5).unwrap(), None, "no old vault, so no undo vault file");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"NEW");
        assert!(!with_suffix(&paths.vault, "-journal").exists());
        assert_eq!(std::fs::read(paths.data.join("restore-undo-5.clf-journal")).unwrap(), b"STRAY");
    }

    #[test]
    fn a_failed_install_puts_a_stray_journal_back() {
        let paths = temp_paths();
        write(&with_suffix(&paths.vault, "-journal"), b"STRAY");
        assert!(install(&paths, 5).is_err()); // no temp file
        assert_eq!(std::fs::read(with_suffix(&paths.vault, "-journal")).unwrap(), b"STRAY");
        assert!(!paths.data.join("restore-undo-5.clf-journal").exists());
        assert!(!paths.vault.exists());
    }

    #[test]
    fn a_failed_rollback_is_reported_in_the_error() {
        // Making the rollback itself fail is hard to arrange, so this checks the message.
        let e = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let notes = ["Your previous vault could not be put back; it is saved as restore-undo-9.clf in the data folder.".to_string()];
        let text = install_failure(e, &notes).to_string();
        assert!(text.contains("could not be put back") && text.contains("restore-undo-9.clf"), "{text}");
        assert!(!text.contains("access denied"), "raw error text stays out of the UI: {text}");
        assert!(matches!(install_failure(io::Error::other("x"), &[]), Error::Io(_)));
    }

    #[test]
    fn roll_back_puts_the_vault_and_the_journal_back() {
        let paths = temp_paths();
        let base = paths.data.join("restore-undo-3.clf");
        write(&base, b"OLD");
        write(&with_suffix(&base, "-journal"), b"OLDJ");
        let notes = roll_back(&paths, &base, Moved { vault: true, journal: true }, 1, Duration::ZERO);
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"OLD");
        assert_eq!(std::fs::read(with_suffix(&paths.vault, "-journal")).unwrap(), b"OLDJ");
    }

    #[test]
    fn roll_back_leaves_the_journal_if_the_vault_did_not_come_back() {
        let paths = temp_paths();
        let base = paths.data.join("restore-undo-3.clf");
        // The undo vault is gone (so putting it back fails) but its journal is there.
        write(&with_suffix(&base, "-journal"), b"OLDJ");
        let notes = roll_back(&paths, &base, Moved { vault: true, journal: true }, 1, Duration::ZERO);
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert!(notes[0].contains("could not be put back") && notes[0].contains("restore-undo-3.clf"));
        assert!(notes[1].contains("restore-undo-3.clf-journal"));
        assert!(with_suffix(&base, "-journal").exists(), "journal stays where it is");
        assert!(!with_suffix(&paths.vault, "-journal").exists(), "and is not put next to a missing vault");
    }

    #[test]
    fn raw_file_errors_become_one_plain_message() {
        for raw in [
            io::Error::from_raw_os_error(32), // sharing violation
            io::Error::from_raw_os_error(5),  // access denied
            io::Error::new(io::ErrorKind::PermissionDenied, "x"),
        ] {
            match plain_io(Error::Io(raw)) {
                Error::Other(m) => assert_eq!(m, IN_USE),
                other => panic!("expected the plain message, got {other:?}"),
            }
        }
        assert!(matches!(plain_io(Error::Io(io::Error::from_raw_os_error(112))), Error::Other(m) if m.contains("free space")));
        assert!(matches!(plain_io(Error::Corrupt), Error::Corrupt));
        assert!(matches!(plain_io(Error::invalid("x")), Error::Invalid(_)));
        assert!(!IN_USE.contains("os error"));
    }

    #[test]
    fn unusual_password_settings_in_the_file_are_refused_before_any_key_work() {
        let paths = temp_paths();
        write(&paths.vault, b"live");
        let (backup, _) = make_backup(&paths, "vault-1.clf", "pw", None, "x");
        // Header layout: magic(4) version(2) m_kib(4)... Not authenticated when
        // read, so only the limit check can stop it before the (huge) Argon2 run.
        let conn = rusqlite::Connection::open(&backup).unwrap();
        let mut header: Vec<u8> = conn.query_row("SELECT data FROM header WHERE id = 1", [], |r| r.get(0)).unwrap();
        header[6..10].copy_from_slice(&2_000_000u32.to_le_bytes());
        conn.execute("UPDATE header SET data = ?1 WHERE id = 1", rusqlite::params![header]).unwrap();
        drop(conn);
        let source = resolve_source(&paths, "vault-1.clf", None).unwrap();
        let started = std::time::Instant::now();
        match verify_and_stage(&paths, &source, "pw", None) {
            Err(Error::Invalid(m)) => {
                assert_eq!(m, "This backup uses unusual password settings and can't be restored here.")
            }
            other => panic!("expected the unusual-settings refusal, got {other:?}"),
        }
        assert!(started.elapsed() < Duration::from_secs(5), "no key derivation should have run");
        assert_eq!(std::fs::read(&paths.vault).unwrap(), b"live");
        assert!(!temp_path(&paths).exists());
    }

    #[test]
    fn rename_retry_gives_up_at_once_for_a_missing_file() {
        let paths = temp_paths();
        let started = std::time::Instant::now();
        let r = rename_retry(&paths.data.join("nope"), &paths.data.join("x"), 6, Duration::from_millis(300));
        assert!(r.is_err());
        assert!(started.elapsed() < Duration::from_millis(250));
    }
}
