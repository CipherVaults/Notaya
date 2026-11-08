# EX-2026-11-08-61: Refresh of the AI-facing records and the 1.6 build-day session entry

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Refresh AI PBS docs" (machine: cloud container), model `claude-sonnet-5-5`, effort high. Short form (4 sections). Times are UTC: message stamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `2ffaa2c` (merge of PR #88, 2026-11-08 11:38 UTC).
- **Asked by:** Ahmed, project chat 2026-11-08 11:41:49 UTC ("make sure the AI PBS docs are up to date and aren't stale") and 11:42:04 ("especially our session summary"); the coordinator's relay of 11:41:55 and 11:42:07 asked that only out-of-date facts change and PBS rules are never weakened.
- **Scope and write area:** `docs/handoff/codex-partner-briefing.md` (sections 2, 5, 6 and a dated header line), `AGENTS.md` (one dated note on where checks run, one short pacing section), `CLAUDE.md` (one dated note in "Cloud threads"), `docs/sessions.md` (entry 49 and its index row), `docs/STATUS.md` (one "As of" line), `docs/handoff/claims.md`, `docs/ledger/claude-sessions.csv` (this thread's row), this report and its index row. **Not touched:** the handover guide for Mohamed (the Build thread edits its section 17), lane branches, product code, the approved-plan snapshot, the amendments record.
- **Added at 16:45 UTC (second pass, after Ahmed's 16:39:49 UTC word to merge when ready):** the briefing, `AGENTS.md`, STATUS and entry 49 refreshed again for what merged since 11:45 (AG PR #93, C4d PR #92, PR #90's records, the 5% stop line, the lane start fix); three **second-hand** authority rows (11:48:49, about 09:03, 01:09:37) in the authority record, flagged as having no quoted words.
- **Out of scope:** new rules, any change of permission or security behaviour, a new amendment (none added; A44 to A46 cover the content).
- **Result state:** *produced*. Not integrated, not accepted.

## 2. Work Performed

1. Read `CLAUDE.md`, STATUS, the claims board, the sessions index and entries 46 and 47, the authority record's tail, the Codex briefing, `AGENTS.md`, the ledger files, EX-2026-11-08-55 (the heartbeat log), the pull request descriptions of #70, #83 and #88, and `git log` of `origin/main`.
2. **Found stale:** the briefing's "where things stand" (written 08:35 UTC, said C4b and C2 were in progress and "two lanes at a time"; by 11:38 UTC C4, AF and C2 were merged and the heartbeat was a permanent task); the claims row for the 1.6 build; STATUS had no line after 11:07; `CLAUDE.md` says cloud threads run the Rust tests under Wine, which is not true in Ahmed's cloud container; no session entry covered 01:10 to 11:45 UTC.
3. **Updated only those facts**, each as a dated correction (nothing old was rewritten): briefing sections 2 (C2 and logging rows), 5 and 6; the `AGENTS.md` and `CLAUDE.md` notes; the claims rows; STATUS; session entry 49.
4. **Checked and left unchanged:** briefing sections 1, 3 and 4 (vision, feel, PBS duties, who does what), the rest of `AGENTS.md` (roles, start of session, hard rules, checks, security review, hand-over), the `CLAUDE.md` rules, and the authority record's A44 to A46 text (consistent with the briefing).

## 3. Verification and What Was Not Run

- **Checked:** each new fact against `git log origin/main`, PR #88's description, the heartbeat log in EX-55 section 5, `docs/ledger/quota-readings.csv`, and `ls src-tauri/src` (the two new table rows name files that exist). The files were read back after editing.
- **Not run:** any code check (no code changed). **Not read:** the project chat after 00:00 UTC (so Ahmed's 01:09:37, 08:57:27 and 09:03 words are stated by effect from the records, not quoted), any lane diff, any Codex or Claude usage screen. The claim that C3 is held until about 15:45 UTC and that C4d and AG are running comes from the coordinator's relay and the branch list, not from a PC reading.

## 4. Open Points and Next Step

- **Authority rows:** added at 16:45 UTC for about 01:09:37, about 09:03 and 11:48:49, but **second-hand and without quoted words** (this thread did not read the project chat; the content follows the records and the coordinator's notes). **Still to do:** replace each row's description with Ahmed's verbatim words and exact time, and add rows for the answers to questions 49 to 53 if any decision beyond the rows above sits in them. Next free amendment number: A47 (check `origin/main` first).
- **Handover guide, section 17:** add C2 (merged 11:38) and the 11:38 state; the Build thread owns it; republish the page (A39).
- **Merge order (done):** PR #83 (session entries 48 and 50, EX-60 and EX-62) merged at 16:40 UTC; this PR's merge with `main` kept every entry (50, 49, 48).
- **Dated correction (2026-11-08 about 16:50 UTC):** the authority rows are now verbatim (read from the project chat; 09:03:55 and 11:48:49 as relayed by the coordinator); the quota-strategy message is 01:09:11; questions 49 to 53 held nothing beyond the rows. The second-hand rows of PR #89 were replaced. Only the handover guide section 17 and the failure-intelligence call remain.
- **Dated correction (2026-11-08 about 16:55 UTC):** the failure-intelligence entries were requested by the coordinator (16:46 UTC) and written: FI-08 and FI-09 in `docs/failure-intelligence.md`, from EX-55 section 5 only (no PC log or chat read beyond Ahmed's 08:30:44 message). FI-10 is the next free entry.
- Consider a failure-intelligence entry (the false "finished" lane and the 30-minute idle); it is the coordinator's call under the admission boundary.
- Result state: **produced**. Acceptance: pending (Ahmed).
