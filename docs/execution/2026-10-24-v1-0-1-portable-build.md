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
- Not run on this exe: the lock-flow and the other passwords scripts, the hourly and back-off checks (they passed on the `0dbe11b` exe, whose source is the same), real-PC USB removal, idle lock, Win+L and sleep. The owner tests the last four himself.

## 4. Open items

- **The owner tests V1.0.1 himself** from a fresh unzip (not from the release folder, which must stay free of `data\`): idle lock, Win+L, sleep and resume, USB removal, plus the new features. Nothing is accepted.
- **Not released.** When the owner is satisfied he runs, in his own terminal (the PC's permission layer blocks it for Claude):

  ```
  gh release create v1.0.1 "E:\Project\Cipherleaf-V1.0.1-2026-10-24\Cipherleaf-V1.0.1-portable.zip" --repo AbyssWatcher2/CipherLeaf --target 1e7afe59ed3b63c6b1e7c1fc950643d33d1b1c20 --title "CipherLeaf V1.0.1" --notes-file "E:\Project\Cipherleaf-V1.0.1-2026-10-24\RELEASE-NOTES.md" --latest
  ```

  Afterwards the release is checked on GitHub (tag, target, one asset, digest equal to the ZIP hash) and recorded here. If the owner finds a defect, the folder and ZIP are rebuilt from a new merged commit, and the hashes above no longer apply.
- The PC session's folders in `%TEMP%` (`clf-v101-release`, `clf-v101`, `clf-assemble-test*`, `clf-v101-extract-*`, and the older `clf-v10`, `clf-a20` and test folders) stay until the owner approves deleting them.
