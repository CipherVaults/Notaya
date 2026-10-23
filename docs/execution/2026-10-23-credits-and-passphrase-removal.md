# EX-2026-10-23-18: Credits for open source components, and the suggested passphrase removed (A22)

Executor: Claude (Claude Code), in the cloud thread session of the owner's project, with an independent completeness check of the notices (model output). Short (4-section) form.

## 1. Task

Two owner requests, both on branch `claude/project-thread-sv2tfm`:

- **Credits.** The owner asked (message time not recorded in this record) for credits and copyright for everything used from outside as open source, "implement it everywhere".
- **Remove the suggested passphrase.** The owner wrote at 2026-10-23 20:12:53 UTC (project thread timestamp): "Remove suggest a strong passphrase". Recorded as A22 in [approval-and-amendments.md](../plan/approval-and-amendments.md). The credits check had just found that the `eff-wordlist` crate behind the suggestion declares AGPL-3.0.

## 2. What changed

- **Credits (committed as `503dd8b` and `d77eb63`).** `THIRD-PARTY-NOTICES.md` at the repository root lists KeePassXC 2.8.0 (and the libraries inside its package), all Rust crates shipped in the Windows exe, all npm packages, the fonts, icons, highlight.js, SQLite and zxcvbn, with version, copyright holder, license and the license texts that ask to travel with the program. Settings, About now has an "Open source licenses…" dialog (`OpenSourceDialog.tsx`). The README has a "Credits and licenses" section. An independent completeness check compared the lists with the lockfiles and the installed packages; it found build-only crates, a missing `tao` copyright line and the EFF wording to fix, which `d77eb63` fixed. The license for Cipherleaf's own code is not chosen, and the notices file says so.
- **Passphrase removal (A22; working tree, not committed when this record was written).** Removed: the "Suggest a strong passphrase" button, the `SuggestedPassphrase` box with its Copy button, the `generate_passphrase` command, the `eff-wordlist` crate and its dependencies, and the unused `random_below` function. Kept: the master-password strength meter. The person now types their own master password in the Turn on security dialog (at least 10 characters and "Strong"). Because the crate is gone, the notices must not list it as shipped; the notices file was not touched by this records task.
- **Records updated for A22:** the authority row and the A22 row (and a note on A5) in the amendments record; [security-design.md](../security-design.md) (the clipboard limit and threat 5 no longer apply, and the intro line); the A21 plan notes; [acceptance.md](../acceptance.md) (V1f, X15); [STATUS.md](../STATUS.md); [sessions.md](../sessions.md). Older execution and session entries were not rewritten.

## 3. Checks

| Check | Result | Source it ran on |
|---|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Passed (cloud container, Linux) | The A22 working tree |
| `npm run build` | Passed (cloud container) | Same |
| Tauri-free Rust core harness (73 tests, including the generator tests) | Passed (cloud container) | Same |
| `cargo test --lib` and `cargo clippy --all-targets` on the app crate | **Not run** after this change (the cloud container cannot compile the Tauri crate) | n/a |
| Real-exe tests on the owner's PC (throwaway data) | **Not run** after this change | n/a |
| `cargo audit` | Not run | n/a |

A pass applies only to the exact source it ran on. Not yet seen: the Turn on security dialog in the real window without the suggestion.

## 4. Open items

- Run `cargo test --lib` and `cargo clippy` on the app crate and the real-exe optional-security script on the PC after A22; commit A22 only when the owner says.
- `THIRD-PARTY-NOTICES.md` was regenerated afterwards for the new lockfile (226 shipped crates, 76 build-only); it and the in-app dialog no longer mention `eff-wordlist` or the EFF wordlist. A separate agent re-checks it against the lockfiles.
- The license for Cipherleaf itself is still not chosen (STATUS question 11).
- Acceptance: pending. The owner has not reviewed A22 or the credits screen.
