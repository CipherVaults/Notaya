# Cipherleaf live checks on Windows, 2026-10-23 (PC half of the deep audit)

- **Tested:** the released **V1.0** exe, `Cipherleaf.exe`, SHA-256 `C43E84220E560BB7A7F6E9C4273794F704B448D1CA8ECC48BC1E3CB72FA8EA11`, taken from the GitHub release asset `Cipherleaf-V1.0-portable.zip` (SHA-256 `CAB1E72E79B9061DA05558A9AFB5CFBCB916B589A32FB318AE110E04D522D53D`, equal to the digest GitHub shows; 174 files). The exe hash equals the one in the shipped `README.txt`. Static checks ran on a fresh clone of `origin/main` (`e09699f`, docs-only difference from the V1.0 tag `0ed3f52`).
- **When:** 2026-10-23, from 22:03 UTC (relay received) to 23:09 UTC (paused at the owner's request). All times are UTC from the system clock.
- **Where:** a Windows 11 PC (12 logical CPUs, 16 GB RAM), user account without administrator rights. Everything ran in `%TEMP%\clf-audit-live` on throwaway vaults with random passphrases. The owner's `E:\Project\Cipherleaf-portable\data`, `.devdata` and the working tree at `E:\Project\cipherleaf` were not opened, run or changed. The exe was never run in place; only copies. KeePassXC was never force-stopped (closed with WM_CLOSE only). Debug port 9335 and later 9361 were set for the child process only; after every run nothing listened on it, and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` was not set in any shell.
- **Not code changes:** no source file was changed. Test scripts were written outside the repository (scratch folder); they can be added under `docs/audit/` on request.
- **Tools installed:** `cargo-audit 0.22.2` and `cargo-deny 0.20.2` into an isolated folder (`--root` and `CARGO_HOME` under `%TEMP%\clf-audit-live\tools`), and `axe-core 4.10.2` into `...\tools\axe` (for the accessibility run, still pending). Nothing was installed into the owner's `~/.cargo`, PATH or repository. `npm ci` and `cargo` builds ran only in the Temp clone (cargo used the normal registry cache).
- **Interference:** another thread ("Notes-first app and README") ran real-exe tests on the same PC from about 22:44 UTC. Cipherleaf allows only one running copy per user session and both of us used port 9335, so several of my runs between 22:44 and 23:09 UTC were disturbed and were **discarded and queued for re-run** (see "Resume point"). Results below are only from clean runs.

Status words: **PASS**, **FAIL** (the check found a problem), **CONFIRMED / NOT CONFIRMED** (for a claim in the code audit), **NOT RUN** (with the reason). Nothing here is an acceptance; the owner decides.

## 1. Top findings from the live checks

| # | Finding | Severity | Evidence |
|---|---|---|---|
| 1 | **The app does contact the network, through WebView2.** With no debug port and the app idle on its first screen, the WebView2 network process made outbound HTTPS connections to Microsoft addresses: `52.123.243.10:443` (TCP and UDP/QUIC) one second after start, `150.171.28.11:443` at 22:17:08 (first run); `150.171.22.17:443` at 22:21:43 and `150.171.27.11:443` at 22:22:41 (second run, process command line `--utility-sub-type=network.mojom.NetworkService`). Cipherleaf.exe itself and KeePassXC made none. The same launch with child-only browser flags (`--disable-background-networking --disable-component-update --disable-sync --no-pings --disable-domain-reliability --metrics-recording-only --disable-features=msWebOOUI,msPdfOOUI,OptimizationHints,Translate,MediaRouter`) made **zero** connections in 70 s. So "nothing is sent anywhere" (README) and "Cipherleaf makes no connections" (security-design P5) are not true for the running product; what is sent was not captured (TLS). The address owner is inferred from the ranges (Microsoft), not verified. | Medium (wording) / Low (risk) | `netwatch` samples of `netstat -ano` every 150 ms for the process tree; CSV kept in the Temp logs |
| 2 | **Audit F-04 CONFIRMED live.** A stored page containing an unknown block (`futureBlock`) between two paragraphs opens as an **empty editor**; a title-only edit then rewrote the stored content to `{"type":"doc","content":[{"type":"paragraph","attrs":{"textAlign":null}}]}`. Content that is not JSON at all behaves the same. | Medium (latent) | 22:31 UTC run, real WebView2 window |
| 3 | **Audit F-10 CONFIRMED, but the foreign page gets no IPC.** In a page in the Trash, a plain click on a link navigated the app window to the linked page (a local test server answered). From the foreign page `window.__TAURI_INTERNALS__` exists, but `vault_status` and `list_notes` were refused: "Command ... not allowed by ACL". That lowers F-10 from the "if they could, this would be High" case to Medium. A CDP-synthesised file drop did **not** navigate (not conclusive: a real OS drag was not driven). In an editable page a plain click did not navigate. | Medium | 22:31 UTC |
| 4 | **Audit F-02 CONFIRMED.** Ten backups named with a future time plus one new lock: the new backup was deleted at once by rotation (names sorted as text). Zero-byte files named like backups count toward the ten and are listed in the restore dialog (size 0). A backup that fails at lock (folder not writable) leaves no message on screen and `take_notice` returns nothing; the only trace is `backup failed: ...` on stderr, which the release exe (GUI) never shows. | Medium | 22:35 UTC |
| 5 | **Audit F-13 CONFIRMED.** The exe started from a folder where the user cannot create files exits after 1.36 s with exit code 3221226505 (0xC0000409, an abort), with **no window and no message**. Its stderr (visible only when redirected) is a Rust panic text that includes the build machine path `C:\Users\moham\.cargo\registry\...`. | Medium | 22:30 UTC |
| 6 | **Audit SE-03 CONFIRMED (parallel unlock).** Eight wrong unlock calls sent at the same time were **all** evaluated (all eight answered "Wrong master password..." after 3.9 s together); only afterwards did the app impose a 59 s wait. Sequential attempts: the 4th is refused with "Too many attempts. Try again in 2 seconds." The counter resets when the app restarts. The point of the delay is limited anyway: one unlock costs only about 135 ms on this PC (Argon2id 256 MiB, 3 passes, 4 lanes), see section 3. | Low | 22:42 UTC |
| 7 | **Audit PK-07 CONFIRMED.** The shipped exe contains 316 strings `C:\Users\moham` (284 of them cargo registry source paths) and rustc commit paths. | Low | `exestrings.py` on the exe |
| 8 | **Audit F-11 CONFIRMED.** The V1.0 `Licenses\THIRD-PARTY-NOTICES.md` says the WebView2 loader licence text "is still to be fetched"; no WebView2 licence file is in the ZIP (`Licenses\` holds the notices, README and the KeePassXC licences/source only). | Medium (compliance) | file listing of the extracted ZIP |
| 9 | **Audit DP-01 CONFIRMED.** `cargo audit` and `cargo deny` both report RUSTSEC-2026-0385 (`ctutils 0.4.3`, "non-constant-time mask generation", published today, fixed in 0.4.4) in the Windows build tree through `digest` and `universal-hash` (argon2/blake2, hkdf/hmac, sha2, poly1305/chacha20poly1305). The five other warnings (proc-macro-error, zbus_names, zvariant, zvariant_derive, glib 0.18.5) are not in the Windows tree (`cargo tree -i` finds nothing). | Low | section 2 |
| 10 | **Audit F-07 NOT CONFIRMED at the claimed size.** One equation block of 35,000 characters opened in 3.1 s (superscripts), 2.0 s (fractions), 5.0 s (plain letters), 3.0 s (subscript chains), 0.8 s (left/right groups); the nested and huge-rule shapes rendered an error in 0.3 s. Earlier test: 5,000 / 20,000 / 35,000 characters took 1.1 / 1.9 / 3.5 s. The audit measured 30 s at 35,000 in headless Chromium; this WebView2 on this PC is about 6 times faster at worst. The UI does block for seconds, so the finding (no length cap) stands at lower severity. | Low | 22:32 and 22:33 UTC |
| 11 | **Process kills are safe.** 14 hard kills at random moments during 300 KB saves: no acknowledged save lost, vault opens, `integrity_check = ok`. 8 kills during turning security on/off with 300 pages: vault opens, 300 pages intact, `integrity_check = ok`. Details in section 3. | Pass | 22:37-22:40 UTC |

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
| Lock on minimize | **NOT RUN / inconclusive**: a minimize via DevTools returned no error but the window state could not be confirmed; the check is queued with a different method. |

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
| Kill mid security switch (300 pages, 8 kills, 80-1,800 ms) | PASS | vault opened every round (securityOn alternated), 300 pages and 8 sampled texts intact, `integrity_check = ok`. Limit: the 4 KB-per-page vault switched in under about 200 ms, so most kills landed after the switch; a larger vault run is queued. |
| Kill during lock with a 60 MB vault (6 kills) | PASS, weak test | vault intact, no truncated backup. Limit: the vault was not dirty after the first lock, so no backup write was in progress; a dirty-lock variant is queued. |
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
| Tamper cases | **inconclusive** (my first attempt changed SQLite structure bytes, not Cipherleaf data); a proper test (header blob, record, deletion, swap) is queued |
| Turning security on/off, change password | partly done | off requires the current password (wrong one refused); on, off, on, password change keep 5 pages; old password refused after the change; a backup made while security was off stays openable **without a password** after "security on" when "keep earlier copies" is chosen (known A21 behaviour; the dialog default removes them). One check after four off/on/change cycles failed ("5 pages + sticky intact"); the cause (page text or sticky) is not yet known; a step-by-step re-run is queued. |

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

## 5. Not yet run (resume point)

I stopped at 23:09 UTC at the owner's request. Nothing of mine is running; the debug ports are closed. Still to run, in this order, on port 9361 and only when no other Cipherleaf is running:

1. **KeePassXC panel** (start, offline preset, tampered exe, Cipherleaf killed while KeePassXC runs, quit). A first attempt was spoiled when I stopped the wrong test copy by mistake (my own process; KeePassXC was then closed politely with WM_CLOSE).
2. **Secrets left behind**: process-memory scan before/after lock for the note, title, sticky text and the typed password; files in the portable folder and `%TEMP%`; stderr; clipboard.
3. **Many notes and very large notes** (5,000 pages; 5 MB and 19 MB pages; 20,000 paragraphs): timings, memory, vault size.
4. **Accessibility in the real window**: keyboard-only walk, focus visibility, contrast per theme, axe-core.
5. Extras: NFC/NFD password, dirty-lock kills on a 60 MB vault, kills during the rekey of a large vault, window capture protection (`GetWindowDisplayAffinity`), tamper cases inside the vault, the cycle failure above, minimize lock.
6. The repository's own real-exe scripts (`restoreflow`, `lockflow`, `optionalsecurity`, passwords A-E) against the V1.0 exe. They were started once but disturbed by the other thread's runs, so no result is reported.
7. Win+L and a real sleep/resume (only with the owner's OK; a wake timer would be needed to observe the resume).

## 6. What could not be verified

A real USB removal, Win+L, a real sleep and resume, SmartScreen/Smart App Control, a screen reader, real keyboard input into the embedded KeePassXC, antivirus behaviour, and what exactly WebView2 sends to Microsoft (TLS; only the connections were seen).
