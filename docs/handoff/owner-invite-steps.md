# For Mohamed: inviting the friend and setting up the repository

Only you can do these (repository settings are blocked for the cloud Claude). Clicks are as GitHub shows them in 2026; if a label differs slightly, the order is what matters. Do 1 and 2 first; 3 and 4 are optional but protect `main`.

**State on 2026-10-29 (~07:58 UTC, read from the GitHub API):** `AE0101` is already listed as a collaborator with write access. If that is your friend, skip to step 2. If not, use step 1 for the right account and remove the other one (step 6).

## 1. Invite the friend

1. Open <https://github.com/AbyssWatcher2/Notaya> and sign in as `AbyssWatcher2`.
2. **Settings** (top row of tabs) → **Collaborators** (left menu, under Access).
3. **Add people** → type their GitHub username or email → pick them → **Add to this repository**. A repository on a personal account has no role choice: a collaborator can read, create branches, push and open PRs, and cannot change settings or delete the repository.
4. Their invitation sits in "Pending" until they accept (they can use <https://github.com/AbyssWatcher2/Notaya/invitations>). Invitations expire after 7 days.

## 2. Merge this PR, then send the friend one link

1. Merge this docs PR with **Create a merge commit**.
2. Send your friend this link (it works once they accepted): <https://github.com/AbyssWatcher2/Notaya/blob/main/docs/handoff/friend-setup.md>. It holds the steps and the first prompt.
3. Tell your Claude threads "pull main" (or the PC session: `git pull --ff-only` in `E:\Project\Notaya\Notaya`, then check `HEAD` equals `origin/main`). Note: PR #39 (V1.4.0) will need `main` merged into it later; the thread that owns it does that.
4. Answer question 27 in `docs/STATUS.md` (it asks you to confirm or change the rules in the guide; you can answer in the project chat).

## 3. Optional: only merge commits (your rule)

**Settings** → **General** → scroll to **Pull Requests** → untick **Allow squash merging** and **Allow rebase merging**, leave **Allow merge commits** ticked. Then nobody can squash by accident.

## 4. Optional: protect `main` so only you can merge

Rulesets on private repositories need a paid GitHub plan (Pro or above). If the menu below is missing or locked, skip this: the rules in `CLAUDE.md` and the guide still apply, but nothing technical stops a push to `main`.

1. **Settings** → **Rules** → **Rulesets** → **New ruleset** → **New branch ruleset**.
2. Name: `main is changed by pull request only`. **Enforcement status:** Active.
3. **Bypass list** → **Add bypass** → **Repository admin** (that is you; set "For pull requests only" if offered).
4. **Target branches** → **Add target** → **Include default branch**.
5. Tick **Restrict deletions**, **Block force pushes** and **Require a pull request before merging**. Under it set **Required approvals** to 1 and tick **Require review from Code Owners**. (`.github/CODEOWNERS` in this PR names you as the code owner of everything, so your friend's PRs cannot be merged until you approve, and you can still merge your own.)
6. **Create**.

## 5. If your friend's Claude cannot see the repository (browser version)

The Claude GitHub app has to be allowed on the repository: open <https://github.com/apps/claude/installations/select_target>, choose your account, and make sure `Notaya` is among the selected repositories. This also lets their cloud sessions open PRs.

## 6. Taking access away later

**Settings** → **Collaborators** → the bin icon next to their name → confirm. Their branches and PRs stay in the repository.

## What your friend can and cannot do

- Can: read all files and history (including old records with your name and PC paths), create branches, push to them, open draft PRs, comment.
- Cannot (technically, with step 4 on): merge without your approval, push to `main`. Cannot (by the written rules, always): release, accept, delete branches, force-push, add libraries, change security behavior, test on real notes.
- Never needs: your passwords, tokens, your vault, or your PC. Do not share any of those. If your friend needs a test on your PC, you start that session yourself.
