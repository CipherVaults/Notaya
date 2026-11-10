# Official-docs findings for lane D (fetched by Claude on 2026-11-08 ~00:45 UTC; Codex has no network)

Sources:
- MCP specification, latest = revision 2026-07-28: https://modelcontextprotocol.io/specification/latest (and /2026-07-28/basic/index, /basic/versioning, /basic/transports, /basic/transports/stdio)
- Claude Code MCP docs: https://code.claude.com/docs/en/mcp
- Codex MCP docs: https://learn.chatgpt.com/docs/extend/mcp?surface=cli (developers.openai.com/codex/mcp redirects there)

## MCP 2026-07-28 ("modern") vs <= 2025-11-25 ("legacy")
- Modern: STATELESS. No `initialize` handshake. Every request carries in `params._meta`:
  `io.modelcontextprotocol/protocolVersion` (required, e.g. "2026-07-28"), `io.modelcontextprotocol/clientCapabilities` (required), `io.modelcontextprotocol/clientInfo` (optional). A request missing a required field -> JSON-RPC error -32602.
  Servers MUST implement `server/discover` (returns supported versions etc.). Unsupported version -> error code -32022 with `data: {"supported": [...], "requested": "..."}`. Other new codes: -32020 HeaderMismatch, -32021 MissingRequiredClientCapability (data.requiredCapabilities). Do not emit other codes in -32020..-32099.
  Result responses MUST include `resultType` ("complete" normally; "input_required" for multi round-trip). Servers SHOULD put `io.modelcontextprotocol/serverInfo` in result `_meta`. Servers do not initiate JSON-RPC requests; clients do not send JSON-RPC responses.
  List-changed style notifications are delivered through the long-lived request `subscriptions/listen` (notifications carry `_meta["io.modelcontextprotocol/subscriptionId"]`). Check docs basic/patterns/subscriptions before implementing.
  Cancellation on stdio: client sends `notifications/cancelled` (see basic/patterns/cancellation).
- Legacy (2025-11-25 and earlier): `initialize` request (client sends protocolVersion, capabilities, clientInfo; server answers with its protocolVersion, capabilities, serverInfo), then client sends `notifications/initialized`; `notifications/tools/list_changed` is a server-to-client notification once initialized.
- Dual-era server (what we want, because current Claude Code and Codex releases may still speak the legacy handshake): a request carrying modern `_meta` is served statelessly; an `initialize` request selects legacy semantics for that stdio process. A dual-era server MAY serve both.
- stdio framing: newline-delimited JSON-RPC, UTF-8, one message per line, no embedded newlines; stdout carries ONLY MCP messages; logs go to stderr. Custom byte-stream transports should reuse this framing.
- Tools JSON Schema default dialect is 2020-12.

## Claude Code registration
- CLI: `claude mcp add --transport stdio <name> -- <command> [args...]` (options before `--`; `--env KEY=value` may repeat; `--scope local|project|user`).
- JSON (`.mcp.json` / `claude mcp add-json`): `{"mcpServers": {"notaya": {"type": "stdio", "command": "C:\\Path\\notaya-bridge.exe", "args": [], "env": {}}}}`.
- Check with `claude mcp list` / `claude mcp get <name>`.

## Codex registration
- CLI: `codex mcp add <name> [--env K=V] -- <command> [args...]`; verify with `codex mcp list`.
- `~/.codex/config.toml`:
  ```toml
  [mcp_servers.notaya]
  command = "C:\\Path\\notaya-bridge.exe"
  args = []
  [mcp_servers.notaya.env]
  KEY = "value"
  ```
  Optional keys: `env_vars` (names to forward), `cwd`.
