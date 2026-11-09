# WP-19 N2: Notion import correctness and arrangement

## 1. Boundary and authority

Ahmed (AE0101), Gaming-PC, Windows sandbox. Baseline: `05a786b454acffd79692471bdb913c4a4852c7d3`, branch `AE0101/v1-6-n2`, clean on entry. First recorded UTC clock: 2026-11-09 23:02:34 (system clock). Ahmed's current task message has no exposed timestamp; authority also includes A52 (2026-11-09 10:42:50 UTC), A53/Q66 (19:31:00 UTC), WP-19 and the 19:35 UTC brief. The local `origin/main` brief was read without fetching. The baseline already contains unfinished step 1 from the automatic checkpoint; this run verifies and completes it before proceeding.

Write area: importer modules/tests, commands/import_notes.rs, commands/notion_db.rs, database/csv.rs and this note. Return: bounded fixes, invented regression tests, exact checks, six visible effects and security self-check. Stop after steps 5/6; canonical rows, scale/progress work and private reruns belong to N3/Claude. No app/browser, network, installs, dependency changes or agent Git writes. Claims, ledger, full session record, commits, pushes and independent review remain Claude's responsibility under the current narrower brief. Result state: in progress, not integrated or accepted.

Configured lane model/effort: `gpt-6.1-sol` / `xhigh` per the local brief. Served model, exact effort, token receipt and before/after quota meters are not exposed to this session; Claude must use the runner receipts for its ledger. No usage numbers are inferred.

## 2. Changes and step evidence

All fixtures are invented; temporary test data is directed to `D:\Notaya Tests\v1-6-N2`.

1. CSV discovery: completed the inherited checkpoint. `Plan.csv_paths` is independent of the 500-entry presentation cap. CSV entries say they are pending, then final outcomes replace them: imported CSVs do not count as omissions, failed CSVs are listed within the cap with an overflow note, and final page counts follow row cleanup. The N1 helper now reads the real input list. Explicit ignored-test run: `n1_database_discovery_does_not_depend_on_capped_skip_report`, 1 passed, 0 failed (exit 0), baseline production/test source; its ignore was then removed. Final suite will verify registration without ignore.
2. Page provenance: every planned page now explicitly records whether it is a generated folder placeholder. Explicitly moved row bodies still leave the standalone page, as before; ancestor cleanup can remove only generated ids. Real source documents survive even with an empty body. `n1_title_only_source_home_survives_row_cleanup`: 1 passed, 0 failed (exit 0), then unignored and strengthened to check that generated containers are still removed. No persistent model/storage field was added.
3. CSV-only containers: CSVs participate in the source directory tree and retain a section index through section pruning/remapping. Zero-document plans are allowed when CSV inputs exist. CSV-only folders do not generate empty pages. Database construction uses the CSV's mapped source section when there is no row page, and refuses a missing mapping instead of choosing the first unrelated section. Both named N1 tests passed (2 passed, 0 failed, exit 0) and were unignored; extra assertions cover a complete CSV-only database and an isolated CSV-only section beside unrelated documents.
4. Id-less folders: precomputed sibling-name groups permit a clean-name match only when exactly one document and one folder qualify, the document has the removed suffix, and the folder has none. Full names, not truncated display titles, define matches. Exact unique stem matches are retained. Ambiguity leaves unmatched structure in place and adds a warning; no links are used as ownership evidence. The named N1 test passed (1 passed, 0 failed, exit 0), then unignored and strengthened to assert one section and no duplicate placeholder. Additional tests cover multiple page/folder candidates and different parents.

## 3. Visible arrangement (A53)

| Item | Effect for a person | N2 state |
|---|---|---|
| 1 | A uniquely matched exported home holds its children without a second section; ambiguity is reported. | Pending |
| 2 | One canonical row body, with links and children directed to the row. | N3, not implemented here |
| 3 | A genuine title-only home survives row cleanup. | Pending |
| 4 | A CSV-only database has a container in its source section. | Pending |
| 5 | Useful content outside an HTML database view survives; an empty wrapper adds no page. | Pending |
| 6 | Same-name databases with different ids stay separate and get a warning. | Pending |

## 4. Checks, risks and hand-back

Pending final checks and security self-check. The private measurement probe will not run. No private export is available to this lane. Source and ZIP limits must stay unchanged. Outside write area, mechanical: none.
