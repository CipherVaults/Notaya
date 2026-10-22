# Execution Summary: EX-2026-10-22-11 — Calculator border fix, Cipherleaf app icon (A15), rebuilt portable build

(Short form: four sections, for a small task.)

## 1. Task Overview

**Task requested** (the owner): at 23:24:31 UTC, remove the white border around the calculator display (screenshot); at 23:26:19, make the app icon Cipherleaf everywhere (they saw the Tauri icon in the taskbar); then, mid-turn, "rebuild and commit".

**Authority:** those messages, in [approval-and-amendments.md](../plan/approval-and-amendments.md); the icon is amendment [A15](../plan/A15-app-icon.md); the commit was authorized by "rebuild and commit". Executor: Claude (Sonnet 5.5). Time: about 23:24–23:40 UTC (transcript and clock).

## 2. Work Performed

1. **Border:** the white border was the app's global focus outline (`*:focus-visible`), which overrides the `outline-none` utility class. Both calculators' expression inputs now set `outline: none` and `box-shadow: none` inline.
2. **Icon:** drew `assets/cipherleaf-icon.svg` (the in-app mark, white on a black rounded tile) and ran `npx tauri icon` to regenerate everything in `src-tauri/icons/` (the `.ico` holds 16, 24, 32, 48, 64 and 256 px). Deleted the `android/` and `ios/` folders the command also made.
3. **Rebuild:** `npm run tauri build`. The first rebuild still embedded the **old Tauri icon** (the build output was cached), found by extracting the exe's icon. After `cargo clean -p cipherleaf` (release and debug output of the app only) a second rebuild (2 m 14 s) embedded the Cipherleaf icon.
4. **Portable folder and ZIP:** the new exe (SHA-256 `6129922CD79432135244EA2E170BE6574B65BA97CEE57A3F2ED053965AA22756`) replaced the old one in `E:\Project\Cipherleaf-portable\` (README hash updated; the folder has no `data\` folder). The ZIP `E:\Project\Cipherleaf-portable.zip` (83.1 MB, SHA-256 `2440EC725CAE0D15C3072E84B3FFBEDA6D1BBD1F9B5719C1EDCB4F438A277FD5`) was made again, replacing the one from 23:22 UTC.

## 3. Verification / Testing

| Check | Outcome | Limits |
|---|---|---|
| Mock UI, both calculators focused: computed outline `none`, no shadow; screenshot shows no border | pass | mock only; the real window was not looked at |
| `npx tsc --noEmit`, `npm run build` | pass (no Rust changed, so `cargo test` was not re-run; it passed at 22:50 UTC on the same Rust code) | |
| The exe's embedded icon, extracted and viewed | Cipherleaf leaf on a dark tile (the first rebuild: the Tauri icon) | the taskbar and title bar on a screen were not looked at; Windows may show a cached old icon |
| Smoke test of the new exe from a temporary copy: started, `data\backups`, `data\keepassxc`, `data\webview` created beside it, 0 non-listening TCP connections, nothing in AppData, closed cleanly, nothing left running; the temp copy was removed (one retry, as the exe was still being released) | pass | one run, first screen, no vault created |
| ZIP: 169 entries, no `data` folder inside; extracted exe hash equal to the source; KeePassXC hash equal to the verified value | pass | |

**Not run:** creating a vault in the release build, the Passwords view, lock, the USB test, `cargo audit`, signing.

## 4. Documentation, Git and Next Step

Records updated: A15 (new), the authority table, STATUS, the acceptance ledger (X11 hash; new X12), the A14 follow-up, the execution index; a wrong "about 23:35 UTC" time given for the earlier ZIP was corrected (it was made at 23:22 UTC). **Git:** committed after this summary under "rebuild and commit" (see `git log`). No push.

**Next step:** the owner looks at the taskbar icon (unpin and re-pin if it still shows the old one), tries the portable build and the calculator, and says what to change. Accepted state: none.
