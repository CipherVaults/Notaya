//! Portable mode: everything lives in a `data` folder next to the executable,
//! so the app can run from a USB stick and leaves nothing behind on the PC.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Paths {
    pub data: PathBuf,
    pub vault: PathBuf,
    pub backups: PathBuf,
    /// WebView2 profile folder (cache, local storage) kept with the app, not in AppData.
    pub webview: PathBuf,
    /// KeePassXC's settings, kept with the vault instead of in AppData (amendment A9).
    pub keepassxc_config: PathBuf,
    /// Folder of the bundled KeePassXC program (amendment A9).
    pub keepassxc_dir: PathBuf,
}

impl Paths {
    pub fn resolve() -> Paths {
        let root = std::env::var_os("CIPHERLEAF_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(default_root);
        let keepassxc_dir = std::env::var_os("CIPHERLEAF_KEEPASSXC")
            .map(PathBuf::from)
            .unwrap_or_else(|| default_keepassxc_dir(&root));
        Paths::at(&root, keepassxc_dir)
    }

    pub fn at(root: &Path, keepassxc_dir: PathBuf) -> Paths {
        let data = root.join("data");
        Paths {
            vault: data.join("vault.clf"),
            backups: data.join("backups"),
            webview: data.join("webview"),
            keepassxc_config: data.join("keepassxc"),
            keepassxc_dir,
            data,
        }
    }

    /// Creates the folders the app keeps its data in, and checks that they can be written to.
    /// `create_dir_all` succeeds for a folder that already exists even when it is read-only
    /// (a write-protected USB stick), so a small file is created and removed to be sure.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.backups)?;
        std::fs::create_dir_all(&self.webview)?;
        std::fs::create_dir_all(&self.keepassxc_config)?;
        let probe = self.data.join(format!(".write-test-{}", std::process::id()));
        std::fs::write(&probe, b"")?;
        let _ = std::fs::remove_file(&probe);
        Ok(())
    }
}

fn default_root() -> PathBuf {
    if cfg!(debug_assertions) {
        // During development, keep test data inside the project (git-ignored).
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(".devdata")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn default_keepassxc_dir(root: &Path) -> PathBuf {
    if cfg!(debug_assertions) {
        // During development, use the verified copy in vendor/ (see vendor/keepassxc-manifest.md).
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("vendor")
            .join("keepassxc-2.8.0")
            .join("KeePassXC-2.8.0-x64")
    } else {
        root.join("KeePassXC")
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
        let paths = Paths::at(&root, root.join("kp"));
        paths.ensure_dirs().unwrap();
        paths.ensure_dirs().unwrap(); // a second start, with the folders already there
        assert!(paths.backups.is_dir() && paths.webview.is_dir() && paths.keepassxc_config.is_dir());
        let leftovers: Vec<_> = std::fs::read_dir(&paths.data)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(".write-test"))
            .collect();
        assert!(leftovers.is_empty(), "the probe file must be removed");
    }

    #[test]
    fn ensure_dirs_fails_when_the_data_folder_cannot_be_made() {
        let root = temp_root();
        std::fs::write(root.join("data"), b"a file where the folder should be").unwrap();
        let paths = Paths::at(&root, root.join("kp"));
        assert!(paths.ensure_dirs().is_err());
    }
}
