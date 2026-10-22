# Cipherleaf: status

This file answers two questions: what is accepted right now, and exactly where work resumes. The two parts stay separate. Update it at meaningful checkpoints, and link to other records rather than copying them.

- **As of:** 2026-10-22, about 14:30 UTC, after A9 D0 and D1 and the first launch
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
- Add `**/.devdata/**` to `server.watch.ignored` in `vite.config.ts` to stop harmless watcher errors in the dev log (not done, because a restart of Vite reloads the open dev window).

### Blockers and decisions

- **The dev app may still be running with WebView2's local debug port open** (`--remote-debugging-port=9222`, bound to 127.0.0.1) because it was restarted that way to read the first screen. Restart it without the port (`npm run tauri dev` in a fresh shell, with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` unset) before putting any real data in it. Its dev vault is in the ignored `.devdata` folder.
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
