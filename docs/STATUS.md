# Cipherleaf: status

This file answers two questions: what is accepted right now, and exactly where work resumes. The two parts stay separate. Update it at meaningful checkpoints, and link to other records rather than copying them.

- **As of:** 2026-10-22, 16:10 UTC, after the owner's first hands-on run and a restart of the dev app without the debug port
- **Owner and decision-maker:** Mohamed Sayed
- **Maintained by:** Claude (Claude Code), for the owner

## Current state (accepted present)

- **Mission state:** **awaiting the owner's hands-on review.** A9 D0 and D1 are done and the dev app launches; work stops here until the owner has tried it. Work happens on branch `feat/a9-keepassxc`.
- **Accepted baseline:** none. The owner hasn't accepted any result yet.
- **Active authority:** the approved plan ([snapshot](plan/approved-plan-2026-10-22.md)) with amendment A9 approved at 13:07:49 UTC, as listed in [approval-and-amendments.md](plan/approval-and-amendments.md).
- **Waiting on the owner:** the hands-on review; decisions on A2 (single-record rollback) and A8 (version pinning); the GPL approach (A9 Q5) before any distribution.

## Continuation (unaccepted work and how to resume)

### What exists

| Item | Where | State | Evidence and its limits |
|---|---|---|---|
| Toolchain | Node 24.20.0, npm 11.19.0, Rust 1.99.1 (MSVC), Git 2.55.0, VS Build Tools 2022 | Complete | `--version` checks, 2026-10-22 |
| Rust backend | `src-tauri/src/` | Implemented; unit-tested | `cargo test --lib`: 27/27 on commit `2743021` (2026-10-22). Unit level only. |
| UI | `src/` | Implemented; type-checked, bundled, launches | `tsc`: 0 errors; `npm run build`: passed; `npm run tauri dev`: the first screen ("Create your vault") renders. Nothing beyond the first screen has been exercised. |
| KeePassXC bundle (A9) | `vendor/` (ignored by Git), `vendor/keepassxc-manifest.md` (committed) | D0 and D1 done | SHA-256 and PGP verified; offline launch test passed; see [`execution/2026-10-22-a9-keepassxc-d0-d1.md`](execution/2026-10-22-a9-keepassxc-d0-d1.md). After a fresh clone the binaries must be downloaded again. |
| Records | `docs/`, `CLAUDE.md` | Current | — |
| Git | Local repository, no remote; branch `feat/a9-keepassxc`; `main` untouched | Commits `f26c1f7`, `0d57ad2`, `b7c8626`, `2743021`, plus the documentation commit that follows | `git log` |
| README | `README.md` | Stale: still the Tauri template text | — |

### Not done

- Everything past the first screen of the running app: creating and unlocking a vault, notes, sticky notes, the KeePassXC panel, auto-lock, screenshot blocking.
- Portable release build with KeePassXC copied in, plus KeePassXC's license files and source link.
- `cargo audit`, `npm audit` and a security review.
- A1 and A3–A7 evidence; README rewrite.
- (Done 16:09 UTC) `**/.devdata/**` is now in `server.watch.ignored` in `vite.config.ts`; the dev log showed 0 watcher errors afterwards. Note: editing `vite.config.ts` while `tauri dev` runs ends the whole dev session, so restart it after such an edit.

### Observed in the owner's first hands-on run (from files only, ~16:03–16:07 UTC)

The owner ran the app and created things before the restart. Claude didn't watch the run; this comes from file evidence in `.devdata\data\`:

- `keepassxc\keepassxc.ini` (16:03) and `Passwords.kdbx` (16:04, valid KeePass signature `03 D9 A2 9A`): the KeePassXC panel started the bundled KeePassXC, which created a database in Cipherleaf's data folder.
- `vault.clf` (16:05, a valid SQLite file) and `backups\vault-….clf` (16:06): a Cipherleaf vault was created, and a lock wrote its encrypted backup.
- The owner has not yet reported what looked right or wrong.

### Blockers and decisions

- **Debug-port exposure (closed at 16:07 UTC).** From 15:58:50 until the restart at 16:06:58 UTC (about 8 minutes) the dev app ran with WebView2's local debug port open (127.0.0.1:9222, any origin). The vault was created during that window. Claude connected only at about 15:59 UTC (two read-only scripts: one read the first screen, one read an input's length), and never afterwards. Whether another local program connected is unknown. The dev app now runs without the port (checked: nothing listening on 9222, no WebView2 process with remote debugging). If that vault will hold real data, change its master password (Settings → Master password & keyfile) or create a fresh vault.
- A2 and A8 need the owner's decision. A1 and A3–A7 need evidence. See [the amendments record](plan/approval-and-amendments.md).

### Do not repeat

- The toolchain install, scaffolding, dependency selection, or the D0 download and verification.
- Rewriting backend or UI modules from scratch. Inspect and fix instead.
- Editing the plan snapshot. Record changes as amendments instead.
- Driving the running dev window by script while the owner may be using it.

### Exact next permitted action

1. The owner tries the dev app (restart it first without the debug port, or keep using it knowing the port is open) and reports what works and what doesn't.
2. Fix the defects found, re-run `cargo test --lib`, `npx tsc --noEmit -p tsconfig.json` and `npm run build`, and add a new Execution Summary.
3. Only after the owner's go-ahead: the portable release build (copy KeePassXC in unmodified with its license files and source link), `cargo audit`, `npm audit`, and a security review.

## Where things live

| Question | Where the answer lives |
|---|---|
| What is authorized now? | The owner's latest instruction, recorded in [approval-and-amendments.md](plan/approval-and-amendments.md) |
| What was approved, and its success criteria? | [approved-plan-2026-10-22.md](plan/approved-plan-2026-10-22.md), the unchanged snapshot |
| What's accepted, and where does work resume? | This file |
| What does the code actually do? | The working tree and its Git history |
| Did checks pass? | Re-running the checks; dated results in [`execution/`](execution/) |
| What does Cipherleaf promise about security? | [security-design.md](security-design.md) |
| How should an AI agent work here? | [`CLAUDE.md`](../CLAUDE.md) |
| Why did choices change? | The amendments record, and the Claude Code conversation transcripts (stored outside the project) |

**Reading order to resume:** `CLAUDE.md` → this file → the amendments record → the live repository (`git status`, `git log`) → the latest Execution Summary → the security design or plan snapshot, as needed.
