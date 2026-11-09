# Whole-app test plan: X-1 addition

Who and where: Ahmed Emara (AE0101), Codex lane X1 on Gaming-PC (Windows sandbox). Baseline: `e4df9185a99fabf6002065c84c921cd8f8278e31`. Produced; real-exe checks not run here. The full whole-app plan and `tools/whole-app/cdp.mjs` were absent at this baseline. Claude should integrate this single numbered check into the Windows/tray area of the whole-app lane's plan, retaining its stable ID.

## Windows and tray

1. **WIN-X1 — KIND B — Start with Windows draws the interface and preserves hidden startup.** Reporter: **Mohamed Sayed (AbyssWatcher2)**, via Ahmed, 2026-11-09 07:38:18 UTC. KIND B means the coordinator runs the real Windows exe outside the Codex sandbox, on throwaway data only.

   Steps:
   - Quit all existing Notaya/Cipherleaf copies yourself. Build the exact candidate release, record its commit and exe SHA-256. Run the commands in [`startup-minimized.md`](../../tools/real-exe-tests/startup-minimized.md) on the baseline and candidate; keep both output folders. The runner creates a unique home under `D:\Notaya Tests\v1-6-X1`, sets both home variables in children only, and refuses an existing app or occupied debug port.
   - Run normal startup (no flag), then `--minimized` (same quoted command shape as `shell.rs::run_command`). The normal window must appear with a populated root, Notaya title bar and nonempty screen. The minimized process must have that DOM while its native main window remains hidden. A second start must exit, leaving the first process alive and showing its populated interface. The native enumeration must report one Notaya window, visible and not minimized after restoration.
   - Repeat minimized startup with a 5-second bundle delay and with animation frames held for 8 seconds on a hidden reload. The report must confirm script requests were actually paused; readiness must complete before animation frames are released, once the status screen has committed. Repeat with a 15-second launch delay for comparison. These are timing simulations, not proof of real sign-in conditions.
   - If the existing Run value already points to the supplied exe, run `-Mode RunKey`. It reads and validates that exact quoted exe plus `--minimized`, without writing the registry. A mismatch is **not run**, not pass; do not repoint a real autostart entry for the test. A true sign-out/sign-in test needs a disposable Windows user/VM, its own normal app setup, and the same commands/logs there; no real user vault.
   - Inspect the actual restored window's pixels and tray menu manually (CDP DOM/layout is not compositor proof). Confirm Open Notaya and normal Quit. Retain reports and the exe hash. Require cleanup to report zero listeners on the debug port; no parent WebView2 environment is changed. Existing tray/lock/flush checks still apply and are not replaced by this startup check.

   Expected: all automated checks true, normal and restored interface drawn, minimized start hidden with tray access, no second surviving app process, port closed. Record real sign-in, protected-vault/auto-lock, and human pixel/tray observations separately when run. Never infer them from this fresh-home harness.
