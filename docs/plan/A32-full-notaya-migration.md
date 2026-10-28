# A32: Full Cipherleaf-to-Notaya migration inside the program (V1.4.0)

**Status:** requested by the owner on 2026-10-28 ("why there's cipherleaf everywhere in the codes it's Notaya now" 17:56:41, "i want full migration in this version" 17:57:43). At 18:00:22 UTC he chose **option C, "Everything incl. encryption"** on the decision card, and named the file: "valut.not" (18:01:07), "nota" (18:01:32), "no nothing else" (18:03:03), so the vault file is **`vault.nota`**. Built in draft [PR #39](https://github.com/AbyssWatcher2/Notaya/pull/39). Not accepted.

## What changed

| Group | Before (V1.3.0 and earlier) | Now (V1.4.0) |
|---|---|---|
| Format names | header magic `CLF1` version 1; key label `cipherleaf/v1/key-encryption-key`; row binding `cipherleaf`; open-vault built-in password `cipherleaf-open-vault-security-off` | magic `NOTA` version 2; `notaya/v2/key-encryption-key`; `notaya`; `notaya-open-vault-security-off`. Same algorithms and layout. Both formats are read; only the new one is written. |
| Data files | `data/vault.clf`, `*.clf` backups, `restore-undo-*.clf`, `set-aside-*.clf` | `data/vault.nota`, `*.nota`, `restore-undo-*.nota`, `set-aside-*.nota`. Every place that lists, restores, imports or rotates accepts both `.nota` and `.clf`. |
| App identifier | `app.cipherleaf.vault` | `app.notaya.notes`, plus a guard (below). The installer keys on the product name, so updates from V1.3.0 are not affected. |
| Names in code | crate `cipherleaf`, `CIPHERLEAF_HOME`, `CIPHERLEAF_OPEN_BACKUP_SECS`, browser keys `cipherleaf.*` | crate `notaya-app` (lib `notaya_lib`), `NOTAYA_HOME`, `NOTAYA_OPEN_BACKUP_SECS`, keys `notaya.*`. The old environment variables are still read; old browser values are copied to the new keys once. The word stays in the password-strength "do not use" list. |

## How an old vault is brought across

1. **At start** (`Paths::adopt_old_vault`): if there is no `data/vault.nota` and there is a `data/vault.clf`, SQLite makes a checked copy (`vault.nota.part`, header read, synced), which is renamed to `vault.nota`. The old file is then set aside as `data/vault-before-notaya.clf` (`-2`, `-3` if that name is taken). It is **never deleted**. If the copy fails (read-only drive, damaged file), the old file stays the live vault for that run, so an empty vault is never started beside the old notes; it is tried again at the next start.
2. **At the first unlock** (`Session::upgrade_format`): the vault is re-keyed in one transaction (a fresh data key and vault id, every record and attachment re-encrypted with the Notaya names, header rewritten), with the same password and keyfile (or none for a vault with security off). All or nothing: on any error the file is unchanged, still readable in the old format, and it is tried again at the next unlock. The upgrade also runs after a restore of an old backup. A backup file is never changed by being looked at; only the live vault is upgraded.
3. Older backups (`*.clf`) stay as they are and open with the password they had; they rotate out together with new ones.

## Guard against an older Notaya

An old Notaya guards against a second copy with a mutex `app.cipherleaf.vault-sim` and a hidden window of class `app.cipherleaf.vault-sic` and title `-siw`. With a new identifier, the new version would not meet them and two copies could open one vault. `src-tauri/src/legacy_guard.rs` takes the old mutex and window too: a copy that starts later (old or new) finds them, sends the same "show your window" message the old guard sends, and exits. Cloud test: two claims in one process under Wine (with `xvfb-run`). **Not tested on Windows:** an installed V1.3.0 running while the new exe starts (and the reverse).

## What is not covered, said plainly

- Older Notaya cannot open the new files (`vault.nota` or an upgraded vault). Going back to V1.3.0 means using `vault-before-notaya.clf` renamed back to `vault.clf`; notes written in V1.4.0 are not in it.
- `vault-before-notaya.clf` and old `.clf` backups keep their old protection: if security was off, anyone with the file can read them. Turning security on offers to delete earlier unprotected copies; that now includes `vault-before-notaya*.clf` when its header says security was off. Nothing else deletes it.
- The older real-vault folder `E:\Project\Notaya\Cipherleaf-portable\data` is not touched by any test.

## Verified in the cloud (2026-10-28, `cargo test --lib --target x86_64-pc-windows-gnu` under Wine)

178 tests pass. New: header formats (round trip, the two formats do not open each other, every header byte is protected), upgrade of open, protected and keyfile vaults, a failed upgrade changes nothing (file byte for byte), upgrade on change of credentials or security, **three real files made by the V1.3.0 code** (`src-tauri/test-data/v1.3.0/`: open, protected, keyfile; they open and upgrade), adoption (copy, kept file, second kept file, leftover `.part`, failure fallback, an existing `vault.nota` never replaced), mixed `.clf`/`.nota` rotation, restore of a V1.3.0 backup, import of a V1.3.0 portable copy, the guard. **Not run:** everything on the PC (upgrade of a copy of a V1.3.0 portable folder, installed update, restore, import, old exe beside the new one).
