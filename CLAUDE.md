# Cipherleaf: instructions for AI agents

These are this repository's working rules for Claude Code. This file doesn't hold the current task or status, and it doesn't grant permission. Current state and authority are in `docs/STATUS.md`.

## At the start of every session

1. Read the owner's latest instruction.
2. Read `docs/STATUS.md`: the mission state (for example, paused), the accepted baseline, and the exact next permitted action.
3. Read `docs/plan/approval-and-amendments.md` for authority and open amendments.
4. Inspect the live repository (`git status`, `git log --oneline -5`) before redoing anything. Preserve existing work.
5. Read `docs/security-design.md` before touching crypto, storage, IPC commands, the CSP, the clipboard or auto-lock code.
6. Open the plan snapshot (`docs/plan/approved-plan-2026-10-22.md`) and the latest file in `docs/execution/` only when the task needs them.

## Rules

- **Stable releases only.** No alpha, beta, rc, preview or nightly packages. npm versions are exact (`.npmrc` sets `save-exact=true`); Rust versions are locked in `src-tauri/Cargo.lock`.
- **Never edit `docs/plan/approved-plan-2026-10-22.md`.** Git keeps it byte-for-byte (`.gitattributes`). Record plan changes in the amendments record.
- **Never commit vault data or secrets:** `.devdata/`, `*.clf`, `*.key`, password CSV exports or `.env` files. Check `git status` before every commit.
- **Ask the owner first** before: installing tools or dependencies, deleting files, making Git commits, adding a remote or pushing, changing security behavior, or resuming development while `docs/STATUS.md` says it's paused.
- **Keep keys and secrets in the Rust core.** The UI receives only what it displays.

## Verification commands

Run from the project root in PowerShell. In a new shell, refresh PATH first:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
```

| Check | Command | Shows | Doesn't show |
|---|---|---|---|
| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault and generator logic | App runtime, OS integration |
| UI type check | `npx tsc --noEmit -p tsconfig.json` | Types in `src/` | `vite.config.ts`, the build, runtime |
| UI build | `npm run build` | The frontend bundles | App behavior |
| Run the app | `npm run tauri dev` | It launches (dev vault in `.devdata/`) | Release or portable behavior |

Report a check that couldn't run as not run, never as passed. A result applies only to the exact source it ran on.

## Records

- After meaningful work, add `docs/execution/YYYY-MM-DD-<topic>.md` (the 12-section Execution Summary, or a 4-section form for small tasks), and update only the changed parts of `docs/STATUS.md`.
- Keep accepted state and unfinished work separate in `docs/STATUS.md`. Only the owner accepts results.
- Update `docs/security-design.md` when a security promise, boundary or limitation changes.
- Update only the records whose facts changed. Don't copy status into other files.

## Stopping

Stop at the endpoint the owner approved. Finishing one step doesn't authorize the next.
