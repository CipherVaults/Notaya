# Cipherleaf

Cipherleaf is a portable Windows notes app. You write notes in notebooks, sections and pages, keep sticky notes on the desktop, and keep everything in one `data` folder, so the app can run from a folder or a USB stick.

**Your notes are saved locally.** They live in one file, `data\vault.clf`, next to the app on your PC or USB stick. There is no cloud, no account and no sync, and the app has no network code, so nothing is sent anywhere. The only time it reaches the web is when you click a web link in a note, which opens in your normal browser.

Security is an optional extra. It is off by default: Cipherleaf opens straight to your notes with no password. You can turn security on or off at any time in Settings. A password manager (KeePassXC) is a second optional extra, with its own switch.

> **Version 1.0, the first release.** The repository is private, portable builds are shared by hand with a few close people, and the exe is not code-signed (Windows SmartScreen may warn). What is and is not proven is in [`docs/acceptance.md`](docs/acceptance.md); the current project state is in [`docs/STATUS.md`](docs/STATUS.md).

## Contents

- [Features](#features)
- [How to use it](#how-to-use-it)
- [Is it safe?](#is-it-safe)
- [Requirements](#requirements)

## Features

### Notes (the core)

- **Notebooks, sections and pages.** A notebook has sections (a new notebook starts with one called "General"), and a section has pages. Pages can hold sub-pages to any depth. Notebooks have a name, an icon and a color.
- **Sidebar.** Resizable and hideable (`Ctrl+\`). The `…` menus offer: notebook (Add section, Edit notebook, Delete notebook), section (Rename, Delete section) and page (Add a page inside, Add to favorites, Move to, Move to trash). The Trash lists deleted pages with Restore and Delete, plus Empty trash.
- **Page tools.** Favorites, move a page to another section, move to Trash (with an Undo), Restore or Delete forever from Trash, and Empty trash. Drag sections and top-level pages to reorder them. The page bar shows the save state ("Saved & encrypted"). Hover it, or open the page's `…` menu, to see the word count and when the page was edited.
- **A block editor.** Hover a block for its handle: `+` adds a block below, and the grip drags the block or opens Change type, Duplicate and Delete. Press `/` for the insert menu: paragraph, three heading sizes, bullet list, numbered list, checklist, collapsible section, quote, highlight box (a callout with a clickable icon), code block, divider, table, picture, equation, inline equation, sub-page, and today's date.
- **Formatting bar** on selected text: block type, font (Default, Sans, Serif, Mono, Handwriting, Rounded), size, bold, italic, underline, strikethrough, code, link, text color and highlight, alignment and remove formatting. `Ctrl+B`, `Ctrl+I` and `Ctrl+U` work as usual.
- **Links.** Paste or type a link in the bar. `Ctrl+click` opens a web link in your browser (http and https only).
- **Pictures.** Add with `/Picture`, paste, or drag and drop. PNG, JPEG, GIF and WebP up to 25 MB. Pictures are stored inside the vault, not as loose files. Align, resize and add a caption.
- **Equations.** Block and inline, typed as TeX (for example `$x^2$`) and drawn on your PC with no network.
- **Search.** `Ctrl+K` searches page titles and text, sticky notes and notebook names, or runs a command (with the box empty: New page, New sticky note, Go to Notes, Appearance & settings, Back up vault now, and Lock vault or Passwords (KeePassXC) when those are on).
- **Autosave.** Pages save about half a second after you stop typing. Text typed just before closing or locking is saved first.
- **Home screen** with "New page", "New sticky note" and your recently edited pages.

### Sticky notes and calculators

- **Sticky notes** are small frameless windows on the desktop. Eight colors, bold, italic, checklist, "Keep on top" (on by default), resize and move; position and size are remembered. They are saved in the vault and are searchable. With security on they hide when the vault locks. Closing one with the X keeps it in the vault; click its tile in "Sticky notes" or press "Show all on desktop" to bring it back.
- **Calculator.** A basic and scientific calculator sits under the sticky notes list (Deg/Rad, history of 30 entries, Copy, percent such as `200 + 10%`). A small one sits inside each sticky note. Nothing you calculate is saved.

### Looks (Settings, Appearance)

- Theme: Black, Graphite, White, Paper or Match Windows.
- Accent color (eight colors, black and white, or your own), font (Inter, Segoe, Serif, Mono), text size, page width (Narrow to Full), density (Comfortable or Compact).
- Window material: Solid, Mica or Acrylic (the last two need Windows 11).
- Lock screen background: Glow, Plain, Mesh or Aurora. Default color for new sticky notes.

### Optional: security

By default security is **off**: notes open straight away, nothing locks, and screenshots of the app are allowed. When you turn it on you get:

- A **master password** that unlocks your notes (at least 10 characters and a "Strong" rating), with a suggested passphrase you can copy. You can add an optional **keyfile**, a file that must be present as well.
- **Auto-lock** after 1 to 60 minutes of inactivity (5 to start with).
- Lock when you press `Win+L`, when the PC sleeps or hibernates, and when the drive with the vault is removed. Optionally lock when the window is minimized.
- A **Lock** button in the title bar and `Ctrl+L`.
- A short wait after wrong passwords (none for the first two tries, then 2 seconds, doubling up to a minute).
- **Block screenshots & screen sharing**: its own switch, **off** until you turn it on. It is never switched on for you.
- **Change master password and keyfile** in Settings ("Change…").
- **Turn off security** from Settings, at any time (see below).

### Optional: passwords (KeePassXC)

Passwords are kept by [KeePassXC](https://keepassxc.org) 2.8.0, bundled with the portable copy and shown inside the Cipherleaf window. This is a separate switch in Settings ("Passwords"), off for a new vault.

- Cipherleaf only starts KeePassXC if its file matches a checked hash. It is set up to make no network connections, with a 5-minute auto-lock and a clipboard that clears after 20 seconds.
- KeePassXC has its own master password, its own lock and its own database file, `data\passwords.kdbx`. Cipherleaf never sees the password or your entries.
- Passwords do not need security, and security does not need passwords.
- Copies of the encrypted passwords file are kept in `data\backups` (the last 10) when it changed, and you can restore one from Settings ("Restore passwords file…"); the file it replaces is kept as `passwords-undo-....kdbx`.
- Quitting never force-closes KeePassXC. If it is still open (for example asking to save), the quit is cancelled with a message.

### Backups and restore

- **Automatic backup** when the vault locks or the app quits after changes, saved in `data\backups` (the last 10 are kept).
- **Save a backup copy** from Settings, or "Back up vault now" in the `Ctrl+K` menu, saves a copy wherever you choose.
- **Restore** from the lock screen, from Settings, or from the start screens. Cipherleaf checks the backup first and keeps your current file as `restore-undo-....clf`. Nothing is deleted.
- **Start over** sets a vault you can't open aside as `set-aside-....clf` and lets you start new notes. Nothing is deleted, and you can bring the old file back with Restore.
- **Open data folder** from Settings.

### Portable

Everything lives in the `data` folder next to `Cipherleaf.exe`. Copy the folder to move your notes. Only one copy of the app runs at a time. Do not share `data\` with other people: it holds your notes.

## How to use it

### Getting started

1. Unzip the folder anywhere (a normal folder or a USB stick) and run `Cipherleaf.exe`. Keep the `KeePassXC` folder next to it if you want the passwords panel.
2. The first time, Cipherleaf opens straight to your notes. There is no password and nothing to set up.
3. Click **Create your first notebook** (or the `+` next to Notebooks). Give it a name, an icon and a color.
4. Press `Ctrl+N` or "New page" (the sidebar must be showing), click the title at the top and type it, then press Enter to move into the page.

### Writing notes

- Press `/` to insert headings, lists, checklists, tables, pictures, equations and more. Select text to get the formatting bar.
- Use the `...` menu next to a page to add a page inside it, favorite it, move it or send it to Trash.
- Press `Ctrl+K` to search everything, or to run a command.
- Useful keys: `Ctrl+K` search, `Ctrl+N` new page, `Ctrl+,` Settings, `Ctrl+\` show or hide the sidebar, `Ctrl+1` Notes, `Ctrl+2` Passwords (when on), `Ctrl+3` Sticky notes, `Ctrl+L` lock (when security is on).
- Sticky notes: open "Sticky notes" in the sidebar and press "New note". In its top bar, the dot picks the color and the pin switches "Keep on top" off or on. The X closes the note (it stays in the vault) and the bin deletes it.

### Turn security on

1. Open **Settings** and find **Security (optional)**. Press **Turn on…**.
2. Choose a master password (or press "Suggest a strong passphrase" and write it down). Add a keyfile if you want one ("Create new" or "Use existing"). Confirm that you understand there is no password reset.
3. Leave "Delete earlier backups made while security was off" ticked unless you want to keep them. Cipherleaf saves a fresh protected backup first and deletes only automatic backups; your own backup copies, `restore-undo` and `set-aside` files are never deleted.
4. Press **Turn on security**. Your notes get a new key under the new password; this takes a moment, longer for big notebooks.
5. From now on the app locks as described above, and you unlock with the password (and keyfile).

### Turn security off

1. Unlock the vault as usual, open **Settings**, **Security (optional)**, and press **Turn off security…**.
2. Read the warnings, type your current master password (and choose your keyfile if you use one), and confirm.
3. Press **Turn off security**. The notes then open without a password. A wrong password changes nothing and counts toward the wait after wrong passwords.

With security off, your notes are still saved in Cipherleaf's encrypted, tamper-checked format, but the key is a **public key built into the program**. So this is **not** confidentiality: anyone who has your `vault.clf` or a backup, and a copy of Cipherleaf, can read it.

### If you forget the password

There is no reset and no recovery. If you lose the master password, or a keyfile you required, the notes cannot be opened. The way forward is **Start over…** on the lock screen: type START OVER and press "Set aside and start over". Your old vault is set aside (never deleted); then press "Start new notes". If you find the password later, use **Restore from a backup…**, choose the `set-aside-....clf` file in `data\` and type its password. Backups made while security was on open with the password they had then.

### Optional passwords panel

1. In **Settings**, find **Passwords with KeePassXC (optional)** and switch "Passwords" on. A Passwords item appears in the sidebar (`Ctrl+2`).
2. The first time, KeePassXC opens its own start screen. Choose "Create new database" and save it as `passwords.kdbx` in the `data` folder, with its own master password. Cipherleaf never sees that password. (If `data` already holds a `.kdbx` file, KeePassXC opens that.)
3. To switch the panel off, use the same switch. Cipherleaf closes KeePassXC politely first. Your `passwords.kdbx` is not deleted.

### Backups and restore

1. Cipherleaf backs up automatically on lock or quit, after changes. The last 10 are in `data\backups`.
2. For a copy you control, use **Settings, Backups, Save a backup copy…** and keep it somewhere else (another drive is best).
3. To restore, choose **Restore from a backup…** (lock screen, start screens, or Settings), pick a backup from the list or "Choose another backup file…", enter "Password of this backup" if it has one, and press "Replace my vault with this backup" ("Restore backup" on the start screens). Your current vault is kept as `restore-undo-....clf`.
4. A backup made while security was off needs no password. Restoring one onto a vault that has a master password asks you to confirm that security will be turned off.

### Moving or backing up the folder

Close Cipherleaf, then copy the whole folder (`Cipherleaf.exe`, `KeePassXC` and `data`). Moving the folder to another drive or PC does not change the password or the keyfile. A keyfile is identified by its content, so keep the same file, not a copy that has been edited.

### Troubleshooting

- **"Wrong master password or keyfile for this Cipherleaf vault."** This is the Cipherleaf password, not the KeePassXC one. If the vault uses a keyfile, it must be the exact file chosen when the vault was made.
- **"The vault file couldn't be read."** Another program (antivirus or a sync tool) may be using the file. Press "Check again" or "Try again". If it stays, use Restore or Start over.
- **"No notes found here."** The folder has backups but no current notes file. Restore a backup, or start new notes.
- **Windows SmartScreen warns about the exe.** The exe is not code-signed. Choose "More info", then "Run anyway" if you trust where you got it.
- **KeePassXC does not start.** It needs the Microsoft Visual C++ 2015 to 2022 x64 runtime, and its folder must be next to `Cipherleaf.exe`.
- **Quit says KeePassXC is still open.** Finish or close it, then close Cipherleaf again.

## Is it safe?

Short answers. The full list is in [`docs/security-design.md`](docs/security-design.md).

- **Local only, no network code.** Notes are saved only in `data\vault.clf` on your PC. Cipherleaf makes no connections of its own (the page security policy allows only the app's internal channel), and KeePassXC is preset to make none. A web link in a note opens in your browser only when you click it.
- **With security on:** notes are encrypted with XChaCha20-Poly1305, with the key derived from your password by Argon2id (256 MiB, 3 passes), and tampering is detected.
- **With security off, the notes are readable by anyone with the folder.** This is by design, and it is stated in Settings.
- **The exe is not signed.** Windows SmartScreen may warn. Cipherleaf does not check itself, so a replaced exe would not be noticed.
- **No password reset.** See above.
- **The lock screen and screenshots.** The lock screen of a protected vault is hidden from screenshots only when the app starts. After a lock during the session it follows your screenshot setting, which is off by default.
- **Not defended:** malware or a keylogger on the PC, someone with administrator rights, reading memory while the app is open, and a file swapped by someone who can write to your `data` folder (for example replacing a protected vault with an open one).
- **Not proven yet.** Most checks so far are unit tests and scripted runs on throwaway data. USB removal, sleep and long idle behavior have not been checked on a real PC. The owner has not formally accepted any result. See [`docs/acceptance.md`](docs/acceptance.md) for exactly what was checked.

## Credits and licenses

Cipherleaf is built on open source software, and we are grateful to everyone who made it:

- **[KeePassXC](https://keepassxc.org) 2.8.0** (GPL-2.0 or GPL-3.0, (c) the KeePassXC Team) for passwords. It is bundled unmodified and runs as a separate program. Its license and the exact source are in the `Licenses\KeePassXC` folder of the portable copy, and at <https://github.com/keepassxreboot/keepassxc/releases/tag/2.8.0>.
- **[Tauri](https://tauri.app)** and the Rust crates it and Cipherleaf use (RustCrypto for the encryption, SQLite for storage, and others).
- **npm packages** for the interface: React, TipTap and ProseMirror (the editor), KaTeX (equations), highlight.js (code colors), lucide icons, Radix, and others.
- **Fonts:** Inter and JetBrains Mono (SIL Open Font License 1.1) and the KaTeX fonts.
- **Word list:** the EFF large word list for suggested passphrases, (c) Electronic Frontier Foundation, CC BY 3.0 US.
- **Microsoft Edge WebView2** shows the interface (required, not bundled).

Every component, its version, copyright holder and license text is in [`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md), which is also in the `Licenses` folder of the portable copy and inside the app (Settings, About, "Open source licenses…"). This is attribution, not legal advice.

**The license for Cipherleaf itself has not been chosen yet.** Until the owner decides, no license is granted to use, copy or share Cipherleaf's own code.

## Requirements

- Windows 10 or 11, 64-bit, with the Microsoft Edge WebView2 runtime (already part of Windows 11 and most Windows 10 installs).
- The Microsoft Visual C++ 2015 to 2022 x64 runtime, needed by the bundled KeePassXC.
- The window is at least 960 by 640 pixels. Mica and Acrylic window materials need Windows 11. Unlocking a password-protected vault uses about 256 MB of memory for a moment.
