//! Secret-aware clipboard for Windows.
//!
//! Copied secrets are tagged so Windows leaves them out of clipboard history
//! (Win+V) and cloud clipboard sync. The caller clears them after a timeout,
//! unless the user has copied something else in the meantime.

use zeroize::Zeroizing;

use crate::error::{Error, Result};

#[cfg(windows)]
mod imp {
    use super::*;
    use windows::core::w;
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, OpenClipboard,
        RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_UNICODETEXT;

    fn open() -> Result<()> {
        // Another app may hold the clipboard for a moment; retry briefly.
        for _ in 0..10 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        Err(Error::Other("The clipboard is busy. Try again.".into()))
    }

    /// Copies `bytes` into a movable global memory block owned by the clipboard.
    unsafe fn set_bytes(format: u32, bytes: &[u8]) -> Result<()> {
        unsafe {
            let h: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1))
                .map_err(|e| Error::Other(e.to_string()))?;
            let ptr = GlobalLock(h) as *mut u8;
            if ptr.is_null() {
                let _ = GlobalFree(Some(h));
                return Err(Error::Other("Clipboard memory error".into()));
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            let _ = GlobalUnlock(h);
            if SetClipboardData(format, Some(HANDLE(h.0))).is_err() {
                let _ = GlobalFree(Some(h));
                return Err(Error::Other("Could not write to the clipboard.".into()));
            }
            Ok(())
        }
    }

    pub fn set_secret(text: &str) -> Result<u32> {
        let mut wide: Zeroizing<Vec<u16>> = Zeroizing::new(text.encode_utf16().collect());
        wide.push(0);
        let bytes: Zeroizing<Vec<u8>> =
            Zeroizing::new(wide.iter().flat_map(|u| u.to_le_bytes()).collect());
        open()?;
        let result = unsafe {
            (|| -> Result<()> {
                EmptyClipboard().map_err(|e| Error::Other(e.to_string()))?;
                set_bytes(CF_UNICODETEXT.0 as u32, &bytes)?;
                let zero = 0u32.to_le_bytes();
                set_bytes(RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing")), &zero)?;
                set_bytes(RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory")), &zero)?;
                set_bytes(RegisterClipboardFormatW(w!("CanUploadToCloudClipboard")), &zero)?;
                Ok(())
            })()
        };
        let _ = unsafe { CloseClipboard() };
        result?;
        Ok(unsafe { GetClipboardSequenceNumber() })
    }

    /// Clears the clipboard only if it still holds what we put there.
    pub fn clear_if_unchanged(sequence: u32) {
        if unsafe { GetClipboardSequenceNumber() } != sequence {
            return;
        }
        if open().is_ok() {
            unsafe {
                let _ = EmptyClipboard();
                let _ = CloseClipboard();
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    pub fn set_secret(_text: &str) -> Result<u32> {
        Err(Error::Other("Clipboard is only supported on Windows.".into()))
    }
    pub fn clear_if_unchanged(_sequence: u32) {}
}

pub use imp::{clear_if_unchanged, set_secret};
