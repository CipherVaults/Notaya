# Database views: board, calendar, timeline, list, gallery, chart (A35, V1.5.0)

Helper notes for the "database views" work. Result state: **produced** and checked in a harness (the real shell and
the Rust backend were not used). Not integrated, not accepted.

## What exists

All in `src/features/database/views/`. Each view is a default-exported component taking `ViewProps`
(`src/features/database/viewTypes.ts`). No new dependencies (`@dnd-kit` for drag and drop, `lucide-react` icons).

| File | What it does |
|---|---|
| `shared.tsx` | Option-color chips, date helpers (day numbers, ISO cells, Monday weeks), value display, group patches (`groupPatch`), small controls (`PropSelect`, `NewTitleInput`, `EmptyState`). Local copies; swap to the shell's `cells/*` later if wanted. |
| `BoardView.tsx` | Columns from `result.groups`; prompt to pick "Group by" when `view.group` is null; drag between columns (`setCells`) and inside a column (`reorderRows`, only when the view has no sorts); swimlanes from `view.subGroup`; collapse / hide / show columns (`GroupBy.collapsed`, `hidden`); "+ New" per column with the column's value; card size and cover (first image of a files property); keyboard (Enter opens, Space starts a keyboard drag). |
| `CalendarView.tsx` | Month and week modes, Monday first, prev / next / today, ranges as bars across days, "N more" popover, click an empty day to type a title and add a row with that date, drag a bar to another day (time and length kept), side list of rows without a date (drag onto a day to schedule), status/select color on bars, property picker for `dateProp`. The grid stays left-to-right in right-to-left pages; titles use `dir="auto"`. |
| `TimelineView.tsx` | Zoom day / week / month / quarter / year, sticky title column, today line, bars from `dateProp` to `endProp` (or the end of the date range), drag to move, drag the edges to resize, Arrow keys move a bar (Shift+Arrow moves the end), dependency arrows (`view.showDeps` + `db.blockedByProp`), sub-item nesting with collapse (`view.showSubItems` + `db.subParentProp`), groups, "No dates" footer with an inline "Set dates" editor, "+ New" (today). |
| `ListView.tsx` | One line per row, properties on the far side, inline checkbox toggle, groups with collapse and "+ New", Arrow keys move focus, Enter opens. |
| `GalleryView.tsx` | Card grid (size small / medium / large, `cardFit`, cover from a files property), groups as sections, "+ New" card. |
| `ChartView.tsx` | Calls `api.chartData`, draws hand-written SVG: bar (vertical / horizontal, grouped / stacked), line (several series), donut with center total and legend, number tile. Settings panel (type, x property, date bucket, number bucket, order, series property, value calculation and property, color, title, horizontal, stacked, show values, hide empty) saved with `updateView({ chart })`. Nice tick values (integer ticks for counts), tooltips, legend, `role="img"` with a text summary, "Show as table" fallback, empty / loading / error (with retry) states. |

## Harness and checks

- `vite.views.config.ts`, `dev-mock/database-views-harness.html|tsx`, `dev-mock/views-core.ts`: a tiny stand-in shell
  (grouping, `setCells`, `createRow`, `reorderRows`, `updateView`, fake `chart_data`) with 20 sample rows (Arabic titles,
  ranges, times, a dependency, sub-items, covers). Every call a view makes is logged in `window.__h.calls`.
  Query: `?view=board|calendar|timeline|list|gallery|chart&theme=sunrise-dark&rtl=1&data=empty|many` and a few more (see the file).
- `dev-mock/database-views-check.cjs`: Playwright script, 115 assertions (drag a card between columns gives
  `setCells(row, {status: id})`, calendar day click gives `createRow` with the date, timeline drag / resize change the dates,
  chart bar / slice / point counts, empty and error states, locked database still edits). Run:
  `npx vite --config vite.views.config.ts` then `node dev-mock/database-views-check.cjs [folder] [view]`.
- Screenshots (looked at): `/mnt/project-files/notaya-build/v150-db/views-*.png` (light, dark, right-to-left).
- Also run on the final source: `npx tsc --noEmit -p tsconfig.json` clean, `npm run build` ok.

## Not done / limits

- Not run inside the real shell or the real app: the shell's `DatabaseView` / `cells/*` were not in the merged branch
  (only the mock backend, `dev-mock/database.ts`), so the views were tested only in the harness. Not run against the Rust
  backend. Real-exe test not run.
- Gallery "page content preview" cover (`cardCover: "page"`): rows do not carry page text, so it shows a placeholder
  (the row icon). Board cards skip the page cover for the same reason.
- Board: a multi-select move removes the source option and adds the target, other options stay (the brief said "replace").
  Group properties that cannot be changed by dropping (date, number, formula...) only allow reordering inside a column.
- Board manual order is saved only when the view has no sorts. `reorderRows` gets every row id (board order first).
- Calendar: week mode is seven day columns (no hour grid). Timed rows show the time. Month view shows 3 lanes per week.
- Timeline: no virtual scrolling (fine below ~500 rows, 220 rows rendered in the check). No drag to reorder rows, no
  creating dependencies by dragging. The end property is chosen in the header; time-of-day is kept when moving but the
  bars work in whole days.
- List: no manual drag ordering. Relation values show titles only for rows in the view (otherwise "N linked").
- Chart: the colors are the option color names the backend sends (blue, orange...) mapped to fixed hues.
- Tailwind classes in new files are picked up when the vite server is (re)started; restart it after adding a view file.
- A view's own state (calendar month, collapsed sub-items, open "N more") is not saved.
