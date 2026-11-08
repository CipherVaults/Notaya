# EX-2026-11-08-58: Notion hub sync with the version plan (record)

> **Records note (added by the ledger thread, EX-2026-11-08-57):** copied unchanged from the project file `notes/2026-11-08-notion-hub-sync-record.md` (written by the Notion-board cloud thread; author Ahmed Emara `AE0101`, machine cloud), at the coordinator's request of 2026-11-08 00:54:59 UTC. Its repo-copy to-do was done in EX-2026-11-08-57: the two Notes edits for 1.6-10 and 2.0-1 are now in `docs/plan/2026-11-07-version-plan.md`. The board cards 1.6-10b and 2.0-1b exist in Notion only.


- **Who and where:** Ahmed's side (`AE0101`), done by Claude in a cloud thread, machine: cloud (no PC used). Time: 2026-11-08 about 00:50 to 00:58 UTC (container clock; Ahmed's messages at 00:49:25 and 00:50:29 UTC).
- **Asked by:** Ahmed, "where is v2.4? and ... is that small feature of v1.8 worth being called that?", then "make these features more clear, because I just thought it was one feature, not multiple (presentation issue)".
- **Source of truth:** `notes/2026-11-07-version-plan.md` (PR #67).
- **Result state:** *produced* (edited in Notion; not accepted by Ahmed).

## Audit result (Version Feature Board, 88 cards for V1.6 to V2.5)
- All plan items 1.6-1 to 2.5-3 exist as cards; Effort matches the plan on every card. 1.9-3 is intentionally absent (moved to 1.7-13). V2.4 has its 6 cards (2.4-1 to 2.4-6) and V2.4 was already in the Human Roadmap table.
- The real problem was presentation: the Human Roadmap showed each version as one paragraph row, the hub text still said "V1.6 to V1.8", and the board is 17 columns wide.
- Extra cards not in the plan, left alone: "Export as button next to Favorites" (1.6, Plan ID 1.6-10 duplicate), "Swipe to select several sticky notes" (2.0, Plan ID 2.0-1 duplicate). Board marks only 1 of 14 cards in V1.8 as "Big" because Big = L or XL; sizes follow the plan and were not changed.

## Changes made (Notion, Notaya hub only)
- Human Roadmap: rewritten with per-version feature counts, a new section "What is in each version, feature by feature" (every feature of 1.6 to 2.5 as its own bullet with effort and a plain description), stale statements fixed (Greek/Arabic in PDF moved to 1.7, whiteboard is 2.3 and 2.4, AI files folder placement, pilot scope, 1.6-15 is no longer a Candidate), "Last updated" line added.
- Hub page: stale "V1.6 to V1.8" wording changed to V1.6 to V2.5; link to the new table view added.
- Board: new table view "Every feature V1.6 to V2.5, one by one" (grouped by Version, shows Plan ID, Feature, Effort, Outcome, Gate).
- Cards: two Later cards renamed (their TOC block and templates parts moved to 1.8-12 and 1.9-16); Migration card order set to 18.
- Not touched: the 1.6 cards' "In progress" status, anything outside the Notaya hub.
- Not checked: how the new view renders in the Notion app (read back through the connector only).

## Correction, 2026-11-08 about 00:58 UTC (Ahmed's message 00:54:25 UTC)
- The two cards I called "not in the plan" are agreed features. "Export as button next to Favorites" belongs to 1.6-10 (Ahmed, 20:56 UTC); "Swipe to select several sticky notes" is the corkboard's box-select in 2.0-1 (Ahmed, 20:31 UTC).
- Both cards kept. Plan IDs are now 1.6-10b and 2.0-1b (no more duplicates), and their "Depends on" names the parent item.
- `notes/2026-11-07-version-plan.md` now names both explicitly in the Notes column of 1.6-10 and 2.0-1. The repo copy `docs/plan/2026-11-07-version-plan.md` still needs the same two edits (not done: no repo change made in this thread).

## Second sync, 2026-11-08 about 15:25 to 15:35 UTC: build state brought into Notion

- **Who and where:** Ahmed's side (`AE0101`), done by Claude in the cloud thread "Sync Notaya Notion hub with current status"; machine: cloud (no PC used). Time: container clock, 15:23 UTC (Ahmed's message 15:23:07 UTC: "make sure Notion for Notaya is updated ... the features board, the status, the roadmap, everything") to about 15:35 UTC.
- **Source of truth:** the repository at `origin/main` `8dc2a7b` (merge of PR #90), `docs/execution/2026-11-08-v1-6-build-start.md` (heartbeat log to 12:04 UTC), PRs #70, #74, #75, #76, #79, #85, #86, #88, #89 as read through GitHub, the lane branches `AE0101/v1-6-c4d` and `AE0101/v1-6-ag`, and the quota readings to 14:41 UTC. The PBS hub was read first. Result state: *produced* (edited in Notion; not accepted by Ahmed).
- **What the board looked like before:** all 15 V1.6 cards were "In progress" (set at 00:30 UTC) with no sign of what was built; the hub checkpoint still said "Nothing has merged yet".
- **Changes made (Notion, Notaya hub only; nothing else touched, no PFS value read or copied):**
  - Board: two new fields, **Build state** (⬜ not started, 🟦 queued · next lane, 🛠️ partly built, 🟡 merged · checked on PC · not accepted, ✅ accepted) and **Build note** (one dated plain sentence), set on the 16 V1.6 cards; Evidence / Source on each now links the pull request. Counts: 11 merged (1.6-1, 5, 7, 8, 9, 10, 10b, 11, 12, 13, 14), 2 partly built (1.6-2, 1.6-6), 2 queued (1.6-3, 1.6-4), 1 not started (1.6-15, stays Planned). Lifecycle was left as it was. V1.7 onward left empty on purpose (= not started).
  - Views: Build state added to "Every feature V1.6 to V2.5, one by one" (with Build note), "Roadmap ahead" and "Features by version".
  - Hub page: the checkpoint callout rewritten (merged lanes, counts, next steps, "nothing accepted, real window not tried"), the Board guide explains Build state / Build note and fixes "No card has this yet", and "Where the truth lives" gained the 1.6 spec, version plan, build record, usage ledger and handover guide.
  - Human Roadmap: "Where we are", "The next step", the V1.6 row and each V1.6 bullet carry the build state; a new "Decisions from Ahmed · November 8" section; a build-day entry in the journey; a sentence that merged is not accepted; "Last updated" line.
- **Judgement calls to check:**
  - "Merged · checked" uses the checks named in each PR (tests, type check, build, audit on Ahmed's PC); lane A's 1.6-8 to 1.6-14 were checked on the lane's source `2b953a3` before the fixes of #86 (#86 ran tsc, build, audit only). The real window was **not** checked for any card.
  - 1.6-5 (bin) says notebooks and sections "carry the same bin marker": the model fields exist, but the PR text names pages, stickies and database items only. Check with the real window.
  - 1.6-2 and 1.6-6 are "partly built" because the history lane (C3), the AI server (C5) and the C4d review fixes are not merged.
- **Conflict between sources, not resolved here:** the coordinator's relay said the history and activity-log lane was running. The repository shows no `v1-6-c3` branch, the latest records (PR #90, 12:03 UTC) say C3 is held until Codex's window resets (15:48 UTC), and the heartbeat's last Codex reading is stale. The board says "queued · next lane". Update when a C3 branch appears.
- **Left stale on purpose or not reachable:** the repo `docs/STATUS.md` "As of" lines still stop at the 00:30 UTC checkpoint (not this thread's task); PR #89 (entry 49, not yet merged) and the missing authority rows for Ahmed's 01:09 and 09:03 messages are open in the repo, so the Roadmap marks the "Mohamed continues" line as reported in the chat with the repository record still being written.
- **Not checked:** how the new fields and views render in the Notion app (read back through the connector only).
