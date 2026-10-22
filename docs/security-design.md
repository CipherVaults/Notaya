# Security design: Cipherleaf

This record states what Cipherleaf protects, where its trust boundaries are, what it promises, and its known limitations. It describes the code as written on 2026-10-22, before the app was ever built or run. Each promise is a claim to verify, not a guarantee. Update this file when a promise, boundary or limitation changes.

A dependency scan, a plaintext search or a passing test can support one specific claim. None of them shows on its own that the app is secure.

## What it protects

- **Vault contents:** notes and pages, sticky notes, logins (usernames, passwords, TOTP secrets, custom fields, password history), pasted images and settings.
- **Keys and credentials:** the master password, the optional keyfile, and the 256-bit data key that encrypts everything.

## Trust boundaries

| Zone | Trust | What crosses the boundary |
|---|---|---|
| Disk / USB stick (`data/vault.clf`, `data/backups/`) | **Untrusted.** Anyone may copy or modify these files. | Only ciphertext plus an authenticated header |
| Rust core (in-process) | Trusted. The only holder of keys. | — |
| UI (WebView2, same process) | Trusted to display data; never receives keys. | Decrypted items it shows (note content, a login when opened); the master password while typing it |
| Windows (clipboard, screen capture, other processes) | Partly trusted | Copied secrets (flagged and auto-cleared); window pixels (capture-protected) |
| Network | Untrusted; unused by default | Only the opt-in breach check: 5 hex characters of a SHA-1 hash, sent to `api.pwnedpasswords.com` |

Every app command is callable by any Cipherleaf window. So a script injected into the UI could read the vault while it's unlocked. The mitigations are a strict Content Security Policy (no remote scripts, styles, images or connections), no remote content, image sniffing, and rejecting SVG.

## Threats it's designed to resist

1. **Stolen or copied vault or backup (offline guessing):** Argon2id with high memory cost, an optional keyfile, and a master-password rule (at least 10 characters and a zxcvbn score of 3 or more).
2. **Tampering with the vault file:** every record and the header are authenticated. Changed bytes, swapped records, records moved between tables or vaults, and edited key-derivation settings are all detected.
3. **Metadata leaks from the file:** titles, item types and timestamps are encrypted, row IDs are random, and payloads are padded to size buckets.
4. **Someone seeing the screen:** windows are excluded from screenshots and screen sharing, and passwords are masked until revealed.
5. **Clipboard leaks:** copied secrets are kept out of Windows clipboard history and cloud clipboard, and cleared after a delay (default 20 s).
6. **An unattended unlocked PC:** auto-lock on idle (default 5 min), Win+L, sleep or hibernate, removal of the drive, optionally on minimize, and on closing the app.
7. **Password guessing in the app:** each attempt costs a full Argon2id run, and after 3 failures waits grow from 2 s up to 60 s.
8. **Malicious pasted content:** only PNG, JPEG, GIF and WebP images (checked by their bytes), at most 25 MB; no remote loads; links open only as http(s), with Ctrl+click.
9. **Network privacy:** no connections by default.
10. **Traces on the host PC:** everything, including the WebView2 profile, lives in the app's `data` folder.

## Out of scope (not defended)

- Malware, keyloggers, screen scrapers, or an administrator-level compromise on the PC, while unlocked or while the password is typed.
- A tampered `Cipherleaf.exe`, for example one replaced on the USB stick. The app doesn't verify itself, and the exe is unsigned.
- Reading memory while unlocked. Hibernation can also write unlocked data to `hiberfil.sys` before the app locks on resume, and memory isn't protected from the page file.
- Restoring an entire older copy of the vault, which can't be detected offline.
- A forgotten master password or a lost required keyfile. Recovery is impossible by design.

## Security promises

| # | Promise | Status |
|---|---|---|
| P1 | Vault data can't be read without the master password, and the keyfile when one is set | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P2 | Any change to the header or a record, or a swapped record, is detected | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P3 | Changing the password keeps all data and doesn't re-encrypt it | Implemented; unit-tested on the 11:37 UTC state (historical) |
| P4 | Keys never leave the Rust core | By design; not independently reviewed |
| P5 | No network traffic unless the breach check is turned on; the check sends only a 5-character hash prefix | Implemented; not yet tested |
| P6 | Copied secrets stay out of clipboard history and are cleared after the set delay | Implemented; not yet tested |
| P7 | Cipherleaf windows appear black in screenshots and screen shares | Implemented; not yet tested |
| P8 | The vault locks on idle, Win+L, sleep, drive removal and app close | Implemented; not yet tested |
| P9 | Nothing is written outside the app's `data` folder | Implemented; not yet tested |
| P10 | Locking removes decrypted data from memory | Best effort. The Rust side wipes on drop; the UI is reloaded, but JavaScript memory can't be wiped with certainty. |

The unit tests ran at 11:37 UTC on 2026-10-22, on the source as it was before one later one-line edit, and haven't been re-run. Nothing has been verified through the running app.

## Known limitations

- **A2 (pending owner decision):** restoring an older copy of a single record isn't detected, although the plan promises it.
- **A1:** Argon2 settings are fixed rather than tuned per PC.
- After a password change, **older backups** in `data/backups/` still open with the old password and keyfile.
- The failed-attempt delay is in memory, so it **resets when the app restarts**.
- **Visible metadata:** the file is recognizably SQLite. Record and attachment counts, approximate sizes, the Argon2 settings, the vault ID and whether a keyfile is required are all visible.
- **Appearance settings in plaintext:** theme, accent, font and similar, in the WebView2 storage inside `data/webview/`.
- TOTP secrets are stored in the same vault as the passwords they protect.

## Cryptography (as implemented)

- **Password handling:** the password is Unicode-normalized (NFC), then stretched with Argon2id v1.3 (256 MiB memory, 3 passes, 4 lanes, 32-byte random salt).
- **Key-encryption key:** HKDF-SHA256 over the Argon2 output, plus the keyfile's SHA-256 when a keyfile is used.
- **Data key:** 256 bits from the OS random generator, wrapped with XChaCha20-Poly1305. The first 67 header bytes are authenticated alongside it.
- **Records:** XChaCha20-Poly1305 with a fresh random 24-byte nonce on every write. The authenticated data is `"cipherleaf"`, the format version, the vault ID, the table tag and the row ID.
- **Padding:** plaintext is padded to buckets of 256 B (up to 4 KiB), 4 KiB (up to 64 KiB) and 64 KiB above that.
- **Header limits:** Argon2 settings read from a file are bounds-checked: memory 8 KiB to 4 GiB, passes 1–64, lanes 1–64.
- **Storage:** SQLite with `secure_delete=ON`, `journal_mode=DELETE` and `synchronous=FULL`.
