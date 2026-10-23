# Cipherleaf

A portable Windows desktop app for encrypted notes and sticky notes, with passwords kept in the official KeePassXC shown inside the same window. It is built to run from a folder or a USB stick and to keep everything in one `data` folder.

> **Status: in development, private, not yet accepted by its owner.** Portable builds are made by hand and shared only with a few close people; there is no public release, and the exe is not code-signed. Work happens on branch `feat/a9-keepassxc` and is pushed to `main` as well. The authoritative state is [`docs/STATUS.md`](docs/STATUS.md); what is and isn't proven is in [`docs/acceptance.md`](docs/acceptance.md).

## What it does

- **Notes** in notebooks, sections and pages (pages can sit inside pages). A block editor: press `/` to insert headings, lists, checklists, collapsible sections, quotes, highlight boxes, code, tables, pictures, equations and sub-pages; select text for a formatting bar (including font and size). Pictures are encrypted too.
- **Sticky notes** on the desktop; they hide when the vault locks. Under them, a **calculator** (basic and scientific), and a small one inside each note; nothing they calculate is saved.
- **Passwords** in KeePassXC 2.8.0 (bundled, unmodified, checked by hash before it starts, set up to stay offline). Cipherleaf never sees its master password.
- **Backups and restore:** when the vault locks, an encrypted backup is saved to `data\backups` (the last 10), and so is a copy of the encrypted KeePassXC passwords file when it changed (the last 10; Cipherleaf never opens it). A backup can be restored inside the app: the vault from the lock screen, the create-vault screen or Settings, the passwords file from Settings while unlocked. A restore checks the backup first and keeps your current file as a `restore-undo-…` or `passwords-undo-…` file; nothing is deleted.
- **Safe to quit and lock:** text typed just before Lock or closing is saved first, and Quit never force-closes KeePassXC (if it stays open, Quit is cancelled with a message).
- **First run:** a suggested passphrase is shown in plain text with a Copy button, so it can be written down.
- **Looks:** Black, Graphite, White, Paper, or follow Windows; accent colors; optional Mica/Acrylic.
- **Security:** a master password (and an optional keyfile) unlocks an encrypted vault; auto-lock; no network code. See [`docs/security-design.md`](docs/security-design.md) for what is promised, what is tested and what isn't.

## Run it (development)

Windows 10 or 11, Node.js, Rust (stable, MSVC) and the WebView2 runtime. In PowerShell, from this folder:

```powershell
npm install
npm run tauri dev
```

The development vault lives in `.devdata\` (ignored by Git). Set `CIPHERLEAF_HOME` to use another folder.

Checks (what each shows and doesn't show is in [`CLAUDE.md`](CLAUDE.md)):

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

A portable copy also carries `Licenses\KeePassXC\` (KeePassXC's licenses and source) and a `README.txt` with the exe's SHA-256. Never put `data\` (your vault) into a ZIP you share.

## Build a portable copy

```powershell
npm install
npm run tauri build
```

This produces only the exe, `src-tauri\target\release\Cipherleaf.exe` (the bundle step is off). Copy it into a new folder together with the verified `KeePassXC\` folder (see the manifest above) and the license files, and start it there; it creates `data\` next to itself. On a new PC the Microsoft Visual C++ 2015–2022 x64 runtime must be installed for KeePassXC.

## Working with GitHub

The repository is private: <https://github.com/AbyssWatcher2/CipherLeaf>. The default branch is `main`; day-to-day work is on `feat/a9-keepassxc` and is pushed to both. Run `git fetch` and `git pull --ff-only` before pushing. Never commit vault data, keyfiles, `.env` files, password exports or the KeePassXC binaries (`.gitignore` covers them; check `git status` first).

## Project records

Work is governed by the records in [`docs/`](docs/): [STATUS](docs/STATUS.md) (accepted state and where to resume), [approved plan and amendments](docs/plan/approval-and-amendments.md), [acceptance ledger](docs/acceptance.md), [session history](docs/sessions.md), [Execution Summaries and their index](docs/execution/README.md), [failure intelligence](docs/failure-intelligence.md), [security design](docs/security-design.md). Rules for AI agents are in [`CLAUDE.md`](CLAUDE.md).

## Third-party software

KeePassXC (GPL 2 or 3), KaTeX 0.19.0 (MIT), the EFF large wordlist (CC BY 3.0 US, © Electronic Frontier Foundation), TipTap, React, Tauri, and the other packages listed in `package.json` and `src-tauri/Cargo.toml`.
