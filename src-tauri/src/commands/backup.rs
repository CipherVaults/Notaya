//! Restoring the vault from a backup (the safe steps are in `crate::restore`).

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use super::vault::{after_unlock, read_keyfile};
use crate::error::{Error, Result};
use crate::actions::Actor;
use crate::import::{self, ImportSummary};
use crate::restore::{self, BackupInfo};
use crate::state::{self, AppState};
use crate::vault::Session;

/// Start of the error returned when a restore would turn security off and was not confirmed.
/// The UI recognises it and asks the person to confirm.
pub const SECURITY_OFF_CONFIRM: &str = "SECURITY_OFF: This backup was made while security was off. Restoring it will turn security off for your notes, so they will open without a password. Tick the box to confirm.";

/// Lets only one restore run at a time (they share one temporary file).
static RESTORE_GATE: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    /// Name of the kept old vault in the data folder, or `None` if there was none.
    undo_file: Option<String>,
}

/// The automatic backups (`data/backups/vault-*.nota`), newest first. Works
/// while locked: it reads only file names, times and sizes.
#[tauri::command(async)]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfo>> {
    list_backups_person(&state)
}

fn list_backups_person(state: &AppState) -> Result<Vec<BackupInfo>> {
    list_backups_core(&state.paths, &Actor::Person)
}

pub(crate) fn list_backups_core(paths: &crate::portable::Paths, _actor: &Actor) -> Result<Vec<BackupInfo>> {
    restore::list_backups(paths)
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;

    #[test]
    fn list_backups_person_and_core_match_while_locked() {
        let root = std::path::PathBuf::from(std::env::var_os("NOTAYA_HOME").unwrap_or_else(|| "D:\\Notaya Tests\\v1-6-C1".into()));
        let state = AppState::new(crate::portable::Paths::at(&root.join(format!("backup-action-{}", crate::model::new_id()))));
        let person = list_backups_person(&state).unwrap();
        let core = list_backups_core(&state.paths, &Actor::Person).unwrap();
        assert_eq!(serde_json::to_vec(&person).unwrap(), serde_json::to_vec(&core).unwrap());
        assert!(person.is_empty());
    }
}

#[tauri::command(async)]
pub fn pick_backup_file(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    pick_backup_file_person(app, state)
}

fn pick_backup_file_person(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    let mut dialog = app.dialog().file().set_title("Choose a Notaya backup").add_filter("Notaya vault", &[crate::filenames::EXT, crate::filenames::OLD_EXT]);
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

/// "Import notes from a portable copy" on the first screen of an installed Notaya: the person
/// chooses the portable Notaya folder in a folder dialog (the UI never passes a path) and its
/// notes file and backups are copied here. Copy only: the portable copy is not changed. Only
/// while there are no notes yet. `None` if the dialog was cancelled.
#[tauri::command(async)]
pub fn import_portable_notes(app: AppHandle, state: State<'_, AppState>) -> Result<Option<ImportSummary>> {
    import_portable_notes_person(app, state)
}

fn import_portable_notes_person(app: AppHandle, state: State<'_, AppState>) -> Result<Option<ImportSummary>> {
    if !state.paths.installed {
        return Err(Error::invalid("Importing is offered only by the installed Notaya."));
    }
    let Some(dir) = app
        .dialog()
        .file()
        .set_title("Choose your portable Notaya folder")
        .blocking_pick_folder()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    // Not while a restore moves the same files, and not while a vault is being opened or created.
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    let _open_gate = state.open_gate();
    if state.is_unlocked() {
        return Err(Error::invalid("This Notaya already has notes open."));
    }
    import::import_portable(&state.paths, &dir).map(Some)
}

/// "Start over" on the lock screen: sets the vault aside as
/// `set-aside-<ms>.nota` in the data folder, so the first-run screen shows again.
/// Only while locked. Nothing is deleted; the old vault can be brought back with
/// "Restore from a backup" and its own password. Other files in the data folder are
/// not touched. Returns the set-aside file's name.
#[tauri::command(async)]
pub fn start_over(state: State<'_, AppState>) -> Result<String> {
    start_over_core(&state)
}

pub(crate) fn start_over_core(state: &AppState) -> Result<String> {
    if state.is_unlocked() {
        return Err(Error::invalid("Lock the vault first."));
    }
    // Not while a restore is moving the same files.
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    // And not while the vault is being opened or created at start-up.
    let _open_gate = state.open_gate();
    if state.is_unlocked() {
        return Err(Error::invalid("Lock the vault first."));
    }
    restore::set_aside(&state.paths, crate::model::now_ms().max(0) as u64)
}

/// Replaces the vault with a backup. Checks the backup first (header, every
/// record and attachment, with the given password and keyfile), keeps the old
/// vault as `restore-undo-<ms>.nota`, then unlocks the restored vault.
///
/// Runs on a worker thread because a lock waits for the windows to save.
#[tauri::command(async)]
pub fn restore_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    password: String,
    keyfile: Option<String>,
    allow_security_off: Option<bool>,
) -> Result<RestoreResult> {
    restore_vault_person(app, state, source, password, keyfile, allow_security_off)
}

fn restore_vault_person(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    password: String,
    keyfile: Option<String>,
    allow_security_off: Option<bool>,
) -> Result<RestoreResult> {
    let password = Zeroizing::new(password);
    state.check_rate_limit()?;
    // One restore at a time (they share one temporary file and the picked path).
    let _gate = RESTORE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    let kf = restore_vault_stage_core(&state, &source, &password, keyfile, allow_security_off)?;

    // The old vault file can only be replaced once it is closed: save the
    // windows' pending edits, make the normal backup, wipe the session.
    // No other window may open or create a vault while the file is swapped.
    let _open_gate = state.open_gate();
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

/// Pure Rust preflight/staging shared by the adapter and locality regressions.
pub(crate) fn restore_vault_stage_core(
    state: &AppState, source: &str, password: &str, keyfile: Option<String>, allow_security_off: Option<bool>,
) -> Result<Option<Zeroizing<[u8; 32]>>> {
    let kf = match read_keyfile(keyfile) {
        Ok(k) => k,
        Err(e) => {
            state.record_failure();
            return Err(e);
        }
    };
    let source = restore::resolve_source(&state.paths, source, state.picked_backup().as_deref())?;

    // Check the exact bytes that will be installed. Any failure removes the
    // copy and leaves the vault alone.
    if let Err(e) = restore::verify_and_stage(&state.paths, &source, password, kf.as_deref()) {
        if matches!(e, Error::BadCredentials) {
            state.record_failure();
        }
        return Err(e);
    }

    // A backup made while security was off would turn security off for a vault that has a
    // master password. That needs the person's explicit confirmation.
    let live_protected =
        state.paths.vault.exists() && Session::read_header(&state.paths.vault).map(|h| !h.open).unwrap_or(true);
    if live_protected && restore::staged_is_open(&state.paths).unwrap_or(false) && allow_security_off != Some(true) {
        restore::remove_temp(&state.paths);
        return Err(Error::invalid(SECURITY_OFF_CONFIRM));
    }

    Ok(kf)
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
    let undo = restore::install(&state.paths, crate::model::now_ms().max(0) as u64)?;
    // Keep the selected file available for retries until installation succeeds.
    state.clear_picked_backup();

    match Session::unlock_any(&state.paths.vault, password, kf) {
        Ok(mut session) => {
            // A backup from V1.3.0 or earlier is re-encrypted in the Notaya format (same password).
            session.upgrade_format_quietly(password, kf);
            // A restored vault with a master password is not stored if the window was closed to the
            // tray meanwhile (the same rule as an unlock); one with security off is stored as ever.
            if session.is_open() {
                state.set_session(session);
            } else {
                state.set_session_unless_hidden(session)?;
            }
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
        let json = serde_json::to_string(&RestoreResult { undo_file: Some("restore-undo-1.clf".into()) }).unwrap();
        assert_eq!(json, r#"{"undoFile":"restore-undo-1.clf"}"#);
        assert_eq!(serde_json::to_string(&RestoreResult { undo_file: None }).unwrap(), r#"{"undoFile":null}"#);
    }

    #[test]
    fn the_failure_payload_has_the_prefix_and_the_message() {
        let e = Error::Other("The backup was restored but could not be opened afterwards.".into());
        let payload = restore_failed_payload(&e);
        assert!(payload.starts_with("restore-failed: "));
        assert_eq!(payload, "restore-failed: The backup was restored but could not be opened afterwards.");
        assert_eq!(restore_failed_payload(&Error::BadCredentials), "restore-failed: Wrong master password or keyfile for this Notaya vault.");
    }
}
