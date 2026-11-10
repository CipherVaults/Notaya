# Notaya model-choice guide: which model and effort for which job

**Who and where:** Ahmed's side (AE0101), cloud thread "Learn the model economics guide", written 2026-11-07 about 23:00 UTC. Result state: **produced** (not yet accepted by Ahmed).

**Sources:** (1) Ahmed's "FFmpeg Encoder Model Economics" artifact, https://claude.ai/artifact/1eLfHxqU4uyhKkdn3ZZjoL (benchmark by Artificial Analysis, fetched 30 Oct 2026; usage from the FFmpeg project). (2) Notaya's own ledger, `docs/ledger/` on `main` (claude-sessions.csv, quota-readings.csv, usage-ledger.md), readings of 2026-11-07 up to 22:32 UTC. Where a number is my own calculation it says "estimate".

**Caution:** the benchmark measures general tasks, once per setting, with no error bars. It tells us the *relative* price of thinking harder. It does not say how good a model is at Notaya's code. Where this guide says a setting "pays" or "wastes", that is judgement applied to Notaya, as in the artifact.

---

## 1. The short answer

| Job | Model and effort | Why in one line |
|---|---|---|
| Coordination (the mother thread: plans, steers, merges) | **Opus 5.5, high** (medium for pure relaying). It ran at *low* tonight. | Low scores 35 on the index, high 52. Decisions and diff inspection are what the coordinator is for. |
| Reading records in full | **Sonnet 5.5, high** | Reading is not limited by reasoning depth, and Sonnet costs about half of Opus per token. |
| Reading that drives security decisions | Opus 5.5, high | The one place a missed sentence is expensive. |
| Discussion and design studies | **Opus 5.5, high**; xhigh only for the AI CLI security model | Ahmed's feature talks set direction, so quality beats price. |
| Studying what the code or exports already do | Sonnet 5.5, high | Same facts at about a fifth of the Opus cost (see section 3). |
| Docs, PBS records, ledger rows, merges | **Sonnet 5.5, high** | Mostly writing down known facts. Opus high for a session summary rebuilt from a long chat. |
| Hash, inventory, CSV and readback checks | A script (`tools/usage-ledger/`), else Haiku 5.5 high | Costs nothing or almost nothing. |
| Building code | **Codex, GPT-6.1 Sol, xhigh** | Scores 52 (equal to Opus high) at about a fifth of the price, on a separate quota. |
| Diff review | **One Opus 5.5 xhigh reviewer** per substantive diff | A review by the author is not independent; Claude reviews what Codex wrote. |
| Never | Opus max, Sonnet xhigh and max, Fable 5.1, Haiku max, review panels | Each is beaten by a cheaper setting (section 4). |

---

## 2. What the benchmark says (from the artifact)

Index = Artificial Analysis Intelligence Index v5.0, higher is better. Cost and time are per benchmark task. Use cost only as a **relative** yardstick.

| Model, effort | Index | Cost per task | Time per task |
|---|---:|---:|---:|
| Opus 5.5 low | 35 | $0.98 | 11 s |
| Opus 5.5 medium | 48 | $2.45 | 19 s |
| Opus 5.5 high | 52 | $3.46 | 37 s |
| **Opus 5.5 xhigh** | **57** | **$6.05** | **2.1 min** |
| Opus 5.5 max | 58 | $10.04 | 10.8 min |
| Sonnet 5.5 high | 41 | $1.64 | 15 s |
| Sonnet 5.5 xhigh | 50 | $4.09 | 34 s |
| Haiku 5.5 high | 30 | $0.16 | 18 s |
| GPT-6.1 Sol high | 50 | $0.59 | 42 s |
| **GPT-6.1 Sol xhigh** | **52** | **$0.76** | **1.9 min** |
| GPT-6.1 Sol max | 55 | $1.27 | 4.7 min |

What to take from it:

- **Effort steps are not equal.** Opus low to medium gives +13 points, medium to high +4, high to xhigh +5, xhigh to max only +1 (for 66% more cost and 5 times the wait). So: high is the sensible floor for judgement work, xhigh is for work where a wrong answer costs a rework loop, max is never worth it.
- **Opus high beats Sonnet xhigh** (52 for $3.46 against 50 for $4.09). If Sonnet needs more than high, switch model, not effort.
- **Sonnet high is the cheap reader** (41 for $1.64), but it is about 11 points below Opus high, so it should not sign off anything.
- **Codex Sol xhigh equals Opus high on the index at $0.76**, and it draws on a separate ChatGPT quota, so building there leaves the Claude quota for design and review. (Codex is not available inside Claude Code; the coordinator calls it.)
- Fable 5.1 loses to a cheaper Opus setting at every effort in the artifact's table.

---

## 3. What Notaya actually spent (ledger, 2026-11-07)

**Totals at 22:32 UTC:** USD 52 of the USD 100 cloud credit used, USD 48 left, expiring 2026-11-08 19:09 UTC. The project's Usage screen shows 157.7M tokens over 15 threads (Opus 4, Sonnet 11), cache hit 97%, outcome 3 PRs and 2 artifacts. Five-hour limit 46% used, weekly limit 6% used (both Pro, account-wide).

**What the money went on (estimate).** I fitted the dollar figures the ledger records for 11 cloud sessions to the token kinds. The fit matches each row within a few cents, so the implied list prices are trustworthy:

| Token kind | Opus 5.5 | Sonnet 5.5 |
|---|---:|---:|
| Cache read (per million) | about $0.20 | about $0.10 |
| Cache write (per million) | about $8.0 | about $4.0 |
| Output, which includes thinking (per million) | about $20.5 | about $9.8 |

Applied to the project totals, that is about USD 62 at list prices (cache reads about 26, cache writes about 24, output about 12; Opus about 45, Sonnet about 18). The credit screen says 52, so the credit is not charged exactly at list or not every thread draws on it. The PC threads may be the difference, which the records do not prove.

What this means in plain words:

1. **97% of the tokens are cache reads, but they are only about 40% of the money.** Cache writes (a thread loading new material for the first time) are another 39%. Output and thinking, the part that effort changes, is only about 19%.
2. **So effort is a smaller lever than context.** A thread that reads a lot of files costs what it costs whatever its effort. The big savings are: fewer fresh threads that each reload the same records, short digests instead of raw records, and Sonnet for reading.
3. **Opus costs about twice Sonnet per token** for the same work. Tonight's same-shaped jobs show it.

**Cost per kind of task, as recorded (list prices):**

| Kind of task | Thread | Model, effort | Cost | Time |
|---|---|---|---:|---|
| Coordination | Coordinator, 19:08 to 22:25 | Opus, low | $8.06 (about $2.4 an hour) | 3 h 17 min |
| Read records in full | Execution reports (96 KB) | Sonnet, high | $2.46 | 7 min |
| Read records in full | Plans and handoffs (22 KB) | Sonnet, high | $1.74 | 3 min |
| Read records, security | Security design and summary | Opus, high | $2.79 | 4 min |
| Read records, very long | PBS hub and FFmpeg lessons | Sonnet, high | $5.43 | 27 min |
| Study the current code | Import and export study | Sonnet, high | $1.46 | 3 min |
| Study the current code | Map everything Notaya does | Opus, high | $6.98 | 10 min |
| Docs, a small merge | Merge PR #56 | Sonnet, high | $0.29 | 30 s |
| Docs and PBS records | Records PR #54 and #55 | Opus, high | about $12 for both (estimate from the fitted prices; no cost figure recorded) | 14 min and 8 min |
| Codex build | none yet | n/a | `codex-runs.csv` is empty | n/a |
| Diff review | none yet | n/a | see estimate below | n/a |

The Opus map and the Sonnet import study are not the same size of job (the map covered the whole app), so do not read it as a clean 4.8 times. Still, both produced a complete written inventory and the Sonnet one cost a fifth.

**Estimates for jobs we have not run yet** (from the fitted prices, assuming a review thread loads about 0.25M tokens fresh, reads about 4M from cache and writes about 30k tokens): one Opus xhigh diff review is **about $3.5 to $6**. A typical Codex slice costs Claude only the coordinator's briefing and inspection, because the coding runs on the ChatGPT quota. The first real Codex runs must be logged in `codex-runs.csv` so this line can become a measurement.

---

## 4. The rules

**Coordination.** Opus 5.5 at **high**, not low. Drop to medium only for stretches of relaying and checklists. Go to xhigh while inspecting a Codex diff that touches crypto, storage, IPC, the CSP, clipboard, auto-lock or the AI CLI, and while designing the AI CLI safety rules. The extra output at high is small money (output was about 33% of the coordinator's $8.06 at low). The coordinator is also the biggest repeat reader (15M cache-read tokens), so keep its context lean: it reads digests from `/mnt/project-files/notes/`, not raw records.

**Reading.** One reader thread per record set, **Sonnet 5.5 high**, writes a digest into `notes/`. Everyone else reads the digest. Opus high only for the security design.

**Discussion and design.** Opus high. Do the Ctrl+F style "try, then tune" talks at high; the AI CLI security model and anything that changes a security promise at xhigh.

**Studies of what exists.** Sonnet high, with the full inventory in `notes/` as the starting point so nobody re-reads the code.

**Docs and PBS records.** Sonnet high writes from known facts (merge notes, ledger rows, execution summaries). Opus high for a session summary that must be reconstructed from a long chat and for any record where a security promise changes (`docs/security-design.md`). A script does anything mechanical: ledger rebuilds, hashes, inventories.

**Building.** Codex GPT-6.1 Sol **xhigh** for the bulk of the code, in well-specified slices. High is enough for small, clear edits, but the price gap is small, so default to xhigh. Not max (index 55 for 1.7 times the cost and 2.5 times the wait).

**Review.** One Opus xhigh reviewer per substantive diff, a different agent from the author. Opus high for UI, CSS and docs-only diffs. The security review of each code PR (guide section 9) is always xhigh. No review panel unless Ahmed asks. The artifact's own history is the warning: a 70-agent review hit the limit after 13 agents, and each Opus subagent cost 137k to 266k tokens.

**Never use:** Opus max; Sonnet xhigh or max; Fable 5.1 at any effort; Haiku max; more than five agents in one workflow; a workflow by default.

**Before launching anything big:** say the expected cost, and do not start a new large thread when the five-hour window is past about 80%.

---

## 5. Budget and the next 24 hours

- **Credit:** USD 48 left until 2026-11-08 19:09 UTC, about 20 hours. Tonight ran at roughly USD 15 an hour for the first 3.4 hours (three busy Opus threads). Spread evenly the credit would last at about USD 2.30 an hour, so it is a budget to use deliberately, not a limit to rush: put the Sonnet reading and record threads first, keep Opus for design, discussion and review.
- **After the credit ends** the same work counts against the Pro five-hour and weekly limits. The artifact measured about 13% of the week per full five-hour window on the FFmpeg project, with about six windows left in its week. **Notaya has no such measurement yet:** the weekly reading is 6% now, but the cloud work was paid by the credit, so it is not a guide to what the same work costs in limits. Take a five-hour and weekly reading before and after the first Opus-heavy day *after* the credit ends, and put it in `quota-readings.csv`.
- **Codex:** no run and no quota reading yet. Its windows are watched separately (Ahmed's rule of 20:41:50 UTC).

---

## 6. What this guide does not know

- The artifact's numbers are a snapshot of 30 October, for the FFmpeg project; model lineup and prices may have moved. Re-fetch before relying on a close call.
- Dollar figures are at list prices; the credit is charged differently by about USD 10 over the project so far, cause unproven.
- Per-thread dollars are not on any screen; the PC threads carry no dollar figure.
- Cost per task is only a proxy for quota drain. Quota weights per model were never verified.
- Review and Codex costs above are estimates until those jobs have run once.
- Effort for each past session comes from the ledger's "effort" column; a few rows say "not recorded".

**Update this guide** when the ledger gets a Codex run, a diff review, or a post-credit quota reading, and when a model or its prices change.
