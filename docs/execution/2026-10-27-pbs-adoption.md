# EX-2026-10-27-29: PBS adopted in its smallest useful form

Short form (4 sections). Docs only. Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

After the read-only PBS understanding review (owner's request 2026-10-27 16:29:35 UTC; the review is a Claude Doc outside the repo, since the request said to create no files), the owner wrote "read them all Review PBS" (19:05:05) and then "That's really goo just want to to follow / that's my answer" (19:08:33). The coordinator relayed this as: follow PBS, take the recommended answer to each of the review's four questions, and implement it through a docs PR. Executor: Claude (Claude Code, "Kevin") in a cloud thread. Baseline: `main` `5013244`. Write area: `CLAUDE.md` and `docs/` only.

## 2. What was done

- Recorded the owner's instruction with its time and the four accepted answers in the authority table and as amendment [A29](../plan/approval-and-amendments.md) (A28 belongs to draft PR #24). The answers: (1) the repo records win for what is authorized and where we are, project memory is a pointer only; (2) the records catch-up goes in one draft docs PR, the skill refresh goes out as a review card for the owner to save; (3) full 13-section session entries only at big stops, none for small tasks; (4) `acceptance.md` and `sessions.md` are officially part of the record set, no Decision Records.
- `CLAUDE.md`: new sections "Cloud threads" and "PBS work cycle" (baseline, scope, return and stop point; permission, observation and acceptance kept apart; the four result states; recording by size of work). Existing rules unchanged.
- `docs/STATUS.md`: "Current state" now holds only what the owner accepted (nothing). The unaccepted lines (V1.1.0, A24 to A27, V1.0.1, A22, A20 and the old mission state) moved unchanged under "Continuation", "Released and merged, not accepted". New "As of" and mission-state lines; questions 8 and 22 marked answered; next permitted action updated.
- `docs/acceptance.md`: a note that the PBS decision is a process decision, not an acceptance. `docs/sessions.md`: a new top entry. `docs/execution/README.md`: an index row.
- Not done on purpose: no Goal file, no Decision Records, no rewrite of old records, no change to `docs/plan/approved-plan-2026-10-22.md`, no code.

## 3. Checks

- Every edited file was read back after editing; the moved STATUS lines were compared with the originals by `git diff` (same text, new place). Links to the new files were checked by hand.
- **Not run:** `tsc`, build, Rust tests (no code changed). Nothing was run on Windows.

## 4. Open items

- The owner merges this PR; then his PC copy `E:\Project\cipherleaf` is fast-forwarded to the same `main` and checked against `origin/main` (record-every-step rule, 2026-10-27 16:57 UTC).
- The refresh of the three saved skills (cipherleaf-change-workflow, cipherleaf-project-coordination, cipherleaf-real-exe-test: drop KeePassXC, effort rule now Opus/Sonnet at high, update the authority row and STATUS next action at the end of a thread) is a separate review card for the owner to save. Skills are outside the repo.
- Nothing is accepted. PBS stays in this smallest form until the owner says otherwise.
