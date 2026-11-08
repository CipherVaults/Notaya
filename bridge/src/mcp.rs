//! Newline-delimited JSON-RPC 2.0 (stdio), pinned MCP 2025-11-25.
//! Official text to recheck before integration: https://modelcontextprotocol.io/specification/2025-11-25
//! Live documentation verification was not possible in the offline lane.
use crate::{
    pipe::{ClientMessage, EventKind, Pairing, ServerMessage},
    tools::{self, Level},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

pub const PROTOCOL_VERSION: &str = "2025-11-25";
const MAX_PENDING: usize = 200;

pub enum Effect {
    Output(Value),
    Connect(String),
    Send(ClientMessage),
    Save(Pairing),
    Close,
}

struct Pending {
    rpc_id: Value,
    progress: Option<Value>,
}

#[derive(Default)]
pub struct Mcp {
    initialized: bool,
    ready: bool,
    connecting: bool,
    connected: bool,
    paired: bool,
    welcome: Option<(Level, bool)>,
    level: Level,
    client: String,
    problem: &'static str,
    next_id: u64,
    pending: BTreeMap<u64, Pending>,
    refresh_id: Option<u64>,
}

fn rpc_result(id: Value, result: Value) -> Effect {
    Effect::Output(json!({"jsonrpc":"2.0","id":id,"result":result}))
}
fn rpc_error(id: Value, code: i32, message: &'static str) -> Effect {
    Effect::Output(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}))
}
fn id_is_valid(id: &Value) -> bool {
    id.is_string() || id.as_i64().is_some() || id.as_u64().is_some()
}

pub fn parse_error() -> Effect {
    rpc_error(Value::Null, -32700, "Parse error")
}

/// Read one bounded MCP line without allocating until a newline on hostile input.
pub fn read_line<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err(io::ErrorKind::UnexpectedEof.into())
            };
        }
        let newline = available.iter().position(|b| *b == b'\n');
        let count = newline.map_or(available.len(), |i| i + 1);
        if bytes.len() + count > crate::pipe::MAX_REQUEST {
            return Err(io::ErrorKind::InvalidData.into());
        }
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return Ok(Some(bytes));
        }
    }
}

pub fn tool_error(id: Value, code: &str) -> Effect {
    rpc_result(
        id,
        json!({"isError":true,"content":[{"type":"text","text":safe_error(code)}]}),
    )
}

/// Never echo OS errors, parse errors, cancellation reasons or the server's error message.
fn safe_error(code: &str) -> &'static str {
    match code {
        "locked" => "locked: Notaya is locked.",
        "not_running" => "not_running: Open Notaya and turn on AI access in its Settings.",
        "off" => "off: AI access is off in Notaya.",
        "stopped" => "stopped: Resume AI access in the Notaya window.",
        "not_paired" => "not_paired: Allow the connection in the Notaya window.",
        "pairing_refused" => "pairing_refused: The connection was refused in Notaya.",
        "level_too_low" => "level_too_low: This action needs a higher level in Notaya.",
        "not_found" => "not_found: The item is not available.",
        "person_editing" => "person_editing: The person is editing this item. Read it again later.",
        "claimed_by_other_ai" => "claimed_by_other_ai: Another AI has claimed this item.",
        "item_changed" => "item_changed: Read the item again before changing it.",
        "not_confirmed" => "not_confirmed: The action was not confirmed in Notaya.",
        "too_big" => "too_big: Use a smaller request or a paged read.",
        "rate_limited" => "rate_limited: Wait before making more calls.",
        _ => "invalid: Notaya could not complete the request.",
    }
}

impl Mcp {
    fn changed(&self) -> Vec<Effect> {
        if self.ready {
            vec![Effect::Output(
                json!({"jsonrpc":"2.0","method":"notifications/tools/list_changed"}),
            )]
        } else {
            vec![]
        }
    }

    fn connect(&mut self) -> Vec<Effect> {
        if self.connected || self.connecting {
            return vec![];
        }
        self.connecting = true;
        vec![Effect::Connect(self.client.clone())]
    }

    pub fn connected(&mut self, paired: bool) {
        self.connecting = false;
        self.connected = true;
        self.paired = paired;
        self.problem = "not_paired";
    }

    fn set_level(&mut self, level: Level) -> Vec<Effect> {
        if self.level == level {
            return vec![];
        }
        self.level = level;
        self.changed()
    }

    pub fn disconnected(&mut self, code: &'static str) -> Vec<Effect> {
        self.connected = false;
        self.connecting = false;
        self.paired = false;
        self.welcome = None;
        self.problem = code;
        self.refresh_id = None;
        let mut effects = self.set_level(Level::Off);
        for (_, pending) in std::mem::take(&mut self.pending) {
            effects.push(tool_error(pending.rpc_id, code));
        }
        effects
    }

    pub fn saved_pairing(&mut self) -> Vec<Effect> {
        self.paired = true;
        self.apply_welcome()
    }

    fn apply_welcome(&mut self) -> Vec<Effect> {
        let Some((level, locked)) = self.welcome else {
            return vec![];
        };
        if !self.paired {
            return vec![];
        }
        self.problem = if locked {
            "locked"
        } else if level == Level::Off {
            "off"
        } else {
            ""
        };
        self.set_level(if locked { Level::Off } else { level })
    }

    fn new_id(&mut self) -> u64 {
        // No wrap on an attainable session lifetime (and zero is reserved for neither client).
        self.next_id = self.next_id.checked_add(1).expect("request id exhausted");
        self.next_id
    }

    fn refresh(&mut self) -> Vec<Effect> {
        if !self.paired || self.refresh_id.is_some() {
            return vec![];
        }
        let id = self.new_id();
        self.refresh_id = Some(id);
        // Hide stale tools immediately; the new list comes only from Notaya's answer.
        let mut effects = self.set_level(Level::Off);
        effects.push(Effect::Send(ClientMessage::Call {
            id,
            action: "ai.permissions.get".into(),
            params: json!({}),
            preview: None,
        }));
        effects
    }

    pub fn input(&mut self, input: Value) -> Vec<Effect> {
        let Some(object) = input.as_object() else {
            return vec![rpc_error(Value::Null, -32600, "Invalid Request")];
        };
        let id = object.get("id").cloned();
        if input["jsonrpc"] != "2.0"
            || !input["method"].is_string()
            || id.as_ref().is_some_and(|id| !id_is_valid(id))
        {
            return vec![rpc_error(Value::Null, -32600, "Invalid Request")];
        }
        if input
            .get("params")
            .is_some_and(|params| !params.is_object())
        {
            return id.map_or_else(Vec::new, |id| vec![rpc_error(id, -32602, "Invalid params")]);
        }
        let method = input["method"].as_str().unwrap();
        let params = &input["params"];
        if id.is_none() {
            if method == "notifications/initialized" && self.initialized && !self.ready {
                self.ready = true;
                return self.connect();
            }
            if method == "notifications/cancelled"
                && self.ready
                && let Some((&pipe_id, _)) = self
                    .pending
                    .iter()
                    .find(|(_, pending)| pending.rpc_id == params["requestId"])
            {
                self.pending.remove(&pipe_id);
                return vec![Effect::Send(ClientMessage::Cancel { id: pipe_id })];
            }
            return vec![];
        }
        let id = id.unwrap();
        if method == "initialize" {
            if self.initialized {
                return vec![rpc_error(id, -32600, "Already initialized")];
            }
            let Some(client) = params["clientInfo"]["name"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
            else {
                return vec![rpc_error(id, -32602, "Invalid params")];
            };
            if !params["protocolVersion"].is_string() || !params["capabilities"].is_object() {
                return vec![rpc_error(id, -32602, "Invalid params")];
            }
            self.client = client.into();
            self.initialized = true;
            self.problem = "not_running";
            return vec![rpc_result(
                id,
                json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {"tools":{"listChanged":true}},
                    "serverInfo":{"name":"notaya-bridge","version":env!("CARGO_PKG_VERSION")},
                    "instructions": tools::DATA_NOTICE
                }),
            )];
        }
        if method == "ping" {
            return vec![rpc_result(id, json!({}))];
        }
        if !self.ready {
            return vec![rpc_error(id, -32600, "Initialize the session first")];
        }
        if method == "tools/list" {
            if params.get("cursor").is_some() {
                return vec![rpc_error(id, -32602, "Invalid params")];
            }
            let mut effects = self.connect();
            effects.push(rpc_result(id, tools::list(self.level)));
            return effects;
        }
        if method != "tools/call" {
            return vec![rpc_error(id, -32601, "Method not found")];
        }
        let arguments = &params["arguments"];
        let (Some(tool), Some(action)) = (params["name"].as_str(), arguments["action"].as_str())
        else {
            return vec![rpc_error(id, -32602, "Invalid params")];
        };
        if !tools::routes(tool, action)
            || !arguments["params"].is_object()
            || arguments.get("preview").is_some_and(|v| !v.is_boolean())
            || arguments.as_object().is_none_or(|o| {
                o.keys()
                    .any(|key| !["action", "params", "preview"].contains(&key.as_str()))
            })
        {
            return vec![rpc_error(id, -32602, "Invalid params")];
        }
        if self.pending.values().any(|pending| pending.rpc_id == id) {
            return vec![rpc_error(id, -32600, "Duplicate request id")];
        }
        if self.pending.len() >= MAX_PENDING {
            return vec![tool_error(id, "rate_limited")];
        }
        if !self.connected || !self.paired || !self.problem.is_empty() {
            let mut effects = self.connect();
            effects.push(tool_error(
                id,
                if self.problem.is_empty() {
                    "not_paired"
                } else {
                    self.problem
                },
            ));
            return effects;
        }
        let progress = params["_meta"]["progressToken"].clone();
        if !progress.is_null() && !id_is_valid(&progress) {
            return vec![rpc_error(id, -32602, "Invalid params")];
        }
        let pipe_id = self.new_id();
        self.pending.insert(
            pipe_id,
            Pending {
                rpc_id: id,
                progress: if progress.is_null() {
                    None
                } else {
                    Some(progress)
                },
            },
        );
        vec![Effect::Send(ClientMessage::Call {
            id: pipe_id,
            action: action.into(),
            params: arguments["params"].clone(),
            preview: arguments["preview"].as_bool(),
        })]
    }

    pub fn server(&mut self, message: ServerMessage) -> Vec<Effect> {
        match message {
            ServerMessage::Welcome { level, locked, .. } => {
                self.welcome = Some((level, locked));
                self.apply_welcome()
            }
            ServerMessage::PairingResult { pairing_id, token } => {
                if self.paired {
                    let mut effects = self.disconnected("invalid");
                    effects.push(Effect::Close);
                    return effects;
                }
                vec![Effect::Save(Pairing {
                    pairing_id,
                    token,
                    client: self.client.clone(),
                })]
            }
            ServerMessage::Result { id, ok, value, rev } => {
                if self.refresh_id == Some(id) {
                    self.refresh_id = None;
                    let level = if ok {
                        serde_json::from_value(value["level"].clone()).unwrap_or(Level::Off)
                    } else {
                        Level::Off
                    };
                    self.problem = if level == Level::Off { "off" } else { "" };
                    return self.set_level(level);
                }
                let Some(pending) = self.pending.remove(&id) else {
                    return vec![];
                };
                if !ok {
                    return vec![tool_error(pending.rpc_id, "invalid")];
                }
                let payload = json!({"value":value,"rev":rev});
                vec![rpc_result(
                    pending.rpc_id,
                    json!({"content":[{"type":"text","text":payload.to_string()}],"structuredContent":payload,"isError":false}),
                )]
            }
            ServerMessage::Error {
                id: Some(id), code, ..
            } => {
                if self.refresh_id == Some(id) {
                    self.refresh_id = None;
                    return self.set_level(Level::Off);
                }
                self.pending
                    .remove(&id)
                    .map_or_else(Vec::new, |pending| vec![tool_error(pending.rpc_id, &code)])
            }
            ServerMessage::Error { code, .. } => {
                let code = match code.as_str() {
                    "locked" => "locked",
                    "off" => "off",
                    "stopped" => "stopped",
                    "pairing_refused" => "pairing_refused",
                    "not_paired" => "not_paired",
                    _ => "invalid",
                };
                let mut effects = self.disconnected(code);
                effects.push(Effect::Close);
                effects
            }
            ServerMessage::Event {
                kind: EventKind::LevelChanged,
                level,
                ..
            } => {
                if let Some(level) = level {
                    self.problem = if level == Level::Off { "off" } else { "" };
                    self.set_level(level)
                } else {
                    self.refresh()
                }
            }
            ServerMessage::Event {
                kind: EventKind::Unlocked,
                ..
            } => self.refresh(),
            ServerMessage::Event {
                kind: EventKind::Locked,
                ..
            } => {
                let mut effects = self.disconnected("locked");
                effects.push(Effect::Close);
                effects
            }
            ServerMessage::Event {
                kind: EventKind::Stopped,
                ..
            } => {
                let mut effects = self.disconnected("stopped");
                effects.push(Effect::Close);
                effects
            }
            ServerMessage::Event {
                kind: EventKind::ConfirmPending,
                id,
                ..
            } => {
                let pending: Vec<_> = self
                    .pending
                    .iter()
                    .filter(|(key, _)| id.is_none_or(|id| id == **key))
                    .collect();
                pending.into_iter().filter_map(|(_, pending)| pending.progress.as_ref().map(|token| Effect::Output(json!({
                    "jsonrpc":"2.0","method":"notifications/progress","params":{"progressToken":token,"progress":0,"message":"Waiting for the person to confirm in the Notaya window."}
                })))).collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::{MAX_REQUEST, MAX_RESPONSE, read_frame, write_frame};
    use std::io::Cursor;

    fn output(effects: &[Effect]) -> Vec<Value> {
        effects
            .iter()
            .filter_map(|e| {
                if let Effect::Output(v) = e {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    fn initialized() -> Mcp {
        let mut mcp = Mcp::default();
        let result = output(&mcp.input(json!({"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":PROTOCOL_VERSION,"capabilities":{},"clientInfo":{"name":"Codex","version":"1"}}})));
        assert_eq!(result[0]["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(
            result[0]["result"]["capabilities"]["tools"]["listChanged"],
            true
        );
        mcp.input(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        mcp.connected(true);
        mcp.server(ServerMessage::Welcome {
            notaya_version: "1.6.0".into(),
            level: Level::Edit,
            locked: false,
        });
        mcp
    }
    fn call(id: Value) -> Value {
        json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"notaya_page","arguments":{"action":"page.get","params":{"format":"blocks"}},"_meta":{"progressToken":"p"}}})
    }

    #[test]
    fn fake_pipe_server_handles_calls_progress_cancel_and_exact_text() {
        let mut mcp = initialized();
        let effects = mcp.input(call(json!("request-1")));
        let Effect::Send(message) = &effects[0] else {
            panic!()
        };
        let mut pipe = Vec::new();
        write_frame(&mut pipe, message, MAX_REQUEST).unwrap();
        // Fake Notaya decodes exactly the bytes the real adapter sends.
        let ClientMessage::Call {
            id, action, params, ..
        } = read_frame(&mut Cursor::new(pipe), MAX_REQUEST).unwrap()
        else {
            panic!()
        };
        assert_eq!(action, "page.get");
        assert_eq!(params["format"], "blocks");
        let progress = output(&mcp.server(ServerMessage::Event {
            kind: EventKind::ConfirmPending,
            id: Some(id),
            level: None,
        }));
        assert_eq!(progress[0]["method"], "notifications/progress");
        let text = "\u{200f}العربية\u{2067} ?Deutsch\u{2069}  \n\tcode  ";
        let mut reply = Vec::new();
        write_frame(
            &mut reply,
            &ServerMessage::Result {
                id,
                ok: true,
                value: json!({"text":text}),
                rev: Some(9),
            },
            MAX_RESPONSE,
        )
        .unwrap();
        let result =
            output(&mcp.server(read_frame(&mut Cursor::new(reply), MAX_RESPONSE).unwrap()));
        assert_eq!(result[0]["id"], "request-1");
        assert_eq!(
            result[0]["result"]["structuredContent"]["value"]["text"],
            text
        );
        assert_eq!(result[0]["result"]["structuredContent"]["rev"], 9);
        mcp.input(call(json!(9)));
        let cancellation = mcp.input(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":9,"reason":"private text"}}));
        assert!(matches!(
            cancellation[0],
            Effect::Send(ClientMessage::Cancel { .. })
        ));
        assert!(mcp.pending.is_empty());
    }

    #[test]
    fn level_events_refresh_from_notaya_and_lock_fails_pending_calls() {
        let mut mcp = initialized();
        let effects = mcp.server(ServerMessage::Event {
            kind: EventKind::LevelChanged,
            id: None,
            level: None,
        });
        let id = effects
            .iter()
            .find_map(|e| {
                if let Effect::Send(ClientMessage::Call { id, action, .. }) = e {
                    assert_eq!(action, "ai.permissions.get");
                    Some(*id)
                } else {
                    None
                }
            })
            .unwrap();
        let changed = output(&mcp.server(ServerMessage::Result {
            id,
            ok: true,
            value: json!({"level":"full"}),
            rev: None,
        }));
        assert_eq!(changed[0]["method"], "notifications/tools/list_changed");
        assert_eq!(mcp.level, Level::Full);
        mcp.input(call(json!(1)));
        let locked = output(&mcp.server(ServerMessage::Event {
            kind: EventKind::Locked,
            id: None,
            level: None,
        }));
        assert!(locked.iter().any(|v| v["result"]["isError"] == true));
        assert_eq!(mcp.level, Level::Off);
        assert!(mcp.pending.is_empty());
    }

    #[test]
    fn first_pairing_waits_for_both_persistence_and_welcome() {
        let mut mcp = initialized();
        mcp.disconnected("not_running");
        mcp.connected(false);
        assert!(
            mcp.server(ServerMessage::Welcome {
                notaya_version: "1.6".into(),
                level: Level::Look,
                locked: false
            })
            .is_empty()
        );
        let result = mcp.server(ServerMessage::PairingResult {
            pairing_id: "p1".into(),
            token: "a".repeat(64),
        });
        assert!(output(&result).is_empty());
        assert_eq!(mcp.level, Level::Off);
        assert!(!mcp.saved_pairing().is_empty());
        assert_eq!(mcp.level, Level::Look);
    }

    #[test]
    fn bad_rpc_never_reaches_the_pipe_and_errors_never_echo_notes() {
        let mut mcp = initialized();
        for invalid in [
            json!([]),
            json!({"jsonrpc":"1.0","id":1,"method":"ping"}),
            json!({"jsonrpc":"2.0","id":null,"method":"ping"}),
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"notaya_session","arguments":{"action":"ai_confirm_answer","params":{}}}}),
        ] {
            assert!(
                !mcp.input(invalid)
                    .iter()
                    .any(|e| matches!(e, Effect::Send(_)))
            );
        }
        assert!(
            mcp.input(json!({"jsonrpc":"2.0","method":"unknown"}))
                .is_empty()
        );
        let effects = mcp.input(call(json!(1)));
        let Effect::Send(ClientMessage::Call { id, .. }) = &effects[0] else {
            panic!()
        };
        let result = output(&mcp.server(ServerMessage::Error {
            id: Some(*id),
            code: "unknown private title".into(),
            message: "private note body".into(),
        }));
        assert!(!result[0].to_string().contains("private"));
        let mut capped = Cursor::new(vec![b'x'; MAX_REQUEST + 1]);
        assert!(read_line(&mut capped).is_err());
        assert!(read_line(&mut Cursor::new(b"{}".to_vec())).is_err());
        let mut lines = Cursor::new(b"{}\r\n{}\n".to_vec());
        assert_eq!(read_line(&mut lines).unwrap().unwrap(), b"{}\r\n");
        assert_eq!(read_line(&mut lines).unwrap().unwrap(), b"{}\n");
        assert!(read_line(&mut lines).unwrap().is_none());
    }
}
