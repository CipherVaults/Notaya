//! The glue of an import: from the files of a [`Vfs`] to ONE new notebook.
//!
//! 1. [`build`] finds the documents (by extension), converts each with the matching converter
//!    and arranges them: one section per top-level folder (or one section "Imported pages" for
//!    loose pages), a page for each document, and Notion's nested folders as sub-pages (a page
//!    `X.md` and a folder `X/` next to it: the pages in `X/` are sub-pages of X; a folder with no
//!    page of its own becomes an empty page that holds its pages). Nothing is written.
//! 2. [`Plan::store_images`] stores the pictures (the caller gives the function that stores one).
//! 3. [`Plan::items`] renders the pages and returns the vault items of the new notebook, to be
//!    saved in one transaction (`Session::put_many`).
//!
//! Pictures are shown with the same address the editor uses for a pasted picture
//! (`http://clatt.localhost/<id>`), and a page cover the same way.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use super::doc::{self, Block, ImageOut, Inline};
use super::source::Vfs;
use super::{clean_title, decode_text, docx, html, markdown, pdf, sniff, strip_notion_id, Control, Ctx, ImgSrc, Images, Skipped, Stop};
use super::{MAX_BINARY_FILE, MAX_IMAGE, MAX_PAGES, MAX_TEXT_FILE};
use crate::model::{new_id, Item, Notebook, Page, Section};

/// What the person is told when they cancel.
pub const CANCELLED: &str = "The import was cancelled. Nothing was added.";
/// What the person is told when nothing at all could be read before the time ran out.
pub const TOO_SLOW: &str = "The import took too long (more than 15 minutes) and was stopped. Nothing was added. Import fewer files at a time.";
/// Folders nested deeper than this are not imported.
const MAX_FOLDERS: usize = 40;
/// Most skipped items listed.
pub const MAX_LISTED: usize = 500;
/// Most picture bytes one import may store.
const MAX_IMAGE_TOTAL: u64 = 1024 * 1024 * 1024;
/// A page whose editor JSON or text is larger than this is not imported (the editor saves up to
/// 20 million characters).
const MAX_PAGE_BYTES: usize = 20_000_000;
/// The rendered pages of one import together.
const MAX_TOTAL_BYTES: usize = 512 * 1024 * 1024;
/// Names the vault accepts.
const MAX_NAME: usize = 120;
const MAX_TITLE: usize = 300;

pub const LOOSE_SECTION: &str = "Imported pages";
const NOTEBOOK_COLOR: &str = "#6366f1";
const NOTEBOOK_ICON: &str = "📓";

/// What the UI is told after an import.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub notebook_id: String,
    pub notebook_name: String,
    pub sections: usize,
    pub pages: usize,
    pub images: usize,
    pub skipped: Vec<Skipped>,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Markdown,
    Text,
    Html,
    Docx,
    Pdf,
}

fn extension(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

fn kind_of(name: &str) -> Option<Kind> {
    match extension(name).as_str() {
        "md" | "markdown" | "mdown" => Some(Kind::Markdown),
        "txt" | "text" => Some(Kind::Text),
        "html" | "htm" | "xhtml" => Some(Kind::Html),
        "docx" => Some(Kind::Docx),
        "pdf" => Some(Kind::Pdf),
        _ => None,
    }
}

fn is_picture_name(name: &str) -> bool {
    matches!(extension(name).as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tif" | "tiff" | "heic" | "avif")
}

/// The page title of a file name: without the extension and the id Notion adds.
fn file_title(name: &str) -> String {
    clean_title(name, kind_of(name).is_some())
}

/// The name without its extension (when it is a document), as Notion names the folder of its sub-pages.
fn stem_of(name: &str) -> &str {
    if kind_of(name).is_some() { name.rsplit_once('.').map_or(name, |(s, _)| s) } else { name }
}

/// Orders names the way people expect: case-insensitive, with numbers by value ("Page 2" before "Page 10").
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(&b),
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let mut n = String::new();
                while let Some(c) = x.next_if(char::is_ascii_digit) {
                    n.push(c);
                }
                let mut m = String::new();
                while let Some(d) = y.next_if(char::is_ascii_digit) {
                    m.push(d);
                }
                let (n, m) = (n.trim_start_matches('0'), m.trim_start_matches('0'));
                let ord = n.len().cmp(&m.len()).then_with(|| n.cmp(m));
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            (Some(c), Some(d)) => {
                if c != d {
                    return c.cmp(&d);
                }
                x.next();
                y.next();
            }
        }
    }
}

/// A folder of the source, with only the documents (and the folders that hold documents).
#[derive(Default)]
struct Dir {
    /// (file name, full path)
    files: Vec<(String, String)>,
    dirs: BTreeMap<String, Dir>,
}

impl Dir {
    fn insert(&mut self, parts: &[&str], full: &str) {
        match parts {
            [] => {}
            [name] => self.files.push((name.to_string(), full.to_string())),
            [first, rest @ ..] => self.dirs.entry(first.to_string()).or_default().insert(rest, full),
        }
    }

    fn sorted_dirs(&self) -> Vec<(&String, &Dir)> {
        let mut v: Vec<_> = self.dirs.iter().collect();
        v.sort_by(|a, b| natural_cmp(a.0, b.0));
        v
    }
}

/// The result of building: nothing is stored yet.
pub struct Plan {
    pub notebook: String,
    pub sections: Vec<String>,
    pub pages: Vec<PlanPage>,
    pub images: Images,
    pub skipped: Vec<Skipped>,
    pub notes: Vec<String>,
}

pub struct PlanPage {
    /// The page's path in the source (links between pages use it).
    pub key: String,
    pub id: String,
    pub section: usize,
    pub parent: Option<usize>,
    pub title: String,
    pub icon: String,
    pub cover: Option<usize>,
    pub blocks: Vec<Block>,
}

struct Builder<'a> {
    vfs: &'a Vfs,
    ctl: &'a Control,
    pages: Vec<PlanPage>,
    images: Images,
    skipped: Vec<Skipped>,
    unlisted: usize,
    /// Set when the import was cancelled or ran out of time: no more documents are read.
    stopped: Option<Stop>,
    pdfs: usize,
}

impl Builder<'_> {
    fn skip(&mut self, item: impl Into<String>, reason: impl Into<String>) {
        if self.skipped.len() < MAX_LISTED {
            self.skipped.push(Skipped { item: item.into(), reason: reason.into() });
        } else {
            self.unlisted += 1;
        }
    }

    /// The pages of `d` (and the folders named like them), not the other folders. Returns the
    /// folders that were used as sub-pages.
    fn files_of(&mut self, d: &Dir, section: usize, parent: Option<usize>) -> HashSet<String> {
        let mut used = HashSet::new();
        let mut files = d.files.clone();
        files.sort_by(|a, b| natural_cmp(&a.0, &b.0));
        // Folders by lower-case name, made once (a long list of files must not search them all each time).
        let mut lower: HashMap<String, &String> = HashMap::new();
        for k in d.dirs.keys() {
            lower.entry(k.to_lowercase()).or_insert(k);
        }
        for (name, path) in files {
            let Some(index) = self.document(&name, &path, section, parent) else { continue };
            let stem = stem_of(&name);
            let sub = d
                .dirs
                .get_key_value(stem)
                .filter(|(k, _)| !used.contains(*k))
                .or_else(|| lower.get(&stem.to_lowercase()).filter(|k| !used.contains(**k)).and_then(|k| d.dirs.get_key_value(*k)));
            if let Some((key, sub)) = sub {
                used.insert(key.clone());
                self.folder(sub, section, Some(index), "");
            }
        }
        used
    }

    /// A folder's pages, then its other folders as pages that hold their pages.
    fn folder(&mut self, d: &Dir, section: usize, parent: Option<usize>, path: &str) {
        let used = self.files_of(d, section, parent);
        for (name, sub) in d.sorted_dirs() {
            if used.contains(name) {
                continue;
            }
            if self.pages.len() >= MAX_PAGES + 1 {
                return;
            }
            let key = format!("dir:{path}/{name}");
            self.pages.push(PlanPage {
                key,
                id: new_id(),
                section,
                parent,
                title: clean_title(name, false),
                icon: String::new(),
                cover: None,
                blocks: Vec::new(),
            });
            let me = self.pages.len() - 1;
            self.folder(sub, section, Some(me), &format!("{path}/{name}"));
        }
    }

    /// Reads and converts one file into a page. `None` (with a reason in the skipped list) when it cannot.
    fn document(&mut self, name: &str, path: &str, section: usize, parent: Option<usize>) -> Option<usize> {
        let kind = kind_of(name)?;
        if self.pages.len() >= MAX_PAGES + 1 {
            return None;
        }
        if self.stopped.is_none() {
            self.stopped = self.ctl.stop_now();
            if self.stopped == Some(Stop::TimedOut) {
                self.skip(path, "The import took too long (more than 15 minutes). Pages from here on were left out; import them separately.");
            }
        }
        if self.stopped.is_some() {
            return None;
        }
        let vfs = self.vfs;
        let limit = if matches!(kind, Kind::Docx | Kind::Pdf) { MAX_BINARY_FILE } else { MAX_TEXT_FILE };
        let bytes = match vfs.read(path, limit) {
            Ok(b) => b,
            Err(e) => {
                self.skip(path, e);
                return None;
            }
        };
        let dir = path.rsplit_once('/').map_or("", |(d, _)| d).to_string();
        let mut declared: Option<String> = None;
        let mut icon = String::new();
        let mut cover = None;
        let mut blocks;
        {
            let mut ctx = Ctx { vfs, ctl: self.ctl, dir, images: &mut self.images, problems: &mut self.skipped, name: path.to_string() };
            match kind {
                Kind::Markdown => {
                    let d = markdown::parse(&decode_text(&bytes), &mut ctx);
                    declared = d.title;
                    blocks = d.blocks;
                }
                Kind::Text => blocks = plain_text(&decode_text(&bytes)),
                Kind::Html => {
                    let d = html::parse_document(&decode_text(&bytes), &mut ctx);
                    declared = d.title;
                    icon = d.icon.unwrap_or_default();
                    cover = d.cover;
                    blocks = d.blocks;
                }
                Kind::Docx => match docx::parse(bytes, &mut ctx, path) {
                    Ok(d) => {
                        declared = d.title;
                        blocks = d.blocks;
                    }
                    Err(e) => {
                        drop(ctx);
                        self.skip(path, e);
                        return None;
                    }
                },
                Kind::Pdf => {
                    drop(ctx);
                    match pdf::extract_with(&bytes, self.ctl, vfs.budget()) {
                        Ok(d) => {
                            if d.empty_pages > 0 {
                                self.skip(path, format!("{} of {} PDF pages had no text (scanned pages and pictures are not read).", d.empty_pages, d.pages));
                            }
                            if doc::is_empty(&d.blocks) {
                                self.skip(path, "No text was found in this PDF (it may be pictures only).");
                                return None;
                            }
                            self.pdfs += 1;
                            blocks = d.blocks;
                        }
                        Err(e) => {
                            self.skip(path, e);
                            return None;
                        }
                    }
                }
            }
        }
        // A converter that was stopped part way hands back what it had: not a whole page, so it is left out.
        if let Some(stop) = self.ctl.stop_now() {
            self.stopped = Some(stop);
            if stop == Stop::TimedOut {
                self.skip(path, "The import took too long (more than 15 minutes) while this file was being read. It and the pages after it were left out.");
            }
            return None;
        }
        let title = take_title(declared, &mut blocks, name);
        self.pages.push(PlanPage {
            key: path.to_string(),
            id: new_id(),
            section,
            parent,
            title,
            icon: if icon.chars().count() <= 8 { icon } else { String::new() },
            cover,
            blocks,
        });
        Some(self.pages.len() - 1)
    }
}

/// Plain text: blank lines separate paragraphs, single line breaks stay line breaks.
fn plain_text(text: &str) -> Vec<Block> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut blocks = Vec::new();
    let mut inl: Vec<Inline> = Vec::new();
    for line in text.split('\n').chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if !inl.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut inl)));
            }
        } else {
            if !inl.is_empty() {
                inl.push(Inline::Break);
            }
            inl.push(doc::t(line.trim_end()));
        }
    }
    blocks
}

/// The page title: the document's own title, else a first level-1 heading (taken out of the
/// body, since the page shows its title above), else the file name.
fn take_title(declared: Option<String>, blocks: &mut Vec<Block>, name: &str) -> String {
    let first = match blocks.first() {
        Some(Block::Heading(1, inl)) => Some(doc::plain(inl)),
        _ => None,
    };
    let mut title = declared.map(|t| doc::collapse_ws(&t)).filter(|t| !t.is_empty());
    if let Some(h) = first.filter(|h| !h.is_empty() && h.chars().count() <= 150) {
        match &title {
            None => {
                title = Some(h);
                blocks.remove(0);
            }
            Some(t) if t.to_lowercase() == h.to_lowercase() => {
                blocks.remove(0);
            }
            Some(_) => {}
        }
    }
    let title = title.unwrap_or_else(|| file_title(name));
    title.chars().take(MAX_TITLE).collect()
}

fn generic_name(n: &str) -> bool {
    let l = n.trim().to_lowercase();
    l.is_empty() || matches!(l.as_str(), "export" | "untitled" | "private & shared" | "notion" | "imported pages" | "documents" | "download") || l.starts_with("export-")
}

fn date_text(now_ms: i64) -> String {
    let (y, m, d, ..) = crate::export::zip::civil(now_ms);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Builds the plan. `name_hint` is the name of the chosen folder, ZIP file or single file (the
/// notebook is named after it). `now_ms` only names an unnamed notebook ("Imported <date>").
pub fn build(vfs: &Vfs, ctl: &Control, name_hint: Option<&str>, now_ms: i64) -> Result<Plan, String> {
    let mut skipped: Vec<Skipped> = Vec::new();
    let mut unlisted = 0usize;
    let mut note_skip = |skipped: &mut Vec<Skipped>, item: String, reason: String| {
        if skipped.len() < MAX_LISTED {
            skipped.push(Skipped { item, reason });
        } else {
            unlisted += 1;
        }
    };
    for (name, reason) in &vfs.refused {
        note_skip(&mut skipped, name.clone(), reason.clone());
    }

    let paths: Vec<&String> = vfs.paths().collect();
    let html_count = paths.iter().filter(|p| kind_of(p) == Some(Kind::Html)).count();
    let mut root = Dir::default();
    let mut docs = 0usize;
    let mut notion_markdown = false;
    for path in paths {
        let name = path.rsplit('/').next().unwrap_or(path);
        let top_index = !path.contains('/') && name.eq_ignore_ascii_case("index.html");
        match kind_of(name) {
            Some(_) if top_index && vfs.has_archives() && html_count > 1 => {
                note_skip(&mut skipped, path.clone(), "The list of links that Notion adds to an HTML export (the pages themselves are imported).".into());
            }
            Some(kind) => {
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() > MAX_FOLDERS {
                    note_skip(&mut skipped, path.clone(), "Folders nested too deeply.".into());
                    continue;
                }
                docs += 1;
                if docs > MAX_PAGES {
                    return Err(format!(
                        "This import has more than {MAX_PAGES} pages. Import it in smaller parts (for example one top-level folder at a time)."
                    ));
                }
                if kind == Kind::Markdown && strip_notion_id(stem_of(name)) != stem_of(name) {
                    notion_markdown = true;
                }
                root.insert(&parts, path);
            }
            None if extension(name) == "csv" => {
                note_skip(&mut skipped, path.clone(), "A Notion database (table) view. Tables are not imported; the pages inside a database are, when they were exported.".into())
            }
            None if is_picture_name(name) => {} // pictures are taken where a page uses them
            None => note_skip(&mut skipped, path.clone(), "This kind of file is not imported.".into()),
        }
    }
    if docs == 0 {
        return Err("No pages were found. Choose Markdown (.md), HTML, Word (.docx) or PDF files, a folder of them, or a ZIP file made by Notion.".into());
    }

    // A single folder that holds everything (a ZIP's wrapper folder) is not a section.
    let mut top = &root;
    let mut wrapper = String::new();
    while top.files.is_empty() && top.dirs.len() == 1 {
        let (name, sub) = top.dirs.iter().next().expect("one folder");
        wrapper = name.clone();
        top = sub;
    }

    let mut b = Builder { vfs, ctl, pages: Vec::new(), images: Images::default(), skipped, unlisted, stopped: None, pdfs: 0 };
    let mut sections: Vec<String> = Vec::new();
    let used = if top.files.is_empty() {
        HashSet::new()
    } else {
        sections.push(LOOSE_SECTION.to_string());
        b.files_of(top, 0, None)
    };
    for (name, sub) in top.sorted_dirs() {
        if used.contains(name) {
            continue;
        }
        sections.push(clean_title(name, false).chars().take(MAX_NAME).collect());
        let section = sections.len() - 1;
        b.folder(sub, section, None, name);
    }
    match b.stopped {
        Some(Stop::Cancelled) => return Err(CANCELLED.into()),
        Some(Stop::TimedOut) if b.pages.is_empty() => return Err(TOO_SLOW.into()),
        _ => {}
    }
    if b.pages.len() > MAX_PAGES {
        return Err(format!("This import has more than {MAX_PAGES} pages. Import it in smaller parts (for example one top-level folder at a time)."));
    }
    // A section with no page left (its documents could not be read) is not created.
    let mut keep: Vec<usize> = Vec::new();
    for s in 0..sections.len() {
        if b.pages.iter().any(|p| p.section == s) {
            keep.push(s);
        }
    }
    if b.pages.is_empty() {
        let first = b.skipped.first().map(|s| format!(" {}: {}", s.item, s.reason)).unwrap_or_default();
        return Err(format!("None of the files could be imported.{first}"));
    }
    let remap: HashMap<usize, usize> = keep.iter().enumerate().map(|(new, old)| (*old, new)).collect();
    for p in &mut b.pages {
        p.section = remap[&p.section];
    }
    let sections: Vec<String> = keep.iter().map(|i| sections[*i].clone()).collect();

    let hint_given = name_hint.is_some();
    let hint = name_hint.map(|h| clean_title(h, false)).filter(|h| !generic_name(h));
    let wrapper = Some(clean_title(&wrapper, false)).filter(|w| !wrapper.is_empty() && !generic_name(w));
    let notebook = match hint.or(wrapper) {
        Some(n) => n,
        None if hint_given && vfs.has_archives() => "Notion import".to_string(),
        None => format!("Imported {}", date_text(now_ms)),
    };
    let notebook: String = notebook.chars().take(MAX_NAME).collect();

    let mut notes = Vec::new();
    if notion_markdown {
        notes.push("These pages come from Notion's Markdown export, which has no text colors or highlights. Export from Notion as HTML to keep them.".to_string());
    }
    if b.pdfs > 0 {
        notes.push("PDF import reads text only: pictures, scanned pages and the page layout are not imported.".to_string());
    }
    if b.unlisted > 0 {
        notes.push(format!("{} more items were not imported than the list shows.", b.unlisted));
    }
    Ok(Plan { notebook, sections, pages: b.pages, images: b.images, skipped: b.skipped, notes })
}

/// `Err` with the message for the person when the import was cancelled or ran out of time.
pub fn check(ctl: &Control) -> Result<(), String> {
    match ctl.stop_now() {
        None => Ok(()),
        Some(Stop::Cancelled) => Err(CANCELLED.into()),
        Some(Stop::TimedOut) => Err(TOO_SLOW.into()),
    }
}

/// The editor's address for a picture in the vault.
pub fn picture_url(id: &str) -> String {
    format!("http://clatt.localhost/{id}")
}

impl Plan {
    /// Stores every picture the pages use. `put` stores one (mime type, bytes) and returns the
    /// vault's id for it; an error from it stops the import. Returns, for each picture, its
    /// editor address, or `None` when it could not be used (it is then listed as skipped).
    pub fn store_images(&mut self, vfs: &Vfs, ctl: &Control, put: &mut dyn FnMut(&str, &[u8]) -> Result<String, String>) -> Result<Vec<Option<String>>, String> {
        let mut urls = Vec::with_capacity(self.images.list.len());
        let mut total = 0u64;
        for i in 0..self.images.list.len() {
            check(ctl)?;
            let label = self.images.list[i].1.clone();
            let bytes = match std::mem::replace(&mut self.images.list[i].0, ImgSrc::Bytes(Vec::new())) {
                ImgSrc::Bytes(b) => Ok(b),
                ImgSrc::File(path) => vfs.read(&path, MAX_IMAGE),
            };
            let usable = bytes.and_then(|b| match sniff(&b) {
                None => Err("Not a PNG, JPEG, GIF or WebP picture.".to_string()),
                Some(_) if total + b.len() as u64 > MAX_IMAGE_TOTAL => Err("Too many pictures in one import (more than 1 GB).".to_string()),
                Some(mime) => Ok((mime, b)),
            });
            match usable {
                Ok((mime, b)) => {
                    total += b.len() as u64;
                    // An error from the vault (disk full, locked) stops the whole import.
                    urls.push(Some(picture_url(&put(mime, &b)?)));
                }
                Err(reason) => {
                    if self.skipped.len() < MAX_LISTED {
                        self.skipped.push(Skipped { item: format!("Picture {label}"), reason });
                    }
                    urls.push(None);
                }
            }
        }
        Ok(urls)
    }

    /// Renders the pages and returns the new notebook, its sections and pages as vault items,
    /// with the summary. Pages too large to save are left out (their sub-pages move up).
    pub fn items(&mut self, ctl: &Control, urls: &[Option<String>], now: i64, notebook_order: i64) -> Result<(Vec<Item>, ImportSummary), String> {
        let image_out = |n: usize| match urls.get(n) {
            Some(Some(u)) => ImageOut::Url(u.clone()),
            _ => ImageOut::Text(format!("[Picture not imported: {}]", self.images.list.get(n).map_or("picture", |x| x.1.as_str()))),
        };
        let mut dropped: HashSet<usize> = HashSet::new();
        let mut rendered: Vec<Option<(String, String)>> = Vec::new();
        let mut cut_rows = 0usize;
        for _ in 0..3 {
            let ids: HashMap<&str, &str> = self.pages.iter().enumerate().filter(|(i, _)| !dropped.contains(i)).map(|(_, p)| (p.key.as_str(), p.id.as_str())).collect();
            let page_id = |k: &str| ids.get(k).map(|s| s.to_string());
            rendered = Vec::with_capacity(self.pages.len());
            let mut too_big: Vec<usize> = Vec::new();
            let mut total = 0usize;
            cut_rows = 0;
            for (i, p) in self.pages.iter().enumerate() {
                check(ctl)?;
                if dropped.contains(&i) {
                    rendered.push(None);
                    continue;
                }
                match doc::render_limited(&p.blocks, &image_out, &page_id, doc::MAX_RENDER_BYTES) {
                    Ok(r) if r.content.len() <= MAX_PAGE_BYTES && r.text.len() <= MAX_PAGE_BYTES => {
                        total = total.saturating_add(r.content.len() + r.text.len());
                        if total > MAX_TOTAL_BYTES {
                            return Err("This import is too large (more than 512 MB of text). Import it in smaller parts.".into());
                        }
                        cut_rows += r.cut_rows;
                        rendered.push(Some((r.content, r.text)));
                    }
                    _ => {
                        too_big.push(i);
                        rendered.push(None);
                    }
                }
            }
            if too_big.is_empty() {
                break;
            }
            dropped.extend(too_big);
        }
        for i in &dropped {
            if self.skipped.len() < MAX_LISTED {
                self.skipped.push(Skipped { item: self.pages[*i].title.clone(), reason: "The page is too large to store (over 20 MB of text), so it was not imported.".into() });
            }
        }
        let notebook_id = new_id();
        let mut items = vec![Item::Notebook(Notebook {
            id: notebook_id.clone(),
            name: self.notebook.clone(),
            color: NOTEBOOK_COLOR.into(),
            icon: NOTEBOOK_ICON.into(),
            order: notebook_order,
            created: now,
            updated: now,
        })];
        let section_ids: Vec<String> = self.sections.iter().map(|_| new_id()).collect();
        for (i, name) in self.sections.iter().enumerate() {
            items.push(Item::Section(Section {
                id: section_ids[i].clone(),
                notebook_id: notebook_id.clone(),
                name: name.clone(),
                color: String::new(),
                order: i as i64,
                created: now,
                updated: now,
            }));
        }
        let kept = rendered.iter().filter(|r| r.is_some()).count();
        for (i, p) in self.pages.iter().enumerate() {
            let Some((content, text)) = rendered[i].take() else { continue };
            let mut parent = p.parent;
            while let Some(q) = parent {
                if dropped.contains(&q) { parent = self.pages[q].parent } else { break }
            }
            items.push(Item::Page(Page {
                id: p.id.clone(),
                section_id: section_ids[p.section].clone(),
                title: p.title.clone(),
                content,
                text,
                order: i as i64,
                favorite: false,
                icon: p.icon.clone(),
                cover: p.cover.and_then(|c| urls.get(c).cloned().flatten()).unwrap_or_default(),
                parent: parent.map(|q| self.pages[q].id.clone()).unwrap_or_default(),
                trashed_at: None,
                created: now,
                updated: now,
            }));
        }
        let images = urls.iter().flatten().count();
        let mut notes = self.notes.clone();
        if cut_rows > 0 {
            notes.push(format!(
                "{cut_rows} table rows were left out because a table was too large (tables keep at most {} columns and {} cells; a page at most {}).",
                doc::MAX_TABLE_COLS,
                doc::MAX_TABLE_CELLS,
                doc::MAX_PAGE_CELLS
            ));
        }
        let mut skipped = self.skipped.clone();
        skipped.truncate(MAX_LISTED);
        let summary = ImportSummary {
            notebook_id,
            notebook_name: self.notebook.clone(),
            sections: self.sections.len(),
            pages: kept,
            images,
            skipped,
            notes,
        };
        Ok((items, summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::html::tests::NOTION_PAGE;
    use crate::importers::zip;
    use serde_json::Value;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
    const ID: &str = "0123456789abcdef0123456789abcdef";
    const NOW: i64 = 1_700_000_000_000;

    fn vfs(files: &[(&str, &[u8])]) -> Vfs {
        Vfs::from_zip_bytes(zip::build(files, true)).unwrap()
    }

    /// Stores nothing: hands out numbered ids.
    fn fake_store(plan: &mut Plan, v: &Vfs) -> Vec<Option<String>> {
        let mut n = 0;
        plan.store_images(v, &Control::standard(), &mut |_, _| {
            n += 1;
            Ok(format!("{n:032x}"))
        })
        .unwrap()
    }

    fn finish(plan: &mut Plan, v: &Vfs) -> (Vec<Item>, ImportSummary) {
        let urls = fake_store(plan, v);
        plan.items(&Control::standard(), &urls, NOW, 0).unwrap()
    }

    fn page<'a>(items: &'a [Item], title: &str) -> &'a Page {
        items
            .iter()
            .find_map(|i| match i {
                Item::Page(p) if p.title == title => Some(p),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no page {title}"))
    }

    fn json(p: &Page) -> Value {
        serde_json::from_str(&p.content).unwrap()
    }

    #[test]
    fn a_notion_html_zip_keeps_folders_ids_pictures_colors_and_links() {
        let sub = "<html><head><title>Sub page</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Sub page</h1></header><div class=\"page-body\"><p>sub text</p><p><img src=\"Sub%20page%200123456789abcdef0123456789abcdef/deep.png\"/></p></div></article></body></html>";
        let top = format!("Private & Shared/Project plan {ID}.html");
        let dir = format!("Private & Shared/Project plan {ID}");
        let files: Vec<(String, Vec<u8>)> = vec![
            (top, NOTION_PAGE.as_bytes().to_vec()),
            (format!("{dir}/cover.png"), PNG.to_vec()),
            (format!("{dir}/photo.png"), [PNG, b"2"].concat()),
            (format!("{dir}/Sub page {ID}.html"), sub.as_bytes().to_vec()),
            (format!("{dir}/Sub page {ID}/deep.png"), [PNG, b"3"].concat()),
            ("Private & Shared/Database 11111111111111111111111111111111.csv".into(), b"a,b".to_vec()),
            ("index.html".into(), b"<ul><li><a href=\"x\">x</a></li></ul>".to_vec()),
        ];
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&refs);
        let mut plan = build(&v, &Control::standard(), Some("Export-4a9d6e1c-0123-4567-89ab-cdef01234567"), NOW).unwrap();
        assert_eq!(plan.notebook, "Notion import", "a generic ZIP name is not used");
        assert_eq!(plan.sections, [LOOSE_SECTION]);
        assert_eq!(plan.pages.len(), 2);
        assert!(plan.skipped.iter().any(|s| s.item.ends_with(".csv") && s.reason.contains("database")));
        assert!(plan.skipped.iter().any(|s| s.item == "index.html"));
        let (items, summary) = finish(&mut plan, &v);
        assert_eq!((summary.sections, summary.pages, summary.images), (1, 2, 3));
        let main = page(&items, "Project plan");
        let child = page(&items, "Sub page");
        assert_eq!(child.parent, main.id, "the folder next to the page holds its sub-pages");
        assert_eq!(main.parent, "");
        assert_eq!(main.icon, "🚀", "Notion's page icon");
        assert!(main.cover.starts_with("http://clatt.localhost/") && main.cover.len() == "http://clatt.localhost/".len() + 32, "Notion's cover: {}", main.cover);
        let c = &main.content;
        assert!(c.contains(crate::importers::doc::TEXT_COLORS[8]) && c.contains(crate::importers::doc::HIGHLIGHTS[5]), "colors");
        assert!(c.contains("Georgia, Cambria, serif"), "Notion's serif page font");
        assert!(c.contains(&format!("\"pageId\":\"{}\"", child.id)), "the link to the sub-page is a page link");
        let first_image = json(main)["content"].as_array().unwrap().iter().find(|b| b["type"] == "image").unwrap().clone();
        assert!(first_image["attrs"]["src"].as_str().unwrap().starts_with("http://clatt.localhost/"));
        assert_eq!(first_image["attrs"]["caption"], "A photo");
        assert!(child.content.contains("http://clatt.localhost/") && child.text == "sub text");
        assert_eq!(summary.notebook_name, "Notion import");
    }

    #[test]
    fn a_link_inside_a_sentence_to_another_imported_page_becomes_an_internal_link() {
        let all: Vec<(&str, &[u8])> = vec![
            ("One.md", b"# One\n\nSee [the second page](Two.md) and [a missing one](Nope.md) and [the web](https://example.com/x)."),
            ("Two.md", b"# Two\n\nback"),
        ];
        let v = vfs(&all);
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        let (items, _) = finish(&mut plan, &v);
        let two = page(&items, "Two").id.clone();
        let one = page(&items, "One").content.clone();
        assert!(one.contains(&format!("\"href\":\"notaya:page/{two}\"")), "{one}");
        assert!(one.contains("https://example.com/x"));
        assert!(!one.contains("notaya-src") && !one.contains("Nope.md"), "no marker or dead link is left: {one}");
        assert!(one.contains("a missing one"), "the text stays");
    }

    #[test]
    fn markdown_folders_become_sections_and_notion_markdown_nests() {
        let top = format!("Top {ID}.md");
        let mut all: Vec<(&str, &[u8])> = vec![
            ("Work/Plan.md", b"# Plan\n\n![shot](img/a.png)\n"),
            ("Work/img/a.png", PNG),
            ("Work/Deeper/Note.md", b"A note without a heading"),
            ("Page 10.md", b"ten"),
            ("Page 2.md", b"two"),
            ("Home.txt", b"line one\nline two\n\nnext paragraph"),
        ];
        let top_md = "# Top\n\n[Child](Top%200123456789abcdef0123456789abcdef/Child%20fedcba9876543210fedcba9876543210.md)\n\n[Missing](Nope%20fedcba9876543210fedcba9876543211.md)\n".to_string();
        let child = format!("Top {ID}/Child fedcba9876543210fedcba9876543210.md");
        let other = format!("Top {ID}/orphan.bin");
        all.push((top.as_str(), top_md.as_bytes()));
        all.push((child.as_str(), b"# Child\n\nbody"));
        all.push((other.as_str(), b"x"));
        let v = vfs(&all);
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        assert_eq!(plan.notebook, "Imported 2023-11-14", "no name to use: the date");
        assert_eq!(plan.sections, [LOOSE_SECTION, "Work"]);
        assert!(plan.notes.iter().any(|n| n.contains("no text colors")), "{:?}", plan.notes);
        let (items, summary) = finish(&mut plan, &v);
        let titles: Vec<&str> = items.iter().filter_map(|i| if let Item::Page(p) = i { Some(p.title.as_str()) } else { None }).collect();
        let pos = |t: &str| titles.iter().position(|x| *x == t).unwrap_or_else(|| panic!("{t} in {titles:?}"));
        assert!(pos("Page 2") < pos("Page 10"), "numbers sort by value: {titles:?}");
        assert!(pos("Top") < pos("Child"));
        let work = items.iter().find_map(|i| if let Item::Section(s) = i { (s.name == "Work").then(|| s.id.clone()) } else { None }).unwrap();
        assert_eq!(page(&items, "Plan").section_id, work);
        // A folder with no page of its own becomes an empty page that holds the pages.
        let deeper = page(&items, "Deeper");
        assert_eq!(page(&items, "Note").parent, deeper.id);
        assert_eq!(page(&items, "Note").section_id, work);
        // Titles: the first level-1 heading is the title and is not repeated in the body.
        assert!(!page(&items, "Plan").content.contains("\"Plan\""));
        let top_page = page(&items, "Top");
        assert!(top_page.content.contains(&format!("\"pageId\":\"{}\"", page(&items, "Child").id)));
        assert!(top_page.text.contains("Missing"), "a link to a page that is not there stays text");
        // Plain text keeps its line breaks.
        let home = page(&items, "Home");
        assert!(home.content.contains("hardBreak") && home.text.contains("line one\nline two"));
        assert_eq!(summary.images, 1);
        assert!(summary.skipped.iter().any(|s| s.item.ends_with("orphan.bin")));
    }

    #[test]
    fn word_and_pdf_files_are_pages() {
        let docx = crate::importers::docx::tests::sample_docx();
        let pdf = crate::importers::pdf::tests::sample_pdf();
        let v = vfs(&[("Reports/q1.docx", &docx), ("Reports/paper.pdf", &pdf), ("Reports/broken.docx", b"not a zip"), ("Reports/blank.pdf", b"%PDF-1.4 nothing")]);
        let mut plan = build(&v, &Control::standard(), Some("Reports"), NOW).unwrap();
        assert_eq!(plan.notebook, "Reports");
        assert!(plan.notes.iter().any(|n| n.contains("text only")), "{:?}", plan.notes);
        let (items, summary) = finish(&mut plan, &v);
        assert_eq!(summary.sections, 1);
        assert_eq!(summary.pages, 2, "{:?}", summary.skipped);
        assert!(summary.skipped.iter().any(|s| s.item.contains("broken.docx")) && summary.skipped.iter().any(|s| s.item.contains("blank.pdf")), "{:?}", summary.skipped);
        let report = page(&items, "Quarterly report");
        assert!(report.content.contains("\"bold\"") && report.content.contains("http://clatt.localhost/"));
        let paper = page(&items, "Annual Report");
        assert!(paper.text.contains("first line of a paragraph"));
    }

    #[test]
    fn a_failed_picture_is_a_note_in_the_page_and_in_the_list() {
        let v = vfs(&[("a.md", b"# A\n\n![ok](ok.png)\n\n![fake](fake.png)\n\n![gone](gone.png)"), ("ok.png", PNG), ("fake.png", b"GIF is a lie, this is text")]);
        let mut plan = build(&v, &Control::standard(), Some("a.md"), NOW).unwrap();
        let (items, summary) = finish(&mut plan, &v);
        assert_eq!(summary.images, 1);
        let p = page(&items, "A");
        assert_eq!(p.content.matches("http://clatt.localhost/").count(), 1);
        assert!(p.text.contains("[Picture not imported: fake]") && p.text.contains("[Picture not imported: gone]"), "{}", p.text);
        assert!(summary.skipped.iter().any(|s| s.item.contains("fake")) && summary.skipped.iter().any(|s| s.item.contains("gone")), "{:?}", summary.skipped);
    }

    #[test]
    fn nothing_to_import_and_too_much_are_refused_with_a_message() {
        let v = vfs(&[("photo.png", PNG), ("data.json", b"{}")]);
        let e = build(&v, &Control::standard(), None, NOW).err().unwrap();
        assert!(e.contains("No pages were found"), "{e}");
        let names: Vec<String> = (0..=MAX_PAGES).map(|i| format!("p{i}.md")).collect();
        let files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), b"x" as &[u8])).collect();
        let v = vfs(&files);
        let e = build(&v, &Control::standard(), None, NOW).err().unwrap();
        assert!(e.contains("more than 5000 pages"), "{e}");
        // Folders nested too deeply are left out, not followed.
        let deep = vec!["d"; MAX_FOLDERS + 2].join("/") + "/x.md";
        let v = vfs(&[(deep.as_str(), b"x"), ("ok.md", b"y")]);
        let plan = build(&v, &Control::standard(), None, NOW).unwrap();
        assert_eq!(plan.pages.len(), 1);
        assert!(plan.skipped.iter().any(|s| s.reason.contains("nested too deeply")));
    }

    #[test]
    fn unreadable_documents_never_crash_the_import() {
        // Truncated and garbage input for every format: an error or a page, never a panic.
        let docx = crate::importers::docx::tests::sample_docx();
        let pdf = crate::importers::pdf::tests::sample_pdf();
        let html = NOTION_PAGE.as_bytes();
        let md = b"# T\n\n- a\n  - b\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn x() {}\n```\n\n> [!NOTE]\n> hi **bold** [l](https://x.y) ![i](i.png)\n";
        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let run = |name: &str, data: &[u8]| {
            let v = vfs(&[(name, data)]);
            if let Ok(mut p) = build(&v, &Control::standard(), None, NOW) {
                let none = vec![None; p.images.list.len()];
                let _ = p.items(&Control::standard(), &none, NOW, 0);
            }
        };
        let samples: [(&str, &[u8]); 4] = [("f.docx", &docx), ("f.pdf", &pdf), ("f.html", html), ("f.md", md)];
        for (name, data) in samples {
            for cut in (0..data.len()).step_by((data.len() / 40).max(1)) {
                run(name, &data[..cut]);
            }
            for _ in 0..30 {
                let mut bad = data.to_vec();
                for _ in 0..8 {
                    let i = (next() % bad.len() as u64) as usize;
                    bad[i] = next() as u8;
                }
                run(name, &bad);
            }
        }
        let junk: Vec<u8> = (0..5000).map(|_| next() as u8).collect();
        for name in ["j.docx", "j.pdf", "j.html", "j.md", "j.txt"] {
            run(name, &junk);
        }
    }

    #[test]
    fn titles_and_names() {
        let mut blocks = vec![Block::Heading(1, vec![doc::t("  Heading  one ")]), Block::Paragraph(vec![doc::t("body")])];
        assert_eq!(take_title(None, &mut blocks, "file.md"), "Heading one");
        assert_eq!(blocks.len(), 1);
        let mut blocks = vec![Block::Heading(1, vec![doc::t("Same")])];
        assert_eq!(take_title(Some("same".into()), &mut blocks, "f.md"), "same");
        assert!(blocks.is_empty());
        let mut blocks = vec![Block::Heading(2, vec![doc::t("Not a title")])];
        assert_eq!(take_title(None, &mut blocks, &format!("My page {ID}.md")), "My page");
        assert_eq!(blocks.len(), 1);
        assert!(generic_name("Export-abc") && generic_name("Private & Shared") && !generic_name("Trip notes"));
        assert_eq!(natural_cmp("Page 2", "Page 10"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("a", "B"), std::cmp::Ordering::Less);
        assert_eq!(date_text(NOW), "2023-11-14");
    }

    fn cancelled() -> Control {
        Control::new(std::time::Duration::from_secs(60), std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)))
    }

    #[test]
    fn cancel_and_deadline_stop_every_stage_with_a_clear_message() {
        let v = vfs(&[("a.md", b"# A\n\n![x](x.png)"), ("b.md", b"b"), ("x.png", PNG)]);
        assert_eq!(build(&v, &cancelled(), None, NOW).err().unwrap(), CANCELLED);
        assert_eq!(build(&v, &Control::expired(), None, NOW).err().unwrap(), TOO_SLOW);
        // Later stages: picture storing and page writing.
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        let e = plan.store_images(&v, &cancelled(), &mut |_, _| Ok("0".repeat(32))).err().unwrap();
        assert_eq!(e, CANCELLED);
        let e = plan.store_images(&v, &Control::expired(), &mut |_, _| Ok("0".repeat(32))).err().unwrap();
        assert_eq!(e, TOO_SLOW);
        let urls = fake_store(&mut plan, &v);
        assert_eq!(plan.items(&cancelled(), &urls, NOW, 0).err().unwrap(), CANCELLED);
        assert_eq!(plan.items(&Control::expired(), &urls, NOW, 0).err().unwrap(), TOO_SLOW);
        assert!(plan.items(&Control::standard(), &urls, NOW, 0).is_ok());
    }

    #[test]
    fn a_page_too_large_to_write_is_skipped_and_the_others_kept() {
        let v = vfs(&[("big.md", b"# Big\n\nx"), ("small.md", b"# Small\n\ny")]);
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        let big = plan.pages.iter().position(|p| p.title == "Big").unwrap();
        // 60 MB of text is over the page budget (which counts what the JSON will take).
        plan.pages[big].blocks = (0..60).map(|_| Block::Paragraph(vec![doc::t("z".repeat(1_000_000))])).collect();
        let (items, summary) = finish(&mut plan, &v);
        assert_eq!(summary.pages, 1);
        assert!(summary.skipped.iter().any(|s| s.item == "Big" && s.reason.contains("too large")), "{:?}", summary.skipped);
        assert!(items.iter().any(|i| matches!(i, Item::Page(p) if p.title == "Small")));
        assert!(!items.iter().any(|i| matches!(i, Item::Page(p) if p.title == "Big")));
    }

    #[test]
    fn rows_cut_from_a_huge_table_are_reported() {
        let mut src = String::from("| a | b |\n|---|---|\n");
        for _ in 0..15_000 {
            src.push_str("| 1 | 2 |\n");
        }
        let v = vfs(&[("t.md", src.as_bytes())]);
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        // The Markdown reader already stops at 20,000 cells; a wider table from another reader is cut by the writer.
        plan.pages[0].blocks = vec![Block::Table((0..3_000).map(|_| (0..100).map(|_| doc::Cell { header: false, colspan: 1, blocks: vec![] }).collect()).collect())];
        let (_, summary) = finish(&mut plan, &v);
        assert!(summary.notes.iter().any(|n| n.contains("table rows were left out")), "{:?}", summary.notes);
    }

    #[test]
    fn many_files_with_folders_named_like_pages_are_matched_quickly() {
        let _heavy = crate::importers::heavy();
        let start = std::time::Instant::now();
        let mut owned: Vec<(String, Vec<u8>)> = Vec::new();
        for i in 0..2_000 {
            owned.push((format!("Page {i}.md"), b"x".to_vec()));
            owned.push((format!("PAGE {i} extra/sub.md"), b"y".to_vec()));
        }
        let files: Vec<(&str, &[u8])> = owned.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&files);
        let plan = build(&v, &Control::standard(), None, NOW).unwrap();
        assert!(plan.pages.len() >= 2_000);
        assert!(start.elapsed().as_secs() < 180, "{:?}", start.elapsed());
    }
}
