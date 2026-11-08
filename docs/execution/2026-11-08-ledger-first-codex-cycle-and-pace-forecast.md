# EX-2026-11-08-57: usage ledger, first Codex cycle, and a pace forecast

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Update the usage ledger and forecast pace", model `claude-sonnet-5-5[1m]`, effort high. Machine: cloud. The Codex numbers were read on Ahmed's Gaming-PC by the "Build Notaya 1.6" thread's PC session and relayed; nothing here ran on a PC from this thread.

## 1. Task Overview

- **Baseline:** `origin/main` `1c98983` (merge of PR #71).
- **Asked by:** Ahmed, project chat 2026-11-08 00:51:23 UTC: *"also update the usage ledger now since this is our first cycle with codex, and try to predict how we are pacing so we can see how much we can actually finish"*, relayed with the coordinator's note of the same minute. A second note at 00:54:59 UTC added two small records (below).
- **Scope and write area:** `docs/ledger/` (the three CSV files, the new `pace-forecast.csv`, `usage-ledger.md`, `README.md`), `.gitignore` (one exception line so the new table is tracked), `tools/usage-ledger/` (the dashboard), `docs/plan/2026-11-07-version-plan.md` (two Notes cells), the Notion hub sync record, claims, session entry 47, STATUS line, this report and its index row.
- **Out of scope:** any feature code, any Codex run, any Notion change, any decision. No amendment (Ahmed gave no new rule).
- **Result state:** *produced*; the CSV files were read back and the dashboard was built from them; nothing is accepted.
- **Stop point:** merged ledger PR and the republished dashboard link.

## 2. What was done

1. **Codex first cycle recorded** from the PC session's report (00:55:04 UTC): six rows in `codex-runs.csv` (start, stop, tokens, 5-hour and weekly percentages), three Codex readings in `quota-readings.csv`, the Claude 5-hour status, and later readings for the coordinator, the checkpoint thread and the Build thread in `claude-sessions.csv`.
2. **Forecast** in `pace-forecast.csv` and a dated block in `usage-ledger.md`: capacity measured (one 5-hour window is about 90 lane-minutes and 15.5% of the Codex week, so the week holds about 6.5 windows), output assumed (window 1 delivered 7 to 14 of 1.6's 45 points), windows and weeks per version, and what the lanes should do.
3. **Dashboard** rebuilt with a Codex section (runs, limits, forecast) and republished to the same link.
4. **Two small records for the coordinator:** the Notes cells of 1.6-10 ("Export as" button) and 2.0-1 (box-select of sticky notes) mirrored into the repo copy of the version plan; the Notion hub sync record added as [EX-2026-11-08-58](2026-11-08-notion-hub-sync-with-version-plan.md).

## 3. Checks (observation)

- **Ran:** the dashboard builder on the CSV files (it prints the session count and totals); a read-back of every edited file; a header-width check on every appended CSV row.
- **Not run:** any app code check (no code changed); the dashboard in a browser other than a headless one; nothing was compared to a Codex screen (the numbers are the PC session's report of Codex's own rate-limit events; `/status` was not run).
- **Correction recorded:** the 00:51 relay said the lanes started at about 00:20 and the limit hit at 00:43 ("about 23 minutes"); the PC session's logs say 00:25:37 and 00:44:22 (about 19 minutes). The ledger uses the logs.

## 4. Open points

- The forecast's weakest number is how far the four cut-off Codex lanes had got (assumed 25 to 75%); inspecting them after the 05:25 UTC reset replaces it.
- Claude's weekly percentage has never been read; the 70% rule cannot be checked. A reading now and one after the 05:00 UTC reset would give a rate.
- The Codex weekly limit resets 2026-11-14 15:44:18 UTC. Whether to upgrade the Codex plan or add credits is Ahmed's decision; no price is in the records.
- Levers listed in the ledger block (shorter contexts, commit after every part, Claude building the small lanes, plan upgrade) are suggestions for Ahmed, not decisions.
