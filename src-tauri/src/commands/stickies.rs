use crate::actions::Actor;
use crate::vault::Session;

use tauri::{AppHandle, Manager, State};

use super::{check_color, limit, sticky};
use crate::error::{Error, Result};
use crate::model::{new_id, now_ms, Item, Sticky};
use crate::state::AppState;
use crate::ui;

#[tauri::command]
pub fn list_stickies(state: State<'_, AppState>) -> Result<Vec<Sticky>> {
    list_stickies_person(&state)
}

pub(crate) fn list_stickies_core(s: &Session, _actor: &Actor) -> Result<Vec<Sticky>> {
    let mut list: Vec<Sticky> = s
        .items()
        .filter_map(|i| match i {
            Item::Sticky(x) if x.trashed_at.is_none() => Some(x.clone()),
            _ => None,
        })
        .collect();
    list.sort_by_key(|x| std::cmp::Reverse(x.updated));
    Ok(list)
}


// Async: creating a window from a sync command can deadlock on Windows.
#[tauri::command(async)]
pub fn create_sticky(app: AppHandle, state: State<'_, AppState>, open: bool) -> Result<Sticky> {
    let (st, protect) = create_sticky_person(&state, open)?;
    if open {
        ui::open_sticky(&app, &state.paths, &st, protect)?;
    }
    Ok(st)
}

pub(crate) fn create_sticky_core(s: &mut Session, _actor: &Actor, open: bool) -> Result<(Sticky, bool)> {
    s.run_as(_actor, |s| {
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
    trashed_at: None, };
    s.put(Item::Sticky(st.clone()))?;
    Ok((st, s.protects_capture()))
    })
}


#[tauri::command]
pub fn get_sticky(state: State<'_, AppState>, id: String) -> Result<crate::changes::Revisioned<Sticky>> {
    state.with(|s| { let st = sticky(s, &id)?; Ok(s.revisioned(&id, st)) })
}

pub(crate) fn get_sticky_core(s: &Session, _actor: &Actor, id: String) -> Result<Sticky> {
    sticky(s, &id)
}


#[tauri::command]
pub fn save_sticky(state: State<'_, AppState>, id: String, content: String, text: String, base_rev: Option<u64>) -> Result<u64> {
    state.with_mut(|s| s.person_save(&id.clone(), base_rev, |s| {
        save_sticky_core(s, &Actor::Person, id.clone(), content, text)?;
        Ok(s.rev(&id))
    }))
}

pub(crate) fn save_sticky_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    content: String,
    text: String,
) -> Result<()> {
    s.run_as(_actor, |s| {
    limit("Sticky note", &content, 2_000_000)?;
    limit("Sticky note", &text, 2_000_000)?;

    let mut st = sticky(s, &id)?;
    if matches!(_actor, Actor::Ai { .. }) && st.trashed_at.is_some() { return Err(Error::invalid("This note is in the bin. Restore it before editing.")); }
    st.content = content;
    st.text = text;
    st.updated = now_ms();
    s.put(Item::Sticky(st))
    })
}


#[tauri::command]
pub fn set_sticky_style(app: AppHandle, state: State<'_, AppState>, id: String, color: String, on_top: bool) -> Result<Sticky> {
    let color = check_color(&color)?;
    let st = set_sticky_style_person(&state, id.clone(), color, on_top)?;
    if let Some(w) = app.get_webview_window(&format!("{}{}", ui::STICKY_PREFIX, id)) {
        let _ = w.set_always_on_top(on_top);
    }
    Ok(st)
}

pub(crate) fn set_sticky_style_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    color: String,
    on_top: bool,
) -> Result<Sticky> {
    s.run_as(_actor, |s| {
    let color = check_color(&color)?;

    let mut st = sticky(s, &id)?;
    st.color = color;
    st.on_top = on_top;
    s.put(Item::Sticky(st.clone()))?;
    Ok(st)
    })
}


#[tauri::command]
pub fn set_sticky_geometry(state: State<'_, AppState>, id: String, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
    set_sticky_geometry_person(&state, id, x, y, w, h)
}

pub(crate) fn set_sticky_geometry_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<()> {
    s.run_as(_actor, |s| {
    if ![x, y, w, h].iter().all(|v| v.is_finite()) {
        return Ok(());
    }

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
    let (st, protect) = open_sticky_person(&state, id.clone())?;
    ui::open_sticky(&app, &state.paths, &st, protect)?;
    Ok(())
}

pub(crate) fn open_sticky_core(s: &mut Session, _actor: &Actor, id: String) -> Result<(Sticky, bool)> {
    s.run_as(_actor, |s| {
    let mut st = sticky(s, &id)?;
    if st.trashed_at.is_some() { return Err(Error::NotFound); }
    st.open = true;
    s.put(Item::Sticky(st.clone()))?;
    Ok((st, s.protects_capture()))
    })
}


#[tauri::command]
pub fn close_sticky(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    close_sticky_person(&state, id.clone())?;
    ui::close_sticky(&app, &id);
    Ok(())
}

pub(crate) fn close_sticky_core(s: &mut Session, _actor: &Actor, id: String) -> Result<()> {
    s.run_as(_actor, |s| {
    let mut st = sticky(s, &id)?;
    st.open = false;
    s.put(Item::Sticky(st))
    })
}


/// Closes every open sticky note window at once ("Close all"), keeping the notes: each one is marked as not
/// open, so Open / Open all brings it back and it does not return by itself at the next unlock. Every window
/// writes its pending edits first (the same step a lock takes). Nothing is deleted.
pub fn hide_all(app: &AppHandle, state: &AppState) -> Result<()> {
    if state.is_unlocked() {
        crate::state::flush_windows(app);
    }
    hide_stickies_person(state)?;
    ui::close_stickies(app);
    Ok(())
}

pub(crate) fn hide_stickies_core(s: &mut Session, _actor: &Actor) -> Result<()> {
    s.run_as(_actor, |s| {
    let open: Vec<Sticky> = s
        .items()
        .filter_map(|i| match i {
            Item::Sticky(x) if x.open => Some(x.clone()),
            _ => None,
        })
        .collect();
    for mut st in open {
        st.open = false;
        s.put(Item::Sticky(st))?;
    }
    Ok(())
    })
}

// Async: it waits for the windows to save, which must not happen on the main thread.
#[tauri::command(async)]
pub fn hide_stickies(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    hide_all(&app, &state)
}

#[tauri::command(async)]
pub fn delete_sticky(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    if state.is_unlocked() { crate::state::flush_windows(&app); }
    delete_sticky_person(&state, id.clone())?;
    ui::close_sticky(&app, &id);
    Ok(())
}

pub(crate) fn delete_sticky_core(s: &mut Session, _actor: &Actor, id: String) -> Result<()> {
    s.run_as(_actor, |s| {
    let mut st = sticky(s, &id)?;
    st.trashed_at.get_or_insert_with(now_ms);
    st.open = false;
    s.put(Item::Sticky(st))
    })
}


/// Deletes several sticky notes in one vault write. Ids that no longer exist (deleted
/// in their own window meanwhile) are skipped; if any other id is not a sticky note,
/// nothing is removed.
#[tauri::command(async)]
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
    if state.is_unlocked() { crate::state::flush_windows(&app); }
    delete_stickies_person(&state, ids.clone())?;
    for id in &ids {
        ui::close_sticky(&app, id);
    }
    Ok(())
}

pub(crate) fn delete_stickies_core(s: &mut Session, _actor: &Actor, ids: Vec<String>) -> Result<()> {
    s.run_as(_actor, |s| {
    let mut ids = ids;
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return Ok(());
    }
    if ids.len() > 10_000 {
        return Err(Error::Invalid("Too many sticky notes at once.".into()));
    }

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
    let mut changed = Vec::new();
    for id in found { let mut st = sticky(s, &id)?; st.trashed_at.get_or_insert_with(now_ms); st.open = false; changed.push(Item::Sticky(st)); }
    s.put_many(changed)
    })
}


fn delete_stickies_person(state: &AppState, ids: Vec<String>) -> Result<()> {
    state.with_mut(|s| delete_stickies_core(s, &Actor::Person, ids))
}


fn delete_sticky_person(state: &AppState, id: String) -> Result<()> {
    state.with_mut(|s| delete_sticky_core(s, &Actor::Person, id))
}


fn close_sticky_person(state: &AppState, id: String) -> Result<()> {
    state.with_mut(|s| close_sticky_core(s, &Actor::Person, id))
}


fn open_sticky_person(state: &AppState, id: String) -> Result<(Sticky, bool)> {
    state.with_mut(|s| open_sticky_core(s, &Actor::Person, id))
}


fn set_sticky_geometry_person(state: &AppState, id: String, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
    if ![x, y, w, h].iter().all(|v| v.is_finite()) {
        return Ok(());
    }
    state.with_mut(|s| set_sticky_geometry_core(s, &Actor::Person, id.clone(), x, y, w, h))
}


fn set_sticky_style_person(state: &AppState, id: String, color: String, on_top: bool) -> Result<Sticky> {
    state.with_mut(|s| set_sticky_style_core(s, &Actor::Person, id, color, on_top))
}


fn save_sticky_person(state: &AppState, id: String, content: String, text: String) -> Result<()> {
    limit("Sticky note", &content, 2_000_000)?;
    limit("Sticky note", &text, 2_000_000)?;
    state.with_mut(|s| save_sticky_core(s, &Actor::Person, id.clone(), content, text))
}


fn get_sticky_person(state: &AppState, id: String) -> Result<Sticky> {
    state.with(|s| get_sticky_core(s, &Actor::Person, id.clone()))
}


fn create_sticky_person(state: &AppState, open: bool) -> Result<(Sticky, bool)> {
    state.with_mut(|s| create_sticky_core(s, &Actor::Person, open))
}


fn list_stickies_person(state: &AppState) -> Result<Vec<Sticky>> {
    state.with(|s| list_stickies_core(s, &Actor::Person))
}


fn hide_stickies_person(state: &AppState) -> Result<()> {
    state.with_mut(|s| hide_stickies_core(s, &Actor::Person))
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;
    #[test]
    fn delete_stickies_person_and_core_match() {
        compare(
            |state| delete_stickies_person(state, vec!["10000000000000000000000000000006".into()]),
            |s, _paths| delete_stickies_core(s, &Actor::Person, vec!["10000000000000000000000000000006".into()]),
        );
    }
    #[test]
    fn delete_sticky_person_and_core_match() {
        compare(
            |state| delete_sticky_person(state, "10000000000000000000000000000006".into()),
            |s, _paths| delete_sticky_core(s, &Actor::Person, "10000000000000000000000000000006".into()),
        );
    }
    #[test]
    fn close_sticky_person_and_core_match() {
        compare(
            |state| close_sticky_person(state, "10000000000000000000000000000006".into()),
            |s, _paths| close_sticky_core(s, &Actor::Person, "10000000000000000000000000000006".into()),
        );
    }
    #[test]
    fn open_sticky_person_and_core_match() {
        compare(
            |state| open_sticky_person(state, "10000000000000000000000000000006".into()),
            |s, _paths| open_sticky_core(s, &Actor::Person, "10000000000000000000000000000006".into()),
        );
    }
    #[test]
    fn set_sticky_geometry_person_and_core_match() {
        compare(
            |state| set_sticky_geometry_person(state, "10000000000000000000000000000006".into(), 150.0, 160.0, 350.0, 360.0),
            |s, _paths| set_sticky_geometry_core(s, &Actor::Person, "10000000000000000000000000000006".into(), 150.0, 160.0, 350.0, 360.0),
        );
    }
    #[test]
    fn set_sticky_style_person_and_core_match() {
        compare(
            |state| set_sticky_style_person(state, "10000000000000000000000000000006".into(), " #ffeeaa ".into(), true),
            |s, _paths| set_sticky_style_core(s, &Actor::Person, "10000000000000000000000000000006".into(), " #ffeeaa ".into(), true),
        );
    }
    #[test]
    fn save_sticky_person_and_core_match() {
        compare(
            |state| {
                save_sticky_person(
                    state,
                    "10000000000000000000000000000006".into(),
                    "{\"type\":\"doc\",\"content\":[]}".into(),
                    "\u{200f}exact\ttext  ".into(),
                )
            },
            |s, _paths| {
                save_sticky_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000006".into(),
                    "{\"type\":\"doc\",\"content\":[]}".into(),
                    "\u{200f}exact\ttext  ".into(),
                )
            },
        );
    }
    #[test]
    fn get_sticky_person_and_core_match() {
        compare(
            |state| get_sticky_person(state, "10000000000000000000000000000006".into()),
            |s, _paths| get_sticky_core(s, &Actor::Person, "10000000000000000000000000000006".into()),
        );
    }
    #[test]
    fn create_sticky_person_and_core_match() {
        compare(
            |state| create_sticky_person(state, true),
            |s, _paths| create_sticky_core(s, &Actor::Person, true),
        );
    }
    #[test]
    fn list_stickies_person_and_core_match() {
        compare(
            |state| list_stickies_person(state),
            |s, _paths| list_stickies_core(s, &Actor::Person),
        );
    }
    #[test]
    fn hide_stickies_person_and_core_match() {
        compare(
            |state| hide_stickies_person(state),
            |s, _paths| hide_stickies_core(s, &Actor::Person),
        );
    }
}
