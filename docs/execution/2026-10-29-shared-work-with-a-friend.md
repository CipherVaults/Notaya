# EX-2026-10-29-38: guide for two people and their two Claudes

Short form (4 sections). Times are UTC: the owner's message timestamp and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `origin/main` `42de167` (merge of PR #36).
- **Owner's words:** 2026-10-29 07:55:55, "so now i want me and my freind work on the same project and his claude code and my claude code can continiu flawless and can be picked uped where it lefts". The coordinator's brief added: a docs PR with a `CLAUDE.md` section and a `docs/handoff` guide for start, resume and handoff; branch naming per person; a claim so two people don't take the same work or ID; who may approve and merge (only Mohamed); test safety; the security review rule; setup steps for the friend with a first prompt; the exact clicks for the owner's invitation. V1.4.0 (PR #39) must not be touched.
- **Scope and write area:** `docs/handoff/` (four new files), `CLAUDE.md` (one start step, one section), `.github/CODEOWNERS` (new), and the records that own the changed facts: the authority row and A34 in `docs/plan/approval-and-amendments.md`, question 27 and one "Where things live" row in `docs/STATUS.md`, this report and its index row. **Out of scope:** any code, repository settings (the owner's clicks), the V1.4.0 branch and PR #39, the invitation itself, the `E:\` PC copy.
- **Work size:** normal (Execution Summary, index row, authority row, STATUS lines). **Returns:** a draft PR. **Stop point:** the draft PR open and reported; the owner merges.

## 2. What was done

- **Read first:** `CLAUDE.md`, `docs/STATUS.md`, the amendments record, `docs/handoff/` (the 2026-10-23 checkpoint), the open PRs (only #39) and the collaborators (the GitHub API lists `AbyssWatcher2` as admin and `AE0101` with write access). The V1.4.0 branch was fetched read-only to see which IDs and files it uses (EX-2026-10-28-37, A31 to A33; a resume note exists only on that branch).
- **`docs/handoff/collaboration-guide.md`:** roles; where permission comes from (collaborator's words direct ordinary work, the ask-first list stays the owner's, a Claude's comment is never permission); start-of-session steps; claims and ID reservation (open PRs and branches are the live board; commands that print the highest EX and A numbers across all remote branches); branch names (`<github-login>/<topic>`, owner's cloud threads keep `claude/project-thread-*`); draft PRs, merge-commit rule, merging `main` in without rebase or force-push; how to stop and hand over (commit and push everything, `WIP:` title, resume note template, claims row); how to pick up the other side's work; test safety for both PCs; the security review rule; what is not shared between the two Claudes; three prompt cards (start, resume, hand over).
- **`docs/handoff/claims.md`:** the people, the claim table (V1.4.0 marked paused and not to be touched; this work), and the next free IDs.
- **`docs/handoff/friend-setup.md`:** accept the invitation, choose local or browser Claude Code, clone, tools for building (the versions the project used), the security helpers, a test folder outside the repository, the first session, what to do when something goes wrong.
- **`docs/handoff/owner-invite-steps.md`:** the owner's clicks for the invitation (noting that `AE0101` already has write access), merging this PR, only-merge-commits, an optional branch ruleset with a code-owner review, the Claude GitHub app, removing access.
- **`CLAUDE.md`:** start step 7 and the section "Working with more than one person" (short form of the guide). No existing line was changed, to avoid clashing with PR #39, which edits two other lines of that file.
- **`.github/CODEOWNERS`:** `* @AbyssWatcher2`. It only has an effect with the ruleset in the owner's step 4; it also requests his review on every PR.
- **Records:** authority row (07:55:55), A34 (proposed), STATUS question 27, a "Where things live" row, the index row.
- **Not decided by Claude, put to the owner (question 27):** the permission rules in section 2 of the guide. They extend the owner-only permission rule in `CLAUDE.md` in one way (the collaborator directs ordinary work), so they are marked proposed.

## 3. Checks

- **Observed:** every relative link in the new and changed records points to an existing file (script run in the checkout, below); the ID commands in the guide were run against the real remote branches (they print EX-2026-10-28-37 and A33, the highest in use); `git status` showed only the intended files and no vault data or secrets.
- **Not run:** anything on a second person's PC (the friend's setup steps were not followed by anyone; install commands for Claude Code are pointed to Anthropic's page, not copied); the GitHub invitation and the ruleset clicks (cloud Claude cannot change repository settings; the click paths are from GitHub's current UI as known, not clicked through); whether a ruleset is available on the owner's GitHub plan; whether the Claude GitHub app is needed for a collaborator's browser sessions (stated as a thing to check). `tsc`, `npm run build` and Rust tests were not run: no code changed. A security review of the diff does not apply to a docs-only change with no code; it was not run.
- **Result state:** *produced*. Not verified beyond the checks above, not integrated, not accepted.

## 4. Open items

- Owner: answer question 27; do the invite and optional settings ([owner-invite-steps.md](../handoff/owner-invite-steps.md)); merge with "Create a merge commit"; then send the friend the setup link and fast-forward his PC copy.
- This PR and PR #39 both add an index row, an amendment row and STATUS lines near the same places. Merge this one first; the V1.4.0 thread then merges `main` into its branch and keeps both sets of rows.
- If the friend's account is not `AE0101`, correct `docs/handoff/claims.md`.
- Nothing else was started.
