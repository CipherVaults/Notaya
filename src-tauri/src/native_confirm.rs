//! Rust-only, single-use approvals. Neither prompts nor approvals cross IPC.
use crate::{error::{Error, Result}, model::Item, state::AppState, vault::Session};
use std::sync::{Arc, Mutex, atomic::Ordering};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub(crate) const CANCELLED: &str = "cancelled";
pub(crate) fn cancelled() -> Error { Error::invalid(CANCELLED) }

pub(crate) struct NativePrompt { pub title: &'static str, pub body: String }
pub(crate) trait NativeApprover: Send + Sync {
    fn ask(&self, prompt: NativePrompt) -> bool;
}
struct Deny;
impl NativeApprover for Deny { fn ask(&self, _: NativePrompt) -> bool { false } }
struct WindowsApprover(AppHandle);
impl NativeApprover for WindowsApprover {
    fn ask(&self, prompt: NativePrompt) -> bool {
        let Some(main) = self.0.get_webview_window("main") else { return false };
        self.0.dialog().message(prompt.body).title(prompt.title).parent(&main)
            .kind(MessageDialogKind::Warning).buttons(MessageDialogButtons::OkCancel)
            .blocking_show()
    }
}

pub(crate) struct Approvals {
    approver: Mutex<Arc<dyn NativeApprover>>,
    // Fail closed on overlapping prompts; never hold the vault mutex here.
    asking: Mutex<()>,
}
impl Default for Approvals {
    fn default() -> Self { Self { approver: Mutex::new(Arc::new(Deny)), asking: Mutex::new(()) } }
}
impl Approvals {
    pub(crate) fn bind(&self, app: AppHandle) {
        *self.approver.lock().unwrap() = Arc::new(WindowsApprover(app));
    }
    pub(crate) fn ask(&self, target: Target) -> Result<Approval> {
        let _gate = self.asking.try_lock().map_err(|_| cancelled())?;
        let approver = self.approver.lock().unwrap().clone();
        if !approver.ask(target.prompt()) { return Err(cancelled()) }
        Ok(Approval { target: Some(target) })
    }
    #[cfg(test)]
    pub(crate) fn inject(&self, approver: impl NativeApprover + 'static) {
        *self.approver.lock().unwrap() = Arc::new(approver);
    }
}

/// Strip control and bidi formatting characters which could disguise a target.
/// At most five titles, each at most 80 scalar values, ever enter a dialog.
pub(crate) fn label(s: &str) -> String {
    let mut chars = s.chars().filter(|c| !c.is_control() && !matches!(*c,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'));
    let mut out: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() { out.push('…'); }
    if out.trim().is_empty() { "Untitled".into() } else { out }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Delete {
    One(String),
    Empty,
    // AI bin.empty: only its visible binned targets, never hidden notebooks.
    Selected(Vec<String>),
}
#[derive(PartialEq, Eq)]
enum Kind {
    Delete { request: Delete, items: Vec<(String, u64)>, titles: Vec<String> },
    Enable { ai: [u8; 32], revision: u64 },
    Pair { id: String, client: String, ai: [u8; 32], revision: u64 },
}
#[derive(PartialEq, Eq)]
pub(crate) struct Target { session: [u8; 16], epoch: u64, kind: Kind }
impl Target {
    fn prompt(&self) -> NativePrompt {
        match &self.kind {
            Kind::Delete { request, items, titles } => {
                let title = if matches!(request, Delete::One(_)) { "Notaya — Delete for good?" }
                    else { "Notaya — Empty the bin?" };
                let mut body = format!("Permanently delete {} item(s) and their history? This cannot be undone.", items.len());
                for title in titles { body.push_str(&format!("\n• {title}")); }
                if items.len() > titles.len() { body.push_str(&format!("\n…and {} more item(s).", items.len() - titles.len())); }
                NativePrompt { title, body }
            }
            Kind::Enable { .. } => NativePrompt { title: "Notaya — Turn on AI access?",
                body: "Turn on AI access? Paired AIs will be able to connect to your notes with the access you give them.".into() },
            Kind::Pair { client, .. } => NativePrompt { title: "Notaya — Connect an AI?",
                body: format!("Allow “{}” to connect to Notaya? It starts at Look and can read notebooks you have not hidden. Allow only a connection you started yourself.", label(client)) },
        }
    }
}

pub(crate) fn delete_target(s: &Session, epoch: u64, request: Delete) -> Result<Target> {
    let roots: Vec<String> = match &request {
        Delete::One(id) => vec![id.clone()],
        Delete::Empty => s.items().filter(|i| Session::own_trash(i).is_some()).map(|i| i.id().into()).collect(),
        Delete::Selected(ids) => ids.clone(),
    };
    for id in &roots {
        let item = s.get(id).ok_or(Error::NotFound)?;
        if !matches!(item, Item::Notebook(_) | Item::Section(_) | Item::Page(_) | Item::Database(_) | Item::Row(_) | Item::Sticky(_)) || !s.in_bin(id) {
            return Err(Error::invalid("Only items in the bin can be permanently deleted."));
        }
    }
    let mut items: Vec<_> = s.items().filter(|i| roots.iter().any(|root| s.is_within(i.id(), root)))
        .map(|i| (i.id().to_owned(), s.rev(i.id()))).collect();
    items.sort();
    let titles = items.iter().take(5).filter_map(|(id, _)| s.get(id)).map(|i| label(match i {
        Item::Notebook(x) => &x.name, Item::Section(x) => &x.name, Item::Page(x) => &x.title,
        Item::Database(x) => &x.title, Item::Row(x) => &x.title, Item::Sticky(x) => &x.text,
        _ => "Untitled",
    })).collect();
    Ok(Target { session: s.native_session_id(), epoch, kind: Kind::Delete { request, items, titles } })
}
fn ai_stamp(s: &Session) -> ([u8; 32], u64) {
    (crate::crypto::sha256(&serde_json::to_vec(&s.settings().ai).expect("AI settings serialize")), s.rev(crate::model::SETTINGS_ID))
}
pub(crate) fn enable_target(s: &Session, epoch: u64) -> Target {
    let (ai, revision) = ai_stamp(s);
    Target { session: s.native_session_id(), epoch, kind: Kind::Enable { ai, revision } }
}
pub(crate) fn pair_target(s: &Session, epoch: u64, id: String, client: String) -> Target {
    let (ai, revision) = ai_stamp(s);
    Target { session: s.native_session_id(), epoch, kind: Kind::Pair { id, client, ai, revision } }
}

/// No Clone, Serialize, constructor from IPC, or approval cache. Even a failed
/// recheck consumes this answer. All comparisons happen under the session lock.
pub(crate) struct Approval { target: Option<Target> }
impl Approval {
    pub(crate) fn consume(&mut self, current: Target) -> Result<()> {
        if self.target.take().is_some_and(|expected| expected == current) { Ok(()) } else { Err(cancelled()) }
    }
    pub(crate) fn delete(&mut self, s: &Session, request: Delete) -> Result<Deletion> {
        let expected = self.target.take();
        let epoch = expected.as_ref().map_or(0, |t| t.epoch);
        let current = delete_target(s, epoch, request)?;
        let ids = match &current.kind { Kind::Delete { items, .. } => items.iter().map(|(id, _)| id.clone()).collect(), _ => unreachable!() };
        if expected.as_ref() != Some(&current) { return Err(cancelled()); }
        Ok(Deletion { ids })
    }
}
pub(crate) struct Deletion { ids: std::collections::BTreeSet<String> }
impl Deletion {
    pub(crate) fn consume(&mut self, ids: &[String]) -> Result<()> {
        if !ids.iter().all(|id| self.ids.contains(id)) { return Err(cancelled()); }
        for id in ids { self.ids.remove(id); }
        Ok(())
    }
}
pub(crate) fn epoch(state: &AppState) -> u64 { state.ai.epoch.load(Ordering::SeqCst) }
pub(crate) fn check_epoch(state: &AppState, expected: u64) -> Result<()> {
    if epoch(state) == expected { Ok(()) } else { Err(cancelled()) }
}

#[cfg(test)]
pub(crate) fn test_delete(s: &Session, request: Delete) -> Approval {
    let approvals = Approvals::default();
    approvals.inject(TestApprover(true));
    delete_target(s, 0, request).map(|t| approvals.ask(t).unwrap()).unwrap_or(Approval { target: None })
}
#[cfg(test)]
pub(crate) struct TestApprover(pub bool);
#[cfg(test)]
impl NativeApprover for TestApprover { fn ask(&self, _: NativePrompt) -> bool { self.0 } }

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use serde_json::{json, Value};
    pub const PAGE: &str = "20000000000000000000000000000003";
    pub const SECTION: &str = "20000000000000000000000000000002";
    pub const STICKY: &str = "20000000000000000000000000000004";
    pub fn fixture() -> Arc<AppState> {
        let (mut s, paths) = crate::changes::tests::fixture();
        s.snapshot_page(PAGE).unwrap();
        let state = Arc::new(AppState::new(paths));
        state.set_session(s);
        state
    }
    pub fn snapshot(state: &AppState) -> Value {
        state.with(|s| {
            let items: std::collections::BTreeMap<_, _> = s.items().map(|i| (i.id(), serde_json::to_value(i).unwrap())).collect();
            let history: std::collections::BTreeMap<_, _> = s.items().filter(|i| matches!(i, Item::Page(_) | Item::Row(_)))
                .map(|i| (i.id(), s.history_list(i.id()).unwrap())).collect();
            Ok(json!({"items": items, "revisions": s.revs, "history": history,
                "activity": s.activity_list(None, None, 100_000)?, "dirty": s.dirty, "hourlyDue": s.hourly_due,
                "fileHash": crate::crypto::sha256(&std::fs::read(&state.paths.vault).unwrap())}))
        }).unwrap()
    }
    pub struct Hook(pub Box<dyn Fn() -> bool + Send + Sync>);
    impl NativeApprover for Hook { fn ask(&self, _: NativePrompt) -> bool { (self.0)() } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_support::*;
    #[test]
    fn prompt_labels_are_bounded_and_cannot_inject_lines_or_bidi_controls() {
        assert_eq!(label("a\r\nb\t\u{202e}c"), "abc");
        assert_eq!(label(&"x".repeat(500)).chars().count(), 81);
        let state=fixture();
        state.with_mut(|s| {
            crate::commands::notes::trash_page_core(s, &crate::actions::Actor::Person, PAGE.into())?;
            for _ in 0..8 {
                let id=crate::commands::notes::create_page_core(s,&crate::actions::Actor::Person,SECTION.into(),"x".repeat(200))?.id.clone();
                crate::commands::notes::trash_page_core(s,&crate::actions::Actor::Person,id)?;
            }
            let prompt=delete_target(s,0,Delete::Empty)?.prompt();
            assert!(prompt.body.starts_with("Permanently delete 9 item(s) and their history?"));
            assert_eq!(prompt.body.lines().filter(|s|s.starts_with('•')).count(),5);
            assert!(prompt.body.ends_with("…and 4 more item(s)."));
            Ok(())
        }).unwrap();
    }
    #[test]
    fn approvals_are_single_use_and_bound_to_session_and_kind() {
        let state=fixture();state.ai.native.inject(TestApprover(true));
        state.with_mut(|s| {
            let mut approval=state.ai.native.ask(enable_target(s,0))?;
            approval.consume(enable_target(s,0))?;
            assert!(approval.consume(enable_target(s,0)).is_err());
            let mut approval=state.ai.native.ask(enable_target(s,0))?;
            assert!(approval.consume(pair_target(s,0,"id".into(),"client".into())).is_err());
            assert!(approval.consume(enable_target(s,0)).is_err());
            Ok(())
        }).unwrap();
        let mut approval=state.with(|s|state.ai.native.ask(enable_target(s,0))).unwrap();
        drop(state.take_session());
        state.set_session(Session::unlock(&state.paths.vault,"throwaway",None).unwrap());
        state.with(|s| { assert!(approval.consume(enable_target(s,0)).is_err()); Ok(()) }).unwrap();
    }
}
