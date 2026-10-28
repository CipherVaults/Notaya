//! The files of one import, as `/`-separated paths, whether they come from a folder, from
//! files picked in the dialog, or from ZIP files (a Notion export, possibly with nested
//! `Part-N.zip` files). Reads are capped per file and in total.
//!
//! On disk: symbolic links and junctions are not followed, hidden files are skipped, and a
//! picture next to a picked file is read only if it lies inside that file's folder (checked
//! again after resolving the real path), so a document cannot point the import at any other
//! file or at a network share.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use super::zip::Archive;
#[cfg(test)]
use super::zip;

/// Everything read or inflated during one import.
pub const MAX_TOTAL: u64 = 2 * 1024 * 1024 * 1024;
/// Most files looked at in a folder.
const MAX_WALK: usize = 50_000;
const MAX_DEPTH: usize = 24;
/// Largest ZIP file read into memory.
pub const MAX_ZIP_FILE: u64 = 1024 * 1024 * 1024;

enum Origin {
    Disk(PathBuf),
    Zip(usize, usize),
}

pub struct Vfs {
    files: BTreeMap<String, Origin>,
    lower: HashMap<String, String>,
    /// Picked files: pictures are also looked up lazily below this (canonical) folder.
    lazy_root: Option<PathBuf>,
    archives: Vec<Archive>,
    budget: Cell<u64>,
    /// Entries or files that were refused while listing, with the reason.
    pub refused: Vec<(String, String)>,
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name == "__MACOSX" || name.eq_ignore_ascii_case("thumbs.db") || name.eq_ignore_ascii_case("desktop.ini")
}

impl Vfs {
    fn empty() -> Vfs {
        Vfs { files: BTreeMap::new(), lower: HashMap::new(), lazy_root: None, archives: Vec::new(), budget: Cell::new(MAX_TOTAL), refused: Vec::new() }
    }

    fn add(&mut self, path: String, origin: Origin) {
        if self.files.contains_key(&path) {
            return;
        }
        self.lower.entry(path.to_lowercase()).or_insert_with(|| path.clone());
        self.files.insert(path, origin);
    }

    /// Files picked in the dialog (all from one folder). Pictures they use are read from
    /// that folder and below, on demand.
    pub fn from_files(paths: &[PathBuf]) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        for p in paths {
            let name = p.file_name().and_then(|n| n.to_str()).ok_or("A file name could not be read.")?.to_string();
            v.add(name, Origin::Disk(p.clone()));
        }
        if let Some(parent) = paths.first().and_then(|p| p.parent()) {
            v.lazy_root = std::fs::canonicalize(parent).ok();
        }
        Ok(v)
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

    /// One or more ZIP files (for example a Notion export). ZIP files inside them are opened
    /// too, one level deep (newer Notion exports put `Export-…-Part-1.zip` inside the ZIP).
    pub fn from_zips(paths: &[PathBuf]) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        for p in paths {
            let len = std::fs::metadata(p).map_err(|e| format!("The file could not be read ({e}).")).map(|m| m.len())?;
            if len > MAX_ZIP_FILE {
                return Err("This ZIP file is larger than 1 GB. Export it in smaller parts.".into());
            }
            let data = std::fs::read(p).map_err(|e| format!("The file could not be read ({e})."))?;
            v.mount(Archive::open(data)?, true)?;
        }
        Ok(v)
    }

    #[cfg(test)]
    pub fn from_zip_bytes(data: Vec<u8>) -> Result<Vfs, String> {
        let mut v = Vfs::empty();
        v.mount(Archive::open(data)?, true)?;
        Ok(v)
    }

    fn mount(&mut self, archive: Archive, nested_ok: bool) -> Result<(), String> {
        let idx = self.archives.len();
        let mut inner = Vec::new();
        for (name, reason) in &archive.rejected {
            self.refused.push((name.clone(), reason.clone()));
        }
        for (i, e) in archive.entries.iter().enumerate() {
            if e.name.split('/').any(is_hidden) {
                continue;
            }
            if e.name.to_ascii_lowercase().ends_with(".zip") {
                if nested_ok {
                    inner.push(i);
                } else {
                    self.refused.push((e.name.clone(), "A ZIP file inside a ZIP file inside a ZIP file (not opened).".into()));
                }
                continue;
            }
            self.add(e.name.clone(), Origin::Zip(idx, i));
        }
        let mut nested = Vec::new();
        for i in inner {
            match archive.read(i, MAX_ZIP_FILE, &self.budget).and_then(Archive::open) {
                Ok(a) => nested.push(a),
                Err(e) => self.refused.push((archive.entries[i].name.clone(), e)),
            }
        }
        self.archives.push(archive);
        for a in nested {
            self.mount(a, false)?;
        }
        Ok(())
    }

    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.files.keys()
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
        real.starts_with(root).then_some(real)
    }

    /// Reads a file, refused when larger than `max` or beyond the total budget.
    pub fn read(&self, path: &str, max: u64) -> Result<Vec<u8>, String> {
        let disk = |p: &Path| -> Result<Vec<u8>, String> {
            let len = std::fs::metadata(p).map_err(|e| format!("Could not be read ({e}).")).map(|m| m.len())?;
            if len > max {
                return Err(format!("Too large (the limit is {} MB).", max / (1024 * 1024)));
            }
            if len > self.budget.get() {
                return Err("The import is too large in total (more than 2 GB of content).".into());
            }
            let data = std::fs::read(p).map_err(|e| format!("Could not be read ({e})."))?;
            if data.len() as u64 > max {
                return Err("Too large.".into());
            }
            self.budget.set(self.budget.get().saturating_sub(data.len() as u64));
            Ok(data)
        };
        match self.files.get(path) {
            Some(Origin::Disk(p)) => disk(p),
            Some(Origin::Zip(a, i)) => self.archives[*a].read(*i, max, &self.budget).map_err(|e| {
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
        let part = zip::build(&[("Page 0123456789abcdef0123456789abcdef.md", b"# Page"), ("deeper.zip", &deepest)], true);
        let outer = zip::build(&[("Export-Part-1.zip", &part)], false);
        let v = Vfs::from_zip_bytes(outer).unwrap();
        let paths: Vec<&String> = v.paths().collect();
        assert_eq!(paths, ["Page 0123456789abcdef0123456789abcdef.md"]);
        assert!(v.refused.iter().any(|(n, _)| n == "deeper.zip"));
    }
}
