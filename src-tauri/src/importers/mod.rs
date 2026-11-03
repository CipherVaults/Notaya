//! Import from other apps: Markdown, HTML, Notion exports (ZIP of Markdown or HTML), Word
//! (.docx) and PDF. Every file is treated as hostile: sizes and counts are capped, ZIP entries
//! are checked (see `zip`), HTML is reduced to a fixed set of blocks and marks (scripts, event
//! handlers and styles never reach the editor), links keep only http, https and mailto, and
//! pictures are taken only from the chosen files (never fetched from the network; a remote
//! picture stays a plain link). The result is written as one NEW notebook, in one transaction.

pub mod doc;
pub mod docx;
pub mod html;
pub mod markdown;
pub mod markup;
pub mod palette;
pub mod pdf;
pub mod plan;
pub mod source;
pub mod zip;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;

use doc::{Inline, Marks};
use source::Vfs;

/// Largest Markdown, HTML or text file.
pub const MAX_TEXT_FILE: u64 = 20 * 1024 * 1024;
/// Largest Word or PDF file.
pub const MAX_BINARY_FILE: u64 = 100 * 1024 * 1024;
/// Largest picture (the same limit as a pasted picture).
pub const MAX_IMAGE: u64 = crate::vault::MAX_ATTACHMENT_BYTES as u64;
/// Most pages and pictures one import may create.
pub const MAX_PAGES: usize = 5_000;
pub const MAX_IMAGES: usize = 5_000;
/// Pictures held in memory before they are stored (Word media, data: URIs).
const MAX_IMAGE_MEMORY: u64 = 512 * 1024 * 1024;

/// The whole import may run this long; after that the converters stop and report it.
pub const MAX_TIME: Duration = Duration::from_secs(15 * 60);

/// Why an import was stopped early.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stop {
    Cancelled,
    TimedOut,
}

/// A deadline plus a cancel flag, checked inside every long loop of every converter. The
/// checks are cheap (one atomic load; the clock is read only every few calls).
pub struct Control {
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    ticks: AtomicU32,
}

impl Control {
    pub fn new(limit: Duration, cancel: Arc<AtomicBool>) -> Control {
        Control { deadline: Instant::now() + limit, cancel, ticks: AtomicU32::new(0) }
    }

    /// No cancel button and the normal time limit (tests).
    #[cfg(test)]
    pub fn standard() -> Control {
        Control::new(MAX_TIME, Arc::new(AtomicBool::new(false)))
    }

    /// A control that is already past its deadline (tests).
    #[cfg(test)]
    pub fn expired() -> Control {
        Control { deadline: Instant::now(), cancel: Arc::new(AtomicBool::new(false)), ticks: AtomicU32::new(0) }
    }

    /// A control with a given deadline (tests).
    #[cfg(test)]
    pub fn with_deadline(deadline: Instant) -> Control {
        Control { deadline, cancel: Arc::new(AtomicBool::new(false)), ticks: AtomicU32::new(0) }
    }

    /// Why the import must stop now, if it must. Reads the clock only on every 64th call,
    /// except that a cancel is seen at once.
    pub fn stop(&self) -> Option<Stop> {
        if self.cancel.load(Ordering::Relaxed) {
            return Some(Stop::Cancelled);
        }
        let n = self.ticks.fetch_add(1, Ordering::Relaxed);
        if n % 64 == 0 && Instant::now() >= self.deadline {
            return Some(Stop::TimedOut);
        }
        None
    }

    /// Like `stop` but always reads the clock (between documents).
    pub fn stop_now(&self) -> Option<Stop> {
        if self.cancel.load(Ordering::Relaxed) {
            return Some(Stop::Cancelled);
        }
        if Instant::now() >= self.deadline {
            return Some(Stop::TimedOut);
        }
        None
    }

    pub fn stopped(&self) -> bool {
        self.stop().is_some()
    }
}

/// One thing that was not imported, and why.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub item: String,
    pub reason: String,
}

/// Where a picture comes from. Pictures are read and checked when the import is written.
pub enum ImgSrc {
    /// A file in the chosen source.
    File(String),
    /// Bytes already in memory (a Word file's media, a data: URI).
    Bytes(Vec<u8>),
}

#[derive(Default)]
pub struct Images {
    pub list: Vec<(ImgSrc, String)>,
    by_key: HashMap<String, usize>,
    memory: u64,
}

impl Images {
    fn push(&mut self, key: String, src: ImgSrc, label: String) -> Option<usize> {
        if let Some(i) = self.by_key.get(&key) {
            return Some(*i);
        }
        if self.list.len() >= MAX_IMAGES {
            return None;
        }
        if let ImgSrc::Bytes(b) = &src {
            if self.memory + b.len() as u64 > MAX_IMAGE_MEMORY {
                return None;
            }
            self.memory += b.len() as u64;
        }
        self.list.push((src, label));
        self.by_key.insert(key, self.list.len() - 1);
        Some(self.list.len() - 1)
    }

    /// A picture given as bytes. `key` makes repeated uses share one attachment.
    pub fn bytes(&mut self, key: &str, bytes: Vec<u8>, label: &str) -> Option<usize> {
        if bytes.len() as u64 > MAX_IMAGE || sniff(&bytes).is_none() {
            return None;
        }
        self.push(format!("mem:{key}"), ImgSrc::Bytes(bytes), label.to_string())
    }
}

/// Identifies an image by its first bytes (PNG, JPEG, GIF, WebP; never SVG).
pub fn sniff(bytes: &[u8]) -> Option<&'static str> {
    crate::commands::notes::sniff_image(bytes)
}

/// What a converter needs while it reads one document.
pub struct Ctx<'a> {
    pub vfs: &'a Vfs,
    pub ctl: &'a Control,
    /// The document's folder inside the source (`""` = the top).
    pub dir: String,
    pub images: &'a mut Images,
    /// Problems found in this document (missing pictures and the like).
    pub problems: &'a mut Vec<Skipped>,
    /// The document's path, used in the problems list.
    pub name: String,
}

impl Ctx<'_> {
    pub fn problem(&mut self, what: impl Into<String>, reason: impl Into<String>) {
        if self.problems.len() < 500 {
            self.problems.push(Skipped { item: format!("{} ({})", what.into(), self.name), reason: reason.into() });
        }
    }

    /// Turns a picture reference into inline content: a stored picture, a plain link for a
    /// remote picture (never fetched), or a short note in italics when it cannot be used.
    pub fn image(&mut self, src: &str, alt: &str) -> Inline {
        let src = src.trim();
        let label = if alt.trim().is_empty() { file_label(src) } else { alt.trim().to_string() };
        if let Some(href) = safe_link(src) {
            // A picture on the internet: kept as a link, never downloaded.
            return Inline::Text(format!("Picture: {label}"), Marks { link: Some(href), ..Marks::default() });
        }
        if let Some(data) = src.strip_prefix("data:") {
            if let Some(bytes) = decode_data_uri(data) {
                let key = format!("{:08x}{}", zip::crc32(&bytes), bytes.len());
                if let Some(i) = self.images.bytes(&key, bytes, &label) {
                    return Inline::Image(i);
                }
            }
            self.problem("Embedded picture", "Not a PNG, JPEG, GIF or WebP picture, or too large.");
            return missing(&label);
        }
        match resolve_rel(&self.dir, src).and_then(|p| self.vfs.find(&p)) {
            Some(path) => match self.images.push(format!("file:{path}"), ImgSrc::File(path.clone()), label.clone()) {
                Some(i) => Inline::Image(i),
                None => {
                    self.problem(&label, "Too many pictures in one import.");
                    missing(&label)
                }
            },
            None => {
                self.problem(&label, "The picture file was not found next to the document.");
                missing(&label)
            }
        }
    }
}

fn missing(label: &str) -> Inline {
    Inline::Text(format!("[Picture not imported: {label}]"), Marks { italic: true, ..Marks::default() })
}

fn file_label(src: &str) -> String {
    let name = src.rsplit(['/', '\\']).next().unwrap_or(src);
    let name: String = percent_decode(name.split(['?', '#']).next().unwrap_or(name)).chars().take(80).collect();
    if name.is_empty() || src.starts_with("data:") { "picture".into() } else { name }
}

/// Keeps a link only if it is http, https or mailto. Control characters are removed first
/// (browsers ignore them, so `java\tscript:` must not slip through).
pub fn safe_link(href: &str) -> Option<String> {
    let h: String = href.trim().chars().filter(|c| !c.is_control()).collect();
    if h.len() > 2048 {
        return None;
    }
    let colon = h.find(':')?;
    let scheme = h[..colon].trim().to_ascii_lowercase();
    let rest = &h[colon + 1..];
    match scheme.as_str() {
        "http" | "https" if rest.starts_with("//") && rest.len() > 2 => Some(h.trim().to_string()),
        "mailto" if rest.contains('@') => Some(h.trim().to_string()),
        _ => None,
    }
}

/// Marks a link to another page of the same import (a relative address of a document that names a
/// file in the source) for the renderer, which turns it into an internal Notaya link once the
/// page ids are known, or drops it when that page was not imported.
pub const PAGE_HREF: &str = "notaya-src:";

/// True when a relative link names a page document: Markdown or HTML (how Notion and Notaya's
/// own ZIP export link their pages), Word (Notaya's own Word ZIP) or PDF (an attachment that the
/// import turns into a page). Query and fragment are ignored.
pub fn is_page_address(href: &str) -> bool {
    let lower = href.split(['?', '#']).next().unwrap_or("").to_ascii_lowercase();
    [".md", ".markdown", ".html", ".htm", ".docx", ".pdf"].iter().any(|ext| lower.ends_with(ext))
}

impl Ctx<'_> {
    pub fn page_href(&self, href: &str) -> Option<String> {
        if !is_page_address(href) {
            return None;
        }
        let path = resolve_rel(&self.dir, href).and_then(|p| self.vfs.find(&p))?;
        Some(format!("{PAGE_HREF}{path}"))
    }
}

/// Decodes `%xx` escapes (invalid ones are kept as they are).
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = ((b[i + 1] as char).to_digit(16), (b[i + 2] as char).to_digit(16)) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Resolves a relative reference inside the source. `None` for anything that is not a
/// plain relative path: a URL or other scheme, an absolute or UNC path, a drive letter, or a
/// path that climbs out of the source with `..`.
pub fn resolve_rel(dir: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('/') || href.starts_with('\\') || href.starts_with('#') {
        return None;
    }
    let path = href.split(['?', '#']).next().unwrap_or("");
    let path = percent_decode(path).replace('\\', "/");
    if path.contains(':') || path.chars().any(|c| c.is_control()) || path.starts_with('/') {
        return None;
    }
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            p => parts.push(p),
        }
    }
    if parts.is_empty() { None } else { Some(parts.join("/")) }
}

/// Decodes the part of a data: URI after `data:` when it is base64 (`image/png;base64,…`).
fn decode_data_uri(data: &str) -> Option<Vec<u8>> {
    let (meta, body) = data.split_once(',')?;
    if !meta.to_ascii_lowercase().ends_with(";base64") || body.len() as u64 > MAX_IMAGE * 4 / 3 + 16 {
        return None;
    }
    base64(body)
}

/// Standard base64 (padding optional, whitespace ignored).
pub fn base64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            c if c.is_ascii_whitespace() => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// A clean title from a file or folder name: the extension (if `strip_ext`) and the 32-hex
/// id Notion adds to every name are removed.
pub fn clean_title(name: &str, strip_ext: bool) -> String {
    let mut s = name.trim();
    if strip_ext {
        if let Some(dot) = s.rfind('.') {
            if s.len() - dot <= 9 {
                s = &s[..dot];
            }
        }
    }
    let s = strip_notion_id(s);
    let s = doc::collapse_ws(&s);
    let s = if s.is_empty() { "Untitled".to_string() } else { s };
    s.chars().take(300).collect()
}

/// Removes a trailing Notion id: ` 0123…(32 hex)` or a dashed UUID.
pub fn strip_notion_id(s: &str) -> String {
    let trimmed = s.trim_end();
    for len in [32usize, 36] {
        if trimmed.len() > len && trimmed.is_char_boundary(trimmed.len() - len) {
            let (head, tail) = trimmed.split_at(trimmed.len() - len);
            let hex_ok = if len == 32 {
                tail.bytes().all(|b| b.is_ascii_hexdigit())
            } else {
                tail.bytes().enumerate().all(|(i, b)| if [8, 13, 18, 23].contains(&i) { b == b'-' } else { b.is_ascii_hexdigit() })
            };
            if hex_ok && head.ends_with([' ', '-', '_']) {
                return head[..head.len() - 1].trim_end().to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Text from file bytes: UTF-8 (BOM removed), or UTF-16 with a BOM.
pub fn decode_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    let utf16 = |le: bool, b: &[u8]| {
        let units: Vec<u16> = b.chunks_exact(2).map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) }).collect();
        String::from_utf16_lossy(&units)
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(true, rest);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(false, rest);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        // Not UTF-8: read it as Windows-1252, the usual alternative for old files.
        Err(_) => bytes.iter().map(|&b| pdf::win_ansi(b)).collect(),
    }
}

/// Runs `f`, and stops the whole test run with a message if it has not finished after `secs`
/// seconds (a hang in a regression test must fail fast, not stall the suite). Also fails when
/// it finished but took longer.
/// Heavy hostile-input tests take this lock so only one of them runs at a time: together they
/// are many times slower than one after the other (allocator contention, notably under Wine),
/// and a time limit would then measure the machine and not the code.
#[cfg(test)]
pub fn heavy() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
pub fn within<T>(secs: u64, what: &str, f: impl FnOnce() -> T) -> T {
    let done = Arc::new(AtomicBool::new(false));
    let watch = done.clone();
    let label = what.to_string();
    std::thread::spawn(move || {
        let start = Instant::now();
        while start.elapsed().as_secs() < secs.saturating_mul(2) {
            if watch.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        eprintln!("HANG: {label} did not finish in {}s", secs * 2);
        std::process::exit(99);
    });
    let start = Instant::now();
    let r = f();
    done.store(true, Ordering::SeqCst);
    assert!(start.elapsed().as_secs() < secs, "{what} took {:?}", start.elapsed());
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_safe_links_survive() {
        assert_eq!(safe_link(" https://example.com/a?b=1 ").as_deref(), Some("https://example.com/a?b=1"));
        assert_eq!(safe_link("HTTP://x.y").as_deref(), Some("HTTP://x.y"));
        assert_eq!(safe_link("mailto:a@b.c").as_deref(), Some("mailto:a@b.c"));
        for bad in ["javascript:alert(1)", "java\tscript:alert(1)", " JaVaScRiPt:x", "data:text/html,<script>", "vbscript:x", "file:///C:/x", "\\\\server\\share", "page.md", "http:", "https:/x", "mailto:nobody"] {
            assert_eq!(safe_link(bad), None, "{bad}");
        }
    }

    #[test]
    fn relative_paths_stay_inside_the_source() {
        assert_eq!(resolve_rel("a/b", "img%20one.png").as_deref(), Some("a/b/img one.png"));
        assert_eq!(resolve_rel("a/b", "../c/x.png?v=1#f").as_deref(), Some("a/c/x.png"));
        for bad in ["../../x.png", "/etc/x.png", "C:/x.png", "\\\\host\\share\\x.png", "https://x/y.png", "file:x.png", "..%2F..%2F..%2Fx.png"] {
            assert_eq!(resolve_rel("a", bad), None, "{bad}");
        }
    }

    #[test]
    fn notion_ids_are_removed_from_titles() {
        assert_eq!(clean_title("Meeting notes 1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d.md", true), "Meeting notes");
        assert_eq!(clean_title("Plan 1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d", false), "Plan");
        assert_eq!(clean_title("1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d.md", true), "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d");
        assert_eq!(clean_title("Report v2.final.docx", true), "Report v2.final");
        assert_eq!(clean_title("   .md", true), "Untitled");
    }

    #[test]
    fn base64_and_text_decoding() {
        assert_eq!(base64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64("aGVs\nbG8").unwrap(), b"hello");
        assert!(base64("a$b").is_none());
        assert_eq!(decode_text(b"\xEF\xBB\xBFhi"), "hi");
        assert_eq!(decode_text(b"\xFF\xFEh\0i\0"), "hi");
        assert_eq!(decode_text(b"caf\xE9"), "café");
        assert_eq!(percent_decode("a%20b%zz%2"), "a b%zz%2");
    }
}
