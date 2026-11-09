# S-FIX2: native confirmations and local keyfiles

**Side / machine:** Ahmed Emara (`AE0101`), Codex developer hand on `GAMING-PC`. **Authority:** A51, Ahmed 2026-11-09 08:22:33 UTC, and this lane's bounded instruction. **Branch:** `AE0101/v1-6-sf2`. **Result:** produced implementation with automated evidence; not integrated, independently reviewed or accepted.

**Source provenance:** the Area 1 findings reviewed `13185af2c1be96a539ec1ec5446e99e0e14def89`. This worktree started from `d910034` and already contained automatic SF2 checkpoints `e891873` and `5fbcf569ca2643bc2a5b4ee05470f86dba8fb8b2` at entry to this completion run. They were preserved and reviewed against current code. Results below apply to that last checkpoint **plus the final working-tree changes**, not the checkpoint alone. Product-source manifest: `D:\Notaya Tests\v1-6-SF2\final-source-manifest-20261109T175844Z.json`, 20 product files, aggregate SHA-256 `e7377a5eb639474fe6cb5043e9a86a87d36d58f81eea58e9e06ba8f27f2bf96a`. The manifest hashes sorted path/hash entries, as recorded in that file. Documentation-only changes follow the source freeze.

**Scope / stop point:** five specified approval routes, keyfile locality in five credential commands, minimal cancellation handling, security promises and person-only checks. No dependency, install, network, application/browser launch, actual native dialog, Git mutation, merge or release by this Codex run. The finding and proof sources on lane S were read only; none were copied into the build. Claude owns commits, pushes, records integration, independent review and the final security decision.

## 1. Per-route changes and exact prompts

All native dialogs use Windows **OK / Cancel**, Warning kind, with the main window as parent. The existing window dialogs and their wording remain first and unchanged. A script can skip the cosmetic window question, but cannot skip the binding Rust question.

| Route | Before | After / native prompt |
|---|---|---|
| `delete_page` | A window script could permanently remove any binned page/subtree without a Rust confirmation. | Rust snapshots the target/subtree, asks, rechecks, then runs the existing purge. Title: **Notaya — Delete for good?** |
| `empty_trash` | A window script could empty the bin directly. | Rust snapshots the complete bin set including descendants, asks, rechecks, then runs the existing purge. Title: **Notaya — Empty the bin?** |
| internal `delete_bin_tree` | Type/bin membership were the only local preconditions. | It now requires a Rust deletion permit; every removed id must be included and is consumed. No additional per-subtree dialog inside an approved batch. Its calling route supplies the corresponding delete/empty prompt. |
| `ai_set_enabled(true)` | A main-window script could pass `ALLOW`. | `ALLOW` is still required, followed by native approval bound to the session and AI settings. Title: **Notaya — Turn on AI access?** |
| `ai_pair_answer(allow=true)` | A main-window script could answer a pending pairing from its emitted id. | The id only requests a native question. The client/request/deadline and settings are rechecked before the broker can allow it. Title: **Notaya — Connect an AI?** |

Delete body, exact template: **“Permanently delete N item(s) and their history? This cannot be undone.”** The actual word is `item` for N=1, otherwise `items` (no literal parentheses). Rust appends `\n• TITLE` for at most five titles, sorted by item id, and if necessary `\n…and M more item(s).` with the same singular rule. Each label is at most 80 Unicode scalar values including `…`; control characters, line separators and bidi formatting are stripped. Whitespace-only labels become `Untitled`. Counts include descendants, not just the selected roots. The existing history/garbage-collection behavior is retained; this is not a physical-erasure promise.

AI enable body, exact: **“Turn on AI access? Paired AIs will be able to connect to your notes with the access you give them.”**

Pairing body, exact template: **“Allow “CLIENT” to connect to Notaya? It starts at Look and can read notebooks you have not hidden. Allow only a connection you started yourself.”** `CLIENT` comes from the pending request held by Rust, sanitized/bounded as above. It is a client-supplied label, not proof of an application's identity.

Native denial, overlap and stale approval return the plain IPC error string `cancelled`, with no operation/history/settings write by the refused call. `TrashView`, `PairingDialog` and `AiAccessPanel` handle that result quietly. Turning AI off retains its earlier behavior. Ordinary AI action confirmations and Strong words still use `ai_confirm_answer` in the window, as do other level/settings controls. Rare AI `bin.delete_forever` / `bin.empty` calls receive the extra native gate after their original action confirmation.

## 2. Approver design and script boundary

`NativeApprover: Send + Sync` exposes `ask(NativePrompt) -> bool`. `Approvals` is held in existing app AI runtime state and defaults to deny. Production startup binds the Windows implementation through `lib.rs`; test-only injection substitutes allow/deny/hooks. No constructor, injection command or answer cache is exposed over IPC.

The existing `tauri-plugin-dialog` is called from Rust, with `.parent(&main)`, Warning kind and `OkCancel`. Command adapters use `tauri::async_runtime::spawn_blocking`; AI dispatch is already on dedicated pipe workers. The prompt is assembled under a brief session read, then that lock is released before `ask`. Hooks that acquire the session lock during `ask` exercise this separation. Missing main window/handle fails closed. A try-lock permits only one native question at a time; overlapping requests cancel.

`Approval` is a private, non-Clone, non-serializable Rust value. It consumes its target even on a failed comparison. Binding includes a random memory-only session identity, operation, exact ids/set, every target revision, prompt labels and AI epoch. Enable/pairing also bind the AI settings hash and settings revision. Pairing additionally checks the broker's exact id, client and deadline under its mutex. Broker `answer("pairing", ..., true)` refuses even an ordinary Rust caller; only the checked native route answers affirmatively. A deletion approval creates a permit that consumes each authorized id, so the sink and retries cannot replay it. No new Tauri command or event exists.

## 3. Keyfile guard and exact refusal rules

Every reader uses `crypto::keyfile_digest` -> `keyfile_path::open`, including `unlock_vault`, `disable_security`, both current/new keyfiles of `change_master`, `enable_security`, and `restore_vault`. Credential/staging cores were extracted without the OS adapters so regression tests call the same production paths.

Before stat/open/read of the supplied path, the guard normalizes `/` to `\` and refuses:

- UNC (also mixed slashes), all caller-supplied `\\?\` / `\\.\` namespaces, NT `\??\`, `\Device\`, `\GLOBAL??\` forms, empty/control-bearing input, Windows invalid filename characters and colons except a drive letter's colon (thus alternate data streams).
- DOS device aliases in any component, case-insensitively, with extensions/trailing dots/spaces accounted for: `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `CONIN$`, `CONOUT$`, `COM1`–`COM9`, `LPT1`–`LPT9`, including superscript 1/2/3 variants. `COM10.key` is an ordinary name.
- Network drive mappings: `QueryDosDeviceW` resolves the drive and SUBST aliases before any path open; redirector/MUP/UNC mappings are refused. `GetDriveTypeW` refuses `DRIVE_REMOTE` (4), unknown (0), or unavailable/no-root (1). Fixed (3), removable (2), CD-ROM (5) and RAM disk (6) remain eligible. Alias depth is bounded at 16.
- Reparse destinations that name remote/device/unsupported volume namespaces; link expansion depth is bounded at 32. Only an internally read `\\?\D:\...` local drive-form reparse destination may lose its extended prefix before checking. Caller-supplied extended drive paths remain refused.

Local ancestors are opened with `FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS`, denying write/delete sharing while inspected. Link targets are inspected without traversing them, checked recursively, and local ancestor handles are retained until the final file is opened. Metadata and digest bytes are read from that same file handle, bounded to 64 MB even if its size changes. Ordinary folders, drive-relative/relative paths and mixed slash local names remain eligible. Unknown or unreadable paths fail closed; the rule is not a network reachability probe.

Exact locality error: **“Choose a keyfile on this PC or a USB drive. Network and device paths cannot be used.”** Other existing read/size errors remain. No app error translation table was found in current `src`; existing IPC errors are English Rust strings passed through `errorText`, so no separate Arabic/other-language table was introduced.

## 4. Files changed (relative to d910034)

- New modules: `src-tauri/src/native_confirm.rs`, `src-tauri/src/keyfile_path.rs`.
- Commands/reader/registration: `src-tauri/src/commands/{notes,ai,vault,backup}.rs`, `src-tauri/src/crypto.rs`, `src-tauri/src/lib.rs`.
- AI integration and tests: `src-tauri/src/ai/{confirm,dispatch,mod,pairing,pipe,smoke}.rs`.
- Small necessary session/test plumbing: `src-tauri/src/vault/{mod,history}.rs`, `src-tauri/src/changes.rs` (details in section 10).
- Cancellation only: `src/components/Sidebar.tsx`, `src/features/ai/PairingDialog.tsx`, `src/features/settings/AiAccessPanel.tsx`.
- Records: `docs/security-design.md`, this execution note, `docs/plan/2026-11-09-whole-app-test-plan.md`.

No changes to dependencies, capabilities, CSP, logging, `commands/database_io.rs`, or `commands/errorlog.rs`. The whole-app plan was absent from baseline; this new file is explicitly an SF2 supplement. Claude must append its SF2-A01–A11 checks to lane W's full plan, preserving W's existing checks/results.

## 5. Tests added and results

Final results are filled from the final-source run below, not from the review's old proof harness. Seventeen new tests cover:

| Module | Added test coverage |
|---|---|
| `commands::notes::native_delete_tests` (4) | Deny delete/empty after trash preserves file hash, items, revisions, history, activity, dirty/hourly state and AI settings; allow runs old purge and survives reopen; title/descendant/bin/restored-target changes invalidate; internal tree permit rejects wrong ids and replay. |
| `commands::ai::native_ai_tests` (4) | Enable deny/allow, unchanged snapshots, no cached answer and old disable path; changed enable settings; pairing deny/allow through actual broker and persistence core, Look level and authentication; script/wrong-kind/replay refusal; changed settings/cancelled request. |
| `native_confirm::tests` (2) | Prompt label count/sanitizing/bounds; single-use approval, operation mismatch and same-vault reopen/session mismatch. |
| `ai::confirm::tests` (1) | Pairing expiry while `ask` waits; wrong request/client refusal. |
| `ai::dispatch::tests` (1) | AI empty-bin denial/changed target; existing permanent-delete test also extended to prove denial preserves the vault before allowing. |
| `commands::vault::keyfile_locality_tests` (1) | All five routes, both master-change keyfiles: 30 refused-path cases, zero guarded opens and identical vault/history/settings snapshot. |
| `keyfile_path::tests` (5) | UNC/extended/mixed/device/ADS aliases before I/O, fixed/USB vs remote resolver policy, synthetic reparse target names, actual local digest reads with dot/space/mixed/relative components, conditional local junction and cycle. |

The conditional junction test prints **NOT RUN** and returns before its junction assertions when Windows cannot create the fixture; that is not evidence that junction reading/cycle handling passed. Other test changes inject approvals into existing parity/history/bridge tests; they do not open a native dialog.

## 6. Checks and exact results

Final Rust run in progress when this draft was written; final result to be inserted before hand-back. Log: `D:\Notaya Tests\v1-6-SF2\cargo-final-20261109T175844Z.log`. Both test-home variables point to `D:\Notaya Tests\v1-6-SF2`; Cargo runs offline with cached dependencies and the shared `D:\Notaya-lanes\_target` build directory.

- `npx tsc --noEmit -p tsconfig.json`: exit **0**, no diagnostics.
- `npm run build -- --configLoader native`: exit **0**, Vite 8.3.5, 4,413 modules, build 2.22 s. Warning: `DatabaseView.tsx` has both dynamic and static imports, so that dynamic import cannot create a separate chunk. No app/browser was launched.
- `git diff --check`: exit **0**; no whitespace errors. Git emits its ordinary LF -> CRLF notices for the two new Rust files.
- `npm audit`: **skipped as instructed**. `cargo audit`, clippy, app/exe/browser checks, real pipe/native integration and independent Claude security review: **not run**.

Preserved preliminary evidence: the first suite (older source before final refinements) had **731 passed, 2 failed, 5 ignored**, 250.34 s: the known registry code-5 failure plus a junction-fixture creation failure. A focused retry attempted while that executable was still running hit linker `LNK1104` (cannot overwrite the running test exe); it did not run tests. A direct filtered unit-test invocation identified the junction failure at fixture creation, not at the keyfile read. The local file read before that point passed. Junction fixture failure is now reported conditionally and explicitly; production path restrictions were not weakened.

## 7. Person-only checks and exact steps

All **NOT RUN**, **KIND A**, listed with precise fixtures, order, wording and expected observations in [SF2-A01–A11](../plan/2026-11-09-whole-app-test-plan.md):

1. Bin a throwaway page with history; cancel the window question (no native prompt), retry/accept it then cancel Windows (no deletion/toast/history change), restore/reopen to confirm.
2. Bin a parent/child; accept both questions personally; see count 2, both titles, old purge behavior and a separate live page retained after reopen.
3. Empty three bin items; native Cancel retains them, retry/OK removes them and retains the live page.
4. Use >5 items and long/control/bidi test titles; see true count, at most five bounded sanitized labels and remainder count; cancel.
5. Complete ALLOW then cancel Windows (AI stays off); retry/OK enables it; turning off has no new prompt.
6. Start a local request named `SF2 test client`; window Allow then native Cancel gives no saved pairing; a new request with both approvals saves exactly one Look pairing; reconnect uses the existing credential.
7. A Full test pairing's delete-for-good and empty-bin requests get their original window/Strong question then native question; cancel or personally approve and inspect the corresponding result.
8. Everyday AI edits, move-to-bin, levels, hidden notebooks and big-change controls keep their window-only behavior.
9. Check native Escape/close, parenting/focus and background idle/Windows lock while a native question is open; a later OK cannot mutate the stale session.
10. Exercise five credential routes on generated local/USB keyfiles, including one ordinary local junction where supported; inspect unchanged notes.
11. Only in an authorized isolated environment with a pre-existing mapping: try mapped-drive/UNC/device keyfiles in the five routes; see the exact locality refusal and no vault change. Network setup/traffic tracing needs separate authority; none was done here.

A person records the integrated commit, matching binary hash, test home and actual outcome. No native dialog was shown or clicked by this run.

## 8. Unverified behavior and open risks

- Native rendering, actual main-window modality, keyboard/close results, shutdown/auto-lock interaction, human consent, and real USB/mapped-network OS behavior remain unverified. Unit tests establish core behavior only.
- The sandbox could not create directory junctions (`mklink` exit 1, “The file or directory is corrupted and unreadable.”); real junction reads/cycles are **not run**, while synthetic remote/device reparse-name refusals and ordinary local reads are tested. Unsupported volume-style reparse targets fail closed.
- The native prompt is a second gate for these rare routes; a compromised renderer can still read unlocked content, move items into the bin, answer everyday window confirmations and change its existing settings. Same-user malware/OS automation is outside this boundary. Client names are untrusted labels, merely displayed safely.
- Native denial does not undo unrelated writes made concurrently while the prompt is open. Changed-target tests compare against the intentional intervening write and prove the refused operation adds nothing.
- Existing purge/backup/history-retention limitations, including lane S Area 2 findings, are not repaired here; old backups may retain deleted material. No physical-erasure or whole-app-security claim.
- No traffic trace, third-party dialog implementation audit or every-reparse-provider claim. Drive namespace races by same-user malware and OS cloud/filter behavior are outside the stated boundary.
- Integration with S-fix1 and lane W, the exact committed checks, Claude independent diff/security review, final usage/session/claims records and partner acceptance remain to be done by their owners.

## 9. Security self-check (guide section 4; self-review, not independent)

| Checklist boundary | Check / finding / limit |
|---|---|
| Secrets stay in Rust; results/events/logs/errors | Checked new approval and keyfile paths: prompts carry only bounded titles/client labels, fixed text/counts; keyfile bytes/digests are not returned or logged. Locality errors contain no path. No new secret output found. |
| Secret lifetime, comparisons, hashed credentials | Keyfile digest bytes and content use existing Zeroizing buffers; bounded same-handle read. Pairing creation/hash/authentication unchanged. Token comparison/lock-zeroization changes **n/a**. |
| Audited crypto, nonces, record authentication | Crypto algorithms/header/AEAD/AAD unchanged: **n/a**. Only keyfile access guard/read bound changed in crypto.rs. |
| Random ids/tokens/nonces | Session binding identity uses existing OS-random helper; no timestamp/counter authentication primitive. Other random sources unchanged. |
| New commands, inputs and Rust authorization | No new IPC command. Existing five operations now pass the private native gate and exact recheck; count/labels originate in Rust, not renderer text. Existing main-label checks retained for AI controls. |
| AI registry, levels, hidden/locked targets | Dispatcher still prepares/rechecks through the existing registry and permissions. Permanent-delete calls now need approval even through shared cores; AI bin-empty retains only its permitted binned roots, preserving relation-destination checks. Other permissions unchanged; full matrix audit **not run**. |
| Confirmation script/replay/expiry binding | Native answers never cross IPC. Request/session/revisions/AI settings bound, answers consumed; pairing deadline/client/request and broker single-answer checks covered. Everyday window confirmations remain script-answerable under A51, explicitly documented; no broader guarantee asserted. |
| Paths, containment, junctions, ADS/devices | Guard traced from all five credential routes, before keyfile metadata/read. Lexical/drive policy and no-open tests passed where reported. Same-handle bounded read and pinned no-follow traversal inspected; actual junction integration **not run** here. Archive/export containment untouched: **n/a**. |
| Sizes before allocation/parsing | Keyfile length precheck plus `take(MAX+1)` stream bound and postcheck; prompt labels bounded before IPC/OS display. Other parsers/frame/image budgets **n/a**. |
| Image/HTML/link sanitation | Untouched: **n/a**. |
| SQL/shell/log text | No new SQL or shell/process execution in production. Test-only mklink uses fixed arguments and generated throwaway paths. No new arbitrary log text; logging lane untouched. |
| Pipe ACL, remote clients, frames/timeouts/concurrency | Transport/ACL/frame code untouched: **n/a**. Native gate serializes questions without holding session locks. Full pipe/OS tests **not run**. |
| Pairing lifetime, revoke, welcome, token logging | Pairing request deadline and single-use checks extended; persistence extraction retains enabled/paused/cancel/epoch/capacity checks. Token generation/storage and revocation semantics unchanged. Fake broker/core tests only; live pairing **not run**. |
| AI event/error privacy and real names | New dispatcher error is fixed `cancelled`; no new note text/event payload. Existing event boundaries otherwise untouched. |
| Encrypted history/activity integrity | Denial snapshots include history/activity/file hash; allow uses old purge and existing history tests. No schema/encryption/hash-chain change. Full historical-integrity review **n/a**. |
| Decrypted caches/lock/disk | Approvals are Rust memory values only, not persisted/serialized, bound to a random session identity and epoch. No new general cache or temp-note data. Lock/restore itself unchanged; real auto-lock with modal UI **not run**. |
| Old vaults and restore security | Staging extraction preserves source/security checks and old format behavior; keyfile rule applies before staging. Existing suite supplies old-format/restore regressions, not a new live restore claim. |
| Strict CSP and active content | Untouched: **n/a**. Frontend diff only quiet cancellation handling; no HTML/eval/network sink. |
| Browser secrets, screenshots, auto-lock | Untouched storage/capture/auto-lock behavior: **n/a**. Native worker lock separation inspected; Windows runtime interaction remains person-only. |
| Stable dependencies/audits/network | No new dependency/version/tool/network code. npm audit skipped by instruction; cargo audit/clippy **not run**. No advisory-clean claim. |
| Secrets/vault data in source/diff/fixtures/logs | Reviewed changed-path list/diff: product files and Markdown only, no vault/key/export/environment file. New test keyfiles/data remain outside repo under the throwaway home. Existing test fixtures are not real notes. |

No additional proven security issue was found in this self-check. That statement does not replace Claude's independent review or resolve the limits above.

## 10. Outside write area, mechanical

- `src-tauri/src/vault/mod.rs`: a memory-only session identity field/getter and initialization in the existing Session constructors. This is necessary binding plumbing so reopening the same vault cannot revive an earlier approval; it changes no record/header/schema/crypto format or disk data. Listed explicitly because the Session struct is outside the named command/module paths.
- `src-tauri/src/changes.rs`, `src-tauri/src/vault/history.rs`: existing tests call approval-injecting test helpers after core signatures gained the required permit. Test-only substitutions; no production behavior change there.

No other outside-area edits. Session/ledger/claims/authority files were intentionally left to Claude under this lane's write boundary. Configured/served model, effort and exact token/quota readings are not exposed by these tools; **not recorded**, to be obtained from the coordinator's run receipt. Runtime partner/machine and source identity are recorded above; no inference from historical quota readings.

## 11. ADDITION FOR AHMED

The authorized visible addition is a Windows OK/Cancel prompt after the unchanged window question for **delete for good, empty the bin, turn AI access on, and approve a new AI pairing**. Exact wording is in section 1. Cancel is quiet and leaves the refused operation undone. The new keyfile error is in section 3. **No window dialog wording changed.** Ordinary AI action/Strong confirmations and level changes remain in the window. No additional feature or restyling was built.

Claude's next step: inspect this diff, preserve the whole-app supplement when integrating lane W, run the required independent Claude security review and exact-commit checks, then commit/push within Ahmed's authority. Person-only checks remain open; there is no acceptance or release claim.
