# Execution Summary: EX-2026-10-22-10 — Fresh portable build; KeePassXC kept for private use

(Short form: four sections, for a small task.)

## 1. Task Overview

**Task requested** (the owner, 2026-10-22 23:14:05 UTC): "Fresh vault. Fresh portable build. KeePassXC keep it for now; it will be for me and close people and friends, so nothing will be released for public."

**Authority and scope:** that message, recorded in [approval-and-amendments.md](../plan/approval-and-amendments.md). It answers STATUS question 4 (the dev vault is not copied) and the GPL question (A9 Q5). Stop at a rebuilt exe and a portable folder with no data. Executor: Claude (Sonnet 5.5). Time: 23:14–23:25 UTC (clock and transcript).

## 2. Work Performed

1. Confirmed the source was clean: HEAD `6f64b72`, no changed paths, the code identical to commit `2105a57` (only documentation commits since), the dev app not running.
2. Deleted the old `dist` folder (git-ignored build output) and ran `npm run tauri build` again: the frontend was rebuilt and the app re-linked, 2 m 16 s.
3. Compared the new exe with the first build: **not byte-identical** (new SHA-256 `A10AAB350C35E0867FBA58EA37D10E89227545769D2453FC528AFA926555FEFD`; the first was `5D7661051537FD30ECFC2D8FCD4C7F24456C9FB5626E9B317895E4058A865AAD`). Rust builds embed build-specific data, so this is expected; it doesn't mean the source differs.
4. Copied the new exe into `E:\Project\Cipherleaf-portable\`, kept the verified `KeePassXC\` (hash equal to `72200BFE…D28D`), `Licenses\KeePassXC\` and the README; updated the README's hash, its source line (code commit `2105a57`; later commits are documentation only), and added a note that the Licenses folder and README must travel with any copy. There is **no `data\` folder**, so the first run creates a fresh vault. The owner's dev vault (`.devdata\data\`) was not touched or copied.
5. Smoke-tested the new exe from a temporary copy (the dev app was not running), then deleted the copy.

## 3. Verification / Testing

| Check | Outcome | Limits |
|---|---|---|
| Source identical to the first build's commit | yes: HEAD only adds documentation commits; the working tree was clean | by `git status` and the log |
| `KeePassXC.exe` hash in the portable folder | equal to the verified value | that file only |
| Smoke test of the new exe from a temporary copy: it started and showed "Cipherleaf"; created `data\backups`, `data\keepassxc`, `data\webview` beside itself; no app, WebView2 or KeePassXC folder appeared in `%APPDATA%` or `%LOCALAPPDATA%`; 0 non-listening TCP connections; no debug port; a polite close ended it with nothing left running | pass | one run, 12 seconds, first screen only; no vault created |
| The real portable folder has no `data\` folder | confirmed after the test | |

**Not run:** creating a vault, the Passwords view, locking, the USB-removal lock (V3c), a PC without the Visual C++ runtime, `cargo audit`, a security review, signing.

The earlier checks on this code (33 Rust tests, `tsc`, `npm run build`, `npm audit`, 66 calculator checks) were not re-run: no source changed. Any source change makes this exe historical.

## 4. Documentation, Git and Next Step

**Records updated:** STATUS (questions 4 and 9, the release row, mission state), the authority table, the acceptance ledger (V3a, X11, new hash), A9's Q5 decision, the security design's licensing note, the execution index. **Git:** nothing committed in this step (the owner didn't ask); the record changes are uncommitted.

**GPL decision recorded:** KeePassXC stays bundled; the build is for the owner, close people and friends, nothing public; the license files, source and README stay with each copy. If a public release is ever planned this must be revisited. This is not legal advice.

**Next step:** the owner runs `E:\Project\Cipherleaf-portable\Cipherleaf.exe`, creates the fresh vault, and says what to change. Accepted state: none.
