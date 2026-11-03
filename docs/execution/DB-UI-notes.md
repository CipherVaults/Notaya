# Database UI notes (A35 / V1.5.0, helper: database UI)

Result state: **produced** and checked in the mock (see "How it was tested"). Not integrated, not accepted.

Branch `worktree-agent-a73c55ee8ce2d870e`, based on 0423733. Nothing pushed, no PR.

## What exists

Mock backend (development only)
- `dev-mock/database.ts` and hooks in `dev-mock/core.ts`: every database command of `src/lib/ipc.ts` answers in a plain browser (`npm run mock`). `?db=1` seeds the Tasks and Projects databases. The mock now also keeps saved page content, so an inline database block survives going to another page and back.
- `dev-mock/database-mock-check.mts`: checks the mock itself.
- `dev-mock/database-ui-check.cjs`: the Playwright script (see below).

Shell and registry
- `src/features/database/DatabaseView.tsx`: toolbar (view tabs, Filter, Sort, Group, Properties, search, view settings, database menu, New with templates), header with icon, title and description, lock handling, compact mode for use inside pages. Props: `{databaseId, viewId?, onViewChange?, compact?}`.
- `src/features/database/views/registry.tsx`: finds `./*View.tsx` with `import.meta.glob`, lazy-loads them, and shows a "coming" notice for a view kind whose file is missing. Only `TableView.tsx` is mine.
- `views/TableView.tsx`: sticky header, frozen title column, resize, drag to reorder columns, row buttons (open, peek, duplicate, trash with undo), groups, footers (calculation per column), wrap, row numbers, keyboard navigation.
- `dbStore.ts`: one shared copy per database, 700 ms debounced saves, serialized save chains, `useSaveCount`, flush on exit.

Cells and editors (`cells/*`)
- Display and editors for text, number, select, multi-select, status, date (single and range), checkbox, URL, email, phone, people, files, relation, rollup, formula, created/edited time, unique id. Formula editor shows a live preview (about 1 s behind, because of the debounce) and the error text.

Panels (`panels/*`)
- Property menu and the "add property" picker (all kinds, relation with optional back side), Filter, Sort, Group, Properties (show, hide, reorder), View settings (name, open rows in, frozen columns, wrap, row numbers, chart settings), Templates dialog.

Rows as pages
- `RowProperties`, breadcrumb on a row page, side peek and center peek (`RowPeek`), "Open as full page" option per view.

App integration
- Sidebar: databases listed beside pages, favorites, "New database…" (bottom button, section menu, "Add a database inside…" on a page), move to section, trash with undo, trash count. `CommandPalette` finds databases and rows. `store/app.ts`: `activeKind`, `openDatabase`, `openRow`. A first-database notice.
- Inline: `src/features/notes/editor/DatabaseBlock.tsx` (TipTap node `databaseBlock`, attrs `databaseId`, `viewId`), `inline.ts`, `DatabasePicker.tsx`, slash entries "Database (inline)" and "Linked view of database…". Removing the block keeps the database.
- `src/styles/app.css`: tokens, plus rules that stop the editor's table styles leaking into the block.

RTL
- Cell text uses `dir="auto"` and logical CSS. Verified with an Arabic row title, and with the whole page forced to `dir=rtl` (mirrored layout).
## How it was tested

- `npx tsc --noEmit -p tsconfig.json`: clean, on the last commit plus the check script change.
- `npm run build`: ok. One warning: `INEFFECTIVE_DYNAMIC_IMPORT` (DatabaseView is imported statically by `DatabasePage` and dynamically by `DatabaseBlock`; harmless, only the chunk split is lost).
- `node dev-mock/database-ui-check.cjs <outdir>` against `npm run mock` (Playwright, Chromium, deviceScaleFactor 1): prints "ALL OK". It covers the seeded table, New database dialog, every property kind, typed values for each kind, formulas (valid and invalid), sort, filter, group, relation and rollup, row pages with body save and breadcrumb, side and center peek, duplicate, trash and undo, templates, inline database and linked view (also after navigating away and back), footer calculation, add view, lock and unlock, palette search, database trash, and the looks (dark, 900 px, Arabic, mirrored) with a check that the page does not scroll sideways.
- Screenshots: `/mnt/project-files/notaya-build/v150-db/ui-*.png` (looked at one by one; problems found were fixed).

## Not run

- `cargo` (any), the Windows exe, a real vault, the real backend commands. Everything above ran against the mock, which was written from `src/lib/ipc.ts` and the lead's Rust sources, so a difference between the mock and the real backend is possible.
- The other helper's views (board, calendar, timeline, list, gallery, chart) were not tested here; in this branch alone they show the "coming" notice.

## Not done / known gaps

- `get_row` command: the UI falls back to `queryView` to read a single row. A real command would be cheaper.
- `attach_file` is called with headers `x-row-id`, `x-prop-id`, `x-file-name`; check that the Rust side reads these.
- No API lists trashed rows, so only the toast Undo restores a row (databases can be restored from the trash view).
- Files cells show names only, no image thumbnails.
- Chart settings are edited in my View settings panel; the chart view itself belongs to the other helper.
- Footers do not reflect the client-side search box. Grouped tables show footers only when a calculation is set.
- Duplicating a property copies its settings, not its values.
- The ribbon / Insert menu has no database entry; only the "/" menu.
- The view tab strip scrolls sideways when there are many views; the last tab can be cut off at the edge.
- Formula preview lags about a second.
