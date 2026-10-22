# A10: Show KeePassXC inside Cipherleaf, black-and-white look, font size

**Status:** applied under the owner's instruction of 2026-10-22 (2026-10-22 16:12:20 UTC, from the transcript; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result.

## What the owner asked for

> the UI/UX doesn't look good change it and i need it black and white and other color too will be good as optional / i need to add dont size as well / the keepassxc has to be inside the app like open it inside cipher not outside it

"dont size" was read as **font size**.

## What changed

| Request | Change |
|---|---|
| Black and white, other colors optional | The default theme is now black and white (dark: pure black, light: white). "Black & white" is the first accent choice; Blue, Sky, Teal, Green, Amber, Orange, Rose and Violet stay as options, plus a custom color. The old violet default is migrated to black and white when settings are read. Mica/Acrylic stay as optional window materials (default: none). |
| Font size | The note editor toolbar gains a font family menu (Default, Sans, Serif, Mono) and a font size control (−/+ in 2 px steps from 8 to 96, a menu of preset sizes, and "Default size"). The existing global text-size slider in Settings is unchanged. |
| KeePassXC inside Cipherleaf | The Passwords view shows the real KeePassXC window **inside the main window**, over the panel area. See "How the embedding works". A header button (and a fallback card) can still open KeePassXC in a separate window. |

## How the embedding works, and why it isn't a true child window

Windows refused `SetParent` on KeePassXC (error 5, access denied), and WebView2 draws on top of ordinary child windows, so a real child window can't be shown. Instead:

1. Cipherleaf starts the verified KeePassXC (unchanged from A9).
2. It finds KeePassXC's top-level window by process image path.
3. It makes that window **owned** by the Cipherleaf main window, removes its title bar and borders, and places it exactly over the Passwords panel area (physical pixels, converted from the panel's on-screen rectangle).
4. It follows the main window when that moves or resizes; hides it when you leave the Passwords view, open the command palette, or lock; and closes it politely (`WM_CLOSE`, then kill after 3 s) when Cipherleaf quits or its window closes.
5. If any step fails, the original window styles are restored, so KeePassXC is never left half converted.

Only window handles and rectangles cross the boundary. Cipherleaf still never sees KeePassXC's master password or entries.

## Limits (stated up front)

- It is an **overlay**, not a true child window. Anything drawn by Cipherleaf's web page (for example the command palette) can't appear over it, so the overlay hides while the palette is open.
- KeePassXC blocks screen capture by default, so it appears black in screenshots and screen shares; Cipherleaf's screenshot setting doesn't govern it.
- Typing and mouse input inside the embedded KeePassXC were **not** verified by hand; only geometry and visibility were checked by script.

## Not changed

Security behavior, the vault format, the KeePassXC version and hash check, and the offline presets are unchanged.
