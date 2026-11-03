# Notaya as an offline Notion: feature gap list and phased roadmap

**Status:** a plan for the owner to decide on. **Nothing is built.** It is not an amendment until the owner picks a phase (then it becomes A35 and the phase is recorded under it). Baseline: `origin/main` `6a8af96` (V1.4.0 released, not yet accepted). Written 2026-11-03 (system clock, UTC) by a cloud thread. **Revised after the web check after a web check of Notion's help pages and release notes** (owner, 18:30:22: "can you search internet for all features till now"); sources in section 11.

## 1. What the owner asked

- 2026-11-03 18:18:10 UTC, Mohamed: "i mean all the features but lcally" (all of Notion's features, but local).
- 2026-11-03 18:20:37 UTC, Mohamed: "not ai or integration or netowrk or cloud or sync or sharing / just like 1:1 if something happend to notion i have my offline backup".

**How this plan reads that.** Notaya becomes the place where the owner's Notion content lives on if Notion is ever lost, and where it stays usable. That gives two jobs, in this order:

1. **Bring a full Notion export in faithfully:** pages, nested pages, databases with their properties, toggles, callouts, columns, files, links.
2. **Edit it afterwards the way Notion allows,** so the content is not frozen: databases you can add rows to, views, formulas and relations, the same blocks.

"1:1" is read as **the same features and the same content, in Notaya's own look** (the owner said earlier it is not a Notion copy and Notion's red inline code is not wanted: see A33). If the owner wants the look copied too, that is a separate decision (section 9).

**Out for good (owner's words):** AI, integrations, network, cloud, sync, sharing. In practice that removes: Notion AI and agents, connectors, the API and webhooks, web clipper, Notion Calendar and Mail, embeds that load from the web, link previews, publishing to the web, guests, permissions, teamspaces, live collaboration, comments between people, page analytics, mobile and web apps, synced external databases. Notaya keeps promise P5 (its own code makes no network connection).

## 2. The big caveat: no real Notion export has been looked at yet

The import in V1.4.0 was built and tested only against **hand-made imitations** of Notion files ([import-what-carries-over.md](import-what-carries-over.md), "Not run"). The web check (section 11) confirmed the general shape of Notion's export but **not** the details that decide import fidelity (file names, ids, how relations and formulas are written). Those need a real export (Phase 0).

What Notion's own help pages and other sources say about the export:

| Export | Holds | Loses or limits |
|---|---|---|
| **HTML** | A zip with one HTML page per page, subpages in folders, images and files saved separately, an `index.html` sitemap. **Comments** (page and block level, resolved and unresolved) and mentioned pages/files/users can be exported | Not documented on the help page (the third-party article does not cover colors, toggles, columns or formulas either) |
| **Markdown & CSV** | Pages as Markdown; each **full-page database as one CSV**; subpage Markdown files; files and images | **Only the current or default view of a database** is exported, so filtered-out rows are absent; **form views cannot be exported**; **relations become text** (linked names survive, the connection does not); callouts are written as HTML; comments are lost |
| **PDF** | Page content as PDF | Not a structure format; custom emoji dropped; subpage ZIP only on Business/Enterprise |

Practical facts from the same pages: "Export all workspace content" (Settings, General) can take **up to 30 hours**, the download link **expires after 7 days**, pages the exporter cannot reach are left out, and **re-uploading an export does not restore a Notion workspace**. Long folder names can break extraction on Windows (260-character path limit; "Create folders for subpages" off helps).

**What this means for the plan:**
- **Views (board, calendar, timeline, chart, form), filters, sorts and groups are not in any export.** Notion's help page only lets you export the current or default view. A database comes back as rows, columns and values. Its views have to be rebuilt in Notaya (with sensible defaults). The API could read them, but the owner excluded network and integrations.
- **Formulas and rollups:** the CSV holds their last value, not the formula. Treated as unverified until a real CSV is seen.
- **Relations** come back as page names; Notaya can rebuild links by matching names, not guaranteed (duplicate names).
- **Comments:** HTML keeps them, so a Notaya "comments on a page" feature (single-user notes, with the author name as text) could carry them over. Newly listed in 4.1.
- Best-looking route (hypothesis): import the **HTML export for pages** (colors, icons, covers, columns, toggles) and the **CSV export for database data**, matched by file names; export a full workspace **once as HTML and once as Markdown & CSV**.

## 3. What Notaya V1.4.0 has today (short)

Notebooks, sections, pages and sub-pages; favorites, trash, search and a command palette; sticky notes with a pop-out calculator; page icon, cover, width and fonts; blocks: headings 1 to 4, bullet, numbered and check lists, collapsible section, **Shelf**, quote, highlight box (callout), code block with syntax colors, divider, equation (TeX), tables, pictures, link to page, links to pages and headings inside text; 18 text and 18 highlight colors; ribbon and "/" menu; export to Markdown, HTML, PDF, Word (one file or ZIP); import of Markdown, HTML, text, Word, PDF (text only), Notion ZIPs and folders into **one new notebook**, all-or-nothing; optional master password with encrypted vault; tray; local error log. Columns and comments exist in the document model but can no longer be created.

**No databases of any kind, no charts, no formulas, no file attachments other than pictures.**

## 4. Feature gap list

Size scale used throughout (rough effort in Notaya's working style, not hours): **S** one small PR; **M** one thread plus its own PC check; **L** two to four threads / PRs; **XL** a release line on its own. "Import" says what the Notion export can feed (unverified until Phase 0). Every row is **local** unless listed in section 5.

### 4.1 Page blocks and page features

| Notion feature | Notaya V1.4.0 | Plan | Size |
|---|---|---|---|
| Text, headings 1-3, bullet / numbered / to-do lists, quote, divider, code, equation, table, image, callout, toggle | Have (headings to 4) | Keep; check fidelity on a real export | - |
| **Toggle headings** (a heading that folds) | Missing | Add; import `<details>` headings | S |
| **Columns** (2-5) | Model exists, cannot be created, imports from HTML | Bring back creating and dragging columns | M |
| **Table of contents** block | Missing | Add (live list of headings) | S |
| **Breadcrumb** block | Missing | Add | S |
| **Page mentions** inside text (`@page`) | Links to pages and headings exist; Notion mentions import as text | Add mention chip; import as link when the page is in the import | M |
| **Date mentions / inline dates** and reminders | Only "insert today" | Date chip; reminders as local Windows notifications from the tray (no network) | M |
| **Bookmark / web link card** | Imports as a "🔖 title" link | Link card from saved title and URL, **no fetching** | S |
| **Embeds** (YouTube, Maps, Figma, PDFs by URL) | Dropped | Shown as a link card only (needs network to render, so out); local PDF/audio/video below | S |
| **File, PDF, audio, video blocks** (attached files) | Imports PDF/Word attachments as a text page; other types listed as left out | Store the file encrypted in the vault, show in the page, "open with" via a safe temp copy; size limit decision needed | L |
| **Synced blocks** | Not supported | Local version (one source block shown in several pages); an export flattens them to copies, so imports cannot relink | M |
| **Buttons** (run actions) | Missing | Only with automations (Phase 6) | M |
| **Page templates** and duplicate page | Missing | Duplicate page (S), page templates (M) | M |
| **Backlinks** ("linked from") | Missing | Add from the existing page-link index | S |
| **Page history / versions** | Whole-vault backups only | Per-page local snapshots (size cost in the vault to decide) | M |
| **Lock page**, **small text**, **full-width per page** | Width exists; others missing | Add | S |
| **Page properties on ordinary pages** | Missing | Comes with databases (Phase 1) | - |
| **Diagrams (Mermaid)**, **table of contents**, **Notion-style markdown shortcuts** | Shortcuts partly (editor default) | Mermaid optional, later; shortcuts check | S |
| **Comments / discussions** | Model remnant, no UI | Between-people comments are out (sharing). But the **HTML export keeps comments**, so a single-user "comments on a page / block" feature (author kept as text) would carry the owner's existing comments over | M |
| **Tabs block** (content in clickable tabs, added 2026) | Missing | Add as a block | M |
| **Presentation mode** (a page shown as slides, 2026) | Missing | Local full-screen slide view of a page; no network needed | M |
| **Page archiving** (hide old pages from search, 2026) | Trash only | Archive state | S |
| **Merge cells in simple tables** (2026), column width, header row/column | Tables resize; no merge | Add cell merge | M |
| **Mermaid diagrams in code blocks** | Missing | Draw offline; no new library without asking | M |
| **Custom emoji** | Standard emoji only | Local custom emoji (small images) | S |
| **Keyboard shortcuts, Markdown shortcuts, `/` and `+` menus, word and character count** | Mostly have | Check gaps against Notion's list | S |
| **Colored blocks** (whole-block color) | Imports as text color | Real block color | S |
| **Page cover positions, icons as images**, custom emoji | Emoji icon + gradient/image cover | Keep; check real export | S |

### 4.2 Databases (the large gap, and what the owner deferred earlier)

| Notion feature | Notaya V1.4.0 | Plan | Size |
|---|---|---|---|
| Database as full page and **inline in a page** (Notion calls the tables "data sources" since 2025: one database can hold several) | Missing | New item type; rows are pages (as in Notion), so rows open as full pages, can have sub-pages, links and search | XL |
| Properties: title, text, number, select, multi-select, **status**, date (with range), checkbox, URL, email, phone | Missing | Phase 1 | L |
| Properties: person, files & media, created time, last edited time, created by, last edited by, ID | Missing | Created/edited time and ID yes; **person / created by / edited by** become plain text (no accounts); files stored in the vault | M |
| Properties: **place** (map location), **button** | Missing | Place as text/coordinates, no map tiles (network); button with automations (Phase 6) | S |
| **Table view**: add / hide / reorder / resize columns, row open, freeze | Missing | Phase 1 | L |
| **Sort, filter (incl. nested), group, sub-group, calculations (sum, count...)**, saved views, several views per database | Missing | Phase 1-2 | L |
| **Board** view, **List** view, **Gallery** view | Missing | Phase 2 (drag cards with the already-bundled drag library) | M each |
| **Calendar** view, **Timeline** view | Missing | Phase 2 | L each |
| **Charts** (bar, line, donut) as a chart view, and **dashboards** (charts + KPIs, 2026) | Missing | Phase 3; drawn by hand as SVG so **no new library** | L |
| **Formulas** | Missing | Phase 3: an expression engine in Rust (testable under Wine). Import carries only the last value, so formulas must be re-typed | L |
| **Relations and rollups** | Missing | Phase 3. Import rebuilds relations by matching titles / ids if the export allows it | L |
| **Sub-items and dependencies** | Missing | Phase 3 | M |
| **Database templates**, recurring templates | Missing | Phase 3 (recurring uses local reminders) | M |
| **CSV import and export** of a database | CSV left out on import | Phase 1 | M |
| Linked views of a database elsewhere | Missing | Phase 2 | M |
| **Forms** (a form that adds a row) | Missing | A local quick-add dialog only; shared forms are out | S |
| Database automations | Missing | Phase 6, local rules engine (no network triggers) | L |
| Lock database, hide properties per view, **open rows as side peek / center peek / full page**, freeze columns, database search, personal vs shared filters (personal only here), advanced filters nested 3 levels | Missing | With views | S |
| **People directory**, column permissions, typed databases | - | Out (accounts, sharing) | - |

### 4.3 Workspace

| Notion feature | Notaya V1.4.0 | Plan | Size |
|---|---|---|---|
| Sidebar tree, favorites, trash, quick find (Ctrl+K), recents | Have (recents to check) | Keep | S |
| **Search** across pages **and database properties**, with filters | Title + text scan only (it walks every item) | Build an in-memory index when databases arrive | M |
| **Import** whole workspace | One new notebook per import, up to 5,000 pages | Raise limits, per-database import, progress, large attachments (Phase 5) | L |
| **Re-import a newer export** to refresh the backup | Not possible (each import = new notebook) | See decision D4: match Notion ids and update. Manual, one way, no sync | L |
| Export | Have (Markdown, HTML, PDF, Word) | Add CSV for databases | M |
| **Templates gallery** | Missing | A few built-in local templates (task list, reading list...) | S |
| **Offline mode**, **version history** (upgraded Oct 2026) | Whole app is offline; whole-vault backups | Per-page history is Phase 4c | - |
| Teamspaces, members, groups, permissions, guest sharing, public pages, page analytics, verified pages | - | Out (sharing) | - |

## 5. What does not make sense locally, and what is offered instead

| Notion feature | Why not | Local alternative |
|---|---|---|
| Notion AI, Agents (own computer/browser), AI meeting notes and transcription, image generation, model choice, credits, enterprise search, Developer Platform, Workers, webhooks, Notion CLI, MCP, database sync (GitHub, Jira, Salesforce), Mail, Calendar scheduling | Cloud / owner excluded | Nothing. (A model on the PC would be huge and unreliable; not proposed) |
| Integrations, API, webhooks, Slack/Drive/GitHub connectors, synced external databases | Network / integration | CSV and file import by hand |
| Web clipper, link previews, live embeds, Notion Calendar, Notion Mail | Network | Link card with the title the owner typed or imported; calendar **view** inside a database (Phase 2) |
| Real-time collaboration, comments between people, @people, permissions, guests, teamspaces | Sharing / accounts | Single user: "person" properties are plain text; reminders go to the owner only |
| Publish to web (Notion Sites), public links | Sharing / cloud | Static HTML export already exists |
| Cloud sync, mobile and web apps | Sync / cloud | The vault is one file the owner can copy himself; unchanged |
| Page analytics, audit log | Multi-user | None |

## 6. Phased roadmap

Each phase is meant to be one release line (the owner's rule: at most two full PC test rounds per release; bugs are fixed on the spot). Releases below are suggestions. Sizes are the sums of section 4.

**Phase 0: Look at a real Notion export (S to M).**
Owner exports his own workspace from Notion (Settings, General, "Export all workspace content"; can take up to 30 hours and the link lasts 7 days; for databases Notion exports only the current/default view): **both** "Markdown & CSV" and "HTML", "Include subpages" on, "Everything" on, into a folder outside the repository (suggest `E:\Project\Notaya\Notaya Tests Excluded\notion-export`). A thread, with the owner's say-so, runs the existing importer on a copy and writes a **fidelity matrix**: what each block type, property and file became, what was lost, and what the databases look like on disk (file names, ids, relation form). Output: a report in `docs/execution/` and a corrected section 2 of this plan. No product code changes except bug fixes the real files expose in the importer. **Why first:** every later phase depends on what the export really contains, and the owner's backup goal is import-first.
*Needs from the owner:* the export, and an answer on whether the files hold private content (they stay on his PC; cloud threads would not see them. A PC (Remote Control) session does the run).

**Phase 1: Databases, table view, CSV (XL; about V1.5.0).**
New data model (see section 7), property types of 4.2 without formula/relation/rollup, rows as pages, table view with add/hide/reorder/resize, sort, filter, group, saved views, inline database block and database as a full page, CSV import and export, **import of Notion database CSV + row pages into real databases** (property types inferred, "all properties" CSV preferred), export of a database to Markdown/HTML/Word/PDF tables. Security rows added to `docs/security-design.md` for any new file or export surface.

**Phase 2: More views (L; about V1.6.0).**
Board, list, gallery, calendar and timeline views; linked views; per-view hidden properties; default views created on import (for example a board when a status property exists), clearly marked as "made by Notaya", since Notion's own views are not in the export.

**Phase 3: Formulas, relations, rollups, charts (XL; about V1.7.0).**
Expression engine, relations and rollups, import that rebuilds relations from titles/ids, sub-items and dependencies, database templates, charts and a chart dashboard (hand-drawn SVG, no new dependency). Imported formulas arrive as values: the app can mark them "formula lost, value kept" and let the owner re-enter them.

**Phase 4: Editor parity (L; about V1.8.0), can be split.**
4a quick wins (S each): toggle headings, table of contents, breadcrumb, backlinks, duplicate page, lock page, block colors, link cards. 4b: columns back, page and date mentions with reminders, synced blocks, page templates, tabs block, comments carried over from the HTML export, presentation mode. 4c: file / PDF / audio / video blocks and per-page history (the two items that grow the vault).

**Phase 5: Big-workspace import and backup refresh (L).**
Limits (5,000 pages, 25 MB per picture, 32 MB per vault record) re-checked against the real export; attachments over the limit; progress and resume; speed of search and lists with tens of thousands of rows (index); optionally **re-import by Notion id** (D4).

**Phase 6: Automations and buttons (L, only if the owner uses them).**
Local rules ("when a status changes, set a date"), button blocks, recurring rows. No network triggers.

Order logic: Phase 0 decides what Phases 1 to 3 must import; databases (the owner's earlier deferral) come before view variety; editor parity can be pulled forward if the real export shows the owner's pages mostly use blocks Notaya lacks (columns, toggle headings, files).

## 7. Cross-cutting design points (for whoever builds Phase 1)

- **Data model.** Add an `Item::Database` record (name, icon, property schema, saved views, where it lives: section, page or inline) and give `Page` optional `database_id` and a map of property values. Rows are ordinary pages. Inline databases are a block in the page document that holds the database id. New fields use `#[serde(default)]` as `icon` and `parent` did. **To check in the code before building:** how a vault with an unknown item kind opens in V1.4.0; if it fails, either a vault-format bump with a warning ("once you create a database, an older Notaya cannot open this vault") or a forward-compatible reader is needed. A backup copy is taken before the first database is written.
- **Where logic lives.** Filters, sorts, formulas and rollups are evaluated **in Rust**, so cloud threads can unit-test them under Wine (the UI can only be type-checked and mock-run). Keys and note contents stay in the Rust core as today.
- **Performance.** Search today scans and decrypts-to-memory every item on each query. A database of thousands of rows needs an in-memory index and paged lists; plan this inside Phase 1, not after.
- **No new libraries.** Charts, CSV, the formula parser and the views reuse what is bundled (`@dnd-kit` for drag, hand-written SVG, the existing hand-written ZIP/XML readers). Anything that needs a dependency is asked first (`CLAUDE.md`).
- **Security.** Every new way data leaves the vault (CSV export, file blocks opened with another program, reminders that show text in Windows notifications and on a lock screen) gets a promise row in `docs/security-design.md` and the unencrypted-file warning the exports already show. Reminders should show a title only, and the owner should be able to turn them off.
- **Records.** Each phase: an amendment (owner's words), an execution report, STATUS lines, then the usual PC tests on throwaway data with a **separate** test folder, never the real vault.
- **Testing a Notion-shaped import.** Once Phase 0 provides real files, add a **small anonymised sample export** to the repository's test data (the owner's choice what to strip) so cloud threads can test importer changes without the owner's PC.

## 8. How big is this, honestly

Notion's full feature set built from nothing is years of work for a team. For the owner's goal (his own content, offline) the heavy and valuable part is **databases (Phases 1 to 3)** and **import fidelity (Phase 0, 5)**; the editor items are mostly small. A realistic reading is: Phases 0 and 1 give a backup that opens and edits his content; Phases 2 and 3 make it behave like his Notion databases; Phases 4 to 6 close the remaining gaps. His usage limit matters: each phase is a release with two PC test rounds, so this is many sessions. Sizes above are relative, not calendar promises.

## 9. Decisions for the owner

- **D1. Start with Phase 0 (real export intake), then Phase 1?** Recommended: yes.
- **D2. Databases get a one-way format change:** after one is created, an older Notaya may not open the vault (a backup copy is made first). OK, or build a forward-compatible reader first (more work)?
- **D3. Views and formulas are probably missing from the export** (section 2). Accept rebuilding them, with Notaya-made default views and formula values kept as text?
- **D4. Re-import to refresh the backup:** keep "each import is a new notebook" (as now), or add a manual one-way "update from a newer export" that matches pages by Notion id (never sync, never network)? Risk: it can overwrite edits made in Notaya unless it asks.
- **D5. Files and videos inside pages:** store them encrypted in the vault (grows the file, size cap needed) or only keep a link to the file left in the export folder?
- **D6. Look:** keep Notaya's own style (assumed, as in A33), or copy Notion's layout and colors more closely?

## 10. Not checked

No real Notion export, no Notion workspace and no running app were used. Section 11 was read from Notion's public pages on 2026-11-03; several help pages were thin (the export page does not name files or say how formulas and relations are written) and the third-party article does not cover colors, toggles, columns or formulas. Some feature areas (automations, templates, forms details, charts, offline limits for databases) came only from short summaries and may be incomplete; the search tool returned link lists for some queries and I did not open every page. The V1.4.0 feature list was read from the repository at `6a8af96`. Sizes and the release numbering are suggestions.

## 11. Sources (read 2026-11-03)

- Notion Help, [Export your content](https://www.notion.com/help/export-your-content): formats, options, full-workspace export limits.
- Breeze, [Notion export formats compared](https://www.breeze.pm/articles/export-from-notion): comments only in HTML; one CSV per database; only the current/default view; relations as text; form views not exportable.
- Notion Help, [Database properties](https://notion.com/help/database-properties): 22 property types (text, number, select, status, multi-select, date, formula, relation, rollup, person, file, checkbox, URL, email, phone, created time, created by, last edited time, last edited by, button, ID, place).
- Notion Help, [Views, filters, sorts](https://www.notion.com/help/views-filters-and-sorts): table, board, timeline, calendar, list, gallery, chart; filters nested three levels; groups and sub-groups; peek modes; freeze columns; linked databases; forms.
- Notion Help, [Writing and editing basics](https://www.notion.com/help/writing-and-editing-basics): block list, embeds, mentions, page options.
- Notion Developers, [Notion-flavored Markdown](https://developers.notion.com/guides/data-apis/enhanced-markdown): complete list of block and inline types (toggle headings, callouts, columns, synced blocks, table of contents, mentions, mermaid, custom blocks).
- Notion Help, [Working offline](https://www.notion.com/help/guides/working-offline-in-notion-everything-you-need-to-know).
- Notion releases: [2026-01-20](https://www.notion.com/releases/2026-01-20), [2026-03-26](https://www.notion.com/releases/2026-03-26) (dashboards, tabs block, presentation mode, H4, page archiving), [2026-05-13](https://www.notion.com/releases/2026-05-13) (developer platform; merge cells in simple tables), and a Releasebot / releases.sh summary of Sep to Nov 2026 (agents, MCP, synced databases, column permissions, version history).
