//! Everything that turns a raw error into text that may be written to the error log.
//!
//! The log must never hold the person's notes, file names, passwords or paths. Rust's and the
//! browser's own error messages sometimes embed a piece of the data they were working on (a string
//! slice that failed, a JSON snippet that did not parse), so nothing is copied from a message as it
//! is: a panic is reduced to a class from a fixed list, and a message from the UI loses everything
//! that could be data. This is best effort (see P27), so the log is still plain text the person can
//! read before sending it.

/// The longest message from the UI that is kept, in characters.
pub const MESSAGE_MAX: usize = 200;
/// Raw input is cut to this many characters before any work is done on it.
const RAW_MAX: usize = 2000;
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

/// Removes everything inside "…", '…', `…`, “…” and ‘…’, quotes included (replaced by "…"). An opening
/// quote that is never closed removes the rest. A `'` after a letter (`don't`) is an apostrophe, and
/// a `'` that closes a quote must not be followed by a letter, so `'don't tell'` is one quote.
pub fn remove_quoted(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev_alnum = i > 0 && chars[i - 1].is_alphanumeric();
        let close = match c {
            '"' => Some('"'),
            '`' => Some('`'),
            '\u{201c}' => Some('\u{201d}'),
            '\u{2018}' => Some('\u{2019}'),
            '\'' if !prev_alnum => Some('\''),
            _ => None,
        };
        let Some(close) = close else {
            out.push(c);
            i += 1;
            continue;
        };
        // Skip to the closing quote.
        i += 1;
        while i < chars.len() {
            let d = chars[i];
            if d == '\\' {
                i += 2;
                continue;
            }
            let ends = d == close && (close != '\'' || !chars.get(i + 1).is_some_and(|n| n.is_alphanumeric()));
            i += 1;
            if ends {
                break;
            }
        }
        out.push('\u{2026}');
    }
    out
}

/// True for a word that looks like a file path or a URL.
fn looks_like_path(word: &str) -> bool {
    let b = word.as_bytes();
    word.contains('\\')
        || word.contains("://")
        || word.starts_with("//")
        || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b.len() == 2 || b[2] == b'/'))
        || (word.starts_with('/') && word[1..].contains('/'))
        || word.starts_with("~/")
}

/// Cuts the message at the first word that looks like a path, because a path may continue with spaces
/// (`C:\Users\x\My notes.docx`).
fn cut_at_path(s: &str) -> String {
    let mut words = Vec::new();
    for w in s.split_whitespace() {
        if looks_like_path(w) {
            words.push("<path removed>");
            break;
        }
        words.push(w);
    }
    words.join(" ")
}

/// For text that mentions JSON or a token: drops everything after the first `:` that follows the
/// mention (a parser's complaint often quotes the input there).
fn cut_after_json_colon(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let from = ["json", "token"].iter().filter_map(|k| lower.find(k)).min();
    let Some(from) = from else { return s.to_string() };
    match s[from..].find(':') {
        Some(rel) => s[..from + rel].to_string(),
        None => s.to_string(),
    }
}

/// "Unexpected token X in JSON": X is one character of the input, so the word after `token` goes.
fn drop_word_after_token(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::new();
    let mut at = 0;
    while let Some(rel) = lower[at..].find("token ") {
        let end = at + rel + "token ".len();
        out.push_str(&s[at..end]);
        let rest = &s[end..];
        let skip = rest.find(char::is_whitespace).unwrap_or(rest.len());
        out.push('\u{2026}');
        at = end + skip;
    }
    out.push_str(&s[at..]);
    out
}

/// A message from the UI, reduced to what cannot be data: no quoted text, no text after a JSON or
/// token complaint's colon, no path, no control characters, at most [`MESSAGE_MAX`] characters.
pub fn frontend_message(raw: &str) -> String {
    let s = strip_controls(&truncate_chars(raw, RAW_MAX));
    let s = cut_after_json_colon(&s);
    let s = remove_quoted(&s);
    let s = cut_at_path(&s);
    let s = drop_word_after_token(&s);
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(&s, MESSAGE_MAX)
}

/// The kind of a UI error: one of three known words, or "other".
pub fn frontend_kind(raw: &str) -> &'static str {
    match raw {
        "error" => "error",
        "unhandledrejection" => "unhandledrejection",
        "react" => "react",
        _ => "other",
    }
}

/// The script a UI error came from: only the file name of a script or page (`index-AbC.js`), without
/// folders, query or fragment, in a safe set of characters. Anything else (a document the person
/// picked, for example) is "unknown".
pub fn frontend_source(raw: &str) -> String {
    let raw = truncate_chars(raw, RAW_MAX);
    let no_query = raw.split(['?', '#']).next().unwrap_or("");
    let name = no_query.rsplit(['/', '\\']).next().unwrap_or("");
    let kept: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')).collect();
    let ext = kept.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if kept.contains('.') && matches!(ext.as_str(), "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "html") {
        truncate_chars(&kept, SOURCE_MAX)
    } else {
        "unknown".to_string()
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
    fn quoted_text_is_removed() {
        let out = remove_quoted(r#"Cannot read 'title' of "my note" and `code` and “a” ‘b’"#);
        for leaked in ["title", "my note", "code", "a\u{201d}", "b\u{2019}"] {
            assert!(!out.contains(leaked), "{out}");
        }
        assert!(remove_quoted("it's fine, don't worry").contains("don't"));
        // A nested apostrophe does not end the quote early.
        assert!(!remove_quoted("see 'don't tell' now").contains("tell"));
        // An unclosed quote removes the rest.
        assert_eq!(remove_quoted("start \"unclosed secret"), "start \u{2026}");
    }

    #[test]
    fn json_and_token_complaints_lose_their_input() {
        let m = frontend_message(r#"Unexpected token 'h', "hello wor"... is not valid JSON"#);
        assert!(!m.contains("hello") && !m.contains("'h'"), "{m}");
        let m = frontend_message("SyntaxError: JSON.parse: unexpected character: my secret note at line 1");
        assert!(!m.contains("secret"), "{m}");
        let m = frontend_message("Unexpected token h in JSON at position 0");
        assert!(!m.contains(" h "), "{m}");
        let m = frontend_message("Unexpected token Z");
        assert!(!m.contains('Z'), "{m}");
    }

    #[test]
    fn paths_and_urls_are_cut() {
        let m = frontend_message(r"Could not read C:\Users\x\secret.docx because of my notes");
        assert!(!m.contains("secret") && !m.contains("Users") && !m.contains("notes"), "{m}");
        let m = frontend_message("failed to load https://tauri.localhost/assets/a.js?q=note");
        assert!(!m.contains("note") && !m.contains("assets"), "{m}");
        let m = frontend_message("open /home/me/docs/diary failed");
        assert!(!m.contains("diary"), "{m}");
    }

    #[test]
    fn controls_are_removed_and_length_is_capped() {
        let m = frontend_message("line one\nFAKE 2026-01-01T00:00:00Z ERROR forged\r\u{0}\u{202e}end");
        assert!(!m.contains('\n') && !m.contains('\r') && !m.contains('\u{0}') && !m.contains('\u{202e}'), "{m:?}");
        let long = frontend_message(&"x".repeat(100_000));
        assert_eq!(long.chars().count(), MESSAGE_MAX);
    }

    #[test]
    fn sources_keep_only_a_script_file_name() {
        assert_eq!(frontend_source("http://tauri.localhost/assets/index-AbC123.js?x=1#y"), "index-AbC123.js");
        assert_eq!(frontend_source(r"C:\Users\x\secret.docx"), "unknown");
        assert_eq!(frontend_source("<script>alert(1)</script>"), "unknown");
        assert_eq!(frontend_source(&format!("{}.js", "a".repeat(500))).len(), SOURCE_MAX);
    }

    #[test]
    fn kinds_come_from_a_fixed_list() {
        assert_eq!(frontend_kind("error"), "error");
        assert_eq!(frontend_kind("unhandledrejection"), "unhandledrejection");
        assert_eq!(frontend_kind("react"), "react");
        assert_eq!(frontend_kind("my secret note"), "other");
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
