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

Interpretation:

| Result | Meaning / next action |
|---|---|
| No CDP page, or process exits | Native/WebView2 setup failed or renderer never became available. Inspect native error dialog, sanitized app log and Application error events. Do not call it a rendered pass. |
| Root empty / title absent | Frontend bundle/bootstrap did not complete. Log/error classes and delayed-page trace matter. |
| Title exists, screen absent | Status/backdrop IPC or React initialization path needs inspection. Do not expose the data folder or note text in shared diagnostics. |
| DOM/layout ready while native window hidden | Expected for minimized startup; no assertion about visible pixels yet. |
| `readyWithoutFrames` false | The startup handshake still depends on animation frames, or the probe could not hook it. Check `hookInstalled`, successful completion and time. |
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
node --test --test-isolation=none .\tools\real-exe-tests\startup-ready.test.mjs
```

It compiles only the actual scheduling function from `src/App.tsx` using the already-installed TypeScript package and injects a fake clock. It proves hidden/no-frame progress, the normal two-frame path, once-only fallback and cleanup/remount behavior. It does not prove Windows rendering or security acceptance.
