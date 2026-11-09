//! Window-only replies. No wire message can construct an answer.
use crate::actions::Confirm;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

struct Pending {
    kind: String,
    word: Option<String>,
    answer: Option<bool>,
    client: Option<String>,
    deadline: Instant,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PairRequest {
    pub id: String,
    pub client: String,
    deadline: Instant,
}
#[derive(Default)]
pub struct Confirmations {
    pending: Mutex<HashMap<String, Pending>>,
    cv: Condvar,
}
impl Confirmations {
    pub fn request(
        &self,
        kind: &str,
        details: Value,
        strength: Confirm,
        emit: impl FnOnce(Value),
        cancelled: impl Fn() -> bool,
    ) -> bool {
        self.wait(
            kind,
            details,
            strength,
            emit,
            cancelled,
            Duration::from_secs(300),
        )
    }
    fn wait(
        &self,
        kind: &str,
        details: Value,
        strength: Confirm,
        emit: impl FnOnce(Value),
        cancelled: impl Fn() -> bool,
        timeout: Duration,
    ) -> bool {
        let id = crate::model::new_id();
        let deadline = Instant::now() + timeout;
        let word = (strength == Confirm::Strong).then(|| {
            const WORDS: [&str; 8] = [
                "MAPLE", "CEDAR", "RIVER", "AMBER", "CLOUD", "BIRCH", "FERN", "STONE",
            ];
            // Independent random words, not a client-supplied challenge.
            let random = crate::crypto::random_bytes::<2>();
            format!(
                "{}-{}",
                WORDS[(random[0] % 8) as usize],
                WORDS[(random[1] % 8) as usize]
            )
        });
        self.pending.lock().unwrap().insert(
            id.clone(),
            Pending {
                kind: kind.into(),
                word: word.clone(),
                answer: None,
                client: (kind == "pairing").then(|| details["client"].as_str().unwrap_or("").to_owned()),
                deadline,
            },
        );
        emit(json!({"id":id,"kind":kind,"details":details,"word":word}));
        let mut pending = self.pending.lock().unwrap();
        loop {
            // Do not hold this mutex while cancelled checks session/connection state.
            drop(pending);
            let stop = cancelled() || Instant::now() >= deadline;
            pending = self.pending.lock().unwrap();
            let answer = pending.get(&id).and_then(|p| p.answer);
            if stop || answer.is_some() || !pending.contains_key(&id) {
                pending.remove(&id);
                return !stop && answer == Some(true);
            }
            pending = self
                .cv
                .wait_timeout(pending, Duration::from_millis(50))
                .unwrap()
                .0;
        }
    }
    pub fn answer(&self, kind: &str, id: &str, allow: bool, word: Option<&str>) -> bool {
        // Even Rust callers must use the native-bound pairing route for approval.
        if kind == "pairing" && allow { return false; }
        self.answer_inner(kind, id, allow, word)
    }
    fn answer_inner(&self, kind: &str, id: &str, allow: bool, word: Option<&str>) -> bool {
        let mut pending = self.pending.lock().unwrap();
        let Some(p) = pending.get_mut(id) else {
            return false;
        };
        if p.kind != kind || p.answer.is_some() || Instant::now() >= p.deadline {
            return false;
        }
        if allow
            && p.word
                .as_deref()
                .is_some_and(|expected| Some(expected) != word)
        {
            return false;
        }
        p.answer = Some(allow);
        self.cv.notify_all();
        true
    }
    pub(crate) fn pairing_request(&self, id: &str) -> Option<PairRequest> {
        let pending = self.pending.lock().unwrap();
        let p = pending.get(id)?;
        if p.kind != "pairing" || p.answer.is_some() || Instant::now() >= p.deadline { return None; }
        Some(PairRequest { id: id.into(), client: p.client.clone()?, deadline: p.deadline })
    }
    pub(crate) fn answer_native_pairing(&self, expected: &PairRequest) -> bool {
        // One mutex covers the exact request check and its single-use answer.
        let mut pending = self.pending.lock().unwrap();
        let Some(p) = pending.get_mut(&expected.id) else { return false; };
        if p.kind != "pairing" || p.answer.is_some() || p.client.as_ref() != Some(&expected.client)
            || p.deadline != expected.deadline || Instant::now() >= p.deadline { return false; }
        p.answer = Some(true);
        self.cv.notify_all();
        true
    }
    pub fn cancel_all(&self) {
        self.pending.lock().unwrap().clear();
        self.cv.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_exact_window_answer_succeeds_and_timeout_and_lock_cancel() {
        let c = Confirmations::default();
        assert!(c.wait(
            "confirm",
            json!({}),
            Confirm::Strong,
            |v| {
                let id = v["id"].as_str().unwrap();
                assert!(!c.answer("pairing", id, true, v["word"].as_str()));
                assert!(!c.answer("confirm", id, true, Some("wrong")));
                assert!(c.answer("confirm", id, true, v["word"].as_str()));
                assert!(!c.answer("confirm", id, true, v["word"].as_str()));
            },
            || false,
            Duration::from_secs(1)
        ));
        assert!(!c.wait(
            "confirm",
            json!({}),
            Confirm::Ask,
            |_| {},
            || false,
            Duration::from_millis(1)
        ));
        assert!(!c.wait(
            "confirm",
            json!({}),
            Confirm::Ask,
            |_| c.cancel_all(),
            || false,
            Duration::from_secs(1)
        ));
    }
    #[test]
    fn native_pairing_answer_after_expiry_or_wrong_request_is_refused() {
        use crate::native_confirm::{test_support::*, TestApprover};
        let state=fixture();let weak=std::sync::Arc::downgrade(&state);
        state.ai.native.inject(Hook(Box::new(move || {
            assert!(weak.upgrade().unwrap().is_unlocked());
            std::thread::sleep(Duration::from_millis(10));true
        })));
        let before=snapshot(&state);
        assert!(!state.ai.confirms.wait("pairing",json!({"client":"Expiring client"}),Confirm::Ask,|v|{
            assert!(crate::commands::ai::ai_pair_answer_core(&state,v["id"].as_str().unwrap().into(),true).is_err());
        },||false,Duration::from_millis(2)));
        assert_eq!(snapshot(&state),before);
        state.ai.native.inject(TestApprover(true));
        assert!(!state.ai.confirms.wait("pairing",json!({"client":"Original"}),Confirm::Ask,|v|{
            let request=state.ai.confirms.pairing_request(v["id"].as_str().unwrap()).unwrap();
            let mut wrong=request.clone();wrong.client="Different".into();
            assert!(!state.ai.confirms.answer_native_pairing(&wrong));
            assert!(state.ai.confirms.answer("pairing",&request.id,false,None));
        },||false,Duration::from_secs(1)));
        assert_eq!(snapshot(&state),before);
    }
}
