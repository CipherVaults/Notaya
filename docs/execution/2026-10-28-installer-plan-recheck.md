# EX-2026-10-28-32: Windows installer plan re-checked for Notaya

Short form (4 sections). Docs only. Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `main` `a4574be` (Notaya V1.2.0 released; the plan branch `claude/project-thread-xphhp4`, draft [PR #23](https://github.com/AbyssWatcher2/CipherLeaf/pull/23), had `main` merged in cleanly).
- **Owner's words:** "Now continue the setup part" (2026-10-28 11:19:57), "So we will do 2 things windows installer and portable version in the release" (11:21:06) and, after the decisions were listed, "All recommendations" (11:56:21; relayed by the coordinator).
- **Scope and write area:** `docs/plan/windows-installer-plan.md` and the records that own the changed facts. No code, no build, no tooling install.
- **Returns:** the updated plan, the decisions still waiting, and records. **Stop point:** the plan and the questions; building waits for the owner's go.

## 2. What was done

- Checked the plan against `main`: the app is Notaya (A28), V1.2.0 is released with one portable ZIP, `portable.rs` is unchanged (the app still always saves next to the exe), `tauri.conf.json` has `bundle.active: false` with the publisher set to Notaya, and the identifier `app.cipherleaf.vault` is kept.
- Updated the plan: Notaya name and file names, installed notes in `%LOCALAPPDATA%\Notaya\data`, a note that the identifier must stay, the version for the installer changed from V1.2.0 to V1.3.0 (V1.2.0 is already out), and a "what changed" section.
- Recorded decision 9 (both the installer and the portable ZIP in each release) and then all other decisions as the recommended answers: the plan's table, two authority rows in `approval-and-amendments.md`, STATUS question 19 and the next-action line, and the index row. Not an amendment: nothing is approved to build.

## 3. Checks

- Read the plan back after editing and searched it for leftover "Cipherleaf" (only the two intended history mentions remain). The `main` facts above were read from `origin/main` files.
- **Not run:** no code changed, so `tsc`, the build and Rust tests were not run. Nothing was run on Windows. NSIS output, WebView2 behavior, the SmartScreen warning and the Tauri uninstaller's delete-data box are from Tauri's documented behavior and have **not** been tested on this app.
- Result state: *produced*. Not verified by a build, not integrated, not accepted.

## 4. Open items

- Owner: a go to build, and his OK for the NSIS tooling download. Both were asked for in the thread; neither is given yet.
- The owner merges PR #23 when he wants it on `main`.
