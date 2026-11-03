---
name: "cipherleaf-real-exe-test"
description: "Use to test CipherLeaf's built exe on the user's PC with throwaway data only, never the real vault."
---

# CipherLeaf real-exe test

For checking a change in the real built app (screens, lock, move, restore, quit). First read CLAUDE.md, docs/failure-intelligence.md, tools/real-exe-tests/README.md. KeePassXC was removed in V1.1.0 (A24), so there is no passwords page or KeePassXC process to test; old passwords-*.kdbx files are only left untouched on disk.

**Hard rules**
- Never open, run against or modify E:\Project\Cipherleaf-portable\data or .devdata. Never run E:\Project\Cipherleaf-portable\Cipherleaf.exe in place.
- Work in a Temp clone or worktree; no checkout, commit or push in E:\Project\cipherleaf.
- Stop only processes the test started.
- Quote times from the system clock in UTC.

**Steps**
1. In the Temp copy: npm ci, tsc, npm run build, cargo test --lib, tauri build. Report test counts.
2. Copy the exe, Licenses and README (no data folder) to a new throwaway Temp folder; set CIPHERLEAF_HOME to a temp folder.
3. Create a vault (with a random password and a keyfile outside the portable folder if testing security); raise the idle auto-lock right away; add a note.
4. Check the behavior on screen (screenshots via the WebView2 debug port). Record exact messages, file names, SHA-256 hashes.
5. Clean up: close the app, close the debug port and confirm nothing listens, ensure WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS is unset. Delete only your own Temp folders, only with the owner's OK.

**Report:** pass/fail per check with exact text, on the exact commit and exe hash; what was not run and why. A result applies only to the source it ran on. Name the result state reached (produced, verified, integrated, accepted); a passing run is verified, not accepted.