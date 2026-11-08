# Codex partner briefing: read this before any lane

- **Who and where:** Ahmed's side (`AE0101`). Written by Claude in the cloud thread "Build Notaya 1.6" (machine: cloud container), 2026-11-08, about 08:35 UTC (system clock).
- **Asked by:** Ahmed, project chat 2026-11-08 08:27:53 UTC: lean on Codex "as much as possible ... for everything basically", make Codex "not just an executor" but a partner who has "the entire documentation, understanding of the system, of the app, of the vision", and do not weaken PBS. Recorded as [A46](../plan/approval-and-amendments.md).
- **Result state:** *produced*. It tells Codex where things stand and how to work. It is not a plan to build anything new.
- **What this file is:** a short map, not a copy of the records. The records win; if this file and a record differ, the record is right and this file needs a dated correction.
- **Dated correction (2026-11-08 about 11:45 UTC, refresh thread "Refresh AI PBS docs", [EX-2026-11-08-61](../execution/2026-11-08-ai-pbs-records-refresh.md)):** sections 2, 5 and 6 were brought up to date (lanes merged since 08:35 UTC, the permanent heartbeat, the three-lane limit, who continues). Sections 1, 3 and 4 were checked and are unchanged.

## 1. What Notaya is, and what must not change

- **A private, offline notes app for Windows** (Tauri 2: a Rust core and a React + TypeScript screen layer). Pages (TipTap editor), sub-pages, notebooks and sections, local databases (table, board, calendar and more views), sticky notes, calculator, import and export (Markdown, HTML, Word, PDF, Notion zips). Everything lives in one encrypted vault file (`vault.nota`, format "NOTA" v2; older `.clf` is still read).
- **The north star** (Ahmed): full offline Notion parity, plus things Notion does not have: a sealed, safe AI door, a whiteboard, dashboards and a Notebook mode (OneNote-like). The order of versions is in [the version plan](../plan/2026-11-07-version-plan.md): 1.6 foundation and AI CLI, 1.7 comfort, 1.8 pages, 1.9 tabs and files, 2.0 corkboard and manual, 2.1 to 2.2 dashboards, 2.3 to 2.4 whiteboard, 2.5 Notebook mode.
- **The feel.** Calm, fast, plain words, nothing clever. Keep the current look (themes, spacing, icons, the Sunrise default). A fix or polish pass makes the existing screens more consistent and more usable. It never restyles them, never renames things people already know, and never adds a visible feature nobody asked for (list any idea of yours as an **addition** in the hand-back note, do not build it).
- **No trademarked or copyrighted names on Notaya's screens** (A42): "Notion" appears only as the source of an import or a source tag. Ahmed's own names are used: "Notebook mode", "Default style".
- **Security by design** ([A41](../plan/approval-and-amendments.md), `docs/security-design.md`): keys and secrets stay in the Rust core; the screen layer gets only what it displays; no network code of our own; no note content in error text or logs; the AI can never change its own permissions; deletes go to the recycle bin; AI edits can be undone; locked notes stay locked. Use proven patterns. Never invent your own crypto.
- **Test safety:** throwaway data only, in a folder outside the repository (`D:\Notaya Tests`, `NOTAYA_HOME` pointing there). Never Ahmed's real vault or notes, never Mohamed's `E:` drive. Close any debug port afterwards and confirm nothing listens (FI-01).

## 2. Where the code is

| Part | Where | Notes |
|---|---|---|
| Vault, crypto, backups | `src-tauri/src/{crypto,vault,backup,restore,state}.rs` | `AppState::with_mut` is the one write path |
| Data model | `src-tauri/src/model.rs` | `Settings`, `Item` kinds, serde defaults keep old vaults readable |
| Commands the screens call | `src-tauri/src/commands/*.rs` | thin wrappers over `*_core(s, actor, ...)` functions |
| Action registry (1.6) | `src-tauri/src/actions/` | one named action list for menus, keys and the AI |
| Databases | `src-tauri/src/database/` | views, filters, saves |
| Import and export | `src-tauri/src/{importers,export}/` | one shared document model (`doc.rs`) |
| Block ids and block edits (1.6, C4) | `src-tauri/src/blocks/` | stable block ids for links and AI edits |
| Change signal, revisions and edit leases (1.6, C2) | `src-tauri/src/changes.rs`, `src-tauri/src/state.rs` | `items-changed` carries only ids, kinds, who changed and revision numbers, never note content; `AppState` holds injected callbacks (`bind_events`), not a Tauri `AppHandle`, so `cargo test --lib` can start |
| Error log and activity log (1.6) | `src-tauri/src/logging/` | text is sanitized, never note content |
| AI bridge program (1.6, D) | `bridge/` | `notaya-bridge.exe`, MCP over a local pipe |
| Screens | `src/features/{notes,database,settings,sticky,search,unlock,ai}`, `src/components`, `src/store/app.ts` | TypeScript; `src/lib/ipc.ts` holds the typed calls |
| Mock for screenshots and scripted clicks | `dev-mock/`, `npm run mock` | fake data, no real backend |
| Records | `docs/` | see section 3 |

## 3. The records are the shared memory (PBS)

PBS (the AI Project Bootstrap System) is how Ahmed, Mohamed, Claude and Codex stay on the same page. It is not optional and Codex must not weaken it. In every session:

1. **Read first:** `CLAUDE.md` (the rules), then `docs/STATUS.md` (state, open questions, the exact next permitted action), then `docs/plan/approval-and-amendments.md` (what is authorized), then your lane's brief in `docs/plan/2026-11-08-v1-6-codex-briefs.md` and the spec sections it names. This briefing replaces re-reading the whole repository; it does not replace those four.
2. **State the task** in PBS form: baseline commit, the partner's message and its time, scope and write area, what you will return, the stop point.
3. **Keep apart:** *permission* (only a partner's own words), *observation* (what you ran, on which exact commit; "not run" is never "passed") and *acceptance* (a partner's word; a merge is not acceptance).
4. **Name the result state:** produced, verified, integrated, accepted. Most work stops at verified.
5. **Write a hand-back note** in the PR description: files changed, checks and results, what did not run, open questions, additions (marked). Claude turns it into the Execution Summary and Session Summary; Codex may draft either when asked, in the 4-section or 13-section form, and Claude reviews it.
6. **Times come from the system clock or the partner's message stamps, never from a guess.**
7. **Every record names the partner's side and the machine** (A38). Commit messages end with `Side: Ahmed (AE0101) · Machine: <machine>`.
8. **Add run usage** (model, effort, tokens, quota before and after) to `docs/ledger/`.

## 4. How the work is shared now (A46)

Codex does as much as it can. Claude directs, keeps the vision and the feel coherent, and merges.

| Work | Who |
|---|---|
| Building a lane from its brief | Codex |
| Bug fixes, failing tests, polish passes, accessibility and small UX consistency fixes | Codex, on a bounded brief from Claude (or Ahmed's report) |
| First-pass security and UX review of a diff | Codex, in a **fresh session that did not write the diff**; Claude then does one lean inspection and the final security call |
| Records, Execution Summary drafts, ledger rows, release-note drafts | Codex drafts when asked; Claude reviews and owns the accuracy |
| Planning, briefs, choosing what goes into a lane, anything that could change the feel or the design, merges, the security decision | Claude |
| Permission, acceptance, public release, licence | Partners only |

When a task would change how Notaya looks or behaves in a way people can see, stop and say so in the hand-back note instead of deciding it. Claude then decides or asks Ahmed.

## 5. Where things stand (dated correction 2026-11-08 about 11:45 UTC, system clock; the first version of this section was written at 08:35 UTC and was out of date by 11:07 UTC; check `docs/STATUS.md` and `git log origin/main` for the live state)

- **Merged into `main` for 1.6 (none accepted by a partner):** C1 action core (PR #74), B exports (PR #75), D bridge program (PR #76), A small UI fixes (PR #79), C4 block ids, block edits and database small saves (PR #85, the C4b branch), AF the four fixes from the independent review of lane A (PR #86), C2 change signals, revisions, edit leases and the recycle bin for pages, stickies and database items (PR #88, 11:38 UTC; 631 Rust tests passed on the exact head). Checks were run by the heartbeat on the PC on the exact commits; read each PR for what ran and what did not.
- **Running or held:** C4d (fixes from the independent review of the block edits, including a high-severity back-relation issue in database saves; `AE0101/v1-6-c4d`) and AG (a fix of lane A's screenshots script plus a database-view restore check; `AE0101/v1-6-ag`) are running. **C3 (history and activity log) is held until Codex's 5-hour window resets, about 15:45 UTC.** Lane E (PR #70) stays a draft until C5 exists.
- **Known follow-up:** the edit-lease error text from C2 names a person ("Ahmed"); it must say "the person" (it is only shown to an AI connection). A real-window UI check for C2 (`dev-mock/c2-ui-check.cjs`) has not run.
- **Next:** C3, then C5 (AI server: pipe, pairing, dispatcher, confirmations), then lane E's screens, integration, the independent security review, the PC round on throwaway data, release notes.
- **Authority:** [A44](../plan/approval-and-amendments.md) is the go for 1.6; [A46](../plan/approval-and-amendments.md) is Codex first. Nothing after 1.6 is a go to build until Ahmed says so for that version. No public release, no licence, no new library without a partner's words.
- **Cloud threads cannot run the Rust checks.** The cloud container has no Wine, no MinGW and no Windows toolchain, and installing them needs Ahmed's own word (A37 item 3; [EX-2026-11-08-55](../execution/2026-11-08-v1-6-build-start.md) section 2). The Gaming-PC heartbeat runs `cargo test --lib`, `tsc`, `npm run build` and `npm audit` on the exact pushed commit. A review done in the cloud says "not run" for Rust unless a PC result is quoted.

## 6. Pacing and the heartbeat (dated correction 2026-11-08 about 11:45 UTC; the 08:35 version said "two lanes at a time" and described a loop that is now a permanent task)

Codex runs on one shared ChatGPT account (plan `plus`) with a 5-hour window and a weekly limit. One full window is about 16% of the week, so a week holds about 6 windows (measured in the first cycle; see `docs/ledger/`). Readings on 2026-11-08: weekly resets 2026-11-14 15:44 UTC; the window that was used up at 08:53 reset at 10:45 UTC and the next resets about 15:45 UTC.

- **The heartbeat is a permanent Windows scheduled task on Ahmed's Gaming-PC** ("Notaya heartbeat", every 10 minutes, script `D:\Notaya-lanes\heartbeat\heartbeat.ps1`, registered by Ahmed himself because Claude Code's permission system refuses to register persistence). Each tick relaunches stopped lanes, runs the checks on pushed commits, makes WIP checkpoint commits, pauses at limits and logs the Codex and Claude meters to `docs/ledger/quota-readings.csv` (about 900 tokens for the Claude probe). It never spends a banked reset on its own.
- **At most three Codex lanes at once.** Four lanes took the 5-hour window from 61% to 100% in 18 minutes (about 6 points of the weekly limit, 08:35 to 08:53 UTC).
- **A lane that stops after a minute or two with exit 0 and no file changes is idle, not finished.** It usually hit the edge of its write area. Mechanical edits just outside a write area are made and listed in the hand-back note; anything bigger is a stop-and-say. (On 2026-11-08 about 30 minutes of Codex time were lost to this, 08:07 to 08:36 UTC; [EX-2026-11-08-55](../execution/2026-11-08-v1-6-build-start.md) section 5.)
- **Banked resets (Ahmed has two):** used only after the whole weekly limit is 100% spent (a bank refills the full week and the 5-hour window). Never automatic.
- **Claude's own pause:** Claude stops directing at 70% of its weekly limit (Ahmed, 2026-11-07 23:59:56 UTC); Codex may use 100%. Claude's meters are read by the heartbeat; no screenshots from Ahmed are needed.
- **Who continues when Ahmed's quotas end:** Mohamed, with **Claude (Pro) only and no Codex** (Ahmed, 2026-11-08 08:57:27 UTC). Mohamed's Claude then also writes code, within its weekly limit. Briefs, STATUS "Continuation" and the handover guide must therefore always be complete enough to resume from.
- Report the quota readings before and after each run for the ledger. The forecast of versions 1.6 to 2.5 against Codex and Claude quota is in the published usage ledger (rebuilt by the ledger thread; PR #83 while open).
