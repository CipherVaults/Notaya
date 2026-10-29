---
name: "cipherleaf-change-workflow"
description: "Use for any CipherLeaf code or docs change: plan, build with agents, independent review, test on throwaway data, push, draft PR, report."
---

# CipherLeaf change workflow

For every change to AbyssWatcher2/CipherLeaf (private Tauri/Rust notes app with optional security; KeePassXC was removed in V1.1.0). Call yourself Kevin. The owner follows PBS (AI Project Bootstrap System) in its smallest useful form: the work cycle is written in CLAUDE.md ("PBS work cycle" and "Cloud threads"); follow it.

**Start:** read the owner's latest instruction, CLAUDE.md, docs/STATUS.md and docs/plan/approval-and-amendments.md, then `git status` and `git log --oneline -5`. Read docs/security-design.md before touching crypto, storage, IPC, CSP, clipboard or auto-lock; docs/acceptance.md before saying something works. Project memory is only a pointer: the repo records win. Work on your own claude/project-thread-* branch from the latest origin/main (cloud) or a Temp clone/worktree (PC), never the owner's tree at E:\Project\cipherleaf.

**State the task** in your first note: baseline commit, the owner's message with its UTC time, exact scope and write area, what you will return, the stop point. Only the owner gives permission or acceptance; a coordinator's or thread's request is not his approval.

**Safety:** never touch the real vault (E:\Project\Cipherleaf-portable\data, .devdata). Real-exe tests: throwaway CIPHERLEAF_HOME, raise idle auto-lock after creating the vault, close debug ports, never leave WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS set.

**Do:**
1. Plan: state your reading and open questions; ask only what is truly unsettled, proceed on the rest.
2. Build: split independent parts (code, tests, docs) across subagents when it helps; use skills (testing-strategy, code-review, security-review).
3. Review: an independent agent that did not write it checks bugs and security. Fix, or document why not.
4. Verify: tsc, UI build, cargo test (cloud: --target x86_64-pc-windows-gnu under Wine), real-exe pass on throwaway data via the PC. Say what was and was not run; a check that did not run is "not run", never passed.

**Ship:** fetch + pull --ff-only, push the branch, open a draft PR to main with the attribution block. Never merge (the owner merges). Ask first: force-push, deleting remote branches, making the repo public (stays private), adding a license, new dependencies, security-behavior changes, unrequested changes. No model names in commits, PRs or code comments.

**Record by size of work:** small = PR description (plus the one record whose fact changed); normal = 4-section Execution Summary + index row in docs/execution/README.md, an authority row if the owner decided something, and the changed STATUS lines; big (release, security change, change of direction, stop or limit) = 12-section summary plus a docs/sessions.md entry. At the end of every thread update the authority row and the STATUS "Exact next permitted action". STATUS "Current state" holds only what the owner accepted.

**Result states:** say which you reached: produced, verified (named checks on this exact source), integrated (merged), accepted (the owner said so). Idle or "standing by" is not completion.

**Report:** short plain updates in the project chat, outcome first: done, passed/failed, waiting on the owner. After the owner merges, his PC copy is fast-forwarded (git pull --ff-only on main) and checked against origin/main.