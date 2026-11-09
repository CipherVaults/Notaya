# C5 PC smoke check

Run from `D:\Notaya-lanes\C5` on Ahmed's Gaming-PC. No installs, network,
real vault, credentials copied from another run, or Git writes are needed.
Each script run creates a new `run-<guid>` under
`D:\Notaya Tests\v1-6-C5-smoke`, sets both `NOTAYA_HOME` and
`CIPHERLEAF_HOME` there, and restores the caller's environment afterwards.
It preserves all test folders. Do not attach private notes to this check.

## 1. Core and bridge transport

```powershell
powershell -NoProfile -File .\tools\ai-smoke\run-pc-check.ps1 -Mode Core -NativeBridgeTests
```

The core test starts the real `AppState` and a new open throwaway vault. It
loads lane D's **actual** MCP, wire and credential-file code, drives MCP
initialization, and crosses in-memory length-prefixed byte pipes into C5's
actual wire decoder and dispatcher. It checks:

- simulated window-approved pairing, Look default, hash-only app storage,
  bridge credential save/load and saved-token authentication;
- a page read, Look refusing an edit, and a simulated person raising it to Edit;
- the same `page.trash` edit refused by its confirmation with the page and
  revision unchanged, then refused while a synthetic person lease is active
  **before** requesting another confirmation, then confirmed and persisted;
- an injected change-sink panic cannot turn the successful AI write into a
  failure; retry carries AI attribution and callbacks can reacquire state;
- vault reopen retains the edit and pairing.

The fake connection uses the real confirmation broker, with a test callback
standing in for a window. Its Hello/Auth handling is a test adapter, not the
production connection loop. It does **not** prove the native C5 server, its
ACLs, first-instance protection, remote-client rejection, the window commands,
tray or dialogs. The intentionally caught panic prints a panic line with
`--nocapture`; the final test result determines success.

`-NativeBridgeTests` additionally runs lane D's Windows native transport tests
including its two normally ignored fake-server tests. They exercise real
named-pipe I/O and server-image checking, **not** the C5 server. Omit this switch
inside the lane sandbox, where named-pipe opens may be denied.

## 2. Real app and bridge

This needs a matching built app **with lane E's AI access, pairing and
confirmation screens integrated**. C5 alone has the commands/events, not
those screens. Do not bypass this prerequisite by injecting approval through
the pipe or enabling a debug endpoint. Claude builds the matching binaries
offline (the smoke config uses the already built frontend without running a
second frontend build):

```powershell
npm run build -- --configLoader native
cargo build --manifest-path .\src-tauri\Cargo.toml --offline --release -p notaya-bridge
npm run tauri build -- --no-bundle --config (Resolve-Path .\tools\ai-smoke\tauri.smoke.conf.json).Path -- --offline
powershell -NoProfile -File .\tools\ai-smoke\run-pc-check.ps1 -Mode Live `
  -AppExe "$env:CARGO_TARGET_DIR\release\Notaya.exe" `
  -BridgeExe "$env:CARGO_TARGET_DIR\release\notaya-bridge.exe"
```

Use the actual app output path if its build names the executable differently.
No existing Notaya/Cipherleaf process may be running: single-instance guards
could otherwise hand off to its real window. The script refuses to start in
that case and never stops or sends input to an existing process. Close it
yourself first. The script copies only the two matching executables beside
each other in its fresh test folder, so the bridge's normal process-image
verification stays enabled. Use a self-contained built app, not an exe that
requires a development server. No WebView2 debug port is enabled.

Follow the console prompts in the **new throwaway window**: start empty notes,
create exactly one page titled `C5 smoke`, enable AI with typed `ALLOW`, allow
the bridge pairing, raise only `C5 PC smoke` to Edit, deny the first page-to-bin
request, type steadily in that page during the script's 20-second preview
poll for the person-editing refusal (a caret alone does not acquire a lease), then
leave the editor for Settings and wait at least four seconds before allowing
the final request. Confirm it in the bin and restore it; the script reads it
back. Confirmation replies come solely from the window. The bridge consumes
MCP over stdio and uses its real authenticated local pipe. MCP request waits
are bounded to 310 seconds, including the app's five-minute confirmation wait.

The script checks the denied edit's revision and the error codes. It ends with
`CHECKPOINT:` lines only after assertions pass, closes its own bridge by PID,
and asks for Quit from the test app's tray. If the test app is still running,
it reports that PID: quit it before another run. Data stays in the test folder.
No existing app process is killed. Record the source commit, working-tree
status, executable hashes, script result and visual observations separately.

## Checks to report

Claude's heartbeat must rerun the full checks on the eventual merge commit:

```powershell
# Set NOTAYA_HOME and CIPHERLEAF_HOME to a fresh throwaway folder first.
cargo test --manifest-path .\src-tauri\Cargo.toml --lib --offline
npx tsc --noEmit -p tsconfig.json
npm run build -- --configLoader native
```

The sandbox's startup-registry test may fail with Windows code 5. Report that
failure; do not suppress it or change Windows security to make the run green.
The coordinator runs audit and Claude's independent security review. A core
pass is not a live-app pass, and neither is partner acceptance. Wider WP-06 PC
cases (Off/quit teardown, lock/Stop, removal, simultaneous AIs, hidden notebooks,
ACL attack attempts and every registered action) remain a separate round.

## Correction log

- 2026-11-09: the Live script (`run-pc-check.ps1 -Mode Live`) was corrected. Its first Live attempt on a real PC failed right after the level change because the script asked the tool `notaya_notes` for the action `page.recent`; that action belongs to the tool `notaya_page`, so Notaya correctly refused the request and the script stopped with "MCP call did not succeed". The script now uses `notaya_page`, and a failed MCP call now prints the JSON-RPC error or the tool error code and message (never note content). The corrected script then passed all five live checkpoints (pairing, page read, denied edit with unchanged revision, person-editing refusal, confirmed edit and restored page read).
