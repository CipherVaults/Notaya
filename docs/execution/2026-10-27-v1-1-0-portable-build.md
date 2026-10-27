# Execution Summary: EX-2026-10-27-24 — V1.1.0 portable folder and ZIP

(4-section form for a small task.)

## 1. Task Overview

The owner answered "Yes please" in the project chat at 2026-10-27 08:21:52 UTC (message timestamp) to making a new release of `main` with the KeePassXC removal (PR #20, A24), the OneNote-style ribbon (PR #18), deleting several sticky notes at once (PR #17) and the pop-out calculators with a shared 200-line history (PR #19). The version is **1.1.0**, because the release removes a feature and adds several. A thread session on the owner's PC built and tested it in a throwaway worktree (`%TEMP%\clf-v110-release`, branch `claude/project-thread-lx90mu`); the project folder `E:\Project\cipherleaf` and every real vault were not used. Times are UTC from the PC's system clock unless marked.

## 2. What was done

- **Version bump**, commit `70d38a1c744dd7dd249a61ba648bce96462ee70b` on top of `main` `4600561`: `package.json`, `package-lock.json` (the root package only), `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and the `cipherleaf` entry in `src-tauri/Cargo.lock`, from 1.0.1 to 1.1.0. Nothing else.
- **Build (08:23:57 to 08:29:19):** `npm ci` ok; `npx tsc --noEmit -p tsconfig.json` ok; `npm run build` ok; `cargo test --lib` **128 passed, 0 failed**; `npm run tauri build` ok. The tree was clean before the commit except for the five version files.
- **Exe:** SHA-256 `FDF988CE87332014D93298EB3D208BB8B2E6A8365A6EC373AFFFFB34BA70B8E5`, 11,263,488 bytes, FileVersion 1.1.0.
- **Folder:** `E:\Project\Cipherleaf-V1.1.0-2026-10-27\Cipherleaf`, 5 files: `Cipherleaf.exe`; `README.txt` (V1.1.0 header, build commit, no KeePassXC or Visual C++ lines, a new "Upgrading from V1.0 or V1.0.1" part that says the old passwords files are left as they are, the exe hash); `Licenses\THIRD-PARTY-NOTICES.md` (copy of the repo file), `Licenses\README.txt` and `Licenses\WebView2-LICENSE.txt` (from `packaging/Licenses/`). No `KeePassXC\` folder any more. No `data\`, `.devdata`, `*.clf`, `*.key`, `*.kdbx`, `.env` or CSV.
- **ZIP:** `E:\Project\Cipherleaf-V1.1.0-2026-10-27\Cipherleaf-V1.1.0-portable.zip`, 5,916,923 bytes, SHA-256 `2E0B94B1046469B63F829C5C8C8D4C40474E32A600AB45C7C93B5686A498A890`, 5 entries under one top-level folder `Cipherleaf/`, 0 backslash entry names. Extracted to Temp: 0 of 5 files differ from the folder, no extra files. A first ZIP made with .NET Framework's `CreateFromDirectory` had backslash entry names; it was deleted before use and the ZIP was written again entry by entry with `/` names.
- **Release notes:** `E:\Project\Cipherleaf-V1.1.0-2026-10-27\RELEASE-NOTES.md` (next to the folder, not inside the ZIP; UTF-8 without BOM). They say KeePassXC is removed and that an old `passwords.kdbx`, `data\keepassxc\` and `passwords-*.kdbx` copies are left untouched and no longer backed up.
- The V1.0 and V1.0.1 folders, ZIPs and GitHub releases and `E:\Project\Cipherleaf-portable` were not touched.

## 3. Checks on the built exe (inside a fresh extraction of the ZIP, throwaway data; the release folder was never started)

Runners from `tools/real-exe-tests/` at `70d38a1`, each printing the tested exe's SHA-256 `FDF988CE…B8E5` (equal to the shipped one):

- **Restore flow** (08:30:29): 12 of 12 PASS, 0 FAIL (list while locked, picked-path rule, UNC refused, traversal refused, wrong password refused with the vault unchanged, restore while locked and while unlocked).
- **Lock flow** (08:31:05), scenarios page, sticky, close: the typed text was saved in each (`page contains "FLUSHTEST-page": true`, `sticky contains "FLUSHTEST-sticky": true`, `page contains "FLUSHTEST-close": true` after closing and restarting; the other lines false by design).
- **Optional security** (08:32:20 to 08:33:30): p1 8/8, p2 21/21, p3 11/11, p4 5/5, p5 2/2, "phases with failures: none" (47 checks; the five Passwords-switch checks were removed in A24).
- After every run: port 9335 listening 0, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` not set, no Cipherleaf process left, the runner's temp folder removed. No `data\` was created in the extraction folder.
- **Not run on this exe:** a look at the ribbon, sticky multi-delete and pop-out calculators in the real window (the calculators were checked in the real exe at `58c299c`, see EX-23); a plain first start with no `CIPHERLEAF_HOME`; upgrading a real V1.0.1 `data` folder that has a passwords file; idle lock, Win+L, sleep and USB removal; `npm audit` and `cargo audit`.

## 4. Open items

- **Not released.** The owner merges the records PR, retests from a fresh unzip if he wants, then runs in his own terminal (the PC's permission layer blocks `gh` writes for Claude):

  ```
  gh release create v1.1.0 "E:\Project\Cipherleaf-V1.1.0-2026-10-27\Cipherleaf-V1.1.0-portable.zip" --repo AbyssWatcher2/CipherLeaf --target 70d38a1c744dd7dd249a61ba648bce96462ee70b --title "CipherLeaf V1.1.0" --notes-file "E:\Project\Cipherleaf-V1.1.0-2026-10-27\RELEASE-NOTES.md" --latest
  ```

  The tag targets the exact commit the exe was built from; it is in `main`'s history once the PR is merged. Afterwards the release is checked on GitHub (tag, target, one asset, digest equal to the ZIP hash) and recorded here.
- Nothing is accepted.
- The throwaway folders `%TEMP%\clf-v110-release` (worktree), `%TEMP%\clf-v110-test-*` (extraction) and `%TEMP%\clf-v110-build.log` stay until the owner approves deleting them.
