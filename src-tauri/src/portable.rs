//! Where the notes live. Portable: everything is in a `data` folder next to the executable,
//! so the app can run from a USB stick and leaves nothing behind on the PC. Installed (by the
//! Windows setup): `%LOCALAPPDATA%\Notaya Notes\data`, because the program folder is not a
//! place to keep notes. The uninstaller never touches that folder.

use std::path::{Path, PathBuf};

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
    pub vault: PathBuf,
    pub backups: PathBuf,
    /// WebView2 profile folder (cache, local storage) kept with the app, not in AppData.
    pub webview: PathBuf,
    /// The notes are in the per-user install location, not next to the executable.
    pub installed: bool,
}

impl Paths {
    pub fn resolve() -> Paths {
        // Tests and tools point the app at a throwaway folder with this.
        if let Some(root) = std::env::var_os("CIPHERLEAF_HOME") {
            return Paths::at(&PathBuf::from(root));
        }
        let (root, installed) = default_root();
        Paths { installed, ..Paths::at(&root) }
    }

    pub fn at(root: &Path) -> Paths {
        let data = root.join("data");
        Paths {
            vault: data.join("vault.clf"),
            backups: data.join("backups"),
            webview: data.join("webview"),
            data,
            installed: false,
        }
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
        assert_ne!(root.file_name().unwrap(), "app.cipherleaf.vault");
        assert_eq!(Paths::at(&root).vault, local.join("Notaya Notes").join("data").join("vault.clf"));
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
}
