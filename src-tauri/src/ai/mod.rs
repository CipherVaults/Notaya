pub mod confirm;
pub mod dispatch;
pub mod pairing;
pub mod pipe;
pub mod rules;
pub mod wire;

#[cfg(test)]
mod smoke;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

type EventSink = Arc<dyn Fn(&str, Value) + Send + Sync>;
#[derive(Default)]
pub struct Runtime {
    pub(crate) native: crate::native_confirm::Approvals,
    pub confirms: confirm::Confirmations,
    pub epoch: AtomicU64,
    pub started: AtomicBool,
    sink: Mutex<Option<EventSink>>,
    reads: Mutex<HashMap<String, (crate::actions::Actor, u64)>>,
}
impl Runtime {
    pub fn bind(&self, sink: impl Fn(&str, Value) + Send + Sync + 'static) {
        *self.sink.lock().unwrap() = Some(Arc::new(sink));
    }
    pub fn emit(&self, event: &str, value: Value) {
        let sink = self.sink.lock().unwrap().clone();
        if let Some(sink) = sink {
            sink(event, value);
        }
    }
    pub fn invalidate(&self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.confirms.cancel_all();
    }
    // Counts only: never retain a page, query, title or note body in the runtime.
    pub fn read(&self, pairing: &pairing::Pairing) {
        let mut reads = self.reads.lock().unwrap();
        let entry = reads
            .entry(pairing.id.clone())
            .or_insert_with(|| (dispatch::actor(pairing), 0));
        entry.1 = entry.1.saturating_add(1);
    }
    pub fn flush_reads(&self, s: &mut crate::vault::Session, id: Option<&str>) {
        let summaries = {
            let mut reads = self.reads.lock().unwrap();
            if let Some(id) = id {
                reads.remove(id).into_iter().collect::<Vec<_>>()
            } else {
                std::mem::take(&mut *reads).into_values().collect()
            }
        };
        for (actor, count) in summaries {
            if let Err(error) = s.record_ai_reads(&actor, count) {
                if let Some(code) = error.fault_code() {
                    crate::logging::backend_error(&code);
                }
            }
        }
    }
}
