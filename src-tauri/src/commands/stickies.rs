use tauri::{AppHandle, Manager, State};

use super::{check_color, limit, sticky};
use crate::error::{Error, Result};
use crate::model::{new_id, now_ms, Item, Sticky};
use crate::state::AppState;
use crate::ui;

#[tauri::command]
pub fn list_stickies(state: State<'_, AppState>) -> Result<Vec<Sticky>> {
    state.with(|s| {
        let mut list: Vec<Sticky> = s
            .items()
            .filter_map(|i| match i {
                Item::Sticky(x) => Some(x.clone()),
                _ => None,
            })
            .collect();
        list.sort_by_key(|x| std::cmp::Reverse(x.updated));
        Ok(list)
    })
}

// Async: creating a window from a sync command can deadlock on Windows.
#[tauri::command(async)]
pub fn create_sticky(app: AppHandle, state: State<'_, AppState>, open: bool) -> Result<Sticky> {
    let (st, protect) = state.with_mut(|s| {
        let settings = s.settings();
        let count = s.items().filter(|i| matches!(i, Item::Sticky(_))).count() as f64;
        let now = now_ms();
        let st = Sticky {
            id: new_id(),
            content: String::new(),
            text: String::new(),
            color: settings.sticky_default_color.clone(),
            x: 140.0 + (count % 8.0) * 36.0,
            y: 140.0 + (count % 8.0) * 36.0,
            w: 320.0,
            h: 320.0,
            on_top: true,
            open,
            created: now,
            updated: now,
        };
        s.put(Item::Sticky(st.clone()))?;
        Ok((st, s.protects_capture()))
    })?;
    if open {
        ui::open_sticky(&app, &state.paths, &st, protect)?;
    }
    Ok(st)
}

#[tauri::command]
pub fn get_sticky(state: State<'_, AppState>, id: String) -> Result<Sticky> {
    state.with(|s| sticky(s, &id))
}

#[tauri::command]
pub fn save_sticky(state: State<'_, AppState>, id: String, content: String, text: String) -> Result<()> {
    limit("Sticky note", &content, 2_000_000)?;
    limit("Sticky note", &text, 2_000_000)?;
    state.with_mut(|s| {
        let mut st = sticky(s, &id)?;
        st.content = content;
        st.text = text;
        st.updated = now_ms();
        s.put(Item::Sticky(st))
    })
}

#[tauri::command]
pub fn set_sticky_style(app: AppHandle, state: State<'_, AppState>, id: String, color: String, on_top: bool) -> Result<Sticky> {
    let color = check_color(&color)?;
    let st = state.with_mut(|s| {
        let mut st = sticky(s, &id)?;
        st.color = color;
        st.on_top = on_top;
        s.put(Item::Sticky(st.clone()))?;
        Ok(st)
    })?;
    if let Some(w) = app.get_webview_window(&format!("{}{}", ui::STICKY_PREFIX, id)) {
        let _ = w.set_always_on_top(on_top);
    }
    Ok(st)
}

#[tauri::command]
pub fn set_sticky_geometry(state: State<'_, AppState>, id: String, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
    if ![x, y, w, h].iter().all(|v| v.is_finite()) {
        return Ok(());
    }
    state.with_mut(|s| {
        let mut st = sticky(s, &id)?;
        st.x = x;
        st.y = y;
        st.w = w.clamp(220.0, 1200.0);
        st.h = h.clamp(180.0, 1200.0);
        s.put(Item::Sticky(st))
    })
}

#[tauri::command(async)]
pub fn open_sticky(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    let (st, protect) = state.with_mut(|s| {
        let mut st = sticky(s, &id)?;
        st.open = true;
        s.put(Item::Sticky(st.clone()))?;
        Ok((st, s.protects_capture()))
    })?;
    ui::open_sticky(&app, &state.paths, &st, protect)?;
    Ok(())
}

#[tauri::command]
pub fn close_sticky(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    state.with_mut(|s| {
        let mut st = sticky(s, &id)?;
        st.open = false;
        s.put(Item::Sticky(st))
    })?;
    ui::close_sticky(&app, &id);
    Ok(())
}

#[tauri::command]
pub fn delete_sticky(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    state.with_mut(|s| {
        sticky(s, &id)?;
        s.remove_many(&[id.clone()])?;
        s.collect_garbage()?;
        Ok(())
    })?;
    ui::close_sticky(&app, &id);
    Ok(())
}

/// Deletes several sticky notes in one vault write. Ids that no longer exist (deleted
/// in their own window meanwhile) are skipped; if any other id is not a sticky note,
/// nothing is removed.
#[tauri::command]
pub fn delete_stickies(app: AppHandle, state: State<'_, AppState>, ids: Vec<String>) -> Result<()> {
    let mut ids = ids;
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return Ok(());
    }
    if ids.len() > 10_000 {
        return Err(Error::Invalid("Too many sticky notes at once.".into()));
    }
    state.with_mut(|s| {
        let mut found = Vec::with_capacity(ids.len());
        for id in &ids {
            match s.get(id) {
                None => {}
                Some(Item::Sticky(_)) => found.push(id.clone()),
                Some(_) => return Err(Error::NotFound),
            }
        }
        if found.is_empty() {
            return Ok(());
        }
        s.remove_many(&found)?;
        s.collect_garbage()?;
        Ok(())
    })?;
    for id in &ids {
        ui::close_sticky(&app, id);
    }
    Ok(())
}
