# Execution Summary: EX-2026-10-30 — renamed to Notaya, sticky-notes icon, Sunrise theme (A28)

(4-section form.)

## 1. Task Overview

The owner said "proceed with this and The app name is Notaya" on 2026-10-27 at 16:59:11 UTC (project transcript), with a screenshot of theme 2, Sunrise, and "The theme sunrise will be the defult while the darker will be can be changed" at 17:03:39 UTC. The coordinator also relayed a standing rule from about 16:57 UTC (exact wording not in this thread): record every step on GitHub and on the PC (amendment, sessions, execution report, STATUS, commit hashes), and after the owner merges, fast-forward his PC copy and confirm it matches. Executor: Claude (Claude Code, "Kevin") in the cloud thread "Design new app icon and name", branch `claude/project-thread-669o55`, draft [PR #24](https://github.com/AbyssWatcher2/CipherLeaf/pull/24). Work from about 17:00 UTC (system clock).

## 2. What was done

- **Name screen first** (web search, best effort, not legal clearance): no app, company or trademark named exactly "Notaya" was found; several note apps and a software company (Nota, Inc.) use "Nota". The owner was told at about 17:01 UTC.
- `main` (`2c8c2e6`, with records PR #25) merged into the branch: `537b919`.
- **Code commit `6003e81`:** the rename, the icon and the two Sunrise looks, as listed in [A28](../plan/A28-rename-notaya-icon-sunrise.md). 55 files: Rust messages and two Rust tests' expected texts, `tauri.conf.json`, `Cargo.toml` description, capabilities description, the UI, `index.html`, the mock, the test scripts, `README.md`, `THIRD-PARTY-NOTICES.md`, the icon files. A new Rust unit test checks that Sunrise is the default and that both Sunrise looks survive `sanitized()`.
- Format constants that still say `cipherleaf` got a comment saying they must stay (A28).
- Records: this report, A28, the authority table and amendments list, `sessions.md`, `STATUS.md`, the execution index (commit after `6003e81`, named in the PR).

## 3. Checks (cloud container, Linux, 2026-10-27 about 17:05 to 17:10 UTC, system clock)

- `npm ci` (from the lockfile, no new packages), then `npx tsc --noEmit -p tsconfig.json`: passed. `npm run build`: passed.
- `npx tauri icon assets/notaya-icon.svg`: all icon files regenerated; `icon.ico` holds 16, 24, 32, 48, 64 and 256 px (read with Pillow).
- Mock UI (`npm run mock`, Playwright screenshots): Home in Sunrise and Sunrise dark, Settings with the seven theme tiles, the lock screen with the new mark. Looked right; screenshots in the project files (`notaya-build/`), not in the repo. The mock server was stopped afterwards.
- An independent review agent (did not write the change) read the diff for data-compatibility, compile and security problems: see the addendum below.
- **Not run here:** `cargo test --lib` (the Linux container has no WebView/GTK libraries and no Windows toolchain), `npm run tauri build`, the real-exe tests. They are to run on the owner's PC with throwaway data only.

## 4. Open items

- Rust unit tests, release build and real-exe test on the owner's PC (throwaway `CIPHERLEAF_HOME`, debug port closed afterwards), including: an existing Cipherleaf 1.1.0 vault (throwaway copy) opens in Notaya.exe with its notes, settings and backups.
- The owner's review and merge of PR #24; then fast-forward `E:\Project\cipherleaf` (`git pull --ff-only` on `main`; never `git add -A` there, an untracked `vendor\` folder exists) and confirm it matches GitHub.
- A release (version number, ZIP named Notaya) is a separate step on the owner's word.
- Optional, not built: Sunrise amber and coral as sticky-note colours.

## Addendum (2026-10-27, about 17:10 to 17:16 UTC, system clock): review, fixes, stop

- **Independent review of `6003e81`: no blockers.** It confirmed by reading that the identifier, data paths, format constants, backup names, `CIPHERLEAF_HOME` and browser keys are unchanged, that the changed Rust test texts match their producers, that the new Rust test should compile, and that there is no security change. Its findings and what was done:
  1. Should fix: the real-exe README's safety rule named `E:\Project\Notaya-portable\data` instead of the owner's real vault folder `E:\Project\Cipherleaf-portable\data`. **Fixed** (names the real folder again, and any future `Notaya-portable\data`).
  2. Should fix (records): A28 said an older Cipherleaf changes nothing when it opens a Sunrise vault; in fact it saves Black once it saves any setting. **A28 corrected.**
  3. Nit: the "Theme color" swatch was always drawn coral and black. **Fixed:** it now shows the theme's real colour (checked in the mock: coral in Sunrise, near-black in Paper, white in Black).
  4. Nit: coral as small text on cream is about 3.5 to 1. **Recorded in A28** (owner's colour; not changed).
  5. Nit: "Match Windows" does not follow Sunrise. **Recorded in A28**, as intended.
- After the fixes: `npx tsc --noEmit -p tsconfig.json` passed, `npm run build` passed (cloud). The mock server was stopped.
- **Stopped on the owner's word** "stop now at resume point" (17:13:39 UTC, project transcript, relayed by the coordinator). PC tests were not started.

### Resume point

- **Done:** code, icon, themes, review fixes, records, all pushed to PR #24 (draft).
- **Not done:** `cargo test --lib`, `npm run tauri build` (after `cargo clean -p cipherleaf`), the real-exe test on throwaway data (including a throwaway Cipherleaf 1.1.0 vault opened in `Notaya.exe`), the owner's merge, the PC copy fast-forward.
- **Exact next step (only when the owner says resume):** on the owner's PC, in a temp clone of branch `claude/project-thread-669o55` (not `E:\Project\cipherleaf`), run `cd src-tauri; cargo test --lib`.

## Addendum 2 (2026-10-27, from 18:57 UTC, system clock): resumed, PC checks

- The owner said "Resume" at 18:56:19 UTC (project transcript, relayed 18:56:34). The coordinator also relayed a rule change dated 17:22 UTC (exact owner words not in this thread): **Sonnet 5.5 for everyday work, Opus 5.5 for hard jobs**. This addendum and the later PC checks are done on the new rule; the code of `6003e81` and the review were done under the older one (Opus, high effort).
- Branch head at resume: `029887b`. PC results follow below as they come in.

## Addendum 3 (2026-10-27, 19:1x UTC, system clock): "Why it's still cipherleaf in tests" (owner, 19:10:41 UTC)

A search of `src/`, `src-tauri/src/`, `index.html`, `tauri.conf.json`, the capability file, the mock and the test scripts found no user-visible "Cipherleaf" left (window titles, dialogs, messages, Settings, README, the real-exe scripts). What still says `cipherleaf` is internal and kept on purpose:

- The app identifier `app.cipherleaf.vault` (single-instance guard), the crate and npm package names, the Rust library name `cipherleaf_lib`, the repository and the folder `E:\Project\cipherleaf`. So `cargo test` prints `cipherleaf` and `cipherleaf_lib`, and the build folders are `...\cipherleaf\...`.
- The temp folders the Rust unit tests make (`cipherleaf-test-*`, `cipherleaf-state-*` and similar) and `CIPHERLEAF_HOME`: names only the tests and the scripts see.
- The vault format strings (`cipherleaf` in the encryption label, the open-vault password, the HKDF label) and the browser-storage keys `cipherleaf.*`: changing them would stop existing notes from opening or reset settings.
- One user-visible spot was missed: the exe's "file description" (Properties > Details) came from `Cargo.toml` and said "Notaya (formerly Cipherleaf) - ...". **Fixed:** it now says "Notaya - portable notes app with optional encryption". This changes only exe metadata, so the exe built on the PC from `91d7068` needs one more rebuild and check before merge.

## Addendum 4 (2026-10-27, 18:58 to 19:15 UTC on the owner's PC, system clock): PC checks of `91d7068`

Run in a temp copy (`%TEMP%\notaya-pr24`, `git rev-parse --short HEAD` = `91d7068`), throwaway data only. Nothing was committed, pushed, merged or deleted from the PC; the real vault and `.devdata` were not touched.

- `npm ci` (198 packages, 0 vulnerabilities), `npx tsc --noEmit -p tsconfig.json` and `npm run build`: passed.
- **`cargo test --lib`: 129 passed, 0 failed** (128 on `main` plus the new Sunrise test).
- `cargo clean -p cipherleaf` then `npm run tauri build`: exit 0, 3 min 41 s, 2 existing compiler warnings. `Notaya.exe` 11,261,952 bytes, SHA-256 `CE0AAD39B0D53FB2EE7F633A2FF663AF96F0EC0E250F575E646A92113DA0E7B4`. Version info: ProductName Notaya, FileVersion 1.1.0. The embedded icon is the two sticky notes; the native window title and `document.title` are "Notaya". **This hash is for `91d7068`; the exe description fix in `a97d5e7` needs a rebuild (see below).**
- **Real-exe test** (own driver, throwaway `CIPHERLEAF_HOME`, debug port 9335 for the test process only; the repo runners were not used because they delete their temp folders and the owner said not to delete any, so `run-restoreflow`, `run-lockflow` and `run-optionalsecurity` were **not run**): a new vault opens with theme `sunrise` by default; Settings shows 7 tiles in the order Sunrise, Sunrise dark, Black, Graphite, White, Paper, Match Windows; the Theme color swatch is coral `rgb(232,85,63)` in both Sunrise looks, `rgb(10,10,10)` in White and Paper, `rgb(255,255,255)` in Black and Graphite; Sunrise dark survives a restart (`data-theme` sunrise-dark, mode dark); a note, a sticky and a calculator entry (6*7=42) persist across a restart; Settings > About shows the "called Cipherleaf up to version 1.1.0" line. After the runs: `port 9335 listening after run: 0`, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` unset.
- **Old-vault compatibility** (Cipherleaf 1.1.0 from `gh release download v1.1.0`, read-only; ZIP SHA-256 `2E0B94B1...A890` matches GitHub; exe SHA-256 `FDF988CE87332014D93298EB3D208BB8B2E6A8365A6EC373AFFFFB34BA70B8E5`): two throwaway vaults made with 1.1.0, one without a password and one with a master password, each with a note, a sticky, a calculator entry (2+2=4), the Paper theme and one automatic backup. Their copies opened in the new `Notaya.exe`: notes, sticky, calculator entry and Paper theme intact, the backup listed, choosing Sunrise applies and is saved. The password vault opens locked, the backup is listed while locked, a wrong password is refused ("Wrong master password or keyfile for this Notaya vault.") and the right one unlocks. **Not done:** restoring from the listed backup (only listed). Harness note: in 1.1.0 `backup_now` opens a file dialog that stalled the first attempt; only the test's own processes were stopped and the port was confirmed closed.
- Cleanup: no test processes left, port closed, the environment variable unset, `E:\Project\cipherleaf` working tree unchanged (only the untracked `vendor/`). Its `HEAD` read `5013244` on `main` at the end, `6981442` at the start: the session ran no git command that moves it, so it was moved by something else (not checked here). Temp folders left under `%TEMP%`: `notaya-pr24`, `clf-notaya-pr24-test`, `clf-notaya-pr24-home`, `clf-v110-release`, `clf-cipherleaf-v110-exe`, `clf-old-A-home`, `clf-old-A2-home`, `clf-old-A2-copy`, `clf-old-B2-home`, `clf-old-B2-copy`.
- **Open:** rebuild `a97d5e7` and check the exe description; the owner's own look at the real window.

## Addendum 5 (2026-10-27, 19:16 to 19:21 UTC on the owner's PC, system clock): description fix `a97d5e7`

Checked at `e537071` in the same temp clone: `cargo test --lib` 129 of 129; `cargo clean -p cipherleaf` and `npm run tauri build` exit 0; `Notaya.exe` 11,261,440 bytes, SHA-256 `4CC4949EE066438E292AA7D4D9C23C7F57DE1A60632DD5708C3D5F1BCDC9AF4E`. Smoke test on throwaway data: opens with title "Notaya", theme sunrise, port closed afterwards. Properties > Details: FileDescription Notaya, ProductName Notaya, FileVersion and ProductVersion 1.1.0; InternalName, OriginalFilename, LegalCopyright and Comments are empty. **CompanyName still said "cipherleaf"** (most likely taken from the identifier `app.cipherleaf.vault`, not confirmed). Fix attempted: `"publisher": "Notaya"` under `bundle` in `tauri.conf.json` (the identifier is unchanged). Result in addendum 6.

## Addendum 6 (2026-10-27, 19:21 to 19:25 UTC on the owner's PC, system clock): publisher fix `d4933ef`

Checked at `d4933ef` in the same temp clone: `git diff e537071 -- src src-tauri` is only `tauri.conf.json`, +1 line (`"publisher": "Notaya"`). `cargo test --lib` 129 of 129; `cargo clean -p cipherleaf` and `npm run tauri build` exit 0. `Notaya.exe` 11,260,928 bytes, SHA-256 `37243FC24796D76C93C4DCF0B21BDD92FED7E419F827F0B67321CD24EE252D8D`. Properties > Details: **CompanyName Notaya** (was "cipherleaf"), FileDescription Notaya, ProductName Notaya, FileVersion and ProductVersion 1.1.0; InternalName, OriginalFilename, LegalCopyright and Comments are empty (tauri-build 2.7.4 does not set them; not changed). Cause found in tauri-build 2.7.4 `src/lib.rs`: CompanyName is `bundle.publisher` if set, otherwise the second dot-segment of the identifier ("cipherleaf"). Smoke test on throwaway data: title "Notaya", theme sunrise, new vault, port 9335 closed afterwards, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` unset, no test processes left; `E:\Project\cipherleaf` status unchanged. No user-visible "Cipherleaf" remains in the exe's file properties.

**State for the owner:** the code is `d4933ef` (the records commits after it are docs only). Tested on the PC: `cargo test --lib` 129 of 129, release build, real-exe test on throwaway data, an old Cipherleaf 1.1.0 vault (with and without a password) opening in `Notaya.exe`, and the exe metadata. Not run: the repo's runners (`run-restoreflow`, `run-lockflow`, `run-optionalsecurity`, they delete their temp folders), a restore from the listed 1.1.0 backup, keyboard-only use and a screen reader. The full real-exe pass above was on `91d7068`; later commits changed only metadata (description, publisher). Not accepted. A release is a separate step.

## Addendum 7 (2026-10-27, about 19:30 UTC, system clock): numbering and merge of `main`

- `main` moved on while this PR waited (PR #27 `5013244`, PR #28 `b11ec6d`: the live-audit comparison, "record every step", and the PBS adoption, A29). Those PRs also used the number EX-2026-10-27-27 (the "record every step" report), so **this report is renumbered EX-2026-10-30** (the numbers 28 and 29 are taken too). Earlier addenda and commit messages that say EX-2026-10-27-27 mean this report.
- `main` was merged into the branch; only the records files conflicted (`STATUS.md`, `sessions.md`, `approval-and-amendments.md`, `execution/README.md`) and were resolved by keeping both sides. No source file changed. `CLAUDE.md` now has the "Cloud threads" and "PBS work cycle" sections from A29; this PR follows them (baseline `b11ec6d`; the task, scope and stop point are in sections 1 and 4 above).

## Addendum 8 (2026-10-27, 19:39 UTC, project transcript): the owner's look at the real window

The owner tried `Notaya.exe` (build of `d4933ef`, in `%TEMP%\notaya-pr24\src-tauri\target\release\`) and said "Okay looks good" at 19:39:24 UTC. Recorded as an informal visual check at X26 in `docs/acceptance.md`; **not an acceptance** (no requirement named, and he did not say "accepted"). It is not known which of the suggested checks he made (new vault, themes, a copy of his old notes). Result state: **verified** on the PC checks above; **integrated** only after he merges PR #24.
