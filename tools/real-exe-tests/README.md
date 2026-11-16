# Real-exe tests

Scripts that start a **copy of the built `Notaya.exe`** with a **throwaway vault**, drive it through the WebView2 debug port, and report failing checks with a failing exit. They close owned processes and ports but retain the run data and outputs. A full run tests the real program (Rust core, WebView, file system), which `cargo test` and the mock UI cannot; synthetic grader-qualification tests do not establish those native results.

## Prerequisites

- An independently verified release build, passed explicitly with `-Exe`; no runner chooses an executable implicitly.
- Node 24 (global `fetch` and `WebSocket`), Windows PowerShell 5.1, WebView2.

## Safety rules

- **Historical Mohamed-side location rule (2026-10-28 14:03:51 UTC):** builds, clones, test installs and throwaway vaults used `E:\Project\Notaya\Notaya Tests Excluded` after antivirus removed test files under `%TEMP%`. This is not permission for another partner's agent to access that drive. Use the current machine's authorized, owned outside-repository test location; quote paths containing spaces.

- **Never** run these against a real vault or `E:\Project\Notaya\Cipherleaf-portable\data` (moved there on 2026-10-28) (the owner's real vault; also any future `Notaya-portable\data`). The runners copy the exe into `%TEMP%\clf-*` (with a `portable.txt` marker, via `Copy-TestExe` in `common.ps1`, so the vault stays in that folder; without the marker a bare folder counts as an installed copy and would use `%LOCALAPPDATA%`) and create a new vault there. Never test on the owner's live window.
- Legacy checks use debug port **9335**, except tray checks use **9337**; the startup runner has its explicit port setting. The debug environment is child-only. Each wrapper asserts listener closure and parent-environment restoration. An unavailable listener query is a failure, not evidence of zero listeners.
- The passphrase is generated fresh for each run from a cryptographic RNG and retained with its synthetic vault inside the owned run home, outside the repository. Its value is never printed, committed or shared. Do not hard-code one. Keep the run and credential available for diagnosis; artifact retention is separate from closing the test process and debug port.
- Only the Notaya the script started is stopped.
- Never commit vault data or secrets (`*.nota`, `*.clf`, `*.key`, `.devdata/`).
- Run only after the work is approved. In `docs/STATUS.md` a paused mission means ask the owner first.

## How to run

From the project root:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$exe = "E:\Project\Notaya\Notaya\src-tauri\target\release\Notaya.exe"

# Restore from backup (about 1 minute)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-restoreflow.ps1 -Exe $exe

# Unsaved edits survive a lock or a window close (a few minutes)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-lockflow.ps1 -Exe $exe -Scenarios page,sticky,close

# Where the notes live: installed copy vs portable copy (about 1 minute; LOCALAPPDATA is changed for the test process only)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-installed.ps1 -Exe $exe

```

Pass `-Exe` explicitly to every runner, including X-1, using an independently verified build. No runner silently selects an old release executable. The five legacy runners propagate a failed Node check and assert process/port closure and restored parent settings; they do not turn printed failures into a successful exit. Their child processes clear inherited Notaya home overrides and use owned throwaway profile paths. All run homes and outputs are retained, including failed runs. A generated test-only credential stays inside its owned run home outside the repository and is never printed; do not commit or share it. Retention is separate from closing the owned app and debug port. The X-1 startup runner also retains its throwaway home and reports for comparison; its commands, safety boundaries and interpretation are in [startup-minimized.md](startup-minimized.md).

## What each test checks

**run-startup-minimized.ps1** (`startup-minimized.mjs`, X-1, reporter Mohamed Sayed / AbyssWatcher2): normal and hidden `--minimized` startup, optional delayed bundle loading and held animation frames on a hidden reload, exact Run-key command validation without registry writes, native window visibility, DOM/layout, and second-instance restore. FIX RUN 2 corrects the read-only invoke-hook defect and adds reload-bound transport/callback and opt-in page stage counters. Requires manual pixel/tray and true sign-in observations separately. Browser-free regressions: `node --test --test-isolation=none tools/real-exe-tests/startup-ready.test.mjs tools/real-exe-tests/startup-probe.test.mjs` and `powershell -NoProfile -File tools/real-exe-tests/startup-cleanup.test.ps1`. Exact baseline/new build and PC commands: [startup-minimized.md](startup-minimized.md#fix-run-2-build-and-comparison).

**run-restoreflow.ps1** (`restoreflow.mjs`):
list_backups works while locked, newest first; live vault has CHANGED text; absolute path that was not picked is refused; UNC path refused; path traversal name refused; wrong password refused; vault unchanged after the failed restore; restore while locked succeeded and returned an undo file; restored vault is open with the ORIGINAL text; restore while unlocked succeeded; second restored vault is open with the CHANGED text; vault is unlocked after restore.

**run-installed.ps1** (`installed.mjs`): an exe copy with no `portable.txt` and no `data` folder (what the setup installs) shows the first screen (Welcome, Start new notes, Import), keeps its notes in `<LOCALAPPDATA>\Notaya Notes\data` (a temp folder here) and not next to the exe, and opens them by itself on the second start; a copy with `portable.txt` keeps its notes in its own `data` folder, leaves LOCALAPPDATA alone and refuses the import command. Not covered: the native folder dialog of the import and the NSIS setup itself.

**run-lockflow.ps1** (`lockflow.mjs`): the page text must survive for `page` and `close`, and the sticky text must survive for `sticky`. Only the relevant target is asserted; the other target need not contain that scenario's text. Empty or unknown scenarios fail before launch. Scenario `close` closes the window, asserts process exit, restarts the exe and checks again.

## Known quirks

- **Idle auto-lock.** Protected-vault flows attempt to raise `idleLockMinutes` to 180 during setup on throwaway data. An earlier lock or failed setup remains a failed run to investigate, not permission to weaken the check or retry blindly.
- **A setup failure is a failed run.** Retain and label its data, outputs and source/executable identities. Investigate and classify the failure before considering another run. A new run needs a documented relevant change and the applicable review; do not blindly repeat a failed setup until it turns green. Preserve the failed result separately from any later qualified result.
- **A lock can replace the page context.** Undefined results or transport failures do not count as success. A test must explicitly reconnect when needed and assert the intended post-lock effect.
- **`take_notice` is consumed by the page itself.** The UI takes the notice when it shows the toast, so a test call usually returns null.
- Starting the exe takes a few seconds; the runners wait 6 s before connecting.

## Lesson: test the exact exe you ship

A result applies only to the exact file it ran on. Each runner prints the SHA-256 of the exe it copied. Rebuild, then run against the new exe, and compare that hash with the one you ship (for example the portable copy) before you report a pass. An old build can pass while the new one is broken.
