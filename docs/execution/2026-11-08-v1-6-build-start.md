# EX-2026-11-08-55: Build of Notaya 1.6 starts: go recorded, briefs for Codex

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Build Notaya 1.6" (machine: cloud container; no PC, no Codex reachable from this thread). Model configured `claude-sonnet-5-5`. Short form (4 sections). Times are UTC: message timestamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `414327f` (merge of PR #67).
- **Authority:** Ahmed, project chat 2026-11-08 00:15:27 UTC ("Yes, I authorized the development of 1.6, and you can ship a tiny program that helps") and 00:18:27 UTC (Codex leads all development; Claude briefs, inspects, pushes, merges and writes the records). Relayed by the coordinator at 00:15:43 and 00:18:33 UTC. Recorded as [A44](../plan/approval-and-amendments.md).
- **Scope and write area:** records only: the authority rows and A44, the claims row, [the Codex briefs](../plan/2026-11-08-v1-6-codex-briefs.md), this report and its index row, the STATUS line.
- **Out of scope:** feature code (Codex builds it), a release, any Notion change.
- **Result state:** *produced*. Nothing built, nothing verified, nothing accepted.
- **Stop point:** this PR (records and briefs). The next step is for Codex to take the lanes, starting with A, B, C1, D and E in parallel.

## 2. Work Performed

1. Read the 1.6 spec in full, `CLAUDE.md`, the claims board and the authority and execution indexes.
2. Recorded the go, the four gates (G3 as one local, one-time copy with a notice and no AI) and the roles (A44).
3. Wrote one brief per lane (A, B, C1, C2, C3, C4, C5, D, E) with the shared rules, the order and the conflict notes of the spec's section 11.
4. Found that this container cannot run the Rust tests (no Wine, no MinGW, no GTK for a Linux build). Installing them needs Ahmed's own word (A37 item 3), so none was installed. `npm ci` was run (project dependencies, as pinned).

## 3. Verification and What Was Not Run

- **Checked:** the records and the briefs were read back after writing; the spec's lane table was followed.
- **Not run:** `cargo test --lib`, `tsc`, `npm run build`, `npm audit` (no code changed; the Rust checks cannot run here, see above).

## 4. Open Points and Next Step

- How the briefs reach Codex (Codex runs on Ahmed's PC, `D:\Notaya`): the coordinator decides; the briefs are in the repository, so a Codex checkout of this branch has them.
- Whether to install Wine and MinGW in the cloud environment so cloud threads can run the Rust checks: needs Ahmed's own word.
- Still pending for the build's milestones: the 13-section Session Summary entry, usage ledger rows, handover guide update (both copies, A39) and the Notion board; they are written when the first lane merges.


## 5. Heartbeat log (one line per event; times UTC)

- **00:43** Codex's usage limit was hit by lanes A, B, C1 and D (one shared ChatGPT account): "You've hit your usage limit ... try again at 7:25 AM" (PC local time, UTC+2, so **05:25 UTC**). Work in `D:\Notaya-lanes\<lane>` is partly written, uncommitted and intact. Lane E finished first and was pushed: `AE0101/v1-6-e` `c36397b`, [PR #70](https://github.com/CipherVaults/Notaya/pull/70), held as a draft until the AI server (C5) exists. Machine: Gaming-PC (Codex), cloud (review).
- **00:45** Heartbeat loop running on the PC (30 minutes; resumes C1, A, B, D after 05:25, then C2 and C4; two Codex lanes at a time). Task Scheduler registration denied by the PC's permission system and not worked around. Lane E's commit message footer shows a mangled middle dot (`Â·`) from a script encoding bug; not rewritten (no force-push).
- **00:46** Ahmed confirmed G3 and the 7 notebooks ("yes, yes for both"), recorded in the authority table.
- **05:25** Codex's 5-hour window reset. **05:45** the heartbeat started C1 and A (two lanes at a time).
- **06:17 to 06:23** lanes C1 (`85c8dcd`), A (`9543b87`) and D (`2e01aef`) pushed; B (`37886fd`) was already finished at its checkpoint.
- **06:24** latest Codex reading: 5-hour window 36%, weekly 22% (resets 5-hour 10:45 UTC, weekly 2026-11-14 15:44 UTC).
- **06:56** correction: the heartbeat wrongly reported C4 as finished and pushed `AE0101/v1-6-c4` (`4e3c457`) with no C4 work (Codex had stopped at a file outside its write area); the branch is to be ignored and C4 restarts on `AE0101/v1-6-c4b`. C2 also stopped for three files outside its area; the areas were widened.
- **07:13 to 07:18** PC checks on the exact commits: C1 588 passed, B 529, D bridge 16 (+517 core), A 521. Merged: C1 (PR #74), B (PR #75), D (PR #76). A held for a conflict with C1 in `commands/vault.rs`; E held (PR #70). Machine: Gaming-PC (Codex, checks), cloud (review, merge).
- **07:58 to 08:01** lane A fix pushed (`2b953a3`, main merged in, conflict solved; checks pass: tsc, build, cargo test --lib, bridge test, audit) and lane C4b pushed (`ee51e5f`, **cargo test failed, exit 101**). **08:05** lane A inspected and merged (PR #79, `2e49b7b`). PC session told to fix C4b and start C2 from the new `main`. Machine: Gaming-PC (Codex, checks), cloud (review, merge).
- **08:07 to 08:36** both restarted lanes (C4b fix, C2) stopped after 80 seconds at their write-area boundary (exit 0, no work); nothing restarted them because the heartbeat only relaunched pending lanes. **About 30 minutes of idle Codex**, noticed by Ahmed from his usage screen. Fixed: boundary stop counts as idle and is relaunched; mechanical edits outside a write area are made and listed; the interval was cut to 10 minutes.
- **08:35 to 08:53** four Codex lanes at once took the 5-hour window from 61% to 100% in 18 minutes (about 6 points of the weekly limit): **run at most 3 lanes at a time**. Limit hit 08:53:35, reset 10:45. Meanwhile lane E's check script was fixed (`0043155`, 40 checks pass) and a fresh Codex session reviewed merged lane A (0 high, 2 medium, 2 low).
- **08:59 to 09:09** Ahmed registered the permanent scheduled task (first the 30-minute version, then at 09:01 the 10-minute one "Notaya heartbeat"); the PC logs Codex and Claude meters each tick (Claude read by a tiny `claude -p` probe, about 900 tokens); Claude pause flag at 70% weekly.
- **10:46 to 10:49** a stale "try again at 12:45 PM" line was misread as tomorrow and cost about 3 minutes after the 10:45 reset; parsing fixed; three lanes started 10:48:46 (C4c fix, C2, AF).
- **11:05 to 11:07** C4b fix `514c2e8` (617 tests pass) and AF `6240e49` pushed; inspected and merged as PR #85 and PR #86. Codex readings 11:05: 5-hour 30%, weekly 36%. Machine: Gaming-PC (Codex, checks), cloud (review, merge).
