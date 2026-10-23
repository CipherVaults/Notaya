# A18: Readable first-run passphrase and restore from backup

**Status:** applied and committed (`1273093`) on the owner's instructions of 2026-10-23 (times not recorded; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. This is a new feature plus a small UI fix. It is security-relevant: it adds a new way to **replace the vault file**, and a copy of the suggested master passphrase to the clipboard.

## What the owner asked for

The owner chose, from Claude's list of ideas after the A17 review: "the first-run passphrase fix (show it in plain text with a copy button) and the in-app restore from backup next, using agents and skills". Later, after two pauses, they said "Finish the restore work", which authorizes finishing, committing, pushing the committed branches to the private repository, and rebuilding the portable folder and ZIP.

## What changed

1. **First run: the suggested passphrase is readable.** "Suggest a strong passphrase" now shows the phrase in plain text in a box (it breaks lines only at the hyphens), with a **Copy** button (it says "Copied" for 2 seconds; if copying fails it says so), a line telling you to write it down, and a warning that the Windows clipboard history (Win+V) may keep a copy. The phrase is held only in the screen's memory and is cleared when you edit the field, create the vault, or leave the screen. Before, the field showed only dots, so you could not write the phrase down.
2. **Restore from a backup, inside the app.** Reachable from the lock screen ("Restore from a backup..."), the create-vault screen ("Already have a backup? Restore it") and Settings, Backups. One dialog (`src/features/unlock/RestoreDialog.tsx`) shows the automatic backups from `data\backups` (newest first) or lets you choose another `.clf` file with the normal file dialog. You type the password of that backup (and pick its keyfile if it has one). The dialog warns that your current vault is kept. Errors appear inside the dialog; on success it says so and the app reloads after about 1.2 seconds.
3. **How a restore works** (code in `src-tauri/src/restore.rs` and `src-tauri/src/commands/backup.rs`; commands `list_backups`, `pick_backup_file`, `restore_vault`):
   - It first checks the rate limit, and only one restore runs at a time.
   - It accepts only (a) a backup of the form `vault-*.clf` given by name, which is looked up inside `data\backups`, or (b) the exact file you chose with the file dialog in this run of the app. Any other full path is refused, and so are network (UNC) and device paths, `..`, a `:` in a name, a wrong file ending, folders, the live vault itself, and files over 2 GiB.
   - It copies the file to `data\restore-check.tmp` and checks **that copy**: the key-derivation settings are limited first (memory up to 1 GiB, passes up to 10, lanes up to 16), then the password is tried, then every record and every attachment is opened.
   - If the vault is open, it is locked first the normal way (windows save pending edits, a backup is made, the session is wiped).
   - The current vault is **renamed** (not deleted) to `data\restore-undo-<number>.clf`; the checked copy is renamed to `vault.clf`; the restored vault is opened. If a step fails, the renames are rolled back. If the rollback itself fails, the message names the file.
   - If a restore fails after the lock, the window goes back to the lock screen and shows the reason once, as a notice.
   - Raw Windows errors (file in use, access denied, disk full) are turned into plain sentences.

## Things to know

- **Restoring brings back the old password.** The restored vault opens with the password and keyfile it had when the backup was made. The vault you replaced stays in `data\restore-undo-<number>.clf` and also opens with its own (newer) password.
- **The `restore-undo` files are never deleted by the app** and are not rotated. They pile up; remove them by hand when you are sure.
- **A crash or power loss between the two renames** can leave no `vault.clf`. The old vault is then in `restore-undo-<number>.clf` and the checked copy in `restore-check.tmp`, and the app would show "Create your vault". There is no automatic recovery or notice for this yet.
- **The suggested passphrase can stay in the clipboard.** The app does not clear it automatically, and Windows clipboard history or a cloud clipboard may keep it. The screen warns about this.
- **Not tested end to end:** choosing a file with the native file dialog and then restoring it. The restore dialog and passphrase box were seen only in the mock UI, not in the real window. Details: [EX-14](../execution/2026-10-23-restore-from-backup.md).

## Not changed

The vault format, the encryption, the unlock delay, and every other feature. The backups still cover only `vault.clf`, not the KeePassXC passwords file `passwords.kdbx` (still on the backlog from [EX-13](../execution/2026-10-23-lock-and-data-loss-fixes.md#findings-not-fixed-backlog)).
