//! Bringing the notes of a portable copy into an installed Notaya (copy only).
//!
//! The rules: the portable copy is never changed, moved or deleted, and nothing is guessed. The
//! vault is copied to `data/import-check.tmp`, that copy's header is read and checked, and only
//! then is it renamed to `data/vault.clf`. It is never done over an existing vault. The file is
//! copied as it is, so a vault with a master password keeps the same password and keyfile and
//! nothing is opened or decrypted here. The automatic backups are copied too, best effort.

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::portable::Paths;
use crate::restore::{
    check_restore_kdf, copy_file, is_backup_name, list_in, rename_retry, MAX_BACKUP_BYTES, RENAME_ATTEMPTS, RENAME_DELAY,
};
use crate::vault::Session;

/// The checked copy of the vault, before it is renamed to `vault.clf`.
const TEMP_NAME: &str = "import-check.tmp";
/// Backups copied at most (the newest first). The app keeps about this many itself.
const MAX_IMPORTED_BACKUPS: usize = 20;

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    /// Backups copied next to the vault.
    pub backups_copied: usize,
    /// Backups that were not copied (too large, unreadable, or already there).
    pub backups_skipped: usize,
}

const NOT_FOUND: &str = "No Notaya notes were found there. Choose the portable Notaya folder (the one with Notaya.exe in it) or its data folder.";
const NETWORK: &str = "A folder on a network or device path can't be used. Copy the portable folder to this PC first.";

/// Where the vault of a portable copy is: `<dir>\data\vault.clf` (the folder with the program in
/// it) or `<dir>\vault.clf` (its data folder). Returns the vault file.
fn find_vault(dir: &Path) -> Option<PathBuf> {
    [dir.join("data").join("vault.clf"), dir.join("vault.clf")].into_iter().find(|p| p.is_file())
}

fn with_journal(path: &Path) -> PathBuf {
    let mut name = path.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push("-journal");
    path.with_file_name(name)
}

/// A file error as one plain sentence (the raw error goes to stderr).
fn plain(e: Error) -> Error {
    let Error::Io(io_err) = e else {
        return e;
    };
    eprintln!("import: file error: {io_err}");
    // 112 = disk full, 39 = handle disk full.
    let full = io_err.kind() == io::ErrorKind::StorageFull || matches!(io_err.raw_os_error(), Some(112 | 39));
    Error::Other(
        if full {
            "There is not enough free space on the drive to copy your notes."
        } else {
            "The notes file could not be copied. Close the portable Notaya if it is running, and close programs that may be using the file (antivirus or a file-sync tool, for example), then try again."
        }
        .to_string(),
    )
}

fn remove_temp(paths: &Paths) {
    let tmp = paths.data.join(TEMP_NAME);
    let _ = fs::remove_file(with_journal(&tmp));
    let _ = fs::remove_file(&tmp);
}

/// Copies the vault (and backups) of the portable copy in `chosen` into `paths`. Only while there
/// is no vault here.
pub fn import_portable(paths: &Paths, chosen: &Path) -> Result<ImportSummary> {
    let s = chosen.to_string_lossy();
    if s.starts_with("\\\\") || s.starts_with("//") {
        return Err(Error::invalid(NETWORK));
    }
    let chosen = fs::canonicalize(chosen).map_err(|_| Error::invalid(NOT_FOUND))?;
    if chosen.to_string_lossy().starts_with(r"\\?\UNC\") {
        return Err(Error::invalid(NETWORK));
    }
    let source = find_vault(&chosen).ok_or_else(|| Error::invalid(NOT_FOUND))?;
    let source = fs::canonicalize(&source).map_err(|_| Error::invalid(NOT_FOUND))?;
    let source_data = source.parent().map(Path::to_path_buf).unwrap_or_else(|| chosen.clone());

    // Never from the place the notes are going to.
    if fs::canonicalize(&paths.data).is_ok_and(|live| live == source_data) {
        return Err(Error::invalid("That is the folder this Notaya already uses. Choose your portable Notaya folder."));
    }
    // A journal next to the vault means a program is writing to it (or crashed while doing so).
    if with_journal(&source).exists() {
        return Err(Error::invalid(
            "The portable Notaya seems to be open (or closed in the middle of saving). Close it (it cannot run next to an installed Notaya, so close this one first, start the portable one, close it, and start this one again), then try again.",
        ));
    }
    let meta = fs::metadata(&source).map_err(|_| Error::invalid("That notes file can't be read."))?;
    if meta.len() == 0 {
        return Err(Error::invalid("That notes file is empty."));
    }
    if meta.len() > MAX_BACKUP_BYTES {
        return Err(Error::invalid("That file is too large to be a Notaya notes file."));
    }

    // Never over notes that are already here.
    if paths.vault.try_exists()? || with_journal(&paths.vault).try_exists()? {
        return Err(Error::invalid("This Notaya already has notes. Importing never replaces them."));
    }

    fs::create_dir_all(&paths.data).map_err(Error::from).map_err(plain)?;
    remove_temp(paths);
    let tmp = paths.data.join(TEMP_NAME);
    let staged = stage(&source, &tmp);
    if staged.is_err() {
        remove_temp(paths);
    }
    staged?;

    // The rename would replace a file that appeared meanwhile, so look once more.
    if paths.vault.try_exists()? {
        remove_temp(paths);
        return Err(Error::invalid("This Notaya already has notes. Importing never replaces them."));
    }
    if let Err(e) = rename_retry(&tmp, &paths.vault, RENAME_ATTEMPTS, RENAME_DELAY) {
        remove_temp(paths);
        return Err(plain(Error::Io(e)));
    }

    let (backups_copied, backups_skipped) = copy_backups(&source_data.join("backups"), &paths.backups);
    Ok(ImportSummary { backups_copied, backups_skipped })
}

/// Copies `source` to `tmp` and checks the copy: a Notaya vault header with sane settings.
fn stage(source: &Path, tmp: &Path) -> Result<()> {
    copy_file(source, tmp).map_err(plain)?;
    let header = match Session::read_header(tmp) {
        Ok(h) => h,
        // Not a SQLite file at all, or tables missing: a damaged or foreign file.
        Err(Error::Db(_) | Error::Corrupt) => {
            return Err(Error::invalid("That file is not a Notaya notes file, or it is damaged."));
        }
        Err(e) => return Err(e),
    };
    check_restore_kdf(&header.kdf)?;
    // Needs a write handle on Windows (FlushFileBuffers); nothing is written.
    OpenOptions::new().write(true).open(tmp).and_then(|f| f.sync_all()).map_err(Error::from).map_err(plain)?;
    Ok(())
}

/// Copies the newest `vault-*.clf` backups. Each one is a new file; a failure only counts as skipped.
fn copy_backups(from: &Path, to: &Path) -> (usize, usize) {
    let Ok(list) = list_in(from, is_backup_name) else {
        return (0, 0);
    };
    if fs::create_dir_all(to).is_err() {
        return (0, list.len().min(MAX_IMPORTED_BACKUPS));
    }
    let (mut copied, mut skipped) = (0, 0);
    for info in list.iter().take(MAX_IMPORTED_BACKUPS) {
        let src = from.join(&info.name);
        let ok = info.size > 0
            && info.size <= MAX_BACKUP_BYTES
            && fs::canonicalize(&src).is_ok_and(|real| real.parent() == fs::canonicalize(from).ok().as_deref())
            && copy_file(&src, &to.join(&info.name)).is_ok();
        if ok {
            copied += 1;
        } else {
            skipped += 1;
        }
    }
    (copied, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    use crate::model::new_id;

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    /// An installed-style target (no vault yet) and a portable-style source.
    fn setup() -> (Paths, PathBuf) {
        let root = std::env::temp_dir().join(format!("notaya-import-{}", new_id()));
        let target = Paths::at(&root.join("installed"));
        fs::create_dir_all(&target.backups).unwrap();
        let portable = root.join("portable");
        fs::create_dir_all(portable.join("data").join("backups")).unwrap();
        (target, portable)
    }

    fn make_open(path: &Path) {
        drop(Session::create_open(path).unwrap());
    }

    #[test]
    fn copies_the_vault_and_backups_and_leaves_the_portable_copy_alone() {
        let (target, portable) = setup();
        let src_vault = portable.join("data").join("vault.clf");
        make_open(&src_vault);
        make_open(&portable.join("data").join("backups").join("vault-100.clf"));
        make_open(&portable.join("data").join("backups").join("vault-predelete-200.clf"));
        fs::write(portable.join("data").join("backups").join("readme.txt"), b"x").unwrap();
        make_open(&portable.join("data").join("restore-undo-5.clf"));
        let before = fs::read(&src_vault).unwrap();

        let summary = import_portable(&target, &portable).unwrap();
        assert_eq!(summary, ImportSummary { backups_copied: 2, backups_skipped: 0 });
        assert_eq!(fs::read(&target.vault).unwrap(), before, "the vault is copied byte for byte");
        assert_eq!(fs::read(&src_vault).unwrap(), before, "the portable vault is unchanged");
        assert!(target.backups.join("vault-100.clf").is_file());
        assert!(target.backups.join("vault-predelete-200.clf").is_file());
        assert!(!target.backups.join("readme.txt").exists());
        assert!(!target.data.join("restore-undo-5.clf").exists(), "old vaults kept by a restore stay where they are");
        assert!(!target.data.join(TEMP_NAME).exists());
        assert!(portable.join("data").join("backups").join("vault-100.clf").is_file());
        assert!(Session::read_header(&target.vault).unwrap().open);
    }

    #[test]
    fn a_vault_with_a_password_keeps_it() {
        let (target, portable) = setup();
        drop(Session::create(&portable.join("data").join("vault.clf"), "correct horse battery", None, FAST).unwrap());
        import_portable(&target, &portable).unwrap();
        assert!(!Session::read_header(&target.vault).unwrap().open);
        assert!(Session::unlock(&target.vault, "correct horse battery", None).is_ok());
    }

    #[test]
    fn the_data_folder_itself_can_be_chosen() {
        let (target, portable) = setup();
        make_open(&portable.join("data").join("vault.clf"));
        assert!(import_portable(&target, &portable.join("data")).is_ok());
        assert!(target.vault.is_file());
    }

    #[test]
    fn nothing_is_replaced_when_there_are_notes_already() {
        let (target, portable) = setup();
        make_open(&portable.join("data").join("vault.clf"));
        make_open(&target.vault);
        let mine = fs::read(&target.vault).unwrap();
        assert!(import_portable(&target, &portable).is_err());
        assert_eq!(fs::read(&target.vault).unwrap(), mine);
        assert!(!target.data.join(TEMP_NAME).exists());
    }

    #[test]
    fn a_folder_without_notes_is_refused() {
        let (target, portable) = setup();
        assert!(import_portable(&target, &portable).is_err());
        assert!(import_portable(&target, &portable.join("missing")).is_err());
        assert!(!target.vault.exists());
    }

    #[test]
    fn a_file_that_is_not_a_vault_is_refused_and_leaves_nothing() {
        let (target, portable) = setup();
        fs::write(portable.join("data").join("vault.clf"), b"this is not a vault").unwrap();
        let err = import_portable(&target, &portable).unwrap_err();
        assert!(err.to_string().contains("not a Notaya notes file"), "{err}");
        assert!(!target.vault.exists());
        assert!(!target.data.join(TEMP_NAME).exists());
    }

    #[test]
    fn an_empty_file_is_refused() {
        let (target, portable) = setup();
        fs::write(portable.join("data").join("vault.clf"), b"").unwrap();
        assert!(import_portable(&target, &portable).is_err());
        assert!(!target.vault.exists());
    }

    #[test]
    fn a_portable_copy_that_is_being_written_is_refused() {
        let (target, portable) = setup();
        let v = portable.join("data").join("vault.clf");
        make_open(&v);
        fs::write(with_journal(&v), b"half a write").unwrap();
        let err = import_portable(&target, &portable).unwrap_err();
        assert!(err.to_string().contains("seems to be open"), "{err}");
        assert!(!target.vault.exists());
    }

    #[test]
    fn the_folder_in_use_cannot_be_the_source() {
        let (target, _portable) = setup();
        make_open(&target.vault);
        let err = import_portable(&target, &target.data).unwrap_err();
        assert!(err.to_string().contains("already uses"), "{err}");
    }

    #[test]
    fn only_the_newest_backups_are_copied() {
        let (target, portable) = setup();
        make_open(&portable.join("data").join("vault.clf"));
        for n in 0..(MAX_IMPORTED_BACKUPS + 5) {
            make_open(&portable.join("data").join("backups").join(format!("vault-{}.clf", 1000 + n)));
        }
        let summary = import_portable(&target, &portable).unwrap();
        assert_eq!(summary.backups_copied, MAX_IMPORTED_BACKUPS);
    }

    #[test]
    fn a_backup_that_is_already_there_is_skipped_not_replaced() {
        let (target, portable) = setup();
        make_open(&portable.join("data").join("vault.clf"));
        make_open(&portable.join("data").join("backups").join("vault-100.clf"));
        fs::write(target.backups.join("vault-100.clf"), b"mine").unwrap();
        let summary = import_portable(&target, &portable).unwrap();
        assert_eq!(summary, ImportSummary { backups_copied: 0, backups_skipped: 1 });
        assert_eq!(fs::read(target.backups.join("vault-100.clf")).unwrap(), b"mine");
    }

    #[test]
    fn network_paths_are_refused() {
        let (target, _portable) = setup();
        assert!(import_portable(&target, Path::new(r"\\server\share\Notaya")).is_err());
        assert!(!target.vault.exists());
    }
}
