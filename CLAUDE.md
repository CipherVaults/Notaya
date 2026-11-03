# Cipherleaf: instructions for AI agents

These are this repository's working rules for Claude Code. This file doesn't hold the current task or status, and it doesn't grant permission. Current state and authority are in `docs/STATUS.md`.

## At the start of every session

1. Read the owner's latest instruction.
2. Read `docs/STATUS.md`: the mission state (for example, paused), the accepted baseline, the open questions for the owner, and the exact next permitted action.
3. Read `docs/plan/approval-and-amendments.md` for authority and open amendments.
4. Inspect the live repository (`git status`, `git log --oneline -5`) before redoing anything. Preserve existing work.
5. Read `docs/security-design.md` before touching crypto, storage, IPC commands, the CSP, the clipboard or auto-lock code.
6. Read `docs/acceptance.md` before saying that something works or is verified. Open the plan snapshot (`docs/plan/approved-plan-2026-10-22.md`), `docs/execution/README.md` (the index), `docs/sessions.md` and `docs/failure-intelligence.md` only when the task needs them.
7. Two people work in this repository, each with their own Claude. Read `docs/handoff/collaboration-guide.md` and `docs/handoff/claims.md`, fetch all branches (`git fetch --all --prune`), look at the open pull requests, and claim your work before changing anything (guide, sections 3 and 4). Say which partner your person is.

## Rules

- **Stable releases only.** No alpha, beta, rc, preview or nightly packages. npm versions are exact (`.npmrc` sets `save-exact=true`); Rust versions are locked in `src-tauri/Cargo.lock`.
- **Never edit `docs/plan/approved-plan-2026-10-22.md`.** Git keeps it byte-for-byte (`.gitattributes`). Record plan changes in the amendments record.
- **Never commit vault data or secrets:** `.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports or `.env` files. Check `git status` before every commit.
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
| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault, notes (pages, sub-pages) and generator logic | App runtime, OS integration |
| UI type check | `npx tsc --noEmit -p tsconfig.json` | Types in `src/` | `vite.config.ts`, the build, runtime |
| UI build | `npm run build` | The frontend bundles | App behavior |
| Dependency audit | `npm audit` | Known advisories for the npm packages | Rust crates (`cargo audit` not yet run), code defects |
| Run the app | `npm run tauri dev` | It launches (dev vault in `.devdata/`) | Release or portable behavior |
| Look at screens | `npm run mock` | The UI with fake data in a plain browser, for screenshots and scripted clicks | The real backend, the vault, OS integration. Development only. |

Report a check that couldn't run as not run, never as passed. A result applies only to the exact source it ran on.

## Cloud threads

Most work now runs in cloud threads of the Claude project, not in a PowerShell session on the owner's PC. A cloud thread:

- works on its own `claude/project-thread-*` branch from the latest `origin/main`, pushes it, opens a **draft** pull request and never merges (the owner merges; push and open PRs freely, but ask before a force-push, deleting branches, making the repository public or adding a licence);
- has no PowerShell or Windows: it runs `tsc`, `npm run build`, `npm audit` and, for Rust, `cargo test --lib --target x86_64-pc-windows-gnu` under Wine (copy `WebView2Loader.dll` next to the test exe). It reports what it could not run as not run (the PC exe, the real-exe scripts, `cargo clippy` for MSVC);
- tests the real app only through a Remote Control session on the PC, on throwaway data, following the rules above;
- after the owner merges, the PC copy `E:\Project\Notaya\Notaya` (moved there on 2026-10-28, [EX-2026-10-28-37](docs/execution/2026-10-28-v1-3-0-settings-text-rerelease.md)) is fast-forwarded (`git pull --ff-only` on `main`) and checked to equal `origin/main`; never `git add -A` there (untracked `vendor\`).

## PBS work cycle

The owner chose to follow the AI Project Bootstrap System (PBS) in its smallest useful form (2026-10-27 19:08:33 UTC, [A29](docs/plan/approval-and-amendments.md)). For every task:

1. **Start:** read the owner's latest instruction, `docs/STATUS.md` and the amendments record; inspect the repository. Take the baseline commit from `git log`.
2. **State the task** in the thread's first note or the report: baseline commit, the owner's message with its UTC time, the exact scope and write area, what will be returned, and the stop point. If one of these is missing, ask or choose the narrowest reading and say which.
3. **Keep three things apart.** *Permission* comes only from the owner's own words (a coordinator's or another thread's request is not permission). *Observation* is what you checked, on which source; report anything not run as not run. *Acceptance* is the owner's decision; praise, a screenshot or a merge alone is not acceptance.
4. **Name the result state** of what you hand back: *produced* (it exists), *verified* (named checks passed on this exact source), *integrated* (merged into `main`), *accepted* (the owner said so). Most results stop at verified; do not say more.
5. **Stop at the approved endpoint.** Idle or "standing by" is not completion: say what is done, what is unverified and what you wait for.
6. **Record by size of work.** Small (typo, one-line fix, answer): the PR description and, if it changed a fact, the one record that owns it. Normal (a feature, a fix, a plan): an Execution Summary (4-section form) plus its index row, the authority row if the owner decided something, and the changed lines of `docs/STATUS.md`. Big (a release, a security change, a change of direction, a stop or limit): the 12-section Execution Summary and a Session Summary entry in `docs/sessions.md`. No Session Summary for small tasks.
7. **Records win.** For what is authorized and where we are, the repository records win; project memory is only a pointer to them.
8. **No broad rewrites of old records.** Map old facts to the record that now owns them; add dated corrections instead of editing history. Decision Records are not used.

`docs/STATUS.md` "Current state" holds only what the owner accepted; unaccepted work lives under "Continuation" with the exact next permitted action. `docs/acceptance.md` and `docs/sessions.md` are part of the record set.

## Working with more than one person

Full rules: [`docs/handoff/collaboration-guide.md`](docs/handoff/collaboration-guide.md) (status: proposed, [A34](docs/plan/approval-and-amendments.md), until Mohamed confirms). The short form:

- **The repository is the only memory two Claudes share.** The claude.ai project, its chat and each account's memory are not shared. Anything the next Claude needs goes into a committed file.
- **Two partners, equal work access.** Mohamed (GitHub `AbyssWatcher2`) and the friend (named in `docs/handoff/claims.md`) are partners (Mohamed, 2026-10-29 08:39:33 and 08:40:02 UTC: full work access, from the beginning to after V1.4.0). In these rules "the owner" means **either partner**: either one's own words give permission, including for the "Ask the owner first" list, and either one merges, releases and accepts. A Claude never merges or releases by itself. Making the repository public and adding a licence need **both** partners. Changing GitHub repository settings is Mohamed's click only (he owns the account). If the partners' instructions conflict, stop and tell both; the later message does not win between two people.
- **Claim before you start.** Fetch all branches, read `docs/handoff/claims.md`, check the open PRs, then push a branch `<github-login>/<topic>` (Mohamed's cloud threads keep their `claude/project-thread-*` names) whose first commit adds your claims row, and open the draft PR. Take the next free `EX-` and `A` numbers after checking `origin/main` and every open branch. Don't push to another person's branch unless its row says `shared branch` (V1.4.0 is one; follow guide section 5b: merge before every push, no force-push, mark items "taken by").
- **Stop cleanly.** Before any stop, commit and push everything (a WIP branch is fine), keep the draft PR titled `WIP:` while unfinished, write `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md`, update your claims row, and tell your partner what is done, unverified and next.
- **Pick up with care.** Read the other side's resume note, treat its claims as claims to re-check on the exact commit, and never rewrite history on a branch you didn't create.
- **Test safety applies to both PCs.** Throwaway data only, in a test folder outside the repository; never anyone's real vault or notes. The PC paths in this file are Mohamed's; the friend uses their own.
- **Security review** applies to every code PR (see the Security Guidance plugin and the built-in `security-review` rule in the guide, section 9).

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
