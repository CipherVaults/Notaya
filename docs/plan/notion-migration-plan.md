# Notion to Notaya: migration plan (API copy plus HTML and Markdown exports) and its limits

**Status:** a plan for the owner to decide on. **Nothing is built.** Companion to [notion-offline-backup-roadmap.md](notion-offline-backup-roadmap.md) (which lists the Notaya features to build). Baseline `origin/main` `6a8af96`. Written 2026-11-03 (system clock, UTC) by a cloud thread. Same record: [EX-2026-11-03-41](../execution/2026-11-03-notion-offline-roadmap.md).

## 1. What the owner asked, and one conflict to confirm

- 2026-11-03 19:03:04 UTC, Mohamed: "if i told you to copy all thing from notion api to notaya / if we built every feature what it will be".
- 2026-11-03 19:06:43 UTC, Mohamed: "give me a plan and what limitations we will face / consider like every future we did already and using export of Notion API you will do it / mix of MD and html from notion".

So this plan **assumes every Notaya feature in the roadmap exists** (databases, all seven view types, formulas, relations, rollups, charts, dashboards, columns, tabs, synced blocks, files and so on) and asks how a full copy of the owner's Notion would work, using **the Notion API together with the HTML and Markdown & CSV exports**.

**Conflict to confirm (D-M1).** At 18:20:37 UTC the owner said "not ... integration or network". The API needs the network once. This plan reads the later messages (19:03 and 19:06) as the owner allowing a **one-time copy tool**, and keeps **Notaya itself fully offline** (promise P5 unchanged). If that is wrong, drop this plan and use only the export route.

## 2. How the copy works

```
Notion ──API (read only, once)──▶ Migrator tool (separate program, on the owner's PC)
Notion ──HTML export + Markdown & CSV export──▶ (files the owner downloads)
                         │
                         ▼
            Migration folder (JSON + downloaded files + report)
                         │  offline
                         ▼
            Notaya "Import from Notion migration" (checks, then saves all-or-nothing)
```

- **The migrator is not part of Notaya.** It is a small separate command-line program in `tools/notion-migrator/` (its own build; Notaya's exe, installer and `Cargo.lock` stay free of any HTTP or TLS code). It is run by the owner, on his PC, once or occasionally. It needs an HTTP/TLS library, which is a **new dependency and needs the owner's OK first** (`CLAUDE.md`).
- **The token.** The owner creates a Notion internal connection (Notion settings, Connections), gives it read-only capabilities (read content, read comments, read user information) and **shares his top-level pages with it**. He passes the token to the tool for that run only (environment variable or typed prompt). The tool never writes it to disk or to a log. After the copy the owner deletes the connection in Notion.
- **Notaya never sees the token or the network.** The import reads the migration folder like it reads a ZIP today (same safety rules: no paths outside the folder, size limits, cancel button, all-or-nothing).
- **Why a migration folder and not "API straight into Notaya":** the cloud threads can test the mapping on recorded sample files; the owner can inspect the folder before importing; Notaya can import the same folder again if the first import is fixed.

### What each source is for

| Source | What it gives | Why it is used |
|---|---|---|
| **Notion API** | Page tree, every block with its type, colors and nesting; database schemas (property types, select/status options, relation targets, **formula expressions**, rollup definitions); every row with typed values; **relations as exact page ids**; **views** (filter, sorts, group-by, visible properties, and the settings of table, board, calendar, timeline, gallery, form, chart, map and **dashboard** views); comments; synced-block originals and where they are reused; file download links | The only source that carries the structure. Read from the Notion help pages for the Views API (version `2025-09-03` or later) |
| **HTML export** | The page as Notion renders it: fonts, icon, cover, block colors, embeds and bookmarks with their titles; comments | A **cross-check** (page and block counts, text) and a **fallback** for pages where the API returns `unsupported` blocks, and the likeliest source for page look settings (font, small text, full width) that the API may not give. What it adds beyond the API is **not verified** |
| **Markdown & CSV export** | Plain readable pages and one CSV per database (current view only) | An independent, human-readable backup, and a **check on row counts and values** (a CSV row that is missing in the API copy is a bug in the tool). Not used as the main source |

So the "mix" is: **API for structure, HTML for look and cross-check, Markdown & CSV as the safety copy and the counting check.**

## 3. What carries over (assuming the Notaya features exist)

| Notion item | Source | In Notaya |
|---|---|---|
| Pages, sub-pages, nesting, order, icon, cover | API (+ HTML) | Same tree. Notaya's notebook > section > page layer is filled from Notion's top-level pages (decision D-M3) |
| Text, headings 1 to 4, lists, to-dos, quotes, dividers, code (language), equations, callouts (icon, color), toggles, toggle headings | API | Same blocks |
| Colors (text, highlight, block) | API | Mapped to the 18 Notaya colors by name (as the import does today) |
| Columns, tables (merged cells if Notaya has them), tabs, table of contents, breadcrumb | API | Same blocks |
| Synced blocks | API gives the original and every copy | **Re-linked** as real synced blocks (better than the export, which flattens them) |
| Images, files, PDFs, audio, video | API (download links expire in about an hour) | Downloaded at once by the tool; stored encrypted in the vault, within Notaya's size limits (section 5) |
| Bookmarks, embeds, link previews | API + HTML (title) | Link cards with the saved title. Nothing is fetched or played. Link previews are read-only in the API |
| Mentions of pages, dates | API | Page links, date chips. Mentions of people become plain names |
| Databases (full page and inline), 22 property types | API | Real databases. **Person / created by / edited by** become text names (read-user capability needed); **button** and **verification** are response-only, so they arrive empty |
| Select, multi-select and status options with colors and status groups | API | Same options |
| **Formulas** | API gives the expression text | Re-created in Notaya's formula engine. Notion has many functions; any function Notaya lacks keeps the **last value** and is listed in the report |
| **Rollups** | API gives relation, property and function | Re-created; a function the engine cannot compute keeps the last value and is listed |
| **Relations** (one-way and two-way) | API gives page ids | Rebuilt **exactly**, not by name matching |
| Views: table, board, list, gallery, calendar, timeline, chart | API | Filters (nested), sorts, groups, hidden properties, widths, calendar/timeline date properties, chart axes and colors rebuilt where Notaya has the same option; other options are dropped and listed |
| **Dashboards** | API (read-only) | Rows and widgets rebuilt as a Notaya dashboard page |
| Comments | API (needs the comments capability) and HTML | Notaya page and block comments, author as text. The API probably returns only open comments, **not verified** |
| Linked database views | API | Rebuilt as linked views |

## 4. Steps (each one is a thread and a report; the owner decides before each)

1. **M0: Look first (S).** With the owner's OK, a thread reads his workspace through the **Notion connector available in this session, read-only** (not the migrator) and reports an inventory: number of pages and databases, which block types, property types and view types he really uses, how many formulas, buttons, forms, automations and dashboards, how much file data. This tells us what to build first and what to skip. No data is copied.
2. **M1: Migrator skeleton and dry run (M).** The tool lists everything it can reach and writes a report only: counts, unsupported blocks, files and sizes, estimated number of requests and time. No content is written. Uses recorded sample responses for tests (anonymised), so cloud threads can test without the owner's token.
3. **M2: Pages and blocks (L).** Page tree, all block types, files, comments. Imported with the Notaya import (works with today's block set plus what Phase 4 adds).
4. **M3: Databases and rows (L).** Schemas, rows, relations. Needs Notaya Phase 1.
5. **M4: Views and dashboards (L).** Needs Notaya Phases 2 and 3 (views, charts).
6. **M5: Formulas and rollups (L).** Needs Notaya Phase 3.
7. **M6: Check (M).** Compare the Notaya copy with the Markdown & CSV export (page counts, row counts, text of a sample) and the HTML export (a sample of pages side by side), then a written list of what is missing.
8. **M7: Finish.** A final full run on the day the owner wants his backup, then a PC check on throwaway data first (never his real Notaya vault), as in the usual rules.

M1 to M3 can start as soon as the owner picks; M4 to M5 wait for the matching Notaya phases. Pages can be copied before databases exist, so the owner gets something usable early.

## 5. Limitations (what we will face)

**From Notion's API**
- **Access only to what is shared.** A connection sees only pages the owner shares with it. Pages in other people's private areas, or teamspaces he cannot open, are not reachable.
- **Speed.** Per-connection limit: 180 requests a minute on non-business plans, 600 on Business and Enterprise; data source queries about 3 a second; Notion also has a shared per-workspace limit. A page needs one request per 100 blocks plus more for nested blocks, comments and properties. A workspace of several thousand pages is **hours, not minutes** (about 170 minutes per 30,000 requests at 180 a minute). The tool must wait, retry on "rate limited" and be resumable.
- **Files.** Notion-hosted file links expire after about an hour, so files must be downloaded straight away. Files linked from other sites are only links; the tool can fetch them if the owner wants (D-M5). Notion's own limit on very large files does not apply to reading, Notaya's does.
- **Not available through the API:** buttons (shown as `unsupported`, actions not exposed), **automations**, form blocks, permissions and sharing, page history, AI content and memory, analytics, trash (archived pages are skipped). Dashboards and charts can be read but not created by the API, which does not matter for copying out.
- **Read-only blocks:** link previews can be read but not created, meeting-notes/transcription blocks have their own endpoints (and are AI content).
- **Formulas, rollups:** the API gives the expression text. Notaya has to understand Notion's formula language; some functions may never match.
- **Page look:** font, small text and full width are probably not in the API (**not verified**); the HTML export is the fallback.
- **People:** user ids only; names need the read-user capability.
- **Unknown / to check on a real run:** whether the API returns resolved comments; database **templates** and recurring templates; custom emoji; synced-block references across pages; how inline databases inside toggles and columns are returned.

**From Notaya (even after every phase)**
- **A copy, not a link:** Notion and Notaya diverge from the day of the copy. No sync (the owner excluded it). A second run is a new import (D4 in the roadmap), which does not merge with edits made in Notaya.
- **Features left out on purpose:** AI, integrations, sharing, live collaboration, embeds that play from the web, link previews that load, mobile and web. Automations and buttons are rebuilt by hand (Phase 6); their definitions cannot be read from Notion.
- **Vault limits:** 5,000 pages per import today (to raise), 25 MB per picture, 32 MB per stored record, so **large videos or big PDFs need a different rule** (D5 in the roadmap).
- **The file format changes** when databases arrive: an older Notaya may not open a newer vault.
- **Workload:** search and lists with tens of thousands of rows need the index described in the roadmap.
- **Windows only.** The tool and Notaya target Windows 10 and 11.
- **Look:** Notaya's own style, not Notion's.

**Process and security**
- **Your content leaves Notion once, to your PC.** The migration folder holds all your content **unencrypted** until you import it and delete it. The token is secret: read-only, short-lived, deleted after use, never saved by the tool or logged.
- **Notion's terms:** copying your own data through the official API with your own token is normal use; Notion can change API limits or plans at any time.
- **Dependencies:** an HTTP/TLS library (and possibly a JSON library) would be new code in the repository. The migrator lives outside Notaya's build; both need security review (`CLAUDE.md`, security-review skill) and a new row in `docs/security-design.md` for the import of migration folders.
- **Testing:** a cloud thread cannot run against the owner's Notion; it tests on recorded, anonymised responses. The first real run is on the owner's PC with his token, on throwaway Notaya data, with the owner present.
- **Honest size:** the migrator is a medium project by itself; the large part is still the Notaya features it needs to land into (databases, views, formulas, dashboards). Nothing here is quick.

## 6. Decisions for the owner

- **D-M1.** Confirm: one-time API copy tool allowed (separate program), Notaya itself stays offline. (Yes / No, export only.)
- **D-M2.** OK to add an HTTP/TLS dependency to the separate tool, after I name the exact library and version?
- **D-M3.** How should Notion's top-level pages map to Notaya? (a) one notebook per top-level page, sections from its children; (b) one notebook "Notion" with sections for the top-level pages; (c) you choose after the M0 inventory. Recommended: (c).
- **D-M4.** Start with **M0** (read-only inventory through the Notion connector in this session)? Recommended: yes. It only reads, writes nothing to Notion, and tells us what you really use.
- **D-M5.** Files hosted outside Notion (links): keep as links, or download them too?
- **D-M6.** Roadmap decisions D1 to D6 in the other document still stand.

## 7. Not checked

No Notion workspace, token or running tool was used. The API facts come from Notion's developer pages read on 2026-11-03 (request limits, block reference, property objects, working with views); the comments reference page could not be fetched, so the comments row is partly from general knowledge. Whether the HTML export adds anything the API lacks, and the real request counts, are unknown until M0 and M1. Sizes are relative.

**Sources:** [Request limits](https://developers.notion.com/reference/request-limits), [Block reference](https://developers.notion.com/reference/block), [Property objects](https://developers.notion.com/reference/property-object), [Working with views](https://developers.notion.com/guides/data-apis/working-with-views), and the export pages listed in the roadmap document.
