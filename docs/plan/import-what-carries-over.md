# Import: what carries over (plain list for the owner)

Status: built in a cloud thread, **produced and tested with made-up sample files only**. No real Notion export, no real Word file and no real exe run were available, so every statement below says what the code does and which test shows it. "Not verified" means exactly that. This file is a description for you, not an amendment; the import is not accepted until you say so.

## How it works in one minute

- **Import pages…** is in the sidebar (hover "Notebooks", the upload button), in the empty sidebar ("Import pages from another app") and in the command palette.
- One file picker: you can select **many files at once and mix kinds**: Markdown (`.md`), plain text (`.txt`), HTML, Word (`.docx`), PDF and **ZIP files** (Notion exports, one or several, for example `Part-1` and `Part-2`). A second choice, **A folder**, imports every supported file inside it.
- The result is always **one new notebook**. Your existing notes are never changed.
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

Carried over: headings 1 to 4 (5 to 9 become 4), the Title style as the page title, bold, italic, underline, strikethrough, highlight color (matched to Notaya's palette), text color (kept as the Word color), bullet and numbered lists with nesting, tables with merged columns, links (http, https, mailto only), pictures (PNG, JPEG, GIF, WebP), line breaks, quotes. Shown by the test `a_word_document_keeps_its_structure` on a hand-made `.docx`.
Not carried over: tracked deletions, comments, footnotes and endnotes, headers and footers, text boxes (their text may appear in reading order but not their position), fonts and sizes, pictures in EMF/WMF format (listed as left out), `.doc` files (old format: the message asks you to save as `.docx`). **Not verified with a file made by Microsoft Word itself.**

## PDF

**Text only.** Carried over: the text in reading order; lines in a clearly larger font become headings; lines are joined into paragraphs. Not carried over: pictures, scanned pages (no text recognition), tables and columns as such, links, page layout. A PDF with a password or restrictions is refused with a message; a PDF whose fonts have no readable character map is refused with a message; a PDF with pages that have no text imports the others and lists the count. Shown by tests on hand-made PDFs; **not verified with PDFs from real programs** (fonts and encodings vary a lot, so expect some PDFs to come out wrong or empty).

## Safety and limits

- Everything is read on this computer; nothing is fetched from the network. Links on disk (symbolic links, junctions) are not followed. Hidden files are skipped.
- A document can only use pictures that are inside the chosen folder, the folder of the chosen file, or the ZIP file. Paths that climb out (`../`), absolute paths, drive letters and network paths are refused (tested).
- Links keep only http, https and mailto (`javascript:` and similar are dropped; tested).
- Notaya's own files (`.nota`, `.clf`) and the data folder are refused as a source (tested).
- Limits: 5,000 pages per import (more is refused with a message), 5,000 pictures (further pictures are left out and listed), 20 MB per Markdown/HTML/text file, 100 MB per Word/PDF file, 25 MB per picture, 1 GB per ZIP file, 2 GB of content in total, 1 GB of pictures, 20 MB per page, 15 minutes. ZIP files get zip-bomb and path checks; a ZIP inside a ZIP is opened one level (Notion's "Part" files), deeper ones are listed as not opened.
- Damaged or hostile files give an error or a plain page and never stop the program (tested with truncated and randomly damaged copies of each format). Not tested: very large real-world files and an import running while the vault locks itself.

## Not run

The real exe, the native file dialogs, Microsoft Word, a real Notion export, a real PDF from a program, importing on the owner's PC. The screens were checked in the mock UI only (light and dark).
