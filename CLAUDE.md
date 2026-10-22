# Cipherleaf: instructions for AI agents

These are this repository's working rules for Claude Code. This file doesn't hold the current task or status, and it doesn't grant permission. Current state and authority are in `docs/STATUS.md`.

## At the start of every session

1. Read the owner's latest instruction.
2. Read `docs/STATUS.md`: the mission state (for example, paused), the accepted baseline, the open questions for the owner, and the exact next permitted action.
3. Read `docs/plan/approval-and-amendments.md` for authority and open amendments.
4. Inspect the live repository (`git status`, `git log --oneline -5`) before redoing anything. Preserve existing work.
5. Read `docs/security-design.md` before touching crypto, storage, IPC commands, the CSP, the clipboard or auto-lock code.
6. Read `docs/acceptance.md` before saying that something works or is verified. Open the plan snapshot (`docs/plan/approved-plan-2026-10-22.md`), `docs/execution/README.md` (the index), `docs/sessions.md` and `docs/failure-intelligence.md` only when the task needs them.

## Rules

- **Stable releases only.** No alpha, beta, rc, preview or nightly packages. npm versions are exact (`.npmrc` sets `save-exact=true`); Rust versions are locked in `src-tauri/Cargo.lock`.
- **Never edit `docs/plan/approved-plan-2026-10-22.md`.** Git keeps it byte-for-byte (`.gitattributes`). Record plan changes in the amendments record.
- **Never commit vault data or secrets:** `.devdata/`, `*.clf`, `*.key`, password CSV exports or `.env` files. Check `git status` before every commit.
- **Ask the owner first** before: installing tools or dependencies, deleting files, making Git commits, adding a remote or pushing, changing security behavior, or resuming development while `docs/STATUS.md` says it's paused.
- **Keep keys and secrets in the Rust core.** The UI receives only what it displays.
- **Never test on the owner's live window or vault.** UI automation (for example a WebView2 debug port) uses a throwaway vault in a separate process, with `CIPHERLEAF_HOME` pointing at a temp folder. Close the debug port afterwards and confirm that nothing listens on it (see `docs/failure-intelligence.md`, FI-01). Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set.
- **Quote times, don't estimate them.** Take a time from the session transcript (the owner's message timestamps) or the system clock (UTC), and say which. If neither is available, write "not recorded".
- **Don't force-stop KeePassXC** (it protects its own process). Close it politely or answer its dialog.

## Verification commands

Run from the project root in PowerShell. In a new shell, refresh PATH first:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
```

| Check | Command | Shows | Doesn't show |
|---|---|---|---|
| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault, notes (pages, sub-pages), KeePassXC and generator logic | App runtime, OS integration |
| UI type check | `npx tsc --noEmit -p tsconfig.json` | Types in `src/` | `vite.config.ts`, the build, runtime |
| UI build | `npm run build` | The frontend bundles | App behavior |
| Dependency audit | `npm audit` | Known advisories for the npm packages | Rust crates (`cargo audit` not yet run), code defects |
| Run the app | `npm run tauri dev` | It launches (dev vault in `.devdata/`) | Release or portable behavior |
| Look at screens | `npm run mock` | The UI with fake data in a plain browser, for screenshots and scripted clicks | The real backend, the vault, OS integration. Development only. |

Report a check that couldn't run as not run, never as passed. A result applies only to the exact source it ran on.

## Records

- After meaningful work, add `docs/execution/YYYY-MM-DD-<topic>.md` (the 12-section Execution Summary, or a 4-section form for small tasks), add it to `docs/execution/README.md`, and update only the changed parts of `docs/STATUS.md`.
- Follow the routine in `docs/execution/2026-10-23-pbs-reconciliation.md`, Appendix A: it says which record to update for each kind of event (an owner instruction, a scope change, a check, a source change, an acceptance, a stop).
- Write a Session Summary entry at the top of `docs/sessions.md` at each substantive stop, change of direction or limit. State what is reconstructed. Never rewrite older entries; add a dated correction.
- Record a consequential failure or near-miss in `docs/failure-intelligence.md` (see its admission boundary). Ordinary friction stays in the session or execution records.
- Keep accepted state and unfinished work separate in `docs/STATUS.md`. Only the owner accepts results; record what they actually decided in `docs/acceptance.md`. Praise or a screenshot is an informal signal, not acceptance.
- Update `docs/security-design.md` when a security promise, boundary or limitation changes.
- Update only the records whose facts changed. Don't copy status into other files. Read a file back after editing it, and be careful when a script writes text that contains backslashes or dollar signs.

## Stopping

Stop at the endpoint the owner approved. Finishing one step doesn't authorize the next. When the owner says "stop", stop; if two instructions conflict, follow the later one and say so.
