# A16: A wider note page, and Paper as the default look

**Status:** applied on the owner's instructions (two messages after 23:39 UTC on 2026-10-22; exact times not recorded; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result.

## What the owner asked for

> make the page bigger it's too tight (with a screenshot of a note: the text column used about half of the window)

> And also the paper theme make it as default and you can refine a bit

## What changed

- **Page width.** The editor widths grew: Narrow 640 → 760 px, Normal 780 → 1040 px, Wide 1040 → 1320 px, and a new **Full** (1680 px) in Settings → Appearance → Page width. The Home and notebook pages use the same width setting instead of a fixed 768 px column. The width is still capped, so a very large window does not give unreadably long lines.
- **Paper is the default look** for a new vault (the app's default, the first paint of the window before the settings load, and the mock). A vault that already has a saved look keeps it; the owner can pick any look in Settings.
- **Paper refined a little**: slightly lighter page (`#faf6ee`), a clearer sidebar (`#f1e9da`), a bit more border contrast, and darker secondary and faint text (the faint text was about 2.9:1 against the page, now about 4:1), so hints and placeholders are easier to read. The Settings swatch matches.

## Things to know

- Existing vaults keep their saved page width setting, but each width is now bigger, so "Normal" and "Wide" look larger without any change by the owner.
- A fresh portable build starts in Paper; the dev vault in `.devdata\data\` keeps whatever look it was saved with.
- Committed as `3a22c56` and included in the portable build and ZIP rebuilt at about 23:49 UTC on "Commit and rebuild the portable build and ZIP" (see [EX-12](../execution/2026-10-22-page-width-and-paper-default.md)).
- The portable folder holds the vault the owner created in their own run; the rebuild kept it. The ZIP has no `data\`.

## Not changed

Behavior, security, the vault format (the width value is stored as text, so "full" needs no migration).
