# Cipherleaf live checks on Windows, 2026-10-23 (PC half of the deep audit)

- **Tested:** the released **V1.0** exe, `Cipherleaf.exe`, SHA-256 `C43E84220E560BB7A7F6E9C4273794F704B448D1CA8ECC48BC1E3CB72FA8EA11`, taken from the GitHub release asset `Cipherleaf-V1.0-portable.zip` (SHA-256 `CAB1E72E79B9061DA05558A9AFB5CFBCB916B589A32FB318AE110E04D522D53D`, equal to the digest GitHub shows; 174 files). The exe hash equals the one in the shipped `README.txt`. Static checks ran on a fresh clone of `origin/main` (`e09699f`, docs-only difference from the V1.0 tag `0ed3f52`).
- **When:** 2026-10-23 from 22:03 UTC (relay received); paused at 23:09 UTC at the owner's request; resumed 23:42 UTC and finished the runs at about 00:15 UTC on 2026-10-24. All times are UTC from the system clock.
- **Where:** a Windows 11 PC (12 logical CPUs, 16 GB RAM), user account without administrator rights. Everything ran in `%TEMP%\clf-audit-live` on throwaway vaults with random passphrases. The owner's `E:\Project\Cipherleaf-portable\data`, `.devdata` and the working tree at `E:\Project\cipherleaf` were not opened, run or changed. The exe was never run in place; only copies. KeePassXC was never force-stopped (closed with WM_CLOSE only). Debug port 9335 and later 9361 were set for the child process only; after every run nothing listened on it, and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` was not set in any shell.
- **Not code changes:** no source file was changed. Test scripts were written outside the repository (scratch folder); they can be added under `docs/audit/` on request.
- **Tools installed:** `cargo-audit 0.22.2` and `cargo-deny 0.20.2` into an isolated folder (`--root` and `CARGO_HOME` under `%TEMP%\clf-audit-live\tools`), and `axe-core 4.10.2` into `...\tools\axe` (used for the accessibility run). Nothing was installed into the owner's `~/.cargo`, PATH or repository. `npm ci` and `cargo` builds ran only in the Temp clone (cargo used the normal registry cache).
- **Interference:** another thread ("Notes-first app and README") ran real-exe tests on the same PC from about 22:44 UTC. Cipherleaf allows only one running copy per user session and both of us used port 9335, so several of my runs between 22:44 and 23:09 UTC were disturbed and were **discarded and re-run** (from 23:42 UTC on port 9361, each run waiting until no other Cipherleaf was running; the repository's own scripts still use 9335). Two of my own runs (t14 kill tests and t18 big-page timing, about 23:53 UTC) overlapped briefly with each other; their results are consistent, but timings from that window may be slightly slower. The repository's `run-lockflow.ps1` collided once and its result is listed separately below.

Status words: **PASS**, **FAIL** (the check found a problem), **CONFIRMED / NOT CONFIRMED** (for a claim in the code audit), **NOT RUN** (with the reason). Nothing here is an acceptance; the owner decides.

## 1. Top findings from the live checks

| # | Finding | Severity | Evidence |
|---|---|---|---|
| 1 | **The app does contact the network, through WebView2.** With no debug port and the app idle on its first screen, the WebView2 network process made outbound HTTPS connections to Microsoft addresses: `52.123.243.10:443` (TCP and UDP/QUIC) one second after start, `150.171.28.11:443` at 22:17:08 (first run); `150.171.22.17:443` at 22:21:43 and `150.171.27.11:443` at 22:22:41 (second run, process command line `--utility-sub-type=network.mojom.NetworkService`). Cipherleaf.exe itself and KeePassXC made none. The same launch with child-only browser flags (`--disable-background-networking --disable-component-update --disable-sync --no-pings --disable-domain-reliability --metrics-recording-only --disable-features=msWebOOUI,msPdfOOUI,OptimizationHints,Translate,MediaRouter`) made **zero** connections in 70 s. So "nothing is sent anywhere" (README) and "Cipherleaf makes no connections" (security-design P5) are not true for the running product; what is sent was not captured (TLS). The address owner is inferred from the ranges (Microsoft), not verified. | Medium (wording) / Low (risk) | `netwatch` samples of `netstat -ano` every 150 ms for the process tree; CSV kept in the Temp logs |
| 2 | **Audit F-04 CONFIRMED live.** A stored page containing an unknown block (`futureBlock`) between two paragraphs opens as an **empty editor**; a title-only edit then rewrote the stored content to `{"type":"doc","content":[{"type":"paragraph","attrs":{"textAlign":null}}]}`. Content that is not JSON at all behaves the same. | Medium (latent) | 22:31 UTC run, real WebView2 window |
| 3 | **Audit F-10 CONFIRMED, but the foreign page gets no IPC.** In a page in the Trash, a plain click on a link navigated the app window to the linked page (a local test server answered). From the foreign page `window.__TAURI_INTERNALS__` exists, but `vault_status` and `list_notes` were refused: "Command ... not allowed by ACL". That lowers F-10 from the "if they could, this would be High" case to Medium. A CDP-synthesised file drop did **not** navigate (not conclusive: a real OS drag was not driven). In an editable page a plain click did not navigate. | Medium | 22:31 UTC |
| 4 | **Audit F-02 CONFIRMED.** Ten backups named with a future time plus one new lock: the new backup was deleted at once by rotation (names sorted as text). Zero-byte files named like backups count toward the ten and are listed in the restore dialog (size 0). A backup that fails at lock (folder not writable) leaves no message on screen and `take_notice` returns nothing; the only trace is `backup failed: ...` on stderr, which the release exe (GUI) never shows. | Medium | 22:35 UTC |
| 5 | **Audit F-13 CONFIRMED.** The exe started from a folder where the user cannot create files exits after 1.36 s with exit code 3221226505 (0xC0000409, an abort), with **no window and no message**. Its stderr (visible only when redirected) is a Rust panic text that includes the build machine path `C:\Users\moham\.cargo\registry\...`. | Medium | 22:30 UTC |
| 6 | **Audit SE-03 CONFIRMED (parallel unlock).** Eight wrong unlock calls sent at the same time were **all** evaluated (all eight answered "Wrong master password..." after 3.9 s together); only afterwards did the app impose a 59 s wait. Sequential attempts: the 4th is refused with "Too many attempts. Try again in 2 seconds." The counter resets when the app restarts. The point of the delay is limited anyway: one unlock costs only about 135 ms on this PC (Argon2id 256 MiB, 3 passes, 4 lanes), see finding 16. | Low | 22:42 UTC |
| 7 | **Audit PK-07 CONFIRMED.** The shipped exe contains 316 strings `C:\Users\moham` (284 of them cargo registry source paths) and rustc commit paths. | Low | `exestrings.py` on the exe |
| 8 | **Audit F-11 CONFIRMED.** The V1.0 `Licenses\THIRD-PARTY-NOTICES.md` says the WebView2 loader licence text "is still to be fetched"; no WebView2 licence file is in the ZIP (`Licenses\` holds the notices, README and the KeePassXC licences/source only). | Medium (compliance) | file listing of the extracted ZIP |
| 9 | **Audit DP-01 CONFIRMED.** `cargo audit` and `cargo deny` both report RUSTSEC-2026-0385 (`ctutils 0.4.3`, "non-constant-time mask generation", published today, fixed in 0.4.4) in the Windows build tree through `digest` and `universal-hash` (argon2/blake2, hkdf/hmac, sha2, poly1305/chacha20poly1305). The five other warnings (proc-macro-error, zbus_names, zvariant, zvariant_derive, glib 0.18.5) are not in the Windows tree (`cargo tree -i` finds nothing). | Low | section 2 |
| 10 | **Audit F-07 NOT CONFIRMED at the claimed size.** One equation block of 35,000 characters opened in 3.1 s (superscripts), 2.0 s (fractions), 5.0 s (plain letters), 3.0 s (subscript chains), 0.8 s (left/right groups); the nested and huge-rule shapes rendered an error in 0.3 s. Earlier test: 5,000 / 20,000 / 35,000 characters took 1.1 / 1.9 / 3.5 s. The audit measured 30 s at 35,000 in headless Chromium; this WebView2 on this PC is about 6 times faster at worst. The UI does block for seconds, so the finding (no length cap) stands at lower severity. | Low | 22:32 and 22:33 UTC |
| 11 | **Process kills are safe.** 14 hard kills at random moments during 300 KB saves: no acknowledged save lost, vault opens, `integrity_check = ok`. 8 kills during turning security on/off with 300 pages: vault opens, 300 pages intact, `integrity_check = ok`. Details in section 3. | Pass | 22:37-22:40 UTC |
| 12 | **Typed and decrypted secrets stay in process memory after a lock** (documented as "best effort", P10). Scan of the Cipherleaf process tree for the note text, title, sticky text and the typed master password: while unlocked 65 / 63 / 31 / 13 hits; 5 s after the lock 22 / 25 / 17 / 4 hits; 60 s after the lock unchanged. After the lock, `Cipherleaf.exe` still held 9 copies of the note text, 8 of the title and 4 of the sticky text (plain-text copies, audit BE-11), and one WebView2 renderer still held the note text (10), title (11), sticky text (8) and **the typed master password (3 UTF-8 copies)**; the Rust side held none of the password. No such text was found in any file of the portable folder, in `%TEMP%` files written during the run, or on stderr. (Counts include freed but not yet overwritten memory; this shows where copies linger, not that they can be read by another program without administrator-level access.) | Low | `memscan.py` (read-only), runs at 23:44 and 00:08 UTC |
| 13 | **Deleting a record from the vault file is not detected (audit CR-02 CONFIRMED live).** Editing the header blob gives "Wrong master password or keyfile..." (not "tampered"); flipping one bit in a record or swapping two records gives "The vault file is damaged or has been tampered with."; **deleting one of 7 record rows unlocks normally and the page is simply gone (4 pages became 3)**. | Low | tamper run, 23:57 UTC |
| 14 | **The KeePassXC integrity check covers the exe only (audit SE-04 CONFIRMED).** One byte appended to `KeePassXC.exe` is refused with "The bundled KeePassXC doesn't match the verified 2.8.0 release, so Cipherleaf won't start it."; one byte appended to `Qt6Core.dll` leaves `verified = true`. | Low | 23:44 UTC |
| 15 | **Accessibility (audit F-09, UX-02, UX-07 CONFIRMED in the real window).** Tab reaches the sidebar page rows, but **Enter on a focused row does not open the page**; after Esc closes the Ctrl+K palette, focus falls to `<body>`; axe-core (WCAG 2 A/AA, 4.10.2) reports serious `nested-interactive` x3 and `color-contrast` x3-5 on Home, Settings and a page, plus `aria-input-field-name` x1 on Settings and page. The faint text colour fails AA in every theme with exactly the audit's numbers: 3.94 (Paper), 3.07 (White), 3.93 (Graphite), 4.43 (Black); the sidebar "Search" row, "Notebooks", "Ctrl N" and the save-state text also fail (2.65-4.15). Every focus stop had a visible outline. | High (keyboard, WCAG 2.1.1) / Medium | 23:50 UTC |
| 16 | **Argon2id cost is low on this PC.** A successful unlock (Argon2id 256 MiB, 3 passes, 4 lanes, plus loading 5 pages) took 129-166 ms (five runs: 139, 129, 128, 143, 166 ms). With 5,000 pages (22 MB) unlock took 230 ms. The settings are fixed (A1), so a copied vault file can be guessed at roughly 7 tries per second on this whole 12-thread machine (a GPU or a cluster would be faster); the app's 2 to 60 s waits do not apply to offline guessing. | Info | 23:52 and 23:48 UTC |
| 17 | **Big vaults and pages are fine for the backend, heavy for memory.** 5,000 pages created in 65 s (13 ms each); `list_notes` 123 ms, search over 5,000 pages 8 ms, one save 7 ms, lock with a 22 MB backup 25 ms, sidebar populated 62 ms after reload. Memory (Cipherleaf + WebView2 tree): 410 MB idle, 547 MB with 5,000 pages, **2.3 GB with a 15 million character page open and 2.6 GB with a 20,000-paragraph page open**. A 5,000,000-character single-paragraph page opened in 0.9 s (159 ms per keystroke), 15,000,000 characters in 2.5 s (427 ms per keystroke), 20,000 paragraphs in 1.7 s. A save of 19,000,000 characters is refused with "This item is too large to save." and 21,000,000 with "Page is too long (max 20000000 characters)." (the real limit is lower than the stated 20 million). | Low | 23:46-23:53 UTC |

## 2. Static and dependency checks (clone of `origin/main`, 22:10-22:14 UTC)

| Check | Result |
|---|---|
| `npm ci` | PASS: 198 packages added, 199 audited |
| `npm audit` and `npm audit --omit=dev` | PASS: 0 vulnerabilities |
| `npx tsc --noEmit` | PASS: exit 0 |
| `npm run build` | PASS: built in 2.54 s (main chunk 1,737 kB) |
| `cargo test --lib` | PASS: **127 passed, 0 failed** in 22.6 s (the records say 107; the audit counted 114 of 127 in its container) |
| `cargo clippy --all-targets` | PASS with warnings: 5 in the lib (`Session::create` vault/mod.rs:52 and `Header::create`/`rewrap_open` header.rs:50,76 never used; `collapsible_if` commands/notes.rs:232; `cloned_ref_to_slice_refs` commands/stickies.rs:130; `redundant_closure` vault/mod.rs:321) and 1 more in tests (`manual_is_multiple_of` crypto.rs:252) |
| `cargo audit` (RustSec, 1,351 advisories of 2026-10-23) | **1 vulnerability** (RUSTSEC-2026-0385 ctutils 0.4.3, fix >= 0.4.4) and 5 warnings (not in the Windows tree). The "is a crate yanked" check could not finish (registry requests timed out), so yanked crates were **not checked**. |
| `cargo deny check advisories bans sources` (0.20.2, no `deny.toml` in the repo, default policy) | advisories **FAILED** (same one), bans ok (14 duplicate-version warnings: windows-sys, syn, sha2, digest, getrandom and others), sources ok. Licence policy not evaluated (no config); `cargo deny list` shows the crate itself as unlicensed and no GPL/AGPL crate. |
| Exe properties | Unsigned (`Get-AuthenticodeSignature`: NotSigned); KeePassXC.exe is signed (Valid, DroidMonkey Apps, LLC). ASLR (dynamic base, high entropy), NX and terminal-server aware on; Control Flow Guard instrumented but with an empty function table (not enforced); **no `vcruntime140.dll` import**, so the exe does not need the VC++ runtime (the audit's refuted claim stays refuted; KeePassXC still ships `vc_redist.x64.exe`). Version info: CompanyName "cipherleaf", copyright empty. |
| SmartScreen / Smart App Control | **NOT RUN**: on this PC Smart App Control is off (policy state 0) and Defender's real-time protection is off, so no prompt can appear. The GitHub download made with `gh` carries no Zone.Identifier (a browser download would). The audit's Smart App Control concern (PK-04) stays unverified. |

## 3. Real-exe results (V1.0 copy, throwaway vaults, clean runs only)

**Locks**

| Check | Result | Evidence |
|---|---|---|
| Idle auto-lock fires and the vault really locks | PASS | idle set to 1 min with the PC already idle 451 s: locked 2.6 s later; window showed the lock screen ("Welcome back..."); `list_notes` and `get_page` refused with "The vault is locked."; after unlock the page text was intact. Limit: timing from a fresh input was not measured. |
| Sleep/resume (stand-in) | PASS (stand-in only) | the test process was frozen for 30 s (22:25:09-22:25:39) to give the monitor the same wall-clock jump as a resume: the vault was locked 3 s later; page intact after unlock. A real sleep was **not** driven. |
| Win+L | **NOT RUN**: it would lock the owner's PC (needs the owner's OK; they are asleep). |
| Lock on minimize | **NOT RUN / inconclusive**: a minimize via DevTools returned no error but the window state could not be confirmed; not repeated. |

**Folder or drive disappears (USB removal)**: a real surprise removal could **not** be driven (no removable drive, no administrator rights for a virtual disk). Stand-ins: the data root as a directory junction that is removed while the app runs (all paths below vanish; the already open file handle stays valid, unlike a real removal) and an ACL that denies creating files.

| Check | Result | Evidence |
|---|---|---|
| Open vault (V1.0 default) while the path vanishes | save returns "Storage error: unable to open database file" (an error, not silent); the vault does **not** lock; no backup exists until the app is closed (backups folder empty after the first edits; one backup after a clean quit) | 22:27 UTC |
| Same, path returns | saves work again without restart in this stand-in; the text saved after the return was the one stored after restart. **This does not disprove F-03:** a real removal also invalidates the handle. | |
| Protected vault while the path vanishes | PASS: locks itself within 647-667 ms; the text saved before is intact after the path returns; text typed less than 0.5 s before the removal was **lost** (the drive-removed lock does not wait for the windows, by design) | 22:28 UTC |
| What the app does on the vanished path | after that lock the app **re-created** `data\backups\` and the WebView2 profile folders at the vanished path and wrote a backup `vault-<time>.clf` there (audit BE-07: it can litter a folder or a reused drive letter) | listing of the recreated folder |
| Read-only folder (create-file denied) | PASS: saves and new pages report "Storage error: unable to open database file"; saves work again after the ACL is removed | 22:30 UTC |

**Data safety**

| Check | Result | Evidence |
|---|---|---|
| Kill mid-save (14 kills, random 0.3-2.5 s, 300 KB saves) | PASS | last acknowledged SEQ-341, stored SEQ-342 (the one in flight); no torn text; no hot journal left; `integrity_check = ok` |
| Kill mid security switch (300 pages, 8 kills, 80-1,800 ms) | PASS | vault opened every round (securityOn alternated), 300 pages and 8 sampled texts intact, `integrity_check = ok`. Limit: this 1.2 MB vault switched in under about 200 ms, so most kills landed after the switch (the 60 MB run below covers the slower case). |
| Kill during a lock that writes a 60 MB backup (8 kills with a dirty vault, 10-700 ms after the lock call) | PASS | after every kill the vault unlocked with 6 pages; every backup file had the full size 60,260,352 bytes (no truncated or zero-byte file); the restore dialog offered 9 complete backups. Limit: a new complete backup appeared after every round, so each lock had finished its backup before the kill (the 60 MB backup takes well under 120 ms here); a kill inside the write itself was never hit, so a half-written backup remains untested (the audit's F-02 point 2 is about that case). An earlier 6-kill run with a clean vault made no backup and is not counted. |
| Kill during turning security on/off on a 60 MB vault (8 kills, 120-1,000 ms) | PASS | vault intact every round (6 pages, `integrity_check = ok`), security state alternated normally |
| Backup and restore round trip | PASS | restore with a wrong password refused ("Wrong master password or keyfile..."); with the right password the original text came back and `restore-undo-<time>.clf` was kept |
| Hot copy: 25 copies of `vault.clf` taken while the app saved (146 saves) | PASS | all 25 pass `integrity_check` |
| Move to another drive and odd paths | PASS | the folder copied to `E:` (a different physical disk) unlocked with its keyfile, and so did a path with spaces, Arabic, Japanese and accents, and a 190-character path; notes intact |
| Two copies at once | PASS | the second exe (other folder, or the same exe again) exits at once with code 0, creates no `data` folder, shows nothing; the first keeps running. Audit BE-13 CONFIRMED: the user gets no message and sees the first vault. |
| Nothing written outside `data\` | NOT CONCLUSIVE | no registry key and no AppData/ProgramData/Recent entry with the app's name; my check also saw a `cipherleaf-backup-<hex>` folder in `%TEMP%`, which I attribute to `cargo test` (the cargo tests create such folders), not proven. |

**Credentials**

| Check | Result | Evidence |
|---|---|---|
| Password rules at "security on" | PASS | 9 characters: "Use at least 10 characters for your master password."; "aaaaaaaaaa", "password123456" and 20 spaces: "That master password is too easy to guess. Try a longer passphrase." |
| Wrong password / missing / wrong / empty / directory keyfile | PASS | "Wrong master password or keyfile for this Cipherleaf vault."; missing path: "The keyfile could not be read."; empty file or directory: "A keyfile must be between 1 byte and 64 MB." Wrong-keyfile checks that never reach Argon2 answer in 3 ms. |
| Throttle (sequential) | PASS | after 3 failures, even the right credentials get "Too many attempts. Try again in 2 seconds."; they work after the wait; the counter resets on restart |
| Throttle (parallel) | FAIL | see finding 6 |
| Tamper cases (inside the vault) | PASS for flips and swaps, FAIL for deletion | see finding 13 |
| Password with accents | PASS | a password with accented letters set in composed form (NFC) also unlocks in decomposed form (NFD); a trailing space is significant |
| Turning security on/off, change password | PASS | off requires the current password (a wrong one is refused); on, off, on and password change keep 5 pages; the old password is refused after the change. A second run with 3 pages, a sticky note and **four full off/on/change/change-back cycles** kept every page text and the sticky text after each single step (the "5 pages + sticky" failure in my first script was a script error, not reproduced). A backup made while security was off stays openable **without a password** after "security on" when "keep earlier copies" is chosen (known A21 behaviour; the dialog's default removes them). |

**KeePassXC panel** (23:43 UTC, exe `KeePassXC.exe` SHA-256 `72200BFE...` from the ZIP)

| Check | Result | Evidence |
|---|---|---|
| Start | PASS | `keepassxc_status`: installed, verified, version 2.8.0; `keepassxc_open` started one KeePassXC process with a visible window in about 6 s |
| Offline preset | PASS | `data\keepassxc\keepassxc.ini` has `CheckForUpdates=false`, `UpdateCheckMessageShown=true`, Browser, SSHAgent and FdoSecrets disabled, `RememberLastDatabases=false`, `LockDatabaseIdleSeconds=300`, `ClearClipboardTimeout=20`; KeePassXC made no network connection (finding 1) |
| Cipherleaf killed while KeePassXC runs | KeePassXC keeps running as an ownerless window | after the kill the KeePassXC process and window stayed; a restarted Cipherleaf reported `running = false` for it (it does not recognise the orphan) but did **not** start a second one (KeePassXC's own single-instance setting) |
| Quit with KeePassXC running (no database) | PASS | Cipherleaf exited; KeePassXC closed politely |
| Modified `KeePassXC.exe` / modified DLL | exe PASS, DLL FAIL (finding 14) | |
| Lock hides the window | **NOT TESTED**: the test did not embed the window in the Passwords view (that needs the UI), so the "hides on lock" promise (P11) was not exercised |
| Cancelled Quit with a KeePassXC dialog open (testC) | PASS | repository script, see below |

**The repository's own real-exe scripts against the V1.0 exe** (exe hash `C43E8422...` printed by each runner; run from the Temp clone, one at a time on a free machine)

| Script | Result |
|---|---|
| `run-restoreflow.ps1` | 12 of 12 PASS (the script has 12 checks; the records say 13) |
| `run-optionalsecurity.ps1` | 52 of 52 PASS |
| passwords `testA` (backup rotation) | 13 of 13 PASS |
| passwords `testB` (restore of the passwords file) | 41 of 41 PASS |
| passwords `testC` c1, c2, c2q (Quit with and without a KeePassXC dialog) | 3/3, 10/10, 10/10 PASS (Cipherleaf exited 0.95 s after the window close with no dialog; with a dialog open both programs stayed open and the exact notice text appeared once) |
| passwords `testE` | 5 of 5 PASS |
| passwords `testD1` (90 locks, long) | NOT RUN (time) |
| `run-lockflow.ps1` (type, then lock or close at once) | first run collided with another run ("no matching window", discarded); clean re-run at 00:11 UTC: page PASS (text saved), sticky note PASS, closing the window PASS (text saved after restart); port 9335 closed, temp folder removed |

**Accessibility in the real window** (WebView2, default Paper theme and the three others)

| Check | Result | Evidence |
|---|---|---|
| Tab order and visible focus | PASS | 40 Tab presses on Home gave 39 stops (1 fell to `<body>`), 60 on Settings gave 60; no stop without a name except one on Settings; every stop had an outline or shadow; the first 5 stops are the title-bar buttons, then sidebar, no skip link |
| Enter on a focused sidebar page row | FAIL | the page does not open (audit F-09 confirmed) |
| Ctrl+K palette, then Esc | FAIL | focus returns to `<body>` (audit UX-02 confirmed) |
| Contrast, text runs below AA (4.5:1), Paper / White / Graphite / Black | FAIL | Home 10 / 10 / 14 / 14, Settings 15 / 15 / 19 / 19, page 4 / 4 / 7 / 7; the faint text colour is the main cause (finding 15). (The 1.07-1.29 ratios my script printed for text in `oklab()` colours are an artefact of the script and are ignored.) |
| axe-core WCAG 2 A/AA | FAIL | serious: `nested-interactive` x3 (Home, Settings, page), `color-contrast` x5 / x5 / x3, `aria-input-field-name` x1 (Settings, page) |
| Screen reader, High Contrast mode, IME | NOT RUN | |

**Other**

| Check | Result | Evidence |
|---|---|---|
| Screenshot blocking (P7 "on = black") | PASS | with `screenCaptureProtection` on, `GetWindowDisplayAffinity` on the main window returns `0x11` (WDA_EXCLUDEFROMCAPTURE); default is `0x0` (off); the value stays `0x11` while locked. (A real capture was not taken.) |

## 4. Audit claims checked

| Audit item | Live result |
|---|---|
| F-02 backup rotation / silent failure | CONFIRMED (finding 4); zero-byte creation by a real failed `VACUUM INTO` (disk full, stick pulled) not reproduced |
| F-03 open vault and a removed drive | PARTLY: open vaults are not locked and make no backup until quit (confirmed); the handle behaviour after a real removal could not be tested |
| F-04 unreadable page overwritten | CONFIRMED |
| F-07 huge equation | NOT CONFIRMED at the claimed size (finding 10) |
| F-10 navigation away | link click CONFIRMED (no IPC from the foreign page); file drop not reproduced with a synthetic drop |
| F-11 WebView2 licence text missing | CONFIRMED |
| F-13 silent exit | CONFIRMED |
| SE-03 parallel unlock | CONFIRMED |
| BE-13 single instance by identifier | CONFIRMED |
| PK-07 paths in the exe | CONFIRMED |
| PK-13 "nothing is sent anywhere" | the README is wrong in practice (finding 1) |
| DP-01 ctutils | CONFIRMED by `cargo audit` and `cargo deny` |
| "VC runtime needed by Cipherleaf.exe" | REFUTED again (no `vcruntime140.dll` import) |
| CR-02 deleted rows not detected | CONFIRMED (finding 13) |
| SE-04 KeePassXC check covers the exe only | CONFIRMED (finding 14) |
| BE-11 plain-text copies of notes stay in memory after a lock | CONFIRMED (finding 12) |
| F-09 / UX-02 / UX-07 keyboard and contrast | CONFIRMED (finding 15) |
| F-01 password change keeps the data key | not tested live (unit-level in the audit) |
| BE-07 re-creating the data folder on a vanished path | CONFIRMED for a protected vault after a drive-removed lock (section 3) |
| TS-05 "13 of 13" restore checks | the script has 12 checks and all 12 pass |
| FE-07 restore button in trashed page, FE-08 sticky Alt+F4, FE-09 `$` rule, SE-07 keyfile UNC, BE-05 keyfile overwrite | not tested |

## 5. Not run, and why

- **Real USB removal / a drive vanishing under open handles:** no removable drive and no administrator rights for a virtual disk. The stand-ins (section 3) do not invalidate file handles, so audit F-03 (open vault: what happens to edits after a real removal and re-insert) is **still open** and needs a real stick.
- **Win+L:** locking the owner's workstation needs their OK; the lock logic reads the desktop state once a second and is untested live.
- **Real sleep and resume:** not driven (it would end this session; a wake timer would be needed to watch the resume). The 30 s process freeze is the only stand-in.
- **Lock on minimize:** a DevTools minimize request was accepted but the window state was not confirmed, so the result is inconclusive.
- **Embedded KeePassXC overlay** (geometry, hide on lock, typing): needs the Passwords view and a real person; not driven.
- **SmartScreen / Smart App Control / antivirus:** both are off on this PC. Not observable.
- **`testD1` (90 locks)** of the repository scripts and a hot-journal case in which the kill lands *inside* a backup write: not run.
- **Screen reader, High Contrast mode, IME, a real OS file drag onto the window, a real screenshot capture of a protected window:** not run.
- **Clipboard after "Copy" of the suggested passphrase:** not run (documented limitation, A18); the clipboard was not read, so none of the owner's clipboard content is in this file.

## 6. What could not be verified

A real USB removal, Win+L, a real sleep and resume, SmartScreen/Smart App Control, a screen reader, real keyboard input into the embedded KeePassXC, antivirus behaviour, and what exactly WebView2 sends to Microsoft (TLS; only the connections were seen).
