use serde::Serialize;
use tauri::State;
use zeroize::Zeroizing;

use crate::error::Result;
use crate::generator::{self, Generated, PassphraseOptions, PasswordOptions, Strength};
use crate::model::Item;
use crate::state::AppState;

#[tauri::command]
pub fn generate_password(options: PasswordOptions) -> Result<Generated> {
    generator::password(&options)
}

#[tauri::command]
pub fn generate_passphrase(options: PassphraseOptions) -> Result<Generated> {
    generator::passphrase(&options)
}

#[tauri::command]
pub fn password_strength(password: String, hints: Vec<String>) -> Strength {
    let password = Zeroizing::new(password);
    let hints: Vec<&str> = hints.iter().map(String::as_str).collect();
    generator::strength(&password, &hints)
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

/// Returns ~70 characters around the first match, for the result list.
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

/// Searches everything in memory. The index is never written to disk.
#[tauri::command]
pub fn search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchHit>> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    let matches_all = |hay: &str| terms.iter().all(|t| hay.contains(t));
    state.with(|s| {
        let mut hits = Vec::new();
        for item in s.items() {
            match item {
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
                Item::Login(l) => {
                    let hay = format!("{} {} {} {} {}", l.title, l.username, l.urls.join(" "), l.tags.join(" "), l.folder)
                        .to_lowercase();
                    if matches_all(&hay) {
                        hits.push(SearchHit {
                            kind: "login",
                            id: l.id.clone(),
                            title: l.title.clone(),
                            subtitle: l.username.clone(),
                            snippet: l.urls.first().cloned().unwrap_or_default(),
                            score: if matches_all(&l.title.to_lowercase()) { 3 } else { 2 },
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
    })
}
