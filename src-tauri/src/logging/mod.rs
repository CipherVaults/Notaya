//! A small local error log, so that when Notaya crashes or fails there is something to look at.
//!
//! * `notaya-log.txt` in the `logs` folder inside the data folder, plain text, one entry per event
//!   with a UTC time, the version and the thread. At 256 KB it becomes `notaya-log.1.txt`, which
//!   replaces the older one, so the two files together stay near 512 KB at most.
//! * It is written without the vault (so a crash before the unlock is recorded too) and is not
//!   encrypted. It never leaves the PC: nothing here touches the network.
//! * It records panics (where, and a class from a fixed list, never the message), errors reported
//!   by the UI, real faults of backend commands (a fixed code), start, clean exit, lock and unlock
//!   (only that they happened), and a "running" flag that shows the next start that the last run
//!   did not end cleanly (a crash that skips the panic hook, a forced stop, a shutdown).
//! * It never records note titles or text, file names, passwords, keys, ids or paths of the
//!   person's files. See `sanitize.rs`, and P27 in `docs/security-design.md`.
//!
//! Writing is best effort: no function here panics, and a failed write is ignored.

mod sanitize;
mod store;

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::Duration;

use serde::Serialize;

use crate::portable::Paths;
use store::{Limiter, Verdict};

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// How long a write waits for another thread's write before it gives up (the UI must not stall).
const LOCK_WAIT: Duration = Duration::from_millis(200);
/// Frames of a panic's backtrace that are written.
const FRAMES: usize = 12;

static DIR: OnceLock<PathBuf> = OnceLock::new();
static OS: OnceLock<String> = OnceLock::new();
static WRITE: Mutex<()> = Mutex::new(());
static CLOSED: AtomicBool = AtomicBool::new(false);
/// Errors from the UI: 20 a minute, 200 a run.
static FRONTEND: Mutex<Limiter> = Mutex::new(Limiter::new(20, 200));
/// Faults of backend commands: 30 a minute, 300 a run.
static BACKEND: Mutex<Limiter> = Mutex::new(Limiter::new(30, 300));

thread_local! {
    /// Set while this thread is inside the logger, so an error in the logger cannot log itself.
    static BUSY: Cell<bool> = const { Cell::new(false) };
}

/// What the Settings screen shows about the log.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogInfo {
    pub folder: String,
    pub bytes: u64,
    pub last_entry: Option<String>,
}

/// The logs folder inside a data folder.
pub fn logs_dir(data: &Path) -> PathBuf {
    data.join("logs")
}

/// The folder in use: the one `init` was given, else the one the paths would resolve to. (Tests never
/// fall back to the real folder, so a test that serializes an error does not write into `.devdata`.)
fn dir() -> PathBuf {
    #[cfg(test)]
    return DIR.get().cloned().unwrap_or_else(|| std::env::temp_dir().join("notaya-log-tests-unused"));
    #[cfg(not(test))]
    DIR.get().cloned().unwrap_or_else(|| logs_dir(&crate::portable::early_data_dir()))
}

/// Remembers the logs folder of this run.
pub fn init(paths: &Paths) {
    let _ = DIR.set(logs_dir(&paths.data));
}

/// The folder the Settings screen may show and open.
pub fn folder(paths: &Paths) -> PathBuf {
    logs_dir(&paths.data)
}

pub fn info(paths: &Paths) -> LogInfo {
    let dir = folder(paths);
    LogInfo { folder: dir.display().to_string(), bytes: store::total_bytes(&dir), last_entry: store::last_entry_time(&dir) }
}

fn thread_label() -> String {
    let t = std::thread::current();
    let name: String = t.name().unwrap_or("unnamed").chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | ' ')).take(40).collect();
    if name.is_empty() { "unnamed".to_string() } else { name }
}

/// One entry: time, level, version, thread, then the text. Lines after the first are indented.
fn format_entry(level: &str, thread: &str, text: &str) -> String {
    let mut lines = text.lines();
    let first = sanitize::line_safe(lines.next().unwrap_or(""), 600);
    let mut out = format!("{} {:<5} v{VERSION} [{thread}] {first}", store::now_utc(), level);
    for line in lines {
        out.push_str("\n    ");
        out.push_str(&sanitize::line_safe(line, 300));
    }
    out
}

/// Writes one entry into `dir`.
fn write_in(dir: &Path, level: &str, text: &str) -> bool {
    store::append(dir, store::ROTATE_AT, &format_entry(level, &thread_label(), text))
}

/// Runs `f` while holding the write lock, waiting a moment for it, and never inside the logger
/// itself. Returns false when it could not run.
fn locked(f: impl FnOnce()) -> bool {
    if BUSY.with(|b| b.replace(true)) {
        return false;
    }
    let mut ran = false;
    let mut f = Some(f);
    for _ in 0..40 {
        match WRITE.try_lock() {
            Ok(_guard) => {
                f.take().map(|f| f());
                ran = true;
                break;
            }
            Err(TryLockError::Poisoned(p)) => {
                let _guard = p.into_inner();
                f.take().map(|f| f());
                ran = true;
                break;
            }
            Err(TryLockError::WouldBlock) => std::thread::sleep(LOCK_WAIT / 40),
        }
    }
    BUSY.with(|b| b.set(false));
    ran
}

/// Writes one entry of fixed text (never text from the person) to the log of this run.
fn emit(level: &str, text: &str) {
    let dir = dir();
    locked(|| {
        write_in(&dir, level, text);
    });
}

// ---------------------------------------------------------------------------------------------
// Start, exit, lock

/// Looks at the "running" flag of the last run, writes the start entry and a new flag. Returns
/// whether the last run had not ended cleanly.
fn start_in(dir: &Path, installed: bool, os: &str) -> bool {
    let previous = store::read_flag(dir);
    if let Some(flag) = &previous {
        write_in(
            dir,
            "WARN",
            &format!(
                "The previous run did not end cleanly (crash, forced stop, shutdown or power loss). It started at {} (version {}).",
                if flag.started.is_empty() { "an unknown time" } else { &flag.started },
                if flag.version.is_empty() { "unknown" } else { &flag.version },
            ),
        );
    }
    let started = store::now_utc();
    write_in(dir, "INFO", &format!("Notaya started ({}).\nWindows: {os}", if installed { "installed" } else { "portable" }));
    store::write_flag(dir, &started, VERSION);
    previous.is_some()
}

fn exit_in(dir: &Path) {
    write_in(dir, "INFO", "Notaya closed normally.");
    store::remove_flag(dir);
}

/// First thing in `setup`: remembers the folder, notes an unclean end of the last run, logs the start
/// and sets the "running" flag.
pub fn start(paths: &Paths) {
    init(paths);
    let os = OS.get_or_init(windows_version).clone();
    let dir = dir();
    locked(|| {
        start_in(&dir, paths.installed, &os);
    });
}

/// Notaya is ending in the normal way (also from the tray's Quit and a failed start): logs it and removes
/// the "running" flag. Only the first call does anything.
pub fn clean_exit() {
    if CLOSED.swap(true, Ordering::SeqCst) {
        return;
    }
    let dir = dir();
    locked(|| exit_in(&dir));
}

/// Why the vault locked, from the fixed list of reasons; anything else is "other".
fn lock_reason(reason: &str) -> &'static str {
    ["manual", "idle", "sleep", "screen-locked", "minimized", "drive-removed", "quit", "tray", "restore"]
        .into_iter()
        .find(|r| *r == reason)
        .unwrap_or("other")
}

pub fn locked_event(reason: &str) {
    emit("INFO", &format!("The vault was locked ({}).", lock_reason(reason)));
}

pub fn unlocked_event() {
    emit("INFO", "The vault was opened.");
}

/// A start that failed before the window opened. `what` is one of two fixed texts.
pub fn start_failed(data_folder: bool, kind: Option<std::io::ErrorKind>) {
    let text = if data_folder {
        format!("Start failed: the data folder cannot be written to{}.", kind.map(|k| format!(" ({k:?})")).unwrap_or_default())
    } else {
        "Start failed: the window could not be opened (the WebView2 runtime may be missing).".to_string()
    };
    emit("ERROR", &text);
}

// ---------------------------------------------------------------------------------------------
// Backend and frontend errors

/// A real fault of a backend command, by its fixed code (see `Error::fault_code`).
pub fn backend_error(code: &str) {
    let ok: String = code.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_')).take(60).collect();
    emit_limited(&BACKEND, "A command failed", &format!("A command failed with a fault of kind {ok}."));
}

/// A record from the UI. Everything is cleaned again here, whatever the UI sent.
pub struct FrontendReport<'a> {
    pub kind: &'a str,
    pub message: &'a str,
    pub source: &'a str,
    pub line: Option<u32>,
}

fn frontend_text(r: &FrontendReport) -> String {
    let line = r.line.map(|l| format!(":{}", l.min(9_999_999))).unwrap_or_default();
    format!(
        "UI error ({}) in {}{}: {}",
        sanitize::frontend_kind(r.kind),
        sanitize::frontend_source(r.source),
        line,
        sanitize::frontend_message(r.message),
    )
}

pub fn frontend_error(r: &FrontendReport) {
    emit_limited(&FRONTEND, "UI errors", &frontend_text(r));
}

/// Writes an error entry unless `limiter` says it is too many; then one line says so.
fn emit_limited(limiter: &Mutex<Limiter>, what: &str, text: &str) {
    let verdict = match limiter.lock() {
        Ok(mut l) => l.check(store::now_secs()),
        Err(p) => p.into_inner().check(store::now_secs()),
    };
    match verdict {
        Verdict::Allow => emit("ERROR", text),
        Verdict::Refuse { first: true } => emit("WARN", &format!("Too many entries: {what} are not logged for a while.")),
        Verdict::Refuse { first: false } => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Panics

/// Where and what class of panic, as the text of one entry.
fn panic_text(file: &str, line: u32, col: u32, class: &str, frames: &[String], os: &str) -> String {
    let mut text = format!("Panic at {}:{line}:{col}: {class}\nWindows: {os}", sanitize::panic_file(file));
    if !frames.is_empty() {
        text.push_str("\nCall stack (innermost first):");
        for f in frames {
            text.push_str("\n  ");
            text.push_str(f);
        }
    }
    text
}

fn record_panic(info: &std::panic::PanicHookInfo<'_>) {
    // The message is looked at only to find its class; it is never written.
    let class = sanitize::panic_class(info.payload_as_str());
    let (file, line, col) = info.location().map(|l| (l.file(), l.line(), l.column())).unwrap_or(("unknown", 0, 0));
    let frames = sanitize::backtrace_frames(&std::backtrace::Backtrace::force_capture().to_string(), FRAMES);
    let os = OS.get().map(String::as_str).unwrap_or("not read yet");
    let text = panic_text(file, line, col, class, &frames, os);
    let dir = dir();
    // The process may abort right after this hook, so the entry is written and flushed here.
    locked(|| {
        write_in(&dir, "PANIC", &text);
    });
}

/// Installs the panic hook. Call it first in `run()`. The release build aborts on a panic, so the
/// hook writes the entry before that happens; the earlier hook still runs afterwards.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        record_panic(info);
        previous(info);
    }));
}

// ---------------------------------------------------------------------------------------------
// Windows version, without a crate

#[cfg(windows)]
fn windows_version() -> String {
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, REG_SZ, REG_VALUE_TYPE};

    fn read(key: HKEY, name: PCWSTR) -> Option<String> {
        let mut kind = REG_VALUE_TYPE::default();
        let mut buf = [0u16; 64];
        let mut size = std::mem::size_of_val(&buf) as u32;
        // SAFETY: `buf` has `size` writable bytes, and `name` is a NUL-terminated wide string.
        let status = unsafe { RegQueryValueExW(key, name, None, Some(&mut kind), Some(buf.as_mut_ptr() as *mut u8), Some(&mut size)) };
        if status != ERROR_SUCCESS || kind != REG_SZ {
            return None;
        }
        let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        let text: String = String::from_utf16_lossy(&buf[..len]).chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '-')).collect();
        Some(text).filter(|t| !t.is_empty())
    }

    let mut key = HKEY::default();
    // SAFETY: `key` is a valid out pointer; the key name is a NUL-terminated wide string.
    let status = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"), None, KEY_QUERY_VALUE, &mut key) };
    if status != ERROR_SUCCESS {
        return "Windows (version not readable)".to_string();
    }
    let build = read(key, w!("CurrentBuildNumber"));
    let display = read(key, w!("DisplayVersion"));
    // SAFETY: `key` was opened above and is closed once.
    unsafe {
        let _ = RegCloseKey(key);
    }
    // Windows 11 still reports itself as "Windows 10" in the product name; the build number tells.
    let family = match build.as_deref().and_then(|b| b.parse::<u32>().ok()) {
        Some(n) if n >= 22000 => "Windows 11",
        Some(_) => "Windows 10 or older",
        None => "Windows",
    };
    let mut text = family.to_string();
    if let Some(b) = build {
        text.push_str(&format!(" (build {b}"));
        if let Some(d) = display {
            text.push_str(&format!(", {d}"));
        }
        text.push(')');
    }
    text
}

#[cfg(not(windows))]
fn windows_version() -> String {
    "not Windows".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("notaya-log-mod-{}", crate::model::new_id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn log_text(dir: &Path) -> String {
        let mut text = String::new();
        for name in [store::OLD_LOG_FILE, store::LOG_FILE] {
            text.push_str(&fs::read_to_string(dir.join(name)).unwrap_or_default());
        }
        text
    }

    #[test]
    fn an_entry_has_a_time_a_level_the_version_and_a_thread() {
        let dir = temp_dir();
        write_in(&dir, "INFO", "first line\nsecond line");
        let text = log_text(&dir);
        let first = text.lines().next().unwrap();
        assert!(first.len() > 20 && first.as_bytes()[4] == b'-' && first.as_bytes()[19] == b'Z', "{first}");
        assert!(first.contains(" INFO ") && first.contains(&format!("v{VERSION}")) && first.contains("[") && first.ends_with("first line"), "{first}");
        assert_eq!(text.lines().nth(1).unwrap(), "    second line");
    }

    #[test]
    fn the_run_marker_finds_an_unclean_end_and_a_clean_exit_removes_it() {
        let dir = temp_dir();
        // First run: no flag before, flag after.
        assert!(!start_in(&dir, false, "Test OS"));
        assert!(store::read_flag(&dir).is_some());
        assert!(!log_text(&dir).contains("did not end cleanly"));
        // The run ends cleanly: no flag, an exit entry.
        exit_in(&dir);
        assert!(store::read_flag(&dir).is_none());
        assert!(log_text(&dir).contains("closed normally"));
        // A run that ended cleanly leaves nothing for the next start to find.
        assert!(!start_in(&dir, true, "Test OS"));
        // A crash: the flag stays. The next start says so, with the old start time, and writes a new flag.
        let old = store::read_flag(&dir).unwrap();
        fs::write(dir.join(store::FLAG_FILE), "started=2026-01-02T03:04:05Z\nversion=1.2.3\n").unwrap();
        assert!(start_in(&dir, false, "Test OS"));
        let text = log_text(&dir);
        assert!(text.contains("did not end cleanly") && text.contains("2026-01-02T03:04:05Z") && text.contains("1.2.3"), "{text}");
        let new = store::read_flag(&dir).unwrap();
        assert_ne!(new.started, "2026-01-02T03:04:05Z");
        assert_eq!(new.version, old.version);
        // Only the start was logged (portable/installed), no paths.
        assert!(text.contains("Notaya started (portable)") && text.contains("Notaya started (installed)"));
    }

    #[test]
    fn a_panic_entry_has_a_place_and_a_class_but_not_the_message() {
        let class = sanitize::panic_class(Some("byte index 3 is not a char boundary; it is inside 'é' of `my diary entry`"));
        let text = panic_text(r"C:\Users\dev\notaya\src-tauri\src\vault\notes.rs", 12, 5, class, &["notaya_lib::vault::notes::slice".to_string()], "Windows 11 (build 22631)");
        assert!(text.contains("src/vault/notes.rs:12:5") && text.contains("byte index") && text.contains("notaya_lib::vault::notes::slice"), "{text}");
        for leaked in ["diary", "Users", "boundary", "'é'"] {
            assert!(!text.contains(leaked), "{leaked} in {text}");
        }
    }

    #[test]
    fn the_panic_hook_writes_the_entry_without_the_message() {
        // Uses the real hook with a throwaway folder; a thread that panics in a test unwinds.
        let dir = temp_dir();
        let _ = DIR.set(dir.clone());
        if DIR.get() != Some(&dir) {
            // Another test in this process already chose a folder; the hook then writes there.
            return;
        }
        let previous = std::panic::take_hook();
        install_panic_hook();
        let handle = std::thread::Builder::new().name("hook-test".into()).spawn(|| {
            let secret = String::from("my secret diary entry");
            let _ = &secret[100..]; // panics with a message that quotes parts of the string
        });
        let _ = handle.unwrap().join();
        std::panic::set_hook(previous);
        let text = log_text(&dir);
        assert!(text.contains("PANIC") && text.contains("hook-test") && text.contains(": start byte index\n"), "{text}");
        assert!(text.contains("src/") && !text.contains("secret") && !text.contains("diary"), "{text}");
    }

    #[test]
    fn hostile_input_never_reaches_the_log() {
        // Text that could be a note, a snippet of one, a file the person picked, or full of control characters.
        let notes = [
            "Dear diary, my password is hunter2",
            r#"{"title":"Tax return 2025","body":"IBAN DE89 3704 0044 0532 0130 00"}"#,
            "He said it's \"my PIN is 4921\" and 'ok'",
            "line one\nFAKE 2026-01-01T00:00:00Z ERROR forged entry\u{0}\u{7}\r\u{202e}",
        ];
        let paths = [r"C:\Users\x\secret.docx", r"C:\Users\x\My private notes.docx", "/home/x/secret.docx", r"\\server\share\secret.docx"];
        let marks = ["Dear diary", "hunter2", "Tax return", "IBAN", "DE89", "4921", "PIN", "secret.docx", "private", "Users", "server", "forged", "FAKE", "home/x"];
        let dir = temp_dir();
        let mut embedded: Vec<String> = Vec::new();
        for n in notes {
            let long = format!("{n} {}", "A".repeat(5000));
            for h in [n, long.as_str()] {
                // The ways such text turns up in the messages of a browser.
                embedded.push(format!("Cannot read properties of undefined (reading '{h}')"));
                embedded.push(format!("Unexpected token '{h}', \"{h}\"... is not valid JSON"));
                embedded.push(format!("SyntaxError: Unexpected token }} in JSON at position 5: {h}"));
                embedded.push(format!("JSON.parse: bad value: {h}"));
                embedded.push(format!("x is not a function (near `{h}`)"));
            }
        }
        for p in paths {
            embedded.push(format!("Failed to open {p} for reading"));
            embedded.push(format!("Error: ENOENT, open '{p}'"));
        }
        for message in &embedded {
            let report = FrontendReport { kind: "error", message, source: "http://tauri.localhost/assets/index-1.js", line: Some(7) };
            write_in(&dir, "ERROR", &frontend_text(&report));
        }
        for h in notes.iter().copied().chain(paths) {
            // The kind and the source are free fields too: whatever comes in, only fixed words go out.
            let report = FrontendReport { kind: h, message: "x is not a function", source: h, line: Some(7) };
            write_in(&dir, "ERROR", &frontend_text(&report));
            // A panic whose message holds the data, and a location that is not a source file.
            let class = sanitize::panic_class(Some(&format!("byte index 3 is out of bounds of `{h}`")));
            let frames = sanitize::backtrace_frames(&format!("  0: {h}\n  1: ok::fn\n"), 5);
            write_in(&dir, "PANIC", &panic_text(h, 1, 2, class, &frames, "Windows"));
            // Backend errors made from the data: the code never holds it.
            let errors = [
                crate::error::Error::Io(std::io::Error::other(h.to_string())),
                crate::error::Error::Other(h.to_string()),
                crate::error::Error::Db(rusqlite::Error::InvalidParameterName(h.to_string())),
                crate::error::Error::Invalid(h.to_string()),
                crate::error::Error::Corrupt,
            ];
            for e in errors {
                if let Some(code) = e.fault_code() {
                    let ok: String = code.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_')).collect();
                    write_in(&dir, "ERROR", &format!("A command failed with a fault of kind {ok}."));
                }
            }
            // A lock reason, and a flag file somebody changed.
            write_in(&dir, "INFO", &format!("The vault was locked ({}).", lock_reason(h)));
            fs::write(dir.join(store::FLAG_FILE), format!("started={h}\nversion={h}\n")).unwrap();
            start_in(&dir, false, "OS");
        }
        let text = log_text(&dir);
        assert!(text.lines().count() > 50);
        for m in marks {
            assert!(!text.contains(m), "{m:?} leaked into the log:\n{text}");
        }
        // Every entry starts with a time stamp or is an indented continuation: nothing could forge a line.
        for line in text.lines() {
            assert!(line.starts_with("    ") || line.as_bytes().get(4) == Some(&b'-'), "{line:?}");
        }
        for line in text.lines().filter(|l| !l.starts_with("    ")) {
            assert!(line.len() < 700, "{} chars", line.len());
        }
    }

    #[test]
    fn a_log_that_cannot_be_written_does_not_stop_anything() {
        let dir = temp_dir();
        let blocked = dir.join("logs");
        fs::write(&blocked, b"a file where the folder should be").unwrap();
        assert!(!write_in(&blocked, "INFO", "lost"));
        assert!(!start_in(&blocked, false, "OS"));
        exit_in(&blocked);
    }

    #[test]
    fn lock_reasons_come_from_a_fixed_list() {
        assert_eq!(lock_reason("idle"), "idle");
        assert_eq!(lock_reason("C:\\secret"), "other");
    }

    #[test]
    fn an_error_inside_the_logger_does_not_log_itself() {
        let ran = std::cell::Cell::new(false);
        assert!(locked(|| {
            // A nested call from the same thread is refused instead of waiting for itself.
            assert!(!locked(|| ran.set(true)));
        }));
        assert!(!ran.get());
        assert!(locked(|| ran.set(true)));
    }

    #[test]
    fn info_reports_size_and_last_entry() {
        let root = temp_dir();
        let paths = Paths::at(&root);
        let info0 = info(&paths);
        assert_eq!(info0.bytes, 0);
        assert!(info0.last_entry.is_none());
        write_in(&folder(&paths), "INFO", "hello");
        let info1 = info(&paths);
        assert!(info1.bytes > 20 && info1.last_entry.is_some());
        assert_eq!(PathBuf::from(&info1.folder), root.join("data").join("logs"));
    }
}
