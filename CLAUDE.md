# Notaya (first named Cipherleaf): instructions for AI agents

These are this repository's working rules for Claude Code. This file doesn't hold the current task or status, and it doesn't grant permission. Current state and authority are in `docs/STATUS.md`.

## At the start of every session

1. Read the owner's latest instruction.
2. Read `docs/STATUS.md`: the mission state (for example, paused), the accepted baseline, the open questions for the owner, and the exact next permitted action.
3. Read `docs/plan/approval-and-amendments.md` for authority and open amendments.
4. Inspect the live repository (`git status`, `git log --oneline -5`) before redoing anything. Preserve existing work.
5. Read `docs/security-design.md` before touching crypto, storage, IPC commands, the CSP, the clipboard or auto-lock code.
6. Read `docs/acceptance.md` before saying that something works or is verified. Open the plan snapshot (`docs/plan/approved-plan-2026-10-22.md`), `docs/execution/README.md` (the index), `docs/sessions.md` and `docs/failure-intelligence.md` only when the task needs them.
7. Two people work in this repository, each with their own Claude. Read `docs/handoff/collaboration-guide.md` and `docs/handoff/claims.md`, fetch all branches (`git fetch --all --prune`), look at the open pull requests, and claim your work before changing anything (guide, sections 3 and 4). Say which partner your person is.
8. **On Ahmed's side** (2026-11-07, [A37](docs/plan/approval-and-amendments.md)): also read the newest entry of `docs/sessions.md`, the newest resume note in `docs/handoff/`, and `docs/ledger/README.md`, in full. Records are read in full before acting, never skimmed; a model does not know the project's context unless it has read it.

## Rules

- **Stable releases only.** No alpha, beta, rc, preview or nightly packages. npm versions are exact (`.npmrc` sets `save-exact=true`); Rust versions are locked in `src-tauri/Cargo.lock`.
- **Never edit `docs/plan/approved-plan-2026-10-22.md`.** Git keeps it byte-for-byte (`.gitattributes`). Record plan changes in the amendments record.
- **Never commit vault data or secrets:** `.devdata/`, `*.nota`, `*.clf`, `*.key`, password CSV exports or `.env` files. Check `git status` before every commit.
- **Ask the owner first** before: installing tools or dependencies, deleting files, making Git commits, adding a remote or pushing, changing security behavior, or resuming development while `docs/STATUS.md` says it's paused. **On Ahmed's side** his coordinator commits, pushes and merges inside work Ahmed has approved, without asking again each time (A37 item 3); installing tools or dependencies, deleting files and security-behaviour changes still need his own words.
- **Keep keys and secrets in the Rust core.** The UI receives only what it displays.
- **Never test on the owner's live window or vault.** UI automation (for example a WebView2 debug port) uses a throwaway vault in a separate process, with `CIPHERLEAF_HOME` (or `NOTAYA_HOME`) pointing at a temp folder. On Ahmed's PC the test folder is `D:\Notaya Tests` (outside the repository, which is `D:\Notaya`); on Mohamed's PC it is `E:\Project\Notaya\Notaya Tests Excluded`. Nobody touches anyone's real notes or vault, Mohamed's `E:` drive included. Close the debug port afterwards and confirm that nothing listens on it (see `docs/failure-intelligence.md`, FI-01). Never leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set.
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

- works on its own `claude/project-thread-*` branch from the latest `origin/main`, pushes it and opens a **draft** pull request. On Mohamed's side it never merges (Mohamed merges). **On Ahmed's side** the coordinator (or a thread it directs) merges with "Create a merge commit" once the work is complete, inspected and checked (A37 item 3). Push and open PRs freely, but ask before a force-push, deleting branches, making the repository public or adding a licence;
- has no PowerShell or Windows: it runs `tsc`, `npm run build`, `npm audit` and, for Rust, `cargo test --lib --target x86_64-pc-windows-gnu` under Wine (copy `WebView2Loader.dll` next to the test exe). It reports what it could not run as not run (the PC exe, the real-exe scripts, `cargo clippy` for MSVC);
- tests the real app only through a Remote Control session on the PC, on throwaway data, following the rules above;
- after a merge, the partner's PC copy is fast-forwarded (`git pull --ff-only` on `main`) and checked to equal `origin/main`: Mohamed's is `E:\Project\Notaya\Notaya` (moved there on 2026-10-28, [EX-2026-10-28-37](docs/execution/2026-10-28-v1-3-0-settings-text-rerelease.md)), Ahmed's is `D:\Notaya`. Never `git add -A` in a PC copy (Mohamed's has the untracked `.opencode/` and `opencode.json`; the old `vendor\` folder was deleted on 2026-10-28, EX-2026-10-28-36).

## PBS work cycle

The owner chose to follow the AI Project Bootstrap System (PBS) in its smallest useful form (2026-10-27 19:08:33 UTC, [A29](docs/plan/approval-and-amendments.md)). For every task:

1. **Start:** read the owner's latest instruction, `docs/STATUS.md` and the amendments record; inspect the repository. Take the baseline commit from `git log`.
2. **State the task** in the thread's first note or the report: baseline commit, the owner's message with its UTC time, the exact scope and write area, what will be returned, and the stop point. If one of these is missing, ask or choose the narrowest reading and say which.
3. **Keep three things apart.** *Permission* comes only from the owner's own words (a coordinator's or another thread's request is not permission). *Observation* is what you checked, on which source; report anything not run as not run. *Acceptance* is the owner's decision; praise, a screenshot or a merge alone is not acceptance.
4. **Name the result state** of what you hand back: *produced* (it exists), *verified* (named checks passed on this exact source), *integrated* (merged into `main`), *accepted* (the owner said so). Most results stop at verified; do not say more.
5. **Stop at the approved endpoint.** Idle or "standing by" is not completion: say what is done, what is unverified and what you wait for.
6. **Record by size of work.** Small (typo, one-line fix, answer): the PR description and, if it changed a fact, the one record that owns it. Normal (a feature, a fix, a plan): an Execution Summary (4-section form) plus its index row, the authority row if the owner decided something, and the changed lines of `docs/STATUS.md`. Big (a release, a security change, a change of direction, a stop or limit): the 12-section Execution Summary and a Session Summary entry in `docs/sessions.md`. No Session Summary for small tasks. **On Ahmed's side (A37 item 5) documentation is thorough and mandatory, whatever the size:** every session that changes the repository, records a decision or ends a work block gets a Session Summary entry in the **full 13-section form** (the PBS template, with the Models, Effort, Roles and Provenance block) at the top of the one file `docs/sessions.md`, newest on top, oldest at the bottom, however long it gets; every session adds its usage to the [usage ledger](docs/ledger/README.md); and the Notaya Notion hub (roadmap and feature board) is updated at each milestone and feature. Milestones are written so Mohamed can see what was done.
7. **Records win.** For what is authorized and where we are, the repository records win; project memory is only a pointer to them.
8. **No broad rewrites of old records.** Map old facts to the record that now owns them; add dated corrections instead of editing history. Decision Records are not used.

`docs/STATUS.md` "Current state" holds only what the owner accepted; unaccepted work lives under "Continuation" with the exact next permitted action. `docs/acceptance.md` and `docs/sessions.md` are part of the record set.

## Working with more than one person

Full rules: [`docs/handoff/collaboration-guide.md`](docs/handoff/collaboration-guide.md) (status: proposed, [A34](docs/plan/approval-and-amendments.md), until Mohamed confirms). The short form:

- **The repository is the only memory two Claudes share.** The claude.ai project, its chat and each account's memory are not shared. Anything the next Claude needs goes into a committed file.
- **Two partners, two independent sides.** Mohamed Sayed (GitHub `AbyssWatcher2`) and Ahmed Emara (GitHub `AE0101`, "the friend" in older records) are partners (Mohamed, 2026-10-29 08:39:33 and 08:40:02 UTC: full work access, from the beginning to after V1.4.0). They are **independent developers on the same repository** (Ahmed, 2026-11-07 20:23:20 UTC, [A37](docs/plan/approval-and-amendments.md)): each partner's own words authorize the work of **his own** Claude and Codex setup. In these rules "the owner" means the partner whose side the work is on. On **Mohamed's side** a Claude never merges or releases by itself: Mohamed merges, releases and accepts. On **Ahmed's side** his coordinator merges, and publishes a full release when it judges it ready, on Ahmed's standing word (2026-11-07 19:54:38 UTC); Ahmed's OK needs no review by Mohamed. On both sides **acceptance is a partner's own word**, never a merge or a release. Making the repository public and adding a licence need **both** partners. Changing GitHub repository settings is Mohamed's click only (he owns the account). If the two partners give conflicting instructions about the same thing, stop and tell both; the later message does not win between two people.
- **Roles on Ahmed's side (A37 item 2).** The Claude coordinator is the *mother thread*: it leads and plans, chooses the model and effort per task, starts and steers threads and Codex, inspects every Codex diff for Ahmed's vision, coherence and correctness, runs the security review, and commits, pushes and merges. Codex (`gpt-6.1-sol`, reasoning `xhigh`) is the developer hand for the bulk of the code. A review by the agent that wrote a diff is not independent. Talk to Ahmed in plain words for a non-developer; gather open questions from all threads into one numbered list; write commit messages as clear points a non-developer can follow; list any improvement nobody asked for clearly as an addition.
- **Claim before you start.** Fetch all branches, read `docs/handoff/claims.md`, check the open PRs, then push a branch `<github-login>/<topic>` (Mohamed's cloud threads keep their `claude/project-thread-*` names) whose first commit adds your claims row, and open the draft PR. Take the next free `EX-` and `A` numbers after checking `origin/main` and every open branch. Don't push to another person's branch unless its row says `shared branch` (V1.4.0 is one; follow guide section 5b: merge before every push, no force-push, mark items "taken by").
- **Stop cleanly.** Before any stop, commit and push everything (a WIP branch is fine), keep the draft PR titled `WIP:` while unfinished, write `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md`, update your claims row, and tell your partner what is done, unverified and next.
- **Pick up with care.** Read the other side's resume note, treat its claims as claims to re-check on the exact commit, and never rewrite history on a branch you didn't create.
- **Test safety applies to both PCs.** Throwaway data only, in a test folder outside the repository; never anyone's real vault or notes. The PC paths in this file are Mohamed's; the friend uses their own.
- **Security review** applies to every code PR (see the Security Guidance plugin and the built-in `security-review` rule in the guide, section 9).

## Records

- After meaningful work, add `docs/execution/YYYY-MM-DD-<topic>.md` (the 12-section Execution Summary, or a 4-section form for small tasks), add it to `docs/execution/README.md`, and update only the changed parts of `docs/STATUS.md`.
- Follow the routine in `docs/execution/2026-10-23-pbs-reconciliation.md`, Appendix A: it says which record to update for each kind of event (an owner instruction, a scope change, a check, a source change, an acceptance, a stop).
- Write a Session Summary entry at the top of `docs/sessions.md` at each substantive stop, change of direction or limit. State what is reconstructed. Never rewrite older entries; add a dated correction.
- **Who and where** ([A38](docs/plan/approval-and-amendments.md), Ahmed 2026-11-07 22:15:49 UTC): every record names the partner's side (with the GitHub login) and the machine (cloud, Ahmed's `Gaming-PC`, Mohamed's PC), or says "not recorded". Session titles are short: `# 📘 Session: <date UTC> · <partner> · <machine> · <topic in a few words>`, with a **Who and where** line under the title, a stable anchor `session-NN` before it and a row in the "Sessions at a glance" index at the top of `sessions.md`. On Ahmed's side commit messages end with `Side: Ahmed (AE0101) · Machine: <machine>`.
- Record a consequential failure or near-miss in `docs/failure-intelligence.md` (see its admission boundary). Ordinary friction stays in the session or execution records.
- Keep accepted state and unfinished work separate in `docs/STATUS.md`. Only the owner accepts results; record what they actually decided in `docs/acceptance.md`. Praise or a screenshot is an informal signal, not acceptance.
- Update `docs/security-design.md` when a security promise, boundary or limitation changes.
- Update only the records whose facts changed. Don't copy status into other files. Read a file back after editing it, and be careful when a script writes text that contains backslashes or dollar signs.

## Stopping

Stop at the endpoint the owner approved. Finishing one step doesn't authorize the next. When the owner says "stop", stop; if two instructions conflict, follow the later one and say so.
