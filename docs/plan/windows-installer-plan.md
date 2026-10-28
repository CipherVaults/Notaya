# Windows installer plan for Notaya (proposal)

- **Date:** written 2026-10-27 about 09:35 UTC; re-checked against `main` `a4574be` (Notaya V1.2.0) on 2026-10-28 at 11:20 to 11:40 UTC (system clock). The app was called Cipherleaf when this was first written; it is Notaya now ([A28](A28-rename-notaya-icon-sunrise.md)).
- **Asked by:** Mohamed, 2026-10-27 09:29 UTC ("Plan an installer setup in the windows so that people can do setup") and 09:30:09 UTC ("Planning now, let's discuss"); and again 2026-10-28 11:19:57 UTC ("Now continue the setup part"), which continues the planning, it is not a go to build.
- **State:** all 10 decisions answered (the recommended ones, 2026-10-28 11:56:21 UTC). **The owner said go** on 2026-10-28 at 12:24:16 UTC ("Ok for all go now"), with the OK to download the NSIS tool on his PC. **Built in the cloud, not yet tested on Windows:** see "What was built" below and [A30](approval-and-amendments.md), [EX-2026-10-28-33](../execution/2026-10-28-windows-installer.md). Not released, not accepted. Result state: *produced* (this document), not verified by any build and not accepted. It is not an amendment: the owner decides the questions at the end, and building starts only when he says so.

## What changed since the first version (2026-10-27 to 2026-10-28)

- The app is **Notaya** (A28). The identifier `app.cipherleaf.vault`, `data\vault.clf`, the vault format and `CIPHERLEAF_HOME` are kept, so old vaults open unchanged. `tauri.conf.json` now has `productName` and `mainBinaryName` Notaya, and `bundle.publisher` Notaya, but `bundle.active` is still `false`.
- **[Notaya V1.2.0](https://github.com/AbyssWatcher2/CipherLeaf/releases/tag/v1.2.0) is released** (the owner published it 2026-10-27 22:28:04 UTC): one portable ZIP, `Notaya-V1.2.0-portable.zip`, 5.9 MB. So the installer is **not** in V1.2.0 and would be the next release.
- `src-tauri/src/portable.rs` is unchanged: a release build still always uses the folder of the exe. The one real code change below is still needed.
- Installed notes would now go in **`%LOCALAPPDATA%\Notaya Notes\data`** (see the change of 2026-10-28 below), not `Cipherleaf`. There is no old installed Cipherleaf folder to worry about, because no installer ever existed.
- The "download WebView2 only if missing" choice does not touch the paused "keep app, no network" plan (still paused).
- **One decision is answered:** each release ships **both** a Windows installer and the portable ZIP (decision 9, owner 2026-10-28 11:21:06 UTC). It also means the plan to build an installer is wanted in principle; the other 9 were answered "All recommendations" (owner, 2026-10-28 11:56:21 UTC). **All 10 decisions are now answered, but no build is approved yet:** the owner has not said go, and the NSIS tooling download still needs his OK.

## Where we start

- Notaya ships as a portable ZIP (V1.2.0, 5.9 MB). The exe keeps everything in a `data` folder next to itself: `data\vault.clf`, `data\backups` and the WebView2 profile in `data\webview` (`src-tauri/src/portable.rs`).
- Installer building is switched off (`"bundle": { "active": false }` in `src-tauri/tauri.conf.json`). Tauri 2 can make a setup.exe (NSIS) and an MSI from the same project.
- **The one real code change:** an installed copy lives in a program folder where it should not write its data. The app must tell "portable" from "installed" and choose its data folder from that. Everything else is installer settings.

## Recommendation

A **per-user setup.exe (NSIS)**, with the **portable ZIP** kept beside it in each release. Installed notes go in **`%LOCALAPPDATA%\Notaya Notes\data`** (it was first written as `%LOCALAPPDATA%\Notaya`, which is where the setup puts the program; see "What was built"). Uninstall never deletes notes. On first start the app offers a one-time **"Import notes from a portable copy"**. WebView2 is downloaded **only if missing**. The setup ships **unsigned** for now, with a short "More info, Run anyway" note.

## The choices

### 1. Installer type: setup.exe (NSIS) or MSI

- **setup.exe (NSIS), recommended.** About the size of the ZIP. Installs without admin rights. You upgrade by running the new setup over the old one. Tauri recommends it for ordinary users.
- **MSI.** Made for companies that deploy software with IT tools. It needs admin rights, installs for all users only, and needs the WiX toolset on the build PC.

### 2. Per-user or all-users

- **Per-user, recommended.** No admin prompt. Installs under the user's `%LOCALAPPDATA%`. Each Windows user has their own copy and their own notes.
- **All-users.** Needs admin rights and installs to Program Files. Useful only on shared PCs. Tauri can let the person choose ("both"), but that adds a screen and more test cases.

### 3. Where notes live

- **Portable (ZIP):** no change. `data` stays next to the exe, so use from a USB stick keeps working.
- **Installed:** `%LOCALAPPDATA%\Notaya Notes\data\` (first proposed as `...\Notaya\data\`, changed on 2026-10-28, see "What was built") holds the vault, the backups and the WebView2 profile. Recommended because it is local, is not synced by roaming profiles, and is not inside a Documents folder that OneDrive redirects.
- **How the app tells them apart:** the ZIP gets a small marker file `portable` next to the exe. An existing `data` folder next to the exe also means portable, so V1.0 to V1.1 ZIP copies keep working unchanged. If there is no marker and no `data` folder, the copy is installed. `CIPHERLEAF_HOME` still overrides both (tests).
- **Trap to avoid:** Tauri's uninstaller has a "Delete the application data" checkbox that wipes the folders named after the app identifier (`app.cipherleaf.vault`). A folder named `Notaya` keeps notes out of its reach. The identifier must also stay as it is: changing it would make a later setup treat the old install as a different app.
- Settings or About shows the data folder and an "Open data folder" button, so people can find their backups.

### 4. Moving from the ZIP to the installed app

- **Recommended:** when an installed copy starts with no vault, it offers "Start new notes" or "Import notes from a portable copy…". Import **copies** the chosen `data\vault.clf` and its backups as they are, so the password doesn't change. It never moves or deletes the original.
- **Simpler alternative:** no button. The README tells people to copy their `data` folder to `%LOCALAPPDATA%\Notaya Notes\`.

### 5. WebView2

The ZIP already needs WebView2. It comes with Windows 11 and almost every updated Windows 10 PC.

- **Download bootstrapper, recommended.** The setup checks for WebView2 and downloads it only if it is missing. It adds nothing to the size. This is the only time the installer goes online, and only on PCs that lack WebView2.
- **Bundled offline installer.** Never uses the network, but the setup grows by about 130 MB.
- **Skip.** No check is made. If WebView2 is missing, the app shows an error.
- The paused "keep app, no network" plan is about the running app. This choice only affects installing on a PC that lacks WebView2.

### 6. Shortcuts

- A Start menu shortcut is always created. A desktop shortcut is a checkbox, ticked by default. "Launch Notaya" appears at the end of setup.

### 7. Uninstall

- **Recommended:** remove the program only and always keep notes and backups. The uninstaller's last screen says where the notes are.
- Alternative: an unticked "Also delete my notes" box. Not recommended, because one wrong tick destroys a vault.

### 8. Upgrades

- People download the new setup.exe and run it. It replaces the program and leaves the notes alone. The setup asks for the app to be closed first.
- **No auto-updater.** Tauri's updater contacts the internet and needs its own signing keys, which goes against the no-network direction. This can be revisited later.
- The rule stays that a newer app opens older vaults.

### 9. SmartScreen and code signing

- Windows shows "Windows protected your PC" for an unsigned setup or exe. People click **More info**, then **Run anyway**. Some antivirus tools are also stricter with unsigned installers.
- **Recommended for now: unsigned**, the same as the ZIP today. The release notes explain the click.
- Later options: Microsoft's Azure Trusted Signing (monthly fee, an identity check, and eligibility that depends on the country) or a classic code-signing certificate (yearly fee, hardware token). Neither one removes the warning right away, because reputation builds up with downloads.

### 10. The portable ZIP

- **Keep it, recommended.** Each release has `Notaya-Setup-x.y.z.exe` and `Notaya-Vx.y.z-portable.zip` (the ZIP is named like `Notaya-V1.2.0-portable.zip` today). The ZIP stays the choice for a USB stick or for leaving nothing behind on the PC.

## What was built (2026-10-28, after the owner's go)

- **One deviation from decision 3, for safety.** The Tauri setup installs the program for the current user into `%LOCALAPPDATA%\Notaya` (read from the bundler's installer template, `tauri-bundler` 2.10.4, line `StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"`). Notes in `%LOCALAPPDATA%\Notaya\data` would sit inside the program folder: after the first start a `data` folder next to the exe would also make the app think it is portable, and the uninstaller works in that folder. So the installed notes folder is **`%LOCALAPPDATA%\Notaya Notes\data`**: not the program folder, and not `%LOCALAPPDATA%\app.cipherleaf.vault`, which the uninstaller's "Delete the application data" box removes. The owner's recommended answer to decision 3 named `%LOCALAPPDATA%\Notaya`; this is the same idea with a name that cannot clash. It is for the owner to confirm.
- **Marker file is `portable.txt`** (not `portable`), a small text file that explains itself. The rule in `src-tauri/src/portable.rs`: `portable.txt` or an existing `data` folder next to the exe means portable; otherwise installed (needs `LOCALAPPDATA`, else it stays next to the exe). `CIPHERLEAF_HOME` still wins (tests).
- **First screen of an installed copy** with no notes: "Welcome to Notaya", **Start new notes** or **Import notes from a portable copy…** (a new `src-tauri/src/import.rs`, command `import_portable_notes`; copy only, details in P22 of `docs/security-design.md`). Earlier, a first start made an empty vault at once; an installed copy now asks first, a portable copy still does not.
- **Setup settings** in `src-tauri/tauri.conf.json`: NSIS only, per-user install, WebView2 downloaded only if missing (silent), `allowDowngrades: false`, installer icon, and `packaging/installer-hooks.nsh` (after an interactive uninstall: "Your notes were NOT deleted. They are still in ..."). The desktop shortcut is the tick box on the setup's last page (ticked by default, read from the template).
- **Not built, on purpose:** an auto-updater; code signing; MSI; a licence page (no licence chosen).
- **Build script** `tools/windows-build/build-windows.ps1` makes the setup, the portable folder (with `portable.txt`) and the ZIP and prints their SHA-256. **Test runner** `tools/real-exe-tests/run-installed.ps1` (the three older runners now copy the exe with a `portable.txt` so they keep using a temp data folder).
- **File names:** `Notaya-V<version>-setup.exe` and `Notaya-V<version>-portable.zip`.
- **Checks so far (cloud):** `tsc`, `npm run build`, 145 Rust unit tests under Wine (16 new). **Not yet run:** the real exe, the NSIS build, installing, upgrading and uninstalling, the folder dialog of the import, SmartScreen.

## The original build list (kept for reference)

1. The data-folder choice in `portable.rs` (the marker or an existing `data` folder means next to the exe; otherwise `%LOCALAPPDATA%\Notaya Notes`), with unit tests.
2. The first-start import screen (copy only) and "Open data folder" in Settings.
3. `tauri.conf.json`: turn on the NSIS bundle in per-user mode, and set the WebView2 mode, shortcuts, installer icon and texts. Keep the notes out of the uninstaller's delete path.
4. A build script that makes the setup.exe and the ZIP (with the `portable` marker) and prints their SHA-256.
5. Records: `security-design.md` (the new data location), an install section in the README, an amendment entry and an execution summary.
6. Tests on the PC with throwaway data only. Install, start, make a new vault, import from a copy of a portable folder, upgrade by running a newer setup, uninstall and check that the notes remain, and check that the portable ZIP still uses its own `data`.

Building on the PC downloads the NSIS tooling the first time (Tauri does this). That counts as installing a tool, so it needs the owner's OK.

## Decisions for the owner

| # | Question | Recommended | Owner's answer |
|---|---|---|---|
| 1 | setup.exe (NSIS), MSI, or both? | setup.exe only | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 2 | Per-user, all-users, or let the person choose? | Per-user | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 3 | Installed notes folder | `%LOCALAPPDATA%\Notaya` | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". **Built as `%LOCALAPPDATA%\Notaya Notes`** because the setup puts the program in `%LOCALAPPDATA%\Notaya`; to be confirmed by the owner. |
| 4 | First-start "Import from portable copy" button, or README steps only? | Button (copy only) | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 5 | WebView2: download if missing, bundle offline (+~130 MB), or skip? | Download if missing | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 6 | Uninstall: always keep notes, or offer a delete box? | Always keep | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 7 | Desktop shortcut ticked by default? | Yes | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 8 | Code signing now, or unsigned for now? | Unsigned for now | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
| 9 | Keep the portable ZIP in each release? | Yes | **Decided: yes.** Owner, 2026-10-28 11:21:06 UTC: "So we will do 2 things windows installer and portable version in the release". Each release ships both. |
| 10 | Which version ships the installer? | V1.3.0 (V1.2.0 is already released) | **Decided: recommended.** Owner, 2026-10-28 11:56:21 UTC: "All recommendations". |
