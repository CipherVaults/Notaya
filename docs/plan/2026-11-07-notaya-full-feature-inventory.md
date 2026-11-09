# Notaya: full inventory of what already exists (code and records), mapped against the roadmap

- **Baseline:** `CipherVaults/Notaya`, `main` at `9474511` (merge of records PR #55, 2026-11-07), app version **1.5.1**. The last code release is v1.5.1 (tag on `95ca8b2`); everything after it on `main` is records only.
- **Asked by:** Ahmed, 2026-11-07 21:00:14 UTC (this thread's opening message): understand everything that already exists before anything is planned, designed or built. The coordinator's brief (21:00:25 UTC) set the scope: read-only, code and all records, every feature with what you see, where it lives, status and gaps; then map the roadmap and note AI CLI actions.
- **Method:** I read the source tree (all ~120 backend commands in `src-tauri/src/lib.rs`, the Rust command and model files, the main UI files: app frame, sidebar, title bar, command palette, editor extensions, "/" menu, ribbon, Mermaid, pictures, Home, settings, sticky notes, databases) and the records (`STATUS.md` in full, the four digests in this folder which cover the execution reports, plans, handoffs, security design, sessions and acceptance, plus spot checks of `acceptance.md`). I also read the Notion **Version Feature Board** (54 cards) and the two **unmerged branches** that hold records missing from `main` (see section 8). I re-checked the claims of the earlier import/export study against the code; one of them was wrong (section 8).
- **Nothing was built, run or changed.** No tests, no app run. "Tested in the real app" below means the records say a PC round tested it on throwaway data; I did not repeat it.
- **Result state of this file:** produced. Not verified, integrated or accepted.

---

## 1. The short version (plain words)

1. **Notaya is already a large app.** It has notebooks, sections, pages and sub-pages; a block editor with a ribbon and a "/" menu; full Notion-style databases with seven views, formulas, relations, charts and automations; sticky notes that float on the desktop; pop-out calculators; import (including Notion exports into real databases); export to PDF, Word, Markdown and HTML; optional encryption with auto-lock and backups; a tray icon and start-with-Windows; right-to-left text; Mermaid diagrams; page icons, covers and a Home page.
2. **What Mohamed has accepted:** the V1.4.0 release (2026-11-03) and the V1.5.1 release (2026-11-05), each "together with the list of checks that were not run". Everything in the app today is inside the accepted V1.5.1 release. Older pieces were never accepted one by one.
3. **Several roadmap items already exist in part.** Moving databases (to another section, by menu), the new-notebook button (it exists but only shows on mouse-over), the Home banner (a built-in gradient that changes daily, not your own picture), the tray (Open, Close sticky notes, Quit), Mermaid (its own "Diagram" block, not a code-block language), export "Save as" (the dialog already has page-with-sub-pages, PDF/Word/Markdown/HTML, one file or ZIP), charts (bar, line, donut, number; click a bar to see its rows), sticky-note multi-select (a Select button, not a box/swipe), and sticky notes on the desktop with "Keep on top" (this is already a form of "desktop pinning").
4. **Things that do not exist at all:** Ctrl+F in-page find, reopening the last page, mouse right-click menus (menus open from a "..." button or the keyboard Menu key), a customizable top bar, editable shortcuts or a shortcuts page, page tabs, a code-block language picker, Mermaid toolbar (zoom, download, copy, wrap), a free-layout or corkboard sticky board, pins and strings, slicers, multi-chart dashboards, Excel `.xlsx` export, a guide, a tour, named-person credits in About, Whiteboard, OneNote mode, and **any way for an outside program (an AI CLI) to talk to Notaya**.
5. **Two bugs worth knowing before building on top:** an inline database inside a page is **silently dropped** from every page export, and the Import dialog still says "Databases are not imported" although they are.
6. **Good news for the AI CLI:** almost all logic already lives in the Rust core as ~120 named commands. The UI is a thin client over them. The automation engine already shows the pattern the CLI needs (a closed list of named actions, run in Rust, with limits). The Markdown importer and exporter can already turn Markdown into page content and back, so an AI would not have to write the editor's internal format. The hard parts are a safe outside entry point, keeping the open window up to date when something else changes the notes, and confirmation and logging for deletes.

---

## 2. How to read the status column

| Label | Meaning |
|---|---|
| **Accepted, real-app tested** | In the accepted V1.5.1 (or V1.4.0) release, and a PC round on throwaway data tested it in the real exe (per the records) |
| **Accepted, not real-app tested** | In the accepted release, but the records show only unit tests, mock-UI checks or nothing for the real app (or say "NOT RUN") |
| **Partial** | Some of the wanted behaviour exists; named gaps |
| **Dormant** | Code exists (usually in the document model) but no button or menu can create it |
| **Missing** | Searched for in the code and not found |

"Accepted" always means the release was accepted as a whole ("i accept" 2026-11-03 18:23:19 UTC; "I approve this version" 2026-11-05 11:08:39 UTC, read by the coordinator as V1.5.1). Mohamed never accepted features one by one.

---

## 3. Inventory by area

### 3.1 App frame: top bar, sidebar, Home, navigation

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| **Top bar ("app bar")** | Logo and "Notaya"; sidebar toggle; Back and Forward arrows; a centred "Search · Ctrl K" box; a Lock button (only with security on); Windows minimize, maximize, close | `src/components/TitleBar.tsx` | Accepted, real-app tested (seen in every PC round) | **Fixed layout.** No Import or Export button, no customization, no per-page actions. The Favorite star is not here: it sits in the page header |
| **Sidebar** | Home, Sticky notes (Ctrl 3), Settings (Ctrl ,); Favorites group; Notebooks tree (notebook > section > pages and sub-pages, databases inside sections and pages); bottom: New page (Ctrl N), New database…, Trash with a count; resizable 200-420 px; hide with Ctrl+\ | `src/components/Sidebar.tsx` (1,251 lines) | Accepted, real-app tested | See the rows below |
| Notebooks | Name, emoji icon, colour; "..." menu: Add section, Edit notebook, Move up/down, Export…, Delete notebook (asks, makes a safety backup first) | `Sidebar.tsx` ~320-375, `NotebookDialog.tsx`; Rust `commands/notes.rs` `create/update/delete_notebook` | Accepted, real-app tested | Reorder only by Move up/down (no drag). **The "+ New notebook" and "Import" buttons in the Notebooks header only appear on mouse-over** (`GroupHeader`, `opacity-0 … group-hover`); "Create your first notebook" shows only when there are none |
| Sections | Drag to reorder inside a notebook; "..." menu: New page, New database…, Rename, Move up/down, Export…, Delete | `SectionBlock` in `Sidebar.tsx`; `reorder`, `create/update/delete_section` | Accepted, real-app tested | Drag only inside the same notebook |
| Pages and sub-pages | Click opens; drag to reorder top-level pages in a section; "..." menu: Add a page inside, Add a database inside…, Add/Remove favorite, Move up/down, Move to (another section, **first 8 listed**), Export…, Move to trash (with Undo toast) | `PageItem`, `PageBranch`; `create_page`, `create_subpage`, `move_page`, `reorder`, `trash_page` | Accepted, real-app tested | No drag between sections or into another page; no "move into a page" (re-parent) at all; Move-to list capped at 8 sections; no duplicate page, no rename from the menu (title is edited in the page) |
| Databases in the tree | Database icon and name under its section or page; "..." menu: Add/Remove favorite, **Move to** (another section, first 8), Move to trash | `DatabaseItem` in `Sidebar.tsx`; `move_database` and `move_page` in Rust | Accepted, not specifically recorded as real-app tested | **No drag** (databases are not in the sortable list); moving always puts it at the top level of a section (`parent` cleared); rows are not shown in the tree |
| Keyboard tree | Arrow keys, Home/End, Enter/Space, Shift+F10 or the Menu key opens a row's menu | `Row` in `Sidebar.tsx` | Accepted (A23 accessibility fix); screen reader not run | – |
| **Right-click** | **No mouse right-click menu anywhere.** In the shipped app the browser menu is blocked except inside text fields (`src/main.tsx`), where WebView2's own Cut/Copy/Paste menu appears | `src/main.tsx` | – | The study note of 20:56 said "right-click on a notebook > Export"; that is wrong: it is the "..." button |
| **Home** | A banner (one of 12 built-in gradients, **changes once a day**), a greeting by time of day ("Good morning" etc. with an emoji), the date, Favorites (8), Recent pages (8, by last edit), Notebooks (24) as cards, a one-time note when security is off | `src/features/notes/HomeView.tsx` | Accepted (V1.5.1 item 12); BUG-7 and BUG-8 found on the PC and fixed (layering), checked in the mock | No own picture; no per-time-of-day banner choice; no settings for Home |
| **Back and forward** | Backspace / Shift+Backspace, Alt+Left/Right, mouse buttons 4/5, the top-bar arrows; 100 places; memory only | `src/lib/navHistory.ts` | Accepted, real-app tested (V1.5.1 item 1) | Lost on restart and on lock (the window reloads) |
| **Reopen last page** | – | `store/app.ts` starts with no open page, so Notaya always opens on Home | **Missing** | Also after an unlock the window reloads and lands on Home |
| Trash | Lists deleted pages and databases; Restore, Delete forever, Empty trash | `NotesView.tsx` `TrashView`; `restore_page`, `delete_page`, `empty_trash` | Accepted | Trashed database **rows** have no list of their own (EX-42 gap) |
| Visible scroll bars | Always-on 12 px scroll bars | `styles/app.css` | Accepted, real-app tested (V1.5.1 item 2) | – |

### 3.2 Search and the command palette

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| **Command palette (Ctrl+K, or the top-bar box)** | One box that searches pages, databases, database rows, notebooks and sticky notes as you type, with a snippet; Actions: New page, New sticky note, Go to Notes, Appearance & settings, Export…, Import pages…, Back up vault now, Lock vault (security on) | `src/components/CommandPalette.tsx`; Rust `commands/tools.rs::search` | Accepted | Search matches every word, case-insensitive, by scanning all items in memory (no index; EX-41 warned this gets slow with many rows); opening a result **does not jump to or highlight** the match inside the page; no filters (by notebook, date, type) |
| Recent searches | The last 5 terms under the box; remove one or clear all; stored encrypted in the vault | `src/features/search/RecentSearches.tsx`, `commands/search_history.rs` | Accepted, real-app tested (V1.5.1 item 9; a security-behaviour change recorded in security-design) | – |
| **Ctrl+F (find in page)** | Notaya has no find-in-page of its own. Ctrl+F is **not** in the list of blocked browser keys (`src/main.tsx` blocks F5, F7, F12, Ctrl+R/P/U/G/J, Ctrl+Shift+I/J/C), so WebView2's own find bar is what appears (Ahmed's board card says it opens "the Edge search") | – | **Missing** | Not verified by me in the real app; a Notaya find bar would also need find-and-replace and highlighting inside the editor |
| Database search | A search box in a database toolbar (client side) | `DatabaseView.tsx`, `search_rows` | Accepted | Column footers ignore the client-side search (EX-42 note) |

### 3.3 Keyboard shortcuts (all fixed in code; no settings page)

| Keys | Does | Where |
|---|---|---|
| Ctrl+K | Command palette | `Shell.tsx` |
| Ctrl+N | New page in the current section | `Sidebar.tsx` |
| Ctrl+1 / Ctrl+3 | Notes / Sticky notes (Ctrl+2 was Passwords, removed in A24; whether Ctrl+3 should become Ctrl+2 was never decided) | `Shell.tsx` |
| Ctrl+, | Settings | `Shell.tsx` |
| Ctrl+\ | Show/hide sidebar | `Shell.tsx` |
| Ctrl+L | Lock (security on) | `Shell.tsx` |
| Backspace, Shift+Backspace, Alt+Left/Right, mouse 4/5 | Back / forward | `navHistory.ts` |
| Ctrl+Z / Ctrl+Y | Undo / redo in the editor and (50 steps) in databases | TipTap; `features/database/undo.ts` |
| Ctrl+B / I / U, Tab / Shift+Tab, and TipTap's standard ones (headings, lists, code, quote, strike) and Markdown typing rules (`#`, `-`, `1.`, `>`, ```` ``` ````, `---`, `[ ]`) | Editor formatting | TipTap StarterKit defaults, `extensions.ts` |
| `/` | Block menu | `slash.ts`, `SlashMenu.tsx` |
| `$x^2$` | Inline equation | `math.ts` |
| ```` ```mermaid ```` + space | Diagram block | `MermaidBlock.ts` |
| Ctrl+click a link | Opens it (http/https only) | `extensions.ts` |
| Ctrl+Enter or Escape in a diagram's code | Leaves the code box | `MermaidBlock.ts` |
| Escape, Delete in sticky select mode | Leave / delete ticked notes | `StickyBoard.tsx` |

**Status:** accepted. **Gaps:** shortcuts are scattered across files (no single list), cannot be changed, there is no shortcuts page, and several actions have none (export, import, favorite, new notebook, new database, new sticky).

### 3.4 Look: themes, appearance, icons, covers, banners

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Themes | Sunrise (default), Sunrise dark, Black, Graphite, White, Paper, Match Windows | `SettingsView.tsx` `THEMES`, `src/lib/theme.ts`, `styles/app.css`; `Settings.theme` in the vault | Accepted, real-app tested | Coral small text on cream is about 3.5:1 contrast; "Match Windows" does not follow Sunrise (A28 notes); faint icons in dark themes (Ahmed: the Favorite star is hard to see; it uses the `text-faint` colour when off) |
| Appearance | Accent colour (presets + custom), font (Inter, Segoe, Serif, Mono), text size 12-20, page width (Narrow, Normal, Wide, Full), density, window material (Solid, Mica, Acrylic), lock-screen background, new sticky colour | `SettingsView.tsx` "Appearance" | Accepted | Appearance is also cached in plain browser storage (security-design limit) |
| Page icons | "Add icon" on the page title: about 600 emoji (search, random) or an uploaded picture | `PageHeader.tsx`, `LookPickers.tsx`, `lib/emoji-data.ts`; `set_page_look` | Accepted (V1.5.1 item 12) | Notebook icons are emoji only; database icons emoji only |
| Page covers | "Add cover": 18 gradients, 16 colours, upload a picture; Reposition; Remove | `PageHeader.tsx`, `LookPickers.tsx` | Accepted; the native picture dialog did not open on a scripted click in the PC round (not a manual test) | Notion's photo covers are not fetched (no network) |
| Home banner | See 3.1 | `HomeView.tsx` | Accepted | Not user-settable |

### 3.5 The page editor and its blocks

Editor: TipTap 3.32 (`src/features/notes/editor/extensions.ts`). Page content is stored as TipTap JSON inside the encrypted page record, with a plain-text copy for search.

| Block or feature | How you make it | Code | Status | Known gaps |
|---|---|---|---|---|
| Paragraph, headings 1-3 (4 in the schema and the ribbon), bullet, numbered, checklist (nested) | "/" menu, ribbon, Markdown rules, "Turn into" | StarterKit, `turnInto.tsx` | Accepted | Heading 4 not in "/" menu |
| Collapsible section (toggle) and **Shelf** (shaded box with a title that folds) | "/" menu, ribbon | `blocks.ts` `Toggle` (`shelf` attribute) | Accepted, real-app tested (V1.4.0) | Toggle headings (a heading that folds) missing; toggles always show open on import |
| Highlight box (callout), quote, divider | "/" menu, ribbon | `blocks.ts` `Callout` | Accepted | Callout colour from Notion becomes a text highlight |
| Table | "/" menu (3x3), ribbon Insert; contextual **Table** ribbon tab: rows, columns, header row, merge/split, delete | TipTap TableKit (resizable) | Accepted | – |
| **Code block** | "/" menu, ribbon, ```` ``` ```` | `CodeBlockLowlight` with lowlight's "common" languages | Accepted | **No language picker.** The block has a language attribute (set by imports, e.g. Notion's `language-…` class), and without one lowlight guesses the language for colouring. No copy button, no wrap toggle, no line numbers |
| **Mermaid diagram ("Diagram")** | "/" menu "Diagram", ribbon Insert, ```` ```mermaid ```` typed, "Turn into > Diagram" (a code block's text becomes the diagram code), imports (Markdown fence, Notion HTML `language-mermaid`) | `MermaidBlock.ts`, `mermaid.ts`; `mermaid` 12.2.0 package | Accepted, real-app tested (V1.5.1 BUG-9 retest: 9 samples draw; needed `freezePrototype` off) | A **separate block, not a code-block language**. Bar has only "Diagram", a "Sample…" picker and "Show/Hide code". **No zoom, download as picture, copy, or wrap.** **Pasting** Mermaid text does not detect it (paste handling covers pictures only). Exports write the code, not a picture. A slow diagram cannot be stopped (15 s limit); `elkjs` is EPL-2.0 (recorded open decision) |
| Equations | "/" menu (block, inline), `$x^2$`, toolbar "make selection an equation" | `math.ts`, KaTeX 0.19.0 | Accepted; mock-checked (security-design P14) | Exported as TeX text |
| **Pictures** | "/" menu, ribbon, paste, drop | `ImageBlock.ts` | Accepted, real-app tested (V1.5.1 item 10) | Free position (anywhere, front/back layer), text wrap (float), align, width %, ratio crop, caption. PNG/JPEG/GIF/WebP only, 25 MB each. Free mode is off inside columns, lists and tables; exports put free pictures in the text flow |
| Sub-page block | "/" menu, ribbon | `turnInto.tsx` "page"; `create_subpage` | Accepted | – |
| **Link to page** (a row) and links inside text to pages and headings (`notaya:page/…`) | "/" menu "Link to page" picker; link tool | `more-blocks.ts` `PageLink`, `internalLinks.ts`, `PagePicker.tsx` | Accepted, real-app tested (V1.4.0) | No `@` mentions; **no backlinks** ("linked from") |
| Inline database / linked view | "/" menu "Database (inline)" and "Linked view of database…" | `DatabaseBlock.tsx`, `features/database/inline.ts` | Accepted | **Not in the ribbon Insert tab.** **Dropped from page exports** (see 3.9) |
| Today's date | "/" menu, ribbon "Date"/"Time stamp" | `SlashMenu.tsx`, `Ribbon.tsx` | Accepted | Inserts plain text, not a date chip |
| Text styles | Bubble toolbar on a selection: Turn into, font, size, bold, italic, underline, strike, code, 18 text and 18 highlight colours, link, align, equation, remove formatting | `BubbleToolbar.tsx`, `lib/palette.ts` | Accepted | – |
| **Ribbon** (OneNote-style) | Tabs Home, Insert, Table (when in a table); collapse remembered | `Ribbon.tsx` | Accepted, real-app tested (combined test 2026-10-27) | Not customizable; keyboard-only use and screen reader not run |
| "/" menu | Text, Lists, Insert, Database, Direction, Quick groups (about 26 items) | `slash.ts`, `SlashMenu.tsx` | Accepted | – |
| Block handle | Drag a block; menu | `BlockHandle.tsx` | Accepted | Real mouse drag was long untested (early records) |
| **Right-to-left text** | Automatic per block (first strong letter) and per line (Shift+Enter); ribbon and "/" force a direction | `direction.ts` | Accepted, real-app tested | Windows keyboard-layout shortcut between lines NOT RUN; per-line direction not in exports |
| Page header | Breadcrumb, save state ("Saved & encrypted" / "Saved on this drive"), word count, **Favorite star**, "..." menu: Move to section, Export…, Move to trash (rows: Save as template) | `PageEditor.tsx` ~220-320 | Accepted | Favorite star is faint gray when off; no duplicate page, page lock, page history, full-width per page |
| **Columns** (side-by-side) | Cannot be made; kept when imported from Notion HTML or old pages | `more-blocks.ts` `Columns`/`Column` | **Dormant** (creation removed in A13 "reword and trim") | – |
| **Comments** | Cannot be made; old/imported comment marks keep their text | `more-blocks.ts` `Comment` | **Dormant** | – |
| Page templates, duplicate page, table of contents, breadcrumb block, synced blocks, tabs block, page history, presentation mode, bookmark cards, file/audio/video blocks | – | – | **Missing** (EX-41's 2026-11-03 list; re-searched, still absent) | – |

### 3.6 Databases (V1.5.0, refined in V1.5.1)

All database logic (filters, sorts, formulas, rollups, charts, automations) runs in Rust (`src-tauri/src/database/`, `commands/database.rs`, `commands/automation.rs`); the UI is `src/features/database/` (about 40 files).

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Databases as full pages and inline | Sidebar "New database…", section/page menus, "/" menu; New database dialog (blank or from a CSV) | `NewDatabaseDialog.tsx`, `create_database` | Accepted; PC round 1 found and fixed "From a CSV file" (Bug 1) | A vault with a database cannot be opened by V1.4.0 or older (by design, a backup is made first) |
| **Property kinds (23)** | Text, Title, Number, Select, Multi-select, Status, Date (with reminder), Person, Files, Checkbox, URL, Email, Phone, Formula, Relation, Rollup, Created time, Created by, Last edited time, Last edited by, ID, Place, Button | `database/types.rs` `PropKind`; `panels/PropertyMenu.tsx`, `cells/*` | Accepted | Person/Created by are plain text (no accounts); files in rows up to 900 MB, slow for hundreds of MB; no thumbnails for file cells; duplicating a property copies settings, not values |
| **Seven views** | Table (freeze, wrap, row numbers, resize, reorder, footers with calculations), Board (drag, swimlanes), Calendar (month/week), Timeline (zoom, dependency arrows), List, Gallery, Chart; view tabs: rename, duplicate, move, delete | `views/*.tsx`, `ViewKind` | Accepted; individual views were mostly checked in the mock/harness (115 assertions), with PC use of the main flows | Calendar week has no hour grid; timeline has no virtual scrolling (slow above ~500 rows) and no drag-to-create dependencies; list has no manual order; gallery "page content" cover is a placeholder |
| Filters, sorts, groups, sub-groups | Nested and/or filters with relative dates; multiple sorts; group by with date and number buckets | `panels/FilterPanel.tsx`, `SortGroupPanels.tsx`; `database/engine.rs` | Accepted | – |
| Formulas | Notion-style formula language | `database/formula.rs` (1,147 lines), `regex.rs`; three Notion differences asserted in `notion_compare.rs` | Accepted | Formula preview lags about 1 s |
| Relations and rollups | Two-way relations, rollups (depth limit 32) | `engine.rs`, `value.rs` | Accepted | – |
| Sub-items, dependencies | "Turn on sub-items" / "Turn on dependencies" | `enable_sub_items`, `enable_dependencies` | Accepted | – |
| Templates | Row templates; "Save as template" from a row page | `TemplatesDialog.tsx` | Accepted | Page templates outside databases missing |
| Row pages | Rows open as full pages (or side/centre peek) with properties above the text | `RowPeek.tsx`, `RowProperties.tsx`, `PageEditor.tsx` | Accepted | – |
| **Charts** | Chart view: **bar, line, donut, number**; x by property (date buckets, number steps), optional second grouping (stacked/grouped), y = count/sum/avg/median/min/max/…; labels, horizontal, colour, title; **click a bar or slice to see its rows** | `views/ChartView.tsx` (hand-drawn SVG), `ChartDrill.tsx`, `database/chart.rs`, `ChartConfig` | Accepted, real-app tested (V1.5.1 item 6, clickable) | **One chart per view.** No dashboard of several charts (EX-42 "dashboards of several charts not built"; the Notion board card says "a chart dashboard", which overstates it), no pie (donut only), no slicers or cross-filtering between charts, no KPI tiles beyond "number" |
| **Automations and the Button property** | When a row is added, a property changes or a date arrives (± minutes), if the row matches a filter: set a property, add a row, show a message, or move the row to trash; a Button property runs actions when pressed | `commands/automation.rs`, `AutomationsDialog.tsx`, `panels/ActionsEditor.tsx` | Accepted (V1.5.1 item 5 fixed "Add an action") | Limits: 3 chain steps, 200 actions, 50 new rows per run; nothing runs while locked or closed (date ones catch up 7 days); no Windows notifications |
| Reminders | Date reminders shown inside Notaya as a toast | `due_reminders_now`, `features/database/notices.tsx` | Accepted | In-app only (Windows notifications need a new crate) |
| Undo and redo | 50 steps per database, Ctrl+Z / Ctrl+Y | `features/database/undo.ts` | Accepted, real-app tested (V1.5.1 item 3) | Dropped when the last view of that database closes; in memory only |
| Lock database | Menu "Lock database" | `set_database_look` `locked` | Accepted | – |
| CSV import / export, other exports | Menu: Import CSV…, Export CSV / Markdown / HTML / Word / PDF (this view) | `commands/database_io.rs`, `database/csv.rs` | Accepted; five export formats tested through the real Save dialog (V1.5.0) | **No Excel `.xlsx`**; CSV is Excel-friendly (BOM, CRLF); 20,000-row export cap; CSV import 2 GB / 100,000 rows / 100 columns |
| Moving a database | Sidebar "..." > Move to (section) | `move_database` | Accepted | No drag, no move into a page, top 8 sections only |

### 3.7 Sticky notes and calculators

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Sticky board (in the app) | A **grid** of note cards (slightly tilted, newest edit first); buttons Select, Calculator, Open all, Close all, New note | `src/features/sticky/StickyBoard.tsx`; `list_stickies` sorts by last edit | Accepted, real-app tested | **Grid only** (no free layout, no corkboard look); **no drag to reorder**; no pins, strings, right-click |
| Sticky windows on the desktop | Each note opens as its own small frameless window you can move and resize anywhere on the desktop; colour (8: Lemon, Blossom, Mint, Sky, Lavender, Peach, Cloud, Charcoal), Bold, Italic, Checklist, calculator, **Keep on top**, delete (click twice), close (kept in the vault); position and size remembered | `StickyWindow.tsx`; `open_sticky`, `set_sticky_geometry`, `set_sticky_style` | Accepted, real-app tested | This already is "pinning to the desktop" in a basic form (always-on-top windows); notes hide when the vault locks |
| Select several | Select button or Ctrl+click; tick by click/Space/Enter; Select all/none; Delete (one transaction, asks first); Escape leaves | `StickyBoard.tsx`, `delete_stickies` | Accepted, real-app tested (combined test) | **No box-select or swipe**; no other bulk actions (colour, open, move) |
| Close all | Closes every sticky window (notes kept); also a tray entry | `hide_stickies` | Accepted; the **tray entry was NOT RUN** | – |
| Calculators | Basic and scientific (sin, asin…, Deg/Rad), own frameless always-on-top 360x640 windows, as many as you like; shared history saved encrypted in the vault (500 lines); Copy writes only the result | `Calculator.tsx`, `calc.ts` (no `eval`), `commands/calculator.rs` | Accepted, real-app tested (20/20, 21/21) | History can hold sensitive sums (security-design limit) |

### 3.8 Import

Verified against the code; the detailed study is `2026-11-07-notaya-import-export-study.md` in this folder (its one error is in section 8 below).

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Import pages | Dialog: pick files or a folder (the Rust side opens the picker); Markdown, text, HTML, Word `.docx`, PDF (text only), ZIP (Notion, Notaya's own), folders, any mix; progress, Cancel; result list with "left out" reasons | `ImportDialog.tsx`; `commands/import_notes.rs`, `importers/*` (~9,000 lines, ~97 converter tests) | Accepted, real-app tested (48/48 real-dialog imports in V1.4.0; Notion rounds in V1.5.0 and V1.5.1) | Always a **new notebook** (no import into an existing place, no re-import/update); page order by name, not Notion's order; Notion views, formulas (as values), people (as text) not carried; embeds/video/audio dropped; no `.xlsx`/CSV as pages; very large imports need ~10x the file size in memory |
| Notion export into real databases | Notion "Markdown & CSV" and "HTML" exports, alone or together, become databases with Status, Number formats, Select/Multi-select, two-way Relations, row pages folded into rows; icons, covers, callouts | `commands/notion_db.rs`, `importers/plan.rs` | Accepted, real-app tested (148/148 relation cells, Hubs test) | Word and PDF output of the Hubs import NOT RUN; **the dialog text still says "Databases (tables)… are not imported"** (`ImportDialog.tsx:162`), which is out of date |
| Entry points | Hover icon in the Notebooks header, "Import pages from another app" (empty sidebar only), command palette | `Sidebar.tsx`, `CommandPalette.tsx`; one store flag `importOpen` | – | **No top-bar Import button** |
| Import from a portable copy (first start), Restore from backup, database Import CSV | Separate features, separate code (`import.rs`, `restore.rs`, `database_io.rs`) | – | Accepted | Must not be mixed up with page import in a redesign |

### 3.9 Export

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Export dialog | Scope: a page, chosen pages, a section, several sections, a notebook, several notebooks; **Include sub-pages** (on by default); format HTML, Markdown, PDF, Word; one file or separate files in a ZIP (re-importable); warns files are not encrypted; the Rust side opens the Save dialog | `ExportDialog.tsx`; `commands/export.rs`, `export/*` (~8,000 lines, ~90 tests) | Accepted, real-app tested (40/40 real-dialog exports) | Sub-pages follow sidebar order, one heading level deeper, each on a new sheet in PDF/Word. PDF: flat bookmarks, **no printed table of contents, no clickable links between pages** (web links are clickable); Word: real headings, no inserted TOC; Greek/Cyrillic/emoji print as "?" in PDF; GIF/WebP not embedded in PDF; Mermaid exported as code |
| **Inline database on a page** | – | `export/doc.rs` has no case for `databaseBlock`; its fallback keeps nothing for an atom with no children | **Bug: silently dropped from every page export** (confirmed in code, not by running) | Databases also never take part in section/notebook exports |
| Entry points | Page "..." menu, sidebar "..." menus (page, section, notebook), command palette, database menu | `openExport(kind, id)` | – | **No top-bar Export button next to the Favorite star** |

### 3.10 Security, encryption, backups

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| Optional security | New notes open with **no password** (stored in the encrypted format with a public built-in key: not secret). Settings > Security: Turn on (master password ≥10 chars and "Strong", optional keyfile), Turn off (needs the password); both re-key the whole vault | `commands/vault.rs`, `vault/*`, `openvault.rs`; `SecurityDialogs.tsx` | Accepted, real-app tested (optional-security script 47-52 checks) | Turning security off means not confidential (by design) |
| Encryption | XChaCha20-Poly1305 per record, Argon2id (256 MiB, 3 passes, 4 lanes) + HKDF, optional keyfile; tamper detection; padded sizes; keys only in Rust | `crypto.rs`, `vault/header.rs`, `vault/store.rs` | Accepted (unit-tested; P4 "keys never leave the core" not independently reviewed) | A2: an older copy of one record is not detected (owner decision open); Argon2 settings fixed (A1) |
| Lock and auto-lock | Lock button / Ctrl+L, idle (1-60 min), Win+L, sleep, drive removal, optional on minimize; every window saves first (≤2 s) | `state.rs` monitor, `system.rs` | Accepted; Lock button and close tested in the real exe; idle, Win+L, sleep, USB reported passing by Mohamed by hand (2026-10-24) | Only with security on; an open vault never locks; wrong-password delay resets on restart |
| Screenshot protection | Settings switch (off by default; only with security on) | `ui.rs` | Accepted | The "on = black" state was never observed in a real window |
| Backups | Automatic on lock and quit (newest 10), hourly for open vaults (3), "before delete" copies (5); Back up now; list | `backup.rs`, `commands/backup.rs` | Accepted, real-app tested | Turning security on (default tick) deletes unprotected backups |
| Restore from backup, Start over | On the lock screen and in Settings; the current vault is kept as `restore-undo-*` / `set-aside-*`, never deleted | `restore.rs`, `RestoreDialog.tsx` | Accepted, real-app tested in parts | Native file-dialog path end to end and a crash between the two renames not tested |
| Content Security Policy, picture checks, no network | Strict CSP (no outside scripts, styles, images, connections); pictures checked by bytes; SVG refused | `tauri.conf.json`, `lib.rs` `clatt` protocol | Accepted | `freezePrototype` is off since V1.5.1 (for Mermaid); WebView2 itself contacts Microsoft at start-up; **every command can be called by any Notaya window** (recorded weak point; relevant to the AI CLI) |

### 3.11 Windows integration and running the app

| Feature | What you see | Code | Status | Known gaps |
|---|---|---|---|---|
| **Tray icon** | Left click opens Notaya; right-click menu: **Open Notaya, Close sticky notes, Quit Notaya** | `src-tauri/src/shell.rs` `build_tray` | Accepted, real-app tested (real mouse, V1.4.0); "Close sticky notes" NOT RUN | No New sticky note, no Calculator, no shortcuts to app parts; the tray icon is not hidden from screenshots |
| Close to tray | Close button hides Notaya (a vault with a password is locked first); Settings switch | `lib.rs` window event, `state::hide_to_tray` | Accepted, real-app tested | An open vault stays open in the tray |
| **Start with Windows**, start minimized | Settings > Windows switches; one per-user registry value | `shell.rs` registry module | Accepted; **start at a real Windows sign-in not tested** (EX-37; project memory lists it as untested) | – |
| Setup and portable | NSIS per-user setup (no admin), Start menu, optional desktop shortcut; portable ZIP keeps notes in `data\` next to the exe; installed notes in `%LOCALAPPDATA%\Notaya Notes`; uninstaller never deletes notes | `tauri.conf.json` bundle, `packaging/`, `portable.rs`, `tools/windows-build/` | Accepted; **the V1.5.0 and V1.5.1 setups were never run**; unsigned (SmartScreen not run) | Empty registry key left after uninstall (harmless) |
| One copy at a time | A second start shows the running window; an older Notaya running is detected and named | `tauri_plugin_single_instance`, `legacy_guard.rs` | Accepted | The second start's command-line arguments are ignored today (a possible hook for a CLI, see 6.3) |
| Error log | Settings > Error log: local crash and error records (no note text), open the folder | `logging/*`, `ErrorLogPanel.tsx` | Accepted, real-app tested (V1.4.0) | A real Rust panic could not be provoked in tests; a killed WebView2 leaves a blank window and logs nothing |
| Start-up failure messages | A native message box if the data folder is read-only or WebView2 is missing | `lib.rs` | Accepted | – |

### 3.12 Settings and About

Settings sections (`SettingsView.tsx`): **Appearance** (3.4), **Windows** (close to tray, start with Windows, start minimized), **Error log**, **Security (optional)** (security on/off, auto-lock minutes, lock when minimized, block screenshots, master password and keyfile), **Backups** (back up now, list, restore, open data folder), **About** (version, vault location, encryption and key-derivation text, network note, **Credits: "Built on open source software" + "Open source licenses…"** dialog with the full third-party notices, "Good to know" list).
**Gaps:** no Shortcuts page, no Home or banner settings, no per-feature toggles; About names no people (Ahmed's board asks for Mohamed Sayed, Ahmed Emara, Claude, ChatGPT); no guide, help or tour.

### 3.13 Developer and test tools (not features for users)

- `npm run mock`: the UI with fake data in a browser (`dev-mock/`, 12 Playwright check scripts: database, views, automations, ribbon, Mermaid, RTL, calculator, page looks).
- `tools/real-exe-tests/`: PowerShell/Node runners for the real exe on throwaway data (restore flow, lock flow, optional security, tray, installed).
- `tools/usage-ledger/`, `docs/ledger/`: the usage ledger (records, 2026-11-07).
- Rust tests: 517 (MSVC) on v1.5.1, plus `roundtrip_tests.rs` (export then import back).

---

## 4. Confirmed missing (searched in `src/` and `src-tauri/src/`)

Ctrl+F find/replace in a page; reopen last page; mouse right-click menus; customizable top bar; editable shortcuts and a shortcuts page; page tabs; code-block language picker; Mermaid zoom/download/copy/wrap and paste detection; page duplicate, page templates, table of contents, backlinks, synced blocks, page history, toggle headings, `@` mentions, date chips; sticky free layout, drag reorder, box/swipe select, pins, strings, corkboard, sticky right-click, tray "new sticky"; slicers, dashboards, pie; `.xlsx`; import into an existing notebook and re-import without duplicates; guide, tour, named-person credits; Whiteboard, handwriting, pens; OneNote mode; **any command-line, local server, named pipe or other outside entry point** (no `clap`, no `tauri-plugin-cli`, no listener; the only arguments read are `--minimized` and the legacy-guard check).

---

## 5. Roadmap items mapped against what exists

Versions are from the Notion Version Feature Board (54 cards, read 2026-11-07 ~21:10 UTC). "Touches" names what a build would change and what it must not break.

| # | Roadmap item (board version) | What exists today | What is actually new | Touches / do not break |
|---|---|---|---|---|
| 1 | **Sticky notes**: grid + free modes, drag to reorder (V1.6), swipe/box select (V1.6), coloured push-pins and strings, corkboard in app, right-click, tray right-click to create, desktop pinning | Grid board; 8 colours; desktop windows with Keep on top (basic desktop pinning **exists**); Select mode with Select all and Delete; Close all/Open all; tray "Close sticky notes" | Free layout and board positions (today `x,y,w,h` are the **desktop window** geometry, not a board position: a second set of coordinates is needed), a sort order field (none today; list is by last edit), box select, pins and strings (new data), corkboard look, right-click menu, tray "New sticky note" | `Sticky` record in `model.rs` (add fields with `#[serde(default)]`), `StickyBoard.tsx`, `commands/stickies.rs`, `shell.rs` tray; keep lock-hides-stickies and the A17 save-before-lock flow |
| 2 | **Mermaid / code blocks** (V1.6: choose a code block's type and language) | Diagram block (separate), "```mermaid", Turn into > Diagram, imports turn Mermaid code into diagrams, samples, show/hide code; code blocks highlight with auto-detect | Language picker on every code block; Mermaid as a code-block language (merge or link the two blocks); paste detection; collapsed code; toolbar zoom, download picture, copy, wrap | `extensions.ts` (CodeBlockLowlight), `MermaidBlock.ts`, `mermaid.ts`, export `doc.rs` (`mermaidBlock`), importers (`doc.rs` Mermaid rule). Keep the hostile-code rules (strict mode, 50,000-char cap, 15 s, sanitizing) and the `freezePrototype` decision |
| 3 | **Export "Save as"** next to Favorites (V1.6) | Full dialog exists (scope, sub-pages, 4 formats, one file or ZIP) | A top-bar/page-header button, "Save as" wording; PDF contents page, nested bookmarks, internal links; Word TOC; **fix the dropped inline database**; databases inside page/notebook exports; `.xlsx` | `ExportDialog.tsx`, `export/*`; keep round-trip tests green |
| 4 | **Import button** in the top bar, clearer icons (V1.6) | Dialog and converters complete; entry points are hidden-ish | A visible button with label; fix the stale dialog sentence; icon contrast in all themes (Favorite star `text-faint`) | `TitleBar.tsx`, `ImportDialog.tsx`, theme tokens; **do not touch converters** |
| 5 | **Customizable top bar and changeable shortcuts** (V1.6) | Fixed top bar; shortcuts hard-coded in 6+ files (3.3) | A shortcut registry (one list), a Settings > Shortcuts page, top-bar action slots with order | `Shell.tsx`, `Sidebar.tsx`, `navHistory.ts`, `main.tsx` (blocked keys), TipTap keymaps; avoid clashing with TipTap's own keys |
| 6 | **New-notebook button always visible** (V1.6) | Exists, but hover-only in the Notebooks header | Show it (and Import) permanently | `Sidebar.tsx` `GroupHeader` only. Tiny |
| 7 | **Home banner per time of day** (V1.6) | Daily rotating gradient + time-of-day greeting | Picture per time slot (morning, afternoon…), user upload, remembered as default | `HomeView.tsx`; a new settings field (encrypted settings record, like `search_history`); pictures through the existing attachment store (`add_attachment`, `clatt`) |
| 8 | **Ctrl+F uses Notaya's search** (V1.6) | Nothing of Notaya's own; WebView2's find bar shows | Find (and maybe replace) inside the open page, highlight, next/previous; decide whether Ctrl+F also searches the database view | `main.tsx` key list, editor plugin (ProseMirror decorations), `DatabaseView.tsx` search box |
| 9 | **Reopen where you left off** (V1.6) | Always opens on Home; lock reloads the window | Remember view + page/database/row (and maybe scroll); restore after start and after unlock | `store/app.ts`; storing the last page id: browser storage is plain text, the settings record is encrypted (choose; a page id reveals little but security-design lists what is plain) |
| 10 | **Move databases by hand in the sidebar** (V1.6) | Move to another section by menu (first 8 sections) | Drag databases to reorder and between sections/pages; databases have no `order` used by the tree today (check `DatabaseMeta`) | `Sidebar.tsx` dnd, `reorder` (handles notebooks, sections, pages; databases to check), `move_database` (always clears `parent`) |
| 11 | **Right-click menus** on pages, sections, databases, notebooks, sticky notes (V1.6) | The same menus exist behind "..." buttons and the keyboard Menu key; the browser menu is blocked | Open the existing menus on `contextmenu`; add Change icon, Duplicate, Rename items; sticky cards need a menu | `Sidebar.tsx` `Row`, `main.tsx` contextmenu blocker, `StickyBoard.tsx`. Keep text fields' copy/paste menu |
| 12 | **Richer tray menu** (V1.6) | Open, Close sticky notes, Quit | New sticky note, Calculator, open Notes/Sticky notes/Settings | `shell.rs` `build_tray`; needs the vault unlocked for new sticky (refuse or open Notaya when locked) |
| 13 | **AI CLI** (V1.6, critical) | No outside entry point; ~120 internal commands (section 6) | Everything in section 6 | All of `commands/*`; security-design (new trust boundary) |
| 14 | **Pie charts and Power BI-like dashboards** (V1.6) | Bar, line, donut, number; one chart per view; click-through to rows | Pie (donut without hole is small), dashboards with several charts and KPIs, layout, animation, names/labels | `ChartView.tsx`, `database/chart.rs`, `ChartConfig`, a new "dashboard" item or view kind (vault format: older Notaya may not read it) |
| 15 | **Slicers** (V1.6) | Per-view filters only | Click-to-filter controls that drive several charts/views at once | Filter engine exists in Rust (`engine.rs`); needs a shared filter state across views |
| 16 | **Notion-style tabs** (V1.6) | None (databases have view tabs; not page tabs) | Several pages open at once in tabs | `store/app.ts` (single `activePageId`), `navHistory.ts`, `NotesView.tsx`; editor instances per tab and the save-before-lock flush for each |
| 17 | **Guide** (V1.7) | None in the app; records only | A guide (an Artifact per the board) in Notaya's theme | Outside the app or a help page; this inventory is its raw material |
| 18 | **App tour** (V1.7) | First-run screen only (Start new notes or Import) | Skippable guided tour | `UnlockScreen.tsx` first run, `Shell.tsx` |
| 19 | **Names in About** (V1.7) | Open-source credits only | A "Made by" line: Mohamed Sayed, Ahmed Emara, Claude, ChatGPT | `SettingsView.tsx` About; `README.md` credits. Tiny |
| 20 | **Whiteboard** (V1.7) | Nothing (no canvas, ink or pen code) | A new section like Sticky notes: canvas, pens, handwriting, templates | New item kind (vault format), new view in `Shell.tsx`; likely a new dependency (ask first) |
| 21 | **OneNote mode** (V1.8) | The ribbon is OneNote-style; notebooks/sections/pages mirror OneNote; free-position pictures exist | Undefined ("details not set"); OneNote's free-canvas pages would build on Whiteboard and free pictures | Not designed |
| – | Later candidates on the board | Per-line direction in exports, lighter big imports, clippy warnings, unrun V1.5.1 checks, audit findings, licence | As recorded | – |
| – | North star: Notion as an always-on local backup | Strong Notion import; databases | Import into existing place + re-import by Notion id (the 32-hex id is already parsed in `plan.rs`), editor parity list (3.5 "Missing" row), views not exportable from Notion | See also EX-41's plan (section 8) |

---

## 6. The AI CLI foundation: what exists and how features could become named actions

### 6.1 Facts in the code today

- **The UI talks to the core only through Tauri IPC commands** (`src/lib/ipc.ts` → `invoke`). There are about 120, registered in `src-tauri/src/lib.rs`. Nearly all real work (vault, notes, databases, formulas, charts, automations, import, export, search, backups) is in Rust. So most "features" already are named functions; an outside channel would wrap them, not rewrite them.
- **No outside entry point exists.** Commands can only be called from a Notaya window. The security design records that any Notaya window can call any command (a known weak point) and defends it with the strict CSP. An AI channel is therefore a **new trust boundary** and a security-behaviour change (needs Ahmed's own words, and a `security-design.md` promise).
- **Locked means refused.** Commands use `state.with(...)`, which fails while the vault is locked. That already matches "locked notes only when unlocked".
- **Paths:** by design the UI never sends file paths; the core opens every file dialog (import, export, restore, keyfile, attach). The converters themselves take paths (`import_from_with`, `export::collect/write/write_zip`), so path-taking AI variants are thin wrappers, but they lift a deliberate rule and need their own limits (for example, never inside the data folder, which the exporter already refuses).
- **Page content format:** pages are TipTap JSON. The **Markdown importer already converts Markdown (and HTML, Word) into that JSON**, and the **Markdown exporter converts it back** (`importers/markdown.rs` + `importers/doc.rs`; `export/markdown.rs` + `export/doc.rs`). An AI can read and write pages as Markdown by reusing them. The export side drops inline database blocks today (3.9), which an AI read would inherit.
- **The open window does not refresh from outside.** The UI keeps its own copy of the tree, settings, sticky list and open database (`store/app.ts`, `features/database/dbStore.ts`), refreshed from each command's reply. Only a few backend events exist (`vault-locked`, `vault-locking`, `app-notice`). If an AI changes notes while the window is open, the window shows stale data and a pending UI save (for example `save_database` sends the whole database object, debounced 700 ms; `save_page` the whole page) can **overwrite** the AI's change. A "something changed" event and per-record version checks are needed before any outside writer.
- **Some behaviour lives only in the UI** and would need moving or duplicating for the AI: inline database insertion into a page (`features/database/inline.ts`), applying row templates and "save as template" (`dbStore.ts`), database undo/redo, navigation history, which page is open, ribbon/sidebar layout, appearance cache.
- **A precedent to copy:** `commands/automation.rs` is a closed list of named actions (`setProp`, `addRow`, `notify`, `trash`), run in Rust, validated (`types.rs` checks kinds), with hard limits and no network/programs/files. The AI action layer can follow the same shape, with an added confirmation tier and an activity log.
- **Possible hook (not a decision):** the single-instance plugin already receives the arguments of a second `Notaya.exe` start and currently ignores them; a CLI could hand a request to the running app this way, or a local named pipe could. Both are design questions with security consequences.

### 6.2 Suggested action names per area (proposal only; maps to existing commands)

Tiers: **read** (allowed while unlocked), **write**, **destructive** (confirmation required, logged), **never for AI** (security and keys).

| Area | Suggested actions | Existing command(s) | Tier / notes |
|---|---|---|---|
| Status | `app.status`, `app.version` | `vault_status` | read |
| Tree | `tree.list` | `list_notes` (returns notebooks, sections, page metadata, databases) | read |
| Notebooks | `notebook.create`, `notebook.update` (name, icon, colour), `notebook.reorder`, `notebook.delete` | `create_notebook`, `update_notebook`, `reorder`, `delete_notebook` | delete = destructive (it already makes a safety backup) |
| Sections | `section.create/update/reorder/delete` | `create_section`, `update_section`, `reorder`, `delete_section` | delete = destructive |
| Pages | `page.get` (JSON or **Markdown**), `page.create`, `page.create_sub`, `page.write` (Markdown in), `page.set_look` (icon, cover), `page.favorite`, `page.move`, `page.reorder`, `page.trash`, `page.restore`, `page.delete_forever`, `trash.empty` | `get_page`, `create_page`, `create_subpage`, `save_page`, `set_page_look`, `set_page_favorite`, `move_page`, `reorder`, `trash_page`, `restore_page`, `delete_page`, `empty_trash`; Markdown via importer/exporter | trash = write (undoable); delete/empty = destructive |
| Pictures | `attachment.add` | `add_attachment` | write; size and type checks already exist |
| Search | `search.query`, `search.history` | `search`, `search_history` | read |
| Databases | `db.create`, `db.get`, `db.update_schema` (properties, views, filters, sorts, groups, charts, automations), `db.query_view`, `db.row.create/get/set_cells/duplicate/reorder/trash`, `db.button.press`, `db.chart.data`, `db.move`, `db.set_look` (incl. lock), `db.enable_sub_items`, `db.enable_dependencies`, `db.search_rows` | `create_database`, `get_database`, `save_database`, `query_view`, `create_row`, `get_row`, `set_cells`, `duplicate_row`, `reorder_rows`, `press_button`, `chart_data`, `move_database`, `set_database_look`, `enable_sub_items`, `enable_dependencies`, `search_rows` | `save_database` replaces the whole object: an AI needs finer actions (add property, add view, set filter) to avoid overwriting; dashboards and slicers (roadmap) should be designed with these names from day one |
| Database files | `db.file.attach`, `db.file.save_as`, `db.file.remove` | `attach_file`, `save_file_as`, `remove_file` | path-taking variants needed |
| Import | `import.preview`, `import.run` (target: new notebook today; "into" later), `import.cancel`, `db.import_csv` | `importers::plan::build`, `import_notes` / `import_from_with`, `import_cancel`, `import_csv` | write; path-taking variant; returns the existing summary and "left out" list |
| Export | `export.run` (scope, ids, format, sub-pages, single/zip, destination), `db.export` (csv, md, html, docx, pdf; xlsx later) | `export_notes`, `export_csv`, `export_database` | read-out (files are unencrypted: log it) |
| Sticky notes | `sticky.list/get/create/write/style/open/close/close_all/delete/delete_many` | `list_stickies`, `get_sticky`, `create_sticky`, `save_sticky`, `set_sticky_style`, `set_sticky_geometry`, `open_sticky`, `close_sticky`, `hide_stickies`, `delete_sticky`, `delete_stickies` | delete = destructive |
| Calculator | `calc.open`, `calc.history`, `calc.add`, `calc.clear` | `open_calculator`, `calc_history`, `add_calc_entry`, `clear_calc_history` | clear = destructive |
| Settings | `settings.get`, `settings.set_appearance`, `shell.get/set` (tray, start with Windows) | `get_settings`, `save_settings`, `get_shell_prefs`, `set_shell_prefs` | write; must not touch security fields |
| Backups | `backup.now`, `backup.list` | `backup_now`, `list_backups` | read/write |
| Error log | `log.open_folder` | `open_log_folder` | read |
| **Never for AI** | – | `unlock_vault`, `enable_security`, `disable_security`, `change_master`, `pick_keyfile`, `create_keyfile`, `restore_vault`, `start_over`, `pick_backup_file`, `import_portable_notes`, `quit_app`, `lock_vault` (debatable), `password_strength` | keys, passwords and whole-vault replacement stay with the person |
| New (Ahmed's ask) | `activity.log` (who: AI or person, what, when, counts), `ui.refresh` event, `ui.open` (show a page to the person) | – | needed for trust and for the "seamless" feel |

### 6.3 Design points to settle with Ahmed (not decided)

1. How the outside program reaches Notaya (second-start arguments, a local pipe, or a CLI that opens the vault file itself while Notaya is closed) and how it proves it is allowed.
2. Whether the AI may work while the window is open (needs change events and version checks, 6.1) or only one at a time.
3. Confirmation for deletes: in the Notaya window, or in the AI's own conversation.
4. Where the activity log lives (encrypted in the vault, like settings) and whether the person can undo an AI change (today: trash for pages, whole-vault backups).

---

## 7. Corrections to the records and notes (found while checking)

1. **Import/export study (this folder, 20:56):** "Right-click on a notebook, a section or a page in the sidebar: Export…" is wrong. There is no mouse right-click menu; the menu is the "..." button (or Shift+F10 / Menu key). Everything else I checked in that study matched the code (Excel absent, inline database dropped, flat PDF bookmarks, stale import sentence, entry points).
2. **Notion board, card "Notion-style databases" (V1.5):** says "a chart dashboard"; the code and EX-42 say one chart per view, no multi-chart dashboard.
3. **Notion board, card "Sticky notes and a built-in calculator" (V1.0):** "also a small one inside each sticky note" was replaced in V1.1.0 by pop-out calculator windows (A25).
4. **Notion board, card "Notes, pages and the block editor" (V1.0):** "covers and icons" were removed in A13 (V1.0 era) and came back in V1.5.1.
5. **The "missing" EX-39 and EX-41 exist on unmerged branches** (both 2026-11-03, Mohamed's side): `claude/project-thread-73zkp7` holds EX-2026-11-03-41 "Notaya as an offline Notion" (`docs/plan/notion-offline-backup-roadmap.md`, a gap list and phased roadmap, and `notion-migration-plan.md`), **postponed by Mohamed 2026-11-03 19:18:50 UTC**; this is draft PR #45. `claude/project-thread-gljl3w` holds EX-2026-11-03-39, a "Notaya Token Saver" Claude Code plugin. Neither is on `main`. EX-41's phases 1-3 (databases, views, formulas, charts) were then built as V1.5.0; its editor-parity list (section 4.1 there) is still almost all open and is the best existing list for Ahmed's Notion north star. A third unmerged branch, `claude/project-thread-mp3lcf`, has one records-renumbering commit.
6. `ImportDialog.tsx:162` "Databases (tables)… are not imported" and `docs/plan/import-what-carries-over.md` are out of date (already noted in the study).
7. `tauri.conf.json` identifier is now `app.notaya.notes` (A32); older records that say the identifier `app.cipherleaf.vault` must stay are history.

---

## 8. What I did not check

- I ran nothing: no build, tests, mock or real app. Every "real-app tested" comes from the records, and every "missing" from code search on `9474511`.
- I did not read every UI file line by line (databases panels and views, cell editors, ExportDialog/ImportDialog internals, unlock screens were read by headers, menus and grep); the database feature list relies on `types.rs`, the menus and the V1.5.0 records.
- WebView2's own Ctrl+F behaviour in the shipped app is inferred from the code (not blocked) and Ahmed's board card, not observed.
- I did not open the 2026-11-03 Notion migration plan in full, the token-saver plugin, or PR #45's discussion.
- Real-app status of individual database views, the Home page upload dialog and Start with Windows at sign-in is as the records state it; I could not add evidence.
