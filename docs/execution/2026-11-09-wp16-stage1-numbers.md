# WP-16 Stage 1: P1–P5 measurements and hand-back

Who and where: Ahmed (`AE0101`), Codex on Gaming-PC in the Windows sandbox, lane `AE0101/v1-6-p1`. Claude owns commits, pushes and the independent review. Result: produced; synthetic golden checks and frontend checks verified, full Rust suite has the known registry failure; handed back for independent review and PC checks. Automatic checkpoint plus uncommitted edits, described below.

Authority: Ahmed's task in this session (message timestamp not supplied), A48 (2026-11-09 06:52:15 / 06:52:23 UTC). Actual starting HEAD: `5809969` (the brief named `3ec4d5e`; the two later commits are records). Start clock: 2026-11-09 07:03:47 UTC. Scope: Stage 1 only, listed database engine/UI/tools/fixtures; this numbers file and one claims row are the only documentation writes. Stop at source hand-back for Claude; Codex makes no commit. No network, new libraries, real vaults or Git writes.

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
```text
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

### P2 alone: gate and raw timings

All three `database::golden::golden_*_byte_for_byte` tests passed after P2, before P3: **3 passed, 0 failed** (tools/db-bench, debug, same engine source included by path). No old fixture was changed.

```text
==== 1000 rows ====

  [plain (no formulas)] no sort/filter                     compute      2.9 ms | JSON     1.0 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] sort by Amount                     compute      3.5 ms | JSON     0.7 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] filter Amount>4000                 compute      1.8 ms | JSON     0.1 ms |    0.1 MB | rows out 220

  [plain (no formulas)] group by Category + sum footer     compute      3.0 ms | JSON     0.4 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] after one cell edit: row_result     0.3 ms (Ctx::new alone 0.4 ms)

  [2 formulas] no sort/filter                     compute      8.8 ms | JSON     0.7 ms |    0.4 MB | rows out 1000

  [2 formulas] sort by Amount                     compute      9.1 ms | JSON     1.2 ms |    0.4 MB | rows out 1000

  [2 formulas] filter Amount>4000                 compute      2.9 ms | JSON     0.2 ms |    0.1 MB | rows out 220

  [2 formulas] sort by formula                    compute      8.9 ms | JSON     0.9 ms |    0.4 MB | rows out 1000

  [2 formulas] filter on formula                  compute      4.0 ms | JSON     0.1 ms |    0.1 MB | rows out 133

  [2 formulas] group by Category + sum footer     compute      9.6 ms | JSON     1.0 ms |    0.4 MB | rows out 1000

  [2 formulas] after one cell edit: row_result     0.4 ms (Ctx::new alone 0.4 ms)

  [2 formulas + rollup] no sort/filter                     compute     11.5 ms | JSON     1.1 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] sort by Amount                     compute     12.5 ms | JSON     1.1 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] filter Amount>4000                 compute      3.6 ms | JSON     0.3 ms |    0.1 MB | rows out 220

  [2 formulas + rollup] sort by formula                    compute     11.1 ms | JSON     0.9 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] filter on formula                  compute      4.8 ms | JSON     0.1 ms |    0.1 MB | rows out 133

  [2 formulas + rollup] group by Category + sum footer     compute     12.6 ms | JSON     1.1 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] after one cell edit: row_result     0.3 ms (Ctx::new alone 0.4 ms)

==== 10000 rows ====

  [plain (no formulas)] no sort/filter                     compute     26.7 ms | JSON     6.2 ms |    2.7 MB | rows out 10000

  [plain (no formulas)] sort by Amount                     compute     43.0 ms | JSON     5.6 ms |    2.7 MB | rows out 10000

  [plain (no formulas)] filter Amount>4000                 compute     19.6 ms | JSON     1.1 ms |    0.6 MB | rows out 2092

  [plain (no formulas)] group by Category + sum footer     compute     36.3 ms | JSON     7.8 ms |    2.9 MB | rows out 10000

  [plain (no formulas)] after one cell edit: row_result     1.6 ms (Ctx::new alone 1.4 ms)

  [2 formulas] no sort/filter                     compute     90.0 ms | JSON    11.1 ms |    3.8 MB | rows out 10000

  [2 formulas] sort by Amount                     compute    114.9 ms | JSON     9.6 ms |    3.8 MB | rows out 10000

  [2 formulas] filter Amount>4000                 compute     31.7 ms | JSON     1.8 ms |    0.8 MB | rows out 2092

  [2 formulas] sort by formula                    compute    105.1 ms | JSON     9.3 ms |    3.8 MB | rows out 10000

  [2 formulas] filter on formula                  compute     45.2 ms | JSON     1.1 ms |    0.5 MB | rows out 1274

  [2 formulas] group by Category + sum footer     compute    103.8 ms | JSON    12.0 ms |    4.0 MB | rows out 10000

  [2 formulas] after one cell edit: row_result     1.6 ms (Ctx::new alone 1.7 ms)

  [2 formulas + rollup] no sort/filter                     compute    123.0 ms | JSON    15.9 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] sort by Amount                     compute    141.4 ms | JSON    13.5 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] filter Amount>4000                 compute     37.8 ms | JSON     2.5 ms |    0.9 MB | rows out 2092

  [2 formulas + rollup] sort by formula                    compute    156.1 ms | JSON    13.2 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] filter on formula                  compute     47.9 ms | JSON     1.5 ms |    0.5 MB | rows out 1274

  [2 formulas + rollup] group by Category + sum footer     compute    128.4 ms | JSON    14.3 ms |    4.4 MB | rows out 10000

  [2 formulas + rollup] after one cell edit: row_result     1.6 ms (Ctx::new alone 2.4 ms)

==== 50000 rows ====

  [plain (no formulas)] no sort/filter                     compute    146.7 ms | JSON    37.1 ms |   13.7 MB | rows out 50000

  [plain (no formulas)] sort by Amount                     compute    285.8 ms | JSON    30.6 ms |   13.7 MB | rows out 50000

  [plain (no formulas)] filter Amount>4000                 compute    111.9 ms | JSON     5.5 ms |    2.8 MB | rows out 10125

  [plain (no formulas)] group by Category + sum footer     compute    220.3 ms | JSON    37.9 ms |   14.4 MB | rows out 50000

  [plain (no formulas)] after one cell edit: row_result     9.8 ms (Ctx::new alone 9.8 ms)

  [2 formulas] no sort/filter                     compute    469.0 ms | JSON    72.2 ms |   19.3 MB | rows out 50000

  [2 formulas] sort by Amount                     compute    608.6 ms | JSON    54.5 ms |   19.3 MB | rows out 50000

  [2 formulas] filter Amount>4000                 compute    169.6 ms | JSON     8.9 ms |    3.9 MB | rows out 10125

  [2 formulas] sort by formula                    compute    584.9 ms | JSON    50.8 ms |   19.3 MB | rows out 50000

  [2 formulas] filter on formula                  compute    239.9 ms | JSON     6.9 ms |    2.4 MB | rows out 6240

  [2 formulas] group by Category + sum footer     compute    512.3 ms | JSON    63.6 ms |   20.0 MB | rows out 50000

  [2 formulas] after one cell edit: row_result    11.3 ms (Ctx::new alone 9.1 ms)

  [2 formulas + rollup] no sort/filter                     compute    610.5 ms | JSON    74.9 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] sort by Amount                     compute    784.8 ms | JSON    65.5 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] filter Amount>4000                 compute    195.9 ms | JSON    12.7 ms |    4.3 MB | rows out 10125

  [2 formulas + rollup] sort by formula                    compute    728.0 ms | JSON    60.4 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] filter on formula                  compute    272.3 ms | JSON     8.3 ms |    2.7 MB | rows out 6240

  [2 formulas + rollup] group by Category + sum footer     compute    686.8 ms | JSON    80.6 ms |   22.0 MB | rows out 50000

  [2 formulas + rollup] after one cell edit: row_result    10.6 ms (Ctx::new alone 9.6 ms)


```

### P2 + P3: gate and final raw timings

The same three tests passed after P3: **3 passed, 0 failed**. Empty-last in both directions, select option order, comparison function, evaluation order and stable incoming tie order are unchanged.

```text
==== 1000 rows ====

  [plain (no formulas)] no sort/filter                     compute      2.7 ms | JSON     0.9 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] sort by Amount                     compute      3.9 ms | JSON     0.6 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] filter Amount>4000                 compute      1.7 ms | JSON     0.1 ms |    0.1 MB | rows out 220

  [plain (no formulas)] group by Category + sum footer     compute      3.1 ms | JSON     0.7 ms |    0.3 MB | rows out 1000

  [plain (no formulas)] after one cell edit: row_result     0.4 ms (Ctx::new alone 0.4 ms)

  [2 formulas] no sort/filter                     compute      7.5 ms | JSON     0.8 ms |    0.4 MB | rows out 1000

  [2 formulas] sort by Amount                     compute      8.7 ms | JSON     1.0 ms |    0.4 MB | rows out 1000

  [2 formulas] filter Amount>4000                 compute      2.6 ms | JSON     0.1 ms |    0.1 MB | rows out 220

  [2 formulas] sort by formula                    compute      9.1 ms | JSON     0.8 ms |    0.4 MB | rows out 1000

  [2 formulas] filter on formula                  compute      3.7 ms | JSON     0.1 ms |    0.1 MB | rows out 133

  [2 formulas] group by Category + sum footer     compute     10.7 ms | JSON     1.6 ms |    0.4 MB | rows out 1000

  [2 formulas] after one cell edit: row_result     0.5 ms (Ctx::new alone 0.5 ms)

  [2 formulas + rollup] no sort/filter                     compute      9.8 ms | JSON     1.1 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] sort by Amount                     compute     10.8 ms | JSON     1.0 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] filter Amount>4000                 compute      3.4 ms | JSON     0.2 ms |    0.1 MB | rows out 220

  [2 formulas + rollup] sort by formula                    compute     10.0 ms | JSON     0.8 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] filter on formula                  compute      3.9 ms | JSON     0.1 ms |    0.1 MB | rows out 133

  [2 formulas + rollup] group by Category + sum footer     compute     10.5 ms | JSON     1.5 ms |    0.4 MB | rows out 1000

  [2 formulas + rollup] after one cell edit: row_result     0.3 ms (Ctx::new alone 0.4 ms)

==== 10000 rows ====

  [plain (no formulas)] no sort/filter                     compute     25.5 ms | JSON     6.5 ms |    2.7 MB | rows out 10000

  [plain (no formulas)] sort by Amount                     compute     39.7 ms | JSON     6.5 ms |    2.7 MB | rows out 10000

  [plain (no formulas)] filter Amount>4000                 compute     20.0 ms | JSON     1.3 ms |    0.6 MB | rows out 2092

  [plain (no formulas)] group by Category + sum footer     compute     37.6 ms | JSON     6.4 ms |    2.9 MB | rows out 10000

  [plain (no formulas)] after one cell edit: row_result     1.4 ms (Ctx::new alone 1.0 ms)

  [2 formulas] no sort/filter                     compute     83.6 ms | JSON    10.9 ms |    3.8 MB | rows out 10000

  [2 formulas] sort by Amount                     compute     91.3 ms | JSON    10.1 ms |    3.8 MB | rows out 10000

  [2 formulas] filter Amount>4000                 compute     30.5 ms | JSON     1.7 ms |    0.8 MB | rows out 2092

  [2 formulas] sort by formula                    compute     92.5 ms | JSON    10.7 ms |    3.8 MB | rows out 10000

  [2 formulas] filter on formula                  compute     43.1 ms | JSON     1.2 ms |    0.5 MB | rows out 1274

  [2 formulas] group by Category + sum footer     compute     95.9 ms | JSON    11.6 ms |    4.0 MB | rows out 10000

  [2 formulas] after one cell edit: row_result     1.7 ms (Ctx::new alone 1.4 ms)

  [2 formulas + rollup] no sort/filter                     compute    116.7 ms | JSON    14.1 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] sort by Amount                     compute    124.1 ms | JSON    12.0 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] filter Amount>4000                 compute     37.0 ms | JSON     2.4 ms |    0.9 MB | rows out 2092

  [2 formulas + rollup] sort by formula                    compute    123.2 ms | JSON    12.7 ms |    4.3 MB | rows out 10000

  [2 formulas + rollup] filter on formula                  compute     45.0 ms | JSON     1.5 ms |    0.5 MB | rows out 1274

  [2 formulas + rollup] group by Category + sum footer     compute    123.7 ms | JSON    13.9 ms |    4.4 MB | rows out 10000

  [2 formulas + rollup] after one cell edit: row_result     1.6 ms (Ctx::new alone 1.6 ms)

==== 50000 rows ====

  [plain (no formulas)] no sort/filter                     compute    138.6 ms | JSON    36.0 ms |   13.7 MB | rows out 50000

  [plain (no formulas)] sort by Amount                     compute    215.2 ms | JSON    29.3 ms |   13.7 MB | rows out 50000

  [plain (no formulas)] filter Amount>4000                 compute    105.9 ms | JSON     5.9 ms |    2.8 MB | rows out 10125

  [plain (no formulas)] group by Category + sum footer     compute    211.5 ms | JSON    37.2 ms |   14.4 MB | rows out 50000

  [plain (no formulas)] after one cell edit: row_result    10.9 ms (Ctx::new alone 10.6 ms)

  [2 formulas] no sort/filter                     compute    430.6 ms | JSON    63.7 ms |   19.3 MB | rows out 50000

  [2 formulas] sort by Amount                     compute    514.1 ms | JSON    54.3 ms |   19.3 MB | rows out 50000

  [2 formulas] filter Amount>4000                 compute    158.2 ms | JSON    10.3 ms |    3.9 MB | rows out 10125

  [2 formulas] sort by formula                    compute    554.3 ms | JSON    53.0 ms |   19.3 MB | rows out 50000

  [2 formulas] filter on formula                  compute    235.8 ms | JSON     6.1 ms |    2.4 MB | rows out 6240

  [2 formulas] group by Category + sum footer     compute    527.1 ms | JSON    64.2 ms |   20.0 MB | rows out 50000

  [2 formulas] after one cell edit: row_result    10.7 ms (Ctx::new alone 6.4 ms)

  [2 formulas + rollup] no sort/filter                     compute    613.4 ms | JSON    74.7 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] sort by Amount                     compute    683.4 ms | JSON    65.1 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] filter Amount>4000                 compute    200.9 ms | JSON    12.5 ms |    4.3 MB | rows out 10125

  [2 formulas + rollup] sort by formula                    compute    695.1 ms | JSON    63.9 ms |   21.3 MB | rows out 50000

  [2 formulas + rollup] filter on formula                  compute    263.8 ms | JSON     7.8 ms |    2.7 MB | rows out 6240

  [2 formulas + rollup] group by Category + sum footer     compute    683.7 ms | JSON    77.3 ms |   22.0 MB | rows out 50000

  [2 formulas + rollup] after one cell edit: row_result     9.5 ms (Ctx::new alone 8.5 ms)


```

### Comparison (compute ms, single runs on this PC)

| Rows | Workload | Before | P2 alone | P2 + P3 |
|---:|---|---:|---:|---:|
| 1,000 | plain (no formulas): sort by Amount | 3.7 | 3.5 | 3.9 |
| 1,000 | 2 formulas: no sort/filter | 14.0 | 8.8 | 7.5 |
| 1,000 | 2 formulas: sort by Amount | 13.8 | 9.1 | 8.7 |
| 1,000 | 2 formulas: sort by formula | 14.3 | 8.9 | 9.1 |
| 1,000 | 2 formulas + rollup: sort by formula | 19.2 | 11.1 | 10.0 |
| 10,000 | plain (no formulas): sort by Amount | 52.4 | 43.0 | 39.7 |
| 10,000 | 2 formulas: no sort/filter | 152.7 | 90.0 | 83.6 |
| 10,000 | 2 formulas: sort by Amount | 162.4 | 114.9 | 91.3 |
| 10,000 | 2 formulas: sort by formula | 212.2 | 105.1 | 92.5 |
| 10,000 | 2 formulas + rollup: sort by formula | 221.8 | 156.1 | 123.2 |
| 50,000 | plain (no formulas): sort by Amount | 305.9 | 285.8 | 215.2 |
| 50,000 | 2 formulas: no sort/filter | 750.2 | 469.0 | 430.6 |
| 50,000 | 2 formulas: sort by Amount | 870.3 | 608.6 | 514.1 |
| 50,000 | 2 formulas: sort by formula | 867.7 | 584.9 | 554.3 |
| 50,000 | 2 formulas + rollup: sort by formula | 1042.3 | 728.0 | 695.1 |

P1 browser before/after: **not run** (Claude on the PC). P4/P5: no speed target. P3 incremental comparison is P2 alone versus P2+P3, not baseline versus P2+P3. Small-run noise is retained: e.g. plain 1,000-row Amount sort was 3.5 ms after P2 and 3.9 ms after P3. Source remained unchanged in the engine between the final benchmark and the final Rust run.

### Source identity and immutable oracle hashes

Claude's automatic heartbeat created `2fe8bf1` at 2026-11-09 07:11:53 UTC while this run was active. Codex issued no Git write. Checks below apply to that checkpoint plus the explicitly listed subsequent worktree edits, not to an assumed clean commit. After the checkpoint, the Board optional transform was corrected and the limit notice was corrected to use the query result rather than the locally searched subset. The numbers record and claims row continue to be updated.

- `views.jsonl`: 9,468,184 bytes, SHA-256 `537da8fb712d57dde48167ead1330db4ce408f4a033e6a435f9849a083a18700`.
- `formulas.jsonl`: 117,476 bytes, SHA-256 `11781aa0589b6056e698b716987e19b8807cdcdf395f915c2f39e62c1d3ce9ad`.
- `large.jsonl`: 9,270,016 bytes, SHA-256 `d6287572f118d94b535ae9a69194bb75c6815ba5ea8e09535880f881d2808d3c`.

Outside write area, mechanical: `.gitattributes` adds `-text` for the golden JSONL files so a Windows checkout cannot turn their LF bytes into CRLF and break the oracle.

## Final checks and hand-back

- `cargo test --lib --offline` (Windows/MSVC, Codex sandbox, application crate): **716 passed, 1 failed, 5 ignored; exit 101; 150.96 s**. Only failure: `shell::tests::the_registry_value_can_be_written_read_and_removed`, sandbox denied registry access (Windows code 5), exactly the known failure in the brief. All three golden tests passed. Raw log: `D:\Notaya Tests\v1-6-P1\cargo-final.txt`. Six compiler warnings are pre-existing (unused `id` in vault/activity; unused claim/release helpers, title_prop, CALCS, with_deadline).
- `npx tsc --noEmit -p tsconfig.json`: **passed, exit 0**, final frontend source. npm offline mode enabled. An earlier run found optional Board transform could be undefined; corrected to `transform ?? null`, then rerun passed.
- `npm run build -- --configLoader native`: **passed, exit 0**, final frontend source, 4,413 modules, Vite build 9.48 s. Existing ineffective-dynamic-import warning: DatabaseView is imported both statically and dynamically. Built CSS contains the table selection attribute selector.
- `tools/db-bench`: **passed, exit 0** before, P2 alone, and P2+P3 at 1,000/10,000/50,000 rows, release/offline; complete raw numbers above. Golden tests also passed separately at each engine stage (3/3 each).
- `git diff --check`: initial record write had CRLF reported as trailing whitespace; corrected to LF, **final passed, exit 0**.
- `npm audit`: **skipped as instructed**. `cargo audit`: **not run**. No new dependency was introduced; only the tool's cached transitive version was reconciled with the app lockfile.

### What changed

P1: memoized table cell content behind a small sortable wrapper; no sortable hooks on sorted/filtered plain rows; only mounted/paged/expanded row IDs participate in sortable context; table-level selection attribute replaces every row's global selection prop. Board card content and plain cards, List rows, Gallery cards are memoized with stable inputs/callbacks. Existing Board group moves remain possible under sorting; only when neither group movement nor manual reorder can do anything is the plain card mounted. Existing mouse/keyboard handlers and classes are retained. Visual/runtime acceptance is not claimed.

P2: exact-text parsed AST/error cache in each query context, dropped with that context. Evaluation uses the same evaluator with a fresh step budget, the current row environment, and unchanged memo/cycle/depth rules. The mutable AST-cache borrow ends before recursive evaluation.

P3: precomputed keys in incoming row order, stable sort of positions, same comparison and empty-last in ascending/descending. P4: one shared visible status notice reports the capped query row count and the database total distinctly; fixes misleading duplicate List/Gallery notices. Local search does not change the reported query cap. P5: standing offline/PC before-and-after routine above.

### Still for Claude

Browser timings and `database-ui-check.cjs` / `database-views-check.cjs`: **not run** here, per sandbox brief. Run before/after on generated 5,000-row data (300 and 1,200 shown); inspect selection/row numbers, keyboard editing, drag reorder, sort/filter disabling of table drag, groups, frozen columns and the P4 notice at >50,000 rows, with and without a view filter and local search. The 100,000-row real-app test and private finance formula/rollup comparison remain **not run**. No browser or debug port was started by Codex. Test data and logs were created only under `D:\Notaya Tests\v1-6-P1`; build products use the shared target and ignored frontend dist.

Security self-check (author, **not independent**): inspected the engine diff, fixtures and changed UI/tool paths. No new input/IPC command, persistence, logging of note content, HTML sink, network code, dependency or privilege path. AST cache is per-query RAM only; immutable ASTs have no row-specific values; existing cycle/depth/step checks remain. Fixture data and benchmark output are synthetic. No findings in this bounded self-check. Claude's Security Guidance/built-in security review and a fresh independent review are **not run by this Codex author session** and remain required.

Model: Codex (exact model ID and reasoning effort not exposed in this session). Token counts and before/after quota readings: **not available**; Claude/heartbeat records the actual meters in the ledger. Only the authorized numbers file and claims row were written; no other authority/session/ledger record was edited. No subagents.

ADDITION: none implemented or proposed beyond the brief. Stop point: hand back source and evidence for Claude's review/commit/push and PC checks; no acceptance, merge or release claimed.

`cargo clippy --lib --offline` (MSVC): **exit 0, 166 warnings**, 17.46 s. This is not warning-free. Full warnings: `D:\Notaya Tests\v1-6-P1\clippy-final.txt`; no broad warning cleanup was attempted in this lane.

### Changed files (relative to starting 5809969)

```text
.gitattributes
docs/execution/2026-11-09-wp16-stage1-numbers.md
docs/handoff/claims.md
src-tauri/src/database/engine.rs
src-tauri/src/database/mod.rs
src-tauri/src/database/golden/mod.rs
src-tauri/src/database/golden/formulas.json
src-tauri/src/database/golden/views.jsonl
src-tauri/src/database/golden/formulas.jsonl
src-tauri/src/database/golden/large.jsonl
src/features/database/DatabaseView.tsx
src/features/database/views/TableView.tsx
src/features/database/views/BoardView.tsx
src/features/database/views/ListView.tsx
src/features/database/views/GalleryView.tsx
src/features/database/views/shared.tsx
tools/db-bench/Cargo.lock
tools/db-bench/src/main.rs
tools/db-bench/src/database.rs
```

Hand-back clock: 2026-11-09 07:16:43 UTC (system clock). All implementation source is unchanged since its respective final checks; subsequent writes are this record, the claims row and mechanical golden line-ending protection. Claude should commit the final tree and rerun the required checks on that exact commit. No commit/push/checkout/merge/add/reset was issued by Codex.

Clippy warning categories include 91 collapsible-if, 10 unnecessary-map-or, 7 type-complexity and 7 manual-is-multiple-of warnings; remaining warnings are in the raw log. No claim that all warnings predate this lane.

## Small fix run after Claude's review (2026-11-09)

Who and where: Ahmed (`AE0101`), Codex on Gaming-PC in the Windows sandbox. Baseline: `7b820d5e3e4769a8b2a7b743a01b26f48e4aae4d`, branch `AE0101/v1-6-p1`. Authority: Ahmed's small-fix request in this session (message timestamp not supplied); observation clock: 2026-11-09 07:46:01 UTC (system clock). Write area: `src/features/database/*` and this record only. Stop at hand-back; Claude owns commits, pushes and the database UI checks.

Restored `canDrag = view.sorts.length === 0` in `src/features/database/views/TableView.tsx:329` and removed its unused `filterIsEmpty` import at line 20. Rows can again be dragged with a filter active when no sort is set. The P1 row split is retained; Rust is unchanged. This supersedes the earlier descriptions of filtered plain rows and filter-disabled table drag above.

**FINDING FOR AHMED (observation only; no change):** "reorderRows gets only the filtered ids when a filter is active". In `src/features/database/views/TableView.tsx:481`, `onRowDrag` builds the order from `result.rows`; line 489 sends that reordered list to `reorderRows`. In plain words, dragging while a filter is on sends only the matching rows, leaving out the rows hidden by the filter. The existing Rust command (`src-tauri/src/commands/database.rs:865`) renumbers the supplied rows from zero and leaves omitted rows' order values unchanged. Clearing the filter can therefore show a different interleaving of matching and hidden rows. This is a static observation, not a runtime reproduction or an approved fix; Ahmed decides any later change.

**ADDITION FOR AHMED (existing P1 behaviour; retained):** `src/features/database/views/BoardView.tsx:285` makes cards no longer pick-up-able when sorts are on and the group property is not editable (also when a configured subgroup cannot be edited). Without manual reorder or an editable group move, line 472 renders `CardFrame` without sortable handlers. The coordinator accepts retaining this behaviour per the current brief; partner acceptance is not claimed. This corrects the earlier "ADDITION: none" statement above.

Checks on `7b820d5` plus this uncommitted TableView fix, with npm offline mode enabled: `npx tsc --noEmit -p tsconfig.json` **passed, exit 0, no diagnostics**; `npm run build -- --configLoader native` **passed, exit 0, 4,413 modules, Vite 2.26 s**. The existing ineffective-dynamic-import warning for DatabaseView remains. Rust/golden tests, database UI checks, `npm audit` and `cargo audit`: **not run in this fix run**. Claude runs the database UI suite on the new committed head. No app, browser or debug port was started.

Security self-check (section 4; author, **not independent**): inspected the P1 runtime source diff from `5809969` to `7b820d5`, this fix, the existing reorder command path, formula parse/eval budgets, synthetic harness code and the benchmark lockfile change. The AST cache is query-local RAM, keyed by exact formula text, holds no row-specific evaluated values and uses the existing evaluator with a fresh step budget; the cache borrow ends before evaluation. No new HTML/eval sink, browser persistence, note-content logging, network path or dependency was found. The benchmark lockfile pins an existing dependency, `itoa`, to 1.0.18. Secrets/crypto, new command validation/AI permissions/confirmations, hostile-file paths, AI pipe, history/restore, CSP and auto-lock changes: **n/a**, no such changes in this lane. No new security finding in this bounded static self-check; the filtered-order observation above remains for Ahmed. Not checked dynamically: cache release on lock, hostile-input/runtime behaviour or drag interactions; frozen JSONL bodies were not re-audited in full. Audits and Claude's Security Guidance/built-in security review were **not run** here; independent review remains Claude's responsibility.
