# Claim board

Who is doing what, and which IDs are taken. Rules are in [collaboration-guide.md](collaboration-guide.md), section 4. **The live board is GitHub** (open PRs and remote branches); this file is the durable copy on `main`. Each PR adds or updates its own row in its first commit. Times are UTC.

**People**

| Person | GitHub | Role | Note |
|---|---|---|---|
| Mohamed Sayed | `AbyssWatcher2` | Partner; owner of the GitHub repository | Full work access; also the only one who can change GitHub repository settings |
| The friend | `AE0101` | Partner (write access on 2026-10-29, 07:58 UTC, listed by the GitHub API) | Full work access from the beginning to after V1.4.0 (Mohamed, 2026-10-29 08:39:33 and 08:40:02 UTC). Confirm with Mohamed that this is the friend before relying on it |

**Claims**

Status words: `proposed`, `in progress`, `paused`, `shared branch`, `handed over`, `done`, `stale`.

| Work | Who | Branch / PR | Started | Status | Resume note / IDs reserved |
|---|---|---|---|---|---|
| V1.4.0 release work (tray, Notaya migration, colors, export, import, editor shelf and links) | Both partners' Claudes | `claude/project-thread-fdn5gx`, [PR #39](https://github.com/AbyssWatcher2/Notaya/pull/39) (merged 2026-11-03 as `452212d`) | 2026-10-28 | **done** (released as v1.4.0 on 2026-11-03; accepted by the owner 2026-11-03 18:23:19 UTC). | `docs/handoff/2026-10-28-v1-4-0-resume-note.md` (on that branch). IDs: EX-2026-10-28-37, A31 to A33 |
| Collaboration setup (this guide, the friend's setup steps, the owner's invite steps) | Mohamed's cloud thread | `claude/project-thread-6bb493` | 2026-10-29 | in progress (draft PR) | IDs: EX-2026-10-29-38, A34 (proposed) |
| V1.5.0: Arabic / right-to-left everywhere + full local databases (with automations and Button, added 2026-11-04) | Mohamed's cloud thread | `claude/project-thread-utju09`, draft PR (WIP) | 2026-11-03 | merged into main as `c27f68e` (2026-11-04 14:24:53 UTC); published as v1.5.0 (2026-11-04 ~15:05 UTC); not accepted | IDs: A35, EX-2026-11-03-42 (the Notion roadmap thread, PR #45, did not take A35: its row says it becomes A35 only if a phase is picked; the owner picked it here) |
| V1.5.1: fixes from the owner's V1.5.0 feedback (2026-11-04 17:02 to 17:56 UTC: backspace and mouse back, scrollbars, undo/redo, tags and dates clickable, automation action dialog, hide all stickies, search history, free-position pictures, dark-theme dropdowns, clickable charts, 2 GB import, icons/covers/Home, Mermaid, Notion Hubs main page report) | Mohamed's cloud thread | `claude/project-thread-utju09` (restarted from main `0e7e580`), [PR #49](https://github.com/AbyssWatcher2/Notaya/pull/49) (merged) | 2026-11-04 | **done** (merged as `95ca8b2` 2026-11-05 07:15:18 UTC; published as [v1.5.1](https://github.com/AbyssWatcher2/Notaya/releases/tag/v1.5.1) 07:43:18 UTC; **accepted by the owner 2026-11-05 11:08:39 UTC** ("I approve this version")) | IDs: A36 (closed as released), EX-2026-11-04-43, FI-06, FI-07 |
| V1.5.1 release records and hand-over (EX-43 section 8e and 12, STATUS, authority rows, sessions, FI-06 and FI-07, resume note) | Mohamed's cloud thread (PC session) | `claude/project-thread-utju09-records`, draft PR #50 | 2026-11-05 | **done**: PR #50 merged by the owner at 08:17:10 UTC (merge commit `cf6dae6`), the PC copy fast-forwarded to it at 08:17:41 UTC | IDs: EX-2026-11-04-43, A36 |
| V1.5.1 acceptance records (acceptance X27, STATUS, authority row, EX-43 section 8f, sessions addendum, claims, resume note) and the PC fast-forward log | Mohamed's cloud thread (PC session) | `claude/project-thread-utju09-records3`, draft PR | 2026-11-05 | **in progress** until the owner merges (draft, docs only, never merged by Claude) | IDs: none new (EX-2026-11-04-43, A36) |

**Next free IDs when this file was written (2026-10-29 07:58 UTC):** execution report EX-2026-10-29-39, amendment A35, failure entry FI-06. Re-check with the commands in the guide before using them. **Updated 2026-11-05 (after V1.5.1):** the next free IDs are probably execution report EX-2026-11-05-44, amendment A37 and failure entry FI-08 (taken so far: EX-43, A36, FI-07); check `origin/main` and the open branches first.
