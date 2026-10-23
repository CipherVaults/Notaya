//! Restoring the vault from a backup (the safe steps are in `crate::restore`).

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use super::vault::{after_unlock, read_keyfile};
use crate::error::{Error, Result};
use crate::restore::{self, BackupInfo};
use crate::state::{self, AppState};
use crate::{keepassxc, pwbackup};
use crate::vault::Session;

/// Lets only one restore run at a time (they share one temporary file).
static RESTORE_GATE: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    /// Name of the kept old vault in the data folder, or `None` if there was none.
    undo_file: Option<String>,
}

/// The automatic backups (`data/backups/vault-*.clf`), newest first. Works
/// while locked: it reads only file names, times and sizes.
#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>> {
    restore::list_backups(&state.paths)
}

#[tauri::command(async)]
pub fn pick_backup_file(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    let mut dialog = app.dialog().file().set_title("Choose a Cipherleaf backup").add_filter("Cipherleaf vault", &["clf"]);
    if state.paths.backups.is_dir() {
        dialog = dialog.set_directory(&state.paths.backups);
    }
    let Some(path) = dialog.blocking_pick_file().and_then(|f| f.into_path().ok()) else {
        return Ok(None);
    };
    // `restore_vault` accepts an absolute path only if it is this one.
    state.set_picked_backup(std::fs::canonicalize(&path)?);
    Ok(Some(path.display().to_string()))
}

/// The automatic copies of the KeePassXC passwords file
/// (`data/backups/passwords-*.kdbx`), newest first. Names, times and sizes only.
#[tauri::command]
pub fn list_password_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>> {
    pwbackup::list(&state.paths)
}

#[tauri::command(async)]
pub fn pick_passwords_backup_file(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    let mut dialog = app
        .dialog()
        .file()
        .set_title("Choose a passwords backup")
        .add_filter("KeePassXC database", &["kdbx"]);
    if state.paths.backups.is_dir() {
        dialog = dialog.set_directory(&state.paths.backups);
    }
    let Some(path) = dialog.blocking_pick_file().and_then(|f| f.into_path().ok()) else {
        return Ok(None);
    };
    // `restore_passwords` accepts an absolute path only if it is this one.
    state.set_picked_passwords_backup(std::fs::canonicalize(&path)?);
    Ok(Some(path.display().to_string()))
}

const KEEPASSXC_OPEN: &str = "KeePassXC is still open. Finish there (it may be asking to save), close it, and try again.";

/// Replaces the KeePassXC passwords file with a backup. Only while Cipherleaf is
/// unlocked (unlocking is how the owner proves who they are). KeePassXC is asked
/// to close first and is never force-ended. The backup's own password is not
/// asked, and Cipherleaf can only check that it is a KeePassXC file; the old
/// file is kept as `passwords-undo-<ms>.kdbx`.
#[tauri::command(async)]
pub fn restore_passwords(state: State<'_, AppState>, source: String) -> Result<RestoreResult> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    // One restore at a time (they share one temporary file and the picked path).
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let source = pwbackup::resolve_source(&state.paths, &source, state.picked_passwords_backup().as_deref())?;

    // The file can only be replaced once KeePassXC has let go of it.
    if !keepassxc::close_gracefully(&state.keepassxc, &state.paths, state::KEEPASSXC_CLOSE_TIMEOUT) {
        // Give it its own window back, so any "save changes?" question can be seen.
        let _ = keepassxc::detach();
        return Err(Error::invalid(KEEPASSXC_OPEN));
    }
    // Cipherleaf may have locked while KeePassXC was closing: then nothing is changed.
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let undo_file = pwbackup::stage_and_install(&state.paths, &source, crate::model::now_ms().max(0) as u64)?;
    // The chosen file has been used. Until this point the pick is kept, so the owner
    // can simply try again.
    state.clear_picked_passwords_backup();
    Ok(RestoreResult { undo_file })
}

/// "Start over" on the lock screen: sets the vault aside as
/// `set-aside-<ms>.clf` in the data folder, so the first-run screen shows again.
/// Only while locked. Nothing is deleted; the old vault can be brought back with
/// "Restore from a backup" and its own password. The KeePassXC passwords file is
/// not touched. Returns the set-aside file's name.
#[tauri::command(async)]
pub fn start_over(state: State<'_, AppState>) -> Result<String> {
    if state.is_unlocked() {
        return Err(Error::invalid("Lock the vault first."));
    }
    // Not while a restore is moving the same files.
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    if state.is_unlocked() {
        return Err(Error::invalid("Lock the vault first."));
    }
    restore::set_aside(&state.paths, crate::model::now_ms().max(0) as u64)
}

/// Replaces the vault with a backup. Checks the backup first (header, every
/// record and attachment, with the given password and keyfile), keeps the old
/// vault as `restore-undo-<ms>.clf`, then unlocks the restored vault.
///
/// Runs on a worker thread because a lock waits for the windows to save.
#[tauri::command(async)]
pub fn restore_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    password: String,
    keyfile: Option<String>,
) -> Result<RestoreResult> {
    let password = Zeroizing::new(password);
    state.check_rate_limit()?;
    // One restore at a time (they share one temporary file and the picked path).
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    let kf = match read_keyfile(keyfile) {
        Ok(k) => k,
        Err(e) => {
            state.record_failure();
            return Err(e);
        }
    };
    let source = restore::resolve_source(&state.paths, &source, state.picked_backup().as_deref())?;

    // Check the exact bytes that will be installed. Any failure removes the
    // copy and leaves the vault alone.
    if let Err(e) = restore::verify_and_stage(&state.paths, &source, &password, kf.as_deref()) {
        if matches!(e, Error::BadCredentials) {
            state.record_failure();
        }
        return Err(e);
    }

    // The old vault file can only be replaced once it is closed: save the
    // windows' pending edits, make the normal backup, wipe the session.
    let locked_here = state.is_unlocked();
    if locked_here {
        state::lock(&app, "restore");
    }

    let result = install_and_open(&app, &state, &password, kf.as_deref());
    if locked_here && let Err(e) = &result {
        // `lock` held back `vault-locked` for a restore; tell the UI now that it
        // failed. The reload would lose the error, so it travels in the payload.
        let _ = app.emit("vault-locked", restore_failed_payload(e));
    }
    result.map(|undo_file| RestoreResult { undo_file })
}

/// Payload of the `vault-locked` event after a failed restore: the prefix, then
/// the same plain text the command returns.
fn restore_failed_payload(e: &Error) -> String {
    format!("restore-failed: {e}")
}

/// Installs the checked copy and unlocks it. Returns the kept old vault's name.
fn install_and_open(
    app: &AppHandle,
    state: &AppState,
    password: &str,
    kf: Option<&[u8; 32]>,
) -> Result<Option<String>> {
    let undo = match restore::install(&state.paths, crate::model::now_ms().max(0) as u64) {
        Ok(u) => {
            // The chosen file has been used. Until this point (for example after a wrong
            // password) the pick is kept, so the owner can simply try again.
            state.clear_picked_backup();
            u
        }
        Err(e) => {
            restore::remove_temp(&state.paths);
            return Err(e);
        }
    };

    match Session::unlock(&state.paths.vault, password, kf) {
        Ok(session) => {
            state.set_session(session);
            after_unlock(app, state);
            Ok(undo)
        }
        Err(e) => {
            let kept = match &undo {
                Some(name) => format!("Your previous vault is kept as {name} in the data folder."),
                None => "There was no previous vault.".to_string(),
            };
            Err(Error::Other(format!("The backup was restored but could not be opened afterwards ({e}). {kept}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_result_uses_camel_case_json() {
        let json = serde_json::to_string(&RestoreResult { undo_file: Some("passwords-undo-1.kdbx".into()) }).unwrap();
        assert_eq!(json, r#"{"undoFile":"passwords-undo-1.kdbx"}"#);
        assert_eq!(serde_json::to_string(&RestoreResult { undo_file: None }).unwrap(), r#"{"undoFile":null}"#);
    }

    #[test]
    fn the_failure_payload_has_the_prefix_and_the_message() {
        let e = Error::Other("The backup was restored but could not be opened afterwards.".into());
        let payload = restore_failed_payload(&e);
        assert!(payload.starts_with("restore-failed: "));
        assert_eq!(payload, "restore-failed: The backup was restored but could not be opened afterwards.");
        assert_eq!(restore_failed_payload(&Error::BadCredentials), "restore-failed: Wrong master password or keyfile for this Cipherleaf vault.");
    }
}
