//! The bundled KeePassXC (amendment A9). Cipherleaf starts it for passwords and
//! asks it to lock when Cipherleaf locks. Only file paths cross this boundary:
//! Cipherleaf never sees or passes KeePassXC's master password.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::portable::Paths;

pub const VERSION: &str = "2.8.0";

/// SHA-256 of the verified `KeePassXC.exe` from the official 2.8.0 x64 release.
/// See `vendor/keepassxc-manifest.md`. Cipherleaf won't start a different file.
const EXE_SHA256: &str = "72200BFE1A6C59162BEE904832F3B292EDE45325EFFCEF54060C6AFF836DD28D";

/// Written once, on first use; later changes made in KeePassXC are kept.
/// Offline by default: no update checks; browser integration, SSH agent and
/// Secret Service off. Locks and clipboard clearing match Cipherleaf's defaults.
const PRESET_CONFIG: &str = "[General]
SingleInstance=true
RememberLastDatabases=false
RememberLastKeyFiles=false
OpenPreviousDatabasesOnStartup=false
BackupBeforeSave=true

[GUI]
CheckForUpdates=false
CheckForUpdatesIncludeBetas=false
MinimizeOnStartup=false
ShowTrayIcon=false

[Security]
IconDownloadFallback=false
LockDatabaseIdle=true
LockDatabaseIdleSeconds=300
LockDatabaseScreenLock=true
LockDatabaseMinimize=false
ClearClipboard=true
ClearClipboardTimeout=20
AutotypeAsk=true

[Browser]
Enabled=false

[SSHAgent]
Enabled=false

[FdoSecrets]
Enabled=false
";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// KeePassXC is present next to Cipherleaf.
    installed: bool,
    /// Its executable matches the verified 2.8.0 release.
    verified: bool,
    version: &'static str,
    /// The password database Cipherleaf will open, if one exists.
    database: Option<String>,
    data_dir: String,
    /// KeePassXC was started by Cipherleaf and is still running.
    running: bool,
}

fn exe(paths: &Paths) -> PathBuf {
    paths.keepassxc_dir.join("KeePassXC.exe")
}

fn config_files(paths: &Paths) -> (PathBuf, PathBuf) {
    (
        paths.keepassxc_config.join("keepassxc.ini"),
        paths.keepassxc_config.join("keepassxc_local.ini"),
    )
}

fn verify(paths: &Paths) -> bool {
    std::fs::read(exe(paths))
        .map(|bytes| hex::encode_upper(crate::crypto::sha256(&bytes)) == EXE_SHA256)
        .unwrap_or(false)
}

/// `data/passwords.kdbx` if present; otherwise the first other `.kdbx` in the
/// data folder, ignoring KeePassXC's `.old.kdbx` backups.
pub fn find_database(paths: &Paths) -> Option<PathBuf> {
    let preferred = paths.data.join("passwords.kdbx");
    if preferred.is_file() {
        return Some(preferred);
    }
    let mut found: Vec<PathBuf> = std::fs::read_dir(&paths.data)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && is_database(p))
        .collect();
    found.sort();
    found.into_iter().next()
}

fn is_database(p: &Path) -> bool {
    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
    name.ends_with(".kdbx") && !name.ends_with(".old.kdbx")
}

fn is_running(child: &mut Option<Child>) -> bool {
    matches!(child.as_mut().map(|c| c.try_wait()), Some(Ok(None)))
}

pub fn status(paths: &Paths, child: &Mutex<Option<Child>>) -> Status {
    let mut guard = child.lock().unwrap_or_else(|e| e.into_inner());
    Status {
        installed: exe(paths).is_file(),
        verified: verify(paths),
        version: VERSION,
        database: find_database(paths).map(|p| p.display().to_string()),
        data_dir: paths.data.display().to_string(),
        running: is_running(&mut guard),
    }
}

fn base_command(paths: &Paths) -> Result<Command> {
    let exe = exe(paths);
    if !exe.is_file() {
        return Err(Error::invalid("KeePassXC isn't included with this copy of Cipherleaf."));
    }
    if !verify(paths) {
        return Err(Error::invalid(
            "The bundled KeePassXC doesn't match the verified 2.8.0 release, so Cipherleaf won't start it.",
        ));
    }
    std::fs::create_dir_all(&paths.keepassxc_config)?;
    let (config, local) = config_files(paths);
    if !config.exists() {
        std::fs::write(&config, PRESET_CONFIG)?;
    }
    let mut cmd = Command::new(exe);
    cmd.arg("--config").arg(&config).arg("--localconfig").arg(&local);
    cmd.current_dir(&paths.keepassxc_dir);
    Ok(cmd)
}

/// Opens the password database in KeePassXC. With no database yet, KeePassXC
/// starts with its file dialogs pointed at Cipherleaf's data folder, so a new
/// database is created there.
pub fn open(paths: &Paths, child: &Mutex<Option<Child>>) -> Result<()> {
    let mut cmd = base_command(paths)?;
    match find_database(paths) {
        Some(db) => {
            cmd.arg(db);
        }
        None => {
            cmd.env("KPXC_INITIAL_DIR", &paths.data);
        }
    }
    let spawned = cmd.spawn().map_err(|e| Error::Other(format!("Couldn't start KeePassXC: {e}")))?;
    let mut guard = child.lock().unwrap_or_else(|e| e.into_inner());
    // A second start just hands the file to the running copy and exits, so keep
    // tracking the first one while it runs.
    if !is_running(&mut guard) {
        *guard = Some(spawned);
    }
    Ok(())
}

/// Asks a KeePassXC started by Cipherleaf to lock all its databases.
/// Does nothing if it isn't running, so it never starts KeePassXC.
pub fn lock_if_running(paths: &Paths, child: &Mutex<Option<Child>>) {
    let mut guard = child.lock().unwrap_or_else(|e| e.into_inner());
    if !is_running(&mut guard) {
        return;
    }
    if let Ok(mut cmd) = base_command(paths) {
        cmd.arg("--lock");
        if let Ok(mut c) = cmd.spawn() {
            // The running copy receives the request; reap this helper process.
            std::thread::spawn(move || {
                let _ = c.wait();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> Paths {
        let root = std::env::temp_dir().join(format!("cipherleaf-kpxc-{}", crate::model::new_id()));
        let paths = Paths::at(&root, root.join("KeePassXC"));
        paths.ensure_dirs().unwrap();
        paths
    }

    #[test]
    fn prefers_passwords_kdbx_and_ignores_backups() {
        let paths = temp_paths();
        assert!(find_database(&paths).is_none());
        std::fs::write(paths.data.join("zeta.old.kdbx"), b"x").unwrap();
        assert!(find_database(&paths).is_none());
        std::fs::write(paths.data.join("alpha.kdbx"), b"x").unwrap();
        assert_eq!(find_database(&paths).unwrap().file_name().unwrap(), "alpha.kdbx");
        std::fs::write(paths.data.join("passwords.kdbx"), b"x").unwrap();
        assert_eq!(find_database(&paths).unwrap().file_name().unwrap(), "passwords.kdbx");
    }

    #[test]
    fn refuses_missing_or_unverified_keepassxc() {
        let paths = temp_paths();
        let child = Mutex::new(None);
        assert!(open(&paths, &child).is_err()); // not installed
        std::fs::create_dir_all(&paths.keepassxc_dir).unwrap();
        std::fs::write(paths.keepassxc_dir.join("KeePassXC.exe"), b"not the real program").unwrap();
        let err = open(&paths, &child).unwrap_err().to_string();
        assert!(err.contains("doesn't match"), "{err}");
        let s = status(&paths, &child);
        assert!(s.installed && !s.verified && !s.running);
    }

    #[test]
    fn preset_config_keeps_keepassxc_offline() {
        for line in ["CheckForUpdates=false", "[Browser]\nEnabled=false", "[SSHAgent]\nEnabled=false", "RememberLastKeyFiles=false"] {
            assert!(PRESET_CONFIG.contains(line), "missing {line}");
        }
    }
}
