# EX-2026-10-28-36: GitHub repository renamed to Notaya

Short form (4 sections). Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `main` `bfece93` (PR #35, V1.3.0 records).
- **Owner's words:** "Yes please rename" (2026-10-28 16:46:38), answering the coordinator's offer to rename the repository from CipherLeaf to Notaya; "renamed it" (16:48:41).
- **Scope and write area:** the GitHub repository name and the records that own the changed facts (this report, its index row, one authority row, the changed lines of `docs/STATUS.md`). Out of scope: the repository description and topics (the owner pastes those himself), code, the internal names `app.cipherleaf.vault`, `CIPHERLEAF_HOME` and the crate, and rewriting older records.
- **Returns:** the result of the rename and what the PC needs. **Stop point:** the records PR and the PC remote change.

## 2. What was done

- Tried `PATCH /repos/AbyssWatcher2/CipherLeaf` with the new name from the cloud thread: refused by the proxy, "Repository settings writes are not permitted through this proxy" (HTTP 403). Nothing changed.
- The owner renamed the repository in GitHub Settings (Settings, General, Repository name) and said so at 16:48:41.
- Checked afterwards: the API returns full name `AbyssWatcher2/Notaya` (private, default branch `main`); the cloud checkout's `origin` was pointed at the new address and `git fetch` returned `main` at `bfece93`; `git ls-remote` on the old address still returns the same `bfece93`, so the old address redirects for git.
- Searched the repository for the old address: it appears only in `docs/` (about 85 links to old pull requests and releases) and nowhere in `README.md`, the code, `packaging/` or the app config. Those links were **left as they are** because they redirect and rewriting history records is excluded by `CLAUDE.md` ("No broad rewrites of old records"). The live pointers in `docs/STATUS.md` (the Git and GitHub row and the "open or released on GitHub" row) now use the new address; a dated correction is in the authority row.
- The PC copy `E:\Project\cipherleaf` was last fast-forwarded to `bfece93` at 16:13:51 UTC (before this task; not logged in the repository until now).

## 3. Checks

- Observed: the API output and the `git fetch` / `git ls-remote` results above, from the cloud container, 2026-10-28 about 16:49 UTC.
- **Not run:** the old address through the GitHub API (this proxy refuses the redirect to a numeric repository id), the old links in a browser, and anything on the PC: the PC remote has not been changed or fetched yet. Release download links on the new address were not opened. The Claude project's own repository setting is outside the repository.
- Result state: *verified* for the rename itself (named checks above). Not accepted.

## 4. Open items

- PC: `git -C E:\Project\cipherleaf remote set-url origin https://github.com/AbyssWatcher2/Notaya.git`, then `git fetch` to confirm; after the owner merges the records PR, fast-forward `main` there and confirm it equals `origin/main`.
- Owner: the repository description and topics (suggested text was given in the project chat; the old text still says Cipherleaf and KeePassXC).
- The coordinator updates the project's repository setting and memory.

## Correction (2026-10-28, after the merge of PR #37)

This report was first numbered EX-2026-10-28-35, but draft [PR #36](https://github.com/AbyssWatcher2/Notaya/pull/36) (test leftovers cleaned up on the PC) already used that number. This report is **EX-2026-10-28-36**; EX-2026-10-28-35 stays with the cleanup report. PR #36's index row sits at the same place in `docs/execution/README.md` as this one, so that branch needs `main` merged in before it can merge.
