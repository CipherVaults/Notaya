# EX-2026-11-07-51: Notaya Manual, first edition

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Write the Notaya manual, first edition" in Ahmed's Claude project "Notaya" (machine: cloud; no PC tool used). Model configured `claude-sonnet-5-5[1m]` (session configuration; the serving model was not re-read).

Short form (4 sections). Normal size: a new document and records, no app code. Times are UTC: message timestamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `67b27e4` (merge of PR #62); app version 1.5.1 (accepted by Mohamed, 2026-11-05 11:08:39 UTC).
- **Authority:** Ahmed's messages of 23:39:31 UTC ("push full throttle ... don't wait for my authority") and 23:49:01 UTC, relayed by the coordinator at 23:49:23 UTC with the brief: write the first edition of the agreed GUIDE feature, covering what Notaya 1.5.1 does today. Recorded as A43 (EX-48); no new amendment.
- **Scope and write area:** `docs/manual/` (the HTML source and a short README), this report, its index row, session entry 45 and its index row, one STATUS block, one ledger row, the claims row. Outside the repository: the published artifact and a copy of the source in the project's shared notes folder.
- **Out of scope:** app code, dependencies, security behaviour, the in-app manual (planned for 1.10), screenshots of the real app, the Notion hub, the handover guide for Mohamed (A39; see section 4).
- **Returns:** the artifact link and draft PR #63. **Stop point:** the manual, its source and these records; nothing is built.
- **Branch:** `claude/project-thread-7ast8y`, from `main` `67b27e4`.

## 2. Work Performed

1. Read `CLAUDE.md`, `docs/STATUS.md` (first 60 lines), the claims board, the full feature inventory of 2026-11-07 (all 320 lines), the version plan (summary, waves 1 to 4, "Later") and the `v1.5.1` release notes. Spot checks: the Sunrise theme tokens in `src/styles/app.css`; the PDF Unicode working notes (Arabic, Hebrew and CJK are supported in PDF since V1.4.0, so the manual says that and keeps the inventory's Greek, Cyrillic and emoji gap).
2. Wrote one HTML page (Sunrise colours, Inter and JetBrains Mono, light and dark): Getting started; Find your way around (with a layout sketch); Notes and pages; Databases; Sticky notes and calculator; Import and export; Look and Windows; Privacy and backups; Keyboard shortcuts; Limits and known issues; Coming soon (versions 1.6 to 1.15 in one paragraph each, marked as a draft, no dates); About this edition (sources, how to keep it current, change log). A search box filters the entries.
3. Used Notaya's own names only; the manual names no other product's brand for a Notaya feature. "Notion" appears only where it is the import source, and "Word", "PDF", "CSV" only as file formats.
4. One check render (Playwright, 1280 px light and 400 px dark): no horizontal overflow, header, contents and first section read well. Published the artifact; saved the source to the shared notes folder and to `docs/manual/`.
5. Claimed the work first (claims row, PR #63), then added these records.

## 3. Verification and Evidence

- **Checks run:** one local render at two widths (no overflow, no console check run); a byte comparison of the repository copy with the shared-folder copy.
- **Not run:** the app. **Every statement comes from the inventory and the release notes, not from running Notaya.** Nothing was checked against the exact commit `67b27e4` beyond reading the inventory, which read `9474511`; only records changed between the two. Not checked: that each keyboard shortcut works in the real exe; WebView2's own Ctrl+F behaviour (the manual says only that Notaya has no find bar of its own); the artifact's rendering in the viewer's dark mode (checked only in a local browser); a read by Ahmed.
- **Result state:** produced; verified only as above; integrated on merge; **not accepted** (acceptance is Ahmed's word).

## 4. Outcome and Open Points

- The manual is private to Ahmed until he shares the artifact link. The edition notes say how each release should refresh it (also in `docs/manual/README.md`).
- **Not done here, for the coordinator or Ahmed:** the handover guide for Mohamed (A39, both copies) has no line for this manual yet; screenshots of the real app (needs a PC session on throwaway data); the Notion hub entry for the GUIDE card.
- **Ledger:** this session's usage is **not measured** here; one row says so, for the coordinator to fill in later.
- Next permitted action: Ahmed reads the manual and says what to change; the next edition follows the next release.
