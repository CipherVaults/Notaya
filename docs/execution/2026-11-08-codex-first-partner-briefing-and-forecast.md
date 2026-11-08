# EX-2026-11-08-59: Codex first, the Codex partner briefing and the roadmap forecast

- **Who and where:** Ahmed's side (`AE0101`), written by Claude in the cloud thread "Build Notaya 1.6" (machine: cloud). Written 2026-11-08 08:31 UTC (system clock).

## 1. Task Overview

- **Baseline:** `origin/main` at `4ee829f` (PR #80 merged).
- **Asked by:** Ahmed, project chat 2026-11-08 08:27:53 UTC (relayed by the coordinator 08:28:23 UTC): lean on Codex for everything it can do, onboard Codex as a partner with the full documentation and vision, keep the app's feel, do not weaken PBS, and build a roadmap forecast in the style of the FFmpeg Encoder forecast.
- **Scope and write area:** `docs/handoff/codex-partner-briefing.md` (new), `AGENTS.md` and `CLAUDE.md` (role lines), `docs/plan/approval-and-amendments.md` (authority row and A46), `docs/STATUS.md`, `tools/usage-ledger/roadmap-forecast.html` (new), this record and its index row.
- **Stop point:** records and the forecast produced; no product code, no security behaviour, no dependency.

## 2. What was done

- Wrote the Codex partner briefing (vision, feel, architecture map, PBS duties, who does what, current state, pacing) and linked it from `AGENTS.md` (start of every session) and `CLAUDE.md` (roles).
- Recorded A46 and the authority row.
- Drafted a roadmap forecast page for versions 1.6 to 2.5 (Codex and Claude quota, presets, sliders, best, likely and worst case), calibrated on the first Codex windows in `docs/ledger/`. **Not published, and deleted afterwards:** the coordinator assigned the ledger and forecast redesign to the thread "Redesign the usage ledger with forecasts"; the draft was handed to that thread and, at Ahmed's word of 08:57:27 UTC ("you may delete the old forecast draft"), removed from the repository (git history keeps it).

## 3. Checks (observation)

- The briefing's pointers were checked against the repository (`ls` of `src-tauri/src`, `src/features`, `docs`). Not run: any code check (no code changed).
- The forecast's numbers come from `docs/ledger/pace-forecast.csv` (plan points), `docs/ledger/codex-runs.csv` and `quota-readings.csv`, and Ahmed's two usage screenshots of 08:27 UTC (Codex 5-hour 38% left, weekly 74% left; Claude 5-hour 30% used, weekly 22% used). Throughput, overhead and Claude cost per plan point are assumptions, marked as such on the page. **A forecast, not a measurement.**

## 4. Open points

- The PC session's lane prompts should start with `AGENTS.md` and the briefing (sent to the PC session at the same time).
- Acceptance: pending (Ahmed).
- Result state: **produced**.
