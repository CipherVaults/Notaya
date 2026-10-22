//! Windows signals that trigger an automatic lock.

#[cfg(windows)]
mod imp {
    use windows::Win32::System::StationsAndDesktops::{
        CloseDesktop, OpenInputDesktop, SwitchDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
    };
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    /// Milliseconds since the last keyboard or mouse input anywhere in the session.
    pub fn idle_ms() -> u64 {
        let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        unsafe {
            if GetLastInputInfo(&mut info).as_bool() {
                GetTickCount().wrapping_sub(info.dwTime) as u64
            } else {
                0
            }
        }
    }

    /// True while the Windows lock screen (Win+L) or another secure desktop is shown.
    pub fn workstation_locked() -> bool {
        unsafe {
            match OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP) {
                Ok(desk) => {
                    let switchable = SwitchDesktop(desk).is_ok();
                    let _ = CloseDesktop(desk);
                    !switchable
                }
                Err(_) => true,
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn idle_ms() -> u64 {
        0
    }
    pub fn workstation_locked() -> bool {
        false
    }
}

pub use imp::{idle_ms, workstation_locked};
