use serde::Serialize;
use tauri::ipc::{InvokeBody, Request};
use tauri::State;

use super::{check_color, limit, notebook, page, section};
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
}

fn tree(s: &Session) -> NotesTree {
    let mut notebooks = Vec::new();
    let mut sections = Vec::new();
    let mut pages = Vec::new();
    for item in s.items() {
        match item {
            Item::Notebook(n) => notebooks.push(n.clone()),
            Item::Section(x) => sections.push(x.clone()),
            Item::Page(p) => pages.push(PageMeta::from(p)),
            _ => {}
        }
    }
    notebooks.sort_by_key(|n| (n.order, n.created));
    sections.sort_by_key(|x| (x.order, x.created));
    pages.sort_by_key(|p| (p.order, p.created));
    NotesTree { notebooks, sections, pages }
}

fn next_order(orders: impl Iterator<Item = i64>) -> i64 {
    orders.max().map_or(0, |m| m + 1)
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<NotesTree> {
    state.with(|s| Ok(tree(s)))
}

#[tauri::command]
pub fn create_notebook(state: State<'_, AppState>, name: String, color: String, icon: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    state.with_mut(|s| {
        let now = now_ms();
        let order = next_order(s.items().filter_map(|i| match i {
            Item::Notebook(n) => Some(n.order),
            _ => None,
        }));
        let nb = Notebook { id: new_id(), name, color, icon, order, created: now, updated: now };
        let first_section = Section {
            id: new_id(),
            notebook_id: nb.id.clone(),
            name: "General".into(),
            color: String::new(),
            order: 0,
            created: now,
            updated: now,
        };
        s.put_many(vec![Item::Notebook(nb), Item::Section(first_section)])?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn update_notebook(state: State<'_, AppState>, id: String, name: String, color: String, icon: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    limit("Icon", &icon, 40)?;
    let color = check_color(&color)?;
    state.with_mut(|s| {
        let mut nb = notebook(s, &id)?;
        nb.name = name;
        nb.color = color;
        nb.icon = icon;
        nb.updated = now_ms();
        s.put(Item::Notebook(nb))?;
        Ok(tree(s))
    })
}

/// Deletes a notebook with all of its sections and pages.
#[tauri::command]
pub fn delete_notebook(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        notebook(s, &id)?;
        let section_ids: Vec<String> = s
            .items()
            .filter_map(|i| match i {
                Item::Section(x) if x.notebook_id == id => Some(x.id.clone()),
                _ => None,
            })
            .collect();
        let mut doomed: Vec<String> = s
            .items()
            .filter_map(|i| match i {
                Item::Page(p) if section_ids.contains(&p.section_id) => Some(p.id.clone()),
                _ => None,
            })
            .collect();
        doomed.extend(section_ids);
        doomed.push(id.clone());
        s.remove_many(&doomed)?;
        s.collect_garbage()?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn create_section(state: State<'_, AppState>, notebook_id: String, name: String, color: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    state.with_mut(|s| {
        notebook(s, &notebook_id)?;
        let now = now_ms();
        let order = next_order(s.items().filter_map(|i| match i {
            Item::Section(x) if x.notebook_id == notebook_id => Some(x.order),
            _ => None,
        }));
        s.put(Item::Section(Section {
            id: new_id(),
            notebook_id,
            name,
            color,
            order,
            created: now,
            updated: now,
        }))?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn update_section(state: State<'_, AppState>, id: String, name: String, color: String) -> Result<NotesTree> {
    limit("Name", &name, 120)?;
    let color = check_color(&color)?;
    state.with_mut(|s| {
        let mut x = section(s, &id)?;
        x.name = name;
        x.color = color;
        x.updated = now_ms();
        s.put(Item::Section(x))?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn delete_section(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        section(s, &id)?;
        let mut doomed: Vec<String> = s
            .items()
            .filter_map(|i| match i {
                Item::Page(p) if p.section_id == id => Some(p.id.clone()),
                _ => None,
            })
            .collect();
        doomed.push(id.clone());
        s.remove_many(&doomed)?;
        s.collect_garbage()?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn create_page(state: State<'_, AppState>, section_id: String, title: String) -> Result<Page> {
    limit("Title", &title, 300)?;
    state.with_mut(|s| {
        section(s, &section_id)?;
        let now = now_ms();
        let order = next_order(s.items().filter_map(|i| match i {
            Item::Page(p) if p.section_id == section_id => Some(p.order),
            _ => None,
        }));
        let p = Page {
            id: new_id(),
            section_id,
            title,
            content: String::new(),
            text: String::new(),
            order,
            favorite: false,
            trashed_at: None,
            created: now,
            updated: now,
        };
        s.put(Item::Page(p.clone()))?;
        Ok(p)
    })
}

#[tauri::command]
pub fn get_page(state: State<'_, AppState>, id: String) -> Result<Page> {
    state.with(|s| page(s, &id))
}

#[tauri::command]
pub fn save_page(state: State<'_, AppState>, id: String, title: String, content: String, text: String) -> Result<PageMeta> {
    limit("Title", &title, 300)?;
    limit("Page", &content, MAX_PAGE_CHARS)?;
    limit("Page", &text, MAX_PAGE_CHARS)?;
    state.with_mut(|s| {
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

#[tauri::command]
pub fn set_page_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> Result<NotesTree> {
    state.with_mut(|s| {
        let mut p = page(s, &id)?;
        p.favorite = favorite;
        s.put(Item::Page(p))?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn move_page(state: State<'_, AppState>, id: String, section_id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        section(s, &section_id)?;
        let mut p = page(s, &id)?;
        p.order = next_order(s.items().filter_map(|i| match i {
            Item::Page(x) if x.section_id == section_id => Some(x.order),
            _ => None,
        }));
        p.section_id = section_id;
        s.put(Item::Page(p))?;
        Ok(tree(s))
    })
}

/// Sets the display order of notebooks, sections or pages to the order given.
#[tauri::command]
pub fn reorder(state: State<'_, AppState>, ids: Vec<String>) -> Result<NotesTree> {
    state.with_mut(|s| {
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
                _ => return Err(Error::NotFound),
            }
        }
        s.put_many(changed)?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn trash_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        let mut p = page(s, &id)?;
        p.trashed_at = Some(now_ms());
        p.favorite = false;
        s.put(Item::Page(p))?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn restore_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        let mut p = page(s, &id)?;
        p.trashed_at = None;
        s.put(Item::Page(p))?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn delete_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        page(s, &id)?;
        s.remove_many(&[id])?;
        s.collect_garbage()?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn empty_trash(state: State<'_, AppState>) -> Result<NotesTree> {
    state.with_mut(|s| {
        let doomed: Vec<String> = s
            .items()
            .filter_map(|i| match i {
                Item::Page(p) if p.trashed_at.is_some() => Some(p.id.clone()),
                _ => None,
            })
            .collect();
        s.remove_many(&doomed)?;
        s.collect_garbage()?;
        Ok(tree(s))
    })
}

/// Identifies an image from its first bytes; the name the UI sends is ignored.
/// SVG is refused on purpose because it can contain scripts.
fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
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
    let mime = sniff_image(bytes)
        .ok_or_else(|| Error::invalid("Only PNG, JPEG, GIF and WebP images are supported."))?;
    state.with_mut(|s| s.put_attachment(mime, bytes))
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
