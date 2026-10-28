# Execution Summary: EX-2026-10-28-36 — Test leftovers cleaned up on the PC

(4-section form, small work: deleting throwaway test folders. No source change.)

## 1. Task Overview

- **Baseline:** `origin/main` `bfece93`.
- **Owner's words:** "yes please" (2026-10-28 16:24:57 UTC, message timestamp) to deleting the test leftovers listed by the coordinator, plus scanning for others. At 16:26:54 UTC he asked "where's the setup in the pc so i can test it", so `v130b` was kept (the coordinator's relay at 16:26:59 UTC).
- **Scope and write area:** deletes only, on the owner's PC, under `%TEMP%`, `E:\Project\Notaya Tests Excluded` and `E:\Project\cipherleaf\vendor`. This record is the only repository change.
- **Stop point:** nothing outside the list was deleted. Times are UTC, PC system clock.

## 2. What was done

Before deleting, each target was checked: the git clones had no unpushed work (their branches are on GitHub), no process ran from any of them, and the only `.clf` files found were throwaway test vaults. `E:\Project\cipherleaf\vendor` was untracked (`git status` showed only it), not tracked by Git and referenced by nothing in the repository.

**Deleted:**

- `%TEMP%\notaya-v120-release` (4.1 GB), `%TEMP%\notaya-v120-zip-extract-tests` (11 MB)
- `E:\Project\Notaya Tests Excluded\run2` (4.2 GB)
- `E:\Project\cipherleaf\vendor` (215 MB, old KeePassXC files); `git status` is clean afterwards
- The 18 throwaway folders in `%TEMP%` named in [EX-2026-10-30](2026-10-27-notaya-rename.md) or its later checks, by exact name: `notaya-pr24`, `notaya-v120-zipcheck`, `clf-notaya-pr24-test`, `clf-notaya-pr24-home`, `clf-v110-release`, `clf-cipherleaf-v110-exe`, `clf-old-A-home`, `clf-old-A2-home`, `clf-old-A2-copy`, `clf-old-B2-home`, `clf-old-B2-copy`, `clf-notaya-e537071-smoke`, `clf-notaya-e537071-home`, `clf-notaya-e537071-home2`, `clf-notaya-d4933ef-smoke`, `clf-notaya-d4933ef-home`, `clf-mig-home1574515994`, `clf-mig-v110-extract1574515994`

**Not deleted, and why:**

- `%TEMP%\notaya-installer-2db590ae` (4.2 GB): access denied (Bitdefender, as before); not forced. **Addendum (coordinator relay 16:38:24 UTC, owner's message 16:38:19 UTC):** the owner restarted his PC and deleted it himself; `Test-Path` on the PC confirmed it is gone.
- `E:\Project\Notaya Tests Excluded\v130b`: the owner is testing the setup from it. `v130` was not on the list and was left.
- The registry key `HKCU\Software\Notaya` and the two `Notaya.lnk` shortcuts (Desktop, Start menu): the owner is installing the V1.3.0 setup now, which recreates them. `%LOCALAPPDATA%\Notaya` and `%LOCALAPPDATA%\Notaya Notes` changed at about 19:27 local time during his test, so they were not touched.
- The `%TEMP%\cipherleaf-backup-*`, `-open-*`, `-restore-*`, `-test-*`, `-store-*`, `-pwbackup-*`, `-kpxc-*`, `-portable-*` and `notaya-import-*` folders (about 3,000 small folders, roughly 55 MB, left by `cargo test` runs) and the `cipherleaf-dev*.log` / `cipherleaf-release*.log` files in `%TEMP%`: they look like test output but were not on the list, so they are listed here for the owner to decide.
- `E:\Project\Cipherleaf-*` release folders, `E:\Project\Cipherleaf-portable`, the NSIS tool cache, `%LOCALAPPDATA%\tauri`: never touched.

**Addendum (owner "delete them", 16:39:05 UTC):** the 3,037 `cipherleaf-*` / `notaya-import-*` test folders and 25 `cipherleaf-dev*` / `-release*` / `-mock` log files in `%TEMP%` were deleted (no process used them, 0 failures). Left: `notaya-v120-release-build.log` and `notaya-v120-release-build-e25f804.log` (not in that group).

**Addendum (owner, 16:43:19 UTC):** "keep everyhting in test folder that we did": `v130` and `v130b` stay in `E:\Project\Notaya Tests Excluded`; nothing more is deleted there. (`run2` had already been deleted at 16:27 UTC, before this message.) A check at 16:42 UTC found no installed Notaya left on the PC (no process, Uninstall entry, registry key, `Notaya` / `Notaya Notes` folder or shortcut).

## 3. Checks

Observation only: the existence and file counts of the targets before the delete, and the delete results printed by PowerShell. One broad wildcard delete in `%TEMP%` was blocked by the session's safety check and was not run; the same folders were then deleted by exact name. Free space on the system drive was 57.5 GB after.

## 4. Result and open points

- **Result state: produced** (the deletes happened on the owner's PC; this record is a docs-only change). Not verified beyond the output above; not accepted.
- **Open:** (1) ~~delete `notaya-installer-2db590ae`~~ done by the owner (see the addendum above); (2) after his setup test: the registry key, the two shortcuts, and whether to delete `v130b`, `v130` and the `cipherleaf-*` test folders; (3) the `Notaya Notes` folder from [FI-05](../failure-intelligence.md) stays unless he says to delete it.
