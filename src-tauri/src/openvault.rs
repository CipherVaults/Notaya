//! Helpers for vaults whose security is off (amendment A21): finding the earlier
//! copies of the vault that are readable by anyone, and removing them on request.
//!
//! "Earlier copies" are the automatic and safety backups (`data/backups/vault-*.nota` (and `.clf`), which
//! includes `vault-predelete-*.nota`, the copy made before a notebook or section is deleted) and the
//! old vaults a restore keeps (`data/restore-undo-*.nota`) and the vaults "Start over"
//! sets aside (`data/set-aside-*.nota`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::portable::Paths;
use crate::restore::{is_backup_name, SET_ASIDE_PREFIX, UNDO_PREFIX};
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
    name.starts_with(UNDO_PREFIX) && crate::filenames::has_vault_ext(name)
}

fn is_set_aside_name(name: &str) -> bool {
    name.starts_with(SET_ASIDE_PREFIX) && crate::filenames::has_vault_ext(name)
}

/// The V1.3.0 vault file that Notaya set aside when it made `vault.nota` from it (`vault-before-notaya.clf`).
fn is_before_notaya_name(name: &str) -> bool {
    name.starts_with("vault-before-notaya") && crate::filenames::has_vault_ext(name)
}

/// A `vault.clf` that stayed behind next to a `vault.nota` (if the start stopped between copying the notes
/// and setting the old file aside): it holds the notes as V1.3.0 left them, so it counts as an earlier copy.
fn left_over_old_vault(paths: &Paths) -> Vec<PathBuf> {
    if paths.vault != paths.old_vault && paths.vault.is_file() && paths.old_vault.is_file() {
        vec![paths.old_vault.clone()]
    } else {
        Vec::new()
    }
}

/// Every earlier copy of the vault in the data folder, including a vault the person set
/// aside with "Start over" (it is never deleted either).
pub fn earlier_copies(paths: &Paths) -> Vec<PathBuf> {
    let mut all = names_in(&paths.backups, is_backup_name);
    all.extend(names_in(&paths.data, is_undo_name));
    all.extend(names_in(&paths.data, is_set_aside_name));
    all.extend(names_in(&paths.data, is_before_notaya_name));
    all.extend(left_over_old_vault(paths));
    all
}

/// True if there is any earlier copy. When the live vault is missing but copies exist,
/// the app must not quietly start an empty vault: the person is asked what to do.
pub fn has_earlier_copies(paths: &Paths) -> bool {
    !earlier_copies(paths).is_empty()
}

/// The backups (automatic and `vault-predelete-*`, and the `vault-before-notaya` file) whose header says "security off": anyone with the file can read them.
/// A file whose header can't be read is left out (it is never touched). The old vaults a
/// restore keeps (`restore-undo-*.clf`) are NOT included: they hold notes that exist nowhere
/// else, so the app never deletes them.
pub fn unprotected_copies(paths: &Paths) -> Vec<PathBuf> {
    let mut all = names_in(&paths.backups, is_backup_name);
    // The copy of the notes as V1.3.0 left them is as readable as an unprotected backup, if security was off.
    all.extend(names_in(&paths.data, is_before_notaya_name));
    all.extend(left_over_old_vault(paths));
    all.into_iter()
        .filter(|p| matches!(Session::read_header(p), Ok(h) if h.open))
        .collect()
}

/// Deletes the unprotected backups, safety backups included (and their `-journal` files) and returns how
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
        let root = std::env::temp_dir().join(format!("notaya-open-{}", new_id()));
        let paths = Paths::at(&root);
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
    fn safety_backups_made_with_security_off_are_deleted_too_but_nothing_else_is() {
        let paths = paths();
        drop(Session::create_open(&paths.backups.join("vault-predelete-1.clf")).unwrap());
        drop(Session::create(&paths.backups.join("vault-predelete-2.clf"), "pw", None, FAST).unwrap());
        drop(Session::create_open(&paths.backups.join("vault-3.clf")).unwrap());
        fs::write(paths.backups.join("vault-predelete-4.clf"), b"not a vault").unwrap();
        drop(Session::create_open(&paths.data.join("restore-undo-5.clf")).unwrap());
        drop(Session::create_open(&paths.data.join("set-aside-6.clf")).unwrap());
        assert_eq!(earlier_copies(&paths).len(), 6);
        assert_eq!(unprotected_copies(&paths).len(), 2);
        assert_eq!(delete_unprotected(&paths), 2);
        assert!(!paths.backups.join("vault-predelete-1.clf").exists());
        assert!(!paths.backups.join("vault-3.clf").exists());
        assert!(paths.backups.join("vault-predelete-2.clf").exists(), "a protected safety backup stays");
        assert!(paths.backups.join("vault-predelete-4.clf").exists(), "an unreadable file stays");
        assert!(paths.data.join("restore-undo-5.clf").exists());
        assert!(paths.data.join("set-aside-6.clf").exists());
    }

    #[test]
    fn hourly_copies_made_with_security_off_are_deleted_too() {
        let paths = paths();
        drop(Session::create_open(&paths.backups.join("vault-hourly-1.clf")).unwrap());
        drop(Session::create(&paths.backups.join("vault-hourly-2.clf"), "pw", None, FAST).unwrap());
        assert_eq!(unprotected_copies(&paths).len(), 1);
        assert_eq!(delete_unprotected(&paths), 1);
        assert!(!paths.backups.join("vault-hourly-1.clf").exists());
        assert!(paths.backups.join("vault-hourly-2.clf").exists(), "a protected hourly copy stays");
    }

    #[test]
    fn an_empty_folder_has_no_earlier_copies() {
        let paths = paths();
        assert!(!has_earlier_copies(&paths));
        assert!(unprotected_copies(&paths).is_empty());
    }

    #[test]
    fn a_vault_set_aside_by_start_over_counts_as_an_earlier_copy_and_is_never_deleted() {
        let paths = paths();
        drop(Session::create_open(&paths.data.join("set-aside-9.clf")).unwrap());
        assert!(has_earlier_copies(&paths));
        assert!(unprotected_copies(&paths).is_empty());
        assert_eq!(delete_unprotected(&paths), 0);
        assert!(paths.data.join("set-aside-9.clf").exists());
    }

    #[test]
    fn the_file_kept_from_v1_3_0_counts_as_an_earlier_copy() {
        let paths = paths();
        drop(Session::create_legacy_open(&paths.data.join("vault-before-notaya.clf")).unwrap());
        drop(Session::create_legacy(&paths.data.join("vault-before-notaya-2.clf"), "pw", None, FAST).unwrap());
        assert!(has_earlier_copies(&paths));
        assert_eq!(unprotected_copies(&paths).len(), 1, "only the one with security off");
        assert_eq!(delete_unprotected(&paths), 1);
        assert!(!paths.data.join("vault-before-notaya.clf").exists());
        assert!(paths.data.join("vault-before-notaya-2.clf").exists(), "a protected one stays");
    }

    #[test]
    fn a_left_over_vault_clf_next_to_vault_nota_counts_as_an_earlier_copy() {
        let paths = paths();
        drop(Session::create_open(&paths.vault).unwrap());
        assert!(!has_earlier_copies(&paths), "the live vault alone is not an earlier copy");
        drop(Session::create_legacy_open(&paths.old_vault).unwrap());
        assert!(has_earlier_copies(&paths));
        assert_eq!(unprotected_copies(&paths), vec![paths.old_vault.clone()]);
        assert_eq!(delete_unprotected(&paths), 1);
        assert!(paths.vault.exists() && !paths.old_vault.exists());
    }
}
