# EX-2026-10-23-16: Start over, clearer unlock errors, keyfile warning (A20)

Executor: Claude (Claude Code), in a cloud session working from GitHub, plus a Remote Control session on the owner's PC for the real-exe test. Short (4-section) form.

## 1. Task Overview

A friend of the owner moved the whole portable folder to another drive. After that, unlocking said the password or file was wrong, and there was no way to start again. The owner passed this on at 17:04:30 UTC and approved a real move test plus a fix at 17:16:33 UTC, then "Push it run tests recheck" at 17:26:10 UTC (session transcript). Scope: [A20](../plan/A20-start-over-and-unlock-wording.md).

## 2. Work Performed

- Code review (commit `f7912fb`): the vault path is relative to the exe (`portable.rs`), the keyfile is used only by its SHA-256 (`crypto::keyfile_digest`), the KDF and header have no path, drive or machine input, and KeePassXC's preset config has `RememberLastKeyFiles=false`. Nothing explains a failure caused by the move itself.
- Real-exe move test on the owner's PC, throwaway data only, 17:21:30–17:22:57 UTC (system clock). The shipped exe (`1B8949CE…E7D6`) was copied without `data\` to `E:\clf-move-test-20261023-172131\`, `CIPHERLEAF_HOME` unset. A vault was made with a random passphrase and a 64-byte keyfile, then the folder and the keyfile were moved to `C:\Users\moham\AppData\Local\Temp\clf-move-test-20261023-172131\` (hashes unchanged). Results: the right passphrase and keyfile unlocked it with the test page intact. A keyfile path at the old place gave "The keyfile could not be read." No keyfile, a wrong keyfile and a wrong passphrase all gave "Wrong password or keyfile." Commands were called through the debug port, so the on-screen text wasn't checked. The port was closed afterwards (nothing listening on 9335) and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` was not set. The test folders were left in place. Not run: a move while the app is running.
- Independent review: one read-only review agent on `edb8512` (2026-10-23, about 18:30 UTC) found no data-loss path and no way to call `start_over` while unlocked. Its medium finding (the lock screen said "made without a keyfile" when the header could not be read) and several UI points were fixed in `94d0235`; the rest is documented in P19. Agent output is model output.
- Change (A20): `restore::set_aside` and the `start_over` command (`src-tauri/src/restore.rs`, `src-tauri/src/commands/backup.rs`, `lib.rs`); new `Error::BadCredentials` text (`error.rs`); the lock screen's "Start over…" dialog, wrong-password hint and no-keyfile note (`src/features/unlock/UnlockScreen.tsx`); `keyfileWarning` and the warning under the keyfile chip (`src/features/unlock/fields.tsx`); `api.startOver` (`src/lib/ipc.ts`); the mock (`dev-mock/core.ts`); P19 and a limitation in `docs/security-design.md`.

## 3. Verification / Testing

| Check | Result | Source it ran on |
|---|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Passed (cloud container, Linux) | The A20 working tree before commit |
| `npm run build` | Passed (cloud container) | Same |
| Mock UI (Playwright): wrong password, Start over dialog, after Start over | Seen as expected (screenshots in the session scratchpad, not committed) | Same |
| `cargo test --lib` (3 new tests: set aside with journal and a taken name, no vault, restore of a set-aside vault) | **Not run in the cloud** (Tauri needs GTK libraries there). To be run on the owner's PC; results go in section 4 or a dated correction. | — |
| Real exe with the A20 code | **Not run yet** | — |

## 4. Documentation, Git and Next Step

Records: [A20](../plan/A20-start-over-and-unlock-wording.md), the amendments table, P19 in security-design.md, the execution index, STATUS.md and sessions.md. Git: committed and pushed to branch `claude/project-thread-p5yntz` only. `main` and `feat/a9-keepassxc` were not changed: the push-both rule covers those two branches, and merging A20 into them waits for the owner. Next: run `cargo test --lib`, the type check and the build on the PC, and check the Start over screen and the warnings in a release exe on throwaway data. Then the owner reviews A20.
