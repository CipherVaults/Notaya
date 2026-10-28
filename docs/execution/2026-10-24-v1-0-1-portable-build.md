# Execution Summary: EX-2026-10-24-22 — V1.0.1 portable folder and ZIP

(4-section form for a small task.)

## 1. Task Overview

The owner merged [PR #9](https://github.com/AbyssWatcher2/CipherLeaf/pull/9) into `main` at 2026-10-24 10:44:26 UTC (merge commit `1e7afe59ed3b63c6b1e7c1fc950643d33d1b1c20`, GitHub time; the owner wrote "Merged" at 10:44:53 UTC) and had asked at 10:15:46 UTC to finish V1.0.1 that day. A PC remote session built the folder and the ZIP from exactly that commit and tested the exe. The owner then wrote (10:52:47 UTC) that he will test things before releasing, so **no release was made**. Times below are UTC, from the PC session's report (system clock).

## 2. What was done

- **Source:** a fresh detached worktree (`%TEMP%\clf-v101-release`) at the merge commit; `git rev-parse HEAD` printed that commit and the tree was clean. It differs from the PR head `5cded34` only by `docs/audit/2026-10-23-deep-audit.md` and `docs/audit/2026-10-23-live-checks.md` (PR #10), so the code is that of `0dbe11b`.
- **Build (11:07:16 to 11:12:28):** the PC session's script `build-release-v101.ps1` (kept in its scratchpad, not in the repo) checks a clean tree, version 1.0.1 in `package.json` and in the exe, and that the target folder does not exist. Then `npm run build` ok, `tsc` ok, `cargo test --lib` 167 passed 0 failed, `tauri` release build ok. Two earlier starts stopped at once, before anything was written, because the script treated native stderr output as a failure and then saw its own build folder as a dirty tree; both were fixed in the script.
- **Exe:** SHA-256 `135C9E06EA4E4E767F05403BA93778B138493DFFDCC8D8E13B4FEFCF53D28EAD`, 11,382,272 bytes, FileVersion 1.0.1. It differs from the PR-head build (`AE23AA01…2CCE`, same size) because the build folder differs, so the tests below were run on this exe.
- **Folder:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24\Cipherleaf`, 176 files, 166,641,014 bytes: `Cipherleaf.exe`; `KeePassXC\` (156 files, every file hash equal to the V1.0 copy); `Licenses\` with `THIRD-PARTY-NOTICES.md` (byte-identical to the repo file), `README.txt`, `WebView2-LICENSE.txt`, `KeePassXC-LIBRARIES-SOURCE.txt` (the last three from `packaging/Licenses/`) and `Licenses\KeePassXC\` (14 files, unchanged from V1.0); `README.txt` (V1.0.1 header, build commit and date, two new lines on hourly backups and the read-only message, the new exe hash). No `data\`, `.devdata`, `*.clf`, `*.key`, `.env` or CSV anywhere.
- **ZIP:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24\Cipherleaf-V1.0.1-portable.zip`, 87,223,923 bytes, SHA-256 `9AD3F86333969AFD74BAC871BA7752C857E623B3FABB1A0C1AF1A6257024B209`, 176 entries, one top-level folder `Cipherleaf/`, 0 backslash entry names. Extracted to Temp: 0 of 176 file hashes differ from the folder, 0 extra files. The ZIP is not byte-for-byte reproducible (file timestamps differ between runs; the dry run gave another hash).
- **Release notes:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24\RELEASE-NOTES.md` (3,757 bytes, UTF-8 without BOM, next to the folder, not inside the ZIP) with the commit and both hashes filled in.
- The V1.0 folder and ZIP (87,186,510 bytes, SHA-256 `CAB1E72E…D53D`) and `E:\Project\Cipherleaf-portable` were not touched.

## 3. Checks on the built exe (inside an extracted copy of the ZIP, throwaway data; the real folder was never started)

- `cargo test --lib` 167 passed; `optionalsecurity` 52/52; `restoreflow` 13/13 (recounted alone: 13 pass, 0 fail; the script's first count said 12 because of a pattern fault in the script); passwords `testE` 5/5 with that folder's own KeePassXC.
- Window check 4/4: opens straight to the notes (version 1.0.1); Settings, About, "Open source licenses…" loads (190,810 characters, includes the WebView2 text); `Licenses\WebView2-LICENSE.txt` is 1,513 bytes.
- Port 9335 was closed after every run, no environment variable was left set, and the throwaway folders were removed.
- **Plain first start (11:36 UTC, after the owner's report that a first start asked for a master password; he then said he had used another version):** a fresh extraction of the ZIP into a new Temp folder, the exe started in place with no `CIPHERLEAF_HOME` and no `CIPHERLEAF_KEEPASSXC` set. No data folder existed beforehand. The first screen was the notes (no password field), with "Create your first notebook" and the one-time note that the notes open without a password. `vault_status`: exists true, unlocked true, security off, no earlier copies, no open error, version 1.0.1; Settings, Security: "Security is off"; screenshot protection off; Passwords off. It created `data\vault.clf` (16,384 bytes), `backups\`, `keepassxc\` and `webview\`, and a normal close made one `vault-<ms>.clf` backup (exit code 0). The V1.0 ZIP and a pre-made empty `data\` were not compared because the check was stopped for the owner's retest.
- Not run on this exe: the lock-flow and the other passwords scripts, the hourly and back-off checks (they passed on the `0dbe11b` exe, whose source is the same), real-PC USB removal, idle lock, Win+L and sleep. The owner tests the last four himself.

## 4. Open items

- **The owner tests V1.0.1 himself** from a fresh unzip (not from the release folder, which must stay free of `data\`): idle lock, Win+L, sleep and resume, USB removal, plus the new features. Nothing is accepted.
- **Not released.** When the owner is satisfied he runs, in his own terminal (the PC's permission layer blocks it for Claude):

  ```
  gh release create v1.0.1 "E:\Project\Cipherleaf-V1.0.1-2026-10-24\Cipherleaf-V1.0.1-portable.zip" --repo AbyssWatcher2/CipherLeaf --target 1e7afe59ed3b63c6b1e7c1fc950643d33d1b1c20 --title "CipherLeaf V1.0.1" --notes-file "E:\Project\Cipherleaf-V1.0.1-2026-10-24\RELEASE-NOTES.md" --latest
  ```

  Afterwards the release is checked on GitHub (tag, target, one asset, digest equal to the ZIP hash) and recorded here. If the owner finds a defect, the folder and ZIP are rebuilt from a new merged commit, and the hashes above no longer apply.
- The PC session's folders in `%TEMP%` (`clf-v101-release`, `clf-v101`, `clf-assemble-test*`, `clf-v101-extract-*`, and the older `clf-v10`, `clf-a20` and test folders) stay until the owner approves deleting them.

## Addendum 2026-10-24 11:5x UTC: this build is superseded (calculator fix)

While testing, the owner reported (11:40:11 UTC) that the Deg/Rad switch of the Calculator seemed to do nothing, and confirmed it on this ZIP. The engine was right (`sin(30)` is 0.5 in Deg and -0.988031624093 in Rad, `asin(0.5)` is 30 in Deg); the cause was the display: after `=` it holds only the answer, and with an empty display nothing visible changed (reproduced in the mock UI with a browser). Fix (front end only): the unit is shown as DEG or RAD next to Copy, and switching it after `=` brings the last expression back (when it has sin, cos or tan and not `ans`) with its answer in the new unit ([PR #12](https://github.com/AbyssWatcher2/CipherLeaf/pull/12), merged 11:46:38 UTC, `c44a293`). A read-only review agent then found three low issues, fixed in PR #13 (clicking the active unit changed the display; `+/-` after `=` left a stale result line; the label's screen-reader text). The checks are `dev-mock/calc-check.cjs` (engine, `dependsOnAngleUnit`) and the new `dev-mock/calc-ui-check.cjs` (a browser against the mock UI), mutation-checked.

The folder and ZIP described above are therefore **not** to be released. They stay in `E:\Project\Cipherleaf-V1.0.1-2026-10-24` unchanged. A rebuild from the final `main` commit goes into `E:\Project\Cipherleaf-V1.0.1-2026-10-24-b`; its commit, hashes, tests and release command are added here once it exists.

## Addendum 2026-10-24 12:0x UTC: the final build (replaces the one above)

Built on the PC at about 11:54 UTC from the merge commit of [PR #13](https://github.com/AbyssWatcher2/CipherLeaf/pull/13), `6fa9ee59040d93704c20a2392a24863069b7e78e` (merged by the owner at 11:49:38 UTC; its tree is identical to `91fc68d`, so it holds the calculator fix of PR #12 and the three review fixes). A clean detached worktree; `git diff --stat 1e7afe5..HEAD` shows only `src/features/sticky/Calculator.tsx`, `calc.ts`, `useCalc.ts`, `dev-mock/calc-check.cjs`, `dev-mock/calc-ui-check.cjs` and docs files.

- **Build:** `cargo test --lib` 167 passed 0 failed; `tsc` and `npm run build` ok; `tauri` release build ok. The build script stopped once (a fault in the script, which read the folder suffix as a date), the half-made target folder was deleted, and the build was run again from scratch.
- **Exe (shipped):** SHA-256 `A5BEE8A702C0740C78D7704EA02ADF3859AAB6A41B12E9860C140948A043C0E7`, 11,382,272 bytes. The first build of the same commit gave `D0F9E8F5375BB5FA5F6C377270C168DC57BF5850837F68CB24841D50477F9A6F` (same size), so the exe is not reproducible bit for bit; every test ran on the shipped one.
- **Folder:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24-b\Cipherleaf`, 176 files, 166,641,014 bytes, laid out as above (KeePassXC 156 files hash-identical to V1.0). No `data\`, `.devdata`, `*.clf`, `*.key`, `.env` or CSV; the exe was never started from it.
- **ZIP:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24-b\Cipherleaf-V1.0.1-portable.zip`, 87,224,165 bytes, SHA-256 `863500C69E6EE614F08E258986F3C285D8E520EFAB603DB53CF47EC769D0E321`, 176 entries, 0 backslash entry names, one top-level folder `Cipherleaf/`. Extracted to Temp: 0 of 176 files differ from the folder, 0 extra files.
- **Release notes:** `E:\Project\Cipherleaf-V1.0.1-2026-10-24-b\RELEASE-NOTES.md` (3,916 bytes, UTF-8 without BOM): the first notes plus a Calculator bullet, the new commit line and the new hashes.
- **Tests on the shipped exe in a fresh extraction of the ZIP (throwaway data; port 9335 closed after each):** optional security 52/52; restoreflow 13/13; passwords testE 5/5; licenses window 4/4; and a real-window calculator check (the CDP port of `dev-mock/calc-ui-check.cjs`) 21/21: the DEG/RAD label on screen and in the hidden text; `sin(30)` is 0.5 in Deg and -0.988031624093 in Rad; after `=` switching to Rad brings `sin(30)` back with the Rad answer and switching to Deg recalls it again; clicking the already active unit leaves the answer; `±` after `=` gives `-(0.5)` with the result line `= -0.5` and switching the unit keeps it; `asin(0.5)` is 30 in Deg and 0.523598775598 in Rad; `2+3=` then Deg leaves 5.
- The first folder `E:\Project\Cipherleaf-V1.0.1-2026-10-24` (ZIP SHA-256 `9AD3F863…B209`) is superseded and untouched; V1.0 and `E:\Project\Cipherleaf-portable` were not touched.
- **Not released when this build was made** (released at 12:09:45 UTC, see the last addendum). The owner retests, then runs, in his own terminal:

  ```
  gh release create v1.0.1 "E:\Project\Cipherleaf-V1.0.1-2026-10-24-b\Cipherleaf-V1.0.1-portable.zip" --repo AbyssWatcher2/CipherLeaf --target 6fa9ee59040d93704c20a2392a24863069b7e78e --title "CipherLeaf V1.0.1" --notes-file "E:\Project\Cipherleaf-V1.0.1-2026-10-24-b\RELEASE-NOTES.md" --latest
  ```

  Afterwards the release is checked on GitHub (tag, target, one asset, digest equal to the ZIP hash) and recorded here.

## Addendum: the release (2026-10-24 12:09:45 UTC)

The owner wrote at 12:02:30 UTC that all his tests were good and that he would run the command, and at 12:10:45 UTC "Check out". He ran the `gh release create` command above in his own terminal. Checked through the GitHub API on 2026-10-24 at about 12:11 UTC (system clock 12:11:27 UTC); the asset was not downloaded and re-hashed:

- [Release v1.0.1](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.0.1), "CipherLeaf V1.0.1", published 12:09:45 UTC, not a draft, not a prerelease.
- Tag `v1.0.1` points at `6fa9ee59040d93704c20a2392a24863069b7e78e`, the commit the final build was made from.
- One asset, `Cipherleaf-V1.0.1-portable.zip`, 87,224,165 bytes (equal to the built ZIP), GitHub-computed digest `sha256:863500c69e6ee614f08e258986f3c285d8e520efab603db53cf47ec769d0e321` (equal to the built ZIP's SHA-256). The asset upload began at 12:02:52 UTC; download count 0 at the check.
- The release notes are the contents of `RELEASE-NOTES.md` (the Calculator bullet, the commit line and the new hashes are in it).
- Release `v1.0` is untouched: tag on `0ed3f525c5a93dabfe9c1e73fe94d4ebe91840da`, one asset of 87,186,510 bytes with digest `cab1e72e79b9061da05558a9afb5cfbcb916b589a32fb318ae110e04d522d53d`.

Not checked: that the GitHub release is marked as the latest one; the downloaded asset. The owner has not accepted the release. His own USB removal, Win+L and sleep tests are his to report.

## Addendum: the owner's manual lock tests and corrected release notes (2026-10-24 12:16 UTC)

- **What the owner reported.** At 12:02:30 UTC he wrote "All my tests is good". At 12:16:22 UTC he wrote "Hey we did those", with a screenshot of the published release notes' first Known-limits bullet ("Not tested on a real PC for USB removal, Win+L or sleep."). At 12:16:47 UTC he answered "Yes" when asked whether he tested USB removal, Win+L, sleep and idle lock by hand and whether they all passed. Times are the message timestamps (relayed by the coordinator).
- **What is not recorded:** which V1.0.1 folder or ZIP he tested, the exact steps (the checklist in the project files asks for security on, a USB stick, a fresh unzip), the PC, and anything he saw. This is the owner's report of manual checks, not an automated test and not an acceptance.
- **Correction.** The first Known-limits bullet on the published release v1.0.1 says these checks were not done; that was written before he reported them and is out of date. The other limits are unchanged, including the planned V1.1 item that an open vault (security off) does not lock when the drive is removed.
- **Corrected notes (applied 12:22:01 UTC, see below).** `RELEASE-NOTES-v1.0.1-corrected.md` (project files; 4,026 bytes, ASCII, SHA-256 `bf12d75189988ef025520305f212e873ab4e2791761205891c0b1355d6444a7a`) differs from the published notes only in that bullet (it now says the owner tested USB removal, Win+L, sleep and idle lock by hand on a real PC and they all passed) and in line endings (the published text has a few carriage returns; the new file has plain line feeds). The owner applies it with `gh release edit v1.0.1` in his own terminal (the PC's permission layer blocks `gh` writes for Claude). The ZIP, the tag and the asset are not touched, so the hashes above stay valid. The release text is checked on GitHub afterwards.
- **Applied.** The owner ran `gh release edit` (his message "Done" at 12:22:07 UTC). GitHub shows the release `updated_at` 12:22:01 UTC and a body that matches the corrected file (read through the API at about 12:23 UTC; compared by reading, not by hash): the first Known-limits bullet now says he tested USB removal, Win+L, sleep and idle lock by hand and they all passed, the other bullets are unchanged, and the carriage returns are gone. The tag is still on `6fa9ee5`, and the one asset is unchanged (same id, 87,224,165 bytes, digest `863500c6…e321`, created 12:02:52 UTC, updated 12:09:44 UTC). `published_at` is still 12:09:45 UTC.
