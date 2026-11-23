//! The files of one import, as `/`-separated paths, whether they come from a folder, from
//! files picked in the dialog, or from ZIP files (a Notion export, possibly with nested
//! `Part-N.zip` files). Reads are capped per file and in total.
//!
//! On disk: symbolic links and junctions are not followed, hidden files are skipped, and a
//! picture next to a picked file is read only if it lies inside that file's folder (checked
//! again after resolving the real path), so a document cannot point the import at any other
//! file or at a network share. Pictures are never read from Notaya's own data folder.
//!
//! The picked ZIPs are read on demand from open files. Payloads share one [`MAX_TOTAL`]
//! budget, including what the Word and PDF converters inflate. Notion part wrappers have
//! separate limits on file I/O and retained memory.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::zip::{Archive, is_export_part};
#[cfg(test)]
use super::zip;

/// Everything read or inflated during one import (a 2 GB file may hold more than 2 GB once unpacked).
pub const MAX_TOTAL: u64 = 4 * 1024 * 1024 * 1024;
/// Most files looked at in a folder.
const MAX_WALK: usize = 50_000;
const MAX_DEPTH: usize = 24;
const MAX_ARCHIVES: usize = 64;
const MAX_REFUSED: usize = 10_000;
const MAX_PICKED_ZIP_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_NESTED_ZIP_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_NESTED_ZIP_MEMORY: u64 = 128 * 1024 * 1024;
const MAX_MOUNTED_DIRECTORY_BYTES: u64 = 128 * 1024 * 1024;

enum Origin {
    Disk(PathBuf),
    Zip(usize, usize, Option<ExportFormat>),
}

#[derive(Clone, Copy)]
enum ExportFormat {
    Html,
    Markdown,
}

pub struct Vfs {
    files: BTreeMap<String, Origin>,
    lower: HashMap<String, String>,
    /// Picked files: pictures are also looked up lazily below this (canonical) folder.
    lazy_root: Option<PathBuf>,
    archives: Vec<Archive>,
    budget: Cell<u64>,
    picked_zip_bytes: u64,
    nested_zip_bytes: u64,
    nested_zip_memory: u64,
    /// Central directory bytes across picked and nested ZIPs, including rejected entries.
    mounted_directory_bytes: u64,
    /// Total archive entries inspected across mounted ZIPs (prevents hidden/nested bypasses).
    mounted_entries: usize,
    /// Notaya's data folder (canonical): nothing inside it is read as a picture.
    protected: Option<PathBuf>,
    /// Entries or files that were refused while listing, with the reason.
    pub refused: Vec<(String, String)>,
}

/// True for a name that ends in `.zip` (any case).
pub fn is_zip_name(name: &str) -> bool {
    name.len() > 4 && name.is_char_boundary(name.len() - 4) && name[name.len() - 4..].eq_ignore_ascii_case(".zip")
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name == "__MACOSX" || name.eq_ignore_ascii_case("thumbs.db") || name.eq_ignore_ascii_case("desktop.ini")
}

fn split_name(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

fn extension(path: &str) -> &str {
    split_name(path).1.rsplit_once('.').map_or("", |(_, ext)| ext)
}

fn export_format(archive: &Archive) -> Option<ExportFormat> {
    let mut html = 0;
    let mut markdown = 0;
    for entry in &archive.entries {
        if entry.name.split('/').any(is_hidden) {
            continue;
        }
        let ext = extension(&entry.name);
        if ["md", "markdown", "mdown"].iter().any(|kind| ext.eq_ignore_ascii_case(kind)) {
            markdown += 1;
        } else if ["html", "htm", "xhtml"].iter().any(|kind| ext.eq_ignore_ascii_case(kind))
            && !split_name(&entry.name).1.eq_ignore_ascii_case("index.html")
        {
            html += 1;
        }
    }
    match markdown.cmp(&html) {
        std::cmp::Ordering::Greater => Some(ExportFormat::Markdown),
        std::cmp::Ordering::Less => Some(ExportFormat::Html),
        std::cmp::Ordering::Equal => None,
    }
}

/// A wrapper's parts are siblings. An index page from the exporter may accompany them, but
/// a direct export with its own documents treats nested ZIP files as ordinary attachments.
fn wrapper_folder(archive: &Archive) -> Option<&str> {
    let visible = || archive.entries.iter().filter(|e| !e.name.split('/').any(is_hidden));
    let mut folders = visible().filter(|e| is_export_part(&e.name)).map(|e| split_name(&e.name).0);
    let folder = folders.next()?;
    if !folders.all(|f| f == folder) || visible().any(|e| {
        let (dir, name) = split_name(&e.name);
        let ext = extension(name);
        ["html", "htm", "xhtml", "md", "markdown", "mdown", "txt", "text", "docx", "pdf", "csv"]
            .iter().any(|kind| ext.eq_ignore_ascii_case(kind))
            && !(dir == folder && name.eq_ignore_ascii_case("index.html"))
    }) {
        return None;
    }
    Some(folder)
}

impl Vfs {
    fn empty() -> Vfs {
        Vfs { files: BTreeMap::new(), lower: HashMap::new(), lazy_root: None, archives: Vec::new(), budget: Cell::new(MAX_TOTAL), picked_zip_bytes: 0, nested_zip_bytes: 0, nested_zip_memory: 0, mounted_directory_bytes: 0, mounted_entries: 0, protected: None, refused: Vec::new() }
    }

    fn push_refused(&mut self, name: String, reason: String) {
        if self.refused.len() < MAX_REFUSED {
            self.refused.push((name, reason));
        }
    }

    fn add(&mut self, path: String, origin: Origin) {
        if let Some(current) = self.files.get_mut(&path) {
            if extension(&path).eq_ignore_ascii_case("csv")
                && matches!(current, Origin::Zip(_, _, Some(ExportFormat::Html)))
                && matches!(origin, Origin::Zip(_, _, Some(ExportFormat::Markdown)))
            {
                *current = origin;
            }
            return;
        }
        self.lower.entry(path.to_lowercase()).or_insert_with(|| path.clone());
        self.files.insert(path, origin);
    }

    /// Files picked in the dialog, in any mix: documents (all from one folder) and ZIP files
    /// (for example Notion exports; part wrappers are opened one level deep). Pictures a
    /// picked document uses are read from its folder and below, on demand.
    pub fn from_files(paths: &[PathBuf]) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        let mut first_doc: Option<&Path> = None;
        for p in paths {
            let name = p.file_name().and_then(|n| n.to_str()).ok_or("A file name could not be read.")?.to_string();
            let meta = std::fs::symlink_metadata(p).map_err(|e| format!("The file could not be read ({e})."))?;
            if !meta.is_file() || meta.file_type().is_symlink() || is_reparse_point(&meta) {
                if is_zip_name(&name) {
                    return Err("A link, junction or folder cannot be opened as a ZIP file.".into());
                }
                v.refused.push((name, "A link or junction (not followed).".into()));
                continue;
            }
            if is_zip_name(&name) {
                v.mount_zip_file(p)?;
                continue;
            }
            if first_doc.is_none() {
                first_doc = p.parent();
            }
            if v.files.contains_key(&name) {
                v.refused.push((name, "Another chosen file has the same name.".into()));
                continue;
            }
            v.add(name, Origin::Disk(p.clone()));
        }
        v.lazy_root = first_doc.and_then(|parent| std::fs::canonicalize(parent).ok());
        Ok(v)
    }

    fn mount_zip_file(&mut self, p: &Path) -> Result<(), String> {
        if self.archives.len() >= MAX_ARCHIVES {
            return Err(format!("More than {MAX_ARCHIVES} ZIP archives. Import them in smaller parts."));
        }
        let room = MAX_PICKED_ZIP_BYTES.saturating_sub(self.picked_zip_bytes);
        let (file, len) = open_plain_bounded(p, room, "The ZIP files chosen are larger than 16 GB together.", self.protected.as_deref())?;
        self.picked_zip_bytes += len;
        self.mount(Archive::open_file(file, len)?, true)
    }

    /// What may still be read or inflated in this import. Word and PDF converters draw from it
    /// too, so no mix of files can use more than [`MAX_TOTAL`] together.
    pub fn budget(&self) -> &Cell<u64> {
        &self.budget
    }

    /// Marks Notaya's data folder: no picture is ever read from inside it, even when a picked
    /// document sits in a folder above it and points down into it.
    pub fn protect(&mut self, data_dir: &Path) {
        let data = std::fs::canonicalize(data_dir).unwrap_or_else(|_| data_dir.to_path_buf());
        if self.lazy_root.as_ref().is_some_and(|r| r.starts_with(&data)) {
            self.lazy_root = None;
        }
        self.protected = Some(data);
    }

    /// Every file in a folder and its subfolders (not following links).
    pub fn from_folder(root: &Path) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        let mut stack = vec![(root.to_path_buf(), String::new(), 0usize)];
        let mut seen = 0usize;
        while let Some((dir, prefix, depth)) = stack.pop() {
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(e) => {
                    v.refused.push((if prefix.is_empty() { ".".into() } else { prefix.clone() }, format!("The folder could not be read ({e}).")));
                    continue;
                }
            };
            for entry in entries.flatten() {
                seen += 1;
                if seen > MAX_WALK {
                    return Err(format!("The folder has more than {MAX_WALK} files. Choose a smaller folder."));
                }
                let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
                if is_hidden(&name) {
                    continue;
                }
                let Ok(meta) = std::fs::symlink_metadata(entry.path()) else { continue };
                let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
                if meta.file_type().is_symlink() || is_reparse_point(&meta) {
                    v.refused.push((rel, "A link or junction (not followed).".into()));
                } else if meta.is_dir() {
                    if depth + 1 < MAX_DEPTH {
                        stack.push((entry.path(), rel, depth + 1));
                    } else {
                        v.refused.push((rel, "Folders nested too deeply.".into()));
                    }
                } else if meta.is_file() {
                    v.add(rel, Origin::Disk(entry.path()));
                }
            }
        }
        Ok(v)
    }

    #[cfg(test)]
    pub fn from_zip_bytes(data: Vec<u8>) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        v.mount(Archive::open(data)?, true)?;
        Ok(v)
    }

    fn mount(&mut self, mut archive: Archive, nested_ok: bool) -> Result<(), String> {
        if self.archives.len() >= MAX_ARCHIVES {
            self.push_refused("archive.zip".into(), format!("More than {MAX_ARCHIVES} ZIP archives (not opened)."));
            return Ok(());
        }
        if archive.directory_size > MAX_MOUNTED_DIRECTORY_BYTES - self.mounted_directory_bytes {
            return Err("The ZIP directories exceed 128 MB together. Import the ZIPs in smaller groups.".into());
        }
        self.mounted_directory_bytes += archive.directory_size;
        let idx = self.archives.len();
        let wrapper = nested_ok.then(|| wrapper_folder(&archive)).flatten().map(str::to_owned);
        let format = export_format(&archive);
        let mut inner = Vec::new();
        for (name, reason) in std::mem::take(&mut archive.rejected) {
            self.push_refused(name, reason);
        }
        let remaining = MAX_WALK - self.mounted_entries;
        let overflow = format!("More than {MAX_WALK} files across the chosen archives.");
        if archive.entries.len() > remaining {
            for entry in archive.entries.drain(remaining..) {
                self.push_refused(entry.name, overflow.clone());
            }
            archive.entries.shrink_to_fit();
        }
        for (i, e) in archive.entries.iter().enumerate() {
            self.mounted_entries += 1;
            if self.files.len() >= MAX_WALK {
                self.push_refused(e.name.clone(), overflow.clone());
                continue;
            }
            if e.name.split('/').any(is_hidden) {
                continue;
            }
            if is_export_part(&e.name) && wrapper.as_deref() == Some(split_name(&e.name).0) {
                inner.push(i);
                continue;
            }
            if is_export_part(&e.name) && !nested_ok {
                self.push_refused(e.name.clone(), "A ZIP file inside a ZIP file inside a ZIP file (not opened).".into());
                continue;
            }
            self.add(e.name.clone(), Origin::Zip(idx, i, format));
        }
        self.archives.push(archive);
        for i in inner {
            let entry = &self.archives[idx].entries[i];
            if self.mounted_entries >= MAX_WALK || self.files.len() >= MAX_WALK || self.archives.len() >= MAX_ARCHIVES {
                self.push_refused(entry.name.clone(), "Archive limit reached across chosen archives.".into());
                continue;
            }
            if entry.size > MAX_NESTED_ZIP_BYTES - self.nested_zip_bytes {
                self.push_refused(entry.name.clone(), "The export parts exceed 8 GB together; choose smaller groups of part ZIPs.".into());
                continue;
            }
            self.nested_zip_bytes += entry.size;
            let memory = MAX_NESTED_ZIP_MEMORY - self.nested_zip_memory;
            match self.archives[idx].open_part(i, memory) {
                Ok((part, memory_used)) => match self.mount(part, false) {
                    Ok(()) => self.nested_zip_memory += memory_used,
                    Err(e) => self.push_refused(self.archives[idx].entries[i].name.clone(), e),
                },
                Err(e) => self.push_refused(self.archives[idx].entries[i].name.clone(), e),
            }
        }
        Ok(())
    }

    /// True when some files come out of a ZIP file.
    pub fn has_archives(&self) -> bool {
        !self.archives.is_empty()
    }

    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.files.keys()
    }

    /// CSVs from the Markdown & CSV export take precedence when both Notion formats are picked.
    pub fn markdown_zip_preferred(&self, path: &str) -> bool {
        matches!(self.files.get(path), Some(Origin::Zip(_, _, Some(ExportFormat::Markdown))))
    }

    pub fn html_zip_source(&self, path: &str) -> bool {
        matches!(self.files.get(path), Some(Origin::Zip(_, _, Some(ExportFormat::Html))))
    }

    /// The stored path for `path` (exact, then ignoring case), or a picture found next to
    /// picked files.
    pub fn find(&self, path: &str) -> Option<String> {
        if self.files.contains_key(path) {
            return Some(path.to_string());
        }
        if let Some(p) = self.lower.get(&path.to_lowercase()) {
            return Some(p.clone());
        }
        self.lazy_path(path).map(|_| path.to_string())
    }

    /// A file below the picked files' folder, if it is a plain file inside it.
    fn lazy_path(&self, rel: &str) -> Option<PathBuf> {
        let root = self.lazy_root.as_ref()?;
        if rel.split('/').any(|p| p.is_empty() || p == "." || p == ".." || p.contains(':') || is_hidden(p)) {
            return None;
        }
        let candidate = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        let meta = std::fs::symlink_metadata(&candidate).ok()?;
        if !meta.is_file() || meta.file_type().is_symlink() || is_reparse_point(&meta) {
            return None;
        }
        let real = std::fs::canonicalize(&candidate).ok()?;
        let outside_data = !self.protected.as_ref().is_some_and(|d| real.starts_with(d));
        (real.starts_with(root) && outside_data).then_some(real)
    }

    /// Reads a file, refused when larger than `max` or beyond the total budget.
    pub fn read(&self, path: &str, max: u64) -> Result<Vec<u8>, String> {
        let disk = |p: &Path| -> Result<Vec<u8>, String> {
            let limit = max.min(self.budget.get());
            let too_large = if limit < max {
                "The import is too large in total (more than 4 GB of content).".to_string()
            } else {
                format!("Too large (the limit is {} MB).", max / (1024 * 1024))
            };
            let data = read_plain_bounded(p, limit, &too_large, self.protected.as_deref())?;
            self.budget.set(self.budget.get().saturating_sub(data.len() as u64));
            Ok(data)
        };
        match self.files.get(path) {
            Some(Origin::Disk(p)) => disk(p),
            Some(Origin::Zip(a, i, _)) => self.archives[*a].read(*i, max, &self.budget).map_err(|e| {
                if e == "Too large." { format!("Too large (the limit is {} MB).", max / (1024 * 1024)) } else { e }
            }),
            None => match self.lazy_path(path) {
                Some(p) => disk(&p),
                None => Err("Not found.".into()),
            },
        }
    }
}

#[cfg(windows)]
fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_meta: &std::fs::Metadata) -> bool {
    false
}

/// Reads a plain disk file (never a symlink/junction or anything under `protected`) up to `max` bytes.
fn read_plain_bounded(p: &Path, max: u64, too_large_err: &str, protected: Option<&Path>) -> Result<Vec<u8>, String> {
    let (file, _) = open_plain_bounded(p, max, too_large_err, protected)?;
    let mut data = Vec::new();
    file.take(max.saturating_add(1)).read_to_end(&mut data).map_err(|e| format!("Could not be read ({e})."))?;
    if data.len() as u64 > max {
        return Err(too_large_err.into());
    }
    Ok(data)
}

fn open_plain_bounded(p: &Path, max: u64, too_large_err: &str, protected: Option<&Path>) -> Result<(std::fs::File, u64), String> {
    let meta = std::fs::symlink_metadata(p).map_err(|e| format!("Could not be read ({e})."))?;
    if !meta.is_file() || meta.file_type().is_symlink() || is_reparse_point(&meta) {
        return Err("A link or junction (not followed).".into());
    }
    if let Some(root) = protected {
        let real = std::fs::canonicalize(p).map_err(|e| format!("Could not be read ({e})."))?;
        if real.starts_with(root) {
            return Err("Refused (inside Notaya's data folder).".into());
        }
    }
    if meta.len() > max {
        return Err(too_large_err.into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000);
    }
    let file = options.open(p).map_err(|e| format!("Could not be read ({e})."))?;
    let fmeta = file.metadata().map_err(|e| format!("Could not be read ({e})."))?;
    if !fmeta.is_file() || fmeta.file_type().is_symlink() || is_reparse_point(&fmeta) {
        return Err("A link or junction (not followed).".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (fmeta.dev(), fmeta.ino()) != (meta.dev(), meta.ino()) {
            return Err("A link or junction (not followed).".into());
        }
    }
    if fmeta.len() != meta.len() || fmeta.len() > max {
        return Err(too_large_err.into());
    }
    Ok((file, fmeta.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> PathBuf {
        let d = std::env::temp_dir().join(format!("notaya-import-{}", crate::model::new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_folder_is_listed_without_hidden_files() {
        let d = temp();
        std::fs::create_dir_all(d.join("Work/Deep")).unwrap();
        std::fs::write(d.join("a.md"), "# A").unwrap();
        std::fs::write(d.join(".hidden.md"), "x").unwrap();
        std::fs::write(d.join("Work/Deep/b.md"), "b").unwrap();
        let v = Vfs::from_folder(&d).unwrap();
        let paths: Vec<&String> = v.paths().collect();
        assert_eq!(paths, ["Work/Deep/b.md", "a.md"]);
        assert_eq!(v.read("a.md", 100).unwrap(), b"# A");
        assert!(v.read("a.md", 2).unwrap_err().starts_with("Too large"));
        assert_eq!(v.find("WORK/deep/B.md").as_deref(), Some("Work/Deep/b.md"));
    }

    #[test]
    fn pictures_next_to_picked_files_stay_inside_their_folder() {
        let d = temp();
        let inner = d.join("notes");
        std::fs::create_dir_all(inner.join("img")).unwrap();
        std::fs::write(inner.join("page.md"), "![](img/x.png)").unwrap();
        std::fs::write(inner.join("img/x.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        std::fs::write(d.join("secret.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        let v = Vfs::from_files(&[inner.join("page.md")]).unwrap();
        assert_eq!(v.find("img/x.png").as_deref(), Some("img/x.png"));
        assert!(v.read("img/x.png", 100).is_ok());
        assert_eq!(v.find("../secret.png"), None);
        assert!(v.read("../secret.png", 100).is_err());
        assert!(v.read("img/../../secret.png", 100).is_err());
    }

    #[test]
    fn nested_zips_are_opened_one_level() {
        let deepest = zip::build(&[("x.md", b"x")], false);
        let part = zip::build(&[("Page 0123456789abcdef0123456789abcdef.md", b"# Page"), ("Part-3.zip", &deepest)], true);
        let outer = zip::build(&[("Wrapper/Export-Part-1.zip", &part), ("Wrapper/Export-Part-2.zip", &zip::build(&[("Second.md", b"# Second")], false))], false);
        let v = Vfs::from_zip_bytes(outer).unwrap();
        let paths: Vec<&String> = v.paths().collect();
        assert_eq!(paths, ["Page 0123456789abcdef0123456789abcdef.md", "Second.md"]);
        assert!(v.refused.iter().any(|(n, _)| n == "Part-3.zip"));
    }

    #[test]
    fn zip_attachments_are_not_opened_as_notions_parts() {
        let embedded = zip::build(&[("Unrelated.md", b"# Unrelated")], false);
        for page in ["Page.html", "Page.XHTML", "Page.mdown", "Page.text"] {
            let direct = zip::build(&[(page, b"Example content"), ("Page/Part-1.zip", &embedded)], false);
            let v = Vfs::from_zip_bytes(direct).unwrap();
            assert!(v.find("Unrelated.md").is_none(), "{page}");
            assert!(v.find("Page/Part-1.zip").is_some(), "{page}");
        }

        let wrapped = zip::build(&[("My-Files-Part-1.zip", &embedded), ("archive.zip", &embedded)], true);
        let v = Vfs::from_zip_bytes(wrapped).unwrap();
        assert!(v.find("Unrelated.md").is_none());
        assert!(v.find("archive.zip").is_some());
    }

    #[test]
    fn nothing_is_read_from_the_data_folder_as_a_picture() {
        let d = temp();
        std::fs::create_dir_all(d.join("data")).unwrap();
        std::fs::write(d.join("data/pic.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        std::fs::write(d.join("pic.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        std::fs::write(d.join("a.md"), "![](data/pic.png)").unwrap();
        // The picked document sits in a folder ABOVE the data folder and points down into it.
        let mut v = Vfs::from_files(&[d.join("a.md")]).unwrap();
        assert_eq!(v.find("data/pic.png").as_deref(), Some("data/pic.png"), "readable before the data folder is marked");
        v.protect(&d.join("data"));
        assert_eq!(v.find("data/pic.png"), None);
        assert!(v.read("data/pic.png", 100).is_err());
        assert!(v.read("pic.png", 100).is_ok(), "its other pictures still are");
        // The picked document sits inside the data folder itself: no lazy pictures at all.
        std::fs::write(d.join("data/n.md"), "![](pic.png)").unwrap();
        let mut v = Vfs::from_files(&[d.join("data/n.md")]).unwrap();
        v.protect(&d.join("data"));
        assert_eq!(v.find("pic.png"), None);
        // A folder ABOVE: `data` equals the lazy root.
        let mut v = Vfs::from_files(&[d.join("data/n.md")]).unwrap();
        v.protect(&d.join("data/"));
        assert!(v.read("pic.png", 100).is_err());
    }

    #[test]
    fn picked_zip_files_share_a_file_size_allowance_without_spending_the_payload_budget() {
        let d = temp();
        let z = zip::build(&[("a.md", b"hello")], true);
        let path = d.join("one.zip");
        std::fs::write(&path, &z).unwrap();
        let mut v = Vfs::empty();
        v.picked_zip_bytes = MAX_PICKED_ZIP_BYTES - 2 * z.len() as u64;
        v.mount_zip_file(&path).unwrap();
        v.mount_zip_file(&path).unwrap();
        assert_eq!(v.picked_zip_bytes, MAX_PICKED_ZIP_BYTES);
        assert!(v.mount_zip_file(&path).unwrap_err().contains("larger than 16 GB together"));
        assert_eq!(v.budget().get(), MAX_TOTAL);
        assert_eq!(v.read("a.md", 100).unwrap(), b"hello");
        assert_eq!(v.budget().get(), MAX_TOTAL - 5);
    }

    #[test]
    fn exact_csv_collision_prefers_markdown_export_in_either_pick_order() {
        let d = temp();
        let html = d.join("a.zip");
        let md = d.join("b.zip");
        let ambiguous = d.join("c.zip");
        std::fs::write(&html, zip::build(&[("Page.html", b"<h1>Page</h1>"), ("Table.csv", b"Title\nFrom HTML")], false)).unwrap();
        std::fs::write(&md, zip::build(&[("Page.markdown", b"# Page"), ("index.html", b"index"), ("Table.csv", b"Title\nFrom Markdown")], true)).unwrap();
        std::fs::write(&ambiguous, zip::build(&[("Other.csv", b"Title\nExample")], false)).unwrap();
        for picks in [[html.clone(), md.clone(), ambiguous.clone()], [md.clone(), html.clone(), ambiguous]] {
            let v = Vfs::from_files(&picks).unwrap();
            assert!(v.markdown_zip_preferred("Table.csv"));
            assert!(!v.markdown_zip_preferred("Other.csv"));
            assert_eq!(v.read("Table.csv", 100).unwrap(), b"Title\nFrom Markdown");
        }
    }

    #[test]
    fn nested_parts_have_separate_byte_and_memory_limits_and_share_the_entry_limit() {
        let inner = zip::build(&[("inner.md", b"# Hello from inside nested zip")], true);
        let outer = zip::build(&[("part-1.zip", &inner)], false);
        let mut v = Vfs::empty();
        v.mount(Archive::open(outer).unwrap(), true).unwrap();
        assert_eq!(v.nested_zip_bytes, inner.len() as u64);
        assert_eq!(v.nested_zip_memory, 0);
        assert_eq!(v.budget().get(), MAX_TOTAL);
        assert_eq!(v.find("inner.md").as_deref(), Some("inner.md"));

        let inner2 = zip::build(&[("too-big.md", b"exceeds remaining zip_total budget")], false);
        let outer2 = zip::build(&[("part-2.zip", &inner2)], false);
        let mut v2 = Vfs::empty();
        v2.nested_zip_bytes = MAX_NESTED_ZIP_BYTES - 4;
        v2.mount(Archive::open(outer2).unwrap(), true).unwrap();
        assert_eq!(v2.find("too-big.md"), None);
        assert!(v2.refused.iter().any(|(n, _)| n == "part-2.zip"));

        let outer3 = zip::build(&[("Export-Part-1.zip", &inner)], true);
        let mut v = Vfs::from_zip_bytes(outer3.clone()).unwrap();
        assert_eq!(v.nested_zip_memory, inner.len() as u64);
        assert_eq!(v.read("inner.md", 100).unwrap(), b"# Hello from inside nested zip");
        v.nested_zip_memory = MAX_NESTED_ZIP_MEMORY;
        v.mount(Archive::open(outer3).unwrap(), true).unwrap();
        assert!(v.refused.iter().any(|(n, reason)| n == "Export-Part-1.zip" && reason.contains("memory allowance")));

        // Global MAX_WALK cap holds across multiple mounted archives (entries beyond MAX_WALK refused).
        let mut v3 = Vfs::empty();
        v3.mounted_entries = MAX_WALK - 1;
        let z_extra = zip::build(&[("kept.md", b"1"), ("overflow.md", b"2")], false);
        v3.mount(Archive::open(z_extra).unwrap(), true).unwrap();
        assert_eq!(v3.find("kept.md").as_deref(), Some("kept.md"));
        assert_eq!(v3.find("overflow.md"), None);
        assert!(v3.refused.iter().any(|(n, _)| n == "overflow.md"));
        assert_eq!(v3.archives[0].entries.len(), 1, "unmounted entry metadata is released");
    }

    #[test]
    fn picked_and_nested_archives_share_the_directory_limit() {
        let zip = zip::build(&[("Kept.md", b"yes"), ("../Refused.md", b"no")], false);
        let archive = Archive::open(zip.clone()).unwrap();
        let mut v = Vfs::empty();
        v.mounted_directory_bytes = MAX_MOUNTED_DIRECTORY_BYTES - archive.directory_size;
        v.mount(archive, true).unwrap();
        assert_eq!(v.mounted_directory_bytes, MAX_MOUNTED_DIRECTORY_BYTES);
        assert!(v.archives[0].rejected.is_empty(), "refusals are retained only by the VFS");
        assert!(v.refused.iter().any(|(name, _)| name == "../Refused.md"));
        assert!(v.mount(Archive::open(zip.clone()).unwrap(), true).unwrap_err().contains("directories exceed"));

        let outer = Archive::open(zip::build(&[("Part-1.zip", &zip)], false)).unwrap();
        let mut v = Vfs::empty();
        v.mounted_directory_bytes = MAX_MOUNTED_DIRECTORY_BYTES - outer.directory_size;
        v.mount(outer, true).unwrap();
        assert!(v.find("Kept.md").is_none());
        assert!(v.refused.iter().any(|(name, reason)| name == "Part-1.zip" && reason.contains("directories exceed")));
    }

    #[test]
    fn from_files_and_read_refuse_links_and_junctions() {
        let d = temp();
        let real = d.join("target.md");
        std::fs::write(&real, b"hello").unwrap();
        let link_md = d.join("linked.md");
        let link_dir = d.join("dir_link");
        let target_dir = d.join("real_dir");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("note.md"), b"ok").unwrap();

        #[cfg(windows)]
        let dir_linked = std::process::Command::new("cmd")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link_dir)
            .arg(&target_dir)
            .output()
            .is_ok_and(|o| o.status.success());
        #[cfg(not(windows))]
        let dir_linked = std::os::unix::fs::symlink(&target_dir, &link_dir).is_ok();

        #[cfg(windows)]
        let file_linked = std::os::windows::fs::symlink_file(&real, &link_md).is_ok();
        #[cfg(not(windows))]
        let file_linked = std::os::unix::fs::symlink(&real, &link_md).is_ok();

        if file_linked {
            let v = Vfs::from_files(&[link_md]).unwrap();
            assert!(v.refused.iter().any(|(n, r)| n == "linked.md" && r.contains("link or junction")));
            assert_eq!(v.find("linked.md"), None);
        }
        assert!(dir_linked, "directory junction/symlink must work");
        let v_folder = Vfs::from_folder(&d).unwrap();
        assert!(v_folder.refused.iter().any(|(n, r)| n == "dir_link" && r.contains("link or junction")));
        assert_eq!(v_folder.find("dir_link/note.md"), None);
        // Folder/junction passed as picked ZIP/file and read_plain_bounded size cap are always enforced.
        assert!(Vfs::from_files(&[d.join("bad.zip")]).is_err());
        let big = d.join("big.md");
        std::fs::write(&big, vec![b'a'; 64]).unwrap();
        let v_pick = Vfs::from_files(&[big.clone()]).unwrap();
        assert!(v_pick.read("big.md", 16).unwrap_err().contains("Too large"));
        assert_eq!(read_plain_bounded(&link_dir, 100, "big", None).unwrap_err(), "A link or junction (not followed).");
    }
}
