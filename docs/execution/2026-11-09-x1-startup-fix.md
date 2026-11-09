# X-1: blank window at Start with Windows, fix lane X1 (EX-2026-11-09-69)

**Who and where:** Ahmed's side (`AE0101`). Built by Codex on the Gaming-PC (lane `AE0101/v1-6-x1`, three runs); reviewed by Claude in the cloud thread; real-exe comparison run by Claude on the Gaming-PC on throwaway data. Reporter of the defect: **Mohamed Sayed (`AbyssWatcher2`)**, via Ahmed, 2026-11-09 07:38:18 UTC (v1.5.1: a blank white window at sign-in with Start with Windows on).

## 1. Task and result state

Fix lane for defect X-1 (spec WP-18). Baseline `main` `5077183`. Candidate: lane head `d1d5dff`. Result state: **verified** on the exact commit by the PC heartbeat (`tsc`, `npm run build`, `npm audit` pass; Rust unchanged, so `cargo test` is not part of this change) and by the real-exe comparison below; **integrated** when merged; **not accepted**. It is **not shown** that this cures Mohamed's blank window: his case happens at a real Windows sign-in, which nobody has reproduced.

## 2. What changed

The window starts hidden and Rust shows it only when the page calls `window_ready`. The old code called it after two animation frames, following the status call. The new code calls it right after React commits the status screen and both lock listeners are registered, with no timer and no animation frame, so a withheld or slow frame can no longer keep the window hidden. Run 3 added the failure path found in my review of run 2: if the status call (one retry) or a lock listener fails, or nothing answers within 5 seconds, the window is still shown. The backdrop call no longer holds the status call back, and a late backdrop answer cannot overwrite newer appearance settings. Rust is unchanged, so `--minimized` still stays hidden while the tray exists. Opt-in startup counters exist only when the test harness creates the probe object; the app creates none.

## 3. Real-exe comparison (Gaming-PC, 2026-11-09 about 18:40 UTC, throwaway homes in `D:\Notaya Tests\v1-6-X1`, counts and times only)

Candidate exe built from `d1d5dff` with `npm run tauri build --no-bundle`, SHA-256 `2F6BA43FC35BAA5D6DFD39575CC05205B211C029DD92752147854A2C54613965`; baseline SHA-256 `5FA039FC…` verified.

| Run | Baseline | Candidate |
|---|---|---|
| Normal start | exit 0, window visible | exit 0, window visible |
| `--minimized` | exit 0, hidden, shown on second launch | same |
| Page delayed 5000 ms | ready after 5127 ms | ready after 5131 ms |
| Frames held 8000 ms | ready only after frames were released (8004 ms), exit 1 | ready after 125 ms, 8 s before release, exit 0 |

Clean-up checked: debug port closed, no Notaya process, no Run key, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` unset, no real vault or window touched.

## 4. Limits and next action

The held-frame run simulates withheld frames on a hidden reload; it is not a real sign-in. The two failure paths (status rejected, listener rejected) have no real-exe switch; only Codex's unit tests cover them. Next: Mohamed (or Ahmed) tries the candidate build with Start with Windows on, then the owner decides. The whole-app pass (A47) re-runs it.
