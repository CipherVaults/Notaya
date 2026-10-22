# Cipherleaf: portable encrypted notes and password vault (Windows)

## Context
You want one portable desktop app that:
- runs from a USB stick or any folder, with no installer;
- combines OneNote-style notebooks, desktop sticky notes and a password manager with a generator;
- uses strong security and a striking UI.

Constraints:
- **Stable releases only.** No alpha, beta, rc or preview versions. Every dependency is pinned to an exact version.
- You're a beginner. I write the code and explain each part as I go.

Decisions you made:
- Unlock with a master password, plus an optional keyfile.
- Sticky notes float on the desktop and hide when the vault locks.
- The app is offline by default. An opt-in breach check is the only network use.
- The app is named **Cipherleaf**.

Current machine state: VS Build Tools 2022 (C++) and WebView2 are installed. Node.js, Rust and Git are not.

## Stack (all stable)
| Layer | Choice | Why |
|---|---|---|
| App shell | **Tauri 2** (Rust core + system WebView2) | ~10 MB single exe instead of Electron's ~150 MB. The Rust core is memory-safe, keys never reach the UI layer, and a permission system limits what the UI can call. |
| Crypto | RustCrypto `argon2`, `chacha20poly1305`, `hkdf`, `sha2`, plus `zeroize`, `secrecy`, `rand` (OS CSPRNG) | Pure Rust with no OpenSSL. `chacha20poly1305` has been audited (NCC Group). |
| Storage | `rusqlite` (bundled SQLite). Every payload is encrypted before it is written. | Single file, crash-safe transactions, safe on USB drives. |
| UI | React 19, TypeScript, Vite, Tailwind CSS 4, shadcn/ui (Radix), Motion, lucide icons, sonner toasts, cmdk palette, dnd-kit, Zustand | Polished and accessible, with smooth animation. |
| Editor | **TipTap** (ProseMirror), latest stable major | Rich OneNote-style editing. Lexical was rejected because it is still pre-1.0. |
| Extras | `zxcvbn` (strength), `totp-rs` (2FA codes), `window-vibrancy` (Win11 Mica), `reqwest` + rustls (breach check only) | |

Note on versions: several standard Rust crypto crates use 0.x numbers but are their stable production releases. Anything tagged `-alpha`, `-beta`, `-rc` or `-pre` is rejected. Exact versions are checked and pinned at build time.

## Security design
**Vault file** `data/vault.clf`: SQLite, with each row stored as an opaque ID plus an encrypted blob.

**Header** (plaintext but authenticated): magic, format version, Argon2 parameters, 32-byte salt, keyfile flag, wrapped data key.

**Key hierarchy**
- A random 256-bit **data key** encrypts everything.
- A **key-encryption key** wraps the data key:
  - Argon2id runs on the password and salt. Defaults are 256 MiB memory, 3 passes and 4 lanes, calibrated to about 1 s and never below 128 MiB.
  - If a keyfile is used, `HKDF-SHA256(argon2_out ‖ SHA-256(keyfile))` combines both into the key.
- Changing the password re-wraps only the data key and doesn't re-encrypt the vault.

**Encryption**
- XChaCha20-Poly1305 with a random 24-byte nonce per write.
- The additional authenticated data is `vault_id ‖ table ‖ row_id ‖ version`, so rows can't be swapped or replayed.
- Payloads are padded to size buckets to hide lengths.
- Timestamps and titles sit inside the encrypted payload.

**Unlock**
- A wrong password or keyfile gives the same generic error.
- Repeated failures get an increasing delay.

**Memory**
- Keys live only in Rust and are zeroized on lock.
- On lock, the UI is reloaded so decrypted content leaves the WebView. This is best effort, because JavaScript memory can't be wiped with certainty.

**Auto-lock triggers**
- Idle timeout (default 5 min)
- Windows session lock or sleep
- Minimize (optional)
- The USB drive being removed

**Clipboard**
- Copying happens from Rust, with Windows flags that exclude the value from clipboard history and cloud clipboard.
- The clipboard auto-clears after 20 s if it still holds the secret.

**Screen capture**
- All windows are content-protected (`set_content_protected`), so they show black in screenshots and screen shares. This can be toggled.

**Hardening**
- Strict CSP that allows no remote origins.
- A minimal Tauri capability set: no fs, shell or http plugins are exposed to the UI.
- Devtools are disabled in release builds.
- No auto-updater, and no telemetry.

**Breach check** (opt-in, off by default)
- Rust sends only the first 5 hex characters of the password's SHA-1 to `api.pwnedpasswords.com` (k-anonymity, with `Add-Padding`).
- That host is the only one allowed.

**Durability**
- Writes use `journal_mode=DELETE` and `synchronous=FULL`, which is safe if a USB stick is pulled.
- Encrypted rolling backups (the last 10) go in `data/backups/`.

**Search**
- An in-memory index is built after unlock and is never written to disk.

## Features
**Notes (OneNote-style)**
- Notebooks, sections and pages in a drag-and-drop tree.
- TipTap editor:
  - headings, highlight and text colors, lists and checklists, tables;
  - code blocks with syntax highlighting;
  - images (encrypted blobs);
  - slash menu and markdown shortcuts.
- Favorites, a trash with restore, word count.

**Sticky notes**
- Frameless, rounded, translucent windows with 8 colors.
- Always-on-top can be toggled, and windows resize.
- Positions are saved encrypted.
- All sticky windows close on lock and reopen on unlock.
- The main window also has a sticky board view.

**Password vault**
- Entry fields: title, username, password, URLs, notes, custom fields, tags, folders and color/icon.
- TOTP codes with a countdown ring.
- Health report covering weak, reused and old passwords.
- CSV import (Chrome, Edge, Bitwarden, 1Password). The app warns you to delete the CSV afterwards.
- Encrypted export.

**Generator**
- Random passwords use the OS CSPRNG with unbiased rejection sampling.
- Required character classes are met by regenerating, which keeps every output equally likely.
- Options: length, character sets, excluding look-alike characters.
- Passphrases use the EFF large wordlist (credited in About).
- Shows entropy in bits.

**Customization**
- Light, dark or system theme; an accent color picker; Mica or Acrylic backdrop.
- Font family and size, editor width, density.
- Per-notebook icon and color, an unlock-screen background, a list of keyboard shortcuts.
- Settings are stored inside the vault, so they travel with it.

**UX polish**
- Custom title bar and a three-pane layout.
- Ctrl+K command palette across notes, passwords and actions.
- Animated unlock screen and smooth transitions.
- Setup wizard with a strength meter, a keyfile option and a clear warning that a lost password can't be recovered.
- Bundled Inter and JetBrains Mono fonts, so nothing loads from the network.

## Portable layout
```
Cipherleaf\            ← any folder or USB stick
  Cipherleaf.exe       ← self-contained (UI embedded)
  data\vault.clf
  data\backups\
  data\webview\        ← WebView2 profile, so nothing goes to %APPDATA%
```
The app never writes to AppData or the registry. WebView2 is built into Windows 10 and 11.

## Project structure
```
cipherleaf/
  src-tauri/src/  lib.rs, portable.rs, crypto/{kdf,aead,keyfile}.rs,
                  vault/{header,store,session,autolock}.rs,
                  notes.rs, passwords.rs, generator.rs, sticky.rs,
                  clipboard.rs, breach.rs, search.rs, settings.rs
  src-tauri/capabilities/default.json, tauri.conf.json (CSP, windows)
  src-tauri/resources/eff_large_wordlist.txt
  src/  features/{unlock,notes,passwords,generator,sticky,settings}/,
        components/ui/ (shadcn), lib/ipc.ts (typed command wrappers), styles/
```

## Build order (each milestone runs)
1. **Install the toolchain first.** Run these with winget, using the stable channels:
   - `winget install --id Rustlang.Rustup -e`, then `rustup default stable` and the `x86_64-pc-windows-msvc` target;
   - `winget install --id OpenJS.NodeJS.LTS -e`;
   - `winget install --id Git.Git -e`.

   Then refresh PATH and check each with `--version`. Also confirm that VS Build Tools has the "Desktop development with C++" workload and a Windows SDK.
2. **Setup.** Move this session to a permanent folder, because the current one is deleted with the session. Scaffold Tauri 2 with React and TypeScript. Add Tailwind, shadcn and the portable data directory.
3. **Crypto and vault core in Rust,** with unit tests.
4. **Create/unlock flow.** Setup wizard, unlock screen, auto-lock, settings shell, theming.
5. **Passwords.** Generator, vault, clipboard, TOTP, health report, CSV import, breach check.
6. **Notes.** Tree, TipTap editor, images, search, trash.
7. **Sticky notes.** Windows and board view.
8. **Customization polish.** Mica, command palette, animations, shortcuts.
9. **Release.** Portable release build, security pass, final checks.

## Verification
**Rust tests** (`cargo test`):
- encrypt and decrypt round trip;
- a wrong password or keyfile fails;
- a tampered header byte or row fails;
- a swapped row is rejected because of the authenticated data;
- changing the password keeps the data;
- the generator meets each character-class rule, plus a loose uniformity test.

**Plaintext scan.** Search `vault.clf` and the backups for a known note phrase and a known password (`Select-String`). There should be no matches.

**Portability**
- Copy `Cipherleaf.exe` to a new folder and run it.
- Confirm nothing new appears in `%APPDATA%` or `%LOCALAPPDATA%`.
- Pull the USB stick while the app is unlocked. It should lock.

**Network.** With the breach check off, `Get-NetTCPConnection -OwningProcess <pid>` should show no connections.

**Manual checks**
- Auto-lock on idle, Win+L and sleep. Sticky notes vanish on lock.
- The clipboard clears and never appears in Win+V history.
- Windows show black in a screenshot.

**Audits.** Run `cargo audit`, `npm audit` and `/security-review` on the code.

## Honest limits (shown in-app too)
- A forgotten password, or a lost required keyfile, means the data can't be recovered. There is no backdoor.
- Malware or a keylogger on the PC can capture secrets while the vault is unlocked.
- An unsigned exe shows a SmartScreen warning. Code signing is optional later.
