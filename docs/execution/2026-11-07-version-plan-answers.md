# EX-2026-11-07-53: version plan updated with Ahmed's answers (V1.6 to V1.9, then V2.0 to V2.5)

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Plan versions 1.6 to 1.8 and beyond" (machine: cloud). Same session as [EX-2026-11-07-50](2026-11-07-version-plan.md). Model configured `claude-opus-5-5[1m]`, effort high. One helper (Sonnet 5.5) renumbered the Notion board cards.

Short form (4 sections). Small: a plan update and records. Times are UTC: message timestamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `67b27e4` (merge of PR #62).
- **Authority:** Ahmed's messages of 23:53:46 and 23:54:20 UTC (project chat), relayed by the coordinator at 23:54:26 with the request to update the plan, its repository copy (new PR, merged) and the Notion hub.
- **Scope and write area:** `docs/plan/2026-11-07-version-plan.md`, two authority rows in `docs/plan/approval-and-amendments.md`, this report and its index row, a dated addendum to session entry 44, one STATUS block, the claims row; outside the repository, the plan in the shared files and the Notion hub (board cards, the roadmap view's name and columns, the roadmap page).
- **Out of scope:** app code, security behaviour (the answers are design direction for the 1.6 spec, not the go for the AI channel), the handover guide (see section 4).

## 2. Work Performed

1. Added section 0 "Ahmed's answers" to the plan with each answer, quoted, and what it changes.
2. Renumbered the versions after 1.9: the old 1.10 to 1.15 are now **2.0 to 2.5** everywhere (plan IDs too, for example 1.10-1 is now 2.0-1).
3. Folded the AI CLI answers into 1.6-6, 1.7-12 and section 5.2 (four levels, Full may delete large parts to the recycle bin after a confirm, hidden notebooks, optional big-change confirm at 20 items, exchange folder inside Notaya's own data folder, two AIs kept apart, AI never draws freehand but may look at a board picture). Page locks and "lock now" are deferred; the bridge stays a proposal.
4. Rewrote section 10 to the questions still open (five).
5. Notion: added V2.0 to V2.5 to the board, moved and rewrote the cards (helper; counts in the PR description), renamed the roadmap view "V1.6 to V2.5" and showed Plan ID, Effort and Gate on it, updated the roadmap page (version table, waves, journey line, a decisions line).

## 3. Verification and Evidence

- **Checks run:** the plan has no remaining "1.10" to "1.15" except in the quoted answer about renumbering; the repository copy differs from the shared-files copy only by its header note. The ID first chosen, EX-51, collided with the Manual's report (PR #63, merged first); renumbered to **EX-53** at merge.
- **Notion:** the workspace's SQL query allowance ran out during this work, so the helper checked the board through a saved view instead (its report is in the PR description).
- **Not run:** app checks (no code changed).
- **Result state:** produced; verified only as above; integrated on merge; not accepted.

## 4. Outcome and Open Points

- Five plan questions remain (plan section 10): the five page gaps in 1.8, the go for the section 9 planning work, page locks and "lock now" (deferred by Ahmed), the bridge, and where confirmations appear.
- **Handover guide (A39):** its line about the plan says "V1.6 to V1.15"; the renumbering to 2.0–2.5 should reach both copies at the next guide update (left for the thread that keeps the guide, to avoid parallel edits of the artifact).
