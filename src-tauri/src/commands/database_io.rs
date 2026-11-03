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
pub struct CsvSummary {
    pub database_id: String,
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
        PropKind::Number => json!(csv::parse_number(t).or_else(|| t.trim_end_matches('%').trim().parse::<f64>().ok().filter(|n| n.is_finite())).ok_or(())?),
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

pub fn build_from_table(mut db: Database, table: &csv::Table, existing_rows: usize, now: i64) -> Result<Built> {
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
        let mut p = new_prop(h, &csv::infer(&vals, true));
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

/// Reads a file chosen with a dialog (at most 50 MB).
fn read_picked(path: &PathBuf) -> Result<String> {
    let meta = std::fs::metadata(path).map_err(|_| Error::invalid("That file could not be read."))?;
    if meta.len() > csv::MAX_BYTES {
        return Err(Error::invalid("That file is larger than 50 MB."));
    }
    let bytes = std::fs::read(path).map_err(|_| Error::invalid("That file could not be read."))?;
    Ok(csv::decode(&bytes))
}

/// Imports a CSV file: as a new database in a section (`database_id` empty), or into an existing
/// database (matching columns by name). `None` if the dialog was cancelled.
#[tauri::command(async)]
pub fn import_csv(app: AppHandle, state: State<'_, AppState>, database_id: Option<String>, section_id: Option<String>) -> Result<Option<CsvSummary>> {
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
        let summary = CsvSummary {
            database_id: built.db.id.clone(),
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
pub fn export_lines(s: &crate::vault::Session, database_id: &str, view_id: Option<&str>) -> Result<Vec<Vec<String>>> {
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
    let rows: Vec<&Row> = ctx.rows_of(&db.id).iter().copied().filter(|r| view.map_or(true, |v| v.filter.is_empty() || ctx.matches(dbref, r, &v.filter))).collect();
    let rows = match view {
        Some(v) => ctx.sorted(dbref, rows, &v.sorts),
        None => rows,
    };
    let mut lines: Vec<Vec<String>> = vec![cols.iter().map(|p| csv::safe_cell(&p.name, false)).collect()];
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
            line.push(csv::safe_cell(&text, num));
        }
        lines.push(line);
    }
    Ok(lines)
}

/// Exports a database (or one of its views, with its filter, sort and visible properties) as CSV.
#[tauri::command(async)]
pub fn export_csv(app: AppHandle, state: State<'_, AppState>, database_id: String, view_id: Option<String>) -> Result<Option<CsvExported>> {
    let (lines, title) = state.with(|s| Ok((export_lines(s, &database_id, view_id.as_deref())?, dbc::database(s, &database_id)?.title.clone())))?;
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
    Ok(Some(CsvExported { file_name: dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), rows: lines.len().saturating_sub(1), columns }))
}

// ---------------------------------------------------------------- files in rows

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRef {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: usize,
}

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

/// Stores a file (raw bytes; the name is sent in the `x-file-name` header, percent-encoded).
/// Pictures keep their picture type (found from the bytes); everything else is stored as plain
/// data, so it can never be shown as a web page.
#[tauri::command(async)]
pub fn attach_file(state: State<'_, AppState>, request: Request<'_>) -> Result<FileRef> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(Error::invalid("Expected file data."));
    };
    let raw_name = request.headers().get("x-file-name").and_then(|v| v.to_str().ok()).unwrap_or("file");
    let name = clean_name(&percent_decode(raw_name));
    let mime = super::notes::sniff_image(bytes).unwrap_or("application/octet-stream");
    // Held until the vault is locked: a picked file must survive until the row's cell names it.
    let id = state.with_mut(|s| {
        let id = s.put_file(mime, bytes)?;
        s.hold_attachment(&id);
        Ok(id)
    })?;
    Ok(FileRef { id, name, mime: mime.into(), size: bytes.len() })
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
const OPENABLE: [&str; 33] = [
    "pdf", "txt", "md", "csv", "rtf", "png", "jpg", "jpeg", "gif", "webp", "bmp", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "mp3", "wav", "ogg", "m4a", "mp4", "mov", "webm", "mkv", "avi", "flac", "epub", "json", "tsv",
];

fn extension(name: &str) -> String {
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

pub fn can_open(name: &str) -> bool {
    OPENABLE.contains(&extension(name).as_str())
}

fn file_bytes(state: &AppState, id: &str) -> Result<Vec<u8>> {
    if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::NotFound);
    }
    state.with(|s| s.get_attachment(id)).map(|(_, b)| b)
}

/// Asks where to save a file of a row and writes it there. `None` if cancelled.
#[tauri::command(async)]
pub fn save_file_as(app: AppHandle, state: State<'_, AppState>, file_id: String, name: String) -> Result<Option<String>> {
    let name = clean_name(&name);
    let bytes = file_bytes(&state, &file_id)?;
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
pub fn open_file(app: AppHandle, state: State<'_, AppState>, file_id: String, name: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let name = clean_name(&name);
    if !can_open(&name) {
        return Err(Error::invalid("This kind of file is not opened from Notaya, because it could run code. Save it first, then open it yourself."));
    }
    let bytes = file_bytes(&state, &file_id)?;
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
    fn export_text_is_safe_and_readable() {
        let (t, n) = cell_text(&Val::Num(2.5), &Property::default());
        assert_eq!((t.as_str(), n), ("2.5", true));
        assert_eq!(cell_text(&Val::Bool(true), &Property::default()).0, "Yes");
        let d = Val::Date(value::DateVal { start: value::from_parts(2026, 1, 2, 0, 0), end: Some(value::from_parts(2026, 1, 4, 0, 0)), has_time: false });
        assert_eq!(cell_text(&d, &Property::default()).0, "2026-01-02 → 2026-01-04");
    }
}
