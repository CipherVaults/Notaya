# Import: what carries over (plain list for the owner)

Status: built in a cloud thread, **produced and tested with made-up sample files only**. No real Notion export, no real Word file and no real exe run were available, so every statement below says what the code does and which test shows it. "Not verified" means exactly that. This file is a description for you, not an amendment; the import is not accepted until you say so.

## How it works in one minute

- **Import pages…** is in the sidebar (hover "Notebooks", the upload button), in the empty sidebar ("Import pages from another app") and in the command palette.
- One file picker: you can select **many files at once and mix kinds**: Markdown (`.md`), plain text (`.txt`), HTML, Word (`.docx`), PDF and **ZIP files** (Notion exports, one or several, for example `Part-1` and `Part-2`). A second choice, **A folder**, imports every supported file inside it.
- The result is **one new notebook**. Your existing notes are never changed. (2026-11-03: a ZIP that Notaya itself exported from several notebooks comes back as **one new notebook for each**; see "Notaya's own ZIP exports" below.)
- Notebook name: the folder, the ZIP file or the single file you chose (a generic name such as "Export-…" becomes "Notion import"); several loose files give "Imported <date>".
- Sections: each top-level folder becomes a section; loose pages go into a section called "Imported pages".
- Pages: named from the page's own title (first level-1 heading, `<title>`, Word title), else from the file name. Notion's 32-letter id at the end of names is removed.
- Sub-pages: in a Notion export a page `X.md` and a folder `X/` next to it means the pages inside `X/` become sub-pages of X. Any other folder inside a section becomes an empty page named after the folder that holds its pages.
- Everything is saved in one step (all or nothing). If anything fails, nothing of the import is left (tested, including pictures already stored).
- At the end a screen shows the counts and the list of things that were left out, with the reason.

## From Notion: HTML export versus Markdown export

Choose **HTML** when you export from Notion. The Markdown export has no colors at all.

| | HTML export | Markdown export |
|---|---|---|
| Text colors and highlight colors (the 9 Notion colors, as text color or background) | Kept, written as Notaya's own palette colors (Ash, Cocoa, Amber, Honey, Moss, Denim, Plum, Orchid, Cherry) | Not in the export, so not imported (a note appears after the import) |
| Colored blocks (whole paragraph, heading, callout, table cell) | Kept as the same colors on the text inside | Not available |
| Page font | Serif and Mono pages stay Serif and Mono; Notion's default (sans) page uses Notaya's default font | Not available |
| Page icon (emoji) | Becomes the page icon | Not available |
| Page cover picture | Becomes the page cover | Not available |
| Headings, bold, italic, underline, strikethrough, inline code, links | Kept | Kept (underline only if Notion wrote it as `<u>`) |
| Bulleted, numbered (with its start number) and to-do lists | Kept, checked state kept | Kept, checked state kept |
| Callouts (with the emoji) | Kept | Kept (Notion writes them as `<aside>`) |
| Toggles | Kept, shown open | Kept, shown open; content after a blank line stays inside the toggle |
| Quotes, code blocks (language kept), dividers | Kept | Kept |
| Tables | Kept, first row as header; cell colors kept | Kept (no colors) |
| Columns | Kept (2 to 5 columns; other counts are put one under the other) | Not available |
| Equations (block and inline) | Kept as TeX equations | Kept as TeX equations (`$$ … $$` and `$ … $`) |
| Pictures | Stored in your notes (encrypted), with the caption | Stored in your notes |
| "Link to page" blocks | Become page links to the imported page (if that page is in the import) | Same, when the link sits alone in its paragraph |
| A page mentioned inside a sentence | Text only, the link is dropped | Text only, the link is dropped |
| (Correction 2026-11-03 to the row above: a link inside a sentence to another page **of the same import** becomes a Notaya link to that page, in HTML and Markdown; this was already so, shown by `a_link_inside_a_sentence_to_another_imported_page_becomes_an_internal_link`. A link to a page that is not in the import is text only.) | | |
| Sub-pages (nested folders) | Kept as sub-pages | Kept as sub-pages |

Where this is shown by a test: HTML export in `importers/html.rs` (`a_notion_html_page_keeps_its_formatting`, `notion_page_fonts_map_to_serif_mono_or_the_default`) and in `importers/plan.rs` (`a_notion_html_zip_keeps_folders_ids_pictures_colors_and_links`); Markdown in `importers/markdown.rs` and `plan.rs` (`markdown_folders_become_sections_and_notion_markdown_nests`). The sample pages were written by hand from what we know of Notion's export format. **Not verified against a real Notion export:** the exact class names Notion writes (we also accept "teal" as Notion's name for green), how Notion writes toggles in Markdown, and anything Notion changed recently.

### Approximated

- Notion's colors are matched **by name** to the nine Notaya colors, not by looking at the shade. An unknown color name is dropped and the text kept.
- A bookmark becomes a line "🔖 title" that is a link (the preview card is not kept).
- Notion's "Default / Serif / Mono" fonts: Serif maps to Notaya's Serif (Georgia), Mono maps to Notaya's Mono (JetBrains Mono), Default (sans) sets nothing, so the page uses whatever font you chose in Notaya. Notaya's Sans, Handwriting and Rounded are never chosen by an import. The font is put on the text, not on the page, so text you type later has your normal font.
- A toggle's header is the first line; its content is shown open.
- A Notion callout's color becomes a highlight on its text, not a colored box.
- A page's properties (database rows) come as the plain lines Notion wrote.
- A heading level above 4 becomes level 4 (Notaya has headings 1 to 4 in the editor).
- A picture that is not PNG, JPEG, GIF or WebP (for example HEIC or SVG) is not imported; a short note "[Picture not imported: name]" stays in the page and the picture is listed as left out.
- Pictures on the internet stay as links ("Picture: name"); nothing is ever downloaded.

### Does NOT carry over

- **Databases and table views** (the `.csv` files): listed as left out. The pages inside a database, if Notion exported them as pages, are imported as pages.
- Comments, page history, who edited what, permissions, sharing links.
- Synced blocks: not verified (Notion may export them as plain copies; whatever it writes is imported as normal content).
- Embeds, videos and audio: frames, video and audio tags are dropped without a note; if Notion also wrote a plain link, the link stays. Not verified against a real export.
- Formulas, relations, rollups (these live in databases).
- Files attached to pages: a PDF or Word attachment inside the export is imported **as a page of its own** (text only); other attachment types are listed as "not imported".
- Notion's top-level `index.html` (a list of links) is skipped and listed.
- Page width, small text and similar layout settings; text sizes.

## Markdown files (and folders of Markdown)

Carried over: headings (`#` and underlined), bold, italic, strikethrough, `==highlight==`, inline code, links, automatic links, task lists, bullet and numbered lists (nested), quotes, code blocks with language, tables, dividers, `$ math $` and `$$ math $$`, hard line breaks, HTML pieces inside the file (cleaned like HTML), GitHub alerts (`> [!NOTE]`) as callouts, the `title:` of a front matter block. Pictures are taken from the files next to the document (relative paths only). Plain `.txt` files keep their line breaks and blank-line paragraphs; nothing is interpreted as formatting. (This differs from "treat .txt like Markdown" on purpose: stars and hashes in a text file are not formatting.)
Not converted (they stay as plain text, or are dropped): footnotes, definition lists, other front matter keys, wiki links (`[[…]]`); pictures given with an absolute path or a web address (kept as links).

## HTML files

Same converter as Notion's HTML. Only a fixed list of elements and marks is kept; scripts, styles, frames, forms, SVG, event handlers and every other attribute are dropped (tested with hostile input). Bold, italic, underline and strikethrough also come from `style` attributes (Google Docs, Word-made HTML). Colors from arbitrary CSS are not imported (only Notion's color classes). Links keep only http, https and mailto.

## Word (.docx)

Carried over: headings 1 to 4 (5 to 9 become 4), the Title style as the page title, bold, italic, underline, strikethrough, highlight color (the Word highlight, or a run's shading fill as Notaya's own Word export writes it, matched to the nearest palette color; gray and near-white shading and paragraph or cell shading are ignored), text color (kept as the Word color), bullet and numbered lists with nesting, tables with merged columns, links (http, https, mailto only), pictures (PNG, JPEG, GIF, WebP), line breaks, quotes. Shown by the test `a_word_document_keeps_its_structure` on a hand-made `.docx`.
Not carried over: tracked deletions, comments, footnotes and endnotes, headers and footers, text boxes (their text may appear in reading order but not their position), fonts and sizes, pictures in EMF/WMF format (listed as left out), `.doc` files (old format: the message asks you to save as `.docx`). **Not verified with a file made by Microsoft Word itself.**

## PDF

**Text only.** Carried over: the text in reading order; lines in a clearly larger font become headings; lines are joined into paragraphs. Not carried over: pictures, scanned pages (no text recognition), tables and columns as such, links, page layout. A PDF with a password or restrictions is refused with a message; a PDF whose fonts have no readable character map is refused with a message; a PDF with pages that have no text imports the others and lists the count. Shown by tests on hand-made PDFs; **not verified with PDFs from real programs** (fonts and encodings vary a lot, so expect some PDFs to come out wrong or empty).

## Notaya's own ZIP exports (added 2026-11-03)

A ZIP made with **Export → Separate files in a ZIP** now goes out and comes back with its structure and its links:

- **The marker file.** Every such ZIP holds `notaya-export.txt`. Its first line is `Notaya export v1`, the second says the layout (`layout: notebooks`, `sections` or `pages`), then one sentence saying what the file is for. It holds **no names and no note content**. The import does not turn it into a page.
- **Notebooks.** With `layout: notebooks` and folders of that shape (only folders at the top, only section folders inside each, pages inside each section folder), each top folder becomes **its own new notebook** named after the folder, and each folder inside it a section. Two notebooks with 7 pages come back as 2 notebooks with the same sections and 7 pages (shown by the round-trip matrix test, which now checks the notebook, section and page counts of every case). The app opens the first new notebook; the closing screen names all of them. Everything is still saved in one step (all or nothing), pictures stay held until then, Cancel and the 15-minute limit still work, and the 5,000-page limit counts all notebooks together. At most 100 notebooks per import: a ZIP with more top folders is imported as before (one notebook, the folders as sections).
- **Sections.** With `layout: sections`, each top folder is a section, also when there is only one (before, a single folder was taken as a wrapper and its pages went into "Imported pages").
- **Pages.** With `layout: pages` the import works as before.
- **Without the marker** (Notion, any other ZIP or folder, or a Notaya ZIP from before this change), or when the folders do not have the shape the marker names, or when the marker is unreadable, too large (over 4 KB) or has another first line, the import works exactly as before. A marker inside a folder is not the ZIP's marker.
- **Links between pages.** A link to another page and the "Link to page" row are exported as real links **when the other page is in the same export**: in a ZIP as a relative link to that page's file (Markdown `[Title](Other%20page.md)`, HTML `<a href="Other%20page.html">`, Word a hyperlink to `Other page.docx`); the "Link to page" row is written as `→` and the linked title. The import turns these back into links to the new pages, and a `→` row back into a "Link to page" row. Names with spaces, `#`, `%`, `&`, parentheses and non-English letters, sub-page folders and `../` across notebooks are tested.
- **One file.** In one HTML file a link to a page of the same export jumps to that page's heading (`#notaya_page_2`); in one Word file it jumps to a bookmark at that page's title. One Markdown file and PDF keep the link's text only (heading anchors differ between Markdown viewers, and the PDF writer has no internal links). Importing one combined file gives one page, so these links come back as text.
- **Not carried:** a link to a page that is **not** in the export is written as its text (never as a `notaya:` address, never as a dead link); a link to a heading goes to the page's file, not the heading; PDF files in a ZIP do not link to each other (PDF import is text only anyway); the order of notebooks, sections and pages comes back sorted by name, not in the sidebar order; two notebooks whose names differ only by capitals come back as `Name` and `name (2)`.

## Safety and limits

- Everything is read on this computer; nothing is fetched from the network. Links on disk (symbolic links, junctions) are not followed. Hidden files are skipped.
- A document can only use pictures that are inside the chosen folder, the folder of the chosen file, or the ZIP file. Paths that climb out (`../`), absolute paths, drive letters and network paths are refused (tested).
- Links keep only http, https and mailto (`javascript:` and similar are dropped; tested).
- Notaya's own files (`.nota`, `.clf`) and the data folder are refused as a source (tested).
- Limits: 5,000 pages per import (more is refused with a message), 5,000 pictures (further pictures are left out and listed), 20 MB per Markdown/HTML/text file, 100 MB per Word/PDF file, 25 MB per picture, 1 GB of picked ZIP files together, 2 GB of content in total (one allowance shared by ZIP pieces, Word and PDF files), 1 GB of pictures. ZIP files get zip-bomb and path checks (with checked arithmetic on every offset and size); a ZIP inside a ZIP is opened one level (Notion's "Part" files), deeper ones are listed as not opened.
- Tables: at most 100 columns, 20,000 cells per table and 100,000 per page (the rest is cut, with a note); a column span is clamped to the column limit. A page is limited to 20 MB, and the writer stops a page early when its estimated size passes 48 MB while it is written; such a page is left out and listed as too large, and the other pages are kept. Word tables nested deeper than 8 levels are read flat; nesting of lists, links and content controls is capped. Markdown stops at 3 million lines per file, and brackets and parentheses in a line nest at most 32 deep.
- PDF: strings up to 1 MiB, character maps up to 8 MiB and 100,000 entries (a mapping to more than 64 characters is cut), 1 million objects, 48 MB of text per object, 32 MB of text per file, 20,000 form objects. These keep a small hostile file from growing into gigabytes.
- Time and cancel: an import that runs longer than 15 minutes is stopped with a clear message and adds nothing. The import dialog has a "Cancel import" button; cancelling (or locking the vault, or the vault locking itself) stops the import at the next check and takes back everything it stored, so nothing visible is left. Checks run inside every long loop of every converter, so a stop takes effect within seconds, not at the end of a file.
- Pictures that import stored are held, so the vault's cleanup cannot remove them before the pages that use them are saved. If the vault locks in the middle, the import notices (the hold is gone), stops with the "locked" message and removes its pictures.
- A picture that sits inside the data folder is refused as a source of pictures in "Files" mode as well (a document placed above the data folder cannot point into it).
- Damaged or hostile files give an error or a plain page and never stop the program (tested with truncated and randomly damaged copies of each format, an unclosed PDF hex string, 50,000 random PDF token mixes, giant column spans, millions of nested Word tables, a PDF character map that maps to long strings, and quadratic-time probes for PDF objects, Markdown lists and HTML raw-text tags, each with a time limit). Not tested: very large real-world files, and release behavior of the real exe (the tests ran in debug builds under Wine).

## Not run

The real exe, the native file dialogs, Microsoft Word, a real Notion export, a real PDF from a program, importing on the owner's PC. The screens were checked in the mock UI only (light and dark).
