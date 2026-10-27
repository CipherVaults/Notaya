# EX-23: Pop-out calculators with a shared saved history (A24)

Short form (4 sections). Times are UTC from the session transcript or the cloud container's clock, as marked.

## 1. Request and scope

The owner's message of 2026-10-27 06:45:11 UTC (transcript), item 2: "the calculator panel in sticky notes section make it as button, then make the calculator to popout when i press in sticky note, and can be opened multiple times, the calculator has shared history saved somewhere even if i closed it". The other items of that message (KeePassXC removal, deleting several sticky notes, a ribbon editor) are done in other threads and are not part of this record.

Reading used: the panel under the Sticky notes list becomes a **Calculator** button; the calculator button on each sticky note also opens a calculator window (the small calculator inside the note is removed, so there is one calculator and one history); each press opens one more window; all windows share one history saved in the vault.

## 2. What changed

- Rust: `commands/calculator.rs` (new: `open_calculator`, `calc_history`, `add_calc_entry`, `clear_calc_history`, the `calc-history-changed` event, unit tests); `ui.rs` (`open_calculator`: frameless, always on top, 360 × 640, cascaded; `close_calculators`); `state.rs` (the lock closes calculator windows); `model.rs` (`Settings.calc_history`, `CalcEntry`, at most 200 lines (100 at first; the owner asked for 200 at 07:50:00 UTC); an older vault without the field loads, and an older Cipherleaf ignores the field); `commands/vault.rs` (`get_settings` leaves the history out; `save_settings` keeps the stored history); `lib.rs` (commands registered); `capabilities/default.json` (`calc-*` added to the window list; no new permissions).
- UI: `Calculator.tsx` is now the calculator window (title bar with minimize and close, Basic/Scientific, Deg/Rad, history below the keys, resize grip); `useCalc.ts` loads and saves the shared history; `StickyBoard.tsx` (Calculator button, panel removed); `StickyWindow.tsx` (its calculator button opens a window); `MiniCalculator.tsx` deleted; `main.tsx` routes `calc-*` windows; `app.css`; `dev-mock/core.ts` (mock commands; open `?label=calc-1` to see a calculator window).
- Records: README, `security-design.md` (P15 and a new limit), amendments (A24), this record, the index, STATUS, sessions.

## 3. Checks

- Cloud container (2026-10-27): `npx tsc --noEmit -p tsconfig.json` passed; `npm run build` passed (after `npm ci`).
- `dev-mock/calc-ui-check.cjs` (now pointed at a calculator window) printed ALL OK in the cloud.
- Mock UI with Playwright (cloud): the Sticky notes screen shows the Calculator button and no panel; a calculator window (`?label=calc-1`) evaluates `200 + 10%` = 220 and `sin(30)` = 0.5 in Deg, and its history lists them.
- **Not run in the cloud:** `cargo test --lib` and any Rust build (the crate needs the `windows` crate and an MSVC toolchain, which the Linux container does not have).
- Owner's PC, throwaway clone `%TEMP%\clf-calc` of commit `f35acb2`, finished 2026-10-27 06:57:59 UTC (system clock): `cargo test --lib` 171 passed, 0 failed, no compiler warnings (including the new `calculator::tests` and `model::tests`); `npm ci` 0 vulnerabilities; `tsc` and `npm run build` passed. The project folder `E:\Project\cipherleaf` and every vault were not touched.
- Owner's PC, commit `58c299c` (after the KeePassXC removal was merged in and the limit raised to 200), 2026-10-27 07:51 to 08:02 UTC (system clock), clone `%TEMP%\clf-calc`: `cargo test --lib` 128 passed (fewer than before because the KeePassXC tests went with A24); `npm ci` 0 vulnerabilities; `tsc` and `npm run build` passed; `tauri build --no-bundle` passed (exe SHA-256 `7AFB7C75…EAF10`). Real exe with a new throwaway `CIPHERLEAF_HOME` (`%TEMP%\clf-calc-home2-105828`), debug port 9335 for that process only, 20 of 20 checks passed: the Calculator button twice opened two windows; 2+2 and 7×6 showed in both windows' history, newest first; both closed, the app quit (exit 0), restarted with the same home, and a new calculator still showed "7×6 = 42" and "2+2 = 4"; 205 more lines left exactly 200 saved (newest first, oldest dropped); Clear in one window emptied the other and the saved history; a sticky note's calculator button opened a window. A first run's script misread the text and was redone from scratch; in that rerun the test driver's close went to the always-on-top calculator instead of the main window, so the test process was stopped by the driver; pressing the main window's own Close with a sticky and a calculator open quit in under a second, exit 0, twice. Afterwards nothing listened on 9335, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` was not set, no Cipherleaf process ran; temp folders left in place.
- **Not run:** closing the calculators on a lock (the vault had security off, which never locks).
- Independent review (an agent that did not write the code): nothing serious. Fixed from it: Deg/Rad recall now uses this window's last expression, not the shared newest line; the window says it is ready only after its lock listener is in place; a calculator window made while the vault was locking is closed at once; a failed Clear reloads the saved history; the records (README, P15, the window limit) were corrected. Left as is: a calculator window keeps the look it opened with until reopened; each "=" rewrites the settings record (small); an older Cipherleaf that saves settings drops the history (recorded in security-design).

## 4. Open points

- Closing on lock with security on: not run in the real app.
- The owner's review. Nothing is accepted.
- The amendment number A24 may clash with another thread working at the same time; renumber on merge if so.
