//! Window-shell behaviour that must work while the vault is locked: the system tray icon,
//! "close hides to the tray", and "start with Windows".
//!
//! These preferences are not notes data, so they are kept in a small plain file next to the
//! vault (`shell-prefs.json`) and not inside the encrypted settings: the close handler and the
//! start-up code need them before anyone has unlocked anything. The file holds two on/off
//! switches and nothing else. "Start with Windows" itself lives in the Windows registry (the
//! per-user `Run` key), not in the file.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::error::{Error, Result};
use crate::portable::Paths;
use crate::state;

/// The name of the Run-key value. It is also the product name, so the uninstaller's own
/// clean-up of "Software\Microsoft\Windows\CurrentVersion\Run" (stock template) removes it.
pub const AUTOSTART_VALUE: &str = "Notaya";
/// Added to the Run command when Notaya should start hidden in the tray.
pub const MINIMIZED_ARG: &str = "--minimized";

const PREFS_FILE: &str = "shell-prefs.json";

/// The two switches kept in `shell-prefs.json`. Both are on by default, so a copy that has no
/// file yet hides to the tray when closed. `start_minimized` only matters when Windows starts
/// Notaya (it decides whether the Run command carries `--minimized`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    pub close_to_tray: bool,
    pub start_minimized: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { close_to_tray: true, start_minimized: true }
    }
}

fn prefs_path(paths: &Paths) -> PathBuf {
    paths.data.join(PREFS_FILE)
}

/// Reads the switches. A missing or unreadable file means the defaults.
pub fn load_prefs(paths: &Paths) -> Prefs {
    std::fs::read_to_string(prefs_path(paths))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes the switches (to a temporary file first, then renamed over the old one).
pub fn save_prefs(paths: &Paths, prefs: &Prefs) -> Result<()> {
    let path = prefs_path(paths);
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(prefs).map_err(|e| Error::Other(e.to_string()))?;
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// The command Windows runs at sign-in: the quoted path of this exe, plus `--minimized` when
/// Notaya should start hidden in the tray.
pub fn run_command(exe: &Path, minimized: bool) -> String {
    let mut command = format!("\"{}\"", exe.display());
    if minimized {
        command.push(' ');
        command.push_str(MINIMIZED_ARG);
    }
    command
}

/// True if a Run value starts exactly this exe (with or without `--minimized`). A value that
/// starts another copy of Notaya does not count, so a copy never reports or removes another
/// copy's entry.
pub fn value_is_for(value: &str, exe: &Path) -> bool {
    let value = value.trim();
    value == run_command(exe, false) || value == run_command(exe, true)
}

/// Whether Windows is set to start this exe at sign-in.
pub fn autostart_enabled(exe: &Path) -> bool {
    registry::read(AUTOSTART_VALUE).is_some_and(|v| value_is_for(&v, exe))
}

/// Turns "start with Windows" on (replacing any older entry) or off. Turning it off removes
/// the entry only if it starts this exe.
pub fn set_autostart(exe: &Path, on: bool, minimized: bool) -> Result<()> {
    if on {
        registry::write(AUTOSTART_VALUE, &run_command(exe, minimized)).map_err(Error::Other)
    } else if autostart_enabled(exe) {
        registry::delete(AUTOSTART_VALUE).map_err(Error::Other)
    } else {
        Ok(())
    }
}

/// True when Windows started Notaya hidden (the Run command carries `--minimized`).
pub fn started_minimized() -> bool {
    std::env::args().skip(1).any(|a| a == MINIMIZED_ARG)
}

/// Brings the main window to the front (from the tray, or from a second start).
pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Puts the Notaya icon in the notification area, with "Open Notaya" and "Quit Notaya".
/// A left click opens the window. Fails if the system gives no tray (then the app does not
/// hide itself anywhere it could not be reached again).
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Notaya", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Notaya", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Notaya")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "quit" => {
                // Saves edits, locks, makes the backup and exits, on a thread of its own so the
                // windows can answer (like the Quit button and the close button do).
                let app = app.clone();
                std::thread::spawn(move || {
                    state::quit(&app);
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg(windows)]
mod registry {
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, REG_VALUE_TYPE,
    };

    const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Opens the per-user Run key, runs `f`, closes the key.
    fn with_key<T>(access: windows::Win32::System::Registry::REG_SAM_FLAGS, f: impl FnOnce(HKEY) -> Result<T, String>) -> Result<T, String> {
        let mut key = HKEY::default();
        // SAFETY: `key` is a valid out pointer and RUN_KEY is a NUL-terminated wide string.
        let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, access, &mut key) };
        if status != ERROR_SUCCESS {
            return Err(format!("Windows refused to open the start-up list (code {}).", status.0));
        }
        let result = f(key);
        // SAFETY: `key` was opened above and is closed once.
        unsafe {
            let _ = RegCloseKey(key);
        }
        result
    }

    pub fn read(name: &str) -> Option<String> {
        let name = wide(name);
        with_key(KEY_QUERY_VALUE, |key| {
            let mut kind = REG_VALUE_TYPE::default();
            let mut size: u32 = 0;
            // SAFETY: first call only asks for the size; the pointers are valid for the call.
            let status = unsafe {
                RegQueryValueExW(key, PCWSTR(name.as_ptr()), None, Some(&mut kind), None, Some(&mut size))
            };
            if status != ERROR_SUCCESS || kind != REG_SZ || size == 0 {
                return Err(String::new());
            }
            let mut buf = vec![0u16; (size as usize).div_ceil(2)];
            let mut size_bytes = (buf.len() * 2) as u32;
            // SAFETY: `buf` holds `size_bytes` writable bytes.
            let status = unsafe {
                RegQueryValueExW(
                    key,
                    PCWSTR(name.as_ptr()),
                    None,
                    Some(&mut kind),
                    Some(buf.as_mut_ptr() as *mut u8),
                    Some(&mut size_bytes),
                )
            };
            if status != ERROR_SUCCESS {
                return Err(String::new());
            }
            let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
            Ok(String::from_utf16_lossy(&buf[..len]))
        })
        .ok()
    }

    pub fn write(name: &str, value: &str) -> Result<(), String> {
        let name = wide(name);
        let data = wide(value);
        // SAFETY (below): the bytes are the UTF-16 text with its terminating NUL, as REG_SZ wants.
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) };
        with_key(KEY_SET_VALUE, |key| {
            let status = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes)) };
            if status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(format!("Windows could not save the start-up entry (code {}).", status.0))
            }
        })
    }

    pub fn delete(name: &str) -> Result<(), String> {
        let name = wide(name);
        with_key(KEY_SET_VALUE, |key| {
            // SAFETY: valid key and NUL-terminated name.
            let status = unsafe { RegDeleteValueW(key, PCWSTR(name.as_ptr())) };
            if status == ERROR_SUCCESS || status.0 == 2 {
                // 2 = ERROR_FILE_NOT_FOUND: already gone.
                Ok(())
            } else {
                Err(format!("Windows could not remove the start-up entry (code {}).", status.0))
            }
        })
    }
}

#[cfg(not(windows))]
mod registry {
    pub fn read(_name: &str) -> Option<String> {
        None
    }
    pub fn write(_name: &str, _value: &str) -> Result<(), String> {
        Err("Start with Windows is only available on Windows.".into())
    }
    pub fn delete(_name: &str) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths() -> Paths {
        let root = std::env::temp_dir().join(format!("cipherleaf-shell-{}", crate::model::new_id()));
        let paths = Paths::at(&root);
        paths.ensure_dirs().unwrap();
        paths
    }

    #[test]
    fn without_a_file_both_switches_are_on() {
        let paths = temp_paths();
        assert_eq!(load_prefs(&paths), Prefs { close_to_tray: true, start_minimized: true });
    }

    #[test]
    fn saved_switches_are_read_back() {
        let paths = temp_paths();
        let prefs = Prefs { close_to_tray: false, start_minimized: true };
        save_prefs(&paths, &prefs).unwrap();
        assert_eq!(load_prefs(&paths), prefs);
        let again = Prefs { close_to_tray: true, start_minimized: false };
        save_prefs(&paths, &again).unwrap();
        assert_eq!(load_prefs(&paths), again);
        assert!(!prefs_path(&paths).with_extension("json.tmp").exists(), "no temporary file is left");
    }

    #[test]
    fn a_damaged_file_or_an_empty_object_gives_the_defaults() {
        let paths = temp_paths();
        std::fs::write(prefs_path(&paths), "{not json").unwrap();
        assert_eq!(load_prefs(&paths), Prefs::default());
        std::fs::write(prefs_path(&paths), "{}").unwrap();
        assert_eq!(load_prefs(&paths), Prefs::default());
        std::fs::write(prefs_path(&paths), r#"{"closeToTray":false}"#).unwrap();
        assert_eq!(load_prefs(&paths), Prefs { close_to_tray: false, start_minimized: true });
    }

    #[test]
    fn the_run_command_quotes_the_exe_and_adds_the_flag_only_when_asked() {
        let exe = Path::new(r"C:\Users\A B\AppData\Local\Notaya\Notaya.exe");
        assert_eq!(run_command(exe, false), r#""C:\Users\A B\AppData\Local\Notaya\Notaya.exe""#);
        assert_eq!(run_command(exe, true), r#""C:\Users\A B\AppData\Local\Notaya\Notaya.exe" --minimized"#);
    }

    #[test]
    fn a_run_value_counts_only_for_the_same_exe() {
        let here = Path::new(r"C:\Apps\Notaya\Notaya.exe");
        let other = Path::new(r"D:\Portable\Notaya\Notaya.exe");
        assert!(value_is_for(&run_command(here, false), here));
        assert!(value_is_for(&run_command(here, true), here));
        assert!(value_is_for(&format!("  {}  ", run_command(here, true)), here));
        assert!(!value_is_for(&run_command(other, true), here));
        assert!(!value_is_for("", here));
        assert!(!value_is_for(r#""C:\Apps\Notaya\Notaya.exe" --other"#, here));
    }

    #[cfg(windows)]
    #[test]
    fn the_registry_value_can_be_written_read_and_removed() {
        // A value name of its own, so a real "Notaya" entry on the test PC is never touched.
        let name = format!("NotayaTest{}", crate::model::new_id());
        assert_eq!(registry::read(&name), None);
        registry::write(&name, r#""C:\x y\Notaya.exe" --minimized"#).unwrap();
        assert_eq!(registry::read(&name).as_deref(), Some(r#""C:\x y\Notaya.exe" --minimized"#));
        registry::delete(&name).unwrap();
        assert_eq!(registry::read(&name), None);
        registry::delete(&name).unwrap(); // already gone is not an error
    }
}
