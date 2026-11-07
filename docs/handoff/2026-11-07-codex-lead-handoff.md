# Notaya hand-off to Ahmed: Codex leads, Claude reviews (2026-11-07)

Written by Claude (Ahmed's own PC, Remote Control session started from Mohamed's account) for Ahmed (GitHub `AE0101`, partner) and for whichever agent picks up next. Times are from the system clock; no PC test or build was run for this note. Per `CLAUDE.md`, the repository records win over this file wherever they differ.

## 1. The new setup

- **Codex CLI / app (model `gpt-6.1-sol`, reasoning `xhigh`, already set in Ahmed's `~/.codex/config.toml`) leads development.** **Claude coordinates and reviews** (independent diff review, security review, records, checking claims on the exact commit). Entry file for Codex: [`AGENTS.md`](../../AGENTS.md), which points at `CLAUDE.md` as the single source of rules.
- **Ahmed is a full partner** (Mohamed, 2026-10-29 08:39:33 and 08:40:02 UTC, recorded in `claims.md` and `CLAUDE.md`): his own words give permission, and he merges, releases and accepts. Only Mohamed changes GitHub repository settings. Making the repository public or adding a licence needs both partners.
- **Repository:** `https://github.com/CipherVaults/Notaya` (the old `AbyssWatcher2/Notaya` and `AbyssWatcher2/CipherLeaf` links in the older records still redirect or are history; do not rewrite them, add a dated correction instead).
- **Ahmed's PC paths:** repo `D:\Claude's Projects\Notaya`; suggested test folder `D:\Notaya Tests` (outside the repo, create it yourself). Every `E:\...` path in the records is **Mohamed's PC**. Never touch his real vault, his `E:` drive or `E:\Project\Notaya\Notaya Tests Excluded`.
- **Shared memory = this repository only.** Mohamed's claude.ai project memory and chat are not available to Ahmed's Claude or to Codex. Everything below was taken from the repo records.

## 2. Release state (from `docs/STATUS.md`, `docs/acceptance.md`)

- **Latest release: v1.5.1, ACCEPTED** by Mohamed (2026-11-05 11:08:39 UTC, "I approve this version", X27). Tag on merge commit `95ca8b2`. `main` was at `4fc269d` (merge of records PR #52) when this note was written.
- Assets (unsigned): `Notaya-V1.5.1-setup.exe` (SHA-256 `B22857ED...A07DA6`) and `Notaya-V1.5.1-portable.zip` (SHA-256 `4576D503...A28E7`); full hashes in `docs/release-notes/v1.5.1.md`.
- History: V1.4.0 accepted 2026-11-03; V1.5.0 published 2026-11-04 with no acceptance of its own; V1.5.1 fixes and additions on top.
- Acceptance grants no new permission. A new release needs a partner's own words. **Nothing is being built right now.**

## 3. Open items

| Item | State (verify before relying) |
|---|---|
| Draft **PR #45**, Notion roadmap / migration plan | A planning PR from a separate thread (see `claims.md`, `plans/notion-*` in the claude.ai project files, not in the repo). It becomes work only if a partner picks a phase. Check with `gh pr view 45` after `gh auth login`. |
| **PR #27**, live-audit comparison | `STATUS.md` says it was merged (`5013244`). What is still open is **Q25**, below. Confirm the PR state with `gh pr view 27`. |
| **Q25** (`STATUS.md`, Open questions) | The live checks of 2026-10-23 compared with V1.1.0: 5 fixed, 3 partly, 1 no longer relevant, **15 still open** (WebView2 traffic, F-04, F-10, SE-03, CR-02, contrast and focus, others). Decision needed: plan a fix release for any of them? Table: `docs/audit/2026-10-27-live-checks-vs-v1-1-0.md`. |
| **`cargo clippy` warnings** | `cargo clippy` (MSVC) was **never run on the final V1.5.1 commit** (not-run list). Warnings were reported by the previous coordinator but the list is not in the repo; run clippy on `main`, record the real list, then decide what to fix. |
| Other not-run checks of V1.5.1 | Setup installer of this build (only on a clean Windows user or VM, untick "Run Notaya", FI-05); tray entry "Close sticky notes"; Windows keyboard-layout shortcut between Arabic and English lines; Playwright mock checks `dev-mock/*-check.cjs` (two new assertions in `rtl-ui-check.cjs` never ran); Word and PDF files of the Notion Hubs import. |
| Other open decisions | Licence of Notaya itself (not chosen; both partners); `elkjs` (EPL-2.0) dependency confirmation; recent searches stored in the vault; Q9, Q10, Q13, Q14, Q23, Q24 in `STATUS.md`. |
| Ideas for next work (only when a partner asks) | Per-line text direction in Word/PDF/HTML export (`export/doc.rs`, `first_strong`); closer colour match for Notion's built-in covers; streaming importer for very large imports (about ten times file size in memory); slow attach of files of hundreds of MB to database rows; free pictures inside columns/lists/tables and in exports; Mermaid abort for huge diagrams. Source: `docs/handoff/2026-11-04-v1-5-1-resume-note.md`. |

## 4. Rules that matter most (full text in `CLAUDE.md`)

Stable releases only, exact versions; ask before installing or adding dependencies; never edit `docs/plan/approved-plan-2026-10-22.md`; never commit vaults, `*.nota`, `*.clf`, `*.key`, `.env`; throwaway data only, `CIPHERLEAF_HOME` on a temp folder, debug port closed and checked afterwards (FI-01); branch `AE0101/<topic>`, draft PR, claims row in the first commit, nobody merges but a partner; report unrun checks as "not run"; quote times from the clock, not from memory.

## 5. PBS usage

The project follows PBS in its smallest form (`CLAUDE.md`, "PBS work cycle"; A29). The skill `.claude/skills/pbs-start-resume-handoff/` is committed and ships `agents/openai.yaml`, so Codex can load it; its bundled references say **PBS v1.0**. Ahmed's own `~/.codex/skills/pbs-start-resume-handoff` is presumably the newer v1.1 (not compared). Do not overwrite either; if Ahmed wants v1.1 in the repo, do it as a separate small PR so the diff shows what changed.

Practice: start with the PBS "Start" or "Recover" prompt, state the task (baseline commit, partner message and time, scope, stop point), keep permission / observation / acceptance apart, label results produced / verified / integrated / accepted, and end with a resume note and a claims-row update. Records by size: small = PR text; normal = 4-section Execution Summary + index row + STATUS lines; big = 12-section summary + `sessions.md` entry.

## 6. Skills and plugins

| Item | Use it? |
|---|---|
| **Security Guidance** plugin (Anthropic) | **Enable** in Claude Code (`/plugin`). Required for every code PR. |
| Built-in **`/security-review`** | Run on every code diff before it goes to a partner; report what each check found and what did not run. |
| `cipherleaf-change-workflow` | **Use** for every code or docs change. |
| `pbs-start-resume-handoff` | **Use** at session start, resume and hand-over. |
| `cipherleaf-real-exe-test` | **Use** only to test the built exe, on throwaway data. Written with Mohamed's `E:` paths; use Ahmed's paths. |
| `cipherleaf-project-coordination` | **Use** only when a Claude acts as the coordinator between threads. |
| `notion-native-artwork`, `personalized-notion-hubs`, `personalized-charts` | **Leave disabled** for Notaya work (Ahmed's own Notion/chart skills, not project skills). |
| Unrelated plugins / connectors (data, HR, design, product-management, productivity suites) | Leave off for this project; they add context cost and no Notaya value. |

These skills still say "CipherLeaf" and mention KeePassXC in places; `CLAUDE.md` wins where they differ.

## 7. Toolchain on Ahmed's PC (checked 2026-11-07 on his machine)

| Tool | State |
|---|---|
| Git 2.40.0 (identity `AE0101`), Node 24.14.0 / npm 11.9.0, Python 3.14, VS Code, winget | present (guide lists Node 24.20.0 / Git 2.55.0; newer patch versions are normally fine) |
| Visual Studio 2022 Community with C++ tools | present |
| WebView2 | not checked separately (Windows 11 includes it) |
| GitHub CLI 2.x | installed, **not signed in**: Ahmed runs `gh auth login` as `AE0101` |
| Rust (rustup, stable MSVC, 1.99.1 used by the project) | **missing** at the time of writing; install on Ahmed's OK (about 1.5 GB; `C:` had 14.8 GB free) |
| Codex | desktop app and `~/.codex` already present; no `codex` command on PATH |
| NSIS | nothing to install; Tauri downloads it on the first `npm run tauri build` |

Then: `npm ci`, `npx tsc --noEmit -p tsconfig.json`, `npm run build`, `cd src-tauri; cargo test --lib`. Report the commit they ran on. (Not run yet on this PC when this note was written.)

## 8. First steps for the next agent

1. Read `AGENTS.md` / `CLAUDE.md`, `docs/STATUS.md`, `docs/plan/approval-and-amendments.md`, `claims.md`, this note. `git fetch --all --prune`; list open PRs.
2. Ahmed says what to work on. Suggested order if he has no preference: finish the toolchain and run the smoke test on `main`; run `cargo clippy` and record the list; decide Q25.
3. Claim on your own branch, work in Codex, review in Claude, security-review, draft PR.
