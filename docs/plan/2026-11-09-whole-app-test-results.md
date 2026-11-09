# Whole-app test results: X-1 addition

Who and where: Ahmed Emara (AE0101), Codex lane X1 on Gaming-PC (Windows sandbox). Baseline: `e4df9185a99fabf6002065c84c921cd8f8278e31`; external heartbeat checkpoint during implementation: `cff91cc3180c5311203cc41d54cd333bd4d757fb`. Produced, not integrated or accepted. This file was absent at the baseline; Claude owns integrating the row into the whole-app register and recording candidate commit/hash and PC observations.

## Defect register

| Defect / check | Reporter | Report and scope | Status | Candidate and evidence needed |
|---|---|---|---|---|
| X-1 / WIN-X1, KIND B | **Mohamed Sayed (AbyssWatcher2)**, via Ahmed, 2026-11-09 07:38:18 UTC | v1.5.1 on Mohamed's PC: blank white window at sign-in with Start with Windows on; not reproduced on Ahmed's PC | **not run** (real exe, sign-in and pixels) | Frontend readiness correction and regression harness produced. Claude runs baseline and candidate commands in `tools/real-exe-tests/startup-minimized.md`, records exact source commit, exe hash, output folders and manual observations. No runtime pass claimed. |

## Root-cause analysis from code, before runtime observation

| Candidate | Evidence for | Evidence against / what is still needed |
|---|---|---|
| Hidden-page paint/readiness stall — **most likely code-level cause** | `ui::build_main` creates a hidden WebView. Original `App.tsx` waits for appearance, status and **two animation frames** before `window_ready`. A hidden page can defer those callbacks; there is no fallback. The body and unlock card also animate from opacity zero. This couples readiness to a paint while intentionally hidden. | Code proves the dependency, not that WebView2 actually withheld frames on Mohamed's PC. Every ordinary start uses a hidden window too, so frame suspension alone does not explain why his sign-in differs. A tray restore ordinarily resumes frames. The persistent-white incident still needs observation. The deterministic scheduler test proves the repair of this seam only. |
| Window exposed before the page/status/animation is ready, or hidden-state initialization order | `shell::show_main` (tray, single-instance and legacy handover) unconditionally unminimizes/shows/focuses. It can expose a page during loading. Also, `lib.rs` sets `tray_hidden` only after building the hidden window and the tray; if an early readiness callback could run before setup finishes, it would see the default false flag. | A first Run-key start does **not** call `show_main`. The hidden flag is set in setup before the normal event loop runs; no evidence that readiness can overtake setup has been observed. Need a second instance/tray open, an early callback trace or a focus event at sign-in. No delayed-show thread or automatic native show was found on the first minimized path. |
| Missing `--minimized`, or wrong executable/Run value | `run_command(exe, prefs.start_minimized)` writes the quoted exe and adds the flag only when selected. A stale Run value or Start minimized off could change the path. | Quoting and parsing are correct for the expected command; existing Rust tests cover quoting and flag. `started_minimized` checks exact argument equality. Need Mohamed's actual command shape and executable version. No registry change is proposed. |
| Explorer/tray unavailable at sign-in | If tray creation fails, `lib.rs` deliberately leaves `tray_hidden` false; a ready window becomes visible so the app stays reachable. | Tray failure by itself should show a working page. There is no tray-dependent frontend load branch. Need native tray observation/logs at sign-in. No sleep, Explorer polling or startup semantics change is added. |
| GPU/compositor/WebView2 runtime initialization failure | Strong timing clue: seen at sign-in on one PC. Can explain DOM present but white pixels, or a dead renderer. | No source proof, crash event or runtime version from that PC yet. The harness distinguishes no CDP page, empty DOM, hidden native window and visible layout, but cannot prove compositor pixels. No GPU flags or WebView2 settings are changed. |
| WebView2 user-data folder locked/not ready | WebView2 uses `paths.webview`; session initialization might make it temporarily unavailable. | `ensure_dirs` runs before window creation, checks the data folder, and creation errors get a native startup error and sanitized log. A failed creation is therefore not silently treated as ready. This does not rule out a later renderer/profile failure. Need startup log/event classes and a no-page result. No profile deletion/reset is proposed. |
| Vault-status/appearance IPC stall or startup JavaScript failure | The body is conditional on status, and backdrop IPC is awaited before fetching it. A pending call or module failure can leave only a title bar or an empty root. | `vault_status` returns explicit states for missing, unreadable and protected vaults; there is no minimized-specific lock-screen branch. Logging and ErrorBoundary already exist (module-load errors can precede the boundary). Need DOM and log classes, particularly whether the custom title bar exists. The readiness timer starts only after a status screen commits; it deliberately does not pretend a failed status call succeeded. |

The smallest supported correction removes the unbounded animation-frame dependency: after the status screen commits, retain two-frame readiness normally, with a once-only 1-second timer fallback and effect cleanup. Rust's visibility/tray guard, single-instance handling, vault, settings, lock and screenshot policy are unchanged. This is a fix for a demonstrated startup seam; **the exact cause of Mohamed's persistent blank pixels remains unconfirmed** until the PC round. No before/after runtime timing or pixel result has been measured in this sandbox.

## Lane X1 source checks and hand-back (2026-11-09)

Source: baseline `e4df9185a99fabf6002065c84c921cd8f8278e31`; the external heartbeat made checkpoints `cff91cc3180c5311203cc41d54cd333bd4d757fb` and `0df2c5ac391baa117d49eb55e653251c8e09c893` during this run. Codex did not commit, stage, push, fetch, switch or reset Git. Checks apply to the final local source represented by the latter checkpoint plus the diagnostic change and this record. The production code in `App.tsx` was unchanged throughout the checks. No real Notaya.exe or browser was run; no install or network operation was performed.

| Check | Exact result |
|---|---|
| `cargo test --lib --offline`, Codex sandbox on Gaming-PC, `CARGO_TARGET_DIR=D:\Notaya-lanes\_target` | **FAILED**, final exit **101**: **713 passed, 1 failed, 4 ignored, 0 measured, 0 filtered**, 138.92 s. Only failure: `shell::tests::the_registry_value_can_be_written_read_and_removed`, `Windows refused to open the start-up list (code 5)`. Six compiler warnings (unused variable and dead code in unchanged files). Final run pinned `TEMP`, `TMP`, `NOTAYA_HOME`, `CIPHERLEAF_HOME` to `D:\Notaya Tests\v1-6-X1\unit-56b67ba0cd614f6db3dad34d3c24e8a5`; full output retained there as `cargo-test.log`. An earlier run used the suite's standard temp directories and had the same counts (180.67 s); the final run corrected fixture placement. |
| `npx tsc --noEmit -p tsconfig.json` | Exit **0**, no errors. |
| `npm run build -- --configLoader native` | Exit **0**; 4413 modules transformed, Vite build 2.54 s. Warning: `DatabaseView.tsx` is imported both dynamically and statically, so the dynamic import does not split that module. |
| `node --check tools/real-exe-tests/startup-minimized.mjs` and `node --check tools/real-exe-tests/startup-ready.test.mjs` | Both exit **0** on final script source. |
| `node --test --test-isolation=none tools/real-exe-tests/startup-ready.test.mjs` | **4 passed, 0 failed, 0 skipped**. Default test isolation was attempted first and failed with sandbox `spawn EPERM`; no claim of a pass for that invocation. |
| Windows PowerShell AST parser, both new `.ps1` scripts | **0 parse errors**. The diagnostic command-redaction function also passed two synthetic assertions (recognized command redacted; unknown arguments omitted), without running live diagnostic collection. |
| `git diff --check` and baseline diff whitespace check | Exit **0**; Git printed only LF/CRLF notices. |
| Real exe/CDP/reproduction, pixels, actual sign-in, protected-vault/auto-lock/tray round | **not run**; Claude runs these outside the sandbox. |
| `npm audit`, `npm ci`, `cargo audit`, `cargo clippy`, independent security review | **not run**. Audit skipped per lane instruction; no installs or dependencies. Claude owns independent `/security-review` and final security decision. |

Raw-file SHA-256 of final executable source/scripts (before any coordinator line-ending conversion):

| File | SHA-256 |
|---|---|
| `src/App.tsx` | `967226269A4035785A908A5489470ABDC513D87B1BB2C553B2CA7C07993BD622` |
| `tools/real-exe-tests/startup-minimized.mjs` | `2D5A4381F867D01F0FBA3A8E42A7F264CD1DE1F3A795AFC96A3487317C671DC9` |
| `tools/real-exe-tests/startup-ready.test.mjs` | `391586C6E1BA0199874338B2D5474164F7BE544EB138C732D5EE77A401550262` |
| `tools/real-exe-tests/run-startup-minimized.ps1` | `4BBDE8C77F03BC6D502E97D5F4449143122F4E2826708737ECAEB3E082AAAE17` |
| `tools/real-exe-tests/startup-diagnostic.ps1` | `38BB3E8DAC1D542B4EC80B78185C72F02F09FB583C448C8802734B45C3A4B1FC` |

Security self-check, guide section 4 (**self-review, not independent**):

| Checklist area | Reviewed result |
|---|---|
| Secrets/keys, zeroization, cryptography, randomness | No new key/secret handling, crypto or random production data: **n/a**. No secrets/vault fixtures in the diff. |
| Commands/IPC and confirmations | Only the existing `window_ready` call is rescheduled, after a committed status screen. Its native `tray_hidden` check and ready/flush registration are unchanged. No new Tauri command, AI authority or confirmation path. Native lock/flush behavior is **not runtime tested** here. |
| Files/paths/parsing | Runner pins child homes/profile under the allowed throwaway root, rejects reparse-point ancestors and existing app processes, validates the exact Run command before direct process creation (no shell evaluation), preserves artifacts, and never writes the Run key. Numeric CDP settings are bounded; WebSocket endpoints must remain on the expected loopback port. Diagnostic exports fixed log classes and allowlisted event fields, never raw text or exe paths. No import/archive/SQL path changed: remaining hostile-file checks **n/a**. |
| AI pipe/bridge | No changes: **n/a**. |
| Storage/history/logs | Vault format, history, decrypted caches and app logging unchanged. Diagnostic reads app-log metadata only and never opens a vault. Storage migration and history checks **n/a** to this diff. |
| Front end | CSP, HTML sinks, storage, capture protection and auto-lock code unchanged. Timer/frame callbacks are functions, once-only and canceled on effect cleanup. Test-only CDP injection and VM evaluation use fixed tool/repository code, not user content, and do not ship in the app. |
| Dependencies/build | No dependency/version/licence change; no installs. Dependency audits **not run**. Test tooling connects only to the explicitly enabled local child debug port; app networking unchanged. |
| Test/debug cleanup | Runner uses child-only environment and normal Quit, with fallback restricted to its process and recorded descendants, and requires zero remaining listeners. No real exe was run here, so this cleanup path is **produced, not runtime verified**. |

No reachable new security finding was identified in the self-check. Runtime, hostile-input review of the wider app, independent review and security acceptance remain separate.

Outside write area, mechanical: **none**. Changed files: `src/App.tsx`; `tools/real-exe-tests/{README.md,startup-minimized.md,startup-minimized.mjs,run-startup-minimized.ps1,startup-ready.test.mjs,startup-diagnostic.ps1}`; the two whole-app plan/results files. No Rust source changed. The whole-app files and helper were absent at the baseline; only the X-1 additions were created, without reconstructing the rest of the test plan. Claude owns the final records, claim/ledger/session updates, review, commit/push and reporter credit in the commit body. Token counts, actual served model/effort and quota readings were not exposed to this run and must not be guessed.

**ADDITION FOR AHMED:** startup timing correction only: if animation frames are delayed, the existing main window can become ready via a timer after its status screen has committed. Normal first-paint timing is retained; `--minimized` still stays hidden when the tray is available. No new visible feature, names, styling or startup setting. Persistent blank pixels remain an open observation question for the real-PC round.
