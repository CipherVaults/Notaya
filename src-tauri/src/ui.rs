//! Window creation and window-level effects (backdrop, capture protection).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::model::Sticky;
use crate::portable::Paths;

pub const STICKY_PREFIX: &str = "sticky-";

pub fn build_main(app: &AppHandle, paths: &Paths) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Cipherleaf")
        .inner_size(1280.0, 820.0)
        .min_inner_size(960.0, 640.0)
        .center()
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .data_directory(paths.webview.clone())
        .content_protected(true)
        .disable_drag_drop_handler()
        // Shown by the UI once it has painted, so there is no white flash.
        .visible(false)
        .build()
}

/// Applies the Windows 11 Mica or Acrylic material. Returns false when the
/// effect isn't available (e.g. Windows 10), so the UI can use a solid background.
pub fn apply_backdrop(window: &WebviewWindow, kind: &str, dark: bool) -> bool {
    #[cfg(windows)]
    {
        let _ = window_vibrancy::clear_mica(window);
        let _ = window_vibrancy::clear_acrylic(window);
        match kind {
            "mica" => window_vibrancy::apply_mica(window, Some(dark)).is_ok(),
            "acrylic" => {
                let tint = if dark { (16, 16, 22, 150) } else { (245, 245, 250, 150) };
                window_vibrancy::apply_acrylic(window, Some(tint)).is_ok()
            }
            _ => false,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (window, kind, dark);
        false
    }
}

pub fn set_capture_protection(app: &AppHandle, on: bool) {
    for window in app.webview_windows().values() {
        let _ = window.set_content_protected(on);
    }
}

/// Puts a sticky note back on screen if its saved position is no longer visible
/// (for example after moving the USB stick to a PC with a smaller monitor).
fn visible_position(app: &AppHandle, x: f64, y: f64) -> (f64, f64) {
    let on_screen = app.available_monitors().unwrap_or_default().iter().any(|m| {
        let scale = m.scale_factor();
        let pos = m.position().to_logical::<f64>(scale);
        let size = m.size().to_logical::<f64>(scale);
        x >= pos.x && y >= pos.y && x < pos.x + size.width - 60.0 && y < pos.y + size.height - 60.0
    });
    if on_screen { (x, y) } else { (120.0, 120.0) }
}

pub fn open_sticky(app: &AppHandle, paths: &Paths, sticky: &Sticky, protect: bool) -> tauri::Result<()> {
    let label = format!("{STICKY_PREFIX}{}", sticky.id);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let (x, y) = visible_position(app, sticky.x, sticky.y);
    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("Sticky note")
        .inner_size(sticky.w.clamp(220.0, 1200.0), sticky.h.clamp(180.0, 1200.0))
        .min_inner_size(220.0, 180.0)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(sticky.on_top)
        .skip_taskbar(true)
        .data_directory(paths.webview.clone())
        .content_protected(protect)
        .disable_drag_drop_handler()
        .visible(false)
        .build()?;
    Ok(())
}

pub fn close_sticky(app: &AppHandle, id: &str) {
    if let Some(w) = app.get_webview_window(&format!("{STICKY_PREFIX}{id}")) {
        let _ = w.destroy();
    }
}

pub fn close_stickies(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with(STICKY_PREFIX) {
            let _ = window.destroy();
        }
    }
}
