# Execution Summary: EX-2026-10-23-19 — V1.0 stable portable folder and ZIP

(4-section form for a small task.)

## 1. Task Overview

The owner merged [PR #3](https://github.com/AbyssWatcher2/CipherLeaf/pull/3) into `main` at 20:45:05 UTC (merge commit `0ed3f525c5a93dabfe9c1e73fe94d4ebe91840da`, GitHub merge time). The owner had asked (19:13:17 UTC) for a rebuilt stable folder and ZIP, and (19:31:52 UTC) for a private GitHub release "CipherLeaf V1.0" with the ZIP. The checks and the build ran on the owner's PC in a PC remote session (the session's own report, model output; exact clock times not recorded).

## 2. What was done

- Built from a fresh worktree of `origin/main` at `0ed3f52` (confirmed with `git rev-parse`): `cargo test --lib` 127 passed 0 failed; `tsc --noEmit` exit 0; `npm run build` ok; `tauri build` (release) ok.
- Release exe SHA-256 `C43E84220E560BB7A7F6E9C4273794F704B448D1CA8ECC48BC1E3CB72FA8EA11` (11,312,640 bytes).
- New folder `E:\Project\Cipherleaf-V1.0-2026-10-23\Cipherleaf` (not over `Cipherleaf-portable`, which was not touched): `Cipherleaf.exe`, `KeePassXC\` (pristine official 2.8.0 portable, 156 files, from the verified download, hash and manifest match), `Licenses\`, `README.txt` (rewritten for V1.0). **No `data\`**, `.devdata`, `*.clf`, `*.key`, `.env` or CSV inside the folder or the ZIP.
- `Licenses\`: `THIRD-PARTY-NOTICES.md` (byte-identical to the repo root file), `README.txt`, and `KeePassXC\` with `COPYING`, `LICENSE.GPL-2`, `LICENSE.GPL-3`, `LICENSE.LGPL-2.1`, `LICENSE.LGPL-3`, `LICENSE.BSD`, `LICENSE.CC0`, `LICENSE.MIT`, `LICENSE.OFL`, `LICENSE.NOKIA-LGPL-EXCEPTION` (from the KeePassXC 2.8.0 files in the project's vendor folder), the source archive `keepassxc-2.8.0-src.tar.xz` (SHA-256 `4ED4B77D375F021FC04648C257FEDFB41DC13C6BD5C4965D8DED31284C9C653C`, matches the upstream digest) with `.DIGEST` and `.sig`, and `SOURCE.txt` (source link https://github.com/keepassxreboot/keepassxc, tag 2.8.0). No licence for Cipherleaf itself was added.
- ZIP `E:\Project\Cipherleaf-V1.0-2026-10-23\Cipherleaf-V1.0-portable.zip`: SHA-256 `CAB1E72E79B9061DA05558A9AFB5CFBCB916B589A32FB318AE110E04D522D53D`, 87,186,510 bytes, 174 files under one top-level folder `Cipherleaf/`. A first attempt had backslash paths and was rebuilt.
- Draft release notes: `E:\Project\Cipherleaf-V1.0-2026-10-23\RELEASE-NOTES.md` (outside the repo).

## 3. Checks on the built exe (throwaway data, a copy of the exe; the release folder stayed free of `data\`)

`run-optionalsecurity.ps1` 52/52; `run-restoreflow.ps1` 13/13; `run-lockflow.ps1` page, sticky and close as designed; passwords testE 5/5 and testC c1 3/3 with the KeePassXC copied from the release folder; real window 12/12 (opens straight to notes, a note survives a restart, Settings, About, "Open source licenses…" loads 178,632 characters, the Turn on security dialog has no suggest button, turning security on works). Port 9335 closed after each run. **Not run:** testA, testB, testC c2/c2q, testD1, USB removal, idle lock, Win+L and sleep on a real PC, `cargo audit`. Not accepted by the owner.

## 4. Open items

- **The GitHub release now exists:** the PC's permission layer had denied `gh release create`, so the owner created it from their own terminal (published 21:32:53 UTC per GitHub). [Release v1.0](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.0): tag `v1.0` points at `0ed3f525c5a93dabfe9c1e73fe94d4ebe91840da`, not a draft, not a prerelease, one asset `Cipherleaf-V1.0-portable.zip` (87,186,510 bytes) whose GitHub-computed SHA-256 digest is `cab1e72e79b9061da05558a9afb5cfbcb916b589a32fb318ae110e04d522d53d` (matches the built ZIP). Checked through the GitHub API on 2026-10-23 at about 21:33 UTC; the asset was not downloaded and re-hashed. Not accepted by the owner.
- The ZIP is built from `0ed3f52`; this records commit lands on `main` after it, so the release tag points at `0ed3f52`, not at the records commit.
- Open gaps in the notices stay open: the WebView2 loader licence text and the source offer for libraries bundled inside KeePassXC (Qt under LGPL). CipherLeaf's own licence is not chosen.
- Version numbers: `Cargo.toml` and `package.json` still say 0.1.0 while `tauri.conf.json` says 1.0.0.
- Leftover PC worktrees `%TEMP%\clf-v10` and `%TEMP%\clf-a20` wait for the owner's approval to delete.
