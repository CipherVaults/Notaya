# A31: Tray, start with Windows, and no "Delete the application data" box (V1.4.0)

**Status:** applied on the owner's instruction (2026-10-28 17:22:17 to 17:23:31 UTC, see the authority rows); code in draft [PR #39](https://github.com/AbyssWatcher2/Notaya/pull/39). Built and tested in the cloud only so far (see [EX-2026-10-28-37](../execution/2026-10-28-v1-3-0-settings-text-rerelease.md)); not yet built or run on the PC; not accepted.

## What the owner asked for

Times are UTC on 2026-10-28, from the project transcript.

> "also i want notaya to be runed all Time in windows / i saw when i close it fully close i want like a minimized or something" (17:22:17, with a screenshot of the Windows tray)

> "start with windows option / start minimized with windows option / close to tray or minimzed will be defult" (17:23:31, edited 17:23:48)

Also from the same hour: the two Settings lines about Cipherleaf and KeePassXC go (17:13:59), and the uninstaller's "Delete the application data" box goes (17:20:47, with two screenshots: the box was ticked or not and the same "notes were NOT deleted" message followed). The version is V1.4.0 ("sorry v1.4", 17:21:07). V1.3.0 is kept as it is.

## What changed

- **Close to the tray, on by default.** The close button hides Notaya to a tray icon (menu: Open Notaya, Quit Notaya; a left click opens). **A vault with a master password is locked before the window is hidden** (same lock as the Lock button); with security off the windows save and a backup is made if anything changed. If no tray icon can be made, the close button quits as before. The Quit button and "Quit Notaya" still lock, back up and exit.
- **Settings > Windows:** "Close button hides Notaya to the tray", "Start with Windows" (off by default; a per-user `Run` value named `Notaya`, `HKCU`, no administrator rights) and "Start minimized to the tray" (adds `--minimized` to that value). The two close/minimized switches are kept in a plain file `shell-prefs.json` next to the vault (they must be readable while locked and hold no notes data); start with Windows is read from the registry.
- **Uninstaller:** the stock "Delete the application data" check box is removed (a copy of tauri-bundler 2.10.4's template in `packaging/installer-template.nsi`, without the box). It only ever cleaned the WebView2 browser folders, never the notes, so it offered a choice that did nothing for them. The uninstaller still removes the program and shortcuts, keeps the notes, and (on a real uninstall) removes the `Run` value.
- **Settings text:** the two "Good to know" bullets about the Cipherleaf name and KeePassXC are removed.
- **Version** 1.4.0.

## Kept on purpose

- The vault format, the app identifier `app.cipherleaf.vault`, `CIPHERLEAF_HOME` and every other internal name.
- No change to encryption, auto-lock timing, screenshot protection or backups beyond the lock/backup on hide described above (security promise P23, [security-design.md](../security-design.md)).

## Things to know

- Windows often puts a new tray icon behind the arrow next to the clock; dragging it out keeps it visible.
- A portable copy's start-up entry names the folder it is in; if the folder moves, turn the switch off and on again.
- The setup closes a running Notaya (in the tray too) before it updates it.
