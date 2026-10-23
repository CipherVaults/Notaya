# A19: Back up and restore the KeePassXC passwords file; Quit never force-ends KeePassXC

**Status:** applied and committed (`442036e`) on the owner's instructions of 2026-10-23 (times not recorded; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. This is a new feature and a behavior change. It is security-relevant: Cipherleaf now **reads and copies the encrypted passwords file** (it never decrypts or opens it), adds a restore that **replaces** that file, and changes what **Quit** does to KeePassXC.

## What the owner asked for

"Next: include the KeePassXC passwords file in backups and restore, and stop Quit from force-closing KeePassXC mid-save. Use agents and skills, and ask me before changing security behavior." Because this changes security behavior, Claude asked two questions and the owner chose:

- **Backups:** "Copy + restore, keep 10". Copy the passwords file on each lock when it changed, keep the last 10 copies, and add an in-app restore.
- **Quit:** "Stay open and tell me". If KeePassXC does not close, Quit is cancelled with a message. Nothing is ever force-ended.

## What changed

1. **Copies of the passwords file on every lock** (`src-tauri/src/pwbackup.rs`).
   - When Cipherleaf locks (after the vault's own backup, before KeePassXC is asked to lock), the passwords database (`data\passwords.kdbx`, or the first other `.kdbx` in `data\`; never `passwords-undo-*`) is copied to `data\backups\passwords-<number>.kdbx`, but **only if its content differs from the newest copy**.
   - Safety checks: the file must start with the 8-byte KeePass signature and be larger than 0; its size and modified time are compared before and after the copy (if it changed during the copy, the copy is skipped); the copy is written to a temp file and renamed; one backup runs at a time (a process-wide lock).
   - The newest **10** copies are kept, ordered by modified time (ties by the number in the name). The copy just made is never deleted. The `vault-*.clf` backups are not affected.
   - It is best effort: an error goes to stderr and never blocks the lock.
   - After a successful Quit, one more copy is made (KeePassXC may have saved while closing).
2. **Restore of the passwords file** (commands `list_password_backups`, `pick_passwords_backup_file`, `restore_passwords`; code in `src-tauri/src/commands/backup.rs` and `pwbackup.rs`). In the app: **Settings, Backups, "Restore passwords file..."** opens the same dialog as the vault restore, in a new "passwords" mode (`src/features/unlock/RestoreDialog.tsx`). It is **not offered on the lock screen**.
   - It works only while Cipherleaf is **unlocked** (checked again after KeePassXC has been closed, just before installing). One restore at a time (shared with the vault restore).
   - The source is either a bare name `passwords-<number>.kdbx` looked up inside `data\backups`, or a full path only if it is the file last chosen in the file dialog (a separate remembered pick from the vault restore's). It refuses network (UNC) and device paths, `..`, a `:` in a name, a wrong file ending, folders, missing or empty files, files over 512 MiB, files without the KeePass signature, the live database, `passwords-undo-*` and temp names.
   - It asks KeePassXC to close politely first and waits up to 10 seconds. If KeePassXC is still open, it stops and says: "KeePassXC is still open. Finish there (it may be asking to save), close it, and try again."
   - Then it copies the source to `data\passwords-restore-check.tmp`, checks the signature and size, **renames** the current database to `data\passwords-undo-<number>.kdbx` (never deleted, never overwritten, never rotated), and renames the copy to the live path. The rename is retried 6 times, 300 ms apart, with a rollback on failure. If there was no database yet, it creates `data\passwords.kdbx` and the undo file is null.
   - **No password is asked and the file cannot be checked beyond its signature.** The dialog says the backup opens with the KeePassXC master password it had when it was made.
   - The code that finds the live database now ignores `passwords-undo-*.kdbx`.
3. **Quit never force-ends KeePassXC.** The old step (kill after 3 seconds) was replaced by "ask KeePassXC to close like its close button, wait, never kill".
   - Quit (the window close and the `quit_app` command) now: locks as before, asks KeePassXC to close and waits up to **10 seconds** (the owner was told 3 seconds in the question; Claude raised it to 10 after the review because a save on close can take several seconds).
   - If KeePassXC is gone: one last passwords copy, then Cipherleaf exits.
   - If KeePassXC is still open: **Cipherleaf does not exit.** KeePassXC's window is detached so it shows as its own normal window (and any "save changes?" question is visible). A notice is stored and shown once as a message: "KeePassXC is still open (it may be asking to save). Finish there, close it, then close Cipherleaf again." Cipherleaf stays open and locked.

## Things to know

- **Up to 10 copies of the encrypted passwords file now sit in `data\backups`.** They open with the KeePassXC master password the file had when the copy was made, so an old master password stays usable through an old copy. They are rotated by modified time, not by age.
- **`passwords-undo-*.kdbx` files are never deleted** by the app and pile up; remove them by hand when you are sure.
- **A hand-placed file named exactly `passwords-<number>.kdbx` in `data\backups` is treated as a backup copy** and can be rotated away.
- **After a cancelled Quit, KeePassXC stays on screen as its own window next to a locked Cipherleaf.** It may be unlocked or waiting for an answer, and the lock request (`--lock`) may be blocked by its own save question. So the older promise "KeePassXC hides when Cipherleaf locks" does not hold in that case.
- **Quit is slower** (up to 10 seconds) when KeePassXC is slow to close.
- **A crash or power loss between the two renames of a restore** leaves no live database (the old one is in `passwords-undo-*`, the new one in `passwords-restore-check.tmp`). There is no recovery notice yet.
- **Not tested:** the new dialog mode and the Settings button were never seen on screen (not even in the mock UI; only type-checked and built); the file dialog for passwords backups end to end; restoring over a database with a name other than `passwords.kdbx` (unit tests only); Quit while KeePassXC is in the system tray or hidden, and Windows log off or shutdown. Details: [EX-15](../execution/2026-10-23-passwords-backup-and-safe-quit.md).

## Not changed

The vault format, the vault backups and the vault restore (A18), the encryption, and every other feature. Backups still cover only `vault.clf` and the passwords file, nothing else outside them.
