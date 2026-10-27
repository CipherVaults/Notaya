# New app icon and name: options (discussion only)

**Status:** options for the owner to choose from. Nothing in the app has changed: the icon files in `src-tauri/icons/`, `assets/cipherleaf-icon.svg` and the name "Cipherleaf" in the code are untouched. Building starts only after the owner picks and says go.

**Asked (times from the project transcript, UTC, 2026-10-27):**
- 13:31:51 "Okay let's design some new icons give me choices and new names"
- 13:32:44 "Cause you know something has been changed"
- 13:46:09 "I don't need leaf" (after round 1). So round 2 below has no leaf in any icon or name.

## Why a new identity

The name and icon came from the first plan, when Cipherleaf was a password-locked vault with KeePassXC bundled. Since V1.1.0 (2026-10-27) KeePassXC is gone and the app is notes-first: pages and sub-pages, a ribbon editor with a `/` menu, sticky notes, pop-out calculators, all stored locally. The password lock and encryption are still there but optional (A21). So the new icon and name should say "notes" first and "private, on your PC" second.

## Round 2 icon options (current)

All six are 1024 × 1024 SVGs in [`round-2/`](round-2/), ready to feed `npx tauri icon <file>.svg` (the same step as [A15](../../plan/A15-app-icon.md)). The sheet shows each at full size and at 48, 32 and 16 px on a dark and a light taskbar.

![Round 2 icon options](round-2/icon-options-sheet.png)

| | Option | Idea | Notes |
|---|---|---|---|
| 1 | [Note with lock](round-2/1-note-with-lock.svg) | A paper note with a small gold padlock badge, on indigo | Says both things in the right order: notes, then lock. |
| 2 | [Sticky note](round-2/2-sticky-note.svg) | A yellow sticky note with a lifted corner, on black | The clearest at 16 px. Matches the sticky notes feature. |
| 3 | [Banded notebook](round-2/3-banded-notebook.svg) | A notebook with an elastic band and a ribbon bookmark | Personal and "kept closed" without a lock symbol. Light tile. |
| 4 | [Keyhole page](round-2/4-keyhole-page.svg) | A page with a keyhole cut through it, on teal | One simple shape; leans a bit more towards "lock". |
| 5 | [Pen nib](round-2/5-pen-nib.svg) | A fountain-pen nib drawing a gold line, on black | About writing; says nothing about privacy. |
| 6 | [Pages with bookmark](round-2/6-pages-with-bookmark.svg) | Stacked pages (pages and sub-pages) with a red bookmark | Keeps today's black-and-white app look. |

## Round 2 name options (current)

None of these has been checked for trademarks or existing apps with the same name; that check comes before a final choice.

| Name | Why |
|---|---|
| **Hushpad** | A quiet, private notepad. "Hush" hints at the optional lock without making it the point. |
| **Quietpad** | Calm and offline: no account, no network, nothing leaves your PC. |
| **Pagekeep** | It keeps your pages and sub-pages, on your own computer. |
| **Stillnote** | Calm, simple notes. |
| **Jotkeep** | For quick jots and sticky notes that stay put. |
| **Homepad** | Your notes stay at home, on your PC. |
| **Paperlock** | Paper plus a lock. Says "lock" more than the app now does. |
| **Folio** | Classic word for a set of pages. Very common, harder to search for. |
| **Margin** | Short; the place you jot things down. Very common word. |

## Recommendation (round 2)

**Icon 1 (Note with lock) with the name Hushpad.** Both say "notes first, private if you want it", which is what the app is now. Runner-up: **icon 2 (Sticky note) with Quietpad**, the friendliest and the clearest in the taskbar.

## Round 1 (history)

The first round used a leaf in every icon and most names. The owner turned the leaf down, so it is kept only as a record: [`round-1/`](round-1/README.md).

## What a rename would touch (for when the owner says go)

- Change: `productName` and the window title in `src-tauri/tauri.conf.json`, in-app text that says "Cipherleaf", the README and release names (exe and ZIP file names).
- Keep, so existing notes keep opening: the app identifier `app.cipherleaf.vault`, the `data\vault.clf` file and its format, and the `CIPHERLEAF_HOME` variable (or accept both old and new).
- Timing: rename before the Windows installer ships. The installer plan ([PR #23](https://github.com/AbyssWatcher2/CipherLeaf/pull/23)) puts installed data in `%LOCALAPPDATA%\Cipherleaf\data`; choosing the name first avoids a folder named after the old one.
- The new icon goes through the same steps as A15: regenerate `src-tauri/icons/`, clean the build (`cargo clean -p cipherleaf`), and expect Windows' icon cache to show the old icon on pinned taskbar entries until re-pinned.
