# A20: Notes first, security and passwords optional (design proposal)

**Status:** proposed design, written 2026-10-23 (system clock, 18:35 UTC). Nothing is built yet. Not approved by the owner.

## What the owner asked

Owner messages, 2026-10-23, 18:32:20 and 18:34:11 UTC (session timestamps):

- Cipherleaf is a notes app at its core. Security is an optional layer that people can turn on or off at any time. The password manager (KeePassXC) is a second optional feature.
- The default is a plain notes app that opens straight to the notes. The vault stays open, with no automatic locking, until the person turns security on.
- Screenshot protection is off by default, always.
- Also: a README for the app with its core features and optional features.

## Reading of the ask (the choices that were mine)

1. **Two independent optional features.** "Security" (master password, keyfile, auto-lock, screenshot protection) and "Passwords" (KeePassXC) each have their own switch. Passwords does not require security. KeePassXC keeps its own master password and its own idle lock either way.
2. **"Off all the time" for screenshot protection** is read as: off by default for every new vault, and it only applies while security is on and the person has switched it on. It is never switched on for the person.
3. **Existing vaults are never changed silently.** A vault that has a master password keeps it and stays locked on start. Only the person can turn security off, and only by typing the current password.

## Design

### Vault format: one new header flag, no new file

The header already wraps the random data key with a key derived from the password. A new authenticated flag, `FLAG_OPEN` (bit 1 of the flags byte), marks a vault whose data key is wrapped with a **built-in, public** key (a fixed string in the program, with cheap Argon2 settings, since there is nothing to guess). All records, attachments, padding, tamper checks and backups stay exactly as they are. Format version stays 1. An older Cipherleaf will refuse such a file as damaged; it cannot misread it.

Because the data key never changes, **turning security on or off only re-wraps the header**. It is instant and cannot lose notes (the same mechanism as "Change master password" today).

### What "security off" means, honestly

Notes are still stored in the encrypted, tamper-checked format, so they are not greppable plain text and damage is detected. **But the key is public, so this is not confidentiality.** Anyone who has `vault.clf` (or a backup) and a copy of Cipherleaf can read it. The Settings text and the README say this in plain words. A separate key file next to the vault was considered and rejected: on a portable app it sits in the same folder (no real protection), and losing it would make the notes unrecoverable.

### Start-up

- First run, no vault and no backups: Cipherleaf silently creates an open vault and shows the notes. No lock screen, no password prompt, no welcome wall.
- First run, no `vault.clf` **but backups exist**: no silent new vault (that would look like lost notes). A small screen offers "Restore from a backup" or "Start new notes".
- Vault file with `FLAG_OPEN`: opened automatically at start.
- Vault file without the flag: the lock screen, exactly as today.
- A damaged vault never falls back to an empty one: the existing error and restore path is shown.

### Security switch (Settings, "Security")

- **Off (default):** no idle lock, no lock on minimize, Win+L, sleep or unplugging, no Lock button or Ctrl+L, no capture protection on any window. Closing the app still saves pending edits first and makes the usual backup. The security options (auto-lock, lock on minimize, screenshot protection, master password) are shown greyed out with the explanation.
- **Turn on:** a dialog asks for a master password (same rules, strength meter, suggested passphrase and optional keyfile as today), the "no password reset" confirmation, and one choice: **delete earlier backups that were made while security was off** (default on; only files whose header has `FLAG_OPEN` are touched; a fresh protected backup is made first). Then the vault is re-wrapped with full Argon2 settings. The options appear with their previous values: auto-lock 5 minutes, lock on minimize off, **screenshot protection off**.
- **Turn off:** asks for the current password (and keyfile), shows the warnings below, and re-wraps the vault under the built-in key. Wrong password counts toward the existing delay and changes nothing.

Warnings shown when turning off: the notes will be readable by anyone who can open this folder or copy `vault.clf`; nothing will lock automatically; older backups in `data\backups` keep the password they were made with; the KeePassXC passwords file is not affected and keeps its own master password.

### Passwords switch (Settings, "Passwords (KeePassXC)")

- Off (default for new vaults): no sidebar item, no Ctrl+2, no palette entry, KeePassXC is never started, hidden or asked to lock. The backups of the passwords file are not made. Nothing in `data\passwords.kdbx` is touched.
- On: today's panel appears. Turning it off closes KeePassXC politely first (the same close routine as Quit and refuses with the existing message if KeePassXC is still open).
- The backend refuses every KeePassXC command while the switch is off, not only the UI.
- Existing vaults: if `data\passwords.kdbx` or KeePassXC settings already exist, the switch starts **on**, so nobody loses their panel. Otherwise off.

### Where the switches are stored

Settings (inside the vault) gets `passwordsEnabled` (three states while loading: unset, on, off). Whether security is on is not a setting. It is the header flag, which is authenticated and readable before unlock.

### Defaults that change

| Item | Before | After |
|---|---|---|
| New vault | Create-vault screen with a required master password | Open vault, straight to notes |
| Screenshot protection | New vaults: on. Main window: always on until unlock | New vaults: off. Window created unprotected unless the vault file has a master password. Never forced on |
| Auto-lock | Always on | Only with security on |
| Passwords panel | Always shown | Own switch |

Existing vaults keep their saved screenshot setting (it cannot be told apart from a choice the person made). The lock screen of a vault that has a master password stays hidden from screenshots, as today.

## Security effects to review

- Downgrade by file swap: someone who can write to `data\` could replace a protected vault with an open one. The app would then show empty notes and no password prompt. That needs the same write access that already lets them replace the exe, and the missing prompt is visible. Recorded as a known limit.
- Open-mode backups and `restore-undo-*.clf` files stay readable by anyone. Turning security on offers to delete them.
- No new IPC command can lower security without the current password: turning security off needs it, and is rate-limited like unlock.
- Keys still stay in the Rust core; the UI never sees the built-in key.
- Restore from a backup: a backup with `FLAG_OPEN` is restored without a password (its key is built in); one with a password asks as today.

## Not changing

Notes, sticky notes, calculator, pictures, backups, restore, KeePassXC embedding, crypto, record format, `docs/plan/approved-plan-2026-10-22.md`.

## Checks planned

Unit tests (open header round trip, flag tampering, rewrap both ways, restore of an open backup, deletion limited to open backups), `cargo test --lib`, `tsc`, `npm run build`, mock UI screens, and a real-exe run on a throwaway `CIPHERLEAF_HOME` only after the PC session is back. An independent security review before the pull request.

## Questions for the owner

See the thread reply. Short form: (a) open mode with a public built-in key, (b) existing password vaults stay protected and keep their screenshot setting, (c) turning on offers to delete earlier unprotected backups.
