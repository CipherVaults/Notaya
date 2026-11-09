# EX-2026-11-09-65: Large databases: research and plan (no build)

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Speed up large databases" (machine: cloud container, Linux; no PC, no Remote Control), model `claude-sonnet-5-5`. Short form (4 sections). Times are UTC: message stamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `805cf77` (merge of PR #103).
- **Asked by:** Ahmed, project chat 2026-11-09 02:08:59 UTC (databases get slow above about 500 rows; a huge imported Excel must still work; find optimisations that do not hurt quality). Coordinator note 02:09:09 UTC: research and a plan, not a build; follow PBS; keep Claude's use lean; do not disturb the 1.6 lanes or the A47 gate.
- **Scope and write area:** read the database code; measure on generated, throwaway data; write the plan; add two development-only measuring tools; records. **No change to `src/` or `src-tauri/`.** Write area: `docs/plan/2026-11-09-large-database-performance-plan.md`, `tools/db-bench/`, `dev-mock/perf-table-check.cjs`, two small additions to the mock (`?rows=N` seeding in `dev-mock/database.ts`, a call timer in `dev-mock/core.ts`), `.gitignore` (one line), the records of this task.
- **Stop point:** the plan and this report. No lane is started; Ahmed decides the order (questions from Q57).
- **Result state:** *produced*. Measurements are *observed* on the cloud container only; nothing is *verified* on a PC, *integrated* or *accepted*.

## 2. Work Performed

1. Read `CLAUDE.md`, `docs/STATUS.md`, the claims board, open PRs (#70 and #45 only; nothing else touches databases), the feature inventory, the version plan, the licence study, and the database code (`database/engine.rs`, `formula.rs`, `types.rs`, `commands/database.rs`, `vault/mod.rs`, `vault/store.rs`, the table and other views, `dbStore.ts`).
2. **Rust engine bench** (`tools/db-bench`): the real engine files, generated data, 1,000 to 200,000 rows; query, sort, filter, group, formulas, rollup, one-edit cost, formula parse, search-like scan, load parse.
3. **Screen bench** (`dev-mock/perf-table-check.cjs`): real React components in headless Chromium against the mock, 5,000 rows; a temporary render counter (not committed) found why every click redraws every row; an experiment with 40 rows per page showed fewer rows alone is not enough.
4. Read licences of the one new library (`@tanstack/react-virtual`, MIT) from npm metadata and `rayon` from crates.io; wrote the ranked plan (P1 to P12), the accuracy gate and the order of work.
5. **Setup used (disclose):** to run the screen bench I ran `npm ci --ignore-scripts` in a **scratch copy** of the repository (outside the repository, in the session's scratch folder) and Cargo downloaded the bench's four small crates there. Nothing was installed on any PC, nothing was added to the repository's dependencies, and nothing outside the container changed. `CLAUDE.md` says to ask before installing; this was a throwaway sandbox measurement, but Ahmed may want to say if he wants that stricter.

## 3. Verification and What Was Not Run

- **Observed:** the figures in the plan, one run each, cloud container. The bench tool was rebuilt from the repository path and ran (1,000 rows) after being moved into `tools/db-bench`; the full table of results is from the first run of the identical source.
- **Not run:** anything on Windows or WebView2; a real vault (open time, memory, import, export at 100,000 rows); Board/List/Gallery/Timeline click timings; the effect of any change (none was made); `cargo test`, `tsc`, `npm run build`, `npm audit`, `cargo audit` (no source in `src/` or `src-tauri/` changed; the Gaming-PC heartbeat covers pushed commits).
- **Risks:** headless Chromium is not WebView2; container CPU is not Ahmed's PC; the mock backend is not Rust (its time is listed apart). Direction and ratios are reliable, absolute milliseconds are not.

## 4. Result, Open Items and Next Step

- **Result:** the engine is fast enough for 50,000 rows (0.4 to 0.7 s with formulas). The slowness is the **screen redrawing every row on every click** (about 1 s per click at 300 rows, growing to 3 to 4 s at 1,200) and the **full 13 to 21 MB reply after every change**; views above 50,000 rows are silently cut. Plan: Stage 1 quick wins (P1 stop redraws, P2 parse formulas once, P3 faster sort, P4 cut notice, P5 measuring routine), Stage 2 virtual rows and windowed replies (P6, P7, P8), Stage 3 engine caches (P9, P10; P11 and P12 only if measured).
- **Open for Ahmed (from Q57):** when to run Stage 1 and Stage 2; OK for the one npm library when its lane starts (A37 item 3).
- **Next permitted action:** on Ahmed's word, the coordinator writes Codex lane briefs from the plan. Until then nothing is built.
