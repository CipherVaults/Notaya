# Notaya AI CLI: the full action list (draft for planning)

**Who and where:** Ahmed (AE0101) via Claude, in a cloud thread (no PC, no Remote Control). Written 2026-11-07, about 23:45 UTC (container clock).
**Baseline:** `origin/main` at `9e3e4e3` (read on this thread; the command list was read from `src-tauri/src/lib.rs`). Nothing was built or run.
**Ahmed's request:** 2026-11-07 23:39:31 UTC (project chat, relayed by the coordinator at 23:39:40): "push ahead full throttle ... planning for these features and phases", including "all of that CLI list". The coordinator's brief asked for every named action the outside AI can take, grouped by area, with what it does, permission level, confirmation, and logging and undo.
**Scope and write area:** this one file only. No code, no repository record changed. The coordinator decides whether and where it is committed.
**Updated:** 2026-11-07, about 23:58 UTC (container clock), with Ahmed's answers of 23:53:46 UTC (section 12).
**Result state:** *produced*. Not verified against a running app (there is no AI CLI yet), not accepted. It is a **planning draft**: names, levels and rules are proposals for Ahmed to approve or change.
**Sources:** project memory (FEATURE DISCUSSION, SECURITY BY DESIGN, AI CLI), the feature inventory (sections 3 and 6), the dashboards, whiteboard and import/export studies, the Notion parity study (23:39 UTC), `docs/security-design.md` (via the security digest), `docs/plan/approval-and-amendments.md` (authority rows 20:38 to 22:33 UTC).
**Style rules kept:** plain words; no trademarked names proposed for Notaya's screens (the word "Notion" appears only where it names the other app as an import source, never as a Notaya screen name).

---

## 1. The short version

1. **About 300 named actions in about 25 areas** cover everything agreed so far: every existing Notaya feature, every feature agreed in the discussion phase, and the page features the Notion parity study found missing (columns, callouts with colour and icon, folding headings, @-mentions and backlinks, links to a saved view, file blocks, row-page layouts, property descriptions).
2. **Four permission levels** (agreed by Ahmed, 23:53:46 UTC), chosen only by Ahmed in Notaya's Settings: **Off** (the default), **Look** (read only), **Edit** (create, change, move, arrange; deletes go to the recycle bin after you confirm), **Full** (Edit plus files in and out through Notaya's own exchange folder, changing Notaya's look and non-security settings, and emptying the bin with a strong confirm). At Full the AI can do whatever you can, including removing whole pages, hubs and notebooks after your confirm (Ahmed: "the AI should be able to have full authority"). The AI can never see or change this switch. Notebooks can be **hidden from the AI** (agreed).
3. **Three kinds of confirmation**: none (ordinary edits, which version history can undo), **Ask** (one click in the Notaya window: deletes to the bin, anything that leaves the vault, and big changes of 20 items or more if you keep that option on), and **Strong** (type a word: deleting forever, emptying the bin). Confirmations appear **in the Notaya window only**, never in the AI's own chat, so the AI cannot approve itself.
4. **Every change is logged and undoable.** Each AI request becomes one "change set" in a tamper-evident activity log, marked "with Claude" (or "with Codex"). The Activity panel has "Undo this change"; page and board edits also show in version history.
5. **A short "never for the AI" list**: passwords, keyfiles, security on/off, restore and start over, the AI switch itself, opening programs, web links or Windows folders, the clipboard. Locking and unlocking (vault, pages, databases) is set aside for now (Ahmed deferred it).
6. **Most existing features are already one step from an action**: about 120 commands in the Rust core do the real work today. The new pieces are the safe door in (section 3), finer-grained saves so an AI change never overwrites what the person types, a "something changed" signal for the open window, the confirmation queue and the activity log (section 9).
7. **Claude and Codex can both be connected** (agreed), each with its own pairing and level; they never edit the same item at once (section 3, item 6).

---

## 2. Rules that apply to every action

These are the agreed SECURITY BY DESIGN rules (Ahmed, 22:45 and 22:49 UTC), written as rules the code must keep. Section 10 lists what goes into `docs/security-design.md` when built.

1. **Sealed in.** The AI reaches only Notaya's own notes and settings through this closed list of actions. No action runs a program, reads or writes an arbitrary file, opens the internet, opens a web link, or touches the clipboard. The only file door is Notaya's own **exchange folder**, kept with Notaya itself (section 8.8).
2. **Off by default.** A new or upgraded Notaya has the AI switch off. Only Ahmed turns it on, in **Settings > AI access**, in the Notaya window. No action, shortcut, menu or import can turn it on.
3. **The AI cannot change its own permissions.** No action reads the pairing key or changes the switch, the level, the exchange folder, the hidden notebooks (2.8) or the confirmation rules. `ai.permissions.get` only tells the AI what it may do.
4. **Locked means refused.** While the vault is locked every action fails with "Notaya is locked". The AI cannot unlock it. A locked database (today's "Lock database") can be read but not edited until the person unlocks it. Locking and unlocking pages, databases or the vault is **not** an AI action for now (Ahmed deferred page locks and "lock now", 23:53:46 UTC).
5. **Deletes go to the recycle bin after a confirm.** Removing a whole item (notebook, section, page, database, row, sticky note, board, dashboard, template, tab group) is a delete: Ask, then the recycle bin. Removing text or blocks inside a page is an edit, undone through version history, with no confirm (otherwise the AI could not edit at all).
6. **Every AI edit is undoable.** Pages, boards and canvases through version history; whole items through the recycle bin; settings and arrangements through "previous value kept" in the activity log. The only things that cannot be taken back are files written to the exchange folder, and those are logged and need Ask.
7. **Tamper-evident log.** Each entry is linked to the one before it (a hash chain, a proven pattern) and stored encrypted in the vault. The AI can read the log but never write, edit or clear it. Only the person can clear it, in Settings, with Strong.
8. **Big changes can get a simple confirm, as an option.** Ahmed (23:53:46 UTC): optional, not forced; 20 is good. So Settings > AI access has **"Ask me before big changes"** with a number (default 20 items). When on, one request that touches 20 items or more, replaces more than half of a page the person wrote, changes a property's kind (which can lose data), or deletes a property, view or column with data in it, asks first. When off, only deletes and things leaving the vault ask.
9. **The person always wins, and AIs don't collide.** If the person or the other AI is editing the same page, row or board, the AI's write is refused with "item changed, read it again" (version check per item), never silently merged over someone else's work.
10. **Same rules as the screens.** An action can do only what the person could do with the mouse, with the same limits (import sizes, automation limits, formula limits, 25 MB pictures, and so on). Nothing an AI writes can contain scripts, HTML, SVG or outside pictures; text and Markdown go through the same cleaning as a paste or an import.

- 2.8 **Hidden from the AI** (offered as an addition, **agreed** by Ahmed 23:53:46 UTC), per notebook (Settings > AI access): a notebook the AI cannot see at all, for example personal finance or medical notes. Off by default, so by default the AI sees everything. A hidden notebook is left out of every read, search, export and count the AI gets; the AI is told only "some notebooks are hidden".

**Additions (not asked for, offered):**
- 2.9 **Preview first.** Every write action accepts `preview: true` and answers with what it would change, without changing anything. Good for "show me before you do it".
- 2.10 **Batches.** Every write accepts several items at once, and `batch.run` runs a list of actions all-or-nothing as one change set (one undo, one confirm).

---

## 3. How the AI reaches Notaya (proposal for planning, not decided)

The inventory left four design points open (section 6.3). A proposal using proven patterns, so we do not invent our own security:

1. **The door:** a small companion program, the **Notaya bridge**, that speaks MCP (the standard tool protocol both Claude Code and Codex already use). It talks to the **running** Notaya over a Windows named pipe that only Ahmed's own Windows account can open (the same pattern desktop password managers use for their command-line tools). No network port.
2. **Pairing:** the first time, Notaya shows a pairing request in its window ("Claude Code wants to connect. Allow?") and remembers that bridge with a key kept in the vault. Ahmed can remove any pairing in Settings.
3. **Only while Notaya runs and is unlocked.** The bridge never opens the vault file itself. When Notaya is closed or locked, every action answers "Notaya is locked or not running".
4. **The window stays live.** When the AI changes something, the open window refreshes that item at once (the "something changed" signal) and shows a small "Claude is working" marker in the top bar with a **Stop AI** button.
5. **Confirmations in the window only** (the proposal; Ahmed did not object). Ask and Strong confirmations appear as a Notaya dialog (or a tray notice that opens it, when the window is hidden). The AI's request waits up to 5 minutes, then fails with "not confirmed". Known limit: an AI that can also control the mouse on the PC could click it; the Strong confirm (typing a word shown in the dialog) reduces this.
6. **Two AIs at once** (agreed: "as long as Claude and Codex do not collide"). Each AI has its own pairing, name and level. Before a job an AI **claims** the items it will work on (`ai.claim`); an item claimed by the other AI refuses writes from it until released or the claim times out (30 minutes without activity). The top-bar marker shows who is working on what, and the activity log names each AI. The coordinator (the mother thread) gives Claude and Codex separate parts of the work; the claims are the safety net, not the plan.

---

## 4. How to read the tables

| Column | Values |
|---|---|
| **Level** | **Look** = read only. **Edit** = create, change, move, arrange, delete to the bin. **Full** = files, settings, emptying the bin. Each level includes the ones before it. |
| **Confirm** | **–** none. **Ask** = one click in the Notaya window. **Ask*** = only when it is a big change (rule 8). **Strong** = type a word in the Notaya window. |
| **Undo** | **H** = version history of that page, board or canvas. **B** = the recycle bin. **P** = previous value kept in the activity log ("Undo this change"). **V** = view only (nothing stored, nothing to undo). **–** cannot be undone (logged). Every write is logged; reads are counted in the log as one line per AI session, not one per read. |
| **Today** | **Exists** = wraps an existing Notaya command. **Part** = partly exists. **New** = comes with an agreed feature. **Gap** = a feature the Notion parity study found missing from both Notaya and the agreed list. |

Names use `area.action` (like `page.create`). Parameters are left to planning.

---

## 5. The AI's own session and the open window

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `ai.hello` | Says Notaya's version, whether the vault is unlocked, and the AI's level | Look | – | V | Part (`vault_status`) |
| `ai.permissions.get` | Lists what this AI may do now (level, hidden notebooks, exchange folder name) | Look | – | V | New |
| `ai.sessions.list` | Which AIs are connected now and what each has claimed | Look | – | V | New |
| `ai.claim` / `ai.release` | Claims or releases items (pages, databases, boards, a notebook) for a job, so the other AI does not edit them | Edit | – | V | New |
| `ai.limits.get` | Lists sizes and limits (import sizes, batch size, timeouts) | Look | – | V | New |
| `ui.where` | Which page, board or tab the person has open | Look | – | V | New |
| `ui.open` | Opens a page, row, database view, board, dashboard or Settings page for the person to see | Edit | – | V | New |
| `ui.highlight` | Briefly highlights a block, row or object so the person sees what the AI means | Edit | – | V | New |
| `ui.notify` | Shows a short message in the Notaya window | Edit | – | V | Part (app notices) |
| `ui.wait_for_person` | Asks the person a question in the window (choices or short text) and waits for the answer | Edit | – | V | New (addition) |
| `batch.run` | Runs several actions as one change set, all or nothing (addition 2.10) | Edit | as its strongest step | P | New |

---

## 6. Notes: notebooks, sections, pages, page content

### 6.1 Tree and notebooks (incl. Default style and Notebook mode)

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `tree.list` | The whole sidebar: notebooks, sections, pages, sub-pages, databases, boards, with ids | Look | – | V | Exists (`list_notes`) |
| `notebook.list` / `notebook.get` | Names, icons, colours, style, counts | Look | – | V | Exists |
| `notebook.create` | New notebook with name, icon, colour and style (Default style or Notebook mode) | Edit | – | B | Exists (style New) |
| `notebook.update` | Rename, change icon or colour | Edit | – | P | Exists |
| `notebook.set_style` | Switch a notebook between **Default style** and **Notebook mode** (layout only; content kept) | Edit | Ask* | P | New |
| `notebook.reorder` | Move a notebook up or down in the list | Edit | – | P | Exists (`reorder`) |
| `notebook.duplicate` | Copies a whole notebook | Edit | Ask* | B | New |
| `notebook.trash` | Notebook to the recycle bin (a safety backup is made first, as today) | Edit | Ask | B | Part (today it deletes after a backup; a notebook bin is new) |

### 6.2 Sections (incl. Notebook mode's coloured tabs)

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `section.list` / `section.get` | Sections of a notebook, in order, with colour | Look | – | V | Exists |
| `section.create` | New section | Edit | – | B | Exists |
| `section.update` | Rename; set its tab colour (Notebook mode) | Edit | – | P | Exists (colour New) |
| `section.reorder` | Order inside the notebook (drag) | Edit | – | P | Exists |
| `section.move` | Move a section to another notebook | Edit | – | P | New |
| `section.duplicate` | Copy a section with its pages | Edit | Ask* | B | New |
| `section.trash` | Section to the recycle bin | Edit | Ask | B | Part (delete exists) |

### 6.3 Pages and sub-pages

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `page.get` | A page's title, look, properties and content, as **Markdown** (default) or as Notaya's own block format | Look | – | V | Exists (Markdown via the exporter; inline databases must be fixed first, section 9) |
| `page.outline` | Headings and block ids only, for long pages | Look | – | V | New |
| `page.recent` / `page.favorites` | Recent and favourite pages | Look | – | V | Exists |
| `page.backlinks` | Pages that link to or mention this page ("linked from") | Look | – | V | Gap |
| `page.create` | New page in a section, from Markdown or empty, optionally from a template | Edit | – | B | Exists |
| `page.create_sub` | New sub-page inside a page | Edit | – | B | Exists |
| `page.write` | Replaces a page's whole content from Markdown | Edit | Ask* | H | Exists (save) |
| `page.append` / `page.insert` | Adds content at the end, or before/after a block | Edit | – | H | New (finer save) |
| `page.replace_block` / `page.delete_blocks` | Changes or removes specific blocks | Edit | – | H | New (finer save) |
| `page.move_blocks` | Moves blocks within a page or to another page | Edit | – | H | New |
| `page.rename` | Changes the title | Edit | – | H | Exists (save) |
| `page.set_look` | Icon (emoji, own icon set, picture in the vault), cover, banner height override, full width for this page | Edit | – | P | Exists (icon, cover); New (rest) |
| `page.favorite` | Adds or removes from Favorites | Edit | – | P | Exists |
| `page.move` | Moves a page to another section, notebook, or **into another page** (re-parent) | Edit | – | P | Part (sections only today) |
| `page.reorder` | Order in the sidebar | Edit | – | P | Exists |
| `page.duplicate` | Copies a page (with or without sub-pages) here or elsewhere | Edit | – | B | New (agreed via right-click) |
| `page.copy_link` | Returns the Notaya link to a page or heading, for use in other pages | Look | – | V | Part |
| `page.set_mode` | Switches one page between normal and **free canvas** | Edit | Ask* | H | New |
| `page.trash` | Page to the recycle bin | Edit | Ask | B | Exists |
| `page.template.list` / `.apply` | Page templates outside databases | Look / Edit | – | H | Gap |
| `page.template.save` / `.trash` | Saves a page as a template; removes one | Edit | – / Ask | B | Gap |
| `page.lock` / `page.unlock` | Reserved; per-page locks are deferred by Ahmed (23:53:46 UTC), no AI action for now | – | – | – | Deferred |

### 6.4 Blocks inside a page

All of these are edits to one page (Level Edit, no confirm, undo through version history). Writing Markdown covers most of them; the named actions exist for things Markdown cannot say.

| Action | What it does | Today |
|---|---|---|
| `block.list` | The blocks of a page with ids and kinds | New |
| `block.insert` | Inserts any block kind: text, headings 1-4, lists, checklist, quote, divider, table, toggle, shelf, equation, picture, sub-page, link to page, inline database or linked view | Exists (via save) |
| `block.update` / `block.turn_into` | Changes a block's text or turns it into another kind ("Turn into") | Exists (UI only today) |
| `block.set_direction` | Right-to-left or left-to-right for a block | Exists |
| `block.style_text` | Bold, italic, colour, highlight, font, size, link on a range | Exists |
| `code.set_language` | Sets a code block's language (the agreed picker) | New |
| `code.set_options` | Wrap on or off, collapsed or open | New |
| `diagram.render` | Turns a code block with Mermaid text into a drawn diagram (agreed merge of Diagram and code block) | Part |
| `diagram.export` | Saves the drawn diagram as a picture into the exchange folder (**Full**, Ask) | New |
| `table.edit` | Add or remove rows and columns, header row, merge or split cells | Exists (UI) |
| `picture.add` | Adds a picture from the exchange folder (**Full**) or one already in the vault | Exists (`add_attachment`, path variant New) |
| `picture.set_layout` | Free position, wrap, align, width, crop, caption | Exists |
| `columns.make` / `columns.set` | **Side-by-side columns**: make, change widths, move blocks between columns | Gap (dormant today; P1 in the parity study) |
| `callout.set` | **Callout box colour and its icon** (emoji or own icon set) | Gap (P1) |
| `heading.set_folding` | **Folding headings** (a heading that folds what is under it); also open or close a toggle | Gap (P1) |
| `mention.insert` | **@-mention** of a page, database, row or date (a date chip, not plain text) | Gap (P1) |
| `link.to_view` | **A link that opens a saved view of a database** | Gap (P1) |
| `file.insert` | **File, audio and video blocks**, and a web bookmark card kept as a link (no fetching) | Gap (P2) |
| `button.insert` | A button block that runs a database action list | Gap (P3) |
| `toc.insert` / `synced.insert` | Table of contents block; synced block | Gap (P3, ask) |

### 6.5 Free canvas pages and Notebook mode

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `canvas.get` | All text boxes, pictures, ink and shapes on a canvas page, with positions | Look | – | V | New |
| `canvas.add_box` | A text box (Markdown inside) at a place | Edit | – | H | New |
| `canvas.move` / `canvas.resize` | Moves or resizes boxes and objects | Edit | – | H | New |
| `canvas.arrange` | Tidies boxes (align, space evenly, stack) | Edit | Ask* | H | New |
| `canvas.insert_space` | The "insert space" tool: pushes things down or right | Edit | – | H | New |
| `canvas.delete_objects` | Removes boxes or objects from the canvas | Edit | – | H | New |
| `canvas.ink.read` | A list of ink strokes (for counting or erasing), never freehand drawing by the AI | Look | – | V | New |
| `canvas.ink.erase` | Erases chosen strokes | Edit | Ask* | H | New |
| `canvas.snapshot` | A picture of the canvas for the AI to look at (deliberate; it can contain private content) | Look | Ask (first time per session) | V | New |

Canvas pages share the whiteboard engine, so the shape, ruler and template actions of section 7.3 also work on a canvas page.

---

## 7. Databases, dashboards, sticky notes, whiteboard

### 7.1 Databases

`save_database` today replaces the whole database object; an AI must use the finer actions below (section 9).

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `db.list` / `db.get` | Databases with properties, views, templates, automations | Look | – | V | Exists |
| `db.query` | Rows of a view, or with the AI's own filter, sort and group; paged | Look | – | V | Exists (`query_view`, `search_rows`) |
| `db.row.get` | One row with all its values and page content | Look | – | V | Exists |
| `db.row_titles` | Titles only, for relations and pickers | Look | – | V | Exists |
| `db.chart.data` | The numbers behind a chart view | Look | – | V | Exists |
| `db.create` | New database (full page or inline in a page), blank, from a template, or from a CSV in the exchange folder (**Full**) | Edit | – | B | Exists |
| `db.rename` / `db.set_look` / `db.set_description` | Name, icon (emoji or own icon set), and a **description** | Edit | – | P | Exists (description Gap) |
| `db.lock` | Set aside for now (Ahmed deferred locking); the person locks and unlocks | – | – | – | Exists (person only) |
| `db.move` | Moves to another section, notebook or **into a page** | Edit | – | P | Part (sections only) |
| `db.reorder` | Drag order among pages in the sidebar (agreed "move databases") | Edit | – | P | New |
| `db.duplicate` | Copies a database (structure only, or with rows) | Edit | Ask* | B | New |
| `db.trash` | Database to the recycle bin | Edit | Ask | B | Exists |
| `db.property.add` | Adds a property of any of the 23 kinds | Edit | – | P | Part (via whole save) |
| `db.property.update` | Rename, options, number format, **description**, formula text, relation, rollup | Edit | – | P | Part (description Gap) |
| `db.property.change_kind` | Changes a property's kind (can lose data) | Edit | Ask | P | Part |
| `db.property.trash` | Removes a property (values kept in the log copy for undo) | Edit | Ask | P | Part |
| `db.property.reorder` / `.hide` | Order and visibility in a view | Edit | – | P | Part |
| `db.formula.test` | Runs a formula on sample rows and returns results or the error, without saving | Look | – | V | New (engine exists) |
| `db.view.add` / `.update` | Adds or changes a view: table, board, calendar, timeline, list, gallery, chart, with filters, sorts, groups, sub-groups, footers, card covers | Edit | – | P | Part |
| `db.view.quick_filters` | The quick filter chips on a view | Edit | – | P | Gap (check) |
| `db.view.reorder` / `.duplicate` / `.rename` | View tabs | Edit | – | P | Part |
| `db.view.trash` | Removes a view | Edit | Ask* | P | Part |
| `db.row.create` | New row with values, optionally from a row template | Edit | – | B | Exists |
| `db.row.set` | Sets cells of one or many rows | Edit | Ask* | P | Exists (`set_cells`) |
| `db.row.write_page` | Writes the page text of a row (Markdown) | Edit | Ask* | H | Exists (save) |
| `db.row.duplicate` / `.reorder` | Copies rows; manual order | Edit | – | B / P | Exists |
| `db.row.trash` | Rows to the recycle bin (needs a list of trashed rows, section 9) | Edit | Ask | B | Part |
| `db.row.layout` | **Row-page layout**: which properties show, in panels, pinned, hidden when empty | Edit | – | P | Gap (P2) |
| `db.template.list` / `.create` / `.update` / `.trash` | Row templates; default template per view; **repeating templates** | Look / Edit | trash Ask | P / B | Exists (default and repeating Gap) |
| `db.sub_items.enable` / `db.dependencies.enable` | Turns on sub-items or dependencies | Edit | – | P | Exists |
| `db.automation.list` / `.create` / `.update` | Automations (when, if, then), inside today's limits | Look / Edit | Ask* | P | Exists |
| `db.automation.pause` / `.trash` | Pauses or removes one | Edit | – / Ask | P | Part |
| `db.button.press` | Presses a Button property on a row (runs its action list) | Edit | as its actions | P | Exists |
| `db.reminders.due` | Reminders due now | Look | – | V | Exists |
| `db.file.list` | Files attached in a Files property | Look | – | V | Exists |
| `db.file.attach` | Attaches a file from the exchange folder | Full | – | P | Part (path variant New) |
| `db.file.save_to_exchange` | Saves an attached file into the exchange folder | Full | Ask | – | Part |
| `db.file.remove` | Removes an attached file | Edit | Ask | P | Exists |

### 7.2 Dashboards

From the dashboards study (section 5), adjusted to the levels here. A dashboard only reads data; no tile can change a row.

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `dashboard.list` / `dashboard.get` | Names, pages, tiles, settings | Look | – | V | New |
| `dashboard.read` | The numbers the person sees, with the current or given slicer choices | Look | – | V | New |
| `dashboard.explain_tile` | The rows behind a number and the measure in plain words | Look | – | V | New |
| `dashboard.check` | Data-health report (empty values, broken relations, tiles pointing at deleted fields) | Look | – | V | New |
| `dashboard.create` / `.duplicate` / `.rename` | Makes, copies, renames | Edit | – | B / P | New |
| `dashboard.template.apply` | Builds from a starter (for example a personal-finance starter) | Edit | – | B | New |
| `dashboard.page.add` / `.rename` / `.reorder` | Dashboard pages | Edit | – | P | New |
| `dashboard.tile.add` / `.update` / `.move` / `.resize` | Tiles: kind, source database, measure, grouping, filters, cross-filter on or off | Edit | – | P | New |
| `dashboard.arrange` | Tidies the layout | Edit | Ask* | P | New |
| `dashboard.slicer.add` / `.update` | Slicers (click-to-filter) | Edit | – | P | New |
| `dashboard.slicer.set` / `.reset` | Chooses slicer values for a read (view only) | Look | – | V | New |
| `dashboard.measure.list` / `.get` / `.test` | Named calculations; test runs without saving | Look | – | V | New |
| `dashboard.measure.create` / `.update` | Saves a named calculation | Edit | – | P | New |
| `dashboard.snapshot.save` | Frozen numbers with a date | Edit | – | B | New |
| `dashboard.export` | PDF, PNG, or a tile's data as CSV or Excel into the exchange folder | Full | Ask | – | New |
| `dashboard.page.trash` / `.tile.trash` / `.measure.trash` / `dashboard.trash` | Removes them (a measure in use is refused unless a replacement is named) | Edit | Ask | B | New |

### 7.3 Sticky notes and the corkboard

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `sticky.list` / `sticky.get` | All notes with text, colour, board place, pins, strings | Look | – | V | Exists (pins, strings New) |
| `sticky.create` | New note with text and colour, on the board and optionally on the desktop | Edit | – | B | Exists |
| `sticky.write` | Changes the text (bold, italic, checklist) | Edit | – | P | Exists |
| `sticky.style` | Colour; keep on top | Edit | – | P | Exists |
| `sticky.open` / `.close` / `.close_all` / `.show_all` / `.hide_all` | Desktop windows | Edit | – | V | Exists (show/hide all New) |
| `sticky.place_on_desktop` | Moves or resizes a desktop note window | Edit | – | P | Exists |
| `board.mode` | Corkboard in **grid** or **free** mode | Edit | – | P | New |
| `board.place` / `board.arrange` | Positions on the corkboard; tidy (grid, by colour, by date) | Edit | Ask* | P | New |
| `board.pin` | A coloured pin on a note | Edit | – | P | New |
| `board.string.add` / `.style` / `.remove` | Coloured strings between notes | Edit | – | P | New |
| `sticky.trash` | Notes to the recycle bin (one or many) | Edit | Ask | B | Part (today deletes after asking; a bin for notes is new) |
| `sticky.to_page` / `page.to_sticky` | Turns a note into a page or a short page into a note (copy) | Edit | – | B | New (addition) |

### 7.4 Whiteboard

From the whiteboard study (section 5). The AI works with shapes, text and notes, never freehand drawing.

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `wb.list` / `wb.get` | Boards; a summary or the full object list | Look | – | V | New |
| `wb.read_text` / `wb.find` / `wb.describe` | All text in reading order; find; the layout in words | Look | – | V | New |
| `wb.snapshot` | A picture of the board for the AI (deliberate, like `canvas.snapshot`) | Look | Ask (first time per session) | V | New |
| `wb.create` | New board, blank or from a template | Edit | – | B | New |
| `wb.rename` / `.set_background` / `.duplicate` / `.move` | Board settings | Edit | – | P / B | New |
| `wb.add_shape` / `.add_text` / `.add_note` / `.add_note_grid` / `.add_image` / `.add_line` / `.add_frame` | Objects (lines include arrows and connectors) | Edit | – | H | New |
| `wb.move` / `.resize` / `.style` / `.group` / `.align` / `.lock_object` | Changing objects | Edit | – | H | New |
| `wb.delete_objects` | Removes objects | Edit | Ask* | H | New |
| `wb.ruler.place` | Places the ruler at a degree | Edit | – | V | New |
| `wb.template.list` / `.preview` / `.apply` / `.save_from_selection` | Templates (weekly planner, Kanban, mind map, SWOT, lined page first) | Look / Edit | – | H / B | New |
| `wb.template.trash` | Removes an own template | Edit | Ask | B | New |
| `wb.pen.list` / `.save` | Favourite pens | Look / Edit | – | P | New |
| `wb.timer.start` / `.pause` / `.stop` | The timer | Edit | – | V | New |
| `wb.link_page` / `wb.note_link` | Links a board object to a page; a board note **linked to** a real sticky note, or a separate copy (both agreed) | Edit | – | H | New |
| `wb.export` | PNG or PDF into the exchange folder | Full | Ask | – | New |
| `wb.trash` | Board to the recycle bin | Edit | Ask | B | New |

---

## 8. The app around the notes

### 8.1 Banner, icons and look

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `look.get` | Theme, accent, font, size, width, density, window material, banner and icon settings | Look | – | V | Exists |
| `look.set_theme` / `.set_appearance` | Theme and appearance | Full | – | P | Exists |
| `banner.get` | Home banner per time of day, height, icon size and position | Look | – | V | New |
| `banner.set_period` | A picture (or a folder, shuffled) for a time-of-day period, from the vault or the exchange folder | Full | – | P | New |
| `banner.set_layout` | Banner height, icon size, icon position (left, centre, overlap) | Full | – | P | New |
| `banner.reset` | Back to the built-in banners | Full | Ask | P | New |
| `banner.reposition` | Frames a page's or the Home banner picture | Edit | – | P | Exists (page covers) |
| `icons.list` | Notaya's own icon set (proposed in the parity study) | Look | – | V | Gap |

### 8.2 App bar (quick buttons)

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `appbar.get` | Buttons in order, icon or icon + label | Look | – | V | New |
| `appbar.add` / `.remove` / `.reorder` | Quick buttons (Import, Export, New page, New sticky note, and the optional ones) | Full | – | P | New |
| `appbar.pin_page` | Pins a page to the bar | Full | – | P | New |
| `appbar.set_style` | Icon only or icon + label | Full | – | P | New |
| `appbar.arrange` / `appbar.reset` | AI arranges the bar (agreed); reset to defaults | Full | Ask | P | New |

### 8.3 Tabs

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `tabs.list` | Open tabs, pinned, groups, order | Look | – | V | New |
| `tabs.open` / `.close` / `.close_others` / `.reopen_closed` | Tabs (closing never deletes anything) | Edit | – | V | New |
| `tabs.pin` / `.move` / `.group` / `.ungroup` / `.rename_group` | Arranging tabs | Edit | – | P | New |

### 8.4 Right-click menus

Every agreed menu item already maps to an action in this list (for example sidebar Rename = `page.rename`, Duplicate = `page.duplicate`, Move = `page.move`, tab Reopen = `tabs.reopen_closed`, link Copy link = `page.copy_link`). Picture "Copy image" and "Save image" stay with the person (clipboard and files). No "Ask AI about this" item.

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `menus.get` | Each menu with its items, shown or hidden | Look | – | V | New |
| `menus.set_item` | Hides or shows an item in a menu | Full | – | P | New |
| `menus.reset` | Back to defaults (one menu or all) | Full | Ask | P | New |

### 8.5 Shortcuts

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `shortcuts.list` | Every action with its keys, global or in-app | Look | – | V | New (fixed today) |
| `shortcuts.check` | Checks a key for clashes before setting it | Look | – | V | New |
| `shortcuts.set` | Rebinds an action (refused if it clashes, unless told to swap) | Full | Ask* | P | New |
| `shortcuts.reset` | One or all to default | Full | Ask | P | New |

Global shortcuts can never be bound to anything that turns the AI on or unlocks the vault.

### 8.6 Calculator and unit converters

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `calc.evaluate` | Works out an expression with Notaya's own calculator engine (no code run) | Look | – | V | Exists (engine) |
| `calc.convert` | Offline unit conversion (length, volume, weight, temperature, energy, area, speed, time, power, data, pressure, angle; no currency) | Look | – | V | New |
| `calc.history` | The history list | Look | – | V | Exists |
| `calc.add` | Adds a line to the history | Edit | – | P | Exists |
| `calc.open` | Opens the calculator window for the person | Edit | – | V | Exists |
| `calc.clear` | Clears the history | Edit | Ask | P | Exists |

### 8.7 Search and find

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `search.all` | Searches pages, databases, rows, notebooks, sticky notes (and boards, dashboards later) with snippets | Look | – | V | Exists |
| `search.history` | The person's recent searches | Look | – | V | Exists |
| `find.in_page` | Finds text in one page incl. folded parts; returns "3 of 12" places | Look | – | V | New |
| `find.replace` | Find and replace in a page (Ctrl+H) | Edit | Ask* | H | New |
| `find.replace_all_notes` | Replace across many pages | Edit | Ask | H | New (addition) |

### 8.8 Import and export (the exchange folder)

The only way files move in or out. Ahmed (23:53:46 UTC, not yet sure): it should be "inside of the app itself somehow, when it's installed and when it's portable, something that's containable". Proposal that fits: an **`exchange` folder that Notaya creates beside its `data` folder**, so it travels with Notaya: in the portable copy next to `data` in the Notaya folder; in the installed copy in `%LOCALAPPDATA%\Notaya Notes\exchange`. It is **beside** `data`, not inside it, because the vault folder must hold only encrypted files (the exporter already refuses to write into `data`). Settings > AI access shows it with an "Open folder" button for the person and an "Empty it" button. The AI can list, read and write only there. Files in it are **not encrypted**, so exports are logged and need Ask, and Notaya reminds the person to empty it. Not picked by the person, so there is nothing to misconfigure; still to confirm with Ahmed.

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `exchange.list` | Files in the exchange folder | Full | – | V | New |
| `import.preview` | What an import would bring in and what would be left out, writing nothing | Full | – | V | Exists (`plan::build`) |
| `import.run` | Imports Markdown, HTML, Word, CSV, PDF, ZIP or another app's export into a new notebook **or into an existing place** | Full | Ask | B | Part (into existing place New) |
| `import.refresh` | Re-imports an updated export **without duplicates**, matching by original id; keeps both versions where the person edited | Full | Ask | H / B | Gap (P1 for the Notion backup) |
| `import.report` | "What did not carry over" and a counts check against the export | Full | – | V | Part (addition from the parity study) |
| `import.cancel` | Stops a running import | Full | – | V | Exists |
| `import.find_by_source_id` | Opens an item by its original id in the other app | Look | – | V | Gap (ids must be stored) |
| `export.run` | Exports a page, pages, section, sections, notebook(s), with sub-pages, as HTML, Markdown, PDF (contents page, bookmarks) or Word, one file or ZIP | Full | Ask | – | Exists (contents page New) |
| `export.database` | A database or view as CSV, Markdown, HTML, Word, PDF or **Excel** | Full | Ask | – | Exists (Excel New) |
| `export.history` | Past imports and exports (from the activity log) | Look | – | V | New |

`import.run` takes a preset for exports from other apps (for example the source app's "HTML + CSV" export). The preset is a format name, not a screen name.

### 8.9 Version history, activity and the recycle bin

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `history.list` | A page's (or board's) versions with dates and who made them | Look | – | V | New |
| `history.get` / `history.compare` | An old version; what was added or removed | Look | – | V | New |
| `history.restore` | Restores an old version (the current one becomes a version too) | Edit | Ask | H | New |
| `activity.list` | The activity log: who (person, Claude, Codex), what, when, counts | Look | – | V | New |
| `activity.undo` | Undoes one of **the AI's own** change sets (never the person's) | Edit | Ask* | P | New |
| `activity.verify` | Checks the log's chain is unbroken | Look | – | V | New |
| `bin.list` | Everything in the recycle bin (pages, databases, rows, notes, boards, dashboards, notebooks) | Look | – | V | Part (pages, databases) |
| `bin.restore` | Brings items back | Edit | – | P | Exists (pages) |
| `bin.delete_forever` | Deletes chosen items for good | Full | Strong | – | Exists |
| `bin.empty` | Empties the bin | Full | Strong | – | Exists |

### 8.10 Backups, settings, guide, tour, about

| Action | What it does | Level | Confirm | Undo | Today |
|---|---|---|---|---|---|
| `backup.list` | Backups with dates and sizes | Look | – | V | Exists |
| `backup.now` | Makes a backup now (good before a big AI job) | Edit | – | V | Exists |
| `settings.get` | Every non-security setting | Look | – | V | Exists |
| `settings.set` | Changes a non-security setting (new sticky colour, page width, tray options, density) | Full | – | P | Exists |
| `guide.list` / `guide.get` | The in-app manual: feature list and explanations | Look | – | V | New |
| `guide.open` | Opens a manual page for the person | Edit | – | V | New |
| `tour.start` | Replays the tour for the person | Edit | – | V | New |
| `about.get` | Version, credits, open-source references | Look | – | V | Part |
| `tray.*` | The tray menu stays the person's; the AI reaches the same things through the actions above | – | – | – | – |

---

## 9. Never for the AI

| Thing | Why | Today's command(s) |
|---|---|---|
| Unlock, lock-screen password, change password, keyfiles | Keys and passwords stay with the person | `unlock_vault`, `change_master`, `pick_keyfile`, `create_keyfile`, `password_strength` |
| Security on or off, auto-lock, screenshot protection | Security behaviour is the person's | `enable_security`, `disable_security`, security fields of `save_settings` |
| Restore a backup, Start over, import a portable copy | Replaces the whole vault | `restore_vault`, `start_over`, `pick_backup_file`, `import_portable_notes` |
| **The AI switch, its level, pairings, exchange folder, hidden notebooks, clearing the log** | The AI cannot change its own permissions | new Settings > AI access |
| Start with Windows | Writes to Windows (registry) | `set_shell_prefs` start-up field |
| Open a web link, a file in another program, Notaya's data or log folder | Sealed in: no programs, no internet, no Windows folders | `open_url`, `open_file`, `open_data_folder`, `open_log_folder` |
| Clipboard (read or write) | Sealed in; the calculator's Copy stays a person's click | – |
| Quit Notaya | Would cut the person's work | `quit_app` |
| Lock the vault, lock or unlock a database or page | Deferred by Ahmed (23:53:46 UTC); revisit when locking is discussed | `lock_vault`, `set_database_look` lock field |

---

## 10. What the Rust core needs first (prerequisites, no code yet)

1. **The door:** the bridge, named pipe, pairing and Settings > AI access (section 3). A new trust boundary: a security-behaviour change that needs Ahmed's own words and new promises in `docs/security-design.md` (proposed: P30 sealed AI channel, P31 AI off by default and no self-permission, P32 confirmations only in the window, P33 tamper-evident activity log).
2. **Small saves:** add, change, move or remove one block, row, property, view, note or board object, with a version number per item, instead of today's whole-page and whole-database saves (`save_page`, `save_database`).
3. **"Something changed" signal** so the open window refreshes, and a check so a pending window save cannot overwrite an AI change (the inventory found `save_database` debounced 700 ms sends the whole object).
4. **Move UI-only logic into Rust:** inline database insertion, applying row templates, which page is open, tabs, app bar and menu settings.
5. **Fix the Markdown round trip** so `page.get` does not drop inline databases (the known export bug), and so columns, callouts, folding headings and mentions survive Markdown both ways (Notaya-specific Markdown extensions).
6. **A recycle bin for everything:** notebooks, sections, rows, sticky notes, boards and dashboards (today only pages and databases).
7. **Version history storage** per page and board, which also backs AI undo.
8. **The confirmation queue and the activity log** (encrypted, chained).
9. **Store original ids on imported items** (needed for `import.refresh` and `import.find_by_source_id`).

---

## 11. Suggested staging for the planning phase (a proposal, versions not assigned)

- **Step A, the foundation:** sections 10.1 to 10.3 and 10.8, levels and confirmations, `ai.*`, `ui.open`, `ui.notify`, and **Look** actions for everything that exists today (tree, pages as Markdown, databases, search, sticky notes, calculator, settings). Low risk, high value: the AI can read and explain all of Notaya.
- **Step B, editing what exists:** **Edit** actions for notebooks, sections, pages, blocks, databases, sticky notes, with small saves (10.2), the bins (10.6), `activity.undo` and version history (10.7).
- **Step C, files:** the exchange folder, `import.*` and `export.*` (incl. into existing place and re-import without duplicates), `backup.now`.
- **From then on:** each new feature (dashboards, whiteboard, corkboard, tabs, app bar, menus, shortcuts, banner, canvas and Notebook mode, the parity gaps) ships **with** its actions from this list, as agreed (Ahmed, 20:38 UTC: every feature gets named AI actions from its first design).

---

## 12. Ahmed's answers (2026-11-07 23:53:46 UTC, project chat, relayed by the coordinator at 23:53:58)

| # | Question | Answer | Status |
|---|---|---|---|
| 1 | Four levels (Off, Look, Edit, Full)? | "I agree with the 4 levels"; Full must give full authority, including deleting whole pages or hubs ("that's what levels are about") | ✅ Agreed |
| 2 | Hidden-from-AI notebooks? | "the idea of hide notebooks from AI. That's good." | ✅ Agreed |
| 3 | Confirm place: Notaya window only? | Not answered directly; no objection | 🟡 Proposal stands (window only) |
| 4 | Big-change threshold | Optional, "not something that's all like have to be forced"; "20 I guess is good enough" | ✅ Agreed: a setting, default 20 |
| 5 | Exchange folder | "inside of the app itself somehow, when it's installed and when it's portable ... containable. I'm not sure." | 🟡 Leaning: a folder beside `data` (section 8.8); to confirm |
| 6 | Per-page lock | "I don't want to answer anything about locking pages right now" | ⏸ Deferred; no AI lock or unlock action for now |
| 7 | Lock now | "I'm not sure" | ⏸ Deferred; not an AI action for now |
| 8 | Snapshots of boards and canvas pages | "pictures, I agree" | ✅ Agreed (once per session, after a confirm) |
| 9 | Claude and Codex at once | Fine "as long as Claude and Codex do not collide ... make sure that you are able to coordinate between them" | ✅ Agreed, with claims and version checks (section 3, item 6) |
| 10 | The bridge (MCP over a local pipe) | "I'm not sure also" (read as question 10; he said "20th") | 🟡 Open: recommendation stands, to explain in plain words in planning |

Still open for planning: 3 (confirm), 5 (exchange folder), 10 (bridge); 6 and 7 deferred until locking is discussed.

---

## 13. What I did not check

- I ran nothing: no app, no tests. "Exists" comes from the inventory (baseline `9474511`) and the command list on `9e3e4e3`.
- I did not re-read `docs/security-design.md` in full; I used the security digest (2026-11-07) and the inventory's section 6.
- I did not design parameters, error codes or the exact Markdown extensions; those are planning work.
- Action counts are approximate (grouped names like `.rename` / `.reorder` count separately).
