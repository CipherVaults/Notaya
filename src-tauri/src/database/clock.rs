//! The local date and time as minutes since 1970 (no time zone database, no network): the
//! Windows local clock, and its offset from UTC. Other systems (the cloud test container) use UTC.

use super::value::from_parts;

/// (local minutes since 1970-01-01 00:00, offset of local time from UTC in minutes)
#[cfg(windows)]
pub fn local_now() -> (i64, i64) {
    use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
    // SAFETY: both calls only fill in a SYSTEMTIME and cannot fail.
    let (l, u) = unsafe { (GetLocalTime(), GetSystemTime()) };
    let local = from_parts(l.wYear as i64, l.wMonth as i64, l.wDay as i64, l.wHour as i64, l.wMinute as i64);
    let utc = from_parts(u.wYear as i64, u.wMonth as i64, u.wDay as i64, u.wHour as i64, u.wMinute as i64);
    (local, local - utc)
}

#[cfg(not(windows))]
pub fn local_now() -> (i64, i64) {
    let ms = crate::model::now_ms();
    let _ = from_parts;
    (ms.div_euclid(60_000), 0)
}
