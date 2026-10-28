//! Exporting pages, a section or a notebook to Markdown, PDF or Word (V1.4.0).
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

pub mod deflate;
pub mod doc;
pub mod docx;
pub mod image;
pub mod markdown;
pub mod pdf;
mod pdf_fonts;
pub mod zip;

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::model::{Item, Page};
use doc::{Block, Cell};
use image::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Pdf,
    Docx,
}

impl Format {
    pub fn parse(s: &str) -> Result<Format> {
        match s {
            "markdown" => Ok(Format::Markdown),
            "pdf" => Ok(Format::Pdf),
            "docx" => Ok(Format::Docx),
            _ => Err(Error::invalid("Unknown export format.")),
        }
    }
    pub fn ext(self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
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
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub include_subpages: bool,
    pub include_trashed: bool,
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
    /// The folder the Markdown pictures were written to, if any.
    pub assets_folder: Option<String>,
}

/* ---------- Choosing the pages ---------- */

struct Tree<'a> {
    pages: Vec<&'a Page>,
    item_name: BTreeMap<&'a str, (&'a str, i64, i64)>,
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
    let mut item_name = BTreeMap::new();
    for i in items {
        match i {
            Item::Notebook(n) => {
                item_name.insert(n.id.as_str(), (n.name.as_str(), n.order, n.created));
            }
            Item::Section(s) => {
                item_name.insert(s.id.as_str(), (s.name.as_str(), s.order, s.created));
            }
            _ => {}
        }
    }
    Tree { pages, item_name }
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

/// The name of a notebook or section, and the file name the save dialog suggests.
pub fn suggested_name(items: &[&Item], scope: &Scope) -> Result<String> {
    let title = match scope {
        Scope::Notebook(id) | Scope::Section(id) => items
            .iter()
            .find_map(|i| match i {
                Item::Notebook(n) if &n.id == id && matches!(scope, Scope::Notebook(_)) => Some(n.name.clone()),
                Item::Section(s) if &s.id == id && matches!(scope, Scope::Section(_)) => Some(s.name.clone()),
                _ => None,
            })
            .ok_or(Error::NotFound)?,
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

/// Builds the document for an export. `attachment` reads one picture from the vault.
pub fn collect(items: &[&Item], scope: &Scope, opts: &Options, attachment: &dyn Fn(&str) -> Option<(String, Vec<u8>)>) -> Result<ExportDoc> {
    let tree = sorted_pages(items, opts);
    let title_of = |id: &str| items.iter().find_map(|i| match i {
        Item::Page(p) if p.id == id => Some(page_title(&p.title)),
        _ => None,
    });
    let mut parts = Vec::new();
    let mut seen = HashSet::new();
    let push_pages = |list: Vec<(&Page, u8)>, base: u8, parts: &mut Vec<Part>| {
        for (p, depth) in list {
            let level = base.saturating_add(depth).min(6);
            parts.push(Part::Page { level, title: page_title(&p.title), blocks: doc::parse(&p.content, &title_of) });
        }
    };
    let sections_of = |notebook: &str| -> Vec<(&str, &str)> {
        let mut v: Vec<(&str, &str, i64, i64)> = items
            .iter()
            .filter_map(|i| match i {
                Item::Section(s) if s.notebook_id == notebook => Some((s.id.as_str(), s.name.as_str(), s.order, s.created)),
                _ => None,
            })
            .collect();
        v.sort_by_key(|x| (x.2, x.3));
        v.into_iter().map(|x| (x.0, x.1)).collect()
    };
    let doc_title;
    match scope {
        Scope::Notebook(id) => {
            let (name, _, _) = *tree.item_name.get(id.as_str()).ok_or(Error::NotFound)?;
            if !items.iter().any(|i| matches!(i, Item::Notebook(n) if &n.id == id)) {
                return Err(Error::NotFound);
            }
            doc_title = name.to_string();
            parts.push(Part::Heading { level: 1, text: name.to_string() });
            for (sid, sname) in sections_of(id) {
                let mut list = Vec::new();
                for r in tree.roots(sid) {
                    tree.subtree(r, 0, opts.include_subpages, &mut list, &mut seen);
                }
                parts.push(Part::Heading { level: 2, text: sname.to_string() });
                push_pages(list, 3, &mut parts);
            }
        }
        Scope::Section(id) => {
            let name = items
                .iter()
                .find_map(|i| match i {
                    Item::Section(s) if &s.id == id => Some(s.name.clone()),
                    _ => None,
                })
                .ok_or(Error::NotFound)?;
            doc_title = name.clone();
            parts.push(Part::Heading { level: 1, text: name });
            let mut list = Vec::new();
            for r in tree.roots(id) {
                tree.subtree(r, 0, opts.include_subpages, &mut list, &mut seen);
            }
            push_pages(list, 2, &mut parts);
        }
        Scope::Pages(ids) => {
            // In the sidebar's order: notebook, section, then position in the page tree.
            let mut position: BTreeMap<&str, usize> = BTreeMap::new();
            let mut notebooks: Vec<(&str, i64, i64)> = items
                .iter()
                .filter_map(|i| match i {
                    Item::Notebook(n) => Some((n.id.as_str(), n.order, n.created)),
                    _ => None,
                })
                .collect();
            notebooks.sort_by_key(|n| (n.1, n.2));
            let mut all = Vec::new();
            let mut walk_seen = HashSet::new();
            for (nb, _, _) in &notebooks {
                for (sid, _) in sections_of(nb) {
                    for r in tree.roots(sid) {
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
            doc_title = match chosen.as_slice() {
                [one] => page_title(&one.title),
                _ => format!("{} pages", list.len()),
            };
            push_pages(list, 1, &mut parts);
        }
    }
    let mut out = ExportDoc { title: doc_title, parts, images: BTreeMap::new(), missing_images: 0 };
    if out.page_count() == 0 {
        return Err(Error::invalid(match scope {
            Scope::Pages(_) => "There is nothing to export: the chosen pages are in the trash or no longer exist.",
            _ => "There is nothing to export: there are no pages here (pages in the trash are left out).",
        }));
    }
    let mut ids = Vec::new();
    for part in &out.parts {
        if let Part::Page { blocks, .. } = part {
            doc::attachments(blocks, &mut ids);
        }
    }
    for id in ids {
        match attachment(&id).and_then(|(mime, bytes)| Kind::from_mime(&mime).map(|kind| Picture { kind, bytes })) {
            Some(p) => {
                out.images.insert(id, p);
            }
            None => out.missing_images += 1,
        }
    }
    Ok(out)
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
    let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
        "LPT9",
    ];
    if RESERVED.contains(&stem.as_str()) {
        s.push('_');
    }
    s
}

/// The path with symbolic links and `..` resolved; a file that does not exist yet is
/// resolved through its folder.
fn resolve(p: &Path) -> PathBuf {
    fs::canonicalize(p)
        .or_else(|_| {
            let dir = p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
            let name = p.file_name().ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
            fs::canonicalize(dir).map(|d| d.join(name))
        })
        .unwrap_or_else(|_| p.to_path_buf())
}

/// Checks the place the person chose. The file gets the format's extension (added when the
/// name has another one; an existing file with the longer name is then refused, because the
/// save dialog asked about a different file). Nothing inside the app's data folder is allowed,
/// so an export can never replace the vault or a backup.
pub fn checked_destination(chosen: &Path, format: Format, data_dir: &Path) -> Result<PathBuf> {
    let has_ext = chosen.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(format.ext()));
    let dest = if has_ext {
        chosen.to_path_buf()
    } else {
        let mut name = chosen.file_name().ok_or_else(|| Error::invalid("Choose a file name for the export."))?.to_os_string();
        name.push(".");
        name.push(format.ext());
        let longer = chosen.with_file_name(name);
        if longer.exists() {
            return Err(Error::invalid(format!(
                "A file named {} already exists. Choose another name.",
                longer.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
            )));
        }
        longer
    };
    let data = resolve(data_dir);
    let target = resolve(&dest);
    if target.starts_with(&data) || crate::filenames::path_has_vault_ext(&dest) {
        return Err(Error::invalid(
            "That is the folder where Notaya keeps your notes and backups. Choose another folder for the export.",
        ));
    }
    if target.is_dir() {
        return Err(Error::invalid("That name is a folder. Choose a file name for the export."));
    }
    Ok(dest)
}

/// Writes `bytes` to a temporary file next to `dest`, flushes it to the disk and renames it
/// into place, so a failed export never leaves a half-written file under the chosen name.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<()> {
    let name = dest.file_name().and_then(|n| n.to_str()).ok_or_else(|| Error::invalid("Unsupported file name."))?;
    let tmp = dest.with_file_name(format!("{name}.{}.tmp", crate::model::now_ms()));
    let result = (|| -> Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, dest)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// Renders the document in the chosen format and writes it to `dest` (already checked with
/// [`checked_destination`]). For Markdown the pictures go into `<name>.assets` next to it.
pub fn write(dest: &Path, format: Format, doc: &ExportDoc) -> Result<ExportSummary> {
    let mut summary = ExportSummary {
        path: dest.display().to_string(),
        pages: doc.page_count(),
        images: doc.images.len(),
        missing_images: doc.missing_images,
        ..ExportSummary::default()
    };
    match format {
        Format::Markdown => {
            let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("export");
            let assets_name = format!("{stem}.assets");
            let out = markdown::render(doc, &assets_name);
            if !out.images.is_empty() {
                let dir = dest.with_file_name(&assets_name);
                if dir.exists() && !dir.is_dir() {
                    return Err(Error::invalid(format!("{assets_name} exists and is not a folder. Choose another name.")));
                }
                fs::create_dir_all(&dir)?;
                // The pictures first, so the .md file never points at pictures that are not there.
                for (name, bytes) in &out.images {
                    write_atomic(&dir.join(name), bytes)?;
                }
                summary.assets_folder = Some(dir.display().to_string());
            }
            write_atomic(dest, out.text.as_bytes())?;
        }
        Format::Docx => write_atomic(dest, &docx::render(doc)?)?,
        Format::Pdf => {
            let out = pdf::render(doc);
            summary.images_left_out = out.images_left_out;
            summary.chars_replaced = out.chars_replaced;
            write_atomic(dest, &out.bytes)?;
        }
    }
    Ok(summary)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::model::{Notebook, Section};

    /// A one-page document around the given blocks, with the test picture.
    pub fn sample_doc(blocks: Vec<Block>) -> ExportDoc {
        let mut images = BTreeMap::new();
        images.insert(doc::tests::IMG_ID.to_string(), Picture { kind: Kind::Png, bytes: image::tests::PILLOW_RGBA.to_vec() });
        ExportDoc { title: "Sample page".into(), parts: vec![Part::Page { level: 1, title: "Sample page".into(), blocks }], images, missing_images: 0 }
    }

    fn page(id: &str, section: &str, parent: &str, title: &str, order: i64, trashed: bool) -> Item {
        Item::Page(Page {
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
        let nb = |id: &str, name: &str, order| Item::Notebook(Notebook { id: id.into(), name: name.into(), color: String::new(), icon: String::new(), order, created: 0, updated: 0 });
        let sec = |id: &str, nbid: &str, name: &str, order| {
            Item::Section(Section { id: id.into(), notebook_id: nbid.into(), name: name.into(), color: String::new(), order, created: 0, updated: 0 })
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
        collect(&refs, &scope, &Options { include_subpages: subpages, include_trashed: trashed }, &|_| None)
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
            assert!(checked_destination(&bad, Format::Pdf, &data).is_err(), "{bad:?}");
        }
        assert_eq!(fs::read(data.join("vault.nota")).unwrap(), b"vault");
        let ok = checked_destination(&root.join("notes.PDF"), Format::Pdf, &data).unwrap();
        assert_eq!(ok, root.join("notes.PDF"));
        assert_eq!(checked_destination(&root.join("notes"), Format::Docx, &data).unwrap(), root.join("notes.docx"));
        assert_eq!(checked_destination(&root.join("notes.txt"), Format::Markdown, &data).unwrap(), root.join("notes.txt.md"));
        fs::write(root.join("old.txt.md"), b"keep").unwrap();
        assert!(checked_destination(&root.join("old.txt"), Format::Markdown, &data).is_err(), "never overwrites a file the dialog did not ask about");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn writing_replaces_the_file_and_leaves_no_temp_files() {
        let root = temp_dir();
        let doc = sample_doc(doc::parse(&doc::tests::every_node_json(), &doc::tests::titles));
        for format in [Format::Markdown, Format::Pdf, Format::Docx] {
            let dest = root.join(format!("My notes.{}", format.ext()));
            fs::write(&dest, b"old").unwrap();
            let s = write(&dest, format, &doc).unwrap();
            assert_eq!(s.pages, 1);
            assert_ne!(fs::read(&dest).unwrap(), b"old");
        }
        assert!(root.join("My notes.assets").join("image-001.png").is_file());
        let leftovers: Vec<_> = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with(".tmp")).collect();
        assert!(leftovers.is_empty());
        // Sample files for a look with real tools, when asked for (see the export record).
        if let Some(dir) = std::env::var_os("NOTAYA_EXPORT_SAMPLES") {
            let dir = PathBuf::from(dir);
            fs::create_dir_all(&dir).unwrap();
            for format in [Format::Markdown, Format::Pdf, Format::Docx] {
                write(&dir.join(format!("sample.{}", format.ext())), format, &doc).unwrap();
            }
            let items = sample_items();
            let refs: Vec<&Item> = items.iter().collect();
            let nb = collect(&refs, &Scope::Notebook("nb".into()), &Options { include_subpages: true, include_trashed: false }, &|_| None).unwrap();
            for format in [Format::Pdf, Format::Docx] {
                write(&dir.join(format!("notebook.{}", format.ext())), format, &nb).unwrap();
            }
        }
        let _ = fs::remove_dir_all(root);
    }
}
