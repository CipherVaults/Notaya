# Execution Summary: EX-2026-10-22-12 — Wider note page, Paper as the default look (A16)

(Short form: four sections, for a small task.)

## 1. Task Overview

**Task requested** (the owner, two messages after 23:39 UTC; exact times not recorded): make the note page bigger because it was too tight (screenshot), and make the Paper look the default and refine it a bit. Then, after the first part was reported: "Commit and rebuild the portable build and ZIP" (time not recorded; the system clock read 23:46 UTC when the commit began).

**Authority:** those messages, in [approval-and-amendments.md](../plan/approval-and-amendments.md); recorded as [A16](../plan/A16-page-width-and-paper-default.md). Executor: Claude (Sonnet 5.5). Time: about 23:40–23:50 UTC (system clock).

## 2. Work Performed

1. **Width:** `WIDTHS` in `src/lib/theme.ts` is now 760 / 1040 / 1320 px (Narrow / Normal / Wide) plus a new `full` (1680 px); `full` was added to the settings type, the Page width control and the comment in `model.rs` (the value is a plain string, so no vault change). `NotesView.tsx` (Home and notebook pages) now uses the same width setting instead of a fixed `max-w-3xl`.
2. **Paper default:** `DEFAULT_APPEARANCE` (`theme.ts`), the Rust `Settings::default()`, `index.html` (first paint) and the mock now use `paper`. Vaults with a saved look are unaffected.
3. **Paper refinement** in `src/styles/app.css` (and the Settings swatch): a slightly lighter page, a clearer sidebar, more border contrast, darker muted and faint text.
4. **Commit** `3a22c56` (the eight source files only; the records were committed afterwards).
5. **Rebuild:** `npm run tauri build` (2 m 11 s) gave `Cipherleaf.exe` with SHA-256 `F4F1E679725021B41F222E852B1CC25BE021FF57C4826640F4A3E4D3FF654800`; its embedded icon was extracted and is the Cipherleaf leaf. The exe replaced the old one in `E:\Project\Cipherleaf-portable\` and the README's hash line was updated.
6. **Found on the way:** the portable folder is no longer fresh: it has a `data\` folder with a vault that the owner's own run of the build created (file times about 23:36–23:40 UTC). It was not touched; only the exe and the README were changed. The earlier records saying the folder has no `data\` were true when written.
7. **ZIP:** `E:\Project\Cipherleaf-portable.zip` was made again from the folder **without `data\`** (169 entries, 0 `data` entries, 83.1 MB, SHA-256 `1AD641640A34773450E5D308DED9493DF5A0F7AF9C194F0729FE00DB6FEFD063`). The extracted exe hash equals the folder's and the extracted KeePassXC hash equals the verified one.

## 3. Verification / Testing

| Check | Outcome | Limits |
|---|---|---|
| Mock UI (headless Edge, 1920×1080): default look is Paper; Home and a note page use the wider column (text about 1050 px) | pass | mock only; the real window was not looked at; Full not looked at |
| `npx tsc --noEmit`, `npm run build` | pass | |
| `cargo test --lib` | pass, 33 tests | the default theme string has no test |
| The throwaway browser and the mock server were stopped | nothing listens on ports 9333 and 1421 | |
| Smoke test of the new exe from a temporary copy of the portable folder (twice): it started, `data\backups`, `data\keepassxc`, `data\webview` and `vault.clf` were there (the vault file is the owner's, copied), 0 non-listening TCP connections, nothing in AppData, closed cleanly, nothing left running; the copy was deleted | pass | the owner's vault was only copied and left locked; the new look in the window was not looked at |
| ZIP: 169 entries, no `data`; extracted exe hash equal to the folder's; extracted KeePassXC hash equal to the folder's | pass | |

**Not run:** the real window, the Full width, the lock screen in Paper, an existing vault's look, the Passwords view and lock in the build, the USB test, `cargo audit`, signing.

## 4. Documentation, Git and Next Step

Records updated: A16 (new), the authority table, STATUS, the acceptance ledger (X11 hash, X13), the execution index. **Git:** code commit `3a22c56`; records in the commit after it (see `git log`). No push.

**Next step:** the owner opens the portable build (it keeps their vault in `data\`) and says whether the page width and the Paper look are right. Accepted state: none.
