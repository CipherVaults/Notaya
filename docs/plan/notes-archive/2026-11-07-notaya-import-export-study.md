# Notaya import and export: what exists today (read-only study)

- **Baseline:** `CipherVaults/Notaya` `main` at `9474511` (merge of PR #55), version 1.5.1.
- **Asked by:** Ahmed, 2026-11-07 20:56:12 UTC (project chat). Scope: study only, no code, no branch, no PR.
- **Method:** I read the source (importers, export, commands, UI, tests) and the records (`docs/plan/import-what-carries-over.md`, EX-40, EX-42, EX-43). I did **not** build or run anything, so every statement below is "what the code and its tests say", not a live observation. No real Notion export was available to me. The records say a real "Hubs" Notion export was tried by Ahmed on 2026-11-05 and fixed (EX-43 item 15); a full real export of everything was not.
- **Result state:** produced (this note). Nothing verified, integrated or accepted.

## 1. The short version

1. **Import is big and mature.** It reads Markdown, plain text, HTML, Word (.docx), PDF, ZIP files (Notion exports, Notaya's own exports), loose files and whole folders, in any mix, and it already turns a Notion "Markdown & CSV" export into **real databases** (columns, Status, Number formats, two-way Relations, row pages folded into rows). One mistake to avoid later: the Import dialog still tells the person "Databases (tables)... are not imported", which is out of date.
2. **Export already is "Save as".** The Export dialog already lets you choose a page (or several pages, a section, several sections, a notebook, several notebooks), **with or without sub-pages**, in **PDF, Word, Markdown or HTML**, as **one file** or as **separate files in a ZIP**. Sub-pages follow their parent in sidebar order and sit one heading level lower. So Ahmed's "hub page with ordered sub-pages in one PDF" works in a basic form today.
3. **The real export gaps** are: no Excel (.xlsx); databases are not part of a page or notebook export (an inline database block on a hub page is **silently dropped**); the PDF has bookmarks but no printed table of contents and no clickable links; nothing is exposed to an AI (all of this is behind buttons and Windows dialogs).
4. **The real import gaps** are what Notion itself does not export well, plus: no import into an existing notebook or page, no Excel/CSV-as-pages, no synced blocks/embeds/video/audio/comments/history, formulas and rollups arrive as plain values, people columns as text.

## 2. Import: how it works today

Code: `src-tauri/src/commands/import_notes.rs` (the command), `src-tauri/src/importers/` (converters), `src-tauri/src/commands/notion_db.rs` (Notion tables). UI: `src/features/notes/ImportDialog.tsx`.

### 2.1 What it accepts

| Source | How it is read | Notes |
|---|---|---|
| Markdown `.md` `.markdown` `.mdown` | `importers/markdown.rs` | CommonMark plus GitHub tables, task lists, strike, `==highlight==`, `$math$`, front matter title, link references, GitHub alerts as callouts, inline HTML (`<aside>`, `<details>`, colour spans), Mermaid fences become diagram blocks |
| Plain text `.txt` `.text` | `plan.rs::plain_text` | Line breaks kept; nothing interpreted as formatting (on purpose) |
| HTML `.html` `.htm` `.xhtml` | `importers/html.rs` | Fixed whitelist of elements; scripts, styles, frames, forms, SVG, event handlers dropped; Notion classes mapped (colours, callouts, toggles, to-do, code languages, equations, bookmarks, columns, page links, icon, cover, page font) |
| Word `.docx` | `importers/docx.rs` | Headings 1-4 (5-9 become 4), bold/italic/underline/strike, highlight, text colour, lists, tables with merged cells, links, pictures (PNG/JPEG/GIF/WebP), and everything Notaya's own Word export writes (code, tasks, captions, shelves) |
| PDF `.pdf` | `importers/pdf.rs` (hand-written reader) | **Text only**, headings guessed from font size. No pictures, no scanned pages (no OCR), no tables/columns/links, no encrypted PDFs |
| Old Word `.doc` | `importers/doc.rs` exists | Per the records, a message asks the person to save as `.docx` (I did not confirm in the picker; the file dialog filter lists only md, markdown, txt, html, htm, docx, pdf, zip) |
| ZIP `.zip` | `importers/zip.rs`, `source.rs` | Hostile-input checks (zip-slip, bombs, ratio). Nested ZIP opened **one level** (Notion's `Part-N.zip`). Several ZIPs picked together are merged |
| A folder | `Vfs::from_folder` | Every supported file; subfolders become sections |
| Any `.csv` inside the import | `notion_db.rs` | Becomes a database (see 2.3) |

The file picker is opened by the Rust core (`import_notes`), never by the UI. The UI sends only `"files"` or `"folder"`. Notaya's own `.nota`/`.clf` files and the data folder are refused.

### 2.2 How it arranges pages

- **One new notebook per import.** Existing notes are never changed or merged. Name: the folder, ZIP or single file (generic names such as "Export-..." become "Notion import"; several loose files become "Imported <date>").
- **Sections:** one per top-level folder; loose pages go to a section "Imported pages". A single wrapper folder is not a section.
- **Sub-pages (Notion style):** a page `X.md` plus a folder `X/` next to it makes everything in `X/` a sub-page of X. Any other folder becomes an empty page that holds its pages. Order is natural-sort by name (Page 2 before Page 10), **not** Notion's own order (Notion does not export order).
- **Notaya's own ZIP export round-trips:** the marker file `notaya-export.txt` (layout `notebooks`/`sections`/`pages`) restores several notebooks, their sections and page links. Without the marker it behaves like any ZIP.
- **Same Notion page in two formats** (HTML + Markdown): matched by the 32-hex Notion id in the file name, imported once, with the HTML's formatting and the Markdown/CSV's table data.
- **Titles:** the document's own title, else first H1, else file name; Notion's trailing id is stripped.
- **Safety and limits:** nothing fetched from the network (web pictures stay links "Picture: name"); links keep only http/https/mailto; 5,000 pages and 5,000 pictures per import; 2 GB per file; 4 GB total; 15-minute limit; **Cancel** button; all-or-nothing save in one transaction (pictures stored first and taken back on failure); backups before the first database is made.
- **Duplicate warning:** importing the same export twice makes a second notebook and says so.

### 2.3 Notion databases (this is the part Ahmed said is "more than you expect")

When the import sees Notion's `Name <id>.csv` and `Name <id>_all.csv` files (or any CSV), it:
- makes **one Notaya database per table**, taking the `_all` file (the plain file is the view and is ignored);
- guesses column kinds: Number (with dollar/percent/thousands formats), Select, Multi-select (comma lists), **Status** (a column called "Status", options grouped todo / inProgress / complete), Date, Checkbox, URL, Email, Text;
- folds each **row page** (the `.md`/`.html` in the table's folder) into its database row (text, icon, cover), unless another page links to it or it has sub-pages (then it stays a page too);
- turns columns whose cells are all links to rows of another table into **real Relations**, and pairs a relation with its mirror into **one two-way relation**;
- makes a safety backup first, and writes a note per database ("Database X: 3 rows, 4 columns, N row pages moved").
Tests: `commands/import_notes.rs` has five Notion-database tests (`the_tables_of_a_notion_export_become_databases`, `notion_rows_become_database_rows_with_relations_status_and_numbers`, `a_flat_notion_export_still_moves_row_pages_into_rows_and_links_relations`, `markdown_csv_and_html_files_together_complete_each_other_into_database_rows`, `two_export_folders_picked_together_make_each_database_once`).

Separately, a database has its own **Import CSV…** (new database in a section, or into an existing one, matching columns by name; CSV/TSV/TXT; up to 100,000 rows and 100 columns) in `commands/database_io.rs`.

### 2.4 What the import drops or gets wrong (Notion features it loses)

From the records and the code; items marked (records) were not re-checked against a real export by me.

| Notion feature | What happens |
|---|---|
| Database views (board, calendar, gallery, timeline, filters, sorts, grouping, hidden columns) | Only the **table data** comes in (the `_all` CSV). Views are not in Notion's export, so they cannot come |
| Formula, rollup, "created by", "last edited" columns | Arrive as their **plain result values**, kept as text (the export has only results) |
| Person/people columns | Look like text columns, stay text |
| Relations that point to a table not in the export, or ambiguous titles | Stay text |
| Inline databases inside a page (linked views) | The database becomes a database; **no inline database block is re-created** on the page that held it (I found no code that writes a `databaseBlock` on import). The HTML-only export of a database page is imported as a page with a properties table, rows as sub-pages |
| Synced blocks | Not verified (records); Notion writes plain copies, which import as normal content |
| Embeds, video, audio, files other than PDF/Word | Frames/video/audio dropped without a note; a plain link beside them stays. A PDF or Word attachment is imported **as a page of its own** (text only). Other attachment types: listed as "not imported" |
| Comments, page history, permissions, sharing, who-edited | Not imported |
| Markdown export (instead of HTML) | **No text colours, highlights, columns, page font, icons or covers** (Notion's Markdown export does not write them). A note tells the person to export HTML as well |
| Callout colour | Becomes a highlight on its text, not a coloured box |
| A page mentioned in a sentence | Becomes a Notaya link only if that page is in the same import; else text |
| Bookmark / link preview card | Becomes a "🔖 title" link line |
| Heading levels above 4 | Become level 4 |
| Toggles | Kept; shown open |
| Page width, small text, text sizes, column widths | Not imported |
| Pictures that are not PNG/JPEG/GIF/WebP (HEIC, SVG, AVIF...) | Not imported; "[Picture not imported: name]" stays in the page and the picture is listed |
| Notion's photo covers (unsplash, nasa...) | Not fetched; built-in gradients/solids map to the nearest Notaya cover **by number** |
| Order of pages/sections | Sorted by name, not Notion's order |
| Top-level `index.html` | Skipped and listed |

The post-import screen lists "Left out" items with reasons, so nothing is dropped without a note, except the frames/video/audio case above.

### 2.5 Out-of-date text to fix later

`ImportDialog.tsx` says "Databases (tables), comments and page history are not imported." Databases **are** imported when the Notion Markdown & CSV export is chosen. The same dialog's own paragraph above it says so, so the two sentences contradict each other. `docs/plan/import-what-carries-over.md` still lists "Databases and table views (the .csv files): listed as left out" under "Does NOT carry over"; that predates V1.5.0 (A35) and is wrong now (it is a record, so correct it with a dated note, not by rewriting).

### 2.6 Other "import" features that are not page import

- **Import from a portable copy** (`import.rs`, `commands/backup.rs::import_portable_notes`): on the lock screen, copies a portable Notaya's whole vault file into an installed one, only if there is no vault yet. It never merges.
- **Restore from a backup** (`restore.rs`): replaces the vault from a backup file on the lock screen.
- **Database CSV import** (2.3 last paragraph).
These are different buttons and different code; the redesign must not mix them up with page import.

### 2.7 Where the buttons are today

- **Sidebar > "Notebooks" header:** a small upload button, "Import pages…" (`Sidebar.tsx:301`). It is a hover action, easy to miss.
- **Empty sidebar:** "Import pages from another app" (`Sidebar.tsx:309`).
- **Command palette:** "Import pages…" (`CommandPalette.tsx:173`).
- **Lock screen:** import portable notes (separate feature).
- **Inside a database's menu:** "Import CSV…" (`DatabaseView.tsx:641`).
There is **no top-bar Import button**. All entry points set one store flag `importOpen`, so one new top-bar button can reuse the same dialog.

## 3. Export: how it works today

Code: `src-tauri/src/commands/export.rs` (command), `src-tauri/src/export/` (`mod.rs` layout, `doc.rs` model, writers `pdf.rs`, `docx.rs`, `markdown.rs`, `html.rs`, `zip.rs`), `commands/database_io.rs` (database export). UI: `src/features/notes/ExportDialog.tsx`.

### 3.1 What it offers

- **Scope** (what to export): one page, chosen pages, a section, several sections, a notebook, several notebooks.
- **Include sub-pages:** a switch (on by default in the dialog). Pages in the trash are left out (the backend has an `include_trashed` flag; the dialog does not expose it).
- **Format:** HTML, Markdown, PDF, Word.
- **Output:** "One file" or "Separate files in a ZIP" (folders Notebook/Section/Page, sub-pages in a folder named like their page, relative links between pages, Markdown pictures in `.assets` folders, plus the `notaya-export.txt` marker so Notaya can re-import it).
- A native "Save as" dialog is opened by the core with a suggested name; files are **not encrypted** (the dialog says so); written to a temp file then renamed; the data folder is refused as a destination.

### 3.2 How sub-pages are ordered in one file

`export/mod.rs`: pages are sorted by `(order, created)` (the sidebar order), the tree is walked depth first, and each page's heading level is the depth below the chosen page plus the base level (notebook export: notebook H1, section H2, pages H3 and below; section export: section H1, pages H2...; capped at level 6). In PDF and Word each page starts on a new page. So a hub exports as: hub, then its sub-pages in the sidebar order, each as a deeper heading. This answers most of Ahmed's "ordered" worry for a single file.

### 3.3 How each format handles the "holistic file" problem

| Format | Navigation inside the one file |
|---|---|
| **PDF** (`pdf.rs`) | **Bookmarks**: one flat list entry per page title and section heading (`/Outlines`, panel opens by default). **No** printed table of contents, **no** nesting in the bookmarks (flat), **no** clickable links between pages (the writer has no internal links; a page link is written as its text) |
| **Word** (`docx.rs`) | Real Heading 1-6 styles, so Word's navigation pane and "insert table of contents" work. Links to pages in the same export are real jumps (bookmarks). **No** table of contents is inserted automatically |
| **HTML** | Links to other exported pages jump to that page's heading (`#notaya_page_N`). No generated table of contents |
| **Markdown** | One `.md` with headings; page links keep their text only; pictures in `.assets` folder |
| **ZIP of files** | Real folder structure, relative links between files, re-importable |

PDF details worth knowing for the redesign: standard Helvetica/Courier (Windows-1252), bundled Arabic and Hebrew fonts, CJK via fonts Windows already has; Greek, Cyrillic and emoji become "?" or are left out (the dialog warns "Choose Word to keep them"); GIF and WebP pictures are not embedded; PNG decoding is partial (interlaced PNG left out). The summary reports counts.

### 3.4 Databases and export

- A **database** (its own screen) can be exported from its menu: **CSV** (with a UTF-8 byte-order mark and CRLF so Excel reads Arabic/Chinese correctly), and **Markdown, HTML, Word, PDF** as "its title and one table" for the chosen view (`export_database`, capped at 20,000 rows). So **Excel can open the CSV, but there is no `.xlsx` writer** (a grep for xlsx finds only an allowed-attachment-extension list and a CSV comment).
- A database **does not take part in page/section/notebook export.** `export/mod.rs::select` handles pages only.
- **An inline database block on a page is silently dropped from every page export.** The editor node `databaseBlock` is an atom with only `databaseId` and `viewId` and no children (`DatabaseBlock.tsx`); `export/doc.rs::block` has no case for it and its fallback for unknown nodes emits nothing when a node has no children or text. I found no test covering it. This is the single most important export bug for a "hub page" use case. (Confirmed by reading the code, not by running an export.)
- Database **rows** (which are pages with cell values) are also not exported as pages; only the table of values is.

### 3.5 Other export facts

- Mermaid blocks export as a captioned code block ("Mermaid diagram (code)"), not as a picture.
- Equations export as TeX text in Word and Markdown (`$...$`); highlight and text colours export as inline HTML in Markdown, shading in Word.
- A node type the exporter does not know keeps its text if it has any, otherwise vanishes (the database block case).
- Round trip: Notaya exports come back through Notaya's import with colours, code, tasks, captions, shelves and page links (tests in `roundtrip_tests.rs`: `every_format_scope_and_output_comes_back_in`, `links_between_exported_pages_come_back_as_links_between_the_imported_pages`, and others). Order comes back sorted by name, not sidebar order.

### 3.6 Where the buttons are today

- **Page "..." menu** in the editor: "Export…" (`PageEditor.tsx:312`).
- **Right-click on a notebook, a section or a page in the sidebar:** "Export…" (`Sidebar.tsx:354, 921, 1146`).
- **Command palette:** "Export…" (page, else notebook).
- **Database menu:** "Export CSV / Markdown / HTML / Word / PDF (this view)".
- **There is no top-bar Export button** next to Favorites. The Favorite star lives in the page header area of `PageEditor.tsx` (`toggleFavorite`, star icon, line ~274). All entry points call `openExport(kind, id)` (and set one store target), so a top-bar button can reuse the existing dialog.

## 4. Tests that cover this (counted from `#[test]` markers; not run by me)

| Area | Tests |
|---|---|
| Importers: `plan` 19, `markdown` 15, `html` 12, `docx` 11, `pdf` 10, `doc` 8, `zip` 6, `source` 5, `markup` 4, `mod` 4, `palette` 3 | 97 |
| `commands/import_notes.rs` (whole-import behaviour: Notion databases, folders, mixed picks, failure leaves nothing, hostile files, cancel, lock mid-import, data-folder refusal) | 17 |
| `commands/notion_db.rs`, `database/csv.rs`, `commands/database_io.rs` | 1 + 9 + 10 |
| Export: `mod` 26, `pdf` 19, `html` 16, `doc` 10, `docx` 8, `markdown` 4, `image` 4, `zip` 2; `commands/export.rs` 1 | 90 |
| `roundtrip_tests.rs` (export then import back) | 5 |
| UI/mock | `dev-mock/` has database and mermaid checks; **there is no mock or real-exe script for the Import or Export dialogs**. The real-exe tests of V1.4.0 were done by hand over Remote Control (EX-40: 40/40 exports, 48/48 imports with the real dialogs) |
Verification state per the records: V1.5.1 `cargo test --lib` MSVC 517 passed on `95ca8b2`; EX-40 real dialogs on throwaway data. **Not run anywhere:** a complete real Notion export of a large workspace, Word and PDF files of the Hubs import, Word files made by real Word at scale, large (multi-GB) imports and memory use.

## 5. Gaps against Ahmed's wishes

| # | Wish | Today | Gap |
|---|---|---|---|
| 1 | "Save as"-style Export, page with sub-pages | **Exists** (dialog with scope, sub-pages switch, format, one file or ZIP) | Mostly **repositioning and wording**: a top-bar button with icon + label next to Favorites; "Save as" naming; the dialog is already the right shape |
| 2 | Sub-pages ordered well in one holistic file, PDF included | Sidebar order, depth-first, deeper headings, each page on a new sheet; PDF has flat bookmarks | **No printed table of contents; PDF bookmarks are flat (not nested by hierarchy); no clickable page links in PDF.** Options: nested outline + a generated contents page (with page numbers) + internal link annotations. Word could get an inserted TOC field |
| 3 | Formats PDF, Word, Markdown | **All exist** (plus HTML and ZIP) | Quality items only (Cyrillic/Greek/emoji in PDF, GIF/WebP pictures, Mermaid as a picture instead of code) |
| 4 | Excel for databases | CSV (Excel-friendly) and a table in Word/PDF/HTML/Markdown | **No `.xlsx`.** Needs a small xlsx writer (it is a ZIP of XML; the repo already has a ZIP writer and an XML-writing Word exporter) with real number/date/checkbox types and one sheet per database, or per view |
| 5 | Databases inside a page/hub exported with it | **Dropped silently** | Bug/gap: handle `databaseBlock` (render the view as a table in the page; in ZIP/Excel as a sheet or CSV next to it) |
| 6 | Whole notebook with its databases | Databases are not in notebook export | Include databases (and their rows' bodies) in notebook/section export, including ZIP, with a way back in |
| 7 | Keep everything the import does | Large surface, see section 2 | Do not touch converters; reposition only (below) |
| 8 | Import button own place, icon + label, clear in all themes | Only a hover upload icon in the sidebar header, an empty-sidebar line and the palette | New top-bar button that sets `importOpen`; same for export. Fix the stale "Databases are not imported" sentence |
| 9 | Import "from Notion exports and all of that" | Strong already (2.3, 2.4) | Remaining Notion gaps in 2.4 (inline database blocks, view definitions, order, synced blocks, embeds) |
| 10 | Import into where I am, not always a new notebook | Always a new notebook | Option "into this notebook/section/as sub-page of this page", plus merge/update for re-import (today a second import duplicates everything) |
| 11 | AI CLI controls all this | **Nothing is exposed**: every picker and save dialog is opened by the core from a click, and the UI never gives a path (a deliberate security rule) | See section 6 |

## 6. How import and export could be named AI actions

Constraint to respect: the project's security rule is that the **UI never supplies paths** and the core opens the dialogs (`docs/security-design.md`; I did not re-read it in full for this study, so confirm before designing). An outside AI would need a different, explicitly authorised entry point, and "AI sees everything, locked notes only when unlocked" means every action must refuse while the vault is locked. The converters themselves are already independent of the UI: `import_from_with(vault, paths, folder, data_dir, now, ctl, backup)` takes paths, a cancel control and a callback, and `export::collect` / `export::write` / `write_zip` take a scope, options and a destination. A path-taking entry point is therefore a thin wrapper, not a rewrite. Suggested names (proposal only):

| Action | Maps to | Notes |
|---|---|---|
| `import_preview(sources[])` | `plan::build` (nothing written) | Returns pages, sections, databases found, and the "left out" list **before** anything is saved; lets an AI confirm with Ahmed |
| `import(sources[], target = new_notebook / notebook / section / page, name?)` | `import_from_with` | Needs the new "import into" option (gap 10); returns the same summary (`notebooks`, `pages`, `images`, `skipped`, `notes`) |
| `import_cancel()` | `import_cancel` | Exists |
| `import_csv(source, database? / section?)` | `import_csv` | Exists, needs a path-taking variant |
| `export(scope, ids[], format, include_subpages, output = single / zip, destination, include_trashed?)` | `export_notes` request, plus a path | The request type already matches; only the dialog is bypassed |
| `export_database(database, view?, format = csv / xlsx / md / html / docx / pdf, destination)` | `export_database`, `export_csv` | Add xlsx |
| `list_exports_and_imports()` / an **activity log** | new | Ahmed asked for an activity log; each import/export should log what, when, who (AI or human), counts |
| `notion_import(zip_or_folder)` | alias of `import` with Notion defaults | Reports exactly the Notion losses of 2.4 in a machine-readable list |
| Destructive follow-ups (`delete_notebook`) | not part of import/export | Confirmation required, as Ahmed said |

Two design points that matter for "export from Notion, then import into Notaya, seamlessly": (1) the Notion side is a different tool (Notion's own API or export; this repo does not talk to Notion and must not fetch from the network), so the AI would fetch or export from Notion with its own Notion connector and then hand **files on disk** to `import`; (2) because import always creates a new notebook and a re-import duplicates everything, an AI-driven "keep my Notion backup in sync" needs the merge/update behaviour of gap 10 and a stable per-page key (Notion's 32-hex id is already parsed for pairing and relations in `plan.rs::notion_file_id`, so it is available to match pages on re-import).

## 7. Suggested order if Ahmed greenlights anything (not decided, not started)

1. Safe fixes first, no design needed: export must not silently drop an inline database block; correct the stale Import dialog sentence.
2. Reposition: top-bar Import and Export buttons (icon + label), reusing the existing dialogs and store flags; check icon contrast in every theme (the Favorite star is faint gray today).
3. Export quality for hubs: nested PDF bookmarks, a generated contents page, internal PDF links; Word TOC field.
4. `.xlsx` for databases; databases inside page/notebook export.
5. "Import into" and re-import without duplicates (this also unlocks the Notion-backup north star).
6. Path-taking, locked-vault-refusing, logged actions for the AI CLI, designed together with the CLI so each feature has named actions from the start.

## 8. What I did not check

- I did not run `cargo test`, `tsc`, a build or the app. Test names and counts are from reading the source.
- I did not open `docs/security-design.md` in full, `docs/acceptance.md` in full, or the PDF/DOCX writers beyond their headers and the outline code; statements about PDF bookmarks and the databaseBlock behaviour come from the code I read (`pdf.rs` outline code, `export/doc.rs::block`, `DatabaseBlock.tsx`) and should be confirmed with a five-minute real export before a fix is scoped.
- I did not verify behaviour against a real, recent Notion export; Notion changes its export format from time to time (the records already show two such fixes).
- I read the tail of `docs/handoff/claims.md`: no row claims import or export work (the open claims are the records PRs #54 and #55). I did not read the other remote branches, so check them and re-read the claims file before starting anything.
