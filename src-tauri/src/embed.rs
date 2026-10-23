//! Shows the bundled KeePassXC inside Cipherleaf's window (amendment A10).
//!
//! A web view is always drawn on top of any child window, so KeePassXC can't be made a
//! child. Instead its window is turned into a borderless window *owned by* Cipherleaf's
//! window and kept exactly over the Passwords area. Windows then moves, minimizes and
//! stacks it together with Cipherleaf, and it looks like part of the app.
//!
//! KeePassXC stays a separate program. Only window handles and rectangles cross this
//! boundary: no passwords, no entries.

use std::path::Path;
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use std::sync::Mutex;

use serde::Deserialize;

use crate::error::{Error, Result};

/// A rectangle in physical pixels, relative to the top-left of Cipherleaf's window.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    /// Keeps a rectangle sane: never negative or absurdly large.
    pub fn clamped(self) -> Rect {
        Rect { x: self.x.clamp(-20_000, 20_000), y: self.y.clamp(-20_000, 20_000), w: self.w.clamp(1, 20_000), h: self.h.clamp(1, 20_000) }
    }
}

/// The window handle of the embedded KeePassXC, or 0 when none is embedded.
static EMBEDDED: AtomicIsize = AtomicIsize::new(0);
/// Cipherleaf's window, the owner of the embedded one.
static PARENT: AtomicIsize = AtomicIsize::new(0);
static ORIGINAL_STYLE: AtomicU32 = AtomicU32::new(0);
static ORIGINAL_EX_STYLE: AtomicU32 = AtomicU32::new(0);
/// Where the panel last asked KeePassXC to be, so it can follow window moves.
static LAST_RECT: Mutex<Option<Rect>> = Mutex::new(None);
/// False while the window is intentionally hidden (leaving the panel, opening the palette).
static SHOWN: AtomicIsize = AtomicIsize::new(0);

pub fn embedded() -> Option<isize> {
    let h = EMBEDDED.load(Ordering::SeqCst);
    if h != 0 && imp::is_window(h) {
        Some(h)
    } else {
        EMBEDDED.store(0, Ordering::SeqCst);
        None
    }
}

/// Finds the main window of the (already running) KeePassXC started from `exe`.
pub fn find_main_window(exe: &Path) -> Option<isize> {
    imp::find_main_window(exe)
}

/// Makes `child` a borderless window owned by `parent` and puts it at `rect`.
pub fn attach(parent: isize, child: isize, rect: Rect) -> Result<()> {
    let rect = rect.clamped();
    let (style, ex) = imp::attach(parent, child, rect)?;
    ORIGINAL_STYLE.store(style, Ordering::SeqCst);
    ORIGINAL_EX_STYLE.store(ex, Ordering::SeqCst);
    PARENT.store(parent, Ordering::SeqCst);
    EMBEDDED.store(child, Ordering::SeqCst);
    SHOWN.store(1, Ordering::SeqCst);
    *LAST_RECT.lock().unwrap_or_else(|e| e.into_inner()) = Some(rect);
    Ok(())
}

/// Moves and resizes the embedded window; shows it if it was hidden.
pub fn place(rect: Rect) -> Result<()> {
    let rect = rect.clamped();
    *LAST_RECT.lock().unwrap_or_else(|e| e.into_inner()) = Some(rect);
    SHOWN.store(1, Ordering::SeqCst);
    match embedded() {
        Some(h) => imp::place(PARENT.load(Ordering::SeqCst), h, rect),
        None => Err(Error::invalid("KeePassXC isn't embedded.")),
    }
}

/// Re-places the window after Cipherleaf's window moved or was resized.
pub fn follow() {
    if SHOWN.load(Ordering::SeqCst) == 0 {
        return;
    }
    let rect = *LAST_RECT.lock().unwrap_or_else(|e| e.into_inner());
    if let (Some(h), Some(rect)) = (embedded(), rect) {
        let _ = imp::place(PARENT.load(Ordering::SeqCst), h, rect);
    }
}

pub fn hide() {
    SHOWN.store(0, Ordering::SeqCst);
    if let Some(h) = embedded() {
        imp::hide(h);
    }
}

/// Turns the embedded window back into a normal top-level KeePassXC window.
pub fn detach() -> Result<()> {
    let Some(h) = embedded() else { return Ok(()) };
    imp::detach(h, ORIGINAL_STYLE.load(Ordering::SeqCst), ORIGINAL_EX_STYLE.load(Ordering::SeqCst))?;
    EMBEDDED.store(0, Ordering::SeqCst);
    SHOWN.store(0, Ordering::SeqCst);
    Ok(())
}

/// Asks the embedded window to close, like clicking its close button.
pub fn close() {
    if let Some(h) = embedded() {
        imp::close(h);
    }
}

/// Asks any window to close, like clicking its close button (for a KeePassXC
/// window that is not the embedded one, found with [`find_main_window`]).
pub fn close_window(h: isize) {
    imp::close(h);
}

// Window style bits (see the Win32 documentation).
const WS_POPUP: u32 = 0x8000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_CLIPCHILDREN: u32 = 0x0200_0000;
const WS_CAPTION: u32 = 0x00C0_0000;
const WS_THICKFRAME: u32 = 0x0004_0000;
const WS_SYSMENU: u32 = 0x0008_0000;
const WS_MINIMIZEBOX: u32 = 0x0002_0000;
const WS_MAXIMIZEBOX: u32 = 0x0001_0000;
const WS_EX_APPWINDOW: u32 = 0x0004_0000;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_WINDOWEDGE: u32 = 0x0000_0100;
const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
const WS_EX_DLGMODALFRAME: u32 = 0x0000_0001;
const WS_EX_STATICEDGE: u32 = 0x0002_0000;

/// A borderless window: no caption, frame, system menu or buttons.
pub fn overlay_style(style: u32) -> u32 {
    (style & !(WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX)) | WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN
}

/// No taskbar button or Alt+Tab entry, and no window edges.
pub fn overlay_ex_style(ex: u32) -> u32 {
    (ex & !(WS_EX_APPWINDOW | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_DLGMODALFRAME | WS_EX_STATICEDGE)) | WS_EX_TOOLWINDOW
}

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use std::path::Path;

    use windows::core::{BOOL, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::ClientToScreen;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsWindow, IsWindowVisible, PostMessageW,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, GWLP_HWNDPARENT, GWL_EXSTYLE, GWL_STYLE, GW_OWNER, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, WM_CLOSE,
    };

    use super::{overlay_ex_style, overlay_style, Rect};
    use crate::error::{Error, Result};

    fn hwnd(raw: isize) -> HWND {
        HWND(raw as *mut c_void)
    }

    pub fn is_window(raw: isize) -> bool {
        unsafe { IsWindow(Some(hwnd(raw))).as_bool() }
    }

    fn image_path(pid: u32) -> Option<String> {
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = vec![0u16; 1024];
            let mut len = buf.len() as u32;
            let r = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
            let _ = CloseHandle(h);
            r.ok()?;
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    struct Search {
        exe: String,
        found: isize,
        area: i64,
    }

    unsafe extern "system" fn enum_proc(h: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut Search) };
        unsafe {
            if !IsWindowVisible(h).as_bool() {
                return BOOL(1);
            }
            // Skip dialogs: they're owned by the main window. (Our own embedded
            // window is owned by Cipherleaf, so it is found by its saved handle instead.)
            if matches!(GetWindow(h, GW_OWNER), Ok(o) if !o.0.is_null()) {
                return BOOL(1);
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(h, Some(&mut pid));
            let same = image_path(pid).map(|p| p.to_lowercase() == search.exe).unwrap_or(false);
            if !same {
                return BOOL(1);
            }
            let mut r = RECT::default();
            if GetWindowRect(h, &mut r).is_ok() {
                let area = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
                if area > search.area {
                    search.area = area;
                    search.found = h.0 as isize;
                }
            }
        }
        BOOL(1)
    }

    pub fn find_main_window(exe: &Path) -> Option<isize> {
        let canon = std::fs::canonicalize(exe).ok()?;
        let text = canon.to_string_lossy().to_string();
        let normal = text.strip_prefix(r"\\?\").unwrap_or(&text).to_lowercase();
        let mut search = Search { exe: normal, found: 0, area: 0 };
        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut search as *mut Search as isize));
        }
        (search.found != 0).then_some(search.found)
    }

    /// Converts a rectangle relative to the parent's client area into screen pixels.
    fn to_screen(parent: isize, rect: Rect) -> Rect {
        let mut origin = POINT { x: rect.x, y: rect.y };
        unsafe {
            let _ = ClientToScreen(hwnd(parent), &mut origin);
        }
        Rect { x: origin.x, y: origin.y, w: rect.w, h: rect.h }
    }

    pub fn attach(parent: isize, child: isize, rect: Rect) -> Result<(u32, u32)> {
        let (p, c) = (hwnd(parent), hwnd(child));
        let (style, ex) = unsafe { (GetWindowLongPtrW(c, GWL_STYLE) as u32, GetWindowLongPtrW(c, GWL_EXSTYLE) as u32) };
        unsafe {
            SetWindowLongPtrW(c, GWL_STYLE, overlay_style(style) as isize);
            SetWindowLongPtrW(c, GWL_EXSTYLE, overlay_ex_style(ex) as isize);
            // Setting the owner makes Windows keep the window above Cipherleaf's and move
            // and minimize it together with it.
            SetWindowLongPtrW(c, GWLP_HWNDPARENT, p.0 as isize);
            let owned = matches!(GetWindow(c, GW_OWNER), Ok(o) if o.0 == p.0);
            if !owned {
                // Put the window back exactly as it was, so a failure leaves nothing half-converted.
                SetWindowLongPtrW(c, GWL_STYLE, style as isize);
                SetWindowLongPtrW(c, GWL_EXSTYLE, ex as isize);
                let _ = SetWindowPos(c, None, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
                return Err(Error::Other("Windows didn't allow KeePassXC to be placed inside Cipherleaf.".into()));
            }
        }
        place(parent, child, rect)?;
        Ok((style, ex))
    }

    pub fn place(parent: isize, child: isize, rect: Rect) -> Result<()> {
        let s = to_screen(parent, rect);
        unsafe {
            SetWindowPos(hwnd(child), None, s.x, s.y, s.w, s.h, SWP_FRAMECHANGED | SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOZORDER)
                .map_err(|e| Error::Other(format!("Couldn't place KeePassXC: {e}")))
        }
    }

    pub fn hide(child: isize) {
        unsafe {
            let _ = ShowWindow(hwnd(child), SW_HIDE);
        }
    }

    pub fn detach(child: isize, style: u32, ex: u32) -> Result<()> {
        let c = hwnd(child);
        unsafe {
            SetWindowLongPtrW(c, GWLP_HWNDPARENT, 0);
            SetWindowLongPtrW(c, GWL_STYLE, style as isize);
            SetWindowLongPtrW(c, GWL_EXSTYLE, ex as isize);
            SetWindowPos(c, None, 120, 120, 1100, 720, SWP_FRAMECHANGED | SWP_SHOWWINDOW | SWP_NOZORDER)
                .map_err(|e| Error::Other(format!("Couldn't detach KeePassXC: {e}")))
        }
    }

    pub fn close(child: isize) {
        unsafe {
            let _ = PostMessageW(Some(hwnd(child)), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::Path;

    use super::Rect;
    use crate::error::{Error, Result};

    pub fn is_window(_: isize) -> bool {
        false
    }
    pub fn find_main_window(_: &Path) -> Option<isize> {
        None
    }
    pub fn attach(_: isize, _: isize, _: Rect) -> Result<(u32, u32)> {
        Err(Error::invalid("Embedding KeePassXC is only supported on Windows."))
    }
    pub fn place(_: isize, _: isize, _: Rect) -> Result<()> {
        Ok(())
    }
    pub fn hide(_: isize) {}
    pub fn detach(_: isize, _: u32, _: u32) -> Result<()> {
        Ok(())
    }
    pub fn close(_: isize) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_style_drops_the_frame_and_makes_a_popup() {
        let overlapped_window = 0x00CF_0000u32 | WS_VISIBLE; // WS_OVERLAPPEDWINDOW | WS_VISIBLE
        let s = overlay_style(overlapped_window);
        assert_eq!(s & WS_CAPTION, 0, "caption must go");
        assert_eq!(s & WS_THICKFRAME, 0, "thick frame must go");
        assert_eq!(s & WS_SYSMENU, 0);
        assert_ne!(s & WS_POPUP, 0, "must be a popup so it has no frame");
    }

    #[test]
    fn overlay_ex_style_hides_from_taskbar_and_removes_edges() {
        let ex = overlay_ex_style(WS_EX_APPWINDOW | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | 0x0000_0008);
        assert_eq!(ex & (WS_EX_APPWINDOW | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE), 0);
        assert_ne!(ex & WS_EX_TOOLWINDOW, 0, "tool window: no taskbar button");
        assert_ne!(ex & 0x8, 0, "unrelated bits are kept");
    }

    #[test]
    fn rect_is_clamped() {
        let r = Rect { x: -99_999, y: 5, w: -4, h: 999_999 }.clamped();
        assert_eq!((r.x, r.y, r.w, r.h), (-20_000, 5, 1, 20_000));
    }
}
