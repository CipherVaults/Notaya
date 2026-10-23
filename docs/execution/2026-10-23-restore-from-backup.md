# Execution Summary: EX-2026-10-23-14 — Readable first-run passphrase and restore from backup (A18), rebuild

## 1. Task Overview

**Task requested** (the owner, 2026-10-23; message times not recorded): the owner's choice "the first-run passphrase fix (show it in plain text with a copy button) and the in-app restore from backup next, using agents and skills". After two "stop for a second" pauses and "Coniniut with ultracode agents sub agents and workflow", the owner asked "should i keep texting in the project thread or here better ?" (answered: keep going in this session for the unfinished restore work; the project thread works from the GitHub copy and cannot see uncommitted files; use the project thread for planning and new tasks; do not let two workers change the same files at once). Then: "Finish the restore work". That message authorizes finishing, committing, pushing the committed branches to the private repository, and rebuilding the portable folder and ZIP.

**Execution mode:** Direct project, with helper agents (section 3). **Executor:** Claude (Sonnet 5.5).

**Date / time:** 2026-10-23 (UTC). System clock: 13:51 UTC when the GitHub push of the earlier records finished; the release build finished about 14:14 UTC; the portable ZIP was made at 14:17 UTC. The owner's messages have no recorded timestamps ("not recorded").

**Authority:** the owner's messages above, in [approval-and-amendments.md](../plan/approval-and-amendments.md); recorded as [A18](../plan/A18-readable-passphrase-and-restore-from-backup.md). **Stopping condition:** stop with the work committed, the portable build and ZIP rebuilt, the records updated, and the committed branches pushed. The records commit and the push come after this file is written, so their hashes are not in this file (the records commit is "the commit after `1273093`").

## 2. Context Used

`CLAUDE.md`, `docs/STATUS.md`, the amendments record, the security design, Claude's memory note about the paused restore work, and the source (unlock screens, backup and vault code, lock sequence from A17).

## 3. Work Performed

**What was built** (all in commit `1273093`, "Restore from backup inside the app, readable first-run passphrase (A18)": 15 files, 1664 insertions, 68 deletions, on top of `616caa8`, branch `feat/a9-keepassxc`):

1. **Readable first-run passphrase.** "Suggest a strong passphrase" shows the phrase in plain text in a box (it wraps only at hyphens), with a Copy button ("Copied" for 2 seconds; on failure it says it could not copy), guidance to write it down, and a warning that Windows clipboard history (Win+V) may keep it. The phrase lives only in component state and is cleared on edit, on create and on unmount.
2. **Restore from a backup.** Reachable from the lock screen, the create-vault screen and Settings, Backups. A shared dialog (`src/features/unlock/RestoreDialog.tsx`, with shared fields in `src/features/unlock/fields.tsx`) lists the automatic backups in `data\backups` or lets the owner choose another `.clf` with the file dialog; it asks for the backup's password and optional keyfile, warns that the current vault is kept, shows errors inline, and reloads 1.2 seconds after success.
3. **Backend.** `src-tauri/src/restore.rs` (pure logic) and `src-tauri/src/commands/backup.rs` with the commands `list_backups` (works while locked), `pick_backup_file` (native file dialog) and `restore_vault(source, password, keyfile)`, which returns `{undoFile}`. Steps in `restore_vault`:
   - rate-limit check; only one restore at a time;
   - source rules: a bare name `vault-*.clf` is looked up inside `data\backups`; any absolute path is accepted only if it equals the file last returned by `pick_backup_file` in this run of the app. That pick stays valid after a wrong password and is cleared only after a successful install. UNC, network and device paths, `..`, `:` in names, a wrong file ending, folders, the live vault, and files over 2 GiB are refused;
   - copy to `data\restore-check.tmp`; verify **that copy** (restore-only Argon2 limits, memory up to 1 GiB, passes up to 10, lanes up to 16, checked before any Argon2 work; then every record and every attachment through `Session::unlock` and the new `Session::verify_attachments`);
   - if the vault is unlocked, lock it first with the normal lock (windows save pending edits, the normal backup is made, the session is wiped; `lock("restore")` does not emit `vault-locked` itself);
   - rename the live vault (and its journal) to `data\restore-undo-<ms>.clf` (never deleted, never rotated); rename the verified copy to `vault.clf` (the rename retries 6 times, 300 ms apart; failures roll back; a failed rollback is reported with the file name); unlock the restored vault. `AppState` got `picked_backup`.
   - A failed restore after the lock emits `vault-locked` with the payload `restore-failed: <message>`; the UI saves the message in `sessionStorage` key `cipherleaf.notice`, reloads, and shows it once as a toast. Raw OS errors (sharing violation, access denied, disk full) are turned into plain sentences.

**Review rounds** (agent reports are model output; Claude checked the key claims in the code):

| Round | Who | Found | Outcome |
|---|---|---|---|
| 1 | An independent review agent | The screen stayed "unlocked" after a failed restore; a hostile Argon2 header; a failed file check treated as "no vault"; the Copy button showed success when copying failed; a missing attachment test | All fixed |
| 2 | The code-review skill and Claude | The picked path was forgotten after a wrong password (an early clear had crept back once when an agent's edit overwrote Claude's fix; Claude removed it again, and a final reviewer confirmed it is cleared only after the install) | Fixed |
| 3 | A final independent review agent | A lost error message on a failed restore; rollback and journal handling; raw OS error text; dead code; weak tests; Enter during IME input; focus after an error | All fixed |
| 3 | A visual check agent (mock UI screenshots at 1280x900 and 480x800, Paper and Black) | The passphrase broke in the middle of a word; a faint restore link; a plain success message; confusing wording about "restore-undo-....clf" | All fixed |

**Agents used** (the owner's standing instruction, see [EX-13](2026-10-23-lock-and-data-loss-fixes.md) section 9): a backend agent, a UI agent, independent review agents (round 1, and the final one in round 3, plus a final reviewer who confirmed the picked-path fix in round 2), a visual check agent, a verification workflow that was started and stopped before it reported, and a records agent (this file and the other records). Their model and effort settings are not recorded. Skills: code-review. The independent reviewers and the visual agent stopped only what they had started and confirmed that ports 9333 and 1421 were free.

**Release build.** `npm run tauri build` finished about 14:14 UTC (file time 5:14 PM local, local = UTC+3). New exe SHA-256 `7BB5303F6AF16668E2C581E9B58489F6E348CB1E961BF365D0C2F0D40E17F3A1`. The previous exe (`D237C5B5…4183`) is superseded; a build in between (`E18FFC82…`) was discarded and never copied.

**The owner's vault in the portable folder.** `E:\Project\Cipherleaf-portable\data\` (305 files, 76,099,122 bytes, `vault.clf` last written 2026-10-23 02:54:14 local) was not touched: same file count, size and stamp after the update.

**Portable folder and ZIP.** The new exe was copied into `E:\Project\Cipherleaf-portable\` and the README's hash line updated. `E:\Project\Cipherleaf-portable.zip` was remade **without** `data\`: 83.1 MB, 169 entries, 0 `data` entries, SHA-256 `03AD0B3E7EBF3E1611B20F4C2DABFE762F2CFDD491D42D28B0969097F0BEE5AC` (made 14:17 UTC). The extracted exe equals the folder's exe and the extracted KeePassXC equals the folder's. The ZIP the owner uploaded to their Drive earlier (SHA-256 `1AD64164…D063`) and the one from before A17 are outdated; the owner should replace them. The exe is unsigned. No smoke test beyond the real-exe tests in section 5.

## 4. Commands / Tools Used

`cargo test --lib`, `cargo clippy --all-targets`, `npx tsc --noEmit -p tsconfig.json`, `npm run build`, `npm run tauri build`, PowerShell hash and ZIP commands, and two real-exe test scripts (see section 5). Skills: code-review. Helper agents as listed in section 3.

## 5. Verification / Testing

All on the exact committed source `1273093`, unless stated.

| Check | Outcome | Applicability and limits |
|---|---|---|
| `cargo test --lib` | pass, 68 tests (39 before; new ones cover restore, install and rollback, source rejection, a hostile header, a corrupted attachment, the picked-path rules, the error payload and the lock-event rule) | unit level only |
| `cargo clippy --all-targets` | only the 3 old style warnings (`notes.rs`, `stickies.rs`, `crypto.rs`) | no new warnings |
| `npx tsc --noEmit -p tsconfig.json` | exit 0 | |
| `npm run build` | passed | |
| **Real-exe restore test** on throwaway vaults in temp folders (WebView2 debug port 9335 for the test process only), run on the exe with SHA-256 `7BB5303F…F3A1`: `list_backups` while locked | **pass**: works and lists the newest first | |
| Same: an absolute path that was not picked | **pass**: refused | |
| Same: a UNC path | **pass**: refused | |
| Same: a path-traversal name | **pass**: refused | |
| Same: a wrong password | **pass**: refused; the live vault is unchanged | |
| Same: restore while locked | **pass**: returns an undo file; the restored vault opens with the ORIGINAL text | |
| Same: restore while unlocked | **pass**: the vault is open with the other text | |
| Same: after the restore | **pass**: the vault is unlocked; the data folder holds `vault.clf` and two `restore-undo-*.clf`, no `restore-check.tmp` | |
| **Real-exe lock regression** on the same exe (type then Lock at once; a sticky note typed into while Lock is clicked in the main window; close the window right after typing) | **pass** for all three (text saved) | the A17 behavior still holds |
| Portable folder and ZIP checks (hashes, entry counts, no `data`, owner's `data\` untouched) | pass | see section 3 |

The debug port was closed after the tests and nothing listens on it, the environment variable was not left set, and the temp folders were deleted. The test scripts (`restoreflow.mjs`, `run-restoreflow.ps1`, `lockflow.mjs`, `run-lockflow.ps1`) are in the session scratchpad, not in the repository.

**Not tested (the not-run list):**

- The native file-dialog path: choosing a file and then restoring it with its absolute path was never run end to end. Only the refusal of unpicked paths and the bare-name path were run in the real exe.
- The restore dialog and the passphrase box were seen only in the mock UI, not in the real window.
- A wrong-password retry on a picked file is covered by unit tests of the picked-path rule only.
- Crash or power loss between the two renames (see section 10).
- Idle lock, Win+L, sleep and drive removal (as in EX-13), and the Passwords view.
- `cargo audit` and the security-review skill were not run. `npm audit` was not repeated after the last source change.

**Test-harness note (ordinary friction).** The first attempt of the real-exe tests failed because the owner's PC had been idle for about 7.6 minutes and the app's own 5-minute idle auto-lock locked each throwaway vault at once. The test scripts now set the throwaway vault's idle-lock setting to 180 minutes right after creating it. The owner's vault was not involved.

## 6. Problems Encountered

- Each review round found real problems (section 3). One regression: an early clear of the picked path was brought back once by an agent's edit that overwrote Claude's fix; Claude removed it again and a final reviewer confirmed the final code.
- A verification workflow was started and stopped before it reported; its leftover mock server and test browser were stopped and the ports confirmed closed. Nothing from it is relied on here.
- The first real-exe attempt failed because of the idle lock (see the harness note above). It was a test setup problem, not an app defect.
- One build between the reviews and the final one (`E18FFC82…`) was discarded because the source changed afterwards.

## 7. Documentation Impact

Added A18 and this file. Updated the execution index, STATUS (as-of, mission state, the Git and release-build rows, open questions), the amendments record (two authority rows and the A18 row), the acceptance ledger (new row X15, X11 hashes), the security design (code-date sentence, threat 5, new promise P17, the out-of-scope line about older copies, new known limitations and the restore boundary) and the session history (a new entry and a dated correction note on the older entry that called the restore work uncommitted). The failure-intelligence record was not changed: nothing here met its admission boundary (the harness friction and the overwritten edit were caught and fixed before any harm).

## 8. Git / Version Control Impact

Code and tests: `1273093`. The records are in the commit after `1273093`. Pushing `main` and `feat/a9-keepassxc` to the private repository <https://github.com/AbyssWatcher2/CipherLeaf> follows the records commit; the owner's "Finish the restore work" covers it. Nothing else is pushed; no public visibility, no force-push.

## 9. Decisions Made During Execution

- **The picked path stays valid after a wrong password** and is cleared only after a successful install, so the owner can retry without choosing the file again. Trade-off: it stays valid until the next pick or a successful install.
- **The current vault is renamed, never deleted,** so a restore can always be undone by hand. Trade-off: the undo files pile up and open with the old password.
- **Restore-only Argon2 limits** (memory up to 1 GiB, passes 10, lanes 16), tighter than the normal header limits, so a hostile file cannot make the app use huge memory.
- **No automatic clearing of the clipboard** for the suggested passphrase (not built); the screen warns about clipboard history instead.
- **Where the project thread fits:** the owner asked which thread to use. Answer given: this session for the unfinished restore work; the project thread for planning and new tasks; never two workers on the same files at once.

## 10. Items That Need Human / Coordinator Review

- The owner should try the new exe: the passphrase box on a new vault, a restore from the lock screen and from Settings, and a restore through the file dialog (not yet tested).
- **Replace the older ZIP** (the Drive copy and any other) with `E:\Project\Cipherleaf-portable.zip`.
- **Security review by the owner:** a new way to replace the vault file ([security-design.md](../security-design.md) promise P17), and a clipboard copy of the suggested passphrase (threat 5).
- **Known limits** (also in the security design):
  - a crash or power loss between the two renames leaves no `vault.clf`; the old vault is in `restore-undo-<number>.clf` and the verified copy in `restore-check.tmp`; the app would show "Create your vault"; there is no automatic recovery or notice yet;
  - `restore-undo-*.clf` files open with the OLD password and keyfile, are never deleted and pile up; restoring also brings back the old password, which matters if it was changed because it leaked;
  - the suggested passphrase is not cleared from the clipboard automatically (Windows clipboard history and a cloud clipboard may keep it);
  - the picked path stays valid until the next pick or a successful install;
  - a late delayed save from the old window could reach the restored vault in the 1.2 seconds before the reload (small; the dialog blocks input);
  - `lock()` is not a barrier against another lock still writing its backup; a restore then fails cleanly with the plain "file is in use" message;
  - any window can call `restore_vault`; it needs no re-authentication of the current vault (the Content Security Policy and the picked-path rule are the defenses);
  - hostile `.clf` files are parsed by SQLite, only after the copy, with the Argon2 limits and the size limit applied.
- The backup rotation still covers only `vault.clf`, not the KeePassXC passwords file (backlog item from EX-13, still open).
- Still open from before: A2, A8, STATUS question 8, evidence for A1 and A3–A7, a license file for the repository, whether the repository may ever be public, and the EX-13 backlog items not done.

## 11. Current Result

The readable passphrase and the in-app restore are committed (`1273093`) and checked as listed in section 5, including real-exe restore tests on throwaway vaults and the lock regression test. The portable folder and ZIP are rebuilt; the owner's vault in the portable folder was not touched. **Not accepted;** only the owner accepts results. The file-dialog path and the real-window look of the new screens are untested.

## 12. Recommended Next Step

The owner tries `E:\Project\Cipherleaf-portable\Cipherleaf.exe` (including a restore), replaces the older ZIP copies, and says what to change or which backlog items come next. Anything that changes security behavior, adds a dependency, makes the repository public or adds a license waits for the owner.

**Do not repeat:** the review rounds, the restore design, the build.
