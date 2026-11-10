# Notaya: digest of plans, handoffs, releases and failures (read 2026-11-07)

Read-only. Source: `CipherVaults/Notaya`, `main` at `7661503` (merge of PR #53, the Codex-lead hand-off), fetched 2026-11-07 about 20:17 to 20:50 UTC (system clock). Nothing was changed in the repository. Where the records and Ahmed's project memory differ, the repository records win (CLAUDE.md, PBS rule 7).

Result state of this file: produced. Every statement comes from a file I read; "not checked" means I did not check it.

---

## 1. Who decides, in one paragraph

Mohamed Sayed (GitHub `AbyssWatcher2`) was the sole owner from 2026-10-22. On 2026-10-29 08:39:33 and 08:40:02 UTC he made "the friend" a full partner ("i want him to have the full work acess", "from the beginegn to after finishing V1.4.0"). `claims.md` names the friend as `AE0101`, which is Ahmed (the 2026-11-07 hand-off note says "Ahmed Emara (GitHub AE0101), partner"). Either partner's own words give permission, merge, release and accept. Making the repo public or adding a licence needs both. GitHub repository settings are Mohamed's click only. A Claude never merges or releases by itself (repo rule); Ahmed's project memory says his OK is enough and the coordinator may merge, which overrides that rule in practice but is **not yet written into the repo** (memory says "record it in repo docs at next change"). The repo moved from `AbyssWatcher2/CipherLeaf` to `AbyssWatcher2/Notaya` (2026-10-28 ~16:48 UTC) and is now `CipherVaults/Notaya`; older records still use the old links on purpose.

## 2. The approved plan and the build in one page

- Request, 2026-10-22 11:01 UTC: a portable, very secure notes vault like OneNote plus sticky notes, customization, password vault and generator, impressive UI, stable software only. Answers: desktop app, beginner, master password plus optional keyfile, sticky notes float and hide on lock, offline with opt-in breach check, name Cipherleaf.
- Plan approved by Mohamed 2026-10-22 11:08:21 UTC (snapshot `docs/plan/approved-plan-2026-10-22.md`, SHA-256 `010F2499...CBA3A`, never edited). Stack: Tauri 2 (Rust + WebView2), React 19 + TypeScript + Vite, TipTap editor, SQLite with every row encrypted (XChaCha20-Poly1305, Argon2id-wrapped data key, optional keyfile through HKDF). Honest limits stated up front: forgotten password is unrecoverable; malware on an unlocked PC wins; unsigned exe shows SmartScreen.
- The product changed a lot from that plan (see section 4): KeePassXC bundled then removed, security became optional, Notion-style blocks then reworded, rename to Notaya, installer, tray, export/import, RTL, databases, Mermaid.

## 3. Release history (from tags, release notes, STATUS, authority table)

| Tag | Published (UTC) | Commit | Notes |
|---|---|---|---|
| v1.0 | 2026-10-23 21:32:53 | `0ed3f52` | First release "CipherLeaf V1.0", portable ZIP, owner ran `gh release create` himself (the PC refused). Includes A21/A22. Not accepted. |
| v1.0.1 | 2026-10-24 12:09:45 | `6fa9ee5` | Audit fixes (A23), Deg/Rad fix. Owner's manual tests (USB removal, Win+L, sleep, idle lock) passed per his 12:16:47 message. Not accepted. |
| v1.1.0 | 2026-10-27 08:39:05 | `70d38a1` | KeePassXC removed (A24), pop-out calculators (A25), sticky multi-delete (A26), ribbon (A27). Not accepted. |
| v1.2.0 | 2026-10-27 22:28:04 | `e25f804` | Renamed Notaya, new icon, Sunrise theme (A28). Not accepted. |
| v1.3.0 | 2026-10-28 16:11:23 | `f970331` | Windows setup (NSIS, per-user) plus portable ZIP (A30). First build `6f582dd` was dropped for an uninstall-message bug (owner tapped "Fix first", 15:27:09). Not accepted. |
| v1.4.0 | 2026-11-03 (after owner's "Publish" 15:50:55) | `452212d` | Tray, Notaya vault format `vault.nota`, export/import, colours, shelf and page links, error log (A31-A33). **Accepted** 2026-11-03 18:23:19 ("i accept"). |
| v1.5.0 | 2026-11-04 ~14:30 | `c27f68e` | RTL/Arabic everywhere, PDF Arabic/CJK, full local databases, automations, Notion combined import (A35). No acceptance of its own. A vault with a database cannot be opened by V1.4.0 or older. |
| v1.5.1 | 2026-11-05 07:43:18 | `95ca8b2` | 15 fixes/additions (A36): back/forward, undo/redo in databases, scroll bars, clickable charts, Close all stickies, 5 recent searches, free-position pictures, 2 GB import, icons/covers/Home, Mermaid. `freezePrototype` turned off. **Accepted** 2026-11-05 11:08:39 ("I approve this version"; the message names no version, the coordinator's reading). Last release. |

Release notes exist in the repo only for v1.3.0, v1.4.0, v1.5.0, v1.5.1 (hashes included). Notes for v1.0 to v1.2.0 are not in the repo.
All assets are unsigned. Each release notes file lists "not checked" items; the biggest recurring ones: SmartScreen, another PC, real Notion export, setup installer on a clean machine (v1.5.0 and v1.5.1 setups were never run).

## 4. Decisions with dates, by theme (all UTC, who = Mohamed unless stated)

Full table: `docs/plan/approval-and-amendments.md` (about 190 dated rows, A1 to A36). Highlights:

**Foundation (2026-10-22)**
- 11:08:21 plan approved. 12:32:12 records go in `docs/`, compact set, local Git (PBS). 13:03:51 and 13:07:49 A9: bundle KeePassXC 2.8.0 for passwords (option D), development resumes.
- 16:12 and 16:55: black/white default and Notion-style block notes (A10, A11); 17:29:57 Notion menus, sub-pages, fonts (A12); 17:44:21 KaTeX 0.19.0 allowed.
- 22:27 "reword and trim" Notion-like features after copyright worry, basic and scientific calculator (A13); 22:55:26 calculator inside each sticky note (A14); 23:14:05 KeePassXC stays bundled for private use (GPL point settled for owner and friends only, not legal advice).
- 23:26 icon (A15), page widths and Paper default (A16).

**Hardening and security behaviour (2026-10-23)**
- A17 fix lost text on lock (FI-04). A18 restore from backup. A19 passwords-file backup, Quit never kills KeePassXC (10 s wait, not the 3 s the owner was asked about).
- 18:32 to 18:47 **A21 notes first**: security and Passwords become optional; a new vault opens with no password using a public built-in key and is *not confidential*; owner agreed 18:39:02 to the three security confirmations. Screenshot protection off by default.
- 19:53 credits and licences. 20:12:53 **A22** remove suggested passphrase and the AGPL `eff-wordlist` crate.
- 20:22:21 owner postponed PBS ("I will tell you everything later"); 22:25:50 approved V1.0.1 (A23) after the deep audit.

**Records and method**
- 2026-10-27 16:57: standing rule, record every step on GitHub and fast-forward his PC copy after each merge. 19:08:33 **A29 adopt PBS in its smallest form**; `acceptance.md` and `sessions.md` join the record set; no Decision Records.
- 2026-10-27 model rule: Sonnet 5.5 for everyday work, Opus 5.5 for hard jobs (relayed, owner's exact words not in thread).
- 2026-11-04 23:09:58: do code edits, builds and tests on the PC through Remote Control, on the thread branch.
- 2026-11-05 06:25: coordinator asserted a "fix bugs on the spot, commit and push is approved" rule; the amendments table records this as the coordinator's relay, **not** the owner's words.

**Product direction**
- 2026-10-27 06:45: remove KeePassXC entirely (A24), pop-out calculator (A25), multi-select delete (A26), ribbon (A27). 08:23: own optional security stays.
- 2026-10-27 16:59 and 17:03: name **Notaya**, two-sticky-notes icon, Sunrise default (A28).
- 2026-10-28: installer decisions all "recommended" (11:56:21), go 12:24:16 (A30); notes folder `%LOCALAPPDATA%\Notaya Notes` confirmed 12:37:12; tray and start-with-Windows (A31); full migration to Notaya naming, vault file `vault.nota`, format `NOTA` v2 (A32, option C chosen 18:00:22); V1.4.0 scope 21:42 (A33: colours, 500-item calc history, export/import). Scope frozen 2026-10-29 02:53:32 (databases and charts later).
- 2026-11-03 19:20 to 19:30: V1.5.0 = RTL + full databases; vault option 1 (new `database` record kind, auto-backup before first database, older Notaya cannot open); files in rows 50 MB cap; PDF must print Arabic/Hebrew/CJK. 2026-11-04 09:05:57 automations and Button added.
- 2026-11-04 17:02 to 17:56 and 2026-11-05: V1.5.1 list (A36). 23:20:04 turn `freezePrototype` off to keep Mermaid. 2026-11-05 06:05:04 keep recent searches stored, accept EPL-2.0 `elkjs`.

**Dropped or paused**
- Native Windows app without WebView2: postponed 2026-10-24 09:19:51, **dropped** 2026-10-28 14:34:32 (PR #11 closed unmerged).
- No-network lock-down of the WebView2 app: **paused** 2026-10-27 07:21:42, never built, not dropped.
- Cipherleaf's own licence: **not chosen**. Making the repo public: not decided (needs both partners).

**Handoff and partnership**
- 2026-10-29 07:55:55 two partners, two Claudes: guide, claim board, setup steps (A34, still "proposed"). 08:51:39 Mohamed's Claude usage ran out, V1.4.0 resume note written for the friend. 08:59:37 the friend's PC replaces Mohamed's for builds and tests.
- 2026-11-07 (Ahmed): **Codex CLI (gpt-6.1-sol, xhigh) leads development, Claude coordinates and reviews**; `AGENTS.md` created; PBS skill refreshed to v1.1 (PR #53, merged). Ahmed's PC: repo `D:\Notaya` (path must not contain an apostrophe, `tauri-winres` breaks on it), tests in `D:\Notaya Tests`.

## 5. Amendments: status as the records say vs what is actually true

| ID | What | Record says | Reality (from other records) |
|---|---|---|---|
| A1 | Fixed Argon2 params (256 MiB, 3, 4) | evidence pending | still open |
| A2 | Single old record can be restored undetected | awaiting owner | still open (STATUS Q9) |
| A3 | Breach check over native TLS | evidence pending | **moot**: breach check removed with A9 P1 |
| A4 | OS `getrandom` | evidence pending | open |
| A5 | EFF wordlist crate | pending | **moot**: removed by A22 |
| A6, A7 | TypeScript 6.0.3 kept; single-instance guard | evidence pending | open |
| A8 | Cargo ranges + lockfile | awaiting owner | still open |
| A9 | Bundle KeePassXC | approved | **superseded and removed by A24** (V1.1.0) |
| A10, A19 | KeePassXC embed; passwords backup/safe Quit | applied | superseded by A24 |
| A11 to A13 | redesign, Notion menus, reword/trim | applied | partly superseded by A13/A25; unaccepted |
| A14 | calculator in sticky note | applied | superseded by A25 pop-out |
| A15, A16 | icon, widths/Paper | applied | icon superseded by A28; Paper no longer default |
| A17, A18, A20 | lock fixes, restore, Start over | applied | released; unaccepted |
| A21, A22 | notes first; passphrase removed | applied | released v1.0; STATUS Q12 (owner review of A21) never closed |
| A23 | V1.0.1 audit fixes | merged | released v1.0.1; unaccepted |
| A24 to A27 | KeePassXC removed, calculators, multi-delete, ribbon | applied/merged | released v1.1.0; unaccepted |
| A28 | Notaya rename/icon/Sunrise | "waiting for owner's merge" | **stale**: merged (PR #24, 19:40:27 on 10-27), released v1.2.0 |
| A29 | PBS adoption | "applied in this docs PR" | merged long ago; in force |
| A30 | Windows installer | "built, not merged, not released" | **stale**: released v1.3.0 |
| A31 to A33 | tray, migration, V1.4.0 features | "applied in draft PR #39, not accepted" | **stale**: released v1.4.0 and accepted 11-03 18:23:19 |
| A34 | Working together | proposed | in daily use; Mohamed's confirmation (STATUS Q27) never recorded |
| A35 | V1.5.0 RTL + databases | "not built yet" | **stale**: released v1.5.0; also its "50 MB (not 50 MB)" is a typo |
| A36 | V1.5.1 | closed as released | accepted 2026-11-05 11:08:39 |

No per-amendment file exists for A22 to A27, A29, A30 (lives in `windows-installer-plan.md`), A34, A36 (lives in EX-2026-11-04-43). A37 is unallocated.

## 6. Open items and open questions (from STATUS, handoff, release notes)

Genuinely open, still relevant:
1. **Licence for Notaya itself**: never chosen (needs both partners). Making the repo public: needs both.
2. **A2 and A8** owner decisions; evidence for A1, A4, A6, A7.
3. **Q25**: 15 live-check findings still open at V1.1.0 (WebView2 traffic to Microsoft at startup, F-04, F-10, SE-03, CR-02, contrast and focus). Needs a decision on a fix release. Table: `docs/audit/2026-10-27-live-checks-vs-v1-1-0.md`.
4. **`cargo clippy`**: run 2026-11-07 on Ahmed's PC: 0 errors, 137 warnings (75 collapsible `if`, etc.), nothing fixed.
5. **PR #45** (draft, 2026-11-03, "Notaya as an offline Notion: gap list and phased roadmap"): the only open PR. Becomes work only if a partner picks a phase. Contents not read by me.
6. **Not-run checks** of v1.5.1: setup installer on a clean user/VM (untick "Run Notaya"), tray "Close sticky notes", Windows keyboard-layout shortcut between Arabic and English lines, Playwright mock checks `dev-mock/*-check.cjs` (two new assertions never ran), Word/PDF output of the Hubs import.
7. Known weak spots listed by the records: very large imports need about 10x file size in memory; attach of hundreds of MB to a row is slow; Mermaid can freeze on huge diagrams; free-placed pictures flow inline in exports and inside columns/lists/tables; per-line text direction not in exports; Notion's built-in covers matched by number not colour; V1.4.0 and older cannot open vaults with a database; old and new Notaya cannot run at once; Notaya "Hubs" main-page report could not be reproduced.
8. Smaller STATUS questions still open: Q10 (backlog of unfixed review findings in EX-13), Q13/Q14 (reset old screenshot setting; offer deleting `restore-undo` files), Q20 (acceptance of v1.0 to v1.3), Q23 (GitHub About text, old skills still say KeePassXC), Q24 (Mohamed's untracked `vendor\`), Q27.
9. Mohamed's own real vault and E: drive remain off limits to everyone.

## 7. Failures and lessons (`docs/failure-intelligence.md`, FI-01 to FI-07)

| ID | When | What happened | Impact | Lesson / rule | In CLAUDE.md? |
|---|---|---|---|---|---|
| FI-01 | 2026-10-22 15:58:50 to 16:06:58 | A WebView2 debug port was left open on Mohamed's live window while he created his vault | None established; unknowable | Never test on the live window; throwaway vault in a separate process, close and verify the port | Yes |
| FI-02 | 2026-10-22 ~17:50 | A `String.replace` with `$` text pasted fragments into two records | Two records wrong for minutes | Use the file tool; read back | Yes |
| FI-03 | 2026-10-22 | Backslashes in a command string became control characters | One wrong mock line | Write backslash text with the file tool, read back | Yes |
| FI-04 | until `dc95fd3` (2026-10-23) | Text typed in the last ~0.5 s before lock or close was lost; shipped to others | None established | Run a "type then lock at once" real-exe test after any save/lock change | **No** (proposed). Owner action open: replace old ZIP copies (e.g. the Drive copy) |
| FI-05 | 2026-10-28 15:54:06 | A setup test left "Run Notaya" ticked and started the app in Mohamed's real session; created `%LOCALAPPDATA%\Notaya Notes` (no vault) | Contained; folder deleted on "Delete it" | Untick "Run Notaya", log the box state, compare real `%LOCALAPPDATA%` before/after | No |
| FI-06 | 2026-11-04 PC round | "Mermaid verified" in cloud/mock was false in the real app because `freezePrototype` was on | Caught before release; cost a round and a security trade-off | Never write "verified" from the mock for runtime-dependent features; mark "NOT RUN in the real app" and test first on the PC | No |
| FI-07 | 2026-11-05 06:34 to 06:37 | A cleanup command would have closed every process named Notaya, including Mohamed's real one | None; tool filter denied it | Select by process id or exe path under the test folder, never by name alone | No (existing line "stop only processes the test started" partly covers it) |

Other recorded frictions worth remembering: two agents editing the same file overwrite each other; the PC's idle lock breaks long real-exe tests (raise `idleLockMinutes`); stale build output keeps old icons (`cargo clean -p cipherleaf`); Bitdefender removed test files in `%TEMP%` (tests moved to an excluded folder); GitHub device-code sign-in can give HTTP 503 (retry); KeePassXC cannot be force-stopped (moot since V1.1.0); the cloud proxy blocks repo-settings writes and branch deletes; exe hashes of test and release builds can differ (v1.4.0: 28 bytes) with identical source; Ahmed's PC: apostrophe in repo path breaks the resource compiler.

## 8. How work is supposed to run (so a new thread follows it)

PBS smallest form: state the task (baseline commit, partner message and UTC time, scope, return, stop point); keep permission, observation and acceptance apart; label results produced, verified, integrated, accepted; records by size (small: PR text; normal: 4-section Execution Summary + index row + STATUS lines; big: 12-section summary + `sessions.md` entry). Claim work on a branch `<login>/<topic>` with the claims row in the first commit, draft PR, no force-push, no rebase of shared branches, merge with "Create a merge commit". Security review on every code PR (Security Guidance plugin plus built-in `/security-review`, review not done by the author). Tests only on throwaway data outside the repo. Max two full PC rounds per release (a third only with the owner's approval). Stable releases only; exact versions; ask before dependencies. Resume note and claims row updated at every stop.

## 9. What is stale or contradictory (the part to act on)

Records:
1. **`claims.md`**: the row "Hand-off to Ahmed" (PR #53) says in progress but #53 is merged; the row "V1.5.1 acceptance records" (PR #52) says in progress but it merged (`4fc269d`); "Collaboration setup" (branch `6bb493`, PR #40) still "in progress"; the friend row says "confirm with Mohamed that this is the friend" although the hand-off names Ahmed. Next free IDs written as EX-44 / A37 / FI-08 (unverified).
2. **`docs/STATUS.md`**: title and big sections still say "Cipherleaf" and "Owner: Mohamed"; "As of" and "Mission state" lines are layered history (2026-10-27/28 text still sits in "Current state"); "Exact next permitted action" is dated 2026-10-27 (talks about PR #24, V1.3.0); it does not mention the Codex-lead setup or Ahmed's role; "Waiting on the owner" Q12 (A21 review) and several others are old; Q27 not formally answered; "What exists" table is mostly 2026-10-23 history.
3. **Amendments table** statuses for A28, A30, A31 to A33, A35 (section 5) say unmerged/unbuilt but they shipped.
4. **Failure log** header says "Last substantive update 2026-10-28" though FI-06 and FI-07 are from 2026-11-05; FI-04 owner action (replace old ZIPs) still open; FI-04 to FI-07 prevention rules never adopted into CLAUDE.md.
5. **Handoff files**: `friend-setup.md` says the first task is V1.4.0 (done); `collaboration-guide.md` still has V1.4.0 as the main job and baseline `42de167`; `2026-10-23-*` handoffs are Cipherleaf-era history (KeePassXC, E: paths). `2026-11-04-v1-5-0-resume-note.md` says "next: merge PR #47" (done).
6. **CLAUDE.md** still has "Claude never merges"; Ahmed's memory says otherwise, and CLAUDE.md names the PC copy `E:\Project\Notaya\Notaya` (Mohamed's).
7. **Branches**: 30 remote branches, nearly all merged `claude/project-thread-*` leftovers; `docs/fix-stale-records` has no commits beyond `main`; branch deletion needs a human (proxy refuses ref deletes, and force/delete needs asking). I did not check each branch's merge state.
8. Repo link drift: records point at `AbyssWatcher2/Notaya` and `AbyssWatcher2/CipherLeaf`; current is `CipherVaults/Notaya`. Releases/PR links in old records may redirect; only the hand-off note says so. Not link-checked.
9. Typo to fix by dated correction, not edit: A35 header "50 MB (not 50 MB)".

Memory vs repo (Ahmed's project memory, 2026-11-07):
- The **v1.6, v1.7, v1.8 roadmap is not in the repo at all** (STATUS says nothing is planned and nothing is being built). Fine, but it needs an amendment (A37) when greenlit.
- Several v1.6 items look **already partly shipped** according to the records, so each needs a "what is missing" check before planning: Mermaid and diagram block (V1.5.1); banner/cover images, page icons and a Home banner (V1.5.1, but not per time-of-day cycle); tray menu with Open/Quit and "Close sticky notes" (V1.4.0, V1.5.1); export (V1.4.0, plus database export V1.5.0); import button (V1.4.0); sticky multi-select delete (A26, V1.1.0); pie/donut/bar/line charts and chart dashboard on databases (V1.5.0, one chart per view; clickable in V1.5.1); code blocks with language (kept on import; exact editor support not confirmed by me). Not found in the records I read: Ctrl+F in-page search, reopen last page, move databases, right-click menus, customizable app bar and editable shortcuts, AI CLI, slicers, Notion-style tabs, Whiteboard, OneNote mode, app tour/guide, About credits for named people (credits for open source exist).
- Memory says Ahmed is "a director, not a developer"; the repo records and hand-off assume a technical partner who runs `cargo` himself. Keep explanations plain.

## 10. What I could not read or did not read

Read in full: `docs/plan/*` (all 22 files), `docs/handoff/*` (all 11), `docs/release-notes/*` (all 4), `docs/failure-intelligence.md`, `docs/STATUS.md`, `AGENTS.md`, `CLAUDE.md` (from context). Also `git tag`, remote branch list, open-PR list (one PR).

**Not read:** `docs/execution/*` (about 80 reports plus the index; they hold the test evidence, hashes and PC-round details), `docs/acceptance.md` (so I rely on STATUS for acceptance rows X-numbers), `docs/sessions.md`, `docs/security-design.md` (P1 to P29, threat list), `docs/audit/*` (the deep audit and live checks), `docs/design/*`, `.claude/skills/*`, `README.md`, `THIRD-PARTY-NOTICES.md`, source code, PR #45's contents, individual stale branches, the Notion hub and FFmpeg docs. Mohamed's chat and the claude.ai project files are not reachable from here. The three Notion pages linked in STATUS were not opened. Nothing I could not read was blocked; I only prioritised what the coordinator listed.

Checks I could not run or did not run: no build, test, link check or hash verification; none of the hashes in the release notes were recomputed.
