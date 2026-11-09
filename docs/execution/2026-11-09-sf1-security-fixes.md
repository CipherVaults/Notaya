# S-FIX1: bounded security fixes for S1-M1 and S1-M2 (EX-2026-11-09-67)

Partner side: Ahmed Emara (`AE0101`). Machine: Gaming-PC, Codex sandbox.
Baseline: `a649a3cd70f4eb2d4e2d1b8fdeff0da2c7a9a65c`, branch `AE0101/v1-6-sf1`.
Authority: Ahmed's current S-FIX1 message in this session; its timestamp was not supplied. First system-clock observation: 2026-11-09 08:12:01 UTC.
Scope: S1-M1 and S1-M2 only, in the exact code/test/docs write area in that message. Return: local diff, regression/check evidence, and this hand-back. Stop: hand back to Claude for independent review, commits and pushes. No integration or acceptance is claimed.
Result state: produced, with the scoped regressions verified and the full-suite failure explicitly retained below; not integrated or accepted. Codex issued no git mutation, network, install, new dependency, browser or Notaya app launch. Claude's heartbeat independently created automatic checkpoints `0e234bd3dfe1e56adf47dbe4ef8007d3da8501cb` at 2026-11-09 08:21:55 UTC and `734d914c58b076d1c116a4476ecc301251925106` at 08:28:05 UTC (records only). Verification resumed on Gaming-PC at 12:42:00 UTC (system clock), with a clean worktree at `734d914`. All seven code/test files still exactly match `0e234bd` (`git diff --exit-code 0e234bd -- <code files>` exit 0); this continuation needed only record/claim edits. The original lane baseline and authority above remain unchanged. Fetch/open-PR checks were not run because this brief prohibits network access.

## 1. What changed and why

### S1-M1: every row-file destination is validated before writing

Old: a stored name without an extension bypassed `checked_destination`. An extensionless attachment could overwrite an existing data-folder file chosen in the picker. The separate vault-extension check still refused `.nota`/`.clf`, with `Choose another file name.`.

New: `checked_destination_path` is the existing validation body extracted from `checked_destination` and exposed only inside the crate. The named-extension path still performs exactly its existing extension handling, then calls that validator. An extensionless row attachment calls the same validator without adding any extension. The picker result still comes from Rust; membership/unlock checks are unchanged. `save_row_file` makes the production validation/write tail testable without a native picker.

The validator resolves existing files or their parent folder, refuses the data folder (including a resolved junction into it), vault extensions and directory destinations before the write. Data/vault refusals use the named-extension path's existing text verbatim: `That is the folder where Notaya keeps your notes and backups. Choose another folder for the export.` No new dialog or UI wording was introduced. The extensionless vault refusal now uses that existing shared text, as requested.

An extensionless output keeps the exact chosen name. Named-extension behaviour is preserved: saving `attachment.json` to a pick named `external.nota` creates `external.nota.json`, leaving `external.nota` unchanged; the rule checks the final destination. A named `.nota` or `.clf` attachment targeting that vault extension is refused.

### S1-M2: renderer messages cannot reach the plaintext log

Old: a free-text `message` passed through lexical redaction and could retain ordinary unquoted note text. Source cleaning also repaired input by extracting/filtering characters.

New: the adapter deserializes the existing `kind` field into `FrontendKind` (`error`, `unhandledrejection`, `react`, `unknown`). Unknown strings map to `unknown`. There is no message member in the adapter or logging report. Serde ignores legacy/hostile extra `message` fields; the current frontend does not send them at all. The lexical renderer-message functions and their obsolete tests were removed, with the hostile-input sink test retained and adapted.

Rust formats only `UI error (<class>) in <source>[:<line>].` Source must be a script/page basename of at most 60 ASCII bytes, every character in `[A-Za-z0-9._-]`, with the existing script/page suffix list (`js`, `mjs`, `cjs`, `ts`, `tsx`, `jsx`, `html`). Invalid or overlong input becomes `unknown` as a whole; it is never repaired into a partial note fragment. The frontend extracts the browser-provided basename before invoking the command. Source is diagnostic metadata; arbitrary messages are never used for diagnostics.

The adapter retains the old numeric handling: finite nonnegative lines only, truncation of fractional parts and a `u32` clamp; the final log line still caps them at 9,999,999. The command has no vault-state parameter or unlock gate and remains usable while locked. Plaintext storage, 256 KiB rotation, 20/minute and 200/run backend renderer limiter, and nonthrowing frontend reporting remain. Duplicate frontend reports now use class/location, without retaining error text as their key.

No screen, dialog, button, displayed wording or layout was changed. Only the two requested security refusals/diagnostic behaviours changed. S1-H1, S1-H2 and S1-Q1 were not touched.

## 2. Files changed

- `src-tauri/src/commands/database_io.rs`: save helper and five destination regressions.
- `src-tauri/src/export/mod.rs`: extract/expose the shared final-path guard; extension handling and existing error literals preserved.
- `src-tauri/src/commands/errorlog.rs`: closed class, no message member, shared adapter conversion.
- `src-tauri/src/logging/mod.rs`: typed report, fixed diagnostic line, three new sink/adapter regressions; adapt existing hostile-input test.
- `src-tauri/src/logging/sanitize.rs`: closed class and strict basename validation; replace obsolete lexical-message tests with bounded-source/class tests.
- `src/lib/errorlog.ts`: the only `log_frontend_error` invoker; window error -> `error` with basename/line, promise rejection -> `unhandledrejection`; no error message/reason is sent.
- `src/components/ErrorBoundary.tsx`: React handler -> `react`; no error message is sent; rendered JSX untouched.
- `docs/handoff/claims.md`: own lane claim/status only.
- This execution note.

## 3. Regression tests and exact results

Final Rust result, run by Codex on Gaming-PC on the code in `734d914` (identical to `0e234bd`): **720 passed, 1 failed, 5 ignored, 0 measured, 0 filtered out**, exit **101**, 162.55 s. The only failure is `shell::tests::the_registry_value_can_be_written_read_and_removed`: `Windows refused to open the start-up list (code 5).` This is the expected sandbox registry denial, not a passing full suite. All eight added tests and both updated sanitizer regressions below passed. Six compiler warnings remain (unused variable `id`, unused `claim_core`, `release_core`, `title_prop`, `CALCS`, `with_deadline`); no warning cleanup was made.

Five destination tests in `commands::database_io::save_destination_tests`:

- `every_attachment_extension_refuses_the_data_folder_before_writing`.
- `every_attachment_extension_refuses_a_link_into_data_before_writing`.
- `every_attachment_extension_saves_to_an_external_folder`.
- `every_attachment_extension_refuses_a_vault_destination_before_writing`.
- `named_extension_rules_and_existing_longer_files_are_unchanged`.

These cover empty/named extensions, existing byte markers, absent forbidden targets, external overwrites, uppercase/lowercase vault extensions, extension preservation and refusal to replace an existing extended filename. Focused final destination run: **5 passed, 0 failed, 0 ignored, 721 filtered out**, exit 0, 0.04 s. Junction creation failed with exit 1 and `The file or directory is corrupted and unreadable.` The test therefore exercised the resolved data path, not a live junction; actual junction containment remains unverified here. Evidence: `D:\Notaya-lanes\out\sf1-destinations-final.txt`. If OS link creation is unavailable, the test explicitly reports the limit and exercises the resolved data path instead.

The 12:43 UTC full-suite verification passed all five destination tests again. A focused junction rerun with `-- --nocapture` also finished **1 passed, 0 failed, 0 ignored, 725 filtered out**, exit 0, 0.04 s, and confirmed the same junction-creation failure and resolved-path fallback. Evidence: `D:\Notaya-lanes\out\sf1-link-verify-20261109T1247Z.txt` (attempt label; actual full-suite times are below).

Three new tests in `logging::tests`:

- `renderer_messages_and_invalid_sources_never_reach_log_bytes`: generated random marker as a note title/body, plain text, quotes, Windows/Unix paths, token-like strings and a URL query, in message/source/class fields; deserialize the IPC shape, run production adapter/formatter and write real log bytes; no marker remains.
- `renderer_classes_produce_fixed_diagnostics_without_a_message_field`: each known/unknown class writes the exact fixed line with a valid source/line, without a message field.
- `renderer_source_and_line_bounds_hold_through_the_adapter`: 60/61/100,000-byte sources; missing source; finite, fractional, negative, nonfinite, missing and huge lines; nonnumeric JSON line rejection.

Updated `logging::sanitize::tests::sources_accept_only_bounded_script_basenames` and `kinds_deserialize_to_a_closed_class` cover invalid characters, path/query/prose/control/Unicode rejection and unknown/huge classes. Existing hostile-input, panic, rotation and limiter tests remain applicable.

All three named renderer tests and both named sanitizer regressions passed in the final full-suite run. Existing logging/store tests passed too; rotation and limiter implementation was not changed.

The first full Rust attempt exposed a test-only mistake: checking that `<chosen>.` did not exist on Windows, which aliases that name to `<chosen>`. The invalid assertion was removed; the production return-path equality and saved bytes already verify the intended unchanged filename. That attempt ended with 719 passed, 2 failed (the invalid assertion and expected registry denial), 5 ignored, exit 101, 182.97 s. Initial evidence is retained in `D:\Notaya-lanes\out\sf1-cargo-test-20261109T0817Z.txt`; it is not the final result.

## 4. Checks and how to verify

All checks are by Codex on Gaming-PC. Final verification ran on HEAD `734d914c58b076d1c116a4476ecc301251925106`, whose seven code/test files exactly match automatic checkpoint `0e234bd3dfe1e56adf47dbe4ef8007d3da8501cb`; records were then completed locally. Initial checks preceded those checkpoints on identical code/test bytes. Final source diff: `D:\Notaya-lanes\out\sf1-source-final.patch`, SHA-256 `4205bd1f717f3d499c1799957547c7b15962a08ec1040fdbf08f128f77cfeca1` (code/tests only, relative to the baseline; hash rechecked during this continuation).

| Check | Exact result |
|---|---|
| `cargo test --lib --offline` (from `src-tauri`) | Exit 101; 720 passed, 1 failed (expected registry code-5 denial above), 5 ignored; 162.55 s |
| `npx tsc --noEmit -p tsconfig.json` | Exit 0 |
| `npm run build -- --configLoader native` | Exit 0; Vite 8.3.5, 4,413 modules transformed, final rerun built in 15.37 s; warning `INEFFECTIVE_DYNAMIC_IMPORT` for `DatabaseView.tsx` imported both statically/dynamically |
| `git diff --check` | Exit 0; whole-lane `git diff a649a3c --check` also exit 0 |
| `npm audit` | Not run, expressly skipped |
| `cargo clippy`, `cargo audit` | Not run, outside the requested offline check set |

Offline npm mode was set, so `npx` could not fetch a missing tool. Existing installed dependencies were used. Rust has `CARGO_NET_OFFLINE=true` and `CARGO_TARGET_DIR=D:\Notaya-lanes\_target`. For Rust checks, `TEMP`, `TMP`, `NOTAYA_HOME` and `CIPHERLEAF_HOME` were all process-locally set to `D:\Notaya Tests\v1-6-SF1`. Generated throwaway data and failed-attempt evidence are preserved; no real notes/vaults were opened.

Final full-suite command started at **2026-11-09 12:43:13.084 UTC** and returned at **12:46:05.447 UTC** (system-clock receipts). Its output is preserved at `D:\Notaya-lanes\out\sf1-cargo-verify-20261109T1247Z.txt`; the `1247Z` in these filenames is an attempt label, not a measured start time. Frontend evidence: `sf1-tsc-verify-20261109T1247Z.txt` (empty output, exit 0) and `sf1-build-verify-20261109T1247Z.txt`, in that same output folder. No source changed after these checks.

To recheck: use those environment values, run the exact four requested commands on Claude's eventual commit, and optionally filter Rust to `save_destination_tests` / `renderer_` / `logging::sanitize::tests`. Use `-- --nocapture` for the link test to see whether an actual junction was available. Retain the existing named-pipe ignored tests and registry failure as limits; do not disable OS protections to make them pass.

A final full-suite attempt on the checkpoint initially failed to link (`LNK1104`, exit 101): another lane was executing the shared `_target\debug\deps\notaya_lib-fdc0922c1a322b2f.exe` (observed start 08:22:28 UTC). No process was interrupted and no shared artifact was deleted. That compile-failure evidence is preserved in `D:\Notaya-lanes\out\sf1-cargo-test-final.txt`; the retry result is recorded separately.

The earlier `sf1-cargo-test-final-retry.txt` ends without a suite result; no completion is inferred from it. The complete 12:43 UTC verification above supplies the final result. Earlier evidence files were kept unchanged.

## 5. Verification limits and next action

No native picker, live IPC invocation, Notaya exe, browser, UI pixels, installer or human acceptance was run/claimed, per the brief. Logging tests cover the IPC payload deserializer, adapter, formatter and production file sink; they do not start a window. Locked availability is preserved structurally by the command continuing to require no `AppState`/session; live locked IPC was not run.

Independent review and Claude's Security Guidance `/security-review` have not been run by this writing session. Claude owns that review, the full session/ledger/authority records and every commit/push. Its next action is to inspect this local diff and evidence, run its independent security review and repeat required checks on the eventual exact commit. No PR, integration or acceptance is claimed.

Model/effort/token counts and quota before/after: not exposed by this tool session; not guessed. Claude/heartbeat should attach the actual readings to the ledger. No ledger or authority record was edited by this lane.

## 6. Security self-check (guide section 4; not independent)

- Secrets/keys: checked the changed IPC/logging path for raw-message or path leakage; no keys, tokens, real notes, vault fixtures or password exports added. Zeroization, secret cloning/comparison, cryptography and random generation changes: n/a. Test markers use existing OS-random `new_id`.
- Commands/IPC: no new command. Existing renderer reporting input is a closed class, strict bounded basename and numeric line; unknown strings are fixed `unknown`. Save destinations remain native-picker supplied and the unlock/membership/attachment-ID gates remain upstream. Security-setting/restore/delete confirmations, AI permissions/registry and window confirmation replay rules: n/a, untouched (S1-H1/H2/Q1 excluded).
- Files/paths/parsing: traced picker -> extension handling -> shared canonicalized final-path validation -> write; refusal-before-write covered with existing byte markers. Input class/source strings cannot forge a log line. Archive/frame/image/HTML/SQL/shell input changes: n/a; only a test creates a fixed-root throwaway junction, with no note-controlled process arguments.
- AI pipe/bridge: n/a, untouched.
- Storage/history/logs: renderer free-text member removed; fixed diagnostics plus validated metadata only; actual plaintext-byte regression covers generated note markers. Existing encrypted history/activity/cache/old-vault/restore logic: n/a, untouched. Rotation and limiter unchanged.
- Frontend: reviewed both reporting caller files; no message/reason leaves the renderer through this command, and rendered JSX is unchanged. CSP, HTML injection, storage, screenshot protection and auto-lock changes: n/a, untouched.
- Dependencies/build: no dependencies, versions, lockfiles, network code or tooling changed. Required offline checks and their limits above; npm/cargo audits not run. No secrets/real vault data in the diff or test fixtures.
- Findings in this self-check: the requested paths are addressed by the diff and regressions; no additional in-scope issue found. This is not independent security approval or a whole-app security conclusion.

## 7. Outside write area, mechanical

None. Build/test outputs are ignored or outside the repository, under the permitted target/output/throwaway roots. No source or shared authority record outside the write area was edited.

## 8. ADDITION FOR AHMED

None. No visible feature, restyle, dialog, button or screen wording was edited. The requested destination fix necessarily refuses the previously permitted extensionless data-folder writes, and extensionless vault refusals now use the existing named-extension guard text rather than `Choose another file name.` (as explicitly requested). Diagnostic log entries use the requested fixed Rust text. Those are the only observable changes.

## 9. Claude's review (cloud thread, Ahmed's side, 2026-11-09 13:40 UTC, system clock)

Reviewer: Claude (Sonnet 5.5), a different agent from the one that wrote the diff. Read in full: the whole diff against `main` (9 files, 423 insertions, 228 deletions), not only this note. Machine: cloud (no Windows, no PC run by me); the PC heartbeat ran `tsc`, `npm run build`, `cargo test --lib` and `npm audit` on the exact pushed commit `6462256` and reported all four as pass.

- **S1-M1 (extensionless attachment destination).** The shared validator now runs on every row-file save, named extension or not. Checked: the data-folder test uses the resolved path, so a link into the data folder is refused too; vault extensions are refused in either letter case; a directory is refused; the refusal happens before any write. Nothing is added to the person's chosen name. The test for a live junction fell back to the resolved path on the PC (junction creation failed), so a live junction stays unverified. Call: **fixed**.
- **S1-M2 (renderer message in the plaintext log).** The `message` member is gone from the command, the adapter and the log report; serde ignores a legacy `message` field. The class is a closed list with unknown strings mapped to `unknown`; the source must be a bounded ASCII script basename, otherwise the whole field becomes `unknown` (nothing is repaired). The only two callers of `reportError` and the one invoker of `log_frontend_error` were checked. Call: **fixed**. One note: a non-string `kind` fails to deserialize and the command returns an error the frontend ignores, which is acceptable.
- **No visible change** (no screen, wording or layout), as Codex states. The only observable changes are the two refusals and the fixed log line.
- **Not verified by anyone:** a live junction, the real exe, a locked-vault IPC call. S1-H1, S1-H2 and S1-Q1 are untouched and belong to S-fix2 and S-fix3.
- Result state: **verified** (heartbeat checks on the exact source, independent diff review); integrated when merged; not accepted.
