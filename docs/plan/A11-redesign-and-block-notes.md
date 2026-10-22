# A11: New look (Black, Graphite, White, Paper) and Notion-style notes with pictures

**Status:** applied under the owner's instruction of 2026-10-22 (2026-10-22 16:55:27 UTC, from the transcript; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. **Partly superseded by [A13](A13-calculator-and-wording.md):** page covers and page icons can no longer be added (existing ones still show).

## What the owner asked for

> okay first i don't like the whole design in the app you will have to redesigned it for something better UI/UX the has theme black only okay and other colors as well and white as well second the notes i need to be like notion notes and i can put pictures in it as well  do those things

Read as: a full visual redesign; themes for **black**, **white** and other colors (kept as accent colors and extra looks); notes that behave like Notion (blocks, a "/" menu, drag to reorder, covers and icons), with pictures.

## What changed

| Area | Change |
|---|---|
| Themes | Four looks: **Black** (true black, the default), **Graphite** (soft dark gray), **White**, **Paper** (warm off-white), plus "Match Windows". Settings → Appearance shows a small preview of each. Accent colors (Black & white, Blue, Sky, Teal, Green, Amber, Orange, Rose, Violet, custom) stay. Mica and Acrylic remain optional (default Solid). Old settings named "dark" and "light" are mapped to Black and White. |
| Layout | One sidebar like Notion's: Home, Passwords, Sticky notes, Settings, Favorites, and the notebook tree (notebooks → sections → pages) that can be dragged to reorder and resized. The old three-column layout is gone. A Home page shows recent pages. Trash is its own page. Smaller corners, flat panels, quieter chrome. |
| Notes | Block editor: type `/` for a menu (text, headings, to-do, bulleted and numbered lists, toggle, quote, callout, divider, code, image, table, today's date). A grip and "+" beside each block let you drag it, duplicate, delete, move up or down, or add a block below. Selecting text shows a toolbar (turn into, bold, italic, underline, strike, code, link, color and highlight, font, size, alignment). Notion-style page icon (emoji) and cover (gradient or your own picture). |
| Pictures | Paste, drop, or `/image` to add pictures; drag the side handles to resize; align left, center or right; add a caption. Pictures are encrypted in the vault like before (PNG, JPEG, GIF, WebP, up to 25 MB). Covers can be a picture too. |
| Storage | Pages gain two optional fields, `icon` and `cover`, stored inside the encrypted page record (older records still load; tested). Cleaning up unused pictures now also counts a page's cover as "in use" (tested). New command `set_page_look`. |
| KeePassXC | The preset now also sets `UpdateCheckMessageShown=true`, so KeePassXC no longer asks "Check for updates on startup?" on first start. Updates stay off. |

## Not changed

The vault format and encryption, key handling, auto-lock, the KeePassXC version and hash check, and the in-window KeePassXC display from A10.

## Known limits

- The block drag handle works on top-level blocks, list items, and blocks inside callouts, toggles and quotes. It does not yet drag table rows or columns.
- Text is stored as before (TipTap JSON). Callouts and toggles are new block types; a page that uses them can't be read by an older build of the app.
- Layout choices (sidebar open/closed and width) and the chosen look are cached in the WebView2 storage inside `data/webview/`, as the appearance settings already were.
