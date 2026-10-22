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

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.backups)?;
        std::fs::create_dir_all(&self.webview)?;
        std::fs::create_dir_all(&self.keepassxc_config)
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
