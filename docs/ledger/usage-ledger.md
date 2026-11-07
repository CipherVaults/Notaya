# Notaya usage ledger: readings, newest first

How to read and refresh this file: [README.md](README.md). The numbers live in [`claude-sessions.csv`](claude-sessions.csv), [`codex-runs.csv`](codex-runs.csv) and [`quota-readings.csv`](quota-readings.csv); this page explains them.

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
