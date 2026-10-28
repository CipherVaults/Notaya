# Execution Summary: EX-2026-10-28-37 — PC folder moved; V1.4.0 (tray, start with Windows, uninstaller box, Settings text)

(4-section form, normal work. The file name keeps the first scope of the thread, "re-release V1.3.0", which became V1.4.0.)

## 1. Task Overview

- **Baseline:** `origin/main` `42de167`.
- **Owner's words (UTC, message timestamps):** 17:01:14 "i would like to remove and re release notaya v 1.3.0 / but befor that i would like to change the project folders to E:\Project\Notaya and change the main folder name to notaya"; 17:07:41 "first" (the repository folder itself); 17:13:59 "that was in the setting in notaya" (the two lines of his screenshot are in the app's Settings, not in the release notes); 17:20:47 "V1.4.1 / also when i cliecked on delte data or unchecked it said the same message"; 17:21:07 "sorry v1.4" (so **V1.4.0**, and V1.3.0 is kept, nothing is deleted); 17:22:17 close to a minimized state, run all the time; 17:23:31 (edited 17:23:48) start with Windows option, start minimized option, close to tray on by default; 17:41:20 and 17:42:06 whether everything is in the new folder.
- **Scope and write area:** `SettingsView.tsx`; version files; `packaging/installer-template.nsi` and `tauri.conf.json`; the tray and start-up code (`shell.rs`, `commands/shell.rs`, `state.rs`, `lib.rs`, `commands/vault.rs`, `Cargo.toml`); the Settings > Windows UI and its mock; the real-exe test scripts and their old paths; the records. Out of scope: encryption, vault format, auto-lock timing, older releases.
- **Stop point:** a draft PR ([#39](https://github.com/AbyssWatcher2/Notaya/pull/39)) with the code and checks. The PC build and tests, the merge, the release notes with the final hashes and the publishing wait for the owner (he merges and says "publish").

## 2. What was done

- **Folder:** the owner moved the repository folder to `E:\Project\Notaya\Notaya` himself (a session cannot rename the folder it runs in), and moved everything that was in `E:\Project` into `E:\Project\Notaya`: the test folder is now `E:\Project\Notaya\Notaya Tests Excluded`, his real portable app is `E:\Project\Notaya\Cipherleaf-portable` (never touched), and the old release folders sit beside it. A read-only Remote Control check at 17:12 UTC found the repository clean, `origin` = `https://github.com/AbyssWatcher2/Notaya.git`, `HEAD` = `origin/main` = `42de167`. Nothing was deleted or changed on the PC. The old paths in the test scripts, their README and `CLAUDE.md` were updated; older records keep their old paths on purpose (they describe what was true then).
- **Settings text:** both bullets removed from "Good to know". The published V1.3.0 notes never had them.
- **Uninstaller:** the stock "Delete the application data" box (it only ever removed the WebView2 folders named after the app identifier, never the notes) is gone, through a copy of tauri-bundler 2.10.4's `installer.nsi` without the box (`packaging/installer-template.nsi`, set as `bundle.windows.nsis.template`). The CLI in use (2.12.4) embeds exactly that bundler version. The state variable stays empty, so the template's delete block never runs; the stock removal of the `Run` value on a real uninstall is kept.
- **Tray and start with Windows (A31):** see [A31](../plan/A31-tray-and-start-with-windows.md). The owner's decision card (recommended option: tray on by default, protected vault locked when hidden) was answered by his 17:23:31 message in words.
- **Independent review:** one read-only agent reviewed the diff (model output, not a person). Findings fixed: a lock reload would have shown the hidden window again (a `tray_hidden` flag now holds it; opening from the tray clears it); an unlock still running when the window is closed could leave the vault open in the tray (a gate now refuses and wipes that session); a vault with security off now gets its ordinary backup on hide when something changed; a failed backup on hide is told in a message box; the Run value is compared without regard to case; two quick Settings switches can no longer undo each other; `run-installed.ps1` starts its copy with close-to-tray off; the records (A31, P9, P22, P23). Not changed: the uninstaller keeps no `HKCU\Software\Notaya\Notaya` clean-up (as before when the box was left unticked).

## 3. Checks

On this source, in the cloud thread: `npx tsc --noEmit -p tsconfig.json` passes; `npm run build` passed on the first commit (the UI changed since, `tsc` passes again); `cargo test --lib --target x86_64-pc-windows-gnu` under Wine: **151 passed, 0 failed** (145 before; 6 new: the prefs file, the Run command, the case-insensitive Run match, and a Run-value write/read/remove in the Wine registry). The Settings > Windows section was looked at in the mock UI (screenshot in the project files, `v1-4-0/settings-windows-section.png`).
**Not run:** the PC build and NSIS setup; the tray icon and its menu in a real window; closing to the tray with a protected and an open vault in the real exe (`tools/real-exe-tests/run-tray.ps1`, written, not run); starting with Windows at a real sign-in; the uninstaller with the old box gone (silent and window); the older real-exe scripts with the new default; `cargo clippy` for MSVC; SmartScreen.

## 4. Result and open points

State: **produced** and **verified** only for the cloud checks above. Open:

- The owner merges PR #39 ("Create a merge commit"). Claude builds and tests on the PC in `E:\Project\Notaya\Notaya Tests Excluded` (the Bitdefender exclusion for the new path is the owner's to set), then the release notes get the final commit and SHA-256 values and the owner says "publish".
- V1.3.0 is **not** removed; V1.4.0 is a new release (tag `v1.4.0`).
- Anyone who updates from V1.3.0 keeps the old uninstaller's box on a manual uninstall until the update has replaced it.
