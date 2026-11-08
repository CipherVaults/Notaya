//! Dual-era newline-delimited MCP, based on the coordinator's official-doc extracts:
//! modern 2026-07-28 request metadata and legacy <= 2025-11-25 initialization.
//! Modern subscriptions are NOT advertised: the offline extract omits their schema.
use super::{
    pipe::{ClientMessage, EventKind, Pairing, ServerMessage},
    tools::{self, Level},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

pub const PROTOCOL_VERSION: &str = "2026-07-28";
pub const LEGACY_VERSION: &str = "2025-11-25";
const VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
const CAPABILITIES_META: &str = "io.modelcontextprotocol/clientCapabilities";
const CLIENT_META: &str = "io.modelcontextprotocol/clientInfo";
const MAX_PENDING: usize = 200;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Era {
    Legacy,
    Modern,
}

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
    era: Era,
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
    legacy_client: String,
    problem: &'static str,
    next_id: u64,
    pending: BTreeMap<u64, Pending>,
    lists: Vec<Pending>,
    refresh_id: Option<u64>,
}

fn rpc_result(id: Value, mut result: Value, era: Era) -> Effect {
    if era == Era::Modern {
        result["resultType"] = json!("complete");
        result["_meta"] = json!({"io.modelcontextprotocol/serverInfo": server_info()});
    }
    Effect::Output(json!({"jsonrpc":"2.0","id":id,"result":result}))
}
fn server_info() -> Value {
    json!({"name":"notaya-bridge","version":env!("CARGO_PKG_VERSION")})
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
        if bytes.len() + count > super::pipe::MAX_REQUEST {
            return Err(io::ErrorKind::InvalidData.into());
        }
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return Ok(Some(bytes));
        }
    }
}

fn tool_error(id: Value, code: &str, era: Era) -> Effect {
    rpc_result(
        id,
        json!({"isError":true,"content":[{"type":"text","text":safe_error(code)}]}),
        era,
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
            effects.push(tool_error(pending.rpc_id, code, pending.era));
        }
        effects.extend(self.finish_lists());
        effects
    }

    fn finish_lists(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.lists)
            .into_iter()
            .map(|pending| rpc_result(pending.rpc_id, tools::list(self.level), pending.era))
            .collect()
    }

    fn select_client(&mut self, client: &str) -> Vec<Effect> {
        if self.client == client {
            return vec![];
        }
        // A different per-request client identity must never inherit another pairing.
        let effects = if self.client.is_empty() {
            vec![]
        } else {
            let mut effects = self.disconnected("not_paired");
            effects.push(Effect::Close);
            effects
        };
        self.client = client.into();
        self.problem = "not_running";
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
        let mut effects = self.set_level(if locked { Level::Off } else { level });
        effects.extend(self.finish_lists());
        effects
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
                let mut effects = self.select_client(&self.legacy_client.clone());
                effects.extend(self.connect());
                return effects;
            }
            if method == "notifications/cancelled"
                && let Some((&pipe_id, _)) = self
                    .pending
                    .iter()
                    .find(|(_, pending)| pending.rpc_id == params["requestId"])
            {
                self.pending.remove(&pipe_id);
                return vec![Effect::Send(ClientMessage::Cancel { id: pipe_id })];
            }
            if method == "notifications/cancelled" {
                self.lists
                    .retain(|pending| pending.rpc_id != params["requestId"]);
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
            self.legacy_client = client.into();
            self.initialized = true;
            let requested = params["protocolVersion"].as_str().unwrap();
            let version = match requested {
                "2024-11-05" | "2025-03-26" | "2025-06-18" | LEGACY_VERSION => requested,
                _ => LEGACY_VERSION,
            };
            return vec![rpc_result(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools":{"listChanged":true}},
                    "serverInfo":server_info(),
                    "instructions": tools::DATA_NOTICE
                }),
                Era::Legacy,
            )];
        }
        let meta = &params["_meta"];
        let modern = !self.initialized
            || [VERSION_META, CAPABILITIES_META, CLIENT_META]
                .iter()
                .any(|key| meta.get(*key).is_some());
        let (era, client) = if modern {
            let Some(version) = meta[VERSION_META].as_str() else {
                return vec![rpc_error(id, -32602, "Invalid params")];
            };
            if !meta[CAPABILITIES_META].is_object() {
                return vec![rpc_error(id, -32602, "Invalid params")];
            }
            if version != PROTOCOL_VERSION {
                return vec![Effect::Output(json!({
                    "jsonrpc":"2.0", "id":id,
                    "error":{"code":-32022,"message":"UnsupportedProtocolVersion",
                        "data":{"supported":[PROTOCOL_VERSION],"requested":version}}
                }))];
            }
            let client = if let Some(info) = meta.get(CLIENT_META) {
                let Some(name) = info["name"].as_str().filter(|s| {
                    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
                }) else {
                    return vec![rpc_error(id, -32602, "Invalid params")];
                };
                name.to_owned()
            } else {
                "MCP client".into()
            };
            (Era::Modern, client)
        } else {
            (Era::Legacy, self.legacy_client.clone())
        };
        if params.get("_meta").is_some_and(|meta| !meta.is_object()) {
            return vec![rpc_error(id, -32602, "Invalid params")];
        }
        if method == "server/discover" && era == Era::Modern {
            // The offline notes specify discovery, but omit its exact result schema.
            // This minimal payload needs verification against the full revision.
            return vec![rpc_result(
                id,
                json!({
                    "protocolVersions":[PROTOCOL_VERSION],
                    "capabilities":{"tools":{}},
                    "instructions":tools::DATA_NOTICE
                }),
                era,
            )];
        }
        if method == "ping" {
            return vec![rpc_result(id, json!({}), era)];
        }
        if era == Era::Legacy && !self.ready {
            return vec![rpc_error(id, -32600, "Initialize the session first")];
        }
        if !["tools/list", "tools/call"].contains(&method) {
            // No invented subscriptions/listen schema and no unsolicited modern
            // list-changed notifications. Clients can request a fresh tools/list.
            return vec![rpc_error(id, -32601, "Method not found")];
        }
        let mut effects = self.select_client(&client);
        effects.extend(self.request(id, method, params, era));
        effects
    }

    fn request(&mut self, id: Value, method: &str, params: &Value, era: Era) -> Vec<Effect> {
        if self
            .pending
            .values()
            .chain(&self.lists)
            .any(|pending| pending.rpc_id == id)
        {
            return vec![rpc_error(id, -32600, "Duplicate request id")];
        }
        if self.pending.len() + self.lists.len() >= MAX_PENDING {
            if method == "tools/list" {
                return vec![rpc_error(id, -32603, "Too many pending requests")];
            }
            return vec![tool_error(id, "rate_limited", era)];
        }
        if method == "tools/list" {
            if params.get("cursor").is_some() {
                return vec![rpc_error(id, -32602, "Invalid params")];
            }
            if !self.connected || !self.paired || self.welcome.is_none() {
                self.lists.push(Pending {
                    rpc_id: id,
                    progress: None,
                    era,
                });
                return self.connect();
            }
            return vec![rpc_result(id, tools::list(self.level), era)];
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
        if !self.connected || !self.paired || !self.problem.is_empty() {
            let mut effects = self.connect();
            effects.push(tool_error(
                id,
                if self.problem.is_empty() {
                    "not_paired"
                } else {
                    self.problem
                },
                era,
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
                era,
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
                    return vec![tool_error(pending.rpc_id, "invalid", pending.era)];
                }
                let payload = json!({"value":value,"rev":rev});
                vec![rpc_result(
                    pending.rpc_id,
                    json!({"content":[{"type":"text","text":payload.to_string()}],"structuredContent":payload,"isError":false}),
                    pending.era,
                )]
            }
            ServerMessage::Error {
                id: Some(id), code, ..
            } => {
                if self.refresh_id == Some(id) {
                    self.refresh_id = None;
                    return self.set_level(Level::Off);
                }
                self.pending.remove(&id).map_or_else(Vec::new, |pending| {
                    vec![tool_error(pending.rpc_id, &code, pending.era)]
                })
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
                    .filter(|(key, pending)| {
                        pending.era == Era::Legacy && id.is_none_or(|id| id == **key)
                    })
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
    use super::super::pipe::{MAX_REQUEST, MAX_RESPONSE, read_frame, write_frame};
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
        let result = output(&mcp.input(json!({"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":LEGACY_VERSION,"capabilities":{},"clientInfo":{"name":"Codex","version":"1"}}})));
        assert_eq!(result[0]["result"]["protocolVersion"], LEGACY_VERSION);
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

    fn modern(mut request: Value) -> Value {
        request["params"]["_meta"][VERSION_META] = json!(PROTOCOL_VERSION);
        request["params"]["_meta"][CAPABILITIES_META] = json!({});
        request["params"]["_meta"][CLIENT_META] = json!({"name":"Codex","version":"1"});
        request
    }

    fn fake_reply(mcp: &mut Mcp, message: ServerMessage) -> Vec<Effect> {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &message, MAX_RESPONSE).unwrap();
        mcp.server(read_frame(&mut Cursor::new(bytes), MAX_RESPONSE).unwrap())
    }

    fn fake_call(effects: &[Effect]) -> u64 {
        let message = effects
            .iter()
            .find_map(|effect| match effect {
                Effect::Send(message) => Some(message),
                _ => None,
            })
            .unwrap();
        let mut bytes = Vec::new();
        write_frame(&mut bytes, message, MAX_REQUEST).unwrap();
        let ClientMessage::Call { id, action, .. } =
            read_frame(&mut Cursor::new(bytes), MAX_REQUEST).unwrap()
        else {
            panic!("expected tool call");
        };
        assert_eq!(action, "page.get");
        id
    }

    fn assert_modern(result: &Value) {
        assert_eq!(result["resultType"], "complete");
        assert_eq!(
            result["_meta"]["io.modelcontextprotocol/serverInfo"],
            server_info()
        );
    }

    #[test]
    fn modern_discover_and_fake_pipe_work_without_initialization() {
        let mut mcp = Mcp::default();
        let discover =
            modern(json!({"jsonrpc":"2.0","id":1,"method":"server/discover","params":{}}));
        let result = output(&mcp.input(discover));
        assert_modern(&result[0]["result"]);
        assert_eq!(
            result[0]["result"]["protocolVersions"],
            json!([PROTOCOL_VERSION])
        );
        assert!(
            result[0]["result"]["capabilities"]["tools"]
                .get("listChanged")
                .is_none()
        );
        assert!(!mcp.initialized && !mcp.ready && !mcp.connected);

        let effects = mcp.input(modern(
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        ));
        assert!(matches!(&effects[0], Effect::Connect(name) if name == "Codex"));
        assert!(
            output(&effects).is_empty(),
            "wait for the authenticated tool list"
        );
        mcp.connected(true);
        let result = output(&fake_reply(
            &mut mcp,
            ServerMessage::Welcome {
                notaya_version: "fake".into(),
                level: Level::Look,
                locked: false,
            },
        ));
        assert_eq!(result.len(), 1, "no unsolicited legacy list-changed");
        assert_eq!(result[0]["id"], 2);
        assert_modern(&result[0]["result"]);
        assert_eq!(result[0]["result"]["tools"].as_array().unwrap().len(), 9);

        let id = fake_call(&mcp.input(modern(call(json!(3)))));
        assert!(
            fake_reply(
                &mut mcp,
                ServerMessage::Event {
                    kind: EventKind::ConfirmPending,
                    id: Some(id),
                    level: None,
                }
            )
            .is_empty(),
            "no undocumented modern progress notification"
        );
        let exact = "\u{200f}\u{645}\u{644}\u{627}\u{62d}\u{638}\u{627}\u{62a}  \n\tcode  ";
        let result = output(&fake_reply(
            &mut mcp,
            ServerMessage::Result {
                id,
                ok: true,
                value: json!({"text":exact}),
                rev: Some(7),
            },
        ));
        assert_eq!(
            result[0]["result"]["structuredContent"]["value"]["text"],
            exact
        );
        assert_modern(&result[0]["result"]);

        let id = fake_call(&mcp.input(modern(call(json!(4)))));
        let cancelled = mcp.input(
            json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":4}}),
        );
        assert!(
            matches!(cancelled[0], Effect::Send(ClientMessage::Cancel { id: actual }) if actual == id)
        );
        assert!(
            fake_reply(
                &mut mcp,
                ServerMessage::Result {
                    id,
                    ok: true,
                    value: json!({}),
                    rev: None
                }
            )
            .is_empty()
        );
        assert!(
            fake_reply(
                &mut mcp,
                ServerMessage::Event {
                    kind: EventKind::LevelChanged,
                    id: None,
                    level: Some(Level::Edit)
                }
            )
            .is_empty()
        );
        let result = output(&mcp.input(modern(
            json!({"jsonrpc":"2.0","id":5,"method":"tools/list","params":{}}),
        )));
        assert_eq!(result[0]["result"]["tools"].as_array().unwrap().len(), 10);
        let result = output(&mcp.input(modern(
            json!({"jsonrpc":"2.0","id":6,"method":"subscriptions/listen","params":{}}),
        )));
        assert_eq!(result[0]["error"]["code"], -32601);
    }

    #[test]
    fn modern_metadata_errors_never_connect_to_the_fake_pipe() {
        let mut mcp = Mcp::default();
        for params in [
            json!({}),
            json!({"_meta":{}}),
            json!({"_meta":{VERSION_META:PROTOCOL_VERSION}}),
            json!({"_meta":{CAPABILITIES_META:{}}}),
            json!({"_meta":{VERSION_META:PROTOCOL_VERSION,CAPABILITIES_META:[]}}),
        ] {
            let effects = mcp
                .input(json!({"jsonrpc":"2.0","id":1,"method":"server/discover","params":params}));
            assert_eq!(effects.len(), 1);
            assert_eq!(output(&effects)[0]["error"]["code"], -32602);
        }
        let mut request = modern(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}));
        request["params"]["_meta"][VERSION_META] = json!("2099-01-01");
        let effects = mcp.input(request.clone());
        assert_eq!(effects.len(), 1);
        let result = output(&effects);
        assert_eq!(result[0]["error"]["code"], -32022);
        assert_eq!(
            result[0]["error"]["data"],
            json!({"supported":[PROTOCOL_VERSION],"requested":"2099-01-01"})
        );
        assert!(!mcp.connecting && !mcp.initialized);
        // Selecting legacy semantics must not bypass modern validation.
        let mut legacy = initialized();
        assert_eq!(output(&legacy.input(request))[0]["error"]["code"], -32022);
        let missing = json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{"_meta":{VERSION_META:PROTOCOL_VERSION}}});
        assert_eq!(output(&legacy.input(missing))[0]["error"]["code"], -32602);
    }

    #[test]
    fn mixed_era_pipe_replies_retain_each_requests_envelope() {
        let mut mcp = initialized();
        let modern_id = fake_call(&mcp.input(modern(call(json!(1)))));
        let legacy_id = fake_call(&mcp.input(call(json!(2))));
        let result = output(&fake_reply(
            &mut mcp,
            ServerMessage::Result {
                id: legacy_id,
                ok: true,
                value: json!({}),
                rev: None,
            },
        ));
        assert!(result[0]["result"].get("resultType").is_none());
        let result = output(&fake_reply(
            &mut mcp,
            ServerMessage::Error {
                id: Some(modern_id),
                code: "private".into(),
                message: "private note".into(),
            },
        ));
        assert_modern(&result[0]["result"]);
        assert!(!result[0].to_string().contains("private"));
        let modern_id = fake_call(&mcp.input(modern(call(json!(3)))));
        let result = output(&fake_reply(
            &mut mcp,
            ServerMessage::Result {
                id: modern_id,
                ok: true,
                value: json!({}),
                rev: None,
            },
        ));
        assert_modern(&result[0]["result"]);
        let result = output(&mcp.input(json!({"jsonrpc":"2.0","id":4,"method":"ping"})));
        assert_eq!(result[0]["result"], json!({}));
    }

    #[test]
    fn changing_modern_client_drops_the_old_pairing_and_pending_requests() {
        let mut mcp = initialized();
        mcp.input(modern(call(json!(1))));
        let mut request = modern(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}));
        request["params"]["_meta"][CLIENT_META]["name"] = json!("Claude Code");
        let effects = mcp.input(request);
        assert!(effects.iter().any(|effect| matches!(effect, Effect::Close)));
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, Effect::Connect(name) if name == "Claude Code"))
        );
        let result = output(&effects);
        let failed = result.iter().find(|value| value["id"] == 1).unwrap();
        assert_modern(&failed["result"]);
        assert_eq!(mcp.level, Level::Off);
        assert!(!mcp.paired && mcp.pending.is_empty());
        let result = output(&mcp.disconnected("not_running"));
        assert_eq!(result[0]["id"], 2);
        assert_modern(&result[0]["result"]);
        assert_eq!(result[0]["result"]["tools"], json!([]));
    }

    #[test]
    fn legacy_negotiates_earlier_versions_and_waits_for_initialized() {
        for (requested, expected) in [
            ("2024-11-05", "2024-11-05"),
            ("2025-03-26", "2025-03-26"),
            ("2025-06-18", "2025-06-18"),
            (LEGACY_VERSION, LEGACY_VERSION),
            ("unknown", LEGACY_VERSION),
        ] {
            let mut mcp = Mcp::default();
            let result = output(&mcp.input(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":requested,"capabilities":{},"clientInfo":{"name":"Codex"}}})));
            assert_eq!(result[0]["result"]["protocolVersion"], expected);
            assert!(result[0]["result"].get("resultType").is_none());
            assert_eq!(
                output(&mcp.input(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})))[0]["error"]
                    ["code"],
                -32600
            );
            let effects = mcp.input(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
            assert!(matches!(&effects[0], Effect::Connect(name) if name == "Codex"));
            mcp.connected(true);
            let changed = output(&fake_reply(
                &mut mcp,
                ServerMessage::Welcome {
                    notaya_version: "fake".into(),
                    level: Level::Look,
                    locked: false,
                },
            ));
            assert_eq!(changed[0]["method"], "notifications/tools/list_changed");
        }
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
