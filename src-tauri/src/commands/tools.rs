use serde::Serialize;
use tauri::{AppHandle, State};
use zeroize::Zeroizing;

use crate::actions::Actor;
use crate::vault::Session;
use crate::error::{Error, Result};
use crate::generator::{self, Strength};
use crate::model::Item;
use crate::state::AppState;

#[tauri::command]
pub fn password_strength(password: String, hints: Vec<String>) -> Strength {
    password_strength_person(password, hints)
}

fn password_strength_person(password: String, hints: Vec<String>) -> Strength {
    let password = Zeroizing::new(password);
    let hints: Vec<&str> = hints.iter().map(String::as_str).collect();
    generator::strength(&password, &hints)
}

/// Opens a web link from a note in the default browser. Only http(s) links.
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<()> {
    open_url_person(app, url)
}

fn open_url_person(app: AppHandle, url: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let u = url.trim();
    if u.to_ascii_lowercase().starts_with("notaya:") {
        // A link to a page inside Notaya is followed by the editor, never by the browser.
        return Err(Error::invalid("That link points inside Notaya, not to a web address."));
    }
    let u = if u.contains("://") { u.to_string() } else { format!("https://{u}") };
    if !(u.starts_with("https://") || u.starts_with("http://")) {
        return Err(Error::invalid("Only web links can be opened."));
    }
    app.opener().open_url(u, None::<&str>).map_err(|e| Error::Other(e.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    kind: &'static str,
    id: String,
    title: String,
    subtitle: String,
    snippet: String,
    score: u32,
}

/// Returns ~90 characters around the first match, for the result list.
fn snippet(text: &str, needle: &str) -> String {
    let lower = text.to_lowercase();
    let Some(byte_pos) = lower.find(needle) else {
        return text.chars().take(90).collect();
    };
    let char_pos = lower[..byte_pos].chars().count();
    let start = char_pos.saturating_sub(30);
    let s: String = text.chars().skip(start).take(90).collect();
    if start > 0 { format!("…{}", s.trim_start()) } else { s }
}

/// Searches notes, notebooks and sticky notes in memory. The index is never
/// written to disk.
#[tauri::command]
pub fn search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>> {
    search_person(&state, query)
}

fn search_person(state: &AppState, query: String) -> Result<Vec<SearchHit>> {
    // An empty search succeeds even while locked, as before.
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    state.with(|s| search_core(s, &Actor::Person, query))
}

pub(crate) fn search_core(s: &Session, _actor: &Actor, query: String) -> Result<Vec<SearchHit>> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    let matches_all = |hay: &str| terms.iter().all(|t| hay.contains(t));
    let mut hits = Vec::new();
    let db_titles: std::collections::HashMap<&str, &str> =
        s.items().filter_map(|i| if let Item::Database(d) = i { if d.trashed_at.is_none() { Some((d.id.as_str(), d.title.as_str())) } else { None } } else { None }).collect();
    for item in s.items() {
        match item {
            Item::Database(d) if d.trashed_at.is_none() && !d.inline && matches_all(&d.title.to_lowercase()) => hits.push(SearchHit {
                kind: "database",
                id: d.id.clone(),
                title: if d.title.is_empty() { "Untitled database".into() } else { d.title.clone() },
                subtitle: String::new(),
                snippet: String::new(),
                score: 2,
            }),
            Item::Row(r) if r.trashed_at.is_none() && db_titles.contains_key(r.db.as_str()) => {
                let title = r.title.to_lowercase();
                let hay = format!("{title} {}", r.text.to_lowercase());
                if matches_all(&hay) {
                    hits.push(SearchHit {
                        kind: "row",
                        id: r.id.clone(),
                        title: if r.title.is_empty() { "Untitled".into() } else { r.title.clone() },
                        subtitle: db_titles.get(r.db.as_str()).map(|t| t.to_string()).unwrap_or_default(),
                        snippet: snippet(&r.text, terms[0]),
                        score: if matches_all(&title) { 3 } else { 1 },
                    });
                }
            }
            Item::Page(p) if p.trashed_at.is_none() => {
                let title = p.title.to_lowercase();
                let body = p.text.to_lowercase();
                if matches_all(&format!("{title} {body}")) {
                    hits.push(SearchHit {
                        kind: "page",
                        id: p.id.clone(),
                        title: if p.title.is_empty() { "Untitled page".into() } else { p.title.clone() },
                        subtitle: String::new(),
                        snippet: snippet(&p.text, terms[0]),
                        score: if matches_all(&title) { 3 } else { 1 },
                    });
                }
            }
            Item::Sticky(st) => {
                if matches_all(&st.text.to_lowercase()) {
                    hits.push(SearchHit {
                        kind: "sticky",
                        id: st.id.clone(),
                        title: st.text.lines().next().unwrap_or("Sticky note").chars().take(60).collect(),
                        subtitle: String::new(),
                        snippet: snippet(&st.text, terms[0]),
                        score: 1,
                    });
                }
            }
            Item::Notebook(n) if matches_all(&n.name.to_lowercase()) => hits.push(SearchHit {
                kind: "notebook",
                id: n.id.clone(),
                title: n.name.clone(),
                subtitle: String::new(),
                snippet: String::new(),
                score: 2,
            }),
            _ => {}
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
    hits.truncate(60);
    Ok(hits)
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;

    #[test]
    fn search_person_and_core_match() {
        compare(
            |state| search_person(state, "exact".into()),
            |s, _| search_core(s, &Actor::Person, "exact".into()),
        );
    }

    #[test]
    fn empty_search_still_succeeds_while_locked() {
        let root = std::path::PathBuf::from(std::env::var_os("NOTAYA_HOME").unwrap_or_else(|| "D:\\Notaya Tests\\v1-6-C1".into()));
        let state = AppState::new(crate::portable::Paths::at(&root));
        assert!(search_person(&state, "  \t".into()).unwrap().is_empty());
        assert!(matches!(search_person(&state, "exact".into()), Err(Error::Locked)));
    }
}
