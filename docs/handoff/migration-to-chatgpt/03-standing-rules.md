# Standing rules and the project-memory snapshot

**Dated pause correction (2026-11-10, R3):** This file's earlier open-item, heartbeat and next-lane statements are historical. Read the [pause resume note](../2026-11-10-pause-resume-note.md) first: development, fixes, scripts and merges are paused on Ahmed's 06:13:35 UTC words, clarified 06:25:54 UTC. A55 still requires his handover acceptance. Scheduled-task removal is not verified.

**Who and where:** Ahmed's side (`AE0101`). Compiled by Claude, cloud, 2026-11-10 about 04:10 UTC (system clock), from Claude's shared project memory (`team/silo`, read at the start of this thread), `CLAUDE.md`, `AGENTS.md` and `docs/plan/approval-and-amendments.md`. Result state: *produced*. **The repository records win** where this file differs; the dated words below were relayed by the previous coordinator and are cited in the amendments record.

This file exists because project memory belonged to Claude's project and ChatGPT cannot read it. Part 1 groups the rules by topic. Part 2 is the memory index word for word, as it stood on 2026-11-10 03:33 UTC, so nothing is lost in the grouping. Part 3 holds the other memory notes that carry ids and links.

## Part 1. The rules, by topic

### Authority and partners
- **Ahmed (`AE0101`) and Mohamed (`AbyssWatcher2`) are partners**, independent developers on one repository (A37, Ahmed 2026-11-07 20:23:20 UTC). Each partner's own words authorise the work of his own AI setup. In the rules, "the owner" means the partner whose side the work is on.
- **Ahmed is a director, not a developer**: plain words, one numbered question list in the project chat (next **Q67**), gather open questions from all threads into one list, write commit messages as clear points a non-developer can follow, list any improvement nobody asked for as an *addition*.
- **Merge on our judgment** (Ahmed, 2026-11-08 16:39): the coordinator merges PRs it judges ready, without asking Ahmed, and reports merges afterwards. (Merge with "Create a merge commit".) Acceptance remains a partner's own word.
- **Public release and licence need both partners.** Changing GitHub repository settings is Mohamed's click only. A release needs a partner's word.
- **Permission is only a partner's own words**; observation is what was checked; acceptance is the partner's decision (PBS).
- **Mohamed has Claude Pro only, no Codex.** When Ahmed's quotas end, Mohamed's Claude continues, so handovers, plans and records must always be clear enough for Mohamed (Ahmed 2026-11-08 01:09; correction 08:57:27).

### Method (PBS)
- **PBS v1.1 applies to ALL Notaya development, every thread and every lane.** Every thread reads the PBS hub and the repo's PBS docs first. PBS must never be weakened.
- **Thorough documentation is mandatory.** Session Summary is ONE file, newest on top (`docs/sessions.md`, 13-section full form). The usage ledger is mandatory. The Notaya Notion hub roadmap is updated at every milestone.
- **Who and which machine (A38):** every record states author (Ahmed/AE0101 or Mohamed/AbyssWatcher2) and machine. Never shorten or rewrite old entries; add dated corrections.
- **Know what exists first:** check code and docs before proposing (feature inventory `docs/plan/2026-11-07-notaya-full-feature-inventory.md`).
- **Be a proactive partner:** offer insights and UX enhancements; test for bugs and quality-of-life issues; flag extras as additions.
- **Repo records win over memory.** Memory is only a pointer.
- **Living handover guide (A39):** `docs/handoff/2026-11-07-handover-guide-for-mohamed.md` (and its HTML copy) is updated at every milestone, decision or new rule, so it never goes stale. Mark each item shared rule / Ahmed's side / needs both.

### Product
- **North star:** full offline Notion parity. **Formulas must be dead accurate.**
- **No trademarked names** in Notaya's UI (A42). Ahmed's own names: "Notebook mode", "Default style".
- **Version plan:** 1.6 AI CLI + safety net + large-database speed + security pass + polish sweep + Notion import fix; 1.7 comfort + PDF emoji; 1.8 page features; 1.9 tabs and Excel; 2.0 corkboard; 2.1 to 2.2 dashboards; 2.3 to 2.4 whiteboard; 2.5 Notebook mode.
- **Full-app test before every release** (Ahmed, 2026-11-09 02:07): before the final release or merge of a version, test the WHOLE app, every existing feature and not only the new ones; polished and optimised before any new features start. Release gate (A47).
- 1.6 scope additions (2026-11-09 06:52 to 06:56): P1 to P8 database speed incl. P8 virtualization of list, gallery, board and timeline (A48, A49); WP-17 full security pass (the reviewer reads, Codex tests; guide `docs/handoff/codex-security-review-guide.md`); WP-18 whole-app polish and tech-debt sweep; X-1 blank window at Start with Windows (reporter Mohamed). Q59: yes, `@tanstack/react-virtual` (MIT, credited). Q60: never wait for Ahmed to install safe open-source dependencies (credit licences). Q61: P8 in 1.6. Q63 and Q64 yes (A51).
- **Notion import fix (critical, Ahmed 2026-11-09 10:42, A52 / WP-19 / card 1.6-19)**, a 1.6 release gate. Problems: missing databases, section import without a home page, loose duplicate row pages whose delete removes the entry. Real data: Ahmed's Notion workspace export of 2026-11-03 (a ZIP of a little over 1 GB) and a folder of single-section exports, on his PC outside the repo (the exact paths are in Part 2). Q66 yes to all six arrangement changes (A53).

### Security by design (reaffirmed 2026-11-09 06:55)
Security is critical across the ENTIRE design; use proven patterns, no home-made security. The CLI is sealed in, off by default, toggled only in Notaya Settings. Deletes go to the recycle bin after confirmation. AI edits are undoable. The activity log is tamper-proof. Locked notes stay locked. The AI can never change its own permissions. Keys and secrets stay in the Rust core; the UI receives only what it displays. Update `docs/security-design.md` when a promise, boundary or limitation changes. Every code PR gets a security review and the security self-check from the guide, section 4.

### Test safety
- **Tests only on throwaway data**, in `D:\Notaya Tests` (outside the repo `D:\Notaya`); Mohamed's: `E:\Project\Notaya\Notaya Tests Excluded`. Never anyone's real vault or notes, never Mohamed's `E:` drive.
- UI automation uses a throwaway vault in a separate process with `NOTAYA_HOME` (or `CIPHERLEAF_HOME`) pointing at a temp folder. Close the WebView2 debug port afterwards and confirm nothing listens on it (`docs/failure-intelligence.md` FI-01). Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set.
- Don't force-stop KeePassXC.
- Never commit vault data or secrets (`.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports, `.env`). Check `git status` before every commit. Stable releases only; exact versions (`.npmrc` `save-exact=true`; Rust in `Cargo.lock`).
- Quote times; don't estimate them (message stamps or the UTC system clock, else "not recorded").
- Verification table (PowerShell from the repo root; refresh PATH first): `cd src-tauri; cargo test --lib`; `npx tsc --noEmit -p tsconfig.json`; `npm run build`; `npm audit`; `npm run tauri dev`; `npm run mock`. A check that could not run is "not run", never "passed". A result applies only to the exact source it ran on. On Ahmed's side the Gaming-PC heartbeat runs the checks on the exact pushed commit; a Rust result must name who ran it and on which commit.

### Roles, models and quota (Codex first)
- **Codex first** (Ahmed 2026-11-08 08:27:53, A46): lean on Codex for EVERYTHING it can do: building, bug fixes, polish, security, UX. The coordinator directs, oversees and keeps the vision and coherence; the app's feel must not change. Codex reads the PBS records, docs, vision, roadmap and architecture before working. Never let Codex sit idle.
- **Codex model split (A54, Ahmed 2026-11-09 21:27:52):** use `gpt-6.1-terra` at high effort (about Sonnet 5.5 level, about 65% cheaper than Sol) for lanes that do not need Sol xhigh: docs, PBS records, simple lanes, test scripts, selector fixes, text-only polish. `gpt-6.1-sol` xhigh stays for risky code: vault, crypto, security, importer, database speed. A Terra lane's diff is read as closely as a Sol one. Codex weekly was about 54% left at 21:27Z on 2026-11-09.
- **Model choice before this migration** (A50, 2026-11-09 07:10): the coordinator chose each thread's model by task complexity; Sonnet 5.5 high preferred over Opus; Haiku 5.5 high for simple tasks; be lean on threads. **After the migration** the same idea holds with OpenAI's models: the strongest model and effort only where the work is risky or decides direction; the cheaper model and lower effort for records, reading and routine; never "max" settings. The old economics are in `docs/plan/notes-archive/2026-11-07-model-choice-guide.md`.
- **Chat quality** (Ahmed 2026-11-09 07:02, reaffirmed 08:51): NEVER shorten project-chat updates or explanations to save quota. Ahmed needs a full, clear status.
- **Quota control** (Ahmed 08:57 and 09:03 on 2026-11-08, critical): the Windows task "Notaya heartbeat" (every 10 minutes, hidden) restarts stopped lanes, pauses at limits and logs Codex's exact meters, pushed to the ledger hourly; a 2-minute hidden guard task closes the gap (Q65); predictive stop line (5-minute look-ahead, 95% hard). Codex's page shows percent LEFT; the meters in the ledger show percent USED. At most three lanes at once. Work in the 5-hour window until about 5% is left, then stop cleanly.
- **Banked resets:** Ahmed has banked Codex resets and applies them himself; no notification needed; never automatic (the heartbeat only pauses and reports).
- **Claude's quota rules** (now history): the 70% weekly pause (Ahmed 2026-11-07 23:59:56) was lifted for 1.6 on 2026-11-09 07:10 (A50, Q62). Claude's weekly meter was 91% used on 2026-11-10 03:21; it resets Fri 2026-11-13 02:00 UTC.
- **Quota strategy** (2026-11-08 01:09): don't spend quota needlessly; Ahmed cannot pay for bigger plans; when his quotas run out, Mohamed's continues.

### Records and style
- Style: moderate emojis (✅ 🟡 ⬜), light segmentation, plain words for Ahmed.
- Commit messages on Ahmed's side end with `Side: Ahmed (AE0101) · Machine: <machine>`.
- Questions in the project chat: one continuing numbered list; STATUS "Open questions" holds the durable copy.

## Part 2. The project-memory index, word for word (as of 2026-11-10 03:33 UTC)

> # Notaya
> - *** CODEX TERRA (Ahmed, 2026-11-09 21:27) ***: use GPT 6.1 Terra high (~Sonnet 5.5 level, ~65% cheaper than Sol) for Codex lanes that don't need Sol xhigh: docs, PBS records, simple lanes, test scripts. Sol xhigh stays for risky code (vault, security, importer, perf). Codex weekly ~54% left at 21:27Z.
> - 2026-11-09 evening: merged #118 SF1, #120 SF2, #121 X-1, #122 A53 (Q66 yes to all six Notion arrangement changes) + N2/N3 briefs, #123 N1 findings, #125 SF3. P6 held (undo regression; fix lane P6b). Queue after 22:51Z reset: N2, P6b, S area 3. Q65 guard done. Next question number: 67.
> - *** NOTION IMPORT FIX IN 1.6 POLISH (Ahmed, 2026-11-09 10:42, critical) ***: recorded as A52 / WP-19 / card 1.6-19 "Notion import with real data", PR #116 (records only), est ~6 pts (4-9), a 1.6 release gate. Lanes: N1 read-only findings (copies to D:\Notaya Tests, structure only, no real values), then N2/N3 fixes after security area 3. Problems: missing databases (lead: importer skips table files without row pages), section import without home page, loose duplicate row pages whose delete removes the entry. Real data: "G:\Other Random Folders & Data [Important] - Movies HDD Edition\New\Notion Exports\Notion Workspace export [November 3.2026].zip" and "...\Notion Exports\Other Specfic testing Exports". TODO coordinator: add 1.6-19 to roadmap artifact, ledger forecast, Notion board, handover guide.
> - *** 1.6 TO THE FINISH LINE (Ahmed, 2026-11-09 07:10, Q62) ***: Claude's 70% weekly cap is LIFTED for 1.6; use up to 100% if needed to finish 1.6 fully safe, documented and prepared. (Supersedes the 70% lines below for 1.6.)
> - *** MODELS + THREADS (Ahmed, 2026-11-09 07:10) ***: coordinator chooses each thread's model by task complexity: prefer Sonnet 5.5 high over Opus; Haiku 5.5 high is fine for simple/cheap tasks. Be lean on threads; only open Claude threads for work needing back-and-forth iteration. Codex (GPT 6.1 Sol xhigh, ~Opus-level, cheaper than Sonnet medium) does everything else.
> - *** CHAT QUALITY (Ahmed, 2026-11-09 07:02, reaffirmed 08:51) ***: NEVER shorten project-chat updates/explanations to save quota; Ahmed needs full clear status. Claude's week is the bottleneck, not Codex.
> - *** BANKED RESETS (Ahmed, 2026-11-09 07:10) ***: Ahmed has banked Codex resets (1 left after 07:36Z) and applies them himself; NO notification needed and never auto-apply.
> - Live AI test (C5) passed all checkpoints 2026-11-09 ~09:12Z on the 08 Nov main build: verified, not accepted. Windows-owned prompts (Q63) need their own hands-on round later. Records lane R1 (Codex) is queued for 12:38Z, Claude reviews it; then the coordinator updates Notion (live test result, merge counts). Claude 5h resets 11:40Z; weekly resets Fri 13 Nov 02:00Z.
> - *** 1.6 SCOPE ADDED (Ahmed, 2026-11-09 06:52-06:56) ***: DB speed work in 1.6 polish: P1-P7 plus P8 (list, gallery, board, timeline virtualization), A48/A49. Also WP-17 full security pass (Claude reviews, Codex tests; Codex guide docs/handoff/codex-security-review-guide.md) and WP-18 whole-app polish/tech-debt sweep. X-1 blank window at Start with Windows (reporter Mohamed). Documented in roadmap (cards 1.6-16..16d, 17, 18), Notion (7 cards), ledger (1.6 ≈79.5 points), handover artifact 1.6 section, PRs #107, #108, #110, #111, #113, draft #109. Q59 yes (@tanstack/react-virtual, MIT, credited). Q60: never wait for Ahmed to install safe open-source deps (credit licences). Q61 = P8 in 1.6. Q62 = cap lifted. Q63/Q64 yes (A51).
> - *** FULL-APP TEST BEFORE EVERY RELEASE (Ahmed, 2026-11-09 02:07) ***: before the final release or merge of a version, fully test the WHOLE app, every existing feature and not only the new ones. Make it polished and optimised before any new features start. Release gate.
> - *** MERGE ON OUR JUDGMENT (Ahmed, 2026-11-08 16:39) ***: Claude merges PRs it judges ready, without asking Ahmed. Report merges after.
> - *** CODEX FIRST (Ahmed, 2026-11-08 08:27) ***: lean on Codex for EVERYTHING it can do: building, bug fixes, polish, security, UX. Claude directs, oversees and keeps the vision and coherence, and the app's feel must not change. Codex must read the PBS records, docs, vision, roadmap and architecture before working (briefing PR #81). PBS must never be weakened. Never let Codex sit idle.
> - *** AUTOMATIC QUOTA CONTROL (Ahmed 08:57/09:03, CRITICAL) ***: Windows task "Notaya heartbeat" (every 10 minutes, now hidden) restarts stopped lanes, pauses at limits and logs Codex's exact meters, pushed to the ledger hourly. Plus 2-minute hidden guard task (Q65). Predictive stop line (5-min look-ahead, 95% hard). Claude usage status line in ~/.claude/settings.json gives exact Claude 5h/weekly readings. Codex's page shows % LEFT; ours shows % used.
> - Ledger: charts plus NOTAYA forecast 1.6 to 2.5. Ledger and roadmap default to the light theme. Roadmap "points" = rough size score (~1 dev-day each), NOT quota. Unused FFmpeg forecast: https://claude.ai/artifact/Fauju9JfwGE5LLYMs8niby.
> - *** QUOTA STRATEGY (Ahmed, 2026-11-08 01:09) ***: don't spend Claude's quota needlessly. Codex builds ALL lanes. Claude normally stops at 70% weekly (lifted for 1.6, see top). Ahmed can't pay for bigger plans. When Ahmed's quotas run out, MOHAMED's quota continues, so handovers, plans and records must always be clear enough for Mohamed. Mohamed has NO Codex, only Claude Pro.
> - 1.6 gates G1-G4 approved. Codex runs on the PC in D:\Notaya-lanes\<lane>. Merged 11-08/09: #74, #75, #76, #79, #81, #85, #86, #88, #90, #92, #93, #97, #98 README (1.5.1 only), C2d, #106 AI server, #70 AI screens, #105, #107, #108, P1 speed lane, #116, #117. Whole-app test pass (250 checks) is the release gate. About box text waits for Mohamed. Roadmap artifact https://claude.ai/artifact/LgHxQi61Fg9p619DU6yT6M. DB perf plan: /mnt/project-files/notes/2026-11-09-large-database-performance-plan.md.
> - Ledger artifact: https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY (rebuild: tools/usage-ledger/build_dashboard.py, republish to the same link). $100 cloud credit used up.
> - *** PBS EVERYWHERE ***: PBS applies to ALL Notaya development, every thread. Every thread must read the PBS hub + repo PBS docs first. Include this in every start_thread instruction.
> - *** SECURITY BY DESIGN (reaffirmed 2026-11-09 06:55) ***: security critical across the ENTIRE design; proven patterns. CLI sealed in, off by default, toggled only in Notaya Settings. Deletes go to recycle bin after confirmation; AI edits undoable; tamper-proof log; locked notes stay locked; AI can never change its own permissions.
> - *** WHO + WHICH MACHINE (A38) ***: EVERY record states author (Ahmed/AE0101 or Mohamed/AbyssWatcher2) and machine. Never shorten or rewrite old entries.
> - *** BE A PROACTIVE PARTNER ***: offer insights and UX enhancements; test for bugs and QoL issues. Flag extras as additions.
> - Ahmed (GitHub AE0101) is a partner with Mohamed (AbyssWatcher2, repo owner). Ahmed is a director, not a developer: use plain words.
> - STYLE: moderate emojis (✅ 🟡 ⬜), light segmentation.
> - *** NO TRADEMARKED NAMES *** in Notaya UI.
> - *** QUESTIONS IN PROJECT CHAT ***: one continuing numbered list in the project chat.
> - Repo: https://github.com/CipherVaults/Notaya. Local D:\Notaya on Gaming-PC. Tests only on throwaway data in D:\Notaya Tests, never Mohamed's vault, E: drive or real notes.
> - Records: docs\sessions.md, docs\plan\approval-and-amendments.md, docs\STATUS.md, docs\ledger\, docs\execution\, docs\handoff\claims.md.
> - Method: PBS v1.1. Notion hub: https://app.notion.com/p/The-AI-Project-Bootstrap-System-3fabd7ade63b81f08ee1f81562c2cdd4 . Repo records win over memory.
> - PBS CORE: thorough documentation mandatory. Session summary is ONE file, newest on top. Usage ledger mandatory. Keep the Notaya Notion hub roadmap updated at every milestone.
> - *** KNOW WHAT EXISTS FIRST ***: check code and docs first (inventory: /mnt/project-files/notes/2026-11-07-notaya-full-feature-inventory.md).
> - Roles: Codex (Sol xhigh, or Terra high for light lanes) leads development. Claude inspects, merges, pushes, writes PBS records and session summaries.
> - Permission: each partner's words authorise their own Claude. Public release and licence need both partners.
> - FFmpeg lessons artifact: https://claude.ai/artifact/7gG8pDGVi3eEoXhbDJscg1 ; reference repo AE0101/FFmpeg-Encoder.
> - Notaya Notion hub: https://app.notion.com/p/411bd7ade63b813ea823db0ceb199248 (board may be updated). Nothing else in Notion may change except the scratch page.
> - PFS hub https://app.notion.com/p/The-Personal-Financial-System-36cbd7ade63b804f90c5c91ef0fad946 is read-only; never copy real values. Formula text is private and never committed.
> - VERSION PLAN (/mnt/project-files/notes/2026-11-07-version-plan.md): 1.6 AI CLI + safety net + large-DB speed + security pass + polish sweep + Notion import fix; 1.7 comfort + PDF emoji; 1.8 page features; 1.9 tabs/Excel; 2.0 corkboard; 2.1-2.2 dashboards; 2.3-2.4 whiteboard; 2.5 Notebook mode.
> - North star: full offline Notion parity. Formulas must be dead accurate.
> - *** LIVING HANDOVER GUIDE ***: artifact https://claude.ai/artifact/UhKn7cS38rp7Sq89YfGLyp + repo docs/handoff/2026-11-07-handover-guide-for-mohamed.md. Update BOTH at every milestone.
> - MODEL CHOICE: see MODELS + THREADS and CODEX TERRA at top. One Opus xhigh diff review for risky code. Never max.

(Typo kept in the quoted path: "Other Specfic testing Exports" is the real folder name. Paths under `/mnt/project-files/notes/` are Claude's project files; in the repo the same files live under `docs/plan/` and `docs/plan/notes-archive/` as listed in `01-migration-guide.md` section 7. `~/.claude/settings.json` was Claude's status line and is not needed once Claude stops being the coordinator.)

## Part 3. Other memory notes (ids and links)

- **Notaya Notion hub:** hub `…411bd7ade63b813ea823db0ceb199248`; Version Feature Board `…05924f0186514045b26b491649aadc34` (data source `9a0ace4e-5f46-4882-aefc-6fe115d4928b`, properties Feature, Version V1.0 to V1.15 and Later, Lifecycle, Size, Area, Outcome, Evidence / Source, Order, Effort, Plan ID, Gate, Depends on, Build state, Build note); views: Features by version (board), Roadmap ahead, Planned small vs big, All features table; Human Roadmap `…411bd7ade63b817fbd74ec9b9f1f4a31`. 2026-11-07 about 23:55 UTC the board was re-staged to the version plan (22 cards updated, 58 created). 2026-11-08 15:30 UTC: **Build state** (not started / queued / partly built / merged·checked on PC·not accepted / accepted) and **Build note** set on the V1.6 cards with the PR link in Evidence / Source; refresh at every lane merge (record `docs/execution/2026-11-08-notion-hub-sync-with-version-plan.md`, PR #91). V1.0 to V1.5 released (V1.4, V1.5 accepted). Ahmed's answers of 2026-11-07: Home banner = own image per time-of-day cycle that stays as default; Whiteboard = own section like sticky notes, Microsoft Whiteboard-style with templates; V1.6 stays one release. (Repo `STATUS.md` wins over the hub.)
- **Usage ledger dashboard:** artifact `https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY`, built from `docs/ledger/*.csv` by `tools/usage-ledger/build_dashboard.py` (template `dashboard-template.html`; forecast by `forecast-model.cjs` fed by `forecast-inputs.csv`). At each session stop add CSV rows, rebuild, publish. $100 cloud credit is used up. See `04-artifacts-and-rebuild.md`.
- **Quota history and hand-over to Mohamed:** Codex quota facts (about 6.5 five-hour windows per week; four lanes at once burned a window in 18 minutes, so at most three); the heartbeat is permanent and automatic (Ahmed registers scheduled tasks himself because Claude Code's permission system refuses to register persistence).
- **Handover guide:** the repo copy and the HTML snapshot must stay identical in content (A39).
