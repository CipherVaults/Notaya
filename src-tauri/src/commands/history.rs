//! Thin window wrappers; the pipe dispatcher calls only the actor-aware cores.
use crate::{
    actions::Actor,
    error::Result,
    state::AppState,
    vault::{
        history::{VersionBody, VersionMeta},
        Session,
    },
};
use tauri::State;

pub(crate) fn list_core(s: &Session, _actor: &Actor, page_id: &str) -> Result<Vec<VersionMeta>> {
    s.history_list(page_id)
}
pub(crate) fn get_core(
    s: &Session,
    _actor: &Actor,
    page_id: &str,
    version_id: &str,
) -> Result<VersionBody> {
    s.history_get(page_id, version_id)
}
pub(crate) fn restore_core(
    s: &mut Session,
    actor: &Actor,
    page_id: &str,
    version_id: &str,
) -> Result<()> {
    s.history_restore(actor, page_id, version_id)
}

#[tauri::command(async)]
pub fn history_list(state: State<'_, AppState>, page_id: String) -> Result<Vec<VersionMeta>> {
    state.with(|s| list_core(s, &Actor::Person, &page_id))
}
#[tauri::command(async)]
pub fn history_get(
    state: State<'_, AppState>,
    page_id: String,
    version_id: String,
) -> Result<VersionBody> {
    state.with(|s| get_core(s, &Actor::Person, &page_id, &version_id))
}
#[tauri::command(async)]
pub fn history_restore(
    state: State<'_, AppState>,
    page_id: String,
    version_id: String,
) -> Result<()> {
    state.with_mut(|s| restore_core(s, &Actor::Person, &page_id, &version_id))
}
#[tauri::command(async)]
pub fn history_snapshot(state: State<'_, AppState>, page_id: String) -> Result<()> {
    state.with_mut(|s| s.snapshot_page(&page_id))
}
