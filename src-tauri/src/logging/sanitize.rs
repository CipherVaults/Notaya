//! Everything that turns a raw error into text that may be written to the error log.
//!
//! The log must never hold the person's notes, file names, passwords or paths. Rust's and the
//! browser's own error messages sometimes embed a piece of the data they were working on (a string
//! slice that failed, a JSON snippet that did not parse), so nothing is copied from a message as it
//! is: panics and UI errors use closed classes. Renderer messages are never accepted by the log.

use serde::Deserialize;

/// The longest accepted script basename, in ASCII bytes. Invalid input is replaced, never repaired.
const SOURCE_MAX: usize = 60;
/// A panic location file name or a backtrace frame is cut to this many characters.
const FRAME_MAX: usize = 100;

/// Starts of Rust's own panic messages that are safe to name. Only the start is kept; everything after
/// it (a slice, an index value, the `Debug` text of an error) may hold data and is dropped.
const PANIC_PREFIXES: &[&str] = &[
    "index out of bounds",
    "byte index",
    "start byte index",
    "end byte index",
    "begin <= end",
    "slice index starts at",
    "range start index",
    "range end index",
    "called `Option::unwrap()` on a `None` value",
    "called `Result::unwrap()` on an `Err` value",
    "already borrowed",
    "already mutably borrowed",
    "attempt to add with overflow",
    "attempt to subtract with overflow",
    "attempt to multiply with overflow",
    "attempt to divide by zero",
    "attempt to divide with overflow",
    "attempt to calculate the remainder with a divisor of zero",
    "attempt to calculate the remainder with overflow",
    "attempt to negate with overflow",
    "attempt to shift left with overflow",
    "attempt to shift right with overflow",
    "explicit panic",
    "not implemented",
    "not yet implemented",
    "internal error: entered unreachable code",
    "assertion failed",
    "assertion `left",
    "capacity overflow",
    "mid > len",
    "removal index",
    "insertion index",
    "swap_remove index",
    "source slice length",
    "destination and source slices have different lengths",
];

/// The class of a panic from its message: a start from the fixed list above, or a generic text.
/// Nothing after the matched start is returned. `None` is a payload that is not text.
pub fn panic_class(message: Option<&str>) -> &'static str {
    let Some(message) = message else {
        return "panic with a payload that is not text";
    };
    if let Some(p) = PANIC_PREFIXES.iter().find(|p| message.starts_with(**p)) {
        return p;
    }
    if message.starts_with("attempt to ") {
        return "attempt to (arithmetic problem)";
    }
    "panic with another message (text not recorded)"
}

/// A character that must never reach the log: controls (so an entry stays on its line and cannot
/// forge another), the Unicode line separators and the bidirectional overrides.
fn is_unsafe_char(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200b}'..='\u{200f}')
}

/// Control characters become spaces; other unsafe ones are dropped.
pub fn strip_controls(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_control() {
                Some(' ')
            } else if is_unsafe_char(c) {
                None
            } else {
                Some(c)
            }
        })
        .collect()
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// The only error classes sent by the window handlers and React boundary. No free text survives.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrontendKind {
    Error,
    UnhandledRejection,
    React,
    #[serde(other)]
    Unknown,
}

impl FrontendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::UnhandledRejection => "unhandledrejection",
            Self::React => "react",
            Self::Unknown => "unknown",
        }
    }
}

/// Only a bounded source basename is metadata. A path, prose, or invalid character causes the
/// entire field to become a fixed placeholder; filtering characters could retain note fragments.
pub fn frontend_source(raw: &str) -> &str {
    if raw.is_empty() || raw.len() > SOURCE_MAX
        || !raw.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')) {
        return "unknown";
    }
    let ext = raw.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if raw.contains('.') && matches!(ext.as_str(), "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "html") {
        raw
    } else {
        "unknown"
    }
}

/// The source file of a panic location (`src\state.rs`): from the last `src` folder on, else the file
/// name, in a safe set of characters. These are paths in the program's own source, not the person's.
pub fn panic_file(raw: &str) -> String {
    let norm = raw.replace('\\', "/");
    let parts: Vec<&str> = norm.split('/').collect();
    let start = parts.iter().rposition(|p| *p == "src").unwrap_or(parts.len().saturating_sub(1));
    let joined = parts[start..].join("/");
    let kept: String = joined.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')).collect();
    // A panic location is always a Rust source file; anything else is not from the compiler.
    if kept.ends_with(".rs") { truncate_chars(&kept, FRAME_MAX) } else { "unknown".to_string() }
}

/// Frames of a printed backtrace (`   3: name` lines), without the `at file:line` lines, hash suffixes
/// and the frames of the panic machinery itself. Names of functions only.
pub fn backtrace_frames(text: &str, max: usize) -> Vec<String> {
    let mut frames = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        let Some((num, name)) = t.split_once(':') else { continue };
        if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let mut name = name.trim().to_string();
        // Remove the `::h0123456789abcdef` hash the compiler appends.
        if let Some(pos) = name.rfind("::h") {
            let tail = &name[pos + 3..];
            if tail.len() == 16 && tail.bytes().all(|b| b.is_ascii_hexdigit()) {
                name.truncate(pos);
            }
        }
        let noise = ["backtrace", "panicking", "rust_begin_unwind", "rust_panic", "panic::", "std::sys::", "logging::"];
        if name.is_empty() || noise.iter().any(|n| name.contains(n)) {
            continue;
        }
        // A function name is a path of identifiers (`a::b`, `<T as U>::f`, `{{closure}}`) or `<unknown>`;
        // free text with spaces and no `::` is not one.
        let ident = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '.');
        if !(name == "<unknown>" || name.contains("::") || name.chars().all(ident)) {
            continue;
        }
        let kept: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '<' | '>' | ' ' | ',' | '&' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | '#' | '$' | '.'))
            .collect();
        frames.push(truncate_chars(&kept, FRAME_MAX));
        if frames.len() >= max {
            break;
        }
    }
    frames
}

/// A fixed text for the log: controls removed and cut to `max` characters. For text the program wrote
/// itself (never for text that came from the person).
pub fn line_safe(s: &str, max: usize) -> String {
    truncate_chars(&strip_controls(s), max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_messages_are_reduced_to_their_class() {
        assert_eq!(panic_class(Some("byte index 5 is out of bounds of `my secret note`")), "byte index");
        assert_eq!(panic_class(Some("start byte index 100 is out of bounds for string of length 21")), "start byte index");
        assert_eq!(panic_class(Some("index out of bounds: the len is 3 but the index is 7")), "index out of bounds");
        assert_eq!(
            panic_class(Some("called `Result::unwrap()` on an `Err` value: Invalid(\"Dear diary\")")),
            "called `Result::unwrap()` on an `Err` value"
        );
        assert_eq!(panic_class(Some("attempt to add with overflow")), "attempt to add with overflow");
        assert_eq!(panic_class(Some("attempt to do something new")), "attempt to (arithmetic problem)");
        assert_eq!(panic_class(Some("already borrowed: BorrowMutError")), "already borrowed");
        assert_eq!(panic_class(Some("the password is hunter2")), "panic with another message (text not recorded)");
        assert_eq!(panic_class(None), "panic with a payload that is not text");
    }

    #[test]
    fn sources_accept_only_bounded_script_basenames() {
        assert_eq!(frontend_source("index-AbC123.js"), "index-AbC123.js");
        assert_eq!(frontend_source(&format!("{}.js", "a".repeat(SOURCE_MAX - 3))).len(), SOURCE_MAX);
        for source in [
            "", "noextension", "notes.docx", "../index.js", "C:/private/index.js",
            r"C:\private\index.js", "http://tauri.localhost/assets/index.js", "index.js?note=1",
            "private note.js", "private\"note.js", "private'note.js", "private:token.js",
            "index.js\n", "index.js\u{202e}", "index-\u{e9}.js",
            &format!("{}.js", "a".repeat(SOURCE_MAX - 2)),
            &"x".repeat(100_000),
        ] {
            assert_eq!(frontend_source(source), "unknown");
        }
    }

    #[test]
    fn kinds_deserialize_to_a_closed_class() {
        for name in ["error", "unhandledrejection", "react", "unknown"] {
            let kind: FrontendKind = serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(kind.as_str(), name);
        }
        for name in ["my secret note", "ERROR", "error\nforged", &"a".repeat(100_000)] {
            let kind: FrontendKind = serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(kind, FrontendKind::Unknown);
        }
    }

    #[test]
    fn panic_files_are_program_source_paths() {
        assert_eq!(panic_file(r"C:\Users\dev\notaya\src-tauri\src\state.rs"), "src/state.rs");
        assert_eq!(panic_file("/rustc/abc/library/core/src/str/mod.rs"), "src/str/mod.rs");
        assert_eq!(panic_file("main.rs"), "main.rs");
        assert_eq!(panic_file(r"C:\Users\x\secret.docx"), "unknown");
    }

    #[test]
    fn backtraces_keep_function_names_only() {
        let text = "   0: std::backtrace::Backtrace::force_capture\n             at /rustc/x/library/std/src/backtrace.rs:1:1\n   1: notaya_lib::logging::hook\n   2: notaya_lib::state::lock::h0123456789abcdef\n             at C:\\Users\\dev\\secret\\state.rs:10\n   3: <unknown>\n   4: core::panicking::panic\n";
        let frames = backtrace_frames(text, 10);
        assert_eq!(frames, vec!["notaya_lib::state::lock".to_string(), "<unknown>".to_string()]);
        // Text that is not a function name is dropped.
        assert!(backtrace_frames("  0: Dear diary, my password is hunter2\n  1: {\"title\":\"Tax\"}\n", 5).is_empty());
        let many: String = (0..50).map(|i| format!("{i:4}: f{i}\n")).collect();
        assert_eq!(backtrace_frames(&many, 12).len(), 12);
    }
}
