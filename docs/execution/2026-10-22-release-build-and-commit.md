# Execution Summary: EX-2026-10-22-09 — Commit of A10–A14 and the first release build

## 1. Task Overview

**Task requested** (the owner, 2026-10-22 22:58:30 UTC): "Commit", "Start build exe for release", and a note that screenshot protection was left off for now and is working.

**Execution mode:** Direct project. **Executor:** Claude (Sonnet 5.5). **Date:** 2026-10-22, about 22:58–23:10 UTC (system clock and transcript).

**Authority:** the instruction above. It answers STATUS question 1 (commit), question 4 in part (go-ahead for the portable build; copying the dev vault was not asked and was not done) and question 6 (screenshot protection). It satisfies the standing rule that a release build needs the owner's go-ahead. **Stopping condition:** a built exe and a portable folder, ready for the owner. The exe was smoke-tested from a temporary copy (section 3, step 5); nothing was distributed.

## 2. Context Used

STATUS, the amendments record, the plan's Verification section (portability criteria V3a–V3c), `src-tauri/tauri.conf.json` (bundling is switched off, so the build produces only the exe), `src-tauri/src/portable.rs` (a release finds `data\` and `KeePassXC\` next to the exe), `vendor/keepassxc-manifest.md`.

## 3. Work Performed

1. **Pre-commit check:** branch `feat/a9-keepassxc`; 33 modified, 1 deleted, 33 untracked paths; none matched vault data or secrets (`.clf`, `.key`, `.env`, `.kdbx`, `.csv`, `.devdata`, `vendor/`, `target/`, `dist/`); `.devdata`, `vendor`, `src-tauri/target`, `node_modules` and `dist` are Git-ignored; the plan snapshot is unchanged.
2. **Code commit:** `2105a57` "A10-A14: in-window KeePassXC, new look, block editor, sub-pages, equations, calculators", 51 files, +5,517/−1,438, with the Sonnet 5.5 trailer. Code, `package.json`, `package-lock.json`, `dev-mock/` and `vite.mock.config.ts` only. The records were left for a second commit (section 8).
3. **Release build:** `npm run tauri build` from the project root (it runs `npm run build` first). Finished in 4 m 32 s. Output `src-tauri\target\release\cipherleaf.exe` (11,363,328 bytes). The source is exactly commit `2105a57`; only documentation was uncommitted at that moment.
4. **Portable folder** `E:\Project\Cipherleaf-portable\` (outside the repository, 158.7 MB): `Cipherleaf.exe`; `KeePassXC\` (a copy of the verified 2.8.0 folder); `Licenses\KeePassXC\` (its license files and the complete source archive `keepassxc-2.8.0-src.tar.xz`); `README.txt` (how to run, what is needed, licenses, the exe's hash). No `data\` folder yet; the app creates it on first run.
5. **Smoke test** (at about 23:08 UTC, after confirming the dev app was not running, so the single-instance guard was not in the way): copied the portable folder to `%TEMP%\cipherleaf-portable-test`, started `Cipherleaf.exe` from there with no environment variables set, waited 12 seconds, checked, closed it politely, confirmed nothing was left running (no app, no WebView2 from the test, no KeePassXC), and deleted the copy. The real portable folder was never run, so it has no `data\` folder.

## 4. Commands / Tools Used

`git status`, `git check-ignore`, `git add`, `git commit -F` (message in a file), `npm run tauri build`, `Copy-Item`, `Get-FileHash`.

## 5. Verification / Testing

| Check | Subject | Outcome | Applicability and limits |
|---|---|---|---|
| Commit contents scan | the staged paths | no sensitive paths; no docs staged by mistake | current for `2105a57` |
| Release build | commit `2105a57` | built | the exe exists |
| Smoke test: launch from a temp copy (plan V3a, limited) | the built exe, copied to `%TEMP%` | **pass**: it started, showed a window titled "Cipherleaf", and created `data\backups`, `data\keepassxc` and `data\webview` next to the exe | one run, 12 seconds, first screen only (no vault created) |
| Nothing written to AppData (plan V3b, limited) | `%APPDATA%` and `%LOCALAPPDATA%` | **pass**: no folder named like the app, its WebView2 profile or KeePassXC before or after | one run, names only |
| Network during the smoke test | the app and its WebView2 processes | **pass**: 0 non-listening TCP connections; no debug port listening | seconds of observation |
| Closing | polite close of the main window | **pass**: the app exited; no WebView2 or KeePassXC left | one run |
| `KeePassXC.exe` SHA-256 in the portable folder | `E:\Project\Cipherleaf-portable\KeePassXC\KeePassXC.exe` | `72200BFE…D28D`, equal to the verified value | current; only that file is hash-checked (the DLLs are not, as already recorded) |
| `Cipherleaf.exe` SHA-256 | the copy in the portable folder | `5D766105…5AAD` (also written in the folder's README.txt) | identifies this build |

Checks on the same source from the previous step still apply: `cargo test --lib` 33/33, `tsc` 0 errors, `npm run build`, `npm audit` 0 vulnerabilities.

**Not run:**
- The plan's V3c (pulling the USB drive locks the app), a real USB stick, creating a vault in the release build, starting KeePassXC from the release build (Passwords view), and the vault locking and sticky notes in the release build.
- `cargo audit`, the security review, a real USB stick, code signing, SmartScreen behavior, the exe on a PC without the VC++ runtime.

## 6. Problems Encountered

- The first commit attempt failed: PowerShell split the message at its quote characters and treated the pieces as file paths. Nothing was committed by it. The message was rewritten into a file and `git commit -F` was used. (A tool quirk; it echoes the quoting risk already recorded in FI-02 and FI-03, with no effect.)

## 7. Documentation Impact

This file; updates to STATUS, the amendments record (authority row), the acceptance ledger (V3a, V5c, a new X11), the execution index and the session history.

## 8. Git / Version Control Impact

One commit made: `2105a57`. The records (this file and the updates above) were committed separately afterwards in a documentation commit under the same instruction; see `git log`. No push, no remote. The portable folder is outside the repository and is not tracked.

## 9. Decisions Made During Execution

- Build only the exe (bundling is switched off in the config), and assemble the portable folder by hand. Basis: the owner asked for an exe for a portable app, not an installer.
- Put the portable folder outside the repository. Basis: it is 158 MB of binaries and shouldn't be tracked.
- Include the KeePassXC license files and its full source archive. Basis: A9's plan and the GPL duty; the final distribution approach (A9 Q5) is still the owner's decision before the build is given to anyone else.
- Smoke-test the exe from a temporary copy once the dev app was found not running. Basis: the single-instance guard would have handed a second launch to a running dev app, and the test must not touch the owner's vault or window. The copy was deleted afterwards.

## 10. Items That Need Human / Coordinator Review

- Try the exe themselves (create a vault, notes, the Passwords view, lock) and report. Reviewer: owner.
- Copy the current dev vault (`.devdata\data\`) into the portable folder, or start fresh? Not done.
- The GPL approach before the build is shared with anyone (A9 Q5).
- Whether to run `cargo audit` and a security review (plan V6) before relying on the build.

## 11. Current Result

**What now holds:** the code is committed (`2105a57`); a release exe built from it exists; the portable folder is assembled with the verified KeePassXC, its licenses and source; the owner's screenshot setting is recorded.

**What does not:** a vault has not been created in the release build, KeePassXC has not been started from it, V3c is untested; nothing is accepted.

**Delivery state:** `E:\Project\Cipherleaf-portable\` (local, not shared).

**Owner acceptance state:** not requested.

## 12. Recommended Next Step

The owner tries the exe, decides whether to move their dev vault into the portable folder or start fresh, and decides how to treat the GPL point if the build will leave their own PCs.

**Do not repeat:** the build, unless the source changes (a source change makes this exe historical).
