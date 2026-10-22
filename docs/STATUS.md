# Cipherleaf: status

This file answers two questions: what is accepted right now, and exactly where work resumes. The two parts stay separate. Update it at meaningful checkpoints, and link to other records rather than copying them.

- **As of:** 2026-10-22, 13:10 UTC, after writing up amendment A9
- **Owner and decision-maker:** Mohamed Sayed
- **Maintained by:** Claude (Claude Code), for the owner

## Current state (accepted present)

- **Mission state:** **paused at a checkpoint.** The owner asked to stop because the usage limit is near and will continue after it resets. A9 is still approved and development is authorized; resume with the next steps below. Work happens on branch `feat/a9-keepassxc`.
- **Accepted baseline:** none. The owner hasn't accepted any result yet.
- **Active authority:** the approved plan ([snapshot](plan/approved-plan-2026-10-22.md)), narrowed by the later instructions listed in [approval-and-amendments.md](plan/approval-and-amendments.md).
- **Waiting on the owner:** decisions on amendments A2 (single-record rollback) and A8 (version pinning).

## Continuation (unaccepted work and how to resume)

### What exists

| Item | Where | State | Evidence and its limits |
|---|---|---|---|
| Toolchain | Node 24.20.0 and npm 11.19.0; Rust 1.99.1 (stable, MSVC); Git 2.55.0; VS Build Tools 2022 with Windows SDK 10.0.26100 | Complete | `--version` checks on 2026-10-22, shortly after 11:08 UTC |
| Project scaffold | Tauri 2, React and TypeScript with Vite (`create-tauri-app` 4.7.5) | Complete | — |
| Rust backend | `src-tauri/src/` (20 files) | Implemented; current state unverified | `cargo test --lib`: 27 of 27 passed at 11:37:17 UTC. Since then, one unused import was removed from `commands/vault.rs` (11:37:47 UTC); file timestamps show no other Rust change. Not re-run. |
| UI | `src/` (29 files) | Implemented; type-checked only | `npx tsc --noEmit -p tsconfig.json`: 0 errors at about 11:51 UTC, and `src/` hasn't changed since. It doesn't cover `vite.config.ts`. Never built or run. |
| App configuration | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` | Implemented; unverified at runtime | Compiled by the 11:37 test build only |
| Dependencies | `package.json` (exact versions), `src-tauri/Cargo.toml` with `Cargo.lock` | Selected and locked; see A8 | npm reported 0 vulnerabilities for the first installs; the final set wasn't audited; `cargo audit` not run |
| PBS records | `docs/`, `CLAUDE.md` | Written 2026-10-22 | — |
| Git | Local repository, no remote; branch `feat/a9-keepassxc` (`main` untouched) | First commit made after the owner's approval | `git log` |
| README | `README.md` | Stale: still the Tauri template text | Not part of this adoption |

### Not started

- Production build (`npm run build`).
- First launch (`npm run tauri dev`) and any runtime checks.
- Portable release build.
- `cargo audit`, `npm audit` and a security review.
- README rewrite.

### Blockers and decisions

- A2 and A8 need the owner's decision. A1 and A3–A7 need evidence. See [the amendments record](plan/approval-and-amendments.md).
- A9 is approved. If D0 shows a problem (failed verification, a missing runtime), stop and ask before D1.

### Do not repeat

- Installing the toolchain; it's installed and checked.
- Scaffolding the project or re-selecting dependencies.
- Rewriting backend or UI modules from scratch. Inspect and fix instead.
- Editing the plan snapshot. Record changes as amendments instead.

### Exact next permitted action

**Checkpoint 2026-10-22 (~13:45 UTC): paused for the owner's usage limit, mid-D1.**

Done since the baseline:

- **Checks re-run** on commit `f26c1f7`: 27/27 Rust tests passed and the type check was clean (13:11–13:13 UTC).
- **D0 complete** (commit `0d57ad2`): KeePassXC 2.8.0 downloaded and verified. Evidence is in `vendor/keepassxc-manifest.md`. The binaries are in the ignored `vendor/` folder, so after a fresh clone they must be downloaded again.
- **D1 backend done** (this checkpoint commit):
  - added `keepassxc.rs`, which verifies the exe hash, launches KeePassXC with its config kept in `data/keepassxc/`, and locks it with `--lock`;
  - added `commands/keepassxc.rs`;
  - removed `logins.rs`, `clipboard.rs`, the login model and two settings (`clipboardClearSeconds`, `breachCheckEnabled`);
  - `open_url` moved to `tools.rs`;
  - removed the `sha1`, `csv`, `totp-rs` and `reqwest` dependencies and three `windows` features.
  - Rust test result for this state: see the commit message.

**Next, in order:**

1. **D1 UI.** Until this is done the frontend doesn't compile against the new backend.
   - Remove `src/features/passwords/*` and `src/features/generator/*`.
   - Add `src/features/passwords/KeePassXCPanel.tsx`, which calls `keepassxc_status`, `keepassxc_open` and `keepassxc_lock`.
   - Update `Shell.tsx`: remove the Generator item and renumber the shortcuts.
   - Update `store/app.ts`: remove logins and the `new-login` intent.
   - Update `CommandPalette.tsx`: remove the login actions and the login hits.
   - Update `lib/ipc.ts` and `lib/types.ts`: remove the login, health, TOTP and password-generation APIs and the two settings; add the KeePassXC status type.
   - Update `SettingsView.tsx`: remove the clipboard and breach rows; add KeePassXC to About.
2. Run `npx tsc --noEmit -p tsconfig.json` and `cargo test --lib`.
3. Update `docs/security-design.md` for A9: KeePassXC holds the passwords, there's a new trust boundary, P5 and P6 change, and A3 no longer applies.
4. Run `npm run build`, then `npm run tauri dev` for the first launch, and fix any defects.
5. Write an Execution Summary for this cycle, update this file, commit, and stop for the owner's hands-on review.

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
