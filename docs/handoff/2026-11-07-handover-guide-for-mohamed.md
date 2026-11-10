# Notaya handover guide for Mohamed (2026-11-07)

**Dated pause correction (2026-11-10, R3):** Later heartbeat, queue and next-lane statements in this historical document are snapshots. Development is paused on Ahmed's 06:13:35 UTC words, clarified 06:25:54 UTC. Read the [pause resume note](2026-11-10-pause-resume-note.md) and [STATUS](../STATUS.md) before any action; no lane restart or merge without fresh authority. A55 transfer remains conditional on Ahmed's handover acceptance.

**Dated R2 correction (2026-11-10):** earlier state and role lines below are historical snapshots. For the later 1.6 lane state, unresolved N2 test conflict and the conditional A55 handover, read [STATUS](../STATUS.md) Continuation and session 55. A merged lane is not partner acceptance.

**Who and where:** written for **Mohamed Sayed (`AbyssWatcher2`)** and his Claude, from **Ahmed's side (`AE0101`)**, by a Claude cloud thread in Ahmed's Claude project "Notaya" (machine: cloud). First written 2026-11-07, about 22:30 UTC (container clock), from the repository at `main` = `987bfc5` (merge of PR #56); updated 2026-11-07 about 23:56 UTC from `main` = `67b27e4` plus the manual PR #63, and again 2026-11-08 about 00:35 UTC from `main` = `c2ead6e` (PR #68) plus PR #69 (see the change log). A published page with the same content (private to Ahmed unless he shares it): <https://claude.ai/artifact/UhKn7cS38rp7Sq89YfGLyp>. This Markdown copy exists because the repository is the only memory the two Claudes share.

**Living document (Ahmed, 2026-11-07 22:33:37 UTC, [A39](../plan/approval-and-amendments.md)).** This guide is kept up to date at every milestone or decision on Ahmed's side, in both copies (this file and the published page), so that when Ahmed's cycle ends Mohamed has something solid with few or no gaps. **Last updated: 2026-11-08 07:18 UTC** (the 1.6 build: lanes C1, B and D merged, section 17; the published page copy is **not yet updated** for this change, see the change log; before that (the checkpoint at 00:35 UTC, PR #69, section 16; the Notaya Manual, PR #63, and the Phase 1 decisions, PR #61). See the change log at the end.

**What this is.** A reading aid: what Ahmed's side has set up, studied and decided so far, so that Mohamed's machine and Claude can carry the same methods when Ahmed's cycle ends. **It is not an authority record.** If it disagrees with `docs/plan/approval-and-amendments.md` or `docs/STATUS.md`, those win.

**Result state:** produced. Not read by Mohamed, not accepted. Nothing was run or tested; this guide describes records, not the app.

## How items are marked

- **[Shared rule]** already in the repository for both sides, or independent of who you are.
- **[Ahmed's side]** decided by Ahmed for his own setup. **Mohamed has not agreed to these yet.** They are offered, not imposed.
- **[Needs both]** cannot happen unless Ahmed and Mohamed each say yes in their own words.

## 1. The short version

Notaya now has two partners who work independently on the same repository: Mohamed Sayed (`AbyssWatcher2`, owner of the GitHub account) and Ahmed Emara (`AE0101`). Each has their own Claude. The only memory the two Claudes share is the repository, so anything that matters is written into a committed file.

- **The method is PBS** (the AI Project Bootstrap System). On Ahmed's side it governs every task and every thread, with thorough records. [Ahmed's side]
- **Every record says whose side and which machine** (rule A38). [Ahmed's side], offered to Mohamed.
- **A usage ledger** tracks how much Claude and Codex work each session used, in `docs/ledger/`. [Ahmed's side]
- **Order of work:** talk through every feature (**done on 2026-11-07, except the AI CLI action list**), then plan and stage across versions (**may start now**, Ahmed 23:39:31 UTC), then build (only on Ahmed's go per feature). Before discussing a feature, check what Notaya already has. Nothing is built yet. [Ahmed's side]
- **The AI CLI is the foundation** of the next work: Claude or Codex will drive Notaya from outside. Every new feature gets named AI actions from its first design. Security is designed in from the start: sealed in, switch only in Settings, proven patterns, no home-made security (section 10). [Ahmed's side]
- **Two design rules for every feature:** no trademarked or copyrighted names inside Notaya (own names, such as "Notebook mode" and "Default style"), and every menu or strip is customizable in Settings (A42). [Ahmed's side]
- **Last accepted release:** V1.5.1, accepted by Mohamed on 2026-11-05 11:08:39 UTC. Nothing newer is accepted. [Shared rule]
- **Making the repository public and adding a licence need both partners.** GitHub repository settings stay Mohamed's click. [Needs both]

## 2. Start a session on your machine

Open Claude Code in your copy of the repository and paste this as the first message. It follows the start steps in `CLAUDE.md` and section 12 of the collaboration guide, with the new records rules added.

```text
You are Claude Code working on Notaya for Mohamed Sayed (GitHub AbyssWatcher2), a partner. Ahmed Emara (AE0101) is the other partner and works independently with his own Claude.

Read, in full and in this order: CLAUDE.md, docs/STATUS.md, docs/plan/approval-and-amendments.md (especially A34, A37, A38 and the dated notes after the table), docs/handoff/collaboration-guide.md, docs/handoff/claims.md, the newest resume note in docs/handoff/, the "Sessions at a glance" index and the newest entry of docs/sessions.md, and docs/ledger/README.md.

Then run: git fetch --all --prune, git status, git log --oneline -5, and list the open pull requests.

Tell me in plain words: the baseline commit, what is in flight and who owns it, what Ahmed's side has decided that I have not yet agreed to, what is free, and what you suggest. Do not change anything until I say what to work on, and claim the work (guide, section 4) before you start.
```

Reading order, if you read it yourself:

1. `CLAUDE.md`: the working rules for any Claude in this repository.
2. `docs/STATUS.md`: what is accepted and exactly where work resumes ("Exact next permitted action").
3. `docs/plan/approval-and-amendments.md`: who said what, with exact times; amendments A1 to A43.
4. `docs/handoff/collaboration-guide.md` and `claims.md`: how two people share one repository, and who has claimed what.
5. The newest entries of `docs/sessions.md` (entries 41 and 43, 2026-11-07: what each feature means and why) and `docs/ledger/README.md`.

## 3. Who can decide what

Three things stay apart in every Notaya record: **permission** (only a partner's own words), **observation** (what a check showed, on which exact source) and **acceptance** (a partner's decision; a merge, a release or a screenshot is never acceptance).

| Question | Answer | Marking |
|---|---|---|
| Whose words authorize work? | Each partner's own words authorize the work of **their own** Claude and Codex setup. Ahmed's govern Ahmed's side; Mohamed's govern Mohamed's side (Ahmed, 2026-11-07 20:23:20 UTC, A37 item 1). | Ahmed's side |
| Can a Claude merge or release? | **Ahmed's side:** his coordinator merges (with "Create a merge commit") once the work is checked, and publishes a full release when it judges it ready. Ahmed's OK is enough and needs no review by Mohamed. **Mohamed's side:** unchanged. A Claude never merges or releases by itself; Mohamed merges, releases and accepts. | Ahmed's side |
| Who accepts a result? | A partner's own word, on both sides. A merge or release by a coordinator is never an acceptance. | Shared rule |
| Make the repository public, or add a licence | Both partners must agree, each in their own words, recorded. The licence for Notaya itself is still not chosen. | Needs both |
| GitHub repository settings (invites, merge types, rulesets, visibility) | Only the owner of the GitHub account can click them: Mohamed. | Shared rule |
| The two partners give conflicting instructions about the same thing | The Claude that sees it stops and tells both. The later message does not win between two people. | Shared rule |
| Still asks first, on both sides | Installing tools or dependencies, deleting files, changing security behavior, force-push, deleting branches. On Ahmed's side the coordinator may commit, push and merge without asking each time; the others still need Ahmed's own words. | Shared rule |

**For Mohamed to confirm.** Amendment A34 (how two partners and two Claudes work together) has been "proposed" since 2026-10-29. Your confirmation of Claude's reading was never recorded (STATUS question 27). Ahmed's A37 settles how the two sides' authority fits, but only for his side. Please confirm or change the A34 reading, and say whether you want any of Ahmed's rules on your side.

## 4. PBS, the method

You adopted PBS in its smallest useful form for Notaya yourself on 2026-10-27 19:08:33 UTC (A29). On 2026-11-07 21:56:54 UTC Ahmed made it the method for **all** Notaya development on his side, "across the entire development of Notaya, not just this task". The skill is in `.claude/skills/pbs-start-resume-handoff/` (method version 1.1). [Ahmed's side]

**The work cycle for every task**

1. **Start.** Read the partner's latest instruction, STATUS and the amendments record. Look at the live repository. Take the baseline commit from `git log`.
2. **State the task.** Baseline commit, the partner's message with its UTC time, exact scope and write area, what will be returned, the stop point. If one is missing, ask or take the narrowest reading and say which.
3. **Keep three things apart:** permission, observation, acceptance.
4. **Name the result state:** *produced* (it exists), *verified* (named checks passed on this exact source), *integrated* (merged into `main`), *accepted* (a partner said so). Most results stop at verified.
5. **Stop at the approved endpoint.** "Standing by" is not completion. Say what is done, what is unverified and what you wait for.
6. **Record by size.** Small: the PR description. Normal: an Execution Summary plus its index row, and the changed lines of STATUS. Big: the 12-section Execution Summary and a Session Summary entry.

**What Ahmed added on top** [Ahmed's side]

- **Thorough, mandatory documentation.** Records are read in full before acting, never skimmed (Ahmed, 20:12:10).
- **Every session that changes the repository, records a decision or ends a work block gets a Session Summary entry in the full 13-section form**, with the "Models, Effort, Roles and Provenance" block. No exemption for small tasks on his side.
- **One session file.** `docs/sessions.md` holds all entries, newest on top, oldest at the bottom, "even if it takes thousands of lines". Older entries are never rewritten; a correction is a dated note.
- **This guide is a living document.** It is updated at every milestone or decision, in both copies, with a change log (A39, Ahmed 22:33:37).
- **The Notion hub** (roadmap and feature board) and the PBS records are updated at each milestone, written so Mohamed can see what was done. The hub is private in Ahmed's Notion; the repository records win where they differ.

## 5. Who wrote what: partner and machine on every record

Ahmed's reason (2026-11-07 22:15:49 UTC): for the first time two different people share the same PBS, so it must be clear which developer did the work and from which machine each decision summary comes, "because naturally you will find different conversation, different decision, different thought passes". He also wants titles that are easy to scan, "but not to be flooded with everything". This is rule **A38**. [Ahmed's side], offered to Mohamed's side.

**Anatomy of a session title**

```text
# 📘 Session: 2026-11-07 · Ahmed · cloud + Gaming-PC · First feature discussion
             date (UTC)    partner  machine             a few words of topic
```

**The five parts of the rule**

1. **Every record names the partner's side** (with GitHub login: Ahmed `AE0101`, Mohamed `AbyssWatcher2`) **and the machine** (cloud, Ahmed's `Gaming-PC`, Mohamed's PC, or a Remote Control session on a named PC). If unknown, write "not recorded". This covers session entries, authority rows, Execution Summaries, ledger rows and claims.
2. **Short titles,** then a **Who and where** line under the title (partner and login, Claude account, machines, time range, records editor).
3. **An index at the top of `sessions.md`**, "Sessions at a glance", lists every entry with number, date, partner, machine and topic. Each entry has a stable anchor `session-NN` (the oldest is 01).
4. **Older records are not rewritten.** Before 2026-11-07, "the owner" in the authority table means **Mohamed**. Execution reports before EX-2026-11-07-44 are Mohamed's side. The index, dated notes and new ledger columns say the rest.
5. **Commit messages on Ahmed's side end with** `Side: Ahmed (AE0101) · Machine: <machine>`, because commits from cloud threads are authored "Claude" and say nothing about the side. They are also written as clear points a non-developer can follow.

**What this means for you.** If your Claude writes in these records, it should state that it is Mohamed's side (`AbyssWatcher2`) and which of your machines it ran on, in the same short-title form. Your Claude keeps its own rules until you say otherwise.

**Reading older records correctly**

- Index entries 1 to 37 are Mohamed's side. Entry 38 (the hand-off to Ahmed's PC) was written by Mohamed's Claude account working for Ahmed.
- Ahmed's voice-to-text sometimes mis-hears words. The records read "Hamatai" as Mohamed, "codecs" as Codex, and "PPS" or "BBS" as PBS.

## 6. Where each record lives

Repository files own the facts. Project memory, chats and Notion are pointers only.

| Question | Record |
|---|---|
| What is accepted, and where does work resume? | `docs/STATUS.md` ("Current state" holds only what a partner accepted; unaccepted work is under "Continuation") |
| What is authorized, who said it, when? | `docs/plan/approval-and-amendments.md` (rows with exact UTC times; amendments A1 to A43) |
| What happened, in order, and why did choices change? | `docs/sessions.md` (one file, newest on top, with the "Sessions at a glance" index) |
| What did a piece of work do, and what was checked? | `docs/execution/` and its index `docs/execution/README.md` |
| Which criteria are met, and who accepted? | `docs/acceptance.md` |
| How much Claude and Codex usage did a session take? | `docs/ledger/` (`README.md`, `usage-ledger.md` and three CSV files) |
| Who is doing what, and which IDs are taken? | `docs/handoff/claims.md` (the live board is GitHub: open PRs and branches) |
| How to pause and pick up work | `docs/handoff/` resume notes and `collaboration-guide.md` |
| What went wrong or nearly did | `docs/failure-intelligence.md` |
| What does Notaya promise about security? | `docs/security-design.md` (read before touching crypto, storage, IPC commands, CSP, clipboard or auto-lock) |
| What does Notaya already do? | The code, and the full inventory `notes/2026-11-07-notaya-full-feature-inventory.md` in Ahmed's project files (it is not in the repository) |
| What does Notaya do today, in plain words? | **The Notaya Manual**, first edition for 1.5.1 (2026-11-07): `docs/manual/notaya-manual-first-edition.html` and its published page <https://claude.ai/artifact/K6HUfVdCJbuSFNCkiiCu3K> (private to Ahmed unless he shares it). It also lists the known limits and a draft "Coming soon". Each release gets a new edition (`docs/manual/README.md`). The repository records win where they differ. |
| The original plan | `docs/plan/approved-plan-2026-10-22.md`. **Never edit it**; record changes as amendments. |

Take the next free `EX-` and `A` numbers only after checking `origin/main` and every open branch (commands in the collaboration guide, section 4). Check `claims.md` for the current next-free line.

## 7. The usage ledger

Ahmed asked for it on 2026-11-07 at 20:12:10 UTC because "we need to analyze how we are even using the data, and tokens going back and forth from where, so we can analyze our work to predict our pacing". It belongs to Notaya only; nothing from his FFmpeg Encoder project's ledger is copied or shared. [Ahmed's side]

| File | One row per |
|---|---|
| `docs/ledger/usage-ledger.md` | A dated block per session or day, newest on top, explaining the numbers in plain words |
| `claude-sessions.csv` | Claude session or task (columns include `partner` and `where`) |
| `codex-runs.csv` | Codex run, with the quota read before and after |
| `quota-readings.csv` | A reading of a usage limit at a known time (5-hour, weekly, credit) |

**First readings (2026-11-07, Ahmed's side):** eight cloud sessions measured at 336,102 output tokens and about 10.67 million weighted tokens (reported cost about USD 18.41 for seven of them); the two Gaming-PC sessions added about 1.06 million weighted tokens; most of the volume is cache reads. "Weighted" is a yardstick, not a bill: input 1, cache write 1.25, cache read 0.1, output 5. No Codex run yet. The ledger README has the current numbers.

**Refresh it at every session stop:** (1) cloud sessions: copy the usage block from the session record into `claude-sessions.csv` with the snapshot time; (2) sessions on a PC: run `tools/usage-ledger/scan_claude_transcript.py` on that session's transcript, on that PC; (3) Codex: one row per run with its tokens and the quota before and after; (4) add a quota reading and a dated block at the top of `usage-ledger.md`, and quote the headline in the session summary.

**Limits.** Quotas are account-wide, so the ledger cannot separate Notaya's share inside one reading. **Your usage runs on your own account and is not visible from Ahmed's side:** if you want it tracked, your Claude adds its own rows (`partner` = `Mohamed (AbyssWatcher2)`). No pacing cap is set: Ahmed said to watch the 5-hour and weekly limits of Claude and Codex so work is not cut off mid-task (STATUS question 28). A ledger page exists (private to Ahmed, rebuilt from these files): <https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY>. The files are the source; the page is a view of them.

## 8. How Ahmed's side works

Ahmed's operating model (A37). Use any of it that suits you. [Ahmed's side]

| Role | What it does |
|---|---|
| Ahmed | Director and partner. Decides the product and gives the go per feature. Explanations to him are in plain words, with a short summary when something finishes, and open questions gathered into one numbered list. Unrequested improvements are welcome but must be listed clearly as additions. |
| Claude coordinator ("mother thread") | Leads and plans. Chooses model and effort for each task, starts and steers threads and Codex, inspects every Codex diff for Ahmed's vision, coherence and correctness, runs the security review, keeps the records and the ledger, and commits, pushes and merges. |
| Claude threads | Cloud (or Remote Control on a PC) sessions for one job each: reading, studying, records, planning. Opus 5.5 or Sonnet 5.5 at high effort. Used first while the 2026-11-07 cloud credit lasts. |
| Codex (`gpt-6.1-sol`, reasoning `xhigh`) | The developer hand that writes the bulk of the code and tests from a bounded brief. It does not merge, release or change records of authority. Its diff is reviewed by Claude, because the agent that wrote a diff is never its only reviewer. |

**Choosing models** (Ahmed 22:58:04; a reading by the coordinator, not yet accepted by him, A43). Model and effort are chosen per task from his model-economics artifact (<https://claude.ai/artifact/1eLfHxqU4uyhKkdn3ZZjoL>): Sonnet 5.5 high for reading and records, Opus 5.5 high for discussion and design, Codex `gpt-6.1-sol` xhigh for building, one Opus xhigh reviewer per substantive diff; never Opus max, Sonnet xhigh, Fable 5.1 or review panels. The coordinator's own chat is capped at low effort by the platform, so heavy work runs in threads. The cloud credit (USD 100) expires 2026-11-08 19:09 UTC; afterwards cloud work counts against the Pro limits. [Ahmed's side]

**Where things run**

- The coordinator decides PC or cloud for each task. Cloud sessions cannot run the Windows exe; checks there are `tsc`, `npm run build`, `npm audit` and Rust tests under Wine. Whatever cannot run is reported as **not run**.
- The real app is tested only through a Remote Control session on the PC, on throwaway data.
- **Ahmed's PC:** repository `D:\Notaya` (the path must not contain an apostrophe), tests in `D:\Notaya Tests`, machine name `Gaming-PC`. **Yours:** `E:\Project\Notaya\Notaya`, tests in `E:\Project\Notaya\Notaya Tests Excluded`.
- After a merge, the PC copy is fast-forwarded (`git pull --ff-only` on `main`) and checked to equal `origin/main`. Never `git add -A` in a PC copy.

**Dated update (2026-11-08 00:00 UTC, A45).** The cloud credit is used up (Ahmed, 23:59:56 UTC), so work runs on his Claude plan with its 5-hour and weekly limits. **Claude may use up to 70% of the weekly quota** until he changes it; **Codex may use up to 100%**, including all its resets. The line "Claude threads... used first while the credit lasts" above is history. Now: **Codex builds, Claude briefs, inspects, merges and records** (Ahmed, 00:18:27); fewer parallel Claude threads, Sonnet instead of Opus where it does the job, short reading lists. [Ahmed's side]

## 9. Three phases, and know what exists first [Ahmed's side]

1. **Discuss every feature** until Ahmed and Claude agree on what it means. No version is assigned. (Ahmed, 21:14:10: "there is a huge difference between me laying down the points like in a text way to remind myself what we need, and ... what we discuss".) **Done on 2026-11-07 at 23:29 UTC** for every feature on his list; only the AI CLI's list of actions is still being collected.
2. **Plan and stage** across versions 1.6, 1.7, 1.8 and on, by what is doable. Ahmed (20:41:50): "we don't have to push everything and shove it into 1.6". At 23:39:31 UTC he said to push ahead in parallel and "Don't wait for my authority" for this mapping and planning work (A43). The coordinator brings a staging plan, a list of Notion features Notaya lacks and the Notion hub update. *This is where we are now.*
   **Phase 2 version plan (2026-11-07, EX-2026-11-07-50, reported by the version-planning thread at 23:47 UTC; draft PR #62, not merged when this line was written, so a claim to re-check):** every agreed feature staged across V1.6 to V1.15 in four waves; foundation first (the AI CLI and its safety net in 1.6); nothing greenlit to build. Ahmed has not yet reviewed it.
3. **Build,** with Codex as the developer hand and the coordinator inspecting every diff, only on Ahmed's go per feature or version.

**Know what exists first (Ahmed, 21:00:14).** Before discussing, planning, designing or building any feature, check what Notaya already has in the code and in the records. Never redesign something that exists, and never break it. The full inventory read the code at `9474511` and found, for example, that there are no mouse right-click menus anywhere, that an inline database is silently dropped from page exports (read in the code, not yet seen in the app), and that no outside entry point for an AI exists yet. Re-check on the current commit before relying on any of it.

**Dated update (2026-11-08 00:35 UTC).** Phase 1 is done and **Phase 2 is finished**: the version plan (1.6 foundation, 1.7 comfort, 1.8 Notion page features, 1.9 tabs and import/export, then **2.0** corkboard and manual, 2.1 and 2.2 dashboards, 2.3 and 2.4 whiteboard, 2.5 Notebook mode; the Notion migration runs beside it) is in `docs/plan/2026-11-07-version-plan.md`; the 1.6 spec (15 work packages, nine lanes for Codex, four gates) in `docs/plan/2026-11-08-v1-6-spec.md`. **Phase 3 has begun for 1.6 only:** Ahmed said go at 00:15:27 UTC (A44) and Codex is running the first lanes on his PC. The older line "the plan ... draft PR #62, not merged" above is stale: PRs #62, #65 and #67 are merged. [Ahmed's side]

## 10. The AI CLI foundation [Ahmed's side]

Ahmed uses Claude and Codex from outside to drive Notaya locally, in the way Claude uses Notion through MCP, but with more power and freedom, "because it's local and it's our code and it's our app".

**Agreed in principle**

- Every part of Notaya is reachable. The AI sees everything; **locked notes only after they are unlocked**.
- The AI may create, edit and delete. **Deletes need confirmation.**
- An **activity log** records everything the AI does.
- **Every feature is designed with named AI actions** from its first design, including moving and copying pages and databases between places.
- **Long-term north star (direction only, not now):** Notaya eventually matches Notion closely enough that Ahmed's whole Notion can be pulled in as-is, as an always-on local backup.

**Safety direction, agreed 22:45:28 and 22:49:30 UTC (A41; direction only, nothing built, no promise in `security-design.md` yet).** Ahmed's reason: he fears the attack surface of an AI that can delete, write, change or even make malware, and also does not want a weaker feature set or a home-made security system ("We are not cybersecurity experts, but we have to do our best").

- Security is part of every feature from its first design.
- **Sealed in:** the AI reaches only Notaya's notes; no programs, no other files, no internet.
- **Off by default;** the on/off switch lives **only in Notaya's own Settings**, so the AI cannot flip it.
- Permission levels (read-only, can edit, full) and limits to chosen notebooks; deletes go to a recycle bin after the person confirms; AI edits are kept in version history and can be undone; a log of AI actions that the AI cannot edit; locked notes stay locked.
- Proven, well-known patterns; one simple confirm for deletes and very large changes; the AI is never given a way to change its own permissions; backups are the safety net.

**Not decided yet**

- The action list itself (a thread is collecting it for Ahmed to review).
- How an outside program reaches Notaya and proves it is allowed.
- Whether the AI may work while the Notaya window is open, and how the window refreshes after an outside change (today the interface keeps its own copy of the data and is not refreshed from outside).
- Where a delete is confirmed, where the activity log lives, and how it is made tamper-proof.

**Security.** The AI CLI creates a new trust boundary. Under `CLAUDE.md`, changing security behavior needs the partner's own words, and `docs/security-design.md` must be updated with any new promise. Keys and secrets stay in the Rust core. Because this touches shared code and shared promises, Ahmed's design should be shown to Mohamed before it is built, and each promise must be written into `docs/security-design.md`, tested and reviewed first. [Needs both]

**Dated update (2026-11-08 00:35 UTC): the items under "Not decided yet" are now mostly decided** (Ahmed 23:53:46, A45; design details in the 1.6 spec, not yet built and not yet in `docs/security-design.md`). Four levels: Off (default), Look, Edit, Full (Full may delete whole hubs after a confirmation). Notebooks can be hidden from the AI. Confirmations appear only in the Notaya window. A big-change confirm is an optional setting, default 20 items. Edits are snapshotted and can be undone; a tamper-evident activity log. The door is a small companion program (`notaya-bridge`) speaking the standard AI tool protocol (MCP) over a private Windows pipe that only the same Windows account may open and that exists only while the AI is switched on; the AI cannot lock or unlock anything (locks deferred) and cannot change its own level. Files move through an exchange folder next to the data folder. Claude and Codex may be connected together if kept apart by claims and version checks. Gates G1 (the door and safety rules), G2 (the companion program), G3 (one local copy at the first 1.6 launch, wording for Ahmed to confirm) and G4 (deletes go to a bin) are approved (A44). **Mohamed should still read the 1.6 spec section 8 (security notes) before it ships.** [Ahmed's side, needs both for the security promises]

## 11. Feature decisions so far

All of these are **Ahmed's** decisions from the 2026-11-07 discussion (full tables with his reasons: `docs/sessions.md`, entries 41 and 43, section "Decisions Made"; the amendments A40 and A41 hold the second half). **None of them is permission to build.** Mohamed has not yet reviewed them. ✅ agreed, 🟡 partly discussed (only the AI CLI's action list is still open), ⬜ not discussed (none left).

| Feature | State | What was agreed | Exists today |
|---|---|---|---|
| Mermaid in code blocks | ✅ | Language picker on every code block; pasted Mermaid is detected and drawn; code collapsed behind a click; zoom, download, copy, wrap. Merge with the existing Diagram block. | Separate Diagram block draws Mermaid; paste not detected; no language picker. |
| Import | ✅ | Keep every importer feature. Add a top-bar Import button. Fix the out-of-date sentence in the Import window. | Very complete importer; entry points are a hover icon, the empty sidebar and the command palette. |
| Export ("Save as") | ✅ | Top-bar Export button. PDF gets a contents page with clickable bookmarks. Excel export for databases. Fix the inline-database export bug after checking it in the real app. | Export dialog with scope, sub-pages, PDF/Word/Markdown/HTML. No Excel. |
| Sticky-notes corkboard | ✅ | In-app board with grid mode and free mode, drag, box-select, coloured pins and strings, right-click menu, New sticky note from the tray. No pins or strings on the Windows desktop. | Grid board, 8 colours, Select mode. Each note can already open as its own desktop window, with Keep on top. |
| App bar (like Office Quick Access) | ✅ | Customizable strip of quick buttons in the top bar. Defaults: Import, Export, New page, New sticky note. Not Favorites, not New notebook. Optional: Calculator and others. Pin any page to it; the AI can arrange it. | Fixed top bar. |
| Browser-style tabs | ✅ | New tab, pin, group, close, move, reopen with Ctrl+Shift+T. Open tabs return after a restart. | One page at a time. |
| Version history and activity | ✅ | Per-page timeline with added/removed highlighted and one-click restore. Activity list marks AI edits ("with Claude"). Check the existing backups first. | Whole-vault backups only. |
| Ctrl+F find (try, then tune) | ✅ | Notaya's own find bar for the open page, including folded and hidden parts, "3 of 12"; a "Search all notes" button widens it; Ctrl+H find and replace. | Ctrl+F shows the browser engine's bar; real search is Ctrl+K. |
| Editable shortcuts | ✅ | Settings page listing every action; click and press new keys; conflict warning; reset one or all; global shortcuts while Notaya is in the background. | Fixed in code, no settings page. |
| Calculator upgrades | ✅ | History panel on the right; offline unit converters (volume, length, weight, temperature, energy, area, speed, time, power, data, pressure, angle). No currency. | Basic and scientific calculators with history below. |
| "+ New notebook" button | ✅ | Always visible at the bottom of the notebook list only. | Shows only on mouse-over. |
| Icons pass | ✅ | Check every icon in every theme with screenshots on throwaway data. Favorite star: clear outline when off, gold when on. | Star is faint grey off, white on. |
| Reorder databases in the sidebar | ✅ | Drag databases among pages like any page. Moving them between pages or notebooks is a separate, later topic. | Databases are stuck below the pages. |
| AI CLI (foundation) | ✅ in principle | See section 10. Action list and safety design still open. | No outside entry point; about 120 internal commands the UI already uses. |
| Home banner | ✅ | Settings picker per time of day (image or folder per period, shuffle, reset to built-in); taller banner by default; big clear page icon overlapping the banner; Settings with live preview for banner height, icon size and icon position; Change and Reposition buttons; per-page override. | Built-in daily gradient with a greeting; cover banners and page icons exist but are small. |
| Whiteboard | ✅ | Its own section; infinite freeform canvas; Microsoft Whiteboard's core offline features rebuilt in Notaya's own style (own original templates, backgrounds, colours, shapes, pens, fonts, sizes, timer); OneNote's Draw tab as style inspiration; mouse, pen, touch and pressure-sensitive drawing tablets; first templates weekly planner, Kanban, mind map, SWOT, lined page; a board sticky note is linked to a real sticky note or a separate copy, per note. Nine finer questions left for planning. | Nothing. |
| Notebook mode (with Default style) | ✅ | OneNote-like layout chosen **per notebook** next to the Default style (the Notion-like layout, the default): coloured section tabs, page list with sub-pages, pages are free infinite canvases; a single page can switch to a free canvas; shares the whiteboard engine; the AI can switch modes. | Ribbon and notebook/section/page layout are OneNote-like already. |
| Right-click menus | ✅ | Menus in sidebar items, page tabs, sticky notes, editor text ("Turn into...") and database rows; plus links and images, several items at once, keyboard hints, Copy link to page, and the empty sidebar space (on trial). Every menu customizable in Settings. No "Ask AI about this". | Menus behind "..." buttons and the Menu key; no mouse right-click. |
| Tray menu | ✅ | Open, New page, New sticky note, Calculator, Quick search, Lock now, Show/hide all stickies, Close stickies, Quit. | Open Notaya, Close sticky notes, Quit. |
| Dashboards (Power BI-style, inside Notaya) | ✅ | Three steps: dashboard pages with live tiles from Notaya databases (never stale); filters, Reset and click-through; named reusable calculations. The AI can build them. Details left to planning. | Bar, line, donut and number charts, one per view. |
| Guide, tour, About | ✅ | Guide: an in-app manual (feature list with images and icons) plus an outside copy kept in step; Tour: light, skippable, never forced; About: credits for both partners, open-source references and licences, Notaya's own licence once both partners choose. | About lists open-source credits only. The outside manual's first edition exists (2026-11-07, PR #63); the in-app manual and the tour are not built. |

Set aside or rejected by Ahmed: "Ask AI about this" in any menu; Favorites and New notebook on the app bar; pins and strings on the Windows desktop; moving databases between pages as part of the sidebar change; "everything into v1.6" and "v1.6 stays one release".

## 12. What is still open

**For Mohamed**

- **A34 / STATUS question 27:** confirm or change Claude's reading of how two partners work together. [Needs both]
- **Which of Ahmed's method rules you want on your side:** PBS for every task, the full 13-section entries, the ledger, and A38's partner-and-machine labels. [Ahmed's side]
- **Licence and public repository:** not decided (STATUS question 11). [Needs both]
- **Review the AI CLI design** before it is built, since it changes a security promise.
- Earlier open items still standing: STATUS questions 9, 10, 13, 14, 20, 23 and 25 (backlog and acceptance items, mostly yours). Also the postponed "Notaya as offline Notion" gap list on draft PR #45, which Ahmed's Notion north star will use.

**For Ahmed's side**

- Review the AI CLI action list when the thread delivers it; the AI CLI's security design (A41) comes before any build.
- Phase 2 (may run now): the Notion parity study (what Ahmed uses Notion for, what Notaya lacks, migration and backup both ways), a staging plan across versions, the Notion hub; then building with Codex on Ahmed's go per feature.
- Details left for planning (STATUS question 29): nine Whiteboard questions, ten dashboard questions, whether the plain word "Whiteboard" is acceptable as a screen name.
- Not yet run in the real exe: Start-with-Windows; `cargo clippy` for MSVC on the V1.5.1 commit.

**Dated update (2026-11-08 00:35 UTC).** Items for Ahmed's side now open: the first-launch copy wording (G3), the seven-notebook proposal for the Notion migration, whether cloud threads may get Wine and MinGW to run the Rust tests, the library approvals at their versions (1.7 and 1.9), and Ahmed's review of the whiteboard and dashboard questions. They are in STATUS question 30 and in session entry 46 under Open Questions. For Mohamed nothing new is asked, but the whole Notion track (below) is Ahmed's side's own work. [Ahmed's side]

## 13. Rules for both sides [Shared rule]

These come from `CLAUDE.md` and apply to every Claude and to Codex.

- **Stable releases only.** No alpha, beta, rc, preview or nightly packages. npm versions are exact; Rust versions are locked.
- **Never edit** `docs/plan/approved-plan-2026-10-22.md`.
- **Never commit vault data or secrets:** `.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports, `.env`. Check `git status` before every commit.
- **Keep keys and secrets in the Rust core.** The interface receives only what it displays.
- **Never test on anyone's live window or real vault.** Use a throwaway vault in a separate process with `NOTAYA_HOME` (or `CIPHERLEAF_HOME`) pointing at the test folder. Close any debug port afterwards and confirm nothing listens (FI-01). Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set. Nobody touches the other partner's notes, or the `E:` drive.
- **Quote times, do not estimate them.** Take them from message timestamps or the system clock (UTC) and say which. Otherwise write "not recorded".
- **Report what did not run as "not run",** never as passed. A result applies only to the exact commit it ran on.
- **Every code PR** gets a security review (the built-in security-review skill, plus the Security Guidance plugin where installed), and the PR says what each check found and what did not run.
- Work on your own branch (`<github-login>/<topic>`, or the name the cloud thread gets), open a **draft** PR, merge only with "Create a merge commit". Never push to someone else's branch or to `main`.
- Do not force-stop KeePassXC if it ever appears on a machine (it protects itself); close it politely.

## 14. Stopping, and picking up Ahmed's work

**When Ahmed's cycle ends, his side should leave:**

- [ ] Everything committed and pushed (a WIP branch is fine), with a draft PR titled `WIP:` if unfinished.
- [ ] A resume note `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md` (done / checked / not run / not finished / decisions by a person / state outside the repo / next permitted action).
- [ ] The claims row updated; `docs/STATUS.md` "Exact next permitted action" brought up to date.
- [ ] A full session entry and its index row; the ledger refreshed; the Notion hub updated.
- [ ] Ahmed's list of open questions, numbered, and which features are discussed, staged or built.

**When you pick it up:**

1. Read the resume note, the PR description and the last commits. Treat claims in them as **claims to re-check on the exact commit**, not facts.
2. Do not push to Ahmed's branches without his say-so. Continue on a new branch from theirs and say "continues #N" in the draft PR.
3. Never rewrite history on a branch you did not create.
4. If a note says "stopped" or "do not merge yet", do not resume without the partner who set that pause.

## 15. Sources and limits

- **Read for the update of 23:50 UTC:** the whole project chat from 21:57 to 23:39 UTC, the model-choice guide, the headings and agreed sections of the dashboards and Whiteboard studies, and the records edited in PR #61 (A40 to A43, entry 43). The Notion parity study (about 23:41 UTC) was not read in full.
- **Read for the first version of this guide:** `CLAUDE.md`, `AGENTS.md`, `docs/STATUS.md` (current state, open questions, next action, where-things-live), A37 and A38 with their dated notes and the rows of 2026-11-07, the top of `docs/sessions.md` (index and entry 41 including Decisions Made and Next Steps), `docs/ledger/README.md` and the first ledger blocks and CSV headers, `docs/handoff/collaboration-guide.md` and `claims.md`, and Ahmed's project memory and shared notes.
- **Not re-read in full for this guide:** the older sessions, the 49+ execution reports, `docs/security-design.md`, `docs/acceptance.md`, and the plan records A9 to A35.
- **Nothing was run or tested.** The "exists today" column comes from Ahmed's inventory of the code at `9474511` and was not re-checked.
- **Freshness:** this is a living document (A39) and is updated as Ahmed's side decides things. Its "last updated" line is at the top. Between updates, check the newest `sessions.md` entry and STATUS before relying on any line here.
- **Read for the update of 2026-11-08 00:35 UTC:** the project chat from 20:51 to 00:24 UTC, the project memory, the repository records at `main` = `c2ead6e`, and the `get_session` records of eight threads. The Notion survey, the studies, the 1.6 spec and the formula file were not re-read in full for this update; their results are quoted from the threads' reports.

## 16. What happened from 23:39 to 00:26 UTC (the checkpoint) [Ahmed's side]

The full account, with Ahmed's questions 1 to 38 and his answers, is `docs/sessions.md` entry 46 and A45. The short version for Mohamed:

- **The Notion track.** Ahmed's long-term goal is that his whole Notion can be copied into Notaya without gaps, so Notaya is built to be a full offline match. A full read-only survey was done on 2026-11-08 (77 top-level items, about 270 Discord pages, a German course in three copies, a finance system with many formulas; counts are the survey's). **Notion is never changed** (a scratch page excepted). Hubs that must work, home pages included: Projects and Ideas registry, Discord notebook, Reading knowledge control center, Personal Financial System, AI Project Bootstrap System, FFmpeg Encoder, Learn German with Mariam (Arabic), Freelancing Arc. **Formulas must be exact:** all formula text was read through the browser (read-only) and stored in the project's private notes with a checksum; it is **not in the repository** because it is finance data. **The backup is manual and driven by an AI** through the AI CLI, never automatic. Seven notebooks as shelves for whole hubs are a proposal. The first real test is a three-page trial after 1.8. [Ahmed's side]
- **Libraries (studies, not approvals).** Pen lines (`perfect-freehand`), a global-shortcut plugin, a Greek and Cyrillic font subset, a colour emoji picture pack for PDFs, `rust_xlsxwriter` and `calamine` for Excel, and the `rmcp` kit for the bridge. All permissive licences; one whiteboard kit (tldraw) was ruled out. Each needs Ahmed's words at its version; the 1.6 spec adds no new crates except extra `windows` features. Study: `docs/plan/2026-11-07-dependency-licence-study.md`. [Ahmed's side]
- **Roles and quota.** Codex builds; Claude briefs, inspects, merges and records. Claude up to 70% of the weekly quota, Codex up to 100% (section 8). [Ahmed's side]
- **In flight at 00:26 UTC.** Codex on 1.6 lanes A, B, C1, D and E on Ahmed's PC (`Gaming-PC`), overnight, on throwaway data only. Nothing built is accepted.
- **What Mohamed may want to look at:** the 1.6 spec (security notes, the vault change at first launch, the new bin for notebooks, sections and sticky notes), and the usage ledger block of 00:26 UTC. [Needs both for the security promises]

## 17. The 1.6 build: where it stands and how to pick it up [Ahmed's side]

Written 2026-11-08 about 07:18 UTC for Mohamed, so that whoever has quota left can continue at any point. Records that own the facts: [STATUS](../STATUS.md) (exact next permitted action), [the Codex briefs](../plan/2026-11-08-v1-6-codex-briefs.md), [EX-2026-11-08-55](../execution/2026-11-08-v1-6-build-start.md) section 5 (heartbeat log), [the 1.6 spec](../plan/2026-11-08-v1-6-spec.md).

- **Method.** Codex (`gpt-6.1-sol`, xhigh) builds each lane in its own git worktree on Ahmed's PC (`D:\Notaya-lanes\<lane>`), on branch `AE0101/v1-6-<lane>`. Claude briefs, inspects each diff, opens the pull request, merges with a merge commit and writes the records. Claude writes no feature code. Codex's sandbox cannot run the Rust tests or write git, so a PC heartbeat runs the checks on the exact pushed commit, makes local checkpoint commits and pushes finished lanes. Everything is on throwaway data.
- **Merged into `main` (2026-11-08):** lane C1, the action core (PR #74; 588 Rust tests pass, 0 fail); lane B, databases inside a page appear in exports (PR #75; 529 pass); lane D, the companion program `notaya-bridge.exe` (PR #76; bridge crate 16 tests pass, 2 real-pipe tests ignored; inert until the AI server exists).
- **Update 2026-11-09 12:00 UTC (A51, A52, live check, silent heartbeat):** `main` is `cce1bfd`. Merged since 07:18: lane P1, speed stage 1 (PR #112, golden accuracy gate, numbers in `docs/execution/2026-11-09-wp16-stage1-numbers.md`), A51 (Ahmed 08:22:33 UTC: a Windows-owned confirmation for deleting forever, emptying the recycle bin, turning AI on and AI pairing; keyfiles on network shares refused; lanes S-fix1 and S-fix2 build it, not built yet), the live AI check passed on Ahmed's own hands (09:12 UTC, verified not accepted), the smoke-script fix, and **A52** (Ahmed 10:42:50 UTC: Notion import with his real exports, **WP-19 / card 1.6-19**: his 2026-11-03 workspace ZIP of about 1 GB and per-section exports show databases not imported and loose pages from a partial import; lane N1 investigates read-only, N2 and N3 fix; the exports stay on his PC). The security pass has areas 1 and 2 done (findings on `AE0101/v1-6-s`: area 1 four issues and one question, area 2 three Medium issues; fix lanes S-fix1 to S-fix3 queued). Defect X-1 (blank window at Start with Windows, reported by Mohamed) has a first fix that did not work; run 2 is queued. The heartbeat now runs hidden (a wscript launcher; the PC-only files are in `D:\Notaya-lanes\heartbeat`, backup of the old task in `task-backup`; Ahmed ran the registration himself). Codex resets at 12:38:07 UTC (weekly at 2026-11-16 07:38:07 UTC); one banked Codex reset remains and only Ahmed applies it. Ahmed asked for lean replies in the project thread from 11:52 UTC until he says detailed mode. **If you continue without Ahmed's Codex:** read `docs/STATUS.md` (As of line and Continuation), then `docs/sessions.md` entry 53, then the briefs; the order of the remaining lanes is in the STATUS line.
- **Update 07:18 UTC (A50):** Ahmed lifted the Claude 70% weekly pause line for 1.6 (up to 100% to finish 1.6 fully safe, documented and prepared), asked for task-based model choice (Sonnet 5.5 high preferred, cheaper where enough) and said he applies Codex banked resets himself without notice. If Ahmed's Claude week runs out, Mohamed's Claude continues from these records.
- **Update 07:10 UTC (A49):** P8 (visible rows for List, Gallery, Board, Timeline) is now part of 1.6 with P1, P6, P7; two more polish items were added: a **full security pass** (WP-17, using `docs/handoff/codex-security-review-guide.md`; the independent review of C5 and the bridge, with token zeroization and the pairing-model docs still open) and a **whole-app polish and technical-debt sweep** (WP-18). All must finish before the whole-app test (A47) closes. If you continue without Codex, the guide is also the checklist for your own Claude review.
- **Update 06:56 UTC:** all 1.6 lanes are merged (C1 to C5, A, B, D, E and the fix lanes); what remains is the whole-app test pass (A47, `docs/plan/2026-11-09-whole-app-test-plan.md` on branch `AE0101/v1-6-w`), the large-database speed lanes P1, P6, P7 (A48, WP-16: plan in `docs/plan/2026-11-09-large-database-performance-plan.md`, measuring tools in `tools/db-bench` and `dev-mock/perf-table-check.cjs`), the independent security review of C5 and the bridge, release notes. Ahmed's live test of the AI window is pending his own hands (steps on his PC in `D:\Notaya-lanes\out\live-run-steps.txt`). Codex banked resets are applied only by Ahmed himself.
- **Update 17:11 UTC:** lanes AG (PR #93), C4d (PR #92) and C3, history and activity log (PR #97) are merged, after C2 (PR #88). A fresh review of C2 found 1 high and 5 medium issues; lane C2d (`AE0101/v1-6-c2d`, `6909482`) fixes them and a double-save bug and is pushed but needs a merge with C3 (conflicts in `changes.rs`, `state.rs`, `PageEditor.tsx`). C5, the AI server, is half built (uncommitted on Ahmed's PC in `D:\Notaya-lanes\C5`). Both stopped at the 5%-left line and resume at about 20:52 UTC. To continue by hand: merge main into `AE0101/v1-6-c2d`, resolve those three files, run the checks, merge; then build C5 from the newest main following spec section 7 and the C5 brief.
- **Update 12:03 UTC:** lane C2 (change signals, edit leases, recycle bin) is merged (PR #88, `2ffaa2c`; 631 tests pass). Not merged: C4d (browser check `block-ids-check` still failing, `fe78527`), AG (screenshots script, `275d393`). Codex's 5-hour window ran out at 12:01 UTC and resets 15:48 UTC; the heartbeat restarts AG, C3 and C4d by itself. Ahmed's rule (11:48:49 UTC): use the window until 5% is left, then stop cleanly. Lane order after the reset: AG fix, C3, C4d fix run 2, then C5, lane E (PR #70), the independent security review, the PC round, release notes. The PC meter now reads Claude and Codex exactly (Claude weekly may read about 1 point low).
- **Update 11:07 UTC:** lanes C4 (block ids, block edits, database small saves; PR #85) and AF (fixes from the independent review of lane A; PR #86) are merged. C2 (recycle bin, change signals) is running; then C3 (history, activity log), C5 (AI server). The heartbeat is a permanent scheduled task on Ahmed's PC (`Notaya heartbeat`, every 10 minutes, script `D:\Notaya-lanes\heartbeat\heartbeat.ps1`), logging Codex and Claude meters to `docs/ledger/quota-readings.csv`.
- **Update 08:05 UTC:** lane A (small fixes WP-08 to WP-14, including 'reopen where you left off') is now merged (PR #79, `2e49b7b`; all PC checks passed). Lane C4b (block ids and block edits, branch `AE0101/v1-6-c4b`) is built but its Rust tests failed (exit 101); Codex is fixing it. C2 starts from `main` `2e49b7b`.
- **Who continues (Ahmed, 08:57:27 UTC):** Mohamed has **Claude (Pro) only and no Codex**. If Ahmed's Codex and Claude quotas end, Mohamed's Claude continues the development (Claude then also writes code, within its weekly limit), so briefs and records must stay complete. Ahmed's Codex banked weekly resets are used only after the whole weekly limit is spent, and never automatically.
- **Built but not merged:** lane E, the AI screens ([PR #70](https://github.com/CipherVaults/Notaya/pull/70), held until the AI server exists, otherwise the real app would show a broken AI panel); lane C4b (failing tests, see above).
- **Next, in order:** A (conflict fix, then merge), then C2 (change signals, revisions, leases, recycle bin for everything), C4 (block ids and block edits) in parallel, then C3 (history and activity log), then C5 (the AI server: pipe, pairing, dispatcher, confirmations), then integration, the independent security review, the PC round and release notes. Widened write areas that were agreed: C4 may add `mod blocks;` to `lib.rs`; C2 may touch `lib.rs` (registration lines), `src/lib/types.ts` and `commands/tools.rs`; A may touch `DatabasePage.tsx` and `DatabaseView.tsx`.
- **How to resume on your side.** Read `docs/STATUS.md` (Continuation), then the briefs, then `git fetch --all --prune` and look at `AE0101/v1-6-*` branches and open pull requests. Treat anything in the PC worktrees as unverified until the checks run on the pushed commit. Codex's limits are the pacing: first window used 15% of the weekly limit, second window 22% in total (the ledger holds the readings); the 5-hour window resets about every 5 hours.
- **Known gaps to close before 1.6 is complete:** the bridge's modern `subscriptions/listen` and `server/discover` details; lane E's Playwright check script (three timing bugs; the patched copy passes 40 checks); the title-bar marker (wired when A is merged); a security review of lanes C5 and D together; the real pipe, installer and registration checks on the PC; the one-time local safety copy at first launch (gate G3, confirmed by Ahmed 00:46:05 UTC, belongs to the vault lane).

## 18. Ahmed's side moves to ChatGPT Pro and Codex (2026-11-10) [Ahmed's side]

Ahmed's Claude weekly quota is nearly used up (about 91% on 2026-11-10 03:21 UTC). On 03:32:47 UTC he asked for a full migration of the lead coordinator and reviewer role to **ChatGPT Pro plus the Codex desktop app** ([A55](../plan/approval-and-amendments.md); it takes effect on his "handover accepted" in ChatGPT). The package is [`migration-to-chatgpt/`](migration-to-chatgpt/README.md). **For Mohamed:** nothing changes on your side. You keep Claude Pro (no Codex). If you must carry on, paste block D from [`migration-to-chatgpt/02-start-prompts.md`](migration-to-chatgpt/02-start-prompts.md) as your first message; you merge, release and accept on your side, and making the repository public or adding a licence still needs both of you. Ahmed may ask you for a second opinion on a risky diff (the one non-OpenAI review), only with your agreement. Section 17 stays the 1.6 state as of 2026-11-09 12:00 UTC; the current state is in the migration guide, section 6, and in `docs/STATUS.md`.

## Change log

Newest first. Each line: when (UTC), what changed, which side's decision it records.

| When (UTC) | Change | Whose decision |
|---|---|---|
| 2026-11-10 04:50 | New section 18: Ahmed's side moves to ChatGPT Pro and Codex (A55); what it means for Mohamed. Repo copy only; the published page was **not** republished (the HTML snapshot in `docs/artifacts/snapshots-2026-11-10/` is the 2026-11-08 content). | Ahmed's side (03:32:47) |
| 2026-11-09 12:00 | Section 17 updated: P1 merged, A51 (Windows-owned confirmations), live AI check passed, A52 (Notion import with real data, WP-19), silent heartbeat, security pass areas 1 and 2, lean-mode note. Repo copy only; the published page is rebuilt by the coordinator. | Ahmed's side (08:22:33, 10:42:50, 11:33, 11:52:54) |
| 2026-11-09 07:18 | Section 17 updated: Claude 70% line lifted for 1.6, model choice by task, banked resets by Ahmed (A50). Repo copy only. | Ahmed's side |
| 2026-11-09 07:10 | Section 17 updated: P8 into 1.6, full security pass (WP-17) and whole-app polish sweep (WP-18) added (A49); new Codex security review guide. Repo copy only. | Ahmed's side |
| 2026-11-09 06:56 | Section 17 updated: all 1.6 lanes merged; whole-app pass (A47) and large-database speed lanes (A48, WP-16) remain. Repo copy only. | Ahmed's side |
| 2026-11-08 17:11 | Section 17 updated: AG, C4d, C3 merged; C2d pushed but needs a merge; C5 half built. Repo copy only. | Ahmed's side |
| 2026-11-08 12:03 | Section 17 updated: lane C2 merged (PR #88), C4d and AG held on browser-check fixes, Codex window exhausted until 15:48, stop-line rule. Repo copy only. | Ahmed's side |
| 2026-11-08 11:07 | Section 17 updated: lanes C4 and AF merged, heartbeat is a permanent task. Repo copy only. | Ahmed's side |
| 2026-11-08 08:05 | Section 17 updated: lane A merged (PR #79), C4b failing tests, C2 next. Repo copy only; published page still outdated. | Ahmed's side (standing orders of 00:18:27) |
| 2026-11-08 07:18 | 1.6 build progress: last-updated line and new section 17 (lanes merged, lanes held, next order, how to resume, known gaps). **Repo copy only: the published page is not yet updated** (a rebuild is queued for the next milestone, A39). Records: PRs #74 to #76, EX-2026-11-08-55 section 5. | Ahmed's side (00:15:27, 00:18:27, 00:30:19, 01:09:37) |
| 2026-11-08 00:35 | Checkpoint after Phase 2 (PR #69): last-updated line; dated updates in sections 8 (quota and roles), 9 (Phase 2 finished, 1.6 go), 10 (AI CLI decisions), 12 (open items); new section 16 (Notion track, libraries, roles, what is in flight). Records: A45, entry 46, EX-2026-11-08-56. | Ahmed's side (23:49:01 to 00:24:41) |
| 2026-11-07 23:56 | Added the Notaya Manual, first edition (for 1.5.1): a row in "Where each record lives" (section 6) and a note in the Guide row of the feature table (section 11). Source in `docs/manual/`, published page linked, PR #63. Versions after 1.9 are numbered 2.0 to 2.5 in the manual, as the coordinator directed. | Ahmed's side (agreed GUIDE feature, 23:04; "push ahead", 23:39:31) |
| 2026-11-07 23:52 | Added a pointer to the Phase 2 version plan (EX-2026-11-07-50, draft PR #62) in section 9, at the version-planning thread's request. | Ahmed's side (23:39:31); plan not yet reviewed by Ahmed |
| 2026-11-07 23:50 | Phase 1 decisions: sections 1, 2, 6, 8 (model choice, credit), 9 (Phase 1 done, Phase 2 may start), 10 (AI CLI safety direction), 11 (right-click, tray, dashboards, guide/tour/About, banner, Whiteboard, Notebook mode and Default style), 12 and 15 updated; new rules: no trademarked names, customizable menus. Records: A40 to A43, entry 43, PR #61. | Ahmed's side (22:40:55 to 23:39:31) |
| 2026-11-07 22:45 | Marked as a living document (A39); added this change log. | Ahmed's side (22:33:37) |
| 2026-11-07 22:30 | First version, from the repository at `main` `987bfc5`. | Ahmed's side (22:24:27) |
