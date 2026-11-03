# EX-2026-11-03-41: plan for Notaya as an offline Notion

Short form (4 sections). Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

- **Baseline:** `origin/main` `6a8af96` (V1.4.0 released, not accepted).
- **Owner's words:** 2026-11-03 18:18:10 "i mean all the features but lcally"; 18:20:37 "not ai or integration or netowrk or cloud or sync or sharing / just like 1:1 if something happend to notion i have my offline backup". The coordinator's brief: planning only, no code; a gap list, what makes sense locally, sizes, a phased roadmap starting with databases and charts; a doc under `docs/plan/` in a draft PR; ask which phase to start.
- **Scope and write area:** `docs/plan/notion-offline-backup-roadmap.md` and `docs/plan/notion-migration-plan.md` (new) and the records that own the changed facts (claims row, one proposal line in the amendments record, one STATUS line, this report and its index row). Out of scope: any code, any dependency, any test run.
- **Work size:** normal. **Returns:** a draft PR. **Stop point:** the PR open and the owner asked which phase.

## 2. What was done

- Read `CLAUDE.md`, STATUS, the amendments record, the claims board, A33 and `import-what-carries-over.md`; checked the live code for what V1.4.0 has (editor blocks, `model.rs`, search, import/export).
- Wrote the roadmap: the owner's narrowed goal (import a full Notion export faithfully, then edit it like Notion), the caveat that no real Notion export has been looked at, the feature gap list (blocks, databases, workspace), what is dropped and why, seven phases (0 to 6), design points, and six decisions for the owner.
- 18:30:22 UTC the owner asked for an internet check of Notion's features. Searched and read Notion's help pages, developer docs and release notes (list in section 11 of the plan) and revised the plan: export holds only the current/default view of a database, relations as text, comments only in HTML; 22 property types; new 2026 features (tabs block, dashboards, presentation mode, page archiving, merge cells).
- 19:03:04 and 19:06:43 UTC (edited 19:08:33) the owner asked for a migration plan using the Notion API plus Markdown, HTML and PDF exports, assuming every feature is built. Read Notion's developer pages (request limits, blocks, properties, views) and wrote [notion-migration-plan.md](../plan/notion-migration-plan.md). A first draft proposed a separate API tool; at 19:14:02 UTC the owner said he meant Claude reading Notion through the connector, so the plan was rewritten (steps M0 to M6, limits, decisions D-M1 to D-M6).
- **Read-only look at the owner's Notion (about 19:15 UTC), on the owner's message 19:14:02:** one workspace check, two searches (titles and paths only, first 50 results) and one database schema fetch. No page body read, nothing written to Notion, no content copied into the repository. Finding: small recent workspace, pages only plus one starter to-do database (three properties). It is a sample, not a count.
- Chose the narrowest reading of "1:1": same features and content in Notaya's own style (A33); the look question is left as decision D6.

## 3. Checks

- **Observed:** the V1.4.0 feature list was read from the source and records at `6a8af96`; the next free IDs were checked against all remote branches (EX-2026-11-03-39 and -40 are in use; -41 chosen; A35 left unreserved because this is a proposal).
- **Not run / not known:** no page body read, no view or comment read, no real Notion export, no Notion workspace, no app run, no build, no tests (docs only). What Notion's export contains and the exact file layout of an export are not confirmed by the web pages read; some feature areas came from short page summaries (stated in section 10 of the plan).
- **Result state:** *produced*. Not verified beyond the above, not integrated, not accepted.

## 4. Open points

- The owner chooses a phase and answers D1 to D6 in the document. Recommended: Phase 0, which needs his own Notion export (HTML and Markdown & CSV) in a test folder outside the repository.

- **Postponed by the owner, 2026-11-03 19:18:50 UTC ("postponed for now").** No phase, no inventory (M0) and no copy is started; the two plan documents stay as drafts for when he resumes.
