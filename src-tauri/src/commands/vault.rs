use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use crate::backup;
use crate::actions::Actor;
use crate::crypto::{self, KdfParams};
use crate::error::{Error, Result};
use crate::generator;
use crate::model::{Item, LastPlace, Settings, Sticky};
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
    /// The vault's header could be read, so `keyfile_required` is known and not a guess.
    header_ok: bool,
    data_dir: String,
    version: String,
    /// An installed Notaya with no notes yet: the first screen offers starting new notes and
    /// importing the notes of a portable copy (also after a Start over, when earlier copies exist).
    offer_import: bool,
}

/// Opens the vault by itself when it has security off, and on a first run creates one,
/// so Notaya starts straight in the notes (A21). A vault with a master password is
/// left alone for the lock screen. Returns why a vault with security off could not be opened.
fn auto_open(app: &AppHandle, state: &AppState) -> Option<String> {
    let _gate = state.open_gate();
    // A quit in progress must not bring the notes back to life.
    if state.is_unlocked() || state.is_quitting() {
        return None;
    }
    if !state.paths.vault.exists() {
        // Earlier copies without a vault look like lost notes: ask first, never start empty.
        // An installed copy also asks on its first start, so the notes of a portable copy can
        // be brought in before an empty vault takes their place.
        if state.paths.installed || openvault::has_earlier_copies(&state.paths) {
            return None;
        }
        return create_open(app, state).err().map(|e| e.to_string());
    }
    match Session::read_header(&state.paths.vault) {
        Ok(h) if h.open => match Session::unlock_open(&state.paths.vault) {
            Ok(mut session) => {
                // A vault from V1.3.0 or earlier is re-encrypted in the Notaya format (no password needed).
                session.upgrade_format_quietly("", None);
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
    vault_status_person(app, state)
}

fn vault_status_person(app: AppHandle, state: State<'_, AppState>) -> Result<VaultStatus> {
    let open_error = auto_open(&app, &state);
    let exists = state.paths.vault.exists();
    let unlocked = state.is_unlocked();
    let (keyfile_required, security_on, header_ok) = if unlocked {
        // Also while unlocked: turning security off needs the keyfile again.
        match state.with(|s| Ok((s.keyfile_required()?, !s.is_open()))) {
            Ok((k, on)) => (k, on, true),
            Err(_) => (false, true, false),
        }
    } else if exists {
        match Session::read_header(&state.paths.vault) {
            Ok(h) => (h.keyfile_required, !h.open, true),
            // Unreadable: treat it as protected, so it is never shown as an open vault.
            Err(_) => (false, true, false),
        }
    } else {
        (false, false, true)
    };
    let earlier_copies = !exists && openvault::has_earlier_copies(&state.paths);
    Ok(VaultStatus {
        exists,
        unlocked,
        keyfile_required,
        security_on,
        earlier_copies,
        open_error,
        header_ok,
        data_dir: state.paths.data.display().to_string(),
        version: app.package_info().version.to_string(),
        offer_import: !exists && state.paths.installed,
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
    if generator::strength(password, &["cipherleaf", "notaya"]).score < 3 {
        return Err(Error::invalid(
            "That master password is too easy to guess. Try a longer passphrase.",
        ));
    }
    Ok(())
}

/// Things to do right after unlocking: restore sticky notes and window effects.
pub(super) fn after_unlock(app: &AppHandle, state: &AppState) {
    state.bind_events(app);
    state.record_success();
    let Ok((protect, stickies)) = state.with(|s| {
        let stickies: Vec<Sticky> = s
            .items()
            .filter_map(|i| match i {
                Item::Sticky(st) if st.open && st.trashed_at.is_none() => Some(st.clone()),
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
    start_notes_person(app, state)
}

fn start_notes_person(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
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
    /// Deleting was asked for, but the fresh backup failed, so the earlier copies were kept.
    backup_failed: bool,
}

/// Turns security on: the notes get a master password (and an optional keyfile). The vault is
/// re-keyed (a fresh data key and vault id, every record and attachment re-encrypted in one
/// all-or-nothing step), so a copy made while security was off can't open data written later.
/// Nothing is lost. If asked, the earlier copies that were made while security was off
/// (readable by anyone, safety backups made before a delete included) are deleted, but only
/// after a fresh backup under the new password has been saved. Restore-undo and set-aside
/// vaults are never deleted.
#[tauri::command(async)]
pub fn enable_security(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
    keyfile: Option<String>,
    remove_unprotected_copies: bool,
) -> Result<EnableSecurityResult> {
    enable_security_person(app, state, password, keyfile, remove_unprotected_copies)
}

fn enable_security_person(
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
    let mut backup_failed = false;
    if remove_unprotected_copies {
        match state.with(|s| backup::make_backup(&state.paths, s)) {
            Ok(_) => removed_copies = openvault::delete_unprotected(&state.paths),
            // No fresh backup: keep the old copies rather than leave none.
            Err(e) => {
                eprintln!("backup after turning security on failed: {e}");
                backup_failed = true;
            }
        }
    }
    Ok(EnableSecurityResult { removed_copies, backup_failed })
}

/// Turns security off. The current master password (and keyfile) are required, and a
/// wrong one counts as a failed attempt like on the lock screen. The vault is re-keyed (like
/// turning security on) and no note changes.
#[tauri::command(async)]
pub fn disable_security(
    app: AppHandle,
    state: State<'_, AppState>,
    current_password: String,
    current_keyfile: Option<String>,
) -> Result<()> {
    disable_security_person(app, state, current_password, current_keyfile)
}

fn disable_security_person(
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

#[tauri::command(async)]
pub fn unlock_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
    keyfile: Option<String>,
) -> Result<()> {
    unlock_vault_person(app, state, password, keyfile)
}

fn unlock_vault_person(
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
        Ok(mut session) => {
            // A vault from V1.3.0 or earlier is re-encrypted in the Notaya format, with the same password.
            session.upgrade_format_quietly(&password, kf.as_deref());
            // Refused (and the session wiped) if the window was closed to the tray meanwhile.
            state.set_session_unless_hidden(session)?;
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

/// Changes the master password and/or keyfile. The vault is re-keyed (a fresh data key and vault
/// id, see `Session::change_credentials`), so an old password with an old header or backup can't
/// open data written afterwards. A fresh backup follows; older backups still open with the old
/// password. If that backup fails the change stands and the owner gets the usual backup notice.
#[tauri::command(async)]
pub fn change_master(
    app: AppHandle,
    state: State<'_, AppState>,
    current_password: String,
    current_keyfile: Option<String>,
    new_password: String,
    new_keyfile: Option<String>,
) -> Result<()> {
    change_master_person(app, state, current_password, current_keyfile, new_password, new_keyfile)
}

fn change_master_person(
    app: AppHandle,
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
    if result.is_ok() {
        if let Err(e) = state.with(|s| backup::make_backup(&state.paths, s)) {
            state::report_backup_failure(&app, &state, &e);
        }
    }
    result
}

#[tauri::command(async)]
pub fn pick_keyfile(app: AppHandle) -> Option<String> {
    pick_keyfile_person(app)
}

fn pick_keyfile_person(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Choose your keyfile")
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.display().to_string())
}

#[tauri::command(async)]
pub fn create_keyfile(app: AppHandle) -> Result<Option<String>> {
    create_keyfile_person(app)
}

fn create_keyfile_person(app: AppHandle) -> Result<Option<String>> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save your new keyfile (keep it somewhere other than the vault)")
        .set_file_name("notaya.key")
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
    backup_now_person(app, state)
}

fn backup_now_person(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save a backup copy")
        .set_file_name(format!("notaya-backup-{}.{}", crate::model::now_ms() / 1000, crate::filenames::EXT))
        .add_filter("Notaya vault", &[crate::filenames::EXT, crate::filenames::OLD_EXT])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    // Written next to the chosen file and checked first; an existing file is replaced only
    // by a copy that passed, and the live vault is refused.
    state.with(|s| backup::save_copy(s, &path, &state.paths.vault))?;
    Ok(Some(path.display().to_string()))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings> {
    get_settings_person(&state)
}

fn get_settings_person(state: &AppState) -> Result<Settings> {
    state.with(|s| get_settings_core(s, &Actor::Person))
}

pub(crate) fn get_settings_core(s: &Session, _actor: &Actor) -> Result<Settings> {
    let mut settings = s.settings();
    // The calculator history has its own commands; the settings screen doesn't need it.
    settings.calc_history.clear();
    settings.search_history.clear();
    settings.replaced_edits.clear();
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<Settings> {
    let protect = save_settings_person(&state, settings)?;
    ui::set_capture_protection(&app, protect);
    get_settings(state)
}

fn save_settings_person(state: &AppState, settings: Settings) -> Result<bool> {
    let settings = settings.sanitized();
    // Keep sanitization before the session check, as in the original adapter.
    state.with_mut(|s| save_settings_core(s, &Actor::Person, settings))
}

pub(crate) fn save_settings_core(s: &mut Session, _actor: &Actor, settings: Settings) -> Result<bool> {
    s.run_as(_actor, |s| {
    let mut settings = settings.sanitized();
    // Histories and the last location change only through their dedicated commands.
    let stored = s.settings();
    settings.calc_history = stored.calc_history;
    settings.search_history = stored.search_history;
    settings.last_place = stored.last_place;
    settings.replaced_edits = stored.replaced_edits;
    s.put(Item::Settings(settings.clone()))?;
    Ok(s.protects_capture())
    })
}

/// Saves only the last location in the unlocked vault, without overwriting other settings.
#[tauri::command]
pub fn set_last_place(state: State<'_, AppState>, last_place: Option<LastPlace>) -> Result<()> {
    set_last_place_person(&state, last_place)
}

fn set_last_place_person(state: &AppState, last_place: Option<LastPlace>) -> Result<()> {
    state.with_mut(|s| set_last_place_core(s, &Actor::Person, last_place))
}

pub(crate) fn set_last_place_core(s: &mut Session, _actor: &Actor, last_place: Option<LastPlace>) -> Result<()> {
    s.run_as(_actor, |s| {
    if last_place.as_ref().is_some_and(|place| !place.valid()) {
        return Err(Error::invalid("The last place is not valid."));
    }
    let mut settings = s.settings();
    if settings.last_place == last_place {
        return Ok(());
    }
    settings.last_place = last_place;
    s.put(Item::Settings(settings))
    })
}

#[cfg(test)]
mod last_place_tests {
    use super::*;
    use crate::model::CalcEntry;

    fn throwaway_path() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("notaya-last-place-{}", crate::model::new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("throwaway.nota")
    }

    #[test]
    fn the_location_reopens_from_the_vault_without_changing_other_settings() {
        let path = throwaway_path();
        let mut session = Session::create_open(&path).unwrap();
        let mut settings = session.settings();
        settings.theme = "paper".into();
        settings.calc_history.push(CalcEntry { expr: "2+2".into(), result: "4".into() });
        settings.search_history.push("test search".into());
        session.put(Item::Settings(settings)).unwrap();
        let place = LastPlace { kind: "page".into(), id: crate::model::new_id(), view_id: None };
        set_last_place_core(&mut session, &Actor::Person, Some(place.clone())).unwrap();
        drop(session);
        let mut reopened = Session::unlock_open(&path).unwrap();
        let stored = reopened.settings();
        assert_eq!(stored.last_place, Some(place));
        assert_eq!(stored.theme, "paper");
        assert_eq!(stored.calc_history[0].result, "4");
        assert_eq!(stored.search_history, ["test search"]);
        set_last_place_core(&mut reopened, &Actor::Person, None).unwrap();
        drop(reopened);
        assert!(Session::unlock_open(&path).unwrap().settings().last_place.is_none());
    }

    #[test]
    fn invalid_locations_do_not_change_the_settings_record() {
        let path = throwaway_path();
        let mut session = Session::create_open(&path).unwrap();
        let before = serde_json::to_vec(&session.settings()).unwrap();
        let bad = LastPlace { kind: "page".into(), id: "a title / not an id".into(), view_id: None };
        assert!(set_last_place_core(&mut session, &Actor::Person, Some(bad)).is_err());
        assert_eq!(serde_json::to_vec(&session.settings()).unwrap(), before);
    }

    #[test]
    fn appearance_saves_preserve_the_last_place_and_histories() {
        let path = throwaway_path();
        let mut session = Session::create_open(&path).unwrap();
        let mut stored = session.settings();
        stored.last_place = Some(LastPlace { kind: "database".into(), id: crate::model::new_id(), view_id: Some(crate::model::new_id()) });
        stored.calc_history.push(CalcEntry { expr: "2+2".into(), result: "4".into() });
        stored.search_history.push("test search".into());
        session.put(Item::Settings(stored.clone())).unwrap();
        save_settings_core(&mut session, &Actor::Person, Settings { theme: "paper".into(), ..Settings::default() }).unwrap();
        let saved = session.settings();
        assert_eq!(saved.last_place, stored.last_place);
        assert_eq!(saved.calc_history[0].result, "4");
        assert_eq!(saved.search_history, stored.search_history);
        assert_eq!(saved.theme, "paper");
    }
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
    window_ready_person(window, state)
}

fn window_ready_person(window: WebviewWindow, state: State<'_, AppState>) {
    state.mark_ready(window.label());
    // Hidden in the tray (by the close button, or started hidden by Windows): a reload after a
    // lock must not bring the window back; opening it from the tray does.
    if window.label() == "main" && state.tray_hidden() {
        return;
    }
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
/// Locks and exits.
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
    open_data_folder_person(app, state)
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;

    #[test]
    fn get_settings_person_and_core_match() {
        compare(
            get_settings_person,
            |s, _| get_settings_core(s, &Actor::Person),
        );
    }

    #[test]
    fn save_settings_person_and_core_match() {
        let input = || Settings { theme: "unknown".into(), calc_history: vec![], search_history: vec![], ..Settings::default() };
        compare(
            |state| save_settings_person(state, input()),
            |s, _| save_settings_core(s, &Actor::Person, input()),
        );
    }

    #[test]
    fn set_last_place_person_and_core_match() {
        let input = || Some(LastPlace { kind: "database".into(), id: "10000000000000000000000000000007".into(), view_id: Some("10000000000000000000000000000009".into()) });
        compare(
            |state| set_last_place_person(state, input()),
            |s, _| set_last_place_core(s, &Actor::Person, input()),
        );
    }
}

fn open_data_folder_person(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(state.paths.data.display().to_string(), None::<&str>)
        .map_err(|e| Error::Other(e.to_string()))
}


/// The window identity comes from Tauri, never from an action parameter.
#[tauri::command]
pub fn editing(window: WebviewWindow, state: State<'_, AppState>, id: String, active: bool) -> Result<()> {
    state.with_mut(|s| {
        if s.get(&id).is_none() { return Err(Error::NotFound); }
        s.coordination.editing(&id, window.label(), active);
        Ok(())
    })
}
