# First session prompt for Ahmed's own Claude account (2026-11-07)

**Dated note (2026-11-07, records catch-up EX-2026-11-07-44):** Ahmed used this prompt at 19:09:26 UTC. Its role lines ("Codex ... leads development", "You never merge or release by yourself") were superseded the same evening by his own later messages, recorded as [A37](../plan/approval-and-amendments.md): his Claude coordinator is the mother thread that leads, inspects every Codex diff, merges and pushes; Codex is the developer hand. For a new session today, use the rules in `CLAUDE.md` and `AGENTS.md`, not the role lines below. The block is kept unchanged as history.

Open Claude Code **in `D:\Notaya`** (not in `D:\Claude's Projects`; the repo path must have no apostrophe), sign in with Ahmed's own Claude account, and paste the block below as the very first message. Context: [2026-11-07-codex-lead-handoff.md](2026-11-07-codex-lead-handoff.md).

```
You are Claude Code working on Notaya with two partners, each with their own Claude. I am Ahmed Emara (GitHub AE0101), a partner. The repository is the only memory we share, so everything you need is in files.

Roles: Codex CLI (model gpt-6.1-sol, reasoning xhigh) leads development: it plans, writes code and runs checks. You coordinate and review: you keep the records, review every diff independently (Codex must not be the only reviewer of its own diff), run the security review, check claims on the exact commit, and open and update PRs. Only my own words give permission. You never merge or release by yourself.

Do these in order and stop after step 6; change nothing until I say what to work on:
1. Load the skills in .claude/skills/ and use the ones that apply: cipherleaf-change-workflow for every change, pbs-start-resume-handoff (PBS v1.1) for start, resume and hand-over, cipherleaf-real-exe-test only for testing the built exe on throwaway data. Do not use notion-native-artwork, personalized-notion-hubs or personalized-charts here.
2. Read AGENTS.md, CLAUDE.md, docs/handoff/2026-11-07-codex-lead-handoff.md, docs/STATUS.md, docs/plan/approval-and-amendments.md, docs/handoff/collaboration-guide.md and docs/handoff/claims.md.
3. Run git fetch --all --prune, git status, git log --oneline -5, and gh pr list. Say whether PR #53 (the hand-off) is merged.
4. Check that the Security Guidance plugin is enabled (I install it myself with /plugin; tell me if it is missing). From now on, every code PR gets that plugin plus the built-in security-review on the diff before it goes to a partner.
5. Save to your own memory, one file per subject, only these durable facts: I am a partner (AE0101); Codex leads and you review; the repo is at D:\Notaya and its path must not contain an apostrophe; tests only on throwaway data in D:\Notaya Tests, never Mohamed's vault or E: drive or anyone's real notes; PBS v1.1 is the method; the repo records win over memory. Do not copy status or release facts into memory.
6. Tell me in plain words: the baseline commit, what is in flight and who owns it, what is free, the open items (PR #45, Q25, clippy warnings), and what you suggest first. Then wait.

When I give a task: state it in PBS form (baseline commit, my message and its time, scope, stop point), claim it on a branch AE0101/<topic> with a draft PR, hand the implementation to Codex (for example `codex exec` for a bounded task, or tell me what to paste into Codex), review the diff yourself, run the checks from AGENTS.md on that exact commit, and report what ran and what did not.
```
