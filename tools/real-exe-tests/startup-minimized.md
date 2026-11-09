# X-1 startup reproduction

Reporter: **Mohamed Sayed (AbyssWatcher2)**, via Ahmed. Coordinator: Claude on Ahmed Emara's side (AE0101), Gaming-PC. Run these **outside the Codex sandbox**, from `D:\Notaya-lanes\X1`. No install or browser launch is needed; the real exe creates its WebView2. Never run against real notes. Quit every existing Notaya/Cipherleaf copy first; the runner refuses to contact one. It sets `NOTAYA_HOME`, `CIPHERLEAF_HOME` and WebView2's profile folder for its own children only, creates a unique throwaway home, and preserves it.

```powershell
$exe = 'D:\Notaya-lanes\_target_release\release\Notaya.exe'
# Ordinary start, without --minimized (visible)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode Normal
# Exact command shape shell.rs writes: "<exe>" --minimized (hidden)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe
# Hidden reload: hold JS bundle requests for 5 seconds, then release
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -DelayPageMs 5000
# Hidden reload: withhold animation frames for 8 seconds
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -HoldFramesMs 8000
# Comparison with a late launch (not a simulation of Explorer/GPU readiness)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -LaunchDelaySeconds 15
# Read existing Run value, validate and launch that exact exe/flag (no registry writes)
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode RunKey
```

`RunKey` refuses unless the existing entry is exactly the quoted supplied executable plus `--minimized` (case ignored as in the app). A refusal is **not run**. The default mode already tests that same command shape without needing to change the real Run key. The runner uses ProcessStartInfo directly, with no shell interpreter or startup registry write. `Normal` is the no-flag control. Capture baseline and candidate separately, recording both source commits and exe hashes: rebuild the release from the final candidate source before claiming a fix. The runner prints the output folder and stores `launch.json` with the exe hash, command, mode and UTC time.

Each run inspects the initial DOM and native hidden/visible state, optionally reloads with delays, and starts the exe a second time (no flag) to exercise the same `show_main` used by the tray's Open Notaya. It checks that the second process exits within 10 seconds and the first stays alive. `initial.json`, `delayed-reload.json` when requested, `restored.json` and native `*-windows.json` retain results. The held-frame probe records successful `window_ready` completion; `readyWithoutFrames` must be true **before** frames resume. Under the old gate it is false even if the page eventually recovers. `delayActuallyApplied` must be true or that simulation is invalid. Tests exit nonzero on a false check or leftover debug listener.

## FIX RUN 2 build and comparison

**Correction:** the first probe assigned to Tauri's read-only `__TAURI_INTERNALS__.invoke` and unconditionally reported `hookInstalled=true`. Its zero invoke counts were invalid. The baseline held-frame failure and candidate-1 failure remain historical observations, but do not prove candidate 1's timer failed. The corrected probe observes the real fetch/native IPC transports and existing success/error callbacks, forwards them unchanged, verifies assignments and binds each report to the injected reload ID. It never replaces Tauri's read-only invoke, calls readiness itself or records IPC payloads/results. See the [corrected results](../../docs/plan/2026-11-09-whole-app-test-results.md#fix-run-2-corrected-analysis-and-hand-back).

Claude runs the following outside the sandbox. Keep the existing baseline and candidate-1 exes/reports. Build into a separate target so the new frontend is embedded and the old artifacts survive. From `D:\Notaya-lanes\X1`, on the final reviewed source:

```powershell
$env:CARGO_TARGET_DIR = 'D:\Notaya-lanes\_target\x1-fix2'
npm run build -- --configLoader native
if ($LASTEXITCODE) { throw 'Frontend build failed' }
cargo build --manifest-path .\src-tauri\Cargo.toml --release --offline
if ($LASTEXITCODE) { throw 'Release build failed' }
$baseline = 'D:\Notaya-lanes\_target_release\release\Notaya.exe' # main 5077183
$candidate1 = 'D:\Notaya-lanes\_target_x1\release\Notaya.exe' # 9b64db2, preserve
$candidate2 = 'D:\Notaya-lanes\_target\x1-fix2\release\Notaya.exe'
Get-FileHash -Algorithm SHA256 -LiteralPath $baseline,$candidate1,$candidate2
```

The existing baseline hash verified during FIX RUN 2 was `5FA039FCD0BE63267601F2FB7E695875470E82D30954CF84B7666E26BB475C3C`; candidate 1 was `8F38C9044CB7BD7D3C0901DEE09265296261187F6A12477FF7B5BA3B70E8911E`. Record the new source commit and SHA-256; stop if a purported retained artifact does not match its recorded hash. The new probe's non-null `bootstrapAtMs` and new effect/microtask counters also establish that the diagnostic frontend is actually embedded. These build commands were **not run by Codex** (only the frontend build was).

Rerun each artifact with the **same corrected runner/probe**, recording every exit code and printed evidence folder. Baseline failure is expected only if the held-frame dependency reproduces with valid observation. The optional candidate-1 rerun settles whether its timer already worked; never overwrite its old report. A failed command returns exit 1; keep that result and continue to the next control only after cleanup reports zero listeners.

```powershell
foreach ($exe in @($baseline, $candidate1, $candidate2)) {
  powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode Normal
  "Normal exit: $LASTEXITCODE"
  powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode Minimized
  "Minimized exit: $LASTEXITCODE"
  powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode Minimized -DelayPageMs 5000
  "Delayed-page exit: $LASTEXITCODE"
  powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\run-startup-minimized.ps1 -Exe $exe -Mode Minimized -HoldFramesMs 8000
  "Held-frame exit: $LASTEXITCODE"
}
```

All artifacts: require `probeDocument=true` and `readyObserved=true` on instrumented reloads. `readyCalls=readyCompleted=1`, `readyRejected=0`, `callbackHookFailed=false`; `fetchHookInstalled=true` and the independently reported native hook state identify coverage. For a held-frame pass, `heldFrames>0`, `readyAtMs<8000`, completion before `framesReleasedAtMs`, `readyWithoutFrames=true`, `frameShimReleased=true`. The held count need not remain 2: the new ready path requests no frame. The report preserves the pre-release observation, plus `afterReleaseProbe`; CDP releases only the test frame shim if its page timer is throttled. Poll timeout remains 45 s. Delay-only runs must now also observe readiness, rather than passing on DOM alone.

New candidate: require non-null `bootstrapAtMs`, `statusRequestedAtMs`, `statusSetAtMs`, `lockListenerReadyAtMs`, `screenCommittedAtMs`, `invokeCalledAtMs`, `invokeCompletedAtMs`; `effectRan>=1`, `startupEffectRan>=1`, `appReadyCalls=1`, `microtaskScheduled=microtaskFired=1` in a production reload; `timerScheduled=timerFired=0` (timer removed). Appearance may complete before or after status. Baseline/candidate 1 have no page stage counters: zeros/null there are unavailable diagnostics. The full counter-to-cause table is in the results record. Missing status → status IPC; missing listener time → listener registration; commit absent despite both → React gate; scheduled without fired plus cleanup → cancellation; app call without observed transport → probe/dispatch; dispatch without callback → IPC response. A successful hidden-tray early return counts as completion. A restored visible window with white pixels still requires compositor/runtime investigation and human inspection.

Interpretation:

| Result | Meaning / next action |
|---|---|
| No CDP page, or process exits | Native/WebView2 setup failed or renderer never became available. Inspect native error dialog, sanitized app log and Application error events. Do not call it a rendered pass. |
| Root empty / title absent | Frontend bundle/bootstrap did not complete. Log/error classes and delayed-page trace matter. |
| Title exists, screen absent | Status/backdrop IPC or React initialization path needs inspection. Do not expose the data folder or note text in shared diagnostics. |
| DOM/layout ready while native window hidden | Expected for minimized startup; no assertion about visible pixels yet. |
| `readyWithoutFrames` false | Check matching reload ID, verified transport hooks, call/completion/rejection counts and stage times before interpreting this as a handshake stall. The old read-only invoke hook was invalid. |
| Restored DOM/layout ready and native visible | Automated layout/visibility pass. A human must still see actual pixels, the tray icon/menu, and Quit. CDP itself can affect rendering/timing. |
| Native visible but pixels white | Compositor/runtime candidate remains. Record runtime version, event classes and whether manual resize/reopen recovers. Do not assume the scheduler fixed it. |
| Cleanup listener nonzero | Test fails. Inspect the owned child WebView2 process and close that process before continuing. Never kill a different process merely because it owns the port. |

The runner uses port 9341 by default, verifies it is initially free, requests normal Quit, then stops only its recorded process/descendants if necessary and requires zero listeners. It never sets the parent's debug environment. It does not delete its evidence/home, reset a WebView2 profile, change GPU flags, create a scheduled task, or simulate an actual sign-out/sign-in. The default fresh home covers the first screen; protected-vault, auto-lock, hide/reload, sticky/calculator and true sign-in remain separate PC checks. Timing data in the reports is observation time, not a clean startup benchmark.

If observation on Mohamed's PC is needed, send him this one command (with this script available locally), run while the blank window still exists:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\real-exe-tests\startup-diagnostic.ps1 > "$env:TEMP\notaya-startup-diagnostic.json"
```

Ask him to send that JSON, the app version, whether the title bar is Notaya's drawn title or Windows' native frame, whether waiting/resize/Open Notaya recovers, and the time since sign-in. For a portable/custom-home copy, he can pass `-LogFile '<its data>\logs\notaya-log.txt'` himself. The script only reads the Run command shape, running app command shape, installed WebView2 versions, fixed app-log classes and allowlisted Application error fields for Notaya/WebView2 (up to 200 recent events). Paths and all raw log/event messages are omitted. It never reads a vault or starts/attaches to the app and never uploads anything. A raw command with unknown arguments is omitted instead of risking a secret. Keep actual sign-in/reproduction uncertainty visible if these diagnostics do not settle it.

Browser-free regression:

```powershell
node --test --test-isolation=none .\tools\real-exe-tests\startup-ready.test.mjs .\tools\real-exe-tests\startup-probe.test.mjs
powershell -NoProfile -File .\tools\real-exe-tests\startup-cleanup.test.ps1
```

The Node tests compile actual production helpers/hook bodies with the existing TypeScript package and run the actual injected probe with Tauri's read-only descriptor. The original four scheduler scenarios remain, updated for the timer-free path: hidden startup, ordinary startup, once-only callback and cleanup/remount. Additional cases cover the actual status/listener gate, a pending backdrop, late appearance completion, opt-in counters, success/rejection and native fallback observation. The PowerShell test extracts the actual cleanup helper and uses fake process objects (no process start/stop), including null/throwing StartTime, unrelated/reused IDs and exited children. The runner retains a process handle on discovery, so unreadable StartTime falls back only to that exact owned child rather than reopening a reused PID. These checks do not prove Windows rendering or security acceptance.
