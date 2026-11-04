# Execution Summary: EX-2026-11-03-42 — Notaya V1.5.0: right-to-left text, PDF letters and full local databases

Result state: **produced**, and **verified** for the checks listed in sections 8 and 8b (cloud checks, and PC round 1 on the exact source `cc44d37`); the fixes and additions made after round 1 (section 8b, and automations in section 2 item 9) have only cloud checks so far; **not integrated, not released, not accepted**.

## 1. Task overview

- **Baseline:** `origin/main` `ee8c32e` (V1.4.0, released and accepted 2026-11-03 18:23:19 UTC).
- **Owner's messages (UTC, 2026-11-03):** 19:20:58, 19:22:15, 19:26:53, 19:28:12, 19:30:11, 19:30:55 (scope), 19:34:07 (two agents at a time), 19:43:06 ("I will tell you when to stop"), 2026-11-04 09:00:04 and 09:01:53 (fix the Notion zip import so rows are not doubled and relations, people and status are real properties), 09:05:57 ("Can you add the automations / Let's try to finish them all"), 20:59:50 and 21:00:14 (my own tests may read and write his Notion), 21:26:31 ("make a big one in notion"), 21:28:10 and 21:29:02 (he has no big export; use it for the tests). Authority: [A35](../plan/A35-v1-5-0-rtl-and-databases.md).
- **Scope:** one release, V1.5.0: Arabic/Hebrew right-to-left in every route; PDF export that prints Arabic, Hebrew and Chinese/Japanese/Korean letters; full Notion-style local databases including automations and the Button property (the owner added these on 2026-11-04 09:05:57; they were first left out); file cap 50 MB; nothing leaves the PC (no AI, network, sync or sharing).
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
9. **Automations and the Button property (added 2026-11-04, [P29](../security-design.md)):** a database has an Automations dialog (database menu) and a Button property. Triggers: a row is added, a property changes (any or one), a date arrives (with an offset before or after). An optional filter says which rows count. Actions: set a property (text, number with "add", select and status by name, checkbox with "switch", date as now / today / today+N / a date, person, multi-select with "add", link to this row, clear), add a row (to this or another database, with values), show a message in Notaya, move the row to the trash. A Button property runs the same actions on its row. All of it runs in the Rust core (`commands/automation.rs`); the UI only edits the definitions. Limits: chains stop after three steps, one run does at most 200 actions and 50 new rows, an action that fails shows a message and the rest carries on. No network, no programs, no files, nothing sent out. Date automations run while Notaya is open (with the reminder check, every minute) and catch up on up to seven days.
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
| Security review (read-only reader agent, by hand; the built-in skill could not run) | 4 findings, all fixed on this branch: (1) the formula regex matcher could overflow the stack on a long text, which would end the app (now bounded in depth, long texts matched on a helper thread with a 64 MB stack, one step budget per search; test on a 1 MB stack); (2) a long chain of rollups of rollups had no depth limit (now 32 levels, then an error; test on a 1 MB stack); (3) old Office and RTF files (doc, xls, ppt, rtf) were on the "open" list although a temp copy has no Mark-of-the-Web (removed, they can still be saved); (4) email and phone cells tried to open `mailto:` and `tel:`, which the backend refuses (they now show as plain text). Items checked and found fine: the `clatt` handler, file names, size caps, CSV limits and formula guard, exports, no `innerHTML` or `eval` in the UI. Not tested: a formula that builds very large strings inside a map (rated low) |
| Cargo tests after the fixes | 482 passed, 0 failed, 1 ignored |
| After the Notion zip rework and the automations (branch tip when written; cloud) | `cargo test --lib` under Wine 495 passed, 0 failed, 1 ignored (10 automation tests, 2 Notion-import tests); `tsc` clean; `npm run build` built (same harmless warning); `npm audit` 0; mock app `dev-mock/database-ui-check.cjs` ALL OK and the new `dev-mock/automations-ui-check.cjs` ALL OK (dialog, two actions, new-row message, Button property and press) |
| Security review of the automation code | By hand (the diff is small and closed-list): checked the commands (`press_button` checks the property is a Button; `save_database` validates every action and trigger, string lengths and counts), loops and fan-out (depth 3, 200 actions, 50 rows per run), silly numbers and dates (found and fixed: `@add:inf`, `@today+99999999999`), leftovers in memory (messages and date bookkeeping are cleared when the vault locks), and the per-minute date check (does nothing when no date automation exists). No finding left open. Not an independent reader this time |

## 8b. PC round 1 (owner's "Start pc tests", 2026-11-04 05:54:15 UTC; source `cc44d37c5e60`, exe SHA-256 `28A814B6CC9A40F2F29104B681A7433E64A1E594B9DBEA9D4F1104D4DE5875BD`; full report on the PC: `E:\Project\Notaya\Notaya Tests Excluded\v150\RESULTS-v150-round1.md`)

Throwaway vault only. Build on the PC: 482 cargo tests passed (MSVC), clippy 0 errors (122 style warnings), tsc clean, npm audit 0, release build OK. Repo runners on the exe: restoreflow 12 PASS, installed 11 PASS, optionalsecurity 47 PASS, lockflow OK, tray OK on a re-run.

| Area | Result |
|---|---|
| A databases (every property kind, views, filters, sorts, groups, relations, rollups, templates, sub-items, inline databases, first-database backup) | PASS except bug 1 |
| B files in cells (attach, open, save as with matching SHA-256, 50 MB limit, .bat/.exe refused) | PASS (the check that no program started for .bat/.exe was wrong in the script and was not re-run) |
| C CSV and exports (Arabic, Hebrew, CJK database exported to 5 formats and re-imported, every cell equal, `= + - @` cells safe) | PASS except bug 1 |
| D the owner's Notion zip (SHA-256 `FA4D0538…A7386C2`) | pages: 55 pages in 2 sections, headings, lists, to-dos, callouts, toggles, tables, code, equations, columns and the Arabic page right-to-left came across; the 10 table CSVs were left out by design (then); the 5 database CSVs imported one by one: 6, 7, 8, 5 and 18 rows, 0 bad cells; person, relation and formatted-number columns arrive as text |
| E right-to-left (auto, manual, slash item, Word and HTML import and export, survives lock and restore) | PASS |
| F compatibility | PASS except note 2 |
| G V1.4.0 smoke through the runners | PASS |

Findings and what was done:
1. **Bug 1:** New database -> "From a CSV file…" showed "That item no longer exists." (the dialog passed no section). **Fixed** in `8705826`.
2. Note: no menu items for Markdown, HTML, Word and PDF export of a database. **Added** in `8705826`.
3. Note: V1.4.0 on a vault with a database says "The vault file is damaged or has been tampered with." (as designed in P27; V1.4.0 is released and unchanged). The first-database notice in V1.5.0 already warns before it happens.
4. Note: version numbers still said 1.4.0. **Bumped to 1.5.0** in `02062d4`.
5. Observation: in the first tray run, the secure phase hit "The vault is locked."; a re-run passed. Not reproduced, cause not found.
6. **New after round 1 (reworked 2026-11-04 after the owner's screenshot "each row still appears twice, and relations, people and status come in as plain text"):** a Notion ZIP import makes a real database from each table in the export (the `_all` file wins), after a safety backup, in the same saving step. Each row page is now **moved into its row** (page body, icon and cover become the row's), so a row exists once and opens with its content; the property lines at the top of a row page are dropped, and the empty folder pages that only held row pages are dropped too. A page that has sub-pages, or that another page links to, is kept as a page. Columns whose cells all point at rows of one table become real **Relation** properties (two-way when the other table points back); a column named Status with a few short values becomes a **Status** with To-do / In progress / Complete groups guessed from the names; columns of a few repeated values become Select or Multi-select; `$12,000.00` and `60%` columns become Number with the right display format. **What the export makes impossible:** the export does not say which columns are people, formulas or rollups, so a person column arrives as text (a Person property can be set on it afterwards), and a formula or rollup column arrives as its last results as plain text (the formula itself is not in the export). Cloud tests only: `the_tables_of_a_notion_export_become_databases` and `notion_rows_become_database_rows_with_relations_status_and_numbers`.

Not run in round 1: the exact 100,000-row boundary, a real clipboard paste, database trash and restore from the sidebar, a final read of the test vault's error log, how the RTL pages and databases look (screenshots are in `v150\shots`).

## 9. Not run

In the cloud: the PC exe and the real backend. On the PC after round 1: the fixes, the reworked ZIP-to-database import (section 8b, items 1, 2, 4, 6) and the automations and Button property have not been run on the PC, and they have not been run in the real backend at all (only the Rust tests and the mock screens); the items in the last paragraph of 8b; Windows reminders (not built).

## 10. Known gaps

No list of trashed rows (only the toast undo restores one); no thumbnails for pictures in file cells; footers ignore the client-side search box; duplicating a property copies settings, not values; no database entry in the ribbon Insert menu (only "/"); the view tab strip scrolls sideways; the formula preview lags about one second; board cards show every property by default; dashboard of several charts is not built; a Notion CSV with no row pages in the export still gives rows with no body; an automation's "only if" filter is not used by a button; automations do not run while Notaya is locked or closed (date ones catch up for a week when it opens).

## 11. Next permitted action

PC round 2 (at most two rounds): retest the fixes and the ZIP-to-database import on the exact new tip, on throwaway data in `E:\Project\Notaya\Notaya Tests Excluded`, never the real vault. Remote Control starts only on the owner's own message.

## 12. Merge and release

The owner merges the draft PR with "Create a merge commit" and says "publish". Claude does not merge or release.
