//! The pop-out calculators. Any number of calculator windows can be open; they share one
//! history, which is stored (encrypted like everything else) in the vault's settings
//! record, so it is still there after the windows close or the app restarts.

use crate::actions::Actor;
use crate::vault::Session;

use tauri::{AppHandle, Emitter, State};

use super::limit;
use crate::error::Result;
use crate::model::{CalcEntry, Item, CALC_HISTORY_MAX};
use crate::state::AppState;
use crate::ui;

/// Sent to every window when the shared history changes, so each calculator shows it.
const CHANGED: &str = "calc-history-changed";

// Async: creating a window from a sync command can deadlock on Windows.
#[tauri::command(async)]
pub fn open_calculator(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    let protect = open_calculator_person(&state)?;
    let window = ui::open_calculator(&app, &state.paths, protect)?;
    // If the vault locked while the window was being made, the lock has already closed the others.
    if !state.is_unlocked() {
        let _ = window.destroy();
    }
    Ok(())
}

/// Vault-side precondition; window creation stays in the UI adapter.
pub(crate) fn open_calculator_core(s: &Session, _actor: &Actor) -> Result<bool> {
    Ok(s.protects_capture())
}

#[tauri::command]
pub fn calc_history(state: State<'_, AppState>) -> Result<Vec<CalcEntry>> {
    calc_history_person(&state)
}

pub(crate) fn calc_history_core(s: &Session, _actor: &Actor) -> Result<Vec<CalcEntry>> {
    Ok(s.settings().calc_history)
}


#[tauri::command]
pub fn add_calc_entry(app: AppHandle, state: State<'_, AppState>, expr: String, result: String) -> Result<Vec<CalcEntry>> {
    limit("The calculation", &expr, 1_000)?;
    limit("The result", &result, 1_000)?;
    let history = add_calc_entry_person(&state, expr, result)?;
    let _ = app.emit(CHANGED, ());
    Ok(history)
}

pub(crate) fn add_calc_entry_core(
    s: &mut Session,
    _actor: &Actor,
    expr: String,
    result: String,
) -> Result<Vec<CalcEntry>> {
    limit("The calculation", &expr, 1_000)?;
    limit("The result", &result, 1_000)?;

    let mut settings = s.settings();
    if !push_entry(&mut settings.calc_history, expr, result) {
        return Ok(settings.calc_history);
    }
    let history = settings.calc_history.clone();
    s.put(Item::Settings(settings))?;
    Ok(history)
}


#[tauri::command]
pub fn clear_calc_history(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    clear_calc_history_person(&state)?;
    let _ = app.emit(CHANGED, ());
    Ok(())
}

pub(crate) fn clear_calc_history_core(s: &mut Session, _actor: &Actor) -> Result<()> {
    let mut settings = s.settings();
    if settings.calc_history.is_empty() {
        return Ok(());
    }
    settings.calc_history.clear();
    s.put(Item::Settings(settings))
}


/// Puts a calculation at the top of the history and drops the oldest past the limit.
/// Returns false (and changes nothing) when it is empty or repeats the newest line,
/// as pressing "=" twice does.
fn push_entry(history: &mut Vec<CalcEntry>, expr: String, result: String) -> bool {
    let expr = expr.trim().to_string();
    let result = result.trim().to_string();
    if expr.is_empty() || result.is_empty() {
        return false;
    }
    if history.first().is_some_and(|h| h.expr == expr && h.result == result) {
        return false;
    }
    history.insert(0, CalcEntry { expr, result });
    history.truncate(CALC_HISTORY_MAX);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(expr: &str, result: &str) -> CalcEntry {
        CalcEntry { expr: expr.into(), result: result.into() }
    }

    #[test]
    fn newest_first_and_capped() {
        let mut h = Vec::new();
        for i in 0..CALC_HISTORY_MAX + 5 {
            assert!(push_entry(&mut h, format!("{i}+1"), (i + 1).to_string()));
        }
        assert_eq!(h.len(), CALC_HISTORY_MAX);
        assert_eq!(h[0].expr, format!("{}+1", CALC_HISTORY_MAX + 4));
    }

    #[test]
    fn skips_empty_lines_and_an_immediate_repeat() {
        let mut h = vec![entry("2+2", "4")];
        assert!(!push_entry(&mut h, "  ".into(), "4".into()));
        assert!(!push_entry(&mut h, "2+2 ".into(), "4".into()));
        assert!(push_entry(&mut h, "4".into(), "4".into()));
        assert!(push_entry(&mut h, "2+2".into(), "4".into()), "an older repeat is still added");
        assert_eq!(h.len(), 3);
    }
}


fn clear_calc_history_person(state: &AppState) -> Result<()> {
    state.with_mut(|s| clear_calc_history_core(s, &Actor::Person))
}


fn add_calc_entry_person(state: &AppState, expr: String, result: String) -> Result<Vec<CalcEntry>> {
    state.with_mut(|s| add_calc_entry_core(s, &Actor::Person, expr, result))
}


fn calc_history_person(state: &AppState) -> Result<Vec<CalcEntry>> {
    state.with(|s| calc_history_core(s, &Actor::Person))
}


fn open_calculator_person(state: &AppState) -> Result<bool> {
    state.with(|s| open_calculator_core(s, &Actor::Person))
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;
    #[test]
    fn clear_calc_history_person_and_core_match() {
        compare(
            |state| clear_calc_history_person(state),
            |s, _paths| clear_calc_history_core(s, &Actor::Person),
        );
    }
    #[test]
    fn add_calc_entry_person_and_core_match() {
        compare(
            |state| add_calc_entry_person(state, "1+2".into(), "3".into()),
            |s, _paths| add_calc_entry_core(s, &Actor::Person, "1+2".into(), "3".into()),
        );
    }
    #[test]
    fn calc_history_person_and_core_match() {
        compare(
            |state| calc_history_person(state),
            |s, _paths| calc_history_core(s, &Actor::Person),
        );
    }
    #[test]
    fn open_calculator_person_and_core_match() {
        compare(
            |state| open_calculator_person(state),
            |s, _paths| open_calculator_core(s, &Actor::Person),
        );
    }
}
