# A20: Start over, clearer unlock errors, keyfile warning

- **Requested by:** the owner, Mohamed, passing on a friend's report (2026-10-23 17:04:30 UTC, session transcript): after moving the portable folder to another drive, unlocking said the password or file was wrong, and there was no way to start again.
- **Approved:** the owner's "Both" (17:16:33 UTC: run a real move test and write the fix) and "Push it run tests recheck" (17:26:10 UTC). Times are from the session transcript.
- **Status:** Applied; awaiting the owner's review. Not accepted.

## Finding

Moving the folder doesn't break unlocking. The vault is always `data\vault.clf` next to the exe, the keyfile counts only by its bytes (SHA-256), and nothing is bound to a path, drive or PC. A real-exe test on throwaway data (2026-10-23 17:21:30–17:22:57 UTC, system clock) moved a vault with a keyfile from E: to C: and unlocked it with the data intact. The same message, "Wrong password or keyfile.", came for a wrong password, a wrong keyfile and a missing keyfile, which made the friend's case hard to read. See [EX-16](../execution/2026-10-23-start-over-and-unlock-wording.md).

## Change

1. **Start over** (lock screen, "Start over…"): only while locked, after typing START OVER. Renames `vault.clf` (and its journal) to `data\set-aside-<ms>.clf`; never deletes. The first-run screen then shows. The old vault comes back with "Restore from a backup" and its old password. The KeePassXC passwords file is not touched.
2. **Wording:** the wrong-credentials error now reads "Wrong master password or keyfile for this Cipherleaf vault." The lock screen adds that KeePassXC has its own password, and says when a vault uses no keyfile.
3. **Keyfile warning:** a chosen keyfile inside the data folder, or of a kind that changes or belongs to an app (`.clf`, `.kdbx`, `.ini`, `.exe`, `.dll`, `.tmp`, `.log`, `.lnk`, `.db`, `.journal`), gets a warning. It is not refused.

Security-relevant (a new way to move the vault file aside): recorded as P19 in [security-design.md](../security-design.md).
