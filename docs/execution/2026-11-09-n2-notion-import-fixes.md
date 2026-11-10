# WP-19 N2: Notion import correctness and arrangement

## 1. Boundary and authority

Ahmed (AE0101), Gaming-PC, Windows sandbox. Baseline: `05a786b454acffd79692471bdb913c4a4852c7d3`, branch `AE0101/v1-6-n2`, clean on entry. First recorded UTC clock: 2026-11-09 23:02:34 (system clock). Ahmed's current task message has no exposed timestamp; authority also includes A52 (2026-11-09 10:42:50 UTC), A53/Q66 (19:31:00 UTC), WP-19 and the 19:35 UTC brief. The local `origin/main` brief was read without fetching. The baseline already contains unfinished step 1 from the automatic checkpoint; this run verifies and completes it before proceeding.

Write area: importer modules/tests, commands/import_notes.rs, commands/notion_db.rs, database/csv.rs and this note. Return: bounded fixes, invented regression tests, exact checks, six visible effects and security self-check. Stop after steps 5/6; canonical rows, scale/progress work and private reruns belong to N3/Claude. No app/browser, network, installs, dependency changes or agent Git writes. Claims, ledger, full session record, commits, pushes and independent review remain Claude's responsibility under the current narrower brief. Result state: **produced with scoped verification; the all-green completion gate is unmet**, not integrated or accepted.

Configured lane model/effort: `gpt-6.1-sol` / `xhigh` per the local brief. Served model, exact effort, token receipt and before/after quota meters are not exposed to this session; Claude must use the runner receipts for its ledger. No usage numbers are inferred.

## 2. Changes and step evidence

All fixtures are invented; temporary test data is directed to `D:\Notaya Tests\v1-6-N2`.

1. CSV discovery: completed the inherited checkpoint. `Plan.csv_paths` is independent of the 500-entry presentation cap. CSV entries say they are pending, then final outcomes replace them: imported CSVs do not count as omissions, failed CSVs are listed within the cap with an overflow note, and final page counts follow row cleanup. The N1 helper now reads the real input list. Explicit ignored-test run: `n1_database_discovery_does_not_depend_on_capped_skip_report`, 1 passed, 0 failed (exit 0), baseline production/test source; its ignore was then removed. Final suite will verify registration without ignore.
2. Page provenance: every planned page now explicitly records whether it is a generated folder placeholder. Pages whose bodies were explicitly moved into rows are still removed; ancestor cleanup can remove only generated ids. Real source documents survive even with an empty body. `n1_title_only_source_home_survives_row_cleanup`: 1 passed, 0 failed (exit 0), then unignored and strengthened to check that generated containers are still removed. No persistent model/storage field was added.
3. CSV-only containers: CSVs participate in the source directory tree and retain a section index through section pruning/remapping. Zero-document plans are allowed when CSV inputs exist. CSV-only folders do not generate empty pages. Database construction uses the CSV's mapped source section when there is no row page, and refuses a missing mapping instead of choosing the first unrelated section. Both named N1 tests passed (2 passed, 0 failed, exit 0) and were unignored; extra assertions cover a complete CSV-only database and an isolated CSV-only section beside unrelated documents.
4. Id-less folders: precomputed sibling-name groups permit a clean-name match only when exactly one document and one folder qualify, the document has the removed suffix, and the folder has none. Full names, not truncated display titles, define matches. Exact unique stem matches are retained. Ambiguity leaves unmatched structure in place and adds a warning; no links are used as ownership evidence. The named N1 test passed (1 passed, 0 failed, exit 0), then unignored and strengthened to assert one section and no duplicate placeholder. Additional tests cover multiple page/folder candidates and different parents.
5. HTML wrappers and conflicting names: HTML wrappers sharing the CSV id are sanitized through the existing converter, omitting collection-content view tables (or unclassed legacy view tables); explicitly marked simple-table content survives. Wrappers with remaining useful content become source pages; empty ones add nothing. HTML/Markdown twins still yield one page and preserve navigation. CSV grouping still uses source identity; a full-name group with different source ids adds one warning, without title-based merging. Both named N1 probes passed (2 passed, 0 failed, exit 0), then unignored. Combined N1/N2 checks so far: 14 passed, 0 failed, including empty/useful wrappers, sanitizer preservation, twins, ambiguity and the negative flat-link policy test.
6. Arbitrary links: rewrote the flat-home probe as a negative assertion that links do not define parentage. It passed explicitly, so the obsolete policy ignore was removed. The canonical-row probe remains ignored with a specific N3 reason. No link-based containment was implemented.

Compatibility conflict awaiting Ahmed's answer: existing tests `a_page_in_both_a_markdown_and_an_html_export_is_imported_once_with_the_html_formatting` and `markdown_csv_and_html_files_together_complete_each_other_into_database_rows` expect shared-id wrappers containing `<p>the table view</p>` / `<p>table view</p>` to disappear. A53 requires retaining that content. Their old expectations remain unchanged while the explicit clarification is pending; neither failure is hidden by ignoring its test. A green suite requires reconciling these obsolete expectations with the instruction to keep every existing test unchanged. No special-case text suppression was added to make the old fixtures pass.

## 3. Visible arrangement (A53)

| Item | Effect for a person | N2 state |
|---|---|---|
| 1 | A uniquely matched exported home holds its children without a second section; ambiguity is reported. | Synthetic checks pass |
| 2 | One canonical row body, with links and children directed to the row. | N3, not implemented here |
| 3 | A genuine title-only home survives row cleanup. | Synthetic checks pass |
| 4 | A CSV-only database has a container in its source section. | Synthetic checks pass |
| 5 | Useful content outside an HTML database view survives; an empty wrapper adds no page. | Synthetic checks pass; old expectations conflict |
| 6 | Same-name databases with different ids stay separate and get a warning. | Synthetic checks pass |

## 4. Checks, risks and hand-back

The `cargo test --lib --offline import` compatibility run completed: **command-wrapper exit 1, 146 passed, 2 failed, 2 ignored, 612 filtered, 36.90 s test execution**. The two failures are exactly the A53 wrapper-expectation conflicts above; every other selected importer/portable-import/roundtrip test passed unchanged, including hostile paths, archive budgets, sanitizer and rollback checks. Six existing compiler warnings (one unused variable, five dead-code warnings), no new warning.

**Final required run:** `cargo test --lib --offline`, Gaming-PC sandbox, started 2026-11-09 23:13:29 UTC (system clock): **exit 101; 752 passed, 3 failed, 7 ignored, 0 measured, 0 filtered; 145.43 s test execution**. Failures: the two unchanged wrapper-expectation conflicts above and `shell::tests::the_registry_value_can_be_written_read_and_removed` (Windows code 5 opening the startup list, the known sandbox-only registry failure). Six existing compiler warnings; no importer failure beyond the two incompatible old expectations. No all-green claim.

Raw log: `D:\Notaya Tests\v1-6-N2\n2-lib-20261109T231329Z.log`. Exact source receipt: `n2-source-20261109T231329Z.json` in the same folder. The external heartbeat made checkpoint `d6ba0738be1bac45c0370aec3a4e06e139009e2f` at 23:11:04 UTC; the full suite ran on that checkpoint plus the final working-tree code/test edits. The five changed Rust files were hashed before the run and compared unchanged afterwards; later edits were to this note only. Agent Git writes: none.

| Rust source relative to src-tauri/src | Final SHA-256 |
|---|---|
| commands/import_notes.rs | AE7ABD194EB4822658A8A8A00B64873E8E4EEB57881110D978B4C832DC749FD6 |
| commands/notion_db.rs | 55E77E37E520C74CFEA423841583E75EFE16E41DEA8FF04097628F16FB290177 |
| importers/html.rs | 429534888A16722B80DE4F1A1ED08828F3E189B1E248F273F5F50AE62CA4D6B9 |
| importers/n1_tests.rs | C3044CDFDDEFF88E52C1469F134D3708B845E52C8859BB92A375274F1FAC1F48 |
| importers/plan.rs | 4D52A6AEDBAD4296CD61FDC6161EE27FB118386AFA2170D9FF3D65EA389E166F |

`git diff --check` and `git diff --check b19d598`: **exit 0**; only the LF-to-CRLF informational notice for this note. No screen-side source changed, so the conditional `npm run build` was not run. All temporary test data was directed to the authorized throwaway root; no real data or application was used.

Eight of the nine N1 regression/policy probes now run normally and pass:

| Step | Test | Final result |
|---|---|---|
| 1 | n1_database_discovery_does_not_depend_on_capped_skip_report | Pass, unignored |
| 2 | n1_title_only_source_home_survives_row_cleanup | Pass, unignored |
| 3 | n1_csv_only_source_reaches_database_import | Pass, unignored |
| 3 | n1_database_without_row_documents_stays_in_source_section | Pass, unignored |
| 4 | n1_folder_without_id_keeps_its_source_home_as_parent | Pass, unignored |
| 5 | n1_source_home_in_html_is_not_discarded_because_csv_shares_id | Pass, unignored |
| 5 | n1_same_named_csv_views_with_different_ids_report_ambiguity | Pass, unignored |
| 6 | n1_flat_home_links_define_containment | Pass as a negative policy assertion, unignored |
| N3 | n1_linked_row_has_one_body_and_links_target_the_row | Ignored: canonical row identity/body, links and children are N3 |

The seven full-suite ignores are the canonical-row probe, the private count-only measurement harness (never invoked), three pre-existing Windows pipe checks requiring PC permissions, the one-time old-engine golden capture and the pre-existing sample-PDF writer. No new compatibility test was ignored. Eight additional N2 regressions pass, including persisted summary counts and failures beyond the presentation cap.

**Security self-check (self-review, not independent):** reviewed the full lane diff against security-guide section 4. The only new data path is Vfs CSV paths -> bounded plan/source-section mapping -> existing CSV parser/database constructor -> existing atomic vault save; retained wrapper content goes through the existing HTML sanitizer and document renderer. Zip entry/count/ratio/expanded-byte guards, bounded inflate during reading, checksums, source containment and symlink/reparse checks are unchanged; regression checks for traversal, lying ZIP sizes/budgets and active HTML passed. No disk extraction, new fetch, raw HTML sink, command, SQL construction, dependency, cryptographic/key, permission, AI pipe, history, clipboard, auto-lock or CSP change. No private export was accessible or read; every added fixture is invented. No network, application/browser or debug listener was started. This is not an independent hostile-file audit; S/N3 findings and Claude's final security review remain separate.

**Open / N3:** canonical rows and navigation/children, removal of the 3,000-page semantic switch, row deletion behavior, measured memory/progress/cancel work, hostile-file follow-ups, private real-data rerun after N3 and person acceptance. Database wrapper detection still follows the existing unique shared-id classification; ambiguous duplicate HTML identities are not guessed. CSV schema still carries exported results rather than native Notion formulas/views. No limit in source.rs, zip.rs or any converter was raised. No dependency/tool was added.

**Outside write area, mechanical:** none. Changed files: commands/import_notes.rs, commands/notion_db.rs, importers/{plan.rs,html.rs,n1_tests.rs}, plus this note. No screen-side source changed; npm build, npm install/ci/audit, clippy, cargo audit, app/browser/picker checks, private rerun and independent Claude review are **not run**. Git mutations, ledger/claims/session updates and publication are Claude's work, not actions taken by this lane.

**ADDITION FOR AHMED - visible arrangement only:** an unambiguous home holds its children; a title-only home stays visible; CSV-only databases stay with their source; useful dashboard content survives; conflicting database names are reported. One canonical row with all its navigation is still N3 work. No restyling.

## 5. Run 2 correction and hand-back (2026-11-10)

### 1. Boundary, diagnosis and fixes

Ahmed (`AE0101`), Codex on `GAMING-PC`, Windows sandbox. Entry: clean branch `AE0101/v1-6-n2`, HEAD `d7209c65ac0929f47b7778cfd0e585c77317171e`. Authority is Ahmed's current bounded run-2 message (timestamp not exposed), with A52/A53 and the 19:35 UTC brief read from the existing local `origin/main`, without fetching. First captured run-2 clock: 2026-11-10 03:03:25 UTC (system). Scope: diagnose and fix the two existing failures, run the full offline library suite, update this execution record, then stop. No N3 implementation. The current instruction assigns commits, pushes, claims, ledger and session records to Claude; this session makes no Git mutations or changes to those records. Configured model/effort remains `gpt-6.1-sol` / `xhigh` per the lane brief; served identity, token receipt and before/after quota readings are not exposed here and must come from Claude's runner receipts.

Both regressions originated in **step 5**, introduced in automatic checkpoint `d6ba073`, carried into `d7209c6`. Read-only history of both files confirms that before this step, the unique HTML file sharing a CSV's source id was added to `hidden_html`; step 5 instead put it in `database_html` (`plan.rs:679` on `d7209c6`) and kept any nonempty converted body (`plan.rs:521-527` on that commit).

| Failing test on d7209c6 | Exact cause | Run-2 fix |
|---|---|---|
| `commands::import_notes::tests::markdown_csv_and_html_files_together_complete_each_other_into_database_rows`, `commands/import_notes.rs:593` | The row bodies correctly moved into rows, but step 5 also created an independent `Projects` wrapper page. The fixture's paragraph is nonempty content, even though its comment calls it only a view. Step 2's generated-parent cleanup and step 3's section mapping did not cause this failure. | The structured wrapper's one plain line becomes the existing database description only after successful database construction, and only then is its fallback source page removed (`commands/notion_db.rs:375-381`). Thus its text is preserved and no duplicate page remains. |
| `importers::plan::tests::a_page_in_both_a_markdown_and_an_html_export_is_imported_once_with_the_html_formatting`, `importers/plan.rs:1234` on d7209c6 | The same step-5 classification added `Table` to the planned standalone pages. Markdown/HTML twin merging itself still worked. The later note assertion also requires the existing phrase `HTML pages of databases`. | Recognize the structured HTML wrapper (`importers/html.rs:242`) and plan its one unformatted line separately as a database header (`importers/plan.rs:536-545`), keeping the ordinary page list unchanged. The summary explains the description/fallback choice and preserves the tested phrase. |

Header candidates still count against the existing page cap, are remapped with their source sections, and use the ordinary renderer, per-page byte cap, total text budget and cancellation checks. Only a unique shared source id connects a header to a CSV (`plan.rs:909`), never a title or its words. Formatting, links, multiple lines/paragraphs, icons, covers, generic HTML source homes and possible child folders keep a page. Incoming links, failed CSV parsing/construction, missing tables and ambiguous database ownership keep the rendered fallback page. Row identity, row link retargeting and the 3,000-page switch are unchanged.

### 2. A53 compatibility correction

**No remaining test/A53 contradiction.** The run-1 statement that these expectations required Ahmed to change the tests was too narrow; this dated correction supersedes it without rewriting the historical result. A53 item 5 says: **"A useful HTML database-view wrapper is kept as a page or as navigation on its database"**. The 19:35 brief explicitly allows **"the database's own header text"**. The existing `Database.description` is already visible below its title; it preserves a plain header without adding a standalone page. The schema and screens need no change. Nothing is classified by special-casing the fixture phrases. Empty view tables still create nothing; useful richer wrappers still keep their page.

The two old tests' complete function bodies were compared with `git show d7209c6:<path>` (line endings normalized): both unchanged. No assertion, fixture, ignore or test registration was changed in them.

### 3. Files changed in run 2

- `src-tauri/src/importers/html.rs`: identify structured wrappers after the existing bounded HTML tree build.
- `src-tauri/src/importers/plan.rs`: defer eligible headers; preserve section, page-count, render and fallback behavior; map headers by source id.
- `src-tauri/src/commands/notion_db.rs`: preserve header text in the built database description, then remove only the consumed fallback page.
- `src-tauri/src/commands/import_notes.rs`: pass the header-id map to database construction.
- `src-tauri/src/importers/n1_tests.rs`: update the two helper call signatures and add three invented regression tests (description persistence/reopen, rich content/navigation/children, failed or ambiguous tables).
- This execution record: dated correction, evidence and hand-back.

### 4. Before/after checks and exact source

| Check | Before (recorded) | After run 2 |
|---|---|---|
| Import-focused offline suite | Run 1: exit 1; 146 passed, 2 failed, 2 ignored, 612 filtered; 36.90 s | `cargo test --lib --offline import`: exit 0; **151 passed, 0 failed, 2 ignored, 0 measured, 612 filtered; 36.21 s** |
| Full library suite, sandbox | Run 1: exit 101; 752 passed, 3 failed, 7 ignored; 145.43 s | `cargo test --lib --offline`: **exit 101; 757 passed, 1 failed, 7 ignored, 0 measured, 0 filtered; 164.36 s** |
| Full library suite, PC heartbeat | d7209c6: exit 101; **753 passed, 2 failed, 7 ignored, 0 measured, 0 filtered; 264.66 s**, read from `D:\Notaya-lanes\out\N2.check-cargo-test.log` | No unsandboxed heartbeat rerun observed for the run-2 source; Claude owns that check |

The import-focused log is `D:\Notaya Tests\v1-6-N2\n2-run2-import-20261110T030903Z.log`. The full `cargo test --lib --offline` started **2026-11-10 03:10:50 UTC** on `d7209c6` plus these five Rust working-tree edits. Log: `D:\Notaya Tests\v1-6-N2\n2-run2-lib-20261110T031049Z.log`. Source receipt with before/after SHA-256 and exit code: `D:\Notaya Tests\v1-6-N2\n2-run2-source-20261110T031049Z.json`.

Finished **03:13:35 UTC** (system clock). The only failure is `shell::tests::the_registry_value_can_be_written_read_and_removed`, `shell.rs:339`: Windows refused to open the start-up list, code 5 (the known sandbox restriction). Both original importer tests and all eight active N1 probes pass. The same seven old ignores remain: N3 canonical row probe, private count-only probe, three PC-only named-pipe checks, old-engine golden capture and sample-PDF writer. Three new regression tests account for the total increasing from 762 to 765; no test was weakened or ignored. Six existing compiler warnings, no new warnings.

The external heartbeat made **`ca525901588d7804cee81c44d7ecba83ce167d8e`** (automatic checkpoint labelled 03:11:38 UTC, commit timestamp 03:11:40 UTC) while the suite ran. It contains exactly the five Rust file changes; the agent issued no Git write. Before/after source hashes match (`source_unchanged: true`), and there is no Rust diff against that checkpoint. Subsequent work changes only this note. Result: **N2 fixes verified by the named library checks on the source now in ca52590; full-suite exit remains 101 for the registry restriction; not integrated or accepted**.

| Rust source relative to src-tauri | Run-2 SHA-256 |
|---|---|
| src/commands/import_notes.rs | 556189B738FD389AE102BA131C236680D4955F0D750AC8EB6934AE56C3445193 |
| src/commands/notion_db.rs | 840EAAB9C81DCB552927434BBF7F44A64E9393CAE0F55C0F1BA78A13A4E67BFA |
| src/importers/html.rs | 4F65392B0FAAC0CF0A8435B54A2F50C88A81E29E7D186BE883A8B29C3583B373 |
| src/importers/n1_tests.rs | D432E53C5BF07FEA779614CDECF3726EF744A66C2E26A274CF8BA14015E010FF |
| src/importers/plan.rs | 9626F56A30A4D58C6217DEDF643FA6C75200B54EDC97FDA25103E743FCDD126E |

`git diff --check` and `git diff --check d7209c6`: exit 0 (only the note's LF/CRLF informational notice). Final full-suite result and source receipt were read back. No further Rust edits or additional test run followed the full suite.

### 5. Six visible changes for Ahmed (current N2 result)

| A53 item | What a person sees | State |
|---|---|---|
| 1 | A uniquely matched exported home holds its children without a second section; ambiguity is reported. | N2 invented checks pass |
| 2 | One canonical database row body, with old page links and children directed to that row. | N3; not implemented here |
| 3 | A genuine title-only source home remains visible. | N2 invented checks pass |
| 4 | A CSV-only database stays in its source container, away from unrelated sections. | N2 invented checks pass |
| 5 | Plain text from a structured database wrapper can sit below the database title, without a duplicate page. Rich content, navigation and children keep a page; a failed/ambiguous table keeps its source page. Empty wrappers add nothing. | Both old failures and all new invented checks pass |
| 6 | Databases with the same name and different source ids stay separate, with a warning. | N2 invented checks pass |

### 6. Remaining work and stop point

Stop at this N2 correction. N3 retains canonical row identity/body, redirected links and children, removal of the 3,000-page navigation switch, row deletion/restore checks, measured memory/progress/cancel work and the remaining hostile-input findings. Claude owns the private rerun after N3, the independent security review, Git commits/pushes and shared records. No private export is available to this lane. These library checks do not establish UI appearance, real-data fidelity or partner acceptance.

### 7. Security self-check (self-review, not independent)

Reviewed the complete run-2 diff and the affected conversion/rendering, CSV construction and persistence path. Hostile HTML goes through the existing bounded tree and sanitizer before it can become plain description text. Description is an existing React input value, not an HTML sink. Deferred pages retain the current page count, per-page render/text and total text budgets; source/ZIP limits and containment, reparse, archive entry/count/ratio/expanded-byte and checksum checks are untouched. Header matching uses source ids and refuses ambiguous ownership; failed construction and incoming navigation preserve the page. No raw HTML storage, new path/shell/SQL/log sink, fetch, dependency, key/crypto/IPC permission, pipe, history, clipboard, auto-lock or CSP change. Fixtures and saved throwaway vault content are invented; no private name, title, value or formula was read or copied. No application, browser, network or debug listener was started.

Existing hostile-file/sanitizer/rollback tests passed in the import-focused run. No new reachable security finding in this self-check; this is not an independent audit. npm install/ci/build/audit, clippy, cargo audit, app/browser/native-picker/UI/private-data checks and Claude's independent review are **not run** in run 2.

### 8. Outside write area, mechanical

**None.** No `source.rs`, `zip.rs`, dependency, lockfile, screen or authority-record change. Test logs, generated fixtures and source receipts are only under `D:\Notaya Tests\v1-6-N2`; build output uses the preconfigured `D:\Notaya-lanes\_target`. No Git mutation was issued.

### 9. ADDITION FOR AHMED: visible arrangement only

A simple database-wrapper description appears beneath the database title instead of as an extra page. Content needing formatting, navigation or children stays on its page. If its database cannot import, that source page stays visible. The other five arrangement items above are unchanged; canonical rows remain N3. No restyling.
