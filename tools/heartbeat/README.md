# Reference copy of the PC heartbeat (snapshot of 2026-11-10)

**Who and where:** Ahmed's side (`AE0101`). Copied from `D:\Notaya-lanes\heartbeat` on the Gaming-PC by Claude (cloud thread "Prepare full migration handoff to ChatGPT Pro", 2026-11-10 about 04:00 UTC), read-only; EX-2026-11-10-73. **This is a snapshot for reading and for rebuilding the PC setup. The live scripts on the PC are what runs.** If the two differ, the PC's copy is current; update this folder with a dated note when it matters.

What is here: `heartbeat.ps1` (the 10-minute tick), `stopline-guard.ps1` (the 2-minute guard), `run-lane.ps1` (starts one Codex lane), the registration scripts (`register-task-hidden.ps1`, `register-guard-task.ps1`) and their hidden launchers (`*.vbs`), the Claude status-line helpers, `terra-trial.ps1` (checks a model id with a one-line trial), `loop.ps1` (the first, older loop), `mcp-doc-notes.md` (official MCP findings used by lane D) and `ORIGINAL-README.txt`.

**Not here (stay on the PC):** logs, state files (`heartbeat-state.json`, `claude-rate.json`), the lane prompt files (`prompt-*.md`, `briefs.md`), lane worktrees and `task-backup\`.

How it works, what each tick does, and what must change now that Claude is not the coordinator: [`docs/handoff/migration-to-chatgpt/05-pc-automation.md`](../../docs/handoff/migration-to-chatgpt/05-pc-automation.md). The scripts contain Ahmed's Windows user name in some paths; there are no keys or tokens (pattern-searched). Registering or changing scheduled tasks is Ahmed's own step.
