# EX-2026-11-08-54: version plan updated with Ahmed's 23:58:44 answers and the Notion survey gaps; 1.6 spec committed

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Plan versions 1.6 to 1.8 and beyond" (machine: cloud). Same session as [EX-2026-11-07-50](2026-11-07-version-plan.md) and [EX-2026-11-07-53](2026-11-07-version-plan-answers.md). Model configured `claude-opus-5-5[1m]`, effort high. Two helpers (Sonnet 5.5) changed the Notion board cards.

Short form (4 sections). Small to normal: a plan update, one copied document and records. Times are UTC: message timestamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `714cbca` (merge of PR #66).
- **Authority:** Ahmed's message of 2026-11-07 23:58:44 UTC (project chat; tablets, emoji in PDF, Excel charts and Excel import), relayed by the coordinator; the coordinator's notes of 23:59:42 UTC (fold the Notion survey's round-2 gaps into the plan, order is the planner's call under Ahmed's 23:54:20 "it's up to you", leave questions 30 to 33 open) and 2026-11-08 00:02:40 UTC (correct the exchange folder to "next to" the data folder, as in the 1.6 spec; commit the 1.6 spec; merge). The coordinator's notes are requests, not Ahmed's permission; the planning itself rests on Ahmed's 23:39:31 UTC go for planning.
- **Scope and write area:** `docs/plan/2026-11-07-version-plan.md`; new `docs/plan/2026-11-08-v1-6-spec.md` (a byte-for-byte copy of the shared file `notes/2026-11-08-v1.6-spec.md`, written by the thread "Write the 1.6 spec"); one authority row; this report and its index row; a dated addendum to session entry 44; one STATUS block; the claims row. Outside the repository: the plan in the shared files, the Notion board cards, the project memory note on the plan.
- **Out of scope:** app code; any build go; the content of the 1.6 spec (copied, not reviewed or changed here); questions 30 to 33.

## 2. Work Performed

1. **Ahmed's 23:58:44 answers** (plan section 0): tablet support is broad (2.3-2: pressure from any tablet or pen screen Windows supports); Greek, Cyrillic and emoji in PDF moved up from 1.9-3 to a new **1.7-13** (critical); **1.9-5** Excel export now includes charts (library `rust_xlsxwriter`, a dependency gate); new **1.9-11** Excel import into databases (library `calamine`, a dependency gate). Gates table and sizes updated.
2. **Notion survey round 2** (parity study section 0), placed by the planner:
   - **1.8:** 1.8-6 adds byte-exact code copy; 1.8-10 file blocks raised to P1; new **1.8-11** links to one block in another page (stable block ids), **1.8-12** contents block, **1.8-13** text direction per block keeping invisible direction characters exactly, **1.8-14** remember each item's Notion id on import.
   - **1.9:** new **1.9-12** saved views carried across, **1.9-13** property descriptions, **1.9-14** row page layouts, **1.9-15** Status groups (an import check only: the groups already exist in `database/types.rs`, checked on `714cbca`), **1.9-16** page templates, a default per database and repeating templates, **1.9-17** sync ledger.
   - **Later:** synced blocks and a button block (not readable through the connector; listed in the "did not carry" report or rebuilt by hand).
   - **Migration track:** M0 and M1 marked done; the pilot after 1.8 runs on Workspace Home, the Projects and Ideas registry and one finance database, and is re-run into a fresh throwaway vault instead of waiting for re-import; hub home pages (about 15) are rebuilt from Claude's read through the connector (M4); ongoing backup manual and AI-driven; the formula route waits for question 30.
   - **AI CLI (section 5.3):** open a page by Notion id (1.8); list the sync ledger and compare a page with its Notion original (1.9).
   - **Decision against the survey's suggestion:** the survey (0.9b) suggested pulling import-into-place, re-import and original dates into 1.8. Not done: 1.8 would grow from about 37 to about 47 points, and a pilot on throwaway data can be re-run from scratch. Recorded in the plan under 1.8.
3. **Exchange folder:** the plan said "inside Notaya's own data folder"; corrected to "next to" it (the coordinator's default relayed 23:55:39 UTC and used in the 1.6 spec, so the data folder keeps only encrypted files). Ahmed's own word was "containable". Same correction in the project memory note.
4. **1.6 spec** committed as `docs/plan/2026-11-08-v1-6-spec.md` (unchanged copy).
5. **Notion board** (helpers): 1.9-3 card moved to 1.7-13; 1.9-5 retitled with charts; new cards 1.9-11, 1.8-11 to 1.8-14, 1.9-12 to 1.9-17; 1.8-6, 1.8-10, 2.3-2 and two Later cards rewritten; 1.9-15 corrected to "check only". Nothing deleted.

## 3. Verification and Evidence

- **Checks run:** the repository plan differs from the shared-files plan only by its header note (diff); the spec copy is byte-identical to the shared file (`cmp`); every plan ID 1.8-11 to 1.8-14 and 1.9-11 to 1.9-17 appears once in the plan's tables; the Status groups claim was checked in `src-tauri/src/database/types.rs` lines 84 and 610.
- **Notion:** the helpers paged the board's "All features" view to check that no new Plan ID existed before creating (the SQL query allowance had run out). Results are the tools' confirmations; the cards were not fetched again. Two older duplicate Plan IDs were seen and left alone: "1.6-10" (Import/Export buttons and the "Export as" button) and "2.0-1" (corkboard and swipe-select).
- **Not run:** app checks (no code changed).
- **Result state:** produced; verified only as above; integrated on merge; not accepted.

## 4. Outcome and Open Points

- Open for Ahmed: the plan's five questions (section 10) and the survey's questions 30 to 33 (formula route, the three German course copies, how Notion stores right-to-left, notebook layout). None of them changes the version order.
- 1.9 is now the heaviest of the later versions (about 46 points); the plan names the items that can split off into a short 1.9.x if needed.
- **Handover guide (A39):** the plan's version shape changed again (survey items); both copies should get a line at the next guide update (left to the thread that keeps the guide).
