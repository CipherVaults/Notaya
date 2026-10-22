# A12: Notion's "Turn into" list and selection toolbar, pages inside pages, fonts and sizes, equations

**Status:** applied under the owner's instruction of 2026-10-22 (2026-10-22 17:29:57 UTC, from the transcript; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. **Partly superseded by [A13](A13-calculator-and-wording.md):** columns, toggle headings, heading 4, "Page in", comments and the emoji button can no longer be created, and the menus and toolbar were reworded and redrawn (older pages still show those blocks). Equations were added after the owner approved the KaTeX library ("Yes, add KaTeX (exact version, stable)…").

## What the owner asked for

Two Notion screenshots (the "Turn into" list and the selection toolbar) with: "make those in the notes and add changing the fonts change size". Then, after being asked about the new dependency: "Yes, add KaTeX (exact version, stable) and build block equations and the inline equation button."

## What changed

| Item in the screenshots | What it does now |
|---|---|
| Turn into list: Text, Heading 1–4, Bulleted, Numbered, To-do, Toggle list, Code, Quote, Callout | Same list, with a check mark on the current type. Reached from the selection toolbar, from the block grip menu ("Turn into"), and mostly from the `/` menu. Heading 4 is new. |
| Toggle heading 1–4 | A toggle whose first line is a heading; click the arrow to hide or show the rest. |
| 2, 3, 4, 5 columns | The block becomes the first column of that many side-by-side columns. |
| Page | Creates a page inside the current page and leaves a link to it where the block was (click to open). |
| Page in | Moves the block to the end of another page you pick from a searchable list. |
| Block equation | Turns the block into an equation; the block's text becomes its TeX. Click an equation to edit its TeX in a small box with a live preview; Enter finishes. |
| Toolbar: Heading 3 ▸ row | Opens the Turn into list. |
| Toolbar: A, B, I, U, clear formatting | Text color and highlight, bold, italic, underline, clear formatting. |
| Toolbar: link, strikethrough, code | Link, strikethrough, inline code. |
| Toolbar: √x | Turns the selected text into an inline TeX equation (spaces around the word stay outside the equation). |
| Toolbar: Comment | Adds a comment to the selected text (yellow underline). Click the text to read, change or delete it. Comments are stored inside the encrypted page. |
| Toolbar: emoji | Inserts an emoji after the selection. |
| Toolbar: "…" | Text alignment. |
| **Fonts and size (the owner's addition)** | A font menu (Default, Sans, Serif, Mono, Handwriting, Rounded) and a size control (−, a menu from 10 to 64, +) sit in the toolbar. |

Pages inside pages: the sidebar shows them nested under their parent (arrow to expand, "+" to add one), the page's top bar shows the path, trashing or deleting a page takes the pages inside it along, restoring a page brings its inside pages back, and moving a page to another section moves them too.

## Storage

Pages gain one optional field, `parent`, in the encrypted page record (older records still load; tested). New command `create_subpage`. The logic that nests, trashes, restores and moves pages is unit-tested.

## Equations and the new dependency

`katex` **0.19.0** (exact, stable, MIT; released 2026-10-01; `npm audit`: 0 vulnerabilities) draws equations. It runs inside the app with no network, its "trust" option is off (equations can't make links or run anything), and its fonts are bundled. Equations are added with the `/` menu ("Block equation" or "Inline equation"), the toolbar button, Turn into, or by typing dollar signs around TeX inside a line (for example `$x^2$`). The TeX is stored in the encrypted page and is searchable. KaTeX is credited in Settings → About.

## Not done (from the screenshots)

- The two icons at the bottom right of the toolbar (reaction and an edit/AI-style button). The app has no AI features and no multi-user reactions, so only the emoji button was made.

## Known limits

- Columns are equal width (no dragging the divider yet).
- "Page in" appends the block to the end of the other page. It doesn't open that page.
- Pages made with these blocks can't be read by an older build of the app.
