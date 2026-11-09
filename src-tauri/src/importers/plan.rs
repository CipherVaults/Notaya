//! The glue of an import: from the files of a [`Vfs`] to ONE new notebook (or, for a ZIP of
//! several notebooks made by Notaya's own export, one new notebook for each).
//!
//! 1. [`build`] finds the documents (by extension), converts each with the matching converter
//!    and arranges them: one section per top-level folder (or one section "Imported pages" for
//!    loose pages), a page for each document, and Notion's nested folders as sub-pages (a page
//!    `X.md` and a folder `X/` next to it: the pages in `X/` are sub-pages of X; a folder with no
//!    page of its own becomes an empty page that holds its pages). Nothing is written.
//!    When the source has Notaya's marker file at its top (`notaya-export.txt`, see
//!    `crate::export::marker_text`) and its folders have the shape the marker names, the layout
//!    is restored: `notebooks` gives a notebook per top folder and a section per folder inside it,
//!    `sections` a section per top folder (even when there is only one). Without the marker, or
//!    when the folders do not fit it, the import works as described above.
//! 2. [`Plan::store_images`] stores the pictures (the caller gives the function that stores one).
//! 3. [`Plan::items`] renders the pages and returns the vault items of the new notebooks, to be
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
use crate::export::Layout;
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
const MAX_IMAGE_TOTAL: u64 = 2 * 1024 * 1024 * 1024;
/// A page whose editor JSON or text is larger than this is not imported (the editor saves up to
/// 20 million characters).
const MAX_PAGE_BYTES: usize = 20_000_000;
/// The rendered pages of one import together.
const MAX_TOTAL_BYTES: usize = 2 * 1024 * 1024 * 1024;
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
    /// The first (usually the only) new notebook, which the app opens.
    pub notebook_id: String,
    pub notebook_name: String,
    /// Every new notebook, the first one included, in order.
    pub notebooks: Vec<NotebookRef>,
    /// Sections and pages of all the new notebooks together.
    pub sections: usize,
    pub pages: usize,
    pub images: usize,
    pub skipped: Vec<Skipped>,
    pub notes: Vec<String>,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotebookRef {
    pub id: String,
    pub name: String,
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

/// The 32-character id Notion puts at the end of a file name in every export (`Page 0123…cdef.md`, also
/// `Table 0123…cdef_all.csv`). `None` for a name without one.
fn notion_file_id(name: &str) -> Option<String> {
    let base = name.rsplit('/').next().unwrap_or(name);
    let stem = base.rsplit_once('.').map_or(base, |(s, _)| s);
    let stem = stem.strip_suffix("_all").unwrap_or(stem);
    let chars: Vec<char> = stem.chars().collect();
    if chars.len() < 32 {
        return None;
    }
    let id: String = chars[chars.len() - 32..].iter().collect();
    let separated = chars.len() == 32 || chars[chars.len() - 33] == ' ';
    (separated && id.chars().all(|c| c.is_ascii_hexdigit())).then(|| id.to_ascii_lowercase())
}

/// The lines of a paragraph (a line break or a newline ends a line).
fn paragraph_lines(inlines: &[Inline]) -> Vec<String> {
    let mut lines = vec![String::new()];
    for i in inlines {
        match i {
            Inline::Text(t, _) => {
                for (n, part) in t.split('\n').enumerate() {
                    if n > 0 {
                        lines.push(String::new());
                    }
                    lines.last_mut().expect("a line").push_str(part);
                }
            }
            Inline::Break => lines.push(String::new()),
            _ => {}
        }
    }
    lines.retain(|l| !l.trim().is_empty());
    lines
}

/// A paragraph that is only `Key: value` lines, as Notion writes a row's properties at the top of its page.
fn is_property_paragraph(inlines: &[Inline]) -> bool {
    let lines = paragraph_lines(inlines);
    !lines.is_empty() && lines.iter().all(|l| l.split_once(':').is_some_and(|(k, _)| !k.trim().is_empty() && k.chars().count() <= 100))
}

/// The properties table at the top of an HTML row page: every row has a name and a value.
fn is_properties_table(rows: &[Vec<doc::Cell>]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.len() == 2)
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
    /// The new notebooks' names (one, unless a Notaya ZIP of several notebooks is imported).
    pub notebooks: Vec<String>,
    pub sections: Vec<String>,
    /// For each section, the notebook it belongs to (an index into `notebooks`).
    pub section_notebook: Vec<usize>,
    pub pages: Vec<PlanPage>,
    /// CSV inputs are independent of the capped, presentation-only skipped list.
    pub csv_paths: Vec<String>,
    pub images: Images,
    pub skipped: Vec<Skipped>,
    pub notes: Vec<String>,
    /// HTML pages that gave their text to the Markdown page with the same Notion id: (the HTML file, the key of
    /// that page). Links that point at the HTML file go to the page.
    pub aliases: Vec<(String, String)>,
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
    /// Notion's built-in cover as one of Notaya's own (`gradient:<name>` or `solid:#rrggbb`); empty = none.
    pub cover_look: String,
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
    /// Markdown file -> the HTML file of the same page (same Notion id) whose text and formatting it takes.
    twins: HashMap<String, String>,
    /// How many pages did take their text from an HTML twin.
    merged: usize,
}

impl Builder<'_> {
    /// The body of an HTML twin: its blocks (without the title), its icon and cover. `None` when it cannot be used.
    fn html_twin(&mut self, h_path: &str) -> Option<(Vec<Block>, String, Option<usize>, String)> {
        let vfs = self.vfs;
        let bytes = vfs.read(h_path, MAX_TEXT_FILE).ok()?;
        let dir = h_path.rsplit_once('/').map_or("", |(d, _)| d).to_string();
        let (mut blocks, icon, cover, cover_look) = {
            let mut ctx = Ctx { vfs, ctl: self.ctl, dir, images: &mut self.images, problems: &mut self.skipped, name: h_path.to_string() };
            let d = html::parse_document(&decode_text(&bytes), &mut ctx);
            (d.blocks, d.icon.unwrap_or_default(), d.cover, d.cover_look.unwrap_or_default())
        };
        if doc::is_empty(&blocks) {
            return None;
        }
        if matches!(blocks.first(), Some(Block::Heading(1, _))) {
            blocks.remove(0);
        }
        Some((blocks, icon, cover, cover_look))
    }

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
                cover_look: String::new(),
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
        let mut cover_look = String::new();
        let mut blocks;
        // The same page exported as HTML too: its text and formatting are used (colors, highlights, fonts).
        let twin = if kind == Kind::Markdown { self.twins.get(path).cloned().and_then(|h| self.html_twin(&h)) } else { None };
        {
            let mut scratch_images = Images::default();
            let mut scratch_problems: Vec<Skipped> = Vec::new();
            let (images, problems) = if twin.is_some() { (&mut scratch_images, &mut scratch_problems) } else { (&mut self.images, &mut self.skipped) };
            let mut ctx = Ctx { vfs, ctl: self.ctl, dir, images, problems, name: path.to_string() };
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
                    cover_look = d.cover_look.unwrap_or_default();
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
        // "→ Title" rows that link to another document (Notaya's own export) are page links again.
        doc::page_link_rows(&mut blocks);
        let title = take_title(declared, &mut blocks, name);
        if let Some((mut html_blocks, html_icon, html_cover, html_cover_look)) = twin {
            // The Markdown page keeps the property lines of a row (`Status: Done`), which the database import reads;
            // the HTML page has them as a table. The rest of the page comes from the HTML.
            if matches!(html_blocks.first(), Some(Block::Table(rows)) if is_properties_table(rows)) {
                if let Some(Block::Paragraph(inl)) = blocks.first().filter(|b| matches!(b, Block::Paragraph(i) if is_property_paragraph(i))) {
                    let lead = Block::Paragraph(inl.clone());
                    html_blocks[0] = lead;
                }
            }
            blocks = html_blocks;
            if icon.is_empty() {
                icon = html_icon;
            }
            cover = cover.or(html_cover);
            if cover_look.is_empty() {
                cover_look = html_cover_look;
            }
            self.merged += 1;
        }
        self.pages.push(PlanPage {
            key: path.to_string(),
            id: new_id(),
            section,
            parent,
            title,
            icon: if icon.chars().count() <= 8 { icon } else { String::new() },
            cover,
            cover_look,
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

/// Most bytes of a marker file that are read.
const MAX_MARKER: u64 = 4 * 1024;
/// Most notebooks one import may create (a ZIP of more notebooks than this is imported as one
/// notebook, its top folders as sections, as without the marker).
pub const MAX_NOTEBOOKS: usize = 100;

/// Notaya's marker file at the top of the source (see `crate::export::marker_text`): `Some` with
/// the layout it names (`None` inside for a layout this version does not know) when the file is
/// there and starts with the fixed first line. Such a file is not imported as a page.
fn marker(vfs: &Vfs) -> Option<Option<Layout>> {
    let name = crate::export::MARKER_NAME;
    if !vfs.paths().any(|p| p == name) {
        return None;
    }
    let bytes = vfs.read(name, MAX_MARKER).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    let mut lines = text.lines();
    if lines.next()?.trim_end() != crate::export::MARKER_FIRST_LINE {
        return None;
    }
    Some(lines.find_map(|l| l.strip_prefix("layout:")).and_then(Layout::parse))
}

/// The folders have the shape of a ZIP of notebooks: nothing but folders at the top, nothing but
/// section folders in each, and pages in each section folder.
fn notebooks_shape(root: &Dir) -> bool {
    root.files.is_empty() && !root.dirs.is_empty() && root.dirs.len() <= MAX_NOTEBOOKS && root.dirs.values().all(|nb| nb.files.is_empty() && !nb.dirs.is_empty() && nb.dirs.values().all(|s| !s.files.is_empty()))
}

/// The folders have the shape of a ZIP of sections: nothing but folders at the top, with pages in each.
fn sections_shape(root: &Dir) -> bool {
    root.files.is_empty() && !root.dirs.is_empty() && root.dirs.values().all(|s| !s.files.is_empty())
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

    let layout = marker(vfs);
    let paths: Vec<&String> = vfs.paths().collect();
    let html_count = paths.iter().filter(|p| kind_of(p) == Some(Kind::Html)).count();
    let mut root = Dir::default();
    let mut docs = 0usize;
    let mut csv_paths = Vec::new();
    let mut notion_markdown = false;
    // The same Notion page in two formats (Markdown and HTML, zips or loose files, in any mix) has the same id at
    // the end of both file names. The Markdown page is kept and takes the HTML page's text and formatting. The
    // HTML page of a database (its table is imported from the CSV file) is not imported as a page.
    let mut md_ids: HashMap<String, Vec<&String>> = HashMap::new();
    let mut html_ids: HashMap<String, Vec<&String>> = HashMap::new();
    let mut csv_ids: HashSet<String> = HashSet::new();
    for p in &paths {
        let name = p.rsplit('/').next().unwrap_or(p);
        let Some(id) = notion_file_id(name) else { continue };
        match kind_of(name) {
            Some(Kind::Markdown) => md_ids.entry(id).or_default().push(p),
            Some(Kind::Html) => html_ids.entry(id).or_default().push(p),
            _ if extension(name) == "csv" => {
                csv_ids.insert(id);
            }
            _ => {}
        }
    }
    let mut twins: HashMap<String, String> = HashMap::new();
    let mut hidden_html: HashSet<String> = HashSet::new();
    let mut database_pages = 0usize;
    for (id, hs) in &html_ids {
        if let [h] = hs.as_slice() {
            if csv_ids.contains(id) {
                hidden_html.insert((*h).clone());
                database_pages += 1;
            } else if let Some([m]) = md_ids.get(id).map(|v| v.as_slice()) {
                twins.insert((*m).clone(), (*h).clone());
                hidden_html.insert((*h).clone());
            }
        }
    }
    let aliases: Vec<(String, String)> = twins.iter().map(|(m, h)| (h.clone(), m.clone())).collect();
    for path in paths {
        if hidden_html.contains(path) {
            continue;
        }
        if layout.is_some() && path == crate::export::MARKER_NAME {
            continue;
        }
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
                csv_paths.push(path.clone());
                // Pending database work is not an omission, including beyond the report cap.
                if skipped.len() < MAX_LISTED {
                    skipped.push(Skipped { item: path.clone(), reason: "A Notion database view, waiting for the database import step.".into() });
                }
            }
            None if is_picture_name(name) => {} // pictures are taken where a page uses them
            None => note_skip(&mut skipped, path.clone(), "This kind of file is not imported.".into()),
        }
    }
    if docs == 0 {
        return Err("No pages were found. Choose Markdown (.md), HTML, Word (.docx) or PDF files, a folder of them, or a ZIP file made by Notion.".into());
    }

    // Notaya's own ZIP export: its marker says what the top folders are, if they fit.
    let restored = match layout.flatten() {
        Some(Layout::Notebooks) if notebooks_shape(&root) => Some(Layout::Notebooks),
        Some(Layout::Sections) if sections_shape(&root) => Some(Layout::Sections),
        _ => None,
    };
    let folder_name = |n: &str| -> String { clean_title(n, false).chars().take(MAX_NAME).collect() };

    let mut b = Builder { vfs, ctl, pages: Vec::new(), images: Images::default(), skipped, unlisted, stopped: None, pdfs: 0, twins, merged: 0 };
    let mut sections: Vec<String> = Vec::new();
    let mut section_notebook: Vec<usize> = Vec::new();
    // Notebooks named after folders (the notebooks layout); otherwise one, named below.
    let mut notebooks: Vec<String> = Vec::new();
    let mut wrapper = String::new();
    match restored {
        Some(Layout::Notebooks) => {
            for (nb_name, nb) in root.sorted_dirs() {
                notebooks.push(folder_name(nb_name));
                for (sec_name, sec) in nb.sorted_dirs() {
                    sections.push(folder_name(sec_name));
                    section_notebook.push(notebooks.len() - 1);
                    b.folder(sec, sections.len() - 1, None, &format!("{nb_name}/{sec_name}"));
                }
            }
        }
        Some(Layout::Sections) => {
            for (name, sub) in root.sorted_dirs() {
                sections.push(folder_name(name));
                section_notebook.push(0);
                b.folder(sub, sections.len() - 1, None, name);
            }
        }
        _ => {
            // A single folder that holds everything (a ZIP's wrapper folder) is not a section.
            let mut top = &root;
            while top.files.is_empty() && top.dirs.len() == 1 {
                let (name, sub) = top.dirs.iter().next().expect("one folder");
                wrapper = name.clone();
                top = sub;
            }
            let used = if top.files.is_empty() {
                HashSet::new()
            } else {
                sections.push(LOOSE_SECTION.to_string());
                section_notebook.push(0);
                b.files_of(top, 0, None)
            };
            for (name, sub) in top.sorted_dirs() {
                if used.contains(name) {
                    continue;
                }
                sections.push(folder_name(name));
                section_notebook.push(0);
                let section = sections.len() - 1;
                b.folder(sub, section, None, name);
            }
        }
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
    let mut section_notebook: Vec<usize> = keep.iter().map(|i| section_notebook[*i]).collect();
    let sections: Vec<String> = keep.iter().map(|i| sections[*i].clone()).collect();

    let notebooks = if notebooks.is_empty() {
        let hint_given = name_hint.is_some();
        let hint = name_hint.map(|h| clean_title(h, false)).filter(|h| !generic_name(h));
        let wrapper = Some(clean_title(&wrapper, false)).filter(|w| !wrapper.is_empty() && !generic_name(w));
        let notebook = match hint.or(wrapper) {
            Some(n) => n,
            // A ZIP with a general name: most likely Notion's, unless it is Notaya's own.
            None if hint_given && vfs.has_archives() && layout.is_none() => "Notion import".to_string(),
            None => format!("Imported {}", date_text(now_ms)),
        };
        vec![notebook.chars().take(MAX_NAME).collect()]
    } else {
        // A notebook with no section left (its documents could not be read) is not created.
        let used: HashSet<usize> = section_notebook.iter().copied().collect();
        let kept: Vec<usize> = (0..notebooks.len()).filter(|n| used.contains(n)).collect();
        let renumber: HashMap<usize, usize> = kept.iter().enumerate().map(|(new, old)| (*old, new)).collect();
        for n in &mut section_notebook {
            *n = renumber[n];
        }
        kept.iter().map(|i| notebooks[*i].clone()).collect()
    };

    let mut notes = Vec::new();
    if b.merged > 0 {
        notes.push(format!("{} pages were in both exports (Markdown and HTML): each was imported once, with the text, colors and fonts of the HTML and the table data of the Markdown and CSV files.", b.merged));
    }
    if database_pages > 0 {
        notes.push(format!("{database_pages} HTML pages of databases were not imported as pages: their tables come from the CSV files."));
    }
    if notion_markdown && b.merged == 0 {
        notes.push("These pages come from Notion's Markdown export, which has no text colors or highlights. Export from Notion as HTML to keep them, or choose the HTML and the Markdown & CSV exports together.".to_string());
    }
    if b.pdfs > 0 {
        notes.push("PDF import reads text only: pictures, scanned pages and the page layout are not imported.".to_string());
    }
    if b.unlisted > 0 {
        notes.push(format!("{} more items were not imported than the list shows.", b.unlisted));
    }
    Ok(Plan { notebooks, sections, section_notebook, pages: b.pages, csv_paths, images: b.images, skipped: b.skipped, notes, aliases })
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
                Some(_) if total + b.len() as u64 > MAX_IMAGE_TOTAL => Err("Too many pictures in one import (more than 2 GB).".to_string()),
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
            let mut ids: HashMap<&str, &str> = self.pages.iter().enumerate().filter(|(i, _)| !dropped.contains(i)).map(|(_, p)| (p.key.as_str(), p.id.as_str())).collect();
            // A link that points at the HTML file of a page that was imported from its Markdown file.
            for (alias, key) in &self.aliases {
                if let Some(id) = ids.get(key.as_str()).copied() {
                    ids.entry(alias.as_str()).or_insert(id);
                }
            }
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
                            return Err("This import is too large (more than 2 GB of text). Import it in smaller parts.".into());
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
        if self.notebooks.is_empty() {
            return Err("There is nothing to import.".into());
        }
        let notebooks: Vec<NotebookRef> = self.notebooks.iter().map(|name| NotebookRef { id: new_id(), name: name.clone() }).collect();
        let mut items: Vec<Item> = notebooks
            .iter()
            .enumerate()
            .map(|(i, nb)| {
                Item::Notebook(Notebook {
                    source: None,
                    id: nb.id.clone(),
                    name: nb.name.clone(),
                    color: NOTEBOOK_COLOR.into(),
                    icon: NOTEBOOK_ICON.into(),
                    order: notebook_order + i as i64,
                    created: now,
                    updated: now,
                trashed_at: None, })
            })
            .collect();
        let section_ids: Vec<String> = self.sections.iter().map(|_| new_id()).collect();
        let mut in_notebook = vec![0i64; notebooks.len()];
        for (i, name) in self.sections.iter().enumerate() {
            let nb = self.section_notebook.get(i).copied().unwrap_or(0).min(notebooks.len().saturating_sub(1));
            items.push(Item::Section(Section {
                source: None,
                id: section_ids[i].clone(),
                notebook_id: notebooks[nb].id.clone(),
                name: name.clone(),
                color: String::new(),
                order: in_notebook[nb],
                created: now,
                updated: now,
            trashed_at: None, }));
            in_notebook[nb] += 1;
        }
        let kept = rendered.iter().filter(|r| r.is_some()).count();
        for (i, p) in self.pages.iter().enumerate() {
            let Some((content, text)) = rendered[i].take() else { continue };
            let mut parent = p.parent;
            while let Some(q) = parent {
                if dropped.contains(&q) { parent = self.pages[q].parent } else { break }
            }
            items.push(Item::Page(Page {
                source: None,
                id: p.id.clone(),
                section_id: section_ids[p.section].clone(),
                title: p.title.clone(),
                content,
                text,
                order: i as i64,
                favorite: false,
                icon: p.icon.clone(),
                cover: p.cover.and_then(|c| urls.get(c).cloned().flatten()).unwrap_or_else(|| p.cover_look.clone()),
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
            notebook_id: notebooks[0].id.clone(),
            notebook_name: notebooks[0].name.clone(),
            notebooks,
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
        assert_eq!(plan.notebooks, ["Notion import"], "a generic ZIP name is not used");
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
    fn a_notion_built_in_cover_and_a_data_emoji_icon_are_kept_without_fetching_anything() {
        let html = |title: &str, cover: &str| {
            format!(r#"<html><head><title>{title}</title></head><body><article class="page sans"><header><img class="page-cover-image" src="{cover}"/><div class="page-header-icon page-header-icon-with-cover"><span class="icon" data-emoji="🏠"></span></div><h1 class="page-title">{title}</h1></header><div class="page-body"><p>Body text</p></div></article></body></html>"#)
        };
        let gradient = html("Hubs", "https://www.notion.so/images/page-cover/gradients_8.png");
        let other = "cccccccccccccccccccccccccccccccc";
        let files: Vec<(String, Vec<u8>)> = vec![
            // An HTML-only page, and a Markdown page with an HTML twin (the twin carries the icon and the cover).
            (format!("Solo {other}.html"), html("Solo", "https://www.notion.so/images/page-cover/solid_red.png").into_bytes()),
            (format!("Hubs {ID}.md"), b"# Hubs\n\nBody text\n".to_vec()),
            (format!("Hubs {ID}.html"), gradient.into_bytes()),
        ];
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&refs);
        let mut plan = build(&v, &Control::standard(), Some("Export"), NOW).unwrap();
        let (items, summary) = finish(&mut plan, &v);
        assert_eq!(summary.images, 0, "nothing on notion.so is fetched or stored");
        let hubs = page(&items, "Hubs");
        assert_eq!((hubs.icon.as_str(), hubs.cover.as_str()), ("🏠", "gradient:lagoon"));
        let solo = page(&items, "Solo");
        assert_eq!((solo.icon.as_str(), solo.cover.as_str()), ("🏠", "solid:#e5484d"));
    }

    #[test]
    fn a_main_page_next_to_a_folder_of_databases_and_pages_is_imported_for_markdown_and_html() {
        // A Notion "Hubs" main page: the page, a folder of the same name with sub-pages and a database.
        let (hub, sub, db, row) = ("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb1", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb3", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb4");
        let html = |t: &str, body: &str| format!("<html><head><title>{t}</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">{t}</h1></header><div class=\"page-body\">{body}</div></article></body></html>");
        let hub_md = format!("# Hubs\n\n[Sub](Hubs%20{hub}/Sub%20{sub}.md)\n\n[Tasks](Hubs%20{hub}/Tasks%20{db}.csv)\n");
        let md: Vec<(String, Vec<u8>)> = vec![
            (format!("Hubs {hub}.md"), hub_md.into_bytes()),
            (format!("Hubs {hub}/Sub {sub}.md"), b"# Sub\n\ntext\n".to_vec()),
            (format!("Hubs {hub}/Tasks {db}.csv"), b"Name,Status\nOne,Done\n".to_vec()),
            (format!("Hubs {hub}/Tasks {db}/One {row}.md"), b"# One\n\nStatus: Done\n".to_vec()),
        ];
        let refs: Vec<(&str, &[u8])> = md.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&refs);
        let plan = build(&v, &Control::standard(), Some("Export"), NOW).unwrap();
        let titles: Vec<&str> = plan.pages.iter().map(|p| p.title.as_str()).collect();
        assert!(titles.contains(&"Hubs"), "the main page (Markdown) is imported: {titles:?}");
        let hh = html("Hubs", &format!("<p><a href=\"Hubs%20{hub}/Sub%20{sub}.html\">Sub</a></p>"));
        let files: Vec<(String, Vec<u8>)> = vec![
            ("index.html".into(), b"<ul><li><a href=\"x\">x</a></li></ul>".to_vec()),
            (format!("Hubs {hub}.html"), hh.into_bytes()),
            (format!("Hubs {hub}/Sub {sub}.html"), html("Sub", "<p>text</p>").into_bytes()),
            (format!("Hubs {hub}/Tasks {db}.html"), html("Tasks", "<table><tr><td>One</td></tr></table>").into_bytes()),
            (format!("Hubs {hub}/Tasks {db}.csv"), b"Name,Status\nOne,Done\n".to_vec()),
            (format!("Hubs {hub}/Tasks {db}/One {row}.html"), html("One", "<p>x</p>").into_bytes()),
        ];
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&refs);
        let plan = build(&v, &Control::standard(), Some("Export"), NOW).unwrap();
        let titles: Vec<&str> = plan.pages.iter().map(|p| p.title.as_str()).collect();
        assert!(titles.contains(&"Hubs"), "the main page (HTML) is imported: {titles:?}");
    }

    #[test]
    fn a_page_in_both_a_markdown_and_an_html_export_is_imported_once_with_the_html_formatting() {
        let (guide, hand, table, row) = ("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4");
        let html_page = |title: &str, body: &str| format!("<html><head><title>{title}</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">{title}</h1></header><div class=\"page-body\">{body}</div></article></body></html>");
        let row_html = format!(
            "<html><head><title>Row</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Row</h1><table class=\"properties\"><tbody><tr class=\"property-row\"><th>Status</th><td>Done</td></tr></tbody></table></header><div class=\"page-body\"><p><mark class=\"highlight-blue_background\">coloured</mark> body</p></div></article></body></html>"
        );
        let files: Vec<(String, Vec<u8>)> = vec![
            (format!("Guide {guide}.md"), b"# Guide\n\nplain markdown text".to_vec()),
            (format!("Guide {guide}.html"), html_page("Guide", "<p><mark class=\"highlight-blue_background\">highlighted</mark> text</p>").into_bytes()),
            (format!("Handbook {hand}.html"), html_page("Handbook", &format!("<p>See <a href=\"Guide%20{guide}.html\">the guide</a>.</p>")).into_bytes()),
            (format!("Table {table}.html"), html_page("Table", "<p>the table view</p>").into_bytes()),
            (format!("Table {table}_all.csv"), b"Name,Status\nRow,Done\n".to_vec()),
            (format!("Row {row}.md"), b"# Row\n\nStatus: Done\n\nrow markdown body".to_vec()),
            (format!("Row {row}.html"), row_html.into_bytes()),
        ];
        let refs: Vec<(&str, &[u8])> = files.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
        let v = vfs(&refs);
        let mut plan = build(&v, &Control::standard(), Some("Export-4a9d6e1c-0123-4567-89ab-cdef01234567"), NOW).unwrap();
        let mut titles: Vec<&str> = plan.pages.iter().map(|p| p.title.as_str()).collect();
        titles.sort();
        assert_eq!(titles, ["Guide", "Handbook", "Row"], "each page once; the database's own HTML page is not a page");
        assert_eq!(plan.aliases.len(), 2);
        assert!(plan.notes.iter().any(|n| n.contains("were in both exports")), "{:?}", plan.notes);
        assert!(plan.notes.iter().any(|n| n.contains("HTML pages of databases")), "{:?}", plan.notes);
        let (items, _) = finish(&mut plan, &v);
        let g = page(&items, "Guide");
        assert!(g.content.contains(crate::importers::doc::HIGHLIGHTS[5]) && !g.text.contains("plain markdown"), "the HTML text, with its colors: {}", g.text);
        let h = page(&items, "Handbook");
        assert!(h.content.contains(&format!("notaya:page/{}", g.id)), "a link to the HTML file goes to the page imported from the Markdown file: {}", h.content);
        let r = page(&items, "Row");
        assert!(r.text.starts_with("Status: Done"), "the row keeps its property lines for the database import: {:?}", r.text);
        assert!(r.text.contains("coloured") && !r.text.contains("row markdown body"), "{:?}", r.text);
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

    fn marker(layout: &str) -> String {
        format!("Notaya export v1\nlayout: {layout}\n\nThis file tells Notaya how to import this ZIP again.\n")
    }

    fn notebook_of<'a>(items: &'a [Item], page_title: &str) -> &'a str {
        let p = page(items, page_title);
        let nb = items.iter().find_map(|i| if let Item::Section(s) = i { (s.id == p.section_id).then_some(s.notebook_id.as_str()) } else { None }).unwrap();
        items.iter().find_map(|i| if let Item::Notebook(n) = i { (n.id == nb).then_some(n.name.as_str()) } else { None }).unwrap()
    }

    #[test]
    fn a_notaya_zip_of_several_notebooks_comes_back_as_several_notebooks() {
        let m = marker("notebooks");
        let v = vfs(&[
            ("notaya-export.txt", m.as_bytes()),
            ("Alpha/Journal/One.md", b"# One\n\nfirst"),
            ("Alpha/Journal/One/Child.md", b"# Child\n\nchild"),
            ("Alpha/Ideas/Idea.md", b"# Idea\n\nidea"),
            ("Beta/Work/Task.md", b"# Task\n\ntask"),
        ]);
        let mut plan = build(&v, &Control::standard(), Some("2 notebooks"), NOW).unwrap();
        assert_eq!(plan.notebooks, ["Alpha", "Beta"], "named after their folders, not the ZIP");
        assert_eq!(plan.sections, ["Ideas", "Journal", "Work"]);
        assert_eq!(plan.section_notebook, [0, 0, 1]);
        assert_eq!(plan.pages.len(), 4, "the marker is no page");
        let (items, summary) = plan.items(&Control::standard(), &[], NOW, 7).unwrap();
        assert_eq!((summary.notebooks.len(), summary.sections, summary.pages), (2, 3, 4));
        assert_eq!(summary.notebook_name, "Alpha");
        assert_eq!(summary.notebook_id, summary.notebooks[0].id);
        assert_eq!(notebook_of(&items, "Task"), "Beta");
        assert_eq!(notebook_of(&items, "Child"), "Alpha");
        assert_eq!(page(&items, "Child").parent, page(&items, "One").id);
        let orders: Vec<i64> = items.iter().filter_map(|i| if let Item::Notebook(n) = i { Some(n.order) } else { None }).collect();
        assert_eq!(orders, [7, 8]);
        let work_order = items.iter().find_map(|i| if let Item::Section(s) = i { (s.name == "Work").then_some(s.order) } else { None });
        assert_eq!(work_order, Some(0), "sections are counted within their notebook");
        // One notebook: its own name, not the ZIP's.
        let v = vfs(&[("notaya-export.txt", m.as_bytes()), ("Alpha/Journal/One.md", b"# One\n\nx")]);
        let plan = build(&v, &Control::standard(), Some("Alpha export"), NOW).unwrap();
        assert_eq!((plan.notebooks.as_slice(), plan.sections.as_slice()), (["Alpha".to_string()].as_slice(), ["Journal".to_string()].as_slice()));
    }

    #[test]
    fn a_notaya_zip_of_one_section_keeps_its_section_name() {
        let v = vfs(&[("notaya-export.txt", marker("sections").as_bytes()), ("Journal/One.md", b"# One\n\nx"), ("Journal/Two.md", b"# Two\n\ny")]);
        let plan = build(&v, &Control::standard(), Some("Journal"), NOW).unwrap();
        assert_eq!((plan.notebooks.as_slice(), plan.sections.as_slice()), (["Journal".to_string()].as_slice(), ["Journal".to_string()].as_slice()));
        assert_eq!(plan.pages.len(), 2);
        // Without the marker the one folder is a wrapper, as for any other ZIP.
        let v = vfs(&[("Journal/One.md", b"# One\n\nx"), ("Journal/Two.md", b"# Two\n\ny")]);
        assert_eq!(build(&v, &Control::standard(), Some("Journal"), NOW).unwrap().sections, [LOOSE_SECTION]);
        // Pages: as before, the marker is just left out.
        let v = vfs(&[("notaya-export.txt", marker("pages").as_bytes()), ("One.md", b"# One\n\nx"), ("One/Sub.md", b"# Sub\n\ny")]);
        let mut plan = build(&v, &Control::standard(), Some("One"), NOW).unwrap();
        assert_eq!((plan.notebooks.len(), plan.sections.as_slice(), plan.pages.len()), (1, [LOOSE_SECTION.to_string()].as_slice(), 2));
        let (items, _) = plan.items(&Control::standard(), &[], NOW, 0).unwrap();
        assert_eq!(page(&items, "Sub").parent, page(&items, "One").id);
    }

    #[test]
    fn a_marker_that_does_not_fit_changes_nothing() {
        let docs: [(&str, &[u8]); 3] = [("Alpha/Journal/One.md", b"# One\n\nx"), ("Beta/Work/Task.md", b"# Task\n\ny"), ("Beta/Loose.md", b"# Loose\n\nz")];
        let without = build(&vfs(&docs), &Control::standard(), Some("x"), NOW).unwrap();
        assert_eq!(without.sections, ["Alpha", "Beta"]);
        let big = format!("{}{}", marker("notebooks"), "x".repeat(5_000));
        // (marker, is it taken as the marker)
        for (label, m, is_marker) in [
            ("a page directly in a notebook folder", marker("notebooks"), true),
            ("an unknown layout", "Notaya export v1\nlayout: shelves\n".to_string(), true),
            ("another first line", "Notaya export v2\nlayout: notebooks\n".to_string(), false),
            ("too large to be a marker", big, false),
        ] {
            let mut files: Vec<(&str, &[u8])> = docs.to_vec();
            files.push(("notaya-export.txt", m.as_bytes()));
            let plan = build(&vfs(&files), &Control::standard(), Some("x"), NOW).unwrap();
            assert_eq!(plan.notebooks, without.notebooks, "{label}");
            if is_marker {
                assert_eq!(plan.sections, without.sections, "{label}");
                assert_eq!(plan.pages.len(), without.pages.len(), "{label}: the marker is no page");
            } else {
                // Not a marker: an ordinary text file, imported as a page like any other.
                assert_eq!(plan.sections, [LOOSE_SECTION, "Alpha", "Beta"], "{label}");
            }
        }
        // A marker inside a folder is not the ZIP's marker; one with another first line is an ordinary text page.
        let v = vfs(&[("Alpha/notaya-export.txt", marker("notebooks").as_bytes()), ("Alpha/Journal/One.md", b"# One\n\nx")]);
        assert_eq!(build(&v, &Control::standard(), None, NOW).unwrap().pages.len(), 2);
        let v = vfs(&[("notaya-export.txt", b"my own notes"), ("One.md", b"# One\n\nx")]);
        assert_eq!(build(&v, &Control::standard(), None, NOW).unwrap().pages.len(), 2);
        // More notebooks than one import may create: one notebook, as without the marker.
        let names: Vec<String> = (0..=MAX_NOTEBOOKS).map(|i| format!("N{i}/S/p.md")).collect();
        let m = marker("notebooks");
        let mut files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), b"x" as &[u8])).collect();
        files.push(("notaya-export.txt", m.as_bytes()));
        let plan = build(&vfs(&files), &Control::standard(), None, NOW).unwrap();
        assert_eq!(plan.notebooks.len(), 1);
        assert_eq!(plan.sections.len(), MAX_NOTEBOOKS + 1);
    }

    #[test]
    fn links_and_rows_from_a_notaya_zip_point_at_the_imported_pages() {
        let a = "# Plan #1 (draft)\n\nsee [link to b](Plan%20%231%20%28draft%29/Sub%20100%25.md) and [c](../../Other/Caf%C3%A9%20%26%20co/%C3%9Cn%C3%AFcode.md)\n\n\u{2192} [Sub 100%](./Plan%20%231%20%28draft%29/Sub%20100%25.md)\n\n\u{2192} Page not found\n\n- \u{2192} [Ünïcode](../../Other/Caf%C3%A9%20%26%20co/%C3%9Cn%C3%AFcode.md)\n";
        let b = "# Sub 100%\n\nback to [the plan](../Plan%20%231%20%28draft%29.md)\n\n[only a link](../Plan%20%231%20%28draft%29.md#h1)\n";
        let m = marker("notebooks");
        let v = vfs(&[
            ("notaya-export.txt", m.as_bytes()),
            ("My notes/Day 1/Plan #1 (draft).md", a.as_bytes()),
            ("My notes/Day 1/Plan #1 (draft)/Sub 100%.md", b.as_bytes()),
            ("Other/Café & co/Ünïcode.md", b"# \xC3\x9Cn\xC3\xAFcode\n\nhello"),
        ]);
        let mut plan = build(&v, &Control::standard(), None, NOW).unwrap();
        let (items, _) = finish(&mut plan, &v);
        let (pa, pb, pc) = (page(&items, "Plan #1 (draft)"), page(&items, "Sub 100%"), page(&items, "Ünïcode"));
        assert_eq!(pb.parent, pa.id);
        let ja = json(pa);
        let blocks = ja["content"].as_array().unwrap();
        let links: Vec<&str> = blocks[0]["content"].as_array().unwrap().iter().filter_map(|t| t["marks"][0]["attrs"]["href"].as_str()).collect();
        assert_eq!(links, [format!("notaya:page/{}", pb.id), format!("notaya:page/{}", pc.id)]);
        let bid = blocks[1]["attrs"]["bid"].as_str().expect("the page link has a block id");
        assert!(bid.len() == 16 && bid.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(blocks[1], serde_json::json!({ "type": "pageLink", "attrs": { "bid": bid, "pageId": pb.id } }), "the row is a page link again");
        assert_eq!(blocks[2]["type"], "paragraph", "a row to a page that was not exported stays text");
        // A row inside a list too (after the empty line a list item starts with).
        assert_eq!(blocks[3]["content"][0]["content"][1]["type"], "pageLink", "{}", pa.content);
        assert_eq!(blocks[3]["content"][0]["content"][1]["attrs"]["pageId"], pc.id.as_str());
        let jb = json(pb);
        assert!(pb.content.contains(&format!("\"href\":\"notaya:page/{}\"", pa.id)));
        assert_eq!(jb["content"][1]["type"], "pageLink", "a paragraph that is only a link to a page: a page link, as for Notion");
        assert!(!pa.content.contains("notaya-src") && !pb.content.contains("notaya-src"));
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
        assert_eq!(plan.notebooks, ["Imported 2023-11-14"], "no name to use: the date");
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
        assert_eq!(plan.notebooks, ["Reports"]);
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
