# A15: The Cipherleaf app icon

**Status:** applied on the owner's instruction (2026-10-22 23:26:19 UTC; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result.

## What the owner asked for

> Make the app icon cipherleaf everywhere cause i saw in the taskbar its tauri

## What changed

- A new icon, `assets/cipherleaf-icon.svg`: the in-app Cipherleaf mark (a leaf whose vein is a keyhole) in white on a black rounded tile, so it shows on dark and light taskbars.
- `npx tauri icon assets/cipherleaf-icon.svg` regenerated every icon file in `src-tauri/icons/` (the Windows `.ico` with 16–256 px sizes, the PNG sizes, the Windows tile logos and the `.icns`). The mobile icon folders the command also makes were deleted because the app is Windows-only.
- The icon is embedded in the exe and used for the window, the taskbar and the file in Explorer. The in-app logo and the title bar already used the same mark.
- The default Tauri icon files are gone from the project.

## Things to know

- A build must pick the icon up: the first rebuild still carried the old Tauri icon until the app's cached build output was cleaned (`cargo clean -p cipherleaf`). The rebuilt exe was checked: its embedded icon is the Cipherleaf icon.
- Windows keeps an **icon cache**. A taskbar or Explorer entry made earlier (or pinned) may keep showing the old Tauri icon until it is unpinned and pinned again, or Explorer is restarted.
- The dev app also needs a rebuild to show the new icon; its cached build output was cleaned, so the next `npm run tauri dev` recompiles from scratch (longer than usual).

## Not changed

Behavior, security, the name (Cipherleaf), the app identifier.
