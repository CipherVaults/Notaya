use crate::{
    actions::Actor,
    error::Result,
    state::AppState,
    vault::{
        activity::{ActivityMeta, UndoSummary},
        Session,
    },
};
use tauri::State;

pub(crate) fn list_core(
    s: &Session,
    _actor: &Actor,
    who: Option<&str>,
    before: Option<u64>,
    limit: usize,
) -> Result<Vec<ActivityMeta>> {
    s.activity_list(who, before, limit)
}
pub(crate) fn verify_core(s: &Session, _actor: &Actor) -> Result<()> {
    s.activity_verify()
}
pub(crate) fn undo_core(s: &mut Session, actor: &Actor, change_set: &str) -> Result<UndoSummary> {
    s.activity_undo(actor, change_set)
}

#[tauri::command(async)]
pub fn activity_list(
    state: State<'_, AppState>,
    who: Option<String>,
    before: Option<u64>,
    limit: Option<usize>,
) -> Result<Vec<ActivityMeta>> {
    state.with(|s| {
        list_core(
            s,
            &Actor::Person,
            who.as_deref(),
            before,
            limit.unwrap_or(100),
        )
    })
}
#[tauri::command(async)]
pub fn activity_verify(state: State<'_, AppState>) -> Result<()> {
    state.with(|s| verify_core(s, &Actor::Person))
}
#[tauri::command(async)]
pub fn activity_undo(state: State<'_, AppState>, change_set: String) -> Result<UndoSummary> {
    state.with_mut(|s| undo_core(s, &Actor::Person, &change_set))
}
// Window only. There is deliberately no corresponding pipe action/core.
#[tauri::command(async)]
pub fn activity_clear(state: State<'_, AppState>, confirmation: String) -> Result<()> {
    state.with_mut(|s| s.clear_activity(&Actor::Person, &confirmation))
}
