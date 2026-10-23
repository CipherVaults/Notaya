use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use crate::crypto::{self, KdfParams};
use crate::error::{Error, Result};
use crate::generator;
use crate::model::{Item, Settings, Sticky};
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
    data_dir: String,
    version: String,
}

#[tauri::command]
pub fn vault_status(app: AppHandle, state: State<'_, AppState>) -> Result<VaultStatus> {
    let exists = state.paths.vault.exists();
    let unlocked = state.is_unlocked();
    let keyfile_required = if exists && !unlocked {
        Session::read_header(&state.paths.vault).map(|h| h.keyfile_required).unwrap_or(false)
    } else {
        false
    };
    Ok(VaultStatus {
        exists,
        unlocked,
        keyfile_required,
        data_dir: state.paths.data.display().to_string(),
        version: app.package_info().version.to_string(),
    })
}

fn read_keyfile(path: Option<String>) -> Result<Option<Zeroizing<[u8; 32]>>> {
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
fn after_unlock(app: &AppHandle, state: &AppState) {
    state.record_success();
    let Ok((settings, stickies)) = state.with(|s| {
        let stickies: Vec<Sticky> = s
            .items()
            .filter_map(|i| match i {
                Item::Sticky(st) if st.open => Some(st.clone()),
                _ => None,
            })
            .collect();
        Ok((s.settings(), stickies))
    }) else {
        return;
    };
    ui::set_capture_protection(app, settings.screen_capture_protection);
    for sticky in &stickies {
        let _ = ui::open_sticky(app, &state.paths, sticky, settings.screen_capture_protection);
    }
}

#[tauri::command(async)]
pub fn create_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
    keyfile: Option<String>,
) -> Result<()> {
    let password = Zeroizing::new(password);
    if state.paths.vault.exists() {
        return Err(Error::VaultExists);
    }
    check_new_password(&password)?;
    let kf = read_keyfile(keyfile)?;
    state.paths.ensure_dirs()?;
    let session = Session::create(&state.paths.vault, &password, kf.as_deref(), KdfParams::DEFAULT)?;
    state.set_session(session);
    after_unlock(&app, &state);
    Ok(())
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

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings> {
    state.with(|s| Ok(s.settings()))
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<Settings> {
    let settings = settings.sanitized();
    state.with_mut(|s| s.put(Item::Settings(settings.clone())))?;
    ui::set_capture_protection(&app, settings.screen_capture_protection);
    Ok(settings)
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
#[tauri::command(async)]
pub fn quit_app(app: AppHandle, state: State<'_, AppState>) {
    state::lock(&app, "quit");
    crate::keepassxc::shutdown(&state.keepassxc);
    app.exit(0);
}

#[tauri::command]
pub fn open_data_folder(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(state.paths.data.display().to_string(), None::<&str>)
        .map_err(|e| Error::Other(e.to_string()))
}
