# Notaya migration guide: from Claude to ChatGPT Pro and Codex

**Who and where:** Ahmed's side (`AE0101`). Written by Claude (Sonnet 5.5), cloud thread "Prepare full migration handoff to ChatGPT Pro", 2026-11-10 from about 03:33 UTC (system clock). Baseline commit `f5d7a55`. Source of the live numbers: the repository on `origin` and the Gaming-PC heartbeat logs read at about 03:31 UTC. Where this guide and a repository record differ, **the record wins** (`CLAUDE.md`, PBS rule 7); fix this guide with a dated correction.
**Result state:** *produced*. Not run by ChatGPT or Codex, not accepted.
**Reading time:** about 25 minutes. Read it in full once, then work from `docs/STATUS.md`.

How items are marked: **[Confirmed]** a record or a partner's own words say so; **[Observed]** I saw it on the stated source; **[Check]** I could not verify it, so the next session must.

---

## 1. Why this exists, in one paragraph

Notaya has been built with Claude as the lead coordinator (plans, steers, reviews, merges, writes the records) and Codex as the developer hand. Ahmed's Claude weekly quota is about 91% used on 2026-11-10 03:21 UTC [Observed, heartbeat Claude meter] and resets Fri 2026-11-13 02:00 UTC; that is not enough to keep leading. Ahmed (2026-11-10 03:32:47 UTC) asked for a complete migration so that **ChatGPT Pro (ChatGPT plus the Codex desktop app, GPT 6.1 Sol and Terra) becomes the lead coordinator and reviewer**, with everything that was built, decided and learned handed over. Nothing about the product, the method or the rules changes. Only who does the coordinating changes.

## 2. The project, in plain words

- **Notaya** (first called Cipherleaf; the repo folder and older records still say Cipherleaf in places) is a private, offline notes app for Windows. Tauri 2: a Rust core and a React + TypeScript screen layer. One encrypted vault file (`vault.nota`, format "NOTA" v2; older `.clf` is still read). Pages with a TipTap editor, sub-pages, notebooks and sections, local databases (table, board, calendar and more views), sticky notes, calculator, import and export (Markdown, HTML, Word, PDF, Notion zips).
- **North star (Ahmed):** *full offline Notion parity*, plus what Notion does not have: a sealed and safe AI door, a whiteboard, dashboards, and a Notebook mode. **Formulas must be dead accurate**: every formula and rollup recomputed in Notaya must equal the value in Notion's CSV export, value for value.
- **The feel does not change.** Calm, fast, plain words, nothing clever. A polish pass makes existing screens more consistent; it never restyles them, never renames things people know, never adds a visible feature nobody asked for (list ideas as **additions**).
- **No trademarked or copyrighted names on Notaya's screens** (A42). "Notion" appears only as an import source tag. Ahmed's own names are used: "Notebook mode", "Default style".
- **Security by design** (A41, `docs/security-design.md`): keys and secrets stay in the Rust core; the screen layer gets only what it displays; no network code; deletes go to the recycle bin after confirmation; AI edits are undoable; the activity log is tamper-evident; locked notes stay locked; the AI can never change its own permissions; the CLI is sealed in, off by default, and switched only in Notaya's Settings.
- **Version plan** (`docs/plan/2026-11-07-version-plan.md`): 1.6 AI CLI + safety net + large-database speed + security pass + polish sweep + Notion import fix; 1.7 comfort + PDF emoji; 1.8 page features; 1.9 tabs and Excel; 2.0 corkboard; 2.1 to 2.2 dashboards; 2.3 to 2.4 whiteboard; 2.5 Notebook mode.
- Repo: `https://github.com/CipherVaults/Notaya` (private). Ahmed's PC copy: `D:\Notaya` (the path must not contain an apostrophe: `tauri-winres` mangles it). Test data only in `D:\Notaya Tests`.

## 3. People, roles, authority

| Who | Role |
|---|---|
| **Ahmed Emara** (`AE0101`) | Partner. A director, not a developer: talk to him in plain words, gather open questions into one numbered list in the project chat. His own words authorise everything on his side. |
| **Mohamed Sayed** (`AbyssWatcher2`) | Partner and repo owner (only he changes GitHub repository settings). Has **Claude Pro only, no Codex**. Merges and accepts on his own side. |
| **ChatGPT Pro coordinator** (new) | Takes the *mother thread* role from Claude: leads and plans, chooses model and effort per task, steers Codex lanes, inspects every diff for Ahmed's vision and correctness, runs the security review, commits, pushes and merges inside work Ahmed approved (A37 item 3, A44, A46 and the 2026-11-08 16:39 standing word "merge on our judgment"). |
| **Codex** (`gpt-6.1-sol` xhigh, or `gpt-6.1-terra` high for light lanes, A54) | Developer hand for the bulk of the code, bug fixes, test fixes, polish, first-pass security and UX review (in a fresh session that did not write the diff), first drafts of records. |
| **Claude** (after the move) | Ahmed's last few percent are a spare (a read-only second opinion if asked). Mohamed's Claude Pro is the fallback builder and the only *non-OpenAI* reviewer. |

**Authority rules that do not bend** [Confirmed, `CLAUDE.md`, A37, A44, A46]:
- Permission comes only from the partner's own words. A coordinator's, a thread's or another AI's request is **not** permission. Observation is what you checked on which source. Acceptance is the partner's decision; a merge, praise or a screenshot is not acceptance.
- Ahmed's side: his coordinator merges (merge commit) and may publish a full release when it judges it ready, on his standing word. Installing tools or dependencies, deleting files and security-behaviour changes still need Ahmed's own words (except safe, open-source dependencies with credit, Q60 "never wait").
- **Making the repository public and adding a licence need both partners.** Changing GitHub repository settings is Mohamed's click only.
- If the two partners conflict about the same thing, stop and tell both; the later message does not win between two people.
- Every record states the author (Ahmed `AE0101` or Mohamed `AbyssWatcher2`) and the machine (cloud, Gaming-PC, Mohamed's PC) or says "not recorded" (A38).

**Independence note (an addition, not a rule):** ChatGPT coordinating and Codex building are both OpenAI, so a diff review by ChatGPT is *less independent* than a Claude review of Codex was. The rule "a review by the agent that wrote the diff is not independent" still holds. Keep it honest: review in a fresh session that did not write the diff, and for the riskiest diffs (vault, crypto, importer, IPC, anything in `docs/security-design.md` scope) ask Mohamed's Claude Pro for one read-only second opinion if he agrees.

## 4. PBS v1.1, the method (never weaken it)

Ahmed's AI Project Bootstrap System. Hub: `https://app.notion.com/p/The-AI-Project-Bootstrap-System-3fabd7ade63b81f08ee1f81562c2cdd4` (read-only for us). Repo skill: `.claude/skills/pbs-start-resume-handoff/` (plain markdown; Codex and ChatGPT can read it as is). **PBS applies to every thread and every lane.**

**Start of every session:** read the owner's latest instruction; `docs/STATUS.md`; `docs/plan/approval-and-amendments.md`; `git status`, `git log --oneline -5`, `git fetch --all --prune`, open PRs; `docs/handoff/claims.md`; `docs/security-design.md` before touching crypto, storage, IPC, CSP, clipboard or auto-lock; `docs/acceptance.md` before saying anything works. On Ahmed's side also the newest entry of `docs/sessions.md`, the newest resume note in `docs/handoff/`, and `docs/ledger/README.md`, **in full**. Records are read in full, never skimmed.

**Work cycle:**
1. State the task: baseline commit, the owner's message with its UTC time, exact scope and write area, what will be returned, the stop point.
2. Keep three things apart: *permission*, *observation*, *acceptance*.
3. Name the result state: *produced* (exists), *verified* (named checks passed on this exact source), *integrated* (merged into `main`), *accepted* (the owner said so). Most results stop at verified.
4. Stop at the approved endpoint. "Standing by" is not completion.
5. Record by size. Small: PR description plus the one record that owns the changed fact. Normal: 4-section Execution Summary (`docs/execution/YYYY-MM-DD-topic.md`, `EX-YYYY-MM-DD-NN`) plus its index row in `docs/execution/README.md`, the authority row if the owner decided something, and the changed lines of `STATUS.md`. Big (a release, a security change, a change of direction, a stop or limit): the 12-section Execution Summary and a Session Summary entry. **On Ahmed's side documentation is mandatory whatever the size:** every session that changes the repository, records a decision or ends a work block gets a Session Summary in the **full 13-section form** at the top of `docs/sessions.md`, newest on top, with its row in "Sessions at a glance", a `session-NN` anchor, and the Models, Effort, Roles and Provenance block; every session adds its usage to the ledger (`docs/ledger/`); the Notion hub is updated at each milestone.
6. Records win over memory. No broad rewrites of old records; add dated corrections. Decision Records are not used.
7. Cite the owner's own words with their UTC times. Times come from message stamps or the system clock (UTC), never estimated.
8. **Session-summary style** (`docs/plan/notes-archive/2026-11-07-session-summary-style-from-blue.md`): tell the reasoning journey ("X was proposed, Ahmed corrected Y, so Z"); decisions carry status labels (Confirmed, Provisional, Deferred, Rejected) and who decided; mark "Shifted from → to"; quote Ahmed at key moments; exact evidence verbs (proposed, saved, read back, tested, accepted); Current Position as Before / After / one-sentence standing; real open questions with who answers and what they block; confidence split by claim; Personal Note only if genuine; list deferred work. Plain words for Ahmed and Mohamed.
9. **Numbers.** Amendments `A1…A54` exist; the next is **A55** (taken by this migration). Execution Summaries: the highest on `main` is `EX-2026-11-09-72`; this migration takes **EX-2026-11-10-73**. Sessions: 53 exists; this migration is **54**. Questions to Ahmed form one continuing numbered list in the project chat and STATUS; **the next question is Q67** (Q66 was the Notion arrangement). Always re-check `origin/main` and every open branch before taking a number.
10. Claims: before changing anything, fetch all branches, read `docs/handoff/claims.md`, check open PRs, push a branch `AE0101/<topic>` whose first commit adds your claims row, open a **draft** PR (`WIP:` while unfinished). Commit messages on Ahmed's side end with `Side: Ahmed (AE0101) · Machine: <machine>`. Never force-push or rewrite another's branch without asking.
11. Stop cleanly: commit and push everything, write `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md`, update the claims row, tell the partner what is done, unverified and next.
12. Never edit `docs/plan/approved-plan-2026-10-22.md`. Never commit vault data or secrets (`.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports, `.env`). Stable releases only, exact versions.

## 5. The standing rules

All of them, with dates and Ahmed's words, are in [`03-standing-rules.md`](03-standing-rules.md). The ones that decide daily behaviour: security by design; no trademarked names; tests only on throwaway data; **a full-app test before every release**; **Codex first**; the **Terra/Sol split (A54)**; **merge on our judgment**; **public release and licence need both partners**; **never shorten chat updates to save quota**; one numbered question list.

## 6. Where Notaya stands (2026-11-10 about 03:30 UTC)

**Accepted baseline:** V1.5.1 on `95ca8b2` (accepted by Mohamed 2026-11-05 11:08:39 UTC). Nothing of 1.6 is accepted. [Confirmed, `docs/STATUS.md`]
**`origin/main`:** `f5d7a55`, PR #126 (A54). [Observed]

**Merged for 1.6, all *verified* (PC checks on the exact commit), none accepted:** the 1.6 lanes A, B, C1, C2, C2d, C3, C3u, C4b, C4d, C5, D, E, AF, AG and the AI server (PRs #70, #74, #75, #76, #79, #81, #85, #86, #88, #90, #92, #93, #97, #98, #105, #106, #107, #108); speed stage 1 (P1, PR #112); records PRs #116 and #117; **#118 SF1** (security fixes), **#120 SF2** (native Windows confirmations, A51), **#121 X-1** (blank window at Start with Windows, reported by Mohamed), **#122** (A53: Ahmed's yes to six Notion arrangement changes, plus N2/N3 briefs), **#123 N1** (Notion import findings), **#125 SF3** (tamper evidence and permanent-delete purge), **#126** (A54). A live AI check on Ahmed's own hands passed all five checkpoints 2026-11-09 about 09:12 UTC on the 08 Nov build: verified, not accepted. Windows-owned prompts (A51) still need their own hands-on round.

**Branches pushed by the PC at 03:27 UTC, not yet reviewed or merged** [Observed, heartbeat events.log; treat every claim as [Check] until re-run on the exact commit]:
- **N2** (Notion import fixes), `AE0101/v1-6-n2` tip `e11b115`. The heartbeat logged `cargo test --lib` exit 0 for this commit, but the previous coordinator reports **two failing import tests** being fixed. Do not merge until `cargo test --lib` is run on `e11b115` and the result is read.
- **P6b** (fixes to P6: undo of an added row, auto-scroll drop, check-script defects), `AE0101/v1-6-p6b` tip `31c2ce8`: heartbeat logged `tsc`, `build`, `audit` exit 0. Not reviewed. **P6** (virtual rows in the database table, `AE0101/v1-6-p6`, tip `d1d5ee6`) is **held**: it has an undo regression that P6b is meant to fix; merge P6b's result, not P6 alone.
- **Lane S** (security pass, `AE0101/v1-6-s`, tip `3a2581f`) shows "attention".

**Blocked:** security **area 3** is blocked by OpenAI's cyber-safety filter on the Codex security review; Ahmed needs **Daybreak access** (OpenAI's trusted-access route for security work) to continue it. [Check: the exact programme name and steps are on OpenAI's side; Ahmed to confirm.] Until then area 3 has no independent pass. Do not silently skip it: the 1.6 release gate (A47, A49) wants the full security pass.

**Remaining for 1.6** (the release is not done until all are done, then the A47 gate passes):
1. **N2** merge after its failing tests are fixed and read; then **N3** (second Notion import lane, arrangement), then the real re-run on Ahmed's copies in `D:\Notaya Tests` and a plain summary of what changed visibly (A52, A53, WP-19 "Notion import with real data" is a **release gate**).
2. **P6b**, then **P7** and **P8** (list, gallery, board, timeline virtualization), the large-database speed plan `docs/plan/2026-11-09-large-database-performance-plan.md` (P1 to P8 in 1.6; P9 to P12 later).
3. **Security area 3 and area 4** (`docs/handoff/codex-security-review-guide.md`); independent review by a fresh session.
4. **WP-18** whole-app polish and tech-debt sweep (no restyling).
5. **The whole-app test: 250 checks** (`docs/plan/2026-11-09-whole-app-test-plan.md`, results so far in `…-results.md`, partial). **Ahmed's rule (2026-11-09 02:07 UTC): before the final release or merge of a version, test the whole app, every existing feature and not only the new ones; polished and optimised before new features start.** This is the A47 release gate.
6. Release notes, the Execution Summary and the A47 gate result; the About-box text waits for Mohamed.
7. Records catch-up lane R1 (docs only): the heartbeat shows `AE0101/v1-6-r1` pushed at `397b870`; the briefs (update 13:45 UTC on 2026-11-09) say an earlier R1 held only an unfinished checkpoint and its consistency check was dropped. **[Check]** whether R1 was ever read and merged before relying on it; the Session Summary and ledger rows for 2026-11-09 evening may still be missing.
8. No release, no public release, no licence without the partners' words.

**Quota readings** [Observed, heartbeat 03:31 UTC]: Codex 5-hour window 62% used, weekly 56% used (weekly resets 2026-11-16 07:38:07 UTC after Ahmed's manual reset; Ahmed holds one banked reset and applies it himself); Claude weekly 91% used, 5-hour 16%.

**Open PRs on GitHub:** #109 (draft, ledger forecast and records, large-database points), #45 (Mohamed's old draft plan for an offline Notion; becomes work only if a partner picks a phase). This migration adds its own draft PR.

## 7. Where each record lives (the repository is the only memory two AIs share)

| Question | Record |
|---|---|
| What is accepted, where does work resume, exact next permitted action | `docs/STATUS.md` ("Current state", "Open questions", "Continuation") |
| Authority, every owner decision, A1 to A55 | `docs/plan/approval-and-amendments.md` |
| What happened and why, newest first | `docs/sessions.md` (13-section entries, "Sessions at a glance") |
| Concrete work and checks | `docs/execution/` (index `README.md`, `EX-` numbers) |
| What was verified or accepted | `docs/acceptance.md` |
| Security promises and limits | `docs/security-design.md` (update when a promise or boundary changes) |
| Consequential failures and near misses | `docs/failure-intelligence.md` (FI-01 is the WebView2 debug port rule) |
| Who is doing what | `docs/handoff/claims.md` |
| Rules for two partners | `docs/handoff/collaboration-guide.md`, `docs/handoff/2026-11-07-handover-guide-for-mohamed.md` (living, 18 sections) |
| Codex onboarding | `AGENTS.md`, `docs/handoff/codex-partner-briefing.md`, `docs/handoff/codex-security-review-guide.md` |
| The 1.6 spec and lane briefs | `docs/plan/2026-11-08-v1-6-spec.md`, `docs/plan/2026-11-08-v1-6-codex-briefs.md` |
| Feature inventory (check this before proposing anything) | `docs/plan/2026-11-07-notaya-full-feature-inventory.md` |
| Version plan | `docs/plan/2026-11-07-version-plan.md` |
| Usage and forecast | `docs/ledger/` (README explains the CSVs), `tools/usage-ledger/` |
| Real-exe test scripts | `tools/real-exe-tests/`, `tools/ai-smoke/`, `tools/db-bench/` |
| PC automation | `tools/heartbeat/` (reference copy), `05-pc-automation.md` |
| Notes that used to live in Claude's project files | `docs/plan/notes-archive/` (index below) |

**Notes archive** (`docs/plan/notes-archive/`): AI CLI action list, import and export study, whiteboard study, dependency and licence study (every library credited, MIT-style, no copyleft surprises), Notion hub sync record, prompt audit of the Claude config, model-choice guide (the economics behind Sonnet-high / Opus-high / Codex-Sol choices), the PBS-vs-FFmpeg records report, a read of the plans and handoffs, a read of the security summary, and the session-summary style lessons. **Kept private on Ahmed's PC and not in Git:** the Notion survey A to H, the exact formulas, the Notion parity study and the dashboards study (they describe his real hubs). Task for the coordinator: when one of them is needed, ask Ahmed.

## 8. Notion

- **Notaya hub** (private draft in Ahmed's Notion): `https://app.notion.com/p/411bd7ade63b813ea823db0ceb199248`. Version Feature Board `…/05924f0186514045b26b491649aadc34` (data source `9a0ace4e-5f46-4882-aefc-6fe115d4928b`; properties Feature, Version, Lifecycle, Size, Area, Outcome, Evidence / Source, Order, Effort, Plan ID, Gate, Depends on, Build state, Build note), Human Roadmap `…/411bd7ade63b817fbd74ec9b9f1f4a31`.
- **The board may be updated** (cards in place, at each lane merge: Build state and Build note, PR link in Evidence / Source). **Nothing else in Notion may change** except the scratch page.
- The **PFS hub** (`…The-Personal-Financial-System-36cbd7ade63b804f90c5c91ef0fad946`) is **read-only**. Never copy real values. Formula text is private and never committed.
- If ChatGPT's Notion connector is enabled [Check], scope it read-only for the PFS hub and write only to the Notaya board. Otherwise Ahmed pastes updates by hand; keep a one-line "Notion to update" list in each Session Summary.
- Roadmap and board mismatches are repaired from `docs/STATUS.md`, which wins over the hub.

## 9. The artifacts, the PC automation, how Claude things map

Artifacts and how to rebuild them: [`04-artifacts-and-rebuild.md`](04-artifacts-and-rebuild.md). PC automation: [`05-pc-automation.md`](05-pc-automation.md). Short translation table of what Claude did and what replaces it:

| Claude did this | Now |
|---|---|
| Project chat with a coordinator, and one Claude thread per piece of work | One ChatGPT Project "Notaya" (instructions + files + chats) for coordination; Codex desktop threads or worktrees for lanes. Keep the **numbered question list** in the Project's chat and in `docs/STATUS.md`. |
| Project memory (`team/silo`) | The repository (records win) plus the ChatGPT Project instructions (block A) and, optionally, ChatGPT memory. The full memory snapshot is in `03-standing-rules.md`. |
| `.claude/skills/*` (change workflow, coordination, real-exe test, PBS) | They are plain Markdown. Point `AGENTS.md` at them; if Codex loads skills from a folder [Check its docs], copy or link them there. `personalized-charts` and `personalized-notion-hubs` already carry an `agents/openai.yaml`. |
| GitHub MCP tools in cloud threads | ChatGPT's GitHub connector (read) and the Codex app's git. Pushes and merges happen from the PC checkout or the GitHub web page [Check which route works first]. |
| Claude Remote Control into the PC | The Codex desktop app already runs on the PC. Real-exe tests follow `docs/handoff` rules and `tools/real-exe-tests/README.md` on throwaway data. |
| Claude artifacts (ledger, roadmap, handover, FFmpeg lessons) | Plain HTML files in the repo and local pages; see `04`. Existing artifact links keep working but only Claude can edit them. |
| Heartbeat probe of Claude's own quota (`claude -p`) | Not needed any more; see `05` for what to switch off. |
| Hearth chat reactions, wakes, thread titles | Not applicable. Keep titles short and goal-first in whatever app is used. |

## 10. Mohamed continues on Claude Pro, if needed

Mohamed has **Claude Pro only, no Codex** (Ahmed 2026-11-08 08:57:27 UTC). If Ahmed's OpenAI quota also runs out or Ahmed is away, Mohamed's Claude can carry on: block D in `02-start-prompts.md` is his start message; he reads `AGENTS.md`, `CLAUDE.md`, `docs/STATUS.md`, this folder, claims and the open PRs, claims his work on a `claude/project-thread-*` branch with a draft PR, and merges and accepts on his side (a Claude on Mohamed's side never merges by itself; Mohamed merges). Mohamed's Claude then also writes code, within its weekly limit, so briefs, STATUS "Continuation" and this guide must always be complete enough to resume from. Mohamed's PC paths: repo `E:\Project\Notaya\Notaya`, tests `E:\Project\Notaya\Notaya Tests Excluded`; nobody touches his real vault or `E:` drive. Making the repo public and adding a licence need both partners; the About-box text waits for Mohamed.

## 11. First-day plan for the ChatGPT coordinator

1. Read, in order: `AGENTS.md`, `CLAUDE.md`, `docs/STATUS.md`, `docs/plan/approval-and-amendments.md` (A37 to A55), this folder (`02` and `03` fully), `docs/handoff/claims.md`, the newest `docs/sessions.md` entry (53, then 54), `docs/ledger/README.md`, `docs/handoff/codex-partner-briefing.md`, `docs/plan/2026-11-08-v1-6-codex-briefs.md` (the "Update" blocks at the end).
2. `git fetch --all --prune`; list open PRs and the remote branches; compare each lane branch tip with section 6 above. **Say in plain words to Ahmed what matches and what does not.** Then stop and wait for his "handover accepted".
3. After the go: read N2 in full, run `cargo test --lib` on `e11b115` (the PC heartbeat does this; or use the PC), read the two failing tests, fix or have Codex fix (Sol xhigh, import code is risky), merge with a merge commit, update records.
4. Read P6b in full, run the browser check, merge it, then re-queue P6 as superseded if appropriate; queue N3, P7, P8, S areas 3 and 4 (area 3 needs Daybreak), WP-18, the 250-check pass. At most three lanes at once, two while Codex is above 60% of its window; Sol xhigh for vault, crypto, security, importer and speed; Terra high for records, test scripts, text-only polish (A54).
5. Keep the checklist honest: results are *produced / verified / integrated / accepted*; "not run" is said as "not run".
6. Each milestone: update `STATUS.md`, the claims row, the ledger CSVs (`tools/usage-ledger/build_dashboard.py` rebuilds the dashboard), the roadmap HTML, the handover guide (both copies, A39), and the Notion board.
7. Ask Ahmed one numbered question at a time (**Q67 next**). Plain words, moderate emojis (✅ 🟡 ⬜), light segmentation. **Never shorten explanations to save quota** (Ahmed 2026-11-09 07:02 and 08:51).

## 12. Honest limits of this handoff

- I could not test anything in ChatGPT or Codex. Every statement about their features is **[Check]**. The heartbeat's lane launcher points at a Codex install inside the Claude app's package folder (`run-lane.ps1`); that must be repointed.
- Live branch and meter facts in section 6 are from 03:27 to 03:31 UTC and age fast. `git fetch` and the heartbeat log win.
- N2's test status is **contradictory** between the heartbeat line and the previous coordinator's report. It is not "verified" until a person or AI has run the tests on `e11b115` and read the output.
- The handover guide for Mohamed (`docs/handoff/2026-11-07-handover-guide-for-mohamed.md`) and its artifact copy: the repo copy gets a short section 18 and a change-log line from this migration; the **artifact copy was not republished** (Claude had no quota to spare; the HTML snapshot in `docs/artifacts/snapshots-2026-11-10/` is the 2026-11-08 version).
- The Notion board and hub were **not** updated by this thread (no change to Notion was asked). Their latest merge counts may lag; STATUS wins.
- The ledger row for this thread has no token numbers ("not recorded"): reading them would cost quota Ahmed needs.
- Daybreak access: the name and the steps are as relayed; Ahmed confirms with OpenAI.
- Nothing in this package grants permission by itself. ChatGPT acts for Ahmed only on Ahmed's own words.

## Change log

- 2026-11-10 (first version): written for Ahmed's migration request of 03:32:47 UTC; recorded as A55, EX-2026-11-10-73, session 54.
