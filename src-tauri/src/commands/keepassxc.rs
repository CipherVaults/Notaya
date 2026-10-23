//! Commands for the bundled KeePassXC (amendments A9 and A10).

use tauri::{State, WebviewWindow};

use crate::embed::Rect;
use crate::error::{Error, Result};
use crate::keepassxc::{self, Status};
use crate::state::AppState;

#[tauri::command(async)]
pub fn keepassxc_status(state: State<'_, AppState>) -> Status {
    keepassxc::status(&state.paths, &state.keepassxc)
}

/// The Passwords feature is optional (A20). While it is off, nothing starts KeePassXC.
fn require_passwords(state: &AppState) -> Result<()> {
    if state.passwords_on() {
        Ok(())
    } else {
        Err(Error::invalid("Passwords is turned off. Turn it on in Settings first."))
    }
}

/// Opens the password database in KeePassXC's own window. Cipherleaf must be unlocked.
/// An embedded KeePassXC is first given back its own window.
#[tauri::command(async)]
pub fn keepassxc_open(state: State<'_, AppState>) -> Result<()> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    require_passwords(&state)?;
    keepassxc::detach()?;
    keepassxc::open(&state.paths, &state.keepassxc)
}

/// Shows KeePassXC inside Cipherleaf's window at the given rectangle (physical
/// pixels, relative to the window). Starts KeePassXC first if needed.
#[tauri::command(async)]
pub fn keepassxc_embed(window: WebviewWindow, state: State<'_, AppState>, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    require_passwords(&state)?;
    let parent = window.hwnd()?.0 as isize;
    keepassxc::embed(&state.paths, &state.keepassxc, parent, Rect { x, y, w, h }, &|| state.is_unlocked())
}

/// Moves and resizes the embedded KeePassXC as the panel changes size. Refused
/// while Cipherleaf is locked, so a late call can't bring the window back.
#[tauri::command]
pub fn keepassxc_place(state: State<'_, AppState>, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
    if !state.is_unlocked() {
        return Err(Error::Locked);
    }
    crate::embed::place(Rect { x, y, w, h })
}

/// Hides the embedded KeePassXC (when leaving the Passwords panel or opening an overlay).
#[tauri::command]
pub fn keepassxc_hide() {
    crate::embed::hide();
}

/// Turns the embedded KeePassXC into a normal separate window.
#[tauri::command(async)]
pub fn keepassxc_detach() -> Result<()> {
    keepassxc::detach()
}

#[tauri::command(async)]
pub fn keepassxc_lock(state: State<'_, AppState>) {
    keepassxc::lock_if_running(&state.paths, &state.keepassxc);
}
