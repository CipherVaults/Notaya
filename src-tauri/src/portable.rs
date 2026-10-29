//! Where the notes live. Portable: everything is in a `data` folder next to the executable,
//! so the app can run from a USB stick and leaves nothing behind on the PC. Installed (by the
//! Windows setup): `%LOCALAPPDATA%\Notaya Notes\data`, because the program folder is not a
//! place to keep notes. The uninstaller never touches that folder.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use crate::filenames::{BEFORE_NOTAYA_FILE, OLD_VAULT_FILE, VAULT_FILE};
use crate::vault::store::Store;
use crate::vault::Session;

/// A file next to the executable that makes it portable. The portable ZIP carries one.
pub const PORTABLE_MARKER: &str = "portable.txt";
/// The folder under `%LOCALAPPDATA%` that an installed copy keeps its notes in. It is neither
/// `Notaya` (where the setup puts the program itself, so the uninstaller works in it) nor the app
/// identifier (the uninstaller's "delete the application data" box removes that folder), so
/// nothing the setup or the uninstaller does can reach the notes.
pub const INSTALLED_FOLDER: &str = "Notaya Notes";

#[derive(Clone, Debug)]
pub struct Paths {
    pub data: PathBuf,
    /// The vault: `data/vault.nota`. While the notes could not be moved from V1.3.0's `vault.clf` yet
    /// (see [`Paths::adopt_old_vault`]) this is that file instead, so nothing is ever started empty.
    pub vault: PathBuf,
    /// The vault file of V1.3.0 and earlier, `data/vault.clf`.
    pub old_vault: PathBuf,
    pub backups: PathBuf,
    /// WebView2 profile folder (cache, local storage) kept with the app, not in AppData.
    pub webview: PathBuf,
    /// The notes are in the per-user install location, not next to the executable.
    pub installed: bool,
}

/// Reads an environment variable by its Notaya name, or by the name it had until V1.3.0 (so scripts and
/// shortcuts that still set the old one keep working). An empty value counts as not set.
pub fn env_var(name: &str, old_name: &str) -> Option<std::ffi::OsString> {
    [name, old_name].iter().filter_map(|n| std::env::var_os(n)).find(|v| !v.is_empty())
}

impl Paths {
    pub fn resolve() -> Paths {
        // Tests and tools point the app at a throwaway folder with this.
        if let Some(root) = env_var("NOTAYA_HOME", "CIPHERLEAF_HOME") {
            let mut paths = Paths::at(&PathBuf::from(root));
            paths.adopt_old_vault();
            return paths;
        }
        let (root, installed) = default_root();
        let mut paths = Paths { installed, ..Paths::at(&root) };
        paths.adopt_old_vault();
        paths
    }

    pub fn at(root: &Path) -> Paths {
        let data = root.join("data");
        Paths {
            vault: data.join(VAULT_FILE),
            old_vault: data.join(OLD_VAULT_FILE),
            backups: data.join("backups"),
            webview: data.join("webview"),
            data,
            installed: false,
        }
    }

    /// V1.4.0 renamed the vault file from `vault.clf` to `vault.nota`. When only the old file exists, this
    /// copies it to `vault.nota` (a checked copy, renamed into place only when complete) and keeps the old
    /// file as `vault-before-notaya.clf`. Notaya never deletes it on its own (only the person's choice to delete earlier unprotected copies does). The notes inside are upgraded to the new
    /// format at the next unlock (`Session::upgrade_format`), because that needs the password. If the copy
    /// cannot be made (a read-only drive, a damaged file), the old file stays the vault, so the notes are
    /// still opened and no empty vault is started; it is tried again at the next start.
    pub fn adopt_old_vault(&mut self) {
        if let Err(e) = self.try_adopt_old_vault() {
            eprintln!("vault: could not make {VAULT_FILE} from {OLD_VAULT_FILE} yet ({e}); using {OLD_VAULT_FILE} for now");
            self.vault = self.old_vault.clone();
        }
    }

    fn try_adopt_old_vault(&self) -> Result<(), String> {
        if self.vault == self.old_vault || self.vault.exists() || !self.old_vault.is_file() {
            return Ok(());
        }
        let part = with_suffix(&self.vault, ".part");
        let _ = fs::remove_file(with_suffix(&part, "-journal"));
        let _ = fs::remove_file(&part); // a leftover from an interrupted start
        let copied = (|| -> Result<(), String> {
            // SQLite makes the copy, so a write that was cut short is rolled back first.
            Store::open(&self.old_vault).and_then(|s| s.backup_to(&part)).map_err(|e| e.to_string())?;
            Session::read_header(&part).map_err(|e| e.to_string())?;
            OpenOptions::new().write(true).open(&part).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
            fs::rename(&part, &self.vault).map_err(|e| e.to_string())
        })();
        if let Err(e) = copied {
            let _ = fs::remove_file(&part);
            return Err(e);
        }
        // The notes are safe in the new file; the old one is only set aside. A failure here is harmless.
        let mut aside = self.data.join(BEFORE_NOTAYA_FILE);
        let mut n = 1;
        while aside.exists() {
            n += 1;
            aside = self.data.join(format!("vault-before-notaya-{n}.clf"));
        }
        let journal = with_suffix(&self.old_vault, "-journal");
        if fs::rename(&self.old_vault, &aside).is_ok() && journal.exists() {
            let _ = fs::rename(&journal, with_suffix(&aside, "-journal"));
        }
        Ok(())
    }

    /// Creates the folders the app keeps its data in, and checks that they can be written to.
    /// `create_dir_all` succeeds for a folder that already exists even when it is read-only
    /// (a write-protected USB stick), so a small file is created and removed to be sure.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.backups)?;
        std::fs::create_dir_all(&self.webview)?;
        let probe = self.data.join(format!(".write-test-{}", std::process::id()));
        std::fs::write(&probe, b"")?;
        let _ = std::fs::remove_file(&probe);
        Ok(())
    }
}

/// The `data` folder as `Paths::resolve` would pick it, without any of its side effects. The error log's
/// panic hook uses it when a panic comes before the paths were resolved.
#[cfg_attr(test, allow(dead_code))] // tests never fall back to the real folder
pub fn early_data_dir() -> PathBuf {
    match env_var("NOTAYA_HOME", "CIPHERLEAF_HOME") {
        Some(root) => PathBuf::from(root).join("data"),
        None => default_root().0.join("data"),
    }
}

/// `path` with `suffix` added to the file name (`vault.nota` -> `vault.nota-journal`).
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

/// The folder that holds `data`, and whether it is the installed location.
fn default_root() -> (PathBuf, bool) {
    if cfg!(debug_assertions) {
        // During development, keep test data inside the project (git-ignored).
        return (PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(".devdata"), false);
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    choose_root(&exe_dir, local.as_deref())
}

/// Portable if the executable's folder has the marker file or already has a `data` folder
/// (every earlier portable copy does, so they keep working unchanged). Otherwise the copy was
/// installed, and its notes go under `%LOCALAPPDATA%`. Without that variable there is nowhere
/// better than next to the executable.
fn choose_root(exe_dir: &Path, local_app_data: Option<&Path>) -> (PathBuf, bool) {
    let portable = exe_dir.join(PORTABLE_MARKER).is_file() || exe_dir.join("data").is_dir();
    match local_app_data.filter(|p| p.is_absolute()) {
        Some(local) if !portable => (local.join(INSTALLED_FOLDER), true),
        _ => (exe_dir.to_path_buf(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cipherleaf-portable-{}", crate::model::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ensure_dirs_makes_the_folders_and_leaves_no_probe_file() {
        let root = temp_root();
        let paths = Paths::at(&root);
        paths.ensure_dirs().unwrap();
        paths.ensure_dirs().unwrap(); // a second start, with the folders already there
        assert!(paths.backups.is_dir() && paths.webview.is_dir());
        let leftovers: Vec<_> = std::fs::read_dir(&paths.data)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(".write-test"))
            .collect();
        assert!(leftovers.is_empty(), "the probe file must be removed");
    }

    #[test]
    fn ensure_dirs_fails_when_a_file_cannot_be_created_in_an_existing_data_folder() {
        // `create_dir_all` succeeds for folders that exist, so only the write probe can notice a
        // folder that can't be written to. A folder where the probe file should go stands in
        // for that (creating a file with that name fails).
        let root = temp_root();
        let paths = Paths::at(&root);
        paths.ensure_dirs().unwrap();
        std::fs::create_dir(paths.data.join(format!(".write-test-{}", std::process::id()))).unwrap();
        assert!(paths.ensure_dirs().is_err(), "the write probe must be what fails here");
    }

    #[test]
    fn ensure_dirs_fails_when_the_data_folder_cannot_be_made() {
        let root = temp_root();
        std::fs::write(root.join("data"), b"a file where the folder should be").unwrap();
        let paths = Paths::at(&root);
        assert!(paths.ensure_dirs().is_err());
    }

    #[test]
    fn a_folder_with_the_marker_or_a_data_folder_is_portable() {
        let local = temp_root();
        let marked = temp_root();
        std::fs::write(marked.join(PORTABLE_MARKER), b"").unwrap();
        assert_eq!(choose_root(&marked, Some(&local)), (marked.clone(), false));

        let with_data = temp_root();
        std::fs::create_dir_all(with_data.join("data")).unwrap();
        assert_eq!(choose_root(&with_data, Some(&local)), (with_data.clone(), false));
    }

    #[test]
    fn a_bare_folder_is_installed_and_uses_local_app_data() {
        let local = temp_root();
        let bare = temp_root();
        let (root, installed) = choose_root(&bare, Some(&local));
        assert!(installed);
        assert_eq!(root, local.join(INSTALLED_FOLDER));
        // The notes folder must not be one the setup or the uninstaller works in: the program goes
        // to `%LOCALAPPDATA%\Notaya`, and the "delete app data" box removes the identifier's folder.
        assert_ne!(root.file_name().unwrap(), "Notaya");
        assert_ne!(root.file_name().unwrap(), "app.notaya.notes");
        assert_ne!(root.file_name().unwrap(), "app.cipherleaf.vault");
        assert_eq!(Paths::at(&root).vault, local.join("Notaya Notes").join("data").join("vault.nota"));
    }

    #[test]
    fn without_a_usable_local_app_data_the_copy_stays_next_to_the_exe() {
        let bare = temp_root();
        assert_eq!(choose_root(&bare, None), (bare.clone(), false));
        assert_eq!(choose_root(&bare, Some(Path::new("relative"))), (bare.clone(), false));
    }

    #[test]
    fn a_marker_that_is_a_folder_does_not_count() {
        let local = temp_root();
        let bare = temp_root();
        std::fs::create_dir_all(bare.join(PORTABLE_MARKER)).unwrap();
        assert!(choose_root(&bare, Some(&local)).1);
    }

    fn temp_paths() -> Paths {
        let root = temp_root();
        let paths = Paths::at(&root);
        std::fs::create_dir_all(&paths.data).unwrap();
        paths
    }

    #[test]
    fn the_old_vault_file_becomes_vault_nota_and_is_kept_aside() {
        let mut paths = temp_paths();
        {
            let mut s = Session::create_legacy_open(&paths.old_vault).unwrap();
            s.put(crate::model::Item::Settings(crate::model::Settings::default())).unwrap();
        }
        let original = std::fs::read(&paths.old_vault).unwrap();
        paths.adopt_old_vault();
        assert_eq!(paths.vault, paths.data.join("vault.nota"));
        assert!(paths.vault.is_file() && !paths.old_vault.exists());
        assert_eq!(std::fs::read(paths.data.join("vault-before-notaya.clf")).unwrap(), original, "the old file is kept untouched");
        assert!(!with_suffix(&paths.vault, ".part").exists());
        // The copy is still in the old format until the first unlock upgrades it.
        let mut s = Session::unlock_open(&paths.vault).unwrap();
        assert!(!s.is_current_format());
        assert!(s.upgrade_format("", None).unwrap());
        drop(s);
        assert!(Session::unlock_open(&paths.vault).unwrap().is_current_format());
        // A second start does nothing more.
        paths.adopt_old_vault();
        assert_eq!(std::fs::read_dir(&paths.data).unwrap().count(), 2, "vault.nota and the kept file only");
    }

    #[test]
    fn an_existing_vault_nota_is_never_replaced() {
        let mut paths = temp_paths();
        drop(Session::create_open(&paths.vault).unwrap());
        drop(Session::create_legacy_open(&paths.old_vault).unwrap());
        let (new_bytes, old_bytes) = (std::fs::read(&paths.vault).unwrap(), std::fs::read(&paths.old_vault).unwrap());
        paths.adopt_old_vault();
        assert_eq!(paths.vault, paths.data.join("vault.nota"));
        assert_eq!(std::fs::read(&paths.vault).unwrap(), new_bytes);
        assert_eq!(std::fs::read(&paths.old_vault).unwrap(), old_bytes, "an old file next to a new one is left alone");
    }

    #[test]
    fn a_second_old_file_is_kept_under_a_free_name() {
        let mut paths = temp_paths();
        std::fs::write(paths.data.join("vault-before-notaya.clf"), b"EARLIER").unwrap();
        drop(Session::create_legacy_open(&paths.old_vault).unwrap());
        paths.adopt_old_vault();
        assert_eq!(std::fs::read(paths.data.join("vault-before-notaya.clf")).unwrap(), b"EARLIER");
        assert!(paths.data.join("vault-before-notaya-2.clf").is_file());
        assert!(paths.vault.is_file());
    }

    #[test]
    fn when_the_copy_fails_the_old_file_stays_the_vault() {
        let mut paths = temp_paths();
        std::fs::write(&paths.old_vault, b"this is not a vault").unwrap();
        paths.adopt_old_vault();
        assert_eq!(paths.vault, paths.old_vault, "no empty vault is started beside the old notes");
        assert_eq!(std::fs::read(&paths.old_vault).unwrap(), b"this is not a vault");
        assert!(!paths.data.join("vault.nota").exists() && !paths.data.join("vault.nota.part").exists());
    }

    #[test]
    fn a_leftover_part_file_is_replaced() {
        let mut paths = temp_paths();
        std::fs::write(paths.data.join("vault.nota.part"), b"half a copy").unwrap();
        drop(Session::create_legacy_open(&paths.old_vault).unwrap());
        paths.adopt_old_vault();
        assert!(Session::unlock_open(&paths.vault).is_ok());
        assert!(!paths.data.join("vault.nota.part").exists());
    }

    #[test]
    fn nothing_happens_without_an_old_file() {
        let mut paths = temp_paths();
        paths.adopt_old_vault();
        assert_eq!(paths.vault, paths.data.join("vault.nota"));
        assert_eq!(std::fs::read_dir(&paths.data).unwrap().count(), 0);
    }
}
