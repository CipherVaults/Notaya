# Libraries and licences for the new features: a study

**Who and where:** Ahmed (AE0101), written by Claude (Sonnet 5.5, high effort) in a cloud thread. **Date and time:** 2026-11-07, about 23:50 to 00:30 UTC (system clock of the cloud machine). **Baseline:** Notaya `main` at `67b27e4` (v1.5.1 plus the plan records). **Asked for by:** the coordinator, from section 9.2 of the version plan. **Result state:** *produced*. This is a study, not a decision. Nothing was installed or added to Notaya; Ahmed's own words are still needed before any of this is built (one "new library" approval each).

> **Dated note (2026-11-07 23:56 UTC, from the coordinator):** the versions after 1.9 were renumbered. Where this study says 1.10 to 1.15, read 2.0 to 2.5 (so "1.13/1.14 whiteboard" is now in the 2.x line, "1.12 dashboards" too, and the 1.10-6 About page is in 2.0). The old numbers are kept below as written; check the current plan for the exact new version of each item.

> **Addendum (2026-11-08 00:05 UTC):** Ahmed answered the questions (broad tablet support, emoji in PDF, Excel charts, Excel import). The recommendations for emoji, Excel and tablets changed; see the addendum at the end of this file, which replaces rows 3 and 4 below.

## 1. The short answer

| # | Need | Recommendation | New code in the program | Licence | When |
|---|---|---|---|---|---|
| 1 | Whiteboard pens with pressure | **perfect-freehand 1.2.3** for the smooth pen line, plus the pen support that is already built into the app's web engine. No big whiteboard kit. | 1 small package (4.5 KB, nothing else under it) | MIT | 1.13 |
| 2 | Global shortcuts | **Tauri's own global-shortcut plugin 2.4.1**, used only from the Rust side so the screen code gets no new powers | 6 small Rust crates | MIT or Apache-2.0 | 1.7 |
| 3 | Fonts for Arabic and Greek in PDF | **Arabic is already done.** For Greek (and Cyrillic) add a third small piece of **Noto Sans**, made the same way as the two fonts already inside the program | no code, one font file, roughly 40 to 80 KB (an estimate, not measured) | SIL Open Font Licence 1.1 | 1.9 |
| 4 | Excel files | **Write it ourselves** on the zip and compression code that Word export already uses. Keep `rust_xlsxwriter` as the named upgrade if we ever want Excel charts. | 0 | none needed | 1.9 |
| 5 | The AI door (MCP over a Windows named pipe) | **Official MCP kit `rmcp` 3.9.0** inside the small bridge program; on the Notaya side use the pipe support in **tokio**, which is already in the build, and set the pipe's access rule by hand | 0 in Notaya itself; the separate bridge program has its own list (67 crates, all permissive) | Apache-2.0 (rmcp), MIT (tokio) | 1.6 |

Every recommended piece uses a permissive licence. None of them is a "copyleft" licence that would force Notaya's own code to be published under the same terms. They only ask for credit (a copyright line and the licence text), which `THIRD-PARTY-NOTICES.md` and the 1.10 About page already provide for.

Two surprises worth knowing first:
- **tldraw**, the best-known whiteboard kit, **cannot be used**: its licence forbids use in a released product without a paid key, it shows a watermark, and it may send usage data to its owner.
- A Windows named pipe is, by default, **readable by everyone on the PC**. Microsoft's own page says so. So the pipe for the AI door needs an explicit "only my Windows account" rule. That is a real security task, not a detail (section 6).

## 2. How I checked, and what I could not

I read the package registries (npm and crates.io) on 2026-11-07 at about 23:55 UTC, the tldraw licence file, Microsoft's pages on named pipes and WebView2 pen input, the source of the shortcut crate, and the Liberation fonts licence header. To count what a crate really adds, I made scratch copies of `Cargo.toml` and `Cargo.lock` outside the repository, added one crate at a time, and listed what appeared that the build did not already have (target: Windows MSVC). The repository was not touched.

**Not done, so please do not read it as checked:** GitHub activity (stars, last commit, open issues) because GitHub was not reachable from this thread; known-vulnerability lists (`cargo audit` has never been run on this project, and I did not run it); the real size of the new font piece (needs the font tool, which I did not install); anything on real Windows or on a drawing tablet; the NOTICE files, if any, inside rmcp. Version numbers are exact as of the readings above.

## 3. Whiteboard: pens with pressure, and the canvas

**What the feature needs** (Ahmed 23:16 to 23:23 UTC): an endless canvas, pen, pencil, marker and highlighter, an eraser, mouse and touch, and above all **pressure from a drawing tablet**, with the tablet's eraser end and palm rejection. The strokes are saved as plain points inside the encrypted vault.

**Two separate jobs.** (a) *Getting the pen data*: that is not a library job. The app draws its screen with the Windows web engine (WebView2), and Microsoft's documentation says it passes on pen **pressure, tilt and rotation** as they come from Windows. Web pages receive that through the standard "pointer events". (b) *Turning a trail of pen points into a smooth, pressure-shaped line*: that is the part a small library does well, and we should not invent the maths ourselves.

| Candidate | Licence | Size and weight | Fit | Verdict |
|---|---|---|---|---|
| **perfect-freehand 1.2.3** | MIT | 4.5 KB of code, nothing under it. Released 2021, last release 2026-02-01 | Takes points with pressure, returns the outline of a natural brush stroke (the same idea used by well-known drawing apps). Does one job only | **Recommended** |
| tldraw 5.6 | Own licence: no production use without a key, watermark, "may send usage data", no standalone distribution | 15.6 MB, 18 packages under it | A whole editor with its own look | **Ruled out** (licence and offline promise) |
| Excalidraw 0.18.1 | MIT | 46.8 MB, 31 packages under it | A whole app with its own interface and file format | Too big; fights our own templates, themes and vault format |
| Konva 10.9.1 | MIT | 1.9 MB, nothing under it, released 2026-10-31 | A "scene" toolkit: select, move, resize, layers | Good fallback for the shapes and selection work in 1.14; not needed for pens |
| Fabric 7.4.0 | MIT | 22 MB | Same job as Konva, much heavier | No |
| signature_pad 5.1.4 | MIT | 0.5 MB | Width follows pen speed, not real pressure | No (pressure is the point) |
| Paper.js 0.17.0 | MIT | 9.4 MB | Vector scripting kit | No |

**Recommendation.** Take perfect-freehand for the stroke line. Read the pen through standard pointer events (pressure, tilt, the eraser end, the difference between pen and finger so a resting palm is ignored). Draw on a plain canvas, only what is on screen. Decide in the 1.13/1.14 spec whether selection, resize handles and layers deserve Konva or our own thin layer; I lean to our own layer first because a board holds few objects but many strokes.

**The one real risk is the tablet driver, not the library.** The web engine sees tablets through Windows' own pen system ("Windows Ink"). Many tablet drivers have a switch for it. If Ahmed's tablet runs in the older "Wintab" mode with Windows Ink off, the app sees only a mouse and pressure disappears, and no library fixes that. **This must be tested on Ahmed's PC with his tablet**; a cloud thread cannot. The pen's eraser end and palm behaviour come from the same system and also need that test.

**Security and size:** the library only does arithmetic; it touches no files and no network. Add it with an exact version and run `npm audit` when added.

## 4. Global shortcuts (Ctrl+Alt+N for a sticky note, calculator pop-up)

| Option | Licence | What it adds | Notes |
|---|---|---|---|
| **tauri-plugin-global-shortcut 2.4.1** (Rust side only) | Apache-2.0 or MIT | 6 crates, measured: the plugin, `global-hotkey` 0.8.0, `keyboard-types` 0.7.0, and three Windows binding crates | Made by the Tauri team, 5.3 million downloads, released 2026-10-10. Its Windows code uses the standard "register a hot key" call, not a keyboard spy (I read the source), so it cannot see other keystrokes |
| Do it ourselves with the Windows bindings already in the build | n/a | 0 new crates | The call exists in the `windows` crate we already use. Costs roughly 150 lines with a small background thread and unsafe code, and we must handle changing shortcuts at runtime |
| The web-side package `@tauri-apps/plugin-global-shortcut` | MIT or Apache-2.0 | one npm package | **Not wanted.** It would let the screen code register shortcuts. Notaya's rule is that the screen gets no extra powers |

**Recommendation:** the Tauri plugin, used from Rust only, with **no** shortcut permission added to the screen's permission file (`capabilities/default.json` stays as it is). It handles the "Ctrl+Alt+N" text format, press and release, and changing shortcuts while running, which the Settings > Shortcuts rebind screen needs. The newest plugin line (3.0.0-alpha) is excluded by the "stable releases only" rule; 2.4.1 is the stable one.

**Honest limits, true for every option:** Windows lets only one program own a combination, so Notaya must say "this one is taken by another program" when registration fails, and Windows keeps some Win-key combinations for itself. The shortcut must be released when Notaya quits or the vault locks, and it should open nothing sensitive while locked.

## 5. Arabic, Greek and emoji in PDF

**What exists today** (I read `pdf.rs`, `pdf_unicode.rs` and `tools/make-pdf-fonts.py`): Latin text uses the standard PDF fonts (Helvetica, Courier). **Arabic and Hebrew already work**: two small subsets of Noto Sans (37 KB and 9.6 KB, SIL Open Font Licence, no reserved name) are inside the program, with Notaya's own letter-joining and left/right ordering. Chinese, Japanese and Korean come from the fonts Windows already has. **What is missing: Greek and Cyrillic (shown as "?"), and emoji (left out).** So plan item 1.9-3 is really "Greek, Cyrillic, emoji", and the Arabic part is already done and needs only a check with a long Arabic sample.

| Option for Greek and Cyrillic | Licence | Size | Notes |
|---|---|---|---|
| **Noto Sans subset** (Greek, Greek Extended, Cyrillic) | SIL OFL 1.1, no reserved name | est. 40 to 80 KB, not measured | Same recipe, same licence text already in `THIRD-PARTY-NOTICES.md`; the same script extended; works the same on every PC. TTF files come from the `@expo-google-fonts/noto-sans` package (MIT and OFL-1.1) like the two existing ones |
| Windows' own fonts (Arial, Segoe UI), read at export time like CJK | Microsoft's; the code already refuses fonts whose embedding bits forbid it | 0 | Output differs from PC to PC, and a PC without the file falls back to "?" again |
| Liberation Sans | OFL 1.1 **with the reserved name "Liberation"** (confirmed in its licence header) | similar | Looks closest to Helvetica, but a cut-down copy must be renamed. Extra step, extra risk |
| Inter (the app's own font) | OFL 1.1 | similar | Fontsource ships web files only; we would need the upstream TTF. Looks right on screen, but the PDF body is Helvetica, so no gain |

**Recommendation:** the Noto Sans subset. The look of Greek letters will be slightly different from the Helvetica Latin letters around them, as Arabic already is.

**Emoji** are a separate, optional choice: the monochrome **Noto Emoji** font (OFL) would give black-and-white emoji, but the font is large (the npm package is 4.5 MB for several variants; a PDF-only copy needs measuring) and adds that weight to the program. Colour emoji need a picture set (for example Twemoji, whose graphics I recall as CC-BY 4.0, which needs a credit line; **unverified**) and PDF picture handling for each emoji. My suggestion: ship Greek and Cyrillic first; decide emoji later (question 2).

**Upkeep for any new font:** a row in `THIRD-PARTY-NOTICES.md` with copyright line and licence text, an update to `tools/make-pdf-fonts.py`, a line in the About page (1.10-6), and a test with Greek and Cyrillic sample text that checks "no `?`".

## 6. Excel (.xlsx) export for databases

An .xlsx file is a zip of small XML files, exactly like the .docx files Notaya already writes. The program already has its own zip writer (`export/zip.rs`), its own compression (`export/deflate.rs`) and a Word writer (`export/docx.rs`) built on them.

| Option | Licence | Adds | Notes |
|---|---|---|---|
| **Our own writer** on the existing zip and compression | none | 0 crates | Matches the plan ("prefer a small own writer"). About 400 lines. Covers what the agreed feature needs: text, numbers, dates, bold header row, column widths, frozen header, filter buttons, right-to-left sheet for Arabic |
| rust_xlsxwriter 0.102.0 | MIT or Apache-2.0 | 6 crates, measured: itself, `zip` 9.4.2 (MIT), `zopfli` (Apache-2.0), `bumpalo`, `typed-path`, `constant_time_eq` | Very active (released 2026-10-29, 6.3 million downloads), write-only. Adds charts, formulas, conditional formats and tables. Worth it only if we later want real Excel charts (for example dashboards in 1.12) |
| umya-spreadsheet 3.1.1 | MIT | 21 direct dependencies | Reads and writes; much more than needed |
| calamine 0.38.0 | MIT | uses `zip` 9 too | Only reads Excel; relevant only if Ahmed wants **importing** .xlsx |
| JavaScript options (`exceljs` 4.4.0, last released 2023-10; `xlsx` on npm stuck at 0.18.5 since 2022) | MIT; Apache-2.0 | 7 to 22 MB | Rejected: they would run in the screen layer, so the data would leave the Rust core, and both are stale |

**Recommendation:** write it ourselves. **Accuracy (Ahmed's finance data):** write each number from the stored decimal text, never through a rounding detour; keep long ids and account numbers as text because Excel keeps only 15 significant digits; write dates as real Excel dates with a date format, percentages with a percent format. The proof is a test that reads the file back with an independent reader (for example calamine as a test-only tool, or LibreOffice if present) and compares every cell with the source. If charts or formulas come in later, move to rust_xlsxwriter at that point.

## 7. The AI door: MCP over a Windows named pipe

Three pieces, three answers.

**A. The bridge program speaking MCP (the language Claude Code and Codex use).**

| Option | Licence | Weight | Notes |
|---|---|---|---|
| **rmcp 3.9.0**, the official Rust kit of the MCP project | Apache-2.0 | Whole bridge tree with the features we need: 67 crates (18 not already in Notaya), all MIT, Apache-2.0, Unlicense or Unicode terms | 46 million downloads, released 2026-11-05. Follows the protocol as it changes |
| Write the protocol by hand | none | 0 | The basics are small (JSON messages over standard input and output), but the protocol keeps evolving; we would own every change |
| The official JavaScript kit | MIT | 17 packages, needs Node.js on the PC | Rejected: Notaya should not depend on Node being installed |

**Recommendation:** rmcp, inside a separate small program (`notaya-bridge.exe`), not inside Notaya. The bridge should be a dumb translator: **all permission levels, confirmations and the log stay in Notaya**, so a flaw in the bridge's library cannot widen what the AI may do.

**B. The pipe on the Notaya side.**

| Option | Licence | Adds | Notes |
|---|---|---|---|
| **tokio's built-in named-pipe support** + the Windows bindings already in the build | MIT | 0 crates (tokio 1.54.1 is already there; one switch, "net", may need turning on, to check at build time) | Rejects remote clients by default |
| interprocess 2.4.7 | 0BSD or Apache-2.0 | 6 crates, measured | Handy wrapper; it adds little over tokio for one pipe |
| Local network port (localhost) | n/a | n/a | **Excluded** by the plan: any program on the PC could knock on it |

**Recommendation:** tokio plus the Windows bindings. **The important finding:** Microsoft's page on pipe security states that the default rules give "read access to members of the Everyone group and the anonymous account". So Notaya must create the pipe with its own rule (only the current Windows account), refuse remote clients, claim the pipe name first so another program cannot squat on it (`first pipe instance`), accept one connection at a time, and check who connected (the process that opened it) before the pairing code. This belongs in the 1.6 security review (planned promises P30 to P33).

**C. Testing.** A Windows pipe cannot be fully proven in the cloud (Wine has partial pipe support). The real check is a Remote Control run on a throwaway vault, as the project rules require.

## 8. What Ahmed would need to approve, and what gets added to the records

Nothing here is a go. When a version is specced, ask once per item:

| Version | Item | What Ahmed approves | Records that change when it is added |
|---|---|---|---|
| 1.6 | rmcp in the bridge program; pipe rules | A new program and its libraries; the pipe security promises | `THIRD-PARTY-NOTICES.md`, `docs/security-design.md` |
| 1.7 | Tauri global-shortcut plugin (Rust side only) | One plugin, 6 crates | `THIRD-PARTY-NOTICES.md` |
| 1.9 | Noto Sans (Greek, Cyrillic) subset; optional emoji font | One font file (OFL); emoji only on his word | `THIRD-PARTY-NOTICES.md`, `tools/make-pdf-fonts.py` |
| 1.9 | Own xlsx writer | none (no library) | docs only |
| 1.13 | perfect-freehand 1.2.3 | One npm package | `THIRD-PARTY-NOTICES.md`; vault-format change is a separate gate |

Every addition: exact version (`save-exact`), stable release only, `npm audit` or a first `cargo audit`, a security look, and the licence text recorded for the About page.

## 9. Questions for Ahmed (numbered, plain)

1. What is your drawing tablet (brand and model), and does its driver have a "Windows Ink" switch that is on? If it only works in the old mode, pressure will not reach Notaya and we need a different plan.
2. Emoji in PDF: leave them out as today, black-and-white (about a megabyte or more added to the program), or colour pictures (more work, a credit line)? My suggestion: leave out for now.
3. Greek and Cyrillic in PDF will look a little different from the Latin text beside them. Fine?
4. Do you foresee wanting real Excel charts or formulas in exports? If yes, we choose `rust_xlsxwriter` from the start; if not, our own small writer.
5. Do you want Excel files **imported** too? That would add the reader library `calamine`.

*Sources, all read on 2026-11-07:* npm registry and crates.io entries for each package named above; tldraw `LICENSE.md` (GitHub); Microsoft Learn: [Named Pipe Security and Access Rights](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights) and [CoreWebView2PointerInfo](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2pointerinfo); the `global-hotkey` Windows source (GitHub); the Liberation Fonts `LICENSE` header (GitHub); the repository files `pdf.rs`, `pdf_unicode.rs`, `tools/make-pdf-fonts.py`, `THIRD-PARTY-NOTICES.md`, `Cargo.toml`, `Cargo.lock`, `capabilities/default.json`.

---

# Addendum, 2026-11-08 00:05 UTC: Ahmed's answers and what changes

**Source:** Ahmed's message of 2026-11-07 23:58:44 UTC (relayed by the coordinator). His words, in short: do not tie anything to one tablet, support tablets broadly; emoji in PDF "should be there" and PDF export is critical; Excel exports should include charts "if possible", otherwise plain tables; Excel import into databases is wanted. His reply to question 3 (Greek and Cyrillic look) was not recorded; the study's recommendation stands until he says otherwise. **This addendum replaces the study's recommendations 3 (emoji), 4 (Excel) and the tablet advice. The text above stays as written.** Added by Claude (Sonnet 5.5, high), cloud thread; same checking method and the same list of things not checked as in section 2 (plus: the picture sets below were not downloaded or measured).

## A. Result table (replaces rows 3 and 4 of the short answer)

| Need | Now recommended | Licence | Added to the program |
|---|---|---|---|
| Whiteboard on any tablet | Same: `perfect-freehand` plus pointer events, now with a broad-support design (section B) | MIT | 1 npm package |
| Emoji in PDF | A bundled **emoji picture pack** (colour), looked up by code-point sequence. First choice: Microsoft's open Fluent Emoji (MIT); fallback: Google's Noto emoji pictures (Apache-2.0) | MIT / Apache-2.0 | one data pack, size not measured (see C) |
| Excel export with charts | **`rust_xlsxwriter` 0.102.0** (this reverses the study's "write our own") | MIT or Apache-2.0 | 6 crates |
| Excel import into databases | **`calamine` 0.38.0** feeding the CSV import path that already exists | MIT | about 10 crates |
| Both Excel pieces together | | | **16 new crates in total, measured**, all permissive |

## B. Tablets: broad support, no model-specific code

Ahmed does not want this tied to his tablet, so the design is "any pen that Windows itself understands":

1. **One path for all pens:** the standard pointer events. Windows Ink pens and tablets reach the web engine through them with pressure, tilt and the eraser end; so do Surface-type pens and touch screens. Mouse is the fallback. No code names a brand. (I believe the common tablet makers' drivers, such as Wacom, Huion, XP-Pen and Gaomon, all offer a "Windows Ink" switch. I did not verify this here.)
2. **Honest pressure detection:** if a pen reports a changing pressure, use it. If it reports a flat value (a mouse, or a driver that does not pass pressure on), switch automatically to `perfect-freehand`'s speed-based "simulated pressure" so lines still look natural.
3. **Addition, a "Tablet check" in Settings:** a small panel that shows live what the app receives (pen or touch or mouse, pressure bar, tilt, eraser end, side button). If the pressure bar does not move, it says to turn on Windows Ink in the tablet's own driver. This turns an invisible driver problem into something a non-developer can see, and it lets Ahmed test any tablet, including a friend's.
4. **Settings that help every device:** a pressure curve (soft to firm), smoothing strength, eraser-end and side-button choices, and palm rejection (ignore finger touches while a pen is near).
5. **The one gap:** a tablet that works only in the old "Wintab" mode and has no Windows Ink option is invisible to the web engine. Supporting it would mean reading the tablet in the Rust part and feeding the canvas, a separate and sizeable job. It is **not planned**; revisit only if the Tablet check shows real people hitting it.
6. **Testing:** only on real hardware. Ahmed's tablet is one data point; a touch laptop or a second tablet would be a better one. A cloud thread cannot test any of it.

## C. Emoji in PDF: the options, with a pick

The PDF writer cannot draw colour emoji today (emoji are left out). Emoji are tricky because many are *sequences* (a family is several emoji joined by an invisible joiner; a skin tone, a flag and a keycap are sequences too). A **picture pack named by code-point sequence** handles all of these with a plain table lookup (longest match first). A font would need glyph-shaping code we do not have.

| Option | Licence | Look | Weight and effort | Verdict |
|---|---|---|---|---|
| **Fluent Emoji** (Microsoft; colour or flat set) | MIT, "Copyright (c) Microsoft Corporation" (checked in the repository's `LICENSE`) | Windows 11's own emoji style, as I believe; so the PDF would match what Ahmed sees on screen (not verified) | Source is SVG; a build-time script would turn it into small PNGs. The npm package `fluentui-emoji` is 120 MB because it holds every style, so we would take only one | **First choice** |
| **Noto emoji pictures** (Google) | Pictures: Apache-2.0; the font: OFL 1.1 (read in the repository's README and LICENSE) | Android-style | Ready-made PNGs named by code points; no conversion tool needed | Good fallback, and the quickest to try |
| Twemoji | Code MIT, **pictures CC-BY 4.0** (checked): a visible credit is required | Twitter style | SVG, the npm package only goes up to Unicode 15 | Works, but the credit rule and old coverage make it third |
| Windows' own Segoe UI Emoji, read at export time | Microsoft's font; the app already refuses fonts whose embedding bits forbid use | Exactly the screen look | Zero size, but colour glyphs are drawn in layers and sequences need shaping code; differs between Windows 10 and 11; embedding permission unchecked | Test later on Ahmed's PC; not the main path |
| Noto Emoji, black and white font (OFL) | OFL 1.1 | Monochrome | The font is large and sequences again need shaping | Only as a last-resort fallback for an emoji missing from the pack |

**Size:** a full colour pack is roughly 3,700 pictures. At a few KB each that is **an estimate of 10 to 20 MB, not measured**. Ways to keep it light: small pictures (about 64 to 96 px, enough for a PDF line), a reduced colour palette, and shipping the pack as a data file that is only read when a PDF contains emoji, not inside the program's main code. Measuring it is the first job of the 1.9 PDF work.

**Records:** add a MIT credit (Microsoft) to `THIRD-PARTY-NOTICES.md` and to the About page. The product name "Fluent" belongs to Microsoft; per Ahmed's no-trademarked-names rule it appears only in the credits, never as a name in Notaya's screens.

## D. Excel: charts and import

**Export with charts.** `rust_xlsxwriter` can write native Excel charts. I read its source: it offers column, bar, line, pie, doughnut, area, scatter, radar and stock charts (plus stacked variants), and it can set a worksheet right-to-left. Notaya's database charts today are bar, line, donut and a "number" tile (`database/chart.rs`), so: bar becomes a column or bar chart, line a line chart, donut a doughnut chart, and the number tile stays a cell. Dashboards (2.x line) can reuse the same path. Plan: one sheet per exported view with the table, the chart beside it, and the right-to-left setting for Arabic content. The earlier "own writer" plan could not reasonably produce charts, so this reverses it.

**Safety and accuracy on export:** always write text as text (never as a formula), so a cell that starts with `=` cannot run; numbers are written exactly as the database holds them (the database keeps them as ordinary decimal numbers, the same as Excel does, which keeps about 15 significant digits); dates as real Excel dates; long ids and account numbers as text. Verification by reading the file back with `calamine` in a test and comparing every cell.

**Import into databases.** `calamine` 0.38.0 (MIT) reads .xlsx, .xlsm, .xls, .xlsb and .ods. It tells dates apart from numbers and can also read the formula text. The existing CSV import already has everything after "read the cells": a `Table` of rows of text, column-type guessing, and `build_from_table` for a new database or into an existing one. So Excel import is a thin front end: read a sheet, turn dates into the same text form, hand the `Table` over. Proposal: when a workbook has several sheets, show the list and let each chosen sheet become one database.

Rules for the import (security and accuracy):
- **Same caps as CSV:** at most 100,000 rows and 100 columns per sheet. calamine itself only logs a warning above Excel's own limits, and I found no guard against a "zip bomb" (a tiny file that expands hugely). So the app must check the file size and the declared unpacked size *before* reading, and refuse above a limit (I suggest 200 MB; Ahmed's call).
- **Macros are never run.** calamine only reads. A macro-enabled file is imported as plain data and the report says macros were ignored.
- **Formulas:** import the last calculated value into the cell; the formula text can be kept in a report or a note column. Notaya's formula language differs from Excel's, so converting is a separate, later feature. Where a formula's value is missing, the report says so; no guessing. For the finance databases this matters: the import report must list what was not carried over and the counts must match.
- **Links, pictures, charts inside the workbook** are not imported in the first version; the report says so.

**Cost and licence:** together with the writer the measured addition is 16 crates: `rust_xlsxwriter`, `calamine`, `zip` 9.4.2 (MIT, shared by both), `zopfli` (Apache-2.0), `bumpalo`, `typed-path`, `constant_time_eq` (CC0 or MIT-0 or Apache-2.0), `atoi_simd`, `codepage`, `core_detect`, `debug_unsafe`, `encoding_rs` (Apache-2.0 or MIT, with BSD-3-Clause for part of its data), `fast-float2`, `multiversion_no_op`, `rustversion`, `simdutf8`. All permissive; `encoding_rs` is the only one with a second licence layer, and its text goes in `THIRD-PARTY-NOTICES.md`.

**Placement (for the coordinator):** both Excel pieces belong with the tabs-and-files version (previously 1.9) next to "import into place" and "re-import"; the dependency approval for both is one request to Ahmed at that spec.

## E. What is still open, and what Ahmed needs to approve

- Open and small: how large a workbook may be (suggested 200 MB); whether several sheets become several databases (suggested yes, with a pick list); Greek and Cyrillic look (no answer recorded).
- Approvals remain per version, as in section 8, now with these changes: the Noto Sans subset stays for Greek and Cyrillic; the emoji pack replaces "optional emoji font"; `rust_xlsxwriter` and `calamine` replace "own xlsx writer".
- **Not run in this addendum:** any build or test (no code changed); a download or size measurement of any emoji set; `cargo audit`; a check of calamine's or rust_xlsxwriter's behaviour on real Excel files; Segoe UI Emoji's embedding bits or colour format; any tablet.
