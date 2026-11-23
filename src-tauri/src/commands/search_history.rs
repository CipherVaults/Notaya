//! The last few searches ("recent searches" in the search box). They are kept in the vault's settings
//! record, so they are encrypted with everything else, wiped from memory when the vault locks, and
//! cleared by the Clear button. Search terms can reveal what a note says, which is why only five are kept.

use crate::actions::Actor;
use crate::vault::Session;

use super::limit;
use crate::error::Result;
use crate::model::{Item, SEARCH_HISTORY_MAX, SEARCH_TERM_MAX};
use crate::state::AppState;
use tauri::State;

#[tauri::command(async)]
pub fn search_history(state: State<'_, AppState>) -> Result<Vec<String>> {
    search_history_person(&state)
}

pub(crate) fn search_history_core(s: &Session, _actor: &Actor) -> Result<Vec<String>> {
    Ok(s.settings().search_history)
}


#[tauri::command(async)]
pub fn add_search_term(state: State<'_, AppState>, term: String) -> Result<Vec<String>> {
    add_search_term_person(&state, term)
}

pub(crate) fn add_search_term_core(s: &mut Session, _actor: &Actor, term: String) -> Result<Vec<String>> {
    limit("The search", &term, SEARCH_TERM_MAX)?;
    change_core(s, |h| push_term(h, &term))
}

#[tauri::command(async)]
pub fn remove_search_term(state: State<'_, AppState>, term: String) -> Result<Vec<String>> {
    remove_search_term_person(&state, term)
}

pub(crate) fn remove_search_term_core(s: &mut Session, _actor: &Actor, term: String) -> Result<Vec<String>> {
    change_core(s, |h| {
        let before = h.len();
        h.retain(|t| *t != term);
        h.len() != before
    })
}

#[tauri::command(async)]
pub fn clear_search_history(state: State<'_, AppState>) -> Result<()> {
    clear_search_history_person(&state)
}

pub(crate) fn clear_search_history_core(s: &mut Session, _actor: &Actor) -> Result<()> {
    change_core(s, |h| {
        let had = !h.is_empty();
        h.clear();
        had
    })
    .map(|_| ())
}

/// Applies `edit` and writes the settings record only when it changed something.
fn change_core(s: &mut Session, edit: impl FnOnce(&mut Vec<String>) -> bool) -> Result<Vec<String>> {
    let mut settings = s.settings();
    if !edit(&mut settings.search_history) {
        return Ok(settings.search_history);
    }
    let history = settings.search_history.clone();
    s.put(Item::Settings(settings))?;
    Ok(history)
}

/// Puts a search at the top (case-insensitive repeats move up instead of doubling) and keeps five.
/// Returns false when nothing changed: an empty term, or the newest line again.
fn push_term(history: &mut Vec<String>, term: &str) -> bool {
    let term = term.split_whitespace().collect::<Vec<_>>().join(" ");
    if term.is_empty() {
        return false;
    }
    if history.first().is_some_and(|h| *h == term) {
        return false;
    }
    let lower = term.to_lowercase();
    history.retain(|h| h.to_lowercase() != lower);
    history.insert(0, term);
    history.truncate(SEARCH_HISTORY_MAX);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_five_kept_repeats_move_up() {
        let mut h = Vec::new();
        for t in ["a", "b", "c", "d", "e", "f"] {
            assert!(push_term(&mut h, t));
        }
        assert_eq!(h, ["f", "e", "d", "c", "b"]);
        assert!(push_term(&mut h, "C"));
        assert_eq!(h, ["C", "f", "e", "d", "b"]);
        assert!(!push_term(&mut h, "C"), "the newest line again changes nothing");
    }

    #[test]
    fn empty_terms_and_extra_spaces() {
        let mut h = Vec::new();
        assert!(!push_term(&mut h, "   "));
        assert!(push_term(&mut h, "  two   words "));
        assert_eq!(h, ["two words"]);
    }
}


fn search_history_person(state: &AppState) -> Result<Vec<String>> {
    state.with(|s| search_history_core(s, &Actor::Person))
}


fn add_search_term_person(state: &AppState, term: String) -> Result<Vec<String>> {
    limit("The search", &term, SEARCH_TERM_MAX)?;
    state.with_mut(|s| add_search_term_core(s, &Actor::Person, term))
}


fn remove_search_term_person(state: &AppState, term: String) -> Result<Vec<String>> {
    state.with_mut(|s| remove_search_term_core(s, &Actor::Person, term))
}


fn clear_search_history_person(state: &AppState) -> Result<()> {
    state.with_mut(|s| clear_search_history_core(s, &Actor::Person))
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;
    #[test]
    fn search_history_person_and_core_match() {
        compare(
            |state| search_history_person(state),
            |s, _paths| search_history_core(s, &Actor::Person),
        );
    }
    #[test]
    fn add_search_term_person_and_core_match() {
        compare(
            |state| add_search_term_person(state, "a search".into()),
            |s, _paths| add_search_term_core(s, &Actor::Person, "a search".into()),
        );
    }
    #[test]
    fn remove_search_term_person_and_core_match() {
        compare(
            |state| remove_search_term_person(state, "a search".into()),
            |s, _paths| remove_search_term_core(s, &Actor::Person, "a search".into()),
        );
    }
    #[test]
    fn clear_search_history_person_and_core_match() {
        compare(
            |state| clear_search_history_person(state),
            |s, _paths| clear_search_history_core(s, &Actor::Person),
        );
    }
}
