# A9: Bundle KeePassXC 2.8.0 for passwords

- **Status:** **approved** by the owner on 2026-10-22 at 13:07:49 UTC. In the same message the owner lifted the development pause and allowed the downloads (see [approval-and-amendments.md](approval-and-amendments.md)). The text below is the approved version; later changes are recorded as amendments.
- **Requested by:** the owner, Mohamed Sayed, on 2026-10-22: a request at 12:50:52 UTC for KDBX import/export and "all KeePassXC features", then the choice of option D at 13:03:51 UTC.
- **Prepared by:** Claude, 2026-10-22.
- **Amends:** the approved plan's **Password vault** and **Generator** features ([snapshot](approved-plan-2026-10-22.md), section "Features"). Everything else in the plan stays as approved.

## Why

The owner wants to import and export KDBX files and to use every KeePassXC feature. The alternatives were weaker:

- **Rebuilding KeePassXC's features in Cipherleaf (option C):** a very large project that adds risk.
- **Using the main Rust KDBX library (`keepass` 0.17.0):** its documentation calls KDBX writing "experimental", which breaks the stable-only rule.

Bundling the official KeePassXC app avoids both. It provides every KeePassXC feature at the quality of a stable release, and KDBX files are read and written by KeePassXC's own tools.

## Before and after

| | Approved plan | With A9 |
|---|---|---|
| Where passwords are stored | Cipherleaf's own vault (`data/vault.clf`) | A KeePassXC database, `data/passwords.kdbx` |
| Where passwords are managed | Cipherleaf's Passwords page | KeePassXC 2.8.0, opened from Cipherleaf |
| Generator | A Cipherleaf page | KeePassXC's generator. Cipherleaf keeps only the passphrase suggestion for its own master password. |
| TOTP, password health, breach check, CSV import | In Cipherleaf | In KeePassXC (reports, HIBP, imports and exports) |
| KDBX files | Not supported | Native: KeePassXC opens and saves them |
| Browser autofill, passkeys, Auto-Type, SSH agent, YubiKey | Not planned | Available in KeePassXC, off until the owner turns them on |
| Notes, sticky notes, settings, auto-lock | Cipherleaf | Unchanged |

## What happens to Cipherleaf's own password section

**Recommended (P1): retire it, and KeePassXC becomes the password section.**

- The **Passwords** item in Cipherleaf's sidebar becomes a small panel with **Open passwords in KeePassXC**. On first use it offers **Create a password database**, and KeePassXC's own wizard does the creating.
- **Removed from Cipherleaf:**
  - logins, TOTP codes, password health, CSV import, the breach check, copying passwords, and the Generator page;
  - the Rust code for these: `src-tauri/src/commands/logins.rs` and the login parts of `model.rs` and `lib.rs`;
  - the UI code: `src/features/passwords/` and `src/features/generator/`.
- **Kept in Cipherleaf:** the master-password strength check and passphrase suggestion used when creating Cipherleaf's vault (in `generator.rs`), plus any clipboard code still needed.
- **Recoverable:** the removed code stays in Git history, once the first commit is made.
- **No migration:** Cipherleaf has never run, so no logins exist yet.

**Alternative for later (P2), needing its own approval:** a Cipherleaf view over `passwords.kdbx` that reads and writes through `keepassxc-cli`, the command-line tool that ships with KeePassXC. That gives one password store with Cipherleaf's look. It also means more code, and secrets passing between the two programs.

**Rejected (P3): keep both password stores.** Every password would exist twice, with no single source of truth.

## Portable folder

```
Cipherleaf\
  Cipherleaf.exe
  data\vault.clf          notes, sticky notes, settings (Cipherleaf)
  data\passwords.kdbx     passwords (KeePassXC format)
  KeePassXC\              official KeePassXC 2.8.0, unmodified, plus its license
    .portable             keeps KeePassXC's settings in KeePassXC\config (to verify, Q4)
```

## How the two programs work together (P1)

- **Opening:** Cipherleaf starts `KeePassXC\KeePassXC.exe data\passwords.kdbx`, which opens in its own window. Cipherleaf passes only file paths, never secrets.
- **Unlocking:** at first there are two master passwords, one for Cipherleaf and one for the KeePassXC database. A single unlock depends on Q1.
- **Locking:** Cipherleaf's auto-lock can't lock KeePassXC unless KeePassXC accepts a lock command (Q2). KeePassXC's own auto-lock and clipboard clearing are preset instead.
- **Offline by default:** KeePassXC is preset not to check for updates or download icons. Browser integration, Auto-Type and the SSH agent stay off until the owner turns them on.
- **Protection gaps:** Cipherleaf's screenshot blocking and lock-on-drive-removal don't cover KeePassXC's window. KeePassXC's own protections apply there.

## Bundling procedure (phase D0)

D0 runs only after the pause is lifted and the owner approves the download.

1. Download three files from the official [KeePassXC 2.8.0 release](https://github.com/keepassxreboot/keepassxc/releases/tag/2.8.0):
   - `KeePassXC-2.8.0-x64.zip` (67.1 MB);
   - `KeePassXC-2.8.0-x64.zip.DIGEST` (91 bytes);
   - `KeePassXC-2.8.0-x64.zip.sig` (537 bytes).
2. Verify the ZIP's SHA-256 against the `.DIGEST` file. Verify the PGP signature against KeePassXC's release key: take its fingerprint from keepassxc.org/verifying-signatures and cross-check it with a second source.
3. Inspect the ZIP. Does it include the Microsoft Visual C++ runtime (Q3)? How does portable mode store settings (Q4)?
4. Keep the binaries out of Git (an ignored `vendor/` folder). Commit only a manifest: the version, source URL, SHA-256 and verification results.
5. At release, copy KeePassXC unmodified into the portable folder, with its license file and a link to its source code.

## Phases and approval gates

| Phase | Work | Gate before it starts |
|---|---|---|
| D0 | Download, verify and test KeePassXC on its own | The owner lifts the pause **and** approves the download |
| D1 | Retire the password section (P1), add the launcher panel, preset KeePassXC, update `security-design.md` and `STATUS.md` | D0's evidence reviewed |
| D2 (optional) | A Cipherleaf view over `passwords.kdbx` (P2) | A separate amendment, approved |
| Release | Include KeePassXC's license and source link | The owner confirms the license approach (Q5) before Cipherleaf goes to anyone else |

When development resumes, `docs/STATUS.md`'s first step comes first: re-running the existing checks. D1 is meant to land before the app's first full launch, so the old password pages aren't tested only to be removed.

## Success criteria

1. The bundled KeePassXC matches the official 2.8.0 files: SHA-256 and signature verified.
2. From Cipherleaf, the owner can create and open `passwords.kdbx` in KeePassXC, and the same file opens in a standalone KeePassXC 2.8.0.
3. KeePassXC's settings and data stay in the portable folder. Nothing new appears in `%APPDATA%` or `%LOCALAPPDATA%` (checked before and after).
4. With the preset settings, KeePassXC makes no network connections at startup.
5. Cipherleaf no longer contains login storage or login commands.
6. KeePassXC's license and source link are in the release folder.
7. The owner tries it hands-on and accepts it.

## Security impact

This goes into `security-design.md` once A9 is approved.

- **New trusted component.** KeePassXC holds every password secret, and Cipherleaf no longer does. KeePassXC is mature: keepassxc.org reports an ANSSI security certification for version 2.7.9. Cipherleaf's login code is new and unreviewed.
- **New boundary.** Cipherleaf launches KeePassXC and passes only file paths (P1).
- **New risks:**
  - An outdated bundled copy. 2.8.0 itself fixed several security issues, so the bundled copy must be kept updated.
  - A tampered `KeePassXC.exe` on the USB stick, the same risk as a tampered `Cipherleaf.exe`. A hash check at launch is optional.
  - KeePassXC may store recent file paths in plain text in its settings.
  - 2.8.0 was released 2026-10-21. It's a new major version that moved to Qt 6. If it shows problems, 2.7.12 is the fallback.
- **Effect on open amendments:**
  - A3 (breach-check TLS) no longer applies, because P1 removes Cipherleaf's breach check.
  - A4 and A5 still apply: key generation and the master-password passphrase.
  - A1, A2 and A6–A8 are unchanged.

## Open questions (answered during D0)

- **Q1:** Can KeePassXC 2.8.0 safely take a master password from Cipherleaf, for example through standard input, so one unlock opens both?
- **Q2:** Can Cipherleaf tell a running KeePassXC to lock?
- **Q3:** Does the portable ZIP include the Visual C++ runtime, or must the PC already have it? The download page says KeePassXC requires it.
- **Q4:** Where exactly does portable KeePassXC 2.8.0 keep its settings, and does it write anything to AppData?
- **Q5:** What exactly does the GPL require when KeePassXC ships next to Cipherleaf? The GNU FAQ couldn't be checked on 2026-10-22.

## Decision requested

Approve A9 as written, with P1 for the password section. Or request changes, or reject it.

Approval is recorded in [approval-and-amendments.md](approval-and-amendments.md). It changes the plan's scope only: development stays paused until the owner lifts it, and the download needs its own approval.

## Sources (checked 2026-10-22)

- [KeePassXC 2.8.0 release and files](https://github.com/keepassxreboot/keepassxc/releases/tag/2.8.0)
- [KeePassXC downloads](https://keepassxc.org/download/)
- [KeePassXC website](https://keepassxc.org/)
- [KeePassXC README](https://github.com/keepassxreboot/keepassxc)
- [keepassxc-cli manual](https://github.com/keepassxreboot/keepassxc/blob/develop/docs/man/keepassxc-cli.1.adoc)
- [keepass-rs README](https://github.com/sseemayer/keepass-rs)
- [keepass on crates.io](https://crates.io/crates/keepass)
