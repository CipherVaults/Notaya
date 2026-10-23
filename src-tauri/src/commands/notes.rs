use serde::Serialize;
use tauri::ipc::{InvokeBody, Request};
use tauri::State;

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

/// Deletes a notebook with all of its sections and pages. A checked backup is made first
/// (there is no trash for a whole notebook); if it fails, nothing is deleted. Runs on a worker
/// thread because the backup can take a moment.
#[tauri::command(async)]
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
        backup::make_backup(&state.paths, s).map_err(|e| backup::safety_failure(&e))?;
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

/// Deletes a section with all of its pages, after a checked backup (see `delete_notebook`).
#[tauri::command(async)]
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
        backup::make_backup(&state.paths, s).map_err(|e| backup::safety_failure(&e))?;
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
    let parent = page(s, parent_id)?;
    let now = now_ms();
    let order = next_order(s.items().filter_map(|i| match i {
        Item::Page(p) if p.parent == parent_id => Some(p.order),
        _ => None,
    }));
    let p = Page {
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
    limit("Title", &title, 300)?;
    state.with_mut(|s| add_subpage(s, &parent_id, title))
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
    state.with_mut(|s| {
        move_subtree(s, &id, &section_id)?;
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
    state.with_mut(|s| {
        trash_subtree(s, &id)?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn restore_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        restore_subtree(s, &id)?;
        Ok(tree(s))
    })
}

#[tauri::command]
pub fn delete_page(state: State<'_, AppState>, id: String) -> Result<NotesTree> {
    state.with_mut(|s| {
        page(s, &id)?;
        let doomed = with_descendants(s, &id);
        s.remove_many(&doomed)?;
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

/// Sets the emoji and cover shown on a page. The cover is `gradient:<name>` or one
/// of this vault's attachment URLs (so a cover image is encrypted like any other).
#[tauri::command]
pub fn set_page_look(state: State<'_, AppState>, id: String, icon: String, cover: String) -> Result<PageMeta> {
    limit("Icon", &icon, 40)?;
    limit("Cover", &cover, 200)?;
    state.with_mut(|s| {
        let mut p = page(s, &id)?;
        p.icon = icon;
        p.cover = cover;
        p.updated = now_ms();
        let meta = PageMeta::from(&p);
        s.put(Item::Page(p))?;
        Ok(meta)
    })
}

#[cfg(test)]
mod subpage_tests {
    use super::*;
    use crate::crypto::KdfParams;

    fn vault() -> Session {
        let dir = std::env::temp_dir().join(format!("cipherleaf-test-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        Session::create(&dir.join("vault.clf"), "pw", None, KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap()
    }

    fn page_in(s: &mut Session, section_id: &str, title: &str) -> Page {
        let p = Page {
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
        s.put(Item::Section(Section { id: sec_a.clone(), notebook_id: new_id(), name: "A".into(), color: String::new(), order: 0, created: 1, updated: 1 })).unwrap();
        s.put(Item::Section(Section { id: sec_b.clone(), notebook_id: new_id(), name: "B".into(), color: String::new(), order: 0, created: 1, updated: 1 })).unwrap();
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
