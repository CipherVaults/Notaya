# EX-2026-10-23-18: Credits for open source components, and the suggested passphrase removed (A22)

Executor: Claude (Claude Code), in the cloud thread session of the owner's project, with an independent completeness check of the notices (model output). Short (4-section) form.

## 1. Task

Two owner requests, both on branch `claude/project-thread-sv2tfm`:

- **Credits.** The owner asked (project thread, 2026-10-23 19:53:20 UTC) for credits and copyright for everything used from outside as open source, "implement it everywhere".
- **Remove the suggested passphrase.** The owner wrote at 2026-10-23 20:12:53 UTC (project thread timestamp): "Remove suggest a strong passphrase". Recorded as A22 in [approval-and-amendments.md](../plan/approval-and-amendments.md). The credits check had just found that the `eff-wordlist` crate behind the suggestion declares AGPL-3.0.

## 2. What changed

- **Credits (committed as `503dd8b` and `d77eb63`).** `THIRD-PARTY-NOTICES.md` at the repository root lists KeePassXC 2.8.0 (and the libraries inside its package), all Rust crates shipped in the Windows exe, all npm packages, the fonts, icons, highlight.js, SQLite and zxcvbn, with version, copyright holder, license and the license texts that ask to travel with the program. Settings, About now has an "Open source licenses…" dialog (`OpenSourceDialog.tsx`). The README has a "Credits and licenses" section. An independent completeness check compared the lists with the lockfiles and the installed packages; it found build-only crates, a missing `tao` copyright line and the EFF wording to fix, which `d77eb63` fixed. The license for Cipherleaf's own code is not chosen, and the notices file says so.
- **Passphrase removal (A22; committed as `d3171e5`, with a follow-up fix `1efad95` that corrected the `rand` copyright text in the notices and the licenses dialog text).** Removed: the "Suggest a strong passphrase" button, the `SuggestedPassphrase` box with its Copy button, the `generate_passphrase` command, the `eff-wordlist` crate and its dependencies, and the unused `random_below` function. Kept: the master-password strength meter. The person now types their own master password in the Turn on security dialog (at least 10 characters and "Strong"). Because the crate is gone, `THIRD-PARTY-NOTICES.md` no longer lists it as shipped (regenerated in `d3171e5`, 226 shipped crates and 76 build-only).
- **Records updated for A22:** the authority row and the A22 row (and a note on A5) in the amendments record; [security-design.md](../security-design.md) (the clipboard limit and threat 5 no longer apply, and the intro line); the A21 plan notes; [acceptance.md](../acceptance.md) (V1f, X15); [STATUS.md](../STATUS.md); [sessions.md](../sessions.md). Older execution and session entries were not rewritten.

## 3. Checks

| Check | Result | Source it ran on |
|---|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Passed (cloud container, Linux) | The A22 working tree before the commit |
| `npm run build` | Passed (cloud container) | Same |
| Tauri-free Rust core harness (73 tests, including the generator tests) | Passed (cloud container) | Same |
| `cargo build`, `cargo test --lib`, `cargo clippy` on the owner's PC (throwaway data; 2026-10-23 evening, exact system clock time not recorded) | Build 0 errors. `cargo test --lib` **127 passed, 0 failed** (130 at `5ef5b07`; 3 passphrase tests removed). Clippy 0 errors, 5 library warnings (the same set as before). | `d3171e5` |
| `tsc`, `npm run build`, `tauri build` (release) on the PC | Passed. Release exe SHA-256 `8BC1C63017888B22402F39101155FB7CEB0474B747314CD5A87436C5024388B6` | `d3171e5` |
| Real-exe scripts on the PC | `run-optionalsecurity.ps1` 52/52; `run-restoreflow.ps1` 13/13; `run-lockflow.ps1` page, sticky and close as designed; passwords testA 13/13, testB 41/41, testC c1 3/3, c2 10/10, c2q 10/10, testE 5/5; no flakes | `d3171e5` exe |
| Real-window checks on the PC | 11 of 11: Settings, About, "Open source licenses…" opens, the notices text loads (about 178,700 characters) and scrolls, Escape closes; the Turn on security dialog has no suggest button; typing a password and turning security on works; the old `generate_passphrase` command now errors | `d3171e5` exe |
| Earlier, same scripts, plus the protected-vault Settings flow | All passed: restart shows the lock screen; "Turn off security…" asks the password again; a wrong password is refused and `vault.clf` stays byte-identical; the right password turns security off with the notes intact; "Turn on…" works again; re-key about 140 ms on a one-page vault and 209 ms (off: 181 ms) on a 400-page 3.5 MB vault; the Start over check passed | `5ef5b07` |
| Re-run of the PC scripts after `1efad95` | **Done in part:** On `1efad95` (owner's PC, throwaway data, 2026-10-23 evening; exact clock time not recorded; the PC session's own report, model output): `cargo test --lib` 127 of 127, clippy 0 errors and the same 5 library warnings, `tsc`, `npm run build` and the release build ok (exe SHA-256 `3FC9AD340788B84DF61FFF1990A9B5E02492EA873DEB4AB1D5532D6250C7C341`), `run-optionalsecurity.ps1` 52/52, `run-restoreflow.ps1` 13/13, and the Open source licenses dialog opens and shows the corrected text. The lock-flow and passwords scripts were not re-run on `1efad95`. | pass (limited) |
| `testD1`; USB removal, idle lock, Win+L and sleep on a real PC; the stable portable build; `cargo audit` | **Not run** | n/a |

A pass applies only to the exact source it ran on. The results above are the PC session's own report (model output) and were not independently re-run.

## 4. Open items

- `THIRD-PARTY-NOTICES.md` was regenerated for the new lockfile (226 shipped crates, 76 build-only); it and the in-app dialog no longer mention `eff-wordlist` or the EFF wordlist.
- Notices gaps still open: the WebView2 loader licence text, the source offer for the libraries bundled inside KeePassXC (including Qt under LGPL), and the `Licenses` folder contents, which must be produced when the portable folder is built.
- The license for Cipherleaf itself is still not chosen (STATUS question 11).
- Acceptance: pending. The owner has not reviewed A22 or the credits screen.
