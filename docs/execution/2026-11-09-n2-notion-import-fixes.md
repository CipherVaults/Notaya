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
