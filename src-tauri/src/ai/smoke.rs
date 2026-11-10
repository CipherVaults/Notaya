//! App core + the actual lane-D bridge over in-memory byte pipes. No app window,
//! Windows pipe ACL or process identity is claimed by this test.
#![allow(dead_code)] // Bridge modules also contain their standalone process adapters.

#[path = "../../../bridge/src/mcp.rs"]
mod mcp;
#[path = "../../../bridge/src/pipe.rs"]
mod pipe;
#[path = "../../../bridge/src/tools.rs"]
mod tools;

use super::{dispatch, pairing, rules::Level, wire};
use crate::{actions::Actor, commands::notes, model::Item, portable::Paths, state::AppState, vault::Session};
use mcp::{Effect, Mcp};
use pipe::Transport;
use serde_json::{json, Value};
use std::{collections::VecDeque, io, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}};

struct FakePipe {
    state: Arc<AppState>,
    pairing: Option<pairing::Pairing>,
    replies: VecDeque<Vec<u8>>,
}

impl FakePipe {
    fn reply(&mut self, message: wire::ServerMessage) {
        let mut bytes = vec![];
        wire::write_frame(&mut bytes, &message, wire::MAX_RESPONSE).unwrap();
        self.replies.push_back(bytes);
    }
    fn welcome(&mut self, level: Level) {
        self.reply(wire::ServerMessage::Welcome {
            notaya_version: env!("CARGO_PKG_VERSION").into(), level, locked: false,
        });
    }
}

impl Transport for FakePipe {
    fn send(&mut self, message: &pipe::ClientMessage) -> io::Result<()> {
        // Serialize lane D's real types, deserialize C5's real types (and vice versa).
        let mut bytes = vec![];
        pipe::write_frame(&mut bytes, message, pipe::MAX_REQUEST)?;
        let message = wire::read_frame(&mut bytes.as_slice(), wire::MAX_REQUEST)?;
        match message {
            wire::ClientMessage::Hello { client, .. } => {
                self.welcome(Level::Off);
                // Fake only the OS connection and window. Use the real confirmation
                // broker, token generation, hashed settings and bridge token storage.
                assert!(self.state.ai.confirms.request(
                    "pairing", json!({"client":client}), crate::actions::Confirm::Ask,
                    |v| self.state.ai.emit("ai-pairing", v), || false,
                ));
                let (p, token) = pairing::create(client);
                self.state.with_mut(|s| {
                    let mut settings = s.settings();
                    settings.ai.pairings.push(p.clone());
                    s.put(Item::Settings(settings))
                }).unwrap();
                self.reply(wire::ServerMessage::PairingResult { pairing_id: p.id.clone(), token: token.to_string() });
                self.welcome(p.level);
                self.pairing = Some(p);
            }
            wire::ClientMessage::Auth { pairing_id, token } => {
                self.pairing = self.state.with(|s| Ok(pairing::authenticate(&s.settings().ai, &pairing_id, &token))).unwrap();
                self.welcome(self.pairing.as_ref().expect("saved token authenticates").level);
            }
            wire::ClientMessage::Call { id, action, params, preview } => {
                let reply = dispatch::call(
                    &self.state, self.pairing.as_ref().unwrap(), &action, params,
                    preview.unwrap_or(false), || false, || {},
                );
                self.reply(match reply {
                    Ok(value) => wire::ServerMessage::Result { id, ok: true, value, rev: None },
                    Err(e) => wire::ServerMessage::Error { id: Some(id), code: dispatch::error_code(&e).into(), message: "Action refused.".into() },
                });
            }
            wire::ClientMessage::Cancel { .. } => panic!("smoke sends no cancellations"),
        }
        Ok(())
    }
    fn poll(&mut self) -> io::Result<Vec<pipe::ServerMessage>> {
        self.replies.drain(..).map(|bytes| pipe::read_frame(&mut bytes.as_slice(), pipe::MAX_RESPONSE)).collect()
    }
}

struct Bridge {
    mcp: Mcp,
    pipe: FakePipe,
}
impl Bridge {
    fn effects(&mut self, effects: Vec<Effect>) -> Vec<Value> {
        let mut work: VecDeque<_> = effects.into();
        let mut output = vec![];
        while let Some(effect) = work.pop_front() {
            match effect {
                Effect::Connect(client) => {
                    self.mcp.connected(false);
                    self.pipe.send(&pipe::ClientMessage::Hello { client, bridge_version: "1.6.0".into() }).unwrap();
                }
                Effect::Send(message) => self.pipe.send(&message).unwrap(),
                Effect::Save(p) => {
                    pipe::save_pairing(&self.pipe.state.paths.data, &p).unwrap();
                    work.extend(self.mcp.saved_pairing());
                }
                Effect::Output(v) => output.push(v),
                Effect::Close => panic!("unexpected connection close"),
            }
            for message in self.pipe.poll().unwrap() { work.extend(self.mcp.server(message)); }
        }
        output
    }
    fn input(&mut self, v: Value) -> Vec<Value> {
        let effects = self.mcp.input(v);
        self.effects(effects)
    }
    fn call(&mut self, id: u64, action: &str, params: Value) -> Value {
        self.input(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{
            "name":"notaya_page","arguments":{"action":action,"params":params}
        }})).into_iter().find(|v| v["id"] == id).expect("one MCP reply")
    }
}

fn error_is(v: &Value, code: &str) {
    assert_eq!(v["result"]["isError"], true, "{v}");
    assert!(v["result"]["content"][0]["text"].as_str().unwrap().starts_with(&format!("{code}:")), "{v}");
}

#[test]
fn bridge_core_smoke_pair_read_confirm_refuse_person_editing_and_ordered_delivery() {
    let root = std::path::PathBuf::from(std::env::var_os("NOTAYA_HOME").expect("throwaway NOTAYA_HOME required"))
        .join(format!("ai-smoke-{}", crate::model::new_id()));
    let paths = Paths::at(&root);
    paths.ensure_dirs().unwrap();
    let mut s = Session::create_open(&paths.vault).unwrap();
    notes::create_notebook_core(&mut s, &Actor::Person, "Smoke only".into(), String::new(), String::new()).unwrap();
    let section = s.items().find_map(|i| match i { Item::Section(x) => Some(x.id.clone()), _ => None }).unwrap();
    let page = notes::create_page_core(&mut s, &Actor::Person, section, "Smoke page".into()).unwrap().id.clone();
    let mut settings = s.settings();
    settings.ai.enabled = true;
    s.put(Item::Settings(settings)).unwrap();
    let state = Arc::new(AppState::new(paths));
    state.set_session(s);
    state.ai.native.inject(crate::native_confirm::TestApprover(true));

    let allow = Arc::new(AtomicBool::new(false));
    let confirms = Arc::new(AtomicUsize::new(0));
    let weak = Arc::downgrade(&state);
    let allow_window = allow.clone();
    let window_confirms = confirms.clone();
    state.ai.bind(move |event, v| {
        let state = weak.upgrade().unwrap();
        // A real PC run must answer these through the Notaya window commands.
        match event {
            "ai-pairing" => crate::commands::ai::ai_pair_answer_core(&state, v["id"].as_str().unwrap().into(), true).unwrap(),
            "ai-confirm" => {
                window_confirms.fetch_add(1, Ordering::SeqCst);
                assert!(state.ai.confirms.answer("confirm", v["id"].as_str().unwrap(), allow_window.load(Ordering::SeqCst), v["word"].as_str()));
            }
            _ => {}
        }
    });
    let mut bridge = Bridge { mcp: Mcp::default(), pipe: FakePipe { state: state.clone(), pairing: None, replies: VecDeque::new() } };
    bridge.input(json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
        "protocolVersion":mcp::LEGACY_VERSION,"capabilities":{},"clientInfo":{"name":"C5 smoke","version":"1"}
    }}));
    bridge.input(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    let saved = pipe::load_pairing(&state.paths.data, "C5 smoke").unwrap().unwrap();
    assert!(!state.with(|s| Ok(serde_json::to_string(&s.settings().ai).unwrap())).unwrap().contains(&saved.token));
    bridge.pipe.send(&pipe::ClientMessage::Auth { pairing_id: saved.pairing_id.clone(), token: saved.token }).unwrap();
    for message in bridge.pipe.poll().unwrap() { bridge.mcp.server(message); }
    println!("CHECKPOINT: fake-pipe pairing, hash-only app storage and saved-token authentication done");

    let read = bridge.call(1, "page.get", json!({"id":page,"format":"blocks"}));
    assert_eq!(read["result"]["isError"], false, "{read}");
    assert_eq!(read["result"]["structuredContent"]["value"]["title"], "Smoke page");
    error_is(&bridge.call(2, "page.trash", json!({"id":page})), "level_too_low");
    // Only the simulated person raises the stored pairing's level.
    state.with_mut(|s| {
        let mut settings = s.settings();
        settings.ai.pairings[0].level = Level::Edit;
        s.put(Item::Settings(settings))
    }).unwrap();
    let effects = bridge.mcp.server(pipe::ServerMessage::Event { kind: pipe::EventKind::LevelChanged, id: None, level: Some(tools::Level::Edit) });
    bridge.effects(effects);
    let before = state.with(|s| Ok(s.rev(&page))).unwrap();
    error_is(&bridge.call(3, "page.trash", json!({"id":page})), "not_confirmed");
    assert!(!state.with(|s| Ok(s.in_bin(&page))).unwrap());
    assert_eq!(state.with(|s| Ok(s.rev(&page))).unwrap(), before);
    assert_eq!(confirms.load(Ordering::SeqCst), 1);
    println!("CHECKPOINT: read, Look-level refusal and denied edit with unchanged revision done");

    state.with_mut(|s| { s.coordination.editing(&page, "smoke-person", true); Ok(()) }).unwrap();
    error_is(&bridge.call(4, "page.trash", json!({"id":page})), "person_editing");
    assert_eq!(confirms.load(Ordering::SeqCst), 1, "lease refusal precedes confirmation");
    assert_eq!(state.with(|s| Ok(s.rev(&page))).unwrap(), before);
    state.unmark_ready("smoke-person"); // closing the synthetic editor releases its lease
    println!("CHECKPOINT: person-editing refusal before confirmation done");

    // Exercise the merge seam: an AI write survives a sink panic, is delivered
    // in order on retry, preserves AI attribution, and can reenter state.
    let attempts = Arc::new(AtomicUsize::new(0));
    let events = Arc::new(Mutex::new(vec![]));
    let tries = attempts.clone();
    let received = events.clone();
    let weak = Arc::downgrade(&state);
    state.bind_events(move |changes| {
        assert!(weak.upgrade().unwrap().is_unlocked(), "callback can acquire session");
        if tries.fetch_add(1, Ordering::SeqCst) == 0 { panic!("injected smoke sink panic"); }
        received.lock().unwrap().push(changes);
        Ok(())
    }, |_| {});
    // Binding drains earlier setup notifications. Use only subsequent delivery evidence.
    state.with_mut(|_| Ok(())).unwrap();
    events.lock().unwrap().clear();
    attempts.store(0, Ordering::SeqCst);
    allow.store(true, Ordering::SeqCst);
    let edited = bridge.call(5, "page.trash", json!({"id":page}));
    assert_eq!(edited["result"]["isError"], false, "{edited}");
    assert!(state.with(|s| Ok(s.in_bin(&page))).unwrap());
    assert_eq!(confirms.load(Ordering::SeqCst), 2);
    state.with_mut(|_| Ok(())).unwrap(); // retries failed notification
    let events = events.lock().unwrap();
    let changed = events.iter().find(|e| e.ids.contains(&page)).expect("AI signal retried");
    assert_eq!(changed.by, "ai");
    assert!(changed.revs[&page] > before);
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    drop(events);
    println!("CHECKPOINT: confirmed edit and panic-isolated AI change delivery done");
    drop(state.take_session()); // flushes the read summary and invalidates old deliveries
    let reopened = Session::unlock_open(&state.paths.vault).unwrap();
    assert!(reopened.in_bin(&page), "edit survived reopening the throwaway vault");
    assert!(pairing::authenticate(&reopened.settings().ai, &saved.pairing_id, &pipe::load_pairing(&state.paths.data, "C5 smoke").unwrap().unwrap().token).is_some());
    println!("CHECKPOINT: durable edit and pairing after reopen done");
}
