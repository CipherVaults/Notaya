//! Helpers for vaults whose security is off (amendment A21): finding the earlier
//! copies of the vault that are readable by anyone, and removing them on request.
//!
//! "Earlier copies" are the automatic backups (`data/backups/vault-*.clf`) and the
//! old vaults a restore keeps (`data/restore-undo-*.clf`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::portable::Paths;
use crate::restore::{is_backup_name, UNDO_PREFIX};
use crate::vault::Session;

fn names_in(dir: &Path, wanted: fn(&str) -> bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| e.file_name().to_str().is_some_and(wanted))
        .map(|e| e.path())
        .collect()
}

fn is_undo_name(name: &str) -> bool {
    name.starts_with(UNDO_PREFIX) && name.ends_with(".clf")
}

/// Every earlier copy of the vault in the data folder.
pub fn earlier_copies(paths: &Paths) -> Vec<PathBuf> {
    let mut all = names_in(&paths.backups, is_backup_name);
    all.extend(names_in(&paths.data, is_undo_name));
    all
}

/// True if there is any earlier copy. When the live vault is missing but copies exist,
/// the app must not quietly start an empty vault: the person is asked what to do.
pub fn has_earlier_copies(paths: &Paths) -> bool {
    !earlier_copies(paths).is_empty()
}

/// The automatic backups whose header says "security off": anyone with the file can read them.
/// A file whose header can't be read is left out (it is never touched). The old vaults a
/// restore keeps (`restore-undo-*.clf`) are NOT included: they hold notes that exist nowhere
/// else, so the app never deletes them.
pub fn unprotected_copies(paths: &Paths) -> Vec<PathBuf> {
    names_in(&paths.backups, is_backup_name)
        .into_iter()
        .filter(|p| matches!(Session::read_header(p), Ok(h) if h.open))
        .collect()
}

/// Deletes the unprotected automatic backups (and their `-journal` files) and returns how
/// many were removed. Protected backups, restore-undo files and anything unreadable stay.
pub fn delete_unprotected(paths: &Paths) -> usize {
    let mut removed = 0;
    for path in unprotected_copies(paths) {
        let mut journal = path.clone().into_os_string();
        journal.push("-journal");
        let _ = fs::remove_file(PathBuf::from(journal));
        if fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    use crate::model::new_id;

    const FAST: KdfParams = KdfParams { m_kib: 64, t: 1, p: 1 };

    fn paths() -> Paths {
        let root = std::env::temp_dir().join(format!("cipherleaf-open-{}", new_id()));
        let paths = Paths::at(&root, root.join("kp"));
        fs::create_dir_all(&paths.backups).unwrap();
        paths
    }

    #[test]
    fn only_open_copies_are_found_and_deleted() {
        let paths = paths();
        drop(Session::create_open(&paths.backups.join("vault-1.clf")).unwrap());
        drop(Session::create(&paths.backups.join("vault-2.clf"), "pw", None, FAST).unwrap());
        drop(Session::create_open(&paths.data.join("restore-undo-5.clf")).unwrap());
        fs::write(paths.backups.join("vault-3.clf"), b"not a vault").unwrap(); // unreadable: left alone
        fs::write(paths.backups.join("notes.txt"), b"x").unwrap();
        assert!(has_earlier_copies(&paths));
        assert_eq!(unprotected_copies(&paths).len(), 1);
        assert_eq!(delete_unprotected(&paths), 1);
        assert!(!paths.backups.join("vault-1.clf").exists());
        assert!(paths.data.join("restore-undo-5.clf").exists(), "an old vault kept by a restore is never deleted");
        assert!(paths.backups.join("vault-2.clf").exists(), "a protected backup stays");
        assert!(paths.backups.join("vault-3.clf").exists(), "an unreadable file stays");
        assert!(paths.backups.join("notes.txt").exists());
        assert_eq!(delete_unprotected(&paths), 0);
    }

    #[test]
    fn an_empty_folder_has_no_earlier_copies() {
        let paths = paths();
        assert!(!has_earlier_copies(&paths));
        assert!(unprotected_copies(&paths).is_empty());
    }
}
