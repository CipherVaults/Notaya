# A21: Notes first, security and passwords optional (design, built)

**Status:** design agreed by the owner at 18:39:02 UTC ("just to confirm agree but the securti can be off and on okay", agreeing to the three confirmations at the end of this file), then built. Built in commits `84b281d` (core, UI, README) and `0929ae5` (real-exe test scripts), and changed after an independent review in `f9d2676` (re-keying, quit and restore safeguards; see the section "Changes after the independent review" below), all pushed; a draft pull request against `main` follows (number: to be filled). The design was written 2026-10-23 (system clock, 18:35 UTC). See [EX-2026-10-23-17](../execution/2026-10-23-notes-first-optional-security.md). Built does not mean accepted: the app crate and real-exe results are PENDING-FILL, the fixes from the independent security review were not re-reviewed (PENDING-FILL), and the owner has not reviewed it.

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

**Turning security on or off re-keys the vault** (changed in `f9d2676`; the first build only re-wrapped the header). The vault gets a fresh data key and a fresh vault id. Every record and attachment is re-encrypted, one at a time, in one all-or-nothing transaction, and then the new header is written. If anything fails, nothing changes in the file or in memory. It takes a moment, and large notebooks take longer.

**Why:** the independent review showed that re-wrapping alone would let any earlier open copy open the protected vault. A copy made while security was off can be opened by anyone, and with the same data key it would also open everything written later. With a fresh key, a copy made while security was off cannot open later data. The same holds the other way: after security is turned off, older protected backups still need their own password and cannot be opened with the new open file.

"Change master password" while security is on still only re-wraps the header. Older backups still open with the old password (an existing limit).

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
- **Turn on:** a dialog asks for a master password (same rules, strength meter, suggested passphrase and optional keyfile as today), the "no password reset" confirmation, and one choice: **delete earlier backups that were made while security was off** (default on; only automatic backups in `data\backups` (`vault-*.clf`) whose header has `FLAG_OPEN`, and their `-journal` files, are touched; a fresh protected backup is made first). Old vaults kept by a restore (`restore-undo-*.clf`) are never deleted, because they hold notes that exist nowhere else. Manual exports made with "Save a backup copy" are never touched. Deleted SQLite journal data may linger on disk. Then the vault is re-keyed under the new password with full Argon2 settings. The options appear with their previous values: auto-lock 5 minutes, lock on minimize off, **screenshot protection off**.
- **Turn off:** asks for the current password (and keyfile), shows the warnings below, and re-keys the vault under the built-in key. Wrong password counts toward the existing delay and changes nothing. A vault with a keyfile can be turned off too (the app reports the keyfile flag while unlocked, so the dialog asks for the keyfile).

Warnings shown when turning off: the notes will be readable by anyone who can open this folder or copy `vault.clf`; nothing will lock automatically; older protected backups in `data\backups` still need the password they were made with and cannot be opened with the new open file; the KeePassXC passwords file is not affected and keeps its own master password.

### Passwords switch (Settings, "Passwords (KeePassXC)")

- Off (default for new vaults): no sidebar item, no Ctrl+2, no palette entry, KeePassXC is never started, hidden or asked to lock. The backups of the passwords file are not made. Nothing in `data\passwords.kdbx` is touched.
- On: today's panel appears. Turning it off closes KeePassXC politely first (the same close routine as Quit and refuses with the existing message if KeePassXC is still open).
- The backend refuses every KeePassXC command while the switch is off, not only the UI. That includes `restore_passwords` and `pick_passwords_backup_file`.
- A quit from the lock screen (no session to ask) still makes the last passwords-file copy, as it always did.
- Existing vaults: for a vault saved before A21 the switch starts **on** if `data\passwords.kdbx` or `keepassxc\keepassxc.ini` exists, so nobody loses their panel. Otherwise off.

### Where the switches are stored

Settings (inside the vault) gets `passwordsEnabled` (three states while loading: unset, on, off). Whether security is on is not a setting. It is the header flag, which is authenticated and readable before unlock.

### Defaults that change

| Item | Before | After |
|---|---|---|
| New vault | Create-vault screen with a required master password | Open vault, straight to notes |
| Screenshot protection | New vaults: on. Main window: always on until unlock | New vaults: off. Window created unprotected unless the vault file has a master password. Never forced on |
| Auto-lock | Always on | Only with security on |
| Passwords panel | Always shown | Own switch |

Existing vaults keep their saved screenshot setting (it cannot be told apart from a choice the person made). The lock screen of a vault that has a master password is hidden from screenshots only when the app starts (the window is created protected). After a lock during the session it follows the saved screenshot setting, which is off by default, so it can be captured. This is a known limit and was not changed.

## Security effects to review

- Downgrade by file swap: someone who can write to `data\` could replace a protected vault with an open one. The app would then show empty notes and no password prompt. That needs the same write access that already lets them replace the exe, and the missing prompt is visible. Recorded as a known limit.
- Open-mode backups and `restore-undo-*.clf` files stay readable by anyone. Turning security on offers to delete the automatic backups that are open. It never deletes `restore-undo-*.clf` files; the person removes them by hand.
- `disable_security` needs the current password and is rate-limited like unlock. But it is not the only way to lower security: `restore_vault` with the confirmation (`allowSecurityOff`) can turn security off, and `enable_security` can delete earlier open backups. Both are callable by any window. The restore confirmation is only a UI gate: a script injected into the UI could pass it. Recorded as a known limit.
- Keys still stay in the Rust core; the UI never sees the built-in key.
- Restore from a backup: a backup with `FLAG_OPEN` is restored without a password (its key is built in); one with a password asks as today. If the live vault has a master password, restoring an open backup is refused with an error starting "SECURITY_OFF:" until the person confirms. The UI then shows a checkbox and the command `restore_vault` takes `allowSecurityOff`.
- Quit: a quit of a vault with security off does not reload the window and does not reopen that vault while the quit runs. If the quit is cancelled because KeePassXC stays open, the vault is opened again (it does not stay locked). A quit of a protected vault is unchanged: it stays locked.
- Crash between the two renames of a restore: the app now shows the "No notes found here" screen (Restore or Start new notes), not "Create your vault".
- The suggested-passphrase Copy button now lives in the Turn on security dialog. The clipboard is not cleared automatically.

## Notes stay local

Added at the owner's request, 2026-10-23, 19:02:09 UTC ("also the notes will be saved localy"). This was already true and is now stated in the README: notes are stored only in `data\vault.clf` (plus backups in `data\backups`) next to the app. No cloud, account or sync. A check of `src-tauri/src` and `src` found no HTTP client, socket or `fetch` call; `src-tauri/Cargo.toml` has no network crate; the CSP `connect-src` allows only the app's internal channel (plus the dev server in dev mode). The one outward action is `open_url`, which opens a link from a note in the default browser, only when the person clicks it. The check is a source search, not a packet capture, so it is not a proof of no traffic from the WebView or KeePassXC.

## Not changing

Notes, sticky notes, calculator, pictures, backups, restore, KeePassXC embedding, crypto, record format, `docs/plan/approved-plan-2026-10-22.md`.

## Checks planned (see the execution summary for what ran)

Unit tests (open header round trip, flag tampering, re-keying both ways with a fresh key and vault id, a failed re-key that changes nothing, restore of an open backup and the staged-copy check, deletion limited to open automatic backups). The Tauri-free core harness runs 69 tests, 13 of them new for A21. App crate results: PENDING-FILL, `cargo test --lib`, `tsc`, `npm run build`, mock UI screens, and a real-exe run on a throwaway `CIPHERLEAF_HOME` only after the PC session is back. An independent security review before the pull request.

## Questions for the owner

See the thread reply. Short form: (a) open mode with a public built-in key, (b) existing password vaults stay protected and keep their screenshot setting, (c) turning on offers to delete earlier unprotected backups.

**Answer:** the owner agreed at 18:39:02 UTC ("just to confirm agree but the securti can be off and on okay"). Also: turning security off needs the current password and shows the warnings.


## Changes after the independent review

One independent review agent (read-only, model output) found 2 high, 3 medium and 6 low issues. Commit `f9d2676` fixed the two high ones (the shared data key, now re-keying; keyfile users unable to turn security off), the three medium ones and most of the low ones. The restore confirmation, the narrower deletion, the quit handling, the Passwords refusals, the crash screen and the Copy button wording above come from that commit. The finding "low 10" (the lock screen can be captured after a lock during the session) was documented, not changed. The fixes were not re-reviewed. The list of fixed and unchanged findings is in [EX-2026-10-23-17](../execution/2026-10-23-notes-first-optional-security.md).
