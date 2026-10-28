//! Keeps an older Notaya from running beside this one on the same notes.
//!
//! Notaya V1.3.0 and earlier guard against a second copy with a Windows mutex and a hidden window
//! that are named after the old app identifier (`app.cipherleaf.vault`). V1.4.0 has a new
//! identifier, so on its own it would not meet them, and an old and a new copy could open one vault
//! at the same time. This module takes the old names too: it holds the old mutex and owns a hidden
//! window with the old class and title. A copy that starts later (old or new) finds them, sends the
//! same "show your window" message the old guard sends (`WM_COPYDATA`), and exits.

#[cfg(windows)]
pub use imp::{claim, Claim, OLD_ID};

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use std::sync::mpsc;
    use std::time::Duration;

    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::DataExchange::COPYDATASTRUCT;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW,
        GetWindowLongPtrW, GetWindowThreadProcessId, RegisterClassExW, SendMessageTimeoutW, SetWindowLongPtrW,
        TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, MSG, SMTO_NORMAL, WM_COPYDATA, WM_CREATE, WNDCLASSEXW,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    };

    /// The identifier of Notaya V1.3.0 and earlier.
    pub const OLD_ID: &str = "app.cipherleaf.vault";
    /// The payload tag of the single-instance message: the working folder and the arguments, NUL separated.
    const COPYDATA_ARGS: usize = 1543;
    const FORWARD_TIMEOUT_MS: u32 = 10_000;

    type Wake = Box<dyn Fn() + Send + Sync>;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Claim {
        /// This copy now holds the names; carry on starting.
        Held,
        /// Another copy holds them. It was told to show its window (or could not be reached); this copy must exit.
        Handed,
    }

    /// Takes the mutex and the hidden window named after `id`, or hands over to the copy that has them.
    /// `wake` runs when a later copy asks this one to show itself.
    pub fn claim(id: &str, wake: impl Fn() + Send + Sync + 'static) -> Result<Claim, String> {
        let mutex_name = HSTRING::from(format!("{id}-sim"));
        let class = HSTRING::from(format!("{id}-sic"));
        let title = HSTRING::from(format!("{id}-siw"));

        // SAFETY: plain Win32 calls with valid, NUL-terminated names that outlive each call.
        let handle = unsafe { CreateMutexW(None, true, &mutex_name) }.map_err(|e| e.to_string())?;
        let exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        if exists {
            // The first copy may still be creating its window: give it a moment.
            for _ in 0..20 {
                if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr())) } {
                    forward(hwnd);
                    return Ok(Claim::Handed);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            // The mutex is taken but there is no window to ask: still do not run a second copy.
            return Ok(Claim::Handed);
        }
        // Held for the life of the process (released by Windows when it ends).
        let _ = handle; // never closed: HANDLE is a plain value, the mutex lives until the process ends

        let wake: Box<Wake> = Box::new(Box::new(wake));
        let wake_ptr = Box::into_raw(wake) as usize;
        let (tx, rx) = mpsc::channel::<Result<(), String>>();
        std::thread::spawn(move || {
            // SAFETY: the window is created and used on this thread only, with a message loop below.
            let created = unsafe { create_window(&class, &title, wake_ptr as *const c_void) };
            let ok = created.is_ok();
            let _ = tx.send(created);
            if ok {
                let mut msg = MSG::default();
                while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        });
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Claim::Held),
            Ok(Err(e)) => Err(format!("the guard window could not be created: {e}")),
            Err(_) => Err("the guard window was not created in time".into()),
        }
    }

    unsafe fn create_window(class: &HSTRING, title: &HSTRING, wake: *const c_void) -> Result<(), String> {
        unsafe {
            let module = GetModuleHandleW(None).map_err(|e| e.to_string())?;
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(window_proc),
                hInstance: module.into(),
                lpszClassName: PCWSTR(class.as_ptr()),
                ..Default::default()
            };
            if RegisterClassExW(&wc) == 0 {
                return Err(format!("class: {}", windows::core::Error::from_thread()));
            }
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                PCWSTR(class.as_ptr()),
                PCWSTR(title.as_ptr()),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(module.into()),
                Some(wake),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
    }

    unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_CREATE => {
                    let create = &*(lparam.0 as *const CREATESTRUCTW);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
                    LRESULT(0)
                }
                WM_COPYDATA => {
                    let wake = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Wake;
                    if !wake.is_null() {
                        (*wake)();
                    }
                    LRESULT(1)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    /// Sends the "show your window" message in the form the old guard understands.
    fn forward(hwnd: HWND) {
        let mut data = std::env::current_dir().unwrap_or_default().to_string_lossy().into_owned();
        for arg in std::env::args_os() {
            data.push('\0');
            data.push_str(&arg.to_string_lossy());
        }
        let bytes = data.as_bytes();
        let cds = COPYDATASTRUCT { dwData: COPYDATA_ARGS, cbData: bytes.len() as u32, lpData: bytes.as_ptr() as *mut c_void };
        // SAFETY: `cds` and the bytes it points to stay alive until the call returns.
        unsafe {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != 0 {
                let _ = AllowSetForegroundWindow(pid);
            }
            SendMessageTimeoutW(
                hwnd,
                WM_COPYDATA,
                WPARAM(0),
                LPARAM(&cds as *const COPYDATASTRUCT as isize),
                SMTO_NORMAL,
                FORWARD_TIMEOUT_MS,
                None,
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        // Needs a desktop to create a window: under Wine in the cloud run it with `xvfb-run -a`.
        #[test]
        fn a_second_copy_wakes_the_first_and_is_told_to_exit() {
            let id = format!("notaya-test-{}", crate::model::new_id());
            let woken = Arc::new(AtomicUsize::new(0));
            let counter = woken.clone();
            assert_eq!(claim(&id, move || { counter.fetch_add(1, Ordering::SeqCst); }).unwrap(), Claim::Held);
            assert_eq!(woken.load(Ordering::SeqCst), 0);
            // The same process asking again behaves like a second program: the mutex exists.
            assert_eq!(claim(&id, || panic!("the second copy must not hold the names")).unwrap(), Claim::Handed);
            assert_eq!(woken.load(Ordering::SeqCst), 1, "the first copy was asked to show its window");
        }
    }
}
