# Execution Summary: EX-2026-10-28-34 — Notaya V1.3.0 (setup and portable ZIP)

(4-section form, big work: a release.)

## 1. Task Overview

- **Baseline:** `origin/main` `52841f1` (installer PR #23 merged as `7874005`, records PR #32). `main` then moved to `37774c6` (PR #33, native app proposal dropped, docs only) and was merged into the branch.
- **Owner's words:** "Release" (2026-10-28 14:35:46 UTC, message timestamp); at 15:27:09 UTC he tapped "Fix first" on the card "Ship V1.3.0 as built, or fix the update message first?".
- **Scope and write area:** the version 1.2.0 to 1.3.0 in five files; one change in `packaging/installer-hooks.nsh`; the release notes (`docs/release-notes/v1.3.0.md`); the records. No other code. **Out of scope:** signing, an updater, the shortcut check, new features.
- **Returns:** two files, `Notaya-V1.3.0-setup.exe` and `Notaya-V1.3.0-portable.zip`, with their hashes, release notes, the merge order and the command for the owner. **Stop point:** the owner merges and publishes himself. Branch `claude/project-thread-fhcwas`, draft [PR #34](https://github.com/AbyssWatcher2/CipherLeaf/pull/34).
- All PC work ran in `E:\Project\Notaya Tests Excluded` only (his rule of 2026-10-28 14:03:51), on throwaway data. Times are UTC, PC system clock.

## 2. What was done

- **Version bump**, commit `6f582dd` (`package.json`, `package-lock.json` root and `packages[""]`, `tauri.conf.json`, `Cargo.toml`, the `cipherleaf` entry in `Cargo.lock`).
- **First build and test** at `6f582dd` (14:38 to 15:14, folder `v130`): `tsc`, `npm run build`, `npm audit` (0), `cargo test --lib` 145 passed; the four runners; silent install, update, uninstall; the V1.2.0 portable ZIP (hash `A7FDD822…CC77D` checked, read only) opened by the new exe unchanged and imported through the real folder dialog; update over an older setup; the interactive setup and uninstall windows. **Finding:** updating through the setup window ("Uninstall before installing", the default on the "Already Installed" page) runs the old uninstaller as a visible wizard and shows the box "Notaya was removed from this PC. ... NOT deleted" in the middle of the update. Nothing is lost. The cause is in the Tauri NSIS template (read from `tauri-bundler` 2.10.4): the setup hides itself and runs the old uninstaller without `/UPDATE`, so the hook's `$UpdateMode` test never matches. These files are **superseded and are not released**.
- **Fix** (owner: "Fix first"), commit `f970331`: the `NSIS_HOOK_POSTUNINSTALL` hook also stays silent when a window titled "Notaya Setup" exists (`FindWindow`). The hook lives in the installed version's own uninstaller, so the fix helps every upgrade **from V1.3.0 on**; an uninstaller of a build that never shipped with it (anything up to V1.2.0-numbered test setups) still shows the box once.
- **Second build and full retest** at `f970331` (folder `v130b`, 15:30 to 16:01): clone clean, `git diff 6f582dd f970331` shows only docs and the hook; `tsc`, `build`, `npm audit` 0 vulnerabilities, `cargo test --lib` **145 passed, 0 failed**; no NSIS warning or error (only two Rust "never used" warnings). Build exit code **not captured** (the files and `hashes.txt` exist; the first attempt died after 1 s because of the spaces in the `-File` path).
  - `Notaya.exe` 11,283,456 bytes SHA-256 `571AC4F9A3AEFC3ECC9756A2066FB44682A7883E171D78A9F9D0D9C1A5546CB7`
  - `Notaya-V1.3.0-portable.zip` 5,924,682 bytes `494ACE82E733F1B83DF6D4A89531712A7A8A4E6D4584769E3E756A518E32A1C9` (6 entries under `Notaya/`: `Notaya.exe`, `portable.txt`, `README.txt`, `Licenses/` three files; no `data`, no vault, key or csv)
  - `Notaya-V1.3.0-setup.exe` 4,974,334 bytes `5A16FD0CA19388E3D3BC6311A41918EA7D98B8EFC76379110F75087FB52BE4C1`
  - Version info 1.3.0 on exe and setup; both unsigned. Re-hashed after the tests: equal. Files: `E:\Project\Notaya Tests Excluded\v130b\out\`.

## 3. Checks on the V1.3.0 files (`f970331`)

- Runners on the exe from the extracted ZIP: `run-installed` all PASS, `run-restoreflow` 12 PASS 0 FAIL, `run-optionalsecurity` 8/8, 21/21, 11/11, 5/5, 2/2, `run-lockflow` text in the right note each time. After each: port 9335 closed, env var unset, no process left.
- Silent install, same setup over it, silent uninstall (exit 0 each; vault SHA-256 unchanged; notes kept). V1.2.0 portable data: the new exe opens a copy of it unchanged (notes, theme Paper, the backup; two layouts); import through the real folder dialog copies `vault.clf` and the backup with identical SHA-256 and leaves the V1.2.0 folder unchanged. The new setup silently over the earlier 1.2.0-numbered setup (`57654608…`): one Uninstall key, DisplayVersion 1.3.0, vault unchanged. The old 1.2.0 setup over the 1.3.0 install: exit 2, nothing changed.
- **Interactive (windows driven through messages to their own process; screenshots in `v130b\shots`):** pages Welcome, Choose Install Location, Install, Finish ("Run Notaya" and "Create desktop shortcut" ticked by default). (B) A standalone interactive uninstall still shows the "NOT deleted" box with the unchanged text. (C) Silent and passive uninstall and upgrade show no box. (F) A copy of the same commit numbered 1.2.9 (not a release file) installed with a note, then the release setup run interactively over it with "Uninstall before installing": the nested uninstaller wizard appears, **no "removed" box**, version 1.3.0, one Uninstall key, vault unchanged, note opens. (D) Over the old 1.2.0-numbered install the box still appears once (old hook, expected). (E) Same-version setup, "Uninstall Notaya" on the page: no box, but the setup then goes on and installs again (stock behaviour; listed in the notes).
- **Incident in test E:** the Finish page was left with "Run Notaya" ticked, so the setup started a test copy of `Notaya.exe` in the owner's real Windows session at 15:54:06; it was closed at once. It created `C:\Users\moham\AppData\Local\Notaya Notes\` with `data\backups` (empty) and `data\webview\EBWebView` (172 WebView2 files) and **no `vault.clf`**. Real `%APPDATA%` unchanged. Not deleted (needs his OK). See FI-05.
- Cleanup: no Uninstall key, no Run entry, port 9335 closed, no process; `HKCU\Software\Notaya` (left by the installer after each uninstall) removed; the two old `Notaya.lnk` files unchanged (same SHA-256). `E:\Project\cipherleaf` not touched by the tests.
- **Not run:** SmartScreen; the WebView2 download; the ticked "Create desktop shortcut" result (the old `Notaya.lnk` blocks the check); the setup on another PC; real mouse and keyboard clicks through the setup; `cargo audit`; `cargo clippy` (MSVC). The build exit code was not recorded.
- Result state: *produced* and *verified* for the checks above on these exact files. Not integrated, not released, not accepted.

## 4. Open items

- **Merge order:** only [PR #34](https://github.com/AbyssWatcher2/CipherLeaf/pull/34) ("Create a merge commit", not squash). The build commit `f970331` is on its branch and so in `main` afterwards. Then, in a normal PowerShell:

  ```
  gh release create v1.3.0 "E:\Project\Notaya Tests Excluded\v130b\out\Notaya-V1.3.0-setup.exe" "E:\Project\Notaya Tests Excluded\v130b\out\Notaya-V1.3.0-portable.zip" --repo AbyssWatcher2/CipherLeaf --target f970331cb9724142e07c926d04e339eaf4b9d6e9 --title "Notaya V1.3.0" --notes-file "E:\Project\Notaya Tests Excluded\v130b\RELEASE-NOTES.md" --latest
  ```

  Afterwards: the release is checked on GitHub (tag, target, two assets, digests equal to the hashes above) and recorded as an addendum here; the PC copy `E:\Project\cipherleaf` is fast-forwarded to `main` and checked equal to `origin/main`. Earlier releases and their assets are not touched.
- The folder `C:\Users\moham\AppData\Local\Notaya Notes\` in the real profile (WebView2 cache only): leave or delete, the owner decides.
- Known and listed in the release notes: the old-hook box on the first upgrade from a build before V1.3.0 (none was released as a setup); "Uninstall Notaya" then re-installing; the stock side picture; the empty registry key; no signing.
- The `v130` folder (superseded first build) and `v130b` stay in the test folder; deleting them needs his OK.
- Nothing is accepted.
