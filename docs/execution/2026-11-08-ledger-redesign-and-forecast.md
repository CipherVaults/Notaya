# EX-2026-11-08-60: usage ledger redesign and the 1.6 to 2.5 forecast

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Redesign the usage ledger with forecasts", model `claude-sonnet-5-5`, effort not recorded (the thread's configured model; its own record was not read). Machine: cloud (the usage figures are Ahmed's screenshots of his PC and the Claude page, relayed by the coordinator). Written 2026-11-08 about 08:45 UTC (system clock).

## 1. Task Overview

- **Baseline:** `origin/main` `d3d0f0b` (merge of PR #82).
- **Asked by:** Ahmed, project chat 2026-11-08 08:27:53 UTC (relayed 08:28:32): lean on Codex, restyle the usage ledger so it is easy to see what uses what, and build a forecast of what the quotas can finish version by version; 08:29:17: the FFmpeg Encoder ledger is **inspiration for purpose, not for the look**; 08:35:26: *"I don't want the FFmpeg encoder forecast. I want to do something similar for Notaya."* The coordinator's note added: Codex's weekly quota plus two banked resets, Claude's overhead capped at 70%, show which versions fit Ahmed's quota and which wait for Mohamed, and fold in the Build thread's draft forecast (PR #82) so only one Notaya forecast exists.
- **Scope and write area:** `tools/usage-ledger/` (`dashboard-template.html` rewritten, `build_dashboard.py`, new `forecast-model.cjs`, a banner on `roadmap-forecast.html`), `docs/ledger/` (new `forecast-inputs.csv`, `forecast-versions.csv`, `project-usage-screen.csv`; `quota-readings.csv`, `claude-sessions.csv`, `pace-forecast.csv`, `usage-ledger.md`, `README.md`), `.gitignore` (three exception lines), the records of this PR, and the published dashboard.
- **Out of scope:** any product code, any Codex run, Notion, and any decision. **No new amendment:** Ahmed's instruction is covered by [A46](../plan/approval-and-amendments.md) (Codex first) and the quota rules of [A45](../plan/approval-and-amendments.md).
- **Result state:** *produced* and published as a private artifact; *verified* only as far as stated in section 3; not integrated; not accepted.
- **Stop point:** the republished dashboard link and a draft PR.

## 2. What was done

1. **A new dashboard page** (same link as before, <https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY>), in Notaya's own colours: the four limits as gauges (Claude's with the 70% line); a forecast with sliders and presets, a to-scale timeline of the ten versions, two allowance charts (Codex windows left, Claude percent left) and a per-version table; "where the usage went" charts (the platform's thread count, a grouping of Claude's work, the Codex lanes, the weekly limits over time, model split, new tokens versus cached rereading); "how the forecast is built" with every input marked measured or rough; then the detail tables.
2. **One forecast model** (`forecast-model.cjs`), inlined into the page and run by `build_dashboard.py --forecast-csv` to write `pace-forecast.csv`, so the page and the file cannot disagree. It adds to the Build thread's draft: a daily window cap, a pause after each version, per-version "held back by", best and worst cases, and who pays (Ahmed's quota or after his banked resets). From the draft it took the two extra Codex weekly readings (22% at 06:24, 26% at 08:07), the plan's points, and a cross-check: the draft's Claude cost per plan point (0.35% of a week) is the same size as this model's 3% per window at 11 points per window (0.27%).
3. **Readings recorded:** Ahmed's two screenshots (five rows), the two relayed Codex readings, the platform's thread usage (`project-usage-screen.csv`, which the README had named since 2026-11-07 but which was never committed).
4. **The old draft** `roadmap-forecast.html` was first marked superseded, then **deleted on Ahmed's word** (08:57:27 UTC, question 51).
5. Ledger block at the top of `usage-ledger.md`, README (new files, the model, the pacing rule, a new rule to read Claude's weekly percentage before and after each lane), this record, the session entry 48, the claims row and one STATUS line.

## 3. Checks (observation)

- The page was built from the CSV files with `python3 -I tools/usage-ledger/build_dashboard.py` and opened in headless Chromium on this container at 1100 and 390 pixels wide, light and dark: no script errors, no sideways scrolling, every chart drawn. **Not checked:** other browsers, the page inside claude.ai's own frame, and the forecast's correctness beyond reading its numbers back (there is no real data for 1.7 onward).
- The model was run in Node 22 for the default and four other settings; the default's dates are in the ledger block. **Not run:** any test of the model against history (there is only one cycle).
- **Not run:** nothing in the app (no Rust, no tsc, no build: no product code changed).

## 4. Open points

- **Questions for Ahmed, answered 2026-11-08 08:57:27 UTC (his numbers 50 to 52):** (a) a banked reset refills the whole Codex week and the 5-hour window and is used only after the week is fully spent (now an input of `rule` kind, not an assumption); (b) the old draft may be deleted (done); (c) **Mohamed has no Codex, only a Claude Pro subscription like Ahmed's**, so the model's Mohamed input became "share of his Claude week spent building" (default 0; a guessed 2% of a Claude week per plan point). The default dates did not change.
- **Also from that message (question 49):** Ahmed wants his quota controlled automatically and asked to be guided. This thread was not given that question's text; the automatic control is **not built here** and is handed back to the coordinator (see the thread's reply).
- The weakest numbers are Claude's cost per Codex window, the share of 1.6 already built (about 18 of 45 points, my estimate) and the speed (11 points per window, from one and a half windows). Replace each with a measurement when it exists.
- Not done: the handover guide for Mohamed and the Notion hub were not updated (nothing in them changed except this dashboard's description); the usage figures of this thread and the Build thread are read at the next refresh.
- Acceptance: pending (Ahmed). Result state: **produced**.
