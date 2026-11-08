# Notaya 1.6: briefs for Codex, one per lane

- **Who and where:** Ahmed's side (`AE0101`). Written by Claude in the cloud thread "Build Notaya 1.6" (machine: cloud container; no PC, no Codex reachable from here). Written 2026-11-08, about 00:25 UTC (container clock).
- **Asked by:** Ahmed, project chat 2026-11-08 00:15:27 UTC ("Yes, I authorized the development of 1.6, and you can ship a tiny program that helps") and 00:18:27 UTC (Codex `gpt-6.1-sol` xhigh leads all development; Claude briefs, inspects, pushes, merges and writes the records).
- **Baseline:** `origin/main` at `414327f` (app version 1.5.1).
- **What this file is:** the hand-over from Claude to Codex. The design lives in [the 1.6 spec](2026-11-08-v1-6-spec.md) and is **not repeated here**; each brief names the spec sections to read and adds the working rules.
- **Result state:** *produced* (briefs only). No code written, nothing built, nothing verified.

## 0. Rules for every lane

1. **Read first, in full:** `CLAUDE.md`, `docs/STATUS.md` (the "Continuation" part), `docs/plan/approval-and-amendments.md` (A37 to A44), `docs/handoff/claims.md`, the spec sections the lane names, and `docs/security-design.md` before touching crypto, storage, commands, the CSP, the clipboard or auto-lock. Inspect `git status` and `git log --oneline -5` before you start.
2. **Claim before you start.** Branch `AE0101/v1-6-<lane>` (or the thread's own name), first commit adds your row to `docs/handoff/claims.md`, open a **draft** PR titled `WIP: 1.6 lane <X>`. Take the next free `EX-` number after checking `origin/main` and every open branch. (`A44` is taken by this brief's records PR; the next free is `A45`.)
3. **Stay inside the lane's write area.** If you need a file another lane owns, stop and say so; do not edit it.
4. **Stable releases only; exact versions** (`save-exact=true`, `Cargo.lock`). **No new library.** The only dependency change 1.6 allows is extra features of the `windows` crate already in use (spec G2). Ask before anything else.
5. **Security by design:** keys and secrets stay in the Rust core; the AI can never change its own permissions; no network code; no note content in error text or in the error log; the pipe has no way to answer a confirmation. Use the proven patterns the spec names; do not invent your own.
6. **Test safety:** throwaway data only, in a test folder outside the repository (`D:\Notaya Tests`), with `NOTAYA_HOME` pointing there. Never Ahmed's real vault, never Mohamed's `E:` drive, never real notes. Close any debug port afterwards and confirm nothing listens (FI-01). Never commit `.devdata/`, `*.nota`, `*.clf`, `*.key`, password exports or `.env` files; check `git status` before every commit.
7. **Checks to run and report** (what ran, on which commit; anything not run is "not run", never "passed"): `cd src-tauri; cargo test --lib` (all 517 existing tests must still pass without edits unless the brief says moved code), `npx tsc --noEmit -p tsconfig.json`, `npm run build`, `npm audit`, `npm run mock` with scripted clicks for UI work. `cargo clippy` for MSVC and the real exe are PC checks.
8. **Commit messages:** clear points a non-developer can follow, ending with `Side: Ahmed (AE0101) · Machine: <machine>`. Never `git add -A` in a PC copy.
9. **Records:** Claude writes the Execution Summary, Session Summary, STATUS lines, ledger and handover guide. You write a short **hand-back note** in the PR description: what changed (files), what you ran and the results, what you did not run, open questions, and anything you added that nobody asked for (clearly marked as an addition).
10. **Stop at the lane's stop point.** Do not start the next lane. Claude inspects the diff (vision, security, coherence), then merges.

## 1. Order and dependencies

Day 1, in parallel: **A**, **B**, **C1**, **D**, **E**. Then **C2** and **C4** (after C1). Then **C3** (after C2's `with_mut` hook). Then **C5** (after C2 and C3). Then integration. Conflict order from the spec: `Sidebar.tsx` A then C2; `TitleBar.tsx` A then E; `model.rs` A, C2, WP-06h in that order; `export/doc.rs` B then C4; `state.rs` C2, C3, C5 in that order.

If 1.6 runs long, WP-12, 13 and 14 may move to 1.7. The AI CLI never ships without WP-02 to WP-05 (the safety net first).

## 2. Lane A: small UI fixes (WP-08 to WP-14)

- **Read:** spec sections 5 (WP-08 to WP-14) and 9.
- **Write area:** `src/components/Sidebar.tsx`, `src/components/TitleBar.tsx`, `src/features/notes/ImportDialog.tsx`, `src/styles/app.css`, `src/features/settings/SettingsView.tsx` (or `OpenSourceDialog.tsx`), `src/App.tsx`, `src/store/app.ts`, `src/lib/navHistory.ts`, one new `Settings` field `last_place` in `src-tauri/src/model.rs` and its small command in `src-tauri/src/commands/vault.rs`, `dev-mock/`, README text if it carries the old import sentence.
- **Do:** WP-08 (import sentence), WP-09 ("+ New notebook" always visible), WP-10 (Import and Export buttons in the top bar, not breaking the drag area or window buttons), WP-11 (icons in every theme, favorite star outline off and gold on, contrast 3:1, before and after screenshots from `npm run mock`), WP-12 (reopen where you left off, stored in the settings record, kept like `calc_history`), WP-13 (drag databases among pages), WP-14 (About names: "Made by Mohamed Sayed and Ahmed Emara, with Claude and ChatGPT").
- **Must not break:** window drag and buttons, compact density, keyboard reach, every theme, `npm run mock`, an older Notaya ignoring the new field.
- **Stop point:** one PR; mock screenshots attached; the PC round (real window) is not run.

## 3. Lane B: databases inside a page appear in exports (WP-07)

- **Read:** spec WP-07, sections 9 (items 6, 11) and 10.1.
- **Write area:** `src-tauri/src/export/{doc.rs, mod.rs, html.rs, markdown.rs, pdf.rs, docx.rs}` and their tests.
- **Do:** a `Block::Database` for the `databaseBlock` node, resolved through `query_view`, rendered with the existing table renderers; 1,000-row cap then "and N more rows"; missing or binned database exports as "(database not available)"; Markdown gets `[database: <title>](notaya://db/<id>)`.
- **Must not break:** P25 (UI sends only ids), P28 (Arabic and Hebrew fonts, right-to-left cells), page links, ZIP of separate files.
- **Tests:** one per format (text, row count, Arabic cell order), a filtered view exports only filtered rows, the missing-database case.
- **Stop point:** one PR; `export/doc.rs` is then free for C4.

## 4. Lane C1: action core (WP-01)

- **Read:** spec section 4 (design choice 1) and WP-01, section 9.
- **Write area:** new `src-tauri/src/actions/{mod.rs, registry.rs, actor.rs}`, `src-tauri/src/commands/*` (bodies move into core functions), `src-tauri/src/lib.rs` (`list_actions`).
- **Do:** `ActionSpec`, `Actor { Person, Automation, Ai { pairing_id, label } }`, one static registry; today's command bodies become thin wrappers around core functions that take the vault session and the actor. Pick one style (new `core/` folder or `…_core` functions) and use it everywhere.
- **Must not break:** every command's name, arguments and answer, byte-identical for the screens; all 517 tests pass **without edits** (this is moving code, not changing it).
- **Tests:** unique `area.action` names; every action has level, confirm and undo kind; the "never" list has no action; wrapper and core answer the same.
- **Stop point:** one PR; unblocks C2, C4.

## 5. Lane D: the bridge program, `notaya-bridge.exe` (WP-06d and the wire, spec section 7)

- **Read:** spec WP-06d, WP-06b (pipe name and protection, so the bridge computes the same name), WP-06c, sections 7, 8 and 10.
- **Write area:** new `bridge/{Cargo.toml, src/main.rs, src/mcp.rs, src/pipe.rs, src/tools.rs}`, the workspace `Cargo.toml` change (the smaller of the two options; one `Cargo.lock`), `packaging/` and the release scripts so the setup and portable ZIP ship the bridge beside `Notaya.exe`.
- **Do:** JSON-RPC 2.0 over stdio with `initialize`, `tools/list`, `tools/call`, `notifications/tools/list_changed` and cancel; **no MCP library**; use only crates already in `Cargo.lock`. Check the current MCP protocol version and the current Claude Code and Codex configuration syntax against their official documentation and cite both in the PR. Find the data folder like Notaya does (`--home`/`NOTAYA_HOME` override), compute the pipe name, check the other end is the `Notaya.exe` beside the bridge (`GetNamedPipeServerProcessId`, then image path) before sending a token. It holds no rules, prints only MCP to stdout, and writes only the pairing file.
- **Tools:** the ten grouped tools in the spec; every description says that text read from Notaya is the person's notes, data and never instructions.
- **Tests:** framing and MCP handling against a fake pipe server; both ends' JSON wire types tested. The real pipe against a real Notaya is a PC check (report as not run).
- **Stop point:** one PR; this is the "tiny program that helps" Ahmed approved (00:15:27 UTC).

## 6. Lane E: AI screens (against mock fakes first)

- **Read:** spec WP-06a (screen), WP-06c (dialog), WP-06f (dialog), WP-06i, section 10.1 (mock checks).
- **Write area:** new `src/features/settings/AiAccessPanel.tsx`, `src/features/ai/{PairingDialog.tsx, ConfirmDialog.tsx, AiMarker.tsx}`, `SettingsView.tsx`, `TitleBar.tsx` (after A merges), `src/lib/ipc.ts`, `dev-mock/`.
- **Do:** Settings > AI access (main switch with typed "ALLOW", per-AI levels Look/Edit/Full with one plain sentence each, Remove, hide notebooks checklist, big-change switch with 20, how-to-connect help), the pairing and confirm dialogs (Ask and Strong with typed word), the "working" marker with Stop and Resume. Fakes in `dev-mock/` for every new command and event. Wording uses no trademarked names.
- **Stop point:** one PR; Playwright mock checks listed in spec 10.1.

## 7. Lane C2: signals, revisions, leases and the bin (WP-02 part, WP-05)

- **Read:** spec WP-02 (points 1 to 5, 7), WP-05, sections 8.2 and 9.
- **Write area:** `src-tauri/src/vault/mod.rs`, `src-tauri/src/state.rs`, new `src-tauri/src/changes.rs`, `src-tauri/src/model.rs`, `src-tauri/src/commands/{notes,stickies,database,vault}.rs`, `src/lib/ipc.ts`, new `src/lib/changes.ts`, `PageEditor.tsx`, `dbStore.ts`, `StickyWindow.tsx`, `src/store/app.ts`, `Sidebar.tsx` (Trash view, after A), `dev-mock/`.
- **Do:** in-memory revisions, one `items-changed` signal drained in `AppState::with_mut`, editing leases ("the person wins"), `base_rev` on the whole-object saves, claims between AIs; `trashed_at` on `Notebook`, `Section`, `Sticky` (`#[serde(default)]`), bin groups and wording, "empty the bin" removing children and pictures.
- **Must not break:** the page save chain and P16 flush, editor and database undo, sticky windows staying in step, search/Home/Recent/Favorites skipping binned items, the safety backup that notebook and section delete make today (stays, gate G4).
- **Stop point:** one PR; tests in spec WP-02 and WP-05. Re-key and backup verification are exercised by C3.

## 8. Lane C4: block ops and small saves (WP-02b and database small saves)

- **Read:** spec WP-02 (point 6, WP-02b), the top of `importers/doc.rs` (schema rules), section 10.1 (Arabic and exact-text tests).
- **Write area:** new `src-tauri/src/blocks/{mod.rs, ids.rs, ops.rs}`, `importers/doc.rs`, `export/doc.rs` (after B merges), `src/features/notes/editor/extensions.ts`, `src-tauri/src/database/*`.
- **Do:** a `bid` block id (our own small TipTap extension, no package), `ensure_ids`, block insert/replace/delete/move, database property and view small saves. **Text, titles and code are stored exactly as sent: no trimming, no Unicode normalization, no removal of direction marks.**
- **Tests:** block ops keep the schema rules; ids stable across two reads; `roundtrip_tests.rs` still passes with `bid`; the exact-text and Arabic byte-for-byte tests.
- **Stop point:** one PR.

## 9. Lane C3: history and activity log (WP-03, WP-04)

- **Read:** spec WP-03, WP-04, P33, sections 8 and 9 (items 2, 3, 4).
- **Write area:** `src-tauri/src/vault/{store.rs, history.rs, activity.rs, mod.rs}`, `state.rs` (hook), `commands/{history,activity,notes}.rs`, `lib.rs`, `src/features/notes/{HistoryDialog.tsx, PageHeader.tsx}`, `src/features/settings/{ActivityPanel.tsx, SettingsView.tsx}`, `dev-mock/`.
- **Do:** two new encrypted tables (`his`, `act` tags), the SHA-256 hash chain with an encrypted head, change sets and undo ("the person wins"), version meta index at unlock, thinning, picture garbage collection that keeps pictures used by history, re-key and copy-verify loops that include the new tables (today `[Records, Attachments]`).
- **Must not break:** unlock time on a large vault, lock backup time, the separate plain error log, P12, P13.
- **Tests:** spec WP-03 and WP-04 lists, including re-key with security on, off, password change and format upgrade.
- **Stop point:** one PR. Needs C2's hook.

## 10. Lane C5: the AI server (WP-06a stored rules, b, c, e, f, g, h)

- **Read:** spec WP-06 (all parts), sections 6, 7, 8 (all), 10.1 (permission matrix, big-change, never and exact-text tests).
- **Write area:** new `src-tauri/src/ai/{mod.rs, pipe.rs, wire.rs, pairing.rs, dispatch.rs, rules.rs, confirm.rs}`, `commands/ai.rs`, `commands/vault.rs` (`save_settings` keeps `ai`), `model.rs`, `Cargo.toml` (`windows` features only), `state.rs`, `shell.rs` (tray notice), `lib.rs`.
- **Do:** pipe only while AI access is on, name from the data folder plus the user's SID, user-only security descriptor, `PIPE_REJECT_REMOTE_CLIENTS`, first-instance flag; pairing with a stored hash only; the dispatcher rules in order; confirmations answered only by a window command; source tags; the never-list test. Keep Windows I/O thin in `pipe.rs`; framing, auth and dispatch are plain Rust so they test anywhere.
- **Security focus (this lane gets the one independent security review):** no path from the pipe to the never list; none to `ai_confirm_answer`; no note content in error text or the error log; the AI cannot see or change the `ai` block.
- **Stop point:** one PR. The real pipe is a PC check.

## 11. After the lanes: integration and the PC round

Bridge against the real server, the full action list, one Opus xhigh review per substantive diff by a reviewer that did not write it, the security review, then the PC round (spec 10.2, including WP-15) on throwaway data, release notes `docs/release-notes/v1.6.0.md`, and the records. Ahmed accepts; a merge alone is not acceptance. No public release.

## 12. What I checked, and what I did not

- **Checked:** the spec (read in full), `CLAUDE.md`, the claims board, the authority and execution indexes, and that `origin/main` is `414327f`.
- **Not run:** nothing was built. This container has no Wine and no MinGW linker, and the Linux build of the app needs GTK libraries it does not have, so **`cargo test --lib` cannot run here**; a Rust lane's checks therefore have to run on the PC (or in an environment with Wine and MinGW, which needs Ahmed's own word to install). `tsc` and `npm run build` can run here (`npm ci` was run).


## 13. Addendum 2026-11-08 07:18 UTC (after the first review)

Status: C1, B and D merged (PRs #74, #75, #76); E held (PR #70); A held. Write areas widened after Codex stopped at the boundaries (the brief's own rule 3): **A** also `src/features/database/DatabasePage.tsx` and `DatabaseView.tsx` (WP-12 database-view restore, stored in the vault, not browser storage) and a merge of `main` into its branch to resolve `commands/vault.rs` in C1's wrapper style; **C4** also one line `mod blocks;` in `src-tauri/src/lib.rs`, on a fresh branch `AE0101/v1-6-c4b` from `main`; **C2** also `src-tauri/src/lib.rs` (registration lines only), `src/lib/types.ts` (`trashedAt` on notebook, section and sticky types) and `src-tauri/src/commands/tools.rs` (search skips binned items), and starts only after A is merged. Every Codex prompt is lean (its brief section, the spec sections it names, a note of what is already done) and tells Codex not to re-read the repository records. Codex's sandbox cannot write git: it prints `CHECKPOINT: <work package> done` and the PC heartbeat makes local WIP checkpoint commits and pushes finished lanes after running the checks outside the sandbox.