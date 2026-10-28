# Execution Summary: EX-2026-10-28-37 — PC folder moved; two Settings lines removed for a V1.3.0 re-release

(4-section form, normal work.)

## 1. Task Overview

- **Baseline:** `origin/main` `42de167`.
- **Owner's words:** "i would like to remove and re release notaya v 1.3.0 / but befor that i would like to change the project folders to E:\Project\Notaya and change the main folder name to notaya" (2026-10-28 17:01:14 UTC, message timestamp). "first" at 17:07:41 meant the repository folder itself. At 17:13:59 he said "that was in the setting in notaya": the two lines in his screenshot are in the app's Settings, not in the release notes.
- **Scope and write area:** `src/features/settings/SettingsView.tsx` (two list items) and the records. Out of scope: any security, storage or installer change; older releases; the Bitdefender setup.
- **Stop point:** a draft PR with the source change and checks. Building on the PC, deleting release `v1.3.0` and its tag, and publishing wait for the owner (merge, then his yes).

## 2. What was done

- **Folder:** the owner moved the repository folder to `E:\Project\Notaya\Notaya` himself (a session cannot rename the folder it runs in). A read-only Remote Control check at 17:12 UTC found it clean, `origin` = `https://github.com/AbyssWatcher2/Notaya.git`, `HEAD` = `origin/main` = `42de167`, `CLAUDE.md` and `docs\STATUS.md` present, and no `E:\Project\cipherleaf` left. He moved everything that was in `E:\Project` into `E:\Project\Notaya`, so the test folder is now `E:\Project\Notaya\Notaya Tests Excluded` (the older release folders sit beside the repository). Nothing was deleted or changed on the PC.
- **Published V1.3.0 notes checked:** they contain neither of the two lines from the screenshot. The lines are in `SettingsView.tsx` ("Good to know": "Notaya was called Cipherleaf up to version 1.1.0…" and "Notaya no longer includes KeePassXC…").
- **Source:** both bullets removed (2 lines deleted). The KaTeX bullet stays. No other file in `src` mentions them.

## 3. Checks

Run in the cloud thread on this exact source: `npm ci`, `npx tsc --noEmit -p tsconfig.json` (passes), `npm run build` (passes). **Not run:** Rust tests (no Rust change), the PC build, the real-exe scripts, the installer scripts, a look at the real window.

## 4. Result and open points

State: **produced** (source and records); **verified** only for `tsc` and the frontend build. Open:

- The owner merges the PR; then Claude builds and tests on the PC in `E:\Project\Notaya\Notaya Tests Excluded` (the Bitdefender exclusion for the new path is the owner's to set).
- The new files have different hashes and a different commit, so the notes (commit, SHA-256, sizes) are rewritten. Release `v1.3.0` and its tag are deleted and recreated only on the owner's yes in the thread; older releases are not touched. Anyone who already downloaded the first V1.3.0 setup (one download counted at 17:00 UTC) has a different file with the same version number.
