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
}

impl Paths {
    pub fn resolve() -> Paths {
        let root = std::env::var_os("CIPHERLEAF_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(default_root);
        Paths::at(&root)
    }

    pub fn at(root: &Path) -> Paths {
        let data = root.join("data");
        Paths {
            vault: data.join("vault.clf"),
            backups: data.join("backups"),
            webview: data.join("webview"),
            data,
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.backups)?;
        std::fs::create_dir_all(&self.webview)
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
