# Real-exe tests

Scripts that start a **copy of the built `Cipherleaf.exe`** with a **throwaway vault** in a temp folder, drive it through the WebView2 debug port, print PASS/FAIL lines and clean up. They test the real program (Rust core, WebView, file system), which `cargo test` and the mock UI cannot.

## Prerequisites

- A release build: `npm run tauri build` (the default exe is `src-tauri\target\release\Cipherleaf.exe`).
- Node 24 (global `fetch` and `WebSocket`), Windows PowerShell 5.1, WebView2.
- For the passwords/Quit tests: a KeePassXC folder, for example `E:\Project\Cipherleaf-portable\KeePassXC`. It is only **copied**, never changed.

## Safety rules

- **Never** run these against a real vault or `E:\Project\Cipherleaf-portable\data`. The runners copy the exe into `%TEMP%\clf-*` and create a new vault there. Never test on the owner's live window.
- The debug port is **9335**, set (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`) for the test process only, never left set. After a run, the runner prints `port 9335 listening after test: 0`. If it is not 0, find out why and close it.
- The passphrase is generated fresh for each run from a cryptographic RNG, written to a temp file and deleted at the end. It is never printed or stored. Do not hard-code one.
- Only the Cipherleaf the script started is stopped. **Never force-stop KeePassXC**: the tests close it politely; if it will not close, the temp folder is kept and reported.
- Never commit vault data or secrets (`*.clf`, `*.kdbx`, `.devdata/`).
- Run only after the work is approved. In `docs/STATUS.md` a paused mission means ask the owner first.

## How to run

From the project root:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$exe = "E:\Project\cipherleaf\src-tauri\target\release\Cipherleaf.exe"

# Restore from backup (about 1 minute)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-restoreflow.ps1 -Exe $exe

# Unsaved edits survive a lock or a window close (a few minutes)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-lockflow.ps1 -Exe $exe -Scenarios page,sticky,close

# Passwords-file backup, restore and Quit tests (pass the KeePassXC folder to copy)
$kp = "E:\Project\Cipherleaf-portable\KeePassXC"
$run = ".\tools\real-exe-tests\passwords\run.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testA.mjs  -WithKP -KeePassXCDir $kp
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testB.mjs  -WithKP -KeePassXCDir $kp
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testC.mjs  -WithKP -KeePassXCDir $kp -Extra c1
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testC.mjs  -WithKP -KeePassXCDir $kp -Extra c2
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testC.mjs  -WithKP -KeePassXCDir $kp -Extra c2q
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testD1.mjs -WithKP -KeePassXCDir $kp
powershell -NoProfile -ExecutionPolicy Bypass -File $run -Exe $exe -Script testE.mjs  -WithKP -KeePassXCDir $kp
```

Every runner first prints the **SHA-256 of the exe it tests**, and at the end the cleanup lines: port closed, env var not set, no process left, temp folder and passphrase file removed. `-Exe` is optional; it defaults to the release exe relative to the scripts. The passwords runner also takes `-Err` to capture the app's stderr.

## What each test checks

**run-restoreflow.ps1** (`restoreflow.mjs`):
list_backups works while locked, newest first; live vault has CHANGED text; absolute path that was not picked is refused; UNC path refused; path traversal name refused; wrong password refused; vault unchanged after the failed restore; restore while locked succeeded and returned an undo file; restored vault is open with the ORIGINAL text; restore while unlocked succeeded; second restored vault is open with the CHANGED text; vault is unlocked after restore.

**run-lockflow.ps1** (`lockflow.mjs`): for each scenario it prints `page contains "FLUSHTEST-<scenario>": true/false` and `sticky contains ...: true/false`. The text typed in that scenario must be saved: the page line must be true for `page` and `close`, the sticky line true for `sticky` (the other line is false by design). Scenario `close` closes the window, restarts the exe and checks again.

**passwords\testA.mjs** (backup of `passwords.kdbx` on lock): A0 no file, nothing copied; A1 first lock makes one copy equal to the live file; A2 unchanged file, still one; A3 changed file, two copies; A3b ten changes, ten copies; A3c eleventh change drops the oldest; A4 thirteen versions leave exactly ten; A4b the remaining ten are the newest, in order; A5 `vault-*.clf` backups untouched; A6 `passwords-undo-1.kdbx`, `backups\other.kdbx`, `data\other.kdbx` untouched; A7 invalid signature, lock works, no copy; A8 empty file, lock works, no copy; A9 vault is locked.

**passwords\testB.mjs** (restore of the passwords file): B0 backups for three versions exist; B1 list_password_backups shape and order; B2 restore while locked fails with "The vault is locked."; B2b it changed nothing; B2c list works while locked; B3 refuses 15 bad inputs (absolute paths, UNC, `..`, garbage, empty, directory, undo files, wrong extension, missing, empty string, live name); B3z and B3y refusals changed nothing and left no temp file; B4/B4a-f good restore of the oldest returns an undo file, live equals that backup, undo equals the previous live file, status points at `passwords.kdbx`; B5a-g a second restore makes a second undo file, planted undo file untouched, vault still unlocked; B6a-e restore with no live database works, no undo file, no temp files.

**passwords\testC.mjs** (Quit with KeePassXC; `-Extra c1|c2|c2q`): c1 Cipherleaf exits after the window is closed and no KeePassXC of the test remains; c2/c2q (KeePassXC has a dialog open) KeePassXC and Cipherleaf are both still running after 8 s and 13 s, the vault is locked, the notice text is delivered exactly once, `take_notice` a second time returns null, the `app-notice` event was emitted, KeePassXC closes politely, closing again exits Cipherleaf. `c2q` uses `quit_app`.

**passwords\testD1.mjs** (long diagnostic, several minutes): the first lock right after create_vault copies the passwords file; 90 changed-file locks each add exactly one copy and drop exactly one once ten exist.

**passwords\testE.mjs**: KeePassXC starts with its locked window (no dialog); the last lock copies V1; Cipherleaf exits after the window close; KeePassXC is closed; a final backup equals the file changed after the last lock.

## Known quirks

- **Idle auto-lock.** The app locks itself after 5 minutes of PC idle time, which breaks a test. Every script raises `idleLockMinutes` to 180 right after `create_vault` (throwaway vault only).
- **The first setup step can be flaky.** If the run fails right at the start (connect, create vault), run it once more before investigating.
- **A lock reloads the page**, so a script that calls the app across a lock may get `undefined` and reconnects (`connectMain`). That is expected.
- **`take_notice` is consumed by the page itself.** The UI takes the notice when it shows the toast, so a test call usually returns null; testC accepts either "UI showed it" or "I got it", but not both.
- Starting the exe takes a few seconds; the runners wait 6 s before connecting.
- UI Automation and keys (`uia.ps1`, `kpwin.ps1`) act only on KeePassXC processes that appeared after the run began.

## Lesson: test the exact exe you ship

A result applies only to the exact file it ran on. Each runner prints the SHA-256 of the exe it copied. Rebuild, then run against the new exe, and compare that hash with the one you ship (for example the portable copy) before you report a pass. An old build can pass while the new one is broken.
