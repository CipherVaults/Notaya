# Notaya usage ledger: readings, newest first

How to read and refresh this file: [README.md](README.md). The numbers live in [`claude-sessions.csv`](claude-sessions.csv), [`codex-runs.csv`](codex-runs.csv) and [`quota-readings.csv`](quota-readings.csv); this page explains them.

---

## 2026-11-07, 23:40 UTC: statements from the chat after the 22:32 screenshots (no new measurement)

**Who and where:** Ahmed's side (AE0101); written by the cloud thread "Record Phase 1 decisions and update handover" (PR #61) from the project chat. Nothing here was read from a screen; the measured figures are the 22:32 UTC block below.

| What | Statement | By whom, when | Status |
|---|---|---|---|
| Credit left | about USD 41, about 19.5 hours to expiry (2026-11-08 19:09 UTC) | the coordinator, 23:36:52 | A statement, source not given. The last measured balance is USD 48 left at 22:32 |
| After the credit | cloud work counts against the Pro five-hour and weekly limits; nothing stops | the coordinator, 22:39:11, from Ahmed's screenshot of the credit help text | A reading of the screen, not seen by this thread |
| Pace | at the evening's pace (about USD 15 an hour for the first hours) the credit would not outlast about 3 more hours of Opus-heavy work; Ahmed wants the credit used before it expires | the coordinator, 22:36:10; Ahmed, 23:37:36 | Estimate and preference |
| Model choice | a guide (`notes/model-choice-guide.md`, project files) suggests Sonnet 5.5 high for reading and records, Opus 5.5 high for design, Codex `gpt-6.1-sol` xhigh for building | the model-economics thread, about 23:00 | A reading, not yet accepted by Ahmed (A43) |

**Rows added:** one `claude-sessions.csv` row for this thread (usage not measured) and one `quota-readings.csv` row for the 23:36 statement. **Not done:** the coordinator's and the study threads' usage after 22:32 is not read; Codex still has no run. The next ledger refresh should read the Phase 2 threads' cost-bearing records and a five-hour and weekly reading.

---

## 2026-11-07, measured limits and credit from Ahmed's screens (22:32 UTC)

**Who and where:** Ahmed's side (AE0101). The figures are read off three screenshots Ahmed sent at 22:32 UTC (the claude.ai plan-usage popup and the project's Usage screen, on his PC); the cloud thread "Publish the Notaya usage ledger" wrote them in. Source and time are in every row of [`quota-readings.csv`](quota-readings.csv) and [`project-usage-screen.csv`](project-usage-screen.csv).

### Measured (screenshots, 22:32 UTC)

| What | Reading | Resets or expires |
|---|---|---|
| Claude five-hour limit (plan: Pro) | **46% used** | 2026-11-08 00:00 UTC (screen: 2:00 AM GMT+2) |
| Claude weekly limit, all models | **6% used** | Friday 2026-11-13 02:00 UTC (screen: Fri 4:00 AM GMT+2) |
| Cloud credit (USD 100) | **USD 48 left, so USD 52 used** | 2026-11-08 **19:09 UTC** (screen: 9:09 PM GMT+2, Nov 8); replaces the assumed 19:00 |
| Project usage (all 15 threads) | **157.7M tokens**, cache hit 97%; Opus 4 threads, Sonnet 11 | not applicable |

Project tokens by kind: Opus: input 17.8k, output 455.8k, cache write 1.8M, cache read 104.9M. Sonnet: input 28.9k, output 268.6k, cache write 2.5M, cache read 47.7M. Weighted with the ledger's formula that is about 24.3 million (Opus 15.0M, Sonnet 9.3M); the CSV rows read earlier in the evening added up to 19.98 million.

### The gap between USD 52 and USD 31.93

The credit screen says **USD 52 used**. The ledger rows that carry a dollar figure add up to **USD 31.93**. So **USD 20.07 is not itemised by thread**; the screen gives no per-thread dollars and the ledger does not guess which thread it belongs to. What the records do show:

- **Ten sessions report a cost** (the cloud sessions read through `get_session`), and for those the ledger tokens match the project's Usage screen to the first digits (for example "Map everything Notaya already does": 17,932,052 tokens in the ledger, 17.9M on screen; "Learn the PBS hub": 17,104,094 and 17.1M). Those ten rows are therefore complete up to their snapshot time.
- **Six rows have no dollar figure** (five threads, one of them in two rows): the Gaming-PC Remote Control threads and the cloud threads bound to Gaming-PC whose record gave no cost. They hold tokens, or none yet for this thread. The earlier token-based estimate for them was USD 20 to 29. The gap of USD 20.07 sits at the low end of that range, which fits, but it does **not prove** that the PC threads draw from the credit: no record says so.
- **Three threads kept working after the snapshot the ledger holds** (tokens on screen against the ledger's last row): "Catch up Notaya records and start the ledger" 39.5M against 30.9M, "Write today's session summary" 29.9M against 11.5M, "Set up Notaya review and coordination" 4.3M against 3.9M. This thread (5.6M, still running) and the coordinator (15.9M against 15.5M) also grew.
- The reported dollar figures are at API list prices; the credit may be charged differently (for example one-hour cache writes cost more than the ledger's 1.25 weight).

The gap is therefore most likely the threads with no dollar figure plus work done after the last reading. The ledger cannot split it further until each thread's cost is read from a cost-bearing record.

### What it means for pace

- **Credit:** USD 48 left for about 20.6 hours, which is about USD 2.30 an hour if it were spread evenly. Tonight so far ran at roughly USD 52 in 3.4 hours (about USD 15 an hour), mostly the three Opus threads.
- **Five-hour window:** 46% used with the window ending at 00:00 UTC.
- **Weekly:** 6% used. Not close to a limit.
- **Codex:** no run, no reading.

---

## 2026-11-07, fourth reading and the dashboard (snapshot 22:25 UTC)

**Who and where:** Ahmed's side (AE0101); cloud thread "Publish the Notaya usage ledger" (branch `claude/project-thread-h4cb12`), reading the session records of the cloud sessions at 22:25 UTC.

**Scope:** (a) the coordinator's cumulative reading, which replaces its 20:33 row; (b) three cloud threads that no earlier reading covered: the import and export study, the feature inventory, the PR #56 merge. The handover-guide thread and this thread were running and are not counted.

| Session | Model (last served) | Effort | Output tokens | Weighted tokens | Reported cost (USD) |
|---|---|---|---:|---:|---:|
| Coordinator (mother thread), cumulative to 22:25, running | Opus 5.5 | low | 131,499 | 2,546,534 | 8.06 |
| Study Notaya's current import and export | Sonnet 5.5 | high | 20,862 | 815,399 | 1.46 |
| Map everything Notaya already does | Opus 5.5 | high | 44,012 | 2,383,156 | 6.98 |
| Merge tonight's session summary PR (PR #56) | Sonnet 5.5 | high | 1,333 | 122,723 | 0.29 |

- **Totals after this reading** (newest row of each session, 15 rows): 19.98 million weighted tokens; 602,997 output tokens; reported cost **USD 31.93** across the 10 cloud sessions that report one. The five Gaming-PC rows carry tokens but no dollar figure.
- **Against the USD 100 credit:** reported spend is USD 31.93. For the PC rows (6.0 million weighted Opus tokens, about 0.6 million Sonnet) a **rough estimate** of USD 20 to 29 follows from the dollars-per-million-weighted-tokens range of reported rows (Opus 2.93 to 4.18, Sonnet 1.76 to 2.35). It is an estimate, not a record, and whether PC sessions draw from the cloud credit is not shown.
- **Quota at 22:25 UTC:** Claude five-hour window "allowed", not using overage, resets 2026-11-08 00:00 UTC. Weekly limit, Codex limits and the credit balance were not read.
- **This thread** (22:24 to about 22:35 UTC): a row is in the CSV with empty numbers, because a running session's own record has no usage block. Its numbers are to be read later from its session id and added as a newer row.
- **Codex:** no run.
- **New:** the dashboard page ("Notaya usage ledger", an Artifact) is built from the three CSV files by `tools/usage-ledger/build_dashboard.py`; the CSV files stay the owner of the facts.

---

## 2026-11-07, the rest of the coordinator session and the two PC threads (snapshots 22:02 to 22:06 UTC)

**Scope:** (a) the two Remote Control sessions on Gaming-PC that the first reading could not measure, now measured from their transcripts on the PC; (b) the records thread that wrote the session summary of the whole coordinator session (PR #56), up to 22:06 UTC.

| Session | Model (last served) | Effort | Output tokens | Weighted tokens | Reported cost (USD) |
|---|---|---|---:|---:|---:|
| Set up Notaya review and coordination (PC, 19:10 to 19:26) | Opus 5.5 and Sonnet 5.5 | not recorded | 10,884 | 810,870 | not reported |
| Plan and build Notaya v1.6, stopped (PC, 19:55 to 19:56) | Sonnet 5.5 | not recorded | 3,492 | 251,473 | not reported |
| Session summary, PR #56 (PC, from 21:56), running | Opus 5.5 | not recorded | 53,340 | 1,793,924 | not reported |

Raw, the two earlier PC threads together: fresh input 72; cache write 416,922; cache read 4,692,381; output 14,376 (33 replies). This thread to 22:06: fresh input 92; cache write 331,342; cache read 11,129,544; output 53,340 (44 replies).

- **The first evening reading is now complete for the PC side:** the two PC threads add 1.06 million weighted tokens to the 10.67 million of the eight cloud sessions measured at 20:21 to 20:33.
- **Pace:** as before, cache reads are most of the volume. This thread re-read a very large context (the whole project chat, two study files, the session file of about 2,300 lines and part of the 1.8 MB Blue reference).
- **Not measured from here:** the coordinator after 20:33 UTC, the two cloud threads "Study Notaya's current import and export" (20:56 to 21:00) and "Map everything Notaya already does" (21:00 to 21:10), and the end of this thread after 22:06. Their session records are cloud records, which this PC thread cannot read; the coordinator can add them at its next ledger refresh.
- **Quota:** no reading was taken in this block. Ahmed's rule of 20:41:50 UTC (no cap; watch the 5-hour and weekly limits of Claude and Codex) answers STATUS question 28; the next refresh should add a reading of both limits.
- **Codex:** no run.

---

## 2026-11-07, second records PR (snapshot 20:47 UTC)

**Scope:** the second work block of the records thread (20:38 to 20:47 UTC, PR #55), counted from 20:38 so that it does not overlap the first reading below.

| Session | Model (last served) | Effort | Output tokens | Weighted tokens | Reported cost (USD) |
|---|---|---|---:|---:|---:|
| Records thread, second block (PR #55) | Opus 5.5 | high | 45,319 | 1,660,771 | not reported |

Raw: fresh input 119; cache write 132,006; cache read 12,690,495; output 45,319 (63 replies). The checks, merge and report after 20:47 are not counted.

- **Pace:** once more, cache reads are most of the volume (about 12.7 of 12.9 million raw tokens). This block re-read large records (the session file is about 2,300 lines) while writing three entries and reordering one day.
- **Quota at 20:45 UTC:** five-hour window "allowed", not using overage, resets 2026-11-08 00:00 UTC.
- **Codex:** no run.

---

## 2026-11-07, Ahmed's first evening (snapshot 20:21 to 20:33 UTC)

**Scope:** every Claude session of the Claude project "Notaya" on 2026-11-07 from 19:08 UTC: the coordinator and nine threads (the setup and v1.6 threads ran as Remote Control sessions on Ahmed's PC). Earlier the same day a session on Ahmed's PC, started from Mohamed's account, wrote hand-off PR #53; it is outside this ledger (another account, not measured). This is the ledger's first reading; nothing before it was measured.

### Claude, measured

| Session | Model (last served) | Effort | Output tokens | Weighted tokens | Reported cost (USD) |
|---|---|---|---:|---:|---:|
| Coordinator (mother thread), running | Opus 5.5 | low | 43,846 | 1,075,329 | 3.27 |
| Explore what Notaya does | Opus 5.5 (record also names Sonnet 5.5) | medium | 3,406 | 194,256 | 0.81 |
| Learn the PBS hub and FFmpeg lessons | Sonnet 5.5 | high | 77,040 | 2,968,649 | 5.43 |
| Build the Notaya Notion hub | Sonnet 5.5 | high | 35,137 | 1,079,101 | 1.90 |
| Read execution reports in full | Sonnet 5.5 | high | 57,899 | 1,230,142 | 2.46 |
| Read plans, handoffs and releases in full | Sonnet 5.5 | high | 19,501 | 835,748 | 1.74 |
| Read security design and session summary | Opus 5.5 | high | 21,729 | 727,333 | 2.79 |
| Records catch-up and this ledger (PR #54), running | Opus 5.5 | high | 77,544 | 2,562,942 | not yet reported |
| **Total of the eight measured sessions** | | | **336,102** | **10,673,499** | **18.41** (seven sessions) |

By model: Sonnet 5.5, four sessions, 6.11 million weighted tokens, USD 11.53; Opus 5.5, four sessions, 4.56 million weighted tokens, USD 6.88 for the three that report a cost. Raw totals: fresh input 24,124; cache write 2,339,107; cache read 60,449,816; output 336,102.

### Claude, not measured

- "Set up Notaya review and coordination" and "Plan and build Notaya v1.6" ran as Remote Control sessions on Gaming-PC; their records are not readable from the cloud. To fill them, run the scanner on their transcripts on the PC (README, step 2).

### Codex

No Codex run was ordered on 2026-11-07 after 19:08 UTC. `codex-runs.csv` has its header only.

### Quota

- Claude five-hour window at 20:33 UTC: "allowed", not using overage, resets 2026-11-08 00:00 UTC (session records; no percentage is given there).
- The USD 100 cloud credit Ahmed mentioned (19:49:26 UTC, "less than 24 hours"): balance and exact expiry not read. The reported costs above (USD 18.41 so far, plus this thread's own) are at API list prices; whether they are what the credit is charged is **not** shown by the records.

### What this says about pace

- **Reading is the biggest cost, and it is mostly cache reads.** 60 of the 63 million raw tokens were cache reads: threads re-reading large records many times. In weighted terms the two long readers (the PBS hub and the records catch-up) are about half of the evening.
- **Sonnet carried most of the reading at about two thirds of the cost.** The one Opus reader (security design and session file) cost USD 2.79 for 0.73 million weighted tokens; the Sonnet readers cost between USD 1.74 and 2.46 for 0.84 to 1.23 million. Per weighted token, Opus is about twice the price of Sonnet here, as expected from list prices.
- **A first yardstick for the credit:** about USD 18 for one evening of reading, planning and records across eight cloud sessions. A feature build with Codex doing the coding should move most of the token volume off Claude.
- These are first readings; a pace needs several, and pacing limits are not set yet (STATUS question 28).

### Not established

The share of the five-hour and weekly limits these sessions used (no percentage reading was taken); Codex quota; Mohamed's usage; whether any of this was charged to the cloud credit.
