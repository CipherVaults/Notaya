# Cipherleaf: execution index

**Purpose:** find an Execution Summary and its open points without replaying its contents.

**Maintainer:** Claude (Claude Code), for the owner.

**Report collection:** this folder, `docs/execution/`.

**Ordering:** newest execution first. Report IDs stay stable even if an older report is corrected.

**State owners (this index doesn't replace them):** [STATUS.md](../STATUS.md) (accepted state, where to resume), [approval-and-amendments.md](../plan/approval-and-amendments.md) (authority), [acceptance.md](../acceptance.md) (requirements versus evidence, and the owner's decisions).

Add an entry for each new Execution Summary (see the routine in [EX-06 Appendix A](2026-10-23-pbs-reconciliation.md#appendix-a-documentation-routine-going-forward)). Dates are UTC.

## Reports

### 2026-11-07 (Ahmed's messages 19:09:26 to 20:23:20 UTC), EX-2026-11-07-44: records catch-up for Ahmed's rules (A37), today's hand-off, stale records, backfilled session entries, and Notaya's own usage ledger

- [2026-11-07-records-catch-up-and-usage-ledger.md](2026-11-07-records-catch-up-and-usage-ledger.md): records only (no app code); PR #54; **verified** by the checks in its section 5; integrated when PR #54 is merged; not accepted (acceptance is a partner's word).

> **Dated note (2026-11-07):** the rows below for EX-42, EX-40, EX-38 and EX-37 keep the wording from before their work finished. They are not rewritten. Their end states: V1.5.0 (EX-42) merged by the owner 2026-11-04 14:24:53 UTC (`c27f68e`, PR #47) and published, not accepted; V1.4.0 (EX-37, EX-40) merged 2026-11-03 15:48:16 UTC (`452212d`, PR #39), published and **accepted** 18:23:19 UTC ([acceptance.md](../acceptance.md), X28); the partner guide (EX-38) merged 2026-10-29 08:52:53 UTC (`253ce56`, PR #40). Current state: [STATUS](../STATUS.md).

### 2026-11-04 (owner's messages 17:02:11 to 17:56:54 UTC), EX-2026-11-04-43: Notaya V1.5.1, fixes and additions after the V1.5.0 feedback (navigation, undo, scroll bars, clickable tags/dates/charts, hide all stickies, recent searches, free pictures, 2 GB limits, icons and covers, Home, Mermaid)

- [2026-11-04-v1-5-1-fixes.md](2026-11-04-v1-5-1-fixes.md): **published** as [v1.5.1](https://github.com/AbyssWatcher2/Notaya/releases/tag/v1.5.1) (2026-11-05, tag on `95ca8b2`); verified in the cloud and on the PC (setup installer, tray entry, keyboard-layout shortcut, Clippy NOT RUN); **accepted by the owner 2026-11-05 11:08:39 UTC** ("I approve this version"). Also covers the 2026-11-05 work: the Hubs import test and its three importer fixes, per-line text direction, the release and the hand-over records. Authority: A36 (closed as released).

### 2026-11-03 (owner's messages 19:20:58 to 21:29:02 UTC; work 19:30 to 22:19 UTC, system clock), EX-2026-11-03-42: Notaya V1.5.0, right-to-left text, PDF letters and full local databases

- Report: [2026-11-03-v1-5-0-rtl-and-databases.md](2026-11-03-v1-5-0-rtl-and-databases.md)
- Task / Goal: "Full database like notion i mean everthing" and Arabic that stays right-to-left everywhere, "except the automations" ([A35](../plan/A35-v1-5-0-rtl-and-databases.md))
- Executor: Claude (Claude Code) in a cloud thread with two helper agents; the PC checks have not run yet
- Outcome: produced and cloud-verified (499 Rust tests, tsc, build, mock-app checks of every view; PC rounds 1 and 2 passed except the bugs fixed since, see report 8b and 8c); the newest fixes and the combined Notion import have not run on the PC; not integrated, not released, not accepted
- Open dependency / next action: PC round 3 (approved 2026-11-04 10:17:54 UTC, throwaway data only), then the owner merges draft PR #47 ("Create a merge commit") and says "publish"

### 2026-11-03 (owner's messages 06:52:53 and 06:54:14 UTC; PC session started after 06:54 UTC and ended before 08:11:56 UTC (thread timestamps)), EX-2026-11-03-40: V1.4.0 features and PC tests

- Report: [2026-11-03-v1-4-0-pc-tests.md](2026-11-03-v1-4-0-pc-tests.md)
- Task / Goal: finish V1.4.0 and test it on the PC on throwaway data; [A33](../plan/A33-v1-4-0-features.md)
- Executor: Claude (Claude Code) in a cloud thread; the PC checks run through Remote Control
- Outcome: produced and verified (350 tests, clippy, runners, migration on copies, error log, export/import in the real exe); not integrated, not released, not accepted
- Open dependency / next action: the owner merges PR #39 ("Create a merge commit") and says "publish"

### 2026-10-29 (owner's messages 07:55:55, 08:33:01, 08:38:36, 08:39:33 and 08:40:02 UTC; work started 07:56 UTC, system clock), EX-2026-10-29-38: guide for two partners and their two Claudes

- Report: [2026-10-29-shared-work-with-a-friend.md](2026-10-29-shared-work-with-a-friend.md)
- Task / Goal: "so now i want me and my freind work on the same project and his claude code and my claude code can continiu flawless and can be picked uped where it lefts"; then "i want him to have the full work acess" / "from the beginegn to after finishing V1.4.0"; amendment [A34](../plan/approval-and-amendments.md) (proposed)
- Executor: Claude (Claude Code) in a cloud thread; docs only
- Outcome: produced (guide, claim board, friend setup, owner invite steps, `CLAUDE.md` section, records); links and the ID commands checked; no code, nothing built
- Integration / release: draft docs PR; V1.4.0 (PR #39) untouched
- Open dependency / next action: the owner's own clicks (invite, optional settings) and his answer to STATUS question 27; after he merges, send the friend the setup link
- Acceptance: not applicable

### 2026-10-28 (owner's messages 17:01:14 to 17:42:06 UTC), EX-2026-10-28-37: PC folder moved to `E:\Project\Notaya\Notaya`; V1.4.0 (tray, start with Windows, uninstaller box, Settings text)

- Report: [2026-10-28-v1-3-0-settings-text-rerelease.md](2026-10-28-v1-3-0-settings-text-rerelease.md)
- Task / Goal: "remove and re release notaya v 1.3.0" which became V1.4.0 ("sorry v1.4"); amendment [A31](../plan/A31-tray-and-start-with-windows.md)
- Executor: Claude (Claude Code) in a cloud thread; the PC checks run through Remote Control
- Outcome: produced (code and records in draft PR #39); verified in the cloud only (`tsc`, build, 151 Rust tests under Wine, one independent review with findings fixed); not yet built or tested on the PC, not released

### 2026-10-28 (owner's messages 16:24:57 to 16:43:19 UTC; work 16:27 to 16:45 UTC, PC system clock), EX-2026-10-28-36: test leftovers cleaned up on the PC

- Report: [2026-10-28-test-leftovers-cleanup.md](2026-10-28-test-leftovers-cleanup.md)
- Task / Goal: "yes please" (delete the listed test folders and scan for others); later "delete them" (the `cipherleaf-*` test folders) and "keep everyhting in test folder that we did" (`v130`, `v130b` stay)
- Executor: Claude (Claude Code) in a cloud thread, deletes run on the PC through Remote Control
- Outcome: about 12.6 GB of throwaway folders deleted (`%TEMP%` clones and `clf-*` test homes, `run2`, `vendor`); no source change
- Integration / release: records PR only
- Open dependency / next action: none; `notaya-installer-2db590ae` was deleted by the owner after a restart, the `cipherleaf-*` test folders at his word (16:39 UTC), and a check at 16:42 UTC found no installed Notaya left on the PC
- Acceptance: not applicable

### 2026-10-28 (owner's messages 16:46:38 and 16:48:41 UTC; work 16:47 to about 17:00 UTC, system clock), EX-2026-10-28-35: GitHub repository renamed to Notaya

- Report: [2026-10-28-github-repo-renamed.md](2026-10-28-github-repo-renamed.md)
- Task / Goal: "Yes please rename" (the repository `AbyssWatcher2/CipherLeaf` becomes `AbyssWatcher2/Notaya`)
- Executor: Claude (Claude Code) in a cloud thread; the rename itself was done by the owner in GitHub Settings (the cloud proxy blocks it)
- Outcome: the new name is live; the old address redirects; no code and no old record link was rewritten
- Integration / release: records PR only; no release
- Open dependency / next action: the owner points his PC copy at the new address; the repository description and topics are his to paste
- Acceptance: pending

### 2026-10-28 (owner's messages 14:35:46 and 15:27:09 UTC; work 14:37 to 16:05 UTC, PC and system clock), EX-2026-10-28-34: Notaya V1.3.0 (setup and portable ZIP)

- Report: [2026-10-28-notaya-v1-3-0-release.md](2026-10-28-notaya-v1-3-0-release.md)
- Task / Goal: "Release" (V1.3.0 with the Windows setup and the portable ZIP); "Fix first" (the update message of the setup window)
- Executor: Claude (Claude Code) in a cloud thread; build and tests in a Remote Control session on the owner's PC, throwaway data, only in `E:\Project\Notaya Tests Excluded`
- Outcome: version 1.3.0 and one installer-hook fix (`f970331`); 145 of 145 tests, `npm audit` 0; setup 4,974,334 bytes (`5A16FD0C…BE4C1`), ZIP 5,924,682 bytes (`494ACE82…2A1C9`); runners, silent and interactive install, update, uninstall, import from the V1.2.0 ZIP pass; a first build (`6f582dd`) was superseded. Not run: SmartScreen, WebView2 download, ticked desktop shortcut, build exit code
- Integration / release: PR #34 merged `9a982da` (16:07:47); [v1.3.0](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.3.0) published 16:11:23 UTC (tag `f970331`, digests equal)
- Open dependency / next action: the owner's acceptance; the PC copy is fast-forwarded after the records PR
- Acceptance: pending

### 2026-10-28 (owner's messages 11:56:21 and 12:24:16 UTC; work 12:25 UTC on, system clock), EX-2026-10-28-33: Windows installer (A30), built in the cloud

- Report: [2026-10-28-windows-installer.md](2026-10-28-windows-installer.md)
- Task / Goal: "All recommendations" (the 10 installer decisions), then "Ok for all go now" (build it; OK for the NSIS download)
- Executor: Claude (Claude Code) in a cloud thread; PC tests to follow in a Remote Control session
- Outcome: per-user NSIS setup settings, `portable.txt` rule, import from a portable copy (copy only), build script and test runner; 145 Rust tests under Wine pass (16 new); the notes folder is `%LOCALAPPDATA%\Notaya Notes` (the program goes in `...\Notaya`)
- Open dependency / next action: PC tests (build, setup install and uninstall in a temp folder, `run-installed.ps1`, the import dialog); the owner confirms the folder name; a release only on his word
- Acceptance: pending

### 2026-10-28 (owner's messages 11:19:57, 11:21:06 and 11:56:21 UTC; work 11:20 to about 12:00 UTC, system clock), EX-2026-10-28-32: Windows installer plan re-checked for Notaya

- Report: [2026-10-28-installer-plan-recheck.md](2026-10-28-installer-plan-recheck.md)
- Task / Goal: "Now continue the setup part"; "So we will do 2 things windows installer and portable version in the release"
- Executor: Claude (Claude Code) in a cloud thread; docs only
- Outcome: plan updated for Notaya V1.2.0 on `main` `a4574be`; all 10 decisions recorded as answered (recommended; installer and portable ZIP in every release); nothing built
- Open dependency / next action: the owner says go; the NSIS tooling download needs his OK
- Acceptance: pending

### 2026-10-27 (owner's message 19:40:47 UTC; work 19:41 to about 22:25 UTC, PC clock), EX-2026-10-27-31: Notaya V1.2.0 portable folder and ZIP

- Report: [2026-10-27-notaya-v1-2-0-portable-build.md](2026-10-27-notaya-v1-2-0-portable-build.md)
- Task / Goal: "Release one" (the first release as Notaya, version 1.2.0)
- Executor: Claude (Claude Code) in a cloud thread; build and tests in Remote Control sessions on the owner's PC, throwaway data only
- Outcome: version bump merged onto `main` `49cd9de` (`e25f804`); `cargo test --lib` 129 of 129, `npm audit` 0, ZIP 5,915,148 bytes (`A7FDD822…CC77D`), exe `D04D42F5…F705C2`; real-exe runners 13 PASS, lock flow ok, 47 of 47; Cipherleaf 1.1.0 vault opens unchanged. The owner's own runs from an Administrator shell failed to find the window (no check ran). Not run: `cargo audit`, password-vault migration, lock events
- Open dependency / next action: none for the release: PR #30 merged (`4e40d1a`), the owner published [v1.2.0](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.2.0) at 22:28:04 UTC and it was verified on GitHub (tag `e25f804`, one asset, digest equal; report addendum)
- Acceptance: pending

### 2026-10-27 (owner's messages 16:59:11 and 17:03:39 UTC; work from about 17:00 UTC, system clock), EX-2026-10-30: renamed to Notaya, sticky-notes icon, Sunrise theme

- Report: [2026-10-27-notaya-rename.md](2026-10-27-notaya-rename.md)
- Task / Goal: "proceed with this and The app name is Notaya"; Sunrise the default, Sunrise dark optional ([A28](../plan/A28-rename-notaya-icon-sunrise.md))
- Executor: Claude (Claude Code) in a cloud thread; PC checks in a Remote Control session
- Outcome: code `6003e81` plus review and exe-metadata fixes (latest code commit `d4933ef`) in draft PR #24; on the owner's PC: `cargo test --lib` 129 of 129, release build, real-exe test and old Cipherleaf 1.1.0 vaults opening in `Notaya.exe`, all on throwaway data; exe properties say Notaya (addenda 4 to 6). Not run: the repo's runner scripts, a restore from the 1.1.0 backup, keyboard-only use
- Acceptance: pending

### 2026-10-27 (owner's messages 19:05:05 and 19:08:33 UTC; work 19:10 to about 19:45 UTC, system clock), EX-2026-10-27-29: PBS adopted in its smallest useful form

- Report: [2026-10-27-pbs-adoption.md](2026-10-27-pbs-adoption.md)
- Task / Goal: "That's really goo just want to to follow / that's my answer" (follow PBS; the recommended answers to the review's four questions)
- Executor: Claude (Claude Code) in a cloud thread, docs only
- Outcome: amendment A29, `CLAUDE.md` gains "Cloud threads" and "PBS work cycle", STATUS "Current state" holds only accepted items, questions 8 and 22 answered
- Open dependency / next action: the owner merges the PR; then the PC copy is fast-forwarded; the skill refresh is a separate review card; nothing is accepted

### 2026-10-27 (owner's messages 17:07:25 to 18:56:19 UTC; work 17:08 to about 19:05 UTC, system clock), EX-2026-10-27-28: live audit (PR #10) compared with V1.1.0

- Report: [2026-10-27-live-audit-comparison.md](2026-10-27-live-audit-comparison.md) (table: [docs/audit/2026-10-27-live-checks-vs-v1-1-0.md](../audit/2026-10-27-live-checks-vs-v1-1-0.md))
- Task / Goal: "check the live audit" / "compare the current verison with this thread to see if everthign got fixed"
- Executor: Claude (Claude Code) in a cloud thread, docs only, read-only code check
- Outcome: PR #10 is merged and its PC session is finished; of the live-check findings 5 are fixed, 3 partly, 1 no longer relevant, 15 still open
- Open dependency / next action: the owner decides whether to plan a fix release; nothing is accepted

### 2026-10-27 (owner's messages 16:51:37 to 16:57:39 UTC; work 16:53 to about 17:05 UTC, system clock), EX-2026-10-27-27: record every step; PC copy on `main`

- Report: [2026-10-27-record-every-step.md](2026-10-27-record-every-step.md)
- Task / Goal: the owner's standing rule "record on both github and the pc ... with everything" / "and everystep", and the update of his PC copy after PR #25
- Executor: Claude (Claude Code) in a cloud thread, docs only; the PC step in a Remote Control session
- Outcome: PC copy switched from `feat/a9-keepassxc` to `main` and fast-forwarded to `2c8c2e6`; the rule recorded in the authority table and STATUS; untracked `vendor\` folder found (STATUS question 24)

### 2026-10-27 (owner's messages 16:36:37 and 16:36:45 UTC; work from about 16:37 UTC, system clock), EX-2026-10-27-26: records catch-up

- Report: [2026-10-27-records-catch-up.md](2026-10-27-records-catch-up.md)
- Task / Goal: the owner's "okay so document them commits and ammedments and the records" / "till this moment": bring the records up to date with what happened from 2026-10-23 20:34 to 2026-10-27 16:36 UTC
- Executor: Claude (Claude Code) in a cloud thread, docs only
- Outcome: authority table filled in from the project chat; A24 and A25 status; A26 (sticky multi-delete) and A27 (ribbon) numbered; 27 Oct commits and merges named; the combined test recorded (EX-2026-10-27-25); sessions log, STATUS and acceptance ledger updated
- Acceptance: pending (the owner merges the docs PR)

### 2026-10-27 (owner's "Yes test" 07:21:42 UTC; work 07:22 to 08:20 UTC, PC system clock), EX-2026-10-27-25: combined test of PRs #17 to #20, then `main` after the merges

- Report: [2026-10-27-combined-test.md](2026-10-27-combined-test.md) (written afterwards, in the records catch-up, from the thread's messages)
- Task / Goal: test A24 to A27 together in the real app on throwaway data and give a merge order; then check `main` `4600561` after the owner's merges
- Executor: a Claude Code session on the owner's PC (model output)
- Outcome: combined build 128/128 Rust tests, existing scripts passed, 23/23 new checks; merge commits `e589baf`, `ef548eb`, `996ae34` pushed to the PR branches; `main` `4600561` 128/128 and 27/27 (200-line history survives a restart)
- Acceptance: pending

### 2026-10-27 (08:22 to 08:34 UTC), EX-2026-10-27-24: V1.1.0 portable folder and ZIP

- Report: [2026-10-27-v1-1-0-portable-build.md](2026-10-27-v1-1-0-portable-build.md)
- Task / Goal: the owner's "Yes please" of 08:21:52 UTC to a release of `main` with A24 (KeePassXC removed), the ribbon, sticky multi-delete and pop-out calculators
- Executor: Claude (Claude Code) in a thread session on the owner's PC, throwaway worktree and data
- Outcome: version 1.1.0 (`70d38a1`); 128/128 Rust tests; ZIP 5,916,923 bytes, SHA-256 `2E0B94B1…A890`; restore 12/12, lock flow 3/3, optional security 47/47 on the shipped exe; released by the owner as v1.1.0 at 08:39:05 UTC, checked on GitHub
- Commits and merges: `70d38a1`, `b090533` (PR #21, merged 08:37:53 UTC as `350c3a2`), `8eb8497` (PR #22, merged 08:42:07 UTC as `87eb7ff`)
- Acceptance: pending

### 2026-10-27 (about 06:45 to 06:55 UTC), EX-2026-10-27-ribbon: OneNote-style ribbon over the page editor

- Report: [2026-10-27-ribbon-editor.md](2026-10-27-ribbon-editor.md)
- Task / Goal: the owner's item 4 of 06:45:11 UTC: a ribbon editor on top like OneNote, keeping the "/" menu
- Executor: Claude (Claude Code) in a cloud thread, with an independent review agent (model output)
- Outcome: Home, Insert and a contextual Table tab; `tsc`, build and 24/24 mock UI checks passed; review findings fixed
- Acceptance: pending
- Open dependency / next action: owner merge of the draft PR; real-exe check on the PC. **Update (records catch-up):** amendment **A27**; code `0e9d626`; checked in the real app in EX-2026-10-27-25; merged by the owner (PR #18, 08:04:20 UTC, `b523df8`); released in V1.1.0

### 2026-10-27 (owner's message 06:45:11 UTC), EX-2026-10-27-sticky-multi-select: select several sticky notes and delete them together

- Report: [2026-10-27-sticky-multi-select.md](2026-10-27-sticky-multi-select.md)
- Task / Goal: tick several sticky notes and delete them with one confirmation
- Executor: Claude (Claude Code) with an independent review agent (model output)
- Outcome: implemented; UI type check, build and a scripted mock test passed; Rust not compiled and the real exe not run
- Acceptance: pending
- Open dependency / next action: `cargo test --lib` and a real-exe try on Windows; the owner merges the PR. **Update (records catch-up):** amendment **A26**; code `31dc920`; Rust tests and a real-exe try done in EX-2026-10-27-25; merged by the owner (PR #17, 08:04:39 UTC, `d69a3f7`); released in V1.1.0

### 2026-10-27 (owner request 06:45:11 UTC; work about 06:46 to 07:10 UTC, system clock), EX-2026-10-27-23: KeePassXC removed (A24)

- Report: [2026-10-27-keepassxc-removed.md](2026-10-27-keepassxc-removed.md)
- Task / Goal: remove KeePassXC from Cipherleaf entirely (code, UI, settings switch, docs, notices, test scripts), keeping notes and the optional security layer
- Executor: Claude (Claude Code) with a documentation agent and an independent review agent (model output)
- Outcome: implemented; `tsc`, UI build and 124 of 124 Rust tests (Windows GNU target under Wine) pass in the cloud; not yet run on the owner's PC
- Acceptance: pending (the owner has not reviewed it)
- Open dependency / next action: the owner reviews and merges the draft PR; a PC pass on throwaway data before the next release. **Update (records catch-up):** code `4104557`; the PC pass was done in EX-2026-10-27-25; merged by the owner (PR #20, 08:04:55 UTC, `7f64b86`); released in V1.1.0

### EX-23: Pop-out calculators with a shared saved history (A24; renumbered A25 in `996ae34`)

- File: [2026-10-27-pop-out-calculator.md](2026-10-27-pop-out-calculator.md)
- Executor: Claude (Claude Code) in a cloud thread session, with an independent review agent
- Outcome: built; `tsc` and `npm run build` passed in the cloud; on the PC (commit `f35acb2`) `cargo test --lib` 171 passed, `tsc` and the build passed
- Integration / review: draft PR into `main`; independent review done (see the record)
- Acceptance: pending
- Open dependency / next action: the owner's review (real exe at `58c299c`: 20 of 20 checks, history survives a restart, 200-line limit)
- Update (records catch-up): the amendment is **A25**; commits `f35acb2`, `5ddd4ec`, `996ae34`, `58c299c`, `b995e4a`; merged by the owner (PR #19, 08:05:20 UTC, `4600561`); released in V1.1.0. This report and EX-2026-10-27-23 share the short id EX-23; ids are not renumbered

### 2026-10-24 (merge 10:44:26 UTC; build about 11:07 to 11:12 UTC), EX-2026-10-24-22: V1.0.1 portable folder and ZIP

- Report: [2026-10-24-v1-0-1-portable-build.md](2026-10-24-v1-0-1-portable-build.md)
- Task / Goal: build the V1.0.1 folder and ZIP from the merged PR #9 (`1e7afe5`) and test the exe in it, for the owner's own testing before the release
- Executor: Claude (Claude Code) with a PC remote session for the build and the real-exe checks (model output)
- Outcome: first folder and ZIP (SHA-256 `9AD3F863…B209`) built and tested, then superseded after the owner's Deg/Rad finding (PR #12, #13); the final build from `6fa9ee5` (ZIP SHA-256 `863500C6…E321`) was released by the owner as GitHub release `v1.0.1` (published 2026-10-24 12:09:45 UTC; tag on `6fa9ee5`; asset digest equal to the ZIP hash; see the report's last addendum)
- Acceptance: pending (the owner said his tests were good and, at 12:16:47 UTC, that USB removal, Win+L, sleep and idle lock passed; those are test reports, not an acceptance)
- Open dependency / next action: the owner's decision on acceptance; the corrected release notes are applied (12:22:01 UTC; see the report's last addendum)

### 2026-10-23 (owner approval 22:25:50 UTC; work times not recorded), EX-2026-10-23-21: V1.0.1 audit fixes (A23)

- Report: [2026-10-23-v1-0-1-audit-fixes.md](2026-10-23-v1-0-1-audit-fixes.md)
- Task / Goal: password change re-keys; safer backups; safety backup and counts before deleting a notebook or section; honest wording with security off; keyboard sidebar; `ctutils` update; updated `Licenses` folder
- Executor: Claude (Claude Code), agents and an independent review (model output), a PC session for the compile and real-exe checks
- Outcome: implemented and reviewed; PC results see the report
- Acceptance: pending (the owner has not reviewed it)
- Open dependency / next action: the owner merges the PR, then the V1.0.1 folder, ZIP and release

### 2026-10-23 (after the owner's message of 21:37:43 UTC; work times not recorded), EX-2026-10-23-20: Notices gaps (WebView2 loader licence, KeePassXC library sources)

- Report: [2026-10-23-notices-gaps.md](2026-10-23-notices-gaps.md)
- Task / Goal: fill the two small gaps in `THIRD-PARTY-NOTICES.md` after the V1.0 release
- Executor: Claude (Claude Code), a research agent and a PC read-only inventory (model output)
- Outcome: WebView2 loader licence text added; KeePassXC bundled libraries documented from real files with a source-pointer section; files added under `packaging/Licenses/`; the V1.0 asset is unchanged and does not include them
- Acceptance: pending (the owner has not reviewed it)
- Open dependency / next action: the owner decides between a V1.0.1 and editing the V1.0 asset

### 2026-10-23 (after the owner's merge at 20:45:05 UTC; work times not recorded), EX-2026-10-23-19: V1.0 stable portable folder and ZIP

- Report: [2026-10-23-v1-portable-build.md](2026-10-23-v1-portable-build.md)
- Task / Goal: build the stable portable folder and ZIP from the merged `main` (`0ed3f52`) and publish the private release "CipherLeaf V1.0" (owner messages of 19:13:17 and 19:31:52 UTC)
- Executor: Claude (Claude Code), PC remote session (model output)
- Outcome: folder and ZIP built and tested on throwaway data (127/127 tests; optional security 52/52, restore flow 13/13; real window 12/12); exe SHA-256 `C43E84220E560BB7A7F6E9C4273794F704B448D1CA8ECC48BC1E3CB72FA8EA11`; ZIP SHA-256 `CAB1E72E79B9061DA05558A9AFB5CFBCB916B589A32FB318AE110E04D522D53D`; no `data\`. The PC's permission layer denied `gh release create`; the owner then created release `v1.0` themselves (tag on `0ed3f52`, asset digest matches the ZIP hash).
- Acceptance: pending (the owner has not reviewed it)
- Open dependency / next action: the owner's review and acceptance

### 2026-10-23 (owner message 20:12:53 UTC for A22; the credits message time and work times not recorded), EX-2026-10-23-18: Credits for open source components and the suggested passphrase removed (A22)

- Report: [2026-10-23-credits-and-passphrase-removal.md](2026-10-23-credits-and-passphrase-removal.md)
- Task / Goal: the owner's request for credits and copyright for all open source used (`THIRD-PARTY-NOTICES.md`, Settings, About, README) and "Remove suggest a strong passphrase" (20:12:53 UTC); amendment A22 in [approval-and-amendments.md](../plan/approval-and-amendments.md)
- Executor: Claude (Claude Code), with an independent completeness check of the notices (model output)
- Outcome: credits committed (`503dd8b`, `d77eb63`). A22 committed as `d3171e5` (follow-up `1efad95`). The suggested passphrase, the `generate_passphrase` command, the `eff-wordlist` crate (declared AGPL-3.0) and `random_below` are removed; the strength meter stays. `tsc`, `npm run build` and the Tauri-free core harness (73 tests) passed in the cloud container. On the owner's PC at `d3171e5` (throwaway data, 2026-10-23 evening): build 0 errors, `cargo test --lib` 127 of 127, clippy 0 errors and 5 library warnings, release build ok, real-exe scripts passing (optional security 52/52, restore flow 13/13) and 11 of 11 real-window checks. On `1efad95` the PC re-ran `cargo test --lib` 127 of 127, clippy, release build (exe SHA-256 `3FC9AD340788B84DF61FFF1990A9B5E02492EA873DEB4AB1D5532D6250C7C341`), optional security 52/52, restore flow 13/13 and the licenses dialog; lock-flow and passwords scripts not re-run. Not run: `testD1`, USB removal, idle, Win+L and sleep, the stable portable build, `cargo audit`.
- Acceptance: pending (the owner has not reviewed it)

### 2026-10-23 (18:32:20 to 18:47:35, owner message times; work times not recorded), EX-2026-10-23-17: Notes first: security and passwords optional (A21)

- Report: [2026-10-23-notes-first-optional-security.md](2026-10-23-notes-first-optional-security.md)
- Task / Goal: the owner's messages at 18:32:20, 18:34:11, 18:35:23, 18:39:02 (agreeing to three security confirmations), 18:40:15 and 18:47:35; amendment [A21](../plan/A21-notes-first-optional-security.md)
- Executor: Claude (Claude Code), with a README helper agent, two independent security review agents (read-only), a README fact-check agent and a PC remote session
- Outcome: Cipherleaf is a notes app first. A new vault is open (no password, never locks, screenshot protection off, not confidential). Security (master password, auto-lock, screenshot protection) and Passwords (KeePassXC) are separate switches in Settings. README rewritten. Built and pushed on branch `claude/project-thread-sv2tfm` (`84b281d`, `0929ae5`; draft [PR #3](https://github.com/AbyssWatcher2/CipherLeaf/pull/3)). Review fixes in `f9d2676` (turning security on or off re-keys the vault; restore confirmation; quit and Passwords safeguards). The Rust core passes 69 unit tests in a Tauri-free harness (13 new for A21); the UI type-checks and builds; screens seen in the mock UI. On the owner's PC at `5ef5b07` (2026-10-23, throwaway data): the app crate builds, `cargo test --lib` 130 of 130, clippy 0 errors and 5 library warnings, real-exe scripts passing (optional security 52 of 52, restore flow 13 of 13). Not run: `testD1`, anything on the owner's real vault, USB removal, idle, Win+L and sleep on a real PC, a portable build or ZIP, `cargo audit`.
- Integration / review: one independent security review (read-only agent) found 2 high, 3 medium and 6 low issues; the fixes are in `f9d2676` and "low 10" was not changed; a second independent review found no high issues and judged the fixes correct, with follow-ups in `dc5860a`; agent reports are model output; Claude checked the README against the code
- Acceptance: pending (the owner reviews A21 on a real PC)
- Open dependency / next action: the owner reviews draft PR #3 and A21 and merges when satisfied (a portable build and ZIP wait for the go-ahead); the owner also decides the saved screenshot setting of existing vaults and the restore-undo files; A2 and A8 decisions

### 2026-10-23 (17:04–17:35 UTC) — EX-2026-10-23-16 — Unlock after moving the portable folder; Start over, clearer errors, keyfile warning (A20)

- Report: [2026-10-23-start-over-and-unlock-wording.md](2026-10-23-start-over-and-unlock-wording.md)
- Task / Goal: a friend's report passed on by the owner (17:04:30 UTC): unlocking failed after moving the portable folder, and there was no way to start over. Approved: "Both" (17:16:33 UTC), "Push it run tests recheck" (17:26:10 UTC)
- Executor: Claude (Claude Code), cloud session plus a Remote Control session on the owner's PC
- Outcome: the move itself is not the cause (real-exe move test passed on throwaway data); added Start over (sets the vault aside, never deletes), clearer wrong-password wording and a keyfile warning
- Integration / review: one independent read-only review (found no data-loss path; its follow-ups are in `94d0235`); agent reports are model output
- Acceptance: pending
- Open dependency / next action: the owner's review (Rust tests and the real-exe check passed on the PC); PR #1 is merged into `main`; bringing it into `feat/a9-keepassxc` and a new portable build wait for the owner

### 2026-10-23 (after 14:17, until about 16:04) — EX-2026-10-23-15 — Back up and restore the KeePassXC passwords file; Quit never force-ends KeePassXC (A19), rebuild

- Report: [2026-10-23-passwords-backup-and-safe-quit.md](2026-10-23-passwords-backup-and-safe-quit.md)
- Task / Goal: the owner's request "include the KeePassXC passwords file in backups and restore, and stop Quit from force-closing KeePassXC mid-save" and their two answers ("Copy + restore, keep 10"; "Stay open and tell me") (message times not recorded; the clock read about 15:30 UTC at the exe build and 16:04 UTC after the ZIP); amendment [A19](../plan/A19-passwords-file-backup-restore-and-safe-quit.md)
- Executor: Claude (Sonnet 5.5), with helper agents (backend, UI, independent review, real-exe tests, records)
- Outcome: every lock keeps up to 10 copies of the encrypted passwords file (only when it changed); Settings, Backups can restore one (the current file is kept as `passwords-undo-*.kdbx`); Quit asks KeePassXC to close and never force-ends it (if it stays open, Quit is cancelled with a message). Committed (`442036e`); portable exe `1B8949CE…E7D6` and ZIP `F0C90846…C0A1` rebuilt (the owner's `data\` untouched, ZIP without it). 107 Rust tests, type check and build pass; real-exe tests on throwaway data passed in two stages (the first found two concurrency defects, fixed by a mutex; the re-run passed). Not tested: the new dialog mode and Settings button on screen, the file-dialog path, Quit with KeePassXC in the tray, log off or shutdown.
- Integration / review: one independent review (10 findings, 8 fixed); agent reports are model output; Claude checked the key claims in the code
- Acceptance: pending (the owner tries the new exe and reviews the security change)
- Open dependency / next action: the owner replaces the older ZIP copies and reviews A19 (a read-and-copy of the encrypted passwords file, a restore that replaces it, and a new Quit); A2 and A8 decisions

### 2026-10-23 (about 11:00–14:20) — EX-2026-10-23-14 — Readable first-run passphrase and restore from backup (A18), rebuild

- Report: [2026-10-23-restore-from-backup.md](2026-10-23-restore-from-backup.md)
- Task / Goal: the owner's choice of "the first-run passphrase fix ... and the in-app restore from backup" and the later "Finish the restore work" (message times not recorded; the clock read 13:51 UTC after the earlier push, about 14:14 at the build and 14:17 after the ZIP); amendment [A18](../plan/A18-readable-passphrase-and-restore-from-backup.md)
- Executor: Claude (Sonnet 5.5), with helper agents (backend, UI, independent reviewers, visual check, records)
- Outcome: the suggested passphrase shows in plain text with a Copy button; a backup can be restored from the lock screen, the create-vault screen and Settings (the old vault is kept as `restore-undo-*.clf`). Committed (`1273093`); portable exe `7BB5303F…F3A1` and ZIP rebuilt (the owner's `data\` untouched, ZIP without it). 68 Rust tests, type check and build pass; real-exe tests on throwaway vaults passed for the restore (locked and unlocked, wrong password, refused paths) and for the lock regression. Not tested: the file-dialog path end to end, the new screens in the real window.
- Integration / review: three review rounds; agent reports are model output; Claude checked the key claims in the code
- Acceptance: pending (the owner tries the new exe)
- Open dependency / next action: the owner replaces the older ZIP copies and reviews A18 (a new way to replace the vault file); A2 and A8 decisions

### 2026-10-23 (about 08:28–09:05) — EX-2026-10-23-13 — Code review, lock and data-loss fixes (A17), rebuild

- Report: [2026-10-23-lock-and-data-loss-fixes.md](2026-10-23-lock-and-data-loss-fixes.md)
- Task / Goal: the owner's messages "re review the codes and apps", "use all skills and agents and sub agents" and "Yes, fix the lock and data-loss problems first ..., then rebuild and commit" (message times not recorded; the clock read 08:28:27 at the start and 09:05:27 after the ZIP); amendment [A17](../plan/A17-lock-and-data-loss-fixes.md)
- Executor: Claude (Sonnet 5.5), with helper agents (review, fix, independent review, records)
- Outcome: text typed just before Lock or close is now saved first (waits at most 2 seconds); a safer lock order; a failed save is rolled back or retried; KeePassXC lock and re-show fixes; sticky delete confirm; calculator +/- fixed. Committed (`dc95fd3`); portable exe and ZIP rebuilt (the owner's `data\` untouched, ZIP without it). 39 Rust tests, type check, build and calculator checks pass; a real-exe test on a throwaway vault passed (the old build failed it). Idle lock, Win+L, sleep and drive removal were not tested. A backlog of findings not fixed is in the report.
- Integration / review: agent reports are model output; Claude checked the top findings against the code
- Acceptance: pending (the owner tries the new exe)
- Open dependency / next action: the owner replaces the older ZIP copies and picks which backlog items come next; A2 and A8 decisions

### 2026-10-22 (23:40–23:55) — EX-2026-10-22-12 — Wider note page, Paper as the default look (A16), commit and rebuild

- Report: [2026-10-22-page-width-and-paper-default.md](2026-10-22-page-width-and-paper-default.md)
- Task / Goal: the owner's messages "make the page bigger it's too tight", "the paper theme make it as default and you can refine a bit" and "Commit and rebuild the portable build and ZIP"; amendment [A16](../plan/A16-page-width-and-paper-default.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: wider editor widths plus a Full option; Home and notebook pages follow the width setting; Paper is the default and slightly refined; committed (`3a22c56`); the portable exe and ZIP rebuilt, the owner's `data\` folder in the portable folder left untouched, the ZIP made without it
- Integration / review: not applicable
- Acceptance: pending (the owner's look at the real window)
- Open dependency / next action: the owner tries the build and says what to change

### 2026-10-22 (23:24–23:40) — EX-2026-10-22-11 — Calculator border fix, Cipherleaf app icon (A15), rebuilt portable build

- Report: [2026-10-22-icon-border-fix-and-rebuild.md](2026-10-22-icon-border-fix-and-rebuild.md)
- Task / Goal: the owner's messages at 23:24:31 (white border), 23:26:19 (icon) and "rebuild and commit"; amendment [A15](../plan/A15-app-icon.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: border removed in both calculators; the Cipherleaf icon generated from `assets/cipherleaf-icon.svg` and used by the exe and window; the exe, portable folder and ZIP rebuilt; smoke-tested; committed
- Integration / review: not applicable
- Acceptance: pending (the owner's check of the taskbar icon and the calculator)
- Open dependency / next action: the owner tries the build; A2 and A8 decisions

### 2026-10-22 (23:14–23:25) — EX-2026-10-22-10 — Fresh portable build; KeePassXC kept for private use

- Report: [2026-10-22-fresh-portable-build.md](2026-10-22-fresh-portable-build.md)
- Task / Goal: the owner's instruction at 23:14:05 (fresh vault, fresh portable build, keep KeePassXC for private use)
- Executor: Claude (Sonnet 5.5)
- Outcome: the exe was rebuilt from the committed code and the portable folder refreshed with no `data` folder (a fresh vault on first run); smoke-tested again (starts, `data` next to the exe, nothing in AppData, no network, closes cleanly). The dev vault was not copied.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on use)
- Open dependency / next action: the owner tries the exe; A2 and A8 decisions

### 2026-10-22 (22:58–23:15) — EX-2026-10-22-09 — Commit of A10–A14 and the first release build

- Report: [2026-10-22-release-build-and-commit.md](2026-10-22-release-build-and-commit.md)
- Task / Goal: the owner's instruction at 22:58:30 (commit; build the exe for release; screenshot note)
- Executor: Claude (Sonnet 5.5)
- Outcome: code committed (`2105a57`); release exe built; portable folder assembled outside the repository with the verified KeePassXC, its licenses and source; smoke-tested from a temporary copy (starts, `data` next to the exe, nothing in AppData, no network, closes cleanly). A vault was not created and the Passwords view not opened in the release build.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on use)
- Open dependency / next action: the owner tries the exe; move the dev vault into the portable folder or start fresh; the GPL approach before sharing (STATUS questions 4 and 9)

### 2026-10-22 (22:40–22:55) — EX-2026-10-22-08 — A14: a small calculator inside each sticky note

- Report: [2026-10-22-a14-calculator-in-sticky-notes.md](2026-10-22-a14-calculator-in-sticky-notes.md)
- Task / Goal: the owner's instruction at 22:40:13; amendment [A14](../plan/A14-calculator-in-sticky-notes.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: a calculator button in each sticky note opens a small basic calculator in the note's colors; shared logic moved into a hook. Type check, build, audit, 33 Rust tests and 66 engine checks pass; works in the mock UI; real sticky window not tried.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the reading was confirmed by the owner at 22:55:26 UTC; commit decision (STATUS question 1)

### 2026-10-22 (22:26–22:38) — EX-2026-10-22-07 — A13: calculator, reworded and trimmed editor

- Report: [2026-10-22-a13-calculator-and-wording.md](2026-10-22-a13-calculator-and-wording.md)
- Task / Goal: the owner's instruction at 22:26:02 and answers at 22:27:00; amendment [A13](../plan/A13-calculator-and-wording.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: calculator built under Sticky notes (basic and scientific); block editor reworded and trimmed with older content still displayed; the name stays Cipherleaf. Engine checks (66), type check, build, audit and 33 Rust tests pass; mock-UI checks pass; the real app was not used.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the owner starts the dev app and reviews A10–A13; commit and portable-build decisions (STATUS questions 1 and 4)

### 2026-10-22 (21:47 onward) — EX-2026-10-23-06 — PBS reconciliation

- Report: [2026-10-23-pbs-reconciliation.md](2026-10-23-pbs-reconciliation.md) (the filename uses the owner's local date)
- Task / Goal: the owner's request at 21:47:32 UTC to reconcile the project under the PBS skill; documentation only
- Executor: Claude (Sonnet 5.5)
- Outcome: records reconciled with the tree checked at 21:49 UTC; acceptance ledger, session history and README added; stale facts corrected. Follow-up at 22:03: the owner approved the CLAUDE.md additions, this index and the failure-intelligence record, now done.
- Integration / review: not applicable (no workers)
- Acceptance: pending (the owner hasn't reviewed the records)
- Open dependency / next action: the owner's product questions in STATUS; the decision-record proposal was not adopted

### 2026-10-22 (17:29–17:55) — EX-2026-10-22-05 — A12: Turn into list, toolbar, sub-pages, fonts, equations

- Report: [2026-10-22-a12-notion-menus-subpages.md](2026-10-22-a12-notion-menus-subpages.md)
- Task / Goal: the owner's request at 17:29:57; KaTeX approved at 17:44:21; amendment [A12](../plan/A12-notion-menus-subpages.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented and checked by unit tests, scripts and screenshots (mock data); partly unverified against the real backend and by hand
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the owner's decision about trimming or rewording Notion-like features (STATUS question 2)

### 2026-10-22 (16:55–17:25) — EX-2026-10-22-04 — A11: new look and Notion-style notes with pictures

- Report: [2026-10-22-a11-redesign-and-block-notes.md](2026-10-22-a11-redesign-and-block-notes.md)
- Task / Goal: the owner's request at 16:55:27; amendment [A11](../plan/A11-redesign-and-block-notes.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented; pictures, covers and icons also checked against a throwaway real vault
- Integration / review: not applicable
- Acceptance: pending; the owner's screenshot with "happy" (18:06:46) is informal only
- Open dependency / next action: as for EX-05

### 2026-10-22 (16:12–16:50) — EX-2026-10-22-03 — A10: KeePassXC inside the window, black-and-white theme, font size

- Report: [2026-10-22-a10-embed-and-appearance.md](2026-10-22-a10-embed-and-appearance.md)
- Task / Goal: the owner's request at 16:12:20; amendment [A10](../plan/A10-embed-and-appearance.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented; embedding checked by script on a throwaway vault; typing inside the embedded KeePassXC not checked
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: the owner's hands-on check of the Passwords view

### 2026-10-22 (13:08–16:10) — EX-2026-10-22-02 — A9: bundled KeePassXC (D0 and D1) and first launch

- Report: [2026-10-22-a9-keepassxc-d0-d1.md](2026-10-22-a9-keepassxc-d0-d1.md)
- Task / Goal: [A9](../plan/A9-bundle-keepassxc.md), approved at 13:07:49
- Executor: Claude (Opus 5.5, then Sonnet 5.5 for the resumed work)
- Outcome: KeePassXC 2.8.0 downloaded and verified (hash and signature); Cipherleaf's own password code replaced by a panel; first launch worked; debug-port exposure recorded
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: the GPL approach before distribution (A9 Q5). The debug-port exposure is now [FI-01](../failure-intelligence.md#fi-01--debug-port-left-open-on-the-owners-live-window).

### 2026-10-22 (11:08–11:51) — EX-2026-10-22-01 — Toolchain, scaffold and first full implementation

- Report: [2026-10-22-initial-build.md](2026-10-22-initial-build.md)
- Task / Goal: the approved plan ([snapshot](../plan/approved-plan-2026-10-22.md)), approved at 11:08:21
- Executor: Claude (Sonnet 5.5 for planning and the scaffold; Opus 5.5 for app code)
- Outcome: code written for the planned features; 27 unit tests passed at 11:37; the UI type-checked; never run at that point
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: A2 and A8 decisions; A1 and A3–A7 evidence
