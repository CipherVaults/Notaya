# Notaya usage ledger: readings, newest first

How to read and refresh this file: [README.md](README.md). The numbers live in [`claude-sessions.csv`](claude-sessions.csv), [`codex-runs.csv`](codex-runs.csv) and [`quota-readings.csv`](quota-readings.csv); this page explains them.

---

## 2026-11-08, 08:30 UTC: the new ledger page, readings at 08:27, and a forecast for 1.6 to 2.5 with banked resets

**Who and where:** Ahmed's side (`AE0101`); written by the cloud thread "Redesign the usage ledger with forecasts" (session 48 in [sessions.md](../sessions.md), [EX-2026-11-08-60](../execution/2026-11-08-ledger-redesign-and-forecast.md)) at Ahmed's request (project chat 08:27:53 UTC: lean on Codex, and build a forecast of what the quotas can finish; 08:29:17 and 08:35:26 UTC: the forecast is for Notaya, taken from FFmpeg Encoder's only for its purpose, not its look). Machine: cloud. The readings are Ahmed's two screenshots of about 08:27 UTC, relayed by the coordinator.

### What was read at 08:27 UTC (measured)

| Limit | Reading | Resets |
|---|---|---|
| Claude, 5-hour (Pro) | 30% used | 10:30 UTC today |
| Claude, weekly | **22% used** (Ahmed's cap is 70%) | Fri 2026-11-13 02:00 UTC |
| Codex, 5-hour (Plus) | 62% used (38% left) | not shown; the window that opened at 05:25 UTC |
| Codex, weekly | **26% used** (74% left) | 2026-11-14 15:44 UTC (from the 00:44 reading; the screen shows none) |
| Codex credits | 0 | none to spend |

The Build thread also relayed two earlier Codex weekly readings: 22% at 06:24 and 26% at 08:07 UTC (not seen here). The platform's own count of Claude threads gave 374.8M tokens for the project: Map Notion workspace 46.6M (12%), Plan versions 40.9M (11%), Catch up records 39.5M (11%), Build Notaya 1.6 34.4M (9%), Write the session summary 29.9M (8%), Map what Notaya does 17.9M (5%); the other 165.6M are the remainder (the coordinator chat and about 20 smaller threads). These are in the new `project-usage-screen.csv`; the file was named in the README since 2026-11-07 but had never been committed.

### What the week readings say (measured, then derived)

- **Claude's week jumped from 6% (22:32 UTC on 11-07) to 22%** in about ten hours. About 1.5 of those hours were still on the credit, so it is not all cost of the night; it is an upper bound on one busy night with many parallel threads (Opus planning, the Notion survey, records), not a typical day.
- **Codex's week is 26% used after about 1.6 windows** (97% of window one, 15% of the week; 62% of window two, about 11%). So **one full window is about 16% of the week and a week holds about 6.1 windows** (was 6.5; the range stays 5.5 to 7).
- **Speed:** about 18 of the 1.6 version's 45 plan points are built or merged (my estimate from which lanes merged, not a measurement: lanes C1, B, A and D merged, E built and held, C4b built but failed its tests), so **about 11 points per window** (range 8 to 14; the earlier assumed value was 10.5).

### The forecast (rough: only the week capacity and the readings are measured)

The model is `tools/usage-ledger/forecast-model.cjs`, fed by `forecast-inputs.csv` and `forecast-versions.csv`; the dashboard lets the reader move every input. Default settings: 11 points per window, 6.1 windows per week, extra work 1.25, **2 banked Codex resets** (as Ahmed said; how a bank works is assumed: it refills the weekly allowance when it is empty), at most 3 windows used in a day, Claude about 3% of its week per Codex window and 4% a day for chat and records, a one-day pause after each version.

| Version | Plan points | Code done (likely) | Best to worst | Paid by |
|---|---:|---|---|---|
| 1.6 (rest) | 27 | 9 Nov | 9 Nov to 10 Nov | Ahmed's quota |
| 1.7 | 36 | 12 Nov | 10 Nov to 20 Nov | Ahmed's quota |
| 1.8 | 37 | 15 Nov | 11 Nov to 30 Nov | Ahmed's quota |
| 1.9 | 59 | 18 Nov | 13 Nov to 20 Dec | Ahmed's quota |
| 2.0 | 25 | 20 Nov | 14 Nov to 28 Dec | Ahmed's quota |
| 2.1 | 20 | 22 Nov | 15 Nov to 4 Jan | after his banked resets: weekly resets or Mohamed |
| 2.2 | 21 | 24 Nov | 16 Nov to 11 Jan | same |
| 2.3 | 18 | 29 Nov | 16 Nov to 17 Jan | same |
| 2.4 | 17 | 30 Nov | 17 Nov to 24 Jan | same |
| 2.5 | 18 | 2 Dec | 22 Nov to 31 Jan | same |

In plain words:

- **At these settings everything from 1.6 to 2.5 is built by about 2 December** (best about 22 November, worst about the end of January). That is much sooner than the 00:58 forecast (mid-December) because the two banked resets are two extra Codex weeks. Without them (preset "No banked resets"): 2.5 about 16 December. With Mohamed adding a week of Codex per week: about 26 November. A careful setting (slow, 1.5 times extra work, two windows a day): about 4 January. A fast setting: about 21 November.
- **Ahmed's own quota (this week, the 14 November reset and two banked resets) carries the work to about 21 November, which is up to version 2.0.** From 2.1 on the work runs on weekly resets only (about one version every 4 to 5 days) or on Mohamed's quota, if he continues. Whether Mohamed's quota exists, and how large, is not visible from here.
- **Codex's weekly limit is what sets the pace; Claude's does not, if the guess of about 3% of its week per window holds.** At the likely settings Claude's allowance never holds the work back; at the "careful" setting it does for about a day and a half. The total Claude directing cost for all ten versions is about 95% of one Claude week, spread over three weeks. That guess is the weakest number: nobody has yet read Claude's weekly percentage before and after a lane.
- **What the forecast does not include:** time waiting for Ahmed's words on gates (new libraries, vault format, security behaviour), the PC round beyond the one-day pause, a failed lane sent back, Notion migration, and anything Mohamed does.

### Where the usage went (from the new charts; my grouping, measured tokens)

About 48.7 million weighted tokens in the 29 session rows: reading and studies 42%, planning and writing 22%, records and ledger 17%, the coordinator chat 11%, directing the build 5%. Almost everything on both sides is cached rereading: 96% of Claude's raw tokens and 97% of Codex's first-cycle tokens. So shorter contexts, not a lower effort setting, are what would stretch a quota.

### Rows added, rule added

`quota-readings.csv`: five rows at 08:27 UTC and two relayed Codex weekly readings (06:24, 08:07). New files: `project-usage-screen.csv`, `forecast-inputs.csv`, `forecast-versions.csv`; `pace-forecast.csv` rewritten by the model (the 00:58 forecast is replaced; its text is kept above). `.gitignore` lets the three new tables through. **New reading rule (README):** at every lane merge, read Claude's weekly percentage before and after, so the 3% guess can be replaced by a measurement.

### Not established

Mohamed's quota; how a banked reset works; the share of 1.6 already built (about 18 of 45 points is my estimate); Claude's cost per Codex window (a guess); the Codex runs after 00:44 UTC (lanes A second pass, C4b and others are inside the 26% but have no rows yet); this thread's own usage and the Build thread's later usage (read at the next refresh).

---

## 2026-11-08, 00:58 UTC: the first Codex cycle, measured, and a forecast of the pace

**Who and where:** Ahmed's side (`AE0101`); written by the cloud thread "Update the usage ledger and forecast pace" (session 47 in [sessions.md](../sessions.md), [EX-2026-11-08-57](../execution/2026-11-08-ledger-first-codex-cycle-and-pace-forecast.md)) at Ahmed's request (project chat 00:51:23 UTC: *"also update the usage ledger now since this is our first cycle with codex, and try to predict how we are pacing so we can see how much we can actually finish"*). Machine: cloud. The Codex numbers were read on Ahmed's Gaming-PC by the Build thread's PC session (lane logs, `~/.codex/sessions`, Codex's own `rate_limits` events) and relayed at 00:55:04 UTC; `/status` was not run. The Claude numbers are `get_session` usage blocks read at 00:51 UTC.

### The first Codex cycle, measured

| Lane | Ran (UTC) | Ended how | Tokens printed ("tokens used": new input + output) | Session-file total (all input incl. cached / output) |
|---|---|---|---:|---:|
| A, first try | 00:25:36 to 00:27:24 | stopped early, no edits | 85,414 | 587,259 / 2,603 |
| A, second pass | 00:27:55 to 00:44:17 | cut off by the limit | 236,453 | 8,169,451 / 33,594 |
| B (export fix) | 00:25:37 to 00:44:02 | cut off | 268,752 | 11,160,807 / 36,073 |
| C1 (action core) | 00:25:37 to 00:43:46 | cut off | 207,645 | 8,590,969 / 41,380 |
| D (bridge program) | 00:25:38 to 00:44:22 | cut off | 202,071 | 4,787,279 / 43,912 |
| E (AI screens) | 00:25:37 to 00:40:59 | **finished** (draft PR #70, no security finding) | 159,263 | 4,598,265 / 32,806 |
| **Cycle** | 00:25:36 to 00:44:22 | | **1,159,598** | **about 38.1 million / 190,000** |

- **Correction to the relay of 00:51:** the lanes started at about 00:25:37 and the limit hit at 00:44:22, so five lanes ran about **19 minutes**, not 23, which is about **90 lane-minutes** of running.
- **Model and effort:** `gpt-6.1-sol`, `xhigh`, all runs (quoted from the process command lines, the Codex config and each log header at 00:29 UTC). Reasoning output was small: 11,000 to 17,000 tokens per lane.
- **Almost everything is cached reading:** about 97% of the 38 million tokens were cached input, that is, the lanes re-reading their growing context at every turn. Only about 1.16 million tokens were new input plus output. This is the same pattern as Claude's (cache reads were most of the volume all evening).

### The two Codex limits, read from Codex itself

| Time (UTC) | 5-hour window used | Weekly used | Note |
|---|---:|---:|---|
| 00:26:05 | 0.0% | 0.0% | before any lane; the first Codex reading ever taken for Notaya |
| 00:40:59 | 86.0% | 13.0% | lane E finished |
| 00:44:22 | **97.0%** | **15.0%** | last reading before the stop; the limit was then reached |

The 5-hour window (300 minutes) resets at **05:25:42 UTC**. The weekly window (10,080 minutes) resets at **2026-11-14 15:44:18 UTC**. The plan type is `plus` and the credits balance is 0. Codex's message offered an upgrade to Pro or buying more credits; neither is done, and whether either is worth it is Ahmed's decision (no price is in these records).

**The finding that matters most: the weekly limit is about 6.5 times a 5-hour window.** Using 97% of the 5-hour window used 15% of the weekly one, so one full window is about 15.5% of a week and the week holds about **6.5 windows** (5.5 if the last 11 points are the better guide, 7 if the first 86). Everything below rests on this.

### Claude side, measured at 00:51 UTC

| Thread | Reported cost (USD) | Weighted tokens | Note |
|---|---:|---:|---|
| Coordinator (Opus, low) | 17.04 (was 3.27 at 20:33) | 5.50 million (was 1.08) | about USD 3.2 an hour while chatting |
| Write the session summary checkpoint (Sonnet, high) | 3.41 | 2.27 million | finished 00:34 |
| Build Notaya 1.6 (Sonnet, high) | 2.77 (was 1.75 at 00:26) | 1.86 million | idle, waiting for Codex |

Claude's 5-hour status is "allowed", no overage, reset 05:00 UTC. **Claude's weekly percentage is still not read**, so the 70% line cannot be checked.

### The pace forecast

A forecast with stated assumptions, not a measurement; the figures are in [`pace-forecast.csv`](pace-forecast.csv).

1. **Capacity (measured):** one 5-hour window is about 90 lane-minutes and about 15.5% of the Codex week; a week is about 6.5 windows.
2. **Output of window 1 (assumed):** the spec sizes 1.6 at about 45 points (small = 1, medium = 3, large = 6, extra large = 12). Lane E is done; the four cut-off lanes are assumed 25 to 75% done. That puts window 1 at **7 to 14 points (15 to 30% of 1.6)**, likely 10.5. This is the weakest number; it will be replaced once the cut-off lanes are inspected.
3. **Overhead (assumed):** fixes after Claude's review, integration and the PC round add 0 to 50% (likely 25%).
4. **Windows needed** = points × overhead ÷ points per window. **Weeks** = windows ÷ windows per week.

| Scope | Points | Windows: low | likely | high | Weeks of Codex budget: low | likely | high |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1.6, what is left | 31 to 38 | 2.2 | **4.1** | 8.1 | 0.3 | 0.6 | 1.5 |
| 1.7 | 36 | 2.6 | 4.3 | 7.7 | 0.4 | 0.7 | 1.4 |
| 1.8 | 37 | 2.6 | 4.4 | 7.9 | 0.4 | 0.7 | 1.4 |
| 1.9 (with the survey's 13 extra points) | 59 | 4.2 | 7.0 | 12.6 | 0.6 | 1.1 | 2.3 |
| 2.0 | 25 | 1.8 | 3.0 | 5.4 | 0.3 | 0.5 | 1.0 |
| 2.1 | 20 | 1.4 | 2.4 | 4.3 | 0.2 | 0.4 | 0.8 |
| 2.2 | 21 | 1.5 | 2.5 | 4.5 | 0.2 | 0.4 | 0.8 |
| 2.3 | 18 | 1.3 | 2.1 | 3.9 | 0.2 | 0.3 | 0.7 |
| 2.4 | 17 | 1.2 | 2.0 | 3.6 | 0.2 | 0.3 | 0.7 |
| 2.5 | 18 | 1.3 | 2.1 | 3.9 | 0.2 | 0.3 | 0.7 |
| **1.7 to 2.5 together** | 251 | **18** | **30** | **54** | **2.6** | **4.6** | **9.8** |

**What can actually be finished, in plain words**

- **This week (until the weekly reset on 2026-11-14 15:44 UTC):** about **5.5 windows of Codex are left** (85% of the week). The likely 4.1 windows for the rest of 1.6 fit, with about 1.4 windows (a third of 1.7) to spare. In the bad case (8 windows) 1.6 spills past the reset. Running flat out, the likely finish for the 1.6 code is **the early hours of 2026-11-09 to 2026-11-10 UTC** (windows open every 5 hours: 05:25, 10:25, 15:25, 20:25, 01:25 ...), then the review, the security review and the PC round.
- **1.7 to 2.5:** about **4.6 weeks of Codex weekly budget (range 2.6 to 9.8)**, so about **mid-December 2026 (range late November to mid-January)** if Codex stays the main builder at the full 100% Ahmed allowed. Roughly **one version every 4 to 5 days**; the big ones (1.9 at about a week, 2.0 to 2.5 at 2 to 4 days each) differ.
- **The weekly limit, not the 5-hour one, now sets the schedule.** The 5-hour resets only decide how work is cut up. Spending one window a day would use the week evenly (6.5 windows over 6.6 days); spending them back to back finishes sooner but then waits for the weekly reset.
- **The Codex budget is not the only one.** Claude's side (planning, reviewing every Codex diff, records, some direct building) has its own weekly limit, capped by Ahmed at 70%, and it has not been read. A rough sketch from the 22:32 reading (6% weekly after about USD 52 of credit spending, if that counted against the plan): 100% of a week is about USD 870 at list prices, the 70% rule about USD 610, and a version costs Claude roughly USD 40 to 120 (a spec, briefs, about eight diff reviews, records), that is 7 to 20% of a week's 70% budget per version, or 0.6 to 1.8 weeks of it for all nine versions from 1.7 to 2.5, against about 4.6 weeks of Codex. On this sketch Claude is not the limit; Codex is. If the credit spending did not count against the plan, the Claude capacity is larger and unknown. A reading now and one at the 05:00 UTC reset would turn this into a rate.

### Should fewer lanes run in parallel?

- **Fewer lanes will not make a window buy more work.** Both limits count usage, not time: five lanes for 19 minutes and two lanes for about 47 minutes should use about the same. It only spreads the same budget over more hours.
- **It still helps, for four reasons.** (1) Cached rereading is 97% of the volume, and every lane starts by reading `CLAUDE.md`, the records and its spec sections; five lanes read them five times. (2) A cut-off lane loses its place and re-reads on resume; with two lanes the limit is less likely to land in the middle of one. (3) The lanes are not all independent (C2 and C4 wait for C1, C3 for C2, C5 for C2 and C3), so after the first three only one or two can run anyway. (4) A lane that finishes early can be inspected and corrected while the window is still open. The heartbeat's **two lanes at a time** is a good setting.
- **Order (already given to the PC session at 00:46 UTC):** C1, then A and B, then D, then C2 and C4.
- **Levers to buy more per week, for Ahmed's decision:** (a) shorter contexts: point each lane at its spec sections and tell it not to re-read the whole records, since cached rereading is the cost, not thinking (so lowering `xhigh` to `high` would likely save little: reasoning output was only 11,000 to 17,000 tokens per lane); (b) have each lane commit after every finished part so a cut-off loses nothing; (c) let Claude build the small lanes directly (A, B), which moves load to the Claude week, subject to the 70% cap; (d) a Codex plan upgrade or extra credits, which Codex itself offered (price not known here).

### Not established

The Codex forecast above rests on the assumed 25 to 75% progress of the four cut-off lanes and on 1.7 onward costing about as much per point as 1.6 (which is the heaviest version on purpose); per-run cost in money (not available); whether Codex counts cached input in its limits (the numbers do not say); Claude's weekly percentage; the Notion-board thread's usage and this thread's own (still running); the PC-side Remote Control session's Claude usage; whether the 22:32 weekly reading included the credit-funded work.

---

## 2026-11-08, 00:26 UTC: the last hour of the credit, eight threads measured, and the new quota rules

**Who and where:** Ahmed's side (AE0101); written by the cloud thread "Write the session summary checkpoint" (session 46 in [sessions.md](../sessions.md)) from the `get_session` usage blocks of eight threads, read at 00:26 UTC. Nothing here came from a screen. Seven of the eight threads had finished; "Build Notaya 1.6" was still running, so its row will grow. The coordinator's own usage after 20:33 UTC and this thread's usage are not read.

| Thread (model, effort) | Reported cost (USD) | Output tokens | Weighted tokens |
|---|---:|---:|---:|
| Map Notion workspace (Sonnet 5.5, high) | 15.74 | 317,180 | 9.42 million |
| Plan versions 1.6 and on (Opus 5.5, high) | 15.84 | 173,194 | 6.06 million |
| Write the 1.6 spec (Opus 5.5, high) | 5.05 | 73,489 | 1.55 million |
| Write the manual (Sonnet 5.5, high) | 3.04 | 61,898 | 1.92 million |
| Draft the AI action list (Opus 5.5, high) | 2.76 | 36,422 | 0.80 million |
| Libraries and licences (Sonnet 5.5, high) | 2.30 | 66,006 | 1.47 million |
| Build Notaya 1.6 (Sonnet 5.5, high; running) | 1.75 | 27,225 | 1.06 million |
| Design the whiteboard (Sonnet 5.5, high; started 23:16) | 0.71 | 22,855 | 0.41 million |
| **Total** | **47.19** | **778,269** | **22.69 million** |

**What this says, in plain words**

- **These eight threads cost about USD 47 at list prices in roughly the last hour and a half.** The two biggest are the Notion survey (it read a workspace of about 3,000 pages and 66 databases, then all the formulas through a browser) and the version plan (Opus, many rounds of changes and Notion board updates). Together they are two thirds of the total.
- **Reading is still the main cost:** about 126 million of the tokens were cache reads (threads re-reading big contexts). Output is only 0.78 million tokens.
- **Opus costs about twice Sonnet per weighted token again:** the 1.6 spec (Opus) was USD 3.25 per million weighted tokens; the library study (Sonnet) USD 1.57. That matches the 22:32 reading and the model-choice guide.
- **The USD 100 credit ran out around 23:59 UTC** (Ahmed's statement, 23:59:56). The sum of the reported costs of all threads is not the same as the credit, because the coordinator's chat and the PC threads are not all in this table; no balance was read.

**The new rules (Ahmed, 2026-11-07 23:59:56 UTC, [A45](../plan/approval-and-amendments.md)):** the cloud credit is used up, so work runs on his Claude plan with its 5-hour and weekly limits. **Claude may use up to 70% of the weekly quota** until he changes it. **Codex may use up to 100%, including all its resets.** So Codex carries the heavy building; Claude plans, briefs, inspects every Codex diff, keeps the records and builds directly only when that fits better. Pacing around the 5-hour resets means fewer Claude threads at once, Sonnet instead of Opus where it does the job, and short reading lists. The session records show the 5-hour window allowed with no overage; the next reset is 05:00 UTC.

**Rows added:** eight rows in `claude-sessions.csv` (the plan and manual rows replace the earlier "not measured" ones) and three rows in `quota-readings.csv` (the 5-hour reading and the two rules).

### Not established

The weekly percentage used so far (no reading yet, so the 70% line cannot be checked); Codex usage (no run recorded yet, the 1.6 lanes have just started); the coordinator's and this thread's usage; whether the credit also paid for the PC threads; the four Sonnet helper sub-agents of the planning thread.

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
