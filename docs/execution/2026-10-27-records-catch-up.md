# EX-2026-10-27-26: records catch-up to 2026-10-27 16:36 UTC

Short form (4 sections). Docs only. Times are UTC: the owner's message timestamps from the project chat and threads, GitHub merge times, and the cloud container's clock for the work.

## 1. Task Overview

The owner wrote "okay so document them commits and ammedments and the records" at 2026-10-27 16:36:37 UTC and "till this moment" at 16:36:45 UTC, after a read-only PBS review (another thread) found that the records had fallen behind since 2026-10-23. Executor: Claude (Claude Code, "Kevin") in a cloud thread on branch `claude/project-thread-sesvhx` from `main` at `87eb7ff`. Sources: the whole project chat from 2026-10-23 13:53 UTC to 16:38 UTC on 2026-10-27, the threads "Test the four new changes together" and "Design new app icon and name", `git log` of `main`, the GitHub pull request list, and the PBS review thread's gap list.

## 2. What was done

- **[approval-and-amendments.md](../plan/approval-and-amendments.md):** the authority table now has the owner's instructions from 2026-10-23 20:34:31 to 2026-10-27 16:36:45 UTC (it stopped at 22:25:50 on 23 Oct and skipped 20:34 to 22:03), with exact chat times and what each one did; a dated correction to the 20:22:21 row (the earlier bootstrap message is 20:20:17 UTC). Questions and status checks are grouped; nothing is recorded as approved that the owner did not say. A24 now says merged (PR #20, `7f64b86`) and released in V1.1.0; A25 gets the status cell it was missing; the sticky multi-delete becomes **A26** (`31dc920`, PR #17, `d69a3f7`) and the ribbon **A27** (`0e9d626`, PR #18, `b523df8`). A short list names the proposals that are not amendments (installer plan, icon/theme/name, native app, no-network plan, PBS).
- **Execution records:** new [EX-2026-10-27-25](2026-10-27-combined-test.md) for the combined test of PRs #17 to #20 and the check of `main` after the merges (reconstructed from the thread; it had no record); addenda naming the commits and merges in the sticky, ribbon, KeePassXC, calculator and V1.1.0 reports (`31dc920`, `0e9d626`, `4104557`, `f35acb2`, `5ddd4ec`, `58c299c`, `b995e4a`, the branch merges `e589baf`, `ef548eb`, `996ae34`, the PR merges `b523df8`, `d69a3f7`, `7f64b86`, `4600561`, `350c3a2`, `87eb7ff`, and `70d38a1`, `b090533`, `8eb8497`); the calculator report's A24 corrected to A25 in an addendum; the index updated.
- **[sessions.md](../sessions.md):** new entries for this catch-up (with the later work of 27 Oct from the chat), the combined test and the sticky multi-delete (both marked reconstructed); a dated correction to the calculator entry (A24 to A25). Older entries are not rewritten.
- **[STATUS.md](../STATUS.md):** "As of", the A24 to A27 line, mission state, active authority, questions 16 and 17 updated, questions 18 to 23 added, the changed rows of "What exists", the exact next permitted action, and "Where things live" (project chat, memory and skills, project files, GitHub).
- **[acceptance.md](../acceptance.md):** rows X20 (V1.0.1 release) to X25 (V1.1.0 release) for A24 to A27 and the two releases, all with the owner's decision pending; X1, X2, X8 (UI part), X10 and X16 marked historical.
- Not changed: code, `docs/security-design.md` (no promise changed), `docs/handoff/`, the plan snapshot, `failure-intelligence.md` (no consequential failure; the records gap is ordinary friction).

## 3. Checks

- Every commit hash named in the changed files was checked against `git log` of `main` at `87eb7ff` (`git cat-file -e` on each), and every PR merge time against the GitHub pull request list.
- Each changed file was read back after editing; the Markdown tables keep their column counts.
- Not run: the app, tests or builds (docs only, nothing in them changed).

## 4. Open items

- The owner merges this docs PR. Nothing is accepted.
- Found but outside the repo: the GitHub About text still says "bundled KeePassXC passwords" (agents cannot write repository settings), and the three saved CipherLeaf skills still mention KeePassXC (one also says medium effort). STATUS question 23.
- The no-network plan (2026-10-27 06:53:56 UTC) exists only in a thread; it is paused, so it was summarised in the authority table, not written up as a plan.
- Two reports share the short id EX-23 (the KeePassXC removal and the calculators); ids stay stable, so neither was renumbered.
