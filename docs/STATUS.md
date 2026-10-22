# Cipherleaf: status

This file answers two questions: what is accepted right now, and exactly where work resumes. The two parts stay separate. Update it at meaningful checkpoints, and link to other records rather than copying them.

- **As of:** 2026-10-22, about 23:40 UTC, after the calculator border fix, the new app icon (A15) and a rebuilt portable build ([EX-11](execution/2026-10-22-icon-border-fix-and-rebuild.md)). The commit that follows records them.
- **Owner and decision-maker:** Mohamed Sayed
- **Maintained by:** Claude (Claude Code), for the owner

## Current state (accepted present)

- **Mission state:** **waiting for the owner to try the portable build and the dev app and review A10–A15.** All are committed on branch `feat/a9-keepassxc` once the commit after EX-11 is made (see `git log`) and unaccepted. The portable folder and its ZIP were rebuilt with the Cipherleaf icon and the calculator fix; the folder has no `data\` folder, so the first run creates a fresh vault. The dev app is not running.
- **Accepted baseline:** none. The owner has not accepted any result. One informal signal is recorded in [acceptance.md](acceptance.md) (the owner's "happy" with a screenshot, 18:06:46 UTC); it is not an acceptance.
- **Active authority:** the approved plan ([snapshot](plan/approved-plan-2026-10-22.md)), amendment A9 (approved 13:07:49 UTC), and the owner's later instructions applied as A10 ([record](plan/A10-embed-and-appearance.md)), A11 ([record](plan/A11-redesign-and-block-notes.md)) and A12 ([record](plan/A12-notion-menus-subpages.md)) and A13 ([record](plan/A13-calculator-and-wording.md)) and A14 ([record](plan/A14-calculator-in-sticky-notes.md)), as listed with exact times in [approval-and-amendments.md](plan/approval-and-amendments.md). Standing rules: nothing is committed until the owner says; no release build without their go-ahead.
- **Waiting on the owner:** see "Open questions for the owner" below.

## Open questions for the owner

Raised by the owner or by Claude since A12; none has been answered yet.

| # | Question | Raised | What it blocks |
|---|---|---|---|
| 1 | ~~Commit A10–A12?~~ **Answered 22:58:30 UTC: commit.** Done: code commit `2105a57`, then a documentation commit (see `git log`). No push, no remote. | 16:50 UTC | Nothing |
| 2 | ~~Reword, or reword and trim, the Notion-like features?~~ **Answered 22:27 UTC: reword and trim.** Done as [A13](plan/A13-calculator-and-wording.md); awaiting the owner's review of how it reads. | 17:56 UTC | Nothing |
| 3 | ~~A calculator under Sticky notes: basic or scientific?~~ **Answered 22:27 UTC: basic and scientific.** Built (A13); awaiting review. | 17:56 UTC | Nothing |
| 4 | ~~Portable build: copy the dev vault or start fresh?~~ **Answered 23:14:05 UTC: fresh vault, fresh portable build.** Done ([EX-10](execution/2026-10-22-fresh-portable-build.md)); the dev vault in `.devdata\data\` was not copied and is untouched. | 18:08 UTC | Nothing |
| 5 | ~~Name and icon~~ **Answered 22:26 UTC: keep Cipherleaf.** No rename and no change of the app identifier. An icon change was not asked for. | 18:09 UTC | Nothing |
| 5b | ~~Is a calculator inside each sticky note what the owner wanted (A14)?~~ **Answered 22:55:26 UTC: yes.** | 22:55 UTC | Nothing |
| 6 | ~~Is screenshot blocking turned back on in the dev vault?~~ **Answered 22:58:30 UTC: left off for now, and "it's working, no worries".** Recorded as the owner's choice; the setting is off in their dev vault. | 17:59 UTC | Nothing |
| 7 | What did "happy" accept? | 18:06 UTC | Recording any acceptance |
| 8 | Confirm that [acceptance.md](acceptance.md) and [sessions.md](sessions.md) join the compact record set, and say whether Decision Records are wanted (Claude recommends not unless asked). The CLAUDE.md additions, the execution index and the failure-intelligence record were approved at 22:03:12 UTC and are done. | 22:00 UTC | Nothing immediate |
| 9 | Still open from before: A2 (single-record rollback) and A8 (version pinning). **The GPL approach (A9 Q5) was answered at 23:14:05 UTC:** keep KeePassXC bundled; the build is for the owner, close people and friends, nothing public; each copy keeps its license files, source and README.txt. | 11:08–23:14 UTC | Acceptance (A2, A8) |

## Continuation (unaccepted work and how to resume)

### What exists

| Item | Where | State | Evidence and its limits |
|---|---|---|---|
| Toolchain | Node 24.20.0, npm 11.19.0, Rust 1.99.1 (MSVC), Git 2.55.0, VS Build Tools 2022 | Complete | `--version` checks, 2026-10-22 |
| Rust backend | `src-tauri/src/` | Implemented; unit-tested | `cargo test --lib`: 33/33 on the working tree at 22:35 UTC, uncommitted (no Rust changed in A13). Unit level only. |
| UI | `src/` | Implemented; type-checked, bundled | `tsc`: 0 errors; `npm run build`: passed; `npm audit`: 0 vulnerabilities, all at 22:35 UTC. The calculator engine passes 66 checks (`dev-mock/calc-check.cjs`); the small calculator inside a sticky note (A14) was checked in the mock UI only. Screens, the editor and the calculator were checked with mock data and scripts; pictures, covers and icons against the real backend in a throwaway vault (before A13); KeePassXC placement by script. Not checked by hand or in the real app: the calculator (including Copy), the reworded menus and toolbar, real mouse dragging, the picture file chooser, the sub-page flow, typing inside KeePassXC. |
| KeePassXC bundle (A9) | `vendor/` (ignored by Git), `vendor/keepassxc-manifest.md` (committed) | D0 and D1 done | SHA-256 and PGP verified; offline launch test passed; see [EX-02](execution/2026-10-22-a9-keepassxc-d0-d1.md). After a fresh clone the binaries must be downloaded again. |
| Mock UI harness | `dev-mock/`, `vite.mock.config.ts`, `npm run mock` | Dev only (uncommitted) | Fakes the Tauri commands so screens can be viewed in a browser; not part of the app build. |
| Records | `docs/`, `CLAUDE.md`, `README.md` | Reconciled at 21:49–22:10 UTC; also new: [execution index](execution/README.md), [failure-intelligence.md](failure-intelligence.md) | [EX-06](execution/2026-10-23-pbs-reconciliation.md). New: [acceptance.md](acceptance.md), [sessions.md](sessions.md). |
| Git | Local repository, no remote; branch `feat/a9-keepassxc`; `main` untouched | HEAD is the documentation commit after `2105a57` (code, A10–A14); see `git log`. No push. | `git log`, `git status` |
| Release build (rebuilt with the icon and the calculator fix, 23:30–23:33 UTC) | `E:\Project\Cipherleaf-portable\` (outside the repository): `Cipherleaf.exe` (SHA-256 `6129922C…2756`), `KeePassXC\`, `Licenses\KeePassXC\` (licenses and the complete source), `README.txt`; no `data\` folder. ZIP of the same folder: `E:\Project\Cipherleaf-portable.zip`, 83.1 MB, SHA-256 `2440EC72…7FD5` (made at about 23:33 UTC; the earlier ZIP, made at 23:22 UTC, is replaced) | Built from the committed code plus the A15 icon and the border fix | [EX-11](execution/2026-10-22-icon-border-fix-and-rebuild.md): its embedded icon is the Cipherleaf icon (checked); it starts, creates `data\` next to the exe, writes nothing to AppData, makes no network connections, closes cleanly. Not tested: creating a vault, the Passwords view, lock, a USB stick (V3c); the icon in the real taskbar (Windows may show an old cached icon). Unsigned. Earlier hashes (`5D766105…`, `A10AAB35…`) are superseded. |
| Owner's dev vault | `.devdata\data\` (ignored by Git) | Real use began 16:03 UTC; contents not inspected by Claude (the owner's screenshot at 18:06 shows a page "Hello Cipherleaf" and two "Untitled" pages) | File timestamps only |
| Test leftovers | About 88 `cipherleaf-*` folders in `%TEMP%` (cargo test vaults, KeePassXC test copies) | Not deleted (the project rule is to ask before deleting) | Folder listing |

### Not done

- Hands-on use of the running app by the owner: the new look, the calculator under Sticky notes, the reworded block editor (drag, "/" menu, toolbar, pictures, equations), sticky notes, auto-lock, screenshot blocking, typing in the embedded KeePassXC, quitting closing KeePassXC, the command palette hiding the overlay, the real vault's migration from the old theme names, scaling other than 100%.
- Release build checks not yet run: creating a vault and the Passwords view in the release exe, V3c (pulling the USB drive locks the app), a real USB stick, running it on a PC without the VC++ runtime, code signing and SmartScreen.
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

1. The owner runs `E:\Project\Cipherleaf-portable\Cipherleaf.exe` (it will ask to create a fresh vault) and the dev app, and says what to change. Anything that goes to other people keeps the `Licenses` folder and `README.txt` with it.
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
