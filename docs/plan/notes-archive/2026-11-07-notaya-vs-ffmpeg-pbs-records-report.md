# Notaya PBS records versus the FFmpeg Encoder reference (read-only report, 2026-11-07)

Baseline: Notaya `main` at `7661503` (PR #53). FFmpeg Encoder repo `AE0101/ffmpeg-encoder` at `2a18036`. Nothing was changed in either repo.

## 1. Where Notaya stands, per its own records
- Last accepted release: **V1.5.1**, accepted by Mohamed on 2026-11-05 11:08:39 UTC (acceptance row X27). The tag sits on merge commit `95ca8b2`.
- V1.5.0 (RTL and databases) and V1.4.0 (tray, export/import) shipped, but have no acceptance rows of their own.
- Open items the records still carry: amendments A2 and A8 (waiting for an owner decision), the licence choice (needs both partners), 15 live-audit items (Q25), and A34 (two partners) which still asks Mohamed to "confirm or change Claude's reading".
- Not run anywhere, per the resume notes: the V1.5.1 setup installer, the tray entry "Close sticky notes", the keyboard-layout shortcut between Arabic and English lines, `cargo clippy` for MSVC.
- Records do not yet know about: the Codex-leads handoff on main (PR #53), Ahmed as a partner on his own PC (`D:\Notaya`), the clippy result (137 warnings), or Ahmed's 2026-11-07 change of authority (coordinator leads, merges and releases).

## 2. What Notaya has, and what it lacks, compared with FFmpeg
| Record | FFmpeg Encoder | Notaya |
|---|---|---|
| Session summary as ONE file, newest on top | Yes. `06_SESSION_SUMMARY.md`: 2,756 lines, 57 entries, newest on top (2026-11-01), oldest at the bottom. Every recent entry uses the full 13 headings, starting with "Models, Effort, Roles and Provenance". | Partly. `docs/sessions.md` is one file (1,326 lines), newest on top by completion time, but since 2026-10-27 entries use a compact 3-part form. Newest entry ends 2026-11-05. No entries for V1.4.0, V1.5.0, the friend-sharing work (A34) or the 2026-11-07 handoff. Some 10-27 entries lack separators. |
| Usage ledger | Yes. Record `41_Usage_Ledger`, plus a live dashboard (four datasets: Claude tasks, Claude windows, Codex runs, quota readings), plus scan scripts. | **None.** No tokens, usage or quota figure anywhere in Notaya's records. Helper agents are logged "model and effort not recorded". |
| Models, effort and provenance per session | Always recorded | Recorded only in the early October entries |
| Numbered owner-decision records | 43 numbered records, e.g. 39 working policy, 40 Codex teammate arrangement, 43 AMD/Intel decision | Authority table plus A1-A36 (good, but ends 2026-11-05 11:08) |
| Execution summaries and index | 53 summaries, one index with newest first | 49 reports (EX-01 to EX-43 plus extras), index present; many statuses stale |
| Current State / Continuation State | Two separate files (01 and 10) | One `STATUS.md` (61k chars) that mixes both and carries dozens of dated historical blocks |
| Acceptance record | n/a in the numbered set | `acceptance.md` is good (outcome, applicability and decision kept apart) but has no rows for V1.2.0, V1.3.0, V1.4.0, V1.5.0 |
| Failure intelligence | n/a (not read) | `failure-intelligence.md`: FI-01 to FI-07, good quality; header still says "last update 2026-10-28" |
| Handoff notes | Dated handoff plus Claude handoff | Resume notes for V1.4.0, V1.5.0, V1.5.1 and the Codex-lead handoff (good) |

## 3. What the FFmpeg usage ledger actually does (worth copying)
Source: record 41 (read in full) and the dashboard's four dataset descriptions. The raw dataset files are stored as separate blobs and were not opened.
- Ahmed asked for it on 2026-11-01 around 03:00 Cairo: cost per task in hourly quota, tokens if possible, and what Codex burns per run.
- Ceilings: Claude 80% of a 5-hour window and 90% weekly; Codex the same (record 40).
- Method: every Claude reply counted once, price-weighted (fresh input 1, cache write 1.25, cache read 0.1, output 5); Claude work split at each commit; each review workflow is its own task. Codex runs carry exact tokens and the quota readings before and after each run.
- Findings: about 8.3% of a Claude 5-hour window and 1.15% of the week per million weighted tokens, so a week is roughly 88 million weighted tokens (about seven full 5-hour windows). For Codex, 1% of the 5-hour window buys about 15k uncached input tokens. A review-and-fix round done by Claude alone cost about 9.8% of its week; the same round with Codex doing the work cost Claude about 2% and Codex about 8% of its own week. Cached re-reads can still cost a lot (one Codex job used 39 points). Long conversations get more expensive per step.
- Refresh routine: run the scan and build scripts at each phase close, upload the four data files, put the headline figures in the dated Execution Summary.
- Caveat in its own words: both quota pools are account-wide; Claude shares are estimates, Codex shares are measured.

## 4. Stale or conflicting in Notaya
1. `STATUS.md`: title still "Cipherleaf"; "as of" 2026-11-05 11:08 UTC; says main is `2812d65` (now `7661503`); "Current state" bloated with history; the "Exact next permitted action" block is dated 2026-10-27.
2. `STATUS.md` and `CLAUDE.md` rules that now conflict with Ahmed's 2026-11-07 19:54 words: "A Claude never merges or releases by itself", "new threads run on Opus at high effort", "the owner means either partner" (Ahmed's OK alone is enough, per Ahmed).
3. `docs/handoff/claims.md`: still lists the collaboration setup as in progress and says Codex leads and Claude reviews, which Ahmed has since reversed.
4. Authority table: no rows after 2026-11-05 11:08:39 UTC. Missing: Ahmed's 19:49 setup and authority change, 19:54 "no need for Mohamed's review", 20:12 documentation and ledger instruction, PR #53.
5. `AGENTS.md` roles line ("Codex leads, Claude reviews") is wrong now.
6. `acceptance.md` and `sessions.md` gaps listed in section 2.
7. Old sections in STATUS ("Continuation", "Released and merged, not accepted") still mention drafts PR #11, #23, #24 and old `E:\Project\cipherleaf` paths.

## 5. Exactly what was read, and what was not
Fully read: `STATUS.md`, `sessions.md`, `acceptance.md`, `claims.md`, `CLAUDE.md`, `AGENTS.md`, the 2026-11-07 Codex-lead handoff, the V1.5.0 and V1.5.1 resume notes, and the PBS skill's main guides (start, choose depth, execution reports, session summaries) plus its hub pages.
Partly read: authority table (rows 1-70 and 236-285 of 285; rows 71-235 not read), execution index (first 60 lines and all 44 headings), `failure-intelligence.md` (intro, admission boundary, start of FI-07, and the seven headings; FI-01 to FI-06 bodies not read).
Not read: the 49 execution reports, the collaboration guide, the 2026-10-28 resume note, the 2026-10-23 checkpoint and project history, friend-setup, owner-invite and first-session files, plan records A9 to A35, release notes, `security-design.md` (71k), audit and design folders, and 25 of the skill's templates read only through the hub.
FFmpeg Encoder: the Notion hub page was opened (last edited 2026-11-01). The Notion "Blue Session Summary" page (provisional, 2026-10-25) was opened but only its heading structure was checked. The GitHub repo was reachable only after attaching it read-only to this session; I read the top entry and heading list of `06_SESSION_SUMMARY.md` (not the 2,756 lines), the head of the execution index, record 41 in full, and the folder listing. The `01`, `10`, `05`, `08` records and the 53 execution summaries were not read. The usage dashboard's raw data files were not opened.
Could not do: reading all ~1.8 MB of Notaya docs verbatim inside one context. A faithful full read needs several threads, each with one slice.
