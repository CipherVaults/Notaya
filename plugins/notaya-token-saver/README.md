# Notaya Token Saver

A small Claude Code plugin that helps you use fewer tokens. It is local only: no network, no extra model calls, no files written. Status: **produced** (see the checks at the bottom); not yet tried inside a real Claude Code session on your PC.

It is honest about size: Claude Code's own estimate (`claude plugin details`) is about **120 tokens added to every session** by the two skill listings. No saving figure is promised. Whether it helps you is something you can see in `/usage`.

## What is in it

| Part | What it does | Costs tokens? |
|---|---|---|
| **Resume warning** (hook) | When you resume an old session whose saved prompt cache has expired, shows one line to you: how long it was idle and about how many tokens your first message will re-send. It only uses numbers Claude Code gives the hook itself. If the numbers are missing, it says nothing. | No. The line is shown to you only; Claude never sees it. |
| **`/notaya-token-saver:checkup`** (skill, manual) | Reads your settings, CLAUDE.md files, skills and `.mcp.json` and lists things that make every request bigger. Read-only. It never prints the values of `env` settings. | Only when you run it. |
| **`/notaya-token-saver:habits`** (skill, manual) | Loads a short list of working habits for Claude for the rest of the session, and a reminder list for you (`/clear`, `/compact`, idle gaps). | Only when you run it. |
| **`claude-md-snippet.md`** | A few lines to paste into your own `CLAUDE.md` if you want the habits on all the time. | About 150 tokens per request if you paste it. Your choice. |

## Install on Windows (PowerShell)

You need Claude Code 2.1.251 or newer (the resume numbers come from that version) and Node.js (you already have it for the Notaya app). Run this once, after this pull request is merged:

```powershell
cd E:\Project\Notaya\Notaya
git pull --ff-only
claude plugin marketplace add E:\Project\Notaya\Notaya
claude plugin install notaya-token-saver@notaya-tools
```

Then close Claude Code and start it again. To check:

1. Type `/plugin`. `notaya-token-saver` should be listed and enabled.
2. Type `/hooks`. A `SessionStart` hook from the plugin should be listed.
3. Type `/notaya-token-saver:checkup`.

Notes:

- `claude plugin marketplace add` records the marketplace in your user settings (`%USERPROFILE%\.claude\settings.json`). Nothing else on the PC changes.
- The plugin runs from your Notaya folder. To update it later: `git pull --ff-only` in that folder, then restart Claude Code.
- A friend or partner installs it the same way, with their own folder path.

Remove it:

```powershell
claude plugin uninstall notaya-token-saver@notaya-tools
claude plugin marketplace remove notaya-tools
```

## Two settings worth adding by hand

These are not part of the plugin (a plugin cannot set them). Edit `%USERPROFILE%\.claude\settings.json` and add the line inside the top-level `{ }`:

```json
"bashOutputMaxChars": 12000
```

What it does (Claude Code docs, needs v2.1.261+): when a **successful** command prints more than that many characters, the full output is saved to a file and Claude gets a short preview plus the file path. The default is 30,000. The allowed range is 4,000 to 128,000. The number 12000 is my suggestion, not a measured value. Failed commands are not shortened.

Your `autoCompactWindow` of 500000 is already inside the valid range (100000 to 1000000). If your limits still drain fast, 300000 to 400000 is worth a try; smaller means more summarising and some lost detail.

## Settings you can change for the hook

Set as environment variables (for example in `settings.json` under `"env"`):

- `NOTAYA_TOKEN_SAVER_MIN_TOKENS`: the resume warning stays quiet below this size. Default `50000`.

## What it deliberately does not do

- It does not shorten tool output with a hook. Claude Code already does that itself (`bashOutputMaxChars`, and `MAX_MCP_OUTPUT_TOKENS` for MCP tools), and a hook can only touch successful output anyway.
- It does not call a model, keep a database, change your settings or rewrite your commands.
- It does not switch your model or effort for you.

## How it was checked

- `claude plugin validate` on the marketplace and the plugin folder: passed (one advice about an icon, one about a marketplace description).
- Installed into a throwaway Claude Code config folder on Linux: `claude plugin details` listed 2 skills and 1 hook.
- The hook script was run by hand with sample inputs (resume, fork, cache still warm, small session, bad JSON, empty input). It printed one line when it should and nothing otherwise, and always exited 0.
- The checkup script was run against a fake settings folder that held a fake secret. The secret did not appear in the output.
- **Not run:** the plugin inside a real Claude Code session, on Windows, or on your PC.
