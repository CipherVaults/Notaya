# EX-2026-11-08-62: usage ledger refresh at 16:00 UTC (readings, recalibrated forecast, republished dashboard)

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Redesign the usage ledger with forecasts" (the same thread as [EX-2026-11-08-60](2026-11-08-ledger-redesign-and-forecast.md)), model `claude-sonnet-5-5` (the thread's configured model; effort not recorded). Machine: cloud. Form: the 4-section form for a small refresh; the full session entry is [session 50](../sessions.md#session-50).

## 1. Task

- **Baseline:** `origin/main` at `8ee9019` (merge of PR #93, lane AG), merged into `claude/project-thread-1lvicq`; the work goes on PR #83.
- **Asked by:** Ahmed, 2026-11-08 16:02:35 UTC (relayed 16:02:44): "make sure that explanation is clear in the Notaya roadmap, and update the usage ledger as well if you can". The coordinator added the roadmap thread's usage row (16:03:17).
- **Scope and write area:** `docs/ledger/` (inputs, versions, readings, sessions, the forecast table, the ledger block), `tools/usage-ledger/dashboard-template.html` (5-hour gauges show "not read" when there is no reading; two hard-coded readings became data-driven), records. **Out of scope:** the roadmap artifact (another thread owns it), any product code, any decision.
- **Result state:** *produced* and republished to the same artifact link; *verified* only as stated in section 2; not integrated; not accepted.

## 2. What was done and checked

1. **Readings recorded** (relayed by the coordinator, not seen on a screen by this thread): Codex week 47% used at 14:20 UTC; Codex 5-hour window ran out at about 12:01 and reset at 15:48 UTC (the new window is not read); Claude week 37% used at about 16:00 UTC. Three rows in `quota-readings.csv`.
2. **Forecast recalibrated** (`forecast-inputs.csv`, `forecast-versions.csv`): 6.3 windows a week (was 6.1); 1.6 about 25 of 45 points done, 20 left (was 27); 10.5 points per window (was 11); Claude 6% of its week per Codex window (was 3) and 3% a day (was 4). `pace-forecast.csv` was rewritten by `python3 -I tools/usage-ledger/build_dashboard.py --forecast-csv`. **Likely finish of 2.5 moved from about 2 December to about 7 December; worst case from 31 January to 19 February; Ahmed's own quota reaches 2.3** (it reached 2.0 on the 08:30 numbers).
3. **Usage row added** for the thread "Build a version roadmap showcase artifact" (as relayed: input 1,750, cache write 489,553, cache read 5,823,528, output 56,298, reported cost 3.11 USD).
4. **Page fixed and rebuilt:** the two 5-hour gauges show "not read" when the input is blank (they would have shown 0%); "Ahmed has used 26%" and the 26% in the gaps list now come from the data; the 1.6 progress estimate text says 25 of 45. Built with `python3 -I tools/usage-ledger/build_dashboard.py` and opened in headless Chromium at 1100 and 390 pixels, light and dark: no script errors, no horizontal scroll. Republished to <https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY>.
5. **White-ish theme (Ahmed, 16:11:39 UTC, relayed 16:11:43):** the page now uses the Notaya Roadmap's palette (white-ish background `#fbfcfa`, leaf green accent, Bricolage Grotesque headings) with a Theme button that cycles Light, Dark, Auto. **Light is the default**, also on a dark-mode computer; the choice is remembered in the browser (`notaya-ledger-theme`). Checked in headless Chromium: default is light, the button cycles, no script errors. The roadmap's own default is that artifact's thread's change (it starts on Auto; one line).
6. Ledger block "2026-11-08, 16:03 UTC" at the top of `usage-ledger.md`; this record and its index row; session entry 50; the claims row; one STATUS line.
- **Not run:** any test of the model against history (still one cycle); a read of this thread's, the Build thread's or the coordinators' own usage (the session list shows none of them): **not recorded**; the heartbeat's rows after 10:11 UTC (not in the repository).

## 3. Open points

- Claude's cost per Codex window is the number that now matters: 15 points of the Claude week in about 7.5 hours is an upper bound (it carries chat, records and the roadmap thread too). If it is near the top of the range, Claude's 70% cap binds before Codex does.
- The explanation Ahmed wants made clear belongs to the roadmap artifact's thread; this thread told him so.
- Feed the Build thread's Codex meter log and the heartbeat's Claude readings into the model when they reach `docs/ledger/`.

## 4. State

Produced and published (private artifact); verified by build and screenshots only; **not integrated, not accepted** (Ahmed). Next permitted action: refresh again at the next lane merge.
