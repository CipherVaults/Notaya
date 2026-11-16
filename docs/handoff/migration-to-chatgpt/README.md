# Migration to ChatGPT Pro and Codex: start here

**Dated pause correction (2026-11-10, R3):** This file's earlier open-item, heartbeat and next-lane statements are historical. Read the [pause resume note](../2026-11-10-pause-resume-note.md) first: development, fixes, scripts and merges are paused on Ahmed's 06:13:35 UTC words, clarified 06:25:54 UTC. A55 still requires his handover acceptance. Scheduled-task removal is not verified.

**Dated R2 correction (2026-11-10):** earlier state and role lines below are historical snapshots. For the later 1.6 lane state, unresolved N2 test conflict and the conditional A55 handover, read [STATUS](../../STATUS.md) Continuation and session 55. A merged lane is not partner acceptance.

**Who and where:** Ahmed's side (`AE0101`). Written by Claude (Sonnet 5.5) in the cloud thread "Prepare full migration handoff to ChatGPT Pro", on 2026-11-10 from about 03:33 UTC (system clock). Baseline commit `f5d7a55` (`main`, PR #126). Machine: cloud, with one read of the Gaming-PC heartbeat folder.
**Asked by:** Ahmed, project chat, 2026-11-10 03:32:47 UTC: Claude's weekly quota is almost gone, so the lead role moves to ChatGPT Pro (ChatGPT plus the Codex desktop app, GPT 6.1 Sol and Terra), with "a full proper handoff ... the context, the situation, the AI PBS, ... the artifacts, how to update them".
**Result state:** *produced*. Nothing here has been run by ChatGPT or Codex yet, and nothing is accepted. The role change takes effect only when Ahmed says so in ChatGPT (see [A55](../../plan/approval-and-amendments.md)).

## For Ahmed: what to paste where (plain words)

**Shortcut (added 2026-11-10 about 04:30 UTC at your request):** the Notaya Notion hub now has a page **[Claude → Blue Handoff](https://app.notion.com/p/414bd7ade63b81bb92efcf9c2402d2b5)** in the same style as your other Blue handoffs. It holds the current checkpoint, the steps, and where everything lives (GitHub, Notion, your PC folders), plus a paste-ready thread prompt. You can tell Blue: "Go read the Claude Handoff in the Notaya Notion hub." The steps below remain the full route.

1. **Make a ChatGPT Project called "Notaya"** (Projects in the ChatGPT app). Paste block **A** from [`02-start-prompts.md`](02-start-prompts.md) into the project's *Instructions*. Upload (or connect) these files so the coordinator can read them: this folder, `docs/STATUS.md`, `docs/plan/approval-and-amendments.md`, `docs/handoff/claims.md`, `AGENTS.md`, `CLAUDE.md`. If the GitHub connector is available in ChatGPT, connect `CipherVaults/Notaya` read-only first.
2. **Open the first chat in that Project** and paste block **B** (the first message). It makes ChatGPT read the records, check the live repository and tell you in plain words where things stand, then stop and wait. It changes nothing.
3. **Open the Codex desktop app on the Gaming-PC**, pointed at `D:\Notaya` (no apostrophe in the path), and paste block **C**. Codex already reads `AGENTS.md` by itself; block C only adds the migration facts.
4. **Put the private notes in a local folder.** The Notion survey and the exact formulas are *not* in the repository on purpose (they hold your real structure and formula text). Copy them to `D:\Claude's Projects\Notaya-private-notes\` (Claude copies them there for you, see "Private notes" below) and give ChatGPT/Codex access to that folder only when a task needs it. Never commit them.
5. **Tell Mohamed** (his own Claude Pro, no Codex): block **D** is his start message if he has to carry on. Nothing changes for him until you both agree.
6. **Say "handover accepted" in ChatGPT when you are happy.** Until then Claude's last few percent stay as a spare; after that the coordinator is ChatGPT.

## What is in this folder

| File | What it is | For whom |
|---|---|---|
| [`01-migration-guide.md`](01-migration-guide.md) | The full handoff: vision, people, PBS method, where 1.6 stands, what is left, how Claude things map to ChatGPT/Codex, first-day plan, honest limits | the ChatGPT coordinator, Codex, Mohamed |
| [`02-start-prompts.md`](02-start-prompts.md) | Four paste-ready blocks (A project instructions, B first message, C Codex desktop, D Mohamed on Claude Pro) and a lane brief template | Ahmed |
| [`03-standing-rules.md`](03-standing-rules.md) | Every standing rule from project memory and `CLAUDE.md`, copied faithfully with dates, plus the memory index word for word | everyone |
| [`04-artifacts-and-rebuild.md`](04-artifacts-and-rebuild.md) | The four Claude artifacts, where their source lives now, how to update or rebuild each without Claude | coordinator, Codex |
| [`05-pc-automation.md`](05-pc-automation.md) | The heartbeat, the 2-minute guard, the lanes folder, the stop line, and what must change once Claude is gone | coordinator, Codex |

Copies added by this migration (safe to read, no private values): `tools/heartbeat/` (the PC scripts), `docs/artifacts/snapshots-2026-11-10/` (roadmap, handover guide and ledger dashboard as plain HTML), `docs/plan/notes-archive/` (project notes that lived only in Claude's project files).

## Private notes (not in the repository)

These stay on your PC, outside Git: `notion-survey/` (eight files A to H), `notion-formulas/2026-11-08-formulas-exact.md`, the Notion parity study, the dashboards study and the FFmpeg lessons page. The records name them so nobody loses them. If a later session needs real values, it asks you; the PFS hub stays read-only.
