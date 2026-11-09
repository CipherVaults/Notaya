# Codex security review guide

- **Who and where:** Ahmed's side (`AE0101`). Written by Claude in the cloud thread "Build Notaya 1.6" (machine: cloud container), 2026-11-09, about 07:10 UTC (system clock).
- **Asked by:** Ahmed, project chat 2026-11-09 06:55:20 UTC: "do not forget security ... we do not want weak code, vulnerable designs ... perhaps you can even transfer the skills to Codex with a version that is suitable for Codex". Recorded as [A49](../plan/approval-and-amendments.md), work package WP-17 of the [1.6 spec](../plan/2026-11-08-v1-6-spec.md).
- **Result state:** *produced*. It is a method, not a finding. It is a port of the approach Claude uses for its built-in `security-review` (A "real, reachable, provable" test for each finding, a fixed checklist, a fixed report shape), shortened and made specific to Notaya. If this file and `docs/security-design.md` differ, the security design wins and this file needs a dated correction.
- **Read first:** `AGENTS.md`, [`codex-partner-briefing.md`](codex-partner-briefing.md), then [`docs/security-design.md`](../security-design.md) (trust boundaries, promises P1 and up, known limitations). Do not review code you have not read in full.

## 1. When you use this

1. **Every lane you build:** before you hand a lane back, review your own diff once with section 4 and put the result in the hand-back note ("security self-check: what I looked at, what I found, what I did not run"). A self-review is **not independent**; it only catches the cheap things.
2. **Lane S (security pass, WP-17):** the independent pass. It runs in a **fresh session that did not write the code under review**. Start from a clean checkout of the exact commit, say the commit hash at the top of the findings.
3. **Always:** a finding must be a real, reachable problem. Do not pad the report. "No findings" with a clear list of what was checked is a good result.

## 2. How to review (the method)

1. **Map the attack surface of the diff or area.** List every place data enters from outside the Rust core: Tauri commands a window can call, the AI pipe and bridge frames, files imported or restored, the clipboard, command line and environment, the vault and backup files on disk. Note who can reach each one: any window, only the unlocked app, only a paired AI connection at a given level.
2. **Follow each input to where it is used.** Write down the path (entry, checks, sink). A sink is a file path, a SQL string, a crypto call, a log or error message, a window event, a process start, an HTML render, a size used for allocation.
3. **Ask what an attacker controls.** The attacker may be: a hostile file (import, restore, Notion zip, image), a script injected into the screen layer, a malicious or confused AI connection, another program on the PC (same user), another Windows user, someone with the vault file but no password. Malware running as the same user with the vault unlocked is **out of scope** (see the security design); say so rather than reporting it.
4. **Prove it.** For each suspected problem, write a **failing test or a small reproduction on throwaway data** (section 6). If you cannot reach it or cannot reproduce it, it is a *question* for Claude, not a finding. Never test on a real vault.
5. **Rate it** (section 5), propose the smallest fix that keeps the design, and say how a regression test would catch it coming back.

## 3. What does **not** count

Report none of these as findings: style, theoretical problems with no input that reaches them, missing hardening that the security design already lists as a known limitation, denial of service by someone who already has the unlocked app, dependency advisories that do not touch a reachable code path (list them in a separate "advisories" line instead), anything that needs malware on the PC while the vault is unlocked, and "could be more secure" with no concrete attack. If you doubt a claim, mark it **unproven** and say what would prove it.

## 4. Checklist for Notaya (check each; write "n/a" when it does not apply)

**Secrets and keys**
- Keys, tokens, the master password and keyfile bytes stay in the Rust core. The screen layer gets only what it displays. No secret in a command result, an event, a log line, an error text, a file name or a panic message.
- Secrets are zeroized on lock where the type allows it (`zeroize`, `secrecy`); no secret is `Clone`d into an unprotected long-lived buffer; no secret is compared with `==` (use a constant-time compare); tokens are stored hashed (the AI pairing tokens are).
- No home-made crypto. Only the audited crates already in use (Argon2id, the AEAD in `crypto`); no nonce reuse; every record authenticated; the authenticated data names the vault format (`"notaya"`).
- Random values (ids, tokens, nonces) come from the OS random source, never from a time or counter.

**Commands and IPC (the screen is a trust boundary: assume a script can call any command)**
- Every new Tauri command is listed, validated (types, lengths, ranges, enum values) and checked for who may call it; commands that change security settings, restore a vault or delete data need their existing confirmations in Rust, not only in the dialog.
- Anything the AI can call goes through the one action registry and its level check; the AI can never change its own permissions or reach a command outside its level; destructive actions are window-confirmed, go to the recycle bin and are undoable; a locked note or a hidden notebook is not readable, searchable or addressable by id from an AI call.
- The window-only confirmation cannot be answered by the AI path, a script, or a replay (check the confirmation id is single use, bound to the exact action and parameters, and expires).

**Files, paths and parsing (hostile input)**
- Paths from a file picker, an archive entry or an import are normalized and kept inside their folder (no `..`, no absolute paths, no drive letters, no device names, no symlink or junction escape, no alternate data streams). Zip entries: size and count limits, no zip-slip, no zip bomb (limit the expanded size while reading, not after).
- Sizes read from a file or a frame are checked **before** allocating (frame length, image size, row counts, string lengths). A malformed input returns an error; it never panics, hangs or exhausts memory.
- Images are sniffed by bytes; SVG and active content are rejected; HTML from an import or Word/Markdown is sanitized before it reaches the editor; links open only as http(s) on an explicit action.
- Text that reaches a SQL string, a file name, a shell or a log is escaped or parameterized. We use parameterized queries; flag any string-built query.

**The AI pipe and bridge**
- The pipe accepts only the current user (ACL), rejects remote clients, has a frame size limit and a time limit on the handshake, and handles many connections without starving the app.
- Pairing: the code is single use, short lived and shown only in the Notaya window; wrong tokens are rate limited; revoking a pairing cuts live connections at once; nothing sensitive is in the welcome frame; the bridge never logs tokens.
- Events and error texts sent to an AI connection contain no note content beyond what that level allows and no real names ("the person", not a name).

**Storage, history and logs**
- History snapshots and the activity log are encrypted with the vault key; the log is append-only and hash-chained and "Check the log" detects an edited, removed or reordered row; log text is sanitized and never contains note content.
- Any cache of decrypted values (for example the speed work's row windows) lives only in RAM, is keyed so one database cannot read another's, is dropped on lock and on write, and is never written to disk, a temp file or a crash dump.
- Old vaults (`.clf`, earlier `.nota`) still open; a restore never lowers a vault's security without the existing checks.

**Front end**
- The strict CSP is unchanged (no remote scripts, styles, images or connections). No `dangerouslySetInnerHTML` or `innerHTML` with data that is not sanitized; no `eval`, `new Function`, or string timers; no `postMessage` without an origin check.
- Nothing secret is stored in `localStorage`, in a URL, or in the DOM; screenshots protection and auto-lock behaviour are unchanged.

**Dependencies and build**
- Versions are exact and stable (no alpha, beta, rc, nightly); `npm audit` is clean; a new crate or package is permissively licensed, its credit is in `THIRD-PARTY-NOTICES.md`, it runs only in the layer it was added for, and it adds no network code. `cargo audit` is run where it can be (report "not run" otherwise).
- No secrets, vault data or password exports in the diff, the tests, the fixtures or the logs.

## 5. Severity

| Level | Meaning | What happens |
|---|---|---|
| **High** | Reaches the vault contents or a key, or lets an AI connection or hostile file do something it must not (read a locked note, change its own permissions, delete without confirmation, escape a folder, run code) | Stop: fix before anything merges; Claude reads the whole fix |
| **Medium** | Weakens a promise or a boundary but needs an unusual step, or leaks metadata or note text into a log or error | Fix in a bounded lane before 1.6 closes, or list for Ahmed with the reason |
| **Low** | Hardening, a missing limit with no reachable harm, a clearer error | Fix when passing, or list |
| **Question** | Might be a problem, not proven | Send to Claude with what would prove it |

## 6. Testing directly (throwaway data only)

- Use a throwaway vault in `D:\Notaya Tests` with `NOTAYA_HOME` pointing at it; never a real vault, never Mohamed's `E:` drive, never a live window. Close any debug port afterwards and confirm nothing listens (FI-01).
- Prefer **Rust unit tests** and the existing check scripts. Try, for each area: oversize and truncated frames, wrong and replayed tokens, a second pairing, a revoked pairing, a call above the connection's level, a locked note by id, a hidden notebook by search, path tricks in names, a zip with `..` entries and a huge expanded size, an image that lies about its type, an old `.clf` file, a log row edited by hand, many connections at once.
- Never click a confirmation dialog as a fake person; confirmations are answered only by a person in the Notaya window. A check that needs one is a person-only step on the whole-app sheet.
- Write what you ran, on which exact commit, and what did **not** run. "Not run" is never "passed".

## 7. Report shape (per area, in `docs/plan/2026-11-09-security-pass-findings.md`)

```
Area: <name>          Commit reviewed: <hash>          Reviewer session: <fresh, did not write this code>
Checked: <the checklist items you covered, and the files you read in full>
Not checked / not run: <honest list>
Findings:
  1. [High|Medium|Low] <one-line title>
     Where: <file:line>   Reachable by: <who, and how>
     Proof: <failing test name or reproduction steps on throwaway data>
     Fix: <smallest change that keeps the design>   Regression: <the test that guards it>
Questions: <unproven items for Claude>
Advisories: <dependency notes with no reachable path>
```

Claude reads every finding and the code around it, decides what is real, and makes the final security call. A fix that changes a security behaviour people can see is listed for Ahmed before it is built (A37 item 3). Nothing is weakened to make a test pass. Update [`docs/security-design.md`](../security-design.md) when a promise, boundary or limit changes.

## 8. Rules that never bend

- Never paste a token, password or key into a prompt, record, commit or test fixture. Use generated throwaway values.
- A security review by the agent that wrote the diff is **not independent**; say so.
- Do not fix and report in the same breath: findings first, then a bounded fix lane after Claude has read them.
- If you find something serious outside your area, stop and tell Claude at once with what you saw; do not test it further on anything real.
