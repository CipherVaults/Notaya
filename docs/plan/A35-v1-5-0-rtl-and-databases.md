# A35: Notaya V1.5.0, Arabic / right-to-left and full local databases

**Status:** approved to build by the owner, 2026-11-03 19:28:12 UTC ("can you do everthing in this version except the automations"). Supersedes the three-release staging in section 3: everything below goes into V1.5.0 except automations (and the Button property, which needs them). Decisions B to E taken as recommended (vault option 1, files in the vault with a 50 MB cap, Notaya's own look); PDF Arabic included because the owner said "everything" (he may drop it). **Owner 2026-11-03 19:30:11 UTC:** file cap is **50 MB** (not 50 MB); PDF export must print Arabic, Hebrew and Chinese/Japanese/Korean letters instead of "?" (in V1.5.0, no longer optional). Not built yet when this file was written. Baseline `origin/main` `ee8c32e` (V1.4.0, released and accepted 2026-11-03 18:23:19 UTC).
**Owner's message:** 2026-11-03 19:20:58 UTC ("if arabic import from other thing will be in the left side, not right like english / Full database like notion i mean everthing") and 19:22:15 UTC ("tell me if you got the full databases from notion and types").
**Correction 2026-11-04 09:05:57 UTC (owner: "Can you add the automations / Let's try to finish them all"):** automations and the Button property are now in V1.5.0; the exclusion above and in section 3 no longer applies. They stay local (no network triggers or actions, no programs, nothing sent out): triggers are a row added, a property changed and a date arriving; actions are set a property, add a row, show a message in Notaya and move to trash. See P29.
**Result state of this file:** produced (not verified, not accepted). Local only: no AI, network, cloud, sync or sharing.

---

## 1. Arabic and right-to-left (RTL) text

**What I checked in the code (V1.4.0):** the editor has no text-direction handling at all (no `dir`, no RTL style anywhere in `src/`). So Arabic, whether imported or typed, sits on the left like English. The HTML export already writes `dir="auto"` on blocks, so the exported file is fine in a browser; the editor is the problem. My reading of your message: **imported Arabic shows aligned left, and it should be right-to-left and aligned right.** I assume typed Arabic should do the same.

**What V1.5.0 does:**
1. Every text block (paragraph, heading, list item, quote, callout, shelf title, table cell, to-do) decides its own direction from its first strong letter: Arabic or Hebrew goes right-to-left and right-aligned, English stays left. Mixed pages work paragraph by paragraph.
2. Lists, check boxes, quote bar, callout icon, table columns and the shelf arrow flip for RTL blocks.
3. A ribbon button and a "/" item to force left-to-right or right-to-left on one block, for the cases the automatic choice gets wrong (numbers-only lines, mixed lines).
4. Import keeps direction: HTML `dir="rtl"` and `lang="ar"`, Word right-to-left paragraphs (`w:bidi`), Markdown and plain text by the automatic rule, pasted text too.
5. Export: HTML already does it; Word gets the right-to-left paragraph flag; Markdown has no direction to keep (plain text, the viewer decides).
6. Search, sort and word count keep working on Arabic text (checked by tests).

**Not in the first cut (decision A):** the **PDF export prints Arabic as "?"** because it uses only the standard PDF fonts. Fixing it means embedding an Arabic font and joining the letters correctly (Arabic letters change shape by position). That is real work (a font file bundled, a shaping routine written by hand because no new libraries). Recommendation: **do it in V1.5.0 if you need to print Arabic pages to PDF**, otherwise leave it and Word/HTML export are the Arabic-safe routes. Tell me which.

**How I will test:** unit tests in Rust for import and export direction, type check and mock-run of the editor in a browser (Arabic samples, screenshots), then your PC round on throwaway data. I cannot see Arabic shaping in the real WebView2 window from the cloud, so that part is checked on the PC.

**Clarified by the owner 2026-11-03 19:26:53 UTC:** when a library or backup is imported or restored, Arabic must stay right-to-left and right-aligned and English stay left, and the same after export. So direction has to survive every route, not only the editor:
- **Backup restore / opening an older vault:** direction is worked out from the text itself, so old Arabic pages show correctly with no migration. A manual choice (button) is stored in the page and travels with the vault and backups.
- **Import (HTML, Word, Markdown, text, PDF text, Notion zip, Notaya's own exports):** HTML `dir`/`lang`, Word `w:bidi`, otherwise the automatic rule, per paragraph.
- **Export:** HTML (`dir` on every block, kept), Word (right-to-left flag per paragraph, new), PDF (only if decision A is yes), Markdown (plain text: the automatic rule decides when it is read back; a manual override is not kept in Markdown).
- **Test:** a round trip (Arabic and English mixed page -> export -> import -> same directions) for HTML and Word, and a restore test from a backup file.

---

## 2. Databases: do I have the full Notion list?

**Short answer:** I have a researched list from Notion's own help pages (PR #45, read 2026-11-03), **not** checked against your live Notion (you postponed that, and I will not read it). Please check the lists below; tell me anything missing.

### 2.1 Property types (22 in Notion's list, plus the title that every row has)

| # | Notion property | Notaya plan |
|---|---|---|
| 1 | Title (the row's name, always there) | Yes. Row = a page, as in Notion |
| 2 | Text | Yes |
| 3 | Number (formats: number, percent, currency, bar/ring display) | Yes |
| 4 | Select | Yes (colored options) |
| 5 | Multi-select | Yes |
| 6 | Status (To do / In progress / Done groups) | Yes |
| 7 | Date (single, range, time, reminder) | Yes (reminder = local Windows notification, tray, title only; can be turned off) |
| 8 | Person | Plain text names (no accounts) |
| 9 | Files and media | Yes, stored encrypted in the vault (size cap, see decision D) |
| 10 | Checkbox | Yes |
| 11 | URL | Yes (opens in the browser only when you click) |
| 12 | Email | Yes |
| 13 | Phone | Yes |
| 14 | Formula | Yes (own expression engine in Rust) |
| 15 | Relation (two-way) | Yes |
| 16 | Rollup | Yes |
| 17 | Created time | Yes |
| 18 | Created by | Plain text (the owner's name) |
| 19 | Last edited time | Yes |
| 20 | Last edited by | Plain text |
| 21 | Button (runs actions) | Yes, later stage (needs the local rules engine) |
| 22 | ID (auto-number with prefix) | Yes |
| 23 | Place (map location) | Address text and coordinates, **no map tiles** (needs network) |

### 2.2 Database features

- **Where:** database as a full page, **inline in a page**, linked view of a database in another page; one database holding several "data sources"; lock database; rows open as full page, side peek or center peek; rows can have sub-pages.
- **Views (7):** table, board, calendar, timeline, list, gallery, chart. Several views per database, saved, each with its own hidden properties, sort, filter, group. Freeze columns, wrap cells, resize and reorder columns, row numbers.
- **Filters:** nested up to 3 levels, AND / OR, per property type operators; **sorts** (several); **group** and **sub-group** (with hidden empty groups, counts); **calculations** in column footers (count, count values, unique, empty, % empty, sum, average, median, min, max, range, earliest, latest, date range, checked %).
- **Charts:** bar, line, donut/pie, number; group by, aggregate, colors, plus a chart dashboard of several charts and number tiles. Drawn by hand as SVG: **no new library**.
- **Formulas:** Notion formula language (math, text, date, logic, `if`, `ifs`, `map`, `filter`, `sum`, `prop()`, lists), with errors shown like Notion.
- **Relations and rollups:** one-way and two-way relations, rollup with every calculation, relation limit of one or many, **sub-items** (parent/child rows) and **dependencies** (blocked by / blocking), both feeding the timeline.
- **Templates:** database templates (including a default template), repeating templates (local reminders), page templates.
- **Forms:** only a local quick-add dialog (shared web forms are out: sharing).
- **Import / export:** CSV in and out (with type guessing and a column-mapping step), **import of a Notion Markdown-and-CSV export** into real databases (property types guessed, relations rebuilt by title; views, filters and formula text are **not in Notion's export** so they come back as values and default views), export to Markdown/HTML/Word/PDF tables.
- **Automations:** local rules ("when status changes, set the date"), no network triggers. Last stage.
- **Search and speed:** search across rows and properties; an index so thousands of rows stay fast.
- **Out (your rules):** people directory, permissions, column permissions, shared/personal filters between people, synced external databases (GitHub, Jira...), AI properties and autofill, API, webhooks, publishing, comments between people.

### 2.3 What I could not confirm

- Exact current Notion limits and 2026 additions I only know from short summaries (automation details, dashboards, tabs). I will list gaps I find as I build.
- How Notion's real export writes relations and formulas: **unknown until you give a real export** (Phase 0 in PR #45). Without it, import of Notion databases is built against the documented shape and marked unverified.

---

## 3. The big problem: size, and how to build "everything"

Databases with all of the above are the largest thing Notaya has had: PR #45 sized them XL + L + XL. One release with all of it would take many sessions and, with your usage limit, a very long wait before you get anything usable.

**Recommendation (decision B): build everything, in three stages, one release per stage**, each fully tested before the next starts:

| Release | Contents |
|---|---|
| **V1.5.0** | Arabic / RTL (all of section 1) + **database core**: data model, all property types except formula / relation / rollup / button, table view, sort, filter, group, calculations, saved views, database as a page and inline, rows as pages, templates, CSV import/export, Notion CSV import, search of rows |
| **V1.6.0** | Board, calendar, timeline, list, gallery, linked views, sub-items, dependencies, peek modes |
| **V1.7.0** | Formulas, relations, rollups, charts and dashboards, buttons and local automations, repeating templates |

If you prefer **everything in one release**, say so; it works but the first thing you can test comes much later. Recommended: three releases.

---

## 4. Vault compatibility (decision C)

**What I found:** a vault stores each record by kind (`notebook`, `section`, `page`). If V1.4.0 meets a kind it does not know, it treats the whole vault as **damaged and refuses to open it** (`Error::Corrupt`, `vault/mod.rs:160`). So once V1.5.0 creates a database, **V1.4.0 cannot open that vault**. A V1.5.0 vault without any database stays fully readable by V1.4.0.

Options:
1. **(Recommended)** New record kind `database`; the vault file format stays the same. V1.5.0 makes an automatic backup copy right before the first database is created, and warns once: "after this, an older Notaya cannot open this vault". V1.5.0 opens every older vault. Simple, safe, one-way.
2. Bump the vault format version so V1.4.0 shows a clean "made by a newer version of Notaya" message instead of "damaged". But that would apply to every vault, even ones with no database; more migration risk. Not recommended.
3. Store databases inside ordinary page records so V1.4.0 still opens the vault. V1.4.0 would then show database rows as empty pages and **silently destroy database data when it saves**. Rejected: data loss.

---

## 5. Other decisions

- **D. Files in rows (and later, pages):** store them encrypted inside the vault (vault grows; I suggest a cap of 50 MB per file as for pictures) or keep only a link to the file on disk? Recommended: inside the vault, 50 MB cap.
- **E. Look:** keep Notaya's own style (as in A33), not a Notion copy. Recommended.
- **F. Notion export (Phase 0):** later, when you want it. Not needed to start.

---

## 6. How the work runs (so nothing surprises you)

1. Branch from `origin/main`, claims row first, draft PR, never merged by me; you merge ("Create a merge commit") and say "publish".
2. Rust logic first (data model, filters, sorts, grouping, calculations, CSV, RTL import/export), tested in the cloud with `cargo test --lib` under Wine; UI type-checked, built, and mock-run in a browser with screenshots.
3. Security: every new way data leaves the vault (CSV export, files opened by other programs, reminders) gets a row in `docs/security-design.md`; the security-review skill is run on the diff before I hand you a code PR.
4. PC testing: at most 2 full rounds per release, throwaway data in `E:\Project\Notaya\Notaya Tests Excluded`, never your real vault; bugs fixed on the spot and only that area retested.
5. Records: amendment A35 (your words), an execution report with its index row, STATUS lines, resume note if I stop.

## 7. What I need from you to start

Reply **"ok recommendations"** to accept: PDF Arabic in V1.5.0 only if you say so (A), three-release staging (B), option 1 for the vault (C), files in the vault with a 50 MB cap (D), Notaya's own look (E). Or change any letter. I will not build before your OK.
