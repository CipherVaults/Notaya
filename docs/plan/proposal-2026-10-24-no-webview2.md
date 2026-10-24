# Proposal: a native Windows Cipherleaf with no browser engine

**Status: PROPOSAL, not approved, not an amendment.** Nothing here changes the approved plan, the code or the authority record. The owner decides (section 12). Written 2026-10-24 (UTC) by Claude (Claude Code), cloud thread, from a read-only look at `main` (`f79bab3`), the deep audit and the research listed at the end. No app code was changed and nothing was built or run. Every effort, size and speed figure is an engineering estimate, not a measurement.

**The owner's requests (2026-10-24, project thread, UTC):**

- 08:59:52: "Can you come up with a plan so the app doesn't use web view or Microsoft. You know some people deleted Edge because they don't want it."
- 09:07:13: "I mean if anyone doesn't have Microsoft Edge, what should we do then."
- 09:11:59: "I meant by local app everything local in build. I want it as Native app so people with Windows 10 or 11 and whatever the updates they're in."
- 09:12:34: "Also I want it to be exactly as we did."

**How this proposal reads them:** a truly native Windows program, no browser engine of any kind, everything it needs inside the build, working on any Windows 10 or 11 at any update level, and looking and behaving like the current app. "Native" here means it uses Windows itself (windows, input, fonts, accessibility, spell-check) and nothing that has to be installed on top (no WebView2, no Edge, no .NET, no Visual C++ redistributable). If "no Microsoft" was also meant to exclude the Windows API layer, that is impossible on Windows; please say so if that was the intent (section 12).

## 1. Requirements this plan is built on

| # | Requirement | Source |
|---|---|---|
| R1 | No browser engine at all (no WebView2, CEF, Electron, Servo, Qt WebEngine) | 08:59, 09:11 |
| R2 | Self-contained: one folder, runs from a USB stick, nothing to install, works on Windows 10 (any build still supported by current Rust, see section 8) and Windows 11 at any update level | 09:11 |
| R3 | **Parity:** same layout, features, look and behaviour as the current app | 09:12 |
| R4 | The existing `vault.clf` opens unchanged, and the old and new app can both read the same vault | implied; section 7 |
| R5 | Stable releases only, no AGPL surprises, licences recorded in the notices | `CLAUDE.md`, A22 lesson |
| R6 | Security promises keep holding (keys stay in the Rust core, no network, capture protection, locks) | `docs/security-design.md` |

## 2. What exists today (measured on `main`)

| Part | Size | Native rewrite impact |
|---|---|---|
| Rust core: crypto, vault, backups, restore, KeePassXC launch and checks, locks, idle monitor, portable paths | about 7,300 lines in `src-tauri/src/` | **Kept.** Only the Tauri glue changes (section 7) |
| UI | about 8,100 lines TS/TSX in `src/`, about 20 KB CSS, React 19, Radix, motion, dnd-kit, cmdk, sonner, lucide | **Rewritten** |
| Notes editor | TipTap 3 (ProseMirror) with about 20 extensions, KaTeX, lowlight | **Rewritten, the hardest part** (section 5) |
| Stored page format | TipTap/ProseMirror JSON text inside an encrypted record (`Page.content`), plus a plain-text copy (`Page.text`) | **The contract to keep** |
| Window layer | `ui.rs`: frameless transparent windows, Mica/Acrylic via `window-vibrancy`, capture protection, sticky windows, KeePassXC overlay (`embed.rs`) | Mostly kept; Win32 code works on any window handle |

## 3. Parity: the inventory and what each part honestly needs

"Exactly as we did" has to be made testable, or it cannot be checked. The reference is the current UI, which can be shown with fake data in a plain browser (`npm run mock`, `tools/mock-screenshots/`). Before any native work, Phase 0 captures a **reference set**: every screen below in each of the 4 looks (Black, Graphite, White, Paper), at 2 window sizes and 100% and 150% scaling. The native build is compared with it screen by screen; the owner signs each screen in `docs/acceptance.md` (only the owner accepts).

**Feasibility key:** **Same** = can match to within a pixel or two on layout and exactly on colours. **Close** = looks the same at a glance, differs in fine detail. **Gap** = will be worse or different at first, even when done well.

| Area | What the current app does | Native feasibility |
|---|---|---|
| Looks and tokens | 4 looks plus "follow Windows", 9 accent colours plus custom, 4 fonts (Inter Variable, Segoe UI Variable, Georgia, JetBrains Mono), text size slider, 4 page widths, 2 densities, soft shadows, translucent surfaces | **Same** for colours, sizes, spacing, fonts (bundled Inter and JetBrains Mono, OFL). **Close** for shadows and translucency (blur needs a renderer that supports it) |
| Window frame | Frameless, transparent, custom title bar, drag and resize, Mica/Acrylic on Windows 11 and solid colour on Windows 10 | **Same** (the `window-vibrancy` call works on any window handle) |
| Text rendering | Chromium text: ClearType or grayscale, its own hinting | **Close**: glyph shapes and spacing differ slightly in every non-browser renderer. Cannot be pixel-identical |
| Lock and unlock screen, first run, Start over, restore dialogs, security dialogs | Forms, password strength meter, keyfile picker, background styles, error wording | **Same** (plain forms and dialogs) |
| Sidebar | Notebook, section and page tree, favourites, trash, drag to reorder, context menus, keyboard tree (V1.0.1), resize handle, lucide icons, emoji page icons | **Same** for layout and icons (icons converted to paths). **Close** for emoji (colour emoji font rendering, to prove). Drag and drop and the keyboard tree must be rebuilt |
| Page header | Emoji icon, gradient or image cover, big title field | **Same** |
| Notes editor | Section 5 | Mixed: see the table there |
| Sticky notes | Separate small frameless always-on-top windows, 8 colours, basic and scientific calculator inside | **Same** (the calculator logic `calc.ts` is plain maths and is ported) |
| Command palette | cmdk search over pages and actions | **Same** |
| Settings | Appearance, Security, Passwords, Backups, About with licence viewer | **Same** |
| Passwords | Bundled KeePassXC shown as an overlay window on top of Cipherleaf | **Same**, same limit as today: popups drawn by Cipherleaf cannot appear above the overlay unless they are their own windows |
| Toasts, tooltips, menus, confirm dialogs | Radix and sonner | **Close**: re-implemented; focus rules and keyboard handling must be rebuilt by hand |
| Animation | `motion` springs and enter/exit transitions | **Close**: tweened in Rust; the feel needs tuning by eye |
| Scrolling, selection, copy and paste, drag of text | Browser behaviour, smooth scrolling, rubber-band selection | **Close to Gap**: native toolkits scroll and select differently in small ways |
| IME and complex scripts | Browser handles CJK composition, right-to-left, shaping | **Gap until proven.** Must be tested (section 5) |
| Spell-check | The browser underlines mistakes | **Gap** unless wired to the Windows spell-checking API (available since Windows 8, nothing to install) |
| Screen readers | Browser exposes the page to NVDA and Narrator | **Gap.** AccessKit, the usual accessibility layer for Rust toolkits, lists "rich text and hypertext" as not supported yet. Controls can be made accessible; the document body reads worse at first |
| Right-click menus in text, Ctrl+scroll zoom, find in page, printing | Browser defaults | Only what we rebuild. Anything not on a list the owner confirms will be absent |

**What cannot match, stated plainly:** pixel-identical text; the breadth of the browser's built-in behaviour (accessibility, spell-check, IME, bidi) on day one; and exactly the same smoothness without tuning. The plan treats these as work items with tests, not as promises.

## 4. Choosing the native toolkit

### Criteria

No browser (R1); no runtime to install (R2); runs on machines with no or weak GPU (R2); licence (R5); accessibility; quality of text layout and editing, because the editor is the product; control over drawing, because R3 means custom-drawn widgets.

### The candidates

| Toolkit | Licence | Strengths | Problems for this app |
|---|---|---|---|
| **Masonry (with Parley text, Vello rendering; Xilem is the optional declarative layer on top)** | Apache-2.0 (Linebender projects; to re-read per crate) | Built on AccessKit; Parley is the most capable pure-Rust text layout (styled spans, selection, IME hooks); can render on the CPU with no GPU (reported in Linebender's 2026 Q1 update); custom widgets are first class | **Pre-1.0, API churn.** Rich-text editing widgets do not exist; we write them. Windows details not covered in the sources read here |
| **iced** | MIT | Mature API (0.14), CPU renderer (`tiny-skia`) and GPU renderer, `rich_text` spans and a plain `text_editor` widget, cosmic-text layout | No accessibility layer (an open issue upstream). No blur shadows on the CPU renderer. Rich-text *editing* is still ours to write |
| **egui** | MIT OR Apache-2.0 | Easiest start; AccessKit integration | Immediate mode re-lays out every frame: poor for long rich documents. Hard to hit pixel parity with a CSS-designed UI. GPU or OpenGL only in the official backends |
| **Slint** | GPL-3.0-only OR Slint Royalty-free 2.0 OR Slint commercial ([licence file](https://cdn.jsdelivr.net/gh/slint-ui/slint@1.16.0/LICENSE.md)) | Polished, software renderer, declarative styling | Licence tied to Cipherleaf's still-undecided licence. `StyledText` is "a first step toward full rich text" (1.16 release notes), and it has no surface for a custom text-editing widget. Rejected |
| **Qt through cxx-qt** (not Rust-native) | Qt LGPL-3.0 (dynamic linking) or GPL or commercial; cxx-qt MIT OR Apache-2.0 | Most mature text editing, tables, IME, accessibility and spell-check hooks of any option; stable releases | C++ toolchain, LGPL compliance (notices, relink right), 20 to 40 MB of Qt DLLs, Qt 6 raises the minimum Windows 10 build (to confirm), and the block editor is still ours (no callouts, toggles, columns, math). The honest "safe but not pure Rust" fallback |
| **Win32 + DirectWrite/Direct2D with `windows-rs`** | Windows APIs (no redistribution) plus MIT/Apache crates | Zero dependencies beyond the OS, smallest exe, best control, system text shaping, colour emoji and IME through the OS | Everything is ours, including widgets, layout, text input service integration and the accessibility provider. Highest effort. Fallback of last resort |
| Dioxus Native / Blitz, Servo, Floem, Makepad, GPUI | various | Interesting | Alpha, no JavaScript, or editor-oriented rather than document-oriented; not suited for a 2026 production editor. Watch only |

### Recommendation

**Masonry with Parley and Vello, with iced (CPU renderer) as the planned fallback and Qt as the out-of-Rust fallback.** Reasons: it is the only Rust stack where accessibility and a serious text engine come together, it has a CPU rendering path so it works on any PC, its licence is permissive, and custom widgets are the normal way to build in it. Its weakness is that it is young. So the choice is **not final until the Phase 1 spike passes** (section 9). Pinning exact versions is already the project's rule (`.npmrc`, `Cargo.lock`).

**Rule check:** `CLAUDE.md` says "stable releases only, no alpha, beta, rc". Masonry and Parley publish 0.x releases that are not tagged pre-release but are described by their authors as early. Whether that satisfies the rule is the owner's call (section 12). If the answer is no, the choice is Qt.

## 5. Rebuilding the notes editor

The editor is not a text box with formatting buttons. It is a block document engine. Today the browser provides cursor, selection, text layout, input methods, undo and drag/drop and TipTap provides the document model on top. Natively we write both.

### 5.1 Architecture

1. **Document model in Rust** (in the core crate): a typed tree mirroring the TipTap schema (node and mark names, attributes) with **an "unknown" variant that keeps any node or attribute it does not understand, exactly as read**. This also fixes audit F-04 (a page with an unknown block opens empty and a title edit overwrites it) as a side effect.
2. **Layout:** convert the tree into laid-out lines with Parley (paragraph flow, list markers, table grids, callout boxes, columns, quote bars); cache per block so typing re-lays-out only the changed block; virtualise long pages (only draw what is visible).
3. **Editing operations** as small, testable transforms: insert text, delete, split and join blocks, indent and outdent lists, toggle marks, change block type ("turn into"), move blocks, table row and column operations. Each is a pure function over the model, so they are unit-tested without a window (the project already tests this way).
4. **Selection and caret:** a model position (block path and offset) mapped to pixels by the layout; multi-block selection; keyboard (arrows, Home/End, Ctrl+arrows, Shift) and mouse (click, drag, double and triple click).
5. **Input methods:** Windows text input through the toolkit (IME composition for Chinese, Japanese, Korean; dead keys; the emoji panel). Right-to-left text and complex shaping come from the text engine's bidi and shaping support. These are listed as risks, because they are exactly where non-browser editors fail first.
6. **Undo and redo** with typing grouped as in the editor today; **autosave through the existing flush protocol** (pending edits are saved before any lock; `flush.ts` logic is ported, the 2-second wait stays).
7. **Clipboard:** plain text and an internal rich format; pasted images become encrypted attachments through the existing Rust path; no HTML parser in v1 (see the gap list).

### 5.2 Feature by feature

| Feature in the editor today | Native approach | Feasibility |
|---|---|---|
| Paragraphs, headings 1 to 4 | Layout with the theme's sizes | Same |
| Bold, italic, underline, strike, inline code, links (Ctrl+click opens, http/https only), highlight with colours, text colour, font family and size, alignment | Text style spans in Parley | Same |
| Bullet, numbered and task lists, nested | Block layout with markers and indentation | Same |
| Quote, divider, callout ("highlight box"), toggle (collapsible), columns | Custom block widgets | Same (columns and toggle need the most care) |
| Code block with syntax colours | `syntect` (MIT) or tree-sitter highlighting instead of highlight.js; monospace layout | **Close:** colours and language coverage differ |
| Tables (resizable columns, add and remove rows and columns, header row) | Grid layout and column-drag handles | Same, a large item |
| Pictures (paste, drop, file picker, encrypted attachments) | `image` crate decoding PNG, JPEG, GIF, WebP; same Rust attachment commands | Same; animated GIF needs a frame timer |
| Equations, block and inline | A pure-Rust KaTeX-compatible renderer: **RaTeX** (MIT; its README claims over 99.5% KaTeX syntax coverage and shows PNG, SVG and PDF output; it uses KaTeX's own fonts, which are what the app shows today). Maturity unverified: tested in the Phase 1 spike against the real equations in test vaults | **Close**, if the spike passes. Fallback: Typst's math or a MathML layout crate |
| Slash menu (`/`), bubble toolbar on selection, block handle (drag to move, menu, "turn into") | Popups and overlays; drag logic ported from dnd-kit's behaviour | Same, but each is a project of its own |
| Markdown shortcuts (`#`, `-`, `1.`, `[]`, `**bold**`), keyboard shortcuts | Input rules table | Same |
| Sub-pages, page links, comment marks, today's date | Model nodes plus calls to the existing Rust commands | Same |
| Find, spell-check, right-click text menu | Not in the current editor except browser defaults | Rebuilt only if the owner lists them |
| Reading by a screen reader | AccessKit nodes per block (heading, list item, table, link) | **Gap**, see section 3 |

### 5.3 The risk, in one line

The editor is the largest piece of work in this plan (roughly half of it), and it is where a data-loss bug would be worst. So the document model and its round-trip tests come before any window (Phase 0), and the editor has a go/no-go spike before the rest is built (Phase 1).

## 6. Self-contained build and Windows 10 and 11 at any update level

- **No runtimes:** static C runtime (already done: the live checks found no `vcruntime140.dll` import), no WebView2, no .NET. The exe uses only Windows system libraries that every Windows 10 and 11 has.
- **Rendering:** GPU rendering through Windows' own graphics stack (Direct3D 12 or Vulkan, chosen by the library) **with an automatic CPU fallback**, so a virtual machine, remote desktop or old graphics driver still works. This fallback is a requirement of R2, to be tested on a machine with no GPU.
- **Windows 10 versions:** current Rust requires Windows 10 for the MSVC target, so very old Windows 7 and 8 are out (they already are for the current app). Windows 10 builds from 1507 on need a test pass (suggested: a Windows 10 1607-era VM, a Windows 10 22H2 PC, a Windows 11 PC). Mica needs Windows 11 and falls back to a solid colour on Windows 10, as today. Capture protection already uses the best call available on the build (to confirm in the test pass).
- **Fonts:** Inter and JetBrains Mono and the equation fonts are embedded in the exe or sit in the folder. System fonts (Segoe UI Variable on Windows 11, Segoe UI on 10, Georgia) are found at start-up with fallbacks.
- **Size and memory (estimates, not measured):** an exe of roughly 15 to 30 MB; idle memory well under the roughly 410 MB measured today for the current app and its WebView2 processes (PR #10). Both must be measured in the spike.
- **No network:** the native app removes the WebView2 start-up connections to Microsoft addresses found in PR #10, which also makes promise P5 simpler to keep.
- **Packaging:** the same portable ZIP shape (exe, `Licenses\`, bundled KeePassXC, `data\` created on first run). KeePassXC and its Visual C++ redistributable note stay as they are.

## 7. Keeping `vault.clf` and the Rust backend

**What stays unchanged:** `crypto.rs`, `vault/`, `model.rs`, `openvault.rs`, `restore.rs`, `pwbackup.rs`, `keepassxc.rs`, `generator.rs`, `portable.rs`, `system.rs`, the Windows-API code in `embed.rs`, and the logic inside `commands/` (the sources of promises P1 to P21, and the unit tests that support them).

**What changes:** the Tauri parts only. `commands/*` functions take `tauri::State`, `AppHandle` and `WebviewWindow`; `state.rs` and `lib.rs` use Tauri events and window handles. Step one is to move the pure logic into a `cipherleaf-core` crate with no Tauri dependency (the deep audit already proposes this split so that continuous integration can run the tests on Linux), keep `src-tauri` as a thin Tauri shell over it, and add the native shell as a second thin client over the same crate. **Both apps then exist in parallel on the same core**, so the current app keeps shipping until the native one is accepted.

**Vault compatibility rules:**

1. No change to the vault header, record format or crypto. The native app reads and writes the same file.
2. `Page.content` stays TipTap JSON in the same schema. The native editor never "upgrades" a page to a different format.
3. Unknown nodes and attributes are preserved byte for byte (section 5.1) so an older or newer app never loses data.
4. `Page.text` (the plain-text copy used for search and word counts) is produced by the same rules as today; a test compares both implementations on a corpus.
5. Attachments keep their ids and the PNG/JPEG/GIF/WebP checks; the `clatt` web scheme disappears because the native UI asks the core for bytes directly.
6. Settings keep the same record. Appearance values that the web UI keeps in local storage (a cache for the lock screen) get a native equivalent in `data\`.
7. A corpus test: open a copy of the oldest real vault from V1.0 and a generated vault with every block type, load every page, save it unchanged, and require semantically identical JSON.

## 8. Security, accessibility and testing impact

- **Security design** needs a rewrite of the "UI (WebView2)" trust row. The UI and the core would be in one process as now; keys still stay in the core. Gains: no Chromium to patch, no web content engine, no CSP to maintain. New surface: our own text and layout code and a pile of new crates (each a supply-chain item for `cargo audit`/`cargo deny`, which should be installed and run). Rich content is parsed only from our own JSON.
- **Accessibility:** controls are accessible through AccessKit and Windows UI Automation; the document body is the known gap. A screen-reader pass with NVDA and Narrator is a release criterion, not a nice-to-have. If the gap is unacceptable for the owner, that decides against pure Rust (Qt has stronger text accessibility).
- **Testing:** today's real-exe scripts drive the app through a WebView2 debug port (`tools/real-exe-tests/`). A native app has no such port. The test harness must be rebuilt (an internal test hook gated out of release builds, or Windows UI Automation), and the owner's rules still hold: throwaway vaults only, `CIPHERLEAF_HOME` in a temp folder, nothing driven on the owner's window or vault. This is real work and is in the phases.
- **Everything must run on the owner's PC.** The cloud container cannot show a Windows window.

## 9. Phases, with the smallest useful first milestone

Effort is in focused calendar weeks for one developer working with Claude, including PC testing. The total is a wide range because the editor is uncertain.

| Phase | What | Weeks | Output and gate |
|---|---|---|---|
| **0. Groundwork** | Owner decisions (section 12). Reference set of screenshots from the mock UI for every screen, look and scale. Feature checklist the owner confirms (section 3). Split `cipherleaf-core` from Tauri with no behaviour change. Document model in Rust plus round-trip tests over a corpus of real pages. | 2 to 3 | Current app unchanged and still passing its tests; core crate and corpus tests merged. **Useful even if the native app is never built.** |
| **1. Spike and go/no-go** | One week each, throwaway branches: (a) Masonry/Parley/Vello, (b) iced with the CPU renderer. A single page view with real text, a caret, selection, typing, IME, undo, and one equation, one table, one image, on the PC. Plus a Windows 10 VM and a no-GPU run. Qt only if both fail. | 2 to 3 | A written comparison with measurements. Go/no-go and final toolkit choice by the owner. |
| **2. Milestone M1: read-only vertical slice** | Native window with frame, 4 looks and fonts; unlock/first-run screens; sidebar tree; open **any page read-only** (all block types, equations, images); settings reading; lock sequence and idle lock; KeePassXC overlay. No editing yet. | 6 to 8 | The owner opens their real-looking test vault in the native app and compares screens with the reference set. Proves vault compatibility, look and rendering. |
| **3. Editing** | Typing, marks, lists, slash menu, bubble toolbar, block handle and drag, tables, images, equations, callouts, toggles, columns, sub-pages, page links, comments, undo, autosave, flush. | 10 to 14 | Editing parity checklist green on the PC; data-loss tests (kills during saves) repeated on the native app. |
| **4. Everything else** | Sticky notes and calculators, command palette, backups and restore dialogs, Start over, security dialogs, trash, favourites, drag in the sidebar, spell-check, toasts. | 5 to 7 | All screens in the reference set implemented. |
| **5. Parity, access and release** | Side-by-side review per screen, accessibility pass, Windows 10 and 11 matrix, no-GPU test, memory and size measurement, security review, notices and licences, real-exe test harness, packaging. | 4 to 6 | Owner sign-off per screen; V2.0 beta ZIP beside the current V1.x. |

**Estimated total: about 29 to 41 weeks (7 to 10 months).** This is a guess, not a measurement, and the editor phases dominate it.

**Smallest first milestone, and what it proves:** Phase 0 plus the Phase 1 spike (about 4 to 6 weeks). It answers the question the whole plan depends on (can a non-browser editor be good enough) before the owner commits months.

## 10. Licences

Declared licences as known from the upstream projects; they must be re-read from the exact crate versions at adoption, and `cargo deny` should enforce the allow-list. Only the entries marked "read" were actually read in this research.

| Component | Licence | Notes |
|---|---|---|
| Masonry, Parley, Vello and related Linebender crates | Apache-2.0 (some Apache-2.0 OR MIT) | Not read in this research |
| winit, wgpu | Apache-2.0 / MIT | Not read |
| AccessKit | Apache-2.0 OR MIT (read from its README; parts derived from Chromium under a BSD-style licence) | Notice needed for the Chromium-derived parts |
| iced | MIT | Fallback; not read |
| RaTeX (equations) | MIT (read from its README) | Needs KaTeX fonts (SIL OFL, already in the notices) |
| syntect, `image` | MIT / MIT OR Apache-2.0 | Not read |
| Inter, JetBrains Mono, KaTeX fonts | SIL OFL 1.1 | Already in the notices |
| lucide icons | ISC | Convert icons to paths at build time |
| Slint | GPL-3.0-only OR Royalty-free 2.0 OR commercial (read) | Not chosen |
| Qt (fallback) | LGPL-3.0 / GPL / commercial | Not chosen |
| Windows APIs | Part of Windows | No redistribution |

**Gate (A22 lesson):** before any crate or font enters the project, read its declared licence and its dependencies' licences. Reject AGPL, SSPL, non-commercial and "source-available" terms. Accept MIT, Apache-2.0, BSD, ISC, OFL fonts and MPL-2.0 at file level. Treat LGPL and GPL choices as owner decisions tied to Cipherleaf's own licence, which is still unchosen.

## 11. Interim step for people without Edge today

The native app is months away. The current V1.0 exits with no message when the WebView2 runtime is missing (audit F-13; code path confirmed, never run on such a PC). Edge being removed does not remove the runtime; a PC without the runtime shows nothing on double-click. A small, separate V1.1 change closes that gap in 2 to 4 days: check for the runtime at start-up and show a native message with a link to Microsoft's runtime download, plus a README entry. That is not part of this proposal and needs the owner's go-ahead. (An optional ZIP carrying Microsoft's Fixed Version runtime is possible but contradicts R1; not recommended now.)

## 12. Decisions for the owner

1. **Confirm the reading in the opening of this proposal:** no browser engine, nothing to install, Windows APIs themselves are fine. Anything else?
2. **Does "stable only" allow 0.x crates** (Masonry, Parley) that are published releases but described as early, with exact pinned versions? If not, the choice becomes Qt.
3. **Which screens and behaviours are in the "exactly as we did" list?** The feature checklist (sections 3 and 5.2) needs the owner's yes or no on: spell-check, find in page, right-click text menu, printing, pasted web formatting (HTML), screen-reader support level.
4. **Accept the gaps?** Text rendering will not be pixel-identical; the editor will read worse to screen readers at first; IME and right-to-left text are tested in the spike, not assumed.
5. **Appetite:** about 7 to 10 months, with the first useful answer in about 4 to 6 weeks. Start with Phase 0 and the Phase 1 spike, or only Phase 0?
6. **Keep shipping the web-view app (V1.x) until the native app is accepted, and ship the small no-Edge message fix (section 11)?**
7. Cipherleaf's own licence is still open and affects Slint or any GPL choice.

## 13. What was not verified

Nothing was built or run. Not verified: every effort, size, speed and memory figure; Masonry/Parley/Vello's Windows behaviour, IME, bidi and editing readiness; whether Slint and egui accessibility claims hold today; RaTeX's accuracy against the app's real equations; the minimum Windows 10 build for Qt 6 and for the CPU/GPU paths; colour emoji rendering in the chosen renderer; licences not marked "read" in section 10; whether the Windows spell-checking API works from a portable app on every Windows 10 build. The network finding cited comes from unmerged draft PR #10.

## Appendix A. Options considered and set aside

| Option | Why not now |
|---|---|
| Keep WebView2 with a clearer error | Interim only (section 11); does not meet R1 |
| Bundle Microsoft's Fixed Version WebView2 | Meets "no Edge" but not R1; adds hundreds of MB; we would own Chromium updates |
| Tauri CEF runtime | Experimental in the Tauri 3 alpha; Windows unverified; blocked by "stable only"; still a browser engine ([docs](https://v3.tauri.app/develop/cef/), [community runtime](https://github.com/SableClient/tauri-runtime-cef)) |
| Electron | MIT, keeps the UI, but a bundled Chromium (breaks R1) and a rewritten shell |
| Servo, Blitz | Browser-like engines, immature for an editor |
| WinUI, WPF, WinForms | Need .NET or Windows App SDK runtimes; break R2 |

## Sources

- Linebender in 2026 Q1: <https://linebender.org/blog/tmil-25/>
- AccessKit: <https://github.com/AccessKit/accesskit> (Apache-2.0 OR MIT; Windows adapter; rich text not supported yet)
- RaTeX: <https://github.com/erweixin/RaTeX> (MIT)
- Slint licences: <https://cdn.jsdelivr.net/gh/slint-ui/slint@1.16.0/LICENSE.md>; Slint 1.16 release notes: <https://slint.dev/blog/slint-1.16-released>
- iced 0.14: <https://github.com/iced-rs/iced/releases/tag/0.14.0>
- Edge uninstall versus WebView2 (Microsoft Q&A): <https://learn.microsoft.com/en-us/answers/a/12737586>
- WebView2 Evergreen versus Fixed Version: <https://learn.microsoft.com/en-gb/microsoft-edge/webview2/concepts/evergreen-vs-fixed-version>
- In this repository: `docs/audit/2026-10-23-deep-audit.md` (F-04, F-13), `docs/security-design.md`, `THIRD-PARTY-NOTICES.md`, `src-tauri/src/` (`ui.rs`, `model.rs`, `state.rs`, `embed.rs`), `src/` (UI), `tools/mock-screenshots/`; draft PR #10 (live checks).
