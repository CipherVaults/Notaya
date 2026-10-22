# A13: A calculator under Sticky notes; reworded and trimmed block editor; the name stays Cipherleaf

**Status:** applied on the owner's instruction (2026-10-22 22:26:02 UTC, and their answers to Claude's two questions at 22:27:00 UTC; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. It partly supersedes [A11](A11-redesign-and-block-notes.md) and [A12](A12-notion-menus-subpages.md).

## What the owner asked for

> Now continue the app development: Calculator. Keep CipherLeaf. Reword, or reword and trim, the features closest to Notion.

Their choices when asked: **reword and trim** (Claude's recommended option) and a calculator with **basic and scientific** modes. The reason behind it, from 17:56 UTC: the design and features look very similar to Notion and they don't want copyright trouble. (This is not legal advice; the aim was to move away from copied wording, order and layout.)

## What changed

### Calculator (Sticky notes view)

- A panel under the sticky notes: **Basic** and **Scientific** modes; **Deg / Rad** for angles.
- Type in the display (Enter calculates, Esc clears, ↑ recalls the last expression) or tap the keys.
- Operators: `+ − × ÷ ^ ( ) ! %`, unary minus, implicit multiplication (`2π`, `3(4+1)`); constants `π`, `e`, `Ans`; functions `√`, `abs`, `ln`, `log`, `eˣ`, `10ˣ`, `sin cos tan`, `sin⁻¹ cos⁻¹ tan⁻¹`, `1/x`. Percent works as on a handheld calculator (`200 + 10%` is 220).
- A live preview of the answer, a **Copy** button, and a history list (click an entry to reuse it).
- **Privacy:** the history lives only in memory. Locking the vault reloads the screen, so it is cleared. Nothing about the calculator is saved to disk. The text you type is parsed by the app's own parser; nothing is run as code, and there is no network use.
- Copy writes only the displayed result to the clipboard and is not cleared automatically (it isn't a secret).

### Wording and layout of the editor (reworded)

- Block names: Paragraph, Large / Medium / Small heading, Bullet list, Numbered list, Checklist, Collapsible section, Quote, Highlight box, Code block, Equation, Sub-page. The `/` menu uses the groups Text, Lists, Insert, Quick, with new descriptions; "Turn into" became "Change type"; the editor hint reads "Press / to insert something, or just start typing".
- The selection toolbar is now one horizontal bar of our own design (block type, font, size, bold, italic, underline, strikethrough, code, link, color, equation, align, remove formatting) instead of the stacked layout taken from the owner's reference screenshots.
- "Notion" no longer appears in the code. The records that describe the history still name it.

### Trimmed (can no longer be created)

Columns, toggle headings, heading 4, "Page in" (moving a block into another page), comments, the emoji button, page covers and page icons.

**Nothing is deleted from existing pages.** The editor still understands those older blocks, so a page that already has columns, comments, a heading 4, a toggle heading, a cover or an icon still shows them (a cover or icon can be removed with a small button on hover). Only creating new ones was taken away.

### Kept

Headings, lists, checklist, collapsible section, quote, highlight box, code block, divider line, table, pictures (resize, align, caption), links, text and highlight colors, fonts and sizes, equations (block and inline), sub-pages, the drag handle and block menu, and the `/` menu.

### Name

Stays **Cipherleaf**. No rename, no icon change, no change to the app identifier.

## Not changed

The vault format, encryption, key handling, auto-lock, the KeePassXC bundle and its display, the themes and the layout from A11.

## Known limits

- Real mouse use and the clipboard copy inside the real app were not checked by hand (only in the mock UI).
- The wording is our own, but the overall idea (a block editor with a `/` menu and drag handles) is common to many editors and stays.
