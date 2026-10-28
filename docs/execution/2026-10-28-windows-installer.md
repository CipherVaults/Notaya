# EX-2026-10-28-33: Windows installer (A30), built in the cloud

Short form (4 sections). Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `main` `a4574be` (Notaya V1.2.0), merged into the branch `claude/project-thread-xphhp4`; draft [PR #23](https://github.com/AbyssWatcher2/CipherLeaf/pull/23).
- **Owner's words:** "All recommendations" (2026-10-28 11:56:21) for the 10 decisions of [the plan](../plan/windows-installer-plan.md); "Ok for all go now" (12:24:16), answering the go to build and the OK to download the NSIS tool on his PC.
- **Scope and write area:** the data-folder choice (`portable.rs`), the import (`import.rs`, one command, the first screen), the setup settings (`tauri.conf.json`, `packaging/installer-hooks.nsh`), a build script (`tools/windows-build/`), the real-exe runners, and the records. No release, no version bump, no merge, no signing, no updater.
- **Returns:** the code, the checks, and what is not yet tested. **Stop point:** the PC tests on throwaway data, then a report; a release only on the owner's word.

## 2. What was done

- `src-tauri/src/portable.rs`: `Paths.installed`; `choose_root` (`portable.txt` or an existing `data` folder next to the exe = portable; otherwise `%LOCALAPPDATA%\Notaya Notes`; `CIPHERLEAF_HOME` still wins). 4 new tests.
- `src-tauri/src/import.rs` (new) and command `import_portable_notes`: copy only, folder chosen by a dialog opened in Rust, only for an installed copy with no notes yet, never over a vault, never from the live folder or a network path, a `-journal` refuses, checked copy then rename, backups copied best effort. 12 new tests. Small visibility changes in `restore.rs`.
- `vault_status` has `offerImport`; an installed copy no longer creates an empty vault at once on its first start. The first screen (`UnlockScreen.tsx`) offers Start new notes and Import. The mock shows it with `?s=start&installed`; a screenshot is in the project files (`installer/first-screen-installed.png`).
- `tauri.conf.json`: bundle on, NSIS only, per-user, WebView2 download if missing, `allowDowngrades` false, installer icon, hooks file (a message after an interactive uninstall that the notes were kept).
- `tools/windows-build/build-windows.ps1` (setup, portable folder with `portable.txt`, ZIP, hashes) and `tools/real-exe-tests/run-installed.ps1` + `installed.mjs`; the three older runners now copy the exe with `portable.txt` (a bare temp folder would now count as installed and use the real `%LOCALAPPDATA%`).
- Records: A30 and an authority row in `approval-and-amendments.md`, P22 and P9 in `docs/security-design.md`, the README install section, STATUS question 19, the plan.
- **Found while building:** the setup's default install folder for a per-user install is `%LOCALAPPDATA%\Notaya` (read from the `tauri-bundler` 2.10.4 template), so notes in `...\Notaya\data` would share the program folder and, once `data` exists next to the exe, be taken for a portable copy. The notes folder was therefore named `Notaya Notes`. This departs from the wording of decision 3; the owner confirmed it ("Yes", 12:37:12).

## 3. Checks

- **Cloud (Linux container; Wine 9 and `x86_64-pc-windows-gnu`, installed in the container for this task):** `npx tsc --noEmit` ok; `npm run build` ok; `cargo test --lib --target x86_64-pc-windows-gnu` under Wine: **145 passed, 0 failed** (129 before; 16 new), on the working tree at the time (not yet a commit); the mock screens looked at (first screen and after the import).
- **Not run:** `npm audit`, `cargo clippy` (MSVC), the real exe, `npm run tauri build` and the NSIS setup (nothing built on Windows yet), `run-installed.ps1` and the three older runners with the new `Copy-TestExe`, install, upgrade and uninstall, the desktop shortcut tick box, the WebView2 download, the native folder dialog of the import, SmartScreen. `build-windows.ps1` was never run. The NSIS hook text was not compiled.
- **Independent review** (a read-only Claude subagent, model output, at `ba83a68`, nothing run): no way found to overwrite or delete notes. Fixed after it (commit after `fc5843d`): the build script now makes `-OutDir` absolute and takes the version from `tauri.conf.json`; the uninstall message is no longer shown during an upgrade or a passive uninstall; the welcome card shows where notes will be kept; Import is also offered after a Start over (earlier copies exist); the README and the import's error say that an installed and a portable Notaya cannot run together (single-instance). Left as they are (documented): a lone `Notaya.exe` without `portable.txt` counts as installed; with `LOCALAPPDATA` unset an installed copy falls back to `<install dir>\data`, which then makes it portable; the setup's own "is it running" check also matches a portable `Notaya.exe`. The saved skill `cipherleaf-real-exe-test` should copy the exe with `portable.txt` (the repo runners already do).
- Result state: *produced*; cloud-*verified* only for the Rust logic and the UI types and build. Not integrated, not accepted.

## 4. Open items

- PC tests on throwaway data through a Remote Control session (the owner's approval prompt applies): build with `build-windows.ps1`, run `run-installed.ps1` and the older runners, install the setup to a temp folder with `/S /D=...` (registry, Start menu and shortcut entries are checked and removed by the uninstall), and import through the folder dialog.
- The owner merges PR #23 when he wants, and decides on the release (V1.3.0 planned; the version bump is not done).
- The README has an install section that talks about "releases that include the setup": true only once a release has it.
