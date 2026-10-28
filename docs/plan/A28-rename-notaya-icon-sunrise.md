# A28: Renamed to Notaya, sticky-notes icon, Sunrise theme

**Status:** applied on the owner's instruction (2026-10-27 16:59:11 UTC); the code is in draft [PR #24](https://github.com/AbyssWatcher2/CipherLeaf/pull/24), commit `6003e81`. On the owner's PC (2026-10-27, 18:58 to 19:15 UTC) `cargo test --lib` passed 129 of 129, the release build, a real-exe test on throwaway data and a throwaway Cipherleaf 1.1.0 vault opening in `Notaya.exe` passed (see [EX-2026-10-30](../execution/2026-10-27-notaya-rename.md), addendum 4); the exe description and publisher fixes (`a97d5e7`, `d4933ef`) were rebuilt and checked on the PC (addenda 5 and 6; CompanyName now Notaya). Not accepted.

## What the owner asked for

Times are UTC on 2026-10-27, from the project transcript.

> "proceed with this and The app name is Notaya" (16:59:11, with a screenshot of theme 2, Sunrise: the two-sticky-notes icon in amber `#ffb020` and coral `#ff6b5a`, accent `#e8553f`, and the light and dark mock-ups)

> "The theme sunrise will be the defult while the darker will be can be changed" (17:03:39)

Before this, the owner picked the icon (round 3, option 5, two sticky notes, 14:00:13) and the Sunrise theme (14:29:43). The options and the name screens are in [`docs/design/`](../design/icon-options/README.md).

## What changed

- **The name is Notaya** everywhere a person sees it: the window and title bar, the HTML title, messages and dialogs (Rust and UI), Settings, the README, `THIRD-PARTY-NOTICES.md`, the real-exe and mock test scripts. The exe is now `Notaya.exe` (`productName` and `mainBinaryName` in `src-tauri/tauri.conf.json`). Settings > About and the README say the app was called Cipherleaf up to version 1.1.0, and the README says how to update a Cipherleaf folder.
- **The icon** is two sticky notes, amber behind and coral in front, on a cream tile: `assets/notaya-icon.svg` (the round 3 option 5 drawing in the Sunrise colours, `docs/design/themes/2-sunrise-icon.svg`). Every file in `src-tauri/icons/` was regenerated from it with `npx tauri icon` (the `.ico` has 16, 24, 32, 48, 64 and 256 px); the Android and iOS folders the command also makes were removed, as in A15. The in-app mark (`src/components/Logo.tsx`) is the same drawing in fixed colours. The old leaf source moved to `docs/design/old-cipherleaf-icon.svg`.
- **Two new looks:** **Sunrise** (cream `#fffaf3`, sidebar `#fbefe0`, text `#2b1d14`) and **Sunrise dark** (`#1a1411`, sidebar `#231b16`, text `#f7ece2`). The accent choice "Theme color" (stored as `mono`, as before) is coral `#e8553f` in both; in the other looks it stays black and white. Settings shows seven theme tiles (Sunrise first).
- **Sunrise is the default** for a new vault (`Settings::default`), before the first unlock (`DEFAULT_APPEARANCE`) and in `index.html`. Sunrise dark, Black, Graphite, White, Paper and Match Windows stay choices in Settings.

## Kept on purpose, so existing notes keep working

- The app identifier `app.cipherleaf.vault` (Windows uses it for the single-instance guard; Cipherleaf and Notaya cannot run at the same time on one PC, which protects the vault file).
- The `data` folder next to the exe, `data\vault.clf` and its format: the AAD prefix `cipherleaf`, the HKDF label `cipherleaf/v1/key-encryption-key` and the built-in open-vault password are unchanged, with a comment at each saying why. Backup names (`vault-*.clf`, `restore-undo-*`, `set-aside-*`) are unchanged.
- `CIPHERLEAF_HOME` and `CIPHERLEAF_OPEN_BACKUP_SECS`, the browser-profile keys (`cipherleaf.appearance`, `cipherleaf.layout`, `cipherleaf.ribbonCollapsed` and the others) and the Rust crate and npm package names (`cipherleaf`, internal only).
- **An existing vault keeps the theme it saved** (most have Paper, the old default). Its owner chooses Sunrise in Settings > Appearance once. This was chosen over switching saved vaults silently, which would override a deliberate choice of Paper.
- An older Cipherleaf that opens a vault saved with a Sunrise look shows it in Black, as it does for any look it does not know, and saves Black as soon as it saves any setting or calculator entry; Notaya then shows Black too until Sunrise is chosen again. Nothing else changes.

## Things to know

- The password strength check now treats both "cipherleaf" and "notaya" as weak words.
- The default file names in the save dialogs are now `notaya.key` and `notaya-backup-<seconds>.clf`.
- Contrast: white text on coral `#e8553f` is about 3.6 to 1, enough for buttons and large text but below 4.5 to 1 for small text. Coral `#e8553f` as text on the Sunrise cream (the active sidebar item, links in Settings) is about 3.5 to 1, also below 4.5 to 1. The owner asked for these exact colours; a darker coral for text only is an option if he wants it.
- "Match Windows" still follows Windows into Black or White, not into the Sunrise looks.
- Windows keeps an icon cache: a pinned taskbar entry or a shortcut to `Cipherleaf.exe` keeps the old icon and points at the old exe. Pin `Notaya.exe` again.
- A build must pick up the new icons: clean the cached build output first (`cargo clean -p cipherleaf`), as found in A15.

## Not changed

Behavior, security (no CSP, capability, IPC or crypto change), the sticky-note colours, the version number (still 1.1.0; a release is a separate step).
