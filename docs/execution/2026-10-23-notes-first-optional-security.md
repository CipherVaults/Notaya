# Execution Summary: EX-2026-10-23-16: Notes first: security and passwords optional (A20)

## 1. Task Overview

**Task requested** (the owner, in the project thread, 2026-10-23, UTC; the times are the owner's message timestamps):

- (a) 18:32:20: "Yes make a readme for the app and the features and optional features. And also do somethings in the app. First the core app is notes the security will be optional and screenshot make it as off all time. The vault stays open all the time unless the person press on button or something like this to make it secure and all security options. Do all this when the resets is over"
- (b) 18:34:11: "I mean cipherleaf is notes app in itself core and development. The security layer will be an optional people can turn off or turn on anytime. And the passkey manger is optional as well"
- (c) 18:35:23: "So turn off screenshot security by default. Make the vault when I press on it it opens without any master password or passwords this will be optional"
- (d) 18:39:02: "just to confirm agree but the securti can be off and on okay". This agrees to the three confirmations Claude asked about security behavior: security off still encrypts with a built-in public key and is not confidential; existing password vaults stay protected and keep their saved screenshot setting; turning security on offers to delete earlier unprotected backups after a fresh protected backup, and turning it off needs the current password and shows warnings.
- (e) 18:40:15: "work on everthing now"
- (f) 18:47:35: "make sure you use agents and subagents"

**Execution mode:** Direct project, in the cloud thread session, with helper agents (section 3). **Executor:** Claude (Claude Code).

**Date / time:** 2026-10-23 (UTC). Times above are the owner's message timestamps. The start and end of the work, the commit times and the build times are not recorded.

**Authority:** the owner's messages above, in [approval-and-amendments.md](../plan/approval-and-amendments.md); recorded as [A20](../plan/A20-notes-first-optional-security.md). The owner's messages do not name the commit, the push or the pull request; Claude read "work on everthing now" as covering them. **Stopping condition:** stop with the work committed and pushed to branch `claude/project-thread-sv2tfm`, a draft pull request against `main` to follow, the records updated, and the PC build, the real-exe results and the independent review marked PENDING. The pull request number and the PENDING results are not in this file yet.

## 2. Context Used

`CLAUDE.md`, `docs/STATUS.md`, the amendments record, the security design, the A20 design file written at 18:35 UTC, and the source (the vault header and session, the settings model, the restore code, the lock and quit paths, the KeePassXC commands, and the UI shell, sidebar, settings and unlock screens).

## 3. Work Performed

**Built** (three commits on branch `claude/project-thread-sv2tfm`, all pushed):

- `84b281d` "Notes first: security and passwords become optional (A20), README rewritten": the core, the UI and the README (about 1675 lines added across 31 files together with the next commit).
- `0929ae5` "Real-exe tests for optional security; existing test scripts use the new start": the new script `tools/real-exe-tests/optionalsecurity.mjs` and its runner, and small changes to the old scripts so they use the new start. The scripts were written. They were **not run**.
- `f9d2676` "A20 review fixes: re-key on turning security on or off, quit and restore safeguards": the code changes after the independent review (see "Changes after the independent review" below).

**Core (Rust):**

1. **Open vault format.** A new authenticated header flag `FLAG_OPEN` (bit 1 of the flags byte). An open vault wraps the data key under a built-in, public password with cheap Argon2 settings (8 MiB, 1 pass, 1 lane). Records, padding and tamper checks are unchanged. Format version stays 1. Session gained `create_open`, `unlock_open`, `unlock_any`, `make_protected`, `make_open`, `is_open` and `protects_capture`. `change_credentials` is refused on an open vault.
2. **Settings.** `screen_capture_protection` now defaults to false (a saved value in an existing vault is kept). New `passwords_enabled` (`Option<bool>`): unset in an old vault means on only if a passwords file exists.
3. **Start.** `vault_status` is now async. It opens an open vault by itself, or creates one on first run. It does not create one when earlier backups or restore-undo files exist but there is no `vault.clf`: then a screen offers "Restore" or "Start new notes" (command `start_notes`). A damaged vault never falls back to an empty one.
4. **New commands.** `enable_security` (password, optional keyfile, `remove_unprotected_copies`): re-keys the vault (see below), then makes a fresh backup, then deletes only earlier automatic backups (`data/backups/vault-*.clf`) whose header says open, plus their journal files. `disable_security`: needs the current password and keyfile, is rate-limited, and a wrong password counts as a failed attempt. `set_passwords_enabled`: turning off first closes KeePassXC politely and refuses if it stays open. `create_vault` was removed.
5. **Locking.** Open vaults never lock: the idle monitor skips them and `lock()` ignores everything except quit and restore. The main window is built content-protected only if `vault.clf` exists with a master password.
6. **Passwords switch.** The KeePassXC commands refuse while Passwords is off. Locks and Quit skip KeePassXC and the passwords-file copies while it is off.
7. **Restore.** A backup made with security off is restored without a password (its key is built in). Since `f9d2676`, restoring such a backup onto a vault that has a master password is refused until the person confirms (error starting "SECURITY_OFF:", a checkbox in the restore dialog, and `restore_vault` takes `allowSecurityOff`). The confirmation is only a UI gate; a script injected into the UI could pass it. Recorded as a known limit.
8. **Concurrency.** An `open_gate` mutex serialises auto-open, create and restore.

**UI:** no lock screen for open vaults; the Lock button and Ctrl+L only with security on; the Passwords sidebar item, palette entry and Ctrl+2 only with Passwords on; a Settings "Security" section with "Turn on" and "Turn off" dialogs (`SecurityDialogs.tsx`) and a Passwords switch.

**README.** Rewritten (core notes, optional security, optional passwords) by a helper agent and checked by Claude against the code.

**Agents used** (the owner's standing instruction, repeated at 18:47:35): a README helper agent (done); one independent security review agent (Opus-class, read-only; done, see below); a PC remote session to compile the Tauri crate and run the tests (started, result PENDING-FILL); a records agent for these documents. Other helper agents: not recorded. Effort settings are not recorded. Agent reports are model output.

**Changes after the independent review** (`f9d2676`). The review found 2 high, 3 medium and 6 low issues. Fixed in `f9d2676`: the two high issues (a shared data key, so any earlier open copy could open the protected vault; users with a keyfile could not turn security off), the three medium issues, and most of the low ones. Not changed: "low 10", the lock screen of a protected vault can be captured after a lock during the session (it follows the saved screenshot setting; documented as a limit). "Low 11" was formatting and was fixed.

Fixed, as seen in the code and the records:

- Turning security on or off now **re-keys** the vault (a fresh data key and vault id; every record and attachment re-encrypted in one all-or-nothing transaction; then the new header). It is no longer only a re-wrap. Reason: re-wrapping alone would let any earlier open copy open the protected vault. It takes a moment, and large notebooks take longer. Changing the master password while security is on still only re-wraps.
- Security can be turned off for a vault with a keyfile (the keyfile flag is now reported while unlocked).
- `enable_security` deletes only automatic backups whose header says open (and their journal files). It never deletes `restore-undo-*.clf` files. Manual exports are never touched. Deleted SQLite journal data may linger on disk.
- Restoring a security-off backup onto a protected vault needs the person's confirmation (see item 7 above).
- A quit of a vault with security off no longer reloads the window and does not reopen that vault while the quit runs. If the quit is cancelled because KeePassXC stays open, the vault is opened again. A quit of a protected vault is unchanged.
- `restore_passwords` and `pick_passwords_backup_file` are refused while Passwords is off. A quit from the lock screen still makes the last passwords-file copy. For a vault saved before A20 the Passwords switch counts as on if a passwords file or `keepassxc\keepassxc.ini` exists.
- After a crash between the two renames of a restore the app shows the "No notes found here" screen (Restore or Start new notes), not "Create your vault".
- The Turn on security dialog says it takes a moment (the Turn off dialog does not say so yet), and the Copy button for the suggested passphrase is in the Turn on security dialog.

Not changed: "low 10" (lock screen capture after a lock during the session). Not re-reviewed: the fixes themselves.

**Environment limits.** The cloud container could not compile the Tauri crate (no GTK libraries). Claude did not install system packages. Claude ran `npm ci --ignore-scripts` in the container only (to type-check and build the UI); nothing was installed on the owner's PC by this work.

## 4. Commands / Tools Used

In the cloud container: `npm ci --ignore-scripts`, `npx tsc --noEmit -p tsconfig.json`, `npm run build`, `npm run mock` with scripted screenshots, and the Rust core unit tests in a Tauri-free harness (the core modules compiled without the Tauri crate). Git for the two commits and the push (the draft pull request follows). Helper agents as listed in section 3.

## 5. Verification / Testing

All results below apply to the source of commits `84b281d`, `0929ae5` and `f9d2676` unless said otherwise. The `tsc` and build results are from `0929ae5`; whether they were re-run after `f9d2676` is not recorded here.

| Check | Outcome | Applicability and limits |
|---|---|---|
| Rust core unit tests in a Tauri-free harness (2026-10-23, after `f9d2676`) | pass, 69 tests, 13 of them new for A20 (header, session including re-keying and the rollback of a failed re-key, model, restore and openvault) | Core logic only. This is **not** `cargo test --lib` on the app crate. The app crate result is PENDING-FILL. |
| `cargo test --lib` on the app crate | PENDING-FILL (a PC session is compiling and running tests) | Not run in the cloud container: no GTK libraries |
| `cargo clippy --all-targets` | not run | |
| `npx tsc --noEmit -p tsconfig.json` | pass | Types only |
| `npm run build` | pass | The bundle only |
| Screens in the mock UI | seen | Fake data in a plain browser; not the real backend, vault or OS integration |
| Real-exe tests on a throwaway `CIPHERLEAF_HOME` (`optionalsecurity.mjs` and the regression scripts) | PENDING-FILL | The scripts are written; not run |
| Independent security review (one read-only agent, Opus-class) | done: 2 high, 3 medium, 6 low issues; fixes in `f9d2676` (see section 3) | Model output. The fixes were not re-reviewed (PENDING-FILL). "Low 10" was not changed. |
| `npm audit`, `cargo audit` | not run | |

**Not tested:** everything that needs the real app: the window flags (content protection) of a new, an old and a protected vault; the first-run auto-create; the Restore or Start-new-notes screen; turning security on and off in the real window; the deletion of earlier unprotected copies; the Passwords switch with a running KeePassXC; idle lock, Win+L, sleep and drive removal with security on; the README in the portable build. No portable build or ZIP was made for A20.

## 6. Problems Encountered

- The Tauri crate cannot be compiled in the cloud container (no GTK libraries), so the app crate, clippy and the real exe could not be checked there. The core was tested in a Tauri-free harness instead, and a PC session was started for the rest.
- The independent review found two high issues in the first build: a shared data key (an earlier open copy could open the protected vault) and users with a keyfile could not turn security off. Both were fixed in `f9d2676`.
- Further problems from the PC session: PENDING-FILL.

## 7. Documentation Impact

Added this file and the A20 status update. Updated the execution index, STATUS (as-of, mission state, open questions 12 to 14, what exists, exact next permitted action), the amendments record (the owner's six messages as authority rows and the A20 row), the security design (code-date sentence, the command paragraph, threats 4, 5 and 6 and 7, P7, P8, P11, P18, new promises P19 and P20, the A20 limits, the open-vault cryptography line) and the session history (a new entry). After the review the records were aligned with `f9d2676` (re-keying, restore confirmation, quit, Passwords refusals, lock screen limit, README and the A20 design file). The acceptance ledger was not changed: nothing is accepted and no check has a result on the app crate yet. The failure-intelligence record was not changed.

## 8. Git / Version Control Impact

Code, tests and README: `84b281d`, `0929ae5` and the review fixes `f9d2676`, on branch `claude/project-thread-sv2tfm`, all pushed. A draft pull request against `main` follows (number: PENDING-FILL). The records in this change are not committed by the records agent; the session commits them. Nothing was force-pushed and no visibility was changed.

## 9. Decisions Made During Execution

- **Asked before changing security behavior** (the owner's standing rule): three confirmations, agreed at 18:39:02 UTC (section 1).
- **Two independent switches.** Security and Passwords are separate; Passwords does not need security.
- **Turning security on or off re-keys the vault** (`f9d2676`), instead of only re-wrapping the header, because of the independent review's finding on earlier open copies.
- **Open vaults use the encrypted format with a public key,** not a plain file and not a key file next to the vault (a key file would sit in the same folder on a portable app and could be lost).
- **Existing vaults are never changed silently.** A vault with a master password stays protected and locked; only the person can turn security off, with the current password. Its saved screenshot setting is kept.
- **No silent new vault when backups exist but `vault.clf` is missing.** A screen offers Restore or Start new notes, so lost notes are not hidden behind an empty vault.
- **`create_vault` removed** in favor of `start_notes` and the auto-create at start.

## 10. Items That Need Human / Coordinator Review

- The owner should review A20 on a real PC (STATUS question 12), including the security change and the README.
- **Known limits** (also in [security-design.md](../security-design.md), "A20"):
  - with security off the notes are not confidential: anyone with `vault.clf` or a backup and a copy of Cipherleaf can read them;
  - a protected `vault.clf` replaced by an open one by someone with write access to `data\` opens with no prompt (the same access already allows replacing the exe);
  - earlier open automatic backups are deleted at turn-on if the person agrees (default on); `restore-undo-*.clf` files and manual exports are never deleted and stay readable by anyone if they were made with security off; after security is turned off, older protected backups still need their own password and cannot be opened with the new open file; changing the master password only re-wraps, so older backups open with the old password (existing limit);
  - restoring an open backup onto a protected vault turns security off after a confirmation that is only a UI gate (a script injected into the UI could pass `allowSecurityOff`);
  - screenshot protection and auto-lock exist only while security is on; the lock screen of a protected vault is hidden from screenshots only when the app starts, and after a lock during the session it follows the saved screenshot setting (off by default; "low 10", not changed);
  - Passwords is independent of security, and KeePassXC relies on its own master password and idle lock;
  - the new commands `enable_security`, `disable_security`, `set_passwords_enabled` and `start_notes`, and `restore_vault` with `allowSecurityOff`, are callable by any window; `disable_security` needs the current password; `enable_security` can delete earlier open automatic backups and `restore_vault` can lower security, without more than a UI dialog, so a script injected into the UI could do either while the vault is open.
- Open questions for the owner, now STATUS questions 13 and 14: whether existing vaults' saved screenshot setting should be reset to off, and whether to delete restore-undo files.
- The PENDING-FILL results (section 5) must be filled in from the PC session, and the fixes from the review should be checked, before anyone relies on A20.
- Still open from before: A2, A8, STATUS question 8, evidence for A1 and A3–A7, a license file, whether the repository may ever be public, and the outdated Drive ZIP.

## 11. Current Result

A20 is built and pushed on branch `claude/project-thread-sv2tfm` (`84b281d`, `0929ae5`, `f9d2676`). The Rust core passes 69 unit tests in a Tauri-free harness (13 new for A20); the UI type-checks and builds; the screens were seen in the mock UI. One independent review found 2 high, 3 medium and 6 low issues; the fixes are in `f9d2676` and "low 10" is not changed. The app crate build and tests, the real-exe tests and a re-check of the fixes are PENDING-FILL. **Not accepted;** only the owner accepts results.

## 12. Recommended Next Step

Fill the PENDING-FILL results from the PC session and a re-check of the review fixes (fix what they find), fill in the pull request number, then ask the owner to review A20 on a real PC. Anything that changes security behavior again, makes the repository public or adds a license waits for the owner.

**Do not repeat:** the design, the core and UI build, the README.
