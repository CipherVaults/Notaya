# Execution Summary: EX-2026-10-27-31 — Notaya V1.2.0 portable folder and ZIP

(4-section form, big work: a release.)

## 1. Task Overview

The owner wrote "Release one" in the project chat at 2026-10-27 19:40:47 UTC (message timestamp), right after merging PR #24 (the Notaya rename, A28). The version is **1.2.0** ("Notaya"), proposed by Claude because the app got a new name, icon and default look; the owner did not pick another number. Scope: version files, release notes, records. Out of scope: new features, audit fixes, the installer, renaming the GitHub repository (it is still `CipherLeaf`). The build and the tests ran on the owner's PC in a throwaway clone (`%TEMP%\notaya-v120-release`); his project folder `E:\Project\cipherleaf` and every real vault were not used. Times are UTC, from the PC's system clock unless marked.

Baseline: `origin/main` `49cd9de` (PR #24 merged at `9510b54`, records PR #29 merged on top). Result state: **produced** and **verified** for the checks listed below; not integrated, not released, not accepted.

## 2. What was done

- **Version bump**, commit `7e84cdc` (first on `9510b54`, then main moved to `49cd9de` with docs only, so `main` was merged into the branch: `e25f80411b5a652b93db02faadc05ddb649d359c`). Five files: `package.json`, `package-lock.json` (root package only), `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, the `cipherleaf` entry in `src-tauri/Cargo.lock`, 1.1.0 to 1.2.0. The diff against `main` `49cd9de` is exactly those five files. A first build on `7e84cdc` was thrown away and nothing from it shipped.
- **Build (19:43 to 19:52):** `npm ci` ok (198 packages, 0 vulnerabilities); `npx tsc --noEmit -p tsconfig.json` ok; `npm run build` ok; `cargo clean -p cipherleaf` (so the new icon is used); `cargo test --lib` **129 passed, 0 failed**; `npm run tauri build` ok (3 min 36 s, 2 existing compiler warnings). `npm audit`: 0 vulnerabilities. `cargo audit`: not run (not installed; nothing was installed).
- **Exe** `Notaya.exe`: 11,260,928 bytes, SHA-256 `D04D42F584CAFCAAA06F7949CCB6E5E11328BCA4A806CEB810AB29F252F705C2`. FileVersion and ProductVersion 1.2.0; ProductName, FileDescription and CompanyName say Notaya; LegalCopyright, OriginalFilename and InternalName are empty. The 256 px icon was extracted and looked at: two sticky notes, amber behind, coral in front, on a cream rounded square.
- **Folder** `E:\Project\Notaya-V1.2.0-2026-10-27\Notaya\`, 5 files: `Notaya.exe`; `README.txt` (V1.2.0, build commit, exe hash, an "Updating from Cipherleaf" part); `Licenses\THIRD-PARTY-NOTICES.md` (equal to the repo file), `Licenses\README.txt`, `Licenses\WebView2-LICENSE.txt`. No `data\`, `.devdata`, `*.clf`, `*.key`, `*.kdbx`, `.env` or CSV.
- **ZIP** `E:\Project\Notaya-V1.2.0-2026-10-27\Notaya-V1.2.0-portable.zip`: 5,915,148 bytes, SHA-256 `A7FDD82201315C95244F72E20069C4B794A0EA7D920C496723B83E40E41CC77D`, 5 entries under one top-level folder `Notaya/`, written entry by entry with `/` names (0 backslash names). A fresh extraction: 0 of 5 files differ from the folder, none extra.
- **Release notes** `E:\Project\Notaya-V1.2.0-2026-10-27\RELEASE-NOTES.md` (next to the folder, not in the ZIP; UTF-8 without BOM). They explain the rename, the icon and the Sunrise default, say that Cipherleaf data opens unchanged and how to update, carry the commit and both hashes, and keep the V1.1.0 known limits plus the new ones (contrast, Match Windows, internal names, an older Cipherleaf shows Black).
- The V1.0, V1.0.1 and V1.1.0 folders, ZIPs and GitHub releases, and `E:\Project\Cipherleaf-portable`, were not touched.

## 3. Checks on the built exe (a fresh extraction of the ZIP, throwaway data; the release folder was never started)

Several attempts were needed, so the order is recorded:

- 19:5x and again after the owner's decision card (19:56:25, "Run them, deletions allowed"): the PC session's automatic permission check ("Security Weaken") refused the runner commands. The owner then saved `C:\Users\moham\.claude\settings.json` with `permissions.allow ["PowerShell"]` (20:08:44) and the refusal continued (the check is not overridden by that rule). Nothing was run in these attempts.
- 22:10 UTC: the owner ran the three runners himself in an **Administrator** PowerShell. Each printed the right exe hash and the app started and made `data\vault.clf`, but every Node script failed with "no main window" or "no matching window", so **no check ran**. Cleanup lines were clean (port closed, env var not set, no process left, temp folders removed). Not called a pass.
- 22:17 to 22:23 UTC, a new Remote Control session, **not elevated** (Medium Mandatory Level):
  - Diagnosis: the shipped exe copy started with the debug port set for the child process only; `/json` after 10 s returned one target (type page, title "Notaya", url `http://tauri.localhost/`); the window was visible; `data\vault.clf` was created. The process was stopped, port 9335 closed, env var unset, the diag folder removed.
  - Unmodified runners from `tools/real-exe-tests/` at `e25f804`, each printing exe SHA-256 `D04D42F5…F705C2`: **restore flow 13 PASS, 0 FAIL** (locked and unlocked restore, bad path, UNC path and traversal refused); **lock flow** page, sticky and close scenarios: the typed text was saved in the right place in each (`FLUSHTEST-page` in the page, `-sticky` in the sticky, `-close` in the page after closing), the app closed as expected; **optional security 47 of 47** (p1 8/8, p2 21/21, p3 11/11, p4 5/5, p5 2/2, "phases with failures: none"; the app reported version 1.2.0).
  - **Migration check:** the V1.1.0 ZIP (SHA-256 `2E0B94B1…A890`) extracted to a temp copy; its `Cipherleaf.exe` (`FDF988CE…B8E5`, version 1.1.0) made a throwaway vault via `CIPHERLEAF_HOME` with notebook "MigNotebook", page "MigPage" (`MIG-NOTE-4417`), a sticky note (`MIG-STICKY-5528`), the calculator line `6*7=42` and the Paper theme, and left one backup. The same data opened with the shipped `Notaya.exe` (version 1.2.0) showed the notebook, page text, sticky, calculator history, theme `paper` and the backup, all intact. No new backup appeared when Notaya closed; the reason was not looked into.
  - After each run: port 9335 closed, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` unset, no Notaya or Cipherleaf process left.
- The cause of the 22:10 failure is probably the elevated shell, but it was **not proven**: the runners were not run elevated by the PC session.
- **Not run:** `cargo audit`; the runners from an elevated shell; a password-protected vault in the migration check (open vault only); restoring from the old backup; the window's look on screen for this exact exe (the owner looked at the PR #24 build at 19:39:24, informally); idle lock, Win+L, sleep and USB removal; keyboard-only use.
- Left on the PC by design (nothing deleted): `%TEMP%\notaya-v120-release` (clone), `%TEMP%\notaya-v120-zip-extract-tests`, `%TEMP%\notaya-v120-zipcheck`, `%TEMP%\clf-mig-v110-extract*`, `%TEMP%\clf-mig-home*`, the build log `%TEMP%\notaya-v120-release-build-e25f804.log`.

## 4. Open items

- **Not released.** The owner merges the PR that carries the version bump and these records (so the tag's commit is in `main`), then runs in his own terminal (agents are blocked from `gh` writes):

  ```
  gh release create v1.2.0 "E:\Project\Notaya-V1.2.0-2026-10-27\Notaya-V1.2.0-portable.zip" --repo AbyssWatcher2/CipherLeaf --target e25f80411b5a652b93db02faadc05ddb649d359c --title "Notaya V1.2.0" --notes-file "E:\Project\Notaya-V1.2.0-2026-10-27\RELEASE-NOTES.md" --latest
  ```

  Afterwards the release is checked on GitHub (tag, target, one asset, digest equal to the ZIP hash) and recorded as an addendum here.
- The owner's PowerShell permission: `C:\Users\moham\.claude\settings.json` now allows all of PowerShell for Claude on that PC (his choice, 20:00:02 and 20:08:44 UTC). The owner rules in `CLAUDE.md` still apply. Remove it when he no longer wants it.
- The owner ran a command at about 22:10 UTC that removed the old test worktrees and every `%TEMP%\clf-*` folder (his pasted output ends with a count of 0). That answers STATUS question 21.
- Nothing is accepted.

## Addendum: the release (2026-10-27 22:28:04 UTC)

The owner merged [PR #30](https://github.com/AbyssWatcher2/CipherLeaf/pull/30) with a merge commit at about 22:26 UTC (GitHub; `main` `4e40d1a`, parents `49cd9de` and `f7ce9c5`). The build commit `e25f804` is in `main`'s history (checked with `git merge-base --is-ancestor`). His PC copy `E:\Project\cipherleaf` was fast-forwarded `49cd9de` to `4e40d1a` and equals `origin/main` (22:27:26 UTC, PC clock; `git status` only `?? vendor/`, `package.json` says 1.2.0). He then ran the `gh release create` command in his own terminal and wrote "Done" at 22:28:11 UTC (message timestamp). Checked through the GitHub API (after 22:28 UTC); the asset was not downloaded and re-hashed:

- [Release v1.2.0](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.2.0), "Notaya V1.2.0", published 22:28:04 UTC, not a draft, not a prerelease, and the latest release (`releases/latest` returns `v1.2.0`).
- Tag `v1.2.0` points at `e25f80411b5a652b93db02faadc05ddb649d359c`, the commit the exe was built from.
- One asset, `Notaya-V1.2.0-portable.zip`, 5,915,148 bytes, GitHub digest `sha256:a7fdd82201315c95244f72e20069c4b794a0ea7d920c496723b83e40e41cc77d`, equal to the built ZIP's size and SHA-256.
- The release text is the prepared notes with the commit and both hashes filled in (read back from the API; the PC file itself was not compared byte for byte).
- Releases `v1.0` (tag on `0ed3f52`, asset 87,186,510 bytes, digest `cab1e72e…d53d`), `v1.0.1` (tag on `6fa9ee5`, asset 87,224,165 bytes, digest `863500c6…e321`) and `v1.1.0` are unchanged.

Not checked: the downloaded asset. The owner has not accepted the release. Still open: the PowerShell allow rule in `C:\Users\moham\.claude\settings.json` (his choice; remove it when he wants), and whether an Administrator shell really was the cause of his failed 22:10 runs.
