# Real-exe tests

Scripts that start a **copy of the built `Notaya.exe`** with a **throwaway vault** in a temp folder, drive it through the WebView2 debug port, print PASS/FAIL lines and clean up. They test the real program (Rust core, WebView, file system), which `cargo test` and the mock UI cannot.

## Prerequisites

- A release build: `npm run tauri build` (the default exe is `src-tauri\target\release\Notaya.exe`).
- Node 24 (global `fetch` and `WebSocket`), Windows PowerShell 5.1, WebView2.

## Safety rules

- **Never** run these against a real vault or `E:\Project\Cipherleaf-portable\data` (the owner's real vault; also any future `Notaya-portable\data`). The runners copy the exe into `%TEMP%\clf-*` (with a `portable.txt` marker, via `Copy-TestExe` in `common.ps1`, so the vault stays in that folder; without the marker a bare folder counts as an installed copy and would use `%LOCALAPPDATA%`) and create a new vault there. Never test on the owner's live window.
- The debug port is **9335**, set (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`) for the test process only, never left set. After a run, the runner prints `port 9335 listening after test: 0`. If it is not 0, find out why and close it.
- The passphrase is generated fresh for each run from a cryptographic RNG, written to a temp file and deleted at the end. It is never printed or stored. Do not hard-code one.
- Only the Notaya the script started is stopped.
- Never commit vault data or secrets (`*.clf`, `*.key`, `.devdata/`).
- Run only after the work is approved. In `docs/STATUS.md` a paused mission means ask the owner first.

## How to run

From the project root:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$exe = "E:\Project\cipherleaf\src-tauri\target\release\Notaya.exe"

# Restore from backup (about 1 minute)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-restoreflow.ps1 -Exe $exe

# Unsaved edits survive a lock or a window close (a few minutes)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-lockflow.ps1 -Exe $exe -Scenarios page,sticky,close

# Where the notes live: installed copy vs portable copy (about 1 minute; LOCALAPPDATA is changed for the test process only)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-installed.ps1 -Exe $exe

```

Every runner first prints the **SHA-256 of the exe it tests**, and at the end the cleanup lines: port closed, env var not set, no process left, temp folder and passphrase file removed. `-Exe` is optional; it defaults to the release exe relative to the scripts.

## What each test checks

**run-restoreflow.ps1** (`restoreflow.mjs`):
list_backups works while locked, newest first; live vault has CHANGED text; absolute path that was not picked is refused; UNC path refused; path traversal name refused; wrong password refused; vault unchanged after the failed restore; restore while locked succeeded and returned an undo file; restored vault is open with the ORIGINAL text; restore while unlocked succeeded; second restored vault is open with the CHANGED text; vault is unlocked after restore.

**run-installed.ps1** (`installed.mjs`): an exe copy with no `portable.txt` and no `data` folder (what the setup installs) shows the first screen (Welcome, Start new notes, Import), keeps its notes in `<LOCALAPPDATA>\Notaya Notes\data` (a temp folder here) and not next to the exe, and opens them by itself on the second start; a copy with `portable.txt` keeps its notes in its own `data` folder, leaves LOCALAPPDATA alone and refuses the import command. Not covered: the native folder dialog of the import and the NSIS setup itself.

**run-lockflow.ps1** (`lockflow.mjs`): for each scenario it prints `page contains "FLUSHTEST-<scenario>": true/false` and `sticky contains ...: true/false`. The text typed in that scenario must be saved: the page line must be true for `page` and `close`, the sticky line true for `sticky` (the other line is false by design). Scenario `close` closes the window, restarts the exe and checks again.

## Known quirks

- **Idle auto-lock.** The app locks itself after 5 minutes of PC idle time, which breaks a test. Every script raises `idleLockMinutes` to 180 right after `create_vault` (throwaway vault only).
- **The first setup step can be flaky.** If the run fails right at the start (connect, create vault), run it once more before investigating.
- **A lock reloads the page**, so a script that calls the app across a lock may get `undefined` and reconnects (`connectMain`). That is expected.
- **`take_notice` is consumed by the page itself.** The UI takes the notice when it shows the toast, so a test call usually returns null.
- Starting the exe takes a few seconds; the runners wait 6 s before connecting.

## Lesson: test the exact exe you ship

A result applies only to the exact file it ran on. Each runner prints the SHA-256 of the exe it copied. Rebuild, then run against the new exe, and compare that hash with the one you ship (for example the portable copy) before you report a pass. An old build can pass while the new one is broken.
