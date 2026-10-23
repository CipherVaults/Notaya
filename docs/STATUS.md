# Cipherleaf: status

This file answers two questions: what is accepted right now, and exactly where work resumes. The two parts stay separate. Update it at meaningful checkpoints, and link to other records rather than copying them.

- **As of:** 2026-10-23, about 16:04 UTC (system clock, when the portable ZIP was made), after the backup and restore of the KeePassXC passwords file and the safe Quit ([A19](plan/A19-passwords-file-backup-restore-and-safe-quit.md), [EX-15](execution/2026-10-23-passwords-backup-and-safe-quit.md)). A19 was committed (`442036e`) and the portable build and ZIP were rebuilt. The commit after `442036e` holds the records; `main` and `feat/a9-keepassxc` are pushed to the private GitHub repository after it. Before that: the readable first-run passphrase and restore from backup ([A18](plan/A18-readable-passphrase-and-restore-from-backup.md), [EX-14](execution/2026-10-23-restore-from-backup.md), `1273093`) and the lock and data-loss fixes ([A17](plan/A17-lock-and-data-loss-fixes.md), [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md), `dc95fd3`).
- **Owner and decision-maker:** Mohamed Sayed
- **Maintained by:** Claude (Claude Code), for the owner

## Current state (accepted present)

- **Mission state:** **waiting for the owner to try the new portable build and the dev app and review A10–A19.** A10–A19 are committed on branch `feat/a9-keepassxc` (A19 in `442036e`; the records for it in the next commit) and unaccepted. The portable folder and its ZIP were rebuilt with everything through A19. The owner's own vault is in the portable folder's `data\` folder and was not touched (the ZIP does not include it). **The copy of the ZIP the owner put on their Drive is outdated** (it still loses text typed just before Lock or close, and has no restore or passwords backup); the owner should replace it with the new ZIP. The dev app is not running.
- **Accepted baseline:** none. The owner has not accepted any result. One informal signal is recorded in [acceptance.md](acceptance.md) (the owner's "happy" with a screenshot, 18:06:46 UTC); it is not an acceptance.
- **Active authority:** the approved plan ([snapshot](plan/approved-plan-2026-10-22.md)), amendment A9 (approved 13:07:49 UTC), and the owner's later instructions applied as A10 ([record](plan/A10-embed-and-appearance.md)), A11 ([record](plan/A11-redesign-and-block-notes.md)) and A12 ([record](plan/A12-notion-menus-subpages.md)) and A13 ([record](plan/A13-calculator-and-wording.md)) and A14 ([record](plan/A14-calculator-in-sticky-notes.md)), then A15–A19 (A17: [record](plan/A17-lock-and-data-loss-fixes.md); A18: [record](plan/A18-readable-passphrase-and-restore-from-backup.md); A19: [record](plan/A19-passwords-file-backup-restore-and-safe-quit.md)), as listed with exact times in [approval-and-amendments.md](plan/approval-and-amendments.md). Standing rules: nothing is committed until the owner says; no release build without their go-ahead.
- **Waiting on the owner:** see "Open questions for the owner" below.

## Open questions for the owner

Raised by the owner or by Claude since A12; the struck-through ones are answered.

| # | Question | Raised | What it blocks |
|---|---|---|---|
| 1 | ~~Commit A10–A12?~~ **Answered 22:58:30 UTC: commit.** Done: code commit `2105a57`, then a documentation commit (see `git log`). (At that time there was no remote; the private GitHub repository was added on 2026-10-23, see the Git and GitHub row.) | 16:50 UTC | Nothing |
| 2 | ~~Reword, or reword and trim, the Notion-like features?~~ **Answered 22:27 UTC: reword and trim.** Done as [A13](plan/A13-calculator-and-wording.md); awaiting the owner's review of how it reads. | 17:56 UTC | Nothing |
| 3 | ~~A calculator under Sticky notes: basic or scientific?~~ **Answered 22:27 UTC: basic and scientific.** Built (A13); awaiting review. | 17:56 UTC | Nothing |
| 4 | ~~Portable build: copy the dev vault or start fresh?~~ **Answered 23:14:05 UTC: fresh vault, fresh portable build.** Done ([EX-10](execution/2026-10-22-fresh-portable-build.md)); the dev vault in `.devdata\data\` was not copied and is untouched. | 18:08 UTC | Nothing |
| 5 | ~~Name and icon~~ **Answered 22:26 UTC: keep Cipherleaf.** No rename and no change of the app identifier. An icon change was not asked for. | 18:09 UTC | Nothing |
| 5b | ~~Is a calculator inside each sticky note what the owner wanted (A14)?~~ **Answered 22:55:26 UTC: yes.** | 22:55 UTC | Nothing |
| 6 | ~~Is screenshot blocking turned back on in the dev vault?~~ **Answered 22:58:30 UTC: left off for now, and "it's working, no worries".** Recorded as the owner's choice; the setting is off in their dev vault. | 17:59 UTC | Nothing |
| 7 | What did "happy" accept? | 18:06 UTC | Recording any acceptance |
| 8 | Confirm that [acceptance.md](acceptance.md) and [sessions.md](sessions.md) join the compact record set, and say whether Decision Records are wanted (Claude recommends not unless asked). The CLAUDE.md additions, the execution index and the failure-intelligence record were approved at 22:03:12 UTC and are done. | 22:00 UTC | Nothing immediate |
| 9 | Still open from before: A2 (single-record rollback) and A8 (version pinning). **The GPL approach (A9 Q5) was answered at 23:14:05 UTC:** keep KeePassXC bundled; the build is for the owner, close people and friends, nothing public; each copy keeps its license files, source and README.txt. | 11:08–23:14 UTC | Acceptance (A2, A8) |
| 10 | Which of the findings not fixed should come next? They are listed under "Findings not fixed (backlog)" in [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md#findings-not-fixed-backlog), with product ideas (the suggested passphrase in plain text and restore from a backup inside the app were done in A18; backing up the KeePassXC passwords file and Quit no longer force-ending KeePassXC were done in A19; still open: the others in that list). | 2026-10-23 | Nothing immediate |
| 11 | Does the owner want the repository to get a license file, and may it ever become public? (Asking first still applies to both.) Also: replace the outdated ZIP on the Drive. | 2026-10-23 | Nothing immediate |

## Continuation (unaccepted work and how to resume)

### What exists

| Item | Where | State | Evidence and its limits |
|---|---|---|---|
| Toolchain | Node 24.20.0, npm 11.19.0, Rust 1.99.1 (MSVC), Git 2.55.0, VS Build Tools 2022 | Complete | `--version` checks, 2026-10-22 |
| Rust backend | `src-tauri/src/` | Implemented; unit-tested | `cargo test --lib`: 107/107 on commit `442036e` (2026-10-23; 39 new tests for A19: passwords copies, passwords restore, rotation, the notice, `close_gracefully`; 68 before); `cargo clippy --all-targets`: only the 3 old warnings. Unit level only. |
| UI | `src/` | Implemented; type-checked, bundled | `tsc`: 0 errors and `npm run build`: passed on `442036e`; `npm audit`: 0 vulnerabilities (earlier on 2026-10-23, not repeated after A18 or A19). The new passphrase box and restore dialog (A18) were seen in the mock UI only, not in the real window. The "passwords" mode of the restore dialog and its Settings button (A19) were never seen on screen, not even in the mock (type-checked and built only). The calculator engine passes 66 checks (`dev-mock/calc-check.cjs`); the small calculator inside a sticky note (A14) was checked in the mock UI only. Screens, the editor and the calculator were checked with mock data and scripts; pictures, covers and icons against the real backend in a throwaway vault (before A13); KeePassXC placement by script. Not checked by hand or in the real app: the calculator (including Copy), the reworded menus and toolbar, real mouse dragging, the picture file chooser, the sub-page flow, typing inside KeePassXC. |
| KeePassXC bundle (A9) | `vendor/` (ignored by Git), `vendor/keepassxc-manifest.md` (committed) | D0 and D1 done | SHA-256 and PGP verified; offline launch test passed; see [EX-02](execution/2026-10-22-a9-keepassxc-d0-d1.md). After a fresh clone the binaries must be downloaded again. |
| Mock UI harness | `dev-mock/`, `vite.mock.config.ts`, `npm run mock` | Dev only (uncommitted) | Fakes the Tauri commands so screens can be viewed in a browser; not part of the app build. |
| Records | `docs/`, `CLAUDE.md`, `README.md` | Reconciled at 21:49–22:10 UTC; also new: [execution index](execution/README.md), [failure-intelligence.md](failure-intelligence.md) | [EX-06](execution/2026-10-23-pbs-reconciliation.md). New: [acceptance.md](acceptance.md), [sessions.md](sessions.md). |
| Handoff and test scripts | `docs/handoff/2026-10-23-handoff-checkpoint.md` (what to do next, what not to repeat, a first message for a new thread) and `docs/handoff/2026-10-23-project-history.md` (the whole history); the real-exe test scripts now live in `tools/real-exe-tests/` and `tools/mock-screenshots/` | Handoff is **prepared** (2026-10-23, about 16:50 UTC), not delivered or acknowledged. The scripts were moved out of the session scratchpad, made portable and given a random throwaway passphrase; vault restore, lock flow and passwords group A were re-run from the new location (all passed); passwords groups B–E and the mock screenshots were not re-run there. | `docs/handoff/`, `tools/real-exe-tests/README.md` |
| Git and GitHub | Private repository <https://github.com/AbyssWatcher2/CipherLeaf> (remote `origin`, 2026-10-23); branches `main` and `feat/a9-keepassxc`; after the first push both were at `8bd8ee8` | Code commit `442036e` (A19: passwords-file backup and restore, safe Quit; 13 files, 1477 insertions, 66 deletions), on top of `a2eebf1` (records for A18); the commit after `442036e` holds the A19 records. `main` and `feat/a9-keepassxc` are pushed to GitHub after that records commit (check `git status` and `gh repo view` for the actual state). Earlier: `1273093` (A18 code), `dc95fd3` (A17 code), `2105a57` (code, A10–A14), `3a22c56` (A16). The restore work that was paused and uncommitted is now committed. GitHub CLI 2.103.0 (`gh`) is installed and signed in as `AbyssWatcher2`. Asking first still applies to: making the repository public, force-pushing, deleting remote branches, a license. | `git log`, `git status`, `gh repo view` |
| Release build (rebuilt with A19: passwords-file backup and restore, safe Quit; exe finished about 15:30 UTC on 2026-10-23, ZIP made 16:04 UTC) | `E:\Project\Cipherleaf-portable\` (outside the repository): `Cipherleaf.exe` (SHA-256 `1B8949CE…E7D6`), `KeePassXC\`, `Licenses\KeePassXC\` (licenses and the complete source), `README.txt`. **The folder also has the owner's `data\` folder** (their vault, from their own run; 305 files, `vault.clf` last written 2026-10-23 02:54:14 local). It was not touched by the rebuild: same file count, size and stamp afterwards. ZIP of the same folder without `data\`: `E:\Project\Cipherleaf-portable.zip`, 83.1 MB, 169 entries, SHA-256 `F0C90846…C0A1` (made 16:04 UTC). **The ZIPs the owner uploaded to their Drive earlier (for example `1AD64164…D063`, exe `F4F1E679…`) are outdated** (lost-text problem, no restore, no passwords backup), and so is the one from before A17. | Built from commit `442036e` (all code through A19) | [EX-15](execution/2026-10-23-passwords-backup-and-safe-quit.md): in the real exe on throwaway data, the passwords backups passed (90 locks each with a changed file: exactly 1 copy added and 1 removed at 10; 32 of 32 first locks made a copy; the 13 backup rules), the passwords restore passed (41 cases, including 15 refusals that changed nothing, undo files, no database yet), Quit passed (a cancelled Quit with KeePassXC waiting in a dialog: both stay open, vault locked, exact message; a second close after KeePassXC was closed exited in under 1 s; the `quit_app` command; the final copy after Quit), and the earlier vault restore and lock tests passed again. A first run on an earlier build found two concurrency defects (fixed by a mutex, re-tested). Not tested: the "Restore passwords file..." dialog and Settings button on screen, the native file-dialog path for passwords backups, Quit with KeePassXC in the tray or hidden, Windows log off or shutdown, the vault file-dialog path, the new vault-restore screens in the real window, a crash between the two renames, idle lock, Win+L, sleep, drive removal, the KeePassXC lock and re-show paths, the sticky delete confirm, calculator +/- in a real window, the Passwords view, a USB stick (V3c), the icon in the real taskbar. Unsigned. Earlier exe hashes (`7BB5303F…`, `D237C5B5…`, `F4F1E679…`, `6129922C…`, `5D766105…`, `A10AAB35…`) are superseded. |
| Owner's dev vault | `.devdata\data\` (ignored by Git) | Real use began 16:03 UTC; contents not inspected by Claude (the owner's screenshot at 18:06 shows a page "Hello Cipherleaf" and two "Untitled" pages) | File timestamps only |
| Test leftovers | About 88 `cipherleaf-*` folders in `%TEMP%` (cargo test vaults, KeePassXC test copies) | Not deleted (the project rule is to ask before deleting) | Folder listing |

### Not done

- Hands-on use of the running app by the owner: the new look, the calculator under Sticky notes, the reworded block editor (drag, "/" menu, toolbar, pictures, equations), sticky notes, auto-lock, screenshot blocking, typing in the embedded KeePassXC, quitting (KeePassXC now closes politely, and a Quit is cancelled with a message if it stays open), the command palette hiding the overlay, the real vault's migration from the old theme names, scaling other than 100%.
- A18 checks not yet run: choosing a backup with the native file dialog and restoring it, the restore dialog and passphrase box in the real window, a crash between the two renames, and a security review of the new restore command. The known limits are listed in [security-design.md](security-design.md).
- A19 checks not yet run: the "Restore passwords file..." dialog and Settings button on screen (not even in the mock), choosing a passwords backup with the native file dialog, restoring over a database with another name than `passwords.kdbx` (unit tests only), Quit with KeePassXC in the system tray or hidden, Windows log off or shutdown during Quit, a crash between the two renames of a passwords restore, and a security review of the new commands. The known limits are listed in [security-design.md](security-design.md).
- Release build checks not yet run: the Passwords view in the release exe, V3c (pulling the USB drive locks the app), a real USB stick, idle lock, Win+L and sleep with the A17 saving step, running it on a PC without the VC++ runtime, code signing and SmartScreen.
- The findings not fixed after the 2026-10-23 review (backlog in [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md#findings-not-fixed-backlog)). Two of them are now done in A19 ("backups do not include the passwords file" and "Quit force-kills KeePassXC after 3 s"); the rest are still open.
- `cargo audit` and a security review (`npm audit` is done).
- A1 and A3–A7 evidence.
- Items listed in the [acceptance ledger](acceptance.md) as not-run.

### Observed in the owner's first hands-on run (from files only, ~16:03–16:07 UTC)

The owner ran the app and created things before the restart. Claude didn't watch the run; this comes from file evidence in `.devdata\data\`:

- `keepassxc\keepassxc.ini` (16:03) and `Passwords.kdbx` (16:04, valid KeePass signature `03 D9 A2 9A`): the KeePassXC panel started the bundled KeePassXC, which created a database in Cipherleaf's data folder.
- `vault.clf` (16:05, a valid SQLite file) and `backups\vault-….clf` (16:06): a Cipherleaf vault was created, and a lock wrote its encrypted backup.
- Later owner evidence: a screenshot at 18:06:46 UTC showing the running app with the Paper theme (so the redesign runs in their vault, and screenshot blocking was off at that moment).

### Blockers and decisions

- **Debug-port exposure (closed at 16:07 UTC): see [FI-01](failure-intelligence.md#fi-01--debug-port-left-open-on-the-owners-live-window).** The one action still open is the owner's: if the dev vault will hold real data, change its master password (Settings → Master password & keyfile) or create a fresh vault. Later UI testing used a throwaway vault with the port closed afterwards.
- A2 and A8 need the owner's decision. A1 and A3–A7 need evidence. See [the amendments record](plan/approval-and-amendments.md).

### Do not repeat

- The toolchain install, scaffolding, dependency selection, or the D0 download and verification.
- Rewriting backend or UI modules from scratch. Inspect and fix instead.
- Editing the plan snapshot. Record changes as amendments instead.
- Driving the running dev window by script while the owner may be using it (FI-01). Use a throwaway vault in a separate process, and close its debug port afterwards.
- Editing `vite.config.ts` while `tauri dev` runs (it ends the session). The `.devdata` watcher ignore is already done.
- Force-stopping KeePassXC (it protects itself); close it politely.

### Exact next permitted action

1. The owner runs the rebuilt `E:\Project\Cipherleaf-portable\Cipherleaf.exe` (it opens the vault already in its `data\` folder) and the dev app, tries a restore from a backup if they want to (the vault restore replaces the vault file, the old one is kept as `restore-undo-*.clf`; Settings, Backups, "Restore passwords file..." replaces the passwords file, the old one is kept as `passwords-undo-*.kdbx`; the passwords restore works only while Cipherleaf is unlocked and asks KeePassXC to close first), tries a Quit with KeePassXC open, replaces the older ZIP copies (for example the one on the Drive) with `E:\Project\Cipherleaf-portable.zip`, and says what to change or which backlog items come next. Anything that goes to other people keeps the `Licenses` folder and `README.txt` with it.
2. Documentation upkeep is authorized now: after any meaningful step, follow the routine in [EX-06 Appendix A](execution/2026-10-23-pbs-reconciliation.md#appendix-a-documentation-routine-going-forward).
3. Code changes, a commit, a release build, any change to security behavior or a new dependency wait for the owner's instruction on the matching question.
4. When the owner says to continue: start the dev app without the debug port, let them test, fix what they report, re-run `cargo test --lib`, `npx tsc --noEmit -p tsconfig.json` and `npm run build`, and add an Execution Summary.

## Where things live

| Question | Where the answer lives |
|---|---|
| What is authorized now? | The owner's latest instruction, recorded with exact times in [approval-and-amendments.md](plan/approval-and-amendments.md) |
| What was approved, and its success criteria? | [approved-plan-2026-10-22.md](plan/approved-plan-2026-10-22.md), the unchanged snapshot |
| Which criteria are met by which evidence, and who accepted? | [acceptance.md](acceptance.md) |
| What's accepted, and where does work resume? | This file |
| What happened, in order, and why did choices change? | [sessions.md](sessions.md); the amendments record; the Claude Code transcripts (outside the project) |
| What did a piece of work do, and what was checked? | [`execution/`](execution/); the [index](execution/README.md) lists every report, newest first |
| What went wrong or nearly did, and what rule prevents it? | [failure-intelligence.md](failure-intelligence.md) |
| What does the code actually do? | The working tree and its Git history |
| Did checks pass? | Re-running the checks; dated results in [`execution/`](execution/) and [acceptance.md](acceptance.md) |
| What does Cipherleaf promise about security? | [security-design.md](security-design.md) |
| How should an AI agent work here? | [`CLAUDE.md`](../CLAUDE.md) |
| Notion pages (private drafts in the owner's workspace, written earlier the same day; historical, not kept current) | [PBS Understanding Review — Cipherleaf](https://app.notion.com/p/40190e048b6081f3b301ee6f3a8c4145), [Chat Response (22 Oct 2026)](https://app.notion.com/p/40190e048b6081c4beb9d195082ace61), [Work Log & Checkpoint (22 Oct 2026)](https://app.notion.com/p/40190e048b608176b1a8ff4fcb7afec7) (the work log stops at 13:26 UTC, before A10) |

**Reading order to resume:** `CLAUDE.md` → this file → the amendments record → the live repository (`git status`, `git log`) → the acceptance ledger → the [execution index](execution/README.md) and the latest Execution Summary → the newest session entry, the security design or the plan snapshot, as needed.
