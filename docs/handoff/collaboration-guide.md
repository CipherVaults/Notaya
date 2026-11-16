# Working together: two people, two Claudes, one repository

**Dated pause correction (2026-11-10, R3):** Later heartbeat, queue and next-lane statements in this historical document are snapshots. Development is paused on Ahmed's 06:13:35 UTC words, clarified 06:25:54 UTC. Read the [pause resume note](2026-11-10-pause-resume-note.md) and [STATUS](../STATUS.md) before any action; no lane restart or merge without fresh authority. A55 transfer remains conditional on Ahmed's handover acceptance.

**Status:** proposed on 2026-10-29 at the owner's request (his messages of 07:55:55 and 08:39:33 UTC, [authority table](../plan/approval-and-amendments.md)); amendment [A34](../plan/approval-and-amendments.md), "proposed" until he confirms it ([STATUS.md](../STATUS.md), question 27). His 08:39:33 message ("i want him to have the full work acess") makes the friend a full partner, so the rules below treat both people alike.

**Why this file exists.** Mohamed and a friend are partners on Notaya, and each works with their own Claude. A Claude session starts cold, and the claude.ai project, its chat and its memory belong to one account, so **the repository is the only memory the two Claudes share.** Everything one Claude needs to continue what the other did must be committed and pushed. If it isn't in the repo, it doesn't exist for the other side.

**Baseline when written:** `origin/main` `42de167`. **In flight:** V1.4.0 (tray, the Notaya migration, colors, export and import) on branch `claude/project-thread-fdn5gx`, draft [PR #39](https://github.com/AbyssWatcher2/Notaya/pull/39), "do not merge yet", stopped for the night at Mohamed's word (2026-10-29 03:08 UTC). At 08:40:02 UTC he said the friend has full work access "from the beginning to after finishing V1.4.0", so **V1.4.0 is shared work from the start, as a shared branch** (section 5b). Its resume note is on that branch (`docs/handoff/2026-10-28-v1-4-0-resume-note.md`) and lists what is unfinished. Work started from `main` in the areas V1.4.0 changes will collide with it (see "Choosing work").

**Dated note (2026-11-07, [EX-2026-11-07-45](../execution/2026-11-07-records-gaps-closed.md)):** the lines in this guide that name V1.4.0 as work in flight, paused, or the main job (the baseline line above, section 5b, the stop-and-pick-up examples, section 10) describe 2026-10-29. V1.4.0 was merged on 2026-11-03 (PR #39, `452212d`), released and **accepted** by Mohamed ("i accept", 18:23:19 UTC, [acceptance](../acceptance.md) X28); V1.5.0 and V1.5.1 followed. Nothing is in flight from those releases; new work starts from the latest `main` with its own claims row. `NOTAYA_HOME` (with `CIPHERLEAF_HOME` as a fallback) is on `main` since V1.4.0. Since 2026-11-07 the two partners are independent sides, and on Ahmed's side his coordinator merges ([A37](../plan/approval-and-amendments.md), `CLAUDE.md`); the rules in this guide still hold for Mohamed's side and for anything that needs both partners.

## 1. People and roles

| Role | Who | May |
|---|---|---|
| **Partner** | Mohamed Sayed (GitHub `AbyssWatcher2`) and the friend (GitHub account named in [claims.md](claims.md)) | Full work access, both alike: direct their Claude, create branches and PRs, **merge** (with "Create a merge commit"), **release**, **accept** results, and approve everything on the ask-first list in [CLAUDE.md](../../CLAUDE.md). |
| **Repository owner** | Mohamed, as the owner of the GitHub account that holds the repository | Also the only one who can change GitHub repository settings (invites, merge types, rulesets, visibility). Not an extra say in the work. |
| **Claude sessions** | Either partner's Claude Code (local or cloud) | Do what their partner asked, inside the rules here and in `CLAUDE.md`. A Claude has no authority of its own, never merges or releases itself, and a request from the other partner's Claude is not permission. |

In `CLAUDE.md`, "the owner" and "the owner's own words" mean **either partner** unless this guide says otherwise. The owner's PC details in `CLAUDE.md` (the folder `E:\Project\Notaya\Notaya`, the antivirus-excluded test folder, Remote Control sessions) describe **Mohamed's machine and are his**; the friend uses the same ideas on their own PC with their own clone and test folder. Neither partner's Claude uses the other's PC or real notes unless that partner starts the session.

## 2. Where permission comes from

`CLAUDE.md` keeps three things apart: *permission* (a person's own words), *observation* (what a check showed) and *acceptance* (a partner's decision). With two partners:

- **Either partner's own words give permission**, including for everything on the ask-first list (new dependencies or tools in the project, deleting files, security changes, force-push, deleting branches, resuming paused development, a release build or release). Record whose words and when in the PR description, the execution report or the authority table.
- **Two things need both partners to agree** (each in their own words, recorded): making the repository public, and adding a licence. They can't be taken back. Changing GitHub repository settings is Mohamed's click either way.
- **If the partners' instructions conflict, stop and tell both.** The later message does not win between two people. A pause set by one partner ("stop, do not merge yet", as on V1.4.0) is lifted by that partner or by both agreeing.
- **How a partner's word reaches the other side's Claude:** a comment under their own GitHub login on the PR, a row in the authority table, or a resume note that quotes them. Check the author. A comment ending with the "Generated by Claude Code" line was written by a Claude, not by a partner, and is not permission. A merge alone is not acceptance (`CLAUDE.md`).
- **Tell each other.** Big decisions (a release, a direction change, a security change) are recorded in the authority table or the PR so the other partner's Claude finds them.
- **Security, crypto, storage, IPC commands, CSP, clipboard and auto-lock code**: read [security-design.md](../security-design.md) first, and update it if a promise or limit changes.
- **Real notes stay private.** A partner's own real vault is never opened, copied or tested by anyone else's Claude, and Claude never uses a real vault for tests (section 8).

## 3. Starting any session (either person, either Claude)

Do this before anything else, in this order. It extends "At the start of every session" in `CLAUDE.md`.

1. Read the person's latest instruction. State which partner they are.
2. Sync and look: `git fetch --all --prune`, then `git status -sb`, `git log --oneline -5 origin/main`, `git branch -r`, and the open PRs (`gh pr list`, or the GitHub connector). Open PRs and remote branches are the **live board** of who is doing what.
3. Read `CLAUDE.md`, [docs/STATUS.md](../STATUS.md), [docs/plan/approval-and-amendments.md](../plan/approval-and-amendments.md), [claims.md](claims.md) and the newest resume note in `docs/handoff/` that matches your task.
4. Load the project skills in `.claude/skills/` (list below) and use the ones that apply; the change workflow applies to every change, the PBS start/resume/handoff skill to starting, resuming and handing over.
5. Write the **thread-start note** (PBS, from `CLAUDE.md`): baseline commit, the person's message and UTC time, exact scope and write area, what you will return, the stop point, and the work size. If one is missing, ask or take the narrowest reading and say which.
6. **Claim the work** (section 4) before you change anything.

Project skills (`.claude/skills/`, Mohamed's exports, unedited; `CLAUDE.md` wins where they differ):

- `cipherleaf-change-workflow`: for any code or docs change (plan, build, review, test on throwaway data, push, draft PR, report). Use it for every change.
- `pbs-start-resume-handoff`: at the start of a session, when resuming paused work, and when handing over (PBS records, resume notes).
- `cipherleaf-real-exe-test`: whenever the built Windows app must be tested, on throwaway data only, never real notes.
- `cipherleaf-project-coordination`: only if your Claude acts as the coordinator that routes work between threads.
- `notion-native-artwork`, `personalized-notion-hubs`, `personalized-charts`: not Notaya work; they are Mohamed's own Notion and chart skills (written for "Ahmed"). Ignore them unless you are asked for that kind of task.

## 4. Claims: two people must not take the same work or the same ID

**The live board is GitHub itself:** an open PR or a pushed branch on `<login>/<topic>` means "taken". `claims.md` is the durable copy on `main`: standing items, reserved IDs, who owns what.

To claim:

1. `git fetch --all --prune`, look at open PRs and branches, read `claims.md`. If anything overlaps your scope, stop and tell your person; do not start in parallel.
2. Create your branch from `origin/main` (section 5) and make your **first commit** the claims row (what, who, branch, started in UTC, status, files or areas you expect to touch). Push it and open a **draft PR** straight away. The claim exists from that moment, not when the work is done.
3. Reserve IDs the same way. **Never reuse an ID.** Take the next free one after checking `origin/main` **and every open branch**:

```bash
# next free Execution Summary number (last number printed is the highest in use)
for b in $(git branch -r | grep -v HEAD); do git grep -h -o -E 'EX-[0-9]{4}-[0-9]{2}-[0-9]{2}-[0-9]+' "$b" -- docs/execution/README.md; done | awk -F- '{print $5}' | sort -n | uniq | tail -1
# next free amendment number
for b in $(git branch -r | grep -v HEAD); do git grep -h -o -E '^\| A[0-9]+ ' "$b" -- docs/plan/approval-and-amendments.md; done | tr -dc '0-9\n' | sort -n | uniq | tail -1
```

   The execution number counts up across days (EX-2026-10-28-36 is followed by -37, whatever the date). Add the reserved IDs to `claims.md` in your first commit.
4. **There is no lock that cannot race.** If two people claim in the same few minutes, the later PR renumbers its IDs when it is merged forward and says so in its description. Neither partner has to untangle it: it is the later PR's job.
5. When the work ends (merged, dropped or handed over), update your row's status in the same PR or a records PR. A stale claim is worse than none: either partner may mark a row `stale` after 7 days without a push, but only after asking the partner who made the claim.

## 5. Branches, commits and pull requests

- **Branch names.** Mohamed's cloud threads get `claude/project-thread-*` names automatically and keep them. Everyone else uses `<github-login-lowercase>/<short-topic>`, for example `ae0101/fix-lock-message`. Never push to someone else's branch, and never to `main`.
- **Start from the latest `origin/main`:** `git fetch origin main && git checkout -b <branch> origin/main`.
- **One topic per branch and PR.** Small, focused. Do not mix a code change with unrelated record cleanups.
- **Commits.** Clear message saying what and why. Check `git status` before every commit: never commit `.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports or `.env` (see `CLAUDE.md`). Never `git add -A` or `git add .` without reading `git status` first.
- **Pull requests are always drafts**, opened as soon as the first commit is pushed. The PR description starts with `Owner of this PR: <login> (Claude session of <partner>)` and carries the thread-start note, **what is done, what was checked (and on which exact commit), and what was not run**. Use the result words from `CLAUDE.md`: produced, verified, integrated, accepted. Most PRs stop at verified.
- **Keep your branch current with `git merge origin/main`.** No rebase and no force-push on a branch anyone else has seen.
- **Merging is a partner's decision, never a Claude's.** Either partner merges, with **"Create a merge commit"** (never squash or rebase). A partner doesn't merge a PR that their own Claude produced without reading its description, what was checked and what was not run. When two PRs wait, say in the PR description which should go first and why; the PR that merges second brings `main` in and resolves the conflict.
- **Shared records are the usual conflict spots** (`docs/STATUS.md`, `docs/execution/README.md`, the amendments table). Keep your edits small, add rows, don't reorder or reword other people's lines, and leave "Current state" alone (it holds only what the owner accepted).

## 5b. Shared branches (V1.4.0 is one)

A branch can be declared **shared** in its claims row. On a shared branch both partners' Claudes may push, so the "never push to another person's branch" rule does not apply to it. To keep two Claudes from overwriting each other:

1. `git fetch origin <branch>` and merge (`git merge origin/<branch>`) **before every push**; never rebase or force-push.
2. **Split by area, not by line.** In the branch's resume note, mark each unfinished item `taken by <login>, <UTC time>` and push that edit first. Don't touch an item someone else marked taken.
3. Commit small and push soon (at least after every finished item), so the other side sees it.
4. Run `git log --oneline origin/<branch> -10` at the start of every session on it: the other side may have moved it since your last look.
5. The branch's PR stays one PR. Its description's "done / checked / not run" lists are updated by whoever finishes an item, with the commit it was checked on.

**V1.4.0 as shared work.** Mohamed paused it for the night, so a partner says "resume V1.4.0" (either one) before work restarts; then the unfinished list in its resume note is the plan. Cloud sessions can't run the Windows exe, so PC tests (upgrade from V1.3.0, installer, tray, export and import in the real app) are run by whichever partner starts a session on their own PC, on throwaway data in their own test folder, and the result is written into the resume note and the PR with the exact commit.

## 6. Stopping and handing over (the part that makes "pick up where it left off" work)

**Standing rule:** the resume note is updated and pushed **after every finished step**, not only at the end, and always when work stops for any reason (a person says stop, a limit, a closed laptop), so the other partner's Claude can pick up from it at any time.

A cloud container is thrown away and a laptop is closed; your head is not shared. **Before any stop** (the person says stop, the session ends, a limit is near, the work is handed over):

1. **Commit and push everything**, even if it is half done or red. A broken push on a WIP branch is better than lost work. Use a clear commit message (`WIP: ...`). If a patch is too unfinished to commit on the branch, commit it as a `.patch` file under `docs/handoff/` the way `import-hardening-wip.patch` was done for V1.4.0.
2. Keep the draft PR open and make sure its **title starts with `WIP:`** while it isn't ready, and its description says "do not merge yet" if that is true.
3. Write or update the **resume note** `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md` using the template below. The note, not your memory, is what the next Claude reads.
4. Update your `claims.md` row: status (`in progress`, `paused`, `handed over`, `done`) and the resume note link.
5. Record the stop where `CLAUDE.md` says (a small task: the PR description and the record that owns the changed fact; a normal task: Execution Summary, index row, STATUS lines). **Stop at the approved endpoint**; "standing by" is not completion.
6. Tell your person, in plain words, what is done, what is unverified and what the next Claude should do first.

### Resume note template

```markdown
# <Topic>: resume note (YYYY-MM-DD, UTC time, who wrote it)

Branch / PR: <branch>, <PR link>. Baseline: <origin/main commit it started from>. Last commit: <hash>.
Owner of the work: <login>. Stopped because: <person said stop / limit / handover>, <their words and time>.

## Done (and where it is)
## Checked (what, on which commit, result) / Not run (be explicit)
## Not finished, in the order to do it
## Decisions already made by a person (their words, time) and open questions for them
## State outside the repo (PC folders, running processes, ports, settings left changed)
## Next permitted action (what the next Claude may do without asking, and what needs a partner's say-so first)
```

## 7. Picking up the other person's work

1. Read their resume note, the PR description and the last commits (`git log -10 origin/<their-branch>`). Treat claims in them as **claims to check**, not facts: re-run the checks that matter on the exact commit before saying anything works.
2. **Do not push to their branch** without their say-so (a partner may allow it for their own branch). Instead: ask your partner to ask the other partner for a **handover**: both sides write "handed over to <login>" in the claims row and a PR comment. Then continue on a new branch `<your-login>/<topic>-cont` cut from their branch, open a draft PR that says "continues #N", and the old branch's owner closes the old PR once the new one is up.
3. Never rewrite history on a branch you did not create.
4. If a note says the work is "stopped at the owner's word" or "do not merge yet" (as V1.4.0 does), do not resume it without the partner who paused it (or both partners) saying so.

## 8. Test safety (both people, always)

- **Never test on a real vault or the real window.** Use throwaway data in a separate process. The data folder comes from an environment variable: `CIPHERLEAF_HOME` on `main` (V1.4.0 renames it to `NOTAYA_HOME` and keeps the old name as a fallback; check `src-tauri/src/portable.rs` on the commit you test). Point it at a fresh temporary folder.
- **Your own real Notaya notes** (an installed copy's `%LOCALAPPDATA%\Notaya Notes`, a portable copy's `data\` folder, a dev run's `.devdata\`) are off limits to tests, builds and cleanups. A setup test can reach them through the Finish page ("Run Notaya" ticked): untick it ([FI-05](../failure-intelligence.md)).
- **Keep tests in one folder outside the repo**, one that is excluded from your antivirus if it removes unsigned test builds (it did on Mohamed's PC). Don't rely on `%TEMP%`.
- UI automation through a WebView2 debug port uses a throwaway vault, and the port is closed afterwards and checked (see `docs/failure-intelligence.md`, FI-01). Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set.
- Never commit vault data, keyfiles or secrets. Never paste tokens, passwords or keys into a prompt, a commit, a PR or a record. Each person signs in to GitHub with their own account; credentials are never shared.
- Report what could not run as **not run**, never as passed. A result applies only to the exact commit it ran on.
- Verification commands and what they show are in `CLAUDE.md`; the real-exe scripts are in `tools/real-exe-tests/` (their examples use Mohamed's paths; use your own).

## 9. Security review rule (every code PR)

Every thread that writes or changes code uses the Anthropic **Security Guidance** plugin (its hooks run by themselves where it is installed) **and**, before the PR goes to the other partner, runs the built-in **security-review** skill on the diff. Fix real findings. The PR description lists what each check found and what was **not** run (the plugin's AI diff review needs credentials that cloud sessions don't have). If you can't install or enable the plugin, say so in the PR; the built-in security review is still required.

## 10. Choosing work

- Work comes from either partner. Agree who takes what by claiming it (section 4); don't take something already claimed.
- **V1.4.0** is the main job until it ships: use its resume note's unfinished list and section 5b. For anything else, avoid the areas V1.4.0 changes (tray and start-with-Windows, the Notaya migration and renames, the vault file format, colors, export, import, the editor's shelf and page links) until it is merged.
- Documentation, tests and review of merged code are always safe starters.
- Open questions and the backlog are in [STATUS.md](../STATUS.md) and `docs/execution/`.

## 11. What the two Claudes do not share

| Not shared | What to do instead |
|---|---|
| Mohamed's claude.ai project, its chat and its memory notes | Anything that matters is written into the repo (`docs/`) by a PR. If you rely on a fact you can't find here, ask the person. |
| Each person's PC (folders, build outputs, installed apps, running processes) | Say what you left behind in the resume note ("State outside the repo"). |
| Conversation context | The thread-start note, the PR description and the resume note carry it. |
| Mohamed's saved skills | `CLAUDE.md` and this folder hold the rules; follow them as written. |

## 12. Prompt cards

Paste these as the first message of a session. Replace the bits in angle brackets.

**Start (new session):**

> You are Claude Code working on Notaya with two people, each with their own Claude. I am <my name> (GitHub <login>), a partner. Load the skills in .claude/skills/ and use the ones that apply (change workflow for every change, PBS start/resume/handoff for session start and handover). Read CLAUDE.md, docs/handoff/collaboration-guide.md, docs/STATUS.md, docs/plan/approval-and-amendments.md and docs/handoff/claims.md. Run `git fetch --all --prune`, list the open PRs and remote branches, and tell me in plain words: the baseline commit, what is in flight and who owns it, what is free, and what you suggest. Do not change anything until I tell you what to work on, and follow the claim steps in section 4 of the guide before you start.

**Resume (pick up a branch):**

> Continue the work on <branch / PR link>. Load the skills in .claude/skills/ and use the ones that apply. Read the resume note it points to, the PR description, claims.md and the last commits. Re-check what matters on the exact commit, tell me what is done, unverified and next, and ask me before pushing to a branch I didn't create (follow section 7 of the guide).

**Hand over (end of session):**

> I am stopping. Use the pbs-start-resume-handoff skill and follow section 6 of docs/handoff/collaboration-guide.md: commit and push everything, update the draft PR, write the resume note, update my claims row, and tell me in plain words what the next Claude must do first. Do not start anything new.
