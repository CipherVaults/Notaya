# Notaya 1.6 independent security pass (WP-17)

Findings first; no fixes are authorized by this report. Claude reviews the evidence and makes the final security call. Later area reviews may append their own sections below.

## Area 1: Tauri commands and CSP

- **Commit reviewed:** `13185af2c1be96a539ec1ec5446e99e0e14def89`, branch `AE0101/v1-6-s`; clean at entry.
- **Reviewer:** fresh Codex session which did not write the reviewed code; Ahmed Emara (`AE0101`) side. Machine: `GAMING-PC` (read from the runtime).
- **Authority:** Ahmed's Area 1 instruction in this conversation (message timestamp unavailable); A49 / WP-17. Review began 2026-11-09; checkpoint clock: 07:46:15 UTC (system tool).
- **Scope:** every registered Tauri command, action catalogue, CSP/capabilities, frontend active-content sinks and the command paths to vault, files, settings, windows and processes.
- **Write area:** this file and proof-only files under `tools/security-pass/area1/`. No product edits, installs, network, Notaya executable, or Git mutations.
- **Return / stop:** command inventory, proven findings and explicitly unproven questions, evidence limits and self-check; stop before fixes for Claude's review. Shared claims, session and usage records remain Claude's responsibility under this bounded brief.
- **Result state:** produced findings; the named standalone core proofs ran successfully on the reviewed source. No integration, acceptance or whole-app security claim.

### Attack surface and method

**121 commands** in `src-tauri/src/lib.rs:198`, plus the **125-entry** catalogue in `src-tauri/src/actions/registry.rs`. The local `main`, `sticky-*` and `calc-*` windows load the same `index.html`; most custom commands use `Actor::Person` without distinguishing these windows. Ten AI-control commands also require the real Tauri window label `main`. The renderer-compromise prerequisite in the brief is assumed: this review did **not** find or demonstrate an XSS entry point. A script in that renderer is distinct from the excluded same-user malware threat.

Inputs traced: serde command arguments, raw attachment bodies/headers, document JSON, IDs and revisions, file-picker results, backup source strings, keyfile paths, settings, error reports and AI approval answers. Sinks: encrypted records/attachments/history/activity, data-folder files/backups, chosen export files, temporary row-file copies, fixed Windows startup registration, browser/Explorer opening, window operations and emitted events. Read wrappers, their cores and the selected downstream checks before rating; speculative paths remain questions. No product fixes.

### Findings

#### S1-H1 [High] Permanent deletion bypasses the existing confirmation dialogs

- **Where:** `src-tauri/src/commands/notes.rs:688` (`delete_page`), `:705` (`empty_trash`), `:1446` (`delete_bin_tree`). Existing UI confirmations are at `src/components/Sidebar.tsx:1312` and `:1329`.
- **Reachable path:** any unlocked local window script -> registered `trash_page(id)` if necessary -> `delete_page(id)` / `empty_trash()` -> `AppState::with_mut` as Person -> core -> bin-membership/type checks -> `remove_many` / `collect_garbage`. The sink deletes encrypted records and associated history (`src-tauri/src/vault/mod.rs:342`). There is no confirmation argument, native prompt or Rust approval check on this IPC route. A bin check does not prevent a script first moving a live page to the bin.
- **Attacker controls:** command selection and the target IDs, obtainable through `list_notes` / `list_bin`. No current password is required once the protected vault is unlocked. This is not a claim that a locked vault can be read.
- **Proof:** `core-repro.rs`, launched by `run-proofs.py`, executed the actual production cores on a newly generated password-protected throwaway vault. It trashed and permanently deleted one page, emptied a second from the bin, then reopened the vault and confirmed both were absent and a third live page survived. No approval was supplied. IPC adapter reachability was established by the full source path; a WebView invoke was **not run**. These specific permanent-delete gates are not listed among the A18/A21 known UI-only limits.
- **Smallest fix:** obtain a trusted Rust-owned native confirmation in both IPC adapters, binding the decision to the exact item/subtree or current bin set, then call the existing deletion cores. Keep the existing bin/storage design. A nonce delivered to the same compromised renderer is insufficient. Claude must list any visible security-dialog change for Ahmed before implementation.
- **Regression:** a denied/missing Rust approval leaves the throwaway vault and history unchanged for both commands; calling trash then delete cannot bypass it; a changed target/bin set invalidates an earlier approval. Keep an explicit person-only native-dialog acceptance step.

#### S1-H2 [High] A main-window script can answer AI action and pairing approvals

- **Where:** `src-tauri/src/commands/ai.rs:159` (`ai_confirm_answer`) and `:177` (`ai_pair_answer`); `src-tauri/src/ai/confirm.rs:68` (emitted ID/word), `:88` (`answer`). Related AI setters are `ai_set_enabled` (`:54`) and `ai_set_level` (`:70`).
- **Reachable path:** dispatcher requests confirmation -> broker stores pending request and emits its ID and, for Strong, the required word -> `lib.rs:154` emits to `main` -> any script running there can submit those fields to the registered answer command -> `main_window` checks the label -> broker accepts -> dispatcher rechecks permissions/revisions and executes (`ai/dispatch.rs:1043`, `:1057`). Pairing approval uses the same pattern with kind `pairing`.
- **Attacker controls:** the answer `allow`, request ID and word read from the main renderer's own event. The caller label is supplied by Tauri and cannot be forged from a sticky window; that does not distinguish a person from a script in `main`. Such a script can also pass the literal `ALLOW` to enable AI and use main-only setters to change levels/hidden notebooks. This does **not** demonstrate a pipe-only client answering a prompt, stealing pairing tokens, or pairing remotely without renderer compromise.
- **Proof:** the source-linked standalone proof invoked the actual broker's Strong action request and pairing request, answering programmatically from the emitted fields with no window or person. Both returned allowed. Wrong kind, wrong word and duplicate-answer controls were rejected. The registered adapter and event route were traced in source; end-to-end WebView/pipe execution was **not run**. IDs being random/single-use and requests expiring do not establish human approval when the script receives the answer material.
- **Smallest fix:** move approval for pairing and destructive AI operations to a trusted native prompt owned by Rust; do not expose an IPC route by which renderer code can authorize it. Gate AI enable/elevation consistently if protection against a compromised main renderer is a requirement. Retain the broker's cancellation, parameter/revision binding, timeout and single-use checks. Claude must reconcile this with the guide's explicit no-script-approval requirement and obtain Ahmed's word for visible security behavior changes.
- **Regression:** receiving/reading an event and invoking either answer endpoint cannot authorize a request without the native approval result; deny/timeout cancels; wrong request/action/parameters/replay fail. Native confirmation itself remains person-only, not automated clicks.

#### S1-M1 [Medium] Extensionless row files skip destination containment

- **Where:** `src-tauri/src/commands/database_io.rs:649` in `save_file_as_person`.
- **Reachable path:** `attach_file` (or a Files-cell update) supplies an extensionless stored name -> `save_file_as(rowId, propId, fileId)` -> unlocked session, row/property membership and 32-hex file ID checks -> native save picker -> `ext.is_empty()` takes `chosen.clone()` instead of `checked_destination` -> only vault-extension check -> `std::fs::write` at `:653`. The usual destination guard is `src-tauri/src/export/mod.rs:990`.
- **Attacker controls:** attachment name/bytes and which stored attachment is saved. The destination is still chosen by a person in the native picker; a script cannot supply it directly. Saving into the app's data folder is nevertheless supposed to be refused (P27). The unusual human choice makes this Medium.
- **Proof:** `run-destination-proof.py` compiled and ran the unchanged destination/write tail, `extension`, `resolve`, `checked_destination` and vault-extension predicate extracted from this exact source. With an extensionless name and a picker-result path inside throwaway `data/shell-prefs.json`, the tail overwrote the marker file. A `.json` attachment at the same destination was refused with bytes unchanged. A direct `.nota` path remained refused. The native picker and full `file_of` path were traced, not exercised; this is a real Windows filesystem reproduction of the write/check branch, not a Python reimplementation.
- **Smallest fix:** apply folder containment and vault-file checks independently of the attachment extension; keep an extensionless output name if that is the intended behavior. Reuse the shared guard after separating extension handling from destination validation.
- **Regression:** save extensionless and named-extension attachments to the data folder, a junction into it, a normal external directory and vault-extension destinations; forbidden writes must leave existing files unchanged. Do not require an actual native picker for the path-helper tests.

#### S1-M2 [Medium] Renderer error reporting can persist note text in the plaintext log

- **Where:** `src-tauri/src/commands/errorlog.rs:23` -> `src-tauri/src/logging/mod.rs:241` -> `src-tauri/src/logging/sanitize.rs:197` (`frontend_message`).
- **Reachable path:** any local window script reads displayed note text while unlocked -> invokes `log_frontend_error({report:{kind:"error",message:<text>,source:"index.js",line:1}})` -> only the numeric line is validated in the adapter -> lexical sanitizer strips quotes/paths/token syntax but preserves ordinary unquoted text -> plaintext `data/logs/notaya-log.txt`. The reporting command itself is available while locked too. Rate limits bound volume, not confidentiality.
- **Attacker controls:** message text and script-like source basename. No key or password disclosure is needed. The plaintext copy remains after the vault session is dropped; reading the encrypted vault file is no longer needed to recover that copied text.
- **Proof:** the standalone core proof created a generated note title in a password-protected throwaway vault, read it through the production page core, dropped the reopened vault session, submitted the displayed title through the exact production logging sink used by the adapter, and found it unchanged in `notaya-log.txt`. No real note or secret was used or printed. A live IPC call was **not run**. The code's claim that UI reports are cleaned is weaker than a guarantee that no note content reaches the log; its best-effort lexical redaction does not cover this concrete input.
- **Smallest fix:** accept a closed error-class/code plus bounded source/line fields from the renderer, and render fixed Rust text for the log; do not persist arbitrary renderer message text. Keep the existing plaintext diagnostic log, rotation and limiter.
- **Regression:** a randomly generated note-title/body marker submitted as message or source cannot appear in log bytes, whether it has quotes/paths or is plain prose; known error codes still produce useful fixed diagnostics.

### Questions (not proven findings)

- **S1-Q1 [Question] Arbitrary keyfile paths can name network/device locations.** `commands/vault.rs:126` -> `crypto.rs:166` reads metadata and bytes of a renderer-supplied path without a picked-path/local-only rule; `unlock_vault`, `disable_security`, `change_master`, `enable_security` and `restore_vault` share it. Unlike backup sources, UNC paths are not rejected before filesystem access. Whether Windows initiates an outbound share/authentication attempt on that path was **not tested** because this run forbids network access. A controlled throwaway local-share test with an OS trace would establish the actual network/authentication consequence. No claim of credential leakage is made. Claude should decide the intended keyfile locality rule before any fix.

### Evidence and limits

- **Final core proof:** `python tools/security-pass/area1/run-proofs.py 'D:\Notaya Tests\v1-6-S\area1-proof-20261109T075233781764Z'`, exit **0**, 2026-11-09 07:57 UTC on `GAMING-PC`, against source commit `13185af2c1be96a539ec1ec5446e99e0e14def89`. Uses the actual source modules and the existing dependency declarations/lockfile with `cargo run --offline`; its small standalone binary has no application entry point, window, dialog or pipe. It uses generated passwords and a cheap test-only KDF, and never calls `Paths::resolve`. Final result: `D:\Notaya Tests\v1-6-S\area1-proof-20261109T075233781764Z\result-20261109T075746607917Z.txt`; final data: the adjacent `data-run-20261109T075746607917Z`. Earlier result/data directories remain preserved. It is not `cargo test --lib` and does not claim that suite passed. Compiler warning: unused legacy-guard imports in this standalone harness.
- **Destination proof:** `python tools/security-pass/area1/run-destination-proof.py`, exit **0**, 2026-11-09 07:53 UTC. Source-linked Rust tail and actual filesystem write. Evidence: `D:\Notaya Tests\v1-6-S\area1-destination-20261109T075352268834Z\result.txt`, adjacent generated Rust source and `data-run`. Compiler warning: the harness's minimal Error type has narrower visibility than its extracted public helper; product code was not changed.
- **Controls observed:** locked shared read gate refused access; unrelated live page survived empty-bin deletion; wrong-kind/wrong-word/duplicate broker replies failed; normal extension path refused the data folder; direct `.nota` output refused.
- **Not run:** Notaya executable; WebView/native-picker/native-prompt/OS opener interaction; whole-app or browser tests; `npm ci`, `tsc`, build, `npm audit`, `cargo test --lib`, clippy, cargo audit. No network, install, debug port, commit, push or other Git mutation by this reviewer. Live PR/fetch checks were omitted under the no-network instruction. No external viewers or Windows security/registry settings were exercised.
- **Not reviewed in depth:** vault cryptography/storage/history integrity and retention beyond the named command sinks (Area 2); hostile document/ZIP/PDF/CSV parser internals and all export encoders (Area 3); pipe ACL/handshake/token/pairing transport and the complete dispatcher permission matrix (Area 4); P7 caches/performance work or commits after this baseline. Linking those modules into a proof is not an audit of them.
- **External checkpoint observation:** the lane heartbeat created `c594b89` (07:52:30 UTC) and `28d2e30` (08:01:53 UTC) while this review ran. This reviewer issued no commit/push command. A final read-only diff from `13185af` confirmed zero changes in product source/configuration/dependency files; only this report and the three proof sources differ. The source-review baseline remains `13185af`, while the current report has further uncommitted additions above the heartbeat checkpoint.
- **Usage provenance for Claude's ledger:** model/effort selected in the coordinator's brief: `gpt-6.1-sol`, `xhigh`; actual served model/effort and exact token/quota readings are not exposed by this session's tools, so remain **not recorded**, not inferred. Shared ledger/session/claims files were intentionally left to Claude because they are outside this run's write area.

### CSP, capabilities, frontend and file/process checks

- Read `src-tauri/src/lib.rs`, `tauri.conf.json`, `capabilities/default.json`, `src-tauri/build.rs`, `src-tauri/src/ui.rs`, `index.html`, `vite.config.ts`, `src/main.tsx`, all production portions of `commands/*.rs` (including helpers after interleaved test modules) and `actions/{actor,mod,registry}.rs`. Existing test bodies were not exhaustively reviewed or run. Selected downstream functions are identified in the inventory and findings, rather than claiming whole-module audits of storage/crypto/dispatch.
- Production CSP: script self only; no unsafe-eval/inline scripts or remote connection hosts; styles self plus unsafe-inline; images self/data/blob and `http://clatt.localhost`; fonts self/data; media/object/frame none; base/form none. Dev policy additionally permits the local Vite websocket, and is not a shipped-app network allowance. Only `style-src` CSP modification is disabled. `freezePrototype:false` is an explicit existing owner-approved limitation; not a finding.
- The one capability applies to main/sticky/calculator labels, with core events/window controls only. No direct fs/shell/opener/dialog/HTTP permissions or remote origin rule. No shell plugin is registered. Dialog/opener are invoked by Rust, whose helpers must therefore enforce their own restrictions. The default `asset:` filesystem protocol is disabled with no scope configured (checked against the installed `tauri-utils-2.10.2` default), distinct from bundled app assets and the custom `clatt` protocol.
- `clatt` uses an unlocked session and parses attachment IDs in `get_attachment`; returns no-store/nosniff, CSP `default-src 'none'; sandbox`, and nonpicture content as octet-stream/attachment. Image MIME is sniffed on the write paths. No script-bearing SVG write path was found. Attachment size is capped by the vault blob writers. IDs are not filesystem paths.
- Frontend search over all `src/` found no `dangerouslySetInnerHTML`, dynamic eval/new Function, postMessage, srcdoc or string timers. The sole application `innerHTML` assignment is the fixed CHEVRON literal in `editor/blocks.ts:134`. Read the full editor extension/blocks/math/page-look paths and Mermaid security/rendering sections: KaTeX trust is false; Mermaid strips directives, initializes strict, cleans generated SVG and imports nodes. No exploitable injection was demonstrated. This is source evidence, not a browser/CSP bypass test or a third-party-library audit.
- Links use `open_url`, restrict final scheme to http(s), and provide no application/program override. Internal `notaya:` links are handled inside the editor and refused by that backend opener. Data/log folder openers use fixed Rust paths. `open_file` checks row/property/file membership, cleans the filename, allows a closed extension list, writes a generated temporary directory and uses the default associated application. No command can specify an arbitrary executable or command-line arguments. File type safety in external viewers was not established by their extension allowlist.
- Import/export adapters use Rust-owned pickers and closed source/scope/format choices; export guards canonicalize the target or parent and refuse the data folder/vault extensions, except S1-M1. Restore uses automatic-backup names or the last picked canonical local path, refuses traversal/UNC/device/live-vault destinations, verifies the staged file and preserves the previous vault. `allowSecurityOff` and enable-security backup deletion remain the explicitly documented A18/A21 UI-only limits, not new findings.
- Security on/off/master changes are session-gated and use the existing credential/rekey paths; password strings/keyfile digests use Zeroizing buffers. Current credentials are required for disabling/changing protected security. `get_settings` removes AI pairing data and histories; `save_settings` preserves stored AI rules, histories and last-place data. It can change the existing auto-lock/screenshot settings without reauthentication, as its current settings contract allows; the AI action exposes a smaller appearance-only field set. No keys/tokens are returned by the reviewed command results.
- Browser storage callers keep appearance/layout/collapse/dismissal choices; searches/calculator history/last-place live in the encrypted settings. The restore notice in sessionStorage is an error notice, not a note-body cache. No new secret-storage finding was established there.
- Other repository HTML files are two development mock harnesses and standalone manual/ledger artifacts. The only product HTML entry is `index.html`; Vite's product config defines no additional HTML input, and Rust windows all load that entry. Mock harnesses were read; manual/ledger HTML are not product WebViews and their separate scripting was not audited.
- **Advisories:** dependency advisory checks not run (offline brief); none asserted clean. No installs or dependency changes.

### Command-by-command inventory

`U` = any local app window, only with an unlocked session through `AppState::with/with_mut` (the shared lock control was exercised). `W` = any local app window without that precondition. `M+U` = real main window plus unlocked state. All parameters have serde/Rust types; a lookup-only ID is not assumed to have an additional length/hex check. All write cores are Person on this IPC path. AI levels/confirmations are enforced separately by the typed dispatcher, not by reading the catalogue or calling arbitrary commands.


| Command / source line | Caller | Input checks | Reach / result |
|---|---|---|---|
| `ai_get_status` (ai.rs:41) | M+U | No supplied arguments; explicit public pairing projection | AI settings/status metadata; no hashes/tokens |
| `ai_set_enabled` (ai.rs:54) | M+U | bool; enabling requires literal ALLOW (script-supplied) | Encrypted AI settings; invalidate live connections; S1-H2 |
| `ai_set_level` (ai.rs:70) | M+U | Pairing lookup; Level enum; Off refused | Pairing level; cancel pending confirmations; S1-H2 |
| `ai_remove_pairing` (ai.rs:90) | M+U | Pairing ID equality; unknown ID is a no-op | Remove pairing; invalidate live connections |
| `ai_set_hidden` (ai.rs:102) | M+U | At most 10,000 IDs; each must name Notebook | Hidden-notebook settings; invalidate connections |
| `ai_set_big_change` (ai.rs:124) | M+U | bool; optional usize threshold 1..500 | AI confirmation settings; invalidate connections |
| `ai_pause` (ai.rs:143) | M+U | No supplied arguments | Encrypted pause flag; invalidate connections |
| `ai_resume` (ai.rs:151) | M+U | No supplied arguments | Encrypted pause flag; invalidate connections |
| `ai_confirm_answer` (ai.rs:159) | M | Kind/ID/pending/word/single-answer broker checks; no human proof | Pending action/pairing approval; S1-H2 |
| `ai_pair_answer` (ai.rs:177) | M | Kind/ID/pending/word/single-answer broker checks; no human proof | Pending action/pairing approval; S1-H2 |
| `list_actions` (mod.rs:67) | W | No supplied arguments; names/labels/icons only | Compiled action metadata, no execution |
| `vault_status` (vault.rs:88) | W | Open gate; header; absent/earlier-copy/install conditions | Vault status/data-directory metadata; may create/open only an open vault |
| `start_notes` (vault.rs:179) | W | Open gate; vault must not exist | Create a new open vault; does not overwrite an existing vault |
| `enable_security` (vault.rs:208) | U | Password >=10 chars and strength >=3; optional keyfile digest | Rekey open vault; optional checked backup then remove old open copies (known limit) |
| `disable_security` (vault.rs:252) | U | Rate limit; current password/keyfile verified in make_open | Rekey protected vault to open; update capture flags |
| `unlock_vault` (vault.rs:289) | W | Rate limit; header/password/keyfile authentication; tray-hidden guard | Install session; restore window/capture state; keyfile path question S1-Q1 |
| `lock_vault` (vault.rs:337) | W | No supplied arguments; open-security-off behavior retained | Flush windows; drop session; close popouts; backup |
| `change_master` (vault.rs:346) | U | Rate limit; new password rule; old credentials verified | Transactional rekey; checked backup; no password/key result |
| `pick_keyfile` (vault.rs:392) | W | Rust-owned native file picker; no UI path argument | Return chosen path, not keyfile bytes |
| `create_keyfile` (vault.rs:406) | W | Rust-owned save picker; no UI path argument | Write 64 random bytes to chosen path; return path (no containment guard here) |
| `backup_now` (vault.rs:427) | U | Rust save picker; checked copy; live vault refused by save_copy | Encrypted copy to chosen path; return its path |
| `get_settings` (vault.rs:453) | U | Remove AI fields, calculation/search histories and replaced edits | Settings for display; no key or pairing hash |
| `save_settings` (vault.rs:472) | U | Settings types; sanitized theme/font-size/idle ranges; preserve AI/history/location | Encrypted settings; updates screenshot/auto-lock choices under current contract |
| `set_last_place` (vault.rs:501) | U | Known page/row/database kind; IDs <=128 allowed ASCII; view only for DB | Encrypted last-place IDs; no note text or filesystem path |
| `set_backdrop` (vault.rs:587) | W | String matched to mica/acrylic or harmless default; bool | Calling window material only |
| `window_ready` (vault.rs:594) | W | Actual Tauri label; tray-hidden main is not shown | Readiness tracking and show/focus window |
| `flush_done` (vault.rs:614) | W | Actual Tauri label and u64 round | Acknowledge only that window in a flush round |
| `quit_app` (vault.rs:621) | W | No supplied arguments | Flush/lock/backup and app exit; no arbitrary process start |
| `take_notice` (vault.rs:629) | W | No supplied arguments; take once | Stored diagnostic text |
| `open_data_folder` (vault.rs:634) | W | No UI path; fixed state.paths.data | OS default folder opener |
| `log_frontend_error` (errorlog.rs:23) | W | Typed report; line finite/nonnegative; lexical redaction/length/rate caps | Plaintext diagnostic log; S1-M2 |
| `log_info` (errorlog.rs:35) | W | No supplied arguments | Fixed log folder, size and last-entry time |
| `open_log_folder` (errorlog.rs:41) | W | No UI path; fixed log directory | Create fixed logs folder and open it |
| `get_shell_prefs` (shell.rs:38) | U | No supplied arguments; current-exe matching | Read fixed preference file/Run value |
| `set_shell_prefs` (shell.rs:59) | U | Three booleans; executable comes from current_exe | Fixed per-user Run value and shell-prefs.json; no caller program/args |
| `list_backups` (backup.rs:35) | W | Fixed backup directory/name rules | Names/sizes/times only; no decryption |
| `pick_backup_file` (backup.rs:63) | W | Rust picker; canonicalize selected path | Store last pick for restore; return displayed path |
| `restore_vault` (backup.rs:141) | W | Rate limit; source allowlist/pick; size/header/authentication; optional allowSecurityOff | Stage/verify/install; preserve old vault; current-vault reauth is known limit |
| `start_over` (backup.rs:117) | W | Must be locked, rechecked under restore/open gates | Set aside only live vault/journal; no deletion; typed UI gate is existing design |
| `import_portable_notes` (backup.rs:85) | W | Installed copy only; native folder picker; locked/no existing vault sink checks | Copy portable vault/backups without modifying source |
| `history_list` (history.rs:34) | U | Page/row exists; lookup ID | Encrypted version metadata |
| `history_get` (history.rs:38) | U | Version belongs to page history; authenticated body/hash | Decrypted selected version, no disk export |
| `history_restore` (history.rs:46) | U | Page/version lookup; not in bin; write check; snapshot first | Apply historical page/row body; no IPC human approval check |
| `history_snapshot` (history.rs:54) | U | Existing page/row; unchanged hash skips duplicate | Encrypted snapshot in vault |
| `activity_list` (activity.rs:29) | U | Optional actor/before; usize limit clamped 1..500 | Activity metadata |
| `activity_verify` (activity.rs:46) | U | No supplied arguments; journal verifier | Integrity result, no raw log body |
| `activity_undo` (activity.rs:50) | U | Change-set lookup/integrity; actor ownership; skip later person edits; target checks | Restore permitted items through shared write path |
| `activity_clear` (activity.rs:55) | U | Person actor and literal CLEAR | Clear encrypted activity log; preserve clear anchor; no native human check |
| `list_notes` (notes.rs:76) | U | No supplied arguments; tree/bin filtering | Notebook/section/page metadata and previews; database metadata |
| `list_bin` (notes.rs:1385) | U | No supplied arguments; own trash and parent-bin computation | Binned item metadata, including title |
| `restore_bin_item` (notes.rs:1433) | U | Existing ID; typed bool; containing-bin permission flag | Restore item and explicitly permitted ancestors |
| `editing` (vault.rs:680) | U | Existing item ID; actual window label; bool | RAM edit-lease bookkeeping |
| `create_notebook` (notes.rs:90) | U | Name <=120; icon <=40; safe hex/name color; update type lookup | Encrypted notebook; create also adds first section |
| `update_notebook` (notes.rs:135) | U | Name <=120; icon <=40; safe hex/name color; update type lookup | Encrypted notebook; create also adds first section |
| `delete_notebook` (notes.rs:179) | U | Typed item lookup; checked safety backup before binning | Restorable container trash flag; no permanent remove |
| `create_section` (notes.rs:208) | U | Name <=120; safe color; notebook/section type lookup | Encrypted section metadata |
| `update_section` (notes.rs:250) | U | Name <=120; safe color; notebook/section type lookup | Encrypted section metadata |
| `delete_section` (notes.rs:282) | U | Typed item lookup; checked safety backup before binning | Restorable container trash flag; no permanent remove |
| `create_page` (notes.rs:311) | U | Title <=300; section or page/row parent lookup; generated IDs | Encrypted new page |
| `get_page` (notes.rs:352) | U | Page or row type lookup; attached revision | Decrypted body/text/metadata displayed by windows |
| `save_page` (notes.rs:369) | U | Title <=300; content/text <=20M chars; page/row lookup; optional base_rev | Revision-aware encrypted whole-page save |
| `set_page_favorite` (notes.rs:487) | U | bool; page/row/database lookup | Favorite flag; binned DB cannot be favorite |
| `set_page_look` (notes.rs:874) | U | Icon <=40; cover <=200; fixed forms/32-hex vault image; icon <=2MB | Encrypted icon/cover; remote/data/file covers refused |
| `create_subpage` (notes.rs:469) | U | Title <=300; section or page/row parent lookup; generated IDs | Encrypted new page |
| `move_page` (notes.rs:540) | U | Source page/DB and destination section lookup; cycle-safe descendant list | Move page subtree or database; no filesystem move |
| `reorder` (notes.rs:567) | U | Vec of existing typed item IDs; no separate vector cap | Only display order via put_many |
| `trash_page` (notes.rs:643) | U | Page/row/database lookup; cycle-safe descendants | Restorable trash flags |
| `restore_page` (notes.rs:664) | U | Type lookup; containing notebook/section must be restored first | Clear trash flags on item/subtree |
| `delete_page` (notes.rs:689) | U | Known item kinds and bin membership; no Rust confirmation | Permanent record/history/attachment removal; S1-H1 |
| `empty_trash` (notes.rs:706) | U | Known item kinds and bin membership; no Rust confirmation | Permanent record/history/attachment removal; S1-H1 |
| `add_attachment` (notes.rs:741) | U | Raw bytes; PNG/JPEG/GIF/WebP magic; vault cap 900MB | Encrypted blob, generated 32-hex ID; SVG refused |
| `create_database` (database.rs:406) | U | Name cap; section and page/row parent lookup; first DB backup; sanitize | Encrypted schema; closed starter choices, unknown choice gives plain schema |
| `get_database` (database.rs:469) | U | Database lookup; repeat-template logic stays within row caps | Database with revision; may add due repeat rows |
| `save_database` (database.rs:529) | U | JSON <=30MB; typed schema; sanitize caps; locked-schema rule; relation validation; base_rev | Atomic schema/row/back-relation writes; security settings unreachable |
| `query_view` (database.rs:562) | U | DB/view lookup; temporary view sanitized like saved view | In-memory query/chart result; no file, process or eval |
| `create_row` (database.rs:641) | U | Title <=300; cells JSON object; schema clean_cell; template lookup; 200k row cap | Encrypted row/relation updates; bounded automations |
| `set_cells` (database.rs:725) | U | Optional title <=300; JSON object; typed/finite/size/id clean_cell checks | Encrypted row/relation updates; bounded automations |
| `press_button` (database.rs:797) | U | Row/database/property lookup; property must be Button | Closed bounded automation actions in vault only |
| `duplicate_row` (database.rs:821) | U | Existing row/schema; row count cap; excludes computed/ID fields | New encrypted row plus relation sync; bounded automations |
| `reorder_rows` (database.rs:857) | U | Each ID must name row; no separate vector cap | Encrypted row order only |
| `search_rows` (database.rs:888) | U | Database ID equality on rows; query text; limit clamped 1..200 | Matching row labels; no files |
| `row_titles` (database.rs:921) | U | Only existing Row variants from supplied IDs; no vector cap | Row ID/title/icon labels |
| `set_database_look` (database.rs:1095) | U | Title/name cap; icon <=40; typed booleans; DB lookup | Title/icon/favorite/schema-lock flag; not vault credentials |
| `move_database` (database.rs:1138) | U | DB and destination section lookup | Encrypted location fields |
| `enable_sub_items` (database.rs:1162) | U | DB lookup; at most 100 properties; generated relation IDs; sanitize | Closed self-relation schema additions |
| `enable_dependencies` (database.rs:1201) | U | DB lookup; at most 100 properties; generated relation IDs; sanitize | Closed self-relation schema additions |
| `chart_data` (database.rs:1239) | U | DB/view lookup; temporary view sanitized like saved view | In-memory query/chart result; no file, process or eval |
| `due_reminders_now` (database.rs:1332) | U | No supplied arguments; date/automation filters and run budgets | Titles/notices; date automations; RAM shown-set |
| `get_row` (database.rs:625) | U | Row/database lookup; in-memory typed values | One row for display |
| `remove_file` (database_io.rs:568) | U | Row/Files-property checks; file ID used only to filter cell | Remove reference; garbage collect unused encrypted blobs |
| `import_csv` (database_io.rs:263) | U | Existing target before native pick; <=2GB read; parser/rows/columns caps; sanitize | New/imported encrypted rows; first DB safety backup |
| `export_csv` (database_io.rs:461) | U | DB/view lookup; format closed for docs; native save pick; checked_destination | Readable chosen export; data folder/vault extension refused |
| `export_database` (database_io.rs:432) | U | DB/view lookup; format closed for docs; native save pick; checked_destination | Readable chosen export; data folder/vault extension refused |
| `attach_file` (database_io.rs:535) | U | Raw bytes and headers; decode/clean filename; row/Files property; <=100 files; 900MB cap | Encrypted blob/reference; magic images or octet-stream |
| `open_file` (database_io.rs:661) | U | Row/property membership; 32-hex file ID; cleaned name; 29-extension allowlist | Generated temp directory/file, default OS opener; no caller executable |
| `save_file_as` (database_io.rs:635) | U | Same membership/ID/name checks; native save pick; extension guard | Readable file; extensionless containment gap S1-M1 |
| `export_notes` (export.rs:70) | U | Closed scope/format/output; 1..10k IDs <=64; <=1k containers; native save pick | Readable single/ZIP export through canonical destination guard |
| `import_notes` (import_notes.rs:300) | U at writes | Closed files/folder mode; Rust picker; max 5k paths; source protection; cancellation/holds | Convert hostile sources, encrypted images then one record transaction; no input paths supplied by UI |
| `import_cancel` (import_notes.rs:80) | W | No supplied arguments | Cancel flag only |
| `list_stickies` (stickies.rs:13) | U | No supplied arguments; excludes binned stickies | Decrypted sticky bodies for display |
| `create_sticky` (stickies.rs:32) | U | bool open; generated ID and fixed initial geometry | Encrypted sticky; optional window with fixed app entry |
| `get_sticky` (stickies.rs:66) | U | Sticky type lookup; revision | Decrypted sticky body/style |
| `save_sticky` (stickies.rs:76) | U | Content/text <=2M chars; Sticky lookup; optional base_rev | Revision-aware encrypted sticky save |
| `set_sticky_style` (stickies.rs:105) | U | Safe color; bool; Sticky lookup | Encrypted style and matching sticky window on-top flag |
| `set_sticky_geometry` (stickies.rs:134) | U | Finite floats; width 220..1200, height 180..1200; Sticky lookup | Encrypted geometry; nonfinite inputs are a no-op |
| `open_sticky` (stickies.rs:163) | U | Sticky exists and not binned | Open fixed app entry under generated stored ID; no supplied URL |
| `close_sticky` (stickies.rs:181) | U | Sticky lookup | Mark closed and close matching fixed-label window |
| `hide_stickies` (stickies.rs:227) | U | No supplied arguments; flush windows first | Mark all closed; close sticky windows; no deletion |
| `delete_sticky` (stickies.rs:232) | U | Sticky lookup; flush first | Restorable bin flag; close window |
| `delete_stickies` (stickies.rs:253) | U except empty | Sort/dedup <=10k; wrong type refuses; missing IDs skipped; flush | Restorable batch bin flags; no permanent removal |
| `open_calculator` (calculator.rs:21) | U | No supplied arguments; recheck unlocked after creation | Fixed app entry with random calc label |
| `calc_history` (calculator.rs:37) | U | No supplied arguments | At most 500 encrypted-history entries to display |
| `add_calc_entry` (calculator.rs:47) | U | Expr/result <=1000 each; trim; empty/duplicate no-op; 500 cap | Encrypted calculator history; content-free changed event |
| `clear_calc_history` (calculator.rs:75) | U | No supplied arguments | Clear encrypted calculation history |
| `password_strength` (tools.rs:13) | W | String and hints Vec types; no additional caps; password Zeroizing | Strength estimate only; no password result |
| `open_url` (tools.rs:25) | W | Trim; reject notaya; default https; only http/https final prefix | Default browser only; cannot choose a program |
| `search` (tools.rs:69) | U except empty | Nonempty trimmed query; in-memory search; bin exclusion; max60 | Decrypted matched snippets for display; empty query returns [] while locked |
| `search_history` (search_history.rs:15) | U | No supplied arguments | Five encrypted stored terms |
| `add_search_term` (search_history.rs:25) | U | Term <=200 chars; whitespace normalization; case-insensitive dedup; five cap | Encrypted search history |
| `remove_search_term` (search_history.rs:35) | U | String equality against stored terms | Remove one encrypted-history term |
| `clear_search_history` (search_history.rs:48) | U | No supplied arguments | Clear encrypted search history |

### Action catalogue cross-check

All 125 entries have a fixed name, schema with `additionalProperties:false`, level, confirmation policy, undo metadata and UI label/icon. The catalogue is data, not an executable route: `list_actions` does not dispatch. The typed dispatcher selects a named entry, checks stored enabled/paused/pairing state, level, schema and targets, then rechecks after approval. No catalogue entry exposes `ai_set_*`, credential changes, restore, keyfile reads, raw attachment writes, OS opening or arbitrary file dialogs. Mixed `settings.set` is filtered to appearance fields and preserves stored AI policy; `db.set_look` preserves the existing database lock. `bin.delete_forever` and `bin.empty` specify Full/Strong, but that metadata is not enforced on Person IPC routes (S1-H1), and Strong still trusts a renderer reply (S1-H2). The complete pipe policy/target matrix remains for Area 4; no blanket pass is implied by this catalogue check.

### Own-file security self-check and hand-back

Reviewed every added proof and this report for write scope, accidental product edits, secrets, process/network behavior and claim accuracy. Only the report and three proof source files were changed by this run relative to its entry baseline. No build hooks or product test modules were changed. Proofs use explicit fresh folders below `D:\Notaya Tests\v1-6-S`, preserve prior data/results, log no actual keys/passwords/note contents, invoke subprocesses with argument arrays, use offline cached dependencies and start only standalone proof executables. No application/network/debug listener was started. No actionable security issue was found in these own files; they are test tools and are not an independent review of themselves. No fixes have been made. Claude owns review, shared records and all commits/pushes.

Repository files changed:

- `docs/plan/2026-11-09-security-pass-findings.md`
- `tools/security-pass/area1/core-repro.rs`
- `tools/security-pass/area1/run-proofs.py`
- `tools/security-pass/area1/run-destination-proof.py`

**Stop point:** Area 1 delivered for Claude's decision: 2 High, 2 Medium, 0 Low; 1 explicitly unproven Question. Do not treat these as owner acceptance or begin fixes from this report alone. Later independent areas should append new level-two sections after Area 1.

## Area 2: Vault, storage, history and activity log

- **Commit reviewed:** 1c129af183c77a7d24b84f3ab19addce6eb80559, branch AE0101/v1-6-s; origin/main a649a3c is included. Clean at entry. Fresh independent Codex session that did not write this code.
- **Who and authority:** Ahmed Emara (AE0101), Gaming-PC (runtime hostname). Ahmed's Area 2 message in this conversation authorizes this bounded WP-17 review; its message time is not exposed. Review and proofs: 2026-11-09, final proof at 08:21 UTC (system clock).
- **PBS scope/write area:** inspect vault crypto, persistence, history/activity integrity, lock lifetime, retention/bin/GC and restore/import; append this section and add proof sources only under tools/security-pass/area2/. Return proven findings, checks, limits and self-check; stop for Claude to read before any fixes. The specific brief overrides routine fetch/claim/ledger/session/commit steps: no network or Git mutation, and Claude owns shared records and all commits/pushes.
- **Result:** produced review with **0 High, 3 Medium, 0 Low** new findings. Standalone proofs reproduced all three on the named product source. No integration, acceptance, whole-app pass or cryptographic-library certification is asserted. Area 1 is not repeated.

### Attack surface and input-to-sink map

An offline attacker can edit a locked vault or a supplied backup's SQLite schema, header and opaque record/attachment/history/activity rows without knowing its password; a person selects local restore/import files; any local renderer can invoke the credential, history, activity and bin adapters; paired AI writes enter through the typed action dispatcher; OS randomness, keyfiles and lock events supply the other inputs. Traced sinks are Argon2/HKDF and AEAD operations, SQLite transactions and encrypted rows, filesystem copy/sync/rename/rotation, decrypted Session/coordination/query state, UI change events, plaintext diagnostics and the page/sticky/bin/history/undo cores. Same-user malware reading an unlocked process remains excluded; the disk modifications in S2-M1/M2 occur with no Session alive, not through that excluded threat.

| Entry | Checks followed | Sink / conclusion |
|---|---|---|
| unlock_vault / disable_security / change_master / enable_security | Password rule for new credentials; rate limiting; zeroizing arguments/digests; header length/magic/version/flags/KDF bounds; keyfile requirement; AEAD unwrap | Session/rekey. No demonstrated new credential bypass. Arbitrary keyfile locality is already S1-Q1 |
| Live file / backup ciphertext | AAD includes format binding/version, vault ID, closed table tag and 16-byte row ID; record ID matches decrypted ID; unpad checked | Record/attachment/history/activity loading. AEAD alteration/swapping is distinct from missing-row membership, below |
| restore_vault / portable import | Native picker/automatic name rule; traversal/ADS/UNC/device checks; size cap and bounded copy; KDF cap before work; staged verification; restore/open gates | Fixed staging/live/undo names; sync and rename; old live file preserved. S2-M1/M2 pass staged journal verification |
| History snapshot/get/restore | Existing page/row; version selected from its metadata index; body row ID and hash; write/AI ancestry checks; snapshot before restore | Two encrypted rows/version; put/put_many; membership deletion gap S2-M2 |
| Activity verify/list/undo/clear | Encrypted head, sequence/backlinks/canonical hash/count; actor ownership and after-hash/person-write checks for undo; Person plus CLEAR for clear | Encrypted chain and atomic write/undo. Head/table erasure gap S2-M1; retained undo body S2-M3 |
| Bin/permanent delete/GC | Supported item kinds, bin membership, cycle-safe ancestry; page/row histories removed with records; attachment references include bin/history/undo/import holds | Current vault purge; S2-M3 is separate from Area 1's missing IPC confirmation |
| AI registry writes | prepare/permission/target checks; confirmation and revision recheck; with_actor -> execute -> atomic_action -> run_action -> shared cores -> put_many/remove_many | All four encrypted tables included in transaction rollback/rekey/copy. Transport/whole action permission matrix reserved for Area 4 |
| Lock/auto-lock/signals/query | Session mutex; take_session invalidates AI and clears notices/change generation; session owns keys/items/index/revisions/leases; monitor handles protected vaults; UI reloads | Session is dropped after backup; query Ctx is per call. No persistent Rust query-result cache exists in this baseline |

### Findings

#### S2-M1 [Medium] Erasing the complete activity table passes 'Check the log'

- **Where:** src-tauri/src/vault/activity.rs:230 (load_activity_head), :459 (activity_verify); src-tauri/src/vault/mod.rs:241; src-tauri/src/vault/store.rs:65. Restore reaches this through src-tauri/src/restore.rs:221 and vault/mod.rs:623.
- **Reachable path:** edit a locked protected vault/backup -> DELETE FROM activity, or DROP TABLE activity -> normal Session::unlock/load -> missing table recreated -> missing head with no remaining rows becomes Head::default -> activity_verify walks zero entries, accepts the default hash/count -> UI 'Check the log' reports success. A later write starts a valid new chain without the required log.cleared anchor. Restore's full verify_attachments/verify_journals path accepts it too.
- **Attacker controls:** SQLite table/rows on disk only. No password, DEK, plaintext, forged AEAD tag, earlier file or renderer execution is needed. Current encrypted records are left byte-identical. This erases evidence of prior activity without modifying notes, so it weakens the append-only/dropped-row promise (WP-03/P33); it is not whole-vault rollback or the documented single-record replay limit.
- **Proof:** tools/security-pass/area2/core-repro.rs, activity-delete-all and activity-drop-table cases. The fixture closes its Session, the attack executes only the SQL above, and the legitimate owner then unlocks. Both return an empty list and successful activity_verify and full journal verification; all record ciphertexts remain identical. A new marker is accepted as a one-entry chain, and restore::verify_and_stage accepts the resulting file with the previous activity erased. Control: deleting one non-head row while retaining the head fails verification.
- **Smallest fix keeping the design:** persist an authenticated journal-initialized/expected-head anchor outside the deletable activity set, and check it before treating absent tables/head as an old pre-C3 vault. Initialize/migrate and update that anchor transactionally with the existing journal. Preserve genuine legacy opening and explicit clear/prune anchors. This protects erasure, not offline replacement of an entire older consistent file.
- **Regression:** close a protected C3 vault with several entries; empty or drop only activity; reopen and verify/restore must reject rather than report a clean empty log. Genuine pre-C3 files, an intentional clear and normal pruning must still work. Keep the current individual-row tamper tests.

#### S2-M2 [Medium] Deleting a whole history snapshot pair is undetected

- **Where:** src-tauri/src/vault/history.rs:331 (load_history), :143 (history_get); src-tauri/src/vault/activity.rs:516 (verify_journals); src-tauri/src/vault/store.rs:80.
- **Reachable path:** edit locked vault/backup -> remove one history metadata row (ID first byte 0) and matching body row (same ID with first byte 1) -> unlock rebuilds index from surviving metadata only -> history_list silently omits the version -> verify_journals validates surviving bodies and compares their count with that same surviving index -> success, including during restore staging.
- **Attacker controls:** ciphertext row existence and visible paired IDs, not decryption or hashes. Removing a pair needs no key and preserves live records and the activity chain. There is no authenticated expected snapshot membership against which the reduced index is checked. The pair layout is sufficient to remove arbitrary versions without knowing their page or text. Unlike thinning or permanent deletion, this happens without any authorized operation.
- **Proof:** history-remove-pair creates two real versions, closes the session, removes the earlier metadata/body pair with parameterized SQL, reopens, and observes one fewer version/NotFound for the removed ID. activity_verify, verify_attachments (which calls verify_journals) and restore::verify_and_stage all succeed. Control: removing only a body with metadata retained is rejected. This demonstrates deletion, not the already documented replay of an older record.
- **Smallest fix keeping the design:** authenticate expected history membership in a compact manifest/root outside the removable pair, updated in the same transactions as snapshot creation, thinning and permanent deletion. Check metadata membership on unlock and full membership/body integrity on restore, while retaining lazy body reads. Do not rely on a count recomputed solely from surviving metadata.
- **Regression:** deleting both rows of any version, all versions of one page or the history table must be detected for an initialized vault; deleting either half alone must still fail. Legitimate thinning, permanent deletion, legacy migration and lazy unlock must continue to pass.

#### S2-M3 [Medium] Permanent sticky deletion leaves its earlier text in current activity undo

- **Where:** src-tauri/src/vault/activity.rs:343 (Previous.before serialization; only Page/Row content and Settings replaced edits are cleared); src-tauri/src/vault/mod.rs:342 (remove_many); src-tauri/src/commands/notes.rs:1446 (delete_bin_tree).
- **Reachable path:** a legitimate AI edits an existing sticky -> prepare_journal serializes its full previous Item::Sticky, including content/text, into the encrypted activity entry -> the person bins then permanently deletes it (delete_page_core/empty_trash_core -> delete_bin_tree -> remove_many) -> current record/replacement history is removed, but immutable activity undo retains that complete prior body. activity_undo skips a missing current item; it does not purge those bytes.
- **Attacker controls / impact:** no bypass of encryption is claimed. A later reader of the file with valid vault credentials can recover the deleted text from current activity ciphertext; ordinary UI/AI list results do not expose the undo payload. The unusual credentialed forensic read makes this Medium. The concern is the 'delete for good really deletes' privacy behavior of WP-04/WP-05: the current file retains unnecessary note text after deletion. This is separate from titles deliberately kept in activity and from old/manual/undo backup copies retaining old content, which are known limits.
- **Proof:** sticky-purge creates a generated body marker, edits the sticky as AI through run_as/put, bins it with delete_sticky_core and permanently deletes it through delete_page_core. It confirms no current item remains and the log still verifies. After Session drop, a Rust reader uses the fixture's credentials with production Header::unlock/crypto::open and public AAD layout to find that generated body in current activity undo. No password, key or marker value is printed. Page/row body clearing is visible in source; row cells/database templates may have analogous retained previous values but were not separately reproduced here.
- **Smallest fix keeping the design:** extend the existing page-body indirection pattern to purgeable non-page undo payloads: activity keeps immutable metadata/hash/reference, while the previous sticky body lives in separate encrypted storage removed on permanent deletion (and checked before undo). Keep the chain valid and retain permitted titles. Do not silently rewrite historical activity hashes. Claude should settle the intended purge contract for row cells/database template bodies at the same boundary before a fix.
- **Regression:** after AI sticky edits, both delete-for-good and empty-bin remove all current-vault undo copies of the body while leaving valid activity metadata/chain. Undo cannot resurrect a permanently deleted item. Repeat for row cells/templates if the purge contract includes those values; ordinary binning must remain undoable. Old backups remain a separately documented limit.

### Coverage, exclusions and evidence limits

- **Read in full:** production crypto.rs; vault/header.rs, store.rs, mod.rs, history.rs and activity.rs; backup.rs and restore.rs production paths; import.rs production paths; openvault.rs; commands/history.rs and commands/activity.rs. Read the crypto/header/store tests and C3 history/activity tests, including tamper, retention, GC, deletion, undo, rollback, rekey and lazy-body tests. Read selected vault/backup/restore test bodies for encrypted disk bytes, row swaps, rekey/old-header attacks, copy membership, path/ADS rejection, corrupt attachments and pre-KDF caps. The remaining older tests were not exhaustively read or run.
- **Related source traced:** state.rs session/transaction/signal/flush/lock/monitor and backup lifecycle; changes.rs coordination; model.rs and database/types.rs secret-bearing types; credential/restore command adapters; bin tree and deletion cores; database engine Ctx cache lifetime; relevant AI prepare/execute/atomic write and undo paths; logging production sinks/sanitizers; App.tsx lock reload and frontend row-title/appearance caches. Reading/linking those selected functions does not imply a complete dispatcher, database-engine or frontend audit.
- **Crypto/secrets checklist:** Argon2id v1.3, NFC normalization, HKDF-SHA256, fresh OS-random key/salt/vault ID and random 24-byte XChaCha nonce; current row binding is notaya and legacy binding cipherleaf. Header AAD covers KDF/flags/salt/ID; closed table tags and row IDs prevent cross-location AEAD substitution. Generic file KDF bounds are 8 KiB..4 GiB, 1..64 passes/lanes; restore caps 1 GiB/10/16 before Argon2; production protected creation/change uses 256 MiB/3/4. Low accepted costs do not let an attacker lower an authenticated existing header. No new password-recovery mechanism was found. Core keys/credential/digest/padded buffers use Zeroizing; Item/history/activity types wipe supported fields on drop. Two DEK equality checks (vault/mod.rs:733, :762) use != rather than an explicit constant-time primitive; no attacker-controlled authenticated-key/timing oracle was demonstrated, so no timing vulnerability is asserted. This is not a compiler/allocator wipe audit.
- **Storage/files checklist:** parameterized values and closed Table names (formatted SQL cannot accept a caller table); SQLite DELETE journal, FULL sync, secure_delete and MEMORY temp store; rekey/copy includes all four data tables, not just old records/attachments. Checked backups compare header, IDs, ciphertext and decrypt each copied row before final rename. Automatic/hourly/predelete rotations are separate. Restore stages and syncs the verified copy, preserves live vault/journal under undo names, and rolls renames back on ordinary failure. Portable import copies with create_new, no overwrite, fixed destinations and bounded reads. No new file-content-to-path escape/overwrite was proven. No general .bak recovery branch was found in these modules; .bak names are excluded by backup name tests. Truncated AEAD input errors rather than indexing unchecked; malformed SQLite/schema/resource exhaustion was not fuzzed.
- **Memory/lock/retention checklist:** shared session gates refuse locked access; take_session invalidates AI, clears automation notices and advances/clears the change queue. Session owns items, history index, head, revisions and leases/claims; protected locks drop it after the backup, matching documented P10. UI reload and closing sticky/calculator windows are best-effort JS cleanup. The database Ctx cells/formula caches are query-scoped, not static; no later proposed persistent query cache was present in this commit. Frontend row-title cache is RAM-only and discarded by reload. History thinning protects latest 20 and recent AI-before versions, then applies age/size policy; binned items/history/import-held pictures are retained for GC. Page/row permanent deletion removes paired history rows; non-page undo retention is S2-M3. OS pagefile/hibernation/crash dumps, guaranteed physical erasure and same-user memory inspection remain outside proved coverage. Clipboard/panic/log source searches found no new credential output; plaintext renderer-message logging remains S1-M2. The panic hook writes fixed classes/function frames but still calls the previous hook; arbitrary stderr capture was not evaluated.
- **Not repeated as findings:** whole-file rollback; documented old-record replay (A2); old backups/restore-undo files opening under old credentials; open-vault confidentiality; optional-security UI-only lowering/backup-deletion gates; restore's known two-rename power-loss window; memory/pagefile and delayed-lock/save limitations; S1-H1/H2/M1/M2/Q1. Authentication of row contents does not imply detection of missing rows, as the new proofs show.
- **Frontend/IPC checklist:** covered the Area 2 command-to-core paths, lock cache lifetime and content-free change signals. No product CSP/capability/UI change was made. HTML/import parser behavior and general frontend injection remain Area 1/3; pipe transport/pairing/complete permission matrix remain Area 4. Dependencies/build/advisories: no network scans, installs or dependency changes; no advisory-clean claim.

### Reproduction and checks actually run

- **Command:** python tools/security-pass/area2/run-proofs.py; final exit **0** on Gaming-PC, 2026-11-09 08:21 UTC, source commit 1c129af183c77a7d24b84f3ab19addce6eb80559. Source-linked standalone Rust binary, not the Notaya executable and not cargo test --lib. It compiles current production modules with rustc and existing Area 1 cached libraries; it checks direct dependency declarations and cached package versions/checksums against the repository. The cache omits only the bridge workspace member; no package is installed/fetched and no build script is invoked.
- **Final evidence:** D:\Notaya Tests\v1-6-S\area2-proof-20261109T082136776920Z\result.txt; compile.txt; generated proof.rs; fresh data subfolder containing each named case. The generated password stays in Rust Zeroizing memory; the test-only KDF is 64 KiB/1/1 and is not a recommendation or change to production defaults. No Paths::resolve, app/window/pipe/native confirmation or OS opener is called. Compiler warning: unused legacy_guard imports in this standalone harness.
- **Observed controls:** individual missing activity row detected; missing history body detected; fresh seal nonces differ; wrong AAD/key and truncated ciphertext refused. Successful reproduction means the assertions demonstrating the defects held, not that the security properties passed.
- **Preserved attempts:** earlier area2-proof-* folders include two successful earlier subsets and failed cache-selection/compile/link/lock-equality setup attempts. They are retained, not substituted for the final run. The lock equality failure was workspace metadata (only notaya-bridge absent), verified before switching to package-identity/declaration comparison. Final result above is the complete run.
- **Not run:** Notaya executable; any real vault; WebView/native picker/native confirmation; Win+L/sleep/drive-removal/capture checks; heap or crash-dump inspection; power-cut/low-disk/junction-race fault injection; malicious SQLite fuzzing; full existing Rust suite, clippy, cargo audit; npm ci/tsc/build/audit; network/fetch/live PR inspection; any Git mutation. No debug listener was started.
- **Usage for Claude's records:** this session's served model, exact effort, token counts and before/after quota meters are not exposed by its tools and are **not recorded**, not inferred from historic defaults. Claude owns ledger/claims/session updates outside this write area.

### Own-file security self-check and hand-back

Reviewed both proof files and this appended report for write scope, secrets, process/network behavior, generated paths and claim accuracy. The runner reads the explicit existing cache, writes only a fresh timestamped directory under D:\Notaya Tests\v1-6-S, invokes rustc and the standalone proof with argument arrays, supplies a throwaway NOTAYA_HOME, and never launches the app or uses a shell-built subprocess command. All destructive SQL targets generated fixtures only. No real password/key/note is stored in source, report or output; no build hooks, production modules, dependencies or configuration were changed. **Own-file self-check: no actionable issue found; self-review is not independent.**

Repository files changed:

- docs/plan/2026-11-09-security-pass-findings.md (append only)
- tools/security-pass/area2/core-repro.rs
- tools/security-pass/area2/run-proofs.py

Area 1 preservation: the original 41,590 bytes remain an exact prefix, SHA-256 8b9c89dcad441b5a024fb18bb78c456f1aa6fffc9600fc34795abe3de1d71744. No Area 1 content was rewritten.

**CHECKPOINT: Area 2 review, three reproductions and hand-back done. Stop here: Claude reads and decides the findings before any fixes.** No fixes, commit or push by this reviewer.

External checkpoint observation (08:25 UTC readback): the heartbeat independently created 1d1fc6ac4224d9b8b448ab65dc18a684b279cbea at 08:21:54 UTC, committing the two Area 2 proof sources. This reviewer issued no Git mutation. A read-only diff from 1c129af confirms only the three listed report/proof files differ; product source/configuration/dependencies remain identical to the reviewed baseline. The report remains for Claude to commit.
