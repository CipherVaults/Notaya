# Notion to Notaya: migration plan (Claude reads Notion through the connector) and its limits

**Status:** a plan for the owner to decide on. **Nothing is built and nothing has been copied.** Companion to [notion-offline-backup-roadmap.md](notion-offline-backup-roadmap.md) (the Notaya features to build). Baseline `origin/main` `6a8af96`. Written 2026-11-03 (system clock, UTC) by a cloud thread. Same record: [EX-2026-11-03-41](../execution/2026-11-03-notion-offline-roadmap.md). **This version replaces an earlier draft (pushed 19:10 UTC) that proposed a separate API tool; the owner meant something else (section 1).**

## 1. What the owner asked

- 2026-11-03 19:03:04 UTC, Mohamed: "if i told you to copy all thing from from notion api to notaya / if we built every feature what it will be".
- 19:06:43 UTC (edited 19:08:33): "give me a plan and what limitations we will face / consider like every future we did already and using export of Notion API you will do it / mix of MD and html or pdfs from notion".
- 19:14:02 UTC: "No i meant you will read as connector notion files and do all of this things assuming".

Read as: **Claude (this assistant) reads the owner's Notion directly through the Notion connector in the session and carries the content into Notaya**, assuming every Notaya feature already exists. Exports (Markdown, HTML, PDF) are used as a mix where they help. **No separate tool, no token to create, no HTTP code in the repository.** Notaya stays offline (P5). The connector is already attached to this session and signed in as the owner (read at 19:15 UTC: workspace "mohamed sayed's Space").

## 2. What a first read-only look found (19:15 UTC)

I ran two searches, one workspace check and one database fetch, and read **titles, paths and one database schema only**. No page body was read, nothing was written to Notion, and nothing is copied into this repository. A search returns at most 50 results, so this is **a sample, not a count**.

- The workspace is **small and recent**: pages dated 2026-10-05 to 2026-10-22.
- The 50 results were **all pages**, in a few groups: one large nested page tree of project-system documents (a parent page with child pages, three or four levels deep), trip-planning pages (one in Arabic), product-comparison pages, and the starter pages Notion creates ("Welcome", "To Do List").
- **One database** was seen: Notion's starter "To Do List DB" with **three properties** (Name, Done, Due Date) and four starter rows. No formulas, relations, rollups, dashboards or other databases showed up in this sample.
- Not yet looked at: page bodies (block types, images, files, columns, toggles, comments), saved views of the database, anything beyond the first 50 results, the sidebar's other sections.

**What this means for the plan.** If the sample is representative, the owner's real content is **mostly text pages in a nested tree**, which Notaya V1.4.0 can already hold. The heavy parts of the roadmap (databases with formulas and relations, dashboards, seven view types) may matter little for *his* content. Reading everything is small enough for Claude to do by hand through the connector. That makes a full copy much cheaper than the plan in the earlier draft, and it changes the order of work: **copy first, build only the features his content needs.** A complete inventory (step M0 below) will confirm or correct this.

## 3. How the copy works

1. **Inventory (read only).** Claude walks the whole workspace through the connector: `search` for entry points, `fetch` for each page and database. Output: a report with counts of pages and databases, block types in use, property and view types, number of images/files and their size, comments, and anything the connector could not return (`truncated`, `unknown_block_count`).
2. **Read each page.** `fetch` returns the page as **Notion-flavored Markdown**: headings, lists, to-dos, toggles (`<details>`), toggle headings, callouts, columns, tables, equations, code, page references and mentions, colors, icon, plus metadata. Databases return the schema; saved views come back as `view://` entities with filters, sorts and display settings; rows come from `query_data_sources`; comments from `get_comments`; uploaded images and files through `get_file_download_urls` (links expire, so files are saved at once).
3. **Write a migration folder** in the project files area: one Markdown/JSON file per page, one JSON file per database (schema, views, rows), the downloaded files, and a manifest with ids and a checksum for each file.
4. **Import into Notaya, offline.** Notaya reads the folder with its own import (same safety rules as today: no paths outside the folder, size limits, Cancel, all-or-nothing). Until the databases and other features exist, the import maps what Notaya has and writes the rest into the report.
5. **Check.** The owner exports once from Notion (Markdown & CSV, HTML and, if wanted, PDF), and a check compares page counts, row counts and a sample of text between the Notaya copy and the exports. The exports also stay as a plain-file backup that does not depend on Notaya.

**What each source is for.**

| Source | Role |
|---|---|
| **Connector (Claude reads)** | Structure and content: pages as Notion Markdown, database schemas, views, rows, comments, file links |
| **HTML export** | A cross-check of text and look (fonts, icon, cover, colors, columns), and a fallback for anything the connector truncates |
| **Markdown & CSV export** | A readable safety copy, and a check on page and row counts |
| **PDF export** | A picture of how a page looked, for comparing by eye. Notaya reads only the text of PDFs, so a PDF cannot rebuild structure |

Which of the three exports adds anything the connector lacks is **not verified**; the inventory step decides whether the owner needs to export at all.

## 4. What carries over (assuming the Notaya features exist)

| Notion item | Via the connector | In Notaya |
|---|---|---|
| Pages, sub-pages, nesting, icon, cover | Yes | Same tree. How Notion's top-level pages map to notebooks and sections is a decision (D-M3) |
| Text, headings, lists, to-dos, quotes, dividers, code, equations, callouts, toggles, toggle headings | Yes (Markdown tags) | Same blocks; colors mapped to Notaya's 18 colors |
| Columns, tables, table of contents, tabs, synced blocks | Yes, in Notion Markdown (synced blocks: original and reference) | Same blocks; synced blocks re-linked |
| Images and files | Download links (expire) | Downloaded once, stored encrypted; subject to Notaya's size limits |
| Bookmarks, embeds | Link and title | Link cards; nothing is fetched or played |
| Mentions of pages and dates | Yes | Page links, date chips; people become plain names |
| Databases and rows | Schema, rows | Real databases when Notaya has them |
| Select, status, multi-select, date, number, checkbox, URL and so on | Yes | Same property types |
| Formulas, rollups | Expression and definition in the schema (to be confirmed per database) | Re-created if Notaya's formula engine knows the function, else last value kept and listed |
| Relations | Page ids in the data | Rebuilt exactly |
| Views (table, board, list, gallery, calendar, timeline, chart) | `view://` with filters, sorts and display settings | Rebuilt where Notaya has the same option, the rest listed |
| Dashboards | Probably (the API reads them); **not tested through the connector** | Rebuilt as a Notaya dashboard page |
| Comments | `get_comments` (open and resolved not verified) | Page and block comments, author as text |

## 5. Limitations (what we will face)

**From the connector**
- **Cost and effort.** Every page is read and written *through Claude*, so each page uses Claude's working capacity twice (reading it, then writing it to a file). For a small workspace that is fine; for thousands of pages it would not be, and the owner's usage limit is tight. The inventory gives the real number before any copying.
- **Exactness.** Text passes through Claude, so a mistake in copying is possible. Counts and checksums per file, plus the comparison with the exports (step 5), are there to catch it. Bulk text should be taken from the exports by program where possible, and the connector used for what exports lack (views, schema, comments).
- **Access.** The connector sees what the owner's account can see. Notion's `unknown_block_ids` / `truncated` markers show blocks it could not return; those are listed, not guessed.
- **Not readable or not copied:** automations, button actions, permissions and sharing, page history as such, AI content and agents, analytics, trash. Buttons and forms arrive as unsupported or as settings only.
- **Files** are signed links that expire; files must be saved when the page is read.
- **Rate limits** exist on Notion's side (the connector's own limits are not documented in what I read); a big workspace needs pauses and a resumable run.
- **Not tested yet:** views and dashboards through the connector on a real database with many views, comments, large files, columns/toggles in a long page.

**From Notaya (even when every phase is built)**
- **A copy, not a link.** Notion and Notaya diverge after the copy. No sync (excluded). A second copy is a new import (roadmap D4).
- **Left out on purpose:** AI, integrations, sharing, live collaboration, embeds that play from the web, link previews that load, mobile and web.
- **Vault limits today:** 5,000 pages per import, 25 MB per picture, 32 MB per record; videos and big PDFs need a rule (roadmap D5).
- **The vault format changes** when databases arrive; an older Notaya may not open a newer vault.
- **PDF export** prints Arabic, Hebrew and CJK letters as "?" (standard fonts); one of the owner's pages is Arabic, so **Word, Markdown or HTML exports from Notaya are the safe ones for that text**. The import of Arabic text from Notion through the connector is plain Unicode and not affected; right-to-left display in the Notaya editor should be checked on that page.
- **Windows only; Notaya's own style**, not Notion's.

**Process and security**
- **Your content passes through Claude's session** while it is read and written. It is your own Notion and your own account, read-only, but it is in the conversation and the temporary files of that session. Nothing is written to Notion. The migration folder holds all content **unencrypted** until imported and deleted.
- **Records:** every step is a report (what was read, counts, what was left out); no page text goes into the repository records.
- **Testing:** the first import into Notaya runs on the owner's PC on throwaway Notaya data, never his real vault; cloud threads can test the mapping on small anonymised sample files.

## 6. Steps (each one is a report; the owner decides before each)

1. **M0: Inventory (S).** Read-only walk of the whole workspace through the connector; report only (counts, types, sizes, gaps). Nothing copied. **Needs the owner's yes.**
2. **M1: Decide what to build.** From the report, shrink or reorder the roadmap: if the content is text pages plus a starter to-do list, Phases 4a and the simple parts of Phase 1 may be enough for a first complete copy.
3. **M2: Copy pages (S to M).** Pages and files to the migration folder; Notaya imports with today's V1.4.0 blocks.
4. **M3: Copy databases (M).** After Notaya has databases; schemas, rows, views.
5. **M4: Formulas, dashboards (M), only if M0 finds any.**
6. **M5: Check (S).** Counts and samples against the exports; list of what is missing.
7. **M6: Finish.** A final copy on the day the owner wants his backup; PC check on throwaway data first.

## 7. Decisions for the owner

- **D-M1.** May Claude run **M0** now: a read-only walk of your whole Notion through the connector, reporting counts and types only? Recommended: yes.
- **D-M2.** Is it OK that your page text passes through Claude's session during the copy (section 5)?
- **D-M3.** Mapping of Notion's top-level pages to Notaya notebooks and sections: decide after M0.
- **D-M4.** Do you also want the Markdown & CSV, HTML and PDF exports as a plain-file backup and a check? (Recommended, they cost you only a download.)
- **D-M5.** Files hosted outside Notion (links): keep as links, or download them?
- **D-M6.** Roadmap decisions D1 to D6 still stand, but M0 may make many of them unnecessary.

## 8. Not checked

No page body, view or comment was read. The workspace look in section 2 is a sample of 50 search results, titles and paths only. What the connector returns for dashboards, views of large databases, comments and long pages is from its tool descriptions and Notion's developer pages (request limits, block reference, property objects, working with views), not from tests. Sizes are relative.

**Sources:** [Request limits](https://developers.notion.com/reference/request-limits), [Block reference](https://developers.notion.com/reference/block), [Property objects](https://developers.notion.com/reference/property-object), [Working with views](https://developers.notion.com/guides/data-apis/working-with-views), and the export pages listed in the roadmap document.
