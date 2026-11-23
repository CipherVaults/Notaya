//! Exporting pages, sections or notebooks to HTML, Markdown, PDF or Word (V1.4.0), as one
//! document or as separate files in a ZIP.
//!
//! Everything happens in the Rust core: the UI sends only ids and a format, the core decrypts
//! the pages, builds the file and writes it where the person chose in the save dialog. The
//! exported file is NOT encrypted (the dialog says so). It is written to a temporary file next
//! to the destination and renamed into place, and the app's data folder (the vault and its
//! backups) is refused as a destination.
//!
//! Several pages, a section or a notebook become ONE document: each page starts with its title
//! as a heading (on a new page in PDF and Word), sections and the notebook get headings of
//! their own, and sub-pages follow their page one heading level lower. Pages in the trash are
//! left out unless `include_trashed` is set.
//!
//! "Separate files in a ZIP" writes one file per page instead (see [`collect_zip`] and
//! [`write_zip`]): folders `Notebook/Section/` (only the levels the choice spans), the file
//! `Page.<ext>`, and the sub-pages of a page in a folder named like the page, next to it
//! (`Section/Page.md` and `Section/Page/Sub-page.md`). Names are made safe for Windows and made
//! unique within their folder without regard to capitals (`Name (2).md`). The ZIP also holds
//! `notaya-export.txt`, which says only how it is laid out (see [`marker_text`]), and links
//! between its pages are relative links to their files.
//!
//! Links to other pages (`notaya:page/<id>` and the "Link to page" row) become real links only
//! when that page is part of the same export: to its file in a ZIP, or to its place in a single
//! HTML or Word file. Any other one is written as its text.

mod arabic;
mod bidi;
pub mod deflate;
pub mod doc;
pub mod docx;
pub mod html;
pub mod image;
pub mod markdown;
pub mod pdf;
mod pdf_fonts;
mod pdf_unicode;
mod ttf;
mod transaction;
pub mod zip;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::database::{
    clock,
    engine::Ctx,
    types::PropKind,
    value::{self, Val},
};
use crate::error::{Error, Result};
use crate::model::{Item, Notebook, Page, Section};
use doc::{Block, Cell};
use image::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Html,
    Markdown,
    Pdf,
    Docx,
}

impl Format {
    pub fn parse(s: &str) -> Result<Format> {
        match s {
            "html" => Ok(Format::Html),
            "markdown" => Ok(Format::Markdown),
            "pdf" => Ok(Format::Pdf),
            "docx" => Ok(Format::Docx),
            _ => Err(Error::invalid("Unknown export format.")),
        }
    }
    pub fn ext(self) -> &'static str {
        match self {
            Format::Html => "html",
            Format::Markdown => "md",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Format::Html => "HTML page",
            Format::Markdown => "Markdown",
            Format::Pdf => "PDF document",
            Format::Docx => "Word document",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// One or more pages, by id (shown in the order of the sidebar).
    Pages(Vec<String>),
    Section(String),
    Notebook(String),
    /// Several sections, whole (with their pages), in the order of the sidebar.
    Sections(Vec<String>),
    /// Several notebooks, whole, in the order of the sidebar.
    Notebooks(Vec<String>),
}

/// The most sections or notebooks one export may name.
pub const MAX_CHOSEN: usize = 1_000;
/// The most picture data one export holds in memory (the vault allows 900 MB for each picture).
pub const MAX_IMAGE_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub include_subpages: bool,
    pub include_trashed: bool,
    /// More picture data than this in one document is refused with a message.
    pub max_image_bytes: u64,
}

impl Default for Options {
    fn default() -> Options {
        Options { include_subpages: false, include_trashed: false, max_image_bytes: MAX_IMAGE_BYTES }
    }
}

/// One picture from the vault, as stored.
pub struct Picture {
    pub kind: Kind,
    pub bytes: Vec<u8>,
}

pub enum Part {
    /// A notebook or section name.
    Heading { level: u8, text: String },
    /// A page: its title is a heading of `level`; its own headings are moved down by `level`.
    Page { level: u8, title: String, blocks: Vec<Block> },
}

/// Everything one export file holds, already decrypted.
pub struct ExportDoc {
    /// The document's title (file properties, PDF title).
    pub title: String,
    pub parts: Vec<Part>,
    /// The vault pictures the pages show, by attachment id.
    pub images: BTreeMap<String, Picture>,
    /// Pictures a page shows that could not be read from the vault.
    pub missing_images: usize,
}

impl ExportDoc {
    /// The pictures in the order the pages show them, each once.
    pub fn image_order(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for part in &self.parts {
            if let Part::Page { blocks, .. } = part {
                doc::attachments(blocks, &mut ids);
            }
        }
        ids.retain(|id| self.images.contains_key(id));
        ids
    }

    pub fn page_count(&self) -> usize {
        self.parts.iter().filter(|p| matches!(p, Part::Page { .. })).count()
    }

    /// The pages (numbered from 1 in document order) that links inside this document point at.
    pub fn anchor_targets(&self) -> HashSet<usize> {
        let mut out = HashSet::new();
        for part in &self.parts {
            if let Part::Page { blocks, .. } = part {
                doc::anchor_targets(blocks, &mut out);
            }
        }
        out
    }
}

/// What an export wrote, for the message shown afterwards.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    pub path: String,
    pub pages: usize,
    pub images: usize,
    /// Pictures that could not be read from the vault.
    pub missing_images: usize,
    /// Pictures the PDF could not hold (GIF, WebP, interlaced PNG).
    pub images_left_out: usize,
    /// Characters the PDF fonts do not have (shown as "?").
    pub chars_replaced: usize,
    /// Of those, Chinese, Japanese or Korean characters because the PC has no usable font for them.
    pub chars_no_font: usize,
    /// The folder the Markdown pictures were written to, if any.
    pub assets_folder: Option<String>,
    /// Plaintext files kept because safe cleanup failed or ownership could not be proved.
    pub cleanup_warnings: Vec<String>,
    /// "Separate files in a ZIP": how many files the ZIP holds (pages and Markdown pictures).
    pub files: usize,
    /// Whether the export is a ZIP of separate files.
    pub archive: bool,
}

/* ---------- Choosing the pages ---------- */

struct Tree<'a> {
    pages: Vec<&'a Page>,
}

fn sorted_pages<'a>(items: &[&'a Item], opts: &Options) -> Tree<'a> {
    let mut pages: Vec<&Page> = items
        .iter()
        .filter_map(|i| match i {
            Item::Page(p) if opts.include_trashed || p.trashed_at.is_none() => Some(p),
            _ => None,
        })
        .collect();
    pages.sort_by_key(|p| (p.order, p.created));
    Tree { pages }
}

impl<'a> Tree<'a> {
    /// Pages at the top of a section: no parent, or a parent that is not shown.
    fn roots(&self, section_id: &str) -> Vec<&'a Page> {
        let here: HashSet<&str> = self.pages.iter().filter(|p| p.section_id == section_id).map(|p| p.id.as_str()).collect();
        self.pages.iter().copied().filter(|p| p.section_id == section_id && (p.parent.is_empty() || !here.contains(p.parent.as_str()))).collect()
    }

    fn children(&self, parent: &str) -> Vec<&'a Page> {
        self.pages.iter().copied().filter(|p| p.parent == parent).collect()
    }

    /// A page followed by the pages inside it (depth first), with their depth below it.
    fn subtree(&self, page: &'a Page, depth: u8, with_children: bool, out: &mut Vec<(&'a Page, u8)>, seen: &mut HashSet<String>) {
        if !seen.insert(page.id.clone()) {
            return;
        }
        out.push((page, depth));
        if with_children {
            for c in self.children(&page.id) {
                self.subtree(c, depth.saturating_add(1), true, out, seen);
            }
        }
    }
}

/// The name of a notebook, section or page, and the file name the save dialog suggests.
pub fn suggested_name(items: &[&Item], scope: &Scope) -> Result<String> {
    let nb_name = |id: &str| {
        items.iter().find_map(|i| match i {
            Item::Notebook(n) if n.id == id => Some(n.name.clone()),
            _ => None,
        })
    };
    let sec_name = |id: &str| {
        items.iter().find_map(|i| match i {
            Item::Section(s) if s.id == id => Some(s.name.clone()),
            _ => None,
        })
    };
    let several = |ids: &[String], name: &dyn Fn(&str) -> Option<String>| -> Result<String> {
        let first = ids.first().ok_or_else(|| Error::invalid("Choose what to export."))?;
        let title = name(first).ok_or(Error::NotFound)?;
        Ok(if ids.len() > 1 { format!("{} and {} more", page_title(&title), ids.len() - 1) } else { title })
    };
    let title = match scope {
        Scope::Notebook(id) => nb_name(id).ok_or(Error::NotFound)?,
        Scope::Section(id) => sec_name(id).ok_or(Error::NotFound)?,
        Scope::Notebooks(ids) => several(ids, &nb_name)?,
        Scope::Sections(ids) => several(ids, &sec_name)?,
        Scope::Pages(ids) => {
            let first = ids.first().ok_or_else(|| Error::invalid("Choose at least one page to export."))?;
            let title = items
                .iter()
                .find_map(|i| match i {
                    Item::Page(p) if &p.id == first => Some(p.title.clone()),
                    _ => None,
                })
                .ok_or(Error::NotFound)?;
            if ids.len() > 1 { format!("{} and {} more", page_title(&title), ids.len() - 1) } else { title }
        }
    };
    Ok(safe_file_name(&title))
}

fn page_title(t: &str) -> String {
    if t.trim().is_empty() { "Untitled".to_string() } else { t.trim().to_string() }
}

/// How a choice is laid out: with notebook and section headings (or folders), with section
/// headings only, or as pages alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Notebooks,
    Sections,
    Pages,
}

/// The chosen pages of one section, in order, each with its depth below the chosen page.
struct Group<'a> {
    notebook_id: &'a str,
    notebook: &'a str,
    section_id: &'a str,
    section: &'a str,
    pages: Vec<(&'a Page, u8)>,
}

struct Selection<'a> {
    shape: Shape,
    groups: Vec<Group<'a>>,
    /// The document's title (file properties, PDF title).
    title: String,
}

/// What `scope` names, in the order of the sidebar. Sections and notebooks must exist; pages that
/// are gone or in the trash are left out. Nothing selected is an error.
fn select<'a>(items: &[&'a Item], scope: &Scope, opts: &Options) -> Result<Selection<'a>> {
    let tree = sorted_pages(items, opts);
    let mut notebooks: Vec<&'a Notebook> = items
        .iter()
        .filter_map(|i| match i {
            Item::Notebook(n) => Some(n),
            _ => None,
        })
        .collect();
    notebooks.sort_by_key(|n| (n.order, n.created));
    let sections_of = |notebook: &str| -> Vec<&'a Section> {
        let mut v: Vec<&'a Section> = items
            .iter()
            .filter_map(|i| match i {
                Item::Section(s) if s.notebook_id == notebook => Some(s),
                _ => None,
            })
            .collect();
        v.sort_by_key(|s| (s.order, s.created));
        v
    };
    let mut seen = HashSet::new();
    let section_group = |nb: &'a Notebook, sec: &'a Section, seen: &mut HashSet<String>| -> Group<'a> {
        let mut list = Vec::new();
        for r in tree.roots(&sec.id) {
            tree.subtree(r, 0, opts.include_subpages, &mut list, seen);
        }
        Group { notebook_id: &nb.id, notebook: &nb.name, section_id: &sec.id, section: &sec.name, pages: list }
    };
    let counted = |n: usize, one: &str, word: &str| if n == 1 { one.to_string() } else { format!("{n} {word}") };
    let (shape, groups, title) = match scope {
        Scope::Notebook(_) | Scope::Notebooks(_) => {
            let ids: Vec<String> = match scope {
                Scope::Notebook(id) => vec![id.clone()],
                Scope::Notebooks(ids) => ids.clone(),
                _ => Vec::new(),
            };
            if ids.is_empty() || ids.len() > MAX_CHOSEN {
                return Err(Error::invalid("Choose what to export."));
            }
            let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
            let chosen: Vec<&Notebook> = notebooks.iter().copied().filter(|n| wanted.contains(n.id.as_str())).collect();
            if chosen.len() != wanted.len() {
                return Err(Error::NotFound);
            }
            let mut groups = Vec::new();
            for nb in &chosen {
                for sec in sections_of(&nb.id) {
                    groups.push(section_group(nb, sec, &mut seen));
                }
            }
            (Shape::Notebooks, groups, counted(chosen.len(), &chosen[0].name, "notebooks"))
        }
        Scope::Section(_) | Scope::Sections(_) => {
            let ids: Vec<String> = match scope {
                Scope::Section(id) => vec![id.clone()],
                Scope::Sections(ids) => ids.clone(),
                _ => Vec::new(),
            };
            if ids.is_empty() || ids.len() > MAX_CHOSEN {
                return Err(Error::invalid("Choose what to export."));
            }
            let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
            let mut groups = Vec::new();
            let mut first = None;
            for nb in &notebooks {
                for sec in sections_of(&nb.id) {
                    if wanted.contains(sec.id.as_str()) {
                        first.get_or_insert(sec.name.as_str());
                        groups.push(section_group(nb, sec, &mut seen));
                    }
                }
            }
            if groups.len() != wanted.len() {
                return Err(Error::NotFound);
            }
            let title = counted(groups.len(), first.unwrap_or(""), "sections");
            (Shape::Sections, groups, title)
        }
        Scope::Pages(ids) => {
            // In the sidebar's order: notebook, section, then position in the page tree.
            let mut position: BTreeMap<&str, usize> = BTreeMap::new();
            let mut all = Vec::new();
            let mut walk_seen = HashSet::new();
            for nb in &notebooks {
                for sec in sections_of(&nb.id) {
                    for r in tree.roots(&sec.id) {
                        tree.subtree(r, 0, true, &mut all, &mut walk_seen);
                    }
                }
            }
            for (i, (p, _)) in all.iter().enumerate() {
                position.insert(p.id.as_str(), i);
            }
            let mut chosen: Vec<&Page> = ids.iter().filter_map(|id| tree.pages.iter().copied().find(|p| &p.id == id)).collect();
            chosen.sort_by_key(|p| position.get(p.id.as_str()).copied().unwrap_or(usize::MAX));
            chosen.dedup_by(|a, b| a.id == b.id);
            let mut list = Vec::new();
            for p in &chosen {
                tree.subtree(p, 0, opts.include_subpages, &mut list, &mut seen);
            }
            let title = match chosen.as_slice() {
                [one] => page_title(&one.title),
                _ => format!("{} pages", list.len()),
            };
            // Group the pages by the section they are in (neighbours in the same section share a group).
            let section_of = |id: &str| {
                items.iter().find_map(|i| match i {
                    Item::Section(s) if s.id == id => Some(s),
                    _ => None,
                })
            };
            let mut groups: Vec<Group<'a>> = Vec::new();
            for (p, depth) in list {
                if groups.last().is_none_or(|g| g.section_id != p.section_id) {
                    let sec = section_of(&p.section_id);
                    let nb = sec.and_then(|s| notebooks.iter().copied().find(|n| n.id == s.notebook_id));
                    groups.push(Group {
                        notebook_id: nb.map_or("", |n| n.id.as_str()),
                        notebook: nb.map_or("", |n| n.name.as_str()),
                        section_id: &p.section_id,
                        section: sec.map_or("", |s| s.name.as_str()),
                        pages: Vec::new(),
                    });
                }
                if let Some(g) = groups.last_mut() {
                    g.pages.push((p, depth));
                }
            }
            (Shape::Pages, groups, title)
        }
    };
    if groups.iter().all(|g| g.pages.is_empty()) {
        return Err(Error::invalid(match scope {
            Scope::Pages(_) => "There is nothing to export: the chosen pages are in the trash or no longer exist.",
            _ => "There is nothing to export: there are no pages here (pages in the trash are left out).",
        }));
    }
    Ok(Selection { shape, groups, title })
}

fn title_lookup<'a>(items: &'a [&'a Item], opts: &'a Options) -> impl Fn(&str) -> Option<String> + 'a {
    move |id: &str| {
        items.iter().find_map(|i| match i {
            Item::Page(p) if p.id == id && (opts.include_trashed || p.trashed_at.is_none()) => Some(page_title(&p.title)),
            _ => None,
        })
    }
}

pub const MAX_DATABASE_ROWS: usize = 1_000;

/// Resolves inline databases from the same snapshot and query engine as `query_view`.
/// No commands, disk access or UI-provided database content are involved. This also serves
/// readers of the document model (for example `page.get`) before choosing a writer.
pub fn resolve_databases(blocks: &mut [Block], ctx: &Ctx<'_>) {
    for b in blocks {
        match b {
            Block::Database { id, view, table } => *table = database_table(ctx, id, view),
            Block::List { items, .. } => items.iter_mut().for_each(|i| resolve_databases(&mut i.blocks, ctx)),
            Block::Quote(x) | Block::Toggle(x) | Block::Shelf(x) | Block::Callout { blocks: x, .. } => resolve_databases(x, ctx),
            Block::Columns(cols) => cols.iter_mut().for_each(|c| resolve_databases(c, ctx)),
            Block::Table(rows) => rows.iter_mut().flatten().for_each(|c| resolve_databases(&mut c.blocks, ctx)),
            _ => {}
        }
    }
}

fn database_table(ctx: &Ctx<'_>, id: &str, view_id: &str) -> Option<doc::DatabaseTable> {
    let db = ctx.dbs.get(id)?; // Ctx excludes binned databases and rows, even for a trash export.
    // DatabaseView falls back to the first saved view when its id is empty or was removed.
    let view = db.view(view_id).or_else(|| db.views.first())?;
    let result = ctx.run_view(db, view);
    let mut props: Vec<_> = db.props.iter().filter(|p| p.kind == PropKind::Title).collect();
    for c in view.columns.iter().filter(|c| c.visible) {
        let Some(p) = db.prop(&c.prop) else { continue };
        if !matches!(p.kind, PropKind::Title | PropKind::Button) && !props.iter().any(|q| q.id == p.id) {
            props.push(p);
        }
    }
    let cell = |text: String, header| Cell {
        header, colspan: 1, rowspan: 1, widths: vec![],
        blocks: vec![Block::Paragraph { align: doc::Align::Left, content: vec![doc::Inline::Text(text, doc::Marks::default())] }],
    };
    let mut rows = vec![props.iter().map(|p| cell(p.name.clone(), true)).collect()];
    for row in result.rows.iter().take(MAX_DATABASE_ROWS) {
        let r = ctx.by_id.get(row.id.as_str())?;
        rows.push(props.iter().map(|p| {
            let text = ctx.value(db, r, p)
                .and_then(|v| database_cell_text(&v, p))
                .unwrap_or_else(|e| format!("Error: {e}"));
            cell(text, false)
        }).collect());
    }
    // The screen engine caps its response at 50,000. Count matching rows again only when
    // it hit that cap, so the export's omitted-row count remains exact for larger databases.
    let count = if result.truncated {
        ctx.rows_of(id).iter().filter(|r| view.filter.is_empty() || ctx.matches(db, r, &view.filter)).count()
    } else {
        result.filtered
    };
    Some(doc::DatabaseTable { title: if db.title.is_empty() { "Untitled database".into() } else { db.title.clone() }, more: count.saturating_sub(MAX_DATABASE_ROWS), rows })
}

fn database_cell_text(v: &Val, p: &crate::database::types::Property) -> std::result::Result<String, String> {
    match v {
        Val::Num(n) => value::format_number(n, &p.number_format),
        Val::Date(d) => Ok(value::format_date(d, &p.date_format)),
        Val::Bool(b) => Ok(if *b { "Yes" } else { "No" }.into()),
        Val::List(list) => Ok(list.iter().map(|v| database_cell_text(v, p)).collect::<std::result::Result<Vec<_>, _>>()?.join(", ")),
        _ => Ok(v.text()),
    }
}

/// Builds the document for an export. `attachment` reads one picture from the vault.
pub fn collect(items: &[&Item], scope: &Scope, opts: &Options, attachment: &dyn Fn(&str) -> Option<(String, Vec<u8>)>) -> Result<ExportDoc> {
    let sel = select(items, scope, opts)?;
    let title_of = title_lookup(items, opts);
    let (now, tz) = clock::local_now();
    let databases = Ctx::new(items.iter().copied(), now, tz);
    // The pages of the document, numbered in order: a link to one of them goes to its place in
    // the file (HTML and Word; Markdown and PDF keep the text).
    let mut number: HashMap<&str, usize> = HashMap::new();
    for (p, _) in sel.groups.iter().flat_map(|g| &g.pages) {
        let n = number.len() + 1;
        number.entry(p.id.as_str()).or_insert(n);
    }
    let page_ref = |id: &str| number.get(id).map(|n| doc::PageRef::Anchor(*n));
    let mut parts = Vec::new();
    let mut last_notebook: Option<&str> = None;
    for g in &sel.groups {
        let base: u8 = match sel.shape {
            Shape::Notebooks => {
                if last_notebook != Some(g.notebook_id) {
                    parts.push(Part::Heading { level: 1, text: g.notebook.to_string() });
                    last_notebook = Some(g.notebook_id);
                }
                parts.push(Part::Heading { level: 2, text: g.section.to_string() });
                3
            }
            Shape::Sections => {
                parts.push(Part::Heading { level: 1, text: g.section.to_string() });
                2
            }
            Shape::Pages => 1,
        };
        for (p, depth) in &g.pages {
            let level = base.saturating_add(*depth).min(6);
            let mut blocks = doc::parse_with(&p.content, &title_of, &page_ref);
            resolve_databases(&mut blocks, &databases);
            parts.push(Part::Page { level, title: page_title(&p.title), blocks });
        }
    }
    let mut out = ExportDoc { title: sel.title, parts, images: BTreeMap::new(), missing_images: 0 };
    let mut ids = Vec::new();
    for part in &out.parts {
        if let Part::Page { blocks, .. } = part {
            doc::attachments(blocks, &mut ids);
        }
    }
    let mut total = 0u64;
    for id in ids {
        match attachment(&id).and_then(|(mime, bytes)| Kind::from_mime(&mime).map(|kind| Picture { kind, bytes })) {
            Some(p) => {
                total += p.bytes.len() as u64;
                if total > opts.max_image_bytes {
                    return Err(Error::invalid(format!(
                        "The pictures in these pages add up to more than {} MB, which is too much for one file. Choose fewer pages, or export them as separate files in a ZIP.",
                        opts.max_image_bytes / (1024 * 1024)
                    )));
                }
                out.images.insert(id, p);
            }
            None => out.missing_images += 1,
        }
    }
    Ok(out)
}

/* ---------- Separate files in a ZIP ---------- */

/// One page of a ZIP export: where it goes and what it says (pictures are read when it is written).
pub struct ZipFile {
    /// The folders above the file (safe names, unique at their level).
    pub dir: Vec<String>,
    /// The file name without extension (unique in its folder).
    pub stem: String,
    pub title: String,
    pub blocks: Vec<Block>,
}

pub struct ZipDoc {
    pub files: Vec<ZipFile>,
    /// What the top folders are, for the marker file (see [`MARKER_NAME`]).
    pub layout: Layout,
}

/// How a ZIP export is laid out, as its marker file says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// One top folder per notebook, with a folder per section inside.
    Notebooks,
    /// One top folder per section.
    Sections,
    /// The chosen pages, in section folders only when they come from several sections.
    Pages,
}

impl Layout {
    pub fn name(self) -> &'static str {
        match self {
            Layout::Notebooks => "notebooks",
            Layout::Sections => "sections",
            Layout::Pages => "pages",
        }
    }

    pub fn parse(s: &str) -> Option<Layout> {
        match s.trim() {
            "notebooks" => Some(Layout::Notebooks),
            "sections" => Some(Layout::Sections),
            "pages" => Some(Layout::Pages),
            _ => None,
        }
    }
}

/// The small text file at the top of every ZIP export that tells Notaya's import how the ZIP is
/// laid out (so several notebooks come back as several notebooks). It holds no note content:
/// only [`MARKER_FIRST_LINE`], the layout, and a sentence for whoever opens it.
pub const MARKER_NAME: &str = "notaya-export.txt";
pub const MARKER_FIRST_LINE: &str = "Notaya export v1";

/// The marker file's text for a layout.
pub fn marker_text(layout: Layout) -> String {
    format!(
        "{MARKER_FIRST_LINE}\nlayout: {}\n\nThis file tells Notaya how to import this ZIP again (Import pages). It holds no notes.\n",
        layout.name()
    )
}

/// A name for a folder or file, made safe for Windows. An empty name becomes `fallback`.
fn safe_part(name: &str, fallback: &str) -> String {
    let t = name.trim();
    if t.is_empty() {
        return fallback.to_string();
    }
    let s = safe_file_name(t);
    if s == "Notaya export" && t != "Notaya export" { fallback.to_string() } else { s }
}

/// Hands out names that are unique inside a folder, ignoring capitals (Windows treats `A` and `a`
/// as one name), by adding ` (2)`, ` (3)`…
#[derive(Default)]
struct Namer {
    used: HashMap<String, HashSet<String>>,
    next: HashMap<String, usize>,
}

impl Namer {
    /// `occupies` lists every name in the folder that the choice `base` would take.
    fn claim(&mut self, dir: &[String], base: &str, occupies: &dyn Fn(&str) -> Vec<String>) -> String {
        let key = dir.join("/").to_lowercase();
        let hint_key = format!("{key}\u{0}{}", base.to_lowercase());
        let set = self.used.entry(key).or_default();
        let mut n = self.next.get(&hint_key).copied().unwrap_or(1);
        loop {
            let cand = if n == 1 { base.to_string() } else { format!("{base} ({n})") };
            let names: Vec<String> = occupies(&cand).into_iter().map(|s| s.to_lowercase()).collect();
            if names.iter().all(|x| !set.contains(x)) {
                set.extend(names);
                self.next.insert(hint_key, n + 1);
                return cand;
            }
            n += 1;
        }
    }
}

/// Deepest folder level for sub-pages; pages below it share the folder of the level above.
const MAX_SUBPAGE_FOLDERS: usize = 4;
/// A ZIP holds at most this many files (the format's own limit, without ZIP64).
pub const MAX_ZIP_ENTRIES: usize = 65_535;
/// The most bytes one ZIP export is built from in memory.
pub const MAX_ZIP_BYTES: u64 = 1024 * 1024 * 1024;

/// Lays out the files of a ZIP export for the pages `scope` names.
pub fn collect_zip(items: &[&Item], scope: &Scope, opts: &Options, format: Format) -> Result<ZipDoc> {
    let sel = select(items, scope, opts)?;
    let page_total: usize = sel.groups.iter().map(|g| g.pages.len()).sum();
    // One entry more for the marker file.
    if page_total + 1 > MAX_ZIP_ENTRIES {
        return Err(Error::invalid(format!("A ZIP can hold at most {MAX_ZIP_ENTRIES} files. Choose fewer pages.")));
    }
    let title_of = title_lookup(items, opts);
    let (now, tz) = clock::local_now();
    let databases = Ctx::new(items.iter().copied(), now, tz);
    let mut namer = Namer::default();
    // The marker's name is taken first, so no folder or page can have it.
    namer.claim(&[], MARKER_NAME, &|c| vec![c.to_string()]);
    let ext = format.ext();
    let folder = |namer: &mut Namer, parent: &[String], name: &str, fallback: &str| -> String {
        namer.claim(parent, &safe_part(name, fallback), &|c| vec![c.to_string()])
    };
    // Folders already handed out: by notebook id and by section id.
    let mut nb_dirs: HashMap<String, Vec<String>> = HashMap::new();
    let mut sec_dirs: HashMap<String, Vec<String>> = HashMap::new();
    let multi_section = sel.groups.iter().filter(|g| !g.pages.is_empty()).map(|g| g.section_id).collect::<HashSet<_>>().len() > 1;
    // The folder where the sub-pages of a page go, by page id.
    let mut child_dirs: HashMap<&str, Vec<String>> = HashMap::new();
    // Every page's place first (folders, unique file name), then the pages are read, so a link
    // to any page of the ZIP can be written as a relative path to its file.
    let mut placed: Vec<(Vec<String>, String, String, &Page)> = Vec::new();
    for g in &sel.groups {
        if g.pages.is_empty() {
            continue;
        }
        let mut base: Vec<String> = Vec::new();
        if sel.shape == Shape::Notebooks {
            base = match nb_dirs.get(g.notebook_id) {
                Some(d) => d.clone(),
                None => {
                    let d = vec![folder(&mut namer, &[], g.notebook, "Notebook")];
                    nb_dirs.insert(g.notebook_id.to_string(), d.clone());
                    d
                }
            };
        }
        if sel.shape != Shape::Pages || multi_section {
            base = match sec_dirs.get(g.section_id) {
                Some(d) => d.clone(),
                None => {
                    let mut d = base.clone();
                    d.push(folder(&mut namer, &base, g.section, "Section"));
                    sec_dirs.insert(g.section_id.to_string(), d.clone());
                    d
                }
            };
        }
        for (p, _) in &g.pages {
            let dir = match child_dirs.get(p.parent.as_str()) {
                Some(d) if !p.parent.is_empty() => d.clone(),
                _ => base.clone(),
            };
            let title = page_title(&p.title);
            let stem = namer.claim(&dir, &safe_part(&title, "Untitled"), &|c| {
                let mut v = vec![format!("{c}.{ext}"), c.to_string()];
                if format == Format::Markdown {
                    v.push(format!("{c}.assets"));
                }
                v
            });
            // Sub-pages go in a folder named like the page; very deep ones stay in the folder above.
            let mut child = dir.clone();
            if dir.len() < base.len() + MAX_SUBPAGE_FOLDERS {
                child.push(stem.clone());
            }
            child_dirs.insert(p.id.as_str(), child);
            placed.push((dir, stem, title, p));
        }
    }
    let file_of: HashMap<&str, (&[String], String)> = placed.iter().map(|(dir, stem, _, p)| (p.id.as_str(), (dir.as_slice(), format!("{stem}.{ext}")))).collect();
    let mut files = Vec::with_capacity(placed.len());
    for (dir, stem, title, p) in &placed {
        let page_ref = |id: &str| file_of.get(id).map(|(to, name)| doc::PageRef::File(relative_href(dir, to, name)));
        let mut blocks = doc::parse_with(&p.content, &title_of, &page_ref);
        resolve_databases(&mut blocks, &databases);
        files.push(ZipFile { dir: dir.clone(), stem: stem.clone(), title: title.clone(), blocks });
    }
    let layout = match sel.shape {
        Shape::Notebooks => Layout::Notebooks,
        Shape::Sections => Layout::Sections,
        Shape::Pages => Layout::Pages,
    };
    Ok(ZipDoc { files, layout })
}

/// The link from a file in folder `from` to the file `name` in folder `to`, both inside the same
/// ZIP: `..` for each folder up, then the folders down, every name percent-encoded. Only names
/// from the exporter's own table go in.
fn relative_href(from: &[String], to: &[String], name: &str) -> String {
    let common = from.iter().zip(to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_string(); from.len() - common];
    parts.extend(to[common..].iter().map(|d| doc::url_segment(d)));
    parts.push(doc::url_segment(name));
    parts.join("/")
}

/// What a ZIP export may hold; the defaults are the limits above (tests use smaller ones).
#[derive(Clone, Copy, Debug)]
pub struct ZipLimits {
    pub max_bytes: u64,
    pub max_entries: usize,
}

impl Default for ZipLimits {
    fn default() -> ZipLimits {
        ZipLimits { max_bytes: MAX_ZIP_BYTES, max_entries: MAX_ZIP_ENTRIES }
    }
}

/// Builds the ZIP in memory, one page at a time (a page's pictures are read, used and dropped
/// before the next page), and writes it to `dest` in one step. `attachment` reads one picture.
pub fn write_zip(dest: &Path, format: Format, zdoc: ZipDoc, attachment: &dyn Fn(&str) -> Option<(String, Vec<u8>)>, limits: ZipLimits) -> Result<ExportSummary> {
    let too_big = || {
        Error::invalid(format!(
            "The export would be larger than {} MB. Choose fewer pages or export in parts.",
            limits.max_bytes / (1024 * 1024)
        ))
    };
    let too_many = || Error::invalid(format!("A ZIP can hold at most {} files. Choose fewer pages or export in parts.", limits.max_entries));
    let mut zip = zip::ZipWriter::new(crate::model::now_ms());
    let mut summary = ExportSummary { path: dest.display().to_string(), archive: true, ..ExportSummary::default() };
    let join = |dir: &[String], name: &str| -> String {
        let mut p = dir.join("/");
        if !p.is_empty() {
            p.push('/');
        }
        p.push_str(name);
        p
    };
    // The marker first (it is not counted among the files the person is told about).
    if limits.max_entries == 0 {
        return Err(too_many());
    }
    zip.add(MARKER_NAME, marker_text(zdoc.layout).as_bytes(), true)?;
    for f in zdoc.files {
        let mut ids = Vec::new();
        doc::attachments(&f.blocks, &mut ids);
        let mut images = BTreeMap::new();
        let mut page_bytes = 0u64;
        for id in ids {
            match attachment(&id).and_then(|(mime, bytes)| Kind::from_mime(&mime).map(|kind| Picture { kind, bytes })) {
                Some(p) => {
                    page_bytes += p.bytes.len() as u64;
                    if zip.byte_len() as u64 + page_bytes > limits.max_bytes {
                        return Err(too_big());
                    }
                    images.insert(id, p);
                }
                None => summary.missing_images += 1,
            }
        }
        summary.images += images.len();
        let page = ExportDoc {
            title: f.title.clone(),
            parts: vec![Part::Page { level: 1, title: f.title, blocks: f.blocks }],
            images,
            missing_images: 0,
        };
        // (path, bytes, compress)
        let mut entries: Vec<(String, Vec<u8>, bool)> = Vec::new();
        let file_name = format!("{}.{}", f.stem, format.ext());
        match format {
            Format::Markdown => {
                let assets = format!("{}.assets", f.stem);
                let out = markdown::render(&page, &assets);
                entries.push((join(&f.dir, &file_name), out.text.into_bytes(), true));
                for (name, bytes) in out.images {
                    entries.push((join(&f.dir, &format!("{assets}/{name}")), bytes, false));
                }
            }
            Format::Html => entries.push((join(&f.dir, &file_name), html::render(&page)?.into_bytes(), true)),
            Format::Docx => entries.push((join(&f.dir, &file_name), docx::render(&page)?, false)),
            Format::Pdf => {
                let out = pdf::render(&page);
                summary.images_left_out += out.images_left_out;
                summary.chars_replaced += out.chars_replaced;
                summary.chars_no_font += out.chars_no_font;
                entries.push((join(&f.dir, &file_name), out.bytes, false));
            }
        }
        for (path, bytes, compress) in entries {
            if zip.entries() >= limits.max_entries {
                return Err(too_many());
            }
            zip.add(&path, &bytes, compress)?;
            summary.files += 1;
            if zip.byte_len() as u64 > limits.max_bytes {
                return Err(too_big());
            }
        }
        summary.pages += 1;
    }
    let bytes = zip.finish()?;
    write_atomic(dest, &bytes)?;
    Ok(summary)
}

/* ---------- Tables (shared by the writers) ---------- */

/// One position in a table's grid of columns and rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// The top-left corner of a cell: its row and its index in that row.
    Cell(usize, usize),
    /// Covered by the cell to the left (merged across).
    Right,
    /// Covered by a cell above (merged down); the number is that cell's column span.
    Down(usize),
    /// No cell here (a row with fewer cells than the others).
    Empty,
}

/// Lays the cells out on a grid, the way the editor (and HTML) places merged cells.
pub fn table_grid(rows: &[Vec<Cell>]) -> Vec<Vec<Slot>> {
    let mut grid: Vec<Vec<Option<Slot>>> = vec![Vec::new(); rows.len()];
    let set = |grid: &mut Vec<Vec<Option<Slot>>>, r: usize, c: usize, s: Slot| {
        if grid[r].len() <= c {
            grid[r].resize(c + 1, None);
        }
        grid[r][c] = Some(s);
    };
    for (r, row) in rows.iter().enumerate() {
        let mut c = 0;
        for (i, cell) in row.iter().enumerate() {
            while grid[r].get(c).is_some_and(Option::is_some) {
                c += 1;
            }
            let (cs, rs) = (cell.colspan.max(1), cell.rowspan.max(1));
            for rr in r..(r + rs).min(rows.len()) {
                for k in 0..cs {
                    let slot = match (rr == r, k == 0) {
                        (true, true) => Slot::Cell(r, i),
                        (false, true) => Slot::Down(cs),
                        _ => Slot::Right,
                    };
                    set(&mut grid, rr, c + k, slot);
                }
            }
            c += cs;
        }
    }
    let width = grid.iter().map(Vec::len).max().unwrap_or(0).min(64);
    grid.into_iter().map(|row| (0..width).map(|c| row.get(c).copied().flatten().unwrap_or(Slot::Empty)).collect()).collect()
}

/* ---------- File names and writing ---------- */

/// A name that Windows accepts as a file name (without extension).
pub fn safe_file_name(title: &str) -> String {
    let mut s: String = title
        .chars()
        .map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    s = s.trim_matches(|c: char| c == '.' || c == ' ').chars().take(80).collect::<String>().trim_end_matches(['.', ' ']).to_string();
    if s.is_empty() {
        return "Notaya export".to_string();
    }
    // Windows treats "NUL.txt" like "NUL": the base name before the first dot decides.
    let stem = s.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    const RESERVED: [&str; 26] = [
        "CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$", "COM0", "LPT0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
        "LPT9",
    ];
    if RESERVED.contains(&stem.as_str()) {
        let at = s.find('.').unwrap_or(s.len());
        s.insert(at, '_');
    }
    s
}

/// The path with symbolic links and `..` resolved; a file that does not exist yet is
/// resolved through its folder.
pub(crate) fn resolve(p: &Path) -> PathBuf {
    fs::canonicalize(p)
        .or_else(|_| {
            let dir = p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
            let name = p.file_name().ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
            fs::canonicalize(dir).map(|d| d.join(name))
        })
        .unwrap_or_else(|_| p.to_path_buf())
}

/// Checks the place the person chose. The file gets the extension `ext` (the format's, or `zip`; added when the
/// name has another one; an existing file with the longer name is then refused, because the
/// save dialog asked about a different file). Nothing inside the app's data folder is allowed,
/// so an export can never replace the vault or a backup.
pub fn checked_destination(chosen: &Path, ext: &str, data_dir: &Path) -> Result<PathBuf> {
    let has_ext = chosen.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(ext));
    let dest = if has_ext {
        chosen.to_path_buf()
    } else {
        let mut name = chosen.file_name().ok_or_else(|| Error::invalid("Choose a file name for the export."))?.to_os_string();
        name.push(".");
        name.push(ext);
        let longer = chosen.with_file_name(name);
        if longer.exists() {
            return Err(Error::invalid(format!(
                "A file named {} already exists. Choose another name.",
                longer.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
            )));
        }
        longer
    };
    checked_destination_path(&dest, data_dir)?;
    Ok(dest)
}

/// Validate a final path independently of extension handling, including extensionless row files.
pub(crate) fn checked_destination_path(dest: &Path, data_dir: &Path) -> Result<()> {
    let data = resolve(data_dir);
    let target = resolve(dest);
    if target.starts_with(&data) || crate::filenames::path_has_vault_ext(dest) {
        return Err(Error::invalid(
            "That is the folder where Notaya keeps your notes and backups. Choose another folder for the export.",
        ));
    }
    if target.is_dir() {
        return Err(Error::invalid("That name is a folder. Choose a file name for the export."));
    }
    Ok(())
}

fn is_export_image_name(file: &str) -> bool {
    let Some((stem, ext)) = file.rsplit_once('.') else { return false };
    stem.strip_prefix("image-").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        && ["png", "jpg", "jpeg", "gif", "webp"].contains(&ext.to_ascii_lowercase().as_str())
}

/// Writes `bytes` to a temporary file next to `dest`, flushes it to the disk and renames it
/// into place, so a failed export never leaves a half-written file under the chosen name.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<()> {
    transaction::write_atomic(dest, bytes)
}

/// Renders the document in the chosen format and writes it to `dest` (already checked with
/// [`checked_destination`]). For Markdown the pictures go into `<name>.assets` next to it.
pub fn write(dest: &Path, format: Format, doc: &ExportDoc, data_dir: &Path) -> Result<ExportSummary> {
    let mut summary = ExportSummary {
        path: dest.display().to_string(),
        pages: doc.page_count(),
        images: doc.images.len(),
        missing_images: doc.missing_images,
        ..ExportSummary::default()
    };
    match format {
        Format::Markdown => {
            let (folder, warnings) = transaction::write_markdown(dest, doc, data_dir)?;
            summary.assets_folder = folder;
            summary.cleanup_warnings = warnings;
        }
        Format::Html => write_atomic(dest, html::render(doc)?.as_bytes())?,
        Format::Docx => write_atomic(dest, &docx::render(doc)?)?,
        Format::Pdf => {
            let out = pdf::render(doc);
            summary.images_left_out = out.images_left_out;
            summary.chars_replaced = out.chars_replaced;
            summary.chars_no_font = out.chars_no_font;
            write_atomic(dest, &out.bytes)?;
        }
    }
    Ok(summary)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::database::types::{Column, Cond, Database, Property, Row, Sort};
    use crate::model::{Notebook, Section};

    /// A real databaseBlock and saved view, with Arabic and Hebrew text, computed values,
    /// a hidden property and a trashed row. No vault or disk access.
    pub fn inline_database_items() -> Vec<Item> {
        let mut items = sample_items();
        let Item::Page(p) = items.iter_mut().find(|i| matches!(i, Item::Page(p) if p.id == "a")).unwrap() else { unreachable!() };
        p.content = serde_json::json!({"type":"doc", "content":[
            {"type":"paragraph", "content":[{"type":"text", "text":"Before"}]},
            {"type":"databaseBlock", "attrs":{"databaseId":"database1", "viewId":"saved"}},
            {"type":"paragraph", "content":[{"type":"text", "text":"After"}]}
        ]}).to_string();
        let mut db = Database::new("database1".into(), "s1".into(), "مهام".into(), 0);
        db.props[0].name = "الاسم".into();
        for (id, name, kind) in [("text", "الوصف", PropKind::Text), ("amount", "Amount", PropKind::Number), ("double", "Double", PropKind::Formula), ("hidden", "Hidden", PropKind::Text)] {
            db.props.push(Property { id: id.into(), name: name.into(), kind, ..Property::default() });
        }
        db.props.iter_mut().find(|p| p.id == "double").unwrap().formula = "prop(\"Amount\") * 2".into();
        db.views[0].id = "saved".into();
        db.views[0].columns = ["title", "text", "amount", "double", "hidden"].iter().map(|id| Column { prop: (*id).into(), visible: *id != "hidden", width: 0 }).collect();
        for (n, title, text) in [(0, "أول", "مرحبا 123 عالم"), (1, "ثان", "שלום"), (2, "محذوف", "TRASHED_CELL")] {
            let mut row = Row::new(format!("row{n}"), &db.id, title.into(), n);
            row.order = n;
            row.cells = serde_json::json!({"text":text, "amount":12.5 * (n + 1) as f64, "hidden":"HIDDEN_CELL"}).to_string();
            row.trashed_at = (n == 2).then_some(5);
            items.push(Item::Row(row));
        }
        items.push(Item::Database(db));
        items
    }

    pub fn inline_database_doc() -> ExportDoc {
        collect_inline(&inline_database_items())
    }

    fn collect_inline(items: &[Item]) -> ExportDoc {
        collect(&items.iter().collect::<Vec<_>>(), &Scope::Pages(vec!["a".into()]), &Options::default(), &|_| None).unwrap()
    }

    fn inline_table(d: &ExportDoc) -> &doc::DatabaseTable {
        let Part::Page { blocks, .. } = &d.parts[0] else { panic!("page") };
        let Block::Database { table: Some(t), .. } = &blocks[1] else { panic!("resolved database") };
        t
    }

    fn cell_string(c: &Cell) -> String {
        let Block::Paragraph { content, .. } = &c.blocks[0] else { panic!("cell text") };
        doc::inline_text(content)
    }

    #[test]
    fn embedded_export_formats_exact_currency_and_surfaces_computed_errors() {
        let mut items = inline_database_items();
        let Item::Database(db) = items.last_mut().unwrap() else { panic!("database") };
        db.props.iter_mut().find(|p| p.id == "amount").unwrap().number_format = "dollar".into();
        db.props.iter_mut().find(|p| p.id == "double").unwrap().formula = "ln(-1e-400)".into();
        db.props.extend([
            Property { id: "link".into(), name: "Link".into(), kind: PropKind::Relation, relation_db: db.id.clone(), ..Default::default() },
            Property { id: "rollup".into(), name: "Rollup".into(), kind: PropKind::Rollup, rollup_relation: "link".into(), rollup_prop: "double".into(), rollup_calc: "sum".into(), ..Default::default() },
        ]);
        db.views[0].columns.push(Column { prop: "rollup".into(), visible: true, width: 0 });
        let row = items.iter_mut().find_map(|i| match i {
            Item::Row(r) if r.id == "row0" => Some(r),
            _ => None,
        }).unwrap();
        row.cells = r#"{"amount":999999999999.98499,"link":["row1"]}"#.into();
        let doc = collect_inline(&items);
        let cells = &inline_table(&doc).rows[1];
        assert_eq!(cell_string(&cells[2]), "$999,999,999,999.98");
        assert_eq!(cell_string(&cells[3]), "Error: Not a real number");
        assert_eq!(cell_string(&cells[4]), "Error: Not a real number");
    }

    #[test]
    fn inline_database_saved_view_filters_sorts_and_hides_columns() {
        let mut items = inline_database_items();
        let Item::Database(db) = items.last_mut().unwrap() else { panic!() };
        db.views[0].sorts = vec![Sort { prop: "amount".into(), descending: true }];
        db.views.push(crate::database::types::View { id: "other".into(), ..Default::default() });
        let d = collect_inline(&items);
        let t = inline_table(&d);
        assert_eq!(t.rows.len(), 3, "header and two live rows");
        assert_eq!(t.rows[0].iter().map(cell_string).collect::<Vec<_>>(), ["الاسم", "الوصف", "Amount", "Double"]);
        assert_eq!(t.rows[1].iter().map(cell_string).collect::<Vec<_>>(), ["ثان", "שלום", "25", "50"]);
        assert_eq!(t.rows[2].iter().map(cell_string).collect::<Vec<_>>(), ["أول", "مرحبا 123 عالم", "12.5", "25"]);
        let Item::Database(db) = items.last_mut().unwrap() else { panic!() };
        db.views[0].filter.conds = vec![Cond { prop: "title".into(), op: "contains".into(), value: "أول".into(), ..Default::default() }];
        let d = collect_inline(&items);
        let t = inline_table(&d);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(cell_string(&t.rows[1][0]), "أول");
        assert_eq!(t.more, 0);
        assert!(!markdown::render(&d, "assets").text.contains("ثان"));
    }

    #[test]
    fn inline_database_caps_after_filtering_and_sorting_and_reports_exact_remainder() {
        let mut items = inline_database_items();
        items.retain(|i| !matches!(i, Item::Row(_)));
        let Item::Database(db) = items.last_mut().unwrap() else { panic!() };
        db.props.truncate(1);
        db.views[0].columns.truncate(1);
        db.views[0].sorts = vec![Sort { prop: "title".into(), descending: true }];
        db.views[0].filter.conds = vec![Cond { prop: "title".into(), op: "contains".into(), value: "Keep".into(), ..Default::default() }];
        // Cross the screen engine's 50,000-row limit as well as the export's 1,000-row limit.
        for n in 0..50_003 {
            items.push(Item::Row(Row::new(format!("cap{n}"), "database1", format!("Keep {n:05}"), n)));
        }
        items.push(Item::Row(Row::new("filtered".into(), "database1", "Excluded".into(), 0)));
        let d = collect_inline(&items);
        let t = inline_table(&d);
        assert_eq!(t.rows.len(), MAX_DATABASE_ROWS + 1);
        assert_eq!(cell_string(&t.rows[1][0]), "Keep 50002");
        assert_eq!(cell_string(&t.rows[MAX_DATABASE_ROWS][0]), "Keep 49003");
        assert_eq!(t.more, 49_003);
        let md = markdown::render(&d, "assets").text;
        assert!(md.contains("and 49003 more rows"));
        assert!(!md.contains("Keep 49002"));
        assert!(!md.contains("Excluded"));
    }

    #[test]
    fn inline_database_missing_or_binned_is_unavailable_in_every_format() {
        for missing in [true, false] {
            let mut items = inline_database_items();
            if missing {
                items.pop();
            } else {
                let Item::Database(db) = items.last_mut().unwrap() else { panic!() };
                db.trashed_at = Some(5);
            }
            // Even an explicitly requested trash export cannot expose a binned database.
            let d = collect(&items.iter().collect::<Vec<_>>(), &Scope::Pages(vec!["a".into()]), &Options { include_trashed: true, ..Options::default() }, &|_| None).unwrap();
            let md = markdown::render(&d, "assets").text;
            let html = html::render(&d).unwrap();
            let word = docx::render_at(&d, 0).unwrap();
            let entries = zip::tests::read_zip(&word);
            let xml = String::from_utf8(entries.iter().find(|e| e.0 == "word/document.xml").unwrap().1.clone()).unwrap();
            let pdf = pdf::render_at(&d, 0);
            // The fixed ASCII placeholder is in the PDF's standard-font content stream.
            let pdf_text = pdf::tests::extract(&pdf.bytes);
            for text in [&md, &html, &xml, &pdf_text] {
                assert!(text.contains("(database not available)"));
                assert!(!text.contains("مرحبا"));
            }
            assert!(!md.contains("notaya://db/"));
        }
    }

    #[test]
    fn inline_database_zip_keeps_separate_pages_and_page_links() {
        let mut items = inline_database_items();
        let Item::Page(p) = items.iter_mut().find(|i| matches!(i, Item::Page(p) if p.id == "a")).unwrap() else { panic!() };
        let mut json: serde_json::Value = serde_json::from_str(&p.content).unwrap();
        json["content"].as_array_mut().unwrap().push(serde_json::json!({"type":"pageLink", "attrs":{"pageId":"b"}}));
        p.content = json.to_string();
        let refs = items.iter().collect::<Vec<_>>();
        for format in [Format::Html, Format::Markdown, Format::Pdf, Format::Docx] {
            let z = collect_zip(&refs, &Scope::Pages(vec!["a".into(), "b".into()]), &Options::default(), format).unwrap();
            assert_eq!(z.files.len(), 2);
            let db_file = z.files.iter().find(|f| f.title == "First").unwrap();
            assert!(matches!(&db_file.blocks[1], Block::Database { table: Some(t), .. } if t.rows.len() == 3));
            assert!(matches!(&db_file.blocks[3], Block::PageLink { target: Some(doc::PageRef::File(href)), .. } if href == &format!("Second.{}", format.ext())));
            let dest = temp_dir().join("inline.zip");
            let result = write_zip(&dest, format, z, &|_| None, ZipLimits::default()).unwrap();
            assert_eq!(result.pages, 2);
            let entries = zip::tests::read_zip(&fs::read(&dest).unwrap());
            assert_eq!(entries.len(), 3, "two page files and the fixed marker");
            let bytes = &entries.iter().find(|e| e.0 == format!("First.{}", format.ext())).unwrap().1;
            let text = match format {
                Format::Html | Format::Markdown => String::from_utf8(bytes.clone()).unwrap(),
                Format::Pdf => pdf::tests::extract(bytes),
                Format::Docx => String::from_utf8(zip::tests::read_zip(bytes).into_iter().find(|e| e.0 == "word/document.xml").unwrap().1).unwrap(),
            };
            assert!(text.contains("مرحبا 123 عالم"));
            if matches!(format, Format::Html | Format::Markdown) {
                assert!(text.contains(&format!("Second.{}", format.ext())));
            }
        }
    }

    #[test]
    fn inline_database_view_fallback_empty_rows_and_exact_cap() {
        let mut items = inline_database_items();
        items.retain(|i| !matches!(i, Item::Row(_)));
        let Item::Page(p) = items.iter_mut().find(|i| matches!(i, Item::Page(p) if p.id == "a")).unwrap() else { panic!() };
        p.content = p.content.replace("saved", "removed-view");
        assert_eq!(inline_table(&collect_inline(&items)).rows.len(), 1, "an empty view still has its header");
        for n in 0..MAX_DATABASE_ROWS {
            items.push(Item::Row(Row::new(format!("exact{n}"), "database1", format!("Row {n}"), 0)));
        }
        let d = collect_inline(&items);
        let t = inline_table(&d);
        assert_eq!(t.rows.len(), MAX_DATABASE_ROWS + 1);
        assert_eq!(t.more, 0);
        assert!(!markdown::render(&d, "assets").text.contains("more rows"));
    }

    #[test]
    fn inline_databases_resolve_inside_every_container() {
        use doc::{ListItem, ListKind};
        let db = || Block::Database { id: "database1".into(), view: "saved".into(), table: None };
        let mut blocks = vec![
            Block::Quote(vec![db()]), Block::Toggle(vec![db()]), Block::Shelf(vec![db()]),
            Block::Callout { icon: String::new(), blocks: vec![db()] },
            Block::Columns(vec![vec![db()]]),
            Block::List { kind: ListKind::Bullet, items: vec![ListItem { checked: None, blocks: vec![db()] }] },
            Block::Table(vec![vec![Cell { header: false, colspan: 1, rowspan: 1, widths: vec![], blocks: vec![db()] }]]),
        ];
        let items = inline_database_items();
        let ctx = Ctx::new(items.iter(), 0, 0);
        resolve_databases(&mut blocks, &ctx);
        let md = markdown::render(&sample_doc(blocks), "assets").text;
        assert_eq!(md.matches("notaya://db/database1").count(), 7);
        assert_eq!(md.matches("مرحبا 123 عالم").count(), 7);
        assert!(!md.contains("database not available"));
    }

    /// A one-page document around the given blocks, with the test picture.
    pub fn sample_doc(blocks: Vec<Block>) -> ExportDoc {
        let mut images = BTreeMap::new();
        images.insert(doc::tests::IMG_ID.to_string(), Picture { kind: Kind::Png, bytes: image::tests::PILLOW_RGBA.to_vec() });
        ExportDoc { title: "Sample page".into(), parts: vec![Part::Page { level: 1, title: "Sample page".into(), blocks }], images, missing_images: 0 }
    }

    fn page(id: &str, section: &str, parent: &str, title: &str, order: i64, trashed: bool) -> Item {
        Item::Page(Page {
            source: None,
            id: id.into(),
            section_id: section.into(),
            title: title.into(),
            content: format!(r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"Body of {title}"}}]}}]}}"#),
            text: String::new(),
            order,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: parent.into(),
            trashed_at: trashed.then_some(5),
            created: order,
            updated: order,
        })
    }

    /// Notebook "nb" with sections "s1" (pages a, a/child, b, trashed t) and "s2" (page c),
    /// and a second notebook with page "z".
    fn sample_items() -> Vec<Item> {
        let nb = |id: &str, name: &str, order| Item::Notebook(Notebook { source: None, id: id.into(), name: name.into(), color: String::new(), icon: String::new(), order, created: 0, updated: 0, trashed_at: None, });
        let sec = |id: &str, nbid: &str, name: &str, order| {
            Item::Section(Section { source: None, id: id.into(), notebook_id: nbid.into(), name: name.into(), color: String::new(), order, created: 0, updated: 0, trashed_at: None, })
        };
        vec![
            nb("nb", "Personal", 0),
            nb("nb2", "Work", 1),
            sec("s2", "nb", "Ideas", 1),
            sec("s1", "nb", "Journal", 0),
            sec("s3", "nb2", "Projects", 0),
            page("b", "s1", "", "Second", 2, false),
            page("a", "s1", "", "First", 1, false),
            page("child", "s1", "a", "Inside first", 0, false),
            page("t", "s1", "", "Thrown away", 3, true),
            page("tc", "s1", "child", "Trashed child", 1, true),
            page("c", "s2", "", "Idea", 0, false),
            page("z", "s3", "", "Other notebook", 0, false),
        ]
    }

    fn outline(d: &ExportDoc) -> Vec<String> {
        d.parts
            .iter()
            .map(|p| match p {
                Part::Heading { level, text } => format!("H{level} {text}"),
                Part::Page { level, title, .. } => format!("P{level} {title}"),
            })
            .collect()
    }

    fn run(scope: Scope, subpages: bool, trashed: bool) -> Result<ExportDoc> {
        let items = sample_items();
        let refs: Vec<&Item> = items.iter().collect();
        collect(&refs, &scope, &Options { include_subpages: subpages, include_trashed: trashed, ..Options::default() }, &|_| None)
    }

    #[test]
    fn a_notebook_export_has_its_sections_and_pages_in_sidebar_order() {
        let d = run(Scope::Notebook("nb".into()), true, false).unwrap();
        assert_eq!(outline(&d), ["H1 Personal", "H2 Journal", "P3 First", "P4 Inside first", "P3 Second", "H2 Ideas", "P3 Idea"]);
        assert_eq!(d.title, "Personal");
        let d = run(Scope::Notebook("nb".into()), false, false).unwrap();
        assert_eq!(outline(&d), ["H1 Personal", "H2 Journal", "P3 First", "P3 Second", "H2 Ideas", "P3 Idea"]);
    }

    #[test]
    fn trashed_pages_are_left_out_unless_asked_for() {
        for scope in [Scope::Notebook("nb".into()), Scope::Section("s1".into()), Scope::Pages(vec!["a".into(), "t".into(), "tc".into()])] {
            let d = run(scope.clone(), true, false).unwrap();
            let all = outline(&d).join(",");
            assert!(!all.contains("Thrown away") && !all.contains("Trashed child"), "{scope:?}: {all}");
            for part in &d.parts {
                if let Part::Page { blocks, .. } = part {
                    assert!(!format!("{blocks:?}").contains("Thrown away"));
                }
            }
        }
        // Only trashed pages chosen: nothing to export.
        assert!(run(Scope::Pages(vec!["t".into()]), true, false).is_err());
        let d = run(Scope::Section("s1".into()), true, true).unwrap();
        assert_eq!(outline(&d), ["H1 Journal", "P2 First", "P3 Inside first", "P4 Trashed child", "P2 Second", "P2 Thrown away"]);
    }

    #[test]
    fn chosen_pages_follow_the_sidebar_and_appear_once() {
        let d = run(Scope::Pages(vec!["z".into(), "c".into(), "child".into(), "a".into()]), true, false).unwrap();
        assert_eq!(outline(&d), ["P1 First", "P2 Inside first", "P1 Idea", "P1 Other notebook"]);
        assert_eq!(d.title, "4 pages");
        let d = run(Scope::Pages(vec!["a".into()]), false, false).unwrap();
        assert_eq!(outline(&d), ["P1 First"]);
        assert_eq!(d.title, "First");
        assert!(matches!(run(Scope::Notebook("nope".into()), true, false), Err(Error::NotFound)));
    }

    #[test]
    fn grid_places_merged_cells() {
        let c = |cs, rs| Cell { header: false, colspan: cs, rowspan: rs, widths: vec![], blocks: vec![] };
        let rows = vec![vec![c(2, 1), c(1, 2)], vec![c(1, 1), c(1, 1)], vec![c(1, 1)]];
        let g = table_grid(&rows);
        assert_eq!(g[0], [Slot::Cell(0, 0), Slot::Right, Slot::Cell(0, 1)]);
        assert_eq!(g[1], [Slot::Cell(1, 0), Slot::Cell(1, 1), Slot::Down(1)]);
        assert_eq!(g[2], [Slot::Cell(2, 0), Slot::Empty, Slot::Empty]);
    }

    #[test]
    fn file_names_are_safe_on_windows() {
        assert_eq!(safe_file_name("My: notes / 2026?"), "My notes 2026");
        assert_eq!(safe_file_name("  ..  "), "Notaya export");
        assert_eq!(safe_file_name("con"), "con_");
        assert_eq!(safe_file_name("NUL.txt"), "NUL_.txt", "the base name before the first dot is what Windows reads");
        assert_eq!(safe_file_name("Com1 .x"), "Com1 _.x");
        assert_eq!(safe_file_name("Trip. "), "Trip");
        assert_eq!(safe_file_name(&"x".repeat(200)).len(), 80);
    }

    fn temp_dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("notaya-export-test-{}", crate::model::new_id()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_data_folder_and_vault_files_are_refused_as_destinations() {
        let root = temp_dir();
        let data = root.join("data");
        fs::create_dir_all(data.join("backups")).unwrap();
        fs::write(data.join("vault.nota"), b"vault").unwrap();
        for bad in [data.join("vault.nota"), data.join("notes.pdf"), data.join("backups").join("x.md"), data.join("sub").join("..").join("y.docx")] {
            assert!(checked_destination(&bad, "pdf", &data).is_err(), "{bad:?}");
        }
        assert_eq!(fs::read(data.join("vault.nota")).unwrap(), b"vault");
        let ok = checked_destination(&root.join("notes.PDF"), "pdf", &data).unwrap();
        assert_eq!(ok, root.join("notes.PDF"));
        assert_eq!(checked_destination(&root.join("notes"), "docx", &data).unwrap(), root.join("notes.docx"));
        assert_eq!(checked_destination(&root.join("notes.txt"), "md", &data).unwrap(), root.join("notes.txt.md"));
        fs::write(root.join("old.txt.md"), b"keep").unwrap();
        assert!(checked_destination(&root.join("old.txt"), "md", &data).is_err(), "never overwrites a file the dialog did not ask about");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_markdown_pictures_folder_is_checked_before_anything_is_written() {
        let root = temp_dir();
        let data = root.join("data");
        fs::create_dir_all(&data).unwrap();
        let doc = sample_doc(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        // A folder of that name that holds other files is refused and left alone.
        let other = root.join("other");
        fs::create_dir_all(other.join("notes.assets")).unwrap();
        fs::write(other.join("notes.assets").join("mine.txt"), b"mine").unwrap();
        assert!(write(&other.join("notes.md"), Format::Markdown, &doc, &data).is_err());
        assert_eq!(fs::read(other.join("notes.assets").join("mine.txt")).unwrap(), b"mine");
        assert!(!other.join("notes.md").exists());
        // A folder of that name inside the data folder is refused.
        assert!(write(&data.join("notes.md"), Format::Markdown, &doc, &data).is_err());
        assert!(!data.join("notes.assets").exists());
        // Legacy picture names do not establish ownership: preserve them and say so.
        let again = root.join("again");
        fs::create_dir_all(again.join("notes.assets")).unwrap();
        fs::write(again.join("notes.assets").join("image-099.png"), b"old").unwrap();
        let summary = write(&again.join("notes.md"), Format::Markdown, &doc, &data).unwrap();
        assert_eq!(fs::read(again.join("notes.assets").join("image-099.png")).unwrap(), b"old");
        assert!(!summary.cleanup_warnings.is_empty());
        assert!(PathBuf::from(summary.assets_folder.unwrap()).join("image-001.png").is_file());
        // A link that leads somewhere else is refused.
        #[cfg(unix)]
        {
            let linked = root.join("linked");
            fs::create_dir_all(&linked).unwrap();
            std::os::unix::fs::symlink(&data, linked.join("notes.assets")).unwrap();
            assert!(write(&linked.join("notes.md"), Format::Markdown, &doc, &data).is_err());
            assert_eq!(fs::read_dir(&data).unwrap().count(), 0);
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn writing_replaces_the_file_and_leaves_no_temp_files() {
        let root = temp_dir();
        let doc = sample_doc(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        for format in [Format::Markdown, Format::Pdf, Format::Docx, Format::Html] {
            let dest = root.join(format!("My notes.{}", format.ext()));
            fs::write(&dest, b"old").unwrap();
            let s = write(&dest, format, &doc, &root.join("data")).unwrap();
            assert_eq!(s.pages, 1);
            assert_ne!(fs::read(&dest).unwrap(), b"old");
            if format == Format::Markdown {
                assert!(PathBuf::from(s.assets_folder.as_deref().unwrap()).join("image-001.png").is_file());
            }
        }
        let leftovers: Vec<_> = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with(".tmp")).collect();
        assert!(leftovers.is_empty());
        // Sample files for a look with real tools, when asked for (see the export record).
        if let Some(dir) = std::env::var_os("NOTAYA_EXPORT_SAMPLES") {
            let dir = PathBuf::from(dir);
            fs::create_dir_all(&dir).unwrap();
            for format in [Format::Markdown, Format::Pdf, Format::Docx, Format::Html] {
                write(&dir.join(format!("sample.{}", format.ext())), format, &doc, &dir.join("data")).unwrap();
            }
            let items = sample_items();
            let refs: Vec<&Item> = items.iter().collect();
            let nb = collect(&refs, &Scope::Notebook("nb".into()), &Options { include_subpages: true, ..Options::default() }, &|_| None).unwrap();
            for format in [Format::Pdf, Format::Docx] {
                write(&dir.join(format!("notebook.{}", format.ext())), format, &nb, &dir.join("data")).unwrap();
            }
        }
        let _ = fs::remove_dir_all(root);
    }

    /* ---------- Several sections and notebooks, and separate files in a ZIP ---------- */

    use super::zip::tests::read_zip;

    fn page_with(id: &str, section: &str, parent: &str, title: &str, order: i64, content: String) -> Item {
        Item::Page(Page {
            source: None,
            id: id.into(),
            section_id: section.into(),
            title: title.into(),
            content,
            text: String::new(),
            order,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: parent.into(),
            trashed_at: None,
            created: order,
            updated: order,
        })
    }

    fn with_picture() -> String {
        format!(
            r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"Look"}}]}},{{"type":"image","attrs":{{"src":"http://clatt.localhost/{}","caption":"Pic"}}}}]}}"#,
            doc::tests::IMG_ID
        )
    }

    fn vault_picture(id: &str) -> Option<(String, Vec<u8>)> {
        (id == doc::tests::IMG_ID).then(|| ("image/png".to_string(), image::tests::PILLOW_RGBA.to_vec()))
    }

    fn zip_of(items: &[Item], scope: Scope, format: Format, subpages: bool, trashed: bool) -> Result<(Vec<(String, Vec<u8>)>, ExportSummary)> {
        let refs: Vec<&Item> = items.iter().collect();
        let opts = Options { include_subpages: subpages, include_trashed: trashed, ..Options::default() };
        let zdoc = collect_zip(&refs, &scope, &opts, format)?;
        let root = temp_dir();
        let dest = root.join("out.zip");
        let summary = write_zip(&dest, format, zdoc, &vault_picture, ZipLimits::default())?;
        let mut entries = read_zip(&fs::read(&dest).unwrap());
        let _ = fs::remove_dir_all(root);
        // Every ZIP starts with its marker; the tests look at the pages.
        assert_eq!(entries.first().map(|(n, _)| n.as_str()), Some(MARKER_NAME));
        entries.remove(0);
        Ok((entries, summary))
    }

    fn names(entries: &[(String, Vec<u8>)]) -> Vec<&str> {
        entries.iter().map(|(n, _)| n.as_str()).collect()
    }

    fn text<'a>(entries: &'a [(String, Vec<u8>)], name: &str) -> &'a str {
        std::str::from_utf8(&entries.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("no {name} in {:?}", names(entries))).1).unwrap()
    }

    #[test]
    fn several_notebooks_and_sections_are_joined_in_sidebar_order() {
        let d = run(Scope::Notebooks(vec!["nb2".into(), "nb".into()]), true, false).unwrap();
        assert_eq!(
            outline(&d),
            ["H1 Personal", "H2 Journal", "P3 First", "P4 Inside first", "P3 Second", "H2 Ideas", "P3 Idea", "H1 Work", "H2 Projects", "P3 Other notebook"]
        );
        assert_eq!(d.title, "2 notebooks");
        let d = run(Scope::Sections(vec!["s3".into(), "s2".into(), "s1".into()]), false, false).unwrap();
        assert_eq!(outline(&d), ["H1 Journal", "P2 First", "P2 Second", "H1 Ideas", "P2 Idea", "H1 Projects", "P2 Other notebook"]);
        assert_eq!(d.title, "3 sections");
        let d = run(Scope::Sections(vec!["s2".into()]), true, false).unwrap();
        assert_eq!(outline(&d), ["H1 Ideas", "P2 Idea"]);
        assert_eq!(d.title, "Ideas");
        // A single section or notebook still gives what it always did.
        assert_eq!(outline(&run(Scope::Section("s2".into()), true, false).unwrap()), ["H1 Ideas", "P2 Idea"]);
    }

    #[test]
    fn unknown_or_empty_choices_are_refused() {
        for scope in [
            Scope::Notebooks(vec!["nb".into(), "nope".into()]),
            Scope::Sections(vec!["nope".into()]),
            Scope::Sections(vec!["nb".into()]), // a notebook id is not a section id
            Scope::Notebooks(vec!["s1".into()]),
        ] {
            assert!(matches!(run(scope.clone(), true, false), Err(Error::NotFound)), "{scope:?}");
        }
        assert!(run(Scope::Notebooks(vec![]), true, false).is_err());
        assert!(run(Scope::Sections((0..=MAX_CHOSEN).map(|i| i.to_string()).collect()), true, false).is_err(), "counts are capped");
        // A repeated id counts once.
        let d = run(Scope::Notebooks(vec!["nb2".into(), "nb2".into()]), true, false).unwrap();
        assert_eq!(outline(&d), ["H1 Work", "H2 Projects", "P3 Other notebook"]);
        let items = sample_items();
        let refs: Vec<&Item> = items.iter().collect();
        assert_eq!(suggested_name(&refs, &Scope::Notebooks(vec!["nb".into(), "nb2".into()])).unwrap(), "Personal and 1 more");
        assert_eq!(suggested_name(&refs, &Scope::Sections(vec!["s2".into()])).unwrap(), "Ideas");
        assert!(suggested_name(&refs, &Scope::Sections(vec!["zzz".into()])).is_err());
    }

    #[test]
    fn a_zip_has_notebook_section_page_folders_and_sub_pages_in_a_folder_of_their_page() {
        let items = sample_items();
        let (entries, summary) = zip_of(&items, Scope::Notebook("nb".into()), Format::Markdown, true, false).unwrap();
        assert_eq!(names(&entries), ["Personal/Journal/First.md", "Personal/Journal/First/Inside first.md", "Personal/Journal/Second.md", "Personal/Ideas/Idea.md"]);
        assert_eq!(summary.pages, 4);
        assert_eq!(summary.files, 4);
        assert!(summary.archive);
        assert!(text(&entries, "Personal/Journal/First.md").contains("# First") && text(&entries, "Personal/Journal/First.md").contains("Body of First"));
        assert!(!text(&entries, "Personal/Journal/First.md").contains("Inside first"), "each page is its own file");
        // Without sub-pages the folder is not made.
        let (entries, _) = zip_of(&items, Scope::Notebook("nb".into()), Format::Html, false, false).unwrap();
        assert_eq!(names(&entries), ["Personal/Journal/First.html", "Personal/Journal/Second.html", "Personal/Ideas/Idea.html"]);
        // Trashed pages are left out, and come back when asked for.
        let (entries, _) = zip_of(&items, Scope::Section("s1".into()), Format::Markdown, true, true).unwrap();
        assert_eq!(
            names(&entries),
            ["Journal/First.md", "Journal/First/Inside first.md", "Journal/First/Inside first/Trashed child.md", "Journal/Second.md", "Journal/Thrown away.md"]
        );
        let (entries, _) = zip_of(&items, Scope::Section("s1".into()), Format::Markdown, true, false).unwrap();
        assert!(names(&entries).iter().all(|n| !n.contains("Thrown away") && !n.contains("Trashed")));
        assert!(zip_of(&items, Scope::Pages(vec!["t".into()]), Format::Markdown, true, false).is_err());
    }

    #[test]
    fn a_zip_of_several_notebooks_has_one_top_folder_each() {
        let items = sample_items();
        let (entries, summary) = zip_of(&items, Scope::Notebooks(vec!["nb2".into(), "nb".into()]), Format::Pdf, true, false).unwrap();
        assert_eq!(
            names(&entries),
            [
                "Personal/Journal/First.pdf",
                "Personal/Journal/First/Inside first.pdf",
                "Personal/Journal/Second.pdf",
                "Personal/Ideas/Idea.pdf",
                "Work/Projects/Other notebook.pdf"
            ]
        );
        assert_eq!(summary.pages, 5);
        for (n, bytes) in &entries {
            assert!(bytes.starts_with(b"%PDF-"), "{n}");
        }
        let (entries, _) = zip_of(&items, Scope::Sections(vec!["s3".into(), "s2".into()]), Format::Docx, true, false).unwrap();
        assert_eq!(names(&entries), ["Ideas/Idea.docx", "Projects/Other notebook.docx"]);
        for (n, bytes) in &entries {
            assert!(bytes.starts_with(b"PK"), "{n}");
        }
    }

    #[test]
    fn chosen_pages_go_into_section_folders_only_when_they_come_from_several_sections() {
        let items = sample_items();
        let (entries, _) = zip_of(&items, Scope::Pages(vec!["b".into(), "a".into()]), Format::Markdown, false, false).unwrap();
        assert_eq!(names(&entries), ["First.md", "Second.md"]);
        let (entries, _) = zip_of(&items, Scope::Pages(vec!["z".into(), "c".into(), "child".into(), "a".into()]), Format::Markdown, true, false).unwrap();
        assert_eq!(names(&entries), ["Journal/First.md", "Journal/First/Inside first.md", "Ideas/Idea.md", "Projects/Other notebook.md"]);
        // A sub-page chosen without its page stands alone; with its page it goes in the page's folder.
        let (entries, _) = zip_of(&items, Scope::Pages(vec!["child".into()]), Format::Markdown, false, false).unwrap();
        assert_eq!(names(&entries), ["Inside first.md"]);
        let (entries, _) = zip_of(&items, Scope::Pages(vec!["child".into(), "a".into()]), Format::Markdown, false, false).unwrap();
        assert_eq!(names(&entries), ["First.md", "First/Inside first.md"]);
    }

    /// Every path of a ZIP is a plain relative path with safe parts, and no two paths differ only by capitals.
    fn assert_safe_paths(entries: &[(String, Vec<u8>)]) {
        let mut seen = HashSet::new();
        for (path, _) in entries {
            assert!(seen.insert(path.to_lowercase()), "{path} repeats (ignoring capitals)");
            assert!(crate::importers::zip::clean_name(path).as_deref() == Some(path.as_str()), "{path} is not a clean relative path");
            assert!(path.len() < 600);
            for part in path.split('/') {
                assert!(!part.is_empty() && part != "." && part != "..", "{path}");
                assert!(!part.contains(['\\', ':', '*', '?', '"', '<', '>', '|']) && !part.chars().any(char::is_control), "{path}");
                assert!(!part.ends_with(['.', ' ']) && !part.starts_with(' '), "{path}");
                let stem = part.split('.').next().unwrap().to_ascii_uppercase();
                assert!(!["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"].contains(&stem.as_str()), "{path} is a reserved Windows name");
                assert!(part.chars().count() <= 90, "{path}");
            }
        }
    }

    #[test]
    fn names_are_safe_and_unique() {
        let nb = Item::Notebook(Notebook { source: None, id: "n".into(), name: "A/B:C".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let sec = Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "..".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let body = || r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x"}]}]}"#.to_string();
        let titles = [
            "CON", "con", "NUL.txt", "..", "...", "../../etc/passwd", "a\\b", "x".repeat(200).as_str(), "", "   ", "Dup", "dup", "DUP", "Dup ", "Page", "Page.assets", "Trailing dot.", "Question? *", "ÄÖÜ", "äöü",
            "عربي",
        ]
        .map(String::from);
        let mut items = vec![nb, sec];
        for (i, t) in titles.iter().enumerate() {
            items.push(page_with(&format!("p{i}"), "s", "", t, i as i64, body()));
        }
        for format in [Format::Markdown, Format::Html] {
            let (entries, summary) = zip_of(&items, Scope::Notebook("n".into()), format, true, false).unwrap();
            assert_eq!(summary.pages, titles.len());
            assert_safe_paths(&entries);
            let all = names(&entries);
            assert!(all.iter().all(|n| n.starts_with("A B C/Section/")), "{all:?}");
            let ext = format.ext();
            for expected in [
                format!("A B C/Section/CON_.{ext}"),
                format!("A B C/Section/con_ (2).{ext}"),
                format!("A B C/Section/Dup.{ext}"),
                format!("A B C/Section/dup (2).{ext}"),
                format!("A B C/Section/DUP (3).{ext}"),
                format!("A B C/Section/Dup (4).{ext}"),
                format!("A B C/Section/Untitled.{ext}"),
                format!("A B C/Section/Untitled (2).{ext}"),
                format!("A B C/Section/ÄÖÜ.{ext}"),
                format!("A B C/Section/äöü (2).{ext}"),
            ] {
                assert!(all.contains(&expected.as_str()), "{expected} missing from {all:?}");
            }
            // "Page" keeps its pictures folder name for itself in Markdown only.
            let page_assets = format!("A B C/Section/Page.assets.{ext}");
            let page_assets_2 = format!("A B C/Section/Page.assets (2).{ext}");
            assert_eq!(all.contains(&page_assets_2.as_str()), format == Format::Markdown, "{all:?}");
            assert_eq!(all.contains(&page_assets.as_str()), format == Format::Html);
            // Long names are cut.
            assert!(all.iter().any(|n| n.ends_with(&format!("{}.{ext}", "x".repeat(80)))));
        }
    }

    #[test]
    fn a_deep_chain_of_sub_pages_stops_nesting() {
        let nb = Item::Notebook(Notebook { source: None, id: "n".into(), name: "N".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let sec = Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "S".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let mut items = vec![nb, sec];
        for i in 0..9 {
            let parent = if i == 0 { String::new() } else { format!("p{}", i - 1) };
            items.push(page_with(&format!("p{i}"), "s", &parent, &format!("Level {i}"), i, "{}".into()));
        }
        let (entries, _) = zip_of(&items, Scope::Section("s".into()), Format::Html, true, false).unwrap();
        assert_eq!(entries.len(), 9);
        assert_safe_paths(&entries);
        let deepest = names(&entries).iter().map(|n| n.matches('/').count()).max().unwrap();
        assert_eq!(deepest, 1 + MAX_SUBPAGE_FOLDERS, "section folder plus {MAX_SUBPAGE_FOLDERS} sub-page folders");
    }

    #[test]
    fn markdown_pictures_sit_next_to_each_page_and_the_links_match() {
        let nb = Item::Notebook(Notebook { source: None, id: "n".into(), name: "Trip".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let sec = Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "Day 1".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, });
        let items = vec![
            nb,
            sec,
            page_with("a", "s", "", "Beach day", 0, with_picture()),
            page_with("b", "s", "a", "More", 1, with_picture()),
            page_with("c", "s", "", "No picture", 2, "{}".into()),
        ];
        let (entries, summary) = zip_of(&items, Scope::Notebook("n".into()), Format::Markdown, true, false).unwrap();
        assert_eq!(
            names(&entries),
            [
                "Trip/Day 1/Beach day.md",
                "Trip/Day 1/Beach day.assets/image-001.png",
                "Trip/Day 1/Beach day/More.md",
                "Trip/Day 1/Beach day/More.assets/image-001.png",
                "Trip/Day 1/No picture.md"
            ]
        );
        assert!(text(&entries, "Trip/Day 1/Beach day.md").contains("![Pic](Beach%20day.assets/image-001.png)"));
        assert!(text(&entries, "Trip/Day 1/Beach day/More.md").contains("![Pic](More.assets/image-001.png)"));
        assert_eq!(entries[1].1, image::tests::PILLOW_RGBA, "the picture bytes are the original");
        assert_eq!((summary.pages, summary.images, summary.files, summary.missing_images), (3, 2, 5, 0));
        // HTML carries its pictures inside, PDF and Word too: no separate picture files.
        let (entries, _) = zip_of(&items, Scope::Notebook("n".into()), Format::Html, true, false).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(text(&entries, "Trip/Day 1/Beach day.html").contains("src=\"data:image/png;base64,"));
    }

    #[test]
    fn pictures_are_read_one_page_at_a_time_and_missing_ones_are_counted() {
        let items = vec![
            Item::Notebook(Notebook { source: None, id: "n".into(), name: "N".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "S".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            page_with("a", "s", "", "A", 0, with_picture()),
            page_with("b", "s", "", "B", 1, with_picture()),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        let zdoc = collect_zip(&refs, &Scope::Section("s".into()), &Options::default(), Format::Html).unwrap();
        // Collecting the layout reads no pictures at all.
        let root = temp_dir();
        let calls = std::cell::RefCell::new(Vec::new());
        let summary = write_zip(
            &root.join("x.zip"),
            Format::Html,
            zdoc,
            &|id| {
                calls.borrow_mut().push(id.to_string());
                if calls.borrow().len() == 2 { None } else { vault_picture(id) }
            },
            ZipLimits::default(),
        )
        .unwrap();
        assert_eq!(calls.borrow().len(), 2, "one read for each page that shows the picture");
        assert_eq!((summary.images, summary.missing_images), (1, 1));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_zip_reads_back_with_the_importers_reader() {
        let items = sample_items();
        let refs: Vec<&Item> = items.iter().collect();
        let zdoc = collect_zip(&refs, &Scope::Notebooks(vec!["nb".into(), "nb2".into()]), &Options { include_subpages: true, ..Options::default() }, Format::Markdown).unwrap();
        let root = temp_dir();
        let dest = root.join("n.zip");
        write_zip(&dest, Format::Markdown, zdoc, &vault_picture, ZipLimits::default()).unwrap();
        let archive = crate::importers::zip::Archive::open(fs::read(&dest).unwrap()).unwrap();
        assert!(archive.rejected.is_empty(), "{:?}", archive.rejected);
        let listed: Vec<&str> = archive.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            listed,
            [
                MARKER_NAME,
                "Personal/Journal/First.md",
                "Personal/Journal/First/Inside first.md",
                "Personal/Journal/Second.md",
                "Personal/Ideas/Idea.md",
                "Work/Projects/Other notebook.md"
            ]
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_marker_says_the_layout_and_nothing_else() {
        let items = sample_items();
        for (scope, layout) in [
            (Scope::Notebooks(vec!["nb".into(), "nb2".into()]), "notebooks"),
            (Scope::Notebook("nb".into()), "notebooks"),
            (Scope::Sections(vec!["s1".into()]), "sections"),
            (Scope::Section("s1".into()), "sections"),
            (Scope::Pages(vec!["a".into()]), "pages"),
        ] {
            let refs: Vec<&Item> = items.iter().collect();
            let zdoc = collect_zip(&refs, &scope, &Options::default(), Format::Html).unwrap();
            let root = temp_dir();
            let s = write_zip(&root.join("x.zip"), Format::Html, zdoc, &vault_picture, ZipLimits::default()).unwrap();
            let entries = read_zip(&fs::read(root.join("x.zip")).unwrap());
            let marker = text(&entries, MARKER_NAME);
            assert_eq!(marker.lines().next(), Some(MARKER_FIRST_LINE));
            assert_eq!(marker.lines().nth(1), Some(format!("layout: {layout}").as_str()), "{scope:?}");
            for name in ["Personal", "Journal", "First", "Body"] {
                assert!(!marker.contains(name), "no names or content in the marker: {marker}");
            }
            assert_eq!(s.files, entries.len() - 1, "the marker is not counted as a page file");
            let _ = fs::remove_dir_all(root);
        }
        // A page or section named like the marker gets another name.
        let mut items = sample_items();
        items.push(page("m", "s2", "", "notaya-export.txt", 9, false));
        let (entries, _) = zip_of(&items, Scope::Section("s2".into()), Format::Markdown, true, false).unwrap();
        assert!(names(&entries).contains(&"Ideas/notaya-export.txt.md"));
        let (entries, _) = zip_of(&items, Scope::Pages(vec!["m".into()]), Format::Markdown, true, false).unwrap();
        assert_eq!(names(&entries), ["notaya-export.txt (2).md"], "its sub-page folder would take the marker's name");
    }

    fn linking(id: &str, section: &str, parent: &str, title: &str, order: i64, targets: &[&str]) -> Item {
        let mut content = vec![format!(r#"{{"type":"paragraph","content":[{{"type":"text","text":"Body of {title}"}}]}}"#)];
        for t in targets {
            content.push(format!(
                r#"{{"type":"paragraph","content":[{{"type":"text","text":"see "}},{{"type":"text","text":"link to {t}","marks":[{{"type":"link","attrs":{{"href":"notaya:page/{t}#h1"}}}}]}}]}}"#
            ));
            content.push(format!(r#"{{"type":"pageLink","attrs":{{"pageId":"{t}"}}}}"#));
        }
        page_with(id, section, parent, title, order, format!(r#"{{"type":"doc","content":[{}]}}"#, content.join(",")))
    }

    #[test]
    fn links_between_pages_of_a_zip_are_relative_paths_to_their_files() {
        let nb = |id: &str, name: &str, order| Item::Notebook(Notebook { source: None, id: id.into(), name: name.into(), color: String::new(), icon: String::new(), order, created: 0, updated: 0, trashed_at: None, });
        let sec = |id: &str, nbid: &str, name: &str, order| Item::Section(Section { source: None, id: id.into(), notebook_id: nbid.into(), name: name.into(), color: String::new(), order, created: 0, updated: 0, trashed_at: None, });
        let items = vec![
            nb("n1", "My notes", 0),
            nb("n2", "Other", 1),
            sec("s1", "n1", "Day 1", 0),
            sec("s2", "n2", "Café & co", 0),
            linking("a", "s1", "", "Plan #1 (draft)", 0, &["b", "c", "gone", "x"]),
            linking("b", "s1", "a", "Sub 100%", 1, &["a", "c"]),
            linking("c", "s2", "", "Ünïcode?", 0, &["b"]),
            page("gone", "s1", "", "Thrown away", 5, true),
        ];
        let (entries, _) = zip_of(&items, Scope::Notebooks(vec!["n1".into(), "n2".into()]), Format::Markdown, true, false).unwrap();
        assert_eq!(names(&entries), ["My notes/Day 1/Plan #1 (draft).md", "My notes/Day 1/Plan #1 (draft)/Sub 100%.md", "Other/Café & co/Ünïcode.md"]);
        let a = text(&entries, "My notes/Day 1/Plan #1 (draft).md");
        assert!(a.contains("[link to b](Plan%20%231%20%28draft%29/Sub%20100%25.md)"), "{a}");
        assert!(a.contains("→ [Sub 100%](Plan%20%231%20%28draft%29/Sub%20100%25.md)"), "{a}");
        assert!(a.contains("[link to c](../../Other/Caf%C3%A9%20%26%20co/%C3%9Cn%C3%AFcode.md)"), "{a}");
        // A trashed page and a page that does not exist: text only, never a notaya: address.
        assert!(a.contains("see link to gone") && a.contains("→ Page not found") && a.contains("see link to x"), "{a}");
        assert!(!a.contains("notaya:"), "{a}");
        let b = text(&entries, "My notes/Day 1/Plan #1 (draft)/Sub 100%.md");
        assert!(b.contains("[link to a](../Plan%20%231%20%28draft%29.md)"), "{b}");
        assert!(b.contains("(../../../Other/Caf%C3%A9%20%26%20co/%C3%9Cn%C3%AFcode.md)"), "{b}");
        let c = text(&entries, "Other/Café & co/Ünïcode.md");
        assert!(c.contains("(../../My%20notes/Day%201/Plan%20%231%20%28draft%29/Sub%20100%25.md)"), "{c}");
        // Only the chosen pages count: a link to a page outside the export is text.
        let (entries, _) = zip_of(&items, Scope::Notebook("n2".into()), Format::Markdown, true, false).unwrap();
        let c = text(&entries, "Other/Café & co/Ünïcode.md");
        assert!(c.contains("see link to b") && c.contains("→ Sub 100%") && !c.contains("](") && !c.contains("notaya:"), "{c}");
        // HTML and Word point at their own files.
        let (entries, _) = zip_of(&items, Scope::Section("s1".into()), Format::Html, true, false).unwrap();
        let a = text(&entries, "Day 1/Plan #1 (draft).html");
        assert!(a.contains("<a href=\"Plan%20%231%20%28draft%29/Sub%20100%25.html\" rel=\"noopener noreferrer\">link to b</a>"), "{a}");
        assert!(a.contains("<p dir=\"auto\">\u{2192} <a href=\"Plan%20%231%20%28draft%29/Sub%20100%25.html\""), "{a}");
        assert!(!a.contains("notaya:") && !a.contains("href=\"#"), "{a}");
        let (entries, _) = zip_of(&items, Scope::Section("s1".into()), Format::Docx, true, false).unwrap();
        let docx = &entries.iter().find(|(n, _)| n == "Day 1/Plan #1 (draft).docx").unwrap().1;
        let parts = read_zip(docx);
        let rels = std::str::from_utf8(&parts.iter().find(|(n, _)| n == "word/_rels/document.xml.rels").unwrap().1).unwrap().to_string();
        assert!(rels.contains("Target=\"Plan%20%231%20%28draft%29/Sub%20100%25.docx\" TargetMode=\"External\""), "{rels}");
        assert_eq!(rels.matches("Sub%20100%25.docx").count(), 1, "one relationship for both links: {rels}");
        assert!(!rels.contains("notaya"));
    }

    #[test]
    fn relative_paths_climb_and_descend() {
        let v = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(relative_href(&v(&["A", "B"]), &v(&["A", "B"]), "x.md"), "x.md");
        assert_eq!(relative_href(&v(&["A", "B"]), &v(&["A", "B", "C"]), "x.md"), "C/x.md");
        assert_eq!(relative_href(&v(&["A", "B", "C"]), &v(&["A", "D"]), "x y.md"), "../../D/x%20y.md");
        assert_eq!(relative_href(&v(&[]), &v(&["a:b"]), "c.md"), "a%3Ab/c.md", "never a scheme");
        for (from, to) in [(v(&["A", "B"]), v(&["C"])), (v(&[]), v(&["A", "B"])), (v(&["A"]), v(&[]))] {
            let href = relative_href(&from, &to, "Page.md");
            let joined = crate::importers::resolve_rel(&from.join("/"), &href);
            let mut want = to.clone();
            want.push("Page.md".into());
            assert_eq!(joined, Some(want.join("/")), "{href}");
        }
    }

    #[test]
    fn links_inside_one_html_or_word_file_go_to_the_page_s_place() {
        let items = vec![
            Item::Notebook(Notebook { source: None, id: "n".into(), name: "N".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "S".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            linking("a", "s", "", "First", 0, &["b", "elsewhere"]),
            linking("b", "s", "", "Second", 1, &[]),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        let d = collect(&refs, &Scope::Section("s".into()), &Options::default(), &|_| None).unwrap();
        assert_eq!(d.anchor_targets().into_iter().collect::<Vec<_>>(), [2]);
        let html = html::render(&d).unwrap();
        assert!(html.contains("<h2 id=\"notaya_page_2\" dir=\"auto\">Second</h2>"), "{html}");
        assert!(html.contains("<h2 dir=\"auto\">First</h2>"), "a page nothing links to has no id");
        assert!(html.contains("<a href=\"#notaya_page_2\" rel=\"noopener noreferrer\">link to b</a>"), "{html}");
        assert!(html.contains("see link to elsewhere") && !html.contains("notaya:"), "{html}");
        let parts = read_zip(&docx::render_at(&d, 0).unwrap());
        let document = std::str::from_utf8(&parts.iter().find(|(n, _)| n == "word/document.xml").unwrap().1).unwrap().to_string();
        assert!(document.contains("<w:bookmarkStart w:id=\"1\" w:name=\"notaya_page_2\"/>"), "{document}");
        assert!(document.contains("<w:bookmarkEnd w:id=\"1\"/>"));
        assert_eq!(document.matches("<w:hyperlink w:anchor=\"notaya_page_2\"").count(), 2, "the link and the row");
        // Markdown in one file keeps the text (heading anchors differ between viewers).
        let md = markdown::render(&d, "x.assets").text;
        assert!(md.contains("see link to b") && md.contains("→ Second") && !md.contains("](") && !md.contains("notaya:"), "{md}");
    }

    #[test]
    fn a_zip_is_written_atomically_and_a_failure_changes_nothing() {
        let items = sample_items();
        let refs: Vec<&Item> = items.iter().collect();
        let opts = Options { include_subpages: true, ..Options::default() };
        let root = temp_dir();
        let dest = root.join("out.zip");
        fs::write(&dest, b"old").unwrap();
        // Too many files: refused with a clear message, the old file stays, nothing else is left.
        let zdoc = collect_zip(&refs, &Scope::Notebook("nb".into()), &opts, Format::Markdown).unwrap();
        let err = write_zip(&dest, Format::Markdown, zdoc, &vault_picture, ZipLimits { max_entries: 2, ..ZipLimits::default() }).unwrap_err().to_string();
        assert!(err.contains("at most 2 files"), "{err}");
        assert_eq!(fs::read(&dest).unwrap(), b"old");
        // Too many bytes: the same.
        let zdoc = collect_zip(&refs, &Scope::Notebook("nb".into()), &opts, Format::Pdf).unwrap();
        let err = write_zip(&dest, Format::Pdf, zdoc, &vault_picture, ZipLimits { max_bytes: 3000, ..ZipLimits::default() }).unwrap_err().to_string();
        assert!(err.contains("larger than") && err.contains("Choose fewer pages"), "{err}");
        assert_eq!(fs::read(&dest).unwrap(), b"old");
        let names_now: Vec<String> = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(names_now, ["out.zip"], "no temporary files");
        // Within the limits it replaces the file.
        let zdoc = collect_zip(&refs, &Scope::Notebook("nb".into()), &opts, Format::Markdown).unwrap();
        let s = write_zip(&dest, Format::Markdown, zdoc, &vault_picture, ZipLimits::default()).unwrap();
        assert_eq!(s.pages, 4);
        assert_eq!(read_zip(&fs::read(&dest).unwrap()).len(), 5, "four pages and the marker");
        let leftovers = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with(".tmp")).count();
        assert_eq!(leftovers, 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_size_limit_counts_pictures_before_they_are_kept() {
        let items = vec![
            Item::Notebook(Notebook { source: None, id: "n".into(), name: "N".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            Item::Section(Section { source: None, id: "s".into(), notebook_id: "n".into(), name: "S".into(), color: String::new(), order: 0, created: 0, updated: 0, trashed_at: None, }),
            page_with("a", "s", "", "A", 0, with_picture()),
        ];
        let refs: Vec<&Item> = items.iter().collect();
        let zdoc = collect_zip(&refs, &Scope::Section("s".into()), &Options::default(), Format::Markdown).unwrap();
        let root = temp_dir();
        let big = |_: &str| Some(("image/png".to_string(), vec![0u8; 5000]));
        let err = write_zip(&root.join("x.zip"), Format::Markdown, zdoc, &big, ZipLimits { max_bytes: 4000, ..ZipLimits::default() }).unwrap_err().to_string();
        assert!(err.contains("larger than"), "{err}");
        assert!(!root.join("x.zip").exists());
        // The same for one combined document.
        let opts = Options { max_image_bytes: 4000, ..Options::default() };
        let err = collect(&refs, &Scope::Section("s".into()), &opts, &big).err().unwrap().to_string();
        assert!(err.contains("too much for one file"), "{err}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zip_destinations_are_checked_like_the_others() {
        let root = temp_dir();
        let data = root.join("data");
        fs::create_dir_all(data.join("backups")).unwrap();
        for bad in [data.join("x.zip"), data.join("backups").join("x.zip"), data.join("a").join("..").join("y.zip"), ] {
            assert!(checked_destination(&bad, "zip", &data).is_err(), "{bad:?}");
        }
        // A vault name never survives: the file always ends in .zip.
        assert_eq!(checked_destination(&root.join("vault.nota"), "zip", &data).unwrap(), root.join("vault.nota.zip"));
        assert_eq!(checked_destination(&root.join("notes.ZIP"), "zip", &data).unwrap(), root.join("notes.ZIP"));
        assert_eq!(checked_destination(&root.join("notes"), "zip", &data).unwrap(), root.join("notes.zip"));
        assert_eq!(checked_destination(&root.join("notes.pdf"), "zip", &data).unwrap(), root.join("notes.pdf.zip"));
        fs::write(root.join("old.pdf.zip"), b"keep").unwrap();
        assert!(checked_destination(&root.join("old.pdf"), "zip", &data).is_err(), "never overwrites a file the dialog did not ask about");
        fs::create_dir_all(root.join("folder.zip")).unwrap();
        assert!(checked_destination(&root.join("folder.zip"), "zip", &data).is_err(), "a folder is no file");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn html_is_a_format_of_its_own() {
        assert_eq!(Format::parse("html").unwrap(), Format::Html);
        assert_eq!(Format::Html.ext(), "html");
        assert!(Format::parse("htm").is_err());
        let root = temp_dir();
        let doc = sample_doc(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        let dest = root.join("page.html");
        let s = write(&dest, Format::Html, &doc, &root.join("data")).unwrap();
        assert_eq!((s.pages, s.assets_folder, s.archive), (1, None, false));
        let html = fs::read_to_string(&dest).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>") && html.contains("data:image/png;base64,"));
        assert!(!root.join("page.assets").exists(), "pictures are inside the file");
        let _ = fs::remove_dir_all(root);
    }
}
