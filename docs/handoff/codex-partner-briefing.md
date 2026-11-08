# Codex partner briefing: read this before any lane

- **Who and where:** Ahmed's side (`AE0101`). Written by Claude in the cloud thread "Build Notaya 1.6" (machine: cloud container), 2026-11-08, about 08:35 UTC (system clock).
- **Asked by:** Ahmed, project chat 2026-11-08 08:27:53 UTC: lean on Codex "as much as possible ... for everything basically", make Codex "not just an executor" but a partner who has "the entire documentation, understanding of the system, of the app, of the vision", and do not weaken PBS. Recorded as [A46](../plan/approval-and-amendments.md).
- **Result state:** *produced*. It tells Codex where things stand and how to work. It is not a plan to build anything new.
- **What this file is:** a short map, not a copy of the records. The records win; if this file and a record differ, the record is right and this file needs a dated correction.

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

## 5. Where things stand (as of 2026-11-08 08:35 UTC; check `docs/STATUS.md` for the live state)

- **Merged into `main` for 1.6:** C1 action core (PR #74), B exports (PR #75), D bridge program (PR #76), A small UI fixes (PR #79). Verified by the PC checks on the exact commits; none accepted by a partner.
- **In progress:** C4b block ops (`AE0101/v1-6-c4b`, three failing tests being fixed) and C2 signals and recycle bin (`AE0101/v1-6-c2`).
- **Next:** C3 history and activity log, C5 AI server (pipe, pairing, dispatcher, confirmations), then lane E's screens (PR #70, held until C5), integration, the independent security review, the PC round on throwaway data, release notes.
- **Authority:** [A44](../plan/approval-and-amendments.md) is the go for 1.6; nothing after 1.6 is a go to build until Ahmed says so for that version. No public release, no licence, no new library without a partner's words.

## 6. Pacing

Codex runs on one shared ChatGPT account with a 5-hour window and a weekly limit. Two lanes at a time. When a limit hits, record the reset time and resume then. Report the quota readings before and after each run for the ledger. The forecast for the roadmap (versions 1.6 to 2.5 against Codex and Claude quota) lives in the published usage ledger, rebuilt by the ledger thread. Quota rules (Ahmed, 2026-11-08 08:57:27 UTC): Codex banked weekly resets are used only after the whole weekly limit is spent (a bank refills the full week and the 5-hour window) and never by the heartbeat on its own; at most three Codex lane processes run at once, because four took a 5-hour window from 61% to 100% in 18 minutes; Mohamed has Claude (Pro) only and no Codex, so when Ahmed's quotas end Mohamed continues with Claude alone.
