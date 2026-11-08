# Notaya 1.5.1 README screenshots

Ten real screenshots of the released **Notaya 1.5.1** portable build (Windows), in the Sunrise light theme (the default). All content is made-up demo content (a Lisbon trip plan, a reading list, a small habit tracker). Each PNG is 1400 x 900.

## Source of the app

- Release: v1.5.1 of CipherVaults/Notaya, asset `Notaya-V1.5.1-portable.zip` (downloaded with `gh`, not built).
- Zip SHA-256 (verified): `4576D5039F59708D219F9E365FF9903E5470A7A0471F324839959C811A3A28E7`
- `Notaya.exe`: file version 1.5.1, SHA-256 `BE7426E9D6B5E1E81CA02779DDE7CEE42EE676DC006FCCD64AE3907ED1EC5ACA` (matches the value written in the README inside the zip; build commit `95ca8b203403fae701e6f60521419329b30a7145`).
- Data: a fresh, empty throwaway vault (`NOTAYA_HOME` set for that one process only), security off. No real vault or notes were used.

## When

Taken on 2026-11-08, about 17:08 to 17:17 UTC (system clock; the last check was 17:16:07 UTC).

## Method

The app was started as a throwaway process with a WebView2 debug port on 127.0.0.1:9333. The demo notebooks, pages, two databases and four sticky notes were created through the app's own commands, then each screen was opened in the app and captured with `Page.captureScreenshot` at a 1400 x 900 viewport. Afterwards the app was quit from inside, the debug port was closed, no Notaya process was left, and the WebView2 debug environment variable was not left set.

## The ten files

| File | Shows |
| --- | --- |
| `01-home.png` | Home: favorites, recent pages, notebooks in the sidebar |
| `02-editor.png` | A page: ribbon, headings, checklist, callout, table, equation |
| `03-table.png` | Reading list database, Table view |
| `04-board.png` | The same database as a Board (grouped by Status) |
| `05-chart.png` | The same database as a Chart view (stacked bar, books by status) |
| `06-stickies.png` | Sticky notes board with four colored notes and the Calculator window (a composite, see note 1) |
| `07-search.png` | Ctrl+K search ("lisbon": a page, a database row and a sticky note) |
| `08-themes.png` | Settings > Appearance, showing the themes |
| `09-arabic.png` | An Arabic page with right-to-left text mixed with English, and an Arabic checklist |
| `10-mermaid.png` | A Mermaid diagram block (flowchart drawn from text) |

## Notes and what was not captured

1. **`06-stickies.png` is a composite** of two real captures from the same session: the Sticky notes board (main window) and the Calculator window, which is a separate window of its own, placed over the board at 70% size. Nothing else was edited in any image.
2. These are captures of the app's content area at 1400 x 900 (the app's own title bar and sidebar are included). They are not full-desktop captures: no Windows taskbar, no desktop, no drop shadow around the main window.
3. Not captured: the lock screen and master-password screens (security stays off in this vault), the dark themes and accent colors in use (only the theme picker in 08), the Gallery, Calendar, Timeline and List database views, the installer, and the sticky notes as separate floating windows.
4. The chart legend in 05 shows an empty "No Tags" entry. That is how the app draws it for this data; it was left as is.
5. Home (01) was shot first. After it, one demo page (Lisbon trip plan) had its icon and cover removed so the whole page, including the equation, fits in one frame (02); the icon was put back afterwards, the cover was not. So the Lisbon trip plan card in 01 has its ocean cover and the page in 02 has none.
