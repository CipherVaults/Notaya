//! The Windows section of Settings: close to the tray, start with Windows, start hidden.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{Error, Result};
use crate::shell::{self, Prefs};
use crate::state::AppState;

/// What the UI shows. `start_with_windows` is read from Windows each time, so it is true only
/// while the start-up entry really starts this exe.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellPrefs {
    pub close_to_tray: bool,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    /// False for a portable copy: its start-up entry names the folder it is in now, so the UI
    /// warns that moving the folder breaks it.
    pub installed: bool,
    /// False when no tray icon could be made: closing then quits, as before.
    pub tray_available: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellPrefsInput {
    pub close_to_tray: bool,
    pub start_with_windows: bool,
    pub start_minimized: bool,
}

fn current_exe() -> Result<std::path::PathBuf> {
    std::env::current_exe().map_err(Error::from)
}

#[tauri::command]
pub fn get_shell_prefs(state: State<'_, AppState>) -> Result<ShellPrefs> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let prefs = shell::load_prefs(&state.paths);
    Ok(ShellPrefs {
        close_to_tray: prefs.close_to_tray,
        start_with_windows: shell::autostart_enabled(&current_exe()?),
        start_minimized: prefs.start_minimized,
        installed: state.paths.installed,
        tray_available: state.tray_ready(),
    })
}

/// Saves the switches and updates (or removes) the start-up entry to match. If Windows refuses
/// the entry, the error says so and the file is not changed.
#[tauri::command]
pub fn set_shell_prefs(state: State<'_, AppState>, prefs: ShellPrefsInput) -> Result<ShellPrefs> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    let exe = current_exe()?;
    shell::set_autostart(&exe, prefs.start_with_windows, prefs.start_minimized)?;
    shell::save_prefs(
        &state.paths,
        &Prefs { close_to_tray: prefs.close_to_tray, start_minimized: prefs.start_minimized },
    )?;
    get_shell_prefs(state)
}
