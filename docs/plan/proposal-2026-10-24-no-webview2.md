# Proposal: run Cipherleaf without WebView2, Edge or other Microsoft components

**Status: PROPOSAL, not approved, not an amendment.** Nothing here changes the approved plan, the code or the authority record. The owner decides (see "Decisions for the owner"). Written 2026-10-24 (UTC) by Claude (Claude Code), cloud thread, from a read-only look at `main` (`f79bab3`), the deep audit and the research listed at the end. No app code was changed and nothing was built or run; every effort figure is an engineering estimate, not a measurement.

**The owner's request (2026-10-24, 08:59:52 UTC):** "Can you come up with a plan so the app doesn't use web view or Microsoft. You know some people deleted Edge because they don't want it."

## 1. The problem, in plain terms

Cipherleaf's interface is a web page (React, TipTap/ProseMirror, KaTeX) drawn by **Microsoft Edge WebView2**, a Chromium engine that Windows supplies. The Rust core (vault, crypto, backups, KeePassXC handling) does not depend on it.

What this costs today:

1. **A hard requirement that is not bundled.** README and the notices say WebView2 must already be on the PC. If it is missing or broken, the audit notes "I double-click and nothing happens" with no message (deep audit, packaging section).
2. **A network side effect.** The live checks (draft [PR #10](https://github.com/AbyssWatcher2/CipherLeaf/pull/10), not merged) saw the WebView2 network process connect to Microsoft addresses within a second of start-up, with Cipherleaf itself and KeePassXC silent. That sits badly with "Cipherleaf makes no connections" (P5) and with the people who do not want Microsoft software.
3. **Weight.** The same checks measured about 410 MB of memory at idle for Cipherleaf plus its WebView2 processes.

**One fact to check first.** Microsoft's answers say that uninstalling the Edge *browser* does not remove the *WebView2 runtime*; they are separate, and Windows features (Widgets, Teams, Office, Outlook) depend on the runtime ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/a/12737586)). So people who "deleted Edge" may still have WebView2, or may have used a removal tool that also took it out. We do not know which. Plan step 0 finds out.

**What "no Microsoft" can mean** (the answer changes the plan):

- **(a)** No Edge browser needed or touched.
- **(b)** No Microsoft *runtime* installed, updated or phoning home on the machine (no WebView2).
- **(c)** No Microsoft code at all in the app, including the Windows API layer. Impossible on Windows itself; only a Linux or macOS port would reach it. This proposal treats Windows as the given platform and stops at (b).

## 2. What the app is made of (measured on `main`)

| Part | Size | Depends on WebView2? |
|---|---|---|
| Rust core: crypto, vault, backups, KeePassXC launch and checks, locks | about 7,300 lines in `src-tauri/src/` | No, except window creation in `ui.rs` and a few `WebviewWindow` parameters in `commands/` |
| UI | about 8,100 lines of TS/TSX in `src/` plus about 20 KB of CSS | Entirely |
| Notes editor | TipTap 3 with 20-odd extensions: headings, lists, tasks, tables, callouts, toggles, columns, page links, comments, code blocks with highlighting, images, KaTeX math, slash menu, bubble toolbar, block drag handle, "turn into" | Entirely (browser `contenteditable`) |
| Stored page format | **TipTap/ProseMirror JSON text** in an encrypted `Page.content` record (`model.rs`), plus a plain-text copy | The format is the real contract between UI and vault |
| Images | Encrypted attachments, served to the page through the custom scheme `clatt` (`http://clatt.localhost`) | Yes, a WebView feature |
| Window effects | Frameless transparent windows, Mica/Acrylic (`window-vibrancy`), `SetWindowDisplayAffinity` capture protection, KeePassXC shown as an owned overlay window | Partly. The Win32 parts work on any window handle. The overlay exists because "WebView2 draws over child windows" (security design) |
| Built-in browser services we get free | Text selection, IME, spell-check, undo, drag and drop, high-DPI, screen-reader support, high contrast | Yes |

Consequence: **the vault format does not change if the new UI keeps reading and writing the same TipTap JSON.** All options below are judged partly on that.

## 3. Options

Effort is in focused engineer-weeks for one developer with Claude doing most of the typing, including the PC testing this project requires. They are rough.

### O0. Keep WebView2, handle absence honestly (about 2 to 4 days)

Detect a missing runtime at start-up, show a native message with what to do, add the "nothing happens" audit fixes, and reword "nothing is sent anywhere" (audit PK-13). Optionally pass WebView2 browser arguments that reduce background traffic (needs a live test; unproven).

- Satisfies: none of (a)/(b); only turns a silent failure into a clear one.
- Licence impact: none. Security: unchanged. Vault: unchanged.

### O1. Ship Microsoft's *Fixed Version* WebView2 runtime inside the ZIP (about 1 to 2 weeks)

Microsoft allows an app to carry a specific runtime version that only that app uses ([Evergreen vs Fixed Version](https://learn.microsoft.com/en-gb/microsoft-edge/webview2/concepts/evergreen-vs-fixed-version)). The app then needs no Edge and no system WebView2. Wry/Tauri would be pointed at the folder (the `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` mechanism; to confirm in a spike).

- Satisfies (a). Does **not** satisfy (b): it is still Microsoft's Chromium, and the network behaviour above must be re-tested, not assumed.
- Size: the ZIP grows by the runtime, in the hundreds of megabytes (exact size to be measured; the current ZIP is 87 MB).
- **We become responsible for updating it.** Microsoft states the Fixed Version "isn't automatically updated". A stale Chromium in a security product is a real cost, reduced but not removed by: no network code, local content only, strict CSP, no remote scripts.
- Licence: Microsoft's redistribution terms for the Fixed Version runtime must be read and added to the notices. Not verified in this research.
- Vault: unchanged. Portability: works on PCs without WebView2; heavier on a USB stick.

### O2. Bundle a non-Microsoft Chromium through Tauri's CEF runtime (about 2 to 4 weeks, but blocked)

Tauri is building a CEF (Chromium Embedded Framework) runtime. Today it is **experimental and arrives with Tauri 3 alpha, not in Tauri 2**, and its docs say bundles grow by hundreds of megabytes, the first build downloads about 1 GB, and on Windows the sandbox does not apply yet ([Tauri docs](https://v3.tauri.app/develop/cef/)). A community extraction for Tauri 2 says its Windows path is "compile-ported blind" and unverified ([SableClient/tauri-runtime-cef](https://github.com/SableClient/tauri-runtime-cef)).

- Satisfies (a) and (b). Keeps the whole UI and the vault unchanged; replaces the `clatt` scheme handler and window glue.
- **Blocked by `CLAUDE.md`:** "Stable releases only. No alpha, beta, rc." It becomes possible when Tauri 3 and its CEF runtime are stable and have been seen running on Windows.
- Licence: CEF is BSD-licensed, Chromium is BSD-3 with bundled third-party parts (including LGPL media code); the Tauri runtime is Apache-2.0 OR MIT. Each must be listed in the notices, as with Chromium-based products generally. We own Chromium updates (same cost as O1).

### O3. Move the shell to Electron, keep the UI (about 4 to 6 weeks)

Electron bundles its own Chromium and is MIT-licensed. React, TipTap and KaTeX would run unchanged. The Rust core would run beside it, most simply as a separate process the UI talks to over a local pipe (a Windows named pipe or stdio), or loaded in-process through a Node add-on.

- Satisfies (a) and (b). Vault unchanged.
- Work: rewrite the shell, the command layer (`src-tauri/src/commands/`), window creation, the `clatt` scheme, capture protection, the KeePassXC overlay hooks, single-instance and the portable-path logic. Re-do the security review for a new IPC model. Electron needs its own hardening list (context isolation, no Node in the page, strict navigation rules).
- Size: roughly 100 to 150 MB on its own. The idle memory is not smaller than WebView2.
- Licence: Electron MIT; Chromium and bundled third-party code as for O2. We own updates.

### O4. A native Rust UI, no browser engine (about 4 to 8 *months*; a plain-text Lite version in about 6 to 10 weeks)

This is the only route that removes the whole web stack. The cost is that **the notes editor must be built, not borrowed.** The browser's editing engine gives us text layout, selection, IME, undo, tables, drag and drop, accessibility and spell-check; a native toolkit gives us little of that for rich text. We would write: a block document model matching the TipTap JSON, rich-text layout and caret handling, tables, columns, toggles, callouts, a slash menu, a block drag handle, image blocks, code highlighting, and an equation renderer.

| Toolkit | Licence | Fit for this app |
|---|---|---|
| **egui** | MIT OR Apache-2.0 | Easy start. Immediate mode with weak multi-line rich-text editing; accessibility is partial (AccessKit). Good for settings, lock screen, lists. Poor for a block editor. |
| **iced** | MIT | Retained, Elm-style. Text editor widget is plain text; no rich text; accessibility is not there yet. |
| **Xilem / Masonry (with Parley text layout, Vello)** | Apache-2.0 | The most promising pure-Rust base for rich text, but young and pre-1.0; API churn would hit a "stable only" rule. |
| **Slint** | `GPL-3.0-only OR Slint Royalty-free 2.0 OR Slint commercial` ([licence file](https://cdn.jsdelivr.net/gh/slint-ui/slint@1.16.0/LICENSE.md)). | Polished, has a software renderer and accessibility work. **Licence trap:** GPL-3 would pull Cipherleaf toward GPL (whose licence is undecided); the Royalty-free option covers proprietary *desktop* apps but has its own terms to read, including attribution. The owner decides Cipherleaf's licence first. |
| **Qt via cxx-qt** | Qt: LGPL-3.0 (dynamic linking) or GPL or commercial; cxx-qt: MIT OR Apache-2.0 | Strongest text and accessibility among native options, and KeePassXC already ships Qt. But a C++ toolchain and Qt DLLs, LGPL compliance (relinking, notices), and still no math or block editor. |
| **Dioxus Native / Blitz** | MIT OR Apache-2.0 | Renders HTML/CSS without a browser, but has no JavaScript, so it cannot host TipTap, and has no `contenteditable`. Alpha. Watch only. |
| **Servo embedding** | MPL-2.0 | A real engine that can run JS, with an embedding crate reported on crates.io in 2026 while the Tauri-Verso effort stalled (secondary source, [blog](https://www.youngju.dev/blog/2026-07-17-servo-embedding-crates-io-verso-archived.en); not verified here). Editing support and Windows maturity unproven. Watch only. |

Further native problems, all to be solved by us:

- **Equations.** KaTeX is JavaScript. Native choices: a LaTeX-to-layout crate (for example `pulldown-latex` for MathML, or Typst's math, whose syntax differs from LaTeX) or an embedded JS engine just for KaTeX. Existing pages with LaTeX would need to keep rendering. Candidates to be proven in a spike, not assumed.
- **Rendering back-ends.** GPU toolkits default to Direct3D 12 on Windows. That is a Windows component, not a runtime, but for a "no Microsoft" reading prefer a Vulkan/OpenGL or software renderer, which may be slow or fail on old PCs and virtual machines.
- **Accessibility and spell-check** that the web gives free would need AccessKit work and a dictionary.
- **The KeePassXC overlay** gets simpler: without WebView2 the "draws over child windows" limit should go away, and KeePassXC could become a true child window (to be tested).
- **Memory and size** drop sharply (an exe of tens of megabytes, no 400 MB browser tree is the reasonable expectation; not measured).
- **Security gains:** no Chromium to patch; a much smaller attack surface for pasted content. **Losses:** a new, unreviewed text engine is itself new attack surface and a new source of data-loss bugs. The audit's F-04 (an unknown block opens as an empty editor and a title edit then overwrites it) shows the cost of getting a document model wrong; a native editor must treat unknown blocks as opaque and preserve them.

### O5. A reduced "Cipherleaf Lite" with a native UI, alongside the full app

A separate edition with plain text and Markdown-style notes, sticky notes, the lock/unlock/backup screens and KeePassXC, built on a native toolkit, reading the same `vault.clf`. TipTap pages open as read-and-edit plain text with a clear "formatting shown simplified, saved safely" rule. This delivers "no browser" sooner than O4 but is a second product to maintain, and it must not silently flatten a page that has columns, toggles or math. Needs the same vault-safety rules as O4 (preserve what you cannot show).

### Not options

- **WinUI / WPF / WinForms:** Microsoft frameworks, the opposite of the request.
- **Qt WebEngine, Ultralight, Sciter, NW.js:** a bundled browser engine again (Qt WebEngine is Chromium) or proprietary/unclear licences; no advantage over O2/O3.
- **Switching host OS only:** on Linux and macOS, Tauri's web view is WebKit (not Microsoft). That is a different request (a port), not a fix for Windows users.

## 4. Comparison

| | O0 keep, explain | O1 fixed WebView2 | O2 Tauri + CEF | O3 Electron | O4 native full | O5 native Lite |
|---|---|---|---|---|---|---|
| No Edge needed | partly | yes | yes | yes | yes | yes |
| No Microsoft runtime | no | no | yes | yes | yes | yes |
| UI code reused | all | all | all | all | none | partly |
| Vault format | same | same | same | same | same JSON kept | same JSON kept |
| ZIP size (est.) | 87 MB | + hundreds of MB | + hundreds of MB | + about 100-150 MB | smaller than today | smallest |
| We patch the engine | no | **yes** | **yes** | **yes** | no | no |
| Allowed by "stable only" today | yes | yes | **no** | yes | partly (Xilem no) | yes (egui/iced/Slint) |
| Effort (est.) | days | 1-2 wks | 2-4 wks after unblock | 4-6 wks | 4-8 months | 6-10 wks |
| Main risk | still needs Edge's runtime | stale Chromium | Windows support unproven | new IPC to re-review | editor quality, data-loss bugs | two products |

## 5. Cross-cutting concerns

**Vault compatibility (`vault.clf`).** Crypto, header, records, backups and restore are Rust and untouched by every option. The only UI-facing contract is `Page.content` (TipTap JSON) and the attachment scheme. Rules for any option: (1) keep the JSON schema as the contract and never rewrite a page into a new format in place; (2) if a new schema is ever wanted, version it, convert on read, keep the old text until the person edits, and take a verified backup first (the V1.0.1 safety-backup machinery exists for this); (3) preserve unknown nodes byte for byte; (4) run the existing restore, lock and Start over scripts against a copy of a real vault from the oldest release.

**Windows portability on a USB stick.** Today the exe depends on a runtime on the host. O1 to O3 make the folder self-contained (a gain) at the cost of size and a bigger unsigned-binary profile for SmartScreen and antivirus. O4/O5 are the most portable if the renderer works on the host's graphics (software fallback required). The `data\` layout, `CIPHERLEAF_HOME` and the profile folder (`data\webview` today) stay inside the stick in every option; a Chromium profile must keep living there and the "no traces" statement in the security design must be rechecked.

**Security design changes.** Any option changes the "UI (WebView2)" row of the trust table and P5 (no connections) wording. O1 to O3 add "a bundled browser engine we must update". O4/O5 remove the browser but add "our own text engine". Update `docs/security-design.md` when one is chosen, and add the network re-test (the PR #10 finding) as a release check. The capability model (UI gets only checked commands, never keys) must survive: in Electron or a separate-process core this is a design task, not automatic.

**Licensing gate (the AGPL lesson, A22).** Before any new dependency enters `Cargo.toml` or `package.json`: read its declared licence and the licences of its dependency tree; reject AGPL, SSPL, "source-available" and non-commercial terms; accept MIT, Apache-2.0, BSD, ISC, MPL-2.0 (file-level copyleft), and LGPL only with dynamic linking and notices; treat GPL/Slint-style dual licences as an owner decision tied to Cipherleaf's own (still unchosen) licence. Add the result to `THIRD-PARTY-NOTICES.md` before release. In section 3, only the Slint licence (read from its licence file) and the Tauri CEF community runtime (read from its repository page) were checked in this research. All other licences are from general knowledge of the upstream projects and must be re-read from the exact crate or package version at adoption.

**Testing.** Everything here needs the Windows PC with throwaway vaults and `CIPHERLEAF_HOME` (CLAUDE.md). The cloud container cannot run any of it, so a spike's result is "built and ran on the PC", never "compiles".

## 6. Recommendation and phases

The cheapest option that fully meets "no WebView2, no Edge, nothing from Microsoft's runtime on the machine" while keeping the product the owner has is **a bundled non-Microsoft Chromium (O2, or O3 if O2 is not ready)**. The option that removes browsers altogether (O4) is a different, much larger project and should be chosen only if the owner wants a small, browser-free app more than the current editor. Meanwhile O0 is worth doing regardless.

1. **Phase 0, find the real problem (about 1 day, no code).** Ask the people who removed Edge what exactly happens: does the app show anything, is "WebView2 Runtime" in Apps & features, which tool removed Edge. Confirm meaning (a) or (b). Merge or read the live-checks PR for the network finding.
2. **Phase 1, O0 (days).** Start-up check with a clear native message, README wording, the "nothing happens" audit item. Ship in a V1.1.
3. **Phase 2, a measured spike (about 1 to 2 weeks, on the PC, throwaway data).** In branches, never in `main`: (i) O1 with a fixed-version runtime, measuring ZIP size, idle memory and network connections; (ii) whichever of O2 or O3 is allowed (Electron now; CEF only if Tauri's CEF runtime has reached stable and runs on Windows). Pass criteria: all existing real-exe scripts, capture protection, KeePassXC overlay, `clatt` images, USB-style move, Start over, restore. Output: a record with measurements, not opinions.
4. **Phase 3, adopt one (2 to 6 weeks).** A new amendment (A24 or next free) with the owner's approval, security-design update, licence entries in the notices, V1.x release with the old build kept available.
5. **Phase 4, optional and separate: O5/O4 research.** A 2-week prototype of one native text editor (Xilem/Parley or egui) opening real TipTap JSON read-only with equations, to decide whether a browser-free edition is realistic. No commitment before the prototype.

If the owner wants **only** a quick relief for people without Edge, do Phase 0 and Phase 1, then O1 as a clearly labelled "no Edge needed" ZIP, and say plainly that it still contains Microsoft's engine.

## 7. Decisions for the owner

1. Which meaning of "no Microsoft": (a) no Edge, (b) no Microsoft runtime, or (c) Linux/macOS too?
2. Is a larger download (hundreds of MB) acceptable for a self-contained build, and are you willing to take on engine updates?
3. Phase 0 first (ask the people who removed Edge), then spike O1 and O3: yes or no?
4. Do you want the browser-free path (O4/O5) researched, knowing it is months of work?
5. Cipherleaf's own licence is still open and affects Slint and any GPL choice.

## 8. What was not verified

Everything in section 3 was researched from documentation and the repository, not run. Unverified: the Fixed Version runtime's real size and redistribution terms; whether Tauri 2 can use a fixed runtime folder without a patch; the Windows state of any CEF runtime; whether transparent windows, Mica and capture protection behave with Chromium builds; Servo's state beyond a secondary source; the licences of crates not named as verified; any effort figure. The network finding comes from an unmerged draft PR (#10).

## Sources

- Tauri CEF runtime page: <https://v3.tauri.app/develop/cef/>
- Tauri CEF community runtime: <https://github.com/SableClient/tauri-runtime-cef> (Apache-2.0 OR MIT)
- WebView2 Evergreen vs Fixed Version: <https://learn.microsoft.com/en-gb/microsoft-edge/webview2/concepts/evergreen-vs-fixed-version>
- Edge uninstall versus WebView2 (Microsoft Q&A): <https://learn.microsoft.com/en-us/answers/a/12737586>
- Slint licences: <https://cdn.jsdelivr.net/gh/slint-ui/slint@1.16.0/LICENSE.md>
- Servo/Verso status (secondary): <https://www.youngju.dev/blog/2026-07-17-servo-embedding-crates-io-verso-archived.en>
- In this repository: `docs/audit/2026-10-23-deep-audit.md`, `docs/security-design.md`, `THIRD-PARTY-NOTICES.md`, `src-tauri/src/ui.rs`, `src-tauri/src/model.rs`, `src/features/notes/editor/`; draft PR #10 (live checks).
