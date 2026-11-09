# Whole-app test plan — lane W, phase 1

**Derived from the repository, not from recollection or a live-app audit.** The backbone is the full [feature inventory](2026-11-07-notaya-full-feature-inventory.md), especially sections 3.1–3.13, cross-checked against the source and records listed below. This is a plan: **every check is NOT RUN**. No feature is declared working, verified or accepted by this document.

- **Who and where:** Ahmed Emara's side (`AE0101`); Codex developer hand, Claude coordinator; machine `GAMING-PC`, 2026-11-09 (UTC system clock).
- **PBS baseline:** branch `AE0101/v1-6-w`, cut from `805cf77`; inspected HEAD `8fa56c386cc96e105a9012beb69600e77a5f7100` (automatic WIP checkpoint). The inventory was already staged when inspected; preserve it. The lane W claims row already existed in that checkpoint; update that one row, never duplicate it.
- **Authority:** Ahmed's current phase-1 message (message time not supplied), and A47, Ahmed 2026-11-09 **02:06:59Z**, [spec §10.3](2026-11-08-v1-6-spec.md#103-release-gate-the-whole-app-a47-ahmed-2026-11-09-020659-utc), [lane W brief](2026-11-08-v1-6-codex-briefs.md) final addendum. Current instruction narrows this run to writing the plan and its claims row. No code, fixtures, test execution, browser, exe, network, installs or Git writes.
- **Return and stop point:** numbered feature checks, existing automation and applicability, gaps and phase-3 measurements; hand back to Claude. Claude owns commits, pushes, PR and the session/execution/usage records outside this write area. Model/effort and token/quota receipts are not observable here; coordinator must use the actual run receipt, not inferred numbers.
- **Gate:** phase 2 executes against the exact integrated C5+E commit and matching binaries. Record passed, failed or not run for every ID; leave blockers visible. Phase 3 uses bounded defect lanes with regressions and performance measurements; preserve the look and names. A47 stays open until spec §10.2 and WP-15 also pass and failures are fixed or listed for Ahmed in an Execution Summary. A plan, unit suite, merge or release is not partner acceptance.

## Sources, code additions and what could not be confirmed

Read the inventory in full, `AGENTS.md`, `CLAUDE.md`, the partner briefing, collaboration/claims and relevant handoffs; use `README.md`, `docs/STATUS.md`, the A47 authority entry, `docs/acceptance.md`, `docs/security-design.md`, the execution index and import/export, optional-security and 1.6 execution records as evidence pointers. Code cross-checks: `src/components/{TitleBar,Sidebar,CommandPalette,Shell}.tsx`; `src/store/app.ts`; `src/features/{notes,database,settings,sticky,unlock,search}` menus/dialogs; editor extensions, ribbon and slash menu; `src-tauri/src/lib.rs` command registration, `commands/`, `model.rs`, `database/types.rs`, `importers/`, `export/`, `vault/{history,activity}.rs`, `actions/`, `changes.rs`; test scripts and test-module declarations. This was selective source inspection, not a line-by-line audit of every implementation.

Corrections/additions to the **1.5.1-era inventory**, without editing that historical file:

- Seven actual database views: **table, board, calendar, timeline, list, gallery, chart** (`ViewKind` and seven view components). Chart has **bar, line, donut, number**; Mermaid's pie diagram is a different feature. `PropKind` has 23 kinds, all checked individually below.
- Lane A has top-bar Import/Export, always-visible New notebook, last-place restore (including database view/row), sidebar database dragging and named credits. Lane B handles inline database export. C2 adds notebook/section/sticky/row bin behavior, revisions and edit leases; C4 adds block IDs and fine-grained saves; C3 adds history and activity. Test those changes rather than repeating old “missing” assertions.
- File **open/save-as/remove** for database attachments, first-database safety backup, **default and daily/weekly/monthly recurring row templates** (`TemplatesDialog.tsx`, `commands/database.rs::apply_repeats`), ordinary/hourly/predelete backup pools, startup failures, migration/rollback and safe appearance caches require explicit checks beyond a superficial screen walk. The template behaviors are additions to the inventory's terse template entry.
- Importers are Markdown/text, HTML, DOCX, PDF plus source/ZIP/planning/conversion helpers and Notion database conversion. There is **no dedicated Obsidian, Evernote `.enex`, RTF or OneNote importer** in this tree. Obsidian ordinary Markdown is tested as Markdown/folder input; wiki links/embeds are not promised.
- Six explicit themes plus **Match Windows**, a resolver option (README calls this seven themes). Test each explicit theme and system mode separately.
- C5 server and E screens are **absent from W HEAD**. Offline, separately inspected cached local refs: C5 `AE0101/v1-6-c5` at `3929dab77d1be6ff199a801fccd7dfedde17a062` supplies `tools/ai-smoke`; E `AE0101/v1-6-e` at `706cd147561e7b61de78d23906939d6e39c9ce42` supplies `ai-mock-check.mjs`/`ai-ui-check.cjs`. These are separate source observations, not integrated W coverage. Re-read their final versions after merge.

**Unconfirmed:** runtime, native dialogs, rendering/assistive technology, current executable/build hashes, actual test discovery/pass counts, live PR/remote overlap (no fetch/network), a usable synthetic old `.clf`/1.5.1 exe/Notion Hubs fixture, and final C5/E integration/security review. Password testA/B/C/E sources are absent from the tracked tools/tree; historical reports cannot establish runnable commands. Obsidian-specific syntax fidelity is unconfirmed. PDF/Word direction/layout and format losses need actual exported-file inspection. No performance baseline or accepted timing threshold exists in this plan.

## Execution contract and throwaway data

Each row is one numbered feature check. **A** = browser/mock script, including browser-free dev-mock contracts; proves UI/store/mock behavior only. **B** = needs real `Notaya.exe`, driven by **Claude on the PC**, sometimes through scripts or real dialogs; Rust unit results support its oracle but cannot alone pass it. **C** = a person must look or perform a physical/session action; can use the real throwaway exe, and is not satisfied by mock assertions. A row's KIND is its required execution route, not a claim an automated script already exists. A runtime fault/time harness that does not exist yet is a gap, not permission to substitute a unit-only pass. “Gaps” distinguishes proposed checks from existing automation.

For phase 2, create a fresh `D:\Notaya Tests\v1-6-W\run-<UTC>-<id>`; set **both** `NOTAYA_HOME` and `CIPHERLEAF_HOME` to that run's isolated home for each child. Outputs stay under that run or `D:\Notaya-lanes\out\W`. Never `.devdata`, a real vault, anyone's personal notes, or Mohamed's `E:` drive. Check single-instance/legacy guards before launching; do not hand off to or close a real app. Preserve fixtures, failed output and before/after copies. Native scripts that clean up their own folders need a reviewed evidence-preservation adaptation before use. No Windows/AV settings changes or installs are implied. WP-15 sign-in/setup uses an approved clean user/VM and must not overwrite Ahmed's installation or startup entry.

Before any debug-port test check the selected port is free, scope WebView2 arguments to that child, then close only test processes, restore environment, and confirm no listener remains (FI-01). Prefer the AI Live runner, which uses **no debug endpoint**. Never use IPC injection to approve an AI request. Restore negative/corruption cases use copies. Large-limit and fault-injection fixtures are generated later, never real data. Every persistence B check includes quit/reopen of the **same test home**, followed by readback; do not accept the optimistic UI alone.

Shared fixture recipes (create later, not in phase 1). **Start each ID and independent variant from a fresh copy of its recipe**, unless the steps explicitly name a preceding state. In particular, the formula edit in W076 must not change the Amount=60 oracle for later checks. Preserve each run's before/after evidence; do not reset/reuse the only copy of a failed fixture.

| Code | Exact throwaway fixture |
|---|---|
| N | Two notebooks `W Alpha`, `W Beta`; Alpha sections `One`, `Two`, Beta `Three`; pages `Parent`, `Child`, `Grandchild`, `Sibling`; unique body tokens per item; one database in a section and one inside Parent. Add ten sections for move-menu limits. |
| T | Separate paragraph/block samples: English `W exact 123`, Arabic `مرحبا بالعالم`, Hebrew `שלום עולם`, mixed numbers/punctuation, German `?Wie geht es?`; title with two trailing spaces; U+200E/U+200F and U+2066/U+2069 controls; code with tabs/trailing spaces. Keep an explicit UTF-8/codepoint manifest. |
| D | `W Tasks`/`W Projects`; three Tasks `A/B/C`, Amount `10/20/30`, statuses To do/In progress/Done, one multi-tag cell, one blank value per property, dates on adjacent days/week/month boundaries plus an end date; paired project relations. Expected total Amount 60, average 20; after excluding C total 30. |
| P | Dedicated page for every editor block, long enough to scroll, plus T; locally generated PNG/JPEG/GIF/WebP, invalid/SVG and >25 MB image; no outside URLs fetched. Small row text file with recorded hash; larger files only for explicit limits. |
| I | Synthetic `.md`, `.txt`, `.html`, `.docx`, text PDF, scan-only PDF, mixed folder and ZIP; `Parent.md`/`Parent/Child.md`, images and ordinary relative links; malformed/cancel/unsafe ZIP variants. Synthetic Notion HTML+CSV with 32-hex IDs, parent/subpage, Tasks/Projects, relation cells and row bodies; Arabic/Hebrew titles/content. Obsidian-style folder has front-matter title, ordinary Markdown links, `[[wiki]]` and `![[embed]]` for limitation reporting. |
| S | Fresh security-off home and separate protected copies, generated throwaway password/keyfile, wrong/missing keyfile. Populate N/D/P, sticky/calculator/search history, bin, page history and activity before each re-key. Never log credentials. |
| R | Distinct `ORIGINAL`/`CHANGED` test backups; protected/open, corrupt/truncated and copied old CLF1 homes; hashes before replacement. Backup-destination failure restricted to a disposable test directory. |
| X | After C5+E integration only: two locally paired test clients `W Reader`/`W Writer`, separate credentials/homes, exchange folder, hidden notebook, exact-text T and numeric D. Never use production AI credentials or remote services. |

## Check counts

| Area | Checks | A mock/script | B real exe/PC | C person |
|---|---:|---:|---:|---:|
| Navigation/organisation | 20 | 10 | 8 | 2 |
| Editor/presentation | 30 | 22 | 6 | 2 |
| Databases | 64 | 13 | 51 | 0 |
| Sticky notes/calculator | 17 | 6 | 10 | 1 |
| Search/palette | 6 | 1 | 4 | 1 |
| Import | 16 | 1 | 14 | 1 |
| Export/round trips | 14 | 0 | 13 | 1 |
| Security/lock | 18 | 0 | 13 | 5 |
| Backup/restore/old vaults | 13 | 0 | 13 | 0 |
| Settings/themes | 16 | 2 | 6 | 8 |
| Windows/tray/distribution | 11 | 0 | 6 | 5 |
| 1.6 foundation/bin/history/activity/AI | 25 | 2 | 22 | 1 |
| **Total** | **250** | **57** | **166** | **27** |

All 250 are not run. A browser assertion cannot satisfy a B or C row. The six explicit themes each have a C check; Match Windows has a separate resolver check.
## Numbered feature checks

### Navigation and organisation (inventory 3.1, 3.3)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W001 | B | First start and security-off notes | Fresh N home | Launch isolated portable copy; create first notebook/page; quit/reopen. | Notes open without password; content persists; no claim of confidentiality. Installed first-start alternatives are W217. |
| W002 | A | Notebook creation/edit | N | New notebook; set name, emoji, colour; edit all three through its menu. | Correct notebook and updated decoration in tree/Home; no other notebook changed. |
| W003 | B | Notebook ordering | N | Move Beta up then down; reopen. | Saved order matches each move; child sections/pages remain attached. |
| W004 | B | Sections | N | Add section; rename/recolour; move up/down; drag within Alpha; reopen. | Correct fields/order persist within Alpha; no cross-notebook move invented. |
| W005 | B | Pages and title editing | N+T | Create page by sidebar and Ctrl+N; change title/body; navigate away/reopen. | Each page belongs to chosen section; Unicode and saved body survive; no duplicate on title edit. |
| W006 | B | Nested sub-pages | N | Add Child inside Parent and Grandchild inside Child using menu and editor sub-page block; reopen each. | Correct parent chain and breadcrumbs; block opens the matching child. |
| W007 | B | Page reorder/move | N | Drag siblings; menu Move up/down; move Parent to Three; inspect descendants and reopen. | Order persists; descendants retain valid ownership; no content lost. Record actual eight-section menu limit. |
| W008 | B | Database move/drag in tree (1.6 A) | N+D | Move database by menu to Three; drag it among page siblings; reopen. | Database and rows intact; saved relative order correct; section move clears page parent as implemented. |
| W009 | A | Favorites | N | Star/unstar page and database from header/tree; open via Favorites and Home. | Star state, group and destination agree; unstar removes entry without deleting item. |
| W010 | A | Home | N+T | Open Home with/without favorites/recent pages; open its notebook/recent cards; dismiss open-vault warning. | Correct date/greeting, bounded card lists and destinations; warning dismissal respected. Daily gradient is not a custom banner feature. |
| W011 | A | Sidebar layout | N | Resize below/above limits; hide/show by button and Ctrl+\; reload mock. | Width clamped to 200–420; content remains usable; stored layout restored. |
| W012 | A | Keyboard tree/menu | N | Focus tree; Arrow keys, Home/End, Enter/Space; Shift+F10/Menu; Escape. | Logical focus/expansion; correct item's menu; no unintended edit/delete. |
| W013 | A | Navigation history | N | Open Parent, database, row; use arrows, Alt+Left/Right and Backspace/Shift+Backspace; type Backspace in editor/input. | Back/forward target correct; typing deletes text rather than navigating; bounded history. Physical mouse route W223. |
| W014 | B | Reopen last place (1.6 A) | N+D | Quit/reopen from page, database's second view, full row and sticky board; repeat lock/unlock with S. | Valid saved target/view restored after hydration, with no stale local-storage database association. |
| W015 | A | Last-place fallback | N | Save a target, bin/delete it, hydrate; also simulate missing saved database view. | Safe existing destination/Home; no blank editor, stale content or incorrect export target. |
| W016 | A | Top-bar import/export and notebook button (1.6 A) | N | Without hover, focus New notebook; activate Import and Export at wide/760px widths; use row context too. | Visible, focusable, clickable; correct dialog/scope; unresolved row cannot export previously selected notebook; no overlapping drag region. |
| W017 | A | Fixed global shortcuts | N | Ctrl+K, Ctrl+1, Ctrl+3, Ctrl+comma, Ctrl+N, Ctrl+\ and Escape dialogs. | Correct palette/view/create/sidebar behavior; focus returns; no duplicate action. Lock shortcut is W172. |
| W018 | C | Scrollbars and compact layout | N+P+D | At 760px and normal width, scroll tree/editor/database/settings with keyboard and mouse. | Visible usable scrollbars, no clipped controls or trapped content; preserve existing look. |
| W019 | C | Keyboard and assistive access | N+D | Tab through title bar/tree/editor/ribbon/dialogs/table; inspect focus and names with available Windows screen reader. | Focus visible and logical, dialogs announced/contained, controls named; record unavailable assistive tooling as not run. |
| W020 | A | Context menus and blocked browser keys | N | Right-click non-text surface then input; use ellipsis/Shift+F10; exercise blocked refresh/devtool keys in mock key handler. | Existing menu paths work; text editing remains available; no new mouse context-menu feature assumed. Real WebView behavior W224. |

### Editor and page presentation (inventory 3.4–3.5)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W021 | B | Autosave and save state | N+T | Type title/body; wait for saved state; switch pages and reopen; repeat with S. | Latest content persists; label distinguishes security on/off; failed save does not falsely report saved. |
| W022 | A | Paragraph/headings and Markdown rules | P | Use slash/ribbon/typing rules for paragraph and H1–H3; ribbon H4; convert back. | Correct node/level/text; H4 need not appear in slash menu. |
| W023 | A | Bullet list | P | Type `- `, add nested items, Tab/Shift+Tab, turn paragraph into bullet. | Nested list structure and text correct. |
| W024 | A | Numbered list | P | Type `1. `; add/nest/outdent items; convert back. | Numbering and indentation correct; text retained. |
| W025 | A | Checklist | P | Create nested tasks; tick/untick; move and undo a task. | Checked state belongs to correct task; nesting and undo intact. |
| W026 | A | Toggle | P | Insert collapsible section; title/body; close/open; undo edit. | Body retained and disclosure accessible; not a toggle-heading feature. |
| W027 | A | Shelf | P | Insert Shelf; edit title/body; fold/unfold. | Shaded shelf and content preserved; distinct from plain toggle. |
| W028 | A | Callout/highlight box | P | Insert highlight box; edit content/colour; convert where supported. | Content and supported colour retained; no executable markup. |
| W029 | A | Quote/divider | P | Insert each through slash/ribbon and typing rules; undo/redo. | Correct quote/rule nodes; adjacent text unchanged. |
| W030 | A | Tables in pages | P | Insert 3×3; add/delete row/column; toggle header; merge/split; resize; undo. | Cells/text retained as expected; contextual Table tab appears; no corrupt structure. |
| W031 | A | Code blocks | T+P | Insert by slash/ribbon/fence; type tabs/trailing spaces; import language-marked sample. | Literal text preserved and highlighting safe; no language-picker/copy-toolbar promise. |
| W032 | A | Mermaid diagrams | T+P | Insert Diagram, typed mermaid fence and Turn into Diagram; cycle nine samples; hide/show code; exit with Escape/Ctrl+Enter. | Valid diagrams render; source retained; errors readable; no editor lock-up. |
| W033 | A | Mermaid hostile/large input | P | Enter invalid syntax, hostile HTML/link labels, >50,000-character source and existing timeout fixture. | Safe error/limit, no script execution/remote load; text can be edited after failure. Record inability to abort renderer as known limit. |
| W034 | A | Inline equation | P | Type `$x^2$`; edit through node view; undo/redo. | Correct TeX/rendering; stable block identity and history; no spurious autosave. |
| W035 | A | Block equation/selection conversion | P | Insert display equation; convert selection; enter invalid TeX; edit back. | Safe error, text preserved, editor remains usable. |
| W036 | B | Picture acquisition/validation | P | Add picture with picker, paste and drop each accepted type; try SVG/mismatched bytes/>25 MB; reopen. | Valid image stored in test vault; unsupported/oversize refused; no external resource loaded. |
| W037 | A | Picture layout | P | Align, width %, crop/ratio, caption, float left/right, free-position/front/back; try inside list/table/imported columns. | Supported layout works; nested free mode restricted; other blocks remain editable. |
| W038 | A | Internal page/heading links | N+P | Insert Link to page; link text to page and heading; activate then bin target. | Correct destination while active; missing target handled safely without external navigation. |
| W039 | B | Web link validation | P | Set http/https and invalid javascript/file link; inspect normalized link and dispatch with offline instrumented PC harness. | Only allowed protocols reach OS opener; no note data in request. Do not make an external network request for this check. |
| W040 | A | Inline database | N+D | Slash Database (inline); edit row; navigate away/back. | Embedded DB exists in tree and page; row changes appear in full DB. |
| W041 | A | Linked database view | N+D | Insert linked view of existing DB; change linked filter/view; edit a cell. | Same underlying rows; saved view settings correct; no duplicate DB. |
| W042 | A | Date/time insertion | P | Use slash date and ribbon date/time stamp. | Local date/time inserted as plain text at caret; no date-chip promise. |
| W043 | A | Text styles/bubble toolbar | P | Select text; exercise font/size, bold/italic/underline/strike/code, 18 text/highlight colours, alignment and remove formatting. | Correct marks/styles on selection only; clear-format keeps text. |
| W044 | A | Ribbon/slash/block conversion | P | Home/Insert/Table tabs; collapse/reload; slash keyboard filtering/select/Escape; Turn into supported block types. | Correct command, focus and stored collapse state; content retained. |
| W045 | C | Block handle and drag | P | With real mouse add below, Duplicate, Move up/down, Delete then undo; drag paragraphs/list/table. | Intended block moves/duplicates exactly once, delete undo restores it; selection and scrolling usable. |
| W046 | A | Automatic/forced RTL | T | Type Arabic/Hebrew/English blocks and Shift+Enter mixed lines; force RTL/LTR/auto using ribbon/slash. | Per-block/line direction matches first strong character/override; punctuation and text retained. Export limitations tested separately. |
| W047 | B | Editor undo/redo and exact text | T | Type/format/move; Ctrl+Z/Ctrl+Y; save/reopen; compare UTF-8/codepoint fixture. | Undo/redo correct in live session; controls/spaces/code text preserved in stored blocks. Undo history persistence not promised. |
| W048 | B | Page icon/cover | N+P | Choose/search/random emoji, upload icon; gradient/solid/upload cover; reposition/remove; reopen. | Chosen look stored; removals work; uploads use actual native picker; page text unchanged. |
| W049 | C | Header, icon/cover and Home layering | N+P | Look at breadcrumb, word count, star off/on, cover buttons and Home glyph at compact/normal width, light/dark. | Correct counts/status, readable star, clickable cover controls, no dimmed/covered glyphs. |
| W050 | B | Dormant columns/comments compatibility | Imported P fixture | Import a supported column/comment-mark sample or open synthetic old page; edit adjacent text; save/export/reopen. | Existing supported column/mark content survives or documented conversion loss is reported; no creation UI or imported-comment parity invented. |

### Databases: every view and property (inventory 3.6)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W051 | B | Database creation/first-use backup | Fresh N+D | Create blank DB from sidebar, section/page and inline entry; cancel another; inspect backup and reopen. | Correct parent/location, no cancelled DB; pre-database safety copy/warning on first use; schema/rows persist. |
| W052 | A | View lifecycle | D | Add each of seven views; rename/duplicate/move/delete a view; switch tabs. | Settings belong to correct view; valid remaining view opens; last-place view agrees. |
| W053 | A | Table view | D | Freeze/wrap/show row numbers, resize/reorder/hide columns, reorder rows and set footer sum. | Row/cell order correct; total 60; settings round-trip through mock store. |
| W054 | A | Board view | D | Group by Status; drag A into Done; add card in group; add swimlane. | Status/group inherited correctly; no row duplication; correct swimlanes. |
| W055 | A | List view | D | Show/hide properties; group/sort; open row. | Correct visible rows/properties and target; no manual-list-order feature assumed. |
| W056 | A | Calendar view | D | Select date field; month/week; next/previous/today; drag date; create dated row. | Correct day/week placement and saved date; no hour-grid promise. |
| W057 | A | Gallery view | D+P | Select cover source/card size/properties; open card with/without image. | Cards/targets correct; missing cover safe; page-content placeholder recorded. |
| W058 | A | Timeline view | D | Select start/end; zoom; resize/move bar; turn dependencies on and inspect arrow. | Correct date range and arrow; no dependency drag-creation promise. |
| W059 | A | Chart bar | D | X Status, Y sum Amount; toggle grouped/stacked/horizontal/labels/colours; click bar. | Bar values total 60 and drill list contains exactly contributing rows. |
| W060 | A | Chart line | D | X date, Y Amount; bucket by day/week/month/year; click point. | Correct totals/order/buckets and contributing-row drill. |
| W061 | A | Chart donut | D | Group Status; count; click slice; hide labels. | Slice counts 1 each and correct drill; no extra pie view assumed. |
| W062 | A | Chart number | D | Count then sum/avg/median/min/max/range/unique on suitable field; filter out C. | Display matches hand-calculated fixture; sum 60→30; drill matches aggregation. |
| W063 | B | Text property | D+T | Add Text; edit/clear multilingual multiline value; reopen. | Exact supported text retained; empty value cleared. |
| W064 | B | Title property | D+T | Rename row from title cell and row page; reopen both. | One consistent title, including Unicode; protected title schema cannot be removed illegally. |
| W065 | B | Number property | D | Set decimal, negative, currency, percent and blank; reopen/query. | Numeric values stored without display-rounding loss; invalid input refused. |
| W066 | B | Select property | D | Create/rename/recolour/reorder options; choose/clear; reopen. | Stable option IDs and correct cell labels/colours. |
| W067 | B | Multi-select property | D | Choose two tags, rename option, remove one, clear; reopen. | Set membership correct; no duplicate tag or stale deleted option. |
| W068 | B | Status property | D | Configure status groups; change via cell and board; reopen. | Correct status/group and matching board placement. |
| W069 | B | Date property | D | Pick start/end/time, clear end/time, reopen around month boundary. | Stored range/time correct; no unintended day shift. |
| W070 | B | Person property | D | Enter/change/clear two person names; reopen. | Plain-text person data retained; no account/network behavior assumed. |
| W071 | B | Files property | D+P | Attach text/image; save as and compare hash; open disposable text file; remove; relaunch. | Correct file round-trip, safe OS open, removed cell cleared; temporary plaintext open-copy cleanup at next start. |
| W072 | B | Checkbox property | D | Toggle on/off/clear; reopen. | Correct boolean/empty semantics and no other row changed. |
| W073 | B | URL property | D | Set valid URL, edit/clear; test unsupported scheme without following link. | Correct stored/display value; unsafe open refused. |
| W074 | B | Email property | D | Set `w@example.invalid`, edit/clear; reopen without dispatching mail app. | Correct value; no mail sent. |
| W075 | B | Phone property | D | Set synthetic international number with spaces; edit/clear; reopen. | Correct supported text; no call initiated. |
| W076 | B | Formula property | D | Formula Amount×2; change A to 11; exercise string/list/date/boolean and relation formulas using existing assertion fixtures; enter syntax error, then repair. | Numeric values 20/40/60 then 22 for A; other results match fixture oracles/recorded Notion differences; safe error; recompute after reopen. |
| W077 | B | Relation property | D | Pair Tasks↔Projects; link/unlink rows on both sides; reopen. | Both directions agree; no duplicate/stale link or cross-database damage. |
| W078 | B | Rollup property | D | Roll up linked Amount sum/count/average; unlink a task; reopen. | Correct numeric aggregation and empty/cycle/depth error rather than hang. |
| W079 | B | Created time property | D | Create a row; edit it later; reopen. | Creation timestamp unchanged by edit. |
| W080 | B | Created by property | D | Create row as person; reopen; repeat via X after integration. | Correct actor/default representation; no account identity invented. |
| W081 | B | Last edited time property | D | Edit row after creation; query/reopen. | Updated timestamp advances for meaningful edit. |
| W082 | B | Last edited by property | D | Edit as person, then automation; repeat AI route with X. | Last editor reflects actual supported actor, not stale prior attribution. |
| W083 | B | ID property | D | Set prefix/options; create/duplicate rows; reopen. | Stable unique row identifiers with expected prefix; existing IDs unchanged. |
| W084 | B | Place property | D | Enter/change/clear synthetic place value through picker/editor; reopen. | Supported structured/display value retained without geocoding/network. |
| W085 | B | Button property | D | Add actions set Amount, add row, notify; press once; reopen. | Valid actions run once on intended row; no phantom stored button value. |
| W086 | B | Property schema lifecycle | D | Add/rename/reorder/duplicate/delete ordinary property; reopen. | Duplicate keeps configuration, not source values; computed fields read-only; unrelated cells retained. |
| W087 | B | Filters | D | Nested AND/OR, blank/string/number/relative-date filters; save/reopen. | Exact expected row sets; invalid/deep filters safely rejected/clamped as source defines. |
| W088 | B | Sorts | D | Multi-sort Amount then Title ascending/descending; blank/tie values; reopen. | Stable correct ordering including null/ties. |
| W089 | B | Groups/subgroups | D | Group Status/tag/date bucket and numeric ranges; subgroup; collapse/reopen. | Correct membership/counts; no missing/duplicated rows. |
| W090 | B | Footer calculations | D | Sum/count/empty/nonempty and representative numeric calculations with a saved filter; compare search separately. | Values match filtered query; record current client-search/footer limitation rather than claiming it fixed. |
| W091 | B | Row creation/duplicate/reorder | D | Create/duplicate row with properties/body; reorder rows; reopen. | New ID, expected copied values/body, preserved source row, valid order. |
| W092 | A | Row pages and peeks | D+P | Open same row full/side/centre; edit properties/body; close/reopen. | Correct row context, property lines and text; no wrong prior notebook export scope. |
| W093 | B | Row templates | D+P | New template and Save as template from row; edit name/icon/default values; create row from it; delete template. | Defined values/body copied with fresh identity; source/template unaffected by new-row edits; no nonexistent duplicate-template menu assumed. |
| W094 | B | Template legacy key safety | Synthetic DB legacy IDs | Apply/save template with legacy `__proto__`, `constructor`, `prototype` keys using controlled backend fixture. | Own-property data preserved safely; prototype not changed; new invalid IDs refused. |
| W095 | B | Default row template | D with two templates | Set Use for every new row on first template; create via New row; switch default and repeat; delete default template and create again. | Only current default applied to new rows; existing rows untouched; deleting default clears it safely; settings persist. |
| W096 | B | Recurring row templates | D with controlled due timestamps | Set Daily, Weekly, Monthly in turn; reopen database when due and again in same period; choose Never; include month boundary with injected time. | At most one due row per opening/period per implemented repeat rule, correct content/date naming; no duplicate same-period row or background closed-app scheduler claimed; Never stops recurrence. |
| W097 | B | Sub-items | D | Turn on sub-items; add child; change relation; show parent/child; reopen. | Correct hierarchy, no cycle or orphan; totals/grouping follow configured view. |
| W098 | B | Dependencies | D | Enable dependencies; link A→B; unlink; inspect timeline; reopen. | Valid dependency relation and arrow; unrelated relationships intact. |
| W099 | B | Database lock | D | Lock DB; attempt schema/view edits then edit row; unlock. | Schema/views protected while intended row edits remain allowed; state persists. |
| W100 | A | Database search/pickers | D+T | Search title/cells; click tag/date displayed in table; clear search. | Correct rows; intended picker opens and accepts change; footer scope limitation visible. |
| W101 | B | Database undo/redo | D | Change cells/schema/views/order; undo/redo; exceed 50 edits; close last view. | Correct last 50 changes; history scoped per DB and dropped when last view closes as designed. |
| W102 | B | Automation: row added | D | Create enabled row-added rule with filter, setProp/addRow/notify actions; add matching/nonmatching rows. | Matching rule runs once with correct values, nonmatching skipped. |
| W103 | B | Automation: property changed | D | Trigger on one property; change another, then target; disable rule and repeat. | Correct trigger and enabled state; no self-trigger loop. |
| W104 | B | Automation: date arrives | D with controlled due dates | Set due trigger/offset; let due; lock/unlock and reopen after synthetic delay. | Runs only unlocked/open; catch-up respects seven-day limit; no duplicate run. |
| W105 | B | Automation limits/trash | D synthetic action chain | Run chained add/set/trash rules up to/beyond limits; inspect activity/bin. | 3-step/200-action/50-new-row limits enforced; reversible trash and readable result; no partial uncontrolled loop. |
| W106 | B | Date reminders | D | Due reminder while unlocked; dismiss; repeat locked and after unlock. | In-app notice only at intended time; no Windows notification promise; no duplicate due notice. |
| W107 | B | CSV database creation/import | I+D | New DB From CSV; import second CSV into existing DB; map fields; cancel a pick. | Correct BOM/quoted/newline/Unicode cells and types; cancelled import unchanged; row/column caps honored. |
| W108 | B | CSV export | D+T | Export filtered/sorted view; inspect BOM/CRLF/quoted cells, reimport. | Correct columns/order/numbers/Arabic/Hebrew; exported cells read back; formula-as-text safety follows current writer. |
| W109 | B | Database Markdown export | D+T | Export this view as Markdown through Save dialog; read file. | View's rows/columns and Unicode represented; no claim it recreates DB schema on page import. |
| W110 | B | Database HTML export | D+T | Export filtered view as HTML; inspect escaped markup and values. | Correct view data, safe text/direction; no extra hidden/binned rows. |
| W111 | B | Database Word export | D+T | Export view as DOCX; reopen in available offline viewer. | Valid document and expected rows/columns; visual RTL review W167. |
| W112 | B | Database PDF export | D+T | Export view as PDF; extract text/count rows then inspect output. | Valid PDF and expected values; missing glyphs reported, not silent; visual review W167. |
| W113 | B | Database validation/size caps | Dedicated D copy | Submit invalid property/cell/type, foreign relation, malformed schema and over-limit fixture through controlled harness. | Refusal/no unrelated changes; schema/value/attachment/export caps enforced. Test 900 MB boundary via bounded fixture, not real attachment. |
| W114 | B | Fine-grained database saves (1.6 C4) | D | Change cell, property, view, template, automation and manual order while another read is pending; try stale revision. | Small writes preserve unrelated state; invalid/stale operation refused; UI refresh and row titles agree. |

### Sticky notes and calculator (inventory 3.7)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W115 | B | Sticky creation/content | T, two new notes | New note; edit title/body; close/reopen desktop note and app. | Saved note appears on board with current text; close retains it. |
| W116 | A | Sticky board | Three notes with distinct edit times | Edit oldest; inspect board; open card. | Grid sorted by last edit; correct note opens; no free-layout/drag-order promise. |
| W117 | B | Sticky styles | Eight disposable notes | Choose Lemon/Blossom/Mint/Sky/Lavender/Peach/Cloud/Charcoal; bold/italic/checklist; reopen. | Correct supported style/content survives. |
| W118 | B | Sticky geometry/on-top | Two sticky windows | Move/resize, toggle Keep on top, close/reopen, change display bounds only in test session. | Geometry remembered and recoverable onscreen; on-top flag works. |
| W119 | A | Sticky multiselect | Three notes | Select and Ctrl+click; click/Space/Enter tick; All/None; Escape; Delete then Cancel. | Correct selection/keyboard states; cancelled bulk deletion leaves all notes. |
| W120 | B | Sticky delete/bulk bin | Three notes | Confirm single two-click delete and multidelete; inspect bin; restore. | Exactly selected notes binned in transaction and restored with styles/text. |
| W121 | B | Open all/Close all | Three stickies | Open all; Close all on board; reopen. | Windows close without deleting notes; no duplicate windows. Tray entry is W216. |
| W122 | B | Sticky save-before-lock | S+T | Type immediately then lock with sticky open; unlock/reopen. | Pending edit flushed within bounded protocol; no decrypted sticky window after lock. |
| W123 | B | Calculator windows | Empty calc history | Open from board and sticky twice; move/close one; lock/unlock. | Independent frameless calculators, intended on-top behavior, no data exposed while locked. |
| W124 | A | Basic calculator | Scratch history | Evaluate `1+2*3`, `(1+2)*3`, `200+10%`, `200-10%`, `0.1+0.2`; keyboard/clear/backspace. | 7, 9, 220, 180, 0.3; display/entry behavior correct. |
| W125 | A | Scientific calculator | Scratch history | Powers, factorial, implicit multiply, sin/asin, logs, constants, Ans; invalid divide/domain expression. | Correct engine values or readable CalcError; no `eval`/script execution. |
| W126 | A | Deg/Rad | Scratch history | Scientific sin(30); evaluate; switch DEG↔RAD after result and before another evaluation. | Result/unit changes as designed and visibly communicates unit; history not mistaken for active expression. |
| W127 | B | Shared calculation history | Two calc windows | Evaluate in each; read history from both; restart. | Same ordered persisted history; no cross-window loss. |
| W128 | B | History cap/clear | 501 synthetic calculations | Add entries; inspect oldest; Clear; restart. | Newest 500 kept; clear shared/persisted. |
| W129 | B | Calculator Copy | Scratch calc `7*6` | Copy result; inspect disposable clipboard; restore/clear test value. | Only displayed `42` copied, no expression/history/secret. |
| W130 | A | Sticky/calculator RTL | T | Type Arabic/Hebrew sticky lines; open calculator/history in RTL context. | Text direction correct and numeric controls/order usable. |
| W131 | C | Floating-window usability | Stickies+calculators | Person inspects resize/drag, focus, colours and overlapping windows at desktop scaling. | Controls legible/clickable, windows recoverable; no styling change required for a pass. |

### Search and palette (inventory 3.2)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W132 | B | Global search | N+D+T+stickies | Search unique title/body/row/notebook/sticky tokens, case variants and multiword queries. | Correct hits/snippets and all-word/case behavior; selecting hit opens correct object. |
| W133 | B | Binned items excluded | N+D+stickies with unique tokens | Bin notebook, section, parent/subpage, database, row, sticky; query tokens after each, restore and repeat. | No binned item/descendant leaks in search or result navigation; restored live items reappear. |
| W134 | A | Palette commands | N | Open Ctrl+K; run each listed New page/sticky, Notes, Settings, Import, Export action. | Correct action/dialog/context once; empty/no-result state usable. Backup/lock command persistence is covered by B checks. |
| W135 | B | Recent searches | Six unique T terms | Search all six; reopen palette/restart; remove one; Clear all. | Newest five stored in vault; remove/clear persists; no content-bearing browser cache. |
| W136 | B | Locked search | S | Search before lock; lock; attempt query/read via test UI/IPC; unlock. | Prior hits cleared; nonempty data access refused while locked; empty query reveals nothing. |
| W137 | C | Find-in-page current behavior | P | Press Ctrl+F in real WebView and dismiss; record actual UI. | Record built-in WebView/find limitation honestly; no Notaya find/replace feature assumed or silently added. |

### Import (inventory 3.8; all converter families)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W138 | B | Markdown importer | I+T | Import headings/lists/tasks/tables/code/math/Mermaid/relative images; inspect resulting blocks. | Supported structure/text and local images retained; no network fetch. |
| W139 | B | Plain-text importer | I `.txt`+T | Import multiline UTF-8 text with blank lines. | Plain paragraphs/text and Unicode retained, not interpreted as executable markup. |
| W140 | B | HTML importer | I HTML+T | Import styles/links/table/callout/toggle/columns, script/event/remote-image samples. | Supported formatting mapped safely; scripts/events absent; remote picture stays link; losses listed. |
| W141 | B | Word importer | I DOCX+T | Pick DOCX with headings/styles/lists/table/local image; also malformed/encrypted unsupported copy. | Supported blocks imported; refusal/omissions for unsupported input; originals untouched. |
| W142 | B | PDF importer | I text PDF and scan-only PDF | Import both; inspect text/order and summary. | Text PDF yields supported text; scan-only not falsely OCR'd; no picture/layout parity promise. |
| W143 | B | Folder/mixed-files import | I mixed folder | Pick folder and mixed file selection in separate runs; inspect sections/parent-child hierarchy. | Each run creates one new notebook; unrelated existing notes untouched; supported files only. |
| W144 | B | ZIP/nested Notion parts | I ZIP with Part-N.zip | Import archive and nested parts; inspect pages/images/hierarchy. | Supported nesting assembled once; no arbitrary filesystem extraction. |
| W145 | B | Notion Markdown+CSV databases | I synthetic Notion export | Import MD+CSV ZIP; inspect Tasks/Projects properties, rows, relation cells, row-page bodies. | Real databases and reciprocal relations; row bodies folded once; formula values/person text limitations recorded. |
| W146 | B | Notion HTML plus CSV merge | I matching IDs in HTML/MD/CSV | Choose both exports together; inspect count, colour/icon/cover/links. | One page per source ID, HTML formatting plus CSV row data; no duplicates; views/formulas not invented. |
| W147 | B | Obsidian/other Markdown folders | I Obsidian-style folder | Import ordinary MD/relative links/images/front-matter title plus wiki/embed examples. | Generic supported Markdown imports; title used/rest front matter discarded as source defines; wiki/embed losses explicitly recorded, no dedicated importer claimed. |
| W148 | A | Import disclosure/entry points | N | Open from title bar, palette, notebook area/empty state; inspect Files/Folder and cancellation UI. | Consistent dialog; current text says databases supported and comments/history omitted; no stale “databases not imported” sentence. |
| W149 | B | Import progress/cancel | Large synthetic I copy | Start import; cancel during conversion; repeat cancel in native picker. | Progress bounded; no partial new notebook or unrelated mutation after cancellation. |
| W150 | B | Import bounds/hostile source | I malicious copies | Try traversal/absolute/UNC ZIP entries, symlink/junction, malformed ZIP, oversized/deep/bomb fixtures via bounded harness. | Safe refusal/skip reasons; no read outside chosen root/data folder; aggregate budget/deadline respected. |
| W151 | B | Import report/original integrity | I with unsupported media/broken link | Hash source; import; inspect left-out list and hash again. | Counts and actionable omissions agree with actual result; source bytes unchanged; unsupported embeds not silently claimed retained. |
| W152 | B | Arabic/Hebrew importer fidelity | I+T all five text formats | Import MD/text/HTML/DOCX/PDF variants; compare supported text/codepoints and block direction. | Unicode preserved within format's documented limits; direction loss separated from text loss. |
| W153 | C | Notion Hubs Word/PDF (WP-15) | Synthetic Hubs-like I, no private export | Import parent/subpage/database-rich fixture; export Word/PDF; person opens both. | Main-page body, database content and Arabic/Hebrew visible; no unexplained empty Hubs page; source-fixture absence means not run. |

### Export and round trips (inventory 3.9)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W154 | B | Export scope/Save dialog | N+P+D | Export page/selected pages/section/selected sections/notebook/selected notebooks; include subpages on/off; cancel pick. | Exactly requested live content and valid filenames; cancel changes nothing; unencrypted-file warning shown; no output in vault data root. |
| W155 | B | Markdown export | P+T | Export single file with blocks/styles/links/image; inspect file/assets. | Supported Markdown/text/assets present; math/Mermaid as source, free pictures in flow; documented direction limits. |
| W156 | B | HTML export | P+T | Export page/section; inspect escaping, links, style/direction and assets. | Safe self-contained supported content; no live external image fetch or script; correct headings/order. |
| W157 | B | Word export | P+T | Save DOCX; inspect ZIP/document XML and open offline. | Valid headings/page breaks/tables/images; text retained; no inserted TOC promise. |
| W158 | B | PDF export | P+T | Save PDF; inspect pages/bookmarks/extracted text, web links without following. | Valid document, subpage breaks, flat bookmarks and supported fonts; known internal-link/TOC limits recorded. |
| W159 | B | Separate files/ZIP | N+P | Export each format as separate files ZIP; inspect marker/hierarchy, names with collisions/path punctuation. | Valid safe ZIP, manifest/assets and distinct files; no path traversal/name overwrite. |
| W160 | B | Inline database export fix (1.6 B) | P with own+linked D | Export MD/HTML/DOCX/PDF, single and ZIP; inspect DB contents. | Inline DB data appears instead of disappearing; correct live rows and linked-view scope; no binned content. |
| W161 | B | Database scope in notebook/section export | N+D | Export enclosing section/notebook with standalone and nested DBs in all formats. | Existing lane-B supported DB inclusion retained; any omitted scope surfaced as defect/limitation, not passed by empty output. |
| W162 | B | MD/HTML/DOCX round trips | N+P+T | For each format and scope, export single+ZIP then import into fresh home; compare semantic manifest. | Supported text/block structure/order/images/links/colours survive; intentional format losses listed by field, not byte-equality asserted for conversion. |
| W163 | B | PDF round trip | P+T | Export text PDF, reimport into fresh notebook; compare textual content. | Supported text returns; text-only PDF import need not recreate pictures/layout/DB schema. Report shaping/extraction losses. |
| W164 | B | Internal-link round trip | N | ZIP-export linked page/subpage/heading and excluded target; reimport. | Exported pages link to new imported IDs; excluded targets degrade safely to text; no stale `notaya:`/source markers. |
| W165 | B | Export validation/failure | N disposable output folder | Attempt data-folder destination, invalid scope and unwritable output; repeat over existing disposable file. | Safe error; no live-vault overwrite; previous output preserved where writer promises atomic replacement; no silent partial success. |
| W166 | B | Arabic/Hebrew round trips | T+P | Export/import MD/HTML/DOCX and text PDF; compare Unicode manifest and supported direction separately. | Arabic/Hebrew/English text retained within declared conversion limits; forced/per-line direction losses documented. |
| W167 | C | Export typography and compatibility | All exported T/P/D files | Person opens Word/PDF/HTML offline: Arabic joins, Hebrew order, mixed punctuation, tables, pictures, CJK/Greek/Cyrillic/emoji. | Legible supported scripts/layout; glyph substitutions/loss warnings explicit; per-line-direction and unsupported-font limits recorded. |

### Security and locking (inventory 3.10; P1–P4, P7–P8, P16, P20–P21)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W168 | B | Security off | Fresh S | Edit/save; Ctrl+L/minimize/wait through idle; restart. | Opens without password, no security lock/capture protection; truthful “not private” wording. |
| W169 | B | Enable security/master strength | S with all journals/items | Reject short/weak/mismatched password; enable with valid generated password and acknowledgment. | Invalid attempt leaves vault unchanged; valid enables protected unlock and retains every populated item/table. |
| W170 | B | Disable security | Protected S | Wrong password/keyfile then correct credentials; quit/reopen. | Wrong attempt unchanged/rate-limited; correct re-key makes open vault with all data, clear warnings. |
| W171 | B | Unlock and guessing delay | Protected S | Wrong password 3+ times, observe increasing delay; correct password; restart and repeat. | Refused guesses reveal no notes; correct unlock succeeds; restart reset limitation recorded. |
| W172 | B | Explicit lock | Protected S+pending T | Lock button, Ctrl+L and palette Lock in separate runs; unlock. | Windows flushed then locked/session cleared; latest successful saves retained; backup made when changed. |
| W173 | C | Idle auto-lock | Protected S, idle 1 minute | Set 1 minute; stop user input for interval; unlock; contrast with security-off copy. | Protected copy locks; open copy stays open; record real elapsed idle and foreground activity. |
| W174 | C | Win+L lock | Protected S | Type disposable marker, Win+L, sign back in. | Test app locked, no visible note windows; inspect saved marker after unlock; do not infer bounded flush for OS emergency path. |
| W175 | C | Sleep/hibernate | Protected S | With approved test desktop sleep/resume, then hibernate/resume if available. | Protected vault locked on return; emergency save limit reported honestly; unavailable hibernate not run. |
| W176 | C | Drive removal | Disposable protected S on approved removable drive | Save, safely simulate actual test-drive loss/removal; return drive, restart/unlock. | Locks/fails safely; saved vault remains readable; no real drive or vault involved. |
| W177 | B | Lock on minimize | Protected S | Toggle option off/on; minimize/restore after edit each time. | Off stays unlocked; on locks; no save regression. |
| W178 | B | Keyfile create/pick/require | S | Create keyfile through dialog, protect vault; restart; missing/wrong/correct file; move test folder and retry. | Required file plus password enforced; no path-only authentication; moved folder still opens with correct bytes. |
| W179 | B | Change master/re-key | Fully populated S | Change password and add/change/remove keyfile; reopen new live vault and copied old backup with respective credentials. | Fresh key/vault binding, all records/attachments/history/activity retained; old credentials cannot open new live data; old backup still uses old credentials. |
| W180 | B | Re-key atomic failure | S copy, injected write/corrupt-journal fixture | Use existing unit fault tests as oracle; on a reviewed instrumented test exe, fail re-key mid-transaction and reopen source copy. | Old complete vault survives or transaction rolls back; no mixed old/new encrypted tables. Runtime fault harness unavailable means not run, regardless of unit pass. |
| W181 | B | Old unprotected-copy cleanup | S plus ordinary/hourly/predelete/open `.nota`/`.clf`, protected backup and outside/manual/undo copies | Enable security with cleanup ticked and unticked on separate copies; inspect hashes/list. | Fresh protected backup first; eligible unprotected automatic copies removed only with selected option; protected/manual/outside/restore-undo/set-aside copies retained; warning truthful. |
| W182 | C | Screenshot protection | Protected S with sticky/calc | Toggle capture switch off/on; capture each real window/lock screen; then security-off copy. | Supported capture exclusion when on, visible when off/security off; startup lock-screen nuance recorded; do not claim all recording tools blocked. |
| W183 | B | Tamper/truncation detection | R copies | Corrupt header/record/attachment, swap row/vault binding, truncate; try opening each disposable copy. | Safe corruption refusal, no accidental empty replacement or note-bearing error. Old-record replay limitation remains documented. |
| W184 | B | Lock flush timeout/failure | S with page/sticky/settings/database edits | Lock/quit immediately; include slow/missing window ack and failed-save injected case. | Bounded ≤2-second explicit flush protocol, retries/unsaved state as designed, no stale save after session replaced. OS-event limits distinguished. |
| W185 | B | UI/IPC/security boundary | S+P hostile fixtures | Inspect IPC responses/events/logs during lock/error/link/image cases; attempt allowed test invokes while locked. | No keys/tokens/passwords echoed in responses/events/logs; entered password only submitted through approved auth flow; CSP/image/type restrictions enforced; no arbitrary command dispatch. This is a bounded test, not an independent security review. |

### Backups, restore and old vaults (inventory 3.10–3.11)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W186 | B | Manual backup | N+S | Back up now via Settings and palette; pick output; restore copy into fresh home; click Open data folder. | Verified readable snapshot of all content/journals; output not plaintext; cancel/live-vault destination safe; opened folder is this test home's data folder. |
| W187 | B | Lock/quit backup rotation | R + 12 edits/lock/quit cycles | Run changed/unchanged cycles with deliberately misleading filenames/clock-order fixtures. | At most newest 10 ordinary valid copies by modification time, newest written not discarded; unchanged session not needlessly copied. |
| W188 | B | Hourly open-vault backup | Security-off R | Use existing test-only interval hook after checking units; edit, pause, no-change interval, repeat 5 times. | Hourly dirty/quiet timing correct; newest 3 in separate pool; ordinary copies unaffected. No live clock change. |
| W189 | B | Predelete safety backup | N+R | Delete notebook/section containing descendants; repeat with backup destination made unavailable inside test home. | Verified safety copy before deletion, newest 5 separate; failed backup refuses deletion; confirm counts accurate. |
| W190 | B | Backup failure/retry | R restricted destination | Fail manual/hourly/quit backup through test-only fault; restore destination; observe notices/backoff. | Useful warning, bounded retry without toast storm, previous good files retained; quit notice shown when applicable. |
| W191 | B | Backup list while locked/unlocked | R mixed copies | Open list from Settings and lock screen; include corrupt/zero-byte and `.clf` entries. | Correct descending available copies; bad copies not counted as verified; no decrypted note data in list. |
| W192 | B | Restore from listed backup | R | Restore ORIGINAL while CHANGED current, locked then unlocked; reopen undo copy. | Verified source installed; current preserved as restore-undo; ORIGINAL active, CHANGED recoverable. |
| W193 | B | Native picked-file restore | R outside backup directory | Choose source through real picker; cancel; choose again and restore; try unpicked absolute/UNC/traversal source. | Only selected/bare-listed safe source accepted; cancel unchanged; native path route genuinely exercised. |
| W194 | B | Restore validation/security downgrade | R corrupt/wrong creds/open+protected | Try corruption/wrong credentials; restore open backup onto protected copy with warning unchecked then checked. | Failure leaves live bytes unchanged; downgrade requires explicit allowed choice; all tables verified before swap. |
| W195 | B | Restore rename/crash rollback | R disposable copies/fault harness | On a reviewed instrumented test exe inject failure between stage/set-aside/install renames and reopen; use existing unit rollback cases as oracle. | Original or valid restored vault recoverable; no silent loss; interrupted files preserved and message clear. Missing runtime harness means not run. |
| W196 | B | Start over/recover | Unopenable disposable S | Wrong/missing typed START OVER; then correct; start new notes; restore set-aside later. | Gate enforced; old vault preserved as set-aside; new notes separate and old recoverable with original credentials. |
| W197 | B | Cipherleaf `.clf` open/upgrade | Synthetic CLF1 open and protected R, optional keyfile | Hash original; open copy in Notaya; inspect format/backups; repeat injected migration failure; reopen. | CLF1 readable, upgraded to NOTA v2 with fresh binding transaction; vault-before-notaya copy byte-identical; old backups untouched; failure leaves old vault valid. |
| W198 | B | 1.5.1 compatibility (spec G3) | Synthetic 1.6 home+matching archived 1.5.1 exe | Give old exe a separate copy only; open/read page/sticky/DB; keep original untouched; then reopen original with new exe. | Existing content readable as G3 requires; unsupported history tables not corrupted in original; absent safe old exe/fixture = not run. Never downgrade a real vault. |

### Settings and themes (inventory 3.4, 3.12)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W199 | C | Sunrise theme | N+P+D | Select Sunrise; inspect Home/editor/table/all view toolbars/dialogs/selected+unselected icons. | Existing palette/layout readable; no invisible focus/star/dropdown/overlap. |
| W200 | C | Sunrise dark theme | N+P+D | Repeat W199 with Sunrise dark. | Same visual criteria on dark surfaces. |
| W201 | C | Black theme | N+P+D | Repeat W199 with Black, including native dropdown/pickers. | Controls and text readable against black; no hidden focus. |
| W202 | C | Graphite theme | N+P+D | Repeat W199 with Graphite. | Correct readable graphite surfaces and controls. |
| W203 | C | White theme | N+P+D | Repeat W199 with White. | Correct readable white surfaces and controls. |
| W204 | C | Paper theme | N+P+D | Repeat W199 with Paper. | Correct readable paper surfaces and controls. |
| W205 | A | Match Windows mode | N mock media-query light/dark | Select Match Windows; change emulated preference twice. | Resolver follows defined light/dark themes; does not invent system Sunrise mapping. Real Windows rendering recorded with theme C checks. |
| W206 | B | Accent/font/text size | N+T | Preset/custom accent; Inter/Segoe/Serif/Mono; size 12 then 20; quit/reopen. | Supported settings persist/apply; no note content written into appearance cache. |
| W207 | B | Width/density/layout settings | N+P | Narrow/Normal/Wide/Full and each density; immediately lock/reopen after pending change. | Correct settings saved, content not truncated/lost; pending settings flush works. |
| W208 | C | Window material | N on supported Windows | Solid/Mica/Acrylic; compare normal/maximized/locked. | Native supported material/fallback works, readable controls; unavailable Windows capability recorded. |
| W209 | B | Lock backdrop | S | Select Glow, Plain, Mesh, Aurora in turn; lock/restart after each. | Chosen built-in backdrop persists without exposing note content; no uploaded lock-picture feature assumed. |
| W210 | B | Default sticky colour | N | Choose each allowed default; create new sticky; inspect existing notes. | New note uses default, existing colour unchanged; selection persists. |
| W211 | B | Error log | Disposable error fixtures | Trigger sanitized frontend error; open Settings Error log and folder; restart. | Local records available without note text/passwords/keys; crash marker claims limited to supported paths. |
| W212 | A | About/credits/licences | Mock Settings | Open About and licences; scroll/search relevant notices; Escape. | Correct app/build version, vault-location/security wording, named Mohamed/Ahmed/Claude/ChatGPT credits, full notices readable; no obsolete Passwords panel. |
| W213 | B | Settings persistence/cache hygiene | S+N | Change appearance/layout/last place; lock/reopen and inspect disposable browser storage/IPC. | Approved appearance/layout cache only; legacy `dbview:` associations removed; encrypted target setting belongs to current vault. |
| W214 | C | Security wording/settings consistency | Open/protected S | Person reviews Home, saved state, Settings, unlock/restore/export warnings. | Plain accurate encryption/network/no-reset/old-copy warnings; version labels/current names coherent. |

### Windows, tray and distribution (inventory 3.11; WP-15)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W215 | B | Close to tray / quit | Open/protected S | Enable tray close; edit then close/restore; disable and close; use Quit. | Close hides with live process; protected locks, open stays open; quit flushes/backups/exits; notes retained. |
| W216 | C | Actual tray menu (WP-15) | Three test stickies | Real mouse left-click tray icon; right-click Open, Close sticky notes, Quit in separate runs. | Window opens, **Close sticky notes** closes all without deleting, Quit exits only test app. Scripted second-start surrogate alone insufficient. |
| W217 | B | Installed/portable paths and first start | Synthetic installed profile+portable home | Run reviewed installed/portable path runner; first-start new/import branches; relaunch each. | Correct isolated data root, no writes to real LocalAppData; portable marker respected; no accidental real-window handoff. |
| W218 | B | Import portable notes | R synthetic portable folder | From installed first start use actual folder picker; copy notes/backups; compare source hashes; retry unsupported/current state. | Copy-only import, original unchanged; correct destination; no overwrite of existing live notes. |
| W219 | B | Single instance/legacy guard | Isolated test app and safe old exe copy | Start second copy of same/current version; separately test legacy guard with synthetic old process. | Existing test window foregrounded, one writer; old app named/refused safely; no interaction with real processes. |
| W220 | C | Setup/upgrade/uninstall (WP-15) | Approved clean Windows user/VM, synthetic prior install | Untick Run Notaya; install per-user, test shortcuts; upgrade; uninstall; compare notes. | Correct setup/shortcuts, no admin needed, user notes kept; no overwrite of Ahmed install; unavailable clean user = not run. |
| W221 | B | Start-with-Windows setting | Clean test user, no existing Notaya Run entry | Toggle on/off through Settings; inspect exact test registry value and persistence. | One per-user value points to test exe with intended args; off removes only its value; never replace existing user's entry. |
| W222 | C | Real sign-in / start minimized (WP-15) | Approved clean user + W221 | Enable start/minimized; sign out/in; observe; disable after. | Test app starts hidden in tray when requested, opens safely via tray; no real startup entry altered. |
| W223 | C | Window frame and mouse navigation | N | Drag title bar; minimize/maximize/restore/resize; mouse buttons 4/5; compact width/high DPI. | Native frame controls work; interactive buttons not drag regions; history targets correct, no clipping. |
| W224 | C | Windows keyboard/WebView behavior (WP-15) | T+P | Switch actual Arabic/English keyboard layout between Shift+Enter lines; test Ctrl+F, input right-click and blocked reload shortcuts. | Text/line direction follows input; expected native text menu remains; no accidental browser refresh/data loss; report platform limitations. |
| W225 | B | Startup failure reporting | Disposable read-only/broken startup fixture | Use test-only unavailable data-root/WebView construction failure harness; then normal writable copy. | Clear native failure and nonzero exit, no silent empty vault; no uninstall/change of real WebView/ACL/security settings. Missing safe harness = not run. |

### 1.6 foundation, bin, history, activity and AI (inventory deltas; spec WP-01–06)

| ID | KIND | Feature | Data | Exact steps | Expected result |
|---|---|---|---|---|---|
| W226 | B | Recycle-bin groups/restore | N+D+stickies | Bin notebook/section/page/subpage/DB/row/sticky independently; inspect groups/count, restore each and parent cases. | All supported kinds listed/restored with hierarchy/content/look, no duplicate descendants, stable selection. |
| W227 | B | Permanent delete/empty bin | Disposable N+D+P | Cancel then confirm Delete forever for each supported kind; empty bin with nested children. | Cancel unchanged; confirmed target irrecoverable within app; only intended bin contents removed; live items untouched. |
| W228 | B | Bin boundaries/attachment retention | P image referenced by live/history/binned items | Bin/restore parent and database; attempt new AI mutation of binned child; let an already queued person save finish; run normal picture GC; restore. | Normal tree/search excludes binned descendants and new AI edit is refused; pending person save may land safely in bin without resurrecting it; referenced pictures survive until no retaining reference remains. |
| W229 | A | Change-signal refresh/revisions | N+D+stickies mocked second actor | Deliver newer/older/duplicate events during reads/saves and lock; simulate stale revision. | Correct store refresh order, newer edit not overwritten; events contain IDs/kinds/actor/revisions only, no note content. |
| W230 | B | Edit leases/person wins | Integrated X + active page/DB editor | Type steadily while AI tries edit; leave page and wait expiry; retry; test two writers' explicit claim/release and lock. | Person edit refused before confirmation; safe retry after lease; conflicting writer refused, leases cleared on lock/disconnect. |
| W231 | A | Block IDs/local editing | Legacy T page | Open without edit; make real edit/split/paste/drag/equation undo-redo; inspect IDs and saves. | Opening alone does not autosave; unique stable addressable IDs; pasted collisions freshened, existing IDs unchanged. |
| W232 | B | Block operations/revision guard | T+X after integration | List/read/insert/update/move/delete supported blocks with expected revision; stale/invalid paths and RTL samples. | Only addressed content changes; stale/invalid request atomic refusal; exact blocks text/direction preserved. |
| W233 | B | History snapshots/preview | T+P | Make spaced edits/title/look changes; open Version history; select versions quickly; restart. | Correct actor/time/word metadata and lazy text preview; no stale response after selection; snapshot policy/dedup holds. |
| W234 | B | Restore page version | History with two versions+P | Restore older title/body/look; inspect current editor and prior-current history entry; repeat row page. | Restored page visible, preceding current version retained; other pages/row ownership/cells not accidentally overwritten. |
| W235 | B | History retention/size cap | Synthetic dated history fixture generated by a reviewed test harness | Seed 7-day recent, hourly through 30 days, daily through 365 days, and over-500 MB soft-cap metadata cases; open test exe, trigger maintenance and inspect retained versions; use backend assertion cases as oracle. | Latest 20/page and AI-before versions up to 90 days protected, even above soft cap; other buckets/thinning/picture retention correct; no real clock manipulation. Missing safe runtime fixture route means not run. |
| W236 | B | Activity list/filter/pagination | >100 synthetic person/automation/AI entries | Open Activity; filter Who; Load older; Refresh; reopen. | Correct chronological paging/actor/counts/titles-at-time; distinct from sanitized error log. |
| W237 | B | Activity integrity | Protected activity fixture | Check the log; corrupt/delete/swap/truncate entry/head copies and verify again. | Valid chain checks; tamper detected; security-off consistent-rewrite limitation disclosed. |
| W238 | B | Undo activity change set | Mixed AI created/edited items, subsequent person edit | Undo change set; inspect restored/skipped counts and bin. | Prior values restored, new items binned; later person edits skipped; security fields unchanged. |
| W239 | B | Clear/prune activity | Activity+history fixture | Wrong then exact CLEAR in window; clear; verify; time-injected prune; inspect history. | Person-only gate; valid clear/prune anchors; activity undo records removed, page history retained. |
| W240 | B | Journal re-key/backup/restore | Fully populated S | Enable/disable security, change master, upgrade CLF1 and restore backup; Check the log and restore a version after each. | All history/activity/undo tables participate atomically; journals/pictures verified; failure rolls back complete transaction. |
| W241 | B | Action registry and permissions matrix | X, integrated final registry | Enumerate every registered action and parameters; attempt each at Off/Look/Edit/Full per pairing, including undeclared/extra fields. | Minimum level and validators match registry/spec for **every action**; declarations without dispatch not claimed implemented; never-list/security commands unavailable. |
| W242 | B | AI access Off/On and pipe lifecycle | X, integrated C5+E | Before enabling inspect pipe absence; type wrong then ALLOW; toggle Off; repeat quit/lock. | UI-only enable; listener only when allowed; Off/quit teardown; lock denies reads/writes; no leftover connection authority. |
| W243 | B | Pairing/reconnect/revoke | X two fresh bridge clients | Deny/allow pairing in window; reconnect saved credentials; remove pairing; expired/replayed handshake. | Default Look; hashed app-side secret, scoped bridge credential; denied/expired/revoked access refused; credentials not in logs. |
| W244 | B | Levels/hidden notebooks | X separate Look/Edit pairings | Set different levels in window; hide notebook; query tree/search/by-ID/relation paths; attempt permissions self-change. | Each client sees only authorized tools/data; hidden descendants inaccessible; AI cannot raise own authority. |
| W245 | B | Ask/Strong/big-change confirms | X 20- and 21-item requests | Preview; deny/allow window confirmation; Strong wrong/right phrase; switch threshold on/off; timeout. | Bound request/revision only; 21 asks and 20 not under configured rule; delete still confirms; pipe cannot answer; denial/timeout no mutation. |
| W246 | B | Live AI read/write fidelity | X+T+D | Through actual bridge read/write blocks and Markdown, query numbers/formulas/rollups, create/move/bin/restore each supported family. | Blocks byte-for-byte exact including controls/spaces; Markdown preserves supported block direction; numeric responses agree with Rust view without rounding; UI refreshes. Enumerate unsupported actions as not run/defect. |
| W247 | B | AI Stop/resume/lock concurrency | X two clients | Start bounded work; click Stop, resume; lock mid-request; try stale retry/claim conflict; quit. | Cancellation and atomicity, locked refusal, neutral person/AI attribution and disconnect cleanup; no partial or silently overwritten work. |
| W248 | B | Bridge stdio and native pipe trust | X matching app/bridge | MCP init/list/call; malformed/oversize frames/EOF; wrong-home, forged server/token, separate test homes; native transport tests. | Protocol errors safe, limits bounded, per-home identity and server-image/ACL/remote-client checks enforced; never connect to a real home. Core smoke cannot prove actual C5 ACLs. |
| W249 | B | AI exchange import/export | X exchange folder+I | Preview/import/export via supported actions; try out-of-root/traversal/reparse/data-folder target and arbitrary attachment paths. | Only explicitly allowed local exchange paths; counts/omissions/log entries correct; no arbitrary filesystem access or remote/network output. |
| W250 | C | AI window experience | Integrated X | Person inspects AI Settings, pairing, Ask/Strong, working/Stop indicator, Activity undo and errors in light/dark themes. | Plain names/permissions, readable controls and focus, no token/secret shown, person can refuse/stop; current feel preserved. |

## What already exists

This inventory is **source availability**, not a claim of recent passes. No checks below were executed in phase 1. Commands below are for the later executor with already available dependencies; no installation is authorized. Record source HEAD plus dirty-file hashes, test output, app/bridge SHA-256, fixture ID, PC/executor and UTC for each run. A historical 517/623/665/etc. count is not today's test discovery.

### dev-mock scripts in W

Browser-free commands run from repo root with the locked existing Node/dependencies. For browser rows start `npm run mock -- --configLoader native --host 127.0.0.1 --port 1421 --strictPort` unless another server is listed. Browser dependencies are **not in package.json**: use a known already-installed Playwright copy; modern scripts accept `NOTAYA_PLAYWRIGHT_MODULE`/`NOTAYA_CHROMIUM_PATH`, older `require('playwright')` scripts need existing module resolution. If unavailable, mark not run; do not install. Route artifacts/temp to the test run; several scripts currently hard-code other lane output/temp roots and need an agreed relocation before W execution. Run servers sequentially to avoid 1423 conflicts. `npx` is shorthand for existing local CLI only, never download-on-demand.

| File under `dev-mock/` | How to run | Source-confirmed coverage/limit |
|---|---|---|
| `automations-ui-check.cjs` | `node dev-mock/automations-ui-check.cjs <shots>` | Automation dialog/action editing and Button UI; mock only. |
| `block-ids-check.cjs` | `node dev-mock/block-ids-check.cjs http://127.0.0.1:1421/` | Legacy opening no autosave, IDs, real edit/equation/history and reload seam; hard-coded C4d temp root. |
| `block-ids-waits-check.cjs` | `node dev-mock/block-ids-waits-check.cjs` | Browser-free readiness/navigation retry and idempotent equation fixture contract. |
| `block-ids-waits.cjs` | Required by the previous two; **not standalone** | Readiness/equation setup helper, no additional feature suite. |
| `c2-check.cjs` | `node dev-mock/c2-check.cjs` | Browser-free production stores/listener with mock signals, revisions, save races and bin contracts. |
| `c2-ui-check.cjs` | Mock server on **1423**, then `node dev-mock/c2-ui-check.cjs http://127.0.0.1:1423` | Real editor editability and mock refresh/bin flows; hard-coded C2 artifact/temp roots. |
| `c3-check.cjs` | `node dev-mock/c3-check.cjs` | Browser-free history/activity IPC/mock contracts and restore state. |
| `c3-ui-check.cjs` | Mock server on **1424**, then `node dev-mock/c3-ui-check.cjs http://127.0.0.1:1424` | Version history/restore and Activity UI; page-options selector fix present; C3/C3u paths hard-coded. |
| `calc-check.cjs` | Compile with local `npx tsc src/features/sticky/calc.ts --ignoreConfig --outDir .calc-check --target es2020 --module commonjs --skipLibCheck --strict`; put `{"type":"commonjs"}` in generated `.calc-check/package.json`; `node dev-mock/calc-check.cjs` | Parser/math/errors/format/angle/Ans, no calculator window. This documented runner expects repo-local generated output; later lane must approve/relocate it, never create it in phase 1. |
| `calc-ui-check.cjs` | `node dev-mock/calc-ui-check.cjs` | Scientific Deg/Rad display/input regression in mock calc window. |
| `database-fixes-ui-check.cjs` | `node dev-mock/database-fixes-ui-check.cjs <shots>`; optional `MOCK_URL` | Scrollbars, undo/redo, tag/date pickers, automation dialog, chart drill and dropdown themes. |
| `database-ui-check.cjs` | `node dev-mock/database-ui-check.cjs <shots> [--only=name]` | Properties, filters/sorts/groups, formulas/relations, row pages/templates/inline DB UI. No Rust semantics proof. |
| `database-views-check.cjs` | `npx vite --config vite.views.config.ts --host 127.0.0.1`; then `node dev-mock/database-views-check.cjs <shots> [board/calendar/timeline/list/gallery/chart]` | Separate views harness **1423**, six non-table views; table covered by database UI. Pass explicit Windows shots path; default is historical Linux path. |
| `database-mock-check.mts` | `node --experimental-strip-types dev-mock/database-mock-check.mts` | Browser-free fake database query/filter/sort/group/row operations; mock correctness only. |
| `equation-identity-check.cjs` | `node dev-mock/equation-identity-check.cjs` | Actual equation input handler with ProseMirror history and DOM doubles; block identity/undo. |
| `icons-covers-ui-check.cjs` | Set PowerShell `$env:MOCK_URL='http://127.0.0.1:1421'`; `node dev-mock/icons-covers-ui-check.cjs <shots>` | Emoji/picture icons, covers, positioning, Home UI; fake dialogs/backend. |
| `lane-a-check.cjs` | `node dev-mock/lane-a-check.cjs` | Browser-free store/navigation, saved place/view/row, cache purge, stale-target fallback, export context. |
| `lane-a-contrast-check.cjs` | `node dev-mock/lane-a-contrast-check.cjs` | Static six-theme icon-token contrast; no actual rendered/accessibility acceptance. |
| `lane-a-shots.cjs` | `node dev-mock/lane-a-shots.cjs http://127.0.0.1:1421` | Six-theme icons/star, Import/Export/New notebook, compact layout, saved database view/pre-lock flush; output in `out/A-shots`; “before” is reconstructed styles, not historical app pixels. |
| `lane-a-wait.cjs` | Required by `lane-a-check`/`lane-a-shots`; **not standalone** | Awaits actual async saved-place equality; no independent suite. |
| `mermaid-ui-check.cjs` | `node dev-mock/mermaid-ui-check.cjs http://127.0.0.1:1421/ <shots>` | Diagram samples, edits and hostile-input/CSP page variants; no native exe proof. |
| `page-look-layers-check.cjs` | Set `MOCK_URL`; `node dev-mock/page-look-layers-check.cjs <shots>` | Cover/icon hit testing and Home glyph layering at 1280/1600. |
| `ribbon-ui-check.cjs` | `node dev-mock/ribbon-ui-check.cjs <shots>` | Home/Insert/Table ribbon and slash-menu coexistence. |
| `rtl-ui-check.cjs` | `node dev-mock/rtl-ui-check.cjs <shots>` | Automatic/forced block/line RTL and editor conversions; no physical Windows layout-key proof. |
| `template-keys-check.cjs` | `node dev-mock/template-keys-check.cjs` | Actual row-to-template consumer with hostile legacy JSON keys; no app. |

All `.cjs` entries, the `.mts` runner and helper distinction are included; **no `.mjs` file exists under dev-mock at W HEAD**. The separately inspected E ref adds `node dev-mock/ai-mock-check.mjs` (Node 24, browser-free fake AI command/event/contract, Off/ALLOW/pairing/level/confirm/Stop); and `node dev-mock/ai-ui-check.cjs` after mock start (AI screens, dialogs, indicator). Its optional variables are `NOTAYA_MOCK_URL`, `NOTAYA_PLAYWRIGHT_MODULE`, `NOTAYA_CHROMIUM_PATH`, `NOTAYA_AI_SHOTS`. They do not prove the Rust permission boundary; integrate/re-read before using.

### Rust tests by area

Run from repo root, with throwaway home and existing locked toolchain/cache: `cargo test --manifest-path src-tauri/Cargo.toml --lib --offline --locked`. Discover actual tests with the same command plus `-- --list`; filter by the module paths below, e.g. append `vault::history::tests`. `cargo test --manifest-path src-tauri/Cargo.toml -p notaya-bridge --offline --locked` runs the separate bridge crate. Do not manufacture a pass count from static `#[test]` occurrences. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --offline --locked` is an additional lint check, with warnings reported, not erased or equated to runtime correctness.

The following lists **every W source file containing tests**, grouped by area; filters are module prefixes (nested test module names need not be uniform). Files such as `database/engine.rs`/`relations.rs`, import/export helpers and block ID helpers also receive indirect coverage from these suites; do not imply a standalone `engine::tests` module exists.

| Area / filter prefixes | Test-bearing source files relative to `src-tauri/src/` | Coverage |
|---|---|---|
| Crypto/model | `crypto.rs`, `model.rs`, `generator.rs`, `filenames.rs` | KDF/encryption/tamper, defaults/serialized models, password strength, safe output names. Generator here is strength logic, not removed password-vault UI. |
| Vault | `vault/{header,store,mod}.rs`, `openvault.rs` | Old/current authenticated header/records, protected/open sessions, keyfile/re-key/rollback/verify, open-vault guard behavior. |
| Recovery | `backup.rs`, `restore.rs`, `import.rs` | Verified copy, independent rotations, cleanup/retry, source validation/rollback/set-aside, portable migration copy semantics. |
| Windows/state | `portable.rs`, `legacy_guard.rs`, `shell.rs`, `state.rs` | Path resolution/overrides, legacy process guard, startup settings, flush rounds/session state. Registry unit tests may fail under sandbox code 5; report, never weaken OS security. |
| Actions/change/block foundation | `actions/mod.rs`, `changes.rs`, `blocks/{mod,ops}.rs` | Named closed registry/never list/actor policy, revisions/ordered callbacks/panic isolation/edit leases, stable IDs and guarded block edits. |
| Notes/window commands | `commands/{notes,stickies,tools,vault,search_history,calculator,backup}.rs` | Notes/subpages/looks/bin, sticky CRUD, search wrapper parity, last place/security command parity, five recent searches and calc history, backup wrapper/core parity. Search parity is not comprehensive binned-search E2E coverage. |
| Database commands | `commands/{database,database_io,automation,notion_db}.rs` | Schema/rows/fine-grained mutation/relations, CSV/files/export, automation limits/triggers and Notion row/relation conversion; person/core parity. |
| Database engine | `database/{types,value,formula,regex,csv,saves,notion_compare}.rs` | Validation/value/filter/sort/group/rollup/chart through engine consumers, formulas and known Notion differences, regex limits, CSV safety, stale/small-save contracts. `print_our_results` alone is diagnostic; assertion tests determine correctness. |
| Import | `commands/import_notes.rs`; `importers/{mod,doc,docx,html,markdown,markup,palette,pdf,plan,source,zip}.rs` | Converter semantics/sanitization/direction, planning/Notion merge, local source boundaries/caps, ZIP parsing and transactional import/cancel; no native-picker proof. |
| Export | `commands/export.rs`; `export/{mod,doc,docx,html,markdown,pdf,image,zip,deflate,arabic,bidi,pdf_fonts,pdf_unicode,ttf}.rs` | Scope/inline DB/content model, writers/assets/ZIP, font subsets/shaping/bidi/limits. Rendering by Word/PDF viewers still manual. |
| Round trips | `roundtrip_tests.rs` (five tests) | `colors_code_tasks_captions_and_shelves_come_back`, `arabic_hebrew_and_english_keep_their_direction`, `every_format_scope_and_output_comes_back_in`, `links_between_exported_pages_come_back_as_links_between_the_imported_pages`, `page_link_rows_inside_a_shelf_come_back_as_rows_inside_a_shelf`. Inline DB writer tests are in `export/mod.rs`/`commands/export.rs`, not a sixth round-trip test. Format losses are explicit, not full original equality. |
| History/activity | `vault/{history,activity}.rs` | Snapshots/dedup/thinning/picture refs, hash-chain verification/prune/clear, undo skipping person edits, actor containment, journal atomicity and all-table re-key/backup. `commands/history.rs`/`activity.rs` have wrappers, no standalone test modules at W. |
| Sanitized logs | `logging/{mod,sanitize,store}.rs` | Crash/error lifecycle, string/path/content sanitization, rotation/store integrity. No actual induced Rust panic/blank-WebView E2E claim. |

Bridge test areas: `bridge/src/{main,mcp,tools,pipe}.rs` test modules cover CLI/home handling, MCP stdio parsing/size/error handling, grouped tool mapping and pipe framing/credentials/server identity. Windows-native pipe tests include ignored fake-server cases; use the C5 runner's `-NativeBridgeTests` route on the PC rather than assuming normal `cargo test` includes them. The separate C5 ref has `ai/{confirm,dispatch,mod,pairing,pipe,rules,smoke,wire}.rs`: broker/typed approval, per-action validation, owner-only commands/lifecycle, pairing credentials, native listener, permission/rule/path policy, Core smoke, wire decoding. Re-inventory final discovered tests after integration; file availability and Core do not establish production native-server security coverage. These are outside W's library suite.

Additional build checks required for phase 2: local `npx tsc --noEmit -p tsconfig.json`, `npm run build -- --configLoader native`. `npm ci` is an install and `npm audit`/`cargo audit` can need network/tools; **not run/authorized in this phase**. Claude records the later coordinator/heartbeat result on the exact integrated commit. No network audit should be silently substituted with an old cached “clean”.

### C5 PC smoke runner modes (separate cached C5 ref)

`tools/ai-smoke/{README.md,run-pc-check.ps1,tauri.smoke.conf.json}` were read through `git show AE0101/v1-6-c5:<path>` at the C5 hash above, **not present in W's files**. Invoke only from a final matching integrated worktree after re-reading. The current runner restricts `-TestRoot` to `D:\Notaya Tests\v1-6-C5-smoke` or `D:\Notaya Tests\v1-6-C5\C5d-smoke`; **it does not accept W's root**. Coordinator must approve a bounded root adaptation or run in its declared isolated C5 root and record that location. It creates/preserves a new run GUID and scopes/restores both home variables/offline mode. No fake command below is presented as executable in W today.

| Mode / exact invocation after integration | What it proves | What it does not prove |
|---|---|---|
| `powershell -NoProfile -File .\tools\ai-smoke\run-pc-check.ps1 -Mode Core` | Actual AppState/new open test vault, bridge MCP/wire/credential code over in-memory byte pipes, callback-approved pairing, hash-only storage, Look refusal/Edit raise, deny/confirm bin action, person lease before confirm, panic isolation, persistence/reopen. | Hello/Auth adapter and synthetic approval are test scaffolding; no production connection loop/native C5 server/ACL/dialogs/tray. |
| Same plus `-NativeBridgeTests` | Runs Core then `cargo test --offline -p notaya-bridge pipe::native::tests -- --include-ignored --nocapture`: real Windows fake-server transport and image checks. | These are lane D's fake-server tests, not C5's production server security. Needs native-pipe permission, PC executor. |
| `powershell -NoProfile -File .\tools\ai-smoke\run-pc-check.ps1 -Mode Live -AppExe <matching-built-Notaya.exe> -BridgeExe <matching-built-notaya-bridge.exe>` | Real paired stdio bridge↔authenticated pipe↔app, window ALLOW/pairing/default Look/Edit; deny request keeps revision; typing for up to 20 seconds acquires person lease; leave editor ≥4 seconds then allow bin request; restore in window and bridge readback. | Requires integrated E, actual UI confirmations and matching binaries adjacent; no debug port; refuses existing Notaya/Cipherleaf. Does not cover every action, hidden notebooks, all Stop/Off/lock cases, two AIs or native ACL attacks. |

Only `Core` and `Live` are accepted modes. `-NativeBridgeTests` is valid with Core only. Build prerequisites in the README: existing frontend `npm run build -- --configLoader native`, offline release bridge cargo build, and tauri no-bundle smoke config using that frontend. The runner records source identity, retains folders, bounds RPC waits at 310 seconds, closes only its own bridge, and asks for test-app tray Quit; it reports a leftover app PID instead of killing an existing process. A `CHECKPOINT` line is emitted only after that script's assertions; it is not whole-app acceptance.

### Older real-exe tools: still applicable versus historical

Read `tools/real-exe-tests/README.md`, `common.ps1`, each runner and JS entry/command flow. Node 24/global fetch+WebSocket (optional-security also `node:sqlite`), PowerShell 5.1, WebView2 and a self-contained exact exe are prerequisites. Do not execute raw `.mjs` against a running real window. The runners copy the exe with `portable.txt`, print its SHA-256, set close-to-tray false except tray tests, and use generated ephemeral test credentials; no credentials belong in the document or committed files. Their headers contain old Mohamed/E: examples; those are **not this run's paths**.

Before reuse, review **both home environment variables**: `common.ps1` inherits the caller and mainly isolates via portable marker, whereas an inherited `NOTAYA_HOME` overrides that marker. For non-path tests set both homes to the disposable copied app's **root** (the resolver appends `data`, so never pass its `data` subfolder); the installed/portable path test must clear overrides for its children and use its isolated LocalAppData. Set TEMP/TMP in the test-only shell to the approved external test root; four runners lack `-Root`. Existing cleanup deletes evidence and needs a preservation adjustment. These are adaptation prerequisites, not edits authorized in phase 1. Confirm pass/fail output and cleanup lines as some runners print summaries rather than robustly failing every shell exit.

| Runner / JS peer | Later command from repo root (explicit reviewed exe path) | Applicability and coverage |
|---|---|---|
| `run-optionalsecurity.ps1` / `optionalsecurity.mjs` | `powershell -NoProfile -File .\tools\real-exe-tests\run-optionalsecurity.ps1 -Exe <test-exe>` | **Relevant, not runtime revalidated**. Five p1–p5 phases: open notes/restart, weak-password refusal, security on/off, failed credentials, cleanup/keep unprotected copies, protected/open restarts and restore downgrade. Current file uses `vault.nota`; historical 52/52 is not current evidence. Debug 9335. |
| `run-restoreflow.ps1` / `restoreflow.mjs` | `powershell -NoProfile -File .\tools\real-exe-tests\run-restoreflow.ps1 -Exe <test-exe>` | **Relevant**. Locked list/newest order, unpicked absolute/UNC/traversal/wrong-password refusals, live unchanged after failure, listed locked/unlocked restore and undo file. Does not exercise native file-picker selection or rename-crash window. Debug 9335. |
| `run-lockflow.ps1` / `lockflow.mjs` | `powershell -NoProfile -File .\tools\real-exe-tests\run-lockflow.ps1 -Exe <test-exe> -Scenarios page,sticky,close` | **Relevant**. Pending edit survives explicit lock/page, sticky and close+restart. Expected true: page for page/close, sticky for sticky; other probe false by design. Not idle/Win+L/sleep/removal. Debug 9335. |
| `run-installed.ps1` / `installed.mjs` | `powershell -NoProfile -File .\tools\real-exe-tests\run-installed.ps1 -Exe <test-exe>` | **Relevant with path-isolation preflight**. Installed first screen/temp LocalAppData/restart versus portable marker/data; portable refuses import. Not actual NSIS setup or portable-import native picker. Debug 9335. |
| `run-tray.ps1` / `tray.mjs` | `powershell -NoProfile -File .\tools\real-exe-tests\run-tray.ps1 -Exe <test-exe> -Root 'D:\Notaya Tests\v1-6-W'` | **Relevant with clean-user/startup-entry gate**. Debug **9337**; open/protected close-to-tray, pending-save/lock, second-start foregrounding and one process, start-with-Windows on/off. Refuses an existing Notaya Run value. Does **not click actual tray menu** or test real sign-in. |
| `common.ps1` | Dot-sourced by each runner; not standalone | Exe checks/copy, cryptographic random passphrase, scoped WebView flags, free/listener checks, test-process cleanup; inherited homes and evidence deletion are limitations above. |
| Passwords `testA`, `testB`, `testC` (`c1/c2/c2q`), `testE`; also `testD1` | **No runnable path/command found in tracked W tree; do not reconstruct a command** | **Historical, obsolete for current Notaya**: A24 removed KeePassXC, password UI/commands/backups; acceptance X1/X2/X10/X16 are historical. EX-17/18/21/22 report old counts, not available present scripts. Exact subcase internals unavailable. Current master-password/security tests are the optional-security/vault checks above, not these removed password-manager tests. Never install/launch KeePassXC to revive them. |

Other available tools: `tools/mock-screenshots/shot.mjs` and `act.mjs` are generic screenshot/CDP action helpers, **not feature assertion suites**. Follow their README with loopback mock 1421 and an isolated already-installed headless Edge on 9333; e.g. `node tools/mock-screenshots/shot.mjs <out.png> http://127.0.0.1:1421/ "" 900 1360 860`, or `node tools/mock-screenshots/act.mjs http://127.0.0.1:1421/ <steps.json>` (VW/VH optional). Close the test browser/server and verify 9333/1421 free. Never inspect a real browser profile.

`packaging/test-bridge-packaging.ps1` tests release packaging using **inert fixture exes**; `python -I packaging/test-bridge-stdio.py` tests actual built bridge stdio/invalid/home handling without a vault/server. Both currently require exactly `NOTAYA_HOME='D:\Notaya Tests\v1-6-D'`; stdio also requires `CARGO_TARGET_DIR` and its debug bridge. They cannot run unmodified under W's allowed home. Packaging/runtime/native server proof remain distinct. Usage-ledger/font-generation/build helpers are operational tools, not whole-app test suites.

## Gaps

These are **coverage gaps**, distinct from inventory features that do not exist. Adding A/B/C rows does not create automation. No existing script covers the whole app end to end on the final 1.6 commit.

| Feature/check IDs | Automation present | Remaining gap for phase 2 |
|---|---|---|
| Organisation/navigation W001–020 | Notes/core tests and lane-A logic/screenshots | Full live notebook/section/page/DB hierarchy/move/restart matrix; real drag/mouse/high-DPI/keyboard accessibility; screen-reader acceptance. |
| Editor W021–050 | Ribbon/RTL/math/Mermaid/look mock scripts; import/export and core tests | No complete real-exe block-by-block authoring suite; actual native image picker/paste/drop, block drag, external opener stub and dormant-content preservation need execution/new harness. |
| Databases W051–114 | Extensive Rust engine/command and mock UI/view harnesses | No complete seven-view/23-property live persistence suite; default/recurring templates, file open/save cleanup, reminder clocks, automation catch-up/limits, row/template/schema concurrency and size-boundary cases need explicit correlation to rows. |
| Stickies/calculator W115–131 | Calc engine/UI, sticky/calc Rust, lockflow sticky | Full desktop geometry/on-top/multiple windows/selection-bin/history-cap/clipboard/RTL live tests absent as one suite; OS pixels need person. |
| Search W132–137 | Wrapper parity/history tests and incidental mock use | Dedicated all-kinds/descendant bin exclusion, locked hits and Unicode search E2E absent; native Ctrl+F is manual observation. |
| Import/export W138–167 | Converter, limits, Notion and five round-trip Rust suites; historical PC reports | No committed comprehensive native-dialog runners for all scopes/formats; full Notion merge/Hubs file-viewer round and documented Obsidian losses; Word/PDF Arabic/Hebrew/glyph/layout require person. |
| Security W168–185 | Vault/state/Rust and optional-security/lockflow | Every populated 1.6 table re-key in real app, keyfile dialogs, screenshot-on actual capture, idle/Win+L/sleep/drive removal and safe fault-injection runtime harness; no current final-build evidence. |
| Recovery W186–198 | Backup/restore/import tests and real-exe restore/installed scripts | Native picked-file restore, actual crash boundary, independent hourly/backoff pools and synthetic old CLF1/1.5.1 fixtures need available safe sources/execution; no tracked all-in-one real-exe runner. |
| Settings/themes W199–214 | Six-theme static tokens/screenshots; appearance mocks | Rendered accessibility/material/capture/system behavior and persist-under-lock across all settings, sanitized real error cases; static contrast cannot pass a visual row. |
| Windows/WP-15 W215–225, W153 | Tray/installed scripts and core OS helpers | Real tray **Close sticky notes**, real keyboard-layout shortcut, Notion-import Word/PDF, clean-user setup, actual sign-in remain mandatory; startup fault fixture/VM may be unavailable. |
| 1.6 W226–240 | C2/C3 contracts/UI, C4 IDs, Rust revisions/bin/journals | Complete multiwindow/bin/GC/history/undo/re-key restore matrix on integrated exe; time/fault harness for retention and failed transaction boundaries. |
| AI W241–250 | C1/bridge tests in W; C5 Core/Live + E mocks in separate cached refs | Integration prerequisite; every-action×level/hidden-data/path/ACL/native lifecycle, two AIs, exact text/numeric parity, Stop/lock/confirmation race cases exceed smoke scope. Never substitute synthetic Core approval for real window approval. |

No test is requested for missing future-only features: native Notaya find/replace, page tabs, customizable menus/shortcuts/top bar, code-language picker, dashboards/slicers, corkboard/strings, whiteboard, Notebook mode, Excel import/export, new tour/help UI. Historical inventory roadmap version labels are superseded by the current version plan. This test plan does not authorize implementing them.

## Performance measurements for phase 3

Use same reviewed release exe/hash, PC, power state, viewport/theme, security mode and fixture manifest before/after a bounded optimisation. No pass threshold is invented: collect baseline, repeat at least five samples, report median and range (p95 only with enough samples), and submit regressions/target proposals to Claude/Ahmed. Preserve correctness/visual criteria first; mock timing is not release timing. No measurements run in phase 1.

| Measurement | Repeatable workload and start/stop | Record |
|---|---|---|
| Startup | Fresh open home, populated open home, protected home (separate password-entry time); timestamp process start→first interactive window and→last-place/editor usable. Separate first launch from warm reopen; do not flush OS caches/change security settings. | Exe hash, data size, cold/warm definition, five durations; avoid calling script's fixed 6-second connection wait startup time. |
| Large vault | Synthetic 1k then 10k pages with known token distribution and attachments; open/unlock, search, save, backup/restore and lock. Increase gradually; stop on failure/resource pressure, preserve outputs. | Vault/item/attachment/history sizes, operation latency, result correctness, errors and memory; cost of Argon2/unlock separate from hydration. |
| Long pages | 1k then 10k blocks (mixed text/lists/table/images/code/RTL), open→editable, type→saved, scroll/selection, undo and export. | Input-to-paint and save completion, visible stalls, document/codepoint equality; no format/ID loss traded for speed. |
| Large databases | 1k then 10k/50k generated rows (within actual source caps); query/filter/sort/formula/relation/rollup; open every view, then table scroll/board/calendar/timeline/chart and edit→recompute. | Per-view query/render/edit times, exact totals/row counts, timeline scaling limitation; separate mock paint from Rust query and full exe end to end. |
| Memory | Baseline open→page/DB→five stickies/two calculators→import/export→lock→reopen cycle, repeat 10 times; sample idle after same settling period and during peak. | Whole **test process tree** working set/private bytes and peak, WebView child PIDs, vault size/history caps and cycle trend; do not claim zeroisation from working-set drop. |

## Result sheet and hand-back contract

Phase 2 records one row per numbered ID in a separate coordinator-owned execution artifact: `ID | exact commit/dirty hashes | exe+bridge hashes | fixture/home | KIND/executor | UTC | passed/failed/not run | observed vs expected | evidence path | defect or blocking reason`. If a row has parameter variants, keep each observation: it passes only when all required variants pass. Report unsupported/inaccessible cases as not run or a named limitation, never quietly omit them. Summaries count each ID once by its assigned KIND. Source inventory above does not supply runtime evidence.

Phase-1 verification is document-only: unique contiguous IDs, required fields/KINDs, counts by area/KIND, local source/command paths or explicitly labelled absent/separate refs, no changed app code, one W claims row and preservation of pre-existing inventory/staged work. No build/unit/browser/real-exe/audit/performance test was run. Claude reviews this produced plan, integrates C5/E, arranges the PC/manual run and owns the records/commit. Stop before phase 2.

<!-- WHOLE_APP_ROUTING_START -->
## Phase-2 slice 1: execution routing and scaffolding

Dated 2026-11-09T03:24:24.999Z (system clock), Ahmed (AE0101), Gaming-PC. The current phase-2 instruction authorizes only tools/whole-app/* and this plan/results sheet, no app edits or execution here. Inspected W HEAD 04fcb96; Claude's integration of main 5077183 is staged, with no unresolved conflicts at orientation. The phase-1 facts above remain historical. No fetch, network, installs, Git writes, browser or real exe ran in this scaffolding session. Claude owns commits, authority/session/usage records and runtime execution. User-message time and quota receipts were not supplied.

The PC runners use fresh GUID homes in D:\Notaya Tests\whole-app and unique archives in D:\Notaya-lanes\out\whole-app. They write fixed latest JSON files there. Both home variables and child TEMP/TMP/profile are isolated. Installed/portable path probes deliberately clear both home overrides for the app child and use only their disposable LocalAppData; no real profile or installation is used. All processes are owned by private kill-on-close Windows jobs. Port availability, teardown and unset WebView arguments are recorded (FI-01). Existing Notaya/Cipherleaf makes B/performance refuse launch. Dependencies/toolchain must already exist; Cargo is offline/locked. No downloads are attempted.

Group A inventories all executable dev-mock CJS/MJS/MTS suites, including integrated AI scripts, and identifies the two consumer-only helpers. Its adapter records original hashes, infrastructure substitutions and executed CJS sources; selectors/assertions stay unchanged. It supplies the installed Playwright/Chrome, relocates output/temp and loopback ports, and transpiles the calculator into external temp. The views harness uses its own free port. Rust discovery precedes every listed module-prefix area; zero discovery/passing tests is not run. Bridge normal/native ignored tests and integrated AI modules are separate supporting suites. Legacy suite-to-ID mapping is supporting evidence; only W126 currently has an explicit complete A mapping. Most existing suites do not cover every variant of their plan row. Script success cannot silently pass the other A rows.

Group B slice 1 runs optional-security p1-p5, restoreflow, independent page/sticky/close lockflow fixtures, and isolated installed/portable first/reopen probes. It adds exit/assertion guards to copies of legacy JS, bounds every phase, preserves fixtures and logs, and performs actual quit/reopen/readback. It does not run the historical password-manager tests (sources absent, feature removed). Existing tray script mixes tray checks with actual startup-registry changes and is deferred to a clean-user/session slice. No AI consent is answered through injected IPC.

Performance defaults to blank startup plus 1,000 fake notes, a 1,000-block mixed text/list/code/RTL page and a 1,000-row database through the real app commands; five warm reopen/probe samples, median/range, first-window time, seven backend view-query times and sampled process-tree memory. Increase sizes explicitly after inspecting smaller fixtures. No performance threshold or zeroisation claim; protected unlock, richer long-page blocks, full view rendering, backup/restore and ten multiwindow cycles remain later measurements.

Routing below covers all 166 B checks. **partial unattended** means the named suite runs but does not complete every plan variant. **later unattended harness** means scriptable work for a later slice, currently not run. **not unattended** means this runner cannot complete it without actual native-dialog/person/OS/AI-consent work or a reviewed fixture/fault/transport harness that it does not supply; this is a current execution limitation, not a claim automation can never exist. All unimplemented checks stay visible as not run in JSON/results. C's 27 checks still need a person.

| ID | Feature | Slice-1 route | Suite or remaining work |
|---|---|---|---|
| W001 | First start and security-off notes | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W003 | Notebook ordering | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W004 | Sections | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W005 | Pages and title editing | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W006 | Nested sub-pages | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W007 | Page reorder/move | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W008 | Database move/drag in tree (1.6 A) | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W014 | Reopen last place (1.6 A) | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W021 | Autosave and save state | partial unattended | lock-page, close-page |
| W036 | Picture acquisition/validation | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W039 | Web link validation | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W047 | Editor undo/redo and exact text | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W048 | Page icon/cover | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W050 | Dormant columns/comments compatibility | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W051 | Database creation/first-use backup | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W063 | Text property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W064 | Title property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W065 | Number property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W066 | Select property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W067 | Multi-select property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W068 | Status property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W069 | Date property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W070 | Person property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W071 | Files property | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W072 | Checkbox property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W073 | URL property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W074 | Email property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W075 | Phone property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W076 | Formula property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W077 | Relation property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W078 | Rollup property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W079 | Created time property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W080 | Created by property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W081 | Last edited time property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W082 | Last edited by property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W083 | ID property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W084 | Place property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W085 | Button property | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W086 | Property schema lifecycle | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W087 | Filters | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W088 | Sorts | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W089 | Groups/subgroups | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W090 | Footer calculations | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W091 | Row creation/duplicate/reorder | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W093 | Row templates | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W094 | Template legacy key safety | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W095 | Default row template | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W096 | Recurring row templates | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W097 | Sub-items | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W098 | Dependencies | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W099 | Database lock | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W101 | Database undo/redo | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W102 | Automation: row added | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W103 | Automation: property changed | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W104 | Automation: date arrives | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W105 | Automation limits/trash | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W106 | Date reminders | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W107 | CSV database creation/import | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W108 | CSV export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W109 | Database Markdown export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W110 | Database HTML export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W111 | Database Word export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W112 | Database PDF export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W113 | Database validation/size caps | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W114 | Fine-grained database saves (1.6 C4) | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W115 | Sticky creation/content | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W117 | Sticky styles | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W118 | Sticky geometry/on-top | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W120 | Sticky delete/bulk bin | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W121 | Open all/Close all | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W122 | Sticky save-before-lock | partial unattended | lock-sticky |
| W123 | Calculator windows | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W127 | Shared calculation history | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W128 | History cap/clear | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W129 | Calculator Copy | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W132 | Global search | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W133 | Binned items excluded | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W135 | Recent searches | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W136 | Locked search | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W138 | Markdown importer | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W139 | Plain-text importer | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W140 | HTML importer | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W141 | Word importer | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W142 | PDF importer | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W143 | Folder/mixed-files import | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W144 | ZIP/nested Notion parts | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W145 | Notion Markdown+CSV databases | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W146 | Notion HTML plus CSV merge | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W147 | Obsidian/other Markdown folders | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W149 | Import progress/cancel | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W150 | Import bounds/hostile source | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W151 | Import report/original integrity | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W152 | Arabic/Hebrew importer fidelity | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W154 | Export scope/Save dialog | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W155 | Markdown export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W156 | HTML export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W157 | Word export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W158 | PDF export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W159 | Separate files/ZIP | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W160 | Inline database export fix (1.6 B) | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W161 | Database scope in notebook/section export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W162 | MD/HTML/DOCX round trips | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W163 | PDF round trip | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W164 | Internal-link round trip | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W165 | Export validation/failure | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W166 | Arabic/Hebrew round trips | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W168 | Security off | partial unattended | optional-security |
| W169 | Enable security/master strength | partial unattended | optional-security |
| W170 | Disable security | partial unattended | optional-security |
| W171 | Unlock and guessing delay | partial unattended | optional-security |
| W172 | Explicit lock | partial unattended | lock-page, lock-sticky |
| W177 | Lock on minimize | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W178 | Keyfile create/pick/require | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W179 | Change master/re-key | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W180 | Re-key atomic failure | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W181 | Old unprotected-copy cleanup | partial unattended | optional-security |
| W183 | Tamper/truncation detection | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W184 | Lock flush timeout/failure | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W185 | UI/IPC/security boundary | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W186 | Manual backup | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W187 | Lock/quit backup rotation | partial unattended | optional-security, restore-flow |
| W188 | Hourly open-vault backup | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W189 | Predelete safety backup | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W190 | Backup failure/retry | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W191 | Backup list while locked/unlocked | partial unattended | restore-flow |
| W192 | Restore from listed backup | partial unattended | restore-flow |
| W193 | Native picked-file restore | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W194 | Restore validation/security downgrade | partial unattended | optional-security, restore-flow |
| W195 | Restore rename/crash rollback | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W196 | Start over/recover | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W197 | Cipherleaf `.clf` open/upgrade | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W198 | 1.5.1 compatibility (spec G3) | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W206 | Accent/font/text size | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W207 | Width/density/layout settings | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W209 | Lock backdrop | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W210 | Default sticky colour | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W211 | Error log | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W213 | Settings persistence/cache hygiene | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W215 | Close to tray / quit | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W217 | Installed/portable paths and first start | partial unattended | installed-paths |
| W218 | Import portable notes | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W219 | Single instance/legacy guard | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W221 | Start-with-Windows setting | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W225 | Startup failure reporting | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W226 | Recycle-bin groups/restore | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W227 | Permanent delete/empty bin | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W228 | Bin boundaries/attachment retention | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W230 | Edit leases/person wins | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W232 | Block operations/revision guard | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W233 | History snapshots/preview | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W234 | Restore page version | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W235 | History retention/size cap | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W236 | Activity list/filter/pagination | later unattended harness | Can be scripted on disposable data; no complete assertion driver for this row in slice 1. |
| W237 | Activity integrity | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W238 | Undo activity change set | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W239 | Clear/prune activity | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W240 | Journal re-key/backup/restore | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W241 | Action registry and permissions matrix | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W242 | AI access Off/On and pipe lifecycle | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W243 | Pairing/reconnect/revoke | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W244 | Levels/hidden notebooks | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W245 | Ask/Strong/big-change confirms | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W246 | Live AI read/write fidelity | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W247 | AI Stop/resume/lock concurrency | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W248 | Bridge stdio and native pipe trust | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
| W249 | AI exchange import/export | not unattended | Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps. |
<!-- WHOLE_APP_ROUTING_END -->
