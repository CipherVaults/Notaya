# Whole-app test results: X-1 addition

<a id="native-w161-owned-markdown-20261122"></a>

## Native Windows embedded and nested database export, 2026-11-22

- **Passed for embedded data and scope (W161, Markdown).** Both saved files contained the owned inline table's exact header and row once between the surrounding paragraphs. The separate linked-view row remained. Only the whole-notebook file contained the neighboring section's page and body.
- **Omissions confirmed (W161, Markdown).** The full-page database created as a child of the anchor page was visible and populated in Notaya. Neither its title nor its marked row appeared in either file, with Include sub-pages on. The earlier standalone-only row and property also remained absent.
- **Unrun on this expanded fixture.** HTML, Word, PDF, ZIP options, database-only selection and exact release packages. Keep the earlier linked-view export results and standalone omission at their own scope.

[Exact source, native setup and saved-file checks](../execution/2026-11-15-safe-export-replacement.md#native-w161-owned-markdown-20261122).

<a id="native-w158-pdf-web-link-20261122"></a>

## Native Windows PDF external web link, 2026-11-22

- **Passed the previously unrun external HTTPS case (W158).** One page, one link, exact destination including query and fragment. The hit rectangle overlapped the displayed label and excluded adjacent plain text. Independent rendering and URL inspection passed for the saved PDF. The destination was never opened.
- **Scope.** This complements the earlier pages, text, bookmark and layout results. It does not extend the PDF performance, font repertoire, exact package or owner visual evidence. The test PDF, test vault and raw evidence stayed local.

[Exact build and independent file readback](../execution/2026-11-15-safe-export-replacement.md#native-w158-pdf-web-link-20261122).

<a id="native-w134-command-palette-20261122"></a>

## Native Windows command search and keyboard selection, 2026-11-22

- **Baseline regression reproduced (W134).** The exact backup query returned the matching note and omitted the command after results settled.
- **Candidate passed the matching scenario.** The exact command appeared before the matching note. One Enter opened one native Save dialog. Cancelling closed it with no new file in the picker directory.
- **Candidate passed the controls.** Normal note search and recent search worked. Matching notes came first for a partial query. A no-hit query stopped offering the previous result. Lock stayed unavailable with security off. Typed Export opened the current-page export dialog.
- **Not run in this batch.** Password-enabled lock, forced search failure, exact release packages and the full whole-app register. Both app processes exited 0 and the two closed disposable databases passed SQLite quick-check.

[Exact source, executable and native procedure](../execution/2026-10-23-restore-from-backup.md#native-w134-command-palette-20261122).

<a id="native-w186-manual-backup-20261122"></a>

## Native Windows manual backup and restore, 2026-11-22

- **Passed for backup creation and restore (W186).** Settings and the command palette each saved a native backup. Both files and the closed restored vault matched byte for byte. Full SQLite schema and sealed rows matched the closed source, including the attachment, history, activity and journal anchors. The note and picture displayed after restore and Version history reopened.
- **Passed for cancellation and live-file refusal.** The cancelled Save created nothing. Notaya refused its own live destination after Windows accepted the overwrite prompt. No temporary sibling remained. The live file was unavailable to hash while open. Closed readback confirmed all saved rows. Explorer opened the exact disposable app data directory.
- **Separate palette finding (W134).** Entering the complete displayed backup action label returned “No matches.” The action worked after clearing the query. Released v1.5.1 source also shows actions only for an empty query. This check used the visible action and recorded the search issue for a v1.6 fix.
- **Separate coverage.** The populated fixture had security off. Its body marker was absent from raw file bytes, and the backup opened without a password. Protected backup scenarios have separate evidence. Exact release packages remain unrun.

[Exact native routes, snapshot identity and readback](../execution/2026-10-23-restore-from-backup.md#native-w186-manual-backup-20261122).

<a id="native-w164-link-roundtrip-20261122"></a>

## Native Markdown ZIP internal-link round trip, 2026-11-22

- **Passed for original native navigation (W164).** Opened both page-link rows in the source. Created an inline link to a specific child heading in Notaya. Clicking it scrolled to the off-screen heading.
- **Passed for export and fresh import.** Exactly two scoped pages plus the layout marker. Both included links used the same encoded relative path without a source address. The excluded page was absent and its arrow and label were plain text. After native import into an empty vault, both links opened the new child under its parent. The excluded label did not navigate. Read-only record IDs were disjoint between the two closed vaults.
- **Documented conversion.** ZIP export retains the page destination and drops the heading fragment. The Markdown kept the heading link's label. A lone link paragraph came back as a page-link row displaying the target page title.
- **Separate coverage.** Other formats and link layouts, any private import, and exact release packages were outside the run.

[Exact source, saved ZIP identity and evidence limits](../execution/2026-11-15-safe-export-replacement.md#native-w164-link-roundtrip-20261122).

<a id="native-w165-export-replacement-20261122"></a>

## Native Windows export validation and replacement, 2026-11-22

- **Passed for validation (W165).** The actual Windows picker submitted a harmless filename under the copied app's data folder. Notaya rejected it and no file was created. An empty section gave the no-pages error before native Save.
- **Passed for preservation on failure.** With an already exported HTML document held open, Windows accepted the overwrite confirmation and Notaya reported access denied. The old document matched its original hash. Each attempted overwrite created and removed a temporary sibling, and the output folder contained only the original file before release.
- **Passed for the permitted replacement.** After closing the test handle, the same Save replaced the previous file. Independent HTML parsing matched the exact new title and body. The underlying disposable vault passed a read-only integrity check and the app exited normally.
- **Separate coverage.** Other formats and injected failures belong to earlier scoped checks. Exact release packages were outside this run.

[Exact build, old and new output identities and limits](../execution/2026-11-15-safe-export-replacement.md#native-w165-export-replacement-20261122).

<a id="native-w154-export-selection-20261122"></a>

## Native Windows export selection and cancellation, 2026-11-22

- **Passed for scope and order (W154).** In a disposable vault with three notebooks, nine sections and ten pages, selected two pages, two sections, and two notebooks in Notaya and saved through the actual Windows dialog. Four parsed HTML outputs contained the exact expected headings and body markers in sidebar order, including a matched subpages-off/on comparison. Unselected controls were absent, the empty selection disabled Export, and the unencrypted-file warning was visible.
- **Passed for native cancellation.** The saved-note selection remained in the dialog, the preexisting sentinel kept its exact bytes, and no export artifact appeared in the output or vault data folder. Saving the retained selection then succeeded.
- **Harness issue.** The native app exited normally, with its own saved process receipt. The enclosing shell returned code 1 because it tried to save a second receipt at that same create-only path. One WebView window-class shutdown diagnostic was retained. No app rerun was needed.
- **Separate coverage.** Prior cases cover one selected page, one section and a whole notebook. This run added the multiple-selection and cancellation routes using HTML. Destination refusal and replacement (W165), all scope/format combinations and exact release packages were outside this run.

[Exact source and output checks](../execution/2026-11-15-safe-export-replacement.md#native-w154-export-selection-20261122).

<a id="native-w153-hub-word-pdf-20261122"></a>

## Notion-style hub to Word and PDF on Windows, 2026-11-22

- **Passed for the native synthetic route (W153).** Imported a parent and nested page and the two related databases. Added a linked view of one imported database to the parent, then used the native Save dialogs for Word and PDF with subpages selected.
- **Passed for file content.** The Word archive passed its CRC check. Its document XML and the PDF's logical text held the expected parent and child bodies, exact Arabic and Hebrew controls and both rows with all five table columns. Both PDF pages rendered and their content was visually inspected.
- **Environment notes.** Poppler reported two display-font warnings. The tested lines and table remained visible. An initial readback assumed the app folder had just one SQLite file and stopped on the backup copies. The direct read-only check then verified the actual closed primary vault. The app itself was run once.
- **Not run.** Word visual rendering, the required person opening both files, real Notion account exports, other database scopes and exact release packages. This leaves W153 open for the person check. Standalone database inclusion (W161) remains separate.

[Exact source, file identities and evidence limits](../execution/2026-11-15-safe-export-replacement.md#native-w153-hub-word-pdf-20261122).

<a id="native-w224-windows-keyboard-20261122"></a>

## Native Windows keyboard and WebView verification, 2026-11-22

- **Passed in the native agent-driven check (W224).** The same four keys produced English, Arabic and English after two existing Windows layout switches. After two Shift+Enter breaks, the lines aligned left, right and left. The page and alignment reopened after restart. The saved HTML contained the exact Unicode lines and two hard breaks in one paragraph.
- **Passed for observed WebView behavior.** Ctrl+F found both English lines. Right-click on the title displayed native editing commands and right-click outside the editor suppressed that browser menu. F5 and Ctrl+R preserved the live selection and the text. The text menu also included Import passwords. That item was only observed.
- **Still open.** Required person-run confirmation, other layouts, and exact frozen-candidate checks.

[Exact source, executable and evidence limits](../execution/2026-10-28-windows-installer.md#native-w224-windows-keyboard-20261122).

<a id="native-w161-container-export-20261122"></a>

## Native Windows section and notebook export, 2026-11-22

- **Passed for linked view and section boundary (W161).** Native Save produced section and notebook files in Markdown, HTML, Word and PDF plus both Markdown ZIPs. The four intended linked-view columns and the one filtered row appeared in every output. Only notebook exports contained the neighboring section's page and body.
- **Failed for standalone database inclusion (W161).** The standalone Projects database was under the exported section and had an extra row and Budget property. All ten exported outputs omitted both. The shared selector in released v1.5.1 already selected pages only. This remains an inherited limitation and unresolved scope decision, separate from the approved inline export fix (WP-07).
- **Separate coverage.** Selection with databases only, other ZIP document types, an owned inline database rather than the tested linked view, and the final release candidate were outside this run.

[Exact executable, file evidence and findings](../execution/2026-11-15-safe-export-replacement.md#native-w161-container-export-20261122).

<a id="windows-portable-import-w217-w218-20261122"></a>

## Windows installed paths and portable-note import, 2026-11-22

- **Installed and portable paths passed (W217).** The established runner passed 11 assertions across new installed notes, installed restart and portable startup. It verified first-start options, exact data roots, reopening, and refusal of the installed-only import command in portable mode. Filesystem and cleanup checks passed; the local test listener closed.
- **Native first-start portable import passed in the tested scope (W218).** Cancelling and choosing an empty folder left no vault or staging file. Choosing the valid disposable source copied the notes and exactly one backup into isolated LocalAppData. The page and its saved body reopened after restart. The source files remained byte-identical, the destination matched, and the vaults passed SQLite integrity checks.
- **Core import passed.** 14 tests included existing vault and backup no-overwrite, open-source refusal, portable/data-folder selection, protected-vault preservation and backup retention. The direct already-open command in an installed window was not run. Setup execution and exact candidate remain separate.

[Source and executable bindings](../execution/2026-10-28-windows-installer.md#windows-portable-import-w217-w218-20261122).

<a id="windows-packages-rehearsal-20261122"></a>

## Windows installer and portable development rehearsal, 2026-11-22

- **Portable first start and relocation passed (W001, portable portion of W217).** Native Windows opened the exact extracted app without a password. A newly saved notebook, page and body reopened from a second path. The original was unchanged. Both vaults passed a read-only SQLite integrity check. The app created its vault beside the executable, with both home overrides absent.
- **Setup build and payload inspection passed.** A real unsigned NSIS setup was generated. Its top-level app and companion extracted independently, the companion matched the input, and the only app delta was the three-byte Tauri NSIS marker. The setup and ZIP member inventories contained no vault or profile.
- **Not run on these artifacts.** Setup execution, per-user install, upgrade and uninstall (W220), installed-mode imports (W218), separate sign-in (W222), and live bridge pairing. Final v1.6 packages and acceptance remain separate.

[Exact identities and verification limits](../execution/2026-10-23-restore-from-backup.md#windows-packages-rehearsal-20261122).

<a id="windows-bridge-startup-20261122"></a>

## Native Windows startup and bridge preflight, 2026-11-22

- **Startup failure and writable control (W225), development artifact.** A test file occupied the data-directory path. The native dialog reported the exact path and Windows error 183 with recovery guidance. Dismissing it gave exit 1, left the file byte-identical and created no vault. With a fresh writable directory, the same executable opened Home without a password. The fresh vault passed a read-only SQLite integrity check after normal process exit.
- **Bridge development executable passed a bounded Windows smoke.** The bridge ran help, replied to a benign JSON-RPC initialization and exited at end of input, and refused an unrecognized option with exit 1. No live companion pipe was opened.
- **Still open.** The WebView construction-failure variant, native live pairing and approvals, and exact-package reruns. No other Windows or release gate changed state.

[Exact inputs and evidence limits](../execution/2026-11-08-v1-6-build-start.md#windows-bridge-startup-20261122).

<a id="g3-w198-native-readback-20261122"></a>

## Native Windows read-only vault compatibility — 2026-11-22

- **Scoped G3/W198 passed.** On the released v1.5.1 executable, the separate copy showed the full ordinary page body, the stored short sticky in its board preview, and the table row with Number value `1729`. Navigation and viewing only.
- **Old visit kept both files byte-identical.** The untouched original and the visited copy both matched the closed v1.6 fixture hash. The fixture contained the history, activity and journal-anchor tables.
- **Current reopen passed.** v1.6 showed the same page, sticky and number. Closed integrity checks passed. Only the reserved settings record changed among ordinary records during the later v1.6 navigation, with corresponding activity and anchor updates. No stored table IDs disappeared and all existing history rows matched.
- **Separate scope.** Security-on compatibility, edits in the old app, first-unlock safety copy (A44), large or new database constructs, and the exact Windows candidate.

[Build bindings and preserved evidence](../execution/2026-11-09-sf3-tamper-and-purge.md#g3-w198-native-readback-20261122).

<a id="restore-w194-native-settings-20261122"></a>

## Native Windows Settings restore — 2026-11-22

- **Wrong password and same-file retry passed (W194).** Windows refused the wrong password without creating a staging or undo file. The correct password restored the protected note without reselecting the file. Its saved body and picture were present. The displaced open vault matched its pre-test bytes.
- **Damaged attachment and downgrade consent passed (W194).** A structurally valid SQLite input with damaged attachment data was refused. Selecting a valid security-off backup against the protected vault showed the warning and disabled the action until checked. The unchecked attempt left the undo inventory and staging unchanged. Explicit consent installed the selected backup byte for byte.
- **Persistence and undo passed on this route.** After restart the open backup required no password and its note body reopened. The protected undo kept all stored rows from the verified native pre-downgrade backup. The two SQLite files had different physical bytes. All tested sources kept their original hashes and all four closed files passed SQLite quick-check.
- **Separate scope.** Interaction starting at the protected lock screen, password entry after a protected restart, keyfiles, interrupted-rename fault tests (W195), and exact packages are outside this run.

[Exact evidence and window-capture limits](../execution/2026-10-23-restore-from-backup.md#restore-w194-native-settings-20261122).

<a id="restore-w194-core-20261122"></a>

## Backup restore validation and Windows packaging preflight — 2026-11-22

- **Core passed (W194, five cases).** Correct and wrong credentials, damaged-file refusal, security-off restore, and recognition of the staged protection mode passed. The protected undo file matched its original bytes and reopened with the old password in the core test.
- **Command guard passed (W194, test-only check).** Missing or false downgrade consent refused the backup without replacing the protected target. A test backup with damaged attachment data was also refused. With explicit consent and a valid backup, the core installed the selected bytes and retained the protected undo.
- **Native restore open.** The Windows app started at the expected lock screen. Desktop input reached a different window. Zero native restore assertions ran. The owned locked test process exited and the original vault bytes matched.
- **Inert packaging preflight passed.** The locked and offline invocation arguments, installer bridge placement, portable members and guide, recorded bridge hash, and exclusion of vault files passed with fixture executables. Exact installer, portable and clean-user gates remain open.

[Build and evidence details](../execution/2026-10-23-restore-from-backup.md#restore-w194-core-20261122).

<a id="native-omissions-20261122"></a>

## Native Windows ordinary omission reporting — 2026-11-22

- **Baseline failed (W151).** The native Files picker imported two pages in one section. The report listed the unsupported audio and omitted the missing linked page. The new Rust control failed with no missing-link entry.
- **Final-tree tests passed.** Seven exact selected importer and round-trip tests passed. An independent review verified the nested path and fragment correction. One build preflight was blocked by a transient Git index lock before Cargo started. The bounded retry and final build finished.
- **Native Windows case passed.** The reviewed app gave exactly two omission entries, including the missing linked page and its source. The original ZIP hash matched. After restart both pages and surrounding text were present. Clicking the valid link opened the saved child. The missing link label was plain text. The closed vault had five records, zero attachments, and returned `ok` from SQLite quick-check.
- **Integrated.** [PR #140](https://github.com/CipherVaults/Notaya/pull/140) and independent main readback matched the tested tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`.
- **Outside this run.** Dedicated Notion page-link figures, targets present in the source but later dropped, other unsupported embed formats, native Linux and exact Windows packages.

[Exact sources, binary and failure evidence](../execution/2026-11-15-safe-export-replacement.md#native-omissions-20261122).

<a id="native-volume-main-20261122"></a>

## Native Windows larger ordinary PDF and ZIP — 2026-11-22

- **240-page text PDF passed on the tested tree.** It imported as one note. After restart, independent HTML inspection verified 240 headers and 7,200 numbered lines in exact order, once each.
- **1,000-note ZIP passed on the tested tree.** It imported as one notebook with one section. After restart and whole-notebook Markdown ZIP export, independent inspection verified the archive CRC, all 1,000 titles and unique note markers, and all 20,000 numbered sentences in order within their own notes. The earlier endpoint sampling remains valid for its own run.
- **Persistence checks passed.** Both original input hashes stayed fixed and both closed disposable vaults passed SQLite quick-check. Missing-content controls were rejected by the same independent output checks.
- **Measured once per input.** Confirmed complete on-screen after about 0.63 s for PDF and 0.68 s for ZIP. During each observation there were two complete simultaneous app/WebView samples. Highest sampled working set was 499.0/507.3 MiB and private bytes 315.6/437.6 MiB, respectively.
- **Separate evidence remains open.** Maximum file sizes, hostile-source tests (W150), a repeated performance comparison, the earlier parser memory finding (DD1065), the 5,000-row redo and release packages.

[Exact build, procedure and bounds](../execution/2026-11-15-safe-export-replacement.md#native-volume-main-20261122).

<a id="native-pdf-logical-merged-20261122"></a>

## Native right-to-left PDF reimport — 2026-11-22

- **Baseline failed (W163).** The real exporter-to-importer regression on `bd5991d` reproduced reversed Arabic and Hebrew strings. The earlier native failure remains recorded.
- **Exact-source tests passed.** `316008dd74cf65b7a891a51a3c288fbe0a7cdc44` passed all six selected tests. Controls covered unmarked glyph decoding, mixed-direction punctuation, a nested span, normal glyph fallback for an unclosed span, a marked Form followed by a separate column, and ordinary refusals.
- **Native Windows passed.** The pinned synthetic PDF imported into a disposable vault and reopened. An independent check matched all 14 expected HTML text items, including exact Arabic and Hebrew characters, English controls and numeric table values. The PDF hash remained unchanged.
- **Integrated.** Independent main recovery matched the tested tree `581274b0715d074778b3251509ee05aa559c9d07`.
- **Not run here.** Larger PDF resource limits, visual layout reproduction, Linux, exact installer and portable packages, or the 5,000-row database redo.

[Execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-pdf-logical-merged-20261122).

<a id="native-word-boundary-merged-20261122"></a>

## Word lists beyond 10,000 items — 2026-11-22

- **Baseline failed (W141).** The added regression on `eeafe701` reported 10,000 direct bullet entries of 10,001. Earlier saved Windows outputs omitted the same entry.
- **Exact-source tests passed.** `5da53a398525c2e6acfcf5ff229ea400770e4906` passed 24 selected document importer and renderer tests, including long direct and style-based bullets and numbering, and the existing oversized-page error with early list-render termination.
- **Native Windows passed.** The two disposable long bullet lists imported separately, reopened, and exported as HTML. Independent verification matched all entries in order and the tail outside the list. The same check rejected both old incomplete outputs.
- **Integrated.** The independently recovered main tree `5bae51cb14ca49cba8b40a0c22825c3d24405067` matched the tested tree.
- **Not run here.** Linux native, exact Windows packages, private vaults, right-to-left PDF reimport (W163), and the 5,000-row database redo (Q1).

[Execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-word-boundary-merged-20261122).

<a id="native-backup-refusal-20261122"></a>

## Native backup cancellation and refusal — 2026-11-22 08:19 Cairo / 06:19 UTC

- **Passed.** Canceling the native file picker returned to Restore with replacement disabled.
- **Passed.** Selecting a nonempty ordinary text file as a disposable unreadable backup enabled the attempt and produced a damage message. Restore remained open. The current vault and both source files kept their bytes. No undo or staged restore file was left. Restart reopened the two pre-existing notes, including the verified Word body and table.
- **Not run.** Password, keyfile, interruption, first-unlock copy and exact-package checks.

[Exact evidence](../execution/2026-10-23-restore-from-backup.md#native-backup-refusal-20261122).

<a id="native-backup-picker-20261122"></a>

## Native file-picker backup restore — 2026-11-22 08:13 Cairo / 06:13 UTC

| Check | Outcome |
|---|---|
| Settings → Restore from a backup → Choose another backup file (A18) | **Passed with security off.** The native picker selected an owned backup outside the app data folder. The replacement opened immediately and showed the same four notes after restart. An opened note showed the body and its expected blue picture. |
| Previous vault retained | **Passed.** Exactly one new undo file matched the displaced preimage byte for byte. An independent native app home opened a copy with the original two notes and verified the original title, table and picture. |
| Input and staged files | **Passed.** Selected backup, external source snapshots and retained undo stayed byte-identical. Immediately after restore the active file matched the selected backup. After opening a note, only the app settings record differed in the compared records. Note, notebook, section, header and attachment rows matched. The staging file was gone. |
| Limits | This check did not exercise a password or keyfile, the failed-file or cancellation path, power loss, first-unlock safety copy (A44), or the installer. |

[Exact evidence](../execution/2026-10-23-restore-from-backup.md#native-backup-picker-20261122).

<a id="native-zip-collisions-20261122"></a>

## Separate-file ZIP collisions and punctuation — 2026-11-22 08:05 Cairo / 06:05 UTC

| Check | Outcome |
|---|---|
| W159 native HTML, Markdown, Word and PDF ZIP output | **Passed on the four-page notebook.** The actual archives had the expected distinct safe filenames, one layout marker, four documents, correct original titles and one unique body per document. No member was overwritten. |
| Pictures | **Passed for the two source PNGs.** Markdown links resolved to the correct sidecar bytes. HTML and DOCX held the original bytes. The independent PDF parser and render found the expected different swatches. |
| HTML and Markdown ZIP reimport | **Passed for title, content and notebook structure.** After restarting, both notebooks had one section and four distinct notes. The closed synthetic vault had four attachments. The sidebar followed the documented name sort. |
| Unrun or separately qualified | ZIP destination replacement, large or deep ZIPs, Word/PDF ZIP reimport, linked-page resolution, and exact package behavior were outside this check. The known PDF text reimport failure remains open. |

[Exact artifact and test evidence](../execution/2026-11-15-safe-export-replacement.md#native-zip-collisions-20261122).

<a id="native-import-titles-merged-20261122"></a>

## Word and Markdown title fixes integrated — 2026-11-22 07:54 Cairo / 05:54 UTC

| Check | Outcome |
|---|---|
| Exact title candidate | **Passed.** 16 Markdown parser tests, 12 Word importer tests and two focused round trips on `c72e51a79ecb696f105f88b9696650713a86fd95`. |
| Saved-page native check (W141 and W162) | **Passed for both title fixtures.** The exact Windows development build imported both files through Windows Files. After restarting, the app showed both correct titles, body text, tables and pictures. Original fixture hashes matched. |
| Integration | **Merged.** [PR #137](https://github.com/CipherVaults/Notaya/pull/137). Independent recovery from main found identical bytes to the tested tree. Hosted checks were absent. |
| Limits | The Word long-list failure, Word inline code, PDF text reimport, vault backup design and package checks remain separate. |

[Exact build and merge evidence](../execution/2026-11-15-safe-export-replacement.md#native-import-titles-merged-20261122).

<a id="native-vault-compatibility-20261122"></a>

## Vault compatibility and Word list limits — 2026-11-22 07:36 Cairo / 05:36 UTC

| Check | Result |
|---|---|
| Released app opens a 1.6 vault copy (G3) | **Passed on a disposable security-off fixture.** The exact published v1.5.1 app showed the saved title, body, two lists, a table and a picture. The newer vault and its copy had identical bytes before and after the visit. |
| One-time pre-upgrade safety copy and notice (A44) | **Failed on the development build.** On first automatic unlock it changed the disposable older vault's schema. The ordinary backup after close already held the new tables. The expected notice and a pre-upgrade copy were absent. A repeat unlock and password-protected fixture were not run. |
| Word list defined by paragraph style (W141) | **Failed at 10,001 items on the development build.** The imported note was saved and reopened. Its HTML output retained the first 10,000 bullets and the plain tail, with item 10,001 missing. The earlier short list result remains valid. |
| Word directly numbered list (W141) | **Failed at 10,001 items in released v1.5.1.** The exact published exe reported success. The reopened note's HTML contained the first 10,000 bullets and the plain tail. Item 10,001 was missing. |
| Scope and limits | The development executable was an unbundled Windows x64 release-profile build. Neither result qualified a v1.6 package, a password-protected vault, or the 5,000-row redo. |

[Vault evidence](../execution/2026-11-09-sf3-tamper-and-purge.md#native-vault-compatibility-20261122) · [Word evidence](../execution/2026-11-15-safe-export-replacement.md#native-word-list-boundary-20261122).

<a id="markdown-title-fix-20261122"></a>

## Native Markdown title fix — 2026-11-22 03:01 Cairo / 01:01 UTC

| Register case | Outcome |
|---|---|
| Picture-bearing Markdown round trip (W162) | **Passed on the Windows development candidate.** Imported the original file that failed on the baseline. After restart the editor showed the correct title, no literal title heading, the expected text and styling, two lists, the table and picture. |
| Regression control | **Expected failure preserved.** The strengthened pictured round-trip test ran once on the old reader, reported the title `page` and exited 101. With the fix, 16 parser tests and both targeted round-trip tests passed. |
| Input and build identity | Both source files retained their original hashes. The release-profile Windows x64 executable was built from the exact candidate. [Trace and limits](../execution/2026-11-15-safe-export-replacement.md#markdown-title-fix-20261122). |
| Separate qualification | Word's inline-code loss remains open. This check did not rerun PDF import, test an installer or portable ZIP, qualify large resources, or clear database redo. |

<a id="windows-document-roundtrips-20261122"></a>

## Native document round trips — 2026-11-22 02:35 Cairo / 00:35 UTC

| Register case | Outcome |
|---|---|
| W162 Markdown with picture | **Failed title preservation.** The exported heading was correct in the file. After reopening the imported note, the filename was the title and the heading appeared as literal hash-prefixed text. Body text, lists, table and picture survived. |
| W162 HTML | **Passed for the fixture.** Reopened title, multilingual text, bold, italic, code mark, both lists, table and picture matched. |
| W162 DOCX | **Partial pass.** Reopened title, text, bold, italic, lists, table and picture matched. The inline code token survived as plain text. The output had a code-styled Word run, which the importer did not turn back into a code mark. |
| Direct file and source checks | **Passed for the named assertions.** The exports kept the expected structure and original PNG bytes in a Markdown sidecar or embedded in HTML and Word. The writer placed the body heading one level deeper under the synthetic title. Two exact non-PDF Rust round-trip tests passed. |
| Limits | This fixture did not exercise PDF, exact package behavior, file replacement recovery, custom list starts, or broad document layout. The Markdown failure remains open. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-document-roundtrips-20261122).

<a id="windows-inline-export-20261122"></a>

## Native inline database export — 2026-11-22 02:24 Cairo / 00:24 UTC

| Register case | Outcome |
|---|---|
| W160 Markdown, HTML, DOCX, PDF | **Passed for the linked-view fixture.** All four actual files contain the exact four visible columns, Alpha's status, 60% and both task labels. Beta and Budget are absent. The table sits between the opening and closing paragraphs. |
| Output inspection | **Passed.** Markdown has the database reference. HTML and DOCX have one matching table. The DOCX archive passed CRC verification. An independent PDF extractor and the rendered page matched the table and surrounding text. |
| Focused source checks | **Passed.** Four writer tests and the saved-view filter, sort and hidden-column test ran by exact name. Native sorting is untested in the one-row view. |
| Limits | Synthetic local data only. The fixture does not qualify large tables, RTL output across formats, export replacement recovery, database redo, PDF reimport or final packages. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-inline-export-20261122).

<a id="windows-notion-paired-import-20261122"></a>

## Native paired Notion export — 2026-11-22 02:13 Cairo / 00:13 UTC

| Register case | Outcome |
|---|---|
| W145 canonical pages and navigation | **Passed for the fixture.** One notebook contained two ordinary pages. Handbook used its HTML body and opened Details through the imported link after restart. The breadcrumb and sidebar showed Details under Handbook. The accented and Arabic text persisted. |
| W146 canonical database and rows | **Passed for the fixture.** Projects and Tasks had two rows each. The full CSV supplied the rows, statuses, currency, percentages and estimates. No filtered sentinel or loose row duplicate appeared. Alpha used its highlighted HTML body, Beta kept its HTML-only body, and both task bodies persisted. Native relation navigation opened both tasks and returned from First task to Alpha. |
| Focused source checks | **Passed.** Three exact Rust tests covered selection of the full CSV, typed values and reciprocal relations, and pairing Markdown, CSV and HTML. |
| Limits | Backup restore, broader export shapes, private data parity, the 5,000-row redo and final packages were outside this fixture. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-notion-paired-import-20261122).

<a id="windows-import-cancel-20261122"></a>

## Native cancellation after conversion — 2026-11-22 02:03 Cairo / 00:03 UTC

| Register case | Outcome |
|---|---|
| W149 actual Cancel import | **Passed for the named fixture.** A picture was already staged. The action removed staged attachments and left vault records unchanged at the cancellation boundary. After reopening, exactly the original notebook, section and page remained, with matching title and body. The earlier picker leg is separate. |
| W149 source controls | **Passed.** Both existing exact Rust tests ran, one for cancel/rollback and one for command registration and delivery. |
| Supplemental checksum | **Failed.** The aggregate checksum of encrypted record rows changed after the restarted editor was opened. The row and attachment counts were unchanged. The cause was not established. |
| Other qualification | Cancellation of other formats and lock-driven cancellation were not run here. Larger import, database redo, released PDF repair and exact package gates remain separate. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-import-cancel-20261122).

<a id="windows-word-styled-lists-20261122"></a>

## Native Word styled-list qualification — 2026-11-22 01:40 Cairo / 2026-11-21 23:40 UTC

| Register case | Verified scope |
|---|---|
| W141 Word import, lists defined by styles | **Passed on the candidate fixture.** Baseline flattened both lists and kept the text. The exact candidate retained two bullets, two decimal items and two plain paragraphs after reopening. Independent HTML readback confirmed the list kinds and exact text. |
| W141 source regression | **Passed.** All 13 DOCX converter tests. The new test covers inherited nesting, direct numbering and cancellation, headings, titles and plain controls. Malformed/encrypted Word variants were not run on native Windows in this slice. |
| Existing gates | PDF text repair held. W149 conversion cancellation remains open. The 5,000-row redo gate and package checks remain open. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#word-styled-lists-20261122).

<a id="release-pdf-and-folder-20261122"></a>

## Released PDF applicability and native folder qualification — 2026-11-22 01:15 Cairo / 2026-11-21 23:15 UTC

| Check | Verified outcome |
|---|---|
| W163 PDF text, published Windows v1.5.1 portable | **Failed.** The same synthetic development-exported PDF imported with both tested Arabic and Hebrew words reversed in the saved note. Confirmed after native reopen and actual HTML readback. English and table value controls matched. The input kept its exact hash. |
| W143 native Folder picker | **Passed for this fixture.** One separate notebook, one section, parent and child bodies correct, and the existing note unchanged after reopening. The mixed Files result is recorded separately. |
| Scope limits | Published-version export, hostile inputs, full package checks and the 5,000-row redo were outside this batch. The PDF repair line awaits Ahmed's release decision. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#released-pdf-folder-20261122).

<a id="windows-document-import-20261122"></a>

## Windows document import qualification — 2026-11-22 01:03 Cairo / 2026-11-21 23:03 UTC

| Register case | Outcome for this batch |
|---|---|
| W139 UTF-8 plain text | **Passed.** Exact mixed scripts, logical text and literal markup, independently confirmed two paragraphs and two hard breaks. Saved text matched after reopen. |
| W140 ordinary HTML import | **Partial pass.** Heading, emphasis, table and color observed. Other feature and safety variants not run. |
| W141 Word import | **Title defect fixed and verified.** Original native failure and red regression preserved. All 12 Word importer tests passed on `fb1256a`. Native same-file import and reopen passed title and body/table/picture. **Separate observed gap:** style-defined bullet flattened. Unsupported variants not run. |
| W143 mixed import | **Files leg passed.** One notebook, one section, three selected formats, three pages and one picture. Folder and prior-note preservation not run. |
| W149 cancellation | **Native picker leg passed.** Cancel and close returned to empty Home. Conversion cancellation not run. |
| W163 PDF round trip | **Failed text fidelity.** English controls and numbers matched, but both Arabic and Hebrew words had reversed character order after reopening. Actual released v1.5.1 executable applicability not checked. |
| W150 hostile input and Q1 database redo | **Held outside this batch.** Daybreak pending, stopped operation untouched, both existing redo failures preserved. |
| Updated build and package | **Windows development build passed.** Exact installer/portable qualification and release acceptance not run. |

[Exact execution evidence](../execution/2026-11-15-safe-export-replacement.md#native-doc-import-20261122).

<a id="windows-native-pdf-zip-20261122"></a>

## Windows baseline qualification — 2026-11-22 00:31 Cairo / 2026-11-21 22:31 UTC

See the [native PDF/ZIP evidence and exact executable](../execution/2026-11-15-safe-export-replacement.md#native-windows-pdf-zip-20261122). These are scoped Windows results at `a806ae3`. They remain separate from the historical startup evidence below and from exact-package qualification.

| Register case | Outcome for this batch |
|---|---|
| W142 PDF import | **Passed.** Ordered text reopened correctly and the scan-only file was listed as unsupported. |
| W144 benign nested ZIP import | **Passed for this fixture.** Two nested pages, picture and child navigation persisted. Hostile path cases remain W150. |
| W151 import reporting/source files | **Partial pass.** Unsupported scan and five unchanged hashes. Broken links and unsupported embeds not run. |
| W154 native export/save | **Partial pass.** One selected page with subpages through the native PDF and ZIP Save dialogs. Other scopes and cancellation not run. |
| W158 PDF export | **Partial pass.** Pages, text, bookmarks and real rendering verified on the exported PDF. External links and broader typography not run. |
| W159 separate-file ZIP export | **Partial pass.** Markdown, nesting, marker, assets and archive integrity. Other formats, collisions and punctuation not run. |
| Larger benign PDF/ZIP samples | **Passed within scope.** All 24 PDF markers in order. 1,000 ZIP notes reported and the first and last bodies sampled, including the last after reopen. Does not qualify maximum size, memory or all 1,000 bodies. |
| Executed product failures | **None observed.** The execution record retains the setup failures. |
| W167 owner visual review | **Skipped.** Independent PDF rendering is recorded separately. |
| W145/W146 Notion data, W149 cancel, W160 multi-format inline database export, W162 all-format document round trips, W163 PDF round trip | **Not run.** |
| W150 hostile source/bounds | **Blocked for this batch.** Daybreak pending. No stopped operation retried. |
| Q1 5,000-row redo | **Open from two failed runs.** Both receipts retained. Not run here. |


Who and where: Ahmed Emara (AE0101), Codex lane X1 on Gaming-PC (Windows sandbox). **Historical run-2 provenance:** FIX RUN 2 baseline `9b64db2f85388c5d800c25926772fdc15a2c539b`; external heartbeat checkpoint `ba047062f4bd5162bf4e6f897b4228051aec8dda` contains that run's tested code, with its results addendum then uncommitted. Original baseline: `e4df9185a99fabf6002065c84c921cd8f8278e31`. Produced and locally checked, not integrated or accepted. Claude owns integration, exact candidate commit/exe hash and real-PC observations. The [FIX RUN 2](#fix-run-2-corrected-analysis-and-hand-back) and run-1 analysis/checks below are historical, superseded where contradicted by run 3.

**Current addendum: [FIX RUN 3](#fix-run-3-readiness-failure-regressions).** The run-2 analysis below remains historical evidence; its status/listener success-only gate is superseded by the bounded failure path described here. Original reporter: **Mohamed Sayed (AbyssWatcher2)**, via Ahmed. Run 2 and run 3 are both **unproven** as fixes for his white-window report.

## Defect register

| Defect / check | Reporter | Report and scope | Status | Candidate and evidence needed |
|---|---|---|---|---|
| X-1 / WIN-X1, KIND B | **Mohamed Sayed (AbyssWatcher2)**, via Ahmed, 2026-11-09 07:38:18 UTC | v1.5.1 on Mohamed's PC: blank white window at sign-in with Start with Windows on | **Run-3 readiness regressions locally verified: 23 tests passed. Real exe not run by Codex; original X-1 fix remains unproven.** Historical baseline reproduced / candidate 1 not fixed in Claude's reported held-frame run; the old readiness observer is invalid. Actual sign-in/pixel cause remains unconfirmed. | Claude runs baseline `5077183` against the exact run-3 candidate after heartbeat checks, using `tools/real-exe-tests/startup-minimized.md`. Historical evidence: baseline `5077183`, SHA-256 `5FA039FC…`; candidate 1 `9b64db2`, `8F38C904…`. Both: normal/minimized/5000 ms delayed-page DOM controls passed; held frames 8000 ms failed at 45 s, `readyCalls=0`, `readyCompleted=0`, `readyAtMs=null`, `hookInstalled=true`, `heldFrames=2`, hidden, `readyWithoutFrames=false`. Candidate-1 delay-page reload also counted zero readiness and was shown by restore. See corrected analysis below and rerun both with the new observer. |

## FIX RUN 3: readiness failure regressions

**Who, authority and scope:** Ahmed's side (`AE0101`), Codex lane X1 on `GAMING-PC`, Windows sandbox. Ahmed's current FIX RUN 3 request (message timestamp not exposed); system-clock checkpoint **2026-11-09 17:57:00 UTC**. Baseline/head `b8a2883d9ac7372815b7471bd86fc9b1e9beaa56`, branch `AE0101/v1-6-x1`; local `origin/main` `066b23cab8ba9e6287cf2699810564fa0f133c3c` read without fetching, including the 13:45 UTC brief update. Result: **produced and locally verified for the named checks**, uncommitted, not integrated or accepted. Stop at hand-back; Claude commits, owns the records/ledger and runs the held-frame comparison after the heartbeat checks. Runtime model/effort, token totals and quota readings are not exposed to this tool session; no values invented.

**Change and reason:** run 2 required a non-null status and both lock listeners before showing the initially hidden window. The production initialization watcher now retries a rejected status request once and permits readiness on the second rejection, on either listener rejection, or after **5000 ms total** across both attempts and listener registration. The successful path still follows the committed status screen and registered listeners through the existing cancellable microtask, without waiting for a frame or timer. The App ref sends `window_ready` once, including after late status/listener completion. Cleanup clears the deadline, cancels the readiness microtask, suppresses late watcher state changes and removes successful/late listeners; rejected lock and notice registrations have handled cleanup paths. A failure fallback permits the existing rendered surface; it does not invent status, unlock a vault, repair missing listeners, or add a new error screen. Rust still owns visibility, including the minimized/tray guard.

### Regression evidence (plain JS, production hooks and watcher; simulated IPC/time)

All names below are in `tools/real-exe-tests/startup-ready.test.mjs`; each passed:

| Test name | Case and observed result |
|---|---|
| `status rejects: one retry also rejects, then window_ready is sent once` | Two attempts rejected; ready once despite a still-pending listener; late listener cannot resend. |
| `status rejects once then succeeds: readiness follows the recovered status without waiting for the bound` | Two attempts, real status available, listener gate satisfied, no failure fallback, ready once. |
| `first lock listener rejects: window_ready is sent once even while status is pending` | `vault-locked` rejects; ready once without waiting for status; remaining listeners removed on cleanup. |
| `second lock listener rejects: window_ready is sent once and cleanup handles the rejected registration` | `vault-locking` rejects; ready once; rejected registration does not produce an unhandled cleanup rejection. |
| `both lock listeners reject: window_ready is sent once with no unhandled registration or cleanup rejection` | Both reject; ready once; notice listener still removed. |
| `status never resolves within the bound: window_ready is sent once at 5000 ms and late status cannot resend` | No readiness at 4999 ms, one call at 5000 ms; late success cannot resend. |

Four more added tests passed: `pending lock registration is also bounded and is removed when it completes after cleanup`; `initialization cleanup cancels the bound and ignores late failures and listener completion`; `late rejection after timeout is handled without a retry or duplicate readiness`; `notice-listener rejection does not disrupt readiness or produce an unhandled cleanup rejection`. The original 13 scheduler, actual-hook, backdrop, appearance and transport-probe tests also passed. An initial run of the new tests failed because the VM harness redeclared `runHook`; the harness was corrected before the passing final run. That failure was in the test harness, not a native-app observation.

### Final-source checks

These checks apply to **the uncommitted run-3 source on `b8a2883`**, not to `b8a2883` alone or any later commit:

| Command | Exact result |
|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Exit **0**, no output/errors. |
| `npm run build -- --configLoader native` | Exit **0**; Vite **8.3.5**, **4414 modules transformed**, built in **10.68 s**. Existing `INEFFECTIVE_DYNAMIC_IMPORT` warning for `DatabaseView.tsx` being imported both statically and dynamically. |
| `node --test --test-isolation=none tools/real-exe-tests/startup-ready.test.mjs tools/real-exe-tests/startup-probe.test.mjs` | Exit **0**; **23 passed, 0 failed, 0 cancelled, 0 skipped, 0 todo**, **360.0987 ms**. |
| `git diff --check` | Exit **0**, no whitespace errors; LF-to-CRLF notices only. |
| `git diff --stat -- src-tauri` | Empty: Rust not touched. |
| `npm audit`, `cargo test --lib`, `cargo clippy`, `cargo audit` | **Not run**. No network, installs or Rust/dependency changes in this run. |
| Real Notaya.exe, browser, heartbeat candidate checks, held-frame baseline/candidate comparison, actual sign-in, Mohamed's PC, human pixels/tray and protected-vault lock/flush | **Not run by Codex**. No proof that run 2 or run 3 fixes the original X-1 report. |

Final tested file SHA-256 (documentation excluded from this fingerprint):

| File | SHA-256 |
|---|---|
| `src/App.tsx` | `4F8DBBF1881FE54E152AE41A84E2CC6DD7FD5F4CA52E051693B86443A0EB59F8` |
| `src/lib/startup.ts` | `8AFBA6B3FC20D23934685CF11F3731E39D4DF5B6FCA0096524E46166C1105F00` |
| `tools/real-exe-tests/startup-ready.test.mjs` | `2760E987CCBE785BAC9E48A622C7E0A2A05058699435F81763642C9EC0A366D2` |

### Security self-check (guide section 4; not independent)

- **Secrets/keys/crypto:** no new secret handling, persistence, random values or crypto; n/a to this diff. No vault, credential or private-note fixtures/logs.
- **Commands/IPC:** traced status/listener outcomes to the existing argument-free `window_ready`; no new command, payload, permission or confirmation route. Rust's tray visibility and lock enforcement are unchanged. A failed listener remains unavailable; this test proves readiness fallback, not real lock-event delivery.
- **Files/paths/parsing; AI pipe/bridge; storage/history/logs:** n/a; none changed. Test input is synthetic and in-memory; no vault is opened.
- **Frontend:** read the full changed App/startup/test files. No new HTML rendering, script evaluation in production, CSP change, network call, browser storage or sensitive diagnostic field. Existing lock/reload and flush handlers are unchanged. Timer callbacks are functions, once-only scheduling/cleanup and late rejection handling are covered. Native auto-lock/screenshot behavior was not run.
- **Dependencies/build:** no dependency or lockfile change; type check/build passed. Audits not run under the offline instruction. Browser-free VM execution exists only in the test harness.
- **Findings:** no additional real, reachable security issue found in this bounded self-check. Independent Codex/Claude security review remains to be done by a session that did not write this diff.

**Files changed:** `src/App.tsx`, `src/lib/startup.ts`, `tools/real-exe-tests/startup-ready.test.mjs`, and X-1 content in this results file and `2026-11-09-whole-app-test-plan.md`. **Outside write area, mechanical:** none. **ADDITION FOR AHMED:** visible changes: none expected; the authorized failure fallback restores readiness signaling only. No Git writes, network, installs, exe/browser launches, real-vault access or debug ports started by Codex. Mohamed Sayed remains credited as the reporter; Claude adds the commit credit.

## FIX RUN 2: corrected analysis and hand-back

**Authority/scope:** Ahmed's current FIX RUN 2 instruction (message timestamp not exposed); read-only orientation began on `9b64db2`, branch `AE0101/v1-6-x1`. System-clock checkpoint: 2026-11-09 12:49:44 UTC. Startup-only source, the startup tooling and these two plan/results records are the write area. Stop at checked local source and hand-back to Claude. Codex issued no Git write, fetch/PR/network, install, real exe or browser command. The external heartbeat independently made automatic checkpoint `ba04706` at 2026-11-09 12:56:44 UTC; the tested eleven-file source fingerprint below remained identical afterwards. Original reporter remains **Mohamed Sayed (AbyssWatcher2)**; Claude credits him in the eventual commit. Runtime report comes from Claude via Ahmed; raw report folders/timestamps were not supplied. Both full artifact hashes were independently read from the existing files in this run, without executing them.

### Corrected root cause and candidate analysis

**Proven cause of the zero-counter evidence: candidates 3 and 5, the harness hook.** Local pinned Tauri 2.12.5 `scripts/core.js` defines `__TAURI_INTERNALS__.invoke` using `Object.defineProperty(..., { value: ... })`, so it is neither writable nor configurable. The old injected function was evaluated as a non-strict script. Its assignment silently failed, then it set `hookInstalled=true` unconditionally. A browser-free VM reproduction using the actual old `frameProbe` and the same descriptor produced `nativeCalls=1`, `readyCalls=0`, `readyCompleted=0`, `hookInstalled=true`, `writable=false`. Thus the first fix was **not demonstrated to work**, but those runs also do **not prove its fallback failed to call Rust**. A hidden native window is expected under `--minimized`; two requested frames prove neither that App's effect ran nor that its timer was cancelled. Preserve the reported failures, correct their interpretation, and rerun with a valid observer.

| Candidate | Code evidence and corrected conclusion |
|---|---|
| 1: readiness effect gated by null status | The original chain is `main.tsx` imports/bootstrap → React App → passive startup effect → `applyAppearance(cached,true)` → `set_backdrop` IPC → `refreshStatus` → `vault_status` IPC → Zustand `set({status})` → committed screen → readiness effect. Appearance CSS can exist while status is null; **the actual UnlockScreen cannot**, as both App and UnlockScreen explicitly require status. A full screen in the reload's DOM rules out a persistently null status there. An IPC delay can still precede it; stage counters locate it. New status loading runs independently of native backdrop completion. |
| 2: cancellation/throttled timer, StrictMode, screen swaps | Candidate 1's timer was scheduled before its rAF request; cleanup cancels both. Dependency is the boolean `hasStatus`, so ordinary non-null status updates and lock/shell child swaps do not cancel it. StrictMode dev replay schedules again, and production does not run dev replay. No code evidence establishes cancellation in the PC run. The old observer cannot distinguish cancellation from successful completion. New layout-effect/microtask scheduling removes timer/paint waits, has cancellation plus a sent ref, and regression tests cover actual hook gating/cleanup/re-entry. |
| 3: captured invoke / Rust early return | Actual wrapper is `src/lib/ipc.ts`, not `api.ts`. `api.windowReady` calls imported Tauri `invoke("window_ready")`; local API 2.12.4 `core.js:340-342` looks up `window.__TAURI_INTERNALS__.invoke` on each call, so it does not capture an old internals function. Rust `commands/vault.rs:598` marks the label ready **before** the tray-hidden early return; the unit return resolves successfully. A valid probe counts this as completion even though the native window stays hidden. The read-only hook, not captured-reference semantics, explains the false zero counts. |
| 4: alternate ready/show path | Complete `rg` over `src/` and `src-tauri/src/` found main App, sticky and calculator readiness calls; only main App is changed. `shell::show_main` is called by tray, legacy handover, single instance and AI notice. It shows/focuses directly and does not call `window_ready`. Hence harness restore visibility does not prove readiness. `lib.rs` sets tray state after tray construction; `window_ready` retains the native guard. No additional `app_ready` route was found. |
| 5: wrong reload / completion / deadline | New-document script is acknowledged before `Page.reload`. Old polling did not bind observations to that document and delay-only runs could pass without seeing readiness. Corrected probe stamps a reload ID and requires main label plus matching ID, exactly one real transport call and success callback. It distinguishes rejection, absent callback hook and late completion; keeps the 45 s observation limit. Held-frame success must precede both actual release and 8000 ms; CDP can release only the test shim if its own page timer is throttled. |
| 6: earlier frame/visibility wait | No `fonts.ready`, decode promise, transition/animation-end await or rAF wait was found along bootstrap/appearance/status initialization. Font imports are CSS; motion/Radix can schedule animation frames but the status request does not await their animation. React's installed scheduler uses its host message loop, not rAF for this effect scheduling. **First explicit awaited paint gate in the original application startup chain was App's two-rAF readiness callback.** Native backdrop and vault-status IPC remain actual async stages; source alone cannot identify a PC-specific pending native call or compositor failure. No GPU/runtime/profile change is made. |

**Fix and people's experience:** readiness now follows the committed status screen and confirmed `vault-locked`/`vault-locking` listener registration through a cancellable microtask. It uses neither rAF nor a timeout. The App ref sends once across screen/status swaps; cleanup cancels pending work and StrictMode can reschedule. Status no longer waits for backdrop IPC. An appearance revision guard prevents a late cached-backdrop completion overwriting newer loaded settings/cache when those calls overlap. Rust retains all visibility decisions (`--minimized` hidden while the tray exists), tray/single-instance/settings/auto-lock behavior and existing animations. No Rust source, setting or visible feature changes. The real blank-pixel incident remains unconfirmed by this sandbox.

**Permanent opt-in diagnostics:** the page only fills a pre-existing `window.__notayaStartupProbe`; it never creates one in normal use. Fields contain counts/times only, never status values, note text, paths, credentials or IPC arguments/results. `timerScheduled=timerFired=0` is intentional: the new readiness path has no timers. `effectRan` counts layout-effect entries, including gated entries; `startupEffectRan` counts bootstrap-effect entries. `bootstrapAtMs`, `appearanceStartedAtMs`/`appearanceCompletedAtMs`, `statusRequestedAtMs`/`statusSetAtMs`, `lockListenerReadyAtMs`, `screenCommittedAtMs`, `microtaskScheduled`/`microtaskFired`, `readyCleanup`, `invokeCalledAtMs`/`invokeCompletedAtMs`, `appReadyCalls`/`appReadyRejected` locate progress. Baseline/candidate 1 do not have these page counters; zeros/null there mean unavailable, not a failed effect.

| One more run shows | Interpretation |
|---|---|
| Wrong document ID / non-main label, or no bootstrap time on the **new** candidate | Invalid observation document or old/failed bundle. Check `probeDocument`, embedding/build identity and page bootstrap before blaming IPC. |
| Bootstrap present, `startupEffectRan=0` | Startup effect not entered (bootstrap/render/commit or JavaScript failure); not timer cancellation. |
| Appearance started but not completed; status request/set and ready progress succeed | Backdrop IPC is pending; it no longer gates status or readiness. |
| `statusRequestedAtMs` present, `statusSetAtMs=null` | Status IPC is pending/rejected, before the readiness scheduler. |
| Status set, no listener-ready time | Lock-listener registration pending/rejected. Ready intentionally withheld to preserve flush/lock behavior. |
| Status/listener times present, no screen-commit time | React status-screen/layout-effect gate not reached. Inspect DOM/commit/error classes. |
| Microtask scheduled but not fired, cleanup count grows | Effect cleanup cancelled pending work. A fresh eligible effect must reschedule; tests guard this. No timer throttling explanation applies to this scheduler. |
| `invokeCalledAtMs` and `appReadyCalls=1`, no transport count | Transport observer missed/failed its hook, or IPC has not reached transport. Check `fetchHookInstalled`, `nativeHookInstalled`, `callbackHookFailed`; not evidence that App skipped invoking. |
| `readyCalls=1`, completion 0, rejection 0 | Command was dispatched but no success/error callback returned by the bound. If page invoke completion exists, observer is invalid. |
| `readyRejected=1` / `appReadyRejected=1` | IPC rejection, distinguished from Rust's successful hidden-tray return. |
| Calls/completions both 1, `readyAtMs<8000` and before frame release, native hidden | Correct minimized readiness; `readyWithoutFrames=true`. Restored opacity/layout/native visibility must then pass, with pixels/tray checked by a person. White pixels despite that leave the compositor/runtime hypothesis open. |

### Verification and return

Final checks ran on working source on `9b64db2`; the external heartbeat then recorded that identical code as `ba04706`. Its eleven-file fingerprint was checked again after the checkpoint; only this results addendum differs from that commit:

| Command/check | Final exact result |
|---|---|
| `npx tsc --noEmit -p tsconfig.json` | Exit **0**, no errors. |
| `npm run build -- --configLoader native` | Exit **0**, Vite built in **1.85 s**. Existing `INEFFECTIVE_DYNAMIC_IMPORT` warning: `DatabaseView.tsx` is imported statically and dynamically. PowerShell renders this stderr warning as `NativeCommandError` when streams are merged for the tail; npm's exit code is 0. |
| `node --check tools/real-exe-tests/startup-minimized.mjs`, `node --check tools/real-exe-tests/startup-ready.test.mjs`, `node --check tools/real-exe-tests/startup-probe.test.mjs` | All three exit **0**. |
| `node --test --test-isolation=none tools/real-exe-tests/startup-ready.test.mjs tools/real-exe-tests/startup-probe.test.mjs` | Exit **0**, **13 passed, 0 failed, 0 cancelled, 0 skipped, 0 todo**, **372.9614 ms**. Original four scheduling scenarios retained with timer-free expectations, plus startup gate/backdrop/appearance and observer regressions. |
| `powershell -NoProfile -File tools/real-exe-tests/startup-cleanup.test.ps1` | Exit **0**, **8 fake-process cases passed**, no live process control. |
| Windows PowerShell AST `Parser::ParseFile` over `run-startup-minimized.ps1`, `startup-cleanup.test.ps1`, `startup-diagnostic.ps1` | Exit **0**, **0 parse errors** for each of the three files. |
| `git diff --check` and `git diff 9b64db2 --check` | Exit **0**; only LF/CRLF notices. Repeated after the documentation readback. |
| Rust test/build, real exe/CDP, pixels, actual sign-in, tray/auto-lock/protected-vault round, audits and independent security review | **not run** in FIX RUN 2. Rust unchanged; npm audit skipped by instruction. |

Source identity (raw bytes before coordinator newline conversion): SHA-256 `bef3c7455de14edf632c05ea7a4ee7dd7576e32f5ba33e6016c9a25e88fb223a` over the following eleven paths sorted lexicographically, each encoded as UTF-8 path + NUL + raw file bytes + NUL: `src/App.tsx`, `src/lib/ipc.ts`, `src/lib/startup.ts`, `src/lib/theme.ts`, `src/main.tsx`, `src/store/app.ts`, `tools/real-exe-tests/run-startup-minimized.ps1`, `tools/real-exe-tests/startup-cleanup.test.ps1`, `tools/real-exe-tests/startup-minimized.mjs`, `tools/real-exe-tests/startup-probe.test.mjs`, `tools/real-exe-tests/startup-ready.test.mjs`. This identifies tested source now in `ba04706`, not an exe hash or real-exe result.

Changed files: the eleven source/scripts just listed; `tools/real-exe-tests/README.md`, `tools/real-exe-tests/startup-minimized.md`; `docs/plan/2026-11-09-whole-app-test-plan.md`, `docs/plan/2026-11-09-whole-app-test-results.md` (**15 files**). Local tests do not establish Windows rendering, real sign-in, protected-vault behavior or acceptance. Exact real-PC build/control/delay/held-frame commands are in [startup-minimized.md](../../tools/real-exe-tests/startup-minimized.md#fix-run-2-build-and-comparison).

Security self-check, [guide section 4](../handoff/codex-security-review-guide.md#4-checklist-for-notaya-check-each-write-na-when-it-does-not-apply), **self-review, not independent**:

| Checklist area | Scope/result |
|---|---|
| Secrets/keys, zeroization, crypto, OS randomness | **n/a** to production changes; no new secret/key/crypto handling, no vault data/credentials in diff/tests. Harness reload IDs use Node's existing OS-backed `randomUUID`, not production security IDs. |
| Commands/IPC/AI permissions/confirmations | Existing `window_ready` only, no new Rust command or authority. Read native handler and all ready/show call sites: registration precedes tray-hidden return. Main now confirms both lock-event listeners before readiness. Flush acknowledgement and native lock/show policy unchanged; real lock behavior **not run**. AI action registry and confirmation changes **n/a**. |
| Files/paths/parsing | Runner's throwaway-root/reparse/port/Run-key checks unchanged. StartTime is null-safe; retained process handle and recorded ID avoid killing a newly reused PID. Eight fake cases verify the helper, not Windows runtime cleanup. Probe numeric bounds/loopback restrictions remain; observer retains callback IDs only to deduplicate and never records payloads/results. Archive/import/SQL checks **n/a**. |
| AI pipe/bridge | **n/a**, unchanged. |
| Storage/history/logs | **n/a** for vault/history/encryption/decrypted caches. Existing non-secret appearance cache remains allowlisted; revision guard prevents stale cached-startup response from rewriting newer settings/cache. Probe counters exist only on the page object created by the harness, with no disk/log/URL storage or status/note content. |
| Front end | CSP, HTML sinks, clipboard and capture protection unchanged. No production eval/string timer/new HTML sink or network. Readiness uses a function microtask with cancellation and one-send guard. Test VM/CDP code is fixed repository code. Existing screen animations retained. |
| Dependencies/build | No added/changed dependency, version, licence or installs. `npm audit` and `cargo audit` **not run**, no clean-audit claim. Frontend types/build passed on the identified source. |
| Debug/test cleanup | No real process/debug listener opened by Codex. Runner still uses child-only environment, normal Quit and zero-listener requirement. Native cleanup/transport instrumentation **not runtime verified**; Claude runs it on throwaway data. |

No reachable new security finding identified in this self-check. Independent Claude review and security acceptance remain outstanding.

Outside write area, mechanical: **none**. No Rust changes; `cargo test --lib --offline` is **not run** in FIX RUN 2, as requested when Rust is unchanged. No npm audit/ci, cargo audit/clippy, install, real exe, browser, independent security review or network. Claude owns independent `/security-review`, exact commit/hash, records/claims/session/ledger and commit/push. Served model/effort, token counts, quota before/after and message timestamp were not exposed; do not infer them from repository defaults.

**ADDITION FOR AHMED:** the existing window can report ready without waiting for hidden-window frames or timers, and cached backdrop work no longer delays loading its status screen. Start minimized, the current appearance and animations stay as configured. No new user-facing feature. Mohamed Sayed's blank-window report still needs the corrected real-exe comparison and pixel/sign-in observation; the first counter-based conclusion was overstated.

## Run 1 historical root-cause analysis (superseded by FIX RUN 2)

| Candidate | Evidence for | Evidence against / what is still needed |
|---|---|---|
| Hidden-page paint/readiness stall — **most likely code-level cause** | `ui::build_main` creates a hidden WebView. Original `App.tsx` waits for appearance, status and **two animation frames** before `window_ready`. A hidden page can defer those callbacks; there is no fallback. The body and unlock card also animate from opacity zero. This couples readiness to a paint while intentionally hidden. | Code proves the dependency, not that WebView2 actually withheld frames on Mohamed's PC. Every ordinary start uses a hidden window too, so frame suspension alone does not explain why his sign-in differs. A tray restore ordinarily resumes frames. The persistent-white incident still needs observation. The deterministic scheduler test proves the repair of this seam only. |
| Window exposed before the page/status/animation is ready, or hidden-state initialization order | `shell::show_main` (tray, single-instance and legacy handover) unconditionally unminimizes/shows/focuses. It can expose a page during loading. Also, `lib.rs` sets `tray_hidden` only after building the hidden window and the tray; if an early readiness callback could run before setup finishes, it would see the default false flag. | A first Run-key start does **not** call `show_main`. The hidden flag is set in setup before the normal event loop runs; no evidence that readiness can overtake setup has been observed. Need a second instance/tray open, an early callback trace or a focus event at sign-in. No delayed-show thread or automatic native show was found on the first minimized path. |
| Missing `--minimized`, or wrong executable/Run value | `run_command(exe, prefs.start_minimized)` writes the quoted exe and adds the flag only when selected. A stale Run value or Start minimized off could change the path. | Quoting and parsing are correct for the expected command; existing Rust tests cover quoting and flag. `started_minimized` checks exact argument equality. Need Mohamed's actual command shape and executable version. No registry change is proposed. |
| Explorer/tray unavailable at sign-in | If tray creation fails, `lib.rs` deliberately leaves `tray_hidden` false; a ready window becomes visible so the app stays reachable. | Tray failure by itself should show a working page. There is no tray-dependent frontend load branch. Need native tray observation/logs at sign-in. No sleep, Explorer polling or startup semantics change is added. |
| GPU/compositor/WebView2 runtime initialization failure | Strong timing clue: seen at sign-in on one PC. Can explain DOM present but white pixels, or a dead renderer. | No source proof, crash event or runtime version from that PC yet. The harness distinguishes no CDP page, empty DOM, hidden native window and visible layout, but cannot prove compositor pixels. No GPU flags or WebView2 settings are changed. |
| WebView2 user-data folder locked/not ready | WebView2 uses `paths.webview`; session initialization might make it temporarily unavailable. | `ensure_dirs` runs before window creation, checks the data folder, and creation errors get a native startup error and sanitized log. A failed creation is therefore not silently treated as ready. This does not rule out a later renderer/profile failure. Need startup log/event classes and a no-page result. No profile deletion/reset is proposed. |
| Vault-status/appearance IPC stall or startup JavaScript failure | The body is conditional on status, and backdrop IPC is awaited before fetching it. A pending call or module failure can leave only a title bar or an empty root. | `vault_status` returns explicit states for missing, unreadable and protected vaults; there is no minimized-specific lock-screen branch. Logging and ErrorBoundary already exist (module-load errors can precede the boundary). Need DOM and log classes, particularly whether the custom title bar exists. The readiness timer starts only after a status screen commits; it deliberately does not pretend a failed status call succeeded. |

The smallest supported correction removes the unbounded animation-frame dependency: after the status screen commits, retain two-frame readiness normally, with a once-only 1-second timer fallback and effect cleanup. Rust's visibility/tray guard, single-instance handling, vault, settings, lock and screenshot policy are unchanged. This is a fix for a demonstrated startup seam; **the exact cause of Mohamed's persistent blank pixels remains unconfirmed** until the PC round. No before/after runtime timing or pixel result has been measured in this sandbox.

## Run 1 historical source checks and hand-back (2026-11-09)

Source: baseline `e4df9185a99fabf6002065c84c921cd8f8278e31`; the external heartbeat made checkpoints `cff91cc3180c5311203cc41d54cd333bd4d757fb` and `0df2c5ac391baa117d49eb55e653251c8e09c893` during this run. Codex did not commit, stage, push, fetch, switch or reset Git. Checks apply to the final local source represented by the latter checkpoint plus the diagnostic change and this record. The production code in `App.tsx` was unchanged throughout the checks. No real Notaya.exe or browser was run; no install or network operation was performed.

| Check | Exact result |
|---|---|
| `cargo test --lib --offline`, Codex sandbox on Gaming-PC, `CARGO_TARGET_DIR=D:\Notaya-lanes\_target` | **FAILED**, final exit **101**: **713 passed, 1 failed, 4 ignored, 0 measured, 0 filtered**, 138.92 s. Only failure: `shell::tests::the_registry_value_can_be_written_read_and_removed`, `Windows refused to open the start-up list (code 5)`. Six compiler warnings (unused variable and dead code in unchanged files). Final run pinned `TEMP`, `TMP`, `NOTAYA_HOME`, `CIPHERLEAF_HOME` to `D:\Notaya Tests\v1-6-X1\unit-56b67ba0cd614f6db3dad34d3c24e8a5`; full output retained there as `cargo-test.log`. An earlier run used the suite's standard temp directories and had the same counts (180.67 s); the final run corrected fixture placement. |
| `npx tsc --noEmit -p tsconfig.json` | Exit **0**, no errors. |
| `npm run build -- --configLoader native` | Exit **0**; 4413 modules transformed, Vite build 2.54 s. Warning: `DatabaseView.tsx` is imported both dynamically and statically, so the dynamic import does not split that module. |
| `node --check tools/real-exe-tests/startup-minimized.mjs` and `node --check tools/real-exe-tests/startup-ready.test.mjs` | Both exit **0** on final script source. |
| `node --test --test-isolation=none tools/real-exe-tests/startup-ready.test.mjs` | **4 passed, 0 failed, 0 skipped**. Default test isolation was attempted first and failed with sandbox `spawn EPERM`; no claim of a pass for that invocation. |
| Windows PowerShell AST parser, both new `.ps1` scripts | **0 parse errors**. The diagnostic command-redaction function also passed two synthetic assertions (recognized command redacted; unknown arguments omitted), without running live diagnostic collection. |
| `git diff --check` and baseline diff whitespace check | Exit **0**; Git printed only LF/CRLF notices. |
| Real exe/CDP/reproduction, pixels, actual sign-in, protected-vault/auto-lock/tray round | **not run**; Claude runs these outside the sandbox. |
| `npm audit`, `npm ci`, `cargo audit`, `cargo clippy`, independent security review | **not run**. Audit skipped per lane instruction; no installs or dependencies. Claude owns independent `/security-review` and final security decision. |

Raw-file SHA-256 of final executable source/scripts (before any coordinator line-ending conversion):

| File | SHA-256 |
|---|---|
| `src/App.tsx` | `967226269A4035785A908A5489470ABDC513D87B1BB2C553B2CA7C07993BD622` |
| `tools/real-exe-tests/startup-minimized.mjs` | `2D5A4381F867D01F0FBA3A8E42A7F264CD1DE1F3A795AFC96A3487317C671DC9` |
| `tools/real-exe-tests/startup-ready.test.mjs` | `391586C6E1BA0199874338B2D5474164F7BE544EB138C732D5EE77A401550262` |
| `tools/real-exe-tests/run-startup-minimized.ps1` | `4BBDE8C77F03BC6D502E97D5F4449143122F4E2826708737ECAEB3E082AAAE17` |
| `tools/real-exe-tests/startup-diagnostic.ps1` | `38BB3E8DAC1D542B4EC80B78185C72F02F09FB583C448C8802734B45C3A4B1FC` |

Security self-check, guide section 4 (**self-review, not independent**):

| Checklist area | Reviewed result |
|---|---|
| Secrets/keys, zeroization, cryptography, randomness | No new key/secret handling, crypto or random production data: **n/a**. No secrets/vault fixtures in the diff. |
| Commands/IPC and confirmations | Only the existing `window_ready` call is rescheduled, after a committed status screen. Its native `tray_hidden` check and ready/flush registration are unchanged. No new Tauri command, AI authority or confirmation path. Native lock/flush behavior is **not runtime tested** here. |
| Files/paths/parsing | Runner pins child homes/profile under the allowed throwaway root, rejects reparse-point ancestors and existing app processes, validates the exact Run command before direct process creation (no shell evaluation), preserves artifacts, and never writes the Run key. Numeric CDP settings are bounded; WebSocket endpoints must remain on the expected loopback port. Diagnostic exports fixed log classes and allowlisted event fields, never raw text or exe paths. No import/archive/SQL path changed: remaining hostile-file checks **n/a**. |
| AI pipe/bridge | No changes: **n/a**. |
| Storage/history/logs | Vault format, history, decrypted caches and app logging unchanged. Diagnostic reads app-log metadata only and never opens a vault. Storage migration and history checks **n/a** to this diff. |
| Front end | CSP, HTML sinks, storage, capture protection and auto-lock code unchanged. Timer/frame callbacks are functions, once-only and canceled on effect cleanup. Test-only CDP injection and VM evaluation use fixed tool/repository code, not user content, and do not ship in the app. |
| Dependencies/build | No dependency/version/licence change; no installs. Dependency audits **not run**. Test tooling connects only to the explicitly enabled local child debug port; app networking unchanged. |
| Test/debug cleanup | Runner uses child-only environment and normal Quit, with fallback restricted to its process and recorded descendants, and requires zero remaining listeners. No real exe was run here, so this cleanup path is **produced, not runtime verified**. |

No reachable new security finding was identified in the self-check. Runtime, hostile-input review of the wider app, independent review and security acceptance remain separate.

Outside write area, mechanical: **none**. Changed files: `src/App.tsx`; `tools/real-exe-tests/{README.md,startup-minimized.md,startup-minimized.mjs,run-startup-minimized.ps1,startup-ready.test.mjs,startup-diagnostic.ps1}`; the two whole-app plan/results files. No Rust source changed. The whole-app files and helper were absent at the baseline; only the X-1 additions were created, without reconstructing the rest of the test plan. Claude owns the final records, claim/ledger/session updates, review, commit/push and reporter credit in the commit body. Token counts, actual served model/effort and quota readings were not exposed to this run and must not be guessed.

**ADDITION FOR AHMED:** startup timing correction only: if animation frames are delayed, the existing main window can become ready via a timer after its status screen has committed. Normal first-paint timing is retained; `--minimized` still stays hidden when the tray is available. No new visible feature, names, styling or startup setting. Persistent blank pixels remain an open observation question for the real-PC round.
