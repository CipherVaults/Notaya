# Large databases: where Notaya gets slow, and the plan to fix it

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Speed up large databases" (machine: cloud container, Linux, no PC). Model `claude-sonnet-5-5`. Execution Summary: [EX-2026-11-09-65](../execution/2026-11-09-large-database-performance-research.md).
**Baseline:** `origin/main` `805cf77` (2026-11-09 02:21 UTC, container clock). Product: released 1.5.1 plus the merged, unaccepted 1.6 lanes.
**Asked by:** Ahmed, project chat 2026-11-09 02:08:59 UTC ("if it's more than 500 rows, it gets slower ... what if someone dumped and imported a huge Excel and wants to use data analytics ... try to see what optimizations can work"). Coordinator note 02:09:09 UTC: research and plan, not a build.
**Result state:** *produced* (research and plan). Nothing in the app was changed. Not verified on a PC, not accepted.
**Dated note (2026-11-09 07:00 UTC, [A48](approval-and-amendments.md) and its addendum, Ahmed 06:52:15 and 06:56:22 UTC):** Ahmed put **P1 to P8 into 1.6** (Stage 1, Stage 2 and the other views), in the polish step before the whole-app test closes. Wherever this plan says P8 "after P6 and P7" or "can wait for 2.x", read it as: in 1.6, after P6 and P7. Stage 3 (P9 to P12) is unchanged. The measurements below are as first written.
**Rule that governs every item below:** formulas, filters, sorts, groups, rollups and totals must give **exactly the same answers** as today. No speed-up may change an answer. See section 6 (the accuracy gate).

---

## 1. The short answer (for Ahmed)

Yes, it can be made fast, and nothing has to be given up. The surprise is **where** the slowness lives.

- **The data engine (Rust) is not the main problem.** It works out a view of 10,000 rows in about 0.1 second, 50,000 rows in about 0.5 second, even with formulas and rollups. Loading a vault with 200,000 rows takes well under half a second of parsing. Search over 200,000 rows takes 25 ms. Those are fine.
- **The screen is the main problem.** Today the table draws up to 300 rows, and **every click, selection or edit redraws all of them** (we counted 600 row redraws for one click on a 300-row table). On the test browser that is about one second per click, and it grows with every "Show more rows" you press (1,200 rows on screen: 3 to 4 seconds per click; editing a title took 20 seconds, though part of that was the test stand-in for the backend). The other views (Board, List, Gallery, Timeline) draw every row at once, so 2,000 rows already means 60,000 to 90,000 page elements.
- **The second problem is how the answer travels.** Every time anything changes, the app recomputes the whole view and sends **every row** (up to 50,000, about 13 to 21 MB of text) to the screen, which has to unpack it. A database above 50,000 rows silently shows only the first 50,000, although the stored limit is 200,000.

So the fix is three layers, cheapest and safest first: stop redrawing everything (quick, no data risk), draw only the rows you can see (a standard technique), and send only the rows you can see (moves work into Rust, removes the 50,000 limit). Then speed-ups for sort, formulas and bulk work in the engine, each proven identical by a comparison test.

---

## 2. What exists today (read from the code at `805cf77`)

| Area | How it works now | Where |
|---|---|---|
| Storage | Every row is one encrypted record in SQLite (`journal_mode=DELETE`, `synchronous=FULL`, `secure_delete=ON`, as chosen for safety). At unlock, **all items are decrypted into one in-memory map** (`Session.items`). Rows hold their cells as JSON text. | `src-tauri/src/vault/mod.rs`, `vault/store.rs`, `database/types.rs` (`Row`) |
| Limits | 200,000 rows per database (`MAX_ROWS_PER_DATABASE`); a view returns at most 50,000 rows (`MAX_ROWS_OUT`); CSV import caps 100,000 rows per the dependency study. | `commands/database.rs`, `database/engine.rs` |
| Query | `query_view` builds a fresh `Ctx` from **every item in the vault**, filters, sorts, groups, works out formulas and rollups for the rows, and returns all of them as JSON. Nothing is kept between calls. | `commands/database.rs::query_view_core`, `engine.rs::Ctx::new`, `run_view` |
| Formulas | Parsed again for every row (`formula::run` = `parse` + `eval`), memoised per (row, property) within one call. Rollups read related rows through the same memo. | `database/formula.rs::run`, `engine.rs::value` |
| After an edit | `set_cells` saves, then builds **another full `Ctx`** only to render one row; the UI then runs `query_view` again 120 ms later (all rows again). | `commands/database.rs::set_cells_core`, `row_result`; `DatabaseView.tsx` (`requery`) |
| Adding a row | `new_row` walks every item twice (count, max order). | `commands/database.rs::new_row` |
| Table view | Real `<table>`; draws the first 300 rows, "Show more rows" adds 300 more; one `useSortable` (drag and drop) per row. Not virtual. | `views/TableView.tsx` (`PAGE = 300`) |
| Other views | Board, List, Gallery, Timeline draw every row returned. Timeline has no virtual scrolling (already noted in the feature inventory). Calendar and Chart are light. | `views/*.tsx` |
| Search | Command palette scans all items in memory per keystroke; database "relation picker" search scans one database. | `commands/tools.rs::search_core`, `database.rs::search_rows_core` |

---

## 3. What was measured

All numbers are **from this cloud container (Linux, one run each, no repeats)**. They show *shape* (what grows, what does not) better than absolute speed; Ahmed's PC will differ. Re-run both tools on the Gaming-PC to calibrate. Nothing was run on Windows, in WebView2, or against a real vault.

### 3.1 The Rust engine (real engine files, generated data) — tool: [`tools/db-bench`](../../tools/db-bench/)

It compiles the **unmodified** `engine.rs`, `formula.rs`, `value.rs`, `types.rs` with two tiny stand-ins and builds N generated rows (number, select, date, text, relation to 2,000 customers; optionally 2 formulas and a rollup). It times the real `run_view` and the JSON the app would send. Release build.

| Rows | Plain view | 2 formulas | 2 formulas + rollup | Sort by number (with formulas) | Sort by a formula column | JSON sent to the screen |
|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 2 ms | 8 ms | 9 ms | 8 ms | 8 ms | 0.4 MB |
| 10,000 | 23 ms | 80 ms | 109 ms | 109 ms | 120 ms | 4 MB |
| 50,000 | 100 ms | 406 ms | 544 ms | 698 ms | 736 ms | 19 to 21 MB |
| 100,000 | n/m | 397 ms (50k cap) | 521 ms (50k cap) | 1,493 ms | 1,244 ms | 19 to 21 MB (capped) |
| 200,000 | n/m | 370 ms (50k cap) | 525 ms (50k cap) | 2,601 ms | 2,350 ms | 19 to 21 MB (capped) |

"n/m" = not measured. At 100,000 and 200,000 rows the result is cut to the first 50,000 rows *after* sorting, so the unsorted figure stays flat while sorting grows faster than the row count. Other findings:

- **After one cell edit, the extra full context costs** 0.3 ms (1k rows), 1.8 ms (10k), 11 ms (50k), 24 ms (100k), 50 to 63 ms (200k). Small, but it grows with the whole vault, not just the database.
- **Formula parsing:** parsing one typical formula takes 1.5 µs; done once per row per formula, so **100,000 parses are about 150 ms of the roughly 300 ms that two formula columns add at 50,000 rows**. Parsing once instead is free speed with identical results.
- **Sorting** looks the key up in a hash map on every comparison (`keyed[a.id]`), which is why sorting 200,000 rows costs 2 to 3 seconds.
- **Loading** (parsing stored rows; decryption not included, ChaCha20 runs about 1 GB/s): 50,000 rows 55 ms, 200,000 rows (92 MB of JSON) 254 ms. Not a bottleneck.
- **Search-like scan** over 200,000 rows with lower-casing: 25.5 ms. Not a bottleneck for rows (page bodies are longer than the generated text, so this is a floor, not a promise).

### 3.2 The screen (real React components, mock backend, generated data) — script: [`dev-mock/perf-table-check.cjs`](../../dev-mock/perf-table-check.cjs)

Headless Chromium (the same engine family as WebView2, but not WebView2), the mock UI (`npm run mock`) with a database of 5,000 tasks (`?rows=5000`). "Long tasks" = time the browser's main thread was blocked. The mock backend is JavaScript, so any time listed under "calls" is **not** the Rust engine.

| Interaction | 300 rows on screen (30,000 page elements) | 1,200 rows on screen (119,000 elements) | 40 rows on screen (4,600 elements) |
|---|---:|---:|---:|
| Select a row (checkbox) | 1,247 ms blocked | 3,689 ms | 227 ms |
| Click a cell (no data call at all) | 865 ms | 2,988 ms | 225 ms |
| Edit a title and press Enter | about 6,000 ms (includes mock backend) | about 21,000 ms (includes mock backend) | not run |
| Sort by a column header | 1,777 ms | about 10,900 ms | not run |
| JavaScript memory | 304 MB | 1,360 MB | not run |

Why a click redraws everything — counted with a temporary render counter (not committed):
1. Each of the 300 rows calls dnd-kit's `useSortable`; every state change in the table makes dnd-kit's shared context change, and **every row re-renders although its own inputs did not change** (the `memo` on `RowView` does not help). Result: 600 row renders per click on 300 rows.
2. Selecting a row changes `anySelected`, which is a prop of every row, so all rows re-render for a second reason.
3. `<SortableContext items={flat.map(...)}>` builds a new list on every render. Memoising only that list gave no real gain in the experiment (1,247 → 804 ms), so the context problem is the main one.

Drawing only 40 rows brought a click from about 900 to about 225 ms: **fewer rows helps but is not enough on its own; the redraw problem must be fixed too**, otherwise 40 rows still costs a quarter of a second per click on this machine.

Other views, 2,000 rows (page elements after opening): Board 74,000, Gallery 90,000, List 59,000, Timeline 24,500, Calendar 1,000, Chart 500. Draw times were 0.1 to 0.6 s on opening; the cost is the element count (it grows with rows without limit, and every later change redraws it).

### 3.3 Not measured (marked so nobody reads them as checked)

Real vault open time and memory with 100,000+ rows; CSV import speed and memory at 100,000 rows; export speed; the real Windows WebView2 timings; how long the screen takes to unpack a 20 MB reply; Board, List, Gallery, Timeline interaction (click) timings; the real effect of any proposed change (no change was made); `cargo audit`.

---

## 4. The plan, ranked by gain for the risk

Sizes use the version plan's scale (XS, S, M, L, XL). "Gain" is from the measurements above. "Fits" is my proposal; Ahmed decides (question Q57 in the thread report).

### Stage 1: quick wins, small and safe (do first)

| # | Change | Gain | Risk | Size | Fits |
|---|---|---|---|---|---|
| P1 | **Stop redrawing every row on every click.** Separate the drag-and-drop row from a plain row (only mount `useSortable` when dragging is allowed, i.e. no sort or filter active, and only for the visible rows), make the selection and "any selected" state a CSS or attribute on the table instead of a prop of every row, keep all props stable. Same fix for Board cards, List rows, Gallery cards. | Click on a 300-row table from about 1 s to a few tens of ms; removes the cost growth per "Show more" | Low (visual and keyboard behaviour must stay identical; the existing mock UI checks cover tables) | S to M | Polish before new features (Ahmed 2026-11-09 02:07: "polished and optimised before any new features"); no data change |
| P2 | **Parse each formula once per query** (cache the parsed formula by its text inside the query context). | About 35 to 40% of the formula time at 50,000 rows (about 150 ms of 400 ms) | Very low: same parsed form, same evaluation, so identical answers | XS | Same pass as P1 |
| P3 | **Sort with precomputed keys and positions** instead of a hash-map lookup per comparison (build the key list once, sort positions, apply the same "empty last" rule and the same `compare`). | Sort cost at 50,000 rows from about +300 ms toward the cost of the keys; at 200,000 rows from 2.6 s to well under 1 s (estimate, not measured) | Low if the comparison test of section 6 passes (ties must keep today's order: use a stable sort on the same key order) | XS to S | Same pass |
| P4 | **Tell the person when a view is cut at 50,000 rows** (a visible notice with the numbers) until P7 removes the cut. Today the `truncated` flag exists but a large database simply shows fewer rows. | Honesty, no speed | None | XS | Same pass (a one-line text) |
| P5 | **Add the two measuring tools to the standing checks**: run `tools/db-bench` and `dev-mock/perf-table-check.cjs` before and after each performance change, on the PC as well, and record the numbers in the Execution Summary. | Makes every claim in this plan checkable | None | XS | Now |

### Stage 2: draw less, send less (the real fix for 100,000+ rows)

| # | Change | Gain | Risk | Size | Fits |
|---|---|---|---|---|---|
| P6 | **Virtual rows in the Table** (draw only the rows in view plus a few above and below; spacer rows keep the scrollbar honest). Library: **`@tanstack/react-virtual`** (MIT, 3.14.26 read on npm 2026-11-09, one dependency `@tanstack/virtual-core` 3.19.11, works with React 19). The frozen columns, sticky header, group headers, keyboard navigation (move to a row that is not drawn yet = scroll to it), shift-select, find, and drag reorder need explicit care. Replaces "Show more rows". | The table cost stops depending on the number of rows: 5,000 or 200,000 rows cost the same as 40 | Medium (many behaviours hang on the table; the existing UI checks must be extended, and a keyboard-only pass is needed) | M to L | Right after the 1.6 gate as its own small release, **before 1.9** (Excel import makes 100,000-row tables easy to create) |
| P7 | **Ask Rust for a window of rows, not all of them.** `query_view` returns the *order* (ids), the group headers and counts, and the footer totals; a new `view_rows(start, count)` returns the values of just the rows asked for (the visible window). Rust keeps the last computed result in memory, keyed by (database, view settings, change counter that already exists in `changes.rs`), so scrolling does not recompute. Any write to the database or a related database drops it. Removes the 50,000-row cut (real limit becomes 200,000). | Reply from 13 to 21 MB to well under 1 MB; screen no longer unpacks 50,000 rows after every edit; no more freeze after a save | Medium to high (new command, new cache with invalidation, every view that reads `result.rows` must use windows; the AI CLI's own view reads must keep working) | L | Together with P6 (the screen asks for windows as it scrolls). Needs a security review (a cache of decrypted values lives only in RAM and is dropped on lock; nothing is written to disk) |
| P8 | **Virtual lists for List, Gallery and Board columns; time-window drawing for Timeline.** Same library as P6. | Element count stops growing with rows (today 60,000 to 90,000 at 2,000 rows) | Medium (Board columns with drag and drop; Gallery grid heights; Timeline dependency arrows) | M each; Timeline L | After P6 and P7; Timeline can wait for 2.x |

### Stage 3: engine speed for analytics (do when Stage 1 and 2 have been measured)

| # | Change | Gain | Risk | Size | Fits |
|---|---|---|---|---|---|
| P9 | **Keep parsed cell values instead of re-reading JSON text per row per query** (parse `cells` once when the row is loaded or saved; keep a typed value per cell). | Roughly halves the "plain view" cost (the plain 50,000-row view is mostly JSON text parsing; to be confirmed by measuring) | Low to medium (memory grows; must stay in sync on every write; zeroise on lock like the rest) | M | Before dashboards 2.1 and 2.2 (they run many aggregate queries) |
| P10 | **Build the query context once per database change, not once per call** (cache `Ctx::new` per database, drop on write): removes the walk over every item in the vault per query and per row result. Also make `new_row` use a per-database counter instead of scanning all items (matters when the AI CLI adds thousands of rows one by one: each add currently scans the whole vault). | Edit round trip 11 ms (50k) to 60 ms (200k) saved; bulk adds stop being quadratic | Medium (cache invalidation is the risk; mitigated by the accuracy gate) | M | With P7 (it needs the same invalidation) |
| P11 | **Use all processor cores for filter, sort keys and formula columns** with **`rayon`** (1.12.0, MIT or Apache-2.0). The memo and cycle guard in `Ctx` use `RefCell` and would need to become thread-safe or per-thread. | 2 to 4 times on a multi-core PC for the heavy cases | Medium to high (the circular-reference and depth rules must still give the same errors) | L | Only if P1 to P10 leave a measured gap; not before 2.1 |
| P12 | **In-memory indexes per property** (sorted column, value buckets) for filter, sort and group-by on stored properties. RAM only. | Filter and sort from "scan all rows" (50 to 100 ms at 100,000 in Rust) to near zero. Only worth it above roughly 500,000 rows, which is above today's limit. | Medium | L | Not planned; reconsider if the 200,000-row cap is ever raised |

### Not recommended now (and why)

- **DuckDB or Polars** (both MIT) for analytics: they would hold a second copy of the decrypted data, add tens of megabytes, and their number handling (floating point, rounding, text-to-number rules) differs from Notaya's own `tidy()` rules and Notion-compatible formula semantics. Ahmed's rule is that formulas are dead accurate, so the in-house engine stays; the aggregates for dashboards come from the same engine (P9 and P10 make them fast). Reconsider only for pivot-table style work.
- **A full-text index (tantivy, MIT)**: search over 200,000 rows takes 25 ms. If ever added it must live in RAM only (no index file on disk), because an index file would hold decrypted text.
- **Changing the SQLite safety settings** (`synchronous=FULL`, `secure_delete`): no storage bottleneck was found; import is one batch transaction. Not touched. If a real 100,000-row import measures slow on the PC, measure first and talk to Ahmed before any change to durability.
- **Making the Rust side incremental** (recompute only the rows an edit touches, including rollups that read other rows): big gain on paper, but it is where accuracy bugs hide. With P6 and P7 the screen no longer waits for the full recompute. Only after P1 to P10, and only with the shadow comparison of section 6 (incremental result compared with full result).

---

## 5. Licences (checked against the dependency licence study, project file `notes/2026-11-08-dependency-licence-study.md`: permissive licences only, exact versions)

| Piece | Version read | Licence | New to Notaya? | Notes |
|---|---|---|---|---|
| `@tanstack/react-virtual` | 3.14.26 (npm, modified 2026-11-03) | MIT | Yes (npm), plus `@tanstack/virtual-core` 3.19.11 (MIT) | Needs a line in `THIRD-PARTY-NOTICES.md`. The version is days old: pin the exact version `npm` offers on the day of the build and note it; installing needs Ahmed's own word (A37 item 3) |
| `react-window` 2.5.3, `react-virtuoso` 4.18.20 | npm 2026-11-09 | MIT | Alternatives, not chosen | TanStack's is headless (fits a real `<table>` with sticky columns); `react-window` is simpler but pushes its own container markup |
| `rayon` | 1.12.0 (crates.io) | MIT or Apache-2.0 (from my knowledge of the crate; not re-read from its repository) | Probably yes (Stage 3 only) | Check it is not already in `Cargo.lock` before adding |
| Already in the app | `@dnd-kit/*`, `rusqlite`, `serde_json` | MIT / Apache-2.0 | No | Nothing to add for P1 to P3, P9, P10 |

Not checked: GitHub activity of these libraries, known vulnerabilities (`npm audit` and `cargo audit` were not run on a changed tree because nothing was added).

---

## 6. The accuracy gate (how "no answer changes" is proven)

Every item in Stage 1 to 3 that touches the engine must pass these before it is merged:

1. **Golden comparison.** `tools/db-bench` already builds generated databases. Extend it (or a Rust test in `src-tauri`) to run the **old** and the **new** code on the same data and compare the full output (`ViewResult` as JSON) **byte for byte**, for: no filter; each filter operator on every property kind; sort by every kind (including formula, rollup, select order, empty-last, ties); group and sub-group; footers (sum, average, median, min, max, range, unique, counts, percentages); formulas using every function; rollups of every calculation; circular references and the depth-32 error. Zero differences is the pass.
2. **Ahmed's real formulas.** The 50 formulas and 16 rollups of the finance system are read exactly and kept outside the repository (project file `notes/notion-formulas/2026-11-08-formulas-exact.md`, outside the repository; never committed). Run them through old and new code on a throwaway copy and compare every row to the export values as in the Notion parity rule. Do not paste their text into any tracked file.
3. **Shadow mode for caches (P7, P10).** In a debug build, after every query served from a cache, also compute it the old way and compare; fail loudly on any difference. Run it across the existing UI checks and a random edit sequence (edit, add, delete, relation change, formula change) on generated data.
4. **Screen behaviour.** The existing mock UI checks (`database-ui-check.cjs`, `database-views-check.cjs`, 115 assertions) must still pass, plus new checks for scrolling to a row that is not drawn, keyboard navigation, shift-select across a window, drag reorder, frozen columns, and group collapse in a windowed table.
5. **Real app.** On the Gaming-PC, a throwaway vault in `D:\Notaya Tests`: generate a 100,000-row database (CSV import), time open, scroll, edit, sort, filter, formula column; compare with the numbers recorded before the change. Test folder, debug port and `CIPHERLEAF_HOME` rules of `CLAUDE.md` apply.
6. **The A47 gate.** The full-app test before any release covers the old features too; a performance release is no exception.

---

## 7. Suggested order and who does what

1. **Now (this thread):** nothing else is built. 1.6 lanes are not touched.
2. **Right after the A47 full-app test finds nothing blocking** (or inside the "polish before new features" step): P1, P2, P3, P4 as one small Codex lane (brief to be written from this plan), P5 as the measuring routine. Expected gain: the table feels instant up to a few thousand rows.
3. **Next small release before 1.7 or at the latest before 1.9:** P6 and P7 together (one lane for Rust, one for the screen), then P8. This is the part that makes 100,000+ rows (a big Excel) comfortable.
4. **Before dashboards (2.1):** P9 and P10; P11 only if measurements ask for it.
5. Each lane: Codex builds from its brief, a fresh session reviews security and accuracy, Claude inspects and merges, measurements are recorded. Mohamed can pick up any lane from this file because the measuring tools are in the repository.

Claude's own use stays small: the briefs and the inspection. The building goes to Codex lanes.

---

## 8. Open questions for Ahmed

See the thread report (numbered from Q57). Short form: (a) when to run Stage 1 (inside the polish step before 1.6 ships, or after it); (b) whether the table should keep a "Show more rows" button for people who prefer it (not recommended) once it is virtual; (c) whether the 200,000-row cap should be raised after P7 (not recommended until measured on the PC); (d) OK to install the one npm library (P6) when its lane starts.

---

## 9. How to reproduce the measurements

```text
# Rust engine (from tools/db-bench; first run downloads the crates the tool lists)
cargo run --release -- 1000 10000 50000
cargo run --release -- parse        # formula parsing cost
cargo run --release -- search       # search-like scan
cargo run --release -- load         # parsing stored rows
# Screen (from the project root; needs Playwright)
npm run mock                        # leave running
node dev-mock/perf-table-check.cjs 5000        # 300 rows on screen
node dev-mock/perf-table-check.cjs 5000 3      # after three "Show more rows"
```

The numbers in this file came from one run each on 2026-11-09 between about 02:10 and 02:25 UTC (container clock), release build, a generated seed of 42, and the mock UI at commit `805cf77` plus the small additions in this PR (`?rows=N` seeding and a call timer in the mock; nothing in `src/` or `src-tauri/` was changed).

---

**Dated correction (2026-11-09 07:10 UTC, Ahmed's side, cloud; [A49](approval-and-amendments.md), Ahmed 06:56:22 UTC):** P8 (virtual List, Gallery and Board columns, Timeline time window, and any other part that needs it) is **part of 1.6** and runs after P6 and P7. Sections 4 and 7 above that place P8 later are superseded for P8 only; P9 to P12 are unchanged. Question (a) of section 8 and the P8 question are answered.
