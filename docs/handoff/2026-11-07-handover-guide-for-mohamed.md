# Notaya handover guide for Mohamed (2026-11-07)

**Who and where:** written for **Mohamed Sayed (`AbyssWatcher2`)** and his Claude, from **Ahmed's side (`AE0101`)**, by a Claude cloud thread in Ahmed's Claude project "Notaya" (machine: cloud). Written 2026-11-07, about 22:30 UTC (container clock), from the repository at `main` = `987bfc5` (merge of PR #56). A published page with the same content (private to Ahmed unless he shares it): <https://claude.ai/artifact/UhKn7cS38rp7Sq89YfGLyp>. This Markdown copy exists because the repository is the only memory the two Claudes share.

**Living document (Ahmed, 2026-11-07 22:33:37 UTC, [A39](../plan/approval-and-amendments.md)).** This guide is kept up to date at every milestone or decision on Ahmed's side, in both copies (this file and the published page), so that when Ahmed's cycle ends Mohamed has something solid with few or no gaps. **Last updated: 2026-11-07 22:45 UTC.** See the change log at the end.

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
- **Order of work:** talk through every feature, then plan and stage across versions, then build. Before discussing a feature, check what Notaya already has. Nothing is built yet. [Ahmed's side]
- **The AI CLI is the foundation** of the next work: Claude or Codex will drive Notaya from outside. Every new feature gets named AI actions from its first design. [Ahmed's side]
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
3. `docs/plan/approval-and-amendments.md`: who said what, with exact times; amendments A1 to A38.
4. `docs/handoff/collaboration-guide.md` and `claims.md`: how two people share one repository, and who has claimed what.
5. The newest entry of `docs/sessions.md` (entry 41, 2026-11-07) and `docs/ledger/README.md`.

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
| What is authorized, who said it, when? | `docs/plan/approval-and-amendments.md` (rows with exact UTC times; amendments A1 to A38) |
| What happened, in order, and why did choices change? | `docs/sessions.md` (one file, newest on top, with the "Sessions at a glance" index) |
| What did a piece of work do, and what was checked? | `docs/execution/` and its index `docs/execution/README.md` |
| Which criteria are met, and who accepted? | `docs/acceptance.md` |
| How much Claude and Codex usage did a session take? | `docs/ledger/` (`README.md`, `usage-ledger.md` and three CSV files) |
| Who is doing what, and which IDs are taken? | `docs/handoff/claims.md` (the live board is GitHub: open PRs and branches) |
| How to pause and pick up work | `docs/handoff/` resume notes and `collaboration-guide.md` |
| What went wrong or nearly did | `docs/failure-intelligence.md` |
| What does Notaya promise about security? | `docs/security-design.md` (read before touching crypto, storage, IPC commands, CSP, clipboard or auto-lock) |
| What does Notaya already do? | The code, and the full inventory `notes/2026-11-07-notaya-full-feature-inventory.md` in Ahmed's project files (it is not in the repository) |
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

**Limits.** Quotas are account-wide, so the ledger cannot separate Notaya's share inside one reading. **Your usage runs on your own account and is not visible from Ahmed's side:** if you want it tracked, your Claude adds its own rows (`partner` = `Mohamed (AbyssWatcher2)`). No pacing cap is set: Ahmed said to watch the 5-hour and weekly limits of Claude and Codex so work is not cut off mid-task (STATUS question 28). There is no ledger dashboard yet; the ledger today is these files.

## 8. How Ahmed's side works

Ahmed's operating model (A37). Use any of it that suits you. [Ahmed's side]

| Role | What it does |
|---|---|
| Ahmed | Director and partner. Decides the product and gives the go per feature. Explanations to him are in plain words, with a short summary when something finishes, and open questions gathered into one numbered list. Unrequested improvements are welcome but must be listed clearly as additions. |
| Claude coordinator ("mother thread") | Leads and plans. Chooses model and effort for each task, starts and steers threads and Codex, inspects every Codex diff for Ahmed's vision, coherence and correctness, runs the security review, keeps the records and the ledger, and commits, pushes and merges. |
| Claude threads | Cloud (or Remote Control on a PC) sessions for one job each: reading, studying, records, planning. Opus 5.5 or Sonnet 5.5 at high effort. Used first while the 2026-11-07 cloud credit lasts. |
| Codex (`gpt-6.1-sol`, reasoning `xhigh`) | The developer hand that writes the bulk of the code and tests from a bounded brief. It does not merge, release or change records of authority. Its diff is reviewed by Claude, because the agent that wrote a diff is never its only reviewer. |

**Where things run**

- The coordinator decides PC or cloud for each task. Cloud sessions cannot run the Windows exe; checks there are `tsc`, `npm run build`, `npm audit` and Rust tests under Wine. Whatever cannot run is reported as **not run**.
- The real app is tested only through a Remote Control session on the PC, on throwaway data.
- **Ahmed's PC:** repository `D:\Notaya` (the path must not contain an apostrophe), tests in `D:\Notaya Tests`, machine name `Gaming-PC`. **Yours:** `E:\Project\Notaya\Notaya`, tests in `E:\Project\Notaya\Notaya Tests Excluded`.
- After a merge, the PC copy is fast-forwarded (`git pull --ff-only` on `main`) and checked to equal `origin/main`. Never `git add -A` in a PC copy.

## 9. Three phases, and know what exists first [Ahmed's side]

1. **Discuss every feature** until Ahmed and Claude agree on what it means. No version is assigned yet. (Ahmed, 21:14:10: "there is a huge difference between me laying down the points like in a text way to remind myself what we need, and ... what we discuss".) *This is where we are now.*
2. **Plan and stage** across versions 1.6, 1.7, 1.8 and on, by what is doable. Ahmed (20:41:50): "we don't have to push everything and shove it into 1.6". The coordinator will bring a staging plan and a list of Notion features Notaya lacks; Ahmed decides.
3. **Build,** with Codex as the developer hand and the coordinator inspecting every diff, only on Ahmed's go per feature or version.

**Know what exists first (Ahmed, 21:00:14).** Before discussing, planning, designing or building any feature, check what Notaya already has in the code and in the records. Never redesign something that exists, and never break it. The full inventory read the code at `9474511` and found, for example, that there are no mouse right-click menus anywhere, that an inline database is silently dropped from page exports (read in the code, not yet seen in the app), and that no outside entry point for an AI exists yet. Re-check on the current commit before relying on any of it.

## 10. The AI CLI foundation [Ahmed's side]

Ahmed uses Claude and Codex from outside to drive Notaya locally, in the way Claude uses Notion through MCP, but with more power and freedom, "because it's local and it's our code and it's our app".

**Agreed in principle**

- Every part of Notaya is reachable. The AI sees everything; **locked notes only after they are unlocked**.
- The AI may create, edit and delete. **Deletes need confirmation.**
- An **activity log** records everything the AI does.
- **Every feature is designed with named AI actions** from its first design, including moving and copying pages and databases between places.
- **Long-term north star (direction only, not now):** Notaya eventually matches Notion closely enough that Ahmed's whole Notion can be pulled in as-is, as an always-on local backup.

**Not decided yet**

- The action list itself.
- How an outside program reaches Notaya and proves it is allowed.
- Whether the AI may work while the Notaya window is open, and how the window refreshes after an outside change (today the interface keeps its own copy of the data and is not refreshed from outside).
- Where a delete is confirmed, where the activity log lives, and whether an AI change can be undone.

**Security.** The AI CLI creates a new trust boundary. Under `CLAUDE.md`, changing security behavior needs the partner's own words, and `docs/security-design.md` must be updated with any new promise. Keys and secrets stay in the Rust core. Because this touches shared code and shared promises, Ahmed's design should be shown to Mohamed before it is built. [Needs both]

## 11. Feature decisions so far

All of these are **Ahmed's** decisions from the 2026-11-07 discussion (full table with his reasons: `docs/sessions.md`, entry 41, section "Decisions Made"). **None of them is permission to build.** Mohamed has not yet reviewed them. ✅ agreed, 🟡 partly discussed, ⬜ not discussed.

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
| Home banner per time of day | 🟡 | The person picks a picture for each part of the day and the picks stay as defaults. How they pick is not decided. | Built-in daily gradient with a greeting. |
| Whiteboard | 🟡 | Its own section like Sticky notes, Microsoft Whiteboard style, with starter templates and changeable pens. Details to come. | Nothing. |
| OneNote mode | 🟡 | Named only. | Ribbon and notebook/section/page layout are already OneNote-like. |
| Right-click menus | 🟡 | Proposal pending: right-click opens the matching menu everywhere (sidebar items, tabs, sticky notes, editor text, database rows). Waiting for Ahmed's answer. | Menus exist behind "..." buttons and the Menu key; no mouse right-click. |
| Tray menu | ⬜ | Idea only: right-click the tray to make a sticky note, open the calculator, jump to parts of Notaya. | Open Notaya, Close sticky notes, Quit. |
| Power BI-like charts, slicers, dashboards | ⬜ | Idea only. Ahmed called them "very complicated". | Bar, line, donut and number charts, one per view. |
| Guide, app tour, About credits | ⬜ | Idea only: a guide in Notaya's style, an optional tour, names in About (Mohamed Sayed, Ahmed Emara, Claude, ChatGPT). | About lists open-source credits only. |

Set aside or rejected by Ahmed: Favorites and New notebook on the app bar; pins and strings on the Windows desktop; moving databases between pages as part of the sidebar change; "everything into v1.6" and "v1.6 stays one release".

## 12. What is still open

**For Mohamed**

- **A34 / STATUS question 27:** confirm or change Claude's reading of how two partners work together. [Needs both]
- **Which of Ahmed's method rules you want on your side:** PBS for every task, the full 13-section entries, the ledger, and A38's partner-and-machine labels. [Ahmed's side]
- **Licence and public repository:** not decided (STATUS question 11). [Needs both]
- **Review the AI CLI design** before it is built, since it changes a security promise.
- Earlier open items still standing: STATUS questions 9, 10, 13, 14, 20, 23 and 25 (backlog and acceptance items, mostly yours). Also the postponed "Notaya as offline Notion" gap list on draft PR #45, which Ahmed's Notion north star will use.

**For Ahmed's side**

- Answer the right-click proposal; discuss the tray menu, dashboards, guide, tour and About credits.
- Finish the partly discussed items: AI CLI action list and safety design, banner picker, Whiteboard, OneNote mode.
- Then a staging plan, then planning in a fresh thread, then building with Codex on Ahmed's go.
- Not yet run in the real exe: Start-with-Windows; `cargo clippy` for MSVC on the V1.5.1 commit.

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

- **Read for this guide:** `CLAUDE.md`, `AGENTS.md`, `docs/STATUS.md` (current state, open questions, next action, where-things-live), A37 and A38 with their dated notes and the rows of 2026-11-07, the top of `docs/sessions.md` (index and entry 41 including Decisions Made and Next Steps), `docs/ledger/README.md` and the first ledger blocks and CSV headers, `docs/handoff/collaboration-guide.md` and `claims.md`, and Ahmed's project memory and shared notes.
- **Not re-read in full for this guide:** the older sessions, the 49+ execution reports, `docs/security-design.md`, `docs/acceptance.md`, and the plan records A9 to A35.
- **Nothing was run or tested.** The "exists today" column comes from Ahmed's inventory of the code at `9474511` and was not re-checked.
- **Freshness:** this is a living document (A39) and is updated as Ahmed's side decides things. Its "last updated" line is at the top. Between updates, check the newest `sessions.md` entry and STATUS before relying on any line here.

## Change log

Newest first. Each line: when (UTC), what changed, which side's decision it records.

| When (UTC) | Change | Whose decision |
|---|---|---|
| 2026-11-07 22:45 | Marked as a living document (A39); added this change log. | Ahmed's side (22:33:37) |
| 2026-11-07 22:30 | First version, from the repository at `main` `987bfc5`. | Ahmed's side (22:24:27) |
