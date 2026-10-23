# Security design: Cipherleaf

This record states what Cipherleaf protects, where its trust boundaries are, what it promises, and its known limitations. It describes the code as of amendments A9 to A18 (the latest, A18, is from 2026-10-23: the readable first-run passphrase and restore from backup, commit `1273093`, which added promise P17 and a clipboard copy of the suggested passphrase; A17, commit `dc95fd3`, changed the lock sequence and added promise P16): passwords are kept in the bundled KeePassXC 2.8.0, shown inside the Cipherleaf window, and Cipherleaf's own vault holds notes, sticky notes and settings. Each promise is a claim to verify, not a guarantee. Update this file when a promise, boundary or limitation changes.

A dependency scan, a plaintext search or a passing test can support one specific claim. None of them shows on its own that the app is secure.

## What it protects

- **Cipherleaf vault contents:** notes and pages, sticky notes, pasted images and settings.
- **Cipherleaf keys and credentials:** the master password, the optional keyfile, and the 256-bit data key that encrypts everything.
- **Passwords (A9):** usernames, passwords, TOTP secrets and attachments live in `data/passwords.kdbx`, protected by KeePassXC's own master password (and optional keyfile). Cipherleaf never reads or stores them.

## Trust boundaries

| Zone | Trust | What crosses the boundary |
|---|---|---|
| Disk / USB stick (`data/vault.clf`, `data/backups/`, `data/passwords.kdbx`) | **Untrusted.** Anyone may copy or modify these files. | Only ciphertext (an authenticated header and records for Cipherleaf's vault; KeePassXC's own encrypted format for passwords) |
| Rust core (in-process) | Trusted. The only holder of Cipherleaf's keys. | — |
| UI (WebView2, same process) | Trusted to display data; never receives keys. | Decrypted note and sticky-note content it shows; the Cipherleaf master password while typing it |
| **Bundled KeePassXC (separate process, A9)** | **Trusted** as a mature, separately maintained program. Cipherleaf starts it only if its executable matches the verified 2.8.0 hash. | Cipherleaf passes only file paths and the options `--config`, `--localconfig` and `--lock`. For the in-window display (A10) it also changes KeePassXC's window style, owner and position (window handles and rectangles only). It never sees KeePassXC's master password or entries. |
| Windows (clipboard, screen capture, other processes) | Partly trusted | Window pixels of Cipherleaf (capture-protected). Passwords copied from KeePassXC are handled by KeePassXC. |
| Network | Untrusted; **unused by Cipherleaf** | Nothing. Cipherleaf has no network code. KeePassXC is preset to make no connections; the owner can turn features on in KeePassXC. |

Every app command is callable by any Cipherleaf window. So a script injected into the UI could read the vault while it's unlocked. Since A18 this includes `restore_vault`, which replaces the vault file and needs no re-authentication of the current vault (it needs the password of the backup being restored). The picked-path rule (see P17) limits what such a script could point it at. The mitigations are a strict Content Security Policy (no remote scripts, styles, images or connections), no remote content, image sniffing, and rejecting SVG.

## Threats it's designed to resist

1. **Stolen or copied vault or backup (offline guessing):** Argon2id with high memory cost, an optional keyfile, and a master-password rule (at least 10 characters and a zxcvbn score of 3 or more).
2. **Tampering with the vault file:** every record and the header are authenticated. Changed bytes, swapped records, records moved between tables or vaults, and edited key-derivation settings are all detected.
3. **Metadata leaks from the file:** titles, item types and timestamps are encrypted, row IDs are random, and payloads are padded to size buckets.
4. **Someone seeing the screen:** Cipherleaf's windows are excluded from screenshots and screen sharing. KeePassXC blocks screen capture by default too, and masks passwords until revealed.
5. **Clipboard leaks:** handled by KeePassXC, which clears copied passwords after a delay (preset to 20 s). **This is no longer fully true (A18):** the Copy button on the first-run screen copies the suggested master passphrase to the clipboard, and Cipherleaf does not clear it automatically. Windows clipboard history (Win+V) or a cloud clipboard may keep it. The screen warns about this.
6. **An unattended unlocked PC:** Cipherleaf auto-locks on idle (default 5 min), Win+L, sleep or hibernate, removal of the drive, optionally on minimize, and on closing the app. When Cipherleaf locks, it first asks every window to save its pending edits (waiting at most 2 seconds, then locking anyway), then hides KeePassXC, closes sticky notes and sends the window back to the lock screen, and only then makes the backup and wipes the session from memory. It also asks a running KeePassXC to lock (`--lock`), even if Cipherleaf did not start it (A17). KeePassXC has its own idle and screen-lock timers (preset: 5 min and on).
7. **Password guessing in the app:** each attempt costs a full Argon2id run, and after 3 failures waits grow from 2 s up to 60 s.
8. **Malicious pasted content:** only PNG, JPEG, GIF and WebP images (checked by their bytes), at most 25 MB; no remote loads; links open only as http(s), with Ctrl+click.
9. **Network privacy:** Cipherleaf makes no connections. KeePassXC's update checks, icon downloads, browser integration, SSH agent and Secret Service are preset off.
10. **Traces on the host PC:** everything, including the WebView2 profile and KeePassXC's settings (`data/keepassxc/`), lives in the app's `data` folder.
11. **A swapped or tampered KeePassXC:** Cipherleaf refuses to start an executable whose SHA-256 doesn't match the verified 2.8.0 release.

## Out of scope (not defended)

- Malware, keyloggers, screen scrapers, or an administrator-level compromise on the PC, while unlocked or while the password is typed.
- A tampered `Cipherleaf.exe`, for example one replaced on the USB stick. The app doesn't verify itself, and the exe is unsigned. A tampered KeePassXC is caught only if its `KeePassXC.exe` changes; its DLLs aren't hash-checked.
- The security of KeePassXC itself, including its file format, its handling of memory and its update cadence. KeePassXC isn't part of Cipherleaf's code or review.
- Reading memory while unlocked. Hibernation can also write unlocked data to `hiberfil.sys` before the app locks on resume, and memory isn't protected from the page file.
- Restoring an entire older copy of the vault, which can't be detected offline. Since A18 the owner can do this on purpose with "Restore from a backup"; the app verifies the backup's own password and records, but cannot know whether it is the newest copy.
- A forgotten master password or a lost required keyfile. Recovery is impossible by design.

## Security promises

| # | Promise | Status |
|---|---|---|
| P1 | Vault data can't be read without the master password, and the keyfile when one is set | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P2 | Any change to the header or a record, or a swapped record, is detected | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P3 | Changing the password keeps all data and doesn't re-encrypt it | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P4 | Keys never leave the Rust core | By design; not independently reviewed |
| P5 | Cipherleaf makes no network connections, and the bundled KeePassXC is started with offline presets | Cipherleaf has no network code (the HTTP library was removed). KeePassXC alone, with the presets, made no connections in an 8-second launch test on 2026-10-22; not yet tested through Cipherleaf. The preset also marks the first-run update question as answered (A11), so KeePassXC does not ask to turn update checks on. |
| P6 | KeePassXC starts only if its executable matches the verified 2.8.0 hash, and its settings stay in `data/keepassxc/` | Implemented; unit-tested (refusal of an unverified file). Not yet tested through the running app. |
| P7 | Cipherleaf windows appear black in screenshots and screen shares | Implemented. The owner turned the setting off and then captured the app (18:06 UTC), so "off" works. The "on = black" state was not observed or tested in this record. |
| P8 | The vault locks on idle, Win+L, sleep, drive removal and app close | Implemented. A17 changed the order of the steps (see P16). In the real exe on a throwaway vault (2026-10-23) the Lock button and closing the window were exercised; idle, Win+L, sleep and drive removal are not tested. |
| P9 | Nothing is written outside the app's `data` folder | Implemented; not yet tested |
| P10 | Locking removes decrypted data from memory | Best effort. The Rust side wipes on drop; the UI is reloaded, but JavaScript memory can't be wiped with certainty. Since A17 the window is told to reset before the (slow) backup runs, and the Rust session is wiped right after the backup, so the lock screen appears sooner; the session stays in memory until the backup is done. |
| P11 | KeePassXC is shown inside the Cipherleaf window as an owned, borderless overlay; it hides when Cipherleaf locks, when the Passwords view is left, and closes when Cipherleaf quits (A10) | Implemented. Geometry, follow-on-move/resize, minimize/restore, hide-on-lock, re-show-on-unlock and detach checked by script on a throwaway vault on 2026-10-22; typing and mouse input not hand-tested; "closes on quit" not re-tested after the last edit. |
| P12 | Pictures and page covers are stored encrypted in the vault and shown only through the app's own `clatt` address; unused pictures are deleted, and a cover counts as in use (A11) | Implemented. A pasted picture and a cover picture were stored, shown and kept after lock/unlock in a throwaway vault on 2026-10-22; the cleanup rule is unit-tested. |
| P13 | Page comments, covers, icons and the nesting of pages (A12) are stored only inside the encrypted page records | Implemented; older records still load (tested). Nothing new is stored outside the vault. |
| P14 | Equations are drawn on this device by the pinned KaTeX 0.19.0 with "trust" off, no network and bundled fonts (A12) | Implemented; rendering checked in the mock only. `npm audit`: 0 vulnerabilities on 2026-10-22. |
| P15 | The calculators (under the Sticky notes list, A13, and inside each sticky note, A14) parse typed text itself (no code evaluation), uses no network, keeps its history only in memory (cleared when the vault locks or the app closes), and its Copy button writes only the displayed result | Implemented; the engine is checked by `dev-mock/calc-check.cjs`; the clipboard copy and the lock-clears-history behavior were not checked in the real app |
| P16 | Before the vault locks, every window writes its pending edits; the lock proceeds after at most 2 seconds even if a window doesn't answer (A17). If a failed save leaves a page unsaved, it is kept and tried again. | Implemented (`vault-locking` event, `flush_done` command, `src/lib/flush.ts`; 4 unit tests of the waiting rounds). Verified in the real exe on a throwaway vault on 2026-10-23: the Lock button with a page and with a sticky note, and closing the window (the previous exe lost the text in the same test). **Not verified:** idle lock, Win+L, sleep, drive removal (no waiting is done then, by design) and the quit command path. See the limits below. |
| P17 | Restoring a backup verifies the backup with its own password before anything is touched, never deletes the current vault (it is kept as `restore-undo-*.clf`), and accepts only the file chosen in the file dialog or an automatic backup by name (A18) | Implemented (`src-tauri/src/restore.rs`, `src-tauri/src/commands/backup.rs`; 29 more unit tests than before A18, covering install and rollback, source rejection, a hostile header, a corrupted attachment and the picked-path rule). Real exe on throwaway vaults (exe `7BB5303F…`, 2026-10-23): the bare-name path, a wrong password (vault unchanged), refused paths (not picked, UNC, traversal), restore while locked and while unlocked. **Not tested end to end:** the file-dialog path (choose a file, then restore). Not tested: a crash between the two renames. See the limits below. |

P1–P3 were unit-tested at 11:37 UTC on 2026-10-22; the whole suite passed again on 2026-10-22 at 21:49 UTC (33 tests, source after commit `9a40a3c` plus the uncommitted A10, A11 and A12 changes), on 2026-10-23 on commit `dc95fd3` (39 tests) and on commit `1273093` (68 tests). Through a running app, P11 and P12 were checked by script on a throwaway vault, and the owner observed P7's "off" state; the rest have not been exercised in the running app. See [acceptance.md](acceptance.md).

## Known limitations

- **A crash or power loss in the middle of a restore (A18).** A restore renames the live vault to `restore-undo-<number>.clf` and then renames the verified copy (`restore-check.tmp`) to `vault.clf`. If the PC loses power between the two renames, there is no `vault.clf`: the old vault is in the `restore-undo` file and the verified copy is in `restore-check.tmp`. The app would then show "Create your vault". There is no automatic recovery or notice yet.
- **`restore-undo-*.clf` files open with the OLD password and keyfile (A18).** They are never deleted or rotated by the app, so they pile up; the owner removes them by hand. Restoring a backup also brings back the password that backup had, which matters if the password was changed because it leaked.
- **The suggested passphrase is not cleared from the clipboard (A18).** After Copy on the first-run screen, Windows clipboard history or a cloud clipboard may keep it. There is no automatic clearing.
- **The picked backup path stays valid (A18)** until the next pick or a successful restore, so a wrong password can be retried without choosing the file again.
- **A late save from the old window (A18).** A delayed save from the window of the old vault could in theory reach the restored vault in the 1.2 seconds before the screen reloads (small; the restore dialog blocks input).
- **`lock()` is not a barrier against another lock still writing its backup (A18).** A restore started at that moment fails cleanly with the plain "file is in use" message.
- **Any window can call `restore_vault` (A18),** with no re-authentication of the current vault. The Content Security Policy and the picked-path rule are the defenses.
- **Hostile `.clf` files are parsed by SQLite (A18).** Only after the copy, with the restore-only Argon2 limits (memory up to 1 GiB, passes up to 10, lanes up to 16, checked before any Argon2 work) and a 2 GiB size limit.
- **A save that hangs is dropped (A17).** If a window can't finish its save within 2 seconds (for example a USB drive that stopped answering), the lock goes ahead and that last edit is lost. There is no automatic timed retry.
- **Lock after sleep or a screen lock may take up to 2 seconds** while the windows are asked to save (a window that is reloading doesn't answer).
- **The backups do not include the KeePassXC passwords file.** `data/backups/` holds copies of `vault.clf` only, not `data/passwords.kdbx`.
- **Quitting force-ends KeePassXC after 3 seconds**, which could interrupt a save KeePassXC is doing.
- **A pasted picture can be removed by the cleanup** if another page is deleted within the save delay of the page that holds the picture (plausible, low frequency; not fixed, see the backlog in [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md#findings-not-fixed-backlog)).
- **A2 (pending owner decision):** restoring an older copy of a single record isn't detected, although the plan promises it.
- **A1:** Argon2 settings are fixed rather than tuned per PC.
- After a password change, **older backups** in `data/backups/` still open with the old password and keyfile.
- The failed-attempt delay is in memory, so it **resets when the app restarts**.
- **Visible metadata:** the file is recognizably SQLite. Record and attachment counts, approximate sizes, the Argon2 settings, the vault ID and whether a keyfile is required are all visible.
- **Appearance settings in plaintext:** theme, accent, font, and layout choices such as the sidebar width, in the WebView2 storage inside `data/webview/`.
- **Two master passwords:** Cipherleaf and KeePassXC are unlocked separately.
- **KeePassXC needs the Microsoft Visual C++ 2015–2022 x64 runtime** installed on the PC; the portable ZIP doesn't include it.
- **Cipherleaf's lock-when-unplugged and screenshot blocking don't cover KeePassXC's window.** KeePassXC has its own protections.
- **KeePassXC may remember recent file paths** in its settings (the preset turns that off; the owner can change it).
- **Bundled copy must be kept updated:** 2.8.0 fixed several security issues, and later releases may too. 2.7.12 is the fallback if 2.8.0 shows problems.
- **Databases made with `keepassxc-cli` default to AES-KDF**, which is weaker than Argon2. Create the database in KeePassXC's own wizard and choose Argon2id.
- **The embedded KeePassXC is an overlay, not a true child window (A10).** Windows refused to make it a child, and WebView2 draws over child windows. Web content drawn by Cipherleaf (such as the command palette) can't appear on top of it, so it hides while the palette is open. If Cipherleaf crashes, KeePassXC keeps running as its own window (with its own idle lock).
- **Window-style changes to KeePassXC.** Cipherleaf removes its title bar and borders while embedded and restores them on detach. This changes only how the window looks and is positioned; KeePassXC's code and settings are untouched.
- **Licensing (A9 Q5):** KeePassXC is GPL v2 or v3. The release must include its license files and source link; the owner confirms this before distributing Cipherleaf. The owner decided on 2026-10-22 23:14 UTC that the build stays private (owner, close people and friends); each copy carries the license files, the source and the README.

## Cryptography (as implemented)

- **Password handling:** the password is Unicode-normalized (NFC), then stretched with Argon2id v1.3 (256 MiB memory, 3 passes, 4 lanes, 32-byte random salt).
- **Key-encryption key:** HKDF-SHA256 over the Argon2 output, plus the keyfile's SHA-256 when a keyfile is used.
- **Data key:** 256 bits from the OS random generator, wrapped with XChaCha20-Poly1305. The first 67 header bytes are authenticated alongside it.
- **Records:** XChaCha20-Poly1305 with a fresh random 24-byte nonce on every write. The authenticated data is `"cipherleaf"`, the format version, the vault ID, the table tag and the row ID.
- **Padding:** plaintext is padded to buckets of 256 B (up to 4 KiB), 4 KiB (up to 64 KiB) and 64 KiB above that.
- **Header limits:** Argon2 settings read from a file are bounds-checked: memory 8 KiB to 4 GiB, passes 1–64, lanes 1–64.
- **Storage:** SQLite with `secure_delete=ON`, `journal_mode=DELETE` and `synchronous=FULL`.
