---
name: "cipherleaf-real-exe-test"
description: "Use to test CipherLeaf's built exe on the user's PC with throwaway data only, never the real vault."
---

# CipherLeaf real-exe test

> **Dated note (2026-11-07, [EX-2026-11-07-45](../../../docs/execution/2026-11-07-records-gaps-closed.md)).** This is Mohamed's exported skill, kept as exported below this note. "CipherLeaf" and "Cipherleaf" are the old name: the app is **Notaya** (A28) and the repository is now `CipherVaults/Notaya` (it was `AbyssWatcher2/CipherLeaf`). The two partners are independent sides ([A37](../../../docs/plan/approval-and-amendments.md), `CLAUDE.md`): on **Mohamed's side** the merge and release lines below stand as written; on **Ahmed's side** his coordinator commits, pushes and merges inside work he approved, publishes a full release when it judges it ready, and acceptance stays Ahmed's own word. Where this skill and `CLAUDE.md` differ, `CLAUDE.md` wins. Paths: the old `E:\Project\Cipherleaf-portable` and `E:\Project\cipherleaf` folders named below are from before 2026-10-28; tests now run only in a test folder outside the repository (`E:\Project\Notaya\Notaya Tests Excluded` on Mohamed's PC, `D:\Notaya Tests` on Ahmed's), with `NOTAYA_HOME` pointing at a throwaway folder there. Copy the exe with `portable.txt` (EX-2026-10-28-33).

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