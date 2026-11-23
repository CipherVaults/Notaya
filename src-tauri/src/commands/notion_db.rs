//! The tables of a Notion export (Markdown & CSV) become databases.
//!
//! Notion writes each database as `Name <id>.csv` (the view) and `Name <id>_all.csv` (every row), and each row
//! as a page in a folder `Name <id>/Row <id>.md` whose first lines are the row's properties. This module reads
//! the tables, makes a database of each, moves each row page's text into its database row (the page itself is
//! not kept), and turns columns that point at other rows into real relations.
//!
//! The export does not say what a column is, so some kinds are guessed (see `csv::infer_notion`): a person column
//! looks like a text column, and a formula or rollup column holds only its result. Those stay text.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use serde_json::{json, Value};

use super::database::{parse_cells, write_cells};
use super::database_io::{build_from_table_with, csv_row_title, csv_title_column};
use crate::database::csv;
use crate::database::types::*;
use crate::error::{Error, Result};
use crate::importers::doc::{Block, Cell, Inline};
use crate::importers::plan::{self, Plan, PlanPage, RowTarget};
use crate::importers::source::Vfs;
use crate::importers::{strip_notion_id, Control, ImportPhase};
use crate::model::{new_id, Item};

const MAX_DATABASES: usize = 200;

/// One table of the export.
pub struct NotionTable {
    /// The table's display name without its id.
    pub title: String,
    pub table: csv::Table,
    /// The CSV files of this table (the plain one and the `_all` one).
    pub paths: Vec<String>,
}

/// A page of the import that sits in a table's folder: a row.
pub struct RowPage {
    pub table: usize,
    pub page: usize,
    /// The Notion id in the file name (32 hex characters), when it has one.
    pub notion_id: Option<String>,
}

pub struct Imported {
    pub items: Vec<Item>,
    pub row_targets: HashMap<String, RowTarget>,
    row_indexes: HashMap<String, usize>,
    header_indexes: HashMap<String, usize>,
    pub covered: Vec<String>,
    pub notes: Vec<String>,
    pub databases: usize,
}

fn split_dir(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

/// The 32-hex Notion id at the end of a name or link (dashes and a `.md` ending allowed).
pub fn notion_id(text: &str) -> Option<String> {
    let t = text.trim().trim_end_matches(')');
    let t = t.strip_suffix(".md").unwrap_or(t);
    let t = t.split(['?', '#']).next().unwrap_or(t);
    let hex: Vec<char> = t.chars().rev().filter(|c| *c != '-').take(32).collect();
    if hex.len() == 32 && hex.iter().all(|c| c.is_ascii_hexdigit()) {
        let id: String = hex.into_iter().rev().collect::<String>().to_ascii_lowercase();
        // The characters just before must not be hex too only when the name had a separator; accept as is.
        return Some(id);
    }
    None
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Reads the tables out of the CSV files of an export (`_all` wins over the plain file).
pub fn read_tables(vfs: &Vfs, csv_paths: &[String], ctl: &Control) -> Result<(Vec<NotionTable>, Vec<String>, Vec<String>)> {
    let check = || plan::check(ctl).map_err(Error::invalid);
    let mut groups: BTreeMap<String, (Option<String>, Option<String>, Vec<String>)> = BTreeMap::new();
    for p in csv_paths {
        check()?;
        let (dir, name) = split_dir(p);
        let stem = name.get(..name.len().saturating_sub(4)).unwrap_or(name);
        let (base, is_all) = match stem.strip_suffix("_all") {
            Some(b) => (b, true),
            None => (stem, false),
        };
        let e = groups.entry(format!("{dir}/{base}")).or_default();
        if is_all { e.1 = Some(p.clone()) } else { e.0 = Some(p.clone()) }
        e.2.push(p.clone());
    }
    // The same database in two exports (a Markdown & CSV export next to an HTML export, in different folders)
    // is one table: the files with the same Notion id are joined, and the `_all` file wins.
    let mut by_id: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in groups.keys() {
        if let Some(id) = notion_id(split_dir(key).1) {
            by_id.entry(id).or_default().push(key.clone());
        }
    }
    for (_, keys) in by_id {
        if keys.len() < 2 || keys.iter().filter(|k| groups[*k].1.is_some()).count() > 1 {
            continue;
        }
        let keep = keys.iter().find(|k| groups[*k].1.is_some()).unwrap_or(&keys[0]).clone();
        for k in keys.into_iter().filter(|k| *k != keep) {
            if let Some(g) = groups.remove(&k) {
                if let Some(target) = groups.get_mut(&keep) {
                    target.2.extend(g.2);
                }
            }
        }
    }
    let mut tables = Vec::new();
    let mut notes = Vec::new();
    // Names are presentation, never database identity. Report each conflicting name once,
    // including groups beyond the database cap; retain the existing id/path grouping.
    let mut names: BTreeMap<String, (String, HashSet<String>)> = BTreeMap::new();
    for key in groups.keys() {
        let base = split_dir(key).1;
        if let Some(id) = notion_id(base) {
            let clean = crate::importers::doc::collapse_ws(&strip_notion_id(base));
            let display = clean.chars().filter(|c| !c.is_control()).take(MAX_NAME).collect();
            names.entry(clean.to_lowercase()).or_insert_with(|| (display, HashSet::new())).1.insert(id);
        }
    }
    for (_, (name, ids)) in names {
        if ids.len() > 1 {
            notes.push(format!("Database name \"{name}\" is ambiguous: its CSV views have different source ids and were kept separate."));
        }
    }
    let mut covered = Vec::new();
    let total = groups.len();
    if total > 0 {
        ctl.progress(ImportPhase::ReadingDatabases, 0, total);
    }
    for (index, (key, (plain, all, paths))) in groups.into_iter().enumerate() {
        check()?;
        if tables.len() >= MAX_DATABASES {
            notes.push(format!("More than {MAX_DATABASES} databases: the rest were not imported."));
            break;
        }
        let Some(path) = all.or(plain) else { continue };
        let (_, base) = split_dir(&key);
        let title: String = strip_notion_id(base).chars().filter(|c| !c.is_control()).take(MAX_NAME).collect();
        let title = if title.trim().is_empty() { "Imported table".to_string() } else { title };
        match vfs.read(&path, csv::MAX_BYTES).map_err(|e| e.to_string()).and_then(|b| csv::parse(&csv::decode(&b))) {
            Ok(table) => {
                covered.extend(paths.iter().cloned());
                tables.push(NotionTable { title, table, paths });
            }
            Err(e) => notes.push(format!("The table \"{title}\" was not imported: {e}")),
        }
        ctl.progress(ImportPhase::ReadingDatabases, index + 1, total);
    }
    check()?;
    Ok((tables, covered, notes))
}

/// Use every discovered CSV to determine ownership, including tables that could not be imported.
/// Plain and `_all` views of the same source count as one owner.
#[derive(Default)]
struct TableOwners<'a> {
    ids: HashMap<&'a str, HashMap<String, Option<usize>>>,
    names: HashMap<&'a str, HashMap<String, (String, Option<usize>)>>,
    unavailable: HashSet<&'a str>,
}

impl<'a> TableOwners<'a> {
    fn new(tables: &[NotionTable], csv_paths: &'a [String], ctl: &Control) -> Result<Self> {
        let loaded: HashMap<&str, usize> = tables.iter().enumerate()
            .flat_map(|(i, t)| t.paths.iter().map(move |path| (path.as_str(), i))).collect();
        let mut owners = Self::default();
        for path in csv_paths {
            plan::check(ctl).map_err(Error::invalid)?;
            let (dir, file) = split_dir(path);
            let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
            let base = stem.strip_suffix("_all").unwrap_or(stem);
            let table = loaded.get(path.as_str()).copied();
            if table.is_none() {
                owners.unavailable.insert(dir);
            }
            let id = notion_id(base);
            if let Some(id) = &id {
                owners.ids.entry(dir).or_default().entry(id.clone())
                    .and_modify(|owner| { if *owner != table { *owner = None; } })
                    .or_insert(table);
            }
            let name = owner_name(base);
            let identity = id.unwrap_or_else(|| base.to_string());
            owners.names.entry(dir).or_default().entry(name)
                .and_modify(|(source, owner)| {
                    if *source != identity || *owner != table { *owner = None; }
                })
                .or_insert((identity, table));
        }
        Ok(owners)
    }

    /// Outer None means there is no adjacent database folder. An unresolved owner must
    /// never fall through to the heuristic for flat exports.
    fn folder(&self, dir: &str) -> Option<Option<usize>> {
        if dir.is_empty() {
            return None;
        }
        let (parent, folder) = split_dir(dir);
        let name = || self.names.get(parent)?.get(&owner_name(folder));
        if let Some(id) = notion_id(folder) {
            if let Some(owner) = self.ids.get(parent).and_then(|ids| ids.get(&id)) {
                return Some(*owner);
            }
            // A different explicit ID cannot match by display name.
            return name().map(|_| None);
        }
        name().map(|(_, owner)| *owner)
    }
}

fn owner_name(name: &str) -> String {
    crate::importers::doc::collapse_ws(&strip_notion_id(name)).to_lowercase()
}

/// The text lines of a paragraph (a line break or a newline ends a line).
fn lines_of(inlines: &[Inline]) -> Vec<String> {
    let mut lines = vec![String::new()];
    for i in inlines {
        match i {
            Inline::Text(t, _) => {
                for (n, part) in t.split('\n').enumerate() {
                    if n > 0 {
                        lines.push(String::new());
                    }
                    lines.last_mut().unwrap().push_str(part);
                }
            }
            Inline::Break => lines.push(String::new()),
            _ => {}
        }
    }
    lines.retain(|l| !l.trim().is_empty());
    lines
}

/// The names in the first column of the properties table at the top of an HTML row page (name | value rows).
fn table_keys(rows: &[Vec<Cell>]) -> Option<Vec<String>> {
    if rows.is_empty() || rows.iter().any(|r| r.len() != 2) {
        return None;
    }
    Some(
        rows.iter()
            .map(|r| match r[0].blocks.first() {
                Some(Block::Paragraph(inl)) => lines_of(inl).join(" ").trim().to_lowercase(),
                _ => String::new(),
            })
            .collect(),
    )
}

/// Whether most of a properties table's names are columns of the table (a few properties may be left out of the CSV).
fn is_properties_of(rows: &[Vec<Cell>], headers: &HashSet<String>) -> bool {
    table_keys(rows).is_some_and(|keys| {
        let hits = keys.iter().filter(|k| headers.contains(*k)).count();
        hits > 0 && hits * 2 >= keys.len()
    })
}

/// Removes the property lines Notion writes at the top of a row page (`Status: Done`): they are columns now.
/// An HTML export has them as a table instead.
fn strip_property_lines(blocks: &mut Vec<Block>, headers: &HashSet<String>) {
    if matches!(blocks.first(), Some(Block::Table(rows)) if is_properties_of(rows, headers)) {
        blocks.remove(0);
        return;
    }
    while let Some(Block::Paragraph(inl)) = blocks.first() {
        let lines = lines_of(inl);
        if lines.is_empty() {
            blocks.remove(0);
            continue;
        }
        let is_prop = |l: &String| l.split_once(':').is_some_and(|(k, _)| headers.contains(&k.trim().to_lowercase()));
        if lines.iter().all(is_prop) {
            blocks.remove(0);
        } else {
            break;
        }
    }
}

/// A flat export puts every row page next to the CSV files instead of in the table's folder. Such a page is a
/// row of the table whose columns name all of its leading property lines (`Status: Done`) and which has a row
/// with the page's title.
fn flat_table_of(candidates: &[usize], titles: &[HashSet<String>], headers: &[HashSet<String>], p: &PlanPage) -> Option<usize> {
    // Property lines (Markdown export) or a properties table (HTML export) at the top of the page.
    let (keys, majority): (Vec<String>, bool) = match p.blocks.first() {
        Some(Block::Paragraph(inl)) => {
            let lines = lines_of(inl);
            if lines.is_empty() {
                return None;
            }
            let keys: Vec<String> = lines.iter().filter_map(|l| l.split_once(':').map(|(k, _)| k.trim().to_lowercase())).collect();
            if keys.len() != lines.len() {
                return None;
            }
            (keys, false)
        }
        Some(Block::Table(rows)) => (table_keys(rows)?, true),
        _ => return None,
    };
    let title = p.title.trim().to_lowercase();
    let mut matches = candidates.iter().copied().filter(|&t| {
        let h = &headers[t];
        let all_columns = if majority {
            let hits = keys.iter().filter(|k| h.contains(*k)).count();
            hits > 0 && hits * 2 >= keys.len()
        } else {
            keys.iter().all(|k| h.contains(k))
        };
        all_columns && titles[t].contains(&title)
    });
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

/// Classifies candidate row pages. Their properties are stripped only once they have a row.
pub fn prepare_pages(tables: &[NotionTable], plan: &Plan, ctl: &Control) -> Result<Vec<RowPage>> {
    if tables.is_empty() {
        return Ok(Vec::new());
    }
    let owners = TableOwners::new(tables, &plan.csv_paths, ctl)?;
    let pages = &plan.pages;
    let mut beside: HashMap<&str, Vec<usize>> = HashMap::new();
    let headers: Vec<HashSet<String>> = tables.iter().map(|t| t.table.header.iter().map(|h| h.trim().to_lowercase()).collect()).collect();
    let mut titles = Vec::with_capacity(tables.len());
    for (i, table) in tables.iter().enumerate() {
        plan::check(ctl).map_err(Error::invalid)?;
        for path in &table.paths {
            let candidates = beside.entry(split_dir(path).0).or_default();
            if candidates.last() != Some(&i) {
                candidates.push(i);
            }
        }
        let title_col = csv_title_column(&table.table.header, DEFAULT_TITLE_PROP_NAME, &[]);
        titles.push(table.table.rows.iter().map(|row| csv_row_title(row, title_col).to_lowercase()).collect());
    }
    let mut rows = Vec::new();
    let mut row_ancestry = Vec::with_capacity(pages.len());
    if !pages.is_empty() {
        ctl.progress(ImportPhase::MatchingRows, 0, pages.len());
    }
    for (index, p) in pages.iter().enumerate() {
        plan::check(ctl).map_err(Error::invalid)?;
        let inside_row = p.parent.is_some_and(|parent| row_ancestry[parent]);
        let table = if p.generated {
            None
        } else {
            // A child of a row is an ordinary page even when its opening text resembles
            // CSV properties. An explicit database folder can still identify a nested table.
            let (dir, _) = split_dir(&p.key);
            match owners.folder(dir) {
                Some(owner) => owner,
                None if !inside_row && !owners.unavailable.contains(dir) => {
                    beside.get(dir).and_then(|c| flat_table_of(c, &titles, &headers, p))
                }
                None => None,
            }
        };
        if let Some(t) = table {
            let (_, name) = split_dir(&p.key);
            rows.push(RowPage { table: t, page: index, notion_id: notion_id(name) });
        }
        row_ancestry.push(inside_row || table.is_some());
        ctl.progress(ImportPhase::MatchingRows, index + 1, pages.len());
    }
    Ok(rows)
}

/// `Title (https://www.notion.so/…-<id>)` or `Title (Folder/Title <id>.md)`, separated by `, `.
fn parse_links(cell: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut rest = cell.trim();
    while !rest.is_empty() {
        // The entry ends at the first ")" that is followed by ", " or the end.
        let mut end = None;
        for (i, _) in rest.match_indices(')') {
            let after = &rest[i + 1..];
            if after.is_empty() || after.starts_with(", ") {
                end = Some(i);
                break;
            }
        }
        let end = end?;
        let seg = &rest[..end];
        let open = seg.rfind(" (")?;
        let (title, link) = (seg[..open].trim(), &seg[open + 2..]);
        // A flat export (no sub-page folders) points at the table's file, as in `Name (Customers.csv)`.
        let lower = link.to_lowercase();
        let linkish = lower.contains("notion.so") || lower.contains(".md") || lower.ends_with(".csv") || lower.ends_with(".html") || lower.starts_with("http");
        // A title with ", " inside is most likely an unlinked entry run into the next one; keep the cell as text.
        if !linkish || title.contains(", ") {
            return None;
        }
        out.push((title.to_string(), link.to_string()));
        rest = rest[end + 1..].trim_start_matches(", ").trim_start();
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Makes the databases and assigns canonical row IDs before the source pages are rendered.
pub fn import_databases(
    tables: &[NotionTable],
    row_pages: &[RowPage],
    plan: &mut Plan,
    now: i64,
    ctl: &Control,
) -> Result<Imported> {
    let mut out = Imported { items: Vec::new(), row_targets: HashMap::new(), row_indexes: HashMap::new(), header_indexes: HashMap::new(), covered: Vec::new(), notes: Vec::new(), databases: 0 };
    if tables.is_empty() {
        return Ok(out);
    }
    let header_pages = plan.database_header_ids();
    let check = || plan::check(ctl).map_err(Error::invalid);
    let total = 3 * tables.len();
    ctl.progress(ImportPhase::FoldingDatabases, 0, total);

    struct Made<'a> {
        built: crate::commands::database_io::Built,
        /// For each CSV row, the source page paired with it, if any.
        source_pages: Vec<Option<&'a RowPage>>,
    }
    let mut made: Vec<Option<Made<'_>>> = Vec::new();

    for (ti, t) in tables.iter().enumerate() {
        check()?;
        let mine: Vec<&RowPage> = row_pages.iter().filter(|r| r.table == ti).collect();
        let section = mine.first().map(|r| plan.pages[r.page].section)
            .or_else(|| t.paths.iter().find_map(|p| plan.csv_sections.get(p).copied()))
            .ok_or_else(|| Error::invalid("The database source section was not found. Nothing was imported."))?;
        let db = Database::new(new_id(), plan.section_ids[section].clone(), t.title.clone(), now);
        let built = match build_from_table_with(db, &t.table, 0, now, true, &check) {
            Ok(b) => b,
            Err(e) => {
                check()?;
                out.notes.push(format!("The table \"{}\" was not imported: {e}", t.title));
                made.push(None);
                ctl.progress(ImportPhase::FoldingDatabases, ti + 1, total);
                continue;
            }
        };
        let headers: HashSet<String> = t.table.header.iter().map(|h| h.trim().to_lowercase()).collect();
        let mut by_title: HashMap<String, VecDeque<&RowPage>> = HashMap::new();
        for source in mine {
            by_title.entry(plan.pages[source.page].title.trim().to_lowercase()).or_default().push_back(source);
        }
        let mut source_pages = Vec::with_capacity(built.rows.len());
        let mut moved = 0usize;
        for row in &built.rows {
            check()?;
            let want = row.title.trim().to_lowercase();
            // Pair equal titles in source order, once per row and once per page.
            let source = by_title.get_mut(&want).and_then(VecDeque::pop_front);
            if let Some(source) = source {
                let page = &mut plan.pages[source.page];
                strip_property_lines(&mut page.blocks, &headers);
                out.row_targets.insert(page.id.clone(), RowTarget { id: row.id.clone(), section });
                moved += 1;
            }
            source_pages.push(source);
        }
        out.notes.push(format!("Database \"{}\": {} rows, {} columns, {} row pages moved into the rows.", t.title, built.rows.len(), t.table.header.len(), moved));
        made.push(Some(Made { built, source_pages }));
        ctl.progress(ImportPhase::FoldingDatabases, ti + 1, total);
    }

    // ---- relations: a text column whose cells all point at rows of one table
    let mut by_notion_id: HashMap<String, (usize, usize)> = HashMap::new();
    let mut by_title: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (ti, m) in made.iter().enumerate() {
        check()?;
        let Some(m) = m else { continue };
        for (ri, row) in m.built.rows.iter().enumerate() {
            check()?;
            if let Some(id) = m.source_pages[ri].and_then(|p| p.notion_id.as_ref()) {
                by_notion_id.insert(id.clone(), (ti, ri));
            }
            by_title.entry(row.title.trim().to_lowercase()).or_default().push((ti, ri));
        }
    }
    let resolve = |title: &str, link: &str| -> Option<(usize, usize)> {
        let decoded = percent_decode(link);
        if let Some(id) = notion_id(&decoded) {
            if let Some(&hit) = by_notion_id.get(&id) {
                return Some(hit);
            }
        }
        match by_title.get(&title.trim().to_lowercase()) {
            Some(v) if v.len() == 1 => Some(v[0]),
            _ => None,
        }
    };
    // (table, property index) -> links per row: (target table, target row)
    let mut links: HashMap<(usize, usize), (usize, Vec<Vec<usize>>)> = HashMap::new();
    for (ti, m) in made.iter().enumerate() {
        check()?;
        let Some(m) = m else {
            ctl.progress(ImportPhase::FoldingDatabases, tables.len() + ti + 1, total);
            continue;
        };
        let tbl = &tables[ti].table;
        for (pi, prop) in m.built.db.props.iter().enumerate().skip(1) {
            if prop.kind != PropKind::Text {
                continue;
            }
            let Some(col) = tbl.header.iter().position(|h| *h == prop.name) else { continue };
            let mut target: Option<usize> = None;
            let mut per_row: Vec<Vec<usize>> = Vec::with_capacity(tbl.rows.len());
            let mut any = false;
            let mut ok = true;
            'rows: for r in &tbl.rows {
                check()?;
                let cell = r.get(col).map(|c| c.trim()).unwrap_or("");
                if cell.is_empty() {
                    per_row.push(Vec::new());
                    continue;
                }
                let Some(entries) = parse_links(cell) else {
                    ok = false;
                    break;
                };
                let mut ids = Vec::new();
                for (title, link) in entries {
                    let Some((tt, tr)) = resolve(&title, &link) else {
                        ok = false;
                        break 'rows;
                    };
                    if *target.get_or_insert(tt) != tt {
                        ok = false;
                        break 'rows;
                    }
                    if !ids.contains(&tr) {
                        ids.push(tr);
                    }
                }
                any = true;
                per_row.push(ids);
            }
            if ok && any {
                if let Some(tt) = target {
                    links.insert((ti, pi), (tt, per_row));
                }
            }
        }
        ctl.progress(ImportPhase::FoldingDatabases, tables.len() + ti + 1, total);
    }
    // Make the properties and write the cells.
    let db_ids: Vec<Option<String>> = made.iter().map(|m| m.as_ref().map(|m| m.built.db.id.clone())).collect();
    let row_ids: Vec<Vec<String>> = made.iter().map(|m| m.as_ref().map(|m| m.built.rows.iter().map(|r| r.id.clone()).collect()).unwrap_or_default()).collect();
    let mut keys: Vec<(usize, usize)> = links.keys().copied().collect();
    keys.sort();
    let mut prop_ids: HashMap<(usize, usize), String> = HashMap::new();
    for (ti, pi) in &keys {
        let m = made[*ti].as_ref().unwrap();
        prop_ids.insert((*ti, *pi), m.built.db.props[*pi].id.clone());
    }
    // A relation and its mirror in the other table (every link has its opposite) become one two-way relation.
    let mut back: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    for (a, (ta, la)) in &links {
        check()?;
        if back.contains_key(a) {
            continue;
        }
        for (b, (tb, lb)) in &links {
            check()?;
            // b lives in the table a points at, and points back at a's table.
            if a == b || back.contains_key(b) || b.0 != *ta || *tb != a.0 {
                continue;
            }
            let forward: HashSet<(usize, usize)> = la.iter().enumerate().flat_map(|(r, v)| v.iter().map(move |t| (r, *t))).collect();
            let backward: HashSet<(usize, usize)> = lb.iter().enumerate().flat_map(|(r, v)| v.iter().map(move |t| (*t, r))).collect();
            if forward == backward {
                back.insert(*a, *b);
                back.insert(*b, *a);
                break;
            }
        }
    }
    for ((ti, pi), (tt, per_row)) in &links {
        let target_db = db_ids[*tt].clone().unwrap_or_default();
        let back_id = back.get(&(*ti, *pi)).and_then(|b| prop_ids.get(b)).cloned().unwrap_or_default();
        let m = made[*ti].as_mut().unwrap();
        {
            let prop = &mut m.built.db.props[*pi];
            prop.kind = PropKind::Relation;
            prop.relation_db = target_db;
            prop.relation_back = back_id;
            prop.options.clear();
        }
        let pid = m.built.db.props[*pi].id.clone();
        for (ri, row) in m.built.rows.iter_mut().enumerate() {
            check()?;
            let mut cells = parse_cells(row);
            cells.remove(&pid);
            let ids = &per_row[ri];
            if !ids.is_empty() {
                cells.insert(pid.clone(), Value::Array(ids.iter().map(|t| json!(row_ids[*tt][*t])).collect()));
            }
            write_cells(row, &cells)?;
        }
        out.notes.push(format!("Relation \"{}\" in \"{}\" links to rows of \"{}\".", m.built.db.props[*pi].name, tables[*ti].title, tables[*tt].title));
    }

    for (ti, m) in made.into_iter().enumerate() {
        check()?;
        if let Some(mut m) = m {
            m.built.db.sanitize().map_err(|e| Error::invalid(e.to_string()))?;
            for (row, source) in m.built.rows.into_iter().zip(m.source_pages) {
                if let Some(source) = source {
                    out.row_indexes.insert(plan.pages[source.page].id.clone(), out.items.len());
                }
                out.items.push(Item::Row(row));
            }
            if let Some(id) = tables[ti].paths.iter().find_map(|p| header_pages.get(p)) {
                let owners = tables.iter().filter(|table| table.paths.iter().any(|p| header_pages.get(p) == Some(id))).count();
                if owners == 1 {
                    out.header_indexes.insert(id.clone(), out.items.len());
                }
            }
            out.items.push(Item::Database(m.built.db));
            out.databases += 1;
            out.covered.extend(tables[ti].paths.iter().cloned());
        }
        ctl.progress(ImportPhase::FoldingDatabases, 2 * tables.len() + ti + 1, total);
    }
    if out.databases > 0 {
        out.notes.push("Notion does not say which columns hold people, formulas or rollups, so those columns arrive as text (a formula column keeps only its last result).".into());
    }
    check()?;
    Ok(out)
}

impl Imported {
    /// Moves rendered bodies without copying them. The caller removes the emptied source items
    /// and uses their former parents to prune empty generated folders.
    pub fn move_bodies(&mut self, items: &mut [Item], ctl: &Control) -> Result<HashSet<String>> {
        let mut removed = HashSet::new();
        for item in items.iter_mut() {
            plan::check(ctl).map_err(Error::invalid)?;
            let Item::Page(page) = item else { continue };
            if let Some(index) = self.row_indexes.get(&page.id) {
                let Item::Row(row) = &mut self.items[*index] else { unreachable!("row index") };
                row.content = std::mem::take(&mut page.content);
                row.text = std::mem::take(&mut page.text);
                row.icon = std::mem::take(&mut page.icon);
                row.cover = std::mem::take(&mut page.cover);
                removed.insert(page.id.clone());
            }
        }
        // A plain wrapper is optional. Preserve it when it has incoming links, children,
        // or text that cannot fit the database's existing description limit.
        for (id, index) in &self.header_indexes {
            plan::check(ctl).map_err(Error::invalid)?;
            if items.iter().chain(&self.items).any(|item| match item {
                Item::Page(p) => p.parent == *id || (p.id != *id && p.content.contains(id)),
                Item::Row(r) => r.content.contains(id),
                _ => false,
            }) {
                continue;
            }
            if let Some(Item::Page(page)) = items.iter_mut().find(|item| item.id() == id) {
                if page.text.chars().take(2_001).count() <= 2_000 {
                    let Item::Database(db) = &mut self.items[*index] else { unreachable!("database index") };
                    db.description = std::mem::take(&mut page.text);
                    removed.insert(id.clone());
                }
            }
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_links_are_read() {
        assert_eq!(notion_id("Alpha 98b9830964384ed4b08e1c3de9ace4e9.md").as_deref(), Some("98b9830964384ed4b08e1c3de9ace4e9"));
        assert_eq!(notion_id("https://www.notion.so/Alpha-98b9830964384ed4b08e1c3de9ace4e9").as_deref(), Some("98b9830964384ed4b08e1c3de9ace4e9"));
        assert_eq!(notion_id("Alpha"), None);
        let l = parse_links("A (Tasks/A%20x.md), B, C (https://www.notion.so/B-98b9830964384ed4b08e1c3de9ace4e9)");
        assert!(l.is_none(), "an entry without a link is not a relation");
        let l = parse_links("A (Tasks/A%20x.md), B (https://www.notion.so/B-98b9830964384ed4b08e1c3de9ace4e9)").unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].0, "A");
        assert_eq!(percent_decode("a%20b%2Fc"), "a b/c");
        // A flat export names the table file instead of a page.
        let l = parse_links("Anna (Customers%20abc.csv), Omar (Customers.csv)").unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[1].0, "Omar");
    }
}
