# Cipherleaf

Cipherleaf is a portable Windows notes app. You write notes in notebooks, sections and pages, and you can keep sticky notes on the desktop. Everything is stored in one `data` folder, so the app can run from a folder or a USB stick.

Security is an optional extra. It is off by default: Cipherleaf opens straight to your notes with no password. You can turn security on or off at any time in Settings. A password manager (KeePassXC) is a second optional extra, with its own switch.

> **Status: in development, private, not yet accepted by its owner.** Portable builds are made by hand and shared only with a few close people. There is no public release, and the exe is not code-signed. The authoritative state is [`docs/STATUS.md`](docs/STATUS.md). What is and is not proven is in [`docs/acceptance.md`](docs/acceptance.md).
>
> **The "notes first, security optional" change is being built now** (design: [`docs/plan/A20-notes-first-optional-security.md`](docs/plan/A20-notes-first-optional-security.md)). This README describes that design. The owner has not accepted it, and it has not been checked on a real PC. Where the design and the code differ, the code and the records win.

## Core: notes

- **Notebooks, sections and pages.** Pages can sit inside other pages (sub-pages).
- **A block editor.** Press `/` to insert headings, lists, checklists, collapsible sections, quotes, highlight boxes, code, tables, pictures, equations and sub-pages. Select text to get a formatting bar, which includes font and size.
- **Pictures.** Paste or add a picture into a page. Pictures are stored in the vault, not as loose files. Only PNG, JPEG, GIF and WebP are accepted, up to 25 MB.
- **Equations.** Drawn on your PC, with no network.
- **Search.** Press `Ctrl+K` to search notes and sticky notes, or to run a command.
- **Sticky notes** on the desktop.
- **Calculators.** A basic and scientific calculator sits under the sticky notes list, and a small one inside each sticky note. Nothing you calculate is saved.
- **Themes and fonts.** Black, Graphite, White, Paper, or follow Windows. Accent colors, four fonts, text size, page width, density, and an optional Mica or Acrylic window material on Windows 11.
- **Backups and restore.** When the vault closes or locks after changes, an automatic backup is saved to `data\backups` (the last 10 are kept). You can restore a backup inside the app. A restore checks the backup first and keeps your current file as a `restore-undo-...` file. Nothing is deleted.
- **Safe to close.** Text you typed just before closing is saved first.
- **Portable.** Everything lives in the `data` folder next to `Cipherleaf.exe`. Copy the folder to move your notes. Do not share `data\` with other people, it holds your notes.

## Optional: security

By default security is **off**. Notes open straight away, nothing locks, and screenshots of the app are allowed.

### What turning security on gives you

- A **master password** that unlocks your notes. You can add an optional **keyfile** (a file that must be present as well).
- **Auto-lock** after a time of inactivity (5 minutes to start with, you can change it).
- Lock when you press Win+L, when the PC sleeps or hibernates, and when the USB drive is removed. You can also lock when the window is minimized.
- A **Lock** button and `Ctrl+L`.
- **Hide from screenshots and screen sharing.** This is its own switch and stays **off** until you turn it on. It is never switched on for you.

### How to turn it on and off

Open Settings and find "Security".

- **Turn on:** Cipherleaf asks you to choose a master password (with a strength check, a suggested passphrase and an optional keyfile) and to confirm that there is no password reset. It also asks whether to delete earlier automatic backups that were made while security was off (these can be read by anyone, see below). Old vaults that a restore kept (`restore-undo-...` files) and backup copies you saved yourself are never deleted. Turning security on gives your notes a new key and re-encrypts all of them. Your notes are kept, but this takes a moment, and a large notebook takes longer. A copy made while security was off cannot open notes saved after you turn security on.
- **Turn off:** Cipherleaf asks for your current password (and keyfile). It then shows warnings. After that, the notes open without a password again. Turning security off also gives your notes a new key and takes a moment.

Warnings you can expect when turning security off:

- Anyone who can open the folder or copy `vault.clf` can read your notes.
- Nothing will lock automatically any more.
- Older protected backups in `data\backups` still need the password they were made with. They cannot be opened with the new open file.
- The KeePassXC passwords file is not affected. It keeps its own master password.

### Be honest about "security off"

With security off, your notes are still saved in Cipherleaf's encrypted, tamper-checked format. But the key is a **public key built into the program**. So this is **not** confidentiality. Anyone who has your `vault.clf` or a backup, and a copy of Cipherleaf, can read your notes. Turn security on if your notes are private.

### If you forget the password

There is no reset and no recovery. If you lose the master password, or a keyfile you required, the notes cannot be opened. Write the password down somewhere safe. Backups made while security was on open with the password they had at that time. If you restore a backup that was made while security was off onto notes that have a master password, Cipherleaf asks you to confirm, because the notes will then open without a password.

## Optional: passwords (KeePassXC)

Passwords are kept by [KeePassXC](https://keepassxc.org) 2.8.0. It is bundled with the portable copy, unmodified, and Cipherleaf only starts it if its file matches the checked hash. It is shown inside the Cipherleaf window.

- **Separate switch.** Settings has a "Passwords (KeePassXC)" switch. It is off for a new vault. While it is off there is no Passwords item, and KeePassXC is never started.
- **Does not need security.** You can use passwords with security off, or security with passwords off.
- **Its own master password.** KeePassXC has its own master password, its own idle lock and its own database file, `data\passwords.kdbx`. Cipherleaf never sees that password or your entries.
- **Offline.** KeePassXC is set up to make no network connections. You can change that inside KeePassXC.
- **Backups.** While the passwords switch is on, the usual backup step keeps up to 10 encrypted copies of the passwords file when it changed (the design does not make them while the switch is off). Cipherleaf never opens it. You can restore a copy from Settings while unlocked.
- **Quit** never force-closes KeePassXC. If it stays open, the quit is cancelled with a message.
- Turning the switch off closes KeePassXC politely first. Your `passwords.kdbx` is not deleted.

KeePassXC needs the Microsoft Visual C++ 2015 to 2022 x64 runtime on the PC.

## Is it safe?

Short answers. The full list is in [`docs/security-design.md`](docs/security-design.md).

- **No network code.** Cipherleaf makes no connections. KeePassXC is preset to make none.
- **The exe is not signed.** Windows SmartScreen may warn. Cipherleaf does not check itself, so a replaced exe would not be noticed.
- **No password reset.** See above.
- **With security off, the notes are readable by anyone with the folder.** This is by design, and it is stated in Settings.
- **The lock screen and screenshots.** The lock screen of a protected vault is hidden from screenshots only when the app starts. After a lock during the session it follows your screenshot setting, which is off by default.
- **Not defended:** malware or a keylogger on the PC, someone with administrator rights, reading memory while the app is open, and a file swapped by someone who can write to your `data` folder (for example replacing a protected vault with an open one).
- **Not proven yet.** Most checks so far are unit tests and scripted runs on throwaway data. The owner has not accepted any result. See [`docs/acceptance.md`](docs/acceptance.md) for exactly what was checked.

## Run it (development)

Windows 10 or 11, Node.js, Rust (stable, MSVC) and the WebView2 runtime. In PowerShell, from this folder:

```powershell
npm install
npm run tauri dev
```

The development vault lives in `.devdata\` (ignored by Git). Set `CIPHERLEAF_HOME` to use another folder.

Checks (what each shows and does not show is in [`CLAUDE.md`](CLAUDE.md)):

```powershell
cd src-tauri; cargo test --lib
npx tsc --noEmit -p tsconfig.json
npm run build
```

`npm run mock` serves the UI with fake data in a plain browser, for looking at screens. It is development-only and not part of the app.

## The KeePassXC files

The KeePassXC binaries live in `vendor\` and are **not** in Git. To recreate them, follow [`vendor/keepassxc-manifest.md`](vendor/keepassxc-manifest.md) (download, then check the SHA-256 and signature). A distributed copy of Cipherleaf must include KeePassXC's license files and a link to its source (see the open licensing question in `docs/STATUS.md`).

## Layout of a portable folder

```text
Cipherleaf\
  Cipherleaf.exe
  KeePassXC\            (from vendor\)
  data\                 (vault, backups, settings, KeePassXC database and settings)
```

A portable copy also carries `Licenses\KeePassXC\` (KeePassXC's licenses and source) and a `README.txt` with the exe's SHA-256. Never put `data\` (your notes) into a ZIP you share.

## Build a portable copy

```powershell
npm install
npm run tauri build
```

This produces only the exe, `src-tauri\target\release\Cipherleaf.exe` (the bundle step is off). Copy it into a new folder together with the verified `KeePassXC\` folder (see the manifest above) and the license files, and start it there. It creates `data\` next to itself. On a new PC the Microsoft Visual C++ 2015 to 2022 x64 runtime must be installed for KeePassXC.

## Working with GitHub

The repository is private: <https://github.com/AbyssWatcher2/CipherLeaf>. The default branch is `main`. Run `git fetch` and `git pull --ff-only` before pushing. Never commit vault data, keyfiles, `.env` files, password exports or the KeePassXC binaries (`.gitignore` covers them; check `git status` first).

## Project records

Work is governed by the records in [`docs/`](docs/): [STATUS](docs/STATUS.md) (accepted state and where to resume), [approved plan and amendments](docs/plan/approval-and-amendments.md), [acceptance ledger](docs/acceptance.md), [session history](docs/sessions.md), [Execution Summaries and their index](docs/execution/README.md), [failure intelligence](docs/failure-intelligence.md), [security design](docs/security-design.md). Rules for AI agents are in [`CLAUDE.md`](CLAUDE.md).

## Third-party software

KeePassXC (GPL 2 or 3), KaTeX 0.19.0 (MIT), the EFF large wordlist (CC BY 3.0 US, © Electronic Frontier Foundation), TipTap, React, Tauri, and the other packages listed in `package.json` and `src-tauri/Cargo.toml`.
