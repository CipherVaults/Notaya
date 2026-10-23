# Cipherleaf deep audit, 2026-10-23

- **Audited source:** `main` at `31197e0` (code identical to release `v1.0`, tag on `0ed3f52`; `git diff 1efad95 0ed3f52 -- src src-tauri` is empty). While the audit ran, `main` moved to `e09699f` (PR #7: notices and `packaging/Licenses/`, docs only). No source file under `src/` or `src-tauri/` changed.
- **Requested by:** the owner ("run deep audit... check its codes, check its security, anything and everything"), 2026-10-23 21:48:54 UTC (message timestamp). Report written at about 22:20 UTC (system clock).
- **Method:** read-only. Eight scoped review agents (backend, cryptography, application security, dependencies, frontend, tests, accessibility and usability, packaging), then two independent verifier agents that tried to refute the top claims. All of this is model output. No code was changed in the repository.
- **What ran:** `npm audit`, `cargo audit` and `cargo deny` (RustSec database of 2026-10-23, 1351 advisories), `tsc`, `vite build`, `node dev-mock/calc-check.cjs`, a Tauri-free copy of the pure Rust modules (`cargo test --lib`: 114 of 127 tests, the other 13 need the Tauri runtime), 12 extra crypto tests and 3 hostile-file smoke tests in scratch crates, KaTeX and TipTap in Node and in headless Chromium against the mock UI, axe-core on every mock screen.
- **What did not run:** anything on Windows. No real exe, no WebView2, no USB removal, no idle lock, no Win+L, no sleep, no screen reader, no SmartScreen, no real vault. The PC was not used.

Severity scale: **Critical** = data loss or secret exposure reachable by a realistic actor. **High** = serious, likely to hit real users. **Medium** = real defect or gap with limited reach or a clear workaround. **Low** = hardening or small defect. **Info** = note. Confidence: **CONFIRMED** (traced in code or reproduced), **PLAUSIBLE** (reasoned, not run end to end). "Verified" means a second, independent agent re-checked it.

## 1. Summary

**No Critical findings. No remote-exploit path was found.** Cipherleaf has no network code, a strict CSP, no HTML injection sink in the UI, and no file-system, shell or HTTP permission for the web view. The cryptographic construction is sound: Argon2id then HKDF, a random 256-bit data key, XChaCha20-Poly1305 with random nonces, associated data that binds version, vault id, table and row, an authenticated header, and an all-or-nothing re-key. Nobody found a way to read, forge or silently change a single record without the key.

What the audit did find falls into four groups:

1. **Data safety (the biggest group).** The backup safety net is less reliable than the documents say, and since V1.0 made "security off" the default, most new vaults take a backup only when the app closes and never react to the drive being pulled. A page whose stored content this build cannot read is shown empty and is then overwritten on the next edit. Deleting a notebook or section is permanent after one confirmation.
2. **Key lifetime.** Changing the master password does not change the data key, so a leaked old password plus any old backup still opens data written later.
3. **Honesty of wording.** The screens say "encrypted" and "Saved & encrypted" for vaults that are not confidential. Some documents overstate (deletion detection, drive removal, the KeePassXC check).
4. **Accessibility and process.** The sidebar tree cannot be used from the keyboard, several text colours fail WCAG AA, right-to-left text is not supported, and there is no CI, no dependency gate and no packaging script.

Counts after merging duplicates (each finding appears once; the same issue reported by several agents is one item): **0 Critical, 1 High (accessibility), about 24 Medium, about 40 Low, about 15 Info.** (Counts are approximate because several register rows bundle related items.) The register in section 4 lists every item.

## 2. Top 10, ranked

| # | Finding | Severity | Verified |
|---|---|---|---|
| 1 | F-01 Changing the master password does not rotate the data key or vault id | Medium | Yes |
| 2 | F-02 The backup safety net can silently fail or delete the newest copy | Medium | Yes |
| 3 | F-03 A vault with security off cannot recover after the drive is pulled, and takes a backup only at quit | Medium | Partly (code yes, Windows behaviour no) |
| 4 | F-04 A page with content this build can't read opens empty and is overwritten on the next edit | Medium (latent) | Yes, reproduced |
| 5 | F-05 Deleting a notebook or section is permanent, with no pre-delete backup | Medium | Yes |
| 6 | F-06 Screens say "encrypted" while the default vault is not confidential | Medium | Yes |
| 7 | F-07 One large equation freezes the app on every open | Medium | Yes, reproduced |
| 8 | F-08 One damaged record makes the whole vault unopenable, with no salvage path | Medium | Agent test only |
| 9 | F-09 The sidebar tree cannot be operated from the keyboard | High (WCAG 2.1.1, Level A) | Yes, reproduced |
| 10 | F-10 Nothing stops the window from navigating away from the app | Medium | Partly (link click yes, file drop no) |

Also near the top, not in the 10: the WebView2 loader licence text is missing from the shipped V1.0 notices (F-11, fixed in docs by PR #7 but not in the released ZIP), the one `cargo audit` advisory `ctutils 0.4.3` (DP-01 in the register), no CI (F-12), and a silent exit when the data folder can't be written (F-13).

One claim made by an audit agent was **refuted** by the verifiers: that `Cipherleaf.exe` needs the Microsoft VC++ runtime. `tauri-build 2.7.4` links the VC runtime statically by default (`static_vcruntime` is in `Cargo.lock`, `staticVCRuntime` defaults to true, and neither `build.rs` nor the config turns it off), so the README's "only KeePassXC needs it" is very likely right. It was not checked on a built exe (see section 6).

## 3. Findings in detail (Medium and above)

Each finding: severity, confidence, where, evidence, impact, suggested fix, effort (S = hours, M = a day or two, L = more). Source IDs show which agent reported it (BE backend, CR crypto, SE security, DP dependencies, FE frontend, TS tests, UX accessibility and usability, PK packaging).

### F-01 Changing the master password does not rotate the data key or vault id
*(CR-01; verified)* **Medium, CONFIRMED**
- **Where:** `src-tauri/src/vault/mod.rs:273-293` (`change_credentials`), `vault/header.rs:65-77` (`rewrap` keeps `vault_id`), `commands/vault.rs:308-335`. Contrast `vault/mod.rs:301-347` (`rekey`, used only when security is switched on or off) and the unit test `rewrap_keeps_data_key_and_vault_id`.
- **Evidence:** a scratch test (`cr_old_password_and_old_header_open_data_written_after_change`) made a vault, changed the password, wrote a page, then put the OLD header into a copy of the CURRENT file. It opened with the OLD password and showed the new page. Automatic backups (up to 10) sit in `data/backups/` on the same stick, so an old header is always lying next to the live vault.
- **Impact:** changing the password because it leaked does not protect anything written afterwards from someone who has the stick and the old password. The A21 review fixed the same flaw for the security switch but not for password change. The documents say only that "older backups still open with the old password" (`security-design.md:84,124`); they do not say later records are exposed too.
- **Fix:** make `change_credentials` use the existing `rekey` (fresh data key and vault id, every row re-encrypted in one transaction; measured at about 200 ms for 3.5 MB), then make a fresh backup and say in the dialog that older backups keep the old password. If the instant re-wrap is preferred, document the limit in `security-design.md` and in the Change password dialog.
- **Effort:** S

### F-02 The backup safety net can silently fail or delete the newest copy
*(BE-01, BE-02, CR-06, PK-10; verified)* **Medium (combined), CONFIRMED**
- **Where:** `src-tauri/src/state.rs:406-424` (`rotate_backup`), `:333-336` (error only `eprintln!`), `vault/store.rs:126-133` (`backup_to`), `commands/vault.rs:202-209,380`.
- **Evidence:**
  1. Backups are named `vault-<wall-clock ms>.clf`. Rotation sorts by file **name** and removes the first entries while more than 10 exist, without protecting the file it just wrote (the passwords-file rotation, `pwbackup.rs:244-251`, does protect `just_made`). Repro: 10 files named from 1790000000000 plus a new `vault-1704067200000.clf`: the new one is deleted. A portable app that moves between PCs with different clocks hits this. A future-dated backup also stays "newest" forever.
  2. `backup_to` runs `VACUUM INTO` straight onto the final name. On failure (disk full, stick pulled) a 0-byte file stays (reproduced on a full tmpfs and in plain SQLite). It matches the backup name pattern, shows in the restore list, and counts toward the 10. `rotate_backup` returns early on error, so junk accumulates until the first success, which then deletes the oldest (good) copies. About 10 failed locks can push all good backups out.
  3. The release exe has no console, so the `eprintln!` is never seen. The owner is never told backups stopped.
  4. The manual "Save a backup copy" deletes the chosen old file before writing the new one, and `backup_now` does not refuse the live `vault.clf` as destination (Windows probably blocks the delete; not tested).
- **Impact:** the safety net the restore feature depends on can be empty without any sign. The live vault is not affected.
- **Fix:** write to `vault-<ms>.tmp`, open and authenticate it with the session key, `sync_all`, rename, then rotate by modified time (or a counter), never deleting the file just written; delete the temp on failure; report a lock-time backup failure through the existing `set_notice` / `app-notice` path; check free space first; refuse `dest == vault`. Document that FAT32 cannot hold a vault over 4 GiB.
- **Effort:** S to M

### F-03 A vault with security off can't recover after the drive is pulled, and is backed up only at quit
*(BE-03, BE-16, PK-10; partly verified)* **Medium, PLAUSIBLE for the Windows behaviour, CONFIRMED for the code**
- **Where:** `state.rs:446-448` (the monitor skips an open vault before the `drive-removed` check at `:457-459`), `state.rs:314-337` (`lock()` returns early for an open vault unless the reason is quit or restore), `vault/store.rs:44-56` (one connection opened once; no reconnect anywhere), `docs/security-design.md:92(b)`.
- **Evidence:** for an open vault nothing notices that `vault.clf` disappeared. The session keeps its SQLite handle on the removed volume; a surprise-removed Windows volume normally invalidates handles, and a re-inserted stick is a new volume. The documents say "saves fail until the drive is back". The code has no path that makes them succeed again. Separately, `rotate_backup` is called only from a dirty quit, `enable_security` and the manual backup, so an open vault gets one backup per clean quit and none after a crash, kill, log-off or shutdown.
- **Impact:** open vaults are the default for new users. Pull the stick, reinsert it, keep typing: edits live only in the web view and are lost on exit. A user who leaves the app open for weeks has no recent backup.
- **Fix:** keep the existence check for open vaults; show a persistent banner; when the file reappears, reopen the store and re-save the dirty items from memory (or offer "Save a copy elsewhere"). Take a periodic backup of open vaults too (for example after N saves or every hour when dirty). Correct the wording in `security-design.md`. The Windows behaviour needs a real test: pull a USB stick with an open vault, reinsert, edit a note.
- **Effort:** M

### F-04 A page with content this build can't read opens empty and is overwritten on the next edit
*(FE-01; verified, reproduced in the mock UI)* **Medium (low likelihood, high impact), CONFIRMED**
- **Where:** `src/features/notes/editor/extensions.ts:156-163` (`parseContent`), `src/features/notes/PageEditor.tsx:124` (`content: parseContent(page.content)`), `doSave` `:65-100`, title `onChange` `:289-293`; same flaw in `src/features/sticky/StickyWindow.tsx:13,104`.
- **Evidence:** TipTap's `createNodeFromContent` returns an empty document and logs "Invalid content" on any unknown node or mark. `enableContentCheck`, `onContentError` and `errorOnInvalidContent` are used nowhere in `src/`. In the mock: a stored page with `{type:"futureBlock"}` between two valid paragraphs opened as an empty editor; a title-only edit then saved `{"type":"doc","content":[{"type":"paragraph"}]}` over the original. A `JSON.parse` failure takes the same path. Opening and closing without editing does not save.
- **Impact:** the shipped build cannot produce unknown nodes today (history only ever added node names), so this is latent. It triggers on version skew: restoring a backup made by a newer build, or running an older exe on a newer vault. That is exactly when the original is the only copy.
- **Fix:** validate before mounting (`enableContentCheck: true` with `onContentError`, or `schema.nodeFromJSON(...).check()`); on failure keep the raw string and open the page read-only with a banner; never auto-save over it. Better: a lenient loader that turns unknown nodes into placeholders and keeps the rest, and also validates attribute types and colour/font values (see Low register FE-05, FE-10).
- **Effort:** M

### F-05 Deleting a notebook or section is permanent, with no pre-delete backup
*(UX-13, B8; verified)* **Medium, CONFIRMED**
- **Where:** `src/components/Sidebar.tsx:179-211,328-343`; `src-tauri/src/commands/notes.rs:92-171` (`delete_notebook`, `delete_section`: `remove_many` plus `collect_garbage`); stickies `StickyBoard.tsx:92-103`.
- **Evidence:** pages go to the trash with an Undo toast and can be restored. Notebooks and sections skip the trash and hard-delete all their pages (trashed ones too) after one confirm that shows no count. Nothing makes a backup before the delete. With security off the app never locks, so the last backup may be a whole session old.
- **Impact:** one confirmation stands between a user and the loss of a notebook's pages.
- **Fix:** show the count in the confirm ("3 sections, 41 pages"); move deleted notebooks and sections into the trash (pages already have `trashedAt`); make a backup right before a bulk delete.
- **Effort:** M

### F-06 Screens say "encrypted" while the default vault is not confidential
*(UX-12; verified)* **Medium, CONFIRMED**
- **Where:** `src/features/notes/NotesView.tsx:42` ("Everything is encrypted the moment you type it."), `:63`, `PageEditor.tsx:220-225` ("Encrypting…", "Saved & encrypted" on every page), `SlashMenu.tsx:37`. `securityOn` is not consulted there. Versus `SettingsView.tsx:224,298` ("Anyone who can open this folder can read them").
- **Impact:** a first-time user is told their notes are encrypted when the key is public and anyone with the folder and a copy of Cipherleaf can read them. This contradicts the project's own rule that an open vault is "not confidential". The text is technically true (the file is in the encrypted format), which makes it more misleading.
- **Fix:** make these strings depend on `securityOn` ("Saved on this drive" when off); add a one-time dismissable note on Home ("Your notes open without a password. Add one in Settings, Security."); offer "Turn on security…" on Home or in the palette; a persistent "Not protected" chip in the title bar (SE-12).
- **Effort:** S

### F-07 One large equation freezes the app on every open
*(FE-03; verified, measured in headless Chromium)* **Medium, CONFIRMED**
- **Where:** `src/features/notes/editor/math.ts:26` (`katex.render(...)` with no `maxSize`, `maxExpand` or length cap), `draw()` at `:101,:136`, live preview `:63`.
- **Evidence:** open-page time with one equation in the page (Chromium): 5k characters 1.1 s, 20k 9.5 s, 35k 30 s, 50k over 60 s. Cost is quadratic; `\rule{999999999em}{999999999em}` renders with `height:999999999em`. The backend allows pages of 20,000,000 characters (`notes.rs:11`).
- **Impact:** a page with one oversized equation (pasted, typed, restored from a hostile backup, or "Make equation" on a long paragraph) hangs the UI on every open, so the user cannot open the page to delete it.
- **Fix:** pass `maxSize: 20` and `maxExpand: 200`; cap TeX length (about 2,000 to 5,000) in `draw()` and show unrendered text above it; `maxLength` on the equation textarea; debounce the live preview.
- **Effort:** S

### F-08 One damaged record makes the whole vault unopenable, with no salvage path
*(CR-03)* **Medium, CONFIRMED (behaviour); likelihood depends on USB corruption**
- **Where:** `vault/mod.rs:102-113` (`load`: `crypto::open(...)?`, JSON parse and id check each abort the whole load), `restore.rs:208-244`, `rekey` (a corrupt attachment also blocks turning security on or off).
- **Impact:** one flipped bit in one note makes every other note unreadable until a backup is restored, and later edits are lost. Tamper evidence works; availability is all-or-nothing.
- **Fix:** add a read-only "recovery open" that skips records failing authentication, lists them and lets the person export what is readable; in `rekey` report or skip an undecryptable row. Do not skip silently on a normal unlock.
- **Effort:** M

### F-09 The sidebar tree cannot be operated from the keyboard
*(UX-01, B4; verified, reproduced)* **High (accessibility, WCAG 2.1.1 Level A), CONFIRMED**
- **Where:** `src/components/Sidebar.tsx:401-451` (`Row` is a non-focusable `div` with `onClick`), `:453-485` (dnd-kit `attributes` and `listeners` on a wrapper `div role="button"`), `:104/123` (only `PointerSensor`, no `KeyboardSensor`).
- **Evidence:** Tab lands on the dnd-kit wrapper; Enter and Space on it open nothing (0 editors, no active row), and do not toggle a section. dnd-kit still announces "press the space bar to pick up", which is false. axe: `nested-interactive`, serious, 5 nodes. There is no keyboard way to reorder pages or sections; notebooks can't be reordered at all.
- **Impact:** a keyboard-only or screen-reader user cannot open a page, section or notebook from the tree. Workarounds exist (Ctrl+K search, Home "Recently edited", favourites, breadcrumb).
- **Fix:** make `Row` a real `button` or `role="treeitem"` in a `role="tree"` with roving tabindex, `aria-expanded`, `aria-selected`; put the drag listeners on a dedicated handle; add `KeyboardSensor` with `sortableKeyboardCoordinates`; add "Move up / down" to the menus.
- **Effort:** M

### F-10 Nothing stops the window from navigating away from the app
*(SE-01, FE-02; partly verified)* **Medium, link click CONFIRMED, file drop PLAUSIBLE**
- **Where:** `src-tauri/src/ui.rs:23,91` (`.disable_drag_drop_handler()` on both window types), no `on_navigation` / `on_new_window` anywhere in `src-tauri/src`, no global `dragover`/`drop` `preventDefault` in `src/`, `extensions.ts:24,147-155` (`openOnClick:false`; `handleClick` acts only with Ctrl), `PageEditor.tsx:62` (trashed pages are non-editable).
- **Evidence:** in a non-editable (trashed) page a plain click on a link navigated the top-level frame in Chromium (reproduced). Editable pages do not navigate. Dropping a file on a non-editor area is expected to navigate to `file:///…` but could not be reproduced in headless Chromium (the drop never fired), so that half is code-supported only.
- **Impact:** the frameless window loads foreign content in place of the UI (no address bar, so it could draw a fake unlock screen) and debounced edits of up to 0.5 s are lost; the app must be restarted. The foreign page gets no app IPC from the capability file (whether non-local origins can call app commands was not verified; if they could, this would be High).
- **Fix:** `on_navigation` allowing only the app origin (and `localhost:1420` in dev) on both window builders, and deny new windows; document-level `dragover`/`drop` `preventDefault` outside the editor; `preventDefault` on every `<a>` click and open only on Ctrl+click.
- **Effort:** S

### F-11 WebView2 loader licence text is missing from the shipped V1.0 notices
*(DP-02, PK-11; PR #7 addresses it in docs)* **Medium (compliance), CONFIRMED**
- `WebView2LoaderStatic.lib` (BSD-3-style, Microsoft) is statically linked; the V1.0 `THIRD-PARTY-NOTICES.md` has no such text. PR #7 (merged after V1.0, `e09699f`) adds `packaging/Licenses/WebView2-LICENSE.txt` and notices text, but the released ZIP and exe predate it and the in-app dialog only carries it after a rebuild.
- **Also open (records already list them):** Qt and libqrencode source offer (LGPL), `vc_redist.x64.exe` and `readline.dll` (GPL-2.0, not used) shipped inside `KeePassXC\`, licence texts for Mesa, the DirectX compilers and the CA bundle.
- **Fix:** rebuild with PR #7's text; ship `packaging/Licenses/*` through a package script (F-14); consider removing unused binaries (`keepassxc-cli.exe`, `readline.dll`, `vc_redist.x64.exe`) from the portable folder after checking `KeePassXC.exe` does not import them.
- **Effort:** S

### F-12 No CI, no dependency gate, no frontend tests, no packaging script
*(TS-07, DP-03, PK-06, PK-09; verified)* **Medium (process), CONFIRMED**
- No `.github/`, no `deny.toml`, no `rust-toolchain.toml`, no test runner in `package.json`, no coverage. Every result in the records is a model-reported PC run on one exact source; the cloud container can't build the Tauri crate (no GTK/webkit), and the "Tauri-free harness" behind the documented counts is not in the repo (the audit rebuilt an equivalent in about 20 lines: 114 of 127 tests pass).
- The portable folder and ZIP are assembled by hand from recorded PowerShell steps (a backslash-path bug already bit once). `tauri.conf.json` has `bundle.active: false`. Nothing enforces "no `data\`, `.devdata`, `*.clf`, `*.key`, `*.kdbx`, `*.csv`, `.env`" in the ZIP.
- **Fix (needs the owner's approval first, per CLAUDE.md):** split the pure modules into a `cipherleaf-core` crate (or a feature gate) so Linux CI can test them; a `windows-latest` workflow running `cargo test --lib --locked`, clippy, `tsc`, `npm run build`, `npm audit`, `cargo audit`/`cargo deny`; `vitest` for `calc.ts`, `flush.ts`, `parseContent`; a `tools/package.ps1` that builds from a clean worktree, verifies the KeePassXC hashes against the manifest, assembles into an empty folder, refuses forbidden names, writes the ZIP with forward slashes, checks it against an expected-file manifest and prints SHA-256s.
- **Effort:** M

### F-13 The app exits silently when it can't create its data folder or window
*(PK-03, SE-13, BE-12; code verified)* **Medium, CONFIRMED (code path); visible Windows behaviour PLAUSIBLE**
- **Where:** `lib.rs:74` (`paths.ensure_dirs()?`), `:79` (`ui::build_main(...)?`), `:178-179` (`.expect(...)`), `main.rs:2` (GUI subsystem), `Cargo.toml:60` (`panic = "abort"`), `portable.rs:52-59`.
- **Impact:** from a read-only stick, Program Files, a locked-down folder, or without WebView2, "I double-click and nothing happens", with no message or log. Running the exe from inside a ZIP viewer parks `data\` in a temp folder (Explorer extracts only the clicked exe), and notes written there are lost when it is cleaned. The README's Troubleshooting has no "nothing happens" entry.
- **Fix:** show a native message box (`MessageBoxW`; the `windows` crate is already a dependency) naming the folder before exiting; warn when the exe path is under `%TEMP%`, `Temp1_` or Program Files, or when a probe write to `data\` fails; add the Troubleshooting entry.
- **Effort:** S to M

### F-14 Packaging risks for the people who receive the ZIP
*(PK-02, PK-04, PK-12, PK-01; PLAUSIBLE, not testable here)* **Medium (PK-04), others Low**
- **Smart App Control (PK-04, Medium):** on Windows 11 with Smart App Control enforcing, an unsigned low-reputation download is blocked outright with no "Run anyway"; the README's "More info, then Run anyway" does not apply. The exe also enumerates windows, matches processes by image path and re-owns another process's window, which looks like overlay-malware heuristics to antivirus. Signing is the real fix and is the owner's purchase decision; until then document "Properties, Unblock on the ZIP before extracting" and keep publishing hashes.
- **No upgrade procedure (PK-02, Low):** unzipping a new release elsewhere starts an empty vault with no password prompt to signal "wrong folder". Add an "Updating" section (replace only `Cipherleaf.exe`, or unzip over the old folder, since the ZIP has no `data\`) and show the data folder on the empty first screen.
- **Older exe on a V1.0 vault (PK-01, Low/Info, documented):** A21 added a header flag without a version bump, so a pre-A21 exe reports a default vault as "damaged or tampered with" (`header.rs:23,179`; `error.rs:12`). A plain version bump would break every existing vault because the version is in every record's AAD, so the fix is: unknown flag bits or unknown record kinds return the "newer version" error, plus a compatibility sentence in the README and release notes. A new record kind also makes an older exe fail the whole load (`model.rs:21-29` has no catch-all).
- **No `longPathAware` in the manifest (PK-12, Low):** deep unzip paths plus WebView2's profile path may break SQLite or WebView2; custom manifest via `WindowsAttributes::app_manifest`.

### F-15 Further Medium items
| Source | Finding | Where | Fix | Effort |
|---|---|---|---|---|
| UX-10 | Right-to-left (Arabic) text is not supported: no `dir`, no `unicode-bidi`, blocks compute `direction: ltr`; punctuation lands on the wrong side, bullets and quote bars sit on the left (screenshots in the scratch folder) | `index.html:2`, `app.css`, `PageEditor.tsx:244` | `dir="auto"` / `unicode-bidi: plaintext` per block and on the title, sidebar, palette; logical CSS properties | M |
| UX-07 | Text contrast fails WCAG AA in every theme: `--faint` is 4.43 (Black), 3.93 (Graphite), 3.07 (White), 3.94 (Paper) on the page background and lower on the sidebar; warn `#e5a023` is 2.24 on White and 2.08 on Paper but is used for text (keyfile warning, field hints); placeholder 2.1 to 2.6. `--text` and `--muted` pass everywhere. Recomputed independently by the verifier | `src/styles/app.css:21-86` | faint `#888888`/`#9c9c9c`/`#676767`/`#675d51`; warn `#8a5a00` and danger `#c62a2f` on light themes | S |
| UX-08 | Coloured accents get the wrong button text colour (`onAccent` cut-off 0.4: white on Teal 2.49, Sky 2.77, Orange 2.80, Rose 3.67, Blue 3.68) and the focus ring (accent at 65%) is 1.65 to 2.44 on light themes (needs 3:1) | `theme.ts:67-76`, `app.css:152-155` | choose text by the larger of the two ratios; draw the focus ring in a fixed contrasting colour | S |
| UX-05 | Missing names and states: two security Switches have no name ("Lock when minimized", "Block screenshots & screen sharing"), slider `aria-label` is on the root but the focusable element is the thumb, selected state of Segmented/theme/swatches is visual only, no `aria-current`, `aria-expanded`, `aria-pressed`; editor has no label | `SettingsView.tsx:210,213`, `ui.tsx:130-176`, `BubbleToolbar.tsx:352-363` | pass `label`; move `aria-label` to the thumb with `aria-valuetext`; radiogroup semantics; add the ARIA states | S |
| UX-02 | Focus is lost to `<body>` after every dialog and the palette closes (Radix has no trigger because dialogs are state-controlled) | `ui.tsx:222-340`, `CommandPalette.tsx:104-107` | save `document.activeElement` on open, restore in `onCloseAutoFocus` | S |
| UX-03/04/17 | Block handle (move, duplicate, delete, change type), sticky cards, sub-page links, hover-only buttons, slash menu (no listbox roles or `aria-activedescendant`), and the bubble toolbar (Tab from the editor goes elsewhere first; no shortcut to reach it) are mouse-only or hard to reach | `BlockHandle.tsx`, `StickyBoard.tsx:69-103`, `more-blocks.ts:70-100`, `SlashMenu.tsx`, `BubbleToolbar.tsx` | shortcuts for block actions; real buttons; `focus-within:opacity-100`; listbox roles; a shortcut that focuses the toolbar | M |
| UX-06 | A wrong master password is not announced: the error is a plain `motion.p` (no `role="alert"`, no `aria-invalid`, "Deriving key…" silent), while the lockout grows to 60 s | `UnlockScreen.tsx:130-147,277-278`, `SecurityDialogs.tsx:67-72` | `role="alert"`, `aria-live`, `aria-describedby` | S |
| UX-14 | No Windows High Contrast / forced-colors support: selected states disappear (Segmented, theme cards, swatches, active page, slider) | no `forced-colors` rule in `src/` | `@media (forced-colors: active)` rules plus ARIA state | M |
| FE-04 | Every autosave replaces the whole `tree`, so the sidebar re-renders all rows (each with a menu and sortable context), `SectionBlock` filters per section, `PageBranch` filters per row (O(n²)); every `pageLink` block subscribes to the whole store; search lowercases all page text under the vault lock; `PageMeta::from` recomputes word counts over every page on each tree build. Not measured with thousands of pages | `PageEditor.tsx:80-83`, `Sidebar.tsx:43,489,613`, `more-blocks.ts:89`, `tools.rs:55-107`, `model.rs:111-127` | pre-index children with `useMemo`, `React.memo` rows, selectors, virtualise large sections, cache word counts; measure on a seeded 5,000-page vault | M to L |
| SE-04 | The KeePassXC integrity check covers `KeePassXC.exe` only; Qt/Botan DLLs and plugins run under a correct exe hash and see the typed KeePassXC master password; the hash is read by path, so `CreateProcess` opens it later (time-of-check to time-of-use); `CIPHERLEAF_KEEPASSXC` can point at any folder holding a genuine exe. The documents already say DLLs are not checked, but P6 and threat 11 read stronger than what it delivers (rated Low by SE; kept here because the ZIP is shared with friends) | `keepassxc.rs:86-90,131-148,269-290`, `portable.rs:24-27` | build-time SHA-256 manifest of the whole `KeePassXC\` folder embedded in the exe and verified at start (verifier confirmed the exe hash and the 155-file count against upstream); reword P6/threat 11 to "the executable"; gate the env var behind a debug build | M |

## 4. Register of Low and Info findings

Verification column: V = re-checked by an independent verifier agent, otherwise single-agent.

### Security and cryptography
| ID | Sev | Finding | Where | Fix | Eff. |
|---|---|---|---|---|---|
| CR-02 | Low (V) | Deleted or truncated rows are not detected, nor is single-record rollback (documented only as A2 for rollback). Deleting the settings row resets settings to defaults. Docs line 29 does not claim deletion is detected, so they are incomplete rather than wrong. For an open vault it is moot (anyone can re-encrypt) | `vault/mod.rs:96-113`, `store.rs`, `restore.rs` | name deletion under A2 and in the limits; later an authenticated manifest (HMAC over the sorted row ids plus a generation counter, updated in the same transaction) | S docs, M manifest |
| SE-03 / CR-05 / BE-15 | Low (V) | Unlock throttle: the check runs before Argon2 and the failure is recorded after, with no gate across them. `#[tauri::command(async)]` runs on Tauri's multi-threaded runtime (tauri-macros 2.7.4), so N parallel `unlock_vault`/`restore_vault` calls all pass the check (then the wait jumps to 60 s). Same shape in `disable_security`, `change_master`. Needs script in the web view; offline guessing on a copied file is unthrottled and faster | `state.rs:229-248`, `commands/vault.rs:267-299,216-242,308-335`, `commands/backup.rs:146-175` | one `Mutex<()>` held across check, verify and record (also removes the N x 256 MiB memory burst) | S |
| CR-04 / BE-08 / CR-09 | Low | Hostile `vault.clf`: open and unlock accept Argon2 up to 4 GiB / 64 passes / 64 lanes (restore uses 1 GiB / 10 / 16); a planted open vault with maximal parameters hangs start-up (measured 12.5 s per unlock at 4 GiB, 3.0 s at 1 GiB, linear in passes). Argon2 returns an out-of-memory error rather than aborting (verifier; "OOM abort" overstated). A recursive-CTE view named `records` made `unlock_any` run past 5 s. `trusted_schema` is on | `crypto.rs:56-61`, `header.rs:176-177`, `commands/vault.rs:41-66`, `store.rs:40-52`, `restore.rs:28-30` | if the open flag is set require `kdf == OPEN_KDF`; apply the restore limits everywhere; `PRAGMA trusted_schema=OFF; cell_size_check=ON`; require `sqlite_master` to hold exactly the three tables, no views or triggers | S |
| SE-02 | Low (V) | The release exe honours `CIPHERLEAF_HOME` / `CIPHERLEAF_KEEPASSXC` (no `cfg(debug_assertions)` gate, not in the security docs) and never clears `WEBVIEW2_*` variables; the test scripts drive the release exe through `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, which proves it. Same-user malware could persist a debug port with an HKCU variable. `devtools` is off only by implication; F5/F12 are blocked by JS only | `portable.rs:21-27`, `lib.rs:41-112`, `src/main.tsx:11-25` | clear `WEBVIEW2_*` at start unless a test switch exists; explicit `.devtools(false)`; tell the owner the trade-off for `tools/real-exe-tests` | S |
| SE-05 | Low | Lock order leaves KeePassXC unlocked longest: `--lock` comes last, after the vault backup and passwords copy, and `lock_if_running` re-hashes the whole exe first; `keepassxc_place` checks `is_unlocked()` then shows the overlay (race over the lock screen); the UI reset depends on one `vault-locked` event; Win+L/sleep are detected by polling | `state.rs:314-350`, `keepassxc.rs:269-290`, `embed.rs:75-83` | send `--lock` first and cache the "verified" result; re-check after `SetWindowPos`; Rust-side forced reload; `WTSRegisterSessionNotification` | M |
| SE-07 | Low | The keyfile path from the UI is not restricted to a picked file (every other path-taking command is); a UNC path sends the user's NTLMv2 response; error text is a weak existence/size oracle. Needs script in the UI | `commands/vault.rs:117-121` | store the picked path in state and accept only that; reject `\\` | S |
| BE-05 | Low (V) | `create_keyfile` uses `fs::write` (truncates an existing file, no `create_new`, no `sync_all`) and both UI callers swallow the error (`.catch(() => null)`); accepting the OS "Replace?" prompt on the default name `cipherleaf.key` can destroy a keyfile a vault still needs | `crypto.rs:175-179`, `SettingsView.tsx:427`, `SecurityDialogs.tsx:107` | `create_new`, `sync_all`, read-back hash, show the error | S |
| BE-07 | Low | `auto_open` creates `data\…` and a new empty `vault.clf` whenever the file is missing and no backup can be listed, including after a drive was pulled (an unwritable path is reported as "new notes file could not be created"; a reused drive letter would get Cipherleaf files) | `commands/vault.rs:41-53`, `openvault.rs:15-24` | refuse to create when `!paths.data.is_dir()`, say the drive or folder is missing | S |
| SE-06 / BE-06 / PK-05 | Low (V) | `panic = "abort"` makes `catch_unwind` in `quit` (`state.rs:372-399`), `ClosingGuard` (`lib.rs:26-38`) and mutex-poison recovery (`state.rs:99-102`) dead code in release; the comments claim otherwise, and tests run with unwinding so cannot show it. A panic mid-lock or mid-quit kills the process with no passwords copy, no `--lock`, and the overlay KeePassXC unlocked until its own 5-minute idle lock. No panic reachable from UI input was found. Non-test `expect`s: `crypto.rs:33,104`, `restore.rs:338`, `lib.rs:179` | `Cargo.toml:60` | `panic = "unwind"` in release, or a `set_hook` that locks KeePassXC and writes a secret-free `crash.log`; fix the comments | S |
| SE-08 | Info | `open_url` passes the raw string to the shell opener after a `starts_with` check; userinfo, spaces, control characters not rejected | `commands/tools.rs:18-27` | parse with the `url` crate, reject userinfo/whitespace, pass `url.as_str()` | S |
| SE-09 | Info | `clatt` handler serves the stored MIME, no CSP or sandbox header; a hostile restored vault could carry `text/html` | `lib.rs:59-72`, `vault/mod.rs:223-232` | allow-list the MIME; `Content-Security-Policy: default-src 'none'; sandbox` | S |
| SE-11 | Info | Host-PC traces: file dialogs add Recent Items for keyfile/backup names; the WebView2 profile can hold a spell-check dictionary and crash reports; Windows Error Reporting dumps from an abort | `commands/vault.rs:337-382` | list them in the limits; set the dialog flag if possible | S |
| SE-12 | Info | No visible "not protected" marker while security is off (a protected vault replaced by an open one opens silently) | `TitleBar.tsx:14,68` | persistent "Not protected" chip (see F-06) | S |
| SE-14 | Info | KeePassXC window lookup compares path strings; `\\?\UNC\…` is not normalised, so network-share and subst paths may fail to match | `embed.rs:177-233` | compare `GetFinalPathNameByHandle` or match by process id and file id | S |
| BE-09 / BE-10 | Low | Cross-process window calls on the lock path (`embed::hide()` while the session is alive) and on the UI thread (`follow()` on every move/resize) wait on KeePassXC's UI thread; the stored HWND can be reused by another window | `state.rs:326-343`, `embed.rs:36-117` | drop the session before cross-process calls; timeouts; store pid with the handle | M / S |
| CR-07 / BE-11 | Low | Zeroization gaps: Argon2 working memory (256 MiB matrix not wiped), the NFC `String`, HKDF/SHA-2 state (`sha2` built without `zeroize`), stack copies of keys, and plaintext concatenations in plain `String`s (`collect_garbage` text of all pages, search lowercasing). Undercuts P10 on the Rust side only under memory forensics, which the docs put out of scope | `crypto.rs:73,89-120`, `vault/mod.rs:247-257`, `tools.rs:67-69` | `hash_password_into_with_memory` with a wiped buffer, `Zeroizing<String>`, `sha2` `zeroize` feature | M |
| CR-08 | Low | No known-answer or golden-file test: a bump of argon2, unicode-normalization or hkdf could lock out existing users with all round-trip tests green. Reserved flag bits, truncated headers and row deletion are untested | tests in `crypto.rs`, `header.rs`, `vault/mod.rs` | commit a golden v1.0 vault fixture and a KDF known-answer test (including a non-ASCII NFC password) | S |
| CR-10 | Info | `Session::put` accepts an upper-case hex row id that `load` later rejects, bricking the vault; unreachable from the current UI | `vault/mod.rs:46-49,107,141-149` | reject non-canonical ids in `seal_item` | S |
| CR-11 | Info | Padding/size classes distinguish item kinds; the settings row is always id 0; saves show which rows changed. Matches the documented metadata limit | `crypto.rs:127-144`, `model.rs:19` | none required | S |
| CR-12 | Info | `backup_now` does not refuse a destination equal to the live vault (Windows should block the delete; untested) | `commands/vault.rs:364-382` | canonicalise and refuse | S |
| DP-01 / SE-10 / PK-09 | Low (V) | `cargo audit`: `RUSTSEC-2026-0385`, `ctutils 0.4.3` ("non-constant-time mask generation", dated 2026-10-23, patched in 0.4.4), in the Windows tree via `digest` and `universal-hash`; the app does not use it directly. A timing side channel needs a local observer. `cargo update -p ctutils` moves it to 0.4.5 and changes only that package's lines. Five more warnings (`proc-macro-error`, `zbus_names`, `zvariant`, `zvariant_derive`, `glib` 0.18.5 unsound) are Linux-only and not in the Windows exe. `npm audit`: 0 vulnerabilities | `src-tauri/Cargo.lock:780-783` | `cargo update -p ctutils`, test, point release; run `cargo audit` before every release | S |
| DP-04 | Low | Most locked npm versions are days old (15 under 7 days, 103 of 199 under 14 days); `.npmrc` does not block install scripts (none found in the tree) | `package-lock.json`, `.npmrc` | `ignore-scripts=true`; a release-age cooldown for updates | S |
| DP-05 | Info | KeePassXC 2.8.0 hashes and the 155-file count reproduce from upstream (ZIP `48ff4f08…`, exe `72200bfe…`); 2.8.0 is two days old and a Qt 6 migration, so expect point releases; PGP signature not verified (key not fetchable). Unused `keepassxc-cli.exe`, `keepassxc-proxy.exe`, `readline.dll`, `vc_redist.x64.exe` ship | `vendor/keepassxc-manifest.md` | watch KeePassXC releases; drop unused binaries | S |
| DP-06 | Info | Licence review: no GPL/LGPL/AGPL crate in `Cargo.lock` (the AGPL `eff-wordlist` removal is confirmed); five MPL-2.0 crates (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`) unmodified; npm shipped set is MIT/BSD/ISC/OFL; every bundled npm package is in the notices. The Rust notices list about 15 crates `cargo tree` does not show for Windows (over-inclusion, safe). Cipherleaf's own licence is unchosen: `cargo deny` reports the crate as unlicensed and `Cargo.toml`/`package.json` have no `license` field. Nothing forces a choice | `THIRD-PARTY-NOTICES.md` | owner decision; do not add a licence unasked | — |
| DP-07 / DP-08 | Info | KaTeX 0.19.1 exists (bug fix, no security note; call site uses `trust:false`); the five Linux-only RustSec warnings above | `package.json` | update on a normal cadence; put the Linux warnings in a `deny.toml` ignore list with a note | S |

### Backend quality and files
| ID | Sev | Finding | Where | Fix | Eff. |
|---|---|---|---|---|---|
| BE-04 | Low | Plain `#[tauri::command]` functions (`delete_notebook`, `delete_section`, `delete_page`, `empty_trash`, `save_page`, `save_sticky`) run inline on the main thread and block on the session mutex that long operations hold: `collect_garbage` does one auto-commit `DELETE` per orphan (500 orphans took 610 ms on a VM disk; likely much slower on a USB stick) and builds one string of all text; `rekey`, Argon2 inside `with_mut`, and the backup commands also hold the lock. No lock-order deadlock was found | `commands/notes.rs:91,155,390,401`, `vault/mod.rs:259-263,327-347`, `commands/vault.rs:202,232,322,380` | one transaction for the GC deletes; make delete and backup commands `async`; do Argon2 before taking the session lock | S to M |
| BE-13 / PK-15 / TS-10 | Low | The single-instance lock is keyed by app identifier, not data folder: a second copy from another stick (or a test copy) hands over to the first instance and exits, so the user sees the first vault. For the CLAUDE.md test rule, a test copy cannot start while the owner's app runs | `lib.rs:44-52`, plugin `windows.rs:62-78` | key by data path or show the folder in the title/About; add `Assert-NoOtherCipherleaf` to `common.ps1` | S |
| BE-14 | Low | `keepassxc_status` reads and hashes the whole `KeePassXC.exe` every 2 s while the Passwords panel is open, holding the process mutex (`base_command` and `lock_if_running` hash again) | `keepassxc.rs:86-90,118-129`, `KeePassXCPanel.tsx:107-115` | cache the verification by (path, size, mtime); don't hold the mutex across it | S |
| BE-16 | Info | `snippet` applies a lowercased char offset to the original text (can misalign; no panic); a 20M-character page can exceed the 32 MiB item limit after typing; `header.rs:11-15` doc says on/off "only re-wraps"; settings strings other than theme are unvalidated; `create_sticky` skips `check_color`; the preset `keepassxc.ini` is written with plain `fs::write` (a partial file would leave KeePassXC on network-enabled defaults); the editor's undo history can reference an attachment already garbage-collected | various | small fixes | S |
| TS-04 | Low | The unit test `closing_never_kills_a_process_that_will_not_close` spawns `ping -n 30`, returns early if the spawn fails (this container: passes vacuously), and on Linux `ping -n` means numeric so it exits at once and the assertion fails. Only Windows runs the intended path; `the_quit_notice_text_is_exact` compares a constant to its own literal | `keepassxc.rs:332-351` | use a portable stand-in (re-run `current_exe()` with an env flag that sleeps); assert the spawn | S |
| TS-09 | Low | If KeePassXC is set to minimise to the tray, Quit cannot finish when Cipherleaf started it (waits 10 s, cancels, detaches a hidden window, message points at a window that doesn't exist); when it didn't start it, the hidden window isn't "visible" so Quit proceeds and KeePassXC stays in the tray | `keepassxc.rs:19-52,225-262`, `embed.rs:176-182`, `state.rs:353-396` | test in a temp copy; force `MinimizeOnClose=false` at every start or reword the notice | S to M |

### Frontend
| ID | Sev | Finding | Where | Fix | Eff. |
|---|---|---|---|---|---|
| FE-05 | Low | A non-string `latex` attribute makes the equation node view throw (`null.trim()`); `check()` passes; there is no React error boundary and no `window.onerror`/`unhandledrejection` handler, so the window can go blank each time that page opens | `math.ts:20-21,101` | coerce in `draw()`; root error boundary with Reload; sanitise attributes in the F-04 loader | S |
| FE-06 | Low | Autosave re-opens a section or notebook the user collapsed (the reveal effect depends on `tree.pages`, which changes identity on every save) | `Sidebar.tsx:67-74`, `PageEditor.tsx:80-83` | depend on `activePageId` only | S |
| FE-07 / B9 | Low (V; the verifier rated it Medium) | "Restore" in a trashed page's banner calls `restore_page` but the editor, title and banner stay read-only until the page is reopened (`readOnly` derives from state loaded once; `editable` is set only at creation). Restoring from the Trash list works | `PageEditor.tsx:23-33,62,125,280` | derive `readOnly` from `tree.pages.find(id)?.trashedAt` and call `editor.setEditable` in an effect | S |
| FE-08 | Low | Sticky windows lose up to 400 ms of typing when closed by Alt+F4 (only the X button flushes; no `onCloseRequested` flush) | `StickyWindow.tsx:86-87,210-218` | flush in `onCloseRequested` | S |
| FE-09 | Low | The `$…$` input rule turns "I paid $5 and $" into an equation | `math.ts:253` | require a non-space after the opening and before the closing `$`, no digit after the closing `$` | S |
| FE-10 | Low | Stored colour, font-family and font-size attributes are concatenated into inline `style` attributes without validation (UI redress from a hostile restored backup; CSS network use is blocked by `img-src`/`font-src`) | `@tiptap/extension-text-style` `:138,176,214`, `tauri.conf.json:20` | allow-pattern validation in the F-04 loader | S |
| FE-11 | Low | Every 500 ms save serialises the whole document (multi-MB for large pages); `file.arrayBuffer()` reads a whole file before the 25 MB check; the `clatt` handler decrypts and copies a whole image per request (`no-store`); a failed save of an oversized page toasts on every attempt | `PageEditor.tsx:75-100`, `extensions.ts:88`, `lib.rs:56-71` | pre-check `file.size`, throttle error toasts, cache decoded images, move decryption off the main thread | M |
| FE-12 | Info | `mailto:`/`tel:` links become `https://mailto:…`; the calculator's chained-name scan is quadratic (200,000 repeats of `pi` took 79.6 s; self-inflicted); `handlePaste` swallows the text/HTML when the clipboard also holds an image (Excel/Word); `img-src` still allows `data:` and `blob:` that nothing uses; dead IPC wrappers (`quitApp`, `keepassxcLock`, `keepassxcDetach`); a few promises without `.catch`; the image-resize drag leaks window listeners if the node view is destroyed mid-drag | various | small fixes | S |
| FE-13 | Info | No eslint although `eslint-disable react-hooks/exhaustive-deps` comments exist (the four disabled spots were checked and are correct); no frontend tests | `package.json` | add eslint with `react-hooks`; vitest | M |

### Tests and records
| ID | Sev | Finding | Where | Fix | Eff. |
|---|---|---|---|---|---|
| TS-01 | Medium | The auto-lock trigger logic (wall-clock jump over 20 s, `!vault.exists()`, idle, `screen_locked && desktop_seen_unlocked`, minimise) is one inline closure on a spawned thread with direct Win32 calls: no seam, no test; the real-exe scripts raise `idleLockMinutes` to 180 so the idle path is never entered. Not pinned by any test: a backwards clock step locks as "sleep"; "drive removed" fires on any disappearance of `vault.clf` (an antivirus quarantine) | `state.rs:421-480`, `system.rs:5-48` | extract `decide_lock(inputs)` and table-test it; put the lock steps behind a trait with a recording fake so the A17 order is assertable on any OS; real-exe rows for idle at 1 minute and drive removal by renaming `vault.clf` in a temp copy | S to M |
| TS-05 | Low | `restoreflow.mjs` ends with `process.exit(0)` and has 12 `check()` calls (the README lists 12) while the records say "13 of 13"; `lockflow.mjs` prints true/false only. `optionalsecurity.mjs` is sound (52 checks, exits 1 on failure) | `tools/real-exe-tests/restoreflow.mjs:95`, `lockflow.mjs:85-108`; `acceptance.md` X18/X19, `STATUS.md:66`, `security-design.md` P20 | add `check()`, a summary and an exit code; copy ledger counts from the summary line | S |
| TS-06 | Low | The rate limit (`state.rs:233-251`), the master-password policy (`check_new_password`, `commands/vault.rs:123-134`), `Settings::sanitized`, `protects_capture` and the image-sniff GIF/truncated cases have no test; `AppState::new` needs no Tauri value, so these are writable today. The wait maths matches the doc (2, 4, 8, 16, 32, 60 s) | listed | add the tests; move `check_new_password` out of the Tauri-dependent file | S |
| TS-02 | Low | Deleting a row or all attachments is accepted by unlock and by restore verification; no unit test covers deletion or a row moved between tables (same as CR-02) | `vault/mod.rs:101-111`, `restore.rs` | see CR-02 | S |
| TS-08 | Low | Records contradict each other: `STATUS.md:5` still says "PR #3 open, not merged, build from before A21" while later lines say merged and built; the Git and Release rows are stale; `acceptance.md` V1a-V1e are marked "current" with evidence from `442036e`; X8 says "66 cases" (the file has about 77); `failure-intelligence.md` FI-04 says the lockflow script is not in the repo (it is in `tools/real-exe-tests/`); the line "`src/lib/flush.ts`; 4 unit tests" really refers to the Rust `Flush` struct (the TypeScript `flush.ts` has no test) | `docs/` | rewrite the "As of" line and the two rows; relabel V1a-V1e | S |
| PK-07 | Low | Builds are not reproducible: two builds of the same source gave different hashes (EX-10), nothing pins the toolchain (no `rust-toolchain.toml`/`rust-version`, no `--locked`/`npm ci` mandate), and standard Rust release binaries keep panic-location paths, probably `C:\Users\moham\.cargo\registry\…` inside the shipped exe (not checked: no exe) | `Cargo.toml:56-61` | `rust-toolchain.toml`, `trim-paths = "all"`, `/Brepro`, `cargo build --locked`, build twice and compare; check `strings Cipherleaf.exe` first | S to M |
| PK-08 | Low | `.gitignore` lacks `*.kdbx`, `*.kdbx.*`, `*.kdb`, `*.keyx`, `*.tmp`, `*.zip`, `*.exe`, an unanchored `data/`, `set-aside-*`, `restore-undo-*`, although the rules say never commit them. Git history of all 59 commits is clean (no binary, vault, key, CSV, .env, certificate, token pattern; the owner's e-mail address is in the commit metadata and `C:\Users\moham\…` in four tracked docs, which matters only if the repo ever becomes public) | `.gitignore` | add the patterns | S |
| PK-13 | Low | README says "nothing is sent anywhere"; WebView2 is a separate Microsoft component that updates itself and may send diagnostics, and KeePassXC is user-configurable | `README.md` | reword to "Cipherleaf itself makes no network connections" | S |
| PK-14 / PK-16 | Info | Version numbers: `Cargo.toml`/`package.json` say 0.1.0, `tauri.conf.json` 1.0.0 (the exe and About show 1.0.0; no `CompanyName`/`LegalCopyright`). Release `v1.0` is published, tag on `0ed3f52`, one asset whose GitHub digest equals the recorded hash, but `immutable: false` (the asset can be replaced under the same URL). Branches `feat/a9-keepassxc`, `docs/fix-stale-records`, `claude/project-thread-p5yntz` are fully merged and stale (delete only with the owner's OK) | — | one number everywhere (`"version": "../package.json"`); enable immutable releases | S |

### Usability and accessibility, remaining
| ID | Sev | Finding | Fix | Eff. |
|---|---|---|---|---|
| UX-09 | Low | Input and switch borders are 1.23 to 1.33:1 against the page (needs 3:1 for non-text boundaries); the off state of a switch is hard to see | stronger boundary for inputs/switches | S |
| UX-11 | Low | Search is a lowercase substring match: "احمد" does not find "أحمد"; diacritics and tatweel are not folded | normalise query and text (NFKD, drop combining marks and tatweel, fold alef/ya variants) | S |
| UX-15 | Low | `prefers-reduced-motion` is honoured only for the lock-screen backdrop; the `motion` library has no `MotionConfig reducedMotion="user"`; the wrong-password shake still moves about 10 px | `<MotionConfig reducedMotion="user">` plus a CSS block | S |
| UX-16 | Low | The text-size slider tops out at 20 px (133%); no page zoom is enabled (not verified in WebView2); the title bar overflows below about 480 CSS px; the page title is a single-line input that cuts long titles | slider maximum about 30 px; collapse the sidebar under about 520 px; wrap the title | M |
| UX-18 | Low | The page title intercepts Enter without an IME composition check (the password box has one) | ignore when `isComposing` or `keyCode === 229` | S |
| UX-19 | Low | Structure: the first 6 Tab stops are title-bar controls with no skip link; two unlabelled `nav`s; no `main` landmark on lock/first-run cards; the page title is an input, not a heading; the sticky board uses `h2` with no `h1` | labels, `<main>`, heading, skip link | S |
| UX-20 | Low | Save-state text isn't a live region; errors inside dialogs appear only as a short bottom-right toast, away from the field | `role="status"`; inline errors; longer error toasts | S |
| UX-21 | Low | Pictures always get `alt=""` | alt from the caption or an alt-text field | S |
| UX-22 | Low | Search results show no location (`subtitle` is always empty); no find-in-page and no export (notes are readable only inside Cipherleaf); `user-select: none` on the body stops copying error text and the data-folder path; no shortcut list; restore list doesn't show whether a backup is protected or open; "Move to" shows at most 8 sections | fill `subtitle`; Markdown/text export; `.selectable`; cheat sheet; lock icon on protected backups; remove the `slice` | S to M |
| UX-23 | Info | The unlayered `*:focus-visible { border-radius: 6px }` overrides `rounded-full` on focused switches and swatches; raw OS text in errors ("Storage error: …"); "Delete earlier backups made while security was off" is ticked by default in Turn on security; the Change master password dialog has no strength meter; 20 px row buttons; the double eye icon in Turn off security (`input::-ms-reveal { display: none }`) | small fixes | S |

## 5. Where the records and the code disagree

These are statements in the project's own documents that the audit found overstated, incomplete or stale. None changes the code's behaviour.
- `security-design.md` P2 / threat 2 list what is detected but not that **deletion** of a record is not (CR-02, TS-02).
- `security-design.md:92(b)` says saves "fail until the drive is back" for an open vault; the code has no path that makes them succeed again, and there is no reconnect (F-03).
- `security-design.md:9,141` call an open vault "tamper-checked"; with a public key anyone with write access can forge a complete valid vault. Only the "protected replaced by open" limit says so.
- P6 / threat 11 ("refuses to start an executable whose hash doesn't match") cover `KeePassXC.exe` only (SE-04).
- P10 says locking removes decrypted data from memory "on the Rust side"; plain `String` copies of all page text (BE-11) and the Argon2 matrix (CR-07) are not wiped.
- P16: "`src/lib/flush.ts`; 4 unit tests" refers to the Rust `Flush` struct; the TypeScript file has none (TS-08).
- "13 of 13" restore-flow checks vs 12 in the committed script (TS-05); X8 "66 cases" vs about 77.
- `CLAUDE.md` / `STATUS.md`: "`cargo audit` not yet run" is now run (one advisory, DP-01); `STATUS.md:5` and two table rows still describe the pre-merge state (TS-08).
- README: "nothing is sent anywhere" (PK-13); "the VC++ runtime is needed only for Passwords" is **probably right** (this audit's first reading that it was wrong was refuted).

## 6. What could not be verified

- Everything that needs Windows: the `cfg(windows)` code, WebView2 and wry behaviour, `SetWindowDisplayAffinity` results (P7 "on = black" has still never been observed), window ownership and `SwitchDesktop`, handle behaviour after a surprise USB removal (F-03), UNC/subst/8.3 paths, file-dialog Recent Items, Crashpad and Windows Error Reporting contents, DPI mixing, the exact screen an older exe shows on a V1.0 vault.
- The built exe and the released ZIP: the shipped `README.txt` and `Licenses\` contents, whether the exe embeds `C:\Users\moham\…` strings, whether the VC runtime is truly static (`dumpbin /dependents` should show no `vcruntime140.dll`), Smart App Control and antivirus behaviour, behaviour from a read-only folder, a ZIP viewer, a deep path, FAT32/exFAT, or without WebView2.
- Idle lock, Win+L, sleep and hibernate, USB removal, log-off and shutdown, KeePassXC in the tray, high DPI, a real screen reader (NVDA, Narrator), real Windows High Contrast, real IME input, keyboard use of the embedded KeePassXC overlay, page zoom and Ctrl+F in WebView2.
- Whether a navigated-away page could call app IPC commands (F-10); how Tauri 2.12.5 schedules the synchronous body of `#[tauri::command(async)]` (the verifier read tauri-macros 2.7.4 and concluded multi-threaded), and so the exact burst size of the throttle bypass.
- File-drop navigation (not reproducible in headless Chromium); performance with thousands of pages (reasoned from code, not measured); USB timings; the exploitability of RUSTSEC-2026-0385.
- The 13 tests in `state.rs`, `commands/backup.rs` and `commands/notes.rs` and `cargo clippy` on the Tauri crate (the container has no GTK/webkit). The test counts reported by the PC sessions (127/127, 52/52, 13/13, testB 41/41) are model output from the owner's PC; only static counts could be cross-checked.
- PGP signatures of the KeePassXC downloads (the signing key was not fetchable); Authenticode signing of the upstream binaries; versions of the image decoders inside the Qt plugins.
- The two verifier agents re-checked 22 claims (13 backend, crypto and packaging, 9 frontend; those are the findings marked verified or V). The remaining findings were reported by one agent each; each was written with a file and line, but they have not had a second reader. All of it is model output and none of it replaces a review by a person.

## 7. Proposed roadmap

Security and data safety first, then quality, then features. Nothing here has been started; every code change needs the owner's go-ahead (CLAUDE.md).

**Phase 0: a small point release (V1.0.1), mostly small fixes (about 2 to 4 days)**
1. DP-01 `cargo update -p ctutils`, rerun tests and `cargo audit`.
2. F-01 password and keyfile change uses `rekey`; add a golden-file test first (CR-08) so the format is pinned.
3. F-04 content check and read-only fallback (notes and stickies), plus attribute validation (FE-05, FE-10).
4. F-02 backups: temp file, verify, rotate by modified time, notify on failure; same for the manual backup.
5. F-07 KaTeX limits; F-10 navigation guard, drop `preventDefault`, link clicks.
6. F-06 honest wording and the "Not protected" chip.
7. Unlock attempt gate (SE-03), `create_keyfile` `create_new` (BE-05), clamp Argon2 parameters on the open path and schema pragmas (CR-04).
8. F-13 a message box when the data folder or window can't be created.
9. F-11 rebuild with the WebView2 text from PR #7; README fixes (Smart App Control, updating, nothing-happens entry, VC++ wording, "nothing is sent anywhere"); version numbers aligned.

**Phase 1: data safety and robustness (about 1 to 2 weeks)**
- F-03 open-vault drive removal (banner, reconnect, re-save from memory) and periodic backups for open vaults; test on a real USB stick.
- F-05 notebooks and sections into the trash, counts in the confirm, a backup before bulk delete.
- F-08 recovery open and an export of readable notes.
- CR-02 / A2: document deletion now; add the authenticated manifest with a generation counter (this is also the foundation for history and sync below).
- `panic` policy (hook or unwind), KeePassXC lock first in the lock sequence, `decide_lock` extraction and tests, whole-folder KeePassXC manifest, env-var gating, single-instance key per data folder, `.gitignore`.
- F-12 (with the owner's approval): CI, `deny.toml`, `cargo-audit` in the verification table, `cargo-core` split, vitest, `tools/package.ps1`, reproducible-build flags.
- Real-exe checks never run: idle lock at 1 minute and drive removal by renaming `vault.clf` in a temp copy need no VM; Win+L, sleep and tray need a person at the PC or a VM; add a `GetWindowDisplayAffinity` check for P7. The tests agent's section 3 has the procedures.

**Phase 2: accessibility and usability (about 1 to 2 weeks, can overlap)**
- F-09 sidebar tree with keyboard support, focus return from dialogs, accessible names and states, error announcements, block-handle shortcuts, toolbar access.
- Contrast tokens, accent text colour and focus ring, forced-colors, reduced motion.
- Right-to-left support (F-15 UX-10) and Arabic-aware search, since the owner's likely users write Arabic.
- Sidebar and search performance (FE-04) after measuring on a seeded 5,000-page vault.

**Phase 3: features (each needs its own plan; ordered by dependency)**
1. **Export and import** (Markdown/text/ZIP of a notebook; import of Markdown and plain text): also answers "my notes can't be read outside Cipherleaf" and gives a safety exit. Do this first.
2. **Note history** (per-page versions): needs the generation counter and manifest from Phase 1 so deletion and rollback are detectable; store versions inside the encrypted vault with a size cap.
3. **Tags** and **templates**: small, mostly UI plus model fields; make the content loader (F-04) the place where new node types are introduced, and bump the format version properly (PK-01).
4. **Optional folder sync**: last, and only after history and the manifest. A single SQLite file in a synced folder invites conflicts and torn copies; plan per-record sync or a merge step, rollback detection, and a clear statement that whole-file rollback can't be detected offline.

**Decisions for the owner** (none taken here): choose Cipherleaf's own licence (no licence is assumed or added); whether to buy a code-signing certificate (Smart App Control, SmartScreen); whether CI may be added; whether stale branches may be deleted; whether to ship `panic = "unwind"`; whether to keep or remove the unused binaries in `KeePassXC\` (legal and size trade-off).

## 8. Notes on this report
- Source reports from the eight scoped agents were returned as chat text (their file writes were refused) and were not saved separately; this file is the consolidated record. Evidence per finding is condensed to the file and line, the key observation and the result of any test.
- Severity labels follow the scale at the top. Where an agent and a verifier disagreed the verifier's rating is used, and the disagreement is noted (FE-07 / B9).
- `docs/STATUS.md`, `docs/sessions.md`, `docs/acceptance.md` and the execution index were **not** updated by this change; the owner decides which findings to turn into amendments.
