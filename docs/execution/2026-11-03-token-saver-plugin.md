# EX-2026-11-03-39: token-saver plugin for Claude Code

Short form (4 sections). Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `origin/main` `8f7fb86` (merge of PR #42).
- **Owner's words:** 2026-11-03 05:48:08, "Can you build a token optimizer plugin or skills"; 05:49:54, "Check also the github what they do with token optimizer plugin / Like this one for example / https://github.com/microsoft/TokenEconomizer". The coordinator's brief: reliable ideas only, based on the research doc "Stretching Claude usage limits", simple Windows install steps, draft PR, no change on the PC.
- **Scope and write area:** `plugins/notaya-token-saver/`, `.claude-plugin/marketplace.json`, this report and its index row, the claims row. **Out of scope:** app code, `CLAUDE.md`, the existing skills, the owner's settings, anything on the PC.
- **Work size:** normal (tooling, no security behaviour of the app). **Returns:** a draft PR. **Stop point:** the draft PR open and reported; the owner merges and installs.

## 2. What was done

- A Claude Code plugin `notaya-token-saver` 0.1.0, listed in a marketplace `notaya-tools` at the repository root:
  - one `SessionStart` hook (`resume|fork`) that shows the person one line when a resumed session's prompt cache has expired: idle time, tokens the first message re-sends, Claude Code's own dollar estimate. It uses only fields Claude Code puts in the hook input (v2.1.251+), writes nothing, calls nothing, adds nothing to Claude's context, and exits 0 on any problem;
  - `/notaya-token-saver:checkup`, a manual skill with a read-only script that reports settings, CLAUDE.md size, skill descriptions and `.mcp.json`; it never prints `env` values and clips what it prints from settings files;
  - `/notaya-token-saver:habits`, a manual skill with seven working habits for Claude and a reminder list for the person;
  - `claude-md-snippet.md` (optional) and a README with Windows install steps and one suggested setting, `bashOutputMaxChars`.
- **What the research and the other plugins changed.** Claude Code already has native, lossless output trimming (`bashOutputMaxChars`: the full output goes to a file, Claude gets a preview and the path; `MAX_MCP_OUTPUT_TOKENS`), so a trimming hook was not built. It could only touch successful output anyway.
- **Microsoft TokenEconomizer** (read as untrusted reference; cloned shallow and read, never run): a research plugin (UC Berkeley and Microsoft Research, Python 3.10+, about 8,000 lines in `tokeneconomizer/`). It has three mechanisms: repeated "micro-skill" facts about a repository, removal of unused tools, and a failure-loop guard with a judge model. The authors report 18.4 to 30.9% lower mean cost on SWE-Bench tasks; that is their own measurement, not checked here. It makes its own model calls, keeps a store in `~/.tokeneconomizer`, and edits the agent's settings. **Not adopted:** too large and too many moving parts for a notes-app owner who wants fewer tokens, and its model calls cost tokens. Ideas kept: fail open (the session carries on if the plugin breaks), say what it did in one line, keep every action reversible. Ideas left for later if wanted: a rule-based "same command failed three times" nudge, and unused-tool cleanup (today a manual `/mcp` and `/plugin` step).
- A short search of GitHub showed several small "token saver" plugins and skills with 0 to 121 stars and savings claims such as "30-60%" or "60-80%". Only names and descriptions were looked at; none was read or run, and the claims were not used.
- Honest cost: `claude plugin details` estimates about 120 tokens added to every session by the two skill listings.

## 3. Checks

All on the exact source of the PR head, in the cloud container (Linux, Claude Code 2.1.317, Node 22). Result state: **produced**, partly **verified** as listed.

- `claude plugin validate` on the repository (marketplace) and on the plugin folder: passed. The plugin-directory profile advice (icon, license) was not acted on; this is not a directory submission.
- Installed from the marketplace into a throwaway config folder: `claude plugin details` listed 2 skills and 1 hook; uninstall and marketplace removal worked.
- The hook script run by hand with eight inputs (resume with expired cache, fork, warm cache, small session, startup, bad JSON, empty input, no stdin): one line when due, nothing otherwise, exit 0 each time.
- The checkup script run against a fake config folder holding a fake secret and a long hostile `model` value: the secret never printed, the long value was clipped.
- Security review (built-in `security-review` rule, done by reading the whole diff, without sub-tasks): no finding at confidence 8 or higher. Notes: the scripts have no network, no write, no shell and no dynamic code; output is built from numbers or clipped strings. One low-risk issue (a project's settings value printed into Claude's context) was fixed by clipping. The Security Guidance plugin's own AI diff review was not run (no API credentials in cloud threads).
- **Not run:** the plugin inside a real Claude Code session; Windows or PowerShell; the owner's PC; a real resume with an expired cache; whether the savings are real (nothing measured).

## 4. Open items

- The owner merges (merge commit), then runs the install steps in `plugins/notaya-token-saver/README.md` on the PC and fast-forwards the PC copy.
- The owner decides whether to add `bashOutputMaxChars` (suggested value 12000 is a judgement, not a measurement) and whether to paste the CLAUDE.md snippet into his user `CLAUDE.md`.
- The existing project skills `notion-native-artwork`, `personalized-notion-hubs` and `personalized-charts` are not Notaya work (guide, section 3) but their descriptions are listed in every session here (about 900 characters). Hiding them with `skillOverrides` is possible and was not done; it is the owner's call.
- Acceptance: not applicable until the owner has used it.
