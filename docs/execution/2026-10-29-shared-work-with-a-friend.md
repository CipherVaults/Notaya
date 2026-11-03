# EX-2026-10-29-38: guide for two partners and their two Claudes

Short form (4 sections). Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `origin/main` `42de167` (merge of PR #36).
- **Owner's words:** 2026-10-29 07:55:55, "so now i want me and my freind work on the same project and his claude code and my claude code can continiu flawless and can be picked uped where it lefts". The coordinator's brief added: a docs PR with a `CLAUDE.md` section and a `docs/handoff` guide for start, resume and handoff; branch naming; a claim so two people don't take the same work or ID; test safety; the security review rule; setup steps for the friend with a first prompt; the exact clicks for the owner's invitation. V1.4.0 (PR #39) must not be touched.
- **Later messages that changed the design:** 08:33:01 "it's not exactly what i want but it's very close" (the first version made the friend a restricted collaborator); 08:38:36 "is there a way like cloud or something" (answered in chat: GitHub plus Claude Code on the web for both; a Windows build in GitHub Actions is an option not built); 08:39:33 "i want him to have the full work acess"; 08:40:02 "from the beginegn to after finishing V1.4.0". Claude read these as: the friend is a full partner for the work, and V1.4.0 is shared work from the start.
- **Scope and write area:** `docs/handoff/` (four new files), `CLAUDE.md` (one start step, one section), and the records that own the changed facts: the authority rows and A34 in `docs/plan/approval-and-amendments.md`, question 27 and one "Where things live" row in `docs/STATUS.md`, this report and its index row. **Out of scope:** any code, repository settings (the owner's clicks), the V1.4.0 branch and PR #39 (not modified), the invitation itself, the `E:\` PC copy, any GitHub Actions workflow or hook (offered, not built).
- **Work size:** normal. **Returns:** a draft PR. **Stop point:** the draft PR open and reported.

## 2. What was done

- **Read first:** `CLAUDE.md`, `docs/STATUS.md`, the amendments record, `docs/handoff/` (the 2026-10-23 checkpoint), the open PRs (only #39) and the collaborators (the GitHub API lists `AbyssWatcher2` as admin and `AE0101` with write access). The V1.4.0 branch was fetched read-only to see which IDs and files it uses (EX-2026-10-28-37, A31 to A33; a resume note exists only on that branch).
- **`docs/handoff/collaboration-guide.md`:** the two partners and what "the owner" means in `CLAUDE.md` (either partner); where permission comes from (either partner's words, including the ask-first list; both partners for making the repo public or adding a licence; conflicts between partners stop the work; a Claude's comment is never permission); start-of-session steps; claims and ID reservation (open PRs and branches are the live board; commands that print the highest EX and A numbers across all remote branches); branch names; **shared branches** (V1.4.0 is one: merge before every push, "taken by" marks, no force-push); draft PRs, merge-commit rule, merging `main` in without rebase; how to stop and hand over (resume note template); picking up the other side's work; test safety for both PCs; the security review rule; what the two Claudes do not share; three prompt cards.
- **`docs/handoff/claims.md`:** the people, the claim table (V1.4.0 as a shared branch, paused for the night; this work), the next free IDs.
- **`docs/handoff/friend-setup.md`:** accept the invitation, choose local or browser Claude Code, clone, tools for building, security helpers, a test folder outside the repository, the first session (including resuming V1.4.0), what to do when something goes wrong.
- **`docs/handoff/owner-invite-steps.md`:** the owner's clicks (the invitation, `AE0101` already has write access, merge this PR, merge commits only, an optional ruleset that only forces pull requests, the Claude GitHub app, removing access), and what the friend can and cannot do.
- **`CLAUDE.md`:** start step 7 and the section "Working with more than one person". No existing line was changed, to avoid clashing with PR #39, which edits two other lines.
- **Records:** two authority rows (07:55:55; 08:39:33 and 08:40:02), A34 (proposed), STATUS question 27, a "Where things live" row, the index row.
- **First version superseded:** the first push of PR #40 (a restricted collaborator and `.github/CODEOWNERS`) was replaced in the same PR; `CODEOWNERS` was removed.
- **Claude's readings put to the owner (question 27):** "full work access" means the friend's words give permission like his, and he may merge, release and accept; making the repository public and adding a licence need both partners; GitHub repository settings stay Mohamed's (GitHub allows only the account owner); real notes and PCs stay private to each person; V1.4.0 restarts when either partner says so.

## 3. Checks

- **Observed:** every relative link in the changed records points to an existing file (script run in the checkout); the ID commands in the guide were run against the real remote branches (they print EX-2026-10-28-37 and A33, the highest in use before this PR); `git status` showed only the intended files and no vault data or secrets.
- **Not run:** anything on a second person's PC (nobody followed the friend's setup steps; Claude Code's install is pointed to Anthropic's page, not copied); the GitHub invitation and ruleset clicks (cloud Claude cannot change repository settings; the paths are from GitHub's UI as known, not clicked through); whether a ruleset is available on the owner's GitHub plan; whether a collaborator's browser sessions need the Claude GitHub app on the repository (listed as a thing to check); whether Actions could run the real-window tests. `tsc`, `npm run build` and Rust tests were not run: no code changed. A security review of the diff does not apply to a docs-only change with no code; it was not run.
- **Result state:** *produced*. Not verified beyond the checks above, not integrated, not accepted.

## 4. Open items

- Owner: answer question 27; do the invite and optional settings ([owner-invite-steps.md](../handoff/owner-invite-steps.md)); merge with "Create a merge commit"; then send the friend the setup link and fast-forward his PC copy.
- This PR and PR #39 both add an index row, amendment rows and STATUS lines near the same places. Merge this one first; the V1.4.0 side then merges `main` into its branch and keeps both sets of rows.
- If the friend's account is not `AE0101`, correct `docs/handoff/claims.md`.
- Offered, not built, awaiting the owner's word: a committed Claude Code hook that briefs a new session and blocks stopping with unpushed work; a Windows build and test in GitHub Actions; GitHub Issues as a live claim board.
- Nothing else was started. V1.4.0 development is not resumed.
