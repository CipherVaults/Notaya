---
name: habits
description: Short list of working habits that keep context small. Run by hand with /notaya-token-saver:habits.
disable-model-invocation: true
---

# Token habits

For the rest of this session, follow these habits. They come from Claude Code's own cost guide (code.claude.com/docs/en/costs) and prompt-caching page. No savings figure is promised.

## How to work

1. **Look before you read.** Search (Grep, Glob) first, then read only the lines you need. Do not re-read a file that is already in this conversation unless it changed.
2. **Keep command output small.** Ask for the part that matters: only the failing tests, the last 40 lines, one matching pattern. On Windows PowerShell use `Select-Object -Last 40` or `Select-String`; in a Bash shell use `tail -n 40` or `grep`. If a command prints a lot, say what you looked for.
3. **Short replies.** Give the result and what it depends on. Do not repeat the plan, quote large diffs back or summarise what the person can see.
4. **No agents unless the job needs them.** Every subagent or parallel agent sends its own requests on top of this one. If one is really needed, use a smaller model for it.
5. **Do not switch model in the middle of a session.** Switching throws away the saved cache for the session. Choose model and effort at the start.
6. **Same failure twice means change approach.** If a command fails the same way twice, stop, read the error and try something different, or ask.
7. **Specific over broad.** Work from the file and the change named in the request. If the request is vague, ask one short question instead of scanning the whole repository.

## For the person (do these yourself; Claude cannot)

- **New task, new session.** Run `/clear` when the topic changes (it costs nothing). Run `/rename` first if you want to come back with `/resume`.
- **Compact at a natural break, with a focus line**, for example `/compact keep the test results and the list of changed files`. Compacting a very large context is itself a large request, so do it before the session gets huge, not after a long idle gap.
- **After about an hour away, the saved cache has expired** (one hour on a Claude subscription inside plan usage, five minutes otherwise). The first message back re-sends the whole conversation. For a big old session, start a new one with a short summary instead.
- **Look at the numbers.** `/context` shows what fills the window. `/usage` shows where your plan usage went (skills, subagents, plugins, MCP servers) and recommends changes. When its footer says "f to fix", pressing `f` has Claude Code apply the ones it can and show you each change first (Claude Code v2.1.305 or newer).
- **Turn off what you do not use:** unused plugins (`/plugin`), MCP servers (`/mcp`), and loops or scheduled tasks that wake an idle session.
