# KeePassXC bundle manifest (A9, phase D0)

This file is the committed record of the third-party KeePassXC files Cipherleaf bundles. The binaries themselves are kept out of Git (`/vendor/*` is ignored). Recreate them from the sources below and check the hashes.

## Files

| File | Source | Size (bytes) | SHA-256 |
|---|---|---|---|
| `KeePassXC-2.8.0-x64.zip` | https://github.com/keepassxreboot/keepassxc/releases/download/2.8.0/KeePassXC-2.8.0-x64.zip | 70,348,471 | `48FF4F088E9CE70E241DA1270E5081528DBA77C38B17DAE4AA6690FFC50FC828` |
| `keepassxc-2.8.0-src.tar.xz` (corresponding source) | https://github.com/keepassxreboot/keepassxc/releases/download/2.8.0/keepassxc-2.8.0-src.tar.xz | 10,867,052 | `4ED4B77D375F021FC04648C257FEDFB41DC13C6BD5C4965D8DED31284C9C653C` |
| `KeePassXC.exe` (inside the ZIP) | — | — | `72200BFE1A6C59162BEE904832F3B292EDE45325EFFCEF54060C6AFF836DD28D` |
| `keepassxc-cli.exe` (inside the ZIP) | — | — | `FF737226E5FAC1517652781908E5FBAC3E626AD45215DB63B6EE86EF2AA75C37` |

Both downloads come with a `.DIGEST` and a `.sig` file from the same release.

## Verification (2026-10-22, about 13:15–13:25 UTC)

- **SHA-256:** both files match their official `.DIGEST` files.
- **PGP signatures:** both are `GOODSIG` / `VALIDSIG` from "KeePassXC Release <release@keepassxc.org>":
  - signing subkey `C1E4CBA3AD78D3AFD894F9E0B7A66F03B59076A8`;
  - primary key `BF5A669F2272CF4324C1FDA8CFB4C2166397D0D2`;
  - signed 2026-10-21 (ZIP) and 2026-10-22 (source).
- **Where the key came from:** keys.openpgp.org (by fingerprint). Its fingerprint matches keepassxc.org/verifying-signatures.
- **How it was checked:** `gpg` from Git for Windows, with a throwaway keyring that was deleted afterwards.

## What D0 found (A9 open questions)

| Question | Answer | Evidence |
|---|---|---|
| Q1: one unlock for both? | Possible: `KeePassXC.exe --pw-stdin` reads the database password from standard input, and `--keyfile` is supported. **Not used in D1**: A9 P1 passes only file paths. | Bundled user guide, "Command-Line Options" |
| Q2: can Cipherleaf lock KeePassXC? | **Yes:** `KeePassXC.exe --lock` locks all open databases in the running instance. | Bundled user guide; ran harmlessly with no database open |
| Q3: Visual C++ runtime included? | **No.** `KeePassXC.exe` imports `MSVCP140.dll`, `VCRUNTIME140.dll` and `VCRUNTIME140_1.dll`, and the ZIP doesn't include them. The PC needs the Visual C++ 2015–2022 x64 runtime; this PC has v14.52. | `dumpbin /dependents`; search of the ZIP |
| Q4: where are settings kept? | By default `%AppData%\KeePassXC` and `%LocalAppData%\KeePassXC`. The ZIP ships a `.portable` marker. Cipherleaf passes `--config` and `--localconfig` explicitly, so settings live in Cipherleaf's `data` folder. **Tested at startup: nothing written to AppData.** | Bundled user guide; launch test |
| Q5: GPL obligations | KeePassXC is "GPL version 2 or (at your option) version 3" (`COPYING`). Cipherleaf ships the license files and the full corresponding source archive next to the program. **The owner confirms this approach before giving Cipherleaf to anyone else.** | `COPYING` from the verified source archive |

## Launch test (preset configuration, 8 seconds, no database open)

- No TCP or UDP endpoints owned by KeePassXC.
- No `%AppData%\KeePassXC` or `%LocalAppData%\KeePassXC` folder created.
- KeePassXC wrote only to the config file it was given.
- KeePassXC protects its own process: other processes can't force-kill it (access denied). A polite close works.

## Other findings

- `keepassxc-cli` creates databases with **AES-KDF (1,000,000 rounds) and AES-256** by default. That's weaker than Argon2, so Cipherleaf doesn't create databases through the CLI; KeePassXC's own wizard does.
- KeePassXC blocks screenshots and screen recording by default (`--allow-screencapture` turns that off).
