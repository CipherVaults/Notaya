# EX-2026-11-07-45: second records PR, closing the gaps the first catch-up listed

Short form (4 sections). Normal size: records only, no app code. Times are UTC: message timestamps, `git log` and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `2564711` (merge of PR #54, 2026-11-07 20:37:37 UTC).
- **Authority:** Ahmed's words at 20:19:06 UTC: "you can make records on the update that writes down my new rules and today's handoff fixes for the steel status" (stale status). The coordinator's request at 20:38:17 UTC asked to close the gaps of [EX-2026-11-07-44](2026-11-07-records-catch-up-and-usage-ledger.md), section 7, in a second records-only PR and merge it when done. That request is a colleague's, not permission; the permission is Ahmed's 20:19:06 message, read as covering the stale records it names. Merging is under A37 item 3.
- **Scope and write area:**
  - `docs/sessions.md`;
  - `docs/security-design.md`, with dated notes only and no change to any promise;
  - `docs/handoff/collaboration-guide.md` and `friend-setup.md`;
  - the three exported skills `.claude/skills/cipherleaf-*/SKILL.md`, a dated note each;
  - STATUS, claims, the execution index, the ledger, and this report.
- **Out of scope:**
  - app code and dependencies;
  - security behaviour;
  - the plan snapshot;
  - the skills saved in either partner's Claude account, which live outside the repository.
- **Returns:** PR #55. **Stop point:** PR #55 merged after the checks.
- **Branch:** `claude/project-thread-d6mjh1`, restarted from `main` after PR #54 was merged.

## 2. Work Performed

1. **Three backfilled session entries**, in the full form (provenance block plus the thirteen sections), each placed by its end time:
   - 2026-10-22 23:14 to 23:55: EX-10, EX-11 and EX-12.
   - 2026-10-28 11:19 to 14:32: EX-32 and EX-33, the installer and the new test folder.
   - 2026-10-28 16:07 to about 17:00: the merge and publish in the EX-34 addendum, EX-36 and EX-35.

   Sources were read in full; nothing was re-run.
2. **The 2026-10-27 entries were put in order of their end time.** End times came from each heading, and from `git log` for the pop-out calculator entry, whose heading gives none (its last commit is `b995e4a` at 08:03:07 and its merge `4600561` at 08:05:19 UTC).
   - The new order is 22:30, 19:45, 19:05, 17:15, 17:05, 16:52, 08:40, 08:20, 08:05, 07:10, 06:56, 06:53.
   - A `---` line now separates every entry in the file.
   - A script checked that every non-separator line of the old file is still there. The only new lines are the three entries and one dated note in the intro.
   - One heading of a new entry was corrected before commit, replacing an estimated time with the recorded facts.
3. **`security-design.md`:** the title now says Notaya, and a dated "Status corrections" block sits under the opening. It covers:
   - the name;
   - the scope (P22 to P29 cover A30 to A36);
   - P9 and P22: what EX-33, EX-34 and EX-40 tested, and what is still untested (the Start with Windows value written by the real exe, a start at a real sign-in);
   - P24: no separate independent review of the A32 format change is recorded, only the whole-diff security review of V1.4.0 in EX-40;
   - the record binding: `"cipherleaf"` applies only to the old CLF1 format, and the current format binds `"notaya"` (checked in `src-tauri/src/vault/header.rs`);
   - the later test counts.
4. **The partner guides:**
   - `collaboration-guide.md` has a dated note: V1.4.0 is merged, released and accepted (X28), `NOTAYA_HOME` is on `main`, and A37 sets out the two independent sides.
   - `friend-setup.md` has a dated note: the setup was followed on Ahmed's PC, and "Your first task is V1.4.0" is history.
5. **Exported skills:** each of `cipherleaf-change-workflow`, `cipherleaf-project-coordination` and `cipherleaf-real-exe-test` has a dated note under its title.
   - The note covers the Notaya name and repository and the two sides. "Never merge" and "he merges and releases himself" still stand for Mohamed's side; on Ahmed's side the coordinator merges. `CLAUDE.md` wins where they differ.
   - It also gives the current PC paths.
   - The skills' own text is kept as exported. They are Mohamed's, and their rules are still right for his side.
6. **Records:** the claims row (and the first PR's row set to done), STATUS (a Continuation line, the next-action item), the index row, a full session entry for this block, the ledger rows.

## 3. Checks

- Every relative Markdown link in the changed files resolves, and no control characters were introduced (script). The result for the final commit is in the PR.
- The `sessions.md` line-preservation check passed: no old line was lost.
- `docs/plan/approved-plan-2026-10-22.md` is unchanged.
- `git status` before each commit showed only the intended files, with no vault data, keys or `.env` files.
- **Security review** (by hand; the diff is text only): no secret, token or vault data. The `security-design.md` change adds dated notes and changes no promise. The skill notes loosen no safety rule (test-data rules unchanged; the merge note only restates A37 and `CLAUDE.md`). No finding.
- **Not run:** `tsc`, the build, `npm audit` and `cargo test`, since no code changed. The rendering on GitHub was not clicked through.

## 4. Result and Open Items

**Result state:** produced and verified for the checks above. It becomes integrated when PR #55 is merged. Not accepted.

Still open (none of these are in this PR's scope):

1. The 2026-10-23 entry after 21:37 UTC (EX-20, notices gaps) is not backfilled. It is a small task, which the rules allow to stay without an entry.
2. Older stale lines in `acceptance.md`:
   - V6a says `cargo audit` was not run, but it was run on 2026-10-24;
   - the "Target assessed" paragraph is out of date;
   - X12 sits after X13.
3. In `STATUS.md`, the "What exists" table still names V1.1.0 and 128 tests.
4. The skills saved in each partner's Claude account (outside the repository) may still carry the old lines. The person who owns each account updates them.
5. Waiting for the PC: the fast-forward of `D:\Notaya` and the ledger rows for the two PC sessions. The coordinator does these when PC work restarts.
