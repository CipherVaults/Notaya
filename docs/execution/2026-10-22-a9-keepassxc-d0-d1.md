# Execution Summary: EX-2026-10-22-02 — A9: bundle KeePassXC (phases D0 and D1) and first launch

## 1. Task Overview

**Task requested:** Approve and carry out amendment A9: bundle the official KeePassXC 2.8.0 for passwords and retire Cipherleaf's own password section; then build and launch the app.

**Execution mode:** Direct project. Claude executes; the owner decides.

**Project / workspace:** Cipherleaf, `E:\Project\cipherleaf`, branch `feat/a9-keepassxc`

**Executor / contributors:** Claude in Claude Code (Opus 5.5 until the owner switched the model; Sonnet 5.5 for the resumed D1 UI work).

**Date / time:** 2026-10-22, from 13:08 UTC (approval) until about 16:10 UTC, with a break while the owner was away. Early times come from the session transcript; the later wall-clock estimates in this file were off by about 1.5 hours, as the system clock showed 16:07 UTC at the restart.

**Authority / task reference:** the owner's instruction at 13:07:49 UTC (A9 approved, pause lifted, downloads allowed). See [`../plan/approval-and-amendments.md`](../plan/approval-and-amendments.md). A pause for the owner's usage limit came at about 13:45 UTC; "continue" followed.

**Scope and stopping condition:** D0 and D1 of A9, then the first build and launch. Stop for the owner's hands-on review. A release build isn't included.

## 2. Context Used

**Inputs provided:** the approved plan, A9 ([`../plan/A9-bundle-keepassxc.md`](../plan/A9-bundle-keepassxc.md)), and the previous Execution Summary.

**Documents actually read:** the A9 proposal; `docs/STATUS.md`; the bundled KeePassXC user guide (command-line options, settings locations); KeePassXC's `Config.cpp` for setting names and defaults; keepassxc.org's signature-verification page.

**Assumptions made:** the PC already has the Microsoft Visual C++ runtime, which holds true on this PC (v14.52).

**Re-alignment notes:** the repository was clean at commit `b7c8626` on resume, matching `docs/STATUS.md`.

## 3. Work Performed

**Actions taken:**

1. Created branch `feat/a9-keepassxc` and committed the baseline (`f26c1f7`, 89 files). Re-ran the checks on it: 27/27 tests and a clean type check.
2. **D0 (commit `0d57ad2`):** downloaded `KeePassXC-2.8.0-x64.zip` and the source archive, verified their SHA-256 digests and PGP signatures, and recorded the findings in `vendor/keepassxc-manifest.md`. Tested the `keepassxc-cli` tool on a throwaway database and launched KeePassXC GUI with the preset settings.
3. **D1 backend (commit `b7c8626`):** added `keepassxc.rs` and its commands; removed the login, clipboard, breach-check and CSV code and the dependencies they used.
4. **D1 UI (commit `2743021`):** replaced the password and generator pages with `KeePassXCPanel.tsx`; removed login types, store state, IPC calls, command-palette actions and two settings; added KeePassXC notes to Settings → About.
5. Updated `docs/security-design.md` for A9.
6. Built the frontend (`npm run build`) and ran `npm run tauri dev` for the first launch.
7. Restarted the dev app once with WebView2's local debug port to read the first screen. Detail in section 6.

**Files / artifacts created:** `src-tauri/src/keepassxc.rs`, `src-tauri/src/commands/keepassxc.rs`, `src/features/passwords/KeePassXCPanel.tsx`, `vendor/keepassxc-manifest.md`, and downloaded binaries in the ignored `vendor/` folder.

**Files / artifacts modified:** `portable.rs`, `state.rs`, `lib.rs`, `model.rs`, `generator.rs`, `commands/{mod,tools}.rs`, `vault/mod.rs` (tests), `Cargo.toml`, `Cargo.lock`, `Shell.tsx`, `CommandPalette.tsx`, `SettingsView.tsx`, `ipc.ts`, `types.ts`, `store/app.ts`, `.gitignore`, and the docs.

**Files / artifacts deleted:** `commands/logins.rs`, `clipboard.rs`, `src/features/passwords/{PasswordsView,LoginForm,TotpRing}.tsx`, `src/features/generator/GeneratorView.tsx`. All are in Git history (`f26c1f7`).

**External state changes:** downloads from github.com and keys.openpgp.org; a throwaway GPG keyring and test databases in the temp folder (deleted or left in `%TEMP%`); a KeePassXC test window was opened and closed.

**Save / publication / readback:** all work is committed locally (no remote). The security-design edit and this record are not yet committed at the time of writing; see section 8.

## 4. Commands / Tools Used

```text
git switch -c feat/a9-keepassxc ; git add / git commit
Invoke-WebRequest <github release assets>      (downloads)
gpg --import / --verify                        (Git for Windows gpg, throwaway keyring)
Get-FileHash ; dumpbin /dependents
keepassxc-cli db-create | db-info | mkdir | ls      (throwaway database)
KeePassXC.exe --config <ini> --localconfig <ini> ; KeePassXC.exe --lock
cargo test --lib ; npx tsc --noEmit -p tsconfig.json ; npm run build ; npm run tauri dev
node cdp.mjs look   (Chrome DevTools Protocol, read-only, dev build only)
```

**Execution context:** Windows 11 Pro 10.0.26300; Rust 1.99.1; Node 24.20.0; WebView2 155.0.4283.62; KeePassXC 2.8.0 x64.

## 5. Verification / Testing

| Check | Subject | Outcome | Limits |
|---|---|---|---|
| `cargo test --lib` | Rust source at commit `2743021` | **Pass**, 27/27 (this run came after the backend change; same count, three tests replaced) | Unit level only |
| `npx tsc --noEmit -p tsconfig.json` | `src/` at commit `2743021` | **Pass**, exit 0, no errors | Not `vite.config.ts`; not runtime |
| `npm run build` | Frontend bundle | **Pass** (27.6 s). Bundle ~1.39 MB JS (432 kB gzipped) | One large chunk |
| `npm run tauri dev` | First launch | **Pass**: compiled (2 m 04 s) and started; a window titled "Cipherleaf" is visible | Only the first screen was checked |
| KeePassXC ZIP and source: SHA-256 | vs. official `.DIGEST` | **Pass** | Hash identity only |
| KeePassXC ZIP and source: PGP signature | vs. release key `BF5A669F…D0D2` | **Pass** (GOODSIG/VALIDSIG) | Key fetched from keys.openpgp.org; fingerprint matches keepassxc.org |
| KeePassXC GUI launch with preset config | 8-second run | **Pass**: no TCP/UDP endpoints; no `%AppData%`/`%LocalAppData%` folders created | No database open |
| App runtime footprint | Running dev build | **Pass for what was seen**: WebView2 profile in `.devdata\data\webview`; no `app.cipherleaf.vault` folder in AppData; 0 TCP connections | Seconds of observation |
| First screen content | Via the debug port | **Pass**: "Create your vault" with the strength meter and the "Suggest a strong passphrase" link; dark theme; backdrop on | Read-only; the owner's hands-on check is still needed |

**Known limitations:** not tested through the running app: creating a vault, unlocking and locking, notes and the editor, sticky notes, the KeePassXC panel, auto-lock, screenshot blocking. The app was never run as a release build.

**Acceptance boundary:** none of the above is accepted. The owner's hands-on review is next.

## 6. Problems Encountered

- Git Bash's `npx` printed npm notices; the type check still returned exit 0, and PowerShell confirmed it.
- **File-watcher errors** in the dev log: Vite watches `.devdata\data\webview\…` and reports EBUSY. Harmless noise. Fix later: add `**/.devdata/**` to `server.watch.ignored` in `vite.config.ts`. Not done now, because a Vite restart would reload the owner's open window.
- **Debug port left open.** To read the first screen, the dev app was restarted with `--remote-debugging-port=9222 --remote-allow-origins=*`, bound to 127.0.0.1. The first screen showed a passphrase typed into the create-vault form, and input had occurred in the last second, so the owner was using the window. The script was stopped and made no changes; it only read text and one input's length. **The debug port is still open on that dev session.** Until the app is restarted without it, any program on this PC could drive that window, including a vault the owner creates in it. Restarting without the port is the fix and is recommended before real use.
- The "Cipherleaf" dev build isn't a Start-menu application, so computer-use screen access couldn't be granted. Cipherleaf's windows are also set to appear black in screenshots by design.

## 7. Documentation Impact

**Documentation created:** this file; `vendor/keepassxc-manifest.md`.

**Documentation updated:** `docs/security-design.md` (A9), `docs/plan/approval-and-amendments.md` (13:07:49 UTC entry; A9 approved), `docs/plan/A9-bundle-keepassxc.md` (status), `docs/STATUS.md`.

**Documentation still needed:** README rewrite; Execution Summary EX-2026-10-22-01's section 9 mentions the removed password code and could get a pointer to this record (left unchanged: it describes what happened then).

## 8. Git / Version Control Impact

**Version-control status:** local repository, branch `feat/a9-keepassxc`, no remote. `main` is untouched.

**Commits:** `f26c1f7` baseline; `0d57ad2` D0; `b7c8626` D1 backend; `2743021` D1 UI. A further commit containing the security-design update and this record follows this summary.

**Remote synchronization:** none.

## 9. Decisions Made During Execution

- **Preset settings for KeePassXC written once, on first start** — Basis: offline defaults. Authority: within A9.
- **Hash-check `KeePassXC.exe` before every start** — Basis: A9's "verified copy" intent. Authority: within A9; the DLLs are not hash-checked (recorded as a limitation).
- **Did not pass KeePassXC's master password** (`--pw-stdin` exists) — Basis: A9 P1 passes only file paths.
- **Stopped scripted UI testing when the owner appeared to be using the window** — Basis: avoid destroying their input.

## 10. Items That Need Human / Coordinator Review

- **Hands-on first run:** create a vault, then try notes, sticky notes, locking, and **Passwords → Create a password database** (this starts KeePassXC). Reviewer: owner.
- **Restart the dev app without the debug port** before any real data goes into it. Reviewer: owner decides when.
- **Still open from earlier:** A2 and A8 decisions; A1 and A3–A7 evidence; the GPL approach (A9 Q5).

## 11. Current Result

**What works now (observed):** the code builds, compiles in debug, launches, shows the create-vault screen, and keeps its files in the portable `data` folder with no network connections. The KeePassXC bundle is verified and runs offline.

**What does not work yet / unverified:** everything past the first screen.

**Delivery state:** committed locally; no release build.

**Owner acceptance state:** not requested.

## 12. Recommended Next Step

**Immediate next step:** the owner's hands-on review of the running dev app (or a restart without the debug port first). Then fix what the review finds.

**Prerequisite / authority:** none beyond the existing approval; a release build and any change to security behavior need the owner's go-ahead.

**Should work continue immediately?** Not on the running window while the owner is using it.

**Do not repeat:** the D0 download and verification; the D1 code removal.
