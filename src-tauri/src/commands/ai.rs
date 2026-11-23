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

#[tauri::command(async)]
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
#[tauri::command(async)]
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
#[tauri::command(async)]
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
#[tauri::command(async)]
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
#[tauri::command(async)]
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
#[tauri::command(async)]
pub fn ai_pause(window: WebviewWindow, state: State<'_, AppState>) -> Result<()> {
    main_window(&window)?;
    update(&state, "ai_pause", |a| {
        a.paused = true;
        Ok(())
    })
}
#[tauri::command(async)]
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

#[cfg(test)]
mod native_ai_tests {
    use super::*;
    use crate::native_confirm::{test_support::*, TestApprover};
    use std::sync::{Arc, Mutex};

    #[test]
    fn enable_deny_preserves_vault_history_and_settings_allow_enables_and_no_answer_is_cached() {
        let state=fixture();state.ai.native.inject(TestApprover(false));
        let before=snapshot(&state);let epoch=native_confirm::epoch(&state);
        assert_eq!(ai_set_enabled_core(&state,true,Some("ALLOW".into())).unwrap_err().to_string(),"cancelled");
        assert_eq!(snapshot(&state),before);assert_eq!(native_confirm::epoch(&state),epoch);
        state.ai.native.inject(TestApprover(true));
        ai_set_enabled_core(&state,true,Some("ALLOW".into())).unwrap();
        assert!(state.with(|s|Ok(s.settings().ai.enabled)).unwrap());
        assert_eq!(native_confirm::epoch(&state),epoch+1);
        state.ai.native.inject(TestApprover(false));let before=snapshot(&state);
        assert!(ai_set_enabled_core(&state,true,Some("ALLOW".into())).is_err());
        assert_eq!(snapshot(&state),before);
        // Turning off still has its original behavior and needs no native answer.
        ai_set_enabled_core(&state,false,None).unwrap();
        assert!(!state.with(|s|Ok(s.settings().ai.enabled)).unwrap());
    }
    #[test]
    fn enable_changed_target_invalidates_answer_without_enabling() {
        let state=fixture();let weak=Arc::downgrade(&state);let saved=Arc::new(Mutex::new(None));let copy=saved.clone();
        state.ai.native.inject(Hook(Box::new(move || {
            let state=weak.upgrade().unwrap();
            state.with_mut(|s| { let mut x=s.settings();x.ai.paused=true;s.put(Item::Settings(x)) }).unwrap();
            *copy.lock().unwrap()=Some(snapshot(&state));true
        })));
        assert_eq!(ai_set_enabled_core(&state,true,Some("ALLOW".into())).unwrap_err().to_string(),"cancelled");
        assert_eq!(snapshot(&state),saved.lock().unwrap().take().unwrap());
    }
    #[test]
    fn pairing_deny_is_unchanged_allow_persists_look_and_script_and_replay_cannot_approve() {
        for allow in [false,true] {
            let state=fixture();
            state.with_mut(|s| { let mut x=s.settings();x.ai.enabled=true;s.put(Item::Settings(x)) }).unwrap();
            state.ai.native.inject(TestApprover(allow));
            let before=snapshot(&state);let mut id=String::new();
            let approved=state.ai.confirms.request("pairing",json!({"client":"SF2 test client"}),crate::actions::Confirm::Ask,|v| {
                id=v["id"].as_str().unwrap().into();
                assert!(!state.ai.confirms.answer("confirm",&id,true,None));
                assert!(!state.ai.confirms.answer("pairing",&id,true,None));
                let result=ai_pair_answer_core(&state,id.clone(),true);
                if allow { result.unwrap(); } else { assert_eq!(result.unwrap_err().to_string(),"cancelled"); }
            },||false);
            assert_eq!(approved,allow);assert_eq!(snapshot(&state),before);
            if approved {
                let (p,token)=pairing::create("SF2 test client".into());
                pairing::persist(&state,p.clone(),native_confirm::epoch(&state),||false).unwrap();
                state.with(|s| {
                    assert_eq!(s.settings().ai.pairings.len(),1);
                    assert_eq!(s.settings().ai.pairings[0].level,Level::Look);
                    assert!(pairing::authenticate(&s.settings().ai,&p.id,&token).is_some());Ok(())
                }).unwrap();
            }
            let after=snapshot(&state);
            assert!(ai_pair_answer_core(&state,id,true).is_err());
            assert_eq!(snapshot(&state),after);
        }
    }
    #[test]
    fn pairing_changed_settings_or_cancelled_request_invalidates_native_answer() {
        for change_settings in [false,true] {
            let state=fixture();
            state.with_mut(|s| { let mut x=s.settings();x.ai.enabled=true;s.put(Item::Settings(x)) }).unwrap();
            let weak=Arc::downgrade(&state);let saved=Arc::new(Mutex::new(None));let copy=saved.clone();
            state.ai.native.inject(Hook(Box::new(move || {
                let state=weak.upgrade().unwrap();
                if change_settings { state.with_mut(|s|{let mut x=s.settings();x.ai.paused=true;s.put(Item::Settings(x))}).unwrap(); }
                else {state.ai.confirms.cancel_all();}
                *copy.lock().unwrap()=Some(snapshot(&state));true
            })));
            assert!(!state.ai.confirms.request("pairing",json!({"client":"Original Rust client"}),crate::actions::Confirm::Ask,|v|{
                assert_eq!(ai_pair_answer_core(&state,v["id"].as_str().unwrap().into(),true).unwrap_err().to_string(),"cancelled");
            },||false));
            assert_eq!(snapshot(&state),saved.lock().unwrap().take().unwrap());
            assert!(state.with(|s|Ok(s.settings().ai.pairings.is_empty())).unwrap());
        }
    }
}
