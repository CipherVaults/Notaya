# Execution Summary: EX-2026-11-03-42 — Notaya V1.5.0: right-to-left text, PDF letters and full local databases

Result state: **produced** (code on the branch, builds, cloud checks pass); **verified** only for the checks listed in section 8, on the exact source named there; **not integrated, not released, not accepted**. The real app (PC exe), the real backend with the new screens and the PDF letters in the real window have **not been run** (section 9).

## 1. Task overview

- **Baseline:** `origin/main` `ee8c32e` (V1.4.0, released and accepted 2026-11-03 18:23:19 UTC).
- **Owner's messages (UTC, 2026-11-03):** 19:20:58, 19:22:15, 19:26:53, 19:28:12, 19:30:11, 19:30:55 (scope), 19:34:07 (two agents at a time), 19:43:06 ("I will tell you when to stop"), 20:59:50 and 21:00:14 (my own tests may read and write his Notion), 21:26:31 ("make a big one in notion"), 21:28:10 and 21:29:02 (he has no big export; use it for the tests). Authority: [A35](../plan/A35-v1-5-0-rtl-and-databases.md).
- **Scope:** one release, V1.5.0: Arabic/Hebrew right-to-left in every route; PDF export that prints Arabic, Hebrew and Chinese/Japanese/Korean letters; full Notion-style local databases except automations and the Button property; file cap 50 MB; nothing leaves the PC (no AI, network, sync or sharing).
- **Write area:** `src-tauri/src/` (database, commands, importers, exporters), `src/` (editor, database features, store), `dev-mock/`, `docs/`.
- **Stop point:** produced and cloud-verified; the owner decides the PC test round (at most two) and merges.

## 2. What was built

1. **Right-to-left** ([RTL-notes.md](RTL-notes.md)): each block takes its direction from its first strong letter; a ribbon button and a "/" item force a direction; import keeps `dir`, `lang` and Word `w:bidi`; Word export writes the right-to-left flag; HTML export keeps `dir`; the manual choice is stored in the page, so backups and restores carry it.
2. **PDF letters** ([PDF-unicode-notes.md](PDF-unicode-notes.md)): embedded font subset, letters joined by a hand-written shaper, no new dependency.
3. **Databases** (Rust core, `src-tauri/src/database/`): kinds of property (text, number, select, multi-select, status, date with range and reminder, person as text, files, checkbox, URL, email, phone, formula, relation, rollup, created/edited time and by, ID, place); Notion-style formulas; nested filters, sorts, groups, calculations; sub-items and dependencies; templates; charts; row pages and peeks; inline and linked views in a page.
4. **Seven views** ([DB-VIEWS-notes.md](DB-VIEWS-notes.md)): table, board, calendar, timeline, list, gallery, chart.
5. **Screens** ([DB-UI-notes.md](DB-UI-notes.md)): sidebar entries, favorites, trash with undo, new-database dialog, panels for filter, sort, group, properties, view settings and formulas, command-palette hits for databases and rows.
6. **Import and export:** CSV and Notion CSV import (limits 50 MB, 100,000 rows, 100 columns, one transaction); export of a database or one view as CSV, Markdown, HTML, Word, PDF; files in cells (attach, open, save as, remove).
7. **In-app reminders:** a toast with the row title, once per run, only while unlocked. There is no Windows notification, because that would need a new crate and the repository rules say to ask first.
8. **Safety backup:** the first database in a vault triggers a backup and a notice; V1.4.0 cannot open such a vault ([security-design.md](../security-design.md), P27 and P28).

## 3. How it was compared with Notion

Per the owner's permission I built a test workspace in his Notion ("Notaya big test workspace": Projects 6, Tasks 18 with sub-tasks and three formulas, People 7, Reading list 8, Meetings 5, and five pages including Arabic, Hebrew and Chinese/Japanese). Earlier, a small comparison database with 20 formulas fixed three engine differences: an empty number compares as 0, date functions on an empty date return empty, and the name sort ignores case with Arabic after Latin. They are asserted in `database/notion_compare.rs`. The Notion connector cannot export a file, so a full export-and-import comparison needs the owner's Notion export ZIP (section 9).

## 4. Decisions Claude took (for the owner to confirm)

- Row file API: `rowId` + `propId` with the file name taken from the stored cell (safer than a name from the UI).
- Files not recognised as pictures are stored as octet-stream and served inline only for four picture types; opening a file writes a plain temp copy only for an allowlist of non-executable extensions and removes it after 10 minutes and at the next start.
- CSV export guards cells that start with `=`, `+`, `-`, `@` with a leading apostrophe; import undoes it.

## 5. Security

Design notes P27 and P28 in `docs/security-design.md`. The built-in `security-review` skill could not run (`origin/HEAD` is not set in the cloud clone), so a read-only reviewer agent went through the diff by hand. Its result is in section 8.

## 6. Files

111 files changed against `ee8c32e` at the time of writing (about 23,000 added lines); see the draft PR for the list.

## 7. Records changed

A35 row (already in `approval-and-amendments.md`), this report and its index row, `docs/STATUS.md` (Continuation), `docs/handoff/claims.md` row, and the resume note `docs/handoff/2026-11-04-v1-5-0-resume-note.md`.

## 8. Checks (cloud, on the branch tip when written)

| Check | Result |
|---|---|
| `cargo test --lib --target x86_64-pc-windows-gnu` under Wine | 480 passed, 0 failed, 1 ignored |
| `npx tsc --noEmit -p tsconfig.json` | clean |
| `npm run build` | built; one harmless warning about a mixed static/dynamic import of `DatabaseView` |
| `npm audit` | 0 vulnerabilities |
| Mock app, `dev-mock/database-ui-check.cjs` (Playwright) | ALL OK (every property kind, filters, sorts, groups, formulas, relations, row pages, templates, inline databases, RTL, dark and narrow looks) |
| Mock app, all seven view kinds added to the seeded Tasks database | each opened, no page errors |
| Independent security review (reader agent, by hand) | see the PR description for the result when it is in; **not yet recorded here** |

## 9. Not run

The PC exe; the real backend with the new screens (the screens were checked only against the mock, which was written from `ipc.ts` and the Rust sources, so a mismatch is possible); `cargo clippy` for MSVC; PDF letters in the real window; opening a real Notion export ZIP; a vault with a database opened by V1.4.0 to confirm the refusal message; Windows reminders (not built).

## 10. Known gaps

No list of trashed rows (only the toast undo restores one); no thumbnails for pictures in file cells; footers ignore the client-side search box; duplicating a property copies settings, not values; no database entry in the ribbon Insert menu (only "/"); the view tab strip scrolls sideways; the formula preview lags about one second; board cards show every property by default; dashboard of several charts and Notion page-body import for CSV rows are not built.

## 11. Next permitted action

The owner asks for a PC test round (Remote Control, throwaway data in `E:\Project\Notaya\Notaya Tests Excluded`, never the real vault), at most two rounds; bugs found are fixed on the spot and the area is retested.

## 12. Merge and release

The owner merges the draft PR with "Create a merge commit" and says "publish". Claude does not merge or release.
