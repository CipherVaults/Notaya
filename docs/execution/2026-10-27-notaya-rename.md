# Execution Summary: EX-2026-10-27-27 — renamed to Notaya, sticky-notes icon, Sunrise theme (A28)

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
