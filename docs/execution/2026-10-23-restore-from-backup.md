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

<a id="restore-w195-handoff-20261122"></a>

### Windows restore interruption components and handoff, 2026-11-22 16:10 Cairo

**Exact scope.** Main `0e08e5489d36fbd2db1fc3a537cdebfa66bb3ad9`, tree `c67a85e52d1bcd71e66e6b20b3699595085445e2`, equals the clean tested source `cbf0193fabe77d04cdd84f40e2560f8dd643c333`. Recovered the original W register at `1ae2c047faccbf75bdb94737f68a51506f5eeb90`. Its W195 requires a reviewed instrumented native executable, observed interruption and same-home reopen. Rust tests serve as the oracle for the native case.

**Component execution.** Locked offline Windows x64 release-profile Cargo, `test --lib` with nine exact selectors and one test thread, exited 0 and printed each named case as passed. The selectors cover retaining and not overwriting undo, no previous vault, preserving the old vault and journal on failed install, stray journal rollback, the failure summary, paired rollback, keeping the journal aside when the old vault does not come back, and stopping immediately when the rename source is missing. Stdout SHA-256 `d002e0e62edf95d55836ba1dc3d16cc016475f238914461741ac986e833c78ad`. The source stayed clean and the ordinary Windows exe retained SHA-256 `2db9c2383d0ce91808d3454af1fa6351e43a419421ccd7f23e09a4a5d60fe169`.

**Native build preparation.** Independently reviewed the two-file local overlay and adopted the reviewer's guard requiring an actual old-vault set-aside. Locked offline Windows x64 release build of the Windows-only, non-default test feature finished with exit 0. Overlay manifest SHA-256 `b594f57e9b4927993930c522d9708d6924b47dcf26cb1634336e4cf048d82491`, restore SHA-256 `09c8622fe7930c11a2bab0a723306504d0a5662e42de850fe8e3f0de11929af7`, exe SHA-256 `98f78be8dd8b19fab86c568c640fa57dbaa08e3c5ce221b492270ae26c358450`. No production code or version changed. A shell parsing preflight was corrected before any overlay write. The local launcher parsed successfully and all three fresh homes contain only the intended synthetic inputs.

**Disposition.** Native product assertions, recovered file comparison and UI messaging are unrun. No app opened during this session. The build completed and closed before handoff. Exact inputs, launcher, overlays and raw evidence are local to the test PC. [Successor instructions](../handoff/2026-11-22-codex-v1-6-resume-note.md#first-executable-step-restore-interrupted-between-renames-w195).

<a id="restore-w194-native-settings-20261122"></a>

### Native Windows Settings backup refusal and restore — 2026-11-22 10:38 Cairo / 08:38 UTC

**Binding and isolation.** Current v1.6 main `bfc7b33842be298f346117f45a13bbae93aec2c6`, tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`. The Windows x64 unbundled executable was built from `1c9e20d61f9cef642d524ab50fded4dd2456c0e8` with the same tree and SHA-256 `cd32e84f1f1742833750138b1bf90c4d82a9c513292e6ba79ddb1683fe4065d5`. All chosen inputs and cloned homes were disposable and local. Restarts used separate process IDs and the same isolated home.

**Lock-screen observation.** The resumed locked fixture reported the password entry through accessibility while the screen capture showed the previous foreground content. In `src-tauri/src/lib.rs:134`, the application deliberately creates the protected startup window with capture exclusion. The separate resumed attempt sent zero input actions and took no restore action. Its locked vault hash and restore inventory stayed unchanged. The regular Settings route starts in an already open notes window. No privacy setting was changed to qualify it.

**Password-only backup.** From the open fixture, Settings and the Windows picker chose one protected backup. The wrong password showed an inline refusal, leaving no new staging or undo. The live file was held exclusively while open, so it was not hashed at the intermediate refusal. The same selected file with the correct password opened the protected note, with its heading, text, list and picture. The now-closed previous open vault matched its original full hash as the first undo. The selected backup matched its input hash.

**Damaged backup and explicit consent.** Saved a protected preimage from the app's verified backup flow before the next test. A structurally valid backup with a shortened sealed attachment was refused before replacement. The previous undo remained alone and staging was empty. A valid security-off backup prompted for explicit consent. The unchecked state kept the action disabled and created no new undo or staging. Checked consent restored that selected input. The closed newly installed file matched the selected backup byte for byte before the readback visit.

**Persistence and protected undo.** The app closed, restarted without a password, and reopened the saved heading, body and list. It then exited normally. Both undo files remained. The protected undo's physical file bytes differed from the SQLite backup copy. A read-only comparison found identical schema and every row across header, records, attachments, history, activity and journal anchors. SQLite quick-check passed on the native preimage, both undo files and restored target, including the restored file after restart. Readback counted four records and one attachment. The five pinned raw inputs all matched their original hashes.

**Disposition.** The W194 Settings scenarios passed on this development binary. The protected lock-screen interaction and protected restart unlock remain unrun in this native observation. The earlier core tests cover protected reopening with its fixture password, within their own scope. Keyfile, interrupted rename, exact Windows packages, the 5,000-row redo (Q1), and Daybreak keep separate dispositions. No product source or release artifact changed. Raw vaults, backups, profiles, and outputs stayed on the PC.

<a id="restore-w194-core-20261122"></a>

### Password-protected restore core and Windows package preflight — 2026-11-22 10:25 Cairo / 08:25 UTC

**Identity.** Live GitHub main `bfc7b33842be298f346117f45a13bbae93aec2c6`, tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`. The native Windows x64 unbundled app was built from `1c9e20d61f9cef642d524ab50fded4dd2456c0e8`, the identical tree, SHA-256 `cd32e84f1f1742833750138b1bf90c4d82a9c513292e6ba79ddb1683fe4065d5`. The isolated exact-source test copy added fixture or assertion functions under test configuration only. Production code and the built application stayed unchanged. The run used locked, offline, Windows x64 release-profile Cargo.

**Fixture preparation and core results.** One exact generator produced the ordinary password-only, security-off and unreadable disposable inputs with product APIs. A separate copy of a security-off backup has a shortened attachment record and passes SQLite structural quick-check. The five existing Rust tests, run from test executable SHA-256 `3395a72fadd741b8033b0a82ac229c3dbb9fc4ac25c130d49fcc956379621357`, passed with a single test thread. They covered protected restore, wrong password preserving bytes and removing staging, damaged database and sealed record refusal, security-off restore, and staged protection detection. Protected restore verified the recovered marker, retained the previous vault byte for byte and opened the retained copy with the previous credential.

**Command protection.** One separate test called the actual restore staging command core. A protected preimage and ordinary open backup refused both absent and false consent, with unchanged live bytes and no temporary or undo file. The structurally valid backup with a damaged attachment was refused even when consent was present. Explicit consent with the valid selected backup admitted staging, which left the live file unchanged. Installation matched the backup bytes, left source bytes intact and kept one byte-identical undo file. Opening without a password was refused. The fixture password reopened the protected undo. This is command and storage evidence. Native UI behavior remains open. Fixture and command-test-only overlays stayed local. Command test executable SHA-256 `2982d594ce1badf250defecb74da1f2822849061fb73b922e56ae0aab399253f`.

**Native attempt.** The pinned development executable launched into the password-only lock screen in an isolated home. Desktop capture showed a different foreground window and a keyboard shortcut reached it. Input stopped before opening the restore dialog or making a selection. Process identity and preimage bytes were checked before stopping the one owned process; the closed vault matched and no staging or undo file existed. Zero restore assertions ran in the native UI. The prior security-off Windows evidence below retains its original scope.

**Independent package check.** The existing packaging fixture had one local test-path substitution to an owned folder. The build script SHA-256 `e9ec0db39775e2812a6721f21546760d79806dff188331ed2e5f7240594fbdad` matched the reviewed tree and was run against inert inputs. It passed locked/offline command arguments, bridge placement in setup resources, companion and guide in the portable ZIP, bridge hash output, and data-file exclusion. One initial PowerShell preflight failed on a reserved variable name and created no artifacts. After correcting the variable, the inert fixture passed. The app, Tauri and npm manifests still identify `1.5.1`, and the bridge identifies `1.6.0`. These versions account for the fixture filenames. Production versions and lockfiles need alignment at the v1.6 candidate cut. Actual NSIS packaging, install/uninstall, portable persistence, clean user qualification and release acceptance remain open.

**Continuation.** Resume the protected native case with the existing disposable vault and the pinned executable only after the correct foreground window is verified. Complete actual selection, wrong password, same-file retry, restart, undo readback and downgrade checkbox scenarios. The 5,000-row database redo (Q1) and Daybreak hold remain separate. Only sanitized evidence was prepared for the records branch. All vaults, test strings, binaries, fixture inputs and raw evidence stay on the PC.

<a id="native-backup-refusal-20261122"></a>

### Native Windows backup refusal — 2026-11-22 08:19 Cairo / 06:19 UTC

**Binding.** Windows x64 unbundled development executable from `c72e51a79ecb696f105f88b9696650713a86fd95`, tree `ad2dddb0c82ca0d67acd9eff29edadf31c32ef3a`, SHA-256 `66100c99dd876880933e56de4296f9ed0c185b77b1941ccc63d48150542b6345`. Only an owned app home, a security-off disposable two-note vault and a 55-byte plain-text file with a backup filename were used.

**Observed.** Cancel returned to the restore dialog without a selected backup and kept the replace button disabled. The separate unreadable-file attempt showed “The vault file is damaged or has been tampered with.” The dialog stayed open. After normal close, the active vault, fixture source and selected file matched their prelaunch hashes. No restore-undo or temporary check file existed. The same executable reopened both pre-existing notes, including the Word body and table value. Its test process exited and the closed vault hash still matched after restart.

**Limits.** The earlier positive picker run is separate. This check covers the ordinary unreadable-file and cancellation paths. It supplies no password, keyfile, first-unlock, power-loss or packaging qualification. All fixtures and raw output stayed on the test PC.

<a id="native-backup-picker-20261122"></a>

### Native Windows file-picker closeout — 2026-11-22 08:13 Cairo / 06:13 UTC

**Scope and exact build.** This closes the file-picker route for a security-off disposable vault, separate from the earlier restore-by-name result. Windows x64 unbundled development executable from `c72e51a79ecb696f105f88b9696650713a86fd95`, SHA-256 `66100c99dd876880933e56de4296f9ed0c185b77b1941ccc63d48150542b6345`. Tree `ad2dddb0c82ca0d67acd9eff29edadf31c32ef3a` matched independently recovered main `eeafe701344f3d8bc6bbf52f10f9634496251392`. The current disposable vault held two pages and two attachments. The other known synthetic vault's ordinary backup held four pages and two attachments.

**Native execution and persisted result.** From Settings chose Restore from a backup, then Choose another backup file. Selected the pinned external backup through the Windows picker and restored with its empty password field. The app displayed the name of the file holding the old vault. Restart showed the correct four titles and note bodies. One note opened with the expected picture. The status receipt matched the actual file inventory, with one new file beginning `restore-undo-` and no staging file.

**Independent recovery and consistency.** The old vault was retained byte-for-byte. An independent app home opened a copy of that file and displayed the previous two notes, including the original note, table and picture. The restored file matched the selected backup before the post-restore readback. After a note visit, closed SQLite verification matched every non-settings row and all header and attachment rows to the backup. The changed row was the reserved app-settings identifier. The selected backup, original snapshots and retained copy matched throughout. Quick-check passed for all inspected closed files.

**Limits.** Real data, password-protected backups, keyfiles, failed selection, malformed-file refusal, crash rollback and installer operation were outside this case. This result says nothing about the separate first-unlock safety copy (A44). All raw materials stay local. The failed attempt to hash an open, exclusively held test vault was retained in the local execution trace.

<a id="windows-packages-rehearsal-20261122"></a>

## Windows x64 development packages and portable relocation

**2026-11-22 11:20 Cairo / 09:20 UTC.** Ahmed's side, Gaming-PC. Inputs came from exact source tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38` at checkout `1c9e20d61f9cef642d524ab50fded4dd2456c0e8`, same tree as GitHub main `bfc7b33842be298f346117f45a13bbae93aec2c6`. [App and bridge build bindings](2026-11-08-v1-6-build-start.md#windows-bridge-startup-20261122).

- **Portable package produced and verified.** ZIP SHA-256 `B45C49A23D58E4C585BFC732F2C832C9AD0BD26E968161DB2EB47515E0C004D1` (9,819,635 bytes). Rehearsal staging replicated the production package structure with an explicit ten-file allowlist: x64 app and bridge, two guides, portable marker and five license and notice files. ZIP integrity, membership and SHA-256 of every extracted member matched. Both Windows binaries matched the previously built inputs. The six tracked text files matched their canonical Git blobs. The archive excluded vaults and profiles.
- **Native portable verification.** Ran the extracted app without `NOTAYA_HOME` or `CIPHERLEAF_HOME`, using an isolated profile. Fresh startup opened without a password. Created a notebook and page and saved a disposable note. Copied the closed vault, test exit preference and all ten verified package members to a second local folder. The live app reopened the same notebook, page and saved body. Both child processes exited 0 normally. The installed data root remained unused in each profile. Read-only integrity checks passed, and both closed vaults matched the originally copied hash. A separate worker verified archive and launch identities, with no access to vault bytes. Local verification receipt SHA-256 `FFF12F8D021339897573DA7E001D8D9574A6D617910DB0C5C758CA4B19CA3C28`.
- **Setup produced and verified at the payload boundary.** Used the locked Tauri CLI 2.12.4, existing verified app and bridge, official NSIS 3.13 and nsis_tauri_utils 0.5.3. The normal Tauri bundle step used isolated tool and output paths with a bridge resource map. No setup was run. The nine-file NSIS listing and an independent extraction showed `Notaya.exe` and `notaya-bridge.exe` side by side. Setup SHA-256 `121EF60146A428FC783A05E41DFF9933E40AAFB75514B54A62170A30316DE1E2` (8,020,380 bytes). Extracted app SHA-256 `1313B53BE727AFF20B2FAF110943EF6E08F648BE84D5D4442791D1010F345395`; the exact three changed bytes select the NSIS bundle type. The bridge hash matched the portable member. Local verification receipt SHA-256 `EBF7DFB5CFB919D9EB44D4879F6ADC1E26ED35F21CD9BEFCADB071C54243BB68`.
- **Attempts kept.** The initial launcher refused the Windows native binding: the published 2.12.4 JavaScript still expected 2.11.4 in its generated optional check. Retrieved the two exact 2.12.4 registry archives, verified their SHA-512 from the lockfile, and used the upstream explicit native binding option on the exact pinned binary in one child process. A verifier initially applied the upstream `NSIS.exe` digest to `makensis.exe`; the corrected check matched the upstream pins for `NSIS.exe` and the companion tool DLL. Neither check changed source or host settings.
- **Still open.** The current development app embeds version 1.5.1 and the bridge embeds 1.6.0. Final version and toolchain freeze and exact package reruns are required. The clean-account native setup, upgrade, uninstall (W220), installed portable import (W218), actual sign-in (W222), and live approvals and pairing remain separate. The development packages, disposable vaults and raw receipts stayed on Gaming-PC.

<a id="native-w186-manual-backup-20261122"></a>

### Native Windows manual backup creation and restore

**2026-11-22 13:04 Cairo / 11:04 UTC.** Ahmed's side, Gaming-PC. Verified GitHub main `bfc7b33842be298f346117f45a13bbae93aec2c6` matched clean source `1c9e20d61f9cef642d524ab50fded4dd2456c0e8` at tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`. Both native development app copies matched SHA-256 `CD32E84F1F1742833750138B1BF90C4D82A9C513292E6BA79DDB1683FE4065D5`. The input was a pinned security-off synthetic fixture from the W194 restore run.

- **Two actual backup routes.** With a saved note, styled heading and list, small picture and visible Version history, used Settings Save a backup copy and the unfiltered Ctrl+K Back up vault now action. Both used the native Save picker, wrote their distinct selected files and reported success. Each output was 36,864 bytes, SHA-256 `005B143D498B551267F19549EF3CFDC1EF1F236F879B9D676BB6F1B60B8B5C2A`.
- **Native guards.** Entered a new filename and cancelled Windows Save. No new output appeared. Chose the exact owned test live file and confirmed the Windows prompt. Notaya said it was using that file right now and required a different name or folder. No temporary sibling remained and the two output identities held. A direct live hash was unavailable under the open file handle. After closure, every sealed row matched the earlier backups. The Open data folder action opened a single Explorer tab at the source test app's exact data directory.
- **Restored actual output.** Started a separate blank home with the same executable, used Settings and the native Choose a Notaya backup dialog and restored the exact palette output. It opened the expected notebook without a password. The page title, heading, body, list and tiny picture matched. Opened the restored Version history panel. The closed restored vault matched the two manual outputs byte for byte. Its prior blank vault survived separately and passed a read-only SQLite integrity check.
- **Independent file check.** All five checked copies passed read-only SQLite integrity. The two manual files, closed populated source and restored vault matched the complete schema and sealed rows across header, records, attachments, history, activity and journal anchor tables. The populated copies held four records, one attachment, two history entries, four activity entries and two journal anchors. The fixture body marker was absent from raw file bytes. The security-off copy uses the built-in key. The pinned original synthetic input remained unchanged. Local receipt SHA-256 `BCDDF7F2CA1B659FB7A6AE45212F56F9737E64C2DB256728045339BA34FEC8FD`.
- **Failure, limits and disposition.** The first direct palette click did not have window geometry. Typing the complete action label showed No matches. Clearing the query exposed the action. Current and released v1.5.1 source both condition action rendering on an empty query in `src/components/CommandPalette.tsx`. Track a v1.6 palette correction separately (W134). Early database readback used a plural table name. Corrected it to the actual singular `journal_anchor` name without repeating a backup. Both owned app instances exited with code 0 and closed their output pipes. Each retained a WebView shutdown diagnostic. Scope is the security-off synthetic snapshot path and the named live-file guard. Protected scenarios, exact packages and release acceptance remain separate. Vaults, backups and raw evidence stayed on the PC.


<a id="native-w134-command-palette-20261122"></a>

### Native Windows command search in the palette

**2026-11-22 13:32 Cairo / 11:32 UTC.** Ahmed's side, Gaming-PC. Scope is the command palette (W134). The Windows backup finding (W186) exposed the regression.

- **Why and what changed.** Typing a visible action previously removed the action list. Match enabled actions by their visible labels, prefer exact matches and keep ranked note results first for partial matches. Tie note results to the active query, provide a separate failed-search message and keep one selection handler per action. The command handlers and Lock eligibility are shared with the original action list.
- **Baseline and candidate.** Baseline Windows executable SHA-256 `CD32E84F1F1742833750138B1BF90C4D82A9C513292E6BA79DDB1683FE4065D5`, tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`, identical to GitHub main `bfc7b33842be298f346117f45a13bbae93aec2c6`. Candidate `33d02033051e46199bf4eb642fe11afa1c5ea2b8`, tree `40db056a061f8b57eef4f5f504309d0cce672bb5`, executable SHA-256 `42D36778F3E8E7FA0BAF60D2C7B6CD7420C9F5594A22F404915C3AFE64079C78`. Exact frontend build and a fresh locked offline Windows MSVC release-mode application build passed.
- **Reproducible Windows check.** In separate empty disposable app homes, import the same ZIP of three Markdown notes. Give one note the exact visible backup-action title, give another a title that starts with New, and give the third a unique phrase in its body. On the baseline, Ctrl+K and the exact backup label returned only the note after settling. On the candidate, Actions preceded Results and both entries were present. One Enter opened one Windows Save dialog. Cancel closed it without a second dialog or a new file at that destination. Body search opened the correct note, a recent search repopulated the field, and the short query returned the matching note ahead of both matching New actions. Changing a found query to one with no hits removed its earlier result after settling. Enter in the no-hit state left the palette open. Lock vault remained absent with security off. A mixed-case Export with three dots reached the current-page Export dialog, which was cancelled.
- **Shutdown, review and limits.** Both apps exited 0 with logs drained. Both closed disposable vaults passed a read-only SQLite quick-check. The launch watcher expired during compilation before it opened an app. The later run used the freshly built executable after verifying its identity. Independent review found error-feedback and partial-match ordering issues in the first diff. The final source corrects both. The parent command regression is preserved as a negative control. Local native receipt SHA-256 `8391B9FBF2C3588D6E4EBA09A7CA2ED201883926AE6A04DF4764D73A73F2DF12`. The raw fixture, vaults and receipts stay local. This batch did not exercise password-enabled Lock, force a failed search or qualify the installer. [Test register](../plan/2026-11-09-whole-app-test-results.md#native-w134-command-palette-20261122).

- **Integration at 13:37 Cairo.** [PR #141](https://github.com/CipherVaults/Notaya/pull/141) merged at `e3f7fa2da4889a257f1e26d50b6fe68eb15153c3`. The independently fetched GitHub main has exactly the tested tree, with the previous main and the tested source commit as parents. Git connectivity and empty source diff checks passed. GitHub returned no hosted workflows or commit statuses for this PR. Exact release package and owner gates stay open.
