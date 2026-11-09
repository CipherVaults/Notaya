# WP-16 Stage 1: P1–P5 measurements and hand-back

Who and where: Ahmed (`AE0101`), Codex on Gaming-PC in the Windows sandbox, lane `AE0101/v1-6-p1`. Claude owns commits, pushes and the independent review. Result: in progress, uncommitted.

Authority: Ahmed's task in this session (message timestamp not supplied), A48 (2026-11-09 06:52:15 / 06:52:23 UTC). Actual starting HEAD: `5809969` (the brief named `3ec4d5e`; the two later commits are records). Start clock: 2026-11-09 07:03:47 UTC. Scope: Stage 1 only, listed database engine/UI/tools/fixtures; this numbers file and one claims row are the only documentation writes. Stop at uncommitted hand-back. No network, new libraries, real vaults or Git writes.

## Accuracy oracle

Captured using the **unchanged** engine at `5809969`, before P2/P3, with `cargo test --lib --offline capture_old_engine_golden -- --ignored --nocapture`: 1 passed. Then `cargo test --offline golden_ -- --nocapture` in tools/db-bench: **3 passed**. Each test creates fresh contexts at a fixed clock and compares the full serialized ViewResult bytes with frozen old-engine JSONL. No sorting, field normalization, rounding or hash-only comparison. The ignored capture refuses to replace nonempty files. Corpus inputs are deterministic Rust fixtures and `formulas.json`, entirely synthetic.

- `database::golden::golden_views_byte_for_byte`: all 23 property kinds, all 24 engine filter operator names (including ignored/unknown operators), ascending/descending sorts, empty values and ties/manual order, groups/subgroups, all footer calculations, nested filters, relative dates, Arabic/Hebrew, missing properties and trashed rows. 9,468,184 old bytes.
- `database::golden::golden_formulas_rollups_byte_for_byte`: every formula function, operators/lambdas/methods, syntax/runtime errors, duplicate formula text, circular references, depth-32 chain, cross-database relations and every rollup calculation over numbers/dates/checkboxes/text/formulas. 117,476 old bytes.
- `database::golden::golden_large_byte_for_byte`: 50,017 generated rows (seed 42), formula sorting/filtering, multi-key ties/empty-last, grouping/totals and the unchanged 50,000-row cap. 9,270,016 old bytes.

Ahmed's private finance formulas/export values are **not run**: not supplied to this lane and never read from real notes. Screen checks and the 100,000-row real-app round remain for Claude on throwaway data. These remain merge/release evidence requirements in plan section 6; the synthetic oracle does not replace them.

## Standing measurement routine (P5)

Run the frozen oracle first, then BOTH measuring tools before and after each performance change, on the same PC and workload. Use release Rust builds; record row count, compute/serialization times, output bytes, source identity and raw output, including regressions. Separate mock calls from Rust engine time. One run is indicative, not a statistically stable speed claim; repeat controlled runs on the PC before publishing a claim.

```powershell
# from tools/db-bench, offline and locked (dependencies already cached)
$env:CARGO_TARGET_DIR='D:\Notaya-lanes\_target\p1-db-bench'
cargo test --offline --locked golden_ -- --nocapture
cargo run --release --offline --locked -- 1000 10000 50000
# from repo root, Claude outside the sandbox, generated mock data only
npm run mock
node dev-mock/perf-table-check.cjs 5000
node dev-mock/perf-table-check.cjs 5000 3
```

Run the existing mock database UI/view checks as well. Browser numbers are **not run in Codex** (Chrome startup unavailable by the lane brief); Claude must fill before/after numbers for P1. P4 is a correctness/visibility notice and has no speed target. P5 is the measurement routine, with no speed target. P2 and P3 are measured separately as well as together.

Tool preparation before baseline: added missing `SourceTag` to the db-bench stand-in, omitted its unused OS-clock module (the engine receives an explicit clock), and selected the app's already-cached `itoa 1.0.18` instead of the tool lockfile's unavailable 1.0.19 using `cargo update --offline -p itoa --precise 1.0.18`. No new library and no app lockfile change. Initial tool runs failed on that unavailable version and unused Windows clock import; those failures are not performance results.

## Results

All numbers apply to the uncommitted source at the stage named. Raw before output follows; intermediate/final results and checks are appended below.
`n```text
==== 1000 rows ====
  [plain (no formulas)] no sort/filter                     compute      3.9 ms | JSON     0.9 ms |    0.3 MB | rows out 1000
  [plain (no formulas)] sort by Amount                     compute      3.7 ms | JSON     0.5 ms |    0.3 MB | rows out 1000
  [plain (no formulas)] filter Amount>4000                 compute      1.7 ms | JSON     0.1 ms |    0.1 MB | rows out 220
  [plain (no formulas)] group by Category + sum footer     compute      3.7 ms | JSON     0.6 ms |    0.3 MB | rows out 1000
  [plain (no formulas)] after one cell edit: row_result     0.3 ms (Ctx::new alone 0.4 ms)
  [2 formulas] no sort/filter                     compute     14.0 ms | JSON     1.3 ms |    0.4 MB | rows out 1000
  [2 formulas] sort by Amount                     compute     13.8 ms | JSON     0.9 ms |    0.4 MB | rows out 1000
  [2 formulas] filter Amount>4000                 compute      3.7 ms | JSON     0.1 ms |    0.1 MB | rows out 220
  [2 formulas] sort by formula                    compute     14.3 ms | JSON     0.8 ms |    0.4 MB | rows out 1000
  [2 formulas] filter on formula                  compute      7.9 ms | JSON     0.1 ms |    0.1 MB | rows out 133
  [2 formulas] group by Category + sum footer     compute     16.0 ms | JSON     1.6 ms |    0.4 MB | rows out 1000
  [2 formulas] after one cell edit: row_result     0.4 ms (Ctx::new alone 0.5 ms)
  [2 formulas + rollup] no sort/filter                     compute     17.7 ms | JSON     1.6 ms |    0.4 MB | rows out 1000
  [2 formulas + rollup] sort by Amount                     compute     20.7 ms | JSON     1.2 ms |    0.4 MB | rows out 1000
  [2 formulas + rollup] filter Amount>4000                 compute      5.2 ms | JSON     0.2 ms |    0.1 MB | rows out 220
  [2 formulas + rollup] sort by formula                    compute     19.2 ms | JSON     1.5 ms |    0.4 MB | rows out 1000
  [2 formulas + rollup] filter on formula                  compute      8.4 ms | JSON     0.1 ms |    0.1 MB | rows out 133
  [2 formulas + rollup] group by Category + sum footer     compute     18.5 ms | JSON     1.3 ms |    0.4 MB | rows out 1000
  [2 formulas + rollup] after one cell edit: row_result     0.4 ms (Ctx::new alone 0.3 ms)
==== 10000 rows ====
  [plain (no formulas)] no sort/filter                     compute     29.7 ms | JSON     7.3 ms |    2.7 MB | rows out 10000
  [plain (no formulas)] sort by Amount                     compute     52.4 ms | JSON     6.4 ms |    2.7 MB | rows out 10000
  [plain (no formulas)] filter Amount>4000                 compute     21.4 ms | JSON     1.3 ms |    0.6 MB | rows out 2092
  [plain (no formulas)] group by Category + sum footer     compute     37.2 ms | JSON     6.7 ms |    2.9 MB | rows out 10000
  [plain (no formulas)] after one cell edit: row_result     2.2 ms (Ctx::new alone 1.5 ms)
  [2 formulas] no sort/filter                     compute    152.7 ms | JSON    12.1 ms |    3.8 MB | rows out 10000
  [2 formulas] sort by Amount                     compute    162.4 ms | JSON    10.0 ms |    3.8 MB | rows out 10000
  [2 formulas] filter Amount>4000                 compute     43.7 ms | JSON     1.8 ms |    0.8 MB | rows out 2092
  [2 formulas] sort by formula                    compute    212.2 ms | JSON    14.4 ms |    3.8 MB | rows out 10000
  [2 formulas] filter on formula                  compute     80.6 ms | JSON     1.1 ms |    0.5 MB | rows out 1274
  [2 formulas] group by Category + sum footer     compute    159.7 ms | JSON    12.0 ms |    4.0 MB | rows out 10000
  [2 formulas] after one cell edit: row_result     1.6 ms (Ctx::new alone 1.6 ms)
  [2 formulas + rollup] no sort/filter                     compute    171.0 ms | JSON    15.6 ms |    4.3 MB | rows out 10000
  [2 formulas + rollup] sort by Amount                     compute    197.7 ms | JSON    13.3 ms |    4.3 MB | rows out 10000
  [2 formulas + rollup] filter Amount>4000                 compute     48.4 ms | JSON     2.4 ms |    0.9 MB | rows out 2092
  [2 formulas + rollup] sort by formula                    compute    221.8 ms | JSON    12.0 ms |    4.3 MB | rows out 10000
  [2 formulas + rollup] filter on formula                  compute     92.8 ms | JSON     1.5 ms |    0.5 MB | rows out 1274
  [2 formulas + rollup] group by Category + sum footer     compute    201.4 ms | JSON    18.6 ms |    4.4 MB | rows out 10000
  [2 formulas + rollup] after one cell edit: row_result     1.9 ms (Ctx::new alone 1.3 ms)
==== 50000 rows ====
  [plain (no formulas)] no sort/filter                     compute    151.6 ms | JSON    38.7 ms |   13.7 MB | rows out 50000
  [plain (no formulas)] sort by Amount                     compute    305.9 ms | JSON    35.6 ms |   13.7 MB | rows out 50000
  [plain (no formulas)] filter Amount>4000                 compute    110.3 ms | JSON     6.2 ms |    2.8 MB | rows out 10125
  [plain (no formulas)] group by Category + sum footer     compute    229.9 ms | JSON    40.4 ms |   14.4 MB | rows out 50000
  [plain (no formulas)] after one cell edit: row_result    10.1 ms (Ctx::new alone 8.4 ms)
  [2 formulas] no sort/filter                     compute    750.2 ms | JSON    69.0 ms |   19.3 MB | rows out 50000
  [2 formulas] sort by Amount                     compute    870.3 ms | JSON    55.7 ms |   19.3 MB | rows out 50000
  [2 formulas] filter Amount>4000                 compute    223.0 ms | JSON    10.1 ms |    3.9 MB | rows out 10125
  [2 formulas] sort by formula                    compute    867.7 ms | JSON    51.2 ms |   19.3 MB | rows out 50000
  [2 formulas] filter on formula                  compute    388.1 ms | JSON     6.3 ms |    2.4 MB | rows out 6240
  [2 formulas] group by Category + sum footer     compute    779.6 ms | JSON    63.5 ms |   20.0 MB | rows out 50000
  [2 formulas] after one cell edit: row_result     9.2 ms (Ctx::new alone 8.6 ms)
  [2 formulas + rollup] no sort/filter                     compute    856.2 ms | JSON    76.2 ms |   21.3 MB | rows out 50000
  [2 formulas + rollup] sort by Amount                     compute   1013.7 ms | JSON    67.3 ms |   21.3 MB | rows out 50000
  [2 formulas + rollup] filter Amount>4000                 compute    269.8 ms | JSON    13.2 ms |    4.3 MB | rows out 10125
  [2 formulas + rollup] sort by formula                    compute   1042.3 ms | JSON    67.2 ms |   21.3 MB | rows out 50000
  [2 formulas + rollup] filter on formula                  compute    450.7 ms | JSON     9.1 ms |    2.7 MB | rows out 6240
  [2 formulas + rollup] group by Category + sum footer     compute    975.3 ms | JSON    77.0 ms |   22.0 MB | rows out 50000
  [2 formulas + rollup] after one cell edit: row_result    11.3 ms (Ctx::new alone 8.5 ms)

```
