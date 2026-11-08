use crate::actions::Actor;
use serde::Serialize;
use tauri::ipc::{InvokeBody, Request};
use tauri::State;

use super::database as dbc;
use super::{check_color, limit, notebook, page, section};
use crate::backup;
use crate::error::{Error, Result};
use crate::model::{new_id, now_ms, Item, Notebook, Page, PageMeta, Section};
use crate::state::AppState;
use crate::vault::Session;

const MAX_PAGE_CHARS: usize = 20_000_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesTree {
    notebooks: Vec<Notebook>,
    sections: Vec<Section>,
    pages: Vec<PageMeta>,
    databases: Vec<dbc::DatabaseMeta>,
}

/// A database row shown as a page (so the page editor opens it unchanged). Its `parent` is the
/// id of its database.
fn row_as_page(s: &Session, r: &crate::database::types::Row) -> Page {
    let section_id = match s.get(&r.db) {
        Some(Item::Database(d)) => d.section_id.clone(),
        _ => String::new(),
    };
    Page {
        source: None,
        id: r.id.clone(),
        section_id,
        title: r.title.clone(),
        content: r.content.clone(),
        text: r.text.clone(),
        order: r.order,
        favorite: r.favorite,
        icon: r.icon.clone(),
        cover: r.cover.clone(),
        parent: r.db.clone(),
        trashed_at: r.trashed_at,
        created: r.created,
        updated: r.updated,
    }
}

fn tree(s: &Session) -> NotesTree {
    let hidden = dbc::pages_under_rows(s);
    let mut notebooks = Vec::new();
    let mut sections = Vec::new();
    let mut pages = Vec::new();
    for item in s.items() {
        match item {
            Item::Notebook(n) if !s.in_bin(&n.id) => notebooks.push(n.clone()),
            Item::Section(x) if !s.in_bin(&x.id) => sections.push(x.clone()),
            // Keep directly binned page metadata for the existing read-only preview.
            // The rendered tree/Home filter trashedAt; live children of binned containers stay hidden.
            Item::Page(p) if !hidden.contains(&p.id) && (!s.in_bin(&p.id) || p.trashed_at.is_some()) => pages.push(PageMeta::from(p)),
            _ => {}
        }
    }
    notebooks.sort_by_key(|n| (n.order, n.created));
    sections.sort_by_key(|x| (x.order, x.created));
    pages.sort_by_key(|p| (p.order, p.created));
    NotesTree { notebooks, sections, pages, databases: dbc::meta_list(s) }
}

fn next_order(orders: impl Iterator<Item = i64>) -> i64 {
    orders.max().map_or(0, |m| m + 1)
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<NotesTree> {
    list_notes_person(&state)
}

fn list_notes_person(state: &AppState) -> Result<NotesTree> {
    state.with(|s| list_notes_core(s, &Actor::Person))
}

pub(crate) fn list_notes_core(s: &Session, _actor: &Actor) -> Result<NotesTree> {
    Ok(tree(s))
}


#[tauri::command]
pub fn create_notebook(state: State<'_, AppState>, name: String, color: String, icon: String) -> Result<NotesTree> {
    create_notebook_person(&state, name, color, icon)
}

fn create_notebook_person(state: &AppState, name: String, color: String, icon: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    state.with_mut(|s| create_notebook_core(s, &Actor::Person, name, color, icon))
}

pub(crate) fn create_notebook_core(
    s: &mut Session,
    _actor: &Actor,
    name: String,
    color: String,
    icon: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    let now = now_ms();
    let order = next_order(s.items().filter_map(|i| match i {
        Item::Notebook(n) => Some(n.order),
        _ => None,
    }));
    let nb = Notebook { source: None, id: new_id(), name, color, icon, order, created: now, updated: now, trashed_at: None, };
    let first_section = Section {
        source: None,
        id: new_id(),
        notebook_id: nb.id.clone(),
        name: "General".into(),
        color: String::new(),
        order: 0,
        created: now,
        updated: now,
    trashed_at: None, };
    s.put_many(vec![Item::Notebook(nb), Item::Section(first_section)])?;
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn update_notebook(state: State<'_, AppState>, id: String, name: String, color: String, icon: String) -> Result<NotesTree> {
    update_notebook_person(&state, id, name, color, icon)
}

fn update_notebook_person(
    state: &AppState,
    id: String,
    name: String,
    color: String,
    icon: String,
) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    state.with_mut(|s| update_notebook_core(s, &Actor::Person, id, name, color, icon))
}

pub(crate) fn update_notebook_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    name: String,
    color: String,
    icon: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    let mut nb = notebook(s, &id)?;
    nb.name = name;
    nb.color = color;
    nb.icon = icon;
    nb.updated = now_ms();
    s.put(Item::Notebook(nb))?;
    Ok(tree(s))
    })
}


/// Moves a notebook and its contents to the bin. A checked backup is made first;
/// if it fails, nothing is binned. Runs on a worker
/// thread because the backup can take a moment.
#[tauri::command(async)]
pub fn delete_notebook(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    delete_notebook_person(&state, id)
}

fn delete_notebook_person(state: &AppState, id: String) -> Result<NotesTree> {
    state.with_mut(|s| delete_notebook_core(s, &Actor::Person, &state.paths, id))
}

pub(crate) fn delete_notebook_core(
    s: &mut Session,
    _actor: &Actor,
    paths: &crate::portable::Paths,
    id: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    let mut item = notebook(s, &id)?;
    if item.trashed_at.is_none() {
        backup::make_safety_backup(paths, s).map_err(|e| backup::safety_failure(&e))?;
        item.trashed_at = Some(now_ms());
        s.put(Item::Notebook(item))?;
    }
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn create_section(state: State<'_, AppState>, notebook_id: String, name: String, color: String) -> Result<NotesTree> {
    create_section_person(&state, notebook_id, name, color)
}

fn create_section_person(state: &AppState, notebook_id: String, name: String, color: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    state.with_mut(|s| create_section_core(s, &Actor::Person, notebook_id, name, color))
}

pub(crate) fn create_section_core(
    s: &mut Session,
    _actor: &Actor,
    notebook_id: String,
    name: String,
    color: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    notebook(s, &notebook_id)?;
    let now = now_ms();
    let order = next_order(s.items().filter_map(|i| match i {
        Item::Section(x) if x.notebook_id == notebook_id => Some(x.order),
        _ => None,
    }));
    s.put(Item::Section(Section {
        source: None,
        id: new_id(),
        notebook_id,
        name,
        color,
        order,
        created: now,
        updated: now,
    trashed_at: None, }))?;
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn update_section(state: State<'_, AppState>, id: String, name: String, color: String) -> Result<NotesTree> {
    update_section_person(&state, id, name, color)
}

fn update_section_person(state: &AppState, id: String, name: String, color: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    state.with_mut(|s| update_section_core(s, &Actor::Person, id, name, color))
}

pub(crate) fn update_section_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    name: String,
    color: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    let mut x = section(s, &id)?;
    x.name = name;
    x.color = color;
    x.updated = now_ms();
    s.put(Item::Section(x))?;
    Ok(tree(s))
    })
}


/// Deletes a section with all of its pages, after a checked backup (see `delete_notebook`).
#[tauri::command(async)]
pub fn delete_section(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    delete_section_person(&state, id)
}

fn delete_section_person(state: &AppState, id: String) -> Result<NotesTree> {
    state.with_mut(|s| delete_section_core(s, &Actor::Person, &state.paths, id))
}

pub(crate) fn delete_section_core(
    s: &mut Session,
    _actor: &Actor,
    paths: &crate::portable::Paths,
    id: String,
) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    let mut item = section(s, &id)?;
    if item.trashed_at.is_none() {
        backup::make_safety_backup(paths, s).map_err(|e| backup::safety_failure(&e))?;
        item.trashed_at = Some(now_ms());
        s.put(Item::Section(item))?;
    }
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn create_page(state: State<'_, AppState>, section_id: String, title: String) -> Result<Page> {
    create_page_person(&state, section_id, title)
}

fn create_page_person(state: &AppState, section_id: String, title: String) -> Result<Page> {
    limit("Title", &title, 300)?;
    state.with_mut(|s| create_page_core(s, &Actor::Person, section_id, title))
}

pub(crate) fn create_page_core(s: &mut Session, _actor: &Actor, section_id: String, title: String) -> Result<Page> {
    s.run_as(_actor, |s| {
    limit("Title", &title, 300)?;
    section(s, &section_id)?;
    let now = now_ms();
    let order = next_order(s.items().filter_map(|i| match i {
        Item::Page(p) if p.section_id == section_id => Some(p.order),
        _ => None,
    }));
    let p = Page {
        source: None,
        id: new_id(),
        section_id,
        title,
        content: String::new(),
        text: String::new(),
        order,
        favorite: false,
        icon: String::new(),
        cover: String::new(),
        parent: String::new(),
        trashed_at: None,
        created: now,
        updated: now,
    };
    s.put(Item::Page(p.clone()))?;
    Ok(p)
    })
}


#[tauri::command]
pub fn get_page(state: State<'_, AppState>, id: String) -> Result<crate::changes::Revisioned<Page>> {
    state.with(|s| { let p = get_page_core(s, &Actor::Person, id.clone())?; Ok(s.revisioned(&id, p)) })
}

fn get_page_person(state: &AppState, id: String) -> Result<Page> {
    state.with(|s| get_page_core(s, &Actor::Person, id))
}

pub(crate) fn get_page_core(s: &Session, _actor: &Actor, id: String) -> Result<Page> {
    match s.get(&id) {
        Some(Item::Row(r)) => Ok(row_as_page(s, r)),
        _ => page(s, &id),
    }
}


#[tauri::command]
pub fn save_page(state: State<'_, AppState>, id: String, title: String, content: String, text: String, base_rev: Option<u64>) -> Result<crate::changes::Revisioned<PageMeta>> {
    state.with_mut(|s| s.person_save(&id.clone(), base_rev, |s| {
        let meta = save_page_core(s, &Actor::Person, id.clone(), title, content, text)?;
        Ok(s.revisioned(&id, meta))
    }))
}

fn save_page_person(state: &AppState, id: String, title: String, content: String, text: String) -> Result<PageMeta> {
    limit("Title", &title, 300)?;
    limit("Page", &content, MAX_PAGE_CHARS)?;
    limit("Page", &text, MAX_PAGE_CHARS)?;
    state.with_mut(|s| save_page_core(s, &Actor::Person, id, title, content, text))
}

pub(crate) fn save_page_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    title: String,
    content: String,
    text: String,
) -> Result<PageMeta> {
    s.run_as(_actor, |s| {
    limit("Title", &title, 300)?;
    limit("Page", &content, MAX_PAGE_CHARS)?;
    limit("Page", &text, MAX_PAGE_CHARS)?;
    // A delayed person save still lands after a bin move, preserving pending typing.
    if matches!(_actor, Actor::Ai { .. }) && s.in_bin(&id) { return Err(Error::invalid("This item is in the bin. Restore it before editing.")); }
    if let Some(Item::Row(r)) = s.get(&id) {
        let mut r = r.clone();
        r.title = title;
        r.content = content;
        r.text = text;
        r.updated = now_ms();
        let meta = PageMeta::from(&row_as_page(s, &r));
        s.put(Item::Row(r))?;
        return Ok(meta);
    }
    let mut p = page(s, &id)?;
    p.title = title;
    p.content = content;
    p.text = text;
    p.updated = now_ms();
    let meta = PageMeta::from(&p);
    s.put(Item::Page(p))?;
    Ok(meta)
    })
}


/// The page itself plus every page nested inside it, at any depth.
fn with_descendants(s: &crate::vault::Session, id: &str) -> Vec<String> {
    let mut out = vec![id.to_string()];
    let mut i = 0;
    while i < out.len() {
        let current = out[i].clone();
        for item in s.items() {
            if let Item::Page(p) = item {
                if p.parent == current && !out.contains(&p.id) {
                    out.push(p.id.clone());
                }
            }
        }
        i += 1;
    }
    out
}

/// Creates a page inside another page. It lives in the same section.
fn add_subpage(s: &mut Session, parent_id: &str, title: String) -> Result<Page> {
    let parent = match s.get(parent_id) {
        Some(Item::Row(r)) => row_as_page(s, r),
        _ => page(s, parent_id)?,
    };
    let now = now_ms();
    let order = next_order(s.items().filter_map(|i| match i {
        Item::Page(p) if p.parent == parent_id => Some(p.order),
        _ => None,
    }));
    let p = Page {
        source: None,
        id: new_id(),
        section_id: parent.section_id.clone(),
        title,
        content: String::new(),
        text: String::new(),
        order,
        favorite: false,
        icon: String::new(),
        cover: String::new(),
        parent: parent_id.to_string(),
        trashed_at: parent.trashed_at,
        created: now,
        updated: now,
    };
    s.put(Item::Page(p.clone()))?;
    Ok(p)
}

#[tauri::command]
pub fn create_subpage(state: State<'_, AppState>, parent_id: String, title: String) -> Result<Page> {
    create_subpage_person(&state, parent_id, title)
}

fn create_subpage_person(state: &AppState, parent_id: String, title: String) -> Result<Page> {
    limit("Title", &title, 300)?;
    state.with_mut(|s| create_subpage_core(s, &Actor::Person, parent_id, title))
}

pub(crate) fn create_subpage_core(s: &mut Session, _actor: &Actor, parent_id: String, title: String) -> Result<Page> {
    s.run_as(_actor, |s| {
    limit("Title", &title, 300)?;
    add_subpage(s, &parent_id, title)
    })
}


#[tauri::command]
pub fn set_page_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> Result<NotesTree> {
    set_page_favorite_person(&state, id, favorite)
}

fn set_page_favorite_person(state: &AppState, id: String, favorite: bool) -> Result<NotesTree> {
    state.with_mut(|s| set_page_favorite_core(s, &Actor::Person, id, favorite))
}

pub(crate) fn set_page_favorite_core(s: &mut Session, _actor: &Actor, id: String, favorite: bool) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    match s.get(&id) {
        Some(Item::Row(r)) => {
            let mut r = r.clone();
            r.favorite = favorite;
            s.put(Item::Row(r))?;
        }
        Some(Item::Database(d)) => {
            let mut d = d.clone();
            d.favorite = favorite && d.trashed_at.is_none();
            s.put(Item::Database(d))?;
        }
        _ => {
            let mut p = page(s, &id)?;
            p.favorite = favorite;
            s.put(Item::Page(p))?;
        }
    }
    Ok(tree(s))
    })
}


/// Moves a page, and the pages inside it, to another section. The page leaves its old parent.
fn move_subtree(s: &mut Session, id: &str, section_id: &str) -> Result<()> {
    section(s, section_id)?;
    let mut p = page(s, id)?;
    p.order = next_order(s.items().filter_map(|i| match i {
        Item::Page(x) if x.section_id == section_id => Some(x.order),
        _ => None,
    }));
    p.section_id = section_id.to_string();
    p.parent = String::new();
    let mut changed = vec![];
    for child in with_descendants(s, id).into_iter().filter(|c| c != id) {
        let mut c = page(s, &child)?;
        c.section_id = section_id.to_string();
        changed.push(Item::Page(c));
    }
    changed.push(Item::Page(p));
    s.put_many(changed)
}

#[tauri::command]
pub fn move_page(state: State<'_, AppState>, id: String, section_id: String) -> Result<NotesTree> {
    move_page_person(&state, id, section_id)
}

fn move_page_person(state: &AppState, id: String, section_id: String) -> Result<NotesTree> {
    state.with_mut(|s| move_page_core(s, &Actor::Person, id, section_id))
}

pub(crate) fn move_page_core(s: &mut Session, _actor: &Actor, id: String, section_id: String) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    if matches!(s.get(&id), Some(Item::Database(_))) {
        section(s, &section_id)?;
        let mut d = dbc::database(s, &id)?;
        d.section_id = section_id;
        d.parent = String::new();
        d.updated = now_ms();
        s.put(Item::Database(d))?;
    } else {
        move_subtree(s, &id, &section_id)?;
    }
    Ok(tree(s))
    })
}


/// Sets the display order of notebooks, sections or pages to the order given.
#[tauri::command]
pub fn reorder(state: State<'_, AppState>, ids: Vec<String>) -> Result<NotesTree> {
    reorder_person(&state, ids)
}

fn reorder_person(state: &AppState, ids: Vec<String>) -> Result<NotesTree> {
    state.with_mut(|s| reorder_core(s, &Actor::Person, ids))
}

pub(crate) fn reorder_core(s: &mut Session, _actor: &Actor, ids: Vec<String>) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    let mut changed = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let order = i as i64;
        match s.get(id).cloned() {
            Some(Item::Notebook(mut n)) => {
                n.order = order;
                changed.push(Item::Notebook(n));
            }
            Some(Item::Section(mut x)) => {
                x.order = order;
                changed.push(Item::Section(x));
            }
            Some(Item::Page(mut p)) => {
                p.order = order;
                changed.push(Item::Page(p));
            }
            Some(Item::Database(mut d)) => {
                d.order = order;
                changed.push(Item::Database(d));
            }
            Some(Item::Row(mut r)) => {
                r.order = order;
                changed.push(Item::Row(r));
            }
            _ => return Err(Error::NotFound),
        }
    }
    s.put_many(changed)?;
    Ok(tree(s))
    })
}


/// Puts a page, and every page inside it, in the trash.
fn trash_subtree(s: &mut Session, id: &str) -> Result<()> {
    page(s, id)?;
    let now = now_ms();
    let mut changed = vec![];
    for pid in with_descendants(s, id) {
        let mut p = page(s, &pid)?;
        if p.trashed_at.is_none() {
            p.trashed_at = Some(now);
        }
        p.favorite = false;
        changed.push(Item::Page(p));
    }
    s.put_many(changed)
}

/// Brings a page back with the pages inside it. If its parent is still in the trash, it returns to the top of its section.
fn restore_subtree(s: &mut Session, id: &str) -> Result<()> {
    let first = page(s, id)?;
    let parent_trashed = !first.parent.is_empty() && matches!(s.get(&first.parent), Some(Item::Page(pp)) if pp.trashed_at.is_some());
    let mut changed = vec![];
    for pid in with_descendants(s, id) {
        let mut p = page(s, &pid)?;
        p.trashed_at = None;
        if pid == id && parent_trashed {
            p.parent = String::new();
        }
        changed.push(Item::Page(p));
    }
    s.put_many(changed)
}

#[tauri::command]
pub fn trash_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    trash_page_person(&state, id)
}

fn trash_page_person(state: &AppState, id: String) -> Result<NotesTree> {
    state.with_mut(|s| trash_page_core(s, &Actor::Person, id))
}

pub(crate) fn trash_page_core(s: &mut Session, _actor: &Actor, id: String) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    match s.get(&id) {
        Some(Item::Row(_)) => dbc::trash_rows(s, &[id.clone()])?,
        Some(Item::Database(_)) => dbc::trash_database(s, &id)?,
        _ => trash_subtree(s, &id)?,
    }
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn restore_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    restore_page_person(&state, id)
}

fn restore_page_person(state: &AppState, id: String) -> Result<NotesTree> {
    state.with_mut(|s| restore_page_core(s, &Actor::Person, id))
}

pub(crate) fn restore_page_core(s: &mut Session, _actor: &Actor, id: String) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    restore_parents(s, &id, false)?;
    match s.get(&id) {
        Some(Item::Notebook(n)) => { let mut n = n.clone(); n.trashed_at = None; s.put(Item::Notebook(n))?; },
        Some(Item::Section(x)) => { let mut x = x.clone(); x.trashed_at = None; s.put(Item::Section(x))?; },
        Some(Item::Sticky(x)) => { let mut x = x.clone(); x.trashed_at = None; x.open = false; s.put(Item::Sticky(x))?; },
        Some(Item::Row(_)) => dbc::restore_rows(s, &[id.clone()])?,
        Some(Item::Database(_)) => dbc::restore_database(s, &id)?,
        _ => restore_subtree(s, &id)?,
    }
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn delete_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    delete_page_person(&state, id)
}

fn delete_page_person(state: &AppState, id: String) -> Result<NotesTree> {
    state.with_mut(|s| delete_page_core(s, &Actor::Person, id))
}

pub(crate) fn delete_page_core(s: &mut Session, _actor: &Actor, id: String) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    delete_bin_tree(s, &id)?;
    Ok(tree(s))
    })
}


#[tauri::command]
pub fn empty_trash(state: State<'_, AppState>) -> Result<NotesTree> {
    empty_trash_person(&state)
}

fn empty_trash_person(state: &AppState) -> Result<NotesTree> {
    state.with_mut(|s| empty_trash_core(s, &Actor::Person))
}

pub(crate) fn empty_trash_core(s: &mut Session, _actor: &Actor) -> Result<NotesTree> {
    s.run_as(_actor, |s| {
    let roots: Vec<String> = s.items().filter(|i| Session::own_trash(i).is_some()).map(|i| i.id().into()).collect();
    for id in roots { if s.get(&id).is_some() { delete_bin_tree(s, &id)?; } }
    Ok(tree(s))
    })
}


/// Identifies an image from its first bytes; the name the UI sends is ignored.
/// SVG is refused on purpose because it can contain scripts.
pub(crate) fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Stores a pasted or dropped image (raw bytes) encrypted and returns its id.
#[tauri::command(async)]
pub fn add_attachment(state: State<'_, AppState>, request: Request<'_>) -> Result<String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(Error::invalid("Expected image data."));
    };
    let _mime = sniff_image(bytes)
        .ok_or_else(|| Error::invalid("Only PNG, JPEG, GIF and WebP images are supported."))?;
    add_attachment_person(&state, bytes)
}

pub(crate) fn add_attachment_core(s: &mut Session, _actor: &Actor, bytes: &[u8]) -> Result<String> {
    s.run_as(_actor, |s| {
    let mime =
        sniff_image(bytes).ok_or_else(|| Error::invalid("Only PNG, JPEG, GIF and WebP images are supported."))?;
    s.put_attachment(mime, bytes)
    })
}

#[cfg(test)]
mod tests {
    use super::sniff_image;

    #[test]
    fn sniffing() {
        assert_eq!(sniff_image(b"\x89PNG\r\n\x1a\nrest"), Some("image/png"));
        assert_eq!(sniff_image(b"\xff\xd8\xff\xe0"), Some("image/jpeg"));
        assert_eq!(sniff_image(b"<svg onload=alert(1)>"), None);
        assert_eq!(sniff_image(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
    }
}

/// The biggest picture that may be a page icon (an icon is shown at a few dozen pixels).
const MAX_ICON_PICTURE_BYTES: usize = 2 * 1024 * 1024;

/// The 32 hex characters of a vault picture in `http://clatt.localhost/<id>` (also the `https` and
/// `clatt://` spellings), or None when the text is anything else (an outside address included).
fn cover_picture_id(url: &str) -> Option<&str> {
    let rest = ["http://clatt.localhost/", "https://clatt.localhost/", "clatt://localhost/", "clatt://"]
        .iter()
        .find_map(|p| url.strip_prefix(p))?;
    (rest.len() == 32 && rest.bytes().all(|b| b.is_ascii_hexdigit())).then_some(rest)
}

/// A cover is empty, `gradient:<name>`, `solid:#rrggbb`, or a picture in this vault, optionally
/// followed by `#y=<0 to 100>` (how far down the picture is shown, in percent).
/// Returns the picture's id when there is one. Never accepts an outside address.
fn check_cover(cover: &str) -> Result<Option<String>> {
    let bad = || Error::invalid("This cover is not allowed. Use a gradient, a colour or a picture from your notes.");
    if cover.is_empty() {
        return Ok(None);
    }
    if let Some(name) = cover.strip_prefix("gradient:") {
        let ok = !name.is_empty() && name.len() <= 24 && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
        return if ok { Ok(None) } else { Err(bad()) };
    }
    if let Some(hex) = cover.strip_prefix("solid:#") {
        let ok = hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit());
        return if ok { Ok(None) } else { Err(bad()) };
    }
    let (url, y) = match cover.split_once("#y=") {
        Some((u, y)) => (u, Some(y)),
        None => (cover, None),
    };
    if let Some(y) = y {
        let ok = !y.is_empty() && y.len() <= 3 && y.bytes().all(|b| b.is_ascii_digit()) && y.parse::<u32>().map_or(false, |n| n <= 100);
        if !ok {
            return Err(bad());
        }
    }
    cover_picture_id(url).map(|id| Some(id.to_ascii_lowercase())).ok_or_else(bad)
}

/// An icon is an emoji or a short text (at most 40 characters), or `pic:<32 hex>`: a small picture
/// in this vault. Pictures are not allowed for database rows (their views only show text).
fn check_icon(icon: &str, allow_picture: bool) -> Result<Option<String>> {
    if let Some(id) = icon.strip_prefix("pic:") {
        if !allow_picture || id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::invalid("This icon is not allowed."));
        }
        return Ok(Some(id.to_ascii_lowercase()));
    }
    limit("Icon", icon, 40)?;
    Ok(None)
}

/// Sets the icon and cover shown on a page or a database row. A cover is `gradient:<name>`,
/// `solid:#rrggbb` or one of this vault's pictures (so it is encrypted like any other), never an
/// outside address. A picture icon is `pic:<id>`, a vault picture of at most 2 MB.
fn apply_page_look(s: &mut Session, id: &str, icon: String, cover: String) -> Result<PageMeta> {
    limit("Cover", &cover, 200)?;
    let is_row = matches!(s.get(id), Some(Item::Row(_)));
    let (old_icon, old_cover) = match s.get(id) {
        Some(Item::Row(r)) => (r.icon.clone(), r.cover.clone()),
        Some(Item::Page(p)) => (p.icon.clone(), p.cover.clone()),
        _ => return Err(Error::NotFound),
    };
    // What the page already has stays as it is, so an old value never blocks changing the other one.
    let icon_pic = if icon == old_icon { None } else { check_icon(&icon, !is_row)? };
    let cover_pic = if cover == old_cover { None } else { check_cover(&cover)? };
    // The picture is only read when it is new for this page (moving a cover up or down keeps it).
    if let Some(pic) = icon_pic {
        let (mime, bytes) = s.get_attachment(&pic).map_err(|_| Error::invalid("This picture is not in your notes."))?;
        if !matches!(mime.as_str(), "image/png" | "image/jpeg" | "image/gif" | "image/webp") {
            return Err(Error::invalid("An icon must be a PNG, JPEG, GIF or WebP picture."));
        }
        if bytes.len() > MAX_ICON_PICTURE_BYTES {
            return Err(Error::invalid("An icon picture must be smaller than 2 MB."));
        }
    }
    if let Some(pic) = cover_pic.filter(|pic| !old_cover.to_ascii_lowercase().contains(pic.as_str())) {
        let (mime, _) = s.get_attachment(&pic).map_err(|_| Error::invalid("This picture is not in your notes."))?;
        if !matches!(mime.as_str(), "image/png" | "image/jpeg" | "image/gif" | "image/webp") {
            return Err(Error::invalid("A cover must be a PNG, JPEG, GIF or WebP picture."));
        }
    }
    if let Some(Item::Row(r)) = s.get(id) {
        let mut r = r.clone();
        r.icon = icon;
        r.cover = cover;
        r.updated = now_ms();
        let meta = PageMeta::from(&row_as_page(s, &r));
        s.put(Item::Row(r))?;
        return Ok(meta);
    }
    let mut p = page(s, id)?;
    p.icon = icon;
    p.cover = cover;
    p.updated = now_ms();
    let meta = PageMeta::from(&p);
    s.put(Item::Page(p))?;
    Ok(meta)
}

#[tauri::command]
pub fn set_page_look(state: State<'_, AppState>, id: String, icon: String, cover: String) -> Result<PageMeta> {
    set_page_look_person(&state, id, icon, cover)
}

fn set_page_look_person(state: &AppState, id: String, icon: String, cover: String) -> Result<PageMeta> {
    limit("Icon", &icon, 40)?;
    limit("Cover", &cover, 200)?;
    state.with_mut(|s| set_page_look_core(s, &Actor::Person, id, icon, cover))
}

pub(crate) fn set_page_look_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    icon: String,
    cover: String,
) -> Result<PageMeta> {
    s.run_as(_actor, |s| {
    limit("Icon", &icon, 40)?;
    limit("Cover", &cover, 200)?;
    apply_page_look(s, &id, icon, cover)
    })
}


#[cfg(test)]
mod subpage_tests {
    use super::*;
    use crate::crypto::KdfParams;

    fn vault() -> Session {
        let dir = std::env::temp_dir().join(format!("notaya-test-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        Session::create(&dir.join("vault.nota"), "pw", None, KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap()
    }

    fn page_in(s: &mut Session, section_id: &str, title: &str) -> Page {
        let p = Page {
            source: None,
            id: new_id(),
            section_id: section_id.into(),
            title: title.into(),
            content: String::new(),
            text: String::new(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            created: 1,
            updated: 1,
        };
        s.put(Item::Page(p.clone())).unwrap();
        p
    }

    fn get(s: &Session, id: &str) -> Page {
        page(s, id).unwrap()
    }

    #[test]
    fn pages_nest_and_follow_their_parent() {
        let mut s = vault();
        let sec_a = new_id();
        let sec_b = new_id();
        s.put(Item::Section(Section { source: None, id: sec_a.clone(), notebook_id: new_id(), name: "A".into(), color: String::new(), order: 0, created: 1, updated: 1, trashed_at: None, })).unwrap();
        s.put(Item::Section(Section { source: None, id: sec_b.clone(), notebook_id: new_id(), name: "B".into(), color: String::new(), order: 0, created: 1, updated: 1, trashed_at: None, })).unwrap();
        let top = page_in(&mut s, &sec_a, "top");
        let child = add_subpage(&mut s, &top.id, "child".into()).unwrap();
        let grandchild = add_subpage(&mut s, &child.id, "grandchild".into()).unwrap();
        assert_eq!(get(&s, &grandchild.id).section_id, sec_a);
        assert_eq!(with_descendants(&s, &top.id).len(), 3);

        // Trashing the top page trashes everything inside it; restoring the child alone moves it out.
        trash_subtree(&mut s, &top.id).unwrap();
        assert!(get(&s, &grandchild.id).trashed_at.is_some());
        restore_subtree(&mut s, &child.id).unwrap();
        assert!(get(&s, &child.id).trashed_at.is_none());
        assert!(get(&s, &grandchild.id).trashed_at.is_none());
        assert_eq!(get(&s, &child.id).parent, "");
        assert!(get(&s, &top.id).trashed_at.is_some());

        // Moving a page takes the pages inside it along.
        move_subtree(&mut s, &child.id, &sec_b).unwrap();
        assert_eq!(get(&s, &child.id).section_id, sec_b);
        assert_eq!(get(&s, &grandchild.id).section_id, sec_b);
        assert_eq!(get(&s, &grandchild.id).parent, child.id);
    }
}

#[cfg(test)]
mod look_tests {
    use super::*;
    use crate::crypto::KdfParams;

    fn vault() -> Session {
        let dir = std::env::temp_dir().join(format!("notaya-test-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        Session::create(&dir.join("vault.nota"), "pw", None, KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap()
    }

    fn new_page(s: &mut Session) -> String {
        let id = new_id();
        s.put(Item::Page(Page {
            source: None,
            id: id.clone(),
            section_id: new_id(),
            title: "t".into(),
            content: String::new(),
            text: String::new(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            created: 1,
            updated: 1,
        }))
        .unwrap();
        id
    }

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest";

    #[test]
    fn covers_are_gradients_colours_or_vault_pictures_only() {
        let id = "0123456789abcdef0123456789abcdef";
        for ok in [
            "",
            "gradient:dusk",
            "gradient:aurora-2",
            "solid:#a1B2c3",
            &format!("http://clatt.localhost/{id}"),
            &format!("http://clatt.localhost/{id}#y=0"),
            &format!("http://clatt.localhost/{id}#y=100"),
            &format!("clatt://localhost/{id}#y=35"),
        ] {
            assert!(check_cover(ok).is_ok(), "should be allowed: {ok}");
        }
        for bad in [
            "https://example.com/a.png",
            "http://example.com/clatt.localhost/0123456789abcdef0123456789abcdef",
            "data:image/png;base64,AAAA",
            "file:///C:/secret.png",
            "//evil.example/x.png",
            "gradient:",
            "gradient:a b",
            "gradient:../x",
            "solid:#12345",
            "solid:red",
            "solid:#12345g",
            &format!("http://clatt.localhost/{id}#y=101"),
            &format!("http://clatt.localhost/{id}#y=-1"),
            &format!("http://clatt.localhost/{id}#y="),
            &format!("http://clatt.localhost/{id}?x=1"),
            &format!("http://clatt.localhost/{id}/x"),
            "http://clatt.localhost/short",
        ] {
            assert!(check_cover(bad).is_err(), "should be refused: {bad}");
        }
    }

    #[test]
    fn a_page_look_keeps_only_allowed_values() {
        let mut s = vault();
        let pid = new_page(&mut s);
        let pic = s.put_attachment("image/png", PNG).unwrap();
        let url = format!("http://clatt.localhost/{pic}");

        // Outside addresses and pictures that are not in the vault are refused; nothing changes.
        assert!(apply_page_look(&mut s, &pid, "".into(), "https://example.com/x.png".into()).is_err());
        assert!(apply_page_look(&mut s, &pid, "".into(), "http://clatt.localhost/00000000000000000000000000000000".into()).is_err());
        assert!(apply_page_look(&mut s, &pid, "pic:00000000000000000000000000000000".into(), "".into()).is_err());
        assert!(apply_page_look(&mut s, &pid, "".into(), format!("{url}#y=250")).is_err());
        assert_eq!(page(&s, &pid).unwrap().cover, "");

        // An icon and a gradient cover.
        let m = apply_page_look(&mut s, &pid, "🚀".into(), "gradient:dusk".into()).unwrap();
        assert_eq!((m.icon.as_str(), m.cover.as_str()), ("🚀", "gradient:dusk"));
        assert!(apply_page_look(&mut s, &pid, "x".repeat(41), "".into()).is_err());

        // A picture cover with a vertical position, then moved, then removed.
        let m = apply_page_look(&mut s, &pid, "🚀".into(), format!("{url}#y=30")).unwrap();
        assert_eq!(m.cover, format!("{url}#y=30"));
        apply_page_look(&mut s, &pid, "🚀".into(), format!("{url}#y=80")).unwrap();
        assert_eq!(page(&s, &pid).unwrap().cover, format!("{url}#y=80"));
        // The picture stays alive while the cover points at it, and goes when nothing does.
        s.collect_garbage().unwrap();
        assert!(s.get_attachment(&pic).is_ok());
        apply_page_look(&mut s, &pid, "🚀".into(), "solid:#112233".into()).unwrap();
        s.collect_garbage().unwrap();
        assert!(s.get_attachment(&pic).is_err());
    }

    #[test]
    fn a_picture_icon_must_be_a_small_vault_picture_and_survives_cleanup() {
        let mut s = vault();
        let pid = new_page(&mut s);
        let pic = s.put_attachment("image/png", PNG).unwrap();
        let m = apply_page_look(&mut s, &pid, format!("pic:{pic}"), "".into()).unwrap();
        assert_eq!(m.icon, format!("pic:{pic}"));
        s.collect_garbage().unwrap();
        assert!(s.get_attachment(&pic).is_ok(), "the icon keeps its picture");

        // Too big, and not a picture at all.
        let mut big = PNG.to_vec();
        big.resize(MAX_ICON_PICTURE_BYTES + 1, 0);
        let big_id = s.put_attachment("image/png", &big).unwrap();
        assert!(apply_page_look(&mut s, &pid, format!("pic:{big_id}"), "".into()).is_err());
        let file_id = s.put_file("application/pdf", b"%PDF").unwrap();
        assert!(apply_page_look(&mut s, &pid, format!("pic:{file_id}"), "".into()).is_err());
        assert!(apply_page_look(&mut s, &pid, "".into(), format!("http://clatt.localhost/{file_id}")).is_err());
        assert_eq!(page(&s, &pid).unwrap().icon, format!("pic:{pic}"));

        // Picture icons are for pages only.
        assert!(check_icon(&format!("pic:{pic}"), false).is_err());
        assert!(check_icon("🚀", false).is_ok());
    }

    #[test]
    fn an_old_value_does_not_block_changing_the_other_one() {
        let mut s = vault();
        let pid = new_page(&mut s);
        // A cover in a form this version would not accept (for example from an older version).
        let mut p = page(&s, &pid).unwrap();
        p.cover = "http://old.example/legacy.png".into();
        s.put(Item::Page(p)).unwrap();
        let m = apply_page_look(&mut s, &pid, "📌".into(), "http://old.example/legacy.png".into()).unwrap();
        assert_eq!(m.icon, "📌");
        // But a new outside cover is still refused.
        assert!(apply_page_look(&mut s, &pid, "📌".into(), "http://other.example/a.png".into()).is_err());
    }

    #[test]
    fn the_tree_entry_carries_the_cover() {
        let mut s = vault();
        let pid = new_page(&mut s);
        let m = apply_page_look(&mut s, &pid, "".into(), "solid:#445566".into()).unwrap();
        assert_eq!(m.cover, "solid:#445566");
        let json = serde_json::to_value(&m).unwrap();
        assert_eq!(json["cover"], "solid:#445566");
    }
}

fn add_attachment_person(state: &AppState, bytes: &[u8]) -> Result<String> {
    state.with_mut(|s| add_attachment_core(s, &Actor::Person, bytes))
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;
    #[test]
    fn set_page_look_person_and_core_match() {
        compare(
            |state| set_page_look_person(state, "10000000000000000000000000000004".into(), "x".into(), "gradient:sunrise".into()),
            |s, _paths| set_page_look_core(s, &Actor::Person, "10000000000000000000000000000004".into(), "x".into(), "gradient:sunrise".into()),
        );
    }
    #[test]
    fn empty_trash_person_and_core_match() {
        compare(
            |state| empty_trash_person(state),
            |s, _paths| empty_trash_core(s, &Actor::Person),
        );
    }
    #[test]
    fn delete_page_person_and_core_match() {
        compare(
            |state| delete_page_person(state, "10000000000000000000000000000004".into()),
            |s, _paths| delete_page_core(s, &Actor::Person, "10000000000000000000000000000004".into()),
        );
    }
    #[test]
    fn restore_page_person_and_core_match() {
        compare(
            |state| restore_page_person(state, "10000000000000000000000000000005".into()),
            |s, _paths| restore_page_core(s, &Actor::Person, "10000000000000000000000000000005".into()),
        );
    }
    #[test]
    fn trash_page_person_and_core_match() {
        compare(
            |state| trash_page_person(state, "10000000000000000000000000000004".into()),
            |s, _paths| trash_page_core(s, &Actor::Person, "10000000000000000000000000000004".into()),
        );
    }
    #[test]
    fn reorder_person_and_core_match() {
        compare(
            |state| reorder_person(state, vec!["10000000000000000000000000000004".into()]),
            |s, _paths| reorder_core(s, &Actor::Person, vec!["10000000000000000000000000000004".into()]),
        );
    }
    #[test]
    fn move_page_person_and_core_match() {
        compare(
            |state| move_page_person(state, "10000000000000000000000000000004".into(), "10000000000000000000000000000003".into()),
            |s, _paths| move_page_core(s, &Actor::Person, "10000000000000000000000000000004".into(), "10000000000000000000000000000003".into()),
        );
    }
    #[test]
    fn set_page_favorite_person_and_core_match() {
        compare(
            |state| set_page_favorite_person(state, "10000000000000000000000000000004".into(), true),
            |s, _paths| set_page_favorite_core(s, &Actor::Person, "10000000000000000000000000000004".into(), true),
        );
    }
    #[test]
    fn create_subpage_person_and_core_match() {
        compare(
            |state| {
                create_subpage_person(
                    state,
                    "10000000000000000000000000000004".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                )
            },
            |s, _paths| {
                create_subpage_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000004".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                )
            },
        );
    }
    #[test]
    fn save_page_person_and_core_match() {
        compare(
            |state| {
                save_page_person(
                    state,
                    "10000000000000000000000000000004".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "{\"type\":\"doc\",\"content\":[]}".into(),
                    "\u{200f}exact\ttext  ".into(),
                )
            },
            |s, _paths| {
                save_page_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000004".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "{\"type\":\"doc\",\"content\":[]}".into(),
                    "\u{200f}exact\ttext  ".into(),
                )
            },
        );
    }
    #[test]
    fn get_page_person_and_core_match() {
        compare(
            |state| get_page_person(state, "10000000000000000000000000000004".into()),
            |s, _paths| get_page_core(s, &Actor::Person, "10000000000000000000000000000004".into()),
        );
    }
    #[test]
    fn create_page_person_and_core_match() {
        compare(
            |state| {
                create_page_person(
                    state,
                    "10000000000000000000000000000003".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                )
            },
            |s, _paths| {
                create_page_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000003".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                )
            },
        );
    }
    #[test]
    fn delete_section_person_and_core_match() {
        compare(
            |state| delete_section_person(state, "10000000000000000000000000000002".into()),
            |s, paths| delete_section_core(s, &Actor::Person, paths, "10000000000000000000000000000002".into()),
        );
    }
    #[test]
    fn update_section_person_and_core_match() {
        compare(
            |state| update_section_person(state, "10000000000000000000000000000002".into(), "Changed".into(), " #ffeeaa ".into()),
            |s, _paths| update_section_core(s, &Actor::Person, "10000000000000000000000000000002".into(), "Changed".into(), " #ffeeaa ".into()),
        );
    }
    #[test]
    fn create_section_person_and_core_match() {
        compare(
            |state| create_section_person(state, "10000000000000000000000000000001".into(), "Changed".into(), " #ffeeaa ".into()),
            |s, _paths| create_section_core(s, &Actor::Person, "10000000000000000000000000000001".into(), "Changed".into(), " #ffeeaa ".into()),
        );
    }
    #[test]
    fn delete_notebook_person_and_core_match() {
        compare(
            |state| delete_notebook_person(state, "10000000000000000000000000000001".into()),
            |s, paths| delete_notebook_core(s, &Actor::Person, paths, "10000000000000000000000000000001".into()),
        );
    }
    #[test]
    fn update_notebook_person_and_core_match() {
        compare(
            |state| update_notebook_person(state, "10000000000000000000000000000001".into(), "Changed".into(), " #ffeeaa ".into(), "x".into()),
            |s, _paths| {
                update_notebook_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000001".into(),
                    "Changed".into(),
                    " #ffeeaa ".into(),
                    "x".into(),
                )
            },
        );
    }
    #[test]
    fn create_notebook_person_and_core_match() {
        compare(
            |state| create_notebook_person(state, "Changed".into(), " #ffeeaa ".into(), "x".into()),
            |s, _paths| create_notebook_core(s, &Actor::Person, "Changed".into(), " #ffeeaa ".into(), "x".into()),
        );
    }
    #[test]
    fn list_notes_person_and_core_match() {
        compare(
            |state| list_notes_person(state),
            |s, _paths| list_notes_core(s, &Actor::Person),
        );
    }
    #[test]
    fn add_attachment_person_and_core_match() {
        compare(
            |state| add_attachment_person(state, b"\x89PNG\r\n\x1a\nthrowaway"),
            |s, _paths| add_attachment_core(s, &Actor::Person, b"\x89PNG\r\n\x1a\nthrowaway"),
        );
    }
}

#[cfg(test)]
mod action_boundary_tests {
    use super::*;

    #[test]
    fn ai_actor_reuses_the_same_page_checks_and_exact_text() {
        crate::actions::tests::compare(
            |state| {
                save_page_person(
                    state,
                    "10000000000000000000000000000004".into(),
                    "title  ".into(),
                    "content\t  ".into(),
                    "\u{200f}text  ".into(),
                )
            },
            |s, _| {
                save_page_core(
                    s,
                    &Actor::Ai {
                        pairing_id: "test".into(),
                        label: "Codex".into(),
                    },
                    "10000000000000000000000000000004".into(),
                    "title  ".into(),
                    "content\t  ".into(),
                    "\u{200f}text  ".into(),
                )
            },
        );
    }

    #[test]
    fn invalid_inputs_still_fail_before_locked_state() {
        let root = std::env::var_os("NOTAYA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("notaya-action-parity"));
        let state = AppState::new(crate::portable::Paths::at(&root));
        assert!(matches!(
            create_page_person(&state, "missing".into(), "x".repeat(301)),
            Err(Error::Invalid(_))
        ));
        assert!(matches!(
            create_page_person(&state, "missing".into(), "valid".into()),
            Err(Error::Locked)
        ));
    }
}


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinEntry {
    id: String, kind: String, title: String, icon: String, trashed_at: i64, parent_in_bin: bool,
}

#[tauri::command]
pub fn list_bin(state: State<'_, AppState>) -> Result<Vec<BinEntry>> {
    state.with(|s| list_bin_core(s, &Actor::Person))
}

pub(crate) fn list_bin_core(s: &Session, _actor: &Actor) -> Result<Vec<BinEntry>> {
    let mut out = Vec::new();
    for item in s.items() {
        let Some(trashed_at) = Session::own_trash(item) else { continue };
        let (kind, title, icon) = match item {
            Item::Notebook(n) => ("notebook", n.name.clone(), n.icon.clone()),
            Item::Section(x) => ("section", x.name.clone(), String::new()),
            Item::Page(x) => ("page", x.title.clone(), x.icon.clone()),
            Item::Database(x) => ("database", x.title.clone(), x.icon.clone()),
            Item::Row(x) => ("row", x.title.clone(), x.icon.clone()),
            Item::Sticky(x) => ("sticky", x.text.lines().next().unwrap_or("Sticky note").chars().take(60).collect(), String::new()),
            _ => continue,
        };
        out.push(BinEntry { id: item.id().into(), kind: kind.into(), title, icon, trashed_at,
            parent_in_bin: Session::parent_of(item).is_some_and(|p| s.in_bin(p)) });
    }
    out.sort_by_key(|i| std::cmp::Reverse(i.trashed_at));
    Ok(out)
}

fn restore_parents(s: &mut Session, id: &str, allowed: bool) -> Result<()> {
    let mut parents = Vec::new();
    let mut current = s.get(id).and_then(Session::parent_of).map(String::from);
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = current {
        if !seen.insert(id.clone()) { return Err(Error::Corrupt); }
        let item = s.get(&id).ok_or(Error::NotFound)?;
        if Session::own_trash(item).is_some() {
            if !allowed { return Err(Error::invalid("parent_in_bin: Restore the containing items too?")); }
            let mut item = item.clone();
            match &mut item { Item::Notebook(x) => x.trashed_at = None, Item::Section(x) => x.trashed_at = None,
                Item::Page(x) => x.trashed_at = None, Item::Database(x) => x.trashed_at = None,
                Item::Row(x) => x.trashed_at = None, _ => {} }
            parents.push(item);
        }
        current = Session::parent_of(item).map(String::from);
    }
    if !parents.is_empty() { s.put_many(parents)?; }
    Ok(())
}

#[tauri::command]
pub fn restore_bin_item(state: State<'_, AppState>, id: String, restore_containers: bool) -> Result<NotesTree> {
    state.with_mut(|s| restore_bin_item_core(s, &Actor::Person, id, restore_containers))
}

pub(crate) fn restore_bin_item_core(s: &mut Session, actor: &Actor, id: String, restore_containers: bool) -> Result<NotesTree> {
    s.run_as(actor, |s| {
        s.get(&id).ok_or(Error::NotFound)?;
        s.check_write(actor, &id, None)?;
        restore_parents(s, &id, restore_containers)?;
        restore_page_core(s, actor, id)
    })
}

fn delete_bin_tree(s: &mut Session, id: &str) -> Result<()> {
    s.get(id).ok_or(Error::NotFound)?;
    let doomed: Vec<String> = s.items().filter(|i| s.is_within(i.id(), id)).map(|i| i.id().into()).collect();
    let dbs: Vec<String> = doomed.iter().filter(|id| matches!(s.get(id), Some(Item::Database(_)))).cloned().collect();
    for db in dbs { if s.get(&db).is_some() { dbc::delete_database(s, &db)?; } }
    let rows: Vec<String> = doomed.iter().filter(|id| matches!(s.get(id), Some(Item::Row(_)))).cloned().collect();
    if !rows.is_empty() { dbc::delete_rows(s, &rows)?; }
    let remaining: Vec<_> = doomed.into_iter().filter(|id| s.get(id).is_some()).collect();
    s.remove_many(&remaining)?;
    s.collect_garbage()?;
    Ok(())
}
