//! Plaintext export transactions only. The mutex covers cooperating writers in this process;
//! metadata/hash rechecks detect observed external changes, not an external compare-and-swap.
use std::collections::HashSet;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{markdown, ExportDoc};
use crate::error::{Error, Result};

static EXPORT_IO: Mutex<()> = Mutex::new(());
const OWNER: &str = ".notaya-export.json";
const SCHEMA: &str = "notaya-markdown-assets-v1";
const MARKER: &str = "<!-- Notaya export assets v1: ";
// Larger/unknown ownership records are preserved, never treated as permission to clean up.
const MAX_OWNER_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn lock() -> MutexGuard<'static, ()> {
    EXPORT_IO.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    PictureWrite(usize), PictureSync(usize), ManifestWrite, ManifestSync,
    DocumentWrite, DocumentSync, DocumentRename, RollbackRemove, OldCleanup,
}
#[cfg(test)]
type Hook<'a> = &'a mut dyn FnMut(Step, &Path) -> std::io::Result<()>;
#[cfg(not(test))]
type Hook<'a> = &'a mut ();
fn checkpoint(hook: Hook<'_>, step: Step, path: &Path) -> std::io::Result<()> {
    #[cfg(test)] { hook(step, path) }
    #[cfg(not(test))] { let _ = (hook, step, path); Ok(()) }
}

fn invalid(message: &str) -> Error { Error::invalid(message) }
fn id_valid(id: &str) -> bool { id.len() == 32 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) }
fn hash(bytes: &[u8]) -> String { hex::encode(crate::crypto::sha256(bytes)) }
fn plain(path: &Path) -> std::io::Result<Metadata> {
    let meta = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    let reparse = { use std::os::windows::fs::MetadataExt; meta.file_attributes() & 0x400 != 0 };
    #[cfg(not(windows))]
    let reparse = false;
    if meta.file_type().is_symlink() || reparse {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Linked export paths are refused"));
    }
    Ok(meta)
}
fn plain_chain(path: &Path) -> std::io::Result<()> {
    for ancestor in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        if !plain(ancestor)?.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Export parent is not a plain folder"));
        }
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    created: Option<SystemTime>,
    #[cfg(unix)] dev: u64,
    #[cfg(unix)] ino: u64,
}
fn identity(meta: &Metadata) -> Identity {
    Identity {
        created: meta.created().ok(),
        #[cfg(unix)] dev: { use std::os::unix::fs::MetadataExt; meta.dev() },
        #[cfg(unix)] ino: { use std::os::unix::fs::MetadataExt; meta.ino() },
    }
}
fn identity_known(value: &Identity) -> bool {
    #[cfg(unix)] { let _ = value; true }
    #[cfg(not(unix))] { value.created.is_some() }
}
fn same_directory(path: &Path, expected: &Identity) -> bool {
    identity_known(expected) && plain(path).is_ok_and(|m| m.is_dir() && identity(&m) == *expected)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stamp { identity: Identity, len: u64, modified: Option<SystemTime> }
fn stamp(meta: &Metadata) -> Stamp { Stamp { identity: identity(meta), len: meta.len(), modified: meta.modified().ok() } }
#[derive(Clone)]
struct Snapshot { path: PathBuf, stamp: Stamp, sha256: String }
impl Snapshot {
    fn read(path: &Path) -> Result<Self> {
        plain_chain(path.parent().unwrap_or(Path::new(".")))?;
        let before = plain(path)?;
        if !before.is_file() { return Err(invalid("Export file is not a plain file.")); }
        if !identity_known(&identity(&before)) || before.modified().is_err() { return Err(invalid("Export file identity is unknown.")); }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        { use std::os::windows::fs::OpenOptionsExt; options.custom_flags(0x00200000); }
        let mut file = options.open(path)?;
        if stamp(&file.metadata()?) != stamp(&before) { return Err(invalid("Export file changed while it was read.")); }
        let mut digest = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop { let n = file.read(&mut buffer)?; if n == 0 { break; } digest.update(&buffer[..n]); }
        let after = plain(path)?;
        if stamp(&before) != stamp(&after) || stamp(&file.metadata()?) != stamp(&after) {
            return Err(invalid("Export file changed while it was read."));
        }
        Ok(Self { path: path.to_path_buf(), stamp: stamp(&after), sha256: hex::encode(digest.finalize()) })
    }
    fn matches(&self) -> bool {
        Self::read(&self.path).is_ok_and(|now| now.stamp == self.stamp && now.sha256 == self.sha256)
    }
}
struct OwnedFile { path: PathBuf, snapshot: Option<Snapshot> }
struct OwnedDir { path: PathBuf, identity: Identity }
fn warning(path: &Path) -> String {
    format!("Some unencrypted export files could not be safely removed from {}. They were kept; remove them yourself only after checking them.", path.display())
}
fn add_warning(warnings: &mut Vec<String>, path: &Path) {
    let text = warning(path);
    if !warnings.contains(&text) { warnings.push(text); }
}
fn with_cleanup_error(primary: Error, warnings: Vec<String>) -> Error {
    if warnings.is_empty() { primary } else { Error::Other(format!("{primary} {}", warnings.join(" "))) }
}
fn rollback(files: &[OwnedFile], dirs: &[OwnedDir], hook: Hook<'_>) -> Vec<String> {
    let mut warnings = Vec::new();
    for file in files.iter().rev() {
        let removable = file.snapshot.as_ref().is_some_and(Snapshot::matches);
        if !removable || checkpoint(hook, Step::RollbackRemove, &file.path).and_then(|_| fs::remove_file(&file.path)).is_err() {
            add_warning(&mut warnings, file.path.parent().unwrap_or(&file.path));
        }
    }
    for dir in dirs.iter().rev() {
        if !same_directory(&dir.path, &dir.identity) || checkpoint(hook, Step::RollbackRemove, &dir.path).and_then(|_| fs::remove_dir(&dir.path)).is_err() {
            add_warning(&mut warnings, &dir.path);
        }
    }
    warnings
}
fn create_dir(path: &Path, dirs: &mut Vec<OwnedDir>) -> Result<()> {
    fs::create_dir(path)?; // Never remove an existing directory after a failed create.
    let meta = plain(path)?;
    dirs.push(OwnedDir { path: path.to_path_buf(), identity: identity(&meta) });
    Ok(())
}
fn stage_file(path: &Path, bytes: &[u8], write: Step, sync: Step, files: &mut Vec<OwnedFile>, hook: Hook<'_>) -> Result<()> {
    plain_chain(path.parent().unwrap_or(Path::new(".")))?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    // Record ownership only after create_new succeeds. A colliding path is never ours.
    files.push(OwnedFile { path: path.to_path_buf(), snapshot: None });
    let original = identity(&file.metadata()?);
    let result = (|| -> Result<()> {
        // A deterministic hook after a real partial write exercises rollback, not a fake success.
        let split = bytes.len() / 2;
        file.write_all(&bytes[..split])?;
        checkpoint(hook, write, path)?;
        file.write_all(&bytes[split..])?;
        checkpoint(hook, sync, path)?;
        file.sync_all()?;
        Ok(())
    })();
    // Close the writer before taking a stable stamp (Windows may finalize write times on close).
    drop(file);
    let owned = Snapshot::read(path).ok().filter(|s| {
        s.stamp.identity == original && s.stamp.len <= bytes.len() as u64
            && s.sha256 == hash(&bytes[..s.stamp.len as usize])
    });
    files.last_mut().expect("created file recorded").snapshot = owned;
    result?;
    if files.last().and_then(|f| f.snapshot.as_ref()).is_none() { return Err(invalid("A staged export file changed before it could be verified.")); }
    Ok(())
}
fn temp_path(dest: &Path, id: &str) -> Result<PathBuf> {
    let name = dest.file_name().and_then(|n| n.to_str()).ok_or_else(|| invalid("Unsupported file name."))?;
    Ok(dest.with_file_name(format!("{name}.{id}.tmp")))
}
fn snapshot_document(dest: &Path) -> Result<Option<Snapshot>> {
    match fs::symlink_metadata(dest) {
        Ok(_) => Snapshot::read(dest).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn unchanged_document(dest: &Path, old: &Option<Snapshot>) -> bool {
    match old { Some(s) => s.matches(), None => matches!(fs::symlink_metadata(dest), Err(e) if e.kind() == std::io::ErrorKind::NotFound) }
}
fn commit(temp: &Path, dest: &Path, files: &[OwnedFile], dirs: &[OwnedDir], previous: Option<&Option<Snapshot>>, hook: Hook<'_>) -> Result<()> {
    checkpoint(hook, Step::DocumentRename, temp)?;
    if previous.is_some_and(|old| !unchanged_document(dest, old)) || dirs.iter().any(|d| !same_directory(&d.path, &d.identity)) {
        return Err(invalid("An export path changed. The previous document was not replaced."));
    }
    if fs::symlink_metadata(dest).is_ok() {
        plain(dest)?;
    }
    if files.iter().any(|f| !f.snapshot.as_ref().is_some_and(Snapshot::matches)) {
        return Err(invalid("A staged export file changed; the previous document was not replaced."));
    }
    fs::rename(temp, dest)?; // The sole commit point. No fallible cleanup turns this into an error.
    Ok(())
}
pub(super) fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(test)] let mut hook = |_: Step, _: &Path| Ok(());
    #[cfg(not(test))] let mut hook = ();
    write_atomic_inner(dest, bytes, &crate::model::new_id(), &mut hook)
}
#[cfg(test)]
pub(super) fn write_atomic_with_id(dest: &Path, bytes: &[u8], id: &str, hook: Hook<'_>) -> Result<()> {
    write_atomic_inner(dest, bytes, id, hook)
}
fn write_atomic_inner(dest: &Path, bytes: &[u8], id: &str, hook: Hook<'_>) -> Result<()> {
    let _guard = lock();
    if !id_valid(id) { return Err(invalid("Invalid export temporary-file ID.")); }
    let temp = temp_path(dest, id)?;
    let mut files = Vec::new();
    let result = (|| {
        stage_file(&temp, bytes, Step::DocumentWrite, Step::DocumentSync, &mut files, hook)?;
        commit(&temp, dest, &files, &[], None, hook)
    })();
    match result { Ok(()) => Ok(()), Err(e) => Err(with_cleanup_error(e, rollback(&files, &[], hook))) }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest { schema: String, document: String, generation: String, files: Vec<Asset> }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset { name: String, bytes: u64, sha256: String }
fn encode_manifest(document: &str, id: &str, images: &[(String, Vec<u8>)], limit: u64) -> Result<Vec<u8>> {
    let files = images.iter().map(|(name, bytes)| Asset { name: name.clone(), bytes: bytes.len() as u64, sha256: hash(bytes) }).collect();
    let manifest = Manifest { schema: SCHEMA.into(), document: document.into(), generation: id.into(), files };
    let bytes = serde_json::to_vec(&manifest).map_err(|_| invalid("Could not write the export ownership record."))?;
    if bytes.len() as u64 > limit.min(MAX_OWNER_BYTES) {
        return Err(invalid("This Markdown export has too many pictures to replace safely. Choose fewer pages or export a ZIP."));
    }
    Ok(bytes)
}struct Generation { path: PathBuf, identity: Identity, files: Vec<Snapshot> }
impl Generation {
    fn matches(&self) -> bool {
        if !same_directory(&self.path, &self.identity) { return false; }
        let Ok(entries) = fs::read_dir(&self.path) else { return false; };
        let mut names = Vec::new();
        for entry in entries { let Ok(entry) = entry else { return false; }; names.push(entry.path()); }
        if names.len() != self.files.len() { return false; }
        self.files.iter().all(|f| names.contains(&f.path) && f.matches())
    }
}
fn read_generation(path: &Path, document: &str) -> Result<Generation> {
    plain_chain(path)?;
    let meta = plain(path)?;
    if !meta.is_dir() { return Err(invalid("An export picture generation is not a plain folder.")); }
    let id = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| invalid("Invalid export generation name."))?;
    if !id_valid(id) { return Err(invalid("Invalid export generation name.")); }
    let owner_path = path.join(OWNER);
    let owner_meta = plain(&owner_path)?;
    if !owner_meta.is_file() || owner_meta.len() > MAX_OWNER_BYTES { return Err(invalid("The export ownership record is not a bounded plain file.")); }
    let owner = Snapshot::read(&owner_path)?;
    if owner.stamp.len > MAX_OWNER_BYTES { return Err(invalid("The export ownership record is too large to verify safely.")); }
    let mut owner_bytes = Vec::new();
    File::open(&owner_path)?.take(MAX_OWNER_BYTES + 1).read_to_end(&mut owner_bytes)?;
    if owner_bytes.len() as u64 > MAX_OWNER_BYTES { return Err(invalid("The export ownership record is too large to verify safely.")); }
    let manifest: Manifest = serde_json::from_slice(&owner_bytes).map_err(|_| invalid("The export ownership record is invalid."))?;
    if manifest.schema != SCHEMA || manifest.document != document || manifest.generation != id || manifest.files.is_empty() || !owner.matches() {
        return Err(invalid("The export picture ownership could not be verified."));
    }
    let mut unique = HashSet::new();
    let mut files = Vec::new();
    for asset in manifest.files {
        if !super::is_export_image_name(&asset.name) || (asset.name.contains('/') || asset.name.contains('\\'))
            || Path::new(&asset.name).file_name().and_then(|s| s.to_str()) != Some(asset.name.as_str())
            || !unique.insert(asset.name.to_ascii_lowercase()) || asset.sha256.len() != 64
            || !asset.sha256.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(invalid("The export ownership record contains an invalid picture name or hash."));
        }
        let file = path.join(&asset.name);
        if !plain(&file)?.is_file() || plain(&file)?.len() != asset.bytes { return Err(invalid("An earlier export picture changed.")); }
        let snapshot = Snapshot::read(&file)?;
        if snapshot.sha256 != asset.sha256 { return Err(invalid("An earlier export picture changed.")); }
        files.push(snapshot);
    }
    files.push(owner); // Remove the ownership record last, after all recorded pictures.
    let generation = Generation { path: path.to_path_buf(), identity: identity(&meta), files };
    if !generation.matches() { return Err(invalid("The export picture folder contains unknown or changed files.")); }
    Ok(generation)
}
fn marker(document: &str, id: &str) -> String { format!("{MARKER}{}:{id} -->\n", hash(document.as_bytes())) }
fn old_id(dest: &Path) -> Result<Option<String>> {
    let document = dest.file_name().and_then(|n| n.to_str()).ok_or_else(|| invalid("Unsupported file name."))?;
    let prefix_for_document = format!("{MARKER}{}:", hash(document.as_bytes()));
    let mut prefix = Vec::new();
    match File::open(dest) {
        Ok(file) => { file.take(160).read_to_end(&mut prefix)?; }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let Some(line) = prefix.split(|b| *b == b'\n').next().and_then(|s| std::str::from_utf8(s).ok()) else { return Ok(None); };
    let Some(id) = line.strip_prefix(&prefix_for_document).and_then(|s| s.strip_suffix(" -->")) else { return Ok(None); };
    Ok(id_valid(id).then(|| id.to_string()))
}
fn inspect_root(root: &Path, document: &str, old: Option<&str>, data_dir: &Path, warnings: &mut Vec<String>) -> Result<Option<Generation>> {
    if super::resolve(root).starts_with(super::resolve(data_dir)) { return Err(invalid("Choose an export folder outside Notaya's data folder.")); }
    match fs::symlink_metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
        Ok(_) => { plain_chain(root)?; }
    }
    let mut selected = None;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().into_string().map_err(|_| invalid("The picture folder holds an unknown file."))?;
        let meta = plain(&entry.path())?;
        if meta.is_file() && super::is_export_image_name(&name) {
            add_warning(warnings, root); // Legacy names are not proof that these files belong to us.
        } else if meta.is_dir() && id_valid(&name) {
            let generation = read_generation(&entry.path(), document)?;
            if old == Some(name.as_str()) { selected = Some(generation); } else { add_warning(warnings, &entry.path()); }
        } else { return Err(invalid("The pictures folder already holds other files. Choose another export name.")); }
    }
    Ok(selected)
}
fn cleanup_old(old: &Generation, hook: Hook<'_>, warnings: &mut Vec<String>) {
    if checkpoint(hook, Step::OldCleanup, &old.path).is_err() || !old.matches() {
        add_warning(warnings, &old.path); return;
    }
    for file in &old.files {
        if !file.matches() || fs::remove_file(&file.path).is_err() { add_warning(warnings, &old.path); return; }
    }
    if !same_directory(&old.path, &old.identity) || fs::remove_dir(&old.path).is_err() { add_warning(warnings, &old.path); }
}

pub(super) fn write_markdown(dest: &Path, doc: &ExportDoc, data_dir: &Path) -> Result<(Option<String>, Vec<String>)> {
    #[cfg(test)] let mut hook = |_: Step, _: &Path| Ok(());
    #[cfg(not(test))] let mut hook = ();
    write_markdown_inner(dest, doc, data_dir, &mut hook, MAX_OWNER_BYTES)
}
#[cfg(test)]
pub(super) fn write_markdown_with(dest: &Path, doc: &ExportDoc, data_dir: &Path, hook: Hook<'_>) -> Result<(Option<String>, Vec<String>)> {
    write_markdown_inner(dest, doc, data_dir, hook, MAX_OWNER_BYTES)
}
fn write_markdown_inner(dest: &Path, doc: &ExportDoc, data_dir: &Path, hook: Hook<'_>, owner_limit: u64) -> Result<(Option<String>, Vec<String>)> {
    let _guard = lock(); // No recursive call to the public atomic writer while this is held.
    plain_chain(dest.parent().unwrap_or(Path::new(".")))?;
    let previous = snapshot_document(dest)?;
    let old_id = old_id(dest)?;
    let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("export");
    let document = dest.file_name().and_then(|s| s.to_str()).ok_or_else(|| invalid("Unsupported file name."))?;
    let assets_name = format!("{stem}.assets");
    let root = dest.with_file_name(&assets_name);
    let mut warnings = Vec::new();
    let old = inspect_root(&root, document, old_id.as_deref(), data_dir, &mut warnings)?;
    if !unchanged_document(dest, &previous) { return Err(invalid("The previous export changed. Choose the export again.")); }
    let id = crate::model::new_id();
    let generation_path = root.join(&id);
    let out = markdown::render(doc, &format!("{assets_name}/{id}"));
    // Check the exact reader-compatible ownership record before creating any staging path.
    let manifest_bytes = if out.images.is_empty() { None } else { Some(encode_manifest(document, &id, &out.images, owner_limit)?) };
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let temp = temp_path(dest, &crate::model::new_id())?;
    let result = (|| -> Result<()> {
        if !out.images.is_empty() {
            match fs::symlink_metadata(&root) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => create_dir(&root, &mut dirs)?,
                Err(e) => return Err(e.into()),
                Ok(_) => plain_chain(&root)?,
            }
            create_dir(&generation_path, &mut dirs)?;
            for (index, (name, bytes)) in out.images.iter().enumerate() {
                stage_file(&generation_path.join(name), bytes, Step::PictureWrite(index), Step::PictureSync(index), &mut files, hook)?;
            }
            stage_file(&generation_path.join(OWNER), manifest_bytes.as_deref().expect("pictures have an ownership record"), Step::ManifestWrite, Step::ManifestSync, &mut files, hook)?;
        }
        let text = if out.images.is_empty() { out.text.clone() } else { format!("{}{}", marker(document, &id), out.text) };
        stage_file(&temp, text.as_bytes(), Step::DocumentWrite, Step::DocumentSync, &mut files, hook)?;
        // Check the previous destination and all new files immediately before replacement.
        // This mutex serializes our writers; these checks are not an external-writer CAS.
        if !unchanged_document(dest, &previous) || dirs.iter().any(|d| !same_directory(&d.path, &d.identity)) {
            return Err(invalid("An export path changed. The previous document was not replaced."));
        }
        commit(&temp, dest, &files, &dirs, Some(&previous), hook)
    })();
    if let Err(e) = result { return Err(with_cleanup_error(e, rollback(&files, &dirs, hook))); }
    // Commit succeeded. Keep the new generation even when any later check/cleanup fails.
    let committed_hash = if out.images.is_empty() { hash(out.text.as_bytes()) } else { hash(format!("{}{}", marker(document, &id), out.text).as_bytes()) };
    if Snapshot::read(dest).is_ok_and(|s| s.sha256 == committed_hash) {
        if let Some(old) = old { cleanup_old(&old, hook, &mut warnings); }
    } else { add_warning(&mut warnings, &root); }
    Ok(((!out.images.is_empty()).then(|| generation_path.display().to_string()), warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{doc, image, Kind, Picture};

    fn directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!("notaya-export-transaction-{}", crate::model::new_id()));
        fs::create_dir(&path).unwrap();
        path
    }
    fn document(label: &str, count: usize) -> ExportDoc {
        let mut content = vec![serde_json::json!({"type":"paragraph","content":[{"type":"text","text":label}]})];
        for n in 1..=count {
            content.push(serde_json::json!({"type":"image","attrs":{"src":format!("http://clatt.localhost/{n:032x}"),"caption":format!("Picture {n}")}}));
        }
        let json = serde_json::json!({"type":"doc","content":content}).to_string();
        let mut out = crate::export::tests::sample_doc(doc::parse(&json, &doc::tests::titles));
        out.images.clear();
        for n in 1..=count { out.images.insert(format!("{n:032x}"), Picture { kind: Kind::Png, bytes: image::tests::PILLOW_RGBA.to_vec() }); }
        out
    }
    fn seed(root: &Path) -> (PathBuf, PathBuf, Generation, Vec<u8>) {
        let dest = root.join("notes.md");
        let (folder, warnings) = write_markdown(&dest, &document("old text", 2), &root.join("data")).unwrap();
        assert!(warnings.is_empty());
        let folder = PathBuf::from(folder.unwrap());
        let generation = read_generation(&folder, "notes.md").unwrap();
        let bytes = fs::read(&dest).unwrap();
        (dest, folder, generation, bytes)
    }
    fn byte_inventory(folder: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
        fs::read_dir(folder).unwrap().map(|entry| {
            let entry = entry.unwrap();
            assert!(fs::symlink_metadata(entry.path()).unwrap().is_file());
            (entry.file_name().into_string().unwrap(), fs::read(entry.path()).unwrap())
        }).collect()
    }
    fn assert_picture_links(dest: &Path, expected: usize) {
        let text = fs::read_to_string(dest).unwrap();
        let mut found = 0;
        for line in text.lines().filter(|line| line.starts_with("![")) {
            let (_, path) = line.split_once("](").expect("generated Markdown image link");
            let path = path.strip_suffix(')').expect("image link ends with a parenthesis");
            let relative = Path::new(path);
            assert!(!relative.is_absolute() && relative.components().all(|p| matches!(p, std::path::Component::Normal(_))));
            assert_eq!(fs::read(dest.parent().unwrap().join(relative)).unwrap(), image::tests::PILLOW_RGBA);
            found += 1;
        }
        assert_eq!(found, expected, "every expected picture must have a resolvable link with its original bytes");
    }
    fn injected() -> std::io::Error { std::io::Error::new(std::io::ErrorKind::Other, "INJECTED export I/O failure") }
    fn no_document_temps(root: &Path) -> bool {
        fs::read_dir(root).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp"))
    }

    #[test]
    fn every_precommit_failure_preserves_the_old_document_and_all_picture_bytes() {
        for fail in [Step::PictureWrite(0), Step::PictureWrite(1), Step::PictureSync(0), Step::PictureSync(1), Step::ManifestWrite, Step::ManifestSync, Step::DocumentWrite, Step::DocumentSync, Step::DocumentRename] {
            let root = directory();
            let (dest, folder, old, bytes) = seed(&root);
            let old_picture_bytes = byte_inventory(&folder);
            let error = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| if step == fail { Err(injected()) } else { Ok(()) }).unwrap_err();
            assert!(error.to_string().contains("INJECTED"), "{fail:?}: {error}");
            assert_eq!(fs::read(&dest).unwrap(), bytes, "{fail:?}");
            assert_eq!(byte_inventory(&folder), old_picture_bytes, "{fail:?}: independently read old bytes changed");
            assert!(old.matches(), "{fail:?}: old pictures or manifest changed");
            assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 1, "{fail:?}: staging left behind");
            assert!(folder.exists() && no_document_temps(&root));
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn a_create_new_collision_never_removes_the_other_writers_temp() {
        let root = directory();
        let dest = root.join("notes.md");
        fs::write(&dest, b"old document").unwrap();
        let id = "00000000000000000000000000000001";
        let temp = temp_path(&dest, id).unwrap();
        fs::write(&temp, b"foreign temporary file").unwrap();
        assert!(write_atomic_with_id(&dest, b"replacement", id, &mut |_, _| Ok(())).is_err());
        assert_eq!(fs::read(&temp).unwrap(), b"foreign temporary file");
        assert_eq!(fs::read(&dest).unwrap(), b"old document");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_rollback_reports_plaintext_leftovers_and_keeps_the_primary_failure() {
        let root = directory();
        let (dest, _, old, bytes) = seed(&root);
        let error = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| {
            if matches!(step, Step::DocumentRename | Step::RollbackRemove) { Err(injected()) } else { Ok(()) }
        }).unwrap_err().to_string();
        assert!(error.contains("INJECTED") && error.contains("unencrypted") && error.contains("kept"), "{error}");
        assert_eq!(fs::read(&dest).unwrap(), bytes);
        assert!(old.matches());
        assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 2);
        assert!(!no_document_temps(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn postcommit_cleanup_failure_is_success_with_a_warning_and_complete_new_pictures() {
        let root = directory();
        let (dest, old_folder, old, _) = seed(&root);
        let (new_folder, warnings) = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| if step == Step::OldCleanup { Err(injected()) } else { Ok(()) }).unwrap();
        let new_folder = PathBuf::from(new_folder.unwrap());
        assert_ne!(old_folder, new_folder);
        assert!(!warnings.is_empty() && old.matches());
        assert!(read_generation(&new_folder, "notes.md").unwrap().matches());
        assert_picture_links(&dest, 2);
        let text = fs::read_to_string(&dest).unwrap();
        assert!(text.contains("new text") && text.contains(new_folder.file_name().unwrap().to_str().unwrap()));
        assert!(no_document_temps(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repeated_successful_replacements_clean_only_the_previous_owned_generation() {
        let root = directory();
        let (dest, mut previous, _, _) = seed(&root);
        for label in ["second successful export", "third successful export", "fourth successful export"] {
            let (new_folder, warnings) = write_markdown(&dest, &document(label, 2), &root.join("data")).unwrap();
            let new_folder = PathBuf::from(new_folder.unwrap());
            assert!(warnings.is_empty());
            assert!(!previous.exists());
            assert!(read_generation(&new_folder, "notes.md").unwrap().matches());
            assert_picture_links(&dest, 2);
            assert!(fs::read_to_string(&dest).unwrap().contains(label));
            assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 1);
            previous = new_folder;
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn replacing_with_no_pictures_commits_before_old_picture_cleanup() {
        let root = directory();
        let (dest, folder, _, _) = seed(&root);
        let (new_folder, warnings) = write_markdown(&dest, &document("no pictures now", 0), &root.join("data")).unwrap();
        assert!(new_folder.is_none() && warnings.is_empty());
        assert!(!folder.exists());
        let text = fs::read_to_string(&dest).unwrap();
        assert!(text.contains("no pictures now") && !text.starts_with(MARKER));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn changed_old_picture_bytes_are_preserved_after_commit() {
        let root = directory();
        let (dest, _, old, _) = seed(&root);
        let picture = old.files[0].path.clone();
        let (_, warnings) = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| {
            if step == Step::OldCleanup { fs::write(&picture, b"foreign changed bytes")?; }
            Ok(())
        }).unwrap();
        assert!(!warnings.is_empty());
        assert_eq!(fs::read(&picture).unwrap(), b"foreign changed bytes");
        assert!(fs::read_to_string(&dest).unwrap().contains("new text"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replaced_old_generation_is_not_deleted() {
        let root = directory();
        let (dest, folder, _, _) = seed(&root);
        let moved = root.join("moved-original-generation");
        let (_, warnings) = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| {
            if step == Step::OldCleanup {
                fs::rename(&folder, &moved)?;
                fs::create_dir(&folder)?;
                for entry in fs::read_dir(&moved)? { let entry = entry?; fs::copy(entry.path(), folder.join(entry.file_name()))?; }
            }
            Ok(())
        }).unwrap();
        assert!(!warnings.is_empty());
        assert!(folder.join(OWNER).is_file() && moved.join(OWNER).is_file());
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_or_unowned_generation_records_refuse_without_changing_old_files() {
        for change in ["schema", "traversal", "hash", "unknown field"] {
            let root = directory();
            let (dest, folder, _, bytes) = seed(&root);
            let owner = folder.join(OWNER);
            let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&owner).unwrap()).unwrap();
            match change {
                "schema" => manifest["schema"] = "foreign".into(),
                "traversal" => manifest["files"][0]["name"] = "../outside.png".into(),
                "hash" => manifest["files"][0]["sha256"] = "00".repeat(32).into(),
                _ => manifest["extra"] = true.into(),
            }
            let changed = serde_json::to_vec(&manifest).unwrap();
            fs::write(&owner, &changed).unwrap();
            let sentinel = root.join("outside.png"); fs::write(&sentinel, b"foreign").unwrap();
            assert!(write_markdown(&dest, &document("new text", 2), &root.join("data")).is_err(), "{change}");
            assert_eq!(fs::read(&dest).unwrap(), bytes);
            assert_eq!(fs::read(&owner).unwrap(), changed);
            assert_eq!(fs::read(&sentinel).unwrap(), b"foreign");
            assert!(no_document_temps(&root));
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn a_changed_destination_or_staged_picture_is_not_silently_overwritten_or_removed() {
        for changed_destination in [false, true] {
            let root = directory();
            let (dest, old_folder, old, bytes) = seed(&root);
            let mut changed_picture = None;
            let result = write_markdown_with(&dest, &document("new text", 2), &root.join("data"), &mut |step, _| {
                if step == Step::DocumentRename {
                    if changed_destination { fs::write(&dest, b"person changed the document")?; }
                    else {
                        let new_folder = fs::read_dir(root.join("notes.assets"))?.map(|e| e.map(|e| e.path())).collect::<std::io::Result<Vec<_>>>()?.into_iter().find(|p| *p != old_folder).unwrap();
                        let picture = new_folder.join("image-001.png");
                        fs::write(&picture, b"foreign staged bytes")?;
                        changed_picture = Some(picture);
                    }
                }
                Ok(())
            });
            assert!(result.is_err());
            assert!(old.matches());
            if changed_destination { assert_eq!(fs::read(&dest).unwrap(), b"person changed the document"); }
            else {
                assert_eq!(fs::read(&dest).unwrap(), bytes);
                assert_eq!(fs::read(changed_picture.unwrap()).unwrap(), b"foreign staged bytes");
                assert!(result.unwrap_err().to_string().contains("unencrypted"));
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn content_lookalikes_cannot_select_another_documents_generation_for_cleanup() {
        for first_line in [false, true] {
            let root = directory();
            let (dest, folder, _, _) = seed(&root);
            let old_bytes = byte_inventory(&folder);
            let id = folder.file_name().unwrap().to_str().unwrap();
            let lookalike = if first_line {
                format!("{MARKER}{}:{id} -->\nuser content", hash(b"other.md"))
            } else {
                format!("user content first\n{MARKER}{}:{id} -->\n", hash(b"notes.md"))
            };
            fs::write(&dest, lookalike).unwrap();
            let (_, warnings) = write_markdown(&dest, &document("new export", 2), &root.join("data")).unwrap();
            assert!(!warnings.is_empty());
            assert_eq!(byte_inventory(&folder), old_bytes);
            assert_picture_links(&dest, 2);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn manifest_cap_is_consistent_at_the_threshold_and_refuses_before_staging() {
        assert_eq!(MAX_OWNER_BYTES, 4 * 1024 * 1024, "the real writer/reader policy remains four MiB");
        for accepted in [false, true] {
            let root = directory();
            let (dest, old_folder, _, old_document) = seed(&root);
            let old_files = byte_inventory(&old_folder);
            let candidate = document("threshold replacement", 2);
            let rendered = markdown::render(&candidate, "notes.assets/00000000000000000000000000000001");
            let encoded = encode_manifest("notes.md", "00000000000000000000000000000001", &rendered.images, MAX_OWNER_BYTES).unwrap();
            // IDs always have 32 ASCII characters, so actual random-ID serialization has this exact length.
            let limit = encoded.len() as u64 - if accepted { 0 } else { 1 };
            let mut callbacks = 0;
            let result = write_markdown_inner(&dest, &candidate, &root.join("data"), &mut |_, _| { callbacks += 1; Ok(()) }, limit);
            if accepted {
                let (folder, warnings) = result.unwrap();
                assert!(warnings.is_empty() && callbacks > 0);
                assert!(!old_folder.exists());
                let folder = PathBuf::from(folder.unwrap());
                assert_eq!(fs::metadata(folder.join(OWNER)).unwrap().len(), limit);
                assert_picture_links(&dest, 2);
            } else {
                assert!(result.unwrap_err().to_string().contains("too many pictures"));
                assert_eq!(callbacks, 0, "no picture, document or rollback operation may start");
                assert_eq!(fs::read(&dest).unwrap(), old_document);
                assert_eq!(byte_inventory(&old_folder), old_files);
                assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 1);
                assert!(no_document_temps(&root));
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn oversized_ownership_records_are_refused_without_new_export_writes() {
        let root = directory();
        let (dest, folder, _, bytes) = seed(&root);
        let owner = folder.join(OWNER);
        OpenOptions::new().write(true).open(&owner).unwrap().set_len(MAX_OWNER_BYTES + 1).unwrap();
        assert!(write_markdown(&dest, &document("new export", 2), &root.join("data")).is_err());
        assert_eq!(fs::read(&dest).unwrap(), bytes);
        assert_eq!(fs::metadata(&owner).unwrap().len(), MAX_OWNER_BYTES + 1);
        assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 1);
        assert!(no_document_temps(&root));
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn linked_assets_are_refused_and_their_targets_are_untouched() {
        let root = directory();
        let (dest, folder, _, bytes) = seed(&root);
        let target = root.join("foreign.png"); fs::write(&target, b"foreign target").unwrap();
        let picture = folder.join("image-001.png");
        fs::remove_file(&picture).unwrap();
        std::os::unix::fs::symlink(&target, &picture).unwrap();
        assert!(write_markdown(&dest, &document("new text", 2), &root.join("data")).is_err());
        assert_eq!(fs::read(&dest).unwrap(), bytes);
        assert_eq!(fs::read(&target).unwrap(), b"foreign target");
        assert!(fs::symlink_metadata(&picture).unwrap().file_type().is_symlink());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_junction_assets_are_refused_without_touching_target_or_link() {
        use std::os::windows::{fs::MetadataExt, process::CommandExt};
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};
        const CMD: &str = r"C:\Windows\System32\cmd.exe";
        fn junction_state(link: &Path, target: &Path) -> (u32, u64, u64, PathBuf) {
            let meta = fs::symlink_metadata(link).unwrap();
            let attrs = meta.file_attributes();
            assert_eq!(attrs & 0x410, 0x410, "expected a directory reparse point");
            let recorded_target = fs::read_link(link).unwrap();
            assert!(recorded_target.is_absolute());
            assert_eq!(fs::canonicalize(&recorded_target).unwrap(), fs::canonicalize(target).unwrap());
            assert_eq!(fs::canonicalize(link).unwrap(), fs::canonicalize(target).unwrap());
            (attrs, meta.creation_time(), meta.last_write_time(), recorded_target)
        }
        fn make_junction(root: &Path, link: &Path, target: &Path) -> std::io::Result<()> {
            let safe = |path: &Path| -> std::io::Result<String> {
                let text = path.to_str().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Non-Unicode fixture path"))?;
                if !path.is_absolute() || text.chars().any(|c| c.is_control() || "&|<>^%!\"()".contains(c)) {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Shell metacharacters in fixture path"));
                }
                Ok(text.to_string())
            };
            let link_text = safe(link)?; let target_text = safe(target)?;
            let home = root.join("cmd-home"); let temp = root.join("cmd-temp");
            fs::create_dir(&home)?; fs::create_dir(&temp)?;
            let roaming = home.join("AppData").join("Roaming");
            let local = home.join("AppData").join("Local");
            fs::create_dir_all(&roaming)?; fs::create_dir_all(&local)?;
            let mut child = Command::new(CMD)
                .args(["/d", "/v:off", "/c"])
                .raw_arg(format!("mklink /J \"{link_text}\" \"{target_text}\""))
                .env_clear().env("SystemRoot", r"C:\Windows").env("WINDIR", r"C:\Windows").env("ComSpec", CMD)
                .env("USERPROFILE", &home).env("HOME", &home).env("APPDATA", &roaming).env("LOCALAPPDATA", &local)
                .env("TEMP", &temp).env("TMP", &temp).current_dir(root)
                .creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .spawn()?;
            // Fixture safety bounds, not a product-performance requirement. Never block in wait().
            let deadline = Instant::now() + Duration::from_secs(5);
            let failure = loop {
                match child.try_wait() {
                    Ok(Some(status)) if status.success() => return Ok(()),
                    Ok(Some(status)) => return Err(std::io::Error::other(format!("Owned mklink failed: {status}"))),
                    Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
                    Ok(None) => break std::io::Error::new(std::io::ErrorKind::TimedOut, "Owned mklink exceeded its fixture bound"),
                    Err(error) => break error,
                }
            };
            let _ = child.kill(); // This exact spawned child only; no process-name cleanup.
            let grace = Instant::now() + Duration::from_secs(2);
            loop {
                if matches!(child.try_wait(), Ok(Some(_))) { return Err(failure); }
                if Instant::now() >= grace {
                    return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Owned mklink unresolved; fixture root must be retained without cleanup"));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let root = directory();
        plain_chain(&root).unwrap();
        let root_created = fs::symlink_metadata(&root).unwrap().creation_time();
        let target = root.join("junction-target"); fs::create_dir(&target).unwrap();
        let target_created = fs::symlink_metadata(&target).unwrap().creation_time();
        // Export-shaped content prevents foreign-file refusal from making a broken link guard pass.
        fs::write(target.join("image-001.png"), image::tests::PILLOW_RGBA).unwrap();
        let expected_target = byte_inventory(&target);
        let dest = root.join("notes.md");
        let original = b"![Old picture](notes.assets/image-001.png)\n";
        fs::write(&dest, original).unwrap();
        let link = root.join("notes.assets");
        assert!(matches!(fs::symlink_metadata(&link), Err(e) if e.kind() == std::io::ErrorKind::NotFound));
        // Any setup failure panics before cleanup, retaining evidence; no admin or symlink fallback.
        make_junction(&root, &link, &target).expect("junction setup failed; retain the owned fixture root");
        let before_link = junction_state(&link, &target);
        assert!(write_markdown(&dest, &document("must not replace", 2), &root.join("data")).is_err());
        assert_eq!(fs::read(&dest).unwrap(), original);
        assert_eq!(byte_inventory(&target), expected_target);
        assert_eq!(junction_state(&link, &target), before_link);
        assert_eq!(fs::symlink_metadata(&target).unwrap().creation_time(), target_created);
        // Remove only the verified, settled, cmd-created junction before any recursive root cleanup.
        assert_eq!(fs::symlink_metadata(&root).unwrap().creation_time(), root_created);
        assert_eq!(fs::symlink_metadata(&root).unwrap().file_attributes() & 0x400, 0);
        fs::remove_dir(&link).unwrap();
        assert!(matches!(fs::symlink_metadata(&link), Err(e) if e.kind() == std::io::ErrorKind::NotFound));
        assert_eq!(byte_inventory(&target), expected_target);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cooperating_exports_hold_one_process_lock_through_staging_commit_and_cleanup() {
        use std::sync::mpsc;
        let root = directory();
        let (dest, _, _, _) = seed(&root);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let first_dest = dest.clone(); let first_data = root.join("data");
        let first = std::thread::spawn(move || write_markdown_with(&first_dest, &document("first concurrent export", 2), &first_data, &mut |step, _| {
            if step == Step::DocumentRename { entered_tx.send(()).unwrap(); release_rx.recv().unwrap(); }
            Ok(())
        }));
        entered_rx.recv().unwrap();
        let held = EXPORT_IO.try_lock().is_err();
        let second_dest = dest.clone(); let second_data = root.join("data");
        let second = std::thread::spawn(move || write_markdown(&second_dest, &document("second concurrent export", 2), &second_data));
        release_tx.send(()).unwrap();
        let first_folder = first.join().unwrap().unwrap().0.unwrap();
        let second_folder = second.join().unwrap().unwrap().0.unwrap();
        assert!(held, "the real export mutex must remain held at the commit boundary");
        assert!(!Path::new(&first_folder).exists());
        assert!(read_generation(Path::new(&second_folder), "notes.md").unwrap().matches());
        assert_picture_links(&dest, 2);
        assert!(fs::read_to_string(&dest).unwrap().contains("second concurrent export"));
        assert_eq!(fs::read_dir(root.join("notes.assets")).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
