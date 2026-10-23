use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams};
use crate::error::{Error, Result};
use crate::generator;
use crate::model::{resolve_passwords, Item, Settings, Sticky};
use crate::openvault;
use crate::state::{self, AppState};
use crate::ui;
use crate::vault::Session;

const MIN_PASSWORD_CHARS: usize = 10;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    exists: bool,
    unlocked: bool,
    keyfile_required: bool,
    /// The vault has a master password (security is on). False for a vault that opens by itself.
    security_on: bool,
    /// There is no vault file but earlier copies exist: the person chooses between
    /// restoring and starting new notes. Nothing is created behind their back.
    earlier_copies: bool,
    /// A vault with security off could not be opened (for example a damaged file).
    open_error: Option<String>,
    data_dir: String,
    version: String,
}

/// Opens the vault by itself when it has security off, and on a first run creates one,
/// so Cipherleaf starts straight in the notes (A20). A vault with a master password is
/// left alone for the lock screen. Returns why a vault with security off could not be opened.
fn auto_open(app: &AppHandle, state: &AppState) -> Option<String> {
    let _gate = state.open_gate();
    // A quit in progress must not bring the notes back to life while KeePassXC closes.
    if state.is_unlocked() || state.is_quitting() {
        return None;
    }
    if !state.paths.vault.exists() {
        // Earlier copies without a vault look like lost notes: ask first, never start empty.
        if openvault::has_earlier_copies(&state.paths) {
            return None;
        }
        return create_open(app, state).err().map(|e| e.to_string());
    }
    match Session::read_header(&state.paths.vault) {
        Ok(h) if h.open => match Session::unlock_open(&state.paths.vault) {
            Ok(session) => {
                state.set_session(session);
                after_unlock(app, state);
                None
            }
            Err(e) => Some(e.to_string()),
        },
        // A master password, or a header that can't be read: the lock screen deals with it.
        _ => None,
    }
}

/// Opens a vault that has security off again after a cancelled quit had closed it.
pub(crate) fn reopen_open_vault(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = auto_open(app, &state);
}

/// Creates a new vault with security off and opens it.
fn create_open(app: &AppHandle, state: &AppState) -> Result<()> {
    state.paths.ensure_dirs()?;
    let session = Session::create_open(&state.paths.vault)?;
    state.set_session(session);
    after_unlock(app, state);
    Ok(())
}

/// Runs on a worker thread: opening a vault also opens sticky notes (windows).
#[tauri::command(async)]
pub fn vault_status(app: AppHandle, state: State<'_, AppState>) -> Result<VaultStatus> {
    let open_error = auto_open(&app, &state);
    let exists = state.paths.vault.exists();
    let unlocked = state.is_unlocked();
    let (keyfile_required, security_on) = if unlocked {
        // Also while unlocked: turning security off needs the keyfile again.
        match state.with(|s| Ok((s.keyfile_required()?, !s.is_open()))) {
            Ok(v) => v,
            Err(_) => (false, true),
        }
    } else if exists {
        match Session::read_header(&state.paths.vault) {
            Ok(h) => (h.keyfile_required, !h.open),
            // Unreadable: treat it as protected, so it is never shown as an open vault.
            Err(_) => (false, true),
        }
    } else {
        (false, false)
    };
    Ok(VaultStatus {
        exists,
        unlocked,
        keyfile_required,
        security_on,
        earlier_copies: !exists && openvault::has_earlier_copies(&state.paths),
        open_error,
        data_dir: state.paths.data.display().to_string(),
        version: app.package_info().version.to_string(),
    })
}

pub(super) fn read_keyfile(path: Option<String>) -> Result<Option<Zeroizing<[u8; 32]>>> {
    path.filter(|p| !p.trim().is_empty())
        .map(|p| crypto::keyfile_digest(&PathBuf::from(p)))
        .transpose()
}

fn check_new_password(password: &str) -> Result<()> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(Error::invalid(format!(
            "Use at least {MIN_PASSWORD_CHARS} characters for your master password."
        )));
    }
    if generator::strength(password, &["cipherleaf"]).score < 3 {
        return Err(Error::invalid(
            "That master password is too easy to guess. Try a longer passphrase.",
        ));
    }
    Ok(())
}

/// Things to do right after unlocking: restore sticky notes and window effects.
pub(super) fn after_unlock(app: &AppHandle, state: &AppState) {
    state.record_success();
    let Ok((protect, stickies)) = state.with(|s| {
        let stickies: Vec<Sticky> = s
            .items()
            .filter_map(|i| match i {
                Item::Sticky(st) if st.open => Some(st.clone()),
                _ => None,
            })
            .collect();
        // Hidden from screenshots only while security is on and the setting is on (default off).
        Ok((s.protects_capture(), stickies))
    }) else {
        return;
    };
    ui::set_capture_protection(app, protect);
    for sticky in &stickies {
        let _ = ui::open_sticky(app, &state.paths, sticky, protect);
    }
}

/// Starts new notes: creates a vault with security off, for the "no vault but earlier
/// copies exist" screen. (On a plain first run `vault_status` does this by itself.)
#[tauri::command(async)]
pub fn start_notes(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    let _gate = state.open_gate();
    if state.paths.vault.exists() {
        return Err(Error::VaultExists);
    }
    create_open(&app, &state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableSecurityResult {
    /// How many earlier copies made while security was off were deleted.
    removed_copies: usize,
}

/// Turns security on: the notes get a master password (and an optional keyfile). Only
/// the key that protects them changes, so nothing is re-encrypted or lost. If asked, the
/// earlier copies that were made while security was off (readable by anyone) are deleted,
/// but only after a fresh backup under the new password has been saved.
#[tauri::command(async)]
pub fn enable_security(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
    keyfile: Option<String>,
    remove_unprotected_copies: bool,
) -> Result<EnableSecurityResult> {
    let password = Zeroizing::new(password);
    check_new_password(&password)?;
    let kf = read_keyfile(keyfile)?;
    state.with_mut(|s| s.make_protected(&password, kf.as_deref(), KdfParams::DEFAULT))?;
    // From here on the vault is protected: the stored screenshot choice (off unless chosen) applies.
    if let Ok(protect) = state.with(|s| Ok(s.protects_capture())) {
        ui::set_capture_protection(&app, protect);
    }
    let mut removed_copies = 0;
    if remove_unprotected_copies {
        match state.with(|s| state::rotate_backup(&state.paths, s)) {
            Ok(()) => removed_copies = openvault::delete_unprotected(&state.paths),
            // No fresh backup: keep the old copies rather than leave none.
            Err(e) => eprintln!("backup after turning security on failed: {e}"),
        }
    }
    Ok(EnableSecurityResult { removed_copies })
}

/// Turns security off. The current master password (and keyfile) are required, and a
/// wrong one counts as a failed attempt like on the lock screen. Nothing else changes.
#[tauri::command(async)]
pub fn disable_security(
    app: AppHandle,
    state: State<'_, AppState>,
    current_password: String,
    current_keyfile: Option<String>,
) -> Result<()> {
    let current_password = Zeroizing::new(current_password);
    state.check_rate_limit()?;
    let current_kf = match read_keyfile(current_keyfile) {
        Ok(k) => k,
        Err(e) => {
            state.record_failure();
            return Err(e);
        }
    };
    let result = state.with_mut(|s| s.make_open(&current_password, current_kf.as_deref()));
    match &result {
        Err(Error::BadCredentials) => state.record_failure(),
        Ok(()) => {
            state.record_success();
            ui::set_capture_protection(&app, false);
        }
        Err(_) => {}
    }
    result
}

/// Switches the Passwords (KeePassXC) feature on or off. Turning it off first asks a
/// running KeePassXC to close; if it stays open the switch stays on.
#[tauri::command(async)]
pub fn set_passwords_enabled(state: State<'_, AppState>, enabled: bool) -> Result<Settings> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    if !enabled {
        if !crate::keepassxc::close_gracefully(&state.keepassxc, &state.paths, state::KEEPASSXC_CLOSE_TIMEOUT) {
            // Give it its own window back, so any "save changes?" question can be seen.
            let _ = crate::keepassxc::detach();
            return Err(Error::invalid(state::KEEPASSXC_STILL_OPEN));
        }
        crate::embed::hide();
    }
    state.with_mut(|s| {
        let mut settings = s.settings();
        settings.passwords_enabled = Some(enabled);
        s.put(Item::Settings(settings))
    })?;
    get_settings(state)
}

#[tauri::command(async)]
pub fn unlock_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
    keyfile: Option<String>,
) -> Result<()> {
    let password = Zeroizing::new(password);
    state.check_rate_limit()?;
    if state.is_unlocked() {
        return Ok(());
    }
    let kf = match read_keyfile(keyfile) {
        Ok(k) => k,
        Err(e) => {
            state.record_failure();
            return Err(e);
        }
    };
    match Session::unlock(&state.paths.vault, &password, kf.as_deref()) {
        Ok(session) => {
            state.set_session(session);
            after_unlock(&app, &state);
            Ok(())
        }
        Err(e) => {
            if matches!(e, Error::BadCredentials) {
                state.record_failure();
            }
            Err(e)
        }
    }
}

/// Runs on a worker thread: the lock first waits for every window (sticky
/// notes included) to save, and their answers need the event-loop thread free.
#[tauri::command(async)]
pub fn lock_vault(app: AppHandle) {
    state::lock(&app, "manual");
}

#[tauri::command(async)]
pub fn change_master(
    state: State<'_, AppState>,
    current_password: String,
    current_keyfile: Option<String>,
    new_password: String,
    new_keyfile: Option<String>,
) -> Result<()> {
    let current_password = Zeroizing::new(current_password);
    let new_password = Zeroizing::new(new_password);
    state.check_rate_limit()?;
    check_new_password(&new_password)?;
    let current_kf = read_keyfile(current_keyfile)?;
    let new_kf = read_keyfile(new_keyfile)?;
    let result = state.with_mut(|s| {
        s.change_credentials(
            &current_password,
            current_kf.as_deref(),
            &new_password,
            new_kf.as_deref(),
            KdfParams::DEFAULT,
        )
    });
    if matches!(result, Err(Error::BadCredentials)) {
        state.record_failure();
    }
    result
}

#[tauri::command(async)]
pub fn pick_keyfile(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Choose your keyfile")
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.display().to_string())
}

#[tauri::command(async)]
pub fn create_keyfile(app: AppHandle) -> Result<Option<String>> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save your new keyfile (keep it somewhere other than the vault)")
        .set_file_name("cipherleaf.key")
        .add_filter("Keyfile", &["key"])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    crypto::generate_keyfile(&path)?;
    Ok(Some(path.display().to_string()))
}

#[tauri::command(async)]
pub fn backup_now(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save an encrypted backup")
        .set_file_name(format!("cipherleaf-backup-{}.clf", crate::model::now_ms() / 1000))
        .add_filter("Cipherleaf vault", &["clf"])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    state.with(|s| s.backup_to(&path))?;
    Ok(Some(path.display().to_string()))
}

/// The settings as the UI sees them: the Passwords switch is always a definite on/off.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings> {
    let mut settings = state.with(|s| Ok(s.settings()))?;
    settings.passwords_enabled = Some(resolve_passwords(settings.passwords_enabled, state::passwords_in_use(&state.paths)));
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<Settings> {
    let mut settings = settings.sanitized();
    let protect = state.with_mut(|s| {
        // The Passwords switch changes only through `set_passwords_enabled` (it has to close
        // KeePassXC first), so a late or stale save from a window can't flip it.
        settings.passwords_enabled = s.settings().passwords_enabled;
        s.put(Item::Settings(settings.clone()))?;
        Ok(s.protects_capture())
    })?;
    ui::set_capture_protection(&app, protect);
    get_settings(state)
}

/// Applies Mica/Acrylic to the calling window. Returns whether it took effect.
#[tauri::command]
pub fn set_backdrop(window: WebviewWindow, kind: String, dark: bool) -> bool {
    ui::apply_backdrop(&window, &kind, dark)
}

/// Called by each window once it has painted its first frame. From then on the
/// window is asked to save its pending edits before the vault locks.
#[tauri::command]
pub fn window_ready(window: WebviewWindow, state: State<'_, AppState>) {
    state.mark_ready(window.label());
    let _ = window.show();
    if window.label() == "main" {
        let _ = window.set_focus();
    }
}

/// Answer to the `vault-locking` event: this window has saved its pending
/// edits. `round` is the number that came with the event.
#[tauri::command]
pub fn flush_done(window: WebviewWindow, state: State<'_, AppState>, round: u64) {
    state.flush_done(window.label(), round);
}

/// Runs on a worker thread, like `lock_vault`, so the windows can answer.
/// Locks, asks KeePassXC to close and exits. If KeePassXC is still open (it may be
/// asking to save) it does NOT exit and never ends KeePassXC: the owner gets the
/// `app-notice` event instead, and the command still returns normally.
#[tauri::command(async)]
pub fn quit_app(app: AppHandle) {
    state::quit(&app);
}

/// A message for the owner that the app couldn't show live, returned once. The UI
/// calls it on startup, because a lock reloads the window and an `app-notice`
/// event sent just before could be missed.
#[tauri::command]
pub fn take_notice(state: State<'_, AppState>) -> Option<String> {
    state.take_notice()
}

#[tauri::command]
pub fn open_data_folder(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(state.paths.data.display().to_string(), None::<&str>)
        .map_err(|e| Error::Other(e.to_string()))
}
