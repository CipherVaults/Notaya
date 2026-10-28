# Notaya

*Formerly Cipherleaf.* The app was renamed after version 1.1.0, when it became a notes app first. Your notes, backups and settings from Cipherleaf open in Notaya unchanged: the `data` folder, `data\vault.clf` and its format stay the same.

Notaya is a portable Windows notes app. You write notes in notebooks, sections and pages, keep sticky notes on the desktop, and keep everything in one `data` folder, so the app can run from a folder or a USB stick.

**Your notes are saved locally.** They live in one file, `data\vault.clf`, next to the app on your PC or USB stick. There is no cloud, no account and no sync, and Notaya's own code has no network code, so your notes are never sent anywhere. One honest caveat: the window is drawn by Microsoft's WebView2 component, and in a test on Windows 11 it made background connections to Microsoft addresses at startup. What it sends was not captured, and Notaya does not currently stop it, so the app is not silent on the network. Notaya's own code never sends your notes anywhere. A web link in a note opens in your normal browser only when you click it.

Security is an optional extra. It is off by default: Notaya opens straight to your notes with no password. While it is off your notes are not private: anyone who can open the folder can read them. You can turn security on or off at any time in Settings.

> **Version 1.0.1** (a fix release after the first release, 1.0). The repository is private, portable builds are shared by hand with a few close people, and the exe is not code-signed (Windows SmartScreen may warn). What is and is not proven is in [`docs/acceptance.md`](docs/acceptance.md); the current project state is in [`docs/STATUS.md`](docs/STATUS.md).

## Contents

- [Features](#features)
- [How to use it](#how-to-use-it)
- [Is it safe?](#is-it-safe)
- [Requirements](#requirements)

## Features

### Notes (the core)

- **Notebooks, sections and pages.** A notebook has sections (a new notebook starts with one called "General"), and a section has pages. Pages can hold sub-pages to any depth. Notebooks have a name, an icon and a color.
- **Sidebar.** Resizable and hideable (`Ctrl+\`). The `…` menus offer: notebook (Add section, Edit notebook, Delete notebook), section (Rename, Delete section) and page (Add a page inside, Add to favorites, Move to, Move to trash). The Trash lists deleted pages with Restore and Delete, plus Empty trash.
- **Page tools.** Favorites, move a page to another section, move to Trash (with an Undo), Restore or Delete forever from Trash, and Empty trash. Drag sections and top-level pages to reorder them. The page bar shows the save state ("Saved on this drive", or "Saved & encrypted" when security is on). Hover it, or open the page's `…` menu, to see the word count and when the page was edited.
- **A block editor.** Hover a block for its handle: `+` adds a block below, and the grip drags the block or opens Change type, Duplicate and Delete. Press `/` for the insert menu: paragraph, three heading sizes, bullet list, numbered list, checklist, collapsible section, quote, highlight box (a callout with a clickable icon), code block, divider, table, picture, equation, inline equation, sub-page, and today's date.
- **Formatting bar** on selected text: block type, font (Default, Sans, Serif, Mono, Handwriting, Rounded), size, bold, italic, underline, strikethrough, code, link, text color and highlight, alignment and remove formatting. `Ctrl+B`, `Ctrl+I` and `Ctrl+U` work as usual.
- **Links.** Paste or type a link in the bar. `Ctrl+click` opens a web link in your browser (http and https only).
- **Pictures.** Add with `/Picture`, paste, or drag and drop. PNG, JPEG, GIF and WebP up to 25 MB. Pictures are stored inside the vault, not as loose files. Align, resize and add a caption.
- **Equations.** Block and inline, typed as TeX (for example `$x^2$`) and drawn on your PC (nothing is fetched for them).
- **Search.** `Ctrl+K` searches page titles and text, sticky notes and notebook names, or runs a command (with the box empty: New page, New sticky note, Go to Notes, Appearance & settings, Back up vault now, and Lock vault when security is on).
- **Autosave.** Pages save about half a second after you stop typing. Text typed just before closing or locking is saved first.
- **Home screen** with "New page", "New sticky note" and your recently edited pages.

### Sticky notes and calculators

- **Sticky notes** are small frameless windows on the desktop. Eight colors, bold, italic, checklist, "Keep on top" (on by default), resize and move; position and size are remembered. They are saved in the vault and are searchable. With security on they hide when the vault locks. Closing one with the X keeps it in the vault; click its tile in "Sticky notes" or press "Show all on desktop" to bring it back.
- **Calculator.** The Calculator button on the Sticky notes screen, or the calculator button on a sticky note, opens a basic and scientific calculator in its own small window (Deg/Rad, Copy, percent such as `200 + 10%`). Open as many as you like. They share one history of the last 200 calculations, saved in your vault (encrypted when security is on), so it is still there after you close them; Clear empties it. Calculator windows close when the vault locks.

### Looks (Settings, Appearance)

- Theme: Black, Graphite, White, Paper or Match Windows.
- Accent color (eight colors, black and white, or your own), font (Inter, Segoe, Serif, Mono), text size, page width (Narrow to Full), density (Comfortable or Compact).
- Window material: Solid, Mica or Acrylic (the last two need Windows 11).
- Lock screen background: Glow, Plain, Mesh or Aurora. Default color for new sticky notes.

### Optional: security

By default security is **off**: notes open straight away, nothing locks, and screenshots of the app are allowed. When you turn it on you get:

- A **master password** that unlocks your notes (at least 10 characters and a "Strong" rating). You can add an optional **keyfile**, a file that must be present as well.
- **Auto-lock** after 1 to 60 minutes of inactivity (5 to start with).
- Lock when you press `Win+L`, when the PC sleeps or hibernates, and when the drive with the vault is removed. Optionally lock when the window is minimized.
- A **Lock** button in the title bar and `Ctrl+L`.
- A short wait after wrong passwords (none for the first two tries, then 2 seconds, doubling up to a minute).
- **Block screenshots & screen sharing**: its own switch, **off** until you turn it on. It is never switched on for you.
- **Change master password and keyfile** in Settings ("Change…"). This also makes a new data key and re-encrypts your notes. Notes saved from now on cannot be opened with the old password, but older backups still open with the old password, so keep them somewhere safe or delete them.
- **Turn off security** from Settings, at any time (see below).

### Backups and restore

- **Automatic backup** when the app quits, and also when the vault locks if you use security, if something changed. Saved in `data\backups`; the newest 10 are kept.
- **Hourly backup (security off only).** An open vault never locks, so while you work it is also backed up about once an hour if something changed and you have paused for a moment (`vault-hourly-<number>.clf`; the newest 3 are kept, so they never push out the older automatic backups).
- **Safety backup before a delete.** Before a notebook or section is deleted, Notaya makes a backup first (`vault-predelete-<number>.clf`). If it can't be made, nothing is deleted. The newest 5 of these are kept.
- If a backup fails, Notaya shows a message.
- **Save a backup copy** from Settings, or "Back up vault now" in the `Ctrl+K` menu, saves a copy wherever you choose.
- **Restore** from the lock screen, from Settings, or from the start screens. Notaya checks the backup first and keeps your current file as `restore-undo-....clf`. Nothing is deleted.
- **Start over** sets a vault you can't open aside as `set-aside-....clf` and lets you start new notes. Nothing is deleted, and you can bring the old file back with Restore.
- **Open data folder** from Settings.

### Install or portable

Releases from V1.3.0 on (see [the installer plan](docs/plan/windows-installer-plan.md)) have two downloads: a **setup** (`Notaya-V<version>-setup.exe`) and the **portable ZIP**. Both are the same app.

- **Setup.** Installs Notaya for the current Windows user (no administrator prompt) with a Start menu shortcut and an optional desktop shortcut. Your notes live in `%LOCALAPPDATA%\Notaya Notes\data` (Settings, "Open data folder" opens it). The program itself is in `%LOCALAPPDATA%\Notaya`. Uninstalling removes the program and never your notes. To update, run the newer setup over the old one (close Notaya first). On its first start with no notes, an installed Notaya asks whether to start new notes or **import the notes of a portable copy**: choose the portable Notaya folder; your notes and backups are copied and the portable folder is not changed (close the portable Notaya first: an installed and a portable Notaya cannot run at the same time, because starting the second only brings the first to the front). The setup downloads Microsoft's WebView2 only if the PC lacks it. It is not code-signed, so Windows SmartScreen may warn: choose "More info", then "Run anyway".
- **Portable.** Everything lives in the `data` folder next to `Notaya.exe` (the ZIP has a `portable.txt` file that makes it so; keep it, and always copy the whole folder, because a lone `Notaya.exe` without it counts as an installed copy and keeps its notes on that PC). Copy the folder to move your notes. Only one copy of the app runs at a time. Do not share `data\` with other people: it holds your notes.

## How to use it

### Getting started

1. Unzip the folder anywhere (a normal folder or a USB stick) and run `Notaya.exe`.
2. The first time, Notaya opens straight to your notes. There is no password and nothing to set up.
3. Click **Create your first notebook** (or the `+` next to Notebooks). Give it a name, an icon and a color.
4. Press `Ctrl+N` or "New page" (the sidebar must be showing), click the title at the top and type it, then press Enter to move into the page.

### Writing notes

- Press `/` to insert headings, lists, checklists, tables, pictures, equations and more. Select text to get the formatting bar.
- Use the `...` menu next to a page to add a page inside it, favorite it, move it or send it to Trash.
- Press `Ctrl+K` to search everything, or to run a command.
- Useful keys: `Ctrl+K` search, `Ctrl+N` new page, `Ctrl+,` Settings, `Ctrl+\` show or hide the sidebar, `Ctrl+1` Notes, `Ctrl+3` Sticky notes, `Ctrl+L` lock (when security is on).
- Sticky notes: open "Sticky notes" in the sidebar and press "New note". In its top bar, the dot picks the color and the pin switches "Keep on top" off or on. The X closes the note (it stays in the vault) and the bin deletes it.

### Turn security on

1. Open **Settings** and find **Security (optional)**. Press **Turn on…**.
2. Choose a master password (a long phrase of several unrelated words is easy to remember and strong) and write it down somewhere safe. Add a keyfile if you want one ("Create new" or "Use existing"). Confirm that you understand there is no password reset.
3. Leave "Delete earlier backups made while security was off" ticked unless you want to keep them. Notaya saves a fresh protected backup first and deletes the files named `vault-*.clf` in `data\backups` that open without a password (the automatic and hourly backups and the safety copies saved before a delete, so a notebook deleted earlier can no longer be brought back from them, and also any `vault-*.clf` you put there yourself); your own backup copies saved elsewhere, `restore-undo` and `set-aside` files are never deleted.
4. Press **Turn on security**. Your notes get a new key under the new password; this takes a moment, longer for big notebooks.
5. From now on the app locks as described above, and you unlock with the password (and keyfile).

### Turn security off

1. Unlock the vault as usual, open **Settings**, **Security (optional)**, and press **Turn off security…**.
2. Read the warnings, type your current master password (and choose your keyfile if you use one), and confirm.
3. Press **Turn off security**. The notes then open without a password. A wrong password changes nothing and counts toward the wait after wrong passwords.

With security off, your notes are saved in Notaya's file format, which is tamper-checked, but the key is a **public key built into the program**. So your notes are **not private**: anyone who has your `vault.clf` or a backup, and a copy of Notaya, can read them. The app says so on the Home screen until you dismiss the note.

### If you forget the password

There is no reset and no recovery. If you lose the master password, or a keyfile you required, the notes cannot be opened. The way forward is **Start over…** on the lock screen: type START OVER and press "Set aside and start over". Your old vault is set aside (never deleted); then press "Start new notes". If you find the password later, use **Restore from a backup…**, choose the `set-aside-....clf` file in `data\` and type its password. Backups made while security was on open with the password they had then.

### Backups and restore

1. Notaya backs up automatically when you quit (and on lock, if you use security), if something changed. The newest 10 are in `data\backups`. With security off it also makes an hourly backup while you work (`vault-hourly-<number>.clf`, newest 3 kept). Before a notebook or section is deleted it makes a safety backup first (`vault-predelete-<number>.clf`, newest 5 kept); if that can't be made, nothing is deleted. To get a deleted notebook back, use Restore with that safety backup: it replaces your whole vault with that copy and keeps the current one as `restore-undo`. If a backup fails, Notaya shows a message.
2. For a copy you control, use **Settings, Backups, Save a backup copy…** and keep it somewhere else (another drive is best).
3. To restore, choose **Restore from a backup…** (lock screen, start screens, or Settings), pick a backup from the list or "Choose another backup file…", enter "Password of this backup" if it has one, and press "Replace my vault with this backup" ("Restore backup" on the start screens). Your current vault is kept as `restore-undo-....clf`.
4. A backup made while security was off needs no password. Restoring one onto a vault that has a master password asks you to confirm that security will be turned off.

### Moving or backing up the folder

Close Notaya, then copy the whole folder (`Notaya.exe` and `data`). Moving the folder to another drive or PC does not change the password or the keyfile. A keyfile is identified by its content, so keep the same file, not a copy that has been edited.

**Updating from Cipherleaf 1.x.** Close Cipherleaf. Put `Notaya.exe` in the same folder as `Cipherleaf.exe`, next to the `data` folder, and start Notaya: it opens the same notes. When you have checked that your notes are there, delete `Cipherleaf.exe`. A shortcut or taskbar pin to `Cipherleaf.exe` must be made again for `Notaya.exe`.

### Upgrading from 1.0 or 1.0.1

- **KeePassXC is no longer part of Notaya.** The Passwords panel and its switch are gone. A passwords file made with an earlier version (`data\passwords.kdbx`, and its copies `passwords-*.kdbx` in `data\backups`) is left in the `data` folder as it is. Notaya does not open, change or back it up any more. To open it, use KeePassXC itself (download it from <https://keepassxc.org>), or the `KeePassXC` folder of your old portable copy if you still have it.
- Your notes, sticky notes, security settings and backups are not changed.

### Troubleshooting

- **"Wrong master password or keyfile for this Notaya vault."** If the vault uses a keyfile, it must be the exact file chosen when the vault was made.
- **"The vault file couldn't be read."** Another program (antivirus or a sync tool) may be using the file. Press "Check again" or "Try again". If it stays, use Restore or Start over.
- **"No notes found here."** The folder has backups but no current notes file. Restore a backup, or start new notes.
- **"Notaya could not start".** The message says either that Notaya cannot save its data in the `data` folder next to the app, or that it cannot open its window. For the first, the usual cause is a read-only folder (a write-protected USB stick, a CD, or a protected place such as Program Files): move the whole Notaya folder somewhere you can write to, for example your Documents folder, and start it again. For the second, the usual cause is a missing Microsoft WebView2 Runtime (Windows normally has it): install the "Evergreen" runtime from Microsoft's website (search for "WebView2 Runtime").
- **Windows SmartScreen warns about the exe.** The exe is not code-signed. Choose "More info", then "Run anyway" if you trust where you got it.

## Is it safe?

Short answers. The full list is in [`docs/security-design.md`](docs/security-design.md).

- **Local only, no network code.** Notes are saved only in `data\vault.clf` on your PC. Notaya's own code makes no connections (the page security policy allows only the app's internal channel). The WebView2 component that Windows uses to draw the window is Microsoft's, and a live test showed it contacting Microsoft addresses at startup; Notaya does not currently stop that and does not claim zero network traffic. A web link in a note opens in your browser only when you click it.
- **With security on:** notes are encrypted with XChaCha20-Poly1305, with the key derived from your password by Argon2id (256 MiB, 3 passes), and tampering is detected.
- **With security off, the notes are readable by anyone with the folder.** This is by design, and it is stated in Settings.
- **The exe is not signed.** Windows SmartScreen may warn. Notaya does not check itself, so a replaced exe would not be noticed.
- **No password reset.** See above.
- **The lock screen and screenshots.** The lock screen of a protected vault is hidden from screenshots only when the app starts. After a lock during the session it follows your screenshot setting, which is off by default.
- **Not defended:** malware or a keylogger on the PC, someone with administrator rights, reading memory while the app is open, and a file swapped by someone who can write to your `data` folder (for example replacing a protected vault with an open one).
- **Not proven yet.** Most checks so far are unit tests and scripted runs on throwaway data. The owner reports that on 2026-10-24 he checked idle lock, Win+L, sleep and USB removal by hand on a real PC and that they passed; there is no detailed record or automated test of them. The owner has not formally accepted any result. See [`docs/acceptance.md`](docs/acceptance.md) for exactly what was checked.

## Credits and licenses

Notaya is built on open source software, and we are grateful to everyone who made it:

- **[Tauri](https://tauri.app)** and the Rust crates it and Notaya use (RustCrypto for the encryption, SQLite for storage, and others).
- **npm packages** for the interface: React, TipTap and ProseMirror (the editor), KaTeX (equations), highlight.js (code colors), lucide icons, Radix, and others.
- **Fonts:** Inter and JetBrains Mono (SIL Open Font License 1.1) and the KaTeX fonts.
- **Microsoft Edge WebView2** shows the interface (required, not bundled).

Every component is listed in [`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md) with its version, copyright holder and license, together with the license texts that go with it. That file is also in the `Licenses` folder of the portable copy and inside the app (Settings, About, "Open source licenses…"). This is attribution, not legal advice.

**The license for Notaya itself has not been chosen yet.** Until the owner decides, no license is granted to use, copy or share Notaya's own code.

## Requirements

- Windows 10 or 11, 64-bit, with the Microsoft Edge WebView2 runtime (already part of Windows 11 and most Windows 10 installs).
- The window is at least 960 by 640 pixels. Mica and Acrylic window materials need Windows 11. Unlocking a password-protected vault uses about 256 MB of memory for a moment.
