# A32: Full Cipherleaf-to-Notaya migration inside the program (V1.4.0)

**Status:** requested by the owner on 2026-10-28 17:56:41 and 17:57:43 UTC ("why there's cipherleaf everywhere in the codes it's Notaya now" / "i want full migration in this version"). This is the plan and the choices; the parts marked **Do now** are in draft [PR #39](https://github.com/AbyssWatcher2/Notaya/pull/39), the parts marked **Choice** wait for the owner. It reverses the A28 decision "internal names kept on purpose" for everything that can change without risk to anyone's notes.

## Inventory (live code only, `git grep -i cipherleaf`, 2026-10-28, head `7df0004`)

| Group | Where | What depends on it |
|---|---|---|
| 1. **Part of the encrypted file format** | `OPEN_PASSWORD "cipherleaf-open-vault-security-off"` (`vault/header.rs`), the AAD prefix `b"cipherleaf"` (`vault/mod.rs`), the HKDF label `cipherleaf/v1/key-encryption-key` (`crypto.rs`) | Every existing vault, backup and open vault is encrypted with these. Changing a byte makes the notes unreadable. Nobody sees these strings. |
| 2. **Data file names** | `vault.clf`, backups `vault-<time>.clf`, `vault-hourly-…`, `vault-predelete-…`, the `.clf` file dialogs, the magic `CLF1` | Users' files, their backups, restore and import code, every older Notaya. |
| 3. **The app identifier** | `app.cipherleaf.vault` in `tauri.conf.json` | The single-instance guard (a Windows mutex named from it). Not used by the installer (it keys on the product name `Notaya`, so V1.3.0 upgrades cleanly either way). |
| 4. **Safe to rename now** | crate `cipherleaf` and library `cipherleaf_lib`; npm package `cipherleaf`; environment variables `CIPHERLEAF_HOME` and `CIPHERLEAF_OPEN_BACKUP_SECS`; browser-profile keys `cipherleaf.appearance`, `.layout`, `.ribbonCollapsed`, `.openVaultNoteDismissed`, `.notice`; comments; the password-strength word list; README and test script wording | Nothing in the encrypted data. Old environment variable names and old browser keys are still read (fallback), so nothing the owner or the test scripts set stops working. |

## Do now (no risk to notes)

Group 4: renamed, with fallbacks (`NOTAYA_HOME` first, then `CIPHERLEAF_HOME`; the new browser keys first, the old ones read once and copied). Comments and the README say Notaya. The word `cipherleaf` stays in the password-strength "do not use" list next to `notaya`. Older release notes and records keep their old names (they describe what was true then).

## Choice 1: the encrypted-format names (group 1)

- **A. Keep them (recommended).** Invisible in the app and in the files; changing them needs a re-encryption of every vault, and a mistake loses notes. They stay documented as "format names, `cipherleaf` on purpose" in one place (`docs/security-design.md`).
- **B. Migrate.** New vaults and backups use Notaya labels (a new format version in the header); an existing vault is opened with the old labels and re-encrypted once on the first unlock, with the old file kept as a copy. Needs two code paths for old and new, a new round of review and tests, and older Notaya cannot open the new vaults. Security-sensitive; about a day of work with reviews.

## Choice 2: file names (group 2)

- **A. Keep `vault.clf` (recommended).** The data folder looks the same as it always did; any older Notaya or your own copies keep working; restore and import need no change.
- **B. Rename to `vault.notaya` and backups to `.notaya`.** The first start copies (never moves) `vault.clf` and the backups to the new names, keeps the old files, reads both in restore and import. Older Notaya cannot open the new file; the real portable vault would be copied on its first start in V1.4.0.

## Choice 3: the app identifier (group 3)

- **A. Keep `app.cipherleaf.vault` (recommended).** An old and a new Notaya on the same notes still block each other.
- **B. Change to a Notaya identifier,** and add a guard of our own: the new version also takes the old mutex name and a hidden window class the old versions look for, so an older Notaya cannot start beside it on the same notes. Without that guard a V1.3.0 and a V1.4.0 could open the same vault at once.

## Tests for any choice that touches data

Throwaway data only, in `E:\Project\Notaya\Notaya Tests Excluded`: a portable V1.3.0 folder opened by the new exe (notes, backups and settings identical, old files untouched), an installed V1.3.0 updated by the new setup, a fresh install, a restore of a V1.3.0 backup, import from a portable copy, and an older exe refusing or ignoring the new files as designed. The owner's real portable vault is never touched.
