//! The pop-out calculators. Any number of calculator windows can be open; they share one
//! history, which is stored (encrypted like everything else) in the vault's settings
//! record, so it is still there after the windows close or the app restarts.

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
    let protect = state.with(|s| Ok(s.protects_capture()))?;
    let window = ui::open_calculator(&app, &state.paths, protect)?;
    // If the vault locked while the window was being made, the lock has already closed the others.
    if !state.is_unlocked() {
        let _ = window.destroy();
    }
    Ok(())
}

#[tauri::command]
pub fn calc_history(state: State<'_, AppState>) -> Result<Vec<CalcEntry>> {
    state.with(|s| Ok(s.settings().calc_history))
}

#[tauri::command]
pub fn add_calc_entry(app: AppHandle, state: State<'_, AppState>, expr: String, result: String) -> Result<Vec<CalcEntry>> {
    limit("The calculation", &expr, 1_000)?;
    limit("The result", &result, 1_000)?;
    let history = state.with_mut(|s| {
        let mut settings = s.settings();
        if !push_entry(&mut settings.calc_history, expr, result) {
            return Ok(settings.calc_history);
        }
        let history = settings.calc_history.clone();
        s.put(Item::Settings(settings))?;
        Ok(history)
    })?;
    let _ = app.emit(CHANGED, ());
    Ok(history)
}

#[tauri::command]
pub fn clear_calc_history(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    state.with_mut(|s| {
        let mut settings = s.settings();
        if settings.calc_history.is_empty() {
            return Ok(());
        }
        settings.calc_history.clear();
        s.put(Item::Settings(settings))
    })?;
    let _ = app.emit(CHANGED, ());
    Ok(())
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
