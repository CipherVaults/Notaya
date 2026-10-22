//! Commands for the bundled KeePassXC (amendment A9).

use tauri::State;

use crate::error::{Error, Result};
use crate::keepassxc::{self, Status};
use crate::state::AppState;

#[tauri::command(async)]
pub fn keepassxc_status(state: State<'_, AppState>) -> Status {
    keepassxc::status(&state.paths, &state.keepassxc)
}

/// Opens the password database in KeePassXC. Cipherleaf must be unlocked.
#[tauri::command(async)]
pub fn keepassxc_open(state: State<'_, AppState>) -> Result<()> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    keepassxc::open(&state.paths, &state.keepassxc)
}

#[tauri::command(async)]
pub fn keepassxc_lock(state: State<'_, AppState>) {
    keepassxc::lock_if_running(&state.paths, &state.keepassxc);
}
