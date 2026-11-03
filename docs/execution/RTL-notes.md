# Right-to-left text (Arabic, Hebrew): notes

Branch work for the owner's request (2026-11-03): "if arabic import from other thing will be in the left side not right like english"; "if i imported library or backup the arabic stays right and english left, with the export as well". Baseline: `origin/main` `ee8c32e` (V1.4.0). Result state: **produced** (not merged, not accepted). Verification is listed below, with what was not run.

This file is a working note for the lead, not the 4-section or 12-section Execution Summary and not an index row; the lead folds it into the records when the work is integrated.

## What changed

**The rule ("automatic").** A text block (paragraph, heading) runs in the direction of its first strong letter: Arabic, Hebrew, Syriac, Thaana, N'Ko, Samaritan, Mandaic letters (and an explicit U+200F mark) are right-to-left; any other letter, or U+200E, is left-to-right. Digits, punctuation, emoji and spaces are skipped; a block with no strong character is left-to-right. The same rule is written twice and kept in step: `first_strong` in `src/features/notes/editor/direction.ts` and in `src-tauri/src/export/doc.rs`.

**Stored choice.** Paragraphs and headings get an optional attribute `dir` in the page JSON: `"ltr"`, `"rtl"` or `null` (automatic). It travels in the vault, backups and exports. Pages saved by older versions have no `dir`, so they are automatic: no migration. (The editor's text-align extension has `defaultAlignment: null`, so older pages do not carry a stored "left" either.)

**Editor** (`src/features/notes/editor/direction.ts`, new; `extensions.ts`, `Ribbon.tsx`, `BubbleToolbar.tsx`, `SlashMenu.tsx`, `ImageBlock.ts`, `more-blocks.ts`, `src/styles/app.css`):
- A TipTap extension `Direction` adds the `dir` attribute (rendered as `dir="auto"` when null, so copied HTML decides by text) and draws the real direction with node decorations. Containers (list items, lists, quotes, highlight boxes, collapsible sections and shelves, tables) take the direction of their first text block. Decorations were used because the browser's own `dir="auto"` on a container skips children that carry a `dir` attribute, so a list could never learn it was right-to-left. An empty line takes the direction of the line before it, so the caret does not jump sides.
- CSS uses logical sides: indent on `li` (`margin-inline-start`) instead of on `ul`/`ol`, so each item's bullet or number goes to its own text's side; quote bar, shelf padding, task check boxes, highlight-box icon (flex order), collapsible arrow (mirrored when closed), table columns, placeholder (`float: inline-start`). `text-align: start`, so alignment follows direction; an explicit alignment (the ribbon's left/center/right/justify, an inline style) still wins. Code blocks stay left-to-right.
- Controls: Home tab, new "Direction" group (right-to-left, left-to-right; clicking the active one goes back to automatic); "/" menu entries "Right-to-left text", "Left-to-right text", "Automatic direction"; the selection toolbar's align pop-up has a second row (LTR, RTL, Auto). They set `dir` on every paragraph and heading in the selection (table cell selections included).
- `dir="auto"` was added to the page title field, the breadcrumb, the sidebar rows, the command palette (input, titles, snippets), the home cards, the page picker, the export dialog's page list, picture captions and page-link titles. Sticky notes use the same extension.

**Rust document model** (`src-tauri/src/export/doc.rs`): new `Dir { Auto, Ltr, Rtl }`, `first_strong`, `explicit_dir`, `content_dir`, `blocks_dir`. To leave the shape of `Block` unchanged (so `export/pdf.rs`, which another helper is editing, compiles untouched), a text block's explicit direction is carried on the `dir` field of `Marks`, repeated on each text run of the block; the first run that has one decides. `Marks` is `Default`, so `dir` is `Auto` everywhere that does not set it. The importers' own model (`importers/doc.rs`) got the same `dir` on its `Marks`.

**Exports.**
- HTML: `dir="rtl"|"ltr"` on a paragraph, heading, list item, table cell or summary line that has an explicit direction, else `dir="auto"` (as before); a list, quote, highlight box, collapsible section or table whose first text runs right-to-left gets `dir="rtl"` (so bullets, the bar and the columns go to the right), otherwise nothing changes from before. The callout icon's margin is logical. CSS text alignment already followed direction (`start`).
- Word: already wrote `w:bidi` and `w:rtl` for text whose first letter was Arabic/Hebrew; now it follows the explicit choice too, also for task items (they were always left-to-right before), and writes `w:bidiVisual` (after `w:tblStyle`, as the schema orders it) for a table whose first text is right-to-left. Alignment: under `w:bidi`, Word's `start` is the right side, so a right-to-left paragraph with no explicit alignment gets no `w:jc` and is right-aligned; an explicit "right" on a right-to-left paragraph also needs no `w:jc`; "right" on a left-to-right paragraph is `w:jc right`.
- Markdown: unchanged (plain text; the text gives the direction on import).
- PDF: not touched (`pdf.rs`, `pdf_fonts.rs` belong to the PDF-fonts helper).

**Imports** (`importers/html.rs`, `importers/docx.rs`, `importers/doc.rs`; Markdown, text, PDF text, Notion zips and folders and Notaya's own exports need nothing and stay automatic):
- HTML: `dir` on the element or an ancestor, CSS `direction:` in `style`, and a right-to-left `lang` (ar, he, fa, ur, ps, sd, ug, yi, dv, ckb, ku) set the direction of the text inside; it passes down. `rtl` is kept wherever it is written. `ltr` is kept only when written on a text element itself (p, h1-h6, li, td, th, summary, caption, figcaption, dt, dd): on a wrapper (`html`, `body`, `div`, `ul`, `table`) it is only a default, because many pages carry `<html dir="ltr">` and the owner wants their Arabic on the right. `auto` resets. A direction on an inline element (`span`, `bdi`...) does not turn the paragraph.
- Word: `w:bidi` on a paragraph, `w:bidiVisual` on a table (its first text is made right-to-left so the editor draws the columns from the right). `w:jc` was and is not read.
- The importers store `dir` on a block only when it differs from what the block's text would give by itself (an Arabic paragraph marked rtl stores nothing, English marked rtl stores `"rtl"`), so imported pages stay automatic where possible.

## What was tested

- `cargo test --lib --target x86_64-pc-windows-gnu` under Wine: **392 passed, 0 failed** (the main checkout's baseline suite was 380; 12 tests added: first-strong rule, old page without `dir` loads as automatic, stored `dir` overrides text, HTML export direction attributes and right-to-left boxes/lists/tables/shelves, Word paragraph/run/table/task item output including `w:jc`, HTML import rules (nesting, `lang`, CSS, weak `ltr`), Word import (`w:bidi`, `w:bidiVisual`), and the export-import round trip `arabic_hebrew_and_english_keep_their_direction` for Markdown, HTML and Word).
- `npx tsc --noEmit -p tsconfig.json`: clean. `npm run build`: built.
- `dev-mock/rtl-ui-check.cjs` (new, Playwright/Chromium against `npm run mock`, `?rtl=1` shows a mixed Arabic/Hebrew/English page): 31 checks, all OK: an old page without `dir` stays left-to-right; Arabic/Hebrew paragraphs, headings, bullets, quote, highlight box, shelf and table run right-to-left and sit on the right; English stays left; an explicit "align left" still wins; typing Arabic flips the line; the ribbon, "/" menu and selection toolbar force and reset the direction; select-all + left-to-right flips the bullets back; the page title field is `dir="auto"`. `dev-mock/ribbon-ui-check.cjs` (the existing ribbon check) still prints ALL OK. Screenshots (1400 px wide, scale 1) are in `/mnt/project-files/notaya-build/v150-rtl/`.

## Known limits

- Markdown and PDF carry no explicit direction: a block forced the other way from its text comes back automatic from Markdown. Word cannot say "left-to-right" for Arabic text (no `w:bidi` is how every Word file says it), so a forced left-to-right Arabic paragraph comes back right-to-left. HTML carries both.
- The model cannot tell an explicit left alignment from the default, so an Arabic paragraph set to "align left" is exported right-aligned. In the editor it stays left (the inline style wins).
- A paragraph with no strong letter (only digits or symbols) is left-to-right, in the editor and in the exports; an empty line takes the previous line's direction in the editor only (it has no visible effect in an export).
- A foreign HTML or Word file whose Arabic text is explicitly left-to-right (`dir="ltr"` on the paragraph itself) is kept left-to-right; only the page-level default is overridden.
- `export/pdf.rs` still draws every line left-to-right (outside this change).

## Not run

- The real app on the PC (Remote Control), a real Word document opened in Word, and a real browser opening the exported HTML: not run. Word's behaviour (`w:bidi` + no `w:jc` = right-aligned, `w:bidiVisual`) follows the OOXML schema and Word's documented behaviour; it was not checked in Word.
- `cargo clippy` for MSVC: not run (clippy ran for the windows-gnu target; the files touched had no new warnings after one collapsible `if` was fixed).
- `npm audit`, the PC exe and the real-exe scripts: not run.
- No security promise changed (no key, storage, IPC, CSP, clipboard or auto-lock code was touched), so `docs/security-design.md` is unchanged.
