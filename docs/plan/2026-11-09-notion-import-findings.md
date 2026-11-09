# WP-19: Notion import investigation, lane N1

**Who and where:** Ahmed (`AE0101`), Codex on `GAMING-PC`, Windows sandbox. **State:** findings and tests produced; individual observations verified as described below; no fixes, integration, acceptance or publication.

**PBS boundary.** This fresh session found a clean worktree on `AE0101/v1-6-n1` at `623ddd9192faa178695d72e07f4f1d27e74d363e`. That includes automatic N1 checkpoints `29aa75d` and `623ddd9`; they already supplied the test module and its four-line registration. Lane fork/base before those checkpoints: `d910034`. Authority: Ahmed's current N1 instruction (message time not exposed), A52 (2026-11-09 10:42:50 UTC), WP-19 and the 10:45 UTC briefs update. First recorded orientation clock: 2026-11-09 18:03:54 UTC. Scope: investigate the seven read-only copies; write this note, invented tests and external throwaway tools only. Stop here for Claude to read the complete findings and tests before N2/N3. No network, installs, application/browser runs or agent Git mutations. Claims, session, ledger and PR updates remain Claude's work under this narrower brief.

The lane's documented model/effort configuration is `gpt-6.1-sol` / `xhigh`; actual served model, effort, token receipt and before/after quota meters are **not available to this session**. Claude should fill its ledger from the runner receipt, not infer usage from elapsed time.

## Measurement and privacy boundary

E1–E6 mean the six small exports **in the order in Ahmed's N1 brief**; E7 means the full workspace ZIP. These aliases reveal no private names. File trees, titles, identifiers and cell values stayed outside the repository and tool output. Only structure and counts appear here. A same-title pair is a duplicate-looking pair, not proof of duplicate identity.

The Python standard-library inspector streamed extraction into a new private run folder. It counted actual bytes before each write, bounded entries (50,000/archive, 100,000/run), individual files (2 GiB), expanded bytes (4 GiB), nesting (one inner ZIP level), compression ratio and path depth. It refused absolute/traversal/device/ADS/invalid Windows names, links and reparse points, checked containment and ancestors, and used exclusive creation. Thirteen invented unsafe-path checks passed. SHA-256 compared in RAM before/after each source; all seven remained read-only and unchanged. No digest or filename list is included here.

All seven original ZIPs were also passed to the existing Rust `Vfs`, planner, image validator, renderer, database builder and row-page cleanup by the ignored environment-driven probe. The six small **extracted folders** gave the same item counts as their ZIPs. Two matching HTML/Markdown ZIP combinations were measured too. Image storage used a callback returning fresh ids; imported items remained in RAM. Only an **empty throwaway vault** was created for database construction. This measures **what would be created**, not successful persistence, attachment storage, backup creation, UI appearance or acceptance. No real vault was opened.

The full inspector refused 71 entry paths under its stricter Windows disk-name policy. It extracted 5,709 of 5,780 content files; two CSVs were among these omissions. Therefore full-export CSV and item results below come from the **original ZIP through Rust**, not the incomplete extracted tree. Metadata counts include those 71 entries. Rust's in-memory reader refused zero entries; accepting a name in RAM does not establish that it is safe to extract on Windows.

For re-entry, private tools/results are under the authorized test root: `tools/investigate-1800.py`, `tools/extra-structure-1800.py`, `tools/probe-1800.py`, and the run folder named by `tools/fresh-run.txt`. These are outside the repository. The test receives source, optional second ZIP, test root, mode and new home exclusively through `N1_SOURCE`, `N1_SOURCE_2`, `N1_TEST_ROOT`, `N1_MODE`, `NOTAYA_HOME`. Run the private probe alone, with panic details suppressed. Do not run all ignored tests against private data.

**Definitions:** pages below are `Item::Page`, rows are `Item::Row`; the two counts are separate. Loose pages means pages in the fallback `Imported pages` section; root pages means pages without a page parent, including roots in named sections. Neither definition means a second reference to a row. Homes are only identified when structure supports that claim; many shallow files do not establish many home pages.

## E1 — small Markdown/image export

- Tree: 138 files: 59 Markdown, 73 WebP, 6 JPEG; zero CSVs/folders/other attachments. All 59 document names have shape `Name <32-hex-id>.md`. File-size bands: 65 at 1–64 KiB, 73 at 64 KiB–1 MiB. Source ZIP: 1–16 MiB. All documents are at depth 0.
- Would create: **1 notebook, 1 section, 59 pages, 0 databases, 0 rows, 59 loose/root pages, 79 accepted images**. Zero final skipped entries. ZIP probe: 1.816 s.
- Missing/duplicated: no source document lost, no repeated document ids or row/page pairs. There is no CSV database in this source to import; hierarchy or a unique home cannot be reconstructed from this flat tree alone. Fallback placement comes from `plan.rs:703–722`, not database rejection.

## E2 — small flat HTML/CSV export

- Tree: 265 files: 263 HTML, 1 CSV, 1 PNG; zero folders/other attachments. Document and CSV names have 32-hex suffixes. CSV: **1 group, 262 records, 69 columns**; no `_all.csv` and no row **folder**, although flat row documents exist. File-size bands: 262 at 1–64 KiB, 2 at 64 KiB–1 MiB, 1 at 1–16 MiB. Source ZIP: 1–16 MiB; all files depth 0.
- Would create: **1 notebook, 1 section, 247 pages, 1 database, 262 rows, 247 loose/root pages, 0 accepted images**. The one local PNG is unused by this path. Zero final skipped entries. ZIP probe: 2.810 s.
- Missing/duplicated: 251 documents classified as rows; 16 bodies moved into rows, **235 remained independent pages alongside their rows**. The remaining 12 pages were not classified as rows. No source home was pruned; this flat layout does not identify one unique home. Cause: the linked-page safeguard in `notion_db.rs:337–374` retains a source page instead of redirecting links to the newly created row. Flat placement is `plan.rs:709–713`. This directly reproduces the loose-page part of problem 2. The separate page and row have different ids; deleting one was not tested on private data.

## E3 — small flat Markdown/CSV export

- Tree: 264 files: 262 Markdown and 2 CSV, zero folders/images/other attachments. One plain CSV and one `_all.csv`, with **different ids**: two importer groups, each with 262 records; column counts 11 and 69. Neither has a row folder. File-size bands: 263 at 1–64 KiB, 1 at 64 KiB–1 MiB. Source ZIP <1 MiB; depth 0.
- Would create: **1 notebook, 1 section, 11 pages, 2 databases, 524 rows, 11 loose/root pages**, zero images/final skipped entries. ZIP probe: 2.208 s.
- Missing/duplicated: 251 source bodies moved into rows, zero classified row pages retained, 11 source pages left. There are **224 distinct normalized row titles appearing in both databases**; equal titles do not prove equal entities. `notion_db.rs:94–129` groups by path/base/id, so `_all` does not supersede a file with another id. This needs an ambiguity policy, not a title-based automatic merge. No distinct home file is identified by the tree; no non-row source document was pruned.

## E4 — small flat HTML database-view export

- Tree: 9 files: 8 HTML and 1 CSV, zero folders/images/other attachments. CSV: **1 group, 64 records, 21 columns**, no `_all` or row folder. All files have 32-hex suffixes, are at depth 0, and are 1–64 KiB. Source ZIP <1 MiB.
- Would create: **1 notebook, 1 section, 0 pages, 1 database, 64 rows, 0 loose/root pages**, zero images/final skipped entries. ZIP probe: 1.608 s.
- Missing/duplicated: **1 HTML file shares the CSV id and is hidden before planning** (`plan.rs:615–619`); the other 7 HTML documents become row bodies (`notion_db.rs:355–381`). That explains zero standalone pages. The shared-id HTML is treated as a database view; this does **not prove a separate home page was lost**. Whether useful navigation/body outside its table should be preserved is a visible policy decision. No repeated document ids or retained row-page pairs.

## E5 — small flat Markdown database-view export

- Tree: 9 files: 7 Markdown and a plain/`_all` CSV pair with the **same id**. One group, **64 records, 21 columns**, no folders/images/other attachments or row folder. File-size bands: 4 below 1 KiB, 5 at 1–64 KiB. Source ZIP <1 MiB; depth 0.
- Would create: **1 notebook, 1 section, 0 pages, 1 database, 64 rows, 0 loose/root pages**, zero images/final skipped entries. ZIP probe: 1.207 s.
- Missing/duplicated: all 7 supplied Markdown documents become row bodies; the CSV contributes 57 further rows without exported bodies. A distinct home page is **not supplied by this file tree**. `_all` correctly wins over the plain view here (`notion_db.rs:94–140`). No database loss or retained row-page duplication.

## E6 — small nested home-page export

- Tree: 80 files: 78 Markdown and a same-id plain/`_all` CSV pair; **6 folders**, depths 1:1, 2:1, 3:1, 4:3. Five folders contain documents without CSV, one contains documents and CSV. File-size bands: 31 below 1 KiB, 49 at 1–64 KiB. Source ZIP <1 MiB; zero images/other attachments. CSV: **1 group, 64 records, 21 columns**; 64 row documents are in an id-less folder with the database's clean name.
- Would create: **1 notebook, 2 sections, 34 pages, 1 database, 64 rows, 1 loose page, 9 root pages**, zero images/final skipped entries. Of 34 pages, 30 are source documents and 4 generated folder placeholders. ZIP probe: 1.207 s.
- Missing/duplicated: the **single shallow home, at file depth 1, survives**. Its clean-name sibling folder omits the id suffix. There are 4 clean-name page/folder pairs but zero exact-stem pairs. `files_of` matches only the complete stem (`plan.rs:302–320`); wrapper stripping then puts the home in the loose section and the unmatched folder in a second section (`plan.rs:703–722`). The home is separated, not deleted. Forty-eight row bodies move; **16 row pages remain separately**, all nested rather than root. There are 18 page/row same-title pairs, but only 16 involve source documents; 2 are generated placeholders. Do not call all 18 duplicates.

## E7 — full workspace ZIP

- Source size: **1–2 GiB**. One outer archive and 6 inner ZIPs: **5,786 file entries including the inner containers; 5,780 content files**. Files: 3,814 HTML, 114 Markdown, 105 CSV, 1,032 PNG, 377 JPG, 243 WebP, 1 JPEG, 16 SVG, 16 PDF, 16 TXT, 1 DOCX, 2 XLSX, 27 JSON and 16 other attachments. Thus **3,961 document files, 1,669 images, 45 other attachments**. No repeated document ids, HTML/Markdown identity pairs or duplicate paths.
- Folders: **1,989**; depth histogram **1:7, 2:17, 3:1747, 4:67, 5:89, 6:49, 7:8, 8:5**. Immediate folder kinds (documents/CSV/images present): images only 1,606; documents only 294; documents+CSV 27; CSV only 49; none of those kinds 11; CSV+images 1; documents+images 1. These are structural categories, not database identity counts.
- Filename shapes: 3,814 `Name <32-hex-id>.html`, 83 `Name <32-hex-id>_all.csv`, 22 other `.csv`; the 114 Markdown files lack that suffix. Exactly **0 complete-stem page/folder pairs**, but 229 clean-name pairs. Only 1 folder has an id suffix. Size bands: **66 below 1 KiB, 4,179 at 1–64 KiB, 1,136 at 64 KiB–1 MiB, 399 at 1–16 MiB**, none larger. Actual extraction reading: 1,330,207,560 expanded bytes, including inner archive containers.
- CSV: **105 groups, 83 `_all` views and 22 other CSVs, 4,755 parsed records, 1–73 columns**. By clean-name folder association: **80 groups have 2,854 row documents; 25 have no matching row documents**. Of the 83 `_all` groups, **80 have row documents, 3 do not**. Exact raw-stem matching alone would incorrectly report all 105 as lacking rows. These are exported CSV groups, not a count of every database that existed in the original workspace.
- Would create: **1 notebook, 5 sections, 1,388 pages, 105 databases, 4,755 rows, 8 loose pages, 28 root pages, 1,463 accepted images**. Planning produces 4,281 pages: 3,920 source documents and 361 generated placeholders. Forty HTML database views are suppressed; one further document fails to become a planned page. There are 188 initial skip records (105 CSV, 38 image-related, 45 unsupported), 189 after image validation, and **84 final skips after removing the successfully imported CSV entries**. Zero table-read notes, archive refusals or database failures. The additional failed-document reason was not separately classified by the probe; do not infer which converter it was.
- Missing/duplicated: **2,805** source row bodies move; **49** source row pages remain separately. Final source pages: 1,115; final generated placeholders: 273. All 3 shallow documents survive. Zero non-row source documents are deleted by cleanup. There are 23 page/row title pairs (13 source pages, 10 placeholders) and 157 distinct row titles across multiple databases; neither is a duplicate-identity verdict. Arrangement causes are the same clean-name mismatch, linked/parent-page retention and CSV placement rules described above.
- Performance: **143.244 s process wall time**, **142.338 s probe body**, **1,444,634,624 bytes peak working set** (about 1,377.7 MiB), Windows process memory counters sampled every 0.2 s and once after exit; exit 0, stderr 0 bytes. Debug library-test process; this session ran its library suite separately. Other work on the PC was not measured. The ZIP is **not refused or silently cut at 1 GiB**, and all CSV groups become databases. This does not mean every attachment/page feature survives: the explicit omissions above remain. No cold/warm comparison, save-time cost, vault-size measurement or real-app memory measurement was made.

## Combinations and current decision map

Choosing E2+E3 together yields **1 notebook, 1 section, 11 pages, 2 databases, 524 rows, 11 loose pages** (1.607 s). Choosing E4+E5 yields **1 notebook, 1 section, 0 pages, 1 database, 64 rows** (0.406 s). Both have zero final skips. HTML/Markdown pairing changes the retention result; combining formats is not a universal cure for ambiguous CSV ids or missing navigation.

| Decision today | Source |
|---|---|
| Read ZIPs into memory; mount one inner ZIP level; keep first same-path entry | `importers/source.rs:78–119`, `:194–235` |
| Bounds and checksums during entry reads; no disk extraction by Rust | `importers/zip.rs:113–232`; 2 GiB compressed selection, 4 GiB shared read budget in `source.rs:25–32` |
| Discover documents by extension; CSV is not itself a document | `importers/plan.rs:99–110` |
| Pair one HTML and one Markdown by id; suppress a single HTML with a CSV id | `plan.rs:596–629` |
| Skip top-level `index.html` for an archive containing other HTML; cap listed skips at 500 | `plan.rs:39`, `:578–588`, `:635–663` |
| Reject more than 5,000 documents, then reject zero documents; CSV-only sources never reach database construction | `plan.rs:646–665`; generated placeholders also count at `:731–732` |
| Strip singleton wrappers; root documents use the fallback section; unmatched top folders become sections | `plan.rs:703–722` |
| Match a document's full stem to a sibling folder; unmatched deeper folders create placeholder pages | `plan.rs:302–350` |
| Use declared title/first heading; remove that heading from body | `plan.rs:506–525`; HTML title `html.rs:228–240`, Markdown/front matter `markdown.rs:38–89` |
| Validate referenced images, render pages and resolve page links | `plan.rs:816–839`, `:844–892`; `doc.rs:486–494` |
| Discover databases from the **capped skip report**, then group CSV views/id and prefer `_all` (maximum 200 databases) | `commands/import_notes.rs:217–219`; `commands/notion_db.rs:94–145` |
| Match row pages by clean folder, or flat leading properties plus title; infer CSV property types | `notion_db.rs:152–160`, `:231–270`; `database/csv.rs:52–153`, `:373–480` |
| Put a database with its first classified row page, else the import's first section | `notion_db.rs:342`; `import_notes.rs:231–233` |
| Pair row bodies by title/order; retain linked pages (only at <=3,000 pages) and pages with children | `notion_db.rs:337–381` |
| Remove moved pages and up to eight levels of empty parents, without checking whether a parent was a generated folder | `import_notes.rs:274–297` |
| Save items atomically only after those steps; not exercised with private items here | `import_notes.rs:223–253` |

The module `src-tauri/src/import.rs` handles portable-vault copying; the Notion pipeline uses `commands/import_notes.rs` and `commands/notion_db.rs`. The historical carry-over description predates database construction from CSV. N1 leaves that historical description unchanged.

## Verdicts on Ahmed's two reports

1. **Missing databases: refined; the proposed no-row-page cause is refuted for these seven sources.** E1 has no CSV; E2/E3/E4/E5/E6/E7 produce 1/2/1/1/1/105 databases. Full-export groups without matching row documents still import, including three `_all` groups. The skip message is intermediate and misleading. Two real defects are independently reproduced by invented inputs: a CSV-only source fails at `plan.rs:664`, and a CSV beyond the 500-entry skip-report cap never reaches `import_notes.rs:217`. Neither condition occurred in the measured seven exports. A database absent from Notion's export cannot be counted as importer loss. Native views, formula definitions, property semantics and relationships require a separate fidelity comparison; CSV does not carry a complete Notion schema.
2. **Arrangement chaos: confirmed; missing-home and deletion claims need separation.** E2's 235 loose row pages and E6's detached surviving home reproduce concrete arrangement problems. Exact-stem matching fails on clean-name folders. Row safeguards retain a separate source page rather than making links/children point at the canonical row. E4 has no standalone pages because its view is suppressed and its row documents are folded; E5 supplies no separate home. An invented title-only home is wrongly pruned by `drop_row_pages`, but **zero such real-source prunings were observed**. Deleting an actual row through its page editor affects that same row by design (`commands/notes.rs:25–47`, `:652–655`); the sidebar lists pages separately and does not list database rows (`src/components/Sidebar.tsx:1192`). The private-data claim that deleting an independent loose page removes a distinct row is **not reproduced**. A person must establish the selected item's kind/id and navigation route.

## Visible arrangement proposals for Ahmed — before any fix

1. Keep an exported home above its children when the folder drops the id suffix; avoid creating a second section for those children. Define ambiguity handling for same clean names first.
2. Give each imported database row one canonical identity/body; redirect existing page links and preserve children under that row before removing the independent source page. Preserve navigation across both small and >3,000-page imports.
3. Keep genuine source homes even when their body is only a title; remove only proven generated placeholders during cleanup.
4. Place a CSV-only database in its source container rather than the first unrelated section, and allow CSV-only imports to create that container.
5. Decide whether an HTML database-view wrapper with useful navigation should become a preserved home/page or navigation on the database. Do not label every view as a missing home or create an empty duplicate automatically.
6. Warn about conflicting view identities. Do not merge two databases merely because their titles or row titles match.

These are proposals, not accepted arrangement decisions. Flat links alone do not prove parentage; automatic parenting of every link is specifically **not justified** by the available evidence.

## Proposed fix ranking by size

| Size/order | Proposal | Arrangement effect | Evidence |
|---|---|---|---|
| Small 1 | Discover CSV inputs independently of the capped presentation list; correct the intermediate skip wording and final summary counts | None for already successful sources | Invented cap failure; seven exports below the cap |
| Small 2 | Carry generated/source provenance into cleanup; never prune a genuine title-only source home | Keeps homes visible | Invented cleanup failure; not observed on these private sources |
| Medium 3 | Make a container plan for CSV-only inputs; retain source-container mapping for databases without row documents | New/correct notebook/section placement | Invented CSV-only and fallback-section failures |
| Medium 4 | Pair id-less folders with source pages by bounded, unambiguous identity/clean-name rules | Restores parentage; fewer stray sections/placeholders | E6 and 229 clean-name pairs in E7; invented folder failure |
| Medium–large 5 | Canonical row mapping before link rendering; transfer bodies, retarget links and children; remove the 3,000-page semantic switch | Removes loose row copies while keeping navigation | E2/E6/E7 and invented linked-row failure |
| Medium, policy first | Preserve meaningful view navigation; report conflicting view identities without title-only merging | May retain a dashboard or distinguish databases | E3/E4; policy-probe tests, not blanket defect verdicts |
| Large, after correctness | Reduce archive/materialization memory and improve phase progress/cancel measurements; keep every hostile-file limit | No intended rearrangement | E7 completes, but uses ~1.35 GiB peak RAM; real save/UI costs unknown |

N2 should take bounded correctness/arrangement work only after Claude and Ahmed resolve the policies above. N3 should measure the remaining memory, progress/cancel and summary work with the hostile-input review findings alongside this report. Raising limits is not the proposed fix.

## Invented regression/policy probes

All inputs are tiny made-up trees constructed in `src-tauri/src/importers/n1_tests.rs`; none comes from private names, values, formulas or export ids. All nine have `#[ignore = "fails until WP-19 fix lanes N2/N3 land"]`, so they do not make the default suite red. Explicit execution currently gives **0 passed, 9 failed**. Three are labelled policy proposals in the code and need a final contract before a fix lane treats their desired outcome as mandatory.

| Test | Reproduces / expected future behavior | Current |
|---|---|---|
| `n1_csv_only_source_reaches_database_import` | CSV-only plan rejected before database stage | fail |
| `n1_database_discovery_does_not_depend_on_capped_skip_report` | 500 unrelated skips hide a later CSV | fail |
| `n1_database_without_row_documents_stays_in_source_section` | Row-less CSV falls into the first unrelated section | fail |
| `n1_title_only_source_home_survives_row_cleanup` | Empty-body genuine home removed with generated parents | fail |
| `n1_linked_row_has_one_body_and_links_target_the_row` | Linked source page retained beside a new row; future contract also checks row body and link target | fail |
| `n1_folder_without_id_keeps_its_source_home_as_parent` | Id-less sibling folder fails full-stem parent matching | fail |
| `n1_flat_home_links_define_containment` | Proposed explicit navigation-to-containment policy; arbitrary links are not ownership evidence | fail; policy |
| `n1_source_home_in_html_is_not_discarded_because_csv_shares_id` | Proposed preservation of view-wrapper content; id match currently suppresses it | fail; policy |
| `n1_same_named_csv_views_with_different_ids_report_ambiguity` | Proposed ambiguity warning for same display name/different ids | fail; policy |

`n1_private_count_probe` is an additional ignored **measurement harness**, not a failing regression: seven ZIP, six folder and two combination runs all exited 0. It catches panics without printing their arguments, outputs counts/categories only, requires a new `home-*` directory under the environment-supplied test root, and never persists imported items. No private source path is embedded in repository code.

## Checks, remaining limits and security self-check

Checks ran in this Codex session on Gaming-PC at `623ddd9192faa178695d72e07f4f1d27e74d363e` plus the final test-file worktree edits (SHA-256 `AB7EB99367666C51D98BEE50943793119CE53304A95C19AAAAA99576ED0C8353`). Counts probes preceded the final assertion/comment-only additions; their executable contains the same measurement harness and production code as the final suite.

- **`cargo test --lib --offline`: exit 101; 716 passed, 1 failed, 15 ignored, 0 measured, 0 filtered; 169.07 s test execution.** Sole failure: `shell::tests::the_registry_value_can_be_written_read_and_removed`, Windows error code 5 opening the startup registry list (the known sandbox restriction). Six existing compiler warnings: one unused variable and five dead-code warnings. No importer test failed in the default suite. This is not an all-green suite.
- **Explicit invented probes:** `cargo test --lib --offline n1_ -- --ignored --skip n1_private_count_probe --test-threads=1`: exit 101; 0 passed, 9 failed, 0 ignored, 723 filtered, 0.67 s. All nine expected failures were observed on the final test file.
- **Private measurement harness:** 15 isolated runs, all exit 0 and stderr 0 bytes; seven ZIPs, six folders, two combinations. No private item was saved.
- **`git diff --check`: exit 0**, no whitespace errors; Git reports only its LF-to-CRLF conversion notice.
- **Privacy check:** external `privacy-1800.py` performs a grep-equivalent fixed-string/word-boundary scan against archive entry components, filename/stem variants, normalized/percent-decoded clean names, Markdown titles and HTML titles/headings, retaining the terms only in RAM. It reads complete document text up to a 20 MiB guard; zero documents exceeded that guard. It scans this note and the entire invented test file, plus the four-line registration (the only lane change to the third file). Numeric count collisions and a declared generic technical/fixture vocabulary are excluded; raw collisions were checked by location without printing matching strings. Final receipt: **0 non-generic name/title matches and 0 filename matches**. No matching private string or list was printed or saved.

Production importer behavior is unchanged. Lane file changes from `d910034`: this new note, `src-tauri/src/importers/n1_tests.rs`, and the **inherited four-line `#[cfg(test)]` module registration** in `src-tauri/src/commands/import_notes.rs` (checkpoint `29aa75d`). This fresh session changed only the note and test file; no additional mechanical edit was needed.

The external heartbeat made automatic checkpoint `e3326900b82bf38f3f006025fc22e6699351802a` at **2026-11-09 18:18:33 UTC**, while checks/reporting were in progress. Its changes are only this note and the tested additions to the test file; the test-file hash above is unchanged. Final check/privacy details are subsequent note-only worktree edits. Codex issued no commit or staging command. Report the check source as this checkpoint plus the note edits, with the test-file identity above, rather than assuming the heartbeat reran any check.

Not run: Notaya application/browser, real 1.5.1 executable, native picker, persistent private-data import, UI/back-navigation/delete reproduction, actual attachment encryption/storage, backup callback, live progress/cancel/lock, npm checks/audits, clippy, cargo audit, Claude Security Guidance or independent Claude security review. No network/install was attempted. Source comparison with `95ca8b2` found the relevant planner decisions and `notion_db.rs` logic unchanged; current branch adds model bookkeeping/block-id rendering, so these are current-library observations with a static link to 1.5.1, not runtime proof on the released binary.

A person-controlled throwaway run on Ahmed's PC would add: the exact selection and import mode; persisted counts after reload; screenshot/navigation evidence for the home and loose pages; whether a selected object is a row or an independent page before deletion and the effect after restore; actual save-time/working-set/attachment costs; and cancel/lock rollback. It would also establish which databases existed in Notion but were absent from the export. Nothing in this investigation establishes aesthetic acceptance or authorizes N2/N3 to choose an arrangement policy.

**Security self-check (self-review, not independent):** reviewed the test-only entry point, environment boundaries, panic/output handling, new inspection tools, source integrity, path containment, reparse/link checks, exclusive writes and counted expansion limits. Real export content is treated as hostile data, never instructions; no active HTML or formula was executed, no URL fetched, no real vault accessed. Secrets/crypto/IPC/AI pipe/history/front-end/CSP/dependency changes: n/a. The Rust archive guards were inspected as part of the behavior map; this is not a full hostile-file security audit. No security behavior was weakened or fixed. The 71 disk-name refusals are a tool-policy difference, not a demonstrated archive escape. Further adversarial archive exhaustion, concurrency/races, UI and independent security review remain not run.

**Handoff:** Claude reads the complete note and test diff, records usage/session/claims and performs its independent review. Codex stops at N1: no fixes, commits, staging, pushes, checkout, merge, reset or further lane starts.
