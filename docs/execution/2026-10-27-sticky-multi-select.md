# EX-2026-10-27-sticky-multi-select: select several sticky notes and delete them together

Short form (4 sections). Times are UTC from the owner's message and the container clock.

## 1. Task Overview

The owner asked on 2026-10-27 at 06:45:11 UTC (item 3 of a four-part message): "select multiple sticky notes to delete". Other parts of that message (KeePass removal, a pop-out calculator, a ribbon editor) are separate work and not in this change.

## 2. What was done

- **Sticky notes screen** (`src/features/sticky/StickyBoard.tsx`): a "Select" button turns on select mode. Clicking a note (or Space/Enter on it) ticks it instead of opening it; Ctrl+click on a note starts select mode from the normal view. The header shows "N selected", "Select all" / "Select none", "Delete" and "Cancel". Escape leaves select mode. Delete asks for confirmation ("Delete N sticky notes? They will be permanently removed.") and then removes them all. Select mode ends by itself when no notes are left.
- **New command `delete_stickies`** (`src-tauri/src/commands/stickies.rs`, registered in `src-tauri/src/lib.rs`, wrapper in `src/lib/ipc.ts`): removes the given sticky notes in one vault transaction (`remove_many`), then the same clean-up as `delete_sticky`, then closes their desktop windows. Ids that no longer exist are skipped; if any id belongs to something that is not a sticky note, nothing is removed. At most 10,000 ids per call. It can do nothing that calling `delete_sticky` repeatedly cannot, so no security promise or boundary changes (`docs/security-design.md` is unchanged).
- **Dev mock** (`dev-mock/core.ts`): handles `delete_sticky` and `delete_stickies`, and returns a fresh list from `list_stickies` so the screen updates.

An independent review agent (read-only) found no blocking bugs or security issues. Its low findings were fixed: Escape in the confirm dialog no longer also clears the ticks, a note deleted in its own window no longer makes the whole delete fail, the mock list refreshes, focus returns to the board after the dialog, ticked notes get a short accessible name, and an empty select mode closes.

## 3. Checks

| Check | Result |
|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Passed (Linux container) |
| `npm run build` | Passed (Linux container) |
| Mock UI, scripted with Playwright (Chromium) | Passed: tick two notes by click and keyboard, Escape inside the dialog keeps the ticks, Escape on the board leaves select mode, Ctrl+click starts it, Select all, Delete all with confirmation, empty board afterwards |
| `cargo test --lib` | **Not run**: the container lacks the GTK libraries the crate needs. The Rust change was only read, not compiled. |
| Real exe on throwaway data | **Not run** |

## 4. Open items

- Compile and run `cargo test --lib` on Windows, and try it in the real app on a throwaway vault.
- The owner's acceptance.
- Another thread is changing the calculator on the same screen; a merge conflict in `StickyBoard.tsx` is possible and small.
