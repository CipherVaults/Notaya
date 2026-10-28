# A33: Other V1.4.0 features

**Status:** requested by the owner on 2026-10-28 21:42:03 UTC ("Calculator history increases to 500 calculations / Adding highlights to texts matted colors / Can change text color / ... exported at different formats Markdown - PDF- word file"), 21:43:04 ("Multiple colors for highlights and text colors") and, as an answer to a question, 21:57:38 ("Yes and pdfs as well": import of Notion exports, Markdown, HTML, Word and PDF). Built by helper agents in separate working copies and merged by the main thread. Not accepted.

(Details, files and verification are added below when each part is merged.)

## Calculator history of 500 (merged)
`CALC_HISTORY_MAX` in `src-tauri/src/model.rs` is 500 (was 200); the newest are kept, the oldest dropped. The history lives in the encrypted settings record: 500 entries of at most 2,000 characters is about 1 MB, far under the 32 MB item limit. The list already scrolled and "Clear" works. Earlier records that say 200 ([acceptance.md](../acceptance.md) X22, STATUS) describe what was true when written.

## Text colour and highlight (merged)
Text colour and highlight were one small pop-up; they are now two buttons in the ribbon and in the floating toolbar. Highlight offers nine matte colours (Stone, Clay, Peach, Butter, Sage, Mist, Lavender, Blush, Brick) and text colour nine mid-tone colours, each palette with a way to remove it ("Default text color", "No highlight"). Swatches have labels, the current choice is shown, and the arrow keys move between them. Highlights are saved as `data-color`. Contrast of the text colours against the six themes was computed: at least 3.2:1 on light and 4.1:1 on dark themes. Older saved colours still display. Sticky-note editors keep the plain highlight (not changed). Verified in the mock UI on five themes with screenshots and scripted clicks; **not run** in the real app.
