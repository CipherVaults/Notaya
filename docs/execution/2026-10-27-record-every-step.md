# EX-2026-10-27-27: record every step on GitHub and the PC; PC copy brought to `main`

Short form (4 sections). Docs only. Times are UTC: the owner's message timestamps, the GitHub merge time, and the cloud container's clock.

## 1. Task Overview

After records PR #25 was merged, the owner wrote "okay from now on you will record on both github and the pc / commits and ammendents and records with everything" (2026-10-27 16:57:24 UTC) and "and everystep" (16:57:39 UTC). Before that he wrote "do it" (16:51:37) to the offer to update his PC copy, and "done" (16:52:45) after merging PR #25 (merged 16:52:41 UTC, `2c8c2e6`). Executor: Claude (Claude Code, "Kevin") in a cloud thread, branch `claude/project-thread-sesvhx` restarted from `main` at `2c8c2e6`; the PC step ran in a Remote Control session on the owner's PC.

## 2. What was done

- **PC copy (16:53 to 16:54 UTC, PC session):** `E:\Project\cipherleaf` was on the stale branch `feat/a9-keepassxc` at `f7912fb`, with a clean working tree. After read-only checks it was switched to `main` (local `main` was also at `f7912fb`) and `git pull --ff-only origin main` fast-forwarded it to `2c8c2e6` (110 files changed). No merge, stash, reset or force; `feat/a9-keepassxc` kept locally and on GitHub. Result: HEAD = `origin/main` = `2c8c2e6`, `## main...origin/main`.
- **Found:** an untracked `vendor\` folder (173 files: KeePassXC 2.8.0 zip, source tarball, signatures and the unpacked program). It was already on the PC; A24 removed the `.gitignore` lines that hid it. Left alone; STATUS question 24. Not touched: `.devdata`, `*.clf`, `E:\Project\Cipherleaf-portable`; the app was not run and nothing installed.
- **The new standing rule** is the top row of the authority table in [approval-and-amendments.md](../plan/approval-and-amendments.md), with a row for "do it" / "done"; [STATUS.md](../STATUS.md) has it in "As of", the active authority and the next permitted action, plus question 24; a [sessions.md](../sessions.md) entry; this report and its index entry.

## 3. Checks

- `2c8c2e6` and its merge time checked with `git log` after `git fetch origin main`; the PC results are that session's own `git rev-parse` output.
- Each changed file read back; the tables keep their column counts.
- Not run: the app, tests or builds (docs only).

## 4. Open items

- The owner merges this PR; then his PC copy is fast-forwarded again, as the rule says.
- The `vendor\` folder (question 24). Nothing is accepted.
