//! These commands are window adapters, never action handlers.
use crate::{
    ai::{pairing, rules::Level},
    error::{Error, Result},
    model::Item,
    state::AppState,
};
use serde_json::{json, Value};
use tauri::{State, WebviewWindow};
use tauri::{AppHandle, Manager};
use crate::native_confirm;

fn main_window(window: &WebviewWindow) -> Result<()> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err(Error::invalid("invalid"))
    }
}
pub(crate) fn update(
    state: &AppState,
    action: &'static str,
    f: impl FnOnce(&mut pairing::Settings) -> Result<()>,
) -> Result<()> {
    state.with_mut(|s| {
        s.run_action(&crate::actions::Actor::Person, action, |s| {
            let mut settings = s.settings();
            f(&mut settings.ai)?;
            s.put(Item::Settings(settings))
        })
    })?;
    // A level update is observed live by each connection. Cancel pending dialogs
    // without tearing down the bridge; it receives level_changed on the pipe.
    if action == "ai_set_level" {
        state.ai.confirms.cancel_all();
    } else {
        state.ai.invalidate();
    }
    Ok(())
}

#[tauri::command]
pub fn ai_get_status(window: WebviewWindow, state: State<'_, AppState>) -> Result<Value> {
    main_window(&window)?;
    state.with(|s| {
        let a = s.settings().ai;
        Ok(
            json!({"enabled":a.enabled,"paused":a.paused,"hiddenNotebooks":a.hidden_notebooks,
            "bigChangeConfirm":a.big_change_confirm,"bigChangeItems":a.big_change_items,
            "pairings":a.pairings.iter().map(|p|json!({"id":p.id,"name":p.name,"level":p.level,
                "created":p.created,"lastUsed":p.last_used})).collect::<Vec<_>>()}),
        )
    })
}
#[tauri::command]
pub async fn ai_set_enabled(
    window: WebviewWindow,
    app: AppHandle,
    enabled: bool,
    confirmation: Option<String>,
) -> Result<()> {
    main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || ai_set_enabled_core(&app.state::<AppState>(), enabled, confirmation))
        .await.map_err(|_| native_confirm::cancelled())?
}
pub(crate) fn ai_set_enabled_core(state: &AppState, enabled: bool, confirmation: Option<String>) -> Result<()> {
    if enabled && confirmation.as_deref() != Some("ALLOW") {
        return Err(Error::invalid("not_confirmed"));
    }
    if !enabled { return update(state, "ai_set_enabled", |a| { a.enabled = false; Ok(()) }); }
    let epoch = native_confirm::epoch(state);
    let target = state.with(|s| Ok(native_confirm::enable_target(s, epoch)))?;
    let mut approval = state.ai.native.ask(target)?;
    state.with_mut(|s| {
        native_confirm::check_epoch(state, epoch)?;
        approval.consume(native_confirm::enable_target(s, epoch))?;
        s.run_action(&crate::actions::Actor::Person, "ai_set_enabled", |s| {
            let mut settings = s.settings(); settings.ai.enabled = true;
            s.put(Item::Settings(settings))
        })
    })?;
    state.ai.invalidate();
    Ok(())
}
#[tauri::command]
pub fn ai_set_level(
    window: WebviewWindow,
    state: State<'_, AppState>,
    pairing_id: String,
    level: Level,
) -> Result<()> {
    main_window(&window)?;
    if level == Level::Off {
        return Err(Error::invalid("invalid"));
    }
    update(&state, "ai_set_level", |a| {
        a.pairings
            .iter_mut()
            .find(|p| p.id == pairing_id)
            .ok_or(Error::NotFound)?
            .level = level;
        Ok(())
    })
}
#[tauri::command]
pub fn ai_remove_pairing(
    window: WebviewWindow,
    state: State<'_, AppState>,
    pairing_id: String,
) -> Result<()> {
    main_window(&window)?;
    update(&state, "ai_remove_pairing", |a| {
        a.pairings.retain(|p| p.id != pairing_id);
        Ok(())
    })
}
#[tauri::command]
pub fn ai_set_hidden(
    window: WebviewWindow,
    state: State<'_, AppState>,
    notebook_ids: Vec<String>,
) -> Result<()> {
    main_window(&window)?;
    state.with(|s| {
        if notebook_ids.len() > 10_000
            || notebook_ids
                .iter()
                .any(|id| !matches!(s.get(id), Some(Item::Notebook(_))))
        {
            return Err(Error::invalid("invalid"));
        }
        Ok(())
    })?;
    update(&state, "ai_set_hidden", |a| {
        a.hidden_notebooks = notebook_ids;
        Ok(())
    })
}
#[tauri::command]
pub fn ai_set_big_change(
    window: WebviewWindow,
    state: State<'_, AppState>,
    enabled: bool,
    items: Option<usize>,
) -> Result<()> {
    main_window(&window)?;
    if items.is_some_and(|n| n == 0 || n > 500) {
        return Err(Error::invalid("invalid"));
    }
    update(&state, "ai_set_big_change", |a| {
        a.big_change_confirm = enabled;
        if let Some(n) = items {
            a.big_change_items = n;
        }
        Ok(())
    })
}
#[tauri::command]
pub fn ai_pause(window: WebviewWindow, state: State<'_, AppState>) -> Result<()> {
    main_window(&window)?;
    update(&state, "ai_pause", |a| {
        a.paused = true;
        Ok(())
    })
}
#[tauri::command]
pub fn ai_resume(window: WebviewWindow, state: State<'_, AppState>) -> Result<()> {
    main_window(&window)?;
    update(&state, "ai_resume", |a| {
        a.paused = false;
        Ok(())
    })
}
#[tauri::command]
pub fn ai_confirm_answer(
    window: WebviewWindow,
    state: State<'_, AppState>,
    id: String,
    allow: bool,
    word: Option<String>,
) -> Result<()> {
    main_window(&window)?;
    if !state
        .ai
        .confirms
        .answer("confirm", &id, allow, word.as_deref())
    {
        return Err(Error::invalid("not_confirmed"));
    }
    Ok(())
}
#[tauri::command]
pub async fn ai_pair_answer(
    window: WebviewWindow,
    app: AppHandle,
    id: String,
    allow: bool,
) -> Result<()> {
    // Pairing approval and action confirmations share a broker but not an identifier.
    main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || ai_pair_answer_core(&app.state::<AppState>(), id, allow))
        .await.map_err(|_| native_confirm::cancelled())?
}
pub(crate) fn ai_pair_answer_core(state: &AppState, id: String, allow: bool) -> Result<()> {
    if !allow {
        if !state.ai.confirms.answer("pairing", &id, false, None) { return Err(Error::invalid("pairing_refused")); }
        return Ok(());
    }
    let epoch = native_confirm::epoch(state);
    let request = state.ai.confirms.pairing_request(&id).ok_or_else(native_confirm::cancelled)?;
    let target = state.with(|s| Ok(native_confirm::pair_target(s, epoch, request.id.clone(), request.client.clone())))?;
    let result = (|| {
        let mut approval = state.ai.native.ask(target)?;
        state.with(|s| {
            native_confirm::check_epoch(state, epoch)?;
            approval.consume(native_confirm::pair_target(s, epoch, request.id.clone(), request.client.clone()))?;
            if !state.ai.confirms.answer_native_pairing(&request) { return Err(native_confirm::cancelled()); }
            Ok(())
        })
    })();
    if result.is_err() { state.ai.confirms.answer("pairing", &id, false, None); }
    result
}
