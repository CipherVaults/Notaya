//! Files and CSV for databases (A35): importing a CSV file as a new database (or into an existing
//! one), exporting a view as CSV, and the files held in a row's "Files" property.
//!
//! As everywhere in Notaya, the UI never gives a path: the dialogs are opened here, the file is
//! read and checked here, and only a summary goes back. A file attached to a row is stored
//! encrypted in the vault. It is never served with the type it claims (`clatt` only shows
//! pictures); opening it in another program writes a short-lived plain copy, and only for types
//! that cannot run code.

use std::path::PathBuf;

use serde::Serialize;
use serde_json::{json, Map, Value};
use tauri::ipc::{InvokeBody, Request};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::database::{self as dbc, ctx_of, first_database_backup, parse_cells, write_cells, FIRST_DATABASE_NOTICE, MAX_ROWS_PER_DATABASE};
use crate::database::csv::{self, Guess};
use crate::database::types::*;
use crate::database::value::{self, Val};
use crate::error::{Error, Result};
use crate::export::checked_destination;
use crate::model::{new_id, now_ms, Item};
use crate::state::AppState;

const COLORS: [&str; 9] = ["blue", "green", "orange", "purple", "pink", "yellow", "red", "brown", "gray"];

// ---------------------------------------------------------------- CSV import

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportResult {
    pub database_id: String,
    /// A new database was made from the file.
    pub created: bool,
    pub title: String,
    pub rows: usize,
    pub columns: usize,
    /// Columns that could not be imported (and why).
    pub skipped: Vec<String>,
    /// Values that did not fit their property and were left empty.
    pub bad_cells: usize,
    pub notice: Option<String>,
}

fn opt(name: &str, i: usize, status: bool) -> SelectOption {
    SelectOption { id: new_id(), name: name.into(), color: COLORS[i % COLORS.len()].into(), group: if status { "todo".into() } else { String::new() } }
}

fn new_prop(name: &str, g: &Guess) -> Property {
    let mut p = Property { id: new_id(), name: name.into(), ..Property::default() };
    match g {
        Guess::Text => p.kind = PropKind::Text,
        Guess::Number => p.kind = PropKind::Number,
        Guess::Checkbox => p.kind = PropKind::Checkbox,
        Guess::Date => p.kind = PropKind::Date,
        Guess::Url => p.kind = PropKind::Url,
        Guess::Email => p.kind = PropKind::Email,
        Guess::Select(v) | Guess::MultiSelect(v) => {
            p.kind = if matches!(g, Guess::Select(_)) { PropKind::Select } else { PropKind::MultiSelect };
            p.options = v.iter().enumerate().map(|(i, n)| opt(n, i, false)).collect();
        }
        Guess::Status(v) => {
            p.kind = PropKind::Status;
            p.options = v
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let mut o = opt(n, i, true);
                    o.group = csv::status_group(n).into();
                    o.color = match o.group.as_str() {
                        "complete" => "green",
                        "inProgress" => "blue",
                        _ => "gray",
                    }
                    .into();
                    o
                })
                .collect();
        }
        Guess::FormattedNumber(fmt) => {
            p.kind = PropKind::Number;
            p.number_format = fmt.clone();
        }
    }
    p
}

fn find_option(p: &mut Property, name: &str) -> Option<String> {
    if let Some(o) = p.options.iter().find(|o| o.name.eq_ignore_ascii_case(name)) {
        return Some(o.id.clone());
    }
    if p.options.len() >= MAX_OPTIONS {
        return None;
    }
    let o = opt(name, p.options.len(), p.kind == PropKind::Status);
    let id = o.id.clone();
    p.options.push(o);
    Some(id)
}

/// One text from a file as the value of a property. `Ok(None)` = empty, `Err(())` = does not fit.
fn convert(p: &mut Property, raw: &str) -> std::result::Result<Option<Value>, ()> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(None);
    }
    Ok(Some(match p.kind {
        PropKind::Text | PropKind::Url | PropKind::Email | PropKind::Phone => json!(csv::unsafe_cell(raw)),
        PropKind::Number => {
            let n = csv::parse_number(t)
                .or_else(|| csv::parse_formatted_number(t).map(|(n, _)| n))
                .or_else(|| t.trim_end_matches('%').trim().parse::<f64>().ok().filter(|n| n.is_finite()))
                .ok_or(())?;
            // A percent property keeps 0.6 for 60%; a plain "60" typed into one stays 60.
            json!(n)
        }
        PropKind::Checkbox => json!(csv::parse_bool(t).ok_or(())?),
        PropKind::Date => {
            let (s, e, time) = csv::parse_date_range(t).ok_or(())?;
            let mut m = Map::new();
            m.insert("start".into(), json!(value::format_iso(s, time)));
            if let Some(e) = e {
                m.insert("end".into(), json!(value::format_iso(e, time)));
            }
            Value::Object(m)
        }
        PropKind::Select | PropKind::Status => json!(find_option(p, t).ok_or(())?),
        PropKind::MultiSelect => {
            let ids: Vec<Value> = csv::split_tags(t).iter().filter_map(|n| find_option(p, n)).map(Value::String).collect();
            if ids.is_empty() {
                return Err(());
            }
            Value::Array(ids)
        }
        PropKind::Person => Value::Array(csv::split_tags(t).into_iter().map(Value::String).collect()),
        PropKind::Place => json!({"address": t}),
        _ => return Err(()),
    }))
}

fn importable(k: PropKind) -> bool {
    !(k.is_computed() || matches!(k, PropKind::Id | PropKind::Files | PropKind::Relation | PropKind::Title))
}

/// Builds the rows (and any new properties and options) from a parsed file. Nothing is stored here.
pub struct Built {
    pub db: Database,
    pub rows: Vec<Row>,
    pub skipped: Vec<String>,
    pub bad_cells: usize,
}

pub fn build_from_table(db: Database, table: &csv::Table, existing_rows: usize, now: i64) -> Result<Built> {
    build_from_table_with(db, table, existing_rows, now, false)
}

/// `notion`: the table comes from a Notion export, so column kinds are guessed the Notion way (see `csv::infer_notion`).
pub fn build_from_table_with(mut db: Database, table: &csv::Table, existing_rows: usize, now: i64, notion: bool) -> Result<Built> {
    if existing_rows + table.rows.len() > MAX_ROWS_PER_DATABASE {
        return Err(Error::invalid("That would be more rows than a database can hold."));
    }
    let ncols = table.header.len();
    let is_new = db.props.len() == 1 && existing_rows == 0;
    // Which property gets which column.
    let mut target: Vec<Option<usize>> = vec![None; ncols];
    let mut skipped: Vec<String> = Vec::new();
    let title_name = db.props[0].name.to_ascii_lowercase();
    let title_col = table
        .header
        .iter()
        .position(|h| h.eq_ignore_ascii_case(&title_name))
        .or_else(|| table.header.iter().position(|h| h.eq_ignore_ascii_case("title") || h.eq_ignore_ascii_case("name")))
        .or_else(|| if db.props.iter().skip(1).any(|p| table.header[0].eq_ignore_ascii_case(&p.name)) { None } else { Some(0) });
    for (i, h) in table.header.iter().enumerate() {
        if Some(i) == title_col {
            continue;
        }
        if let Some(pi) = db.props.iter().position(|p| p.name.eq_ignore_ascii_case(h)) {
            if importable(db.props[pi].kind) {
                target[i] = Some(pi);
            } else {
                skipped.push(format!("{h}: Notaya works this property out or it holds files or links, so it cannot be filled from a file."));
            }
            continue;
        }
        if db.props.len() >= MAX_PROPERTIES {
            skipped.push(format!("{h}: a database can have at most {MAX_PROPERTIES} properties."));
            continue;
        }
        let vals: Vec<&str> = table.rows.iter().map(|r| r[i].as_str()).collect();
        let mut p = new_prop(h, &if notion { csv::infer_notion(h, &vals, true) } else { csv::infer(&vals, true) });
        // Spreadsheet cells guarded against formulas come back as they were typed.
        if p.kind == PropKind::Text {
            p.name = h.clone();
        }
        db.props.push(p);
        target[i] = Some(db.props.len() - 1);
    }
    for v in &mut db.views {
        if !v.columns.is_empty() {
            for (i, _) in target.iter().enumerate() {
                if let Some(pi) = target[i] {
                    let id = db.props[pi].id.clone();
                    if !v.columns.iter().any(|c| c.prop == id) {
                        v.columns.push(Column { prop: id, visible: true, width: 0 });
                    }
                }
            }
        }
    }
    let base_order = if is_new { 0 } else { existing_rows as i64 };
    let mut rows: Vec<Row> = Vec::with_capacity(table.rows.len());
    let mut bad_cells = 0usize;
    for (n, r) in table.rows.iter().enumerate() {
        let title: String = title_col.map(|c| csv::unsafe_cell(&r[c])).unwrap_or_default().chars().filter(|c| !c.is_control()).take(300).collect();
        let mut row = Row::new(new_id(), &db.id, title.trim().to_string(), now);
        row.order = base_order + n as i64;
        if db.props.iter().any(|p| p.kind == PropKind::Id) {
            row.number = db.next_id.max(1);
            db.next_id = row.number + 1;
        }
        let mut cells = Map::new();
        for (i, t) in target.iter().enumerate() {
            let Some(pi) = *t else { continue };
            let mut p = db.props[pi].clone();
            match convert(&mut p, &r[i]) {
                Ok(Some(v)) => {
                    cells.insert(p.id.clone(), v);
                }
                Ok(None) => {}
                Err(()) => bad_cells += 1,
            }
            // A select value may have added an option.
            if p.options.len() != db.props[pi].options.len() {
                db.props[pi].options = p.options;
            }
        }
        write_cells(&mut row, &cells)?;
        rows.push(row);
    }
    // A text column whose values were all empty stays a text property. Done.
    db.updated = now;
    db.sanitize()?;
    Ok(Built { db, rows, skipped, bad_cells })
}

/// Reads a file chosen with a dialog (at most 2 GB).
fn read_picked(path: &PathBuf) -> Result<String> {
    let meta = std::fs::metadata(path).map_err(|_| Error::invalid("That file could not be read."))?;
    if meta.len() > csv::MAX_BYTES {
        return Err(Error::invalid("That file is larger than 2 GB."));
    }
    let bytes = std::fs::read(path).map_err(|_| Error::invalid("That file could not be read."))?;
    Ok(csv::decode(&bytes))
}

/// Imports a CSV file: as a new database in a section (`database_id` empty), or into an existing
/// database (matching columns by name). `None` if the dialog was cancelled.
#[tauri::command(async)]
pub fn import_csv(app: AppHandle, state: State<'_, AppState>, database_id: Option<String>, section_id: Option<String>) -> Result<Option<CsvImportResult>> {
    // Check the target before asking for a file.
    state.with(|s| match (&database_id, &section_id) {
        (Some(id), _) => dbc::database(s, id).map(|_| ()),
        (None, Some(sec)) if matches!(s.get(sec), Some(Item::Section(_))) => Ok(()),
        _ => Err(Error::NotFound),
    })?;
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Import a CSV file")
        .add_filter("CSV or TSV", &["csv", "tsv", "txt"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let text = read_picked(&path)?;
    let table = csv::parse(&text).map_err(Error::invalid)?;
    let stem: String = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default().chars().filter(|c| !c.is_control()).take(MAX_NAME).collect();
    state.with_mut(|s| {
        let now = now_ms();
        let (db, existing, first) = match &database_id {
            Some(id) => {
                let db = dbc::database(s, id)?;
                let n = s.items().filter(|i| matches!(i, Item::Row(r) if r.db == db.id)).count();
                (db, n, false)
            }
            None => {
                let first = first_database_backup(&state, s)?;
                let order = s.items().filter_map(|i| if let Item::Database(d) = i { Some(d.order) } else { None }).max().map_or(0, |m| m + 1);
                let mut db = Database::new(new_id(), section_id.clone().unwrap_or_default(), if stem.trim().is_empty() { "Imported table".into() } else { stem.clone() }, now);
                db.order = order;
                (db, 0, first)
            }
        };
        let columns = table.header.len();
        let mut built = build_from_table(db, &table, existing, now)?;
        let summary = CsvImportResult {
            database_id: built.db.id.clone(),
            created: database_id.is_none(),
            title: built.db.title.clone(),
            rows: built.rows.len(),
            columns,
            skipped: std::mem::take(&mut built.skipped),
            bad_cells: built.bad_cells,
            notice: if first { Some(FIRST_DATABASE_NOTICE.into()) } else { None },
        };
        let mut items: Vec<Item> = built.rows.into_iter().map(Item::Row).collect();
        items.push(Item::Database(built.db));
        s.put_many(items)?;
        Ok(Some(summary))
    })
}

// ---------------------------------------------------------------- CSV export

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvExported {
    pub path: String,
    pub file_name: String,
    pub rows: usize,
    pub columns: usize,
}

fn cell_text(v: &Val, p: &Property) -> (String, bool) {
    match v {
        Val::Null => (String::new(), false),
        Val::Bool(b) => ((if *b { "Yes" } else { "No" }).into(), false),
        Val::Num(n) => (value::num_str(*n), true),
        Val::Str(s) => (s.clone(), false),
        Val::Date(d) => {
            let a = value::format_iso(d.start, d.has_time);
            match d.end {
                Some(e) => (format!("{a} → {}", value::format_iso(e, d.has_time)), false),
                None => (a, false),
            }
        }
        Val::List(l) => (l.iter().map(|x| cell_text(x, p).0).collect::<Vec<_>>().join(", "), false),
    }
}

/// The rows of a view (or all rows) as CSV lines. Used by the command and by tests.
pub fn export_lines(s: &crate::vault::Session, database_id: &str, view_id: Option<&str>, spreadsheet: bool) -> Result<Vec<Vec<String>>> {
    let db = dbc::database(s, database_id)?;
    let ctx = ctx_of(s);
    let dbref = ctx.dbs.get(db.id.as_str()).copied().unwrap_or(&db);
    let view = view_id.and_then(|v| db.view(v));
    let cols: Vec<&Property> = match view {
        Some(v) if !v.columns.is_empty() => {
            let mut out: Vec<&Property> = vec![&db.props[0]];
            for c in &v.columns {
                if c.visible {
                    if let Some(p) = db.prop(&c.prop) {
                        if p.kind != PropKind::Title {
                            out.push(p);
                        }
                    }
                }
            }
            for p in &db.props {
                if !v.columns.iter().any(|c| c.prop == p.id) && !out.iter().any(|q| q.id == p.id) {
                    out.push(p);
                }
            }
            out
        }
        _ => db.props.iter().collect(),
    };
    // A button holds no value: it is left out of an export.
    let cols: Vec<&Property> = cols.into_iter().filter(|p| p.kind != PropKind::Button).collect();
    let rows: Vec<&Row> = ctx.rows_of(&db.id).iter().copied().filter(|r| view.map_or(true, |v| v.filter.is_empty() || ctx.matches(dbref, r, &v.filter))).collect();
    let rows = match view {
        Some(v) => ctx.sorted(dbref, rows, &v.sorts),
        None => rows,
    };
    let guard = |t: &str, num: bool| if spreadsheet { csv::safe_cell(t, num) } else { t.to_string() };
    let mut lines: Vec<Vec<String>> = vec![cols.iter().map(|p| guard(&p.name, false)).collect()];
    for r in rows {
        let mut line = Vec::with_capacity(cols.len());
        for p in &cols {
            let (text, num) = match p.kind {
                PropKind::Title => (r.title.clone(), false),
                PropKind::Files => {
                    let cells = parse_cells(r);
                    let names: Vec<String> = cells.get(&p.id).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|f| f.get("name").and_then(|n| n.as_str())).map(String::from).collect()).unwrap_or_default();
                    (names.join(", "), false)
                }
                _ => match ctx.value(dbref, r, p) {
                    Ok(v) => cell_text(&v, p),
                    Err(_) => (String::new(), false),
                },
            };
            line.push(guard(&text, num));
        }
        lines.push(line);
    }
    Ok(lines)
}

/// The rows of a database (or one view) as a document with a heading and one table, for the
/// HTML, Markdown, PDF and Word writers.
pub fn table_doc(lines: Vec<Vec<String>>, title: &str) -> crate::export::ExportDoc {
    use crate::export::doc::{Align, Block, Cell, Inline, Marks};
    use crate::export::{ExportDoc, Part};
    let cell = |text: &str, header: bool| Cell {
        header,
        colspan: 1,
        rowspan: 1,
        widths: Vec::new(),
        blocks: vec![Block::Paragraph { align: Align::Left, content: vec![Inline::Text(text.to_string(), Marks { bold: header, ..Marks::default() })] }],
    };
    let table: Vec<Vec<Cell>> = lines.iter().enumerate().map(|(i, l)| l.iter().map(|t| cell(t, i == 0)).collect()).collect();
    ExportDoc {
        title: title.to_string(),
        parts: vec![Part::Page { level: 1, title: title.to_string(), blocks: vec![Block::Table(table)] }],
        images: Default::default(),
        missing_images: 0,
    }
}

/// Exports a database (or one view of it) as HTML, Markdown, PDF or Word: its title and one table.
/// `None` if the save dialog was cancelled.
#[tauri::command(async)]
pub fn export_database(app: AppHandle, state: State<'_, AppState>, database_id: String, view_id: Option<String>, format: String) -> Result<Option<crate::export::ExportSummary>> {
    use crate::export::{self, Format};
    let format = Format::parse(&format)?;
    let (lines, title) = state.with(|s| Ok((export_lines(s, &database_id, view_id.as_deref(), false)?, dbc::database(s, &database_id)?.title.clone())))?;
    if lines.len() > 20_001 {
        return Err(Error::invalid("This view has too many rows for a document. Export it as CSV, or add a filter."));
    }
    let safe: String = title.chars().map(|c| if c.is_control() || "\\/:*?\"<>|".contains(c) { '_' } else { c }).collect();
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Export")
        .set_file_name(format!("{}.{}", if safe.trim().is_empty() { "Database" } else { safe.trim() }, format.ext()))
        .add_filter(format.label(), &[format.ext()])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let dest = export::checked_destination(&chosen, format.ext(), &state.paths.data)?;
    export::write(&dest, format, &table_doc(lines, &title), &state.paths.data).map(Some)
}

/// Exports a database (or one of its views, with its filter, sort and visible properties) as CSV.
#[tauri::command(async)]
pub fn export_csv(app: AppHandle, state: State<'_, AppState>, database_id: String, view_id: Option<String>) -> Result<Option<CsvExported>> {
    let (lines, title) = state.with(|s| Ok((export_lines(s, &database_id, view_id.as_deref(), true)?, dbc::database(s, &database_id)?.title.clone())))?;
    let safe: String = title.chars().map(|c| if c.is_control() || "\\/:*?\"<>|".contains(c) { '_' } else { c }).collect();
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Export as CSV")
        .set_file_name(format!("{}.csv", if safe.trim().is_empty() { "Database" } else { safe.trim() }))
        .add_filter("CSV", &["csv"])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let dest = checked_destination(&chosen, "csv", &state.paths.data)?;
    let columns = lines.first().map_or(0, |l| l.len());
    std::fs::write(&dest, csv::write(&lines)).map_err(|e| Error::Other(format!("The file could not be saved: {e}")))?;
    Ok(Some(CsvExported { path: dest.display().to_string(), file_name: dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), rows: lines.len().saturating_sub(1), columns }))
}

// ---------------------------------------------------------------- files in rows

fn clean_name(raw: &str) -> String {
    let n: String = raw.chars().filter(|c| !c.is_control() && !"\\/:*?\"<>|".contains(*c)).collect();
    let n = n.trim().trim_matches('.').trim().to_string();
    let n: String = n.chars().take(200).collect();
    if n.is_empty() {
        "file".into()
    } else {
        n
    }
}

const MAX_FILES_PER_CELL: usize = 100;

fn header(request: &Request<'_>, name: &str) -> String {
    request.headers().get(name).and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}

/// The row and its Files property, checked.
fn files_target(s: &crate::vault::Session, row_id: &str, prop_id: &str) -> Result<Row> {
    let r = dbc::row(s, row_id)?;
    let db = dbc::database(s, &r.db)?;
    match db.prop(prop_id) {
        Some(p) if p.kind == PropKind::Files => Ok(r),
        _ => Err(Error::NotFound),
    }
}

fn file_list(r: &Row, prop_id: &str) -> Vec<Value> {
    parse_cells(r).get(prop_id).and_then(|v| v.as_array()).cloned().unwrap_or_default()
}

fn store_files(r: &mut Row, prop_id: &str, files: Vec<Value>) -> Result<()> {
    let mut m = parse_cells(r);
    if files.is_empty() {
        m.remove(prop_id);
    } else {
        m.insert(prop_id.to_string(), Value::Array(files));
    }
    write_cells(r, &m)?;
    r.updated = now_ms();
    Ok(())
}

/// Stores a file (raw bytes) in the row's Files property. The row, the property and the name
/// (percent-encoded) travel in the headers `x-row-id`, `x-prop-id` and `x-file-name`. Pictures
/// keep their picture type (found from the bytes); everything else is stored as plain data, so
/// it can never be shown as a web page. Returns the property's files.
#[tauri::command(async)]
pub fn attach_file(state: State<'_, AppState>, request: Request<'_>) -> Result<Vec<Value>> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(Error::invalid("Expected file data."));
    };
    let row_id = header(&request, "x-row-id");
    let prop_id = header(&request, "x-prop-id");
    let name = clean_name(&percent_decode(&header(&request, "x-file-name")));
    state.with_mut(|s| attach_to_row(s, &row_id, &prop_id, &name, bytes))
}

pub fn attach_to_row(s: &mut crate::vault::Session, row_id: &str, prop_id: &str, name: &str, bytes: &[u8]) -> Result<Vec<Value>> {
    let mime = super::notes::sniff_image(bytes).unwrap_or("application/octet-stream");
    let mut r = files_target(s, row_id, prop_id)?;
    let mut list = file_list(&r, prop_id);
    if list.len() >= MAX_FILES_PER_CELL {
        return Err(Error::invalid("This property already holds the most files it can."));
    }
    let id = s.put_file(mime, bytes)?;
    list.push(json!({"id": id, "name": name, "mime": mime, "size": bytes.len()}));
    store_files(&mut r, prop_id, list.clone())?;
    if let Err(e) = s.put(Item::Row(r)) {
        let _ = s.collect_garbage();
        return Err(e);
    }
    Ok(list)
}

/// Takes a file out of the row's Files property; the stored copy is removed with it.
#[tauri::command(async)]
pub fn remove_file(state: State<'_, AppState>, row_id: String, prop_id: String, file_id: String) -> Result<Vec<Value>> {
    state.with_mut(|s| {
        let mut r = files_target(s, &row_id, &prop_id)?;
        let list: Vec<Value> = file_list(&r, &prop_id).into_iter().filter(|f| f.get("id").and_then(|i| i.as_str()) != Some(file_id.as_str())).collect();
        store_files(&mut r, &prop_id, list.clone())?;
        s.put(Item::Row(r))?;
        s.collect_garbage()?;
        Ok(list)
    })
}

/// The stored name of a file in a row, and its bytes.
fn file_of(state: &AppState, row_id: &str, prop_id: &str, file_id: &str) -> Result<(String, Vec<u8>)> {
    if file_id.len() != 32 || !file_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::NotFound);
    }
    state.with(|s| {
        let r = files_target(s, row_id, prop_id)?;
        let name = file_list(&r, prop_id)
            .iter()
            .find(|f| f.get("id").and_then(|i| i.as_str()) == Some(file_id))
            .and_then(|f| f.get("name").and_then(|n| n.as_str()).map(clean_name))
            .ok_or(Error::NotFound)?;
        let (_, bytes) = s.get_attachment(file_id)?;
        Ok((name, bytes))
    })
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() && s.is_char_boundary(i + 1) && s.is_char_boundary(i + 3) {
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

/// Types that open in another program without running code. Anything else can only be saved.
const OPENABLE: [&str; 29] = [
    "pdf", "txt", "md", "csv", "png", "jpg", "jpeg", "gif", "webp", "bmp", "docx", "xlsx", "pptx", "odt", "ods", "odp", "mp3", "wav", "ogg", "m4a", "mp4", "mov", "webm", "mkv", "avi", "flac", "epub", "json", "tsv",
];

fn extension(name: &str) -> String {
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

pub fn can_open(name: &str) -> bool {
    OPENABLE.contains(&extension(name).as_str())
}

/// Asks where to save a file of a row and writes it there. `None` if cancelled.
#[tauri::command(async)]
pub fn save_file_as(app: AppHandle, state: State<'_, AppState>, row_id: String, prop_id: String, file_id: String) -> Result<Option<String>> {
    let (name, bytes) = file_of(&state, &row_id, &prop_id, &file_id)?;
    let ext = extension(&name);
    let mut dlg = app.dialog().file().set_title("Save file").set_file_name(&name);
    if !ext.is_empty() {
        dlg = dlg.add_filter(ext.to_uppercase(), &[ext.as_str()]);
    }
    let Some(chosen) = dlg.blocking_save_file().and_then(|f| f.into_path().ok()) else {
        return Ok(None);
    };
    let dest = if ext.is_empty() { chosen.clone() } else { checked_destination(&chosen, &ext, &state.paths.data)? };
    if crate::filenames::path_has_vault_ext(&dest) {
        return Err(Error::invalid("Choose another file name."));
    }
    std::fs::write(&dest, bytes).map_err(|e| Error::Other(format!("The file could not be saved: {e}")))?;
    Ok(Some(dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()))
}

/// Opens a file of a row in the program Windows chooses for it. A plain copy is written to a new
/// folder in the system's temp folder and removed again after ten minutes (and at the next start).
/// Only types that cannot run code are opened this way; the rest must be saved first.
#[tauri::command(async)]
pub fn open_file(app: AppHandle, state: State<'_, AppState>, row_id: String, prop_id: String, file_id: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let (name, bytes) = file_of(&state, &row_id, &prop_id, &file_id)?;
    if !can_open(&name) {
        return Err(Error::invalid("This kind of file is not opened from Notaya, because it could run code. Save it first, then open it yourself."));
    }
    let dir = open_dir().join(format!("{}-{}", OPEN_PREFIX, new_id()));
    std::fs::create_dir_all(&dir).map_err(|e| Error::Other(e.to_string()))?;
    let path = dir.join(&name);
    std::fs::write(&path, bytes).map_err(|e| Error::Other(e.to_string()))?;
    let cleanup = dir.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(600));
        let _ = std::fs::remove_dir_all(cleanup);
    });
    app.opener().open_path(path.to_string_lossy(), None::<&str>).map_err(|e| Error::Other(e.to_string()))
}

const OPEN_PREFIX: &str = "notaya-open";

fn open_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Removes plain copies left behind by an earlier run (called once at start).
pub fn clean_open_copies() {
    if let Ok(rd) = std::fs::read_dir(open_dir()) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(OPEN_PREFIX) && e.path().is_dir() {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn table(csv_text: &str) -> csv::Table {
        csv::parse(csv_text).unwrap()
    }

    #[test]
    fn a_new_database_gets_guessed_kinds_and_values() {
        let t = table("Name,Qty,Done,Due,Status,Tags\nA,1,Yes,2026-01-05,Open,\"x, y\"\nB,2.5,No,\"March 3, 2026\",Done,x\nC,,Yes,,Open,y\nD,4,No,2026-02-01,Done,\"y, x\"\n");
        let db = Database::new("d".into(), "sec".into(), "T".into(), 1);
        let b = build_from_table(db, &t, 0, 5).unwrap();
        let kinds: Vec<PropKind> = b.db.props.iter().map(|p| p.kind).collect();
        assert_eq!(kinds, vec![PropKind::Title, PropKind::Number, PropKind::Checkbox, PropKind::Date, PropKind::Select, PropKind::MultiSelect]);
        assert_eq!(b.rows.len(), 4);
        assert_eq!(b.rows[0].title, "A");
        let c = parse_cells(&b.rows[1]);
        let qty = &b.db.props[1].id;
        assert_eq!(c.get(qty).unwrap().as_f64(), Some(2.5));
        assert_eq!(c.get(&b.db.props[3].id).unwrap()["start"], "2026-03-03");
        assert!(parse_cells(&b.rows[2]).get(qty).is_none(), "an empty cell stays empty");
        assert_eq!(b.bad_cells, 0);
    }

    /// The owner's "To Do List DB" in Notion (6 rows; its "Done" view, filtered on Done checked, shows 1).
    /// Shaped like a Notion CSV export: dates as "July 1, 2025", checkboxes as Yes / No.
    #[test]
    fn the_owners_notion_todo_list_imports_and_filters_like_notion() {
        use crate::database::engine::Ctx;
        let t = table("Name,Done,Due Date\nCheck the box to mark items as done,Yes,\"July 1, 2025\"\nClick the due date below to change it,No,\"July 1, 2025\"\nClick this text to see more details,No,\"July 1, 2025\"\nClick the blue + button to add a task,No,\"July 1, 2025\"\nSee finished items in the \"Done\" view,No,\"July 1, 2025\"\nClick me to learn to hide checked items,No,\"July 1, 2025\"\n");
        let db = Database::new("d".into(), "sec".into(), "To Do List DB".into(), 1);
        let b = build_from_table(db, &t, 0, 5).unwrap();
        let kinds: Vec<PropKind> = b.db.props.iter().map(|p| p.kind).collect();
        assert_eq!(kinds, vec![PropKind::Title, PropKind::Checkbox, PropKind::Date]);
        assert_eq!(b.rows.len(), 6);
        let done_id = b.db.props[1].id.clone();
        let mut view = View { id: "done".into(), name: "Done".into(), ..View::default() };
        view.filter.conds.push(Cond { prop: done_id, op: "is".into(), value: "true".into(), ..Cond::default() });
        let items: Vec<Item> = std::iter::once(Item::Database(b.db.clone())).chain(b.rows.iter().cloned().map(Item::Row)).collect();
        let ctx = Ctx::new(items.iter(), value::from_parts(2026, 11, 3, 12, 0), 0);
        let out = ctx.run_view(&b.db, &view);
        assert_eq!(out.total, 6);
        assert_eq!(out.rows.len(), 1, "Notion's Done view shows one row");
        assert_eq!(out.rows[0].title, "Check the box to mark items as done");
        let all = ctx.run_view(&b.db, &View::default());
        assert_eq!(all.rows.len(), 6, "Notion's To Do view shows all six");
    }

    #[test]
    fn existing_columns_are_matched_by_name_and_bad_cells_counted() {
        let mut db = Database::new("d".into(), "sec".into(), "T".into(), 1);
        let mut qty = Property { id: "q".into(), name: "Qty".into(), kind: PropKind::Number, ..Property::default() };
        qty.number_format = "number".into();
        let mut f = Property { id: "f".into(), name: "Total".into(), kind: PropKind::Formula, formula: "1".into(), ..Property::default() };
        f.name = "Total".into();
        db.props.extend([qty, f]);
        let t = table("Name,qty,Total,Extra\nA,5,9,hello\nB,oops,9,\n");
        let b = build_from_table(db, &t, 3, 5).unwrap();
        assert_eq!(b.rows.len(), 2);
        assert_eq!(b.bad_cells, 1);
        assert_eq!(b.skipped.len(), 1, "{:?}", b.skipped);
        assert!(b.db.props.iter().any(|p| p.name == "Extra" && p.kind == PropKind::Text));
        assert_eq!(b.rows[0].order, 3, "new rows go after the existing ones");
        assert_eq!(parse_cells(&b.rows[0]).get("q").unwrap().as_f64(), Some(5.0));
    }

    #[test]
    fn select_options_grow_and_formula_guards_come_back() {
        let mut db = Database::new("d".into(), "sec".into(), "T".into(), 1);
        db.props.push(Property { id: "s".into(), name: "Stage".into(), kind: PropKind::Select, options: vec![SelectOption { id: "o1".into(), name: "Open".into(), ..SelectOption::default() }], ..Property::default() });
        let t = table("Name,Stage\n'=1+1,open\nB,Closed\n");
        let b = build_from_table(db, &t, 0, 5).unwrap();
        assert_eq!(b.rows[0].title, "=1+1");
        assert_eq!(parse_cells(&b.rows[0]).get("s").unwrap(), "o1");
        let stage = b.db.props.iter().find(|p| p.id == "s").unwrap();
        assert_eq!(stage.options.len(), 2);
    }

    #[test]
    fn row_limit_is_enforced() {
        let db = Database::new("d".into(), "sec".into(), "T".into(), 1);
        let t = table("Name\nA\n");
        assert!(build_from_table(db, &t, MAX_ROWS_PER_DATABASE, 5).is_err());
    }

    #[test]
    fn file_names_and_types_are_cleaned() {
        assert_eq!(clean_name("..\\..\\evil<>.exe"), "evil.exe");
        assert_eq!(clean_name("   "), "file");
        assert_eq!(percent_decode("a%20b%2Fc%E2%9C%93"), "a b/c✓");
        assert_eq!(percent_decode("100%"), "100%");
        assert!(can_open("Report.PDF"));
        assert!(!can_open("setup.exe"));
        assert!(!can_open("run.bat"));
        assert!(!can_open("page.html"));
        assert!(!can_open("vector.svg"));
        assert!(!can_open("noext"));
    }

    #[test]
    fn files_are_attached_to_a_files_property_only() {
        let dir = std::env::temp_dir().join(format!("notaya-att-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut s = crate::vault::Session::create_open(&dir.join("vault.nota")).unwrap();
        let mut db = Database::new(new_id(), "sec".into(), "F".into(), 1);
        db.props.push(Property { id: "f".into(), name: "Docs".into(), kind: PropKind::Files, ..Property::default() });
        db.props.push(Property { id: "t".into(), name: "Text".into(), kind: PropKind::Text, ..Property::default() });
        s.put(Item::Database(db.clone())).unwrap();
        let r = Row::new(new_id(), &db.id, "r".into(), 1);
        let rid = r.id.clone();
        s.put(Item::Row(r)).unwrap();
        assert!(attach_to_row(&mut s, &rid, "t", "x.pdf", b"data").is_err(), "not a Files property");
        assert!(attach_to_row(&mut s, "nope", "f", "x.pdf", b"data").is_err());
        let list = attach_to_row(&mut s, &rid, "f", "x.pdf", b"%PDF data").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["mime"], "application/octet-stream");
        let id = list[0]["id"].as_str().unwrap().to_string();
        assert_eq!(s.get_attachment(&id).unwrap().1, b"%PDF data");
        // A script saved as .html is still plain data.
        let l2 = attach_to_row(&mut s, &rid, "f", "evil.html", b"<script>alert(1)</script>").unwrap();
        assert_eq!(l2[1]["mime"], "application/octet-stream");
        assert_eq!(file_list(&dbc::row(&s, &rid).unwrap(), "f").len(), 2);
    }

    #[test]
    fn a_table_document_has_one_page_with_the_rows() {
        let d = table_doc(vec![vec!["Name".into(), "Qty".into()], vec!["a".into(), "1".into()]], "My table");
        assert_eq!(d.page_count(), 1);
        assert_eq!(d.title, "My table");
    }

    #[test]
    fn a_table_document_is_written_in_every_format() {
        use crate::export::{self, Format};
        let dir = std::env::temp_dir().join(format!("notaya-tbl-{}", new_id()));
        let data = std::env::temp_dir().join(format!("notaya-tbl-data-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        for f in [Format::Html, Format::Markdown, Format::Pdf, Format::Docx] {
            let d = table_doc(vec![vec!["الاسم".into(), "Qty".into()], vec!["أحمد".into(), "1".into()], vec!["日本語".into(), "2".into()]], "جدول Table");
            let dest = dir.join(format!("t.{}", f.ext()));
            export::write(&dest, f, &d, &data).unwrap();
            assert!(std::fs::metadata(&dest).unwrap().len() > 20, "{:?}", f.ext());
        }
        let html = std::fs::read_to_string(dir.join("t.html")).unwrap();
        assert!(html.contains("أحمد") && html.contains("<table"));
        let md = std::fs::read_to_string(dir.join("t.md")).unwrap();
        assert!(md.contains("| الاسم") || md.contains("الاسم"), "{md}");
    }

    #[test]
    fn export_text_is_safe_and_readable() {
        let (t, n) = cell_text(&Val::Num(2.5), &Property::default());
        assert_eq!((t.as_str(), n), ("2.5", true));
        assert_eq!(cell_text(&Val::Bool(true), &Property::default()).0, "Yes");
        let d = Val::Date(value::DateVal { start: value::from_parts(2026, 1, 2, 0, 0), end: Some(value::from_parts(2026, 1, 4, 0, 0)), has_time: false });
        assert_eq!(cell_text(&d, &Property::default()).0, "2026-01-02 → 2026-01-04");
    }
}
