# Notaya: security design, session summary and acceptance, read in full (2026-11-07)

**What this is:** a faithful digest of six Notaya records, read in full with nothing skimmed, for the coordinator and for Ahmed. It is a read-only result: nothing in the repository was changed. Where this digest and the repository differ, the repository wins.

**Baseline:** `CipherVaults/Notaya`, `main` at `7661503` (merge of PR #53, "Handoff: Codex lead", 2026-11-07 19:22:15 UTC, from `git log`). Read on 2026-11-07, starting 20:17:04 UTC (the thread's start time); the container clock read 20:20:43 UTC when the digest was written.

**Files read in full**

| File | Size | Lines | Read |
|---|---|---|---|
| `docs/security-design.md` | 71,580 bytes | 153 (very long lines) | every byte, in three pieces |
| `docs/sessions.md` (the session summary) | 119,347 bytes | 1,326 | every line |
| `docs/STATUS.md` | 60,955 bytes | 158 | every byte |
| `docs/acceptance.md` | 38,153 bytes | 90 | every byte |
| `AGENTS.md` | 4,701 bytes | 53 | every line |
| `CLAUDE.md` | 11,720 bytes | 93 | every line |

Also looked at, for cross-checks only (not part of the ask, not read in full): the list of entries in `docs/execution/README.md`, the 2026-11-07 hand-off note `docs/handoff/2026-11-07-codex-lead-handoff.md` (read through once), a search of `docs/plan/approval-and-amendments.md` and `docs/handoff/claims.md`.

**Could not read:** nothing that was asked for. Two things the records point to are not in the repository at all: the report of the first independent security review of A21 (the session summary says "the review report itself is not stored in the repository") and the Claude memory note `project_restore-feature-in-progress` mentioned in the 2026-10-23 11:00 entry.

---

## 1. Security design (`docs/security-design.md`)

### 1.1 What the document is

It says what Notaya (still called Cipherleaf in the title) protects, where its trust boundaries are, what it promises, and its known limits. It states that "each promise is a claim to verify, not a guarantee", and that a dependency scan, a plaintext search or a passing test can support one claim but never proves on its own that the app is secure. Its opening paragraph describes the code "as of amendments A9 to A24"; later promises (P22 to P29) and limits were added for A30 to A36, so the opening line is older than the body.

### 1.2 What it protects, and the big change in A21

- **Notes, pages, sticky notes, pasted pictures and settings.** Since A21 (2026-10-23) these are **confidential only while security is on**. With security off they are still stored in the encrypted, tamper-checked format, but the key is a public, built-in one, so anyone with the file and a copy of the app can read them. Since A23 the screens stop saying "encrypted" while security is off.
- **Keys:** the master password, the optional keyfile, and the 256-bit data key.
- **Passwords: removed in A24 (2026-10-27).** KeePassXC was bundled from A9 to A23; since A24 Notaya does not start it and does not touch `passwords.kdbx` or its copies. Old files stay on disk with KeePassXC's own encryption and are no longer backed up.

**Why optional security (A21):** Mohamed asked for a notes app first, where security, auto-lock and screenshot protection are switches the person can turn on or off at any time.

### 1.3 Trust boundaries

| Zone | Trust | What crosses |
|---|---|---|
| Disk or USB stick (vault file, backups) | Untrusted: anyone may copy or change it | Only ciphertext |
| Rust core | Trusted: the only holder of keys | — |
| The window (WebView2, same process) | Trusted to show data, never gets keys | Decrypted text it shows; the master password while it is typed |
| Windows (clipboard, screen capture, other programs) | Partly trusted | Window pixels (hidden from capture only when security and the setting are on); the clipboard only through the calculator's Copy button |
| Network | Untrusted and unused by Notaya's own code | WebView2 itself was seen contacting Microsoft at start-up (live check 2026-10-23) |

**Recorded weak point:** every app command can be called by any Notaya window, so a script injected into the window could read an unlocked vault, and could call `restore_vault` (no re-authentication of the current vault), `enable_security` (sets a password the owner does not know and can delete earlier open backups), `disable_security` (needs the current password, rate-limited) and `start_notes`. The checkbox that lets a restore turn security off is only a UI gate. **Why it is accepted:** the defenses are a strict Content Security Policy (no outside scripts, styles, images or connections), no remote content, picture checks by bytes, SVG refused, and the "picked path" rule for restores.

### 1.4 Threats it is built to resist (11, two of them removed)

1. A stolen vault or backup guessed offline: Argon2id with high memory, optional keyfile, a password rule (at least 10 characters and a zxcvbn score of 3+).
2. Tampering with the file: every record and the header are authenticated; swapped or moved records and edited key settings are detected.
3. Metadata leaks: titles, types and times are encrypted, row ids random, sizes padded to buckets.
4. Someone seeing the screen: only with security on **and** screenshot protection switched on (off by default since A21). The lock screen is hidden from capture only at app start.
5. Clipboard leaks: the app puts no secret on the clipboard; its only write is the calculator's Copy. (Between A18 and A22 a suggested-passphrase Copy button broke this; removed in A22.)
6. An unattended unlocked PC: with security on, auto-lock on idle (5 min default), Win+L, sleep, drive removal, optionally minimize, and close. **An open vault never locks.** Lock order (A17): every window saves (at most 2 s), sticky notes close, the window returns to the lock screen, then the backup, then the keys are wiped.
7. Password guessing in the app: each try costs a full Argon2id run; after 3 failures, waits from 2 s up to 60 s.
8. Hostile pasted content: only PNG, JPEG, GIF, WebP checked by bytes, at most 25 MB; links open only as http(s) with Ctrl+click.
9. Network privacy: Notaya's own code makes no connections (but see P5).
10. Traces on the PC: everything, including the WebView2 profile, stays in the app's `data` folder.
11. A swapped KeePassXC: removed in A24.

### 1.5 Out of scope (not defended, on purpose)

Malware, keyloggers or an admin-level attacker; a replaced `Notaya.exe` (the exe is unsigned and does not check itself); reading memory while unlocked, hibernation and the page file; restoring a whole older copy of the vault (undetectable offline); a forgotten password or lost keyfile (no recovery by design); and **confidentiality while security is off**.

### 1.6 The promises (P1 to P29) and their evidence

| # | Promise, in short | State in the document |
|---|---|---|
| P1 | Data can't be read without the password (and keyfile) | Unit-tested (2026-10-22, historical) |
| P2 | Any change or swap of header or record is detected | Unit-tested (historical) |
| P3 | Changing the password keeps data; since A23 it also re-keys everything | Unit-tested; PC results in EX-21 |
| P4 | Keys never leave the Rust core | By design; **not independently reviewed** |
| P5 | Notaya's own code makes no network connections | True for its code. **Limit:** WebView2 connects to Microsoft at start-up; flags that stop it exist but applying them is a security-behaviour change left for the owner (planned "V1.1", still open as Q25/Q26) |
| P6 | KeePassXC hash check | Removed in A24 |
| P7 | Windows black in screenshots | Only with security on and the setting on. The "on = black" state was **never observed or tested** |
| P8 | Locks on idle, Win+L, sleep, drive removal, close | Only with security on. Lock button and close tested in the real exe; idle, Win+L, sleep, drive removal reported passing by Mohamed by hand (2026-10-24 12:16:47 UTC) with no record of build or steps; no automated test |
| P9 | Nothing written outside the `data` folder | Two exceptions: the installed copy's folder under `%LOCALAPPDATA%\Notaya Notes` (A30) and one registry value for "Start with Windows" (A31). "Not yet tested" |
| P10 | Locking removes decrypted data from memory | Best effort; JavaScript memory can't be wiped for certain |
| P11 | KeePassXC overlay and Quit | Removed in A24; the new Quit (lock, back up, exit) is not tested in the real exe in this record |
| P12 | Pictures and covers stored encrypted, served only via the app's own `clatt` address; unused pictures deleted | Implemented; V1.5.1 tightened what a cover or icon may be (gradient, colour, or a picture already in the vault; outside, `data:` and `file:` addresses refused); unit-tested |
| P13 | Comments, covers, icons and page nesting only inside encrypted records | Implemented |
| P14 | Equations drawn locally by KaTeX 0.19.0, "trust" off, no network | Checked in the mock only |
| P15 | Calculators parse text themselves (no code run), no network; Copy writes only the result; history (up to 500 lines) saved encrypted in the vault | Engine checked by script; windows seen in the mock |
| P16 | Before a lock, every window saves; lock goes ahead after 2 s anyway | Verified in the real exe for Lock button and close; idle, Win+L, sleep, drive removal and quit **not verified** |
| P17 | Restore checks the backup with its own password first, never deletes the current vault (kept as `restore-undo-*`), accepts only the picked file or an automatic backup by name | 29 unit tests; real-exe checks; **not tested end to end:** choose a file in the dialog, then restore; a crash between the two renames |
| P18 | Passwords-file copies | Removed in A24 |
| P19 | "Start over" only while locked, needs typing START OVER, never deletes (renames to `set-aside-*`) | Tested on the PC; one independent review found no data-loss path |
| P20 | Security is optional; a new vault is open and not confidential; turning security on or off re-keys the whole vault | Unit tests, PC tests (130/130), real window; two independent reviews (model output). USB, idle, Win+L, sleep not run then |
| P21 | Screenshot protection off by default, only with security on | Same evidence as P20; the window flags were never observed in a real window |
| P22 | Installed copy keeps notes in `%LOCALAPPDATA%\Notaya Notes\data`; uninstaller never deletes notes; import from a portable copy is copy-only, the Rust side opens the dialog | Built and unit-tested under Wine; the cell still says real exe, setup and dialog "not yet tested on Windows" (later tested per EX-33/34, but this cell was not updated) |
| P23 | Close button hides to tray; a protected vault is **locked before** hiding; start with Windows writes one per-user registry value | Rust tests; PC checks in EX-37. Limits: the tray icon is not hidden from screenshots; an open vault stays open in the tray |
| P24 | New vault format `NOTA` v2 (same algorithms, new labels); old `CLF1` read and re-encrypted at first unlock; old file kept as `vault-before-notaya.clf` | 178 tests under Wine incl. real V1.3.0 files. Not run on Windows: upgrade of a real V1.3.0 folder, an old exe beside the new one. "An independent review … is recorded in EX-37 when done" (not confirmed here) |
| P25 | Export (HTML, Markdown, PDF, Word, ZIP) runs in the Rust core; the dialog warns the file is **not encrypted**; refuses to write into `data` or with a vault extension; safe file names; HTML has its own strict content policy and no scripts; page links only built by the exporter | Many unit tests; sample files checked with outside tools. **Not tested:** real exe, real save dialog, Word, a Windows PDF viewer, Explorer ZIP, browsers |
| P26 | Import treats every file as hostile: fixed list of blocks, no scripts or SVG, path checks, zip-bomb limits, size and time limits (2 GB etc. since V1.5.1), Cancel, all-or-nothing save | Hardened after an independent hostile-input review; 81 import tests. **Not tested:** real exe, real dialogs, real Notion exports and real-world PDFs, the release build |
| P27 | Databases (V1.5.0): encrypted records; filters, formulas, charts run in Rust; files in rows served as downloads only; CSV cells starting with `=`,`+`,`-`,`@` get an apostrophe | Unit-tested; **"not exercised in the running app yet"** |
| P28 | PDF fonts: bundled Arabic and Hebrew fonts; CJK fonts read only from the Windows Fonts folder by fixed name | Unit-tested; a look at a real PDF still needed |
| P29 | Automations and buttons: a closed list of actions, no network, no programs, no files outside the vault; loops stop after 3 steps, 200 actions, 50 rows | 10 unit tests; mock UI; **not exercised in the running app yet** |

The summary paragraph under the table still describes evidence only up to A21 (tests on 2026-10-22 and 2026-10-23).

### 1.7 Known limits, with the reasons given

**A24 (KeePassXC removed):** old passwords files and their copies are left alone, never backed up or deleted by the app, and an old KeePassXC password still opens an old copy. Quit now just locks and exits.

**A21 (optional security):**
- Security off is not confidentiality (public built-in key).
- Someone with write access to `data\` can swap a protected vault for an open one; the app then opens with no prompt. Not defended (the same person could replace the exe).
- Earlier open backups and `restore-undo-*` files stay readable; turning security on offers to delete open automatic backups but never the undo files, because they may hold notes that exist nowhere else.
- Restoring an open backup onto a protected vault turns security off after a checkbox; a UI gate only.
- No auto-lock and no screenshot protection with security off.
- **Backups (A23):** written to a temp file, checked, synced, renamed. Kept: newest 10 normal, 3 hourly (open vaults, only after 30 s of no input), 5 "before delete" copies. Turning security on with the default tick deletes every unprotected backup, including the safety copies. A backup briefly blocks other commands (about 1 s per 200 MB on SSD).
- **Re-keying** on turning security on or off. **Why:** the independent review found that only re-wrapping kept the same data key, so an earlier open copy could open the protected vault.
- Two independent reviews of A21 (model output): first 2 high, 3 medium, 6 low (fixed in `f9d2676`, except the lock-screen capture after a mid-session lock); second found no high issues. Recorded as still true: an open backup can be restored onto a protected vault from the lock screen without a password; an open vault does not react to drive removal.

**A20 (Start over):** needs no password while locked (whoever can use the locked PC could rename the file by hand); set-aside files open with the OLD password and pile up.

**Earlier limits (A1 to A19, and V1.5.1 additions):**
- A crash between the two renames of a restore leaves no `vault.clf`; no automatic recovery.
- `restore-undo-*` files keep the OLD password and are never cleaned up; restoring also brings back that backup's password (matters if the password leaked).
- A late save from the old window could reach a restored vault in a 1.2 s window (small).
- A save that hangs more than 2 s at lock is dropped (no automatic retry).
- **Calculator history is saved** in the vault (could hold PINs worked out by hand).
- **Recent searches are saved** (V1.5.1): last five terms, encrypted, wiped from memory on lock; Mohamed approved keeping them (2026-11-05 06:05:04 UTC).
- **Back/forward history** is in memory only.
- **`Object.prototype` is no longer frozen** (V1.5.1, Mohamed's card 2026-11-04 23:20:04 UTC). **Why:** Mermaid's libraries break when it is frozen, so no diagram drew in the real exe (BUG-9). Lost: one defense in depth against prototype pollution. Still standing: only bundled scripts run, no network, imports cleaned in Rust, Mermaid in strict mode.
- **Mermaid diagrams** are treated as hostile code: strict mode, size and edge caps, a 15 s limit, init directives removed, the drawing cleaned again before display, no new network host. Tested in the mock only, not in the real WebView2 window.
- `restore_vault` callable by any window; hostile `.clf` files parsed by SQLite only after copying, with Argon2 limits and a 2 GiB cap.
- A pasted picture can be removed by the cleanup in a rare timing case (not fixed, backlog EX-13).
- **A2 (open owner decision):** restoring an older copy of a single record is not detected, although the plan promised it.
- **A1:** Argon2 settings are fixed, not tuned per PC.
- After a password change, older backups still open with the old password.
- The wrong-password wait resets when the app restarts.
- Visible metadata: the file is plainly SQLite; counts, rough sizes, Argon2 settings, vault id and "keyfile required" are visible.
- Appearance settings (theme, fonts, sidebar width) are stored in plain text in the WebView2 storage.

### 1.8 Cryptography as built

Password normalised (NFC), Argon2id v1.3 (256 MiB, 3 passes, 4 lanes, 32-byte salt); key-encryption key by HKDF-SHA256 (plus the keyfile's SHA-256); 256-bit random data key wrapped with XChaCha20-Poly1305; every record XChaCha20-Poly1305 with a fresh 24-byte nonce, bound to format, vault id, table and row id; padding buckets 256 B / 4 KiB / 64 KiB; header Argon2 values bounds-checked; SQLite with `secure_delete=ON`, `journal_mode=DELETE`, `synchronous=FULL`. Open vault: a header flag and a public password with cheap Argon2 (8 MiB, 1 pass). The crypto section still says the record binding is `"cipherleaf"`; P24 says new vaults use `notaya` (the crypto section was not updated for A32).

### 1.9 Things in the security design that look stale or inconsistent

- The title and most text still say Cipherleaf.
- The opening "as of A9 to A24" line does not mention A30 to A36.
- P9 and P22 say "not yet tested", while STATUS and EX-33/34 record PC tests of the installer.
- P24 promises an independent review "when done"; whether it happened is not stated here.
- The cryptography section's binding string predates P24.
- The paragraph under the promise table stops at A21.

---

## 2. The session summary (`docs/sessions.md`)

### 2.1 What it is

One file, newest first, meant to hold "what happened, why, how understanding changed and where to resume". It does not hold the live state (STATUS), permission (amendments record), work details (execution summaries) or acceptance. Old entries are never rewritten; corrections are added as dated notes.

### 2.2 What it covers (31 entries)

| Date (UTC) | Entry | Form |
|---|---|---|
| 2026-11-04 17:02 to 11-05 07:43 | V1.5.1: fixes, PC rounds, BUG-9 (Mermaid vs frozen prototype), Hubs import fixes, text direction, a near-miss that would have closed the owner's real app (FI-07), release; addendum: Mohamed's "I approve this version" 11:08:39 | Short (what happened, understanding, resume) though it calls itself a "big stop" |
| 2026-10-28 14:35 to 16:10 | Prepare V1.3.0 with the Windows setup; "Run Notaya" on the Finish page touched the real profile (FI-05) | Short |
| 2026-10-24 to 10-28 | Native Windows app plan, then dropped | Short |
| 2026-10-27 19:40 to 22:28 | V1.2.0 release | Short |
| 2026-10-27 16:29 to 19:08 | PBS review, then adoption in smallest form (A29) | Short, though called "fuller" |
| 2026-10-27 17:07 to 18:56 | Live audit (PR #10) compared with V1.1.0: 15 rows still open | Short |
| 2026-10-27 16:51 to 16:57 | "Record every step" rule; PC copy on `main` (with three later addenda) | Short |
| 2026-10-27 13:31 to 17:15 | Icon, theme, name; rename to Notaya | Short |
| 2026-10-27 16:36 | Records catch-up (partly reconstructed) | Short |
| 2026-10-27 07:21 to 08:20 | Combined test of four PRs (reconstructed later) | Short |
| 2026-10-27 08:21 to 08:40 | V1.1.0 portable build | Short |
| 2026-10-27 06:45 (four entries) | Ribbon; KeePassXC removed (A24); pop-out calculators (with a correction: really A25); sticky multi-delete (A26, reconstructed) | Short |
| 2026-10-23 after 22:25 (to 10-24 12:22) | V1.0.1 audit fixes (A23), WebView2 traffic wording, release v1.0.1, Mohamed's manual lock tests | Short but long |
| 2026-10-23 after 20:45 | V1.0 portable build and release | Short |
| 2026-10-23 20:12 | Credits and suggested passphrase removed (A22) | Short plus resume |
| 2026-10-23 18:32 to 18:47 | Notes first, optional security (A21) | Medium (models table, what happened, resume) |
| 2026-10-23 17:00 to 21:45 | PC catch-up and test-folder cleanup | Short |
| 2026-10-23 17:04 to 17:35 | Start over (A20) | Very short |
| 2026-10-23 14:17 to 16:04 | Passwords-file backup, safe Quit (A19) | Medium |
| 2026-10-23 13:55 to 14:20 | Restore from backup (A18) | Medium |
| 2026-10-23 11:00 to 13:55 | Work paused; GitHub repository created (with a correction) | Medium |
| 2026-10-23 from 08:28 | Code review, lock and data-loss fixes (A17) | **Full** (12+ sections) |
| 2026-10-22 22:58 | Commit of A10 to A14, first release build | Full |
| 2026-10-22 22:40 | A14 calculator in sticky notes | Full |
| 2026-10-22 22:26 | A13 calculator and wording | Full |
| 2026-10-22 21:32 | PBS reconciliation | Full |
| 2026-10-22 15:49 to 18:10 | First run, debug-port exposure (FI-01), A10 to A12 (reconstructed) | Full |
| 2026-10-22 10:53 to 13:30 | Plan, first build, PBS adoption, KeePassXC (A9) (reconstructed) | Full |

### 2.3 Periods that are missing (no session entry at all)

Checked against the execution index and `git log`:

1. **2026-10-22 23:14 to 23:55:** fresh portable build (EX-10), app icon A15 (EX-11), wider pages and Paper default A16 (EX-12). No entry; the A17 entry only mentions them in passing.
2. **2026-10-23 after 21:37:** notices gaps (EX-20). No entry (small, so arguably allowed).
3. **2026-10-28 11:19 to 14:33:** the Windows installer plan re-check and the installer build and PC tests (A30, EX-32, EX-33). No entry.
4. **2026-10-28 16:24 to 17:42:** test leftovers cleaned on the PC (about 12.6 GB, EX-36), GitHub repository renamed to Notaya (EX-35), PC folder moved and V1.4.0 started (A31, EX-37). No entry, and the V1.3.0 publish itself (16:10 to 16:11) appears only in STATUS.
5. **2026-10-29:** the two-partner guide (A34, EX-38), which made Ahmed a full partner. That is a change of direction, which the rules call "big", and it has no entry.
6. **2026-10-29 to 2026-11-03:** all of V1.4.0 (export, import, tray, new vault format, PC tests EX-40), its merge and release on 2026-11-03 15:48 to 15:50, and **the first ever acceptance** ("i accept", 18:23:19 UTC). A release and an acceptance, no entry.
7. **2026-11-03 19:20 to 2026-11-04 14:30:** all of V1.5.0 (right-to-left text, PDF letters, databases, automations; A35, EX-42) and its release. No entry.
8. **2026-11-05 after 11:08 to 2026-11-07:** the hand-off to Ahmed (Codex leads, Claude reviews), `AGENTS.md`, the PBS skill refresh to v1.1, the toolchain on Ahmed's PC, the `cargo clippy` run (137 warnings), PRs #52 and #53. A change of roles, no entry; it lives only in the hand-off note.

### 2.4 Other problems with the file

- **Short form at big stops.** `CLAUDE.md` asks for the 12-section summary plus a session entry at a big stop (a release, a security change, a change of direction). From 2026-10-23 17:04 on, every entry is short form, including the V1.0, V1.0.1, V1.1.0, V1.2.0 and V1.5.1 releases and the PBS adoption. Only the entries from 2026-10-22 and the 2026-10-23 A17 entry have the full form.
- **Order is not strictly newest first.** The 2026-10-27 entries are out of order (19:40, then 16:29, 17:07, 16:51, 13:31, 16:36, then 07:21 above 08:21, then the 06:45 group). Ahmed's rule (2026-11-07 20:12) is one file, newest on top, oldest at the bottom.
- **The intro is stale.** Line 5 still says "the three newest entries (A13, A14, the commit and build)".
- **Missing separators** (`---`) between several 2026-10-27 entries.
- **Reconstructed entries** are marked as such (the two first 2026-10-22 blocks, the combined test, the sticky multi-delete, parts of the records catch-up and the middle of V1.5.1).
- **Model and effort** are recorded only in the older entries; most later entries say "not recorded" or leave it out. There is no usage or token record anywhere in these files, which Ahmed now says is mandatory.

---

## 3. STATUS (`docs/STATUS.md`)

- **Latest "as of": 2026-11-05 after 11:08 UTC.** V1.5.1 is published and accepted; accepted baseline **v1.5.1 on `95ca8b2`**. `main` was `2812d65` then. Nothing about 2026-11-06 or 2026-11-07 (Ahmed's hand-off, `AGENTS.md`, Codex leading) is in STATUS.
- **"Owner and decision-maker: Mohamed Sayed".** Ahmed is not named there; his partner status is only in question 27, `claims.md`, `CLAUDE.md` and the hand-off note.
- **Exact next permitted action:** waiting for a partner's next instruction; nothing is being built. A new release, the licence for Notaya, and making the repository public need a partner's own words (the last two need both).
- **Not run for V1.5.1 (unchanged by the acceptance):** the setup installer of this build, the tray entry "Close sticky notes", the Windows keyboard-layout shortcut between Arabic and English lines, `cargo clippy` for MSVC (since run on Ahmed's PC on 2026-11-07: 0 errors, 137 warnings, per the hand-off note, not in STATUS), the Playwright mock checks of this release, Word and PDF files of the Hubs import.
- **Open questions for the owner still open:** Q9 (A2 single-record rollback, A8 version pinning), Q10 (backlog from EX-13), Q11 (licence; may the repository ever be public), Q12 (review A21 on a real PC), Q13 (reset saved screenshot settings?), Q14 (offer to delete undo files?), Q20 (acceptance of V1.0 to V1.1.0 and A10 to A27), Q23 (GitHub About text and saved skills still mention KeePassXC), Q24 (untracked `vendor\` on the PC), Q25 (15 open live-audit items incl. WebView2 traffic, F-04, F-10, SE-03, CR-02, contrast and focus), Q26 (cutting WebView2 traffic, postponed), Q27 (Mohamed to confirm the reading of the two-partner rules).
- **Inside STATUS, older text is kept as history.** The "Current state" section still carries a mission-state line from 2026-10-28 14:33 UTC that lists V1.1.0 to V1.3.0 as "not accepted"; the "What exists" table still lists V1.1.0 as the current release build, 128 Rust tests and KeePassXC paths; "Exact next permitted action" is dated 2026-10-27 19:08 UTC and talks about the installer PR. The newest truth is in the first bullet and the V1.5.1 bullet. A fresh reader could easily be misled.
- **Continuation:** V1.5.0 (published, never accepted on its own), V1.4.0 (accepted 2026-11-03 18:23:19), V1.0.1 to V1.3.0 (released, never accepted).

---

## 4. Acceptance (`docs/acceptance.md`) and the gaps

### 4.1 What has actually been accepted

Only two owner decisions are acceptances, both Mohamed's:

1. **V1.4.0 release:** "i accept", 2026-11-03 18:23:19 UTC. Recorded in the prose and in STATUS, **but there is no ledger row for it** (no X-number).
2. **V1.5.1 release:** "I approve this version", 2026-11-05 11:08:39 UTC, row **X27**. The message names no version; "V1.5.1" is the coordinator's reading.

Both accepted the release as published with its not-run list. **Neither accepts any single requirement row.** Every other row (V1a to V6c, X1 to X26) is "pending". Informal signals that are not acceptance: "happy" (2026-10-22, confirmed to mean nothing), "Okay looks good" (2026-10-27, X26).

### 4.2 Gaps

- **Releases with no acceptance:** V1.0, V1.0.1, V1.1.0, V1.2.0, V1.3.0, V1.5.0.
- **Releases with no ledger row at all:** V1.0 (no X row; X20 is V1.0.1), V1.2.0, V1.3.0, V1.4.0 (accepted but no row), V1.5.0. The newest rows jump from X26 (rename, 2026-10-27) to X27 (V1.5.1).
- **Features with no row:** the installer (A30), tray and start with Windows (A31), the NOTA v2 format (A32), export and import (A33), databases and automations (A35), the V1.5.1 items other than as part of X27.
- **Plan checks never run:** V3c (pull the USB stick: locks) "not-run"; V5b (clipboard) not-run; V6c (`/security-review` on the code) **not-run**; V2 the plaintext scan on a real vault not run; V5c "on = black" screenshots inconclusive.
- **Stale row:** V6a says `cargo audit` "Not run", but STATUS and the sessions file record `cargo audit` with 0 vulnerabilities on 2026-10-24 (V1.0.1 lock file).
- **V4 network** is recorded as a fail for "no connections" (WebView2 connects); still open.
- **Open decisions that block acceptance:** A2 and A8; evidence for A1 and A3 to A7; whether to retire V1f and V5b.
- **The "Target assessed" paragraph** at the top still describes branch `feat/a9-keepassxc` and commit `2105a57` with later updates stacked on top, ending at 2026-10-23.
- Row X12 sits after X13 (cosmetic).

---

## 5. `AGENTS.md` and `CLAUDE.md`

### 5.1 `CLAUDE.md` (rules for any Claude)

- **Start of every session:** read the latest instruction, STATUS, the amendments record, inspect the repository; read the security design before touching crypto, storage, commands, the content policy, the clipboard or auto-lock; read the acceptance ledger before saying something works; read the collaboration guide and claims, fetch all branches, claim work first.
- **Rules:** stable releases only, exact versions; never edit the plan snapshot; never commit vault data or secrets; **ask first** before installing tools or dependencies, deleting files, committing, pushing, changing security behaviour, or resuming while paused; keys only in Rust; never test on a real window or vault (throwaway data, `CIPHERLEAF_HOME` in a temp folder, close the debug port, FI-01); quote times, never estimate; never force-stop KeePassXC.
- **Cloud threads:** own branch from `origin/main`, draft PR, **never merge** (the owner merges); ask before force-push, deleting branches, making it public or adding a licence; report unrun checks as not run.
- **PBS work cycle (A29):** state the task (baseline, owner's message and time, scope, return, stop point); keep permission, observation and acceptance apart; name results produced, verified, integrated or accepted; stop at the endpoint; record by size (small: PR text; normal: 4-section summary, index row, STATUS lines; big: 12-section summary plus a session entry).
- **Two partners:** "the owner" means either partner; either one's words give permission and either merges, releases and accepts; **a Claude never merges or releases by itself**; public repository and licence need both; repository settings are Mohamed's click; if the partners' instructions conflict, stop and tell both.
- **Records:** execution summaries, index, STATUS, session entries at big stops, failure intelligence, the acceptance ledger only on the owner's actual decision, the security design when a promise changes.

### 5.2 `AGENTS.md` (entry point for Codex, added 2026-11-07)

Points to `CLAUDE.md` as the single source of truth ("if the two ever differ, `CLAUDE.md` wins"). Sets the roles from Ahmed (2026-11-07): **Codex leads development** (plans, code, tests, checks); **Claude coordinates and reviews** (records, independent diff review, security review; a review by the agent that wrote the code is not independent); **Ahmed is the decision-maker for this checkout, and only his own words give permission.** Tests only on throwaway data (suggested `D:\Notaya Tests`), Mohamed's real vault and `E:` drive are off limits. Every code PR gets Claude's `/security-review` and the Security Guidance plugin. Hand over with a resume note and a claims-row update.

### 5.3 Conflicts and gaps between the two files and the project's current rules

1. **Who gives permission.** `CLAUDE.md`: either partner. `AGENTS.md`: only Ahmed, for this checkout. `AGENTS.md` itself says `CLAUDE.md` wins, so as written either partner can give permission.
2. **Merging.** Both files say a Claude never merges or releases by itself. Ahmed's instruction of 2026-11-07 19:54 UTC (in project memory only) lets the coordinator merge and release by its own judgement and needs no review by Mohamed. That is **not yet in any repository record**, and the repository rule says records win over memory. It also touches the `CLAUDE.md` rule that conflicting partner instructions mean "stop and tell both", and Q27 (Mohamed's confirmation of the partner rules) is still open.
3. **Who leads.** `CLAUDE.md` describes Claude doing the work in cloud threads; `AGENTS.md` says Codex leads. `CLAUDE.md` was not updated for the new roles.
4. **Records.** `CLAUDE.md` asks for a 12-section summary and a session entry at big stops; the session file has not followed that since 2026-10-23 (section 2.4). Ahmed's 2026-11-07 rule asks for thorough records and a usage ledger; neither file mentions a usage ledger.
5. **Names.** `CLAUDE.md` is still titled "Cipherleaf"; `AGENTS.md` says Notaya.
6. **Amendments record.** A search found no entries dated 2026-11-06 or 2026-11-07, so Ahmed's role change and authority rules have no authority row yet.

---

## 6. In one paragraph

The security design is detailed and honest: it lists 29 promises, marks three as removed, and is open about what was never tested. Many of the newer features (databases, automations, export, import, Mermaid) are tested only by unit tests or the mock, and were never tried in the real app. The biggest standing risks it records are: no protection at all while security is off (the default for new vaults); any window can call the security-changing commands; WebView2 contacting Microsoft; the frozen-prototype defense turned off for Mermaid; and the screenshot and drive-removal promises never observed in the real app. The session file is one newest-first file, but it is out of order on 2026-10-27, mostly short form since 2026-10-23, and has nothing for V1.4.0, V1.5.0, the partner guide or the 2026-11-07 hand-off. Only two releases were ever accepted (V1.4.0 and V1.5.1), no single requirement is accepted, and the newest authority rules from Ahmed are not yet in the repository.
