# EX-2026-11-07-50: Phase 2 version plan (V1.6 to V1.15)

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Plan versions 1.6 to 1.8 and beyond" in Ahmed's Claude project "Notaya" (machine: cloud; the session is bound to Ahmed's Gaming-PC but no PC tool was used). Model configured `claude-opus-5-5[1m]`, effort high (from the session's own configuration). One helper (Sonnet 5.5) updated the Notion board cards.

Short form (4 sections). Normal size: a plan and records, no app code. Times are UTC: message timestamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `9e3e4e3` (merge of PR #60); app version 1.5.1.
- **Authority:** Ahmed's message of 23:39:31 UTC (project chat, relayed by the coordinator at 23:39:51): plan and stage every discussed feature across versions, small and big, with the AI CLI list, update the Notion hub, use parallel work, "don't wait for my authority" for the planning work. This is **Phase 2 (planning)** in Ahmed's order of work (21:14:10). It is **not** a go to build anything. The coordinator's brief added: dependencies (AI CLI and security first, whiteboard before Notebook mode, dashboards in three steps, don't cram 1.6), fold in the Notion parity study and the AI CLI action list, never touch the PFS hub.
- **Scope and write area:** `docs/plan/2026-11-07-version-plan.md` (new; copy of the shared-files plan), this report, its index row, session entry 44 and its index row, one STATUS block, one ledger row, the claims row; outside the repository, the plan in the project's shared files and the Notaya Notion hub (Version Feature Board cards and properties, Human Roadmap page).
- **Out of scope:** app code, dependencies, security behaviour, the plan snapshot, the authority table (the Phase 1 records thread, EX-49, records Ahmed's 23:39 message), the handover guide (the same thread is editing it; see section 4).
- **Returns:** draft PR #62. **Stop point:** the plan, the hub and these records; nothing is built.
- **Branch:** `claude/project-thread-p6lh4r`, from `main` `9e3e4e3`.

## 2. Work Performed

1. Read in full: the feature inventory (shared files), the dashboards, whiteboard and import/export studies (design and staging sections), the Notion parity study (23:41 UTC), the AI CLI action list (23:44 UTC; sections 1 to 3, 10 to 12 in full, the area tables by heading), `docs/STATUS.md` "Exact next permitted action", A37 and its dated note, the project memory list of agreed features, the model-choice guide, and every V1.6 to Later card on the Version Feature Board.
2. Wrote the plan: four waves, versions 1.6 to 1.15, every item with a plan ID, a size (XS to XL), what it means in plain words, dependencies and gates (security behaviour, dependency, vault format). Folded in the parity study's five P1 page gaps (1.8) and its migration track (pilot after 1.8, bulk after 1.9), and the AI CLI action list's steps A and B (1.6) and C (1.7).
3. Notion hub: added the versions V1.9 to V1.15 and the properties Effort, Plan ID, Gate and Depends on to the Version Feature Board; updated the existing cards to their new versions; created cards for the new plan items (the helper's counts are in the PR description); rewrote the roadmap page's "Where we are", "The next step" and the version table, added "The plan in four waves" and a journey line. The PFS hub was not touched.
4. Claims row first (EX-50 reserved; EX-49 is the Phase 1 records thread), then this report and the records.

## 3. Verification and Evidence

- **Checks run:** the plan's item IDs were cross-checked against the board with one SQL query after the cards were written (helper's report); the repository copy was checked to differ from the shared-files copy only by its header note; EX-50 and session 44 were checked free on `origin/main` and every remote branch (highest in use: EX-49 on `claude/project-thread-1qq69j`, session 42 on `main`; 43 is left to the Phase 1 records thread).
- **Not run:** any app check (no code changed); a read by Ahmed.
- **Result state:** produced; verified only as above; integrated on merge; **not accepted** (acceptance is Ahmed's word).

## 4. Outcome and Open Points

- Ahmed has eight numbered questions in the plan (section 10), plus the AI CLI list's ten (its section 12), which shape the 1.6 spec.
- **Handover guide (A39):** this is a milestone, so both copies of the guide need a line pointing to the plan. The Phase 1 records thread is editing the guide right now; the line was asked of that thread to avoid two edits of the same file at once. If it is not there when this PR merges, the next records PR adds it.
- **Ledger:** this session's usage is **not measured** here (a running session's own record carries no usage block); the coordinator reads it later.
- Next permitted action (planning only): the 1.6 spec and the dependency and licence study (plan section 9), once Ahmed has read the plan. Building waits for his word.
