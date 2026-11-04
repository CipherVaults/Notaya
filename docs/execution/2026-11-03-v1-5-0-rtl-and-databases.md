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

## 8c. After PC round 2 (2026-11-04, owner's messages 09:40:41 to 10:30:24 UTC)

Round 2 used the owner's "Notebook 2.zip" (a second large Notion notebook with every database feature) and by-hand use in Notaya. Findings and fixes, all cloud-verified only (Rust tests, tsc, build):

1. BUG-2 and BUG-3 (flat Notion export: relation cells that name the table file; row pages next to the CSV files): fixed in `38d4df0` and `6d8e21d`.
2. BUG-4 (a line forced left-to-right or right-to-left printed in the wrong direction in PDF), OBS-3 (first-database notice after a CSV import), OBS-4 (PDF export text), a warning when a notebook of the same name already exists: fixed in `1ad3364`.
3. **Combined import (owner 10:16:09 to 10:26:06 UTC, "will be in v1.5"):** a Markdown & CSV export and an HTML export of the same Notion workspace complete each other, as ZIP files, folders or loose files. Rule chosen: pages are matched by the 32-character Notion id at the end of the file name; a page found once in each is imported once, with the HTML text and formatting, plus the Markdown/CSV property lines and data for database rows. HTML database view pages (same id as a CSV) are not imported as pages. Cloud tests added in `plan.rs` and `import_notes.rs`.
4. Security review of the new code (self-review, 10:40 UTC): files are read only through the existing `Vfs` with the existing size limit; the pairing uses names from the archive listing only, never builds file-system paths; no new dependency, no network. No finding.

**PC round 3 (2026-11-04 ~11:23 to 11:45 UTC, PC clock, source `735ad16`):** passed: BUG-2/3 (Notebook 2.zip: 104 row pages moved into rows, 148 of 148 relation cells equal to the CSV; the earlier zip 78 of 78), OBS-3, OBS-4, the duplicate-notebook warning, seven views, automations, PDF export. Failed: **BUG-5** (English text forced right-to-left stayed at the left edge in the PDF); fixed in the cloud in the next commit (alignment uses the chosen direction), cloud-tested only. Not run: the combined import (no HTML zip was in the folder when the run looked), clippy, database export to MD/HTML/Word.

**PC round 3, continued (2026-11-04 12:50 to 13:45 UTC, PC clock, source `f3abf36`):** passed: BUG-5 (forced right-to-left English right-aligned in the PDF); the combined import from the two zips and from 231 loose files (105 pages in both exports imported once, 7 HTML database pages skipped, 7 databases, 104 rows moved, 148 of 148 relation cells equal to the CSV, 9 row bodies). **Failed: BUG-6** (the two export folders picked together made 14 databases and no relations, because the table files were grouped by folder). Fixed in the cloud: table files with the same Notion id are one table, the `_all` file wins; new test `two_export_folders_picked_together_make_each_database_once` fails without the fix; 500 Rust tests pass. Not testable with that data: text colors and links between pages. Disclosure: the test session sent one mouse click and Ctrl+A to the owner's foreground window (a browser with AnyDesk) around 13:23 to 13:28 UTC; nothing was seen to change. **BUG-6 retest on the PC (13:45 to ~14:25 UTC, PC clock, source `c482411`, exe SHA-256 `9232AD59EE60...14B2`): passed.** Two extracted folders picked together: 7 databases (was 14), 104 rows, 7 relation properties, 148 of 148 relation cells equal to the CSV, 9 row bodies, no duplicate pages; the two-zip import gives the identical result. Not re-run on `c482411`: loose files (path unchanged). Result state: verified on that source, not accepted.

**Remaining checks on `c482411` (owner's "Do this", 2026-11-04 14:17:41 UTC; PC 14:18 to ~14:50 UTC):** `npm audit` 0 vulnerabilities (cloud). `cargo clippy --lib` (MSVC) on the PC: exit 0, 0 errors, 135 warnings (132 in round 2), nothing fixed. Database export (3 rows, text, select, number, date, checkbox, relation, formula, an Arabic row) to CSV, Markdown, HTML, Word and PDF through the real Save dialog: headers and values right in all five, Arabic right-to-left in HTML (`dir="auto"`), Word (`w:bidi`) and PDF; cosmetic only: narrow PDF columns wrap dates. Colors and a page link in a combined import, on a **hand-made** mini export (not a real Notion export): 2 pages once each, blue/red/green/yellow/purple colors kept, the link opens the child page. Real Notion exports with colors and links remain untested.

## 9. Not run

In the cloud: the PC exe and the real backend. On the PC after round 1: the fixes, the reworked ZIP-to-database import (section 8b, items 1, 2, 4, 6) and the automations and Button property have not been run on the PC, and they have not been run in the real backend at all (only the Rust tests and the mock screens); the items in the last paragraph of 8b; Windows reminders (not built).

## 10. Known gaps

No list of trashed rows (only the toast undo restores one); no thumbnails for pictures in file cells; footers ignore the client-side search box; duplicating a property copies settings, not values; no database entry in the ribbon Insert menu (only "/"); the view tab strip scrolls sideways; the formula preview lags about one second; board cards show every property by default; dashboard of several charts is not built; a Notion CSV with no row pages in the export still gives rows with no body; an automation's "only if" filter is not used by a button; automations do not run while Notaya is locked or closed (date ones catch up for a week when it opens).

## 11. Next permitted action

PC round 3 (owner-approved 2026-11-04 10:17:54 UTC): retest sections 8b and 8c on the exact new tip, including the combined HTML + Markdown/CSV import, on throwaway data in `E:\Project\Notaya\Notaya Tests Excluded`, never the real vault. Remote Control starts only on the owner's own message.

## 12. Merge and release

The owner merges the draft PR with "Create a merge commit" and says "publish". Claude does not merge or release.

## 13. Release (2026-11-04)

The owner merged PR #47 at 14:24:53 UTC (merge commit `c27f68ed974f39ae9e550e53b8a557c9eec53911`) and wrote "Publish" in this thread at 14:30:06 UTC. The PC session built from exactly that commit in a fresh clone (`cargo test --lib` 500 passed, 1 ignored; `tsc` ok), smoke-tested the portable build on throwaway data (a database with two rows, one Arabic, survived a restart), and ran `gh release create v1.5.0` (tag on `c27f68e`, Latest). Assets and SHA-256 digests on GitHub equal the table in `docs/release-notes/v1.5.0.md`; v1.4.0 and older releases were not touched. **Not run:** the setup installer of this build (the test PC already has an installed Notaya registry key). Result state: published; not accepted.
