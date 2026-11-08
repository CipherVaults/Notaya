//! Commands for databases (A35). Every value from the UI is checked here; the engine in
//! `crate::database` works out what a view shows.

use crate::actions::Actor;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tauri::State;

use super::automation;
use super::limit;
use crate::backup;
use crate::database::clock;
use crate::database::engine::{Ctx, RowOut, ViewResult};
use crate::database::types::*;
use crate::database::value::{self, parse_date};
use crate::error::{Error, Result};
use crate::model::{new_id, now_ms, Item, Page};
use crate::state::AppState;
use crate::vault::Session;

pub const MAX_ROWS_PER_DATABASE: usize = 200_000;
const MAX_CELL_TEXT: usize = 100_000;

// ---------------------------------------------------------------- lookups

pub fn database(s: &Session, id: &str) -> Result<Database> {
    match s.get(id) {
        Some(Item::Database(d)) => Ok(d.clone()),
        _ => Err(Error::NotFound),
    }
}

pub fn row(s: &Session, id: &str) -> Result<Row> {
    match s.get(id) {
        Some(Item::Row(r)) => Ok(r.clone()),
        _ => Err(Error::NotFound),
    }
}

pub(crate) fn ctx_of(s: &Session) -> Ctx<'_> {
    let (now, tz) = clock::local_now();
    Ctx::new(s.items(), now, tz)
}

/// The tree entry of a database (like `PageMeta` for a page).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMeta {
    pub id: String,
    pub section_id: String,
    pub parent: String,
    pub inline: bool,
    pub title: String,
    pub icon: String,
    pub favorite: bool,
    pub locked: bool,
    pub order: i64,
    pub rows: usize,
    pub trashed_at: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

pub fn meta_list(s: &Session) -> Vec<DatabaseMeta> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for it in s.items() {
        if let Item::Row(r) = it {
            if r.trashed_at.is_none() {
                *counts.entry(r.db.as_str()).or_default() += 1;
            }
        }
    }
    let mut v: Vec<DatabaseMeta> = s
        .items()
        .filter_map(|i| match i {
            Item::Database(d) => Some(DatabaseMeta {
                id: d.id.clone(),
                section_id: d.section_id.clone(),
                parent: d.parent.clone(),
                inline: d.inline,
                title: d.title.clone(),
                icon: d.icon.clone(),
                favorite: d.favorite,
                locked: d.locked,
                order: d.order,
                rows: counts.get(d.id.as_str()).copied().unwrap_or(0),
                trashed_at: d.trashed_at,
                created: d.created,
                updated: d.updated,
            }),
            _ => None,
        })
        .collect();
    v.sort_by_key(|d| (d.order, d.created));
    v
}

/// Ids of the pages inside a row (its sub-pages at any depth).
fn row_subpages(s: &Session, row_id: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut frontier = vec![row_id.to_string()];
    while let Some(cur) = frontier.pop() {
        for it in s.items() {
            if let Item::Page(p) = it {
                if p.parent == cur && !out.contains(&p.id) {
                    out.push(p.id.clone());
                    frontier.push(p.id.clone());
                }
            }
        }
    }
    out
}

/// Pages that sit inside a database row at any depth (hidden from the sidebar; reached from the row).
pub fn pages_under_rows(s: &Session) -> HashSet<String> {
    let rows: HashSet<&str> = s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.id.as_str()) } else { None }).collect();
    if rows.is_empty() {
        return HashSet::new();
    }
    let mut under: HashSet<String> = HashSet::new();
    loop {
        let before = under.len();
        for it in s.items() {
            if let Item::Page(p) = it {
                if !under.contains(&p.id) && (rows.contains(p.parent.as_str()) || under.contains(&p.parent)) {
                    under.insert(p.id.clone());
                }
            }
        }
        if under.len() == before {
            return under;
        }
    }
}

// ---------------------------------------------------------------- cells

fn text_cell(v: &Value, max: usize) -> Option<Value> {
    match v {
        Value::String(s) => {
            let t: String = s.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\t').take(max).collect();
            if t.is_empty() {
                None
            } else {
                Some(Value::String(t))
            }
        }
        _ => None,
    }
}

/// Checks one cell value against its property and returns the cleaned value (None = empty).
/// Err for a value of the wrong shape.
pub fn clean_cell(s: &Session, db: &Database, prop: &Property, v: &Value) -> Result<Option<Value>> {
    if v.is_null() {
        return Ok(None);
    }
    let bad = || Error::invalid(format!("That value does not fit the property \"{}\".", prop.name));
    Ok(match prop.kind {
        PropKind::Title => return Err(Error::invalid("The title is set with the row's name.")),
        PropKind::Text => text_cell(v, MAX_CELL_TEXT),
        PropKind::Url | PropKind::Email | PropKind::Phone => text_cell(v, 2000),
        PropKind::Number => match v {
            Value::Number(n) => {
                let f = n.as_f64().ok_or_else(bad)?;
                if !f.is_finite() {
                    return Err(bad());
                }
                Some(json!(f))
            }
            Value::String(t) if t.trim().is_empty() => None,
            Value::String(t) => Some(json!(t.trim().parse::<f64>().ok().filter(|f| f.is_finite()).ok_or_else(bad)?)),
            _ => return Err(bad()),
        },
        PropKind::Select | PropKind::Status => match v {
            Value::String(id) if prop.option(id).is_some() => Some(v.clone()),
            Value::String(id) if id.is_empty() => None,
            _ => return Err(bad()),
        },
        PropKind::MultiSelect => match v {
            Value::Array(a) => {
                let mut out: Vec<Value> = Vec::new();
                for x in a {
                    let id = x.as_str().ok_or_else(bad)?;
                    if prop.option(id).is_none() {
                        return Err(bad());
                    }
                    if !out.contains(x) {
                        out.push(x.clone());
                    }
                }
                if out.is_empty() {
                    None
                } else {
                    Some(Value::Array(out))
                }
            }
            _ => return Err(bad()),
        },
        PropKind::Date => match v {
            Value::Object(o) => {
                let start = o.get("start").and_then(|x| x.as_str()).and_then(parse_date).ok_or_else(bad)?;
                let mut out = Map::new();
                out.insert("start".into(), json!(value::format_iso(start.start, start.has_time)));
                if let Some(e) = o.get("end").and_then(|x| x.as_str()).filter(|x| !x.is_empty()) {
                    let end = parse_date(e).ok_or_else(bad)?;
                    if end.start < start.start {
                        return Err(Error::invalid("The end date is before the start date."));
                    }
                    out.insert("end".into(), json!(value::format_iso(end.start, end.has_time || start.has_time)));
                }
                if let Some(r) = o.get("reminder").and_then(|x| x.as_i64()) {
                    if (0..=40_320).contains(&r) {
                        out.insert("reminder".into(), json!(r));
                    }
                }
                Some(Value::Object(out))
            }
            _ => return Err(bad()),
        },
        PropKind::Person => match v {
            Value::Array(a) => {
                let names: Vec<Value> = a.iter().filter_map(|x| x.as_str()).map(|t| t.trim()).filter(|t| !t.is_empty()).take(100).map(|t| json!(t.chars().take(100).collect::<String>())).collect();
                if names.is_empty() {
                    None
                } else {
                    Some(Value::Array(names))
                }
            }
            _ => return Err(bad()),
        },
        PropKind::Files => match v {
            Value::Array(a) => {
                let mut out: Vec<Value> = Vec::new();
                for x in a.iter().take(100) {
                    let id = x.get("id").and_then(|i| i.as_str()).ok_or_else(bad)?;
                    if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(bad());
                    }
                    let name: String = x.get("name").and_then(|i| i.as_str()).unwrap_or("file").chars().filter(|c| !c.is_control() && !"\\/:*?\"<>|".contains(*c)).take(200).collect();
                    let mime: String = x.get("mime").and_then(|i| i.as_str()).unwrap_or("application/octet-stream").chars().filter(|c| c.is_ascii_graphic()).take(100).collect();
                    let size = x.get("size").and_then(|i| i.as_u64()).unwrap_or(0);
                    out.push(json!({"id": id, "name": if name.is_empty() { "file".to_string() } else { name }, "mime": mime, "size": size}));
                }
                if out.is_empty() {
                    None
                } else {
                    Some(Value::Array(out))
                }
            }
            _ => return Err(bad()),
        },
        PropKind::Checkbox => match v {
            Value::Bool(b) => Some(json!(*b)),
            _ => return Err(bad()),
        },
        PropKind::Relation => match v {
            Value::Array(a) => {
                let mut out: Vec<Value> = Vec::new();
                for x in a {
                    let id = x.as_str().ok_or_else(bad)?;
                    match s.get(id) {
                        Some(Item::Row(r)) if r.db == prop.relation_db => {}
                        _ => return Err(Error::invalid("A linked row no longer exists.")),
                    }
                    if !out.contains(x) {
                        out.push(x.clone());
                    }
                }
                if prop.relation_limit_one {
                    out.truncate(1);
                }
                if out.is_empty() {
                    None
                } else {
                    Some(Value::Array(out))
                }
            }
            _ => return Err(bad()),
        },
        PropKind::Place => match v {
            Value::Object(o) => {
                let address: String = o.get("address").and_then(|x| x.as_str()).unwrap_or("").chars().take(500).collect();
                let lat = o.get("lat").and_then(|x| x.as_f64()).filter(|x| (-90.0..=90.0).contains(x));
                let lng = o.get("lng").and_then(|x| x.as_f64()).filter(|x| (-180.0..=180.0).contains(x));
                if address.is_empty() && lat.is_none() && lng.is_none() {
                    None
                } else {
                    Some(json!({"address": address, "lat": lat, "lng": lng}))
                }
            }
            _ => return Err(bad()),
        },
        PropKind::Formula | PropKind::Rollup | PropKind::CreatedTime | PropKind::CreatedBy | PropKind::LastEditedTime | PropKind::LastEditedBy | PropKind::Id | PropKind::Button => {
            return Err(Error::invalid(format!("\"{}\" is worked out by Notaya and cannot be typed in.", prop.name)))
        }
    })
    .map(|o| o.filter(|_| db.prop(&prop.id).is_some()))
}

pub(crate) fn parse_cells(r: &Row) -> Map<String, Value> {
    serde_json::from_str::<Value>(&r.cells).ok().and_then(|v| if let Value::Object(m) = v { Some(m) } else { None }).unwrap_or_default()
}

pub(crate) fn write_cells(r: &mut Row, m: &Map<String, Value>) -> Result<()> {
    let text = serde_json::to_string(m).map_err(|e| Error::Other(e.to_string()))?;
    if text.len() > 20_000_000 {
        return Err(Error::invalid("This row holds too much data."));
    }
    r.cells = text;
    Ok(())
}

// ---------------------------------------------------------------- starter databases

fn opt(name: &str, color: &str, group: &str) -> SelectOption {
    SelectOption { id: new_id(), name: name.into(), color: color.into(), group: group.into() }
}

fn prop(name: &str, kind: PropKind) -> Property {
    Property { id: new_id(), name: name.into(), kind, ..Property::default() }
}

fn starter(db: &mut Database, kind: &str) {
    match kind {
        "tasks" => {
            db.props[0].name = "Task".into();
            let mut status = prop("Status", PropKind::Status);
            status.options = vec![opt("Not started", "gray", "todo"), opt("In progress", "blue", "inProgress"), opt("Done", "green", "complete")];
            let due = prop("Due", PropKind::Date);
            let mut pri = prop("Priority", PropKind::Select);
            pri.options = vec![opt("Low", "gray", ""), opt("Medium", "yellow", ""), opt("High", "red", "")];
            let done = prop("Done", PropKind::Checkbox);
            let (sid, did) = (status.id.clone(), due.id.clone());
            db.props.extend([status, due, pri, done]);
            let mut board = View { id: new_id(), name: "Board".into(), kind: ViewKind::Board, ..View::default() };
            board.group = Some(GroupBy { prop: sid.clone(), sort: "manual".into(), ..GroupBy::default() });
            let mut cal = View { id: new_id(), name: "Calendar".into(), kind: ViewKind::Calendar, ..View::default() };
            cal.date_prop = did;
            db.views.push(board);
            db.views.push(cal);
        }
        "reading" => {
            db.props[0].name = "Title".into();
            let author = prop("Author", PropKind::Text);
            let mut status = prop("Status", PropKind::Status);
            status.options = vec![opt("To read", "gray", "todo"), opt("Reading", "blue", "inProgress"), opt("Finished", "green", "complete")];
            let mut rating = prop("Rating", PropKind::Number);
            rating.number_display = "ring".into();
            rating.number_max = 5.0;
            let mut tags = prop("Tags", PropKind::MultiSelect);
            tags.options = vec![opt("Fiction", "purple", ""), opt("Non-fiction", "orange", ""), opt("Reference", "brown", "")];
            db.props.extend([author, status, rating, tags]);
            db.views.push(View { id: new_id(), name: "Gallery".into(), kind: ViewKind::Gallery, ..View::default() });
        }
        "projects" => {
            db.props[0].name = "Project".into();
            let mut status = prop("Status", PropKind::Status);
            status.options = vec![opt("Planned", "gray", "todo"), opt("Active", "blue", "inProgress"), opt("Done", "green", "complete")];
            let dates = prop("Dates", PropKind::Date);
            let did = dates.id.clone();
            let mut notes_p = prop("Notes", PropKind::Text);
            notes_p.description = String::new();
            db.props.extend([status, dates, notes_p]);
            let mut tl = View { id: new_id(), name: "Timeline".into(), kind: ViewKind::Timeline, ..View::default() };
            tl.date_prop = did;
            db.views.push(tl);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- the commands

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseFull {
    pub database: Database,
    /// A database-wide notice, for example "this database is empty because ...".
    pub notice: Option<String>,
}

/// The first database changes what an older Notaya can read, so a backup comes first.
/// Returns true when this is the first database (and the backup was made).
pub(crate) fn first_database_backup(state: &AppState, s: &Session) -> Result<bool> {
    first_database_backup_core(s, &Actor::Person, &state.paths)
}

pub(crate) fn first_database_backup_core(s: &Session, _actor: &Actor, paths: &crate::portable::Paths) -> Result<bool> {
    let first = !s.items().any(|i| matches!(i, Item::Database(_)));
    if first {
        backup::make_safety_backup(paths, s).map_err(|e| backup::safety_failure(&e))?;
    }
    Ok(first)
}

pub(crate) const FIRST_DATABASE_NOTICE: &str = "Notaya made a backup copy first. After this, an older Notaya (V1.4.0 or earlier) cannot open this vault.";

#[tauri::command]
pub fn create_database(state: State<'_, AppState>, section_id: String, parent: String, inline: bool, title: String, starter_kind: String) -> Result<DatabaseFull> {
    create_database_person(&state, section_id, parent, inline, title, starter_kind)
}

fn create_database_person(
    state: &AppState,
    section_id: String,
    parent: String,
    inline: bool,
    title: String,
    starter_kind: String,
) -> Result<DatabaseFull> {
    limit("Name", &title, MAX_NAME)?;
    state.with_mut(|s| {
        create_database_core(
            s,
            &Actor::Person,
            &state.paths,
            section_id,
            parent,
            inline,
            title,
            starter_kind,
        )
    })
}

pub(crate) fn create_database_core(
    s: &mut Session,
    actor: &Actor,
    paths: &crate::portable::Paths,
    section_id: String,
    parent: String,
    inline: bool,
    title: String,
    starter_kind: String,
) -> Result<DatabaseFull> {
    limit("Name", &title, MAX_NAME)?;
    match s.get(&section_id) {
        Some(Item::Section(_)) => {}
        _ => return Err(Error::NotFound),
    }
    if !parent.is_empty() && !matches!(s.get(&parent), Some(Item::Page(_)) | Some(Item::Row(_))) {
        return Err(Error::NotFound);
    }
    // The first database changes what an older Notaya can read, so a backup comes first.
    let first = first_database_backup_core(s, actor, paths)?;
    let now = now_ms();
    let order = s.items().filter_map(|i| if let Item::Database(d) = i { Some(d.order) } else { None }).max().map_or(0, |m| m + 1);
    let mut db = Database::new(new_id(), section_id, if title.trim().is_empty() { "Untitled database".into() } else { title }, now);
    db.parent = parent;
    db.inline = inline;
    db.order = order;
    starter(&mut db, &starter_kind);
    db.sanitize()?;
    s.put(Item::Database(db.clone()))?;
    Ok(DatabaseFull { database: db, notice: if first { Some(FIRST_DATABASE_NOTICE.into()) } else { None } })
}


#[tauri::command]
pub fn get_database(state: State<'_, AppState>, id: String) -> Result<Database> {
    get_database_person(&state, id)
}

fn get_database_person(state: &AppState, id: String) -> Result<Database> {
    state.with_mut(|s| get_database_core(s, &Actor::Person, id))
}

pub(crate) fn get_database_core(s: &mut Session, _actor: &Actor, id: String) -> Result<Database> {
    let mut db = database(s, &id)?;
    let (now, _) = clock::local_now();
    if apply_repeats(s, &mut db, now)? {
        s.put(Item::Database(db.clone()))?;
    }
    Ok(db)
}


/// Repeating templates add their row when one is due (daily, weekly or monthly), once per
/// period, when the database is opened. Returns true if the database record changed.
fn apply_repeats(s: &mut Session, db: &mut Database, now: i64) -> Result<bool> {
    let mut changed = false;
    let today = value::start_of_day(now);
    let templates = db.templates.clone();
    for (i, t) in templates.iter().enumerate() {
        let (period_start, name_date) = match t.repeat.as_str() {
            "daily" => (today, value::format_pattern(today, "MMM D, YYYY")),
            "weekly" => (value::start_of_week(today), format!("week of {}", value::format_pattern(value::start_of_week(today), "MMM D, YYYY"))),
            "monthly" => (value::start_of_month(today), value::format_pattern(value::start_of_month(today), "MMMM YYYY")),
            _ => continue,
        };
        if t.repeat_last >= period_start {
            continue;
        }
        let mut r = new_row(s, db, if t.name.is_empty() { name_date.clone() } else { format!("{} – {}", t.name, name_date) })?;
        apply_template(&mut r, t);
        s.put(Item::Row(r))?;
        db.templates[i].repeat_last = period_start;
        changed = true;
    }
    Ok(changed)
}

fn apply_template(r: &mut Row, t: &Template) {
    if !t.cells.is_empty() {
        r.cells = t.cells.clone();
    }
    r.content = t.content.clone();
    r.text = t.text.clone();
    if !t.icon.is_empty() {
        r.icon = t.icon.clone();
    }
}

/// Saves the schema, views and templates. Relation properties asked for with `relationBack: "+"`
/// get their other side made here; removed properties are cleaned out of the rows and out of
/// other databases that point at them.
#[tauri::command]
pub fn save_database(state: State<'_, AppState>, database_json: String) -> Result<Database> {
    save_database_person(&state, database_json)
}

fn save_database_person(state: &AppState, database_json: String) -> Result<Database> {
    if database_json.len() > 30_000_000 {
        return Err(Error::invalid("This database definition is too large."));
    }
    let incoming: Database =
        serde_json::from_str(&database_json).map_err(|_| Error::invalid("That database definition is not valid."))?;
    let _ = &incoming;
    state.with_mut(|s| save_database_core(s, &Actor::Person, database_json))
}

pub(crate) fn save_database_core(s: &mut Session, _actor: &Actor, database_json: String) -> Result<Database> {
    if database_json.len() > 30_000_000 {
        return Err(Error::invalid("This database definition is too large."));
    }
    let mut incoming: Database = serde_json::from_str(&database_json).map_err(|_| Error::invalid("That database definition is not valid."))?;
    let old = database(s, &incoming.id)?;
    if old.locked && incoming.locked {
        return Err(Error::invalid("This database is locked. Unlock it to change its properties."));
    }
    // Fields the UI may not change.
    incoming.created = old.created;
    incoming.section_id = old.section_id.clone();
    incoming.parent = old.parent.clone();
    incoming.inline = old.inline;
    incoming.order = old.order;
    incoming.favorite = old.favorite;
    incoming.trashed_at = old.trashed_at;
    incoming.next_id = old.next_id;
    // Date automations: Notaya keeps the time up to which dates were handled; a new one starts now.
    let now_min = clock::local_now().0;
    for a in &mut incoming.automations {
        a.last_run = old.automations.iter().find(|o| o.id == a.id).map_or(now_min, |o| o.last_run);
    }
    incoming.updated = now_ms();
    incoming.sanitize()?;
    // Property kinds cannot change in place (a value of the old kind would be wrong); the UI makes a new property.
    for p in &incoming.props {
        if let Some(op) = old.prop(&p.id) {
            if op.kind != p.kind {
                return Err(Error::invalid("A property's type cannot be changed here."));
            }
        }
    }
    // Check relations, rollups and formulas refer to things that exist.
    let mut changed_items: Vec<Item> = Vec::new();
    let mut incoming_extra: Vec<(String, Property)> = Vec::new(); // (target db id, new back property)
    for p in &mut incoming.props {
        match p.kind {
            PropKind::Relation => {
                let target_exists = p.relation_db == incoming.id || matches!(s.get(&p.relation_db), Some(Item::Database(_)));
                if !target_exists {
                    return Err(Error::invalid(format!("The relation \"{}\" needs a database to link to.", p.name)));
                }
                if p.relation_back == "+" {
                    let back = Property {
                        id: new_id(),
                        name: format!("Related to {} ({})", if incoming.title.is_empty() { "Untitled" } else { &incoming.title }, p.name),
                        kind: PropKind::Relation,
                        relation_db: incoming.id.clone(),
                        relation_back: p.id.clone(),
                        ..Property::default()
                    };
                    p.relation_back = back.id.clone();
                    incoming_extra.push((p.relation_db.clone(), back));
                }
            }
            PropKind::Formula => {
                if let Err(e) = crate::database::formula::parse(&p.formula) {
                    // A formula with a typo is kept (shown as an error in its cells) so work is not lost.
                    let _ = e;
                }
            }
            _ => {}
        }
    }
    // Back properties made in another database (or this one, for a self-relation).
    let mut self_extra: Vec<Property> = Vec::new();
    for (target, back) in incoming_extra {
        if target == incoming.id {
            self_extra.push(back);
        } else {
            let mut tdb = database(s, &target)?;
            if tdb.props.len() >= MAX_PROPERTIES {
                return Err(Error::invalid("The other database has too many properties."));
            }
            let col = Column { prop: back.id.clone(), visible: true, width: 0 };
            for v in &mut tdb.views {
                v.columns.push(col.clone());
            }
            tdb.props.push(back);
            tdb.updated = now_ms();
            changed_items.push(Item::Database(tdb));
        }
    }
    for b in self_extra {
        let col = Column { prop: b.id.clone(), visible: true, width: 0 };
        for v in &mut incoming.views {
            v.columns.push(col.clone());
        }
        incoming.props.push(b);
    }
    // Removed properties: cut them out of rows and out of properties that point at them.
    let removed: Vec<Property> = old.props.iter().filter(|p| incoming.prop(&p.id).is_none()).cloned().collect();
    let rows_here: Vec<String> = s.items().filter_map(|i| if let Item::Row(r) = i { if r.db == incoming.id { Some(r.id.clone()) } else { None } } else { None }).collect();
    if !removed.is_empty() {
        let removed_ids: HashSet<&str> = removed.iter().map(|p| p.id.as_str()).collect();
        for rid in &rows_here {
            let mut r = row(s, rid)?;
            let mut m = parse_cells(&r);
            let before = m.len();
            m.retain(|k, _| !removed_ids.contains(k.as_str()));
            if m.len() != before {
                write_cells(&mut r, &m)?;
                changed_items.push(Item::Row(r));
            }
        }
        // A removed relation also removes its other side; rollups on removed relations go too.
        for p in removed.iter().filter(|p| p.kind == PropKind::Relation && !p.relation_back.is_empty()) {
            if p.relation_db == incoming.id {
                if let Some(i) = incoming.props.iter().position(|q| q.id == p.relation_back) {
                    incoming.props.remove(i);
                }
            } else if let Ok(mut tdb) = database(s, &p.relation_db) {
                if tdb.props.iter().any(|q| q.id == p.relation_back) {
                    tdb.props.retain(|q| q.id != p.relation_back);
                    tdb.props.retain(|q| !(q.kind == PropKind::Rollup && q.rollup_relation == p.relation_back));
                    for v in &mut tdb.views {
                        v.columns.retain(|c| tdb.props.iter().any(|q| q.id == c.prop));
                    }
                    changed_items.push(Item::Database(tdb));
                }
            }
        }
        let gone: HashSet<String> = removed.iter().map(|p| p.id.clone()).collect();
        incoming.props.retain(|q| !(q.kind == PropKind::Rollup && gone.contains(&q.rollup_relation)));
        let keep: HashSet<String> = incoming.props.iter().map(|q| q.id.clone()).collect();
        for v in &mut incoming.views {
            v.columns.retain(|c| keep.contains(&c.prop));
        }
    }
    // A new ID property numbers the rows that exist, oldest first.
    let new_id_prop = incoming.props.iter().any(|p| p.kind == PropKind::Id) && !old.props.iter().any(|p| p.kind == PropKind::Id);
    if new_id_prop {
        let mut rows: Vec<Row> = rows_here.iter().filter_map(|id| row(s, id).ok()).collect();
        rows.sort_by_key(|r| (r.created, r.order));
        let mut n = incoming.next_id.max(1);
        for mut r in rows {
            if r.number == 0 {
                r.number = n;
                n += 1;
                changed_items.push(Item::Row(r));
            }
        }
        incoming.next_id = n;
    } else {
        incoming.next_id = old.next_id;
    }
    incoming.sanitize()?;
    changed_items.push(Item::Database(incoming.clone()));
    s.put_many(changed_items)?;
    Ok(incoming)
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub database_id: String,
    pub view_id: String,
    /// A view being edited and not saved yet.
    pub view: Option<View>,
}

#[tauri::command]
pub fn query_view(state: State<'_, AppState>, request: QueryRequest) -> Result<ViewResult> {
    query_view_person(&state, request)
}

fn query_view_person(state: &AppState, request: QueryRequest) -> Result<ViewResult> {
    state.with(|s| query_view_core(s, &Actor::Person, request))
}

pub(crate) fn query_view_core(s: &Session, _actor: &Actor, request: QueryRequest) -> Result<ViewResult> {
    let db = database(s, &request.database_id)?;
    let view = match request.view {
        Some(mut v) => {
            // Same limits as a saved view.
            let mut tmp = db.clone();
            if let Some(i) = tmp.views.iter().position(|x| x.id == v.id) {
                tmp.views[i] = v.clone();
            } else {
                v.id = new_id();
                tmp.views.push(v.clone());
            }
            tmp.sanitize()?;
            tmp.views.iter().find(|x| x.id == v.id).cloned().ok_or(Error::NotFound)?
        }
        None => db.view(&request.view_id).cloned().ok_or(Error::NotFound)?,
    };
    let ctx = ctx_of(s);
    let dbref = ctx.dbs.get(db.id.as_str()).copied().unwrap_or(&db);
    Ok(ctx.run_view(dbref, &view))
}


pub(crate) fn new_row(s: &Session, db: &mut Database, title: String) -> Result<Row> {
    let count = s.items().filter(|i| matches!(i, Item::Row(r) if r.db == db.id)).count();
    if count >= MAX_ROWS_PER_DATABASE {
        return Err(Error::invalid("This database has the most rows it can hold."));
    }
    let order = s.items().filter_map(|i| if let Item::Row(r) = i { if r.db == db.id { Some(r.order) } else { None } } else { None }).max().map_or(0, |m| m + 1);
    let mut r = Row::new(new_id(), &db.id, title, now_ms());
    r.order = order;
    if db.props.iter().any(|p| p.kind == PropKind::Id) {
        r.number = db.next_id.max(1);
        db.next_id = r.number + 1;
    }
    Ok(r)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowResult {
    pub row: RowOut,
}

pub(crate) fn row_result(s: &Session, id: &str) -> Result<RowResult> {
    let r = row(s, id)?;
    let db = database(s, &r.db)?;
    let ctx = ctx_of(s);
    let dbref = ctx.dbs.get(db.id.as_str()).copied().unwrap_or(&db);
    let rr = ctx.rows_of(&db.id).iter().find(|x| x.id == id).copied().unwrap_or(&r);
    Ok(RowResult { row: ctx.row_out_pub(dbref, rr) })
}

/// One row with its cell values, unchanged.
#[tauri::command]
pub fn get_row(state: State<'_, AppState>, row_id: String) -> Result<RowResult> {
    get_row_person(&state, row_id)
}

fn get_row_person(state: &AppState, row_id: String) -> Result<RowResult> {
    state.with(|s| get_row_core(s, &Actor::Person, row_id))
}

pub(crate) fn get_row_core(s: &Session, _actor: &Actor, row_id: String) -> Result<RowResult> {
    row_result(s, &row_id)
}


/// Adds a row. `cells` is a JSON object (property id -> value) checked like `set_cells`;
/// `template_id` starts from a template (or "" for the database's default template, "-" for none).
#[tauri::command]
pub fn create_row(state: State<'_, AppState>, database_id: String, title: String, cells: String, template_id: String) -> Result<RowResult> {
    create_row_person(&state, database_id, title, cells, template_id)
}

fn create_row_person(
    state: &AppState,
    database_id: String,
    title: String,
    cells: String,
    template_id: String,
) -> Result<RowResult> {
    limit("Title", &title, 300)?;
    let given = parse_given(&cells)?;
    let _ = &given;
    state.with_mut(|s| create_row_core(s, &Actor::Person, database_id, title, cells, template_id))
}

pub(crate) fn create_row_core(
    s: &mut Session,
    _actor: &Actor,
    database_id: String,
    title: String,
    cells: String,
    template_id: String,
) -> Result<RowResult> {
    limit("Title", &title, 300)?;
    let given = parse_given(&cells)?;
    let id = add_row_inner(s, &database_id, title, &given, &template_id)?;
    let mut run = automation::Run::default();
    automation::fire(s, &mut run, &id, &automation::Event::Added, 0);
    automation::finish(run);
    row_result(s, &id)
}


fn parse_given(cells: &str) -> Result<Map<String, Value>> {
    if cells.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(cells) {
        Ok(Value::Object(g)) => Ok(g),
        _ => Err(Error::invalid("Those cell values are not valid.")),
    }
}

/// Makes and saves a row (no automations run here). Returns its id.
pub(crate) fn add_row_inner(s: &mut Session, database_id: &str, title: String, given: &Map<String, Value>, template_id: &str) -> Result<String> {
    let mut db = database(s, database_id)?;
    let mut r = new_row(s, &mut db, title)?;
    let tid = match template_id {
        "-" => String::new(),
        "" => db.default_template.clone(),
        other => other.to_string(),
    };
    if let Some(t) = db.templates.iter().find(|t| t.id == tid) {
        apply_template(&mut r, t);
    }
    let mut m = parse_cells(&r);
    for (k, v) in given {
        let p = db.prop(k).ok_or(Error::NotFound)?.clone();
        match clean_cell(s, &db, &p, v)? {
            Some(c) => {
                m.insert(k.clone(), c);
            }
            None => {
                m.remove(k);
            }
        }
    }
    write_cells(&mut r, &m)?;
    let id = r.id.clone();
    let mut items = relation_sync_diff(s, &id, None, &m, &db)?;
    items.push(Item::Row(r));
    items.push(Item::Database(db));
    s.put_many(items)?;
    Ok(id)
}

/// For every relation property that has another side, makes the other rows list this row (or not)
/// to match `new` (compared with `old`). Returns the changed rows.
fn relation_sync_diff(s: &Session, row_id: &str, old: Option<&Map<String, Value>>, new: &Map<String, Value>, db: &Database) -> Result<Vec<Item>> {
    let mut out: HashMap<String, Row> = HashMap::new();
    for p in db.props.iter().filter(|p| p.kind == PropKind::Relation && !p.relation_back.is_empty()) {
        let ids = |m: &Map<String, Value>| -> Vec<String> { m.get(&p.id).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() };
        let before = old.map(&ids).unwrap_or_default();
        let after = ids(new);
        for target in before.iter().filter(|x| !after.contains(x)).chain(after.iter().filter(|x| !before.contains(x))) {
            let adding = after.contains(target);
            let mut t = match out.remove(target) {
                Some(t) => t,
                None => match s.get(target) {
                    Some(Item::Row(t)) => t.clone(),
                    _ => continue,
                },
            };
            let mut tm = parse_cells(&t);
            let mut list: Vec<String> = tm.get(&p.relation_back).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            if adding {
                if !list.contains(&row_id.to_string()) {
                    list.push(row_id.to_string());
                }
                // A "one row only" other side keeps just the newest link.
                if let Some(tdb) = match s.get(&t.db) { Some(Item::Database(d)) => Some(d), _ => None } {
                    if tdb.prop(&p.relation_back).map_or(false, |q| q.relation_limit_one) && list.len() > 1 {
                        list = vec![row_id.to_string()];
                    }
                }
            } else {
                list.retain(|x| x != row_id);
            }
            if list.is_empty() {
                tm.remove(&p.relation_back);
            } else {
                tm.insert(p.relation_back.clone(), json!(list));
            }
            write_cells(&mut t, &tm)?;
            out.insert(target.clone(), t);
        }
    }
    Ok(out.into_values().map(Item::Row).collect())
}

/// Changes cells (and the title) of one row. `patch` is a JSON object: property id -> new value (null clears).
#[tauri::command]
pub fn set_cells(state: State<'_, AppState>, row_id: String, title: Option<String>, patch: String) -> Result<RowResult> {
    set_cells_person(&state, row_id, title, patch)
}

fn set_cells_person(state: &AppState, row_id: String, title: Option<String>, patch: String) -> Result<RowResult> {
    if let Some(t) = &title {
        limit("Title", t, 300)?;
    }
    let given = parse_given(&patch)?;
    let _ = &given;
    state.with_mut(|s| set_cells_core(s, &Actor::Person, row_id, title, patch))
}

pub(crate) fn set_cells_core(
    s: &mut Session,
    _actor: &Actor,
    row_id: String,
    title: Option<String>,
    patch: String,
) -> Result<RowResult> {
    if let Some(t) = &title {
        limit("Title", t, 300)?;
    }
    let given = parse_given(&patch)?;
    let changed = change_row(s, &row_id, title, &given)?;
    if !changed.is_empty() {
        let mut run = automation::Run::default();
        automation::fire(s, &mut run, &row_id, &automation::Event::Changed(&changed), 0);
        automation::finish(run);
    }
    row_result(s, &row_id)
}


/// Changes cells (and the title) of a row and saves it (no automations run here).
/// Returns the ids of the properties whose value changed ("title" for the title).
pub(crate) fn change_row(s: &mut Session, row_id: &str, title: Option<String>, given: &Map<String, Value>) -> Result<Vec<String>> {
    let mut r = row(s, row_id)?;
    let db = database(s, &r.db)?;
    let old = parse_cells(&r);
    let mut m = old.clone();
    for (k, v) in given {
        let p = db.prop(k).ok_or(Error::NotFound)?.clone();
        match clean_cell(s, &db, &p, v)? {
            Some(c) => {
                m.insert(k.clone(), c);
            }
            None => {
                m.remove(k);
            }
        }
    }
    let mut changed: Vec<String> = given.keys().filter(|k| old.get(*k) != m.get(*k)).cloned().collect();
    write_cells(&mut r, &m)?;
    if let Some(t) = title {
        if t != r.title {
            changed.push(TITLE_PROP.to_string());
        }
        r.title = t;
    }
    r.updated = now_ms();
    let mut items = relation_sync_diff(s, row_id, Some(&old), &m, &db)?;
    items.push(Item::Row(r));
    s.put_many(items)?;
    changed.sort();
    Ok(changed)
}

/// Presses a button property: runs its actions on the row.
#[tauri::command]
pub fn press_button(state: State<'_, AppState>, row_id: String, prop_id: String) -> Result<RowResult> {
    press_button_person(&state, row_id, prop_id)
}

fn press_button_person(state: &AppState, row_id: String, prop_id: String) -> Result<RowResult> {
    state.with_mut(|s| press_button_core(s, &Actor::Person, row_id, prop_id))
}

pub(crate) fn press_button_core(s: &mut Session, _actor: &Actor, row_id: String, prop_id: String) -> Result<RowResult> {
    let r = row(s, &row_id)?;
    let db = database(s, &r.db)?;
    if db.prop(&prop_id).map_or(true, |p| p.kind != PropKind::Button) {
        return Err(Error::NotFound);
    }
    let mut run = automation::Run::default();
    automation::fire(s, &mut run, &row_id, &automation::Event::Button(&prop_id), 0);
    automation::finish(run);
    row_result(s, &row_id)
}


#[tauri::command]
pub fn duplicate_row(state: State<'_, AppState>, row_id: String) -> Result<RowResult> {
    duplicate_row_person(&state, row_id)
}

fn duplicate_row_person(state: &AppState, row_id: String) -> Result<RowResult> {
    state.with_mut(|s| duplicate_row_core(s, &Actor::Person, row_id))
}

pub(crate) fn duplicate_row_core(s: &mut Session, _actor: &Actor, row_id: String) -> Result<RowResult> {
    let src = row(s, &row_id)?;
    let mut db = database(s, &src.db)?;
    let mut r = new_row(s, &mut db, if src.title.is_empty() { String::new() } else { format!("{} (copy)", src.title) })?;
    r.content = src.content.clone();
    r.text = src.text.clone();
    r.icon = src.icon.clone();
    r.cover = src.cover.clone();
    let mut m = parse_cells(&src);
    // Computed and unique-looking things are not copied; relations are, with their other side updated.
    m.retain(|k, _| db.prop(k).map_or(false, |p| !p.kind.is_computed() && p.kind != PropKind::Id));
    write_cells(&mut r, &m)?;
    let id = r.id.clone();
    let mut items = relation_sync_diff(s, &id, None, &m, &db)?;
    items.push(Item::Row(r));
    items.push(Item::Database(db));
    s.put_many(items)?;
    let mut run = automation::Run::default();
    automation::fire(s, &mut run, &id, &automation::Event::Added, 0);
    automation::finish(run);
    row_result(s, &id)
}


/// Puts the rows in the given order (ids of rows of one database).
#[tauri::command]
pub fn reorder_rows(state: State<'_, AppState>, ids: Vec<String>) -> Result<()> {
    reorder_rows_person(&state, ids)
}

fn reorder_rows_person(state: &AppState, ids: Vec<String>) -> Result<()> {
    state.with_mut(|s| reorder_rows_core(s, &Actor::Person, ids))
}

pub(crate) fn reorder_rows_core(s: &mut Session, _actor: &Actor, ids: Vec<String>) -> Result<()> {
    let mut changed = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let mut r = row(s, id)?;
        r.order = i as i64;
        changed.push(Item::Row(r));
    }
    s.put_many(changed)
}


/// Rows of a database whose title contains `query` (for the relation picker).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowHit {
    pub id: String,
    pub title: String,
    pub icon: String,
}

#[tauri::command]
pub fn search_rows(state: State<'_, AppState>, database_id: String, query: String, limit_n: usize) -> Result<Vec<RowHit>> {
    search_rows_person(&state, database_id, query, limit_n)
}

fn search_rows_person(state: &AppState, database_id: String, query: String, limit_n: usize) -> Result<Vec<RowHit>> {
    let q = query.trim().to_lowercase();
    let _ = &q;
    state.with(|s| search_rows_core(s, &Actor::Person, database_id, query, limit_n))
}

pub(crate) fn search_rows_core(
    s: &Session,
    _actor: &Actor,
    database_id: String,
    query: String,
    limit_n: usize,
) -> Result<Vec<RowHit>> {
    let q = query.trim().to_lowercase();
    let mut v: Vec<RowHit> = s
        .items()
        .filter_map(|i| match i {
            Item::Row(r) if r.db == database_id && r.trashed_at.is_none() && (q.is_empty() || r.title.to_lowercase().contains(&q)) => Some(RowHit { id: r.id.clone(), title: r.title.clone(), icon: r.icon.clone() }),
            _ => None,
        })
        .collect();
    v.sort_by(|a, b| value::natural_cmp(&a.title, &b.title));
    v.truncate(limit_n.clamp(1, 200));
    Ok(v)
}


/// A lighter listing than `query_view`: every row's id and title (relation labels in the UI).
#[tauri::command]
pub fn row_titles(state: State<'_, AppState>, ids: Vec<String>) -> Result<Vec<RowHit>> {
    row_titles_person(&state, ids)
}

fn row_titles_person(state: &AppState, ids: Vec<String>) -> Result<Vec<RowHit>> {
    state.with(|s| row_titles_core(s, &Actor::Person, ids))
}

pub(crate) fn row_titles_core(s: &Session, _actor: &Actor, ids: Vec<String>) -> Result<Vec<RowHit>> {
    Ok(ids
        .iter()
        .filter_map(|id| match s.get(id) {
            Some(Item::Row(r)) => Some(RowHit { id: r.id.clone(), title: r.title.clone(), icon: r.icon.clone() }),
            _ => None,
        })
        .collect())
}


// ---------------------------------------------------------------- trash and delete (rows and databases)

/// Puts rows (with the pages inside them) in the trash.
pub fn trash_rows(s: &mut Session, ids: &[String]) -> Result<()> {
    let now = now_ms();
    let mut changed = Vec::new();
    for id in ids {
        let mut r = row(s, id)?;
        if r.trashed_at.is_none() {
            r.trashed_at = Some(now);
        }
        r.favorite = false;
        changed.push(Item::Row(r));
        for pid in row_subpages(s, id) {
            if let Some(Item::Page(p)) = s.get(&pid) {
                let mut p = p.clone();
                if p.trashed_at.is_none() {
                    p.trashed_at = Some(now);
                }
                changed.push(Item::Page(p));
            }
        }
    }
    s.put_many(changed)
}

pub fn restore_rows(s: &mut Session, ids: &[String]) -> Result<()> {
    let mut changed = Vec::new();
    for id in ids {
        let mut r = row(s, id)?;
        r.trashed_at = None;
        changed.push(Item::Row(r));
        for pid in row_subpages(s, id) {
            if let Some(Item::Page(p)) = s.get(&pid) {
                let mut p: Page = p.clone();
                p.trashed_at = None;
                changed.push(Item::Page(p));
            }
        }
    }
    s.put_many(changed)
}

/// Deletes rows for good, with their sub-pages; links to them in other rows are removed.
pub fn delete_rows(s: &mut Session, ids: &[String]) -> Result<()> {
    let doomed: HashSet<String> = ids.iter().cloned().collect();
    let mut remove: Vec<String> = Vec::new();
    for id in ids {
        row(s, id)?;
        remove.push(id.clone());
        remove.extend(row_subpages(s, id));
    }
    // Cut the links out of other rows.
    let relation_props: HashMap<String, Vec<String>> = s
        .items()
        .filter_map(|i| if let Item::Database(d) = i { Some((d.id.clone(), d.props.iter().filter(|p| p.kind == PropKind::Relation).map(|p| p.id.clone()).collect())) } else { None })
        .collect();
    let mut changed = Vec::new();
    for it in s.items() {
        if let Item::Row(r) = it {
            if doomed.contains(&r.id) {
                continue;
            }
            let Some(props) = relation_props.get(&r.db) else { continue };
            if props.is_empty() || !doomed.iter().any(|d| r.cells.contains(d.as_str())) {
                continue;
            }
            let mut r = r.clone();
            let mut m = parse_cells(&r);
            let mut touched = false;
            for pid in props {
                if let Some(Value::Array(a)) = m.get(pid).cloned() {
                    let kept: Vec<Value> = a.into_iter().filter(|x| x.as_str().map_or(true, |id| !doomed.contains(id))).collect();
                    touched = true;
                    if kept.is_empty() {
                        m.remove(pid);
                    } else {
                        m.insert(pid.clone(), Value::Array(kept));
                    }
                }
            }
            if touched {
                write_cells(&mut r, &m)?;
                changed.push(Item::Row(r));
            }
        }
    }
    s.put_many(changed)?;
    s.remove_many(&remove)?;
    s.collect_garbage()?;
    Ok(())
}

pub fn trash_database(s: &mut Session, id: &str) -> Result<()> {
    let mut d = database(s, id)?;
    if d.trashed_at.is_none() {
        d.trashed_at = Some(now_ms());
    }
    d.favorite = false;
    s.put(Item::Database(d))
}

pub fn restore_database(s: &mut Session, id: &str) -> Result<()> {
    let mut d = database(s, id)?;
    d.trashed_at = None;
    // If its page or section is still in the trash it returns to the top of its section.
    if !d.parent.is_empty() && matches!(s.get(&d.parent), Some(Item::Page(p)) if p.trashed_at.is_some()) {
        d.parent = String::new();
    }
    s.put(Item::Database(d))
}

/// Deletes a database and all its rows. Other databases lose the relations that pointed at it
/// (and the rollups on those relations).
pub fn delete_database(s: &mut Session, id: &str) -> Result<()> {
    database(s, id)?;
    let rows: Vec<String> = s.items().filter_map(|i| if let Item::Row(r) = i { if r.db == id { Some(r.id.clone()) } else { None } } else { None }).collect();
    let mut changed = Vec::new();
    let others: Vec<Database> = s.items().filter_map(|i| if let Item::Database(d) = i { if d.id != id { Some(d.clone()) } else { None } } else { None }).collect();
    for mut d in others {
        let dead: Vec<String> = d.props.iter().filter(|p| p.kind == PropKind::Relation && p.relation_db == id).map(|p| p.id.clone()).collect();
        if dead.is_empty() {
            continue;
        }
        d.props.retain(|p| !dead.contains(&p.id) && !(p.kind == PropKind::Rollup && dead.contains(&p.rollup_relation)));
        let keep: HashSet<String> = d.props.iter().map(|p| p.id.clone()).collect();
        for v in &mut d.views {
            v.columns.retain(|c| keep.contains(&c.prop));
        }
        let drows: Vec<String> = s.items().filter_map(|i| if let Item::Row(r) = i { if r.db == d.id { Some(r.id.clone()) } else { None } } else { None }).collect();
        for rid in drows {
            let mut r = row(s, &rid)?;
            let mut m = parse_cells(&r);
            let before = m.len();
            m.retain(|k, _| !dead.contains(k));
            if m.len() != before {
                write_cells(&mut r, &m)?;
                changed.push(Item::Row(r));
            }
        }
        changed.push(Item::Database(d));
    }
    s.put_many(changed)?;
    let mut remove = Vec::new();
    for r in &rows {
        remove.push(r.clone());
        remove.extend(row_subpages(s, r));
    }
    remove.push(id.to_string());
    s.remove_many(&remove)?;
    s.collect_garbage()?;
    Ok(())
}

#[tauri::command]
pub fn set_database_look(state: State<'_, AppState>, id: String, title: String, icon: String, favorite: bool, locked: bool) -> Result<DatabaseMeta> {
    set_database_look_person(&state, id, title, icon, favorite, locked)
}

fn set_database_look_person(
    state: &AppState,
    id: String,
    title: String,
    icon: String,
    favorite: bool,
    locked: bool,
) -> Result<DatabaseMeta> {
    limit("Name", &title, MAX_NAME)?;
    limit("Icon", &icon, 40)?;
    state.with_mut(|s| set_database_look_core(s, &Actor::Person, id, title, icon, favorite, locked))
}

pub(crate) fn set_database_look_core(
    s: &mut Session,
    _actor: &Actor,
    id: String,
    title: String,
    icon: String,
    favorite: bool,
    locked: bool,
) -> Result<DatabaseMeta> {
    limit("Name", &title, MAX_NAME)?;
    limit("Icon", &icon, 40)?;
    let mut d = database(s, &id)?;
    d.title = title;
    d.icon = icon;
    d.favorite = favorite && d.trashed_at.is_none();
    d.locked = locked;
    d.updated = now_ms();
    s.put(Item::Database(d))?;
    meta_list(s).into_iter().find(|m| m.id == id).ok_or(Error::NotFound)
}


/// Moves a database to another section (rows stay with it).
#[tauri::command]
pub fn move_database(state: State<'_, AppState>, id: String, section_id: String) -> Result<()> {
    move_database_person(&state, id, section_id)
}

fn move_database_person(state: &AppState, id: String, section_id: String) -> Result<()> {
    state.with_mut(|s| move_database_core(s, &Actor::Person, id, section_id))
}

pub(crate) fn move_database_core(s: &mut Session, _actor: &Actor, id: String, section_id: String) -> Result<()> {
    if !matches!(s.get(&section_id), Some(Item::Section(_))) {
        return Err(Error::NotFound);
    }
    let mut d = database(s, &id)?;
    d.section_id = section_id;
    d.parent = String::new();
    d.updated = now_ms();
    s.put(Item::Database(d))
}


/// Turns sub-items on: a "Parent item" and a "Sub-items" relation, both pointing at this database.
#[tauri::command]
pub fn enable_sub_items(state: State<'_, AppState>, id: String) -> Result<Database> {
    enable_sub_items_person(&state, id)
}

fn enable_sub_items_person(state: &AppState, id: String) -> Result<Database> {
    state.with_mut(|s| enable_sub_items_core(s, &Actor::Person, id))
}

pub(crate) fn enable_sub_items_core(s: &mut Session, _actor: &Actor, id: String) -> Result<Database> {
    let mut d = database(s, &id)?;
    if !d.sub_parent_prop.is_empty() {
        return Ok(d);
    }
    if d.props.len() + 2 > MAX_PROPERTIES {
        return Err(Error::invalid("This database has too many properties."));
    }
    let parent = Property { id: new_id(), name: "Parent item".into(), kind: PropKind::Relation, relation_db: d.id.clone(), relation_limit_one: true, ..Property::default() };
    let child = Property { id: new_id(), name: "Sub-items".into(), kind: PropKind::Relation, relation_db: d.id.clone(), relation_back: parent.id.clone(), ..Property::default() };
    let mut parent = parent;
    parent.relation_back = child.id.clone();
    d.sub_parent_prop = parent.id.clone();
    d.sub_child_prop = child.id.clone();
    for v in &mut d.views {
        v.columns.push(Column { prop: parent.id.clone(), visible: false, width: 0 });
        v.columns.push(Column { prop: child.id.clone(), visible: false, width: 0 });
    }
    d.props.push(parent);
    d.props.push(child);
    d.updated = now_ms();
    d.sanitize()?;
    s.put(Item::Database(d.clone()))?;
    Ok(d)
}


/// Turns dependencies on: "Blocked by" and "Blocking" relations, both pointing at this database.
#[tauri::command]
pub fn enable_dependencies(state: State<'_, AppState>, id: String) -> Result<Database> {
    enable_dependencies_person(&state, id)
}

fn enable_dependencies_person(state: &AppState, id: String) -> Result<Database> {
    state.with_mut(|s| enable_dependencies_core(s, &Actor::Person, id))
}

pub(crate) fn enable_dependencies_core(s: &mut Session, _actor: &Actor, id: String) -> Result<Database> {
    let mut d = database(s, &id)?;
    if !d.blocked_by_prop.is_empty() {
        return Ok(d);
    }
    if d.props.len() + 2 > MAX_PROPERTIES {
        return Err(Error::invalid("This database has too many properties."));
    }
    let mut by = Property { id: new_id(), name: "Blocked by".into(), kind: PropKind::Relation, relation_db: d.id.clone(), ..Property::default() };
    let mut blocking = Property { id: new_id(), name: "Blocking".into(), kind: PropKind::Relation, relation_db: d.id.clone(), ..Property::default() };
    by.relation_back = blocking.id.clone();
    blocking.relation_back = by.id.clone();
    d.blocked_by_prop = by.id.clone();
    d.blocking_prop = blocking.id.clone();
    for v in &mut d.views {
        v.columns.push(Column { prop: by.id.clone(), visible: false, width: 0 });
        v.columns.push(Column { prop: blocking.id.clone(), visible: false, width: 0 });
    }
    d.props.push(by);
    d.props.push(blocking);
    d.updated = now_ms();
    d.sanitize()?;
    s.put(Item::Database(d.clone()))?;
    Ok(d)
}


#[tauri::command]
pub fn chart_data(state: State<'_, AppState>, request: QueryRequest) -> Result<crate::database::chart::ChartData> {
    chart_data_person(&state, request)
}

fn chart_data_person(state: &AppState, request: QueryRequest) -> Result<crate::database::chart::ChartData> {
    state.with(|s| chart_data_core(s, &Actor::Person, request))
}

pub(crate) fn chart_data_core(
    s: &Session,
    _actor: &Actor,
    request: QueryRequest,
) -> Result<crate::database::chart::ChartData> {
    let db = database(s, &request.database_id)?;
    let view = match request.view {
        Some(v) => v,
        None => db.view(&request.view_id).cloned().ok_or(Error::NotFound)?,
    };
    // Same limits as a saved view.
    let mut tmp = db.clone();
    if let Some(i) = tmp.views.iter().position(|x| x.id == view.id) {
        tmp.views[i] = view.clone();
    } else {
        tmp.views.push(view.clone());
    }
    tmp.sanitize()?;
    let view = tmp.views.iter().find(|x| x.id == view.id).cloned().ok_or(Error::NotFound)?;
    let ctx = ctx_of(s);
    let dbref = ctx.dbs.get(db.id.as_str()).copied().unwrap_or(&db);
    Ok(crate::database::chart::chart_data(&ctx, dbref, &view))
}


// ---------------------------------------------------------------- reminders

/// A date with a reminder that has come due. Only the row's title is sent (no cell values).
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub key: String,
    pub row_id: String,
    pub title: String,
    pub database_id: String,
    pub database_title: String,
    /// When the date starts, as text. Empty for a message from an automation.
    pub when: String,
    /// An automation's message: the name of the automation or button ("" for a date reminder).
    pub message: String,
}

/// Reminders that came due in the last 24 hours (a date without a time counts as 09:00).
/// Pure: `shown` is the keys already reported.
pub fn due_reminders(ctx: &Ctx, shown: &HashSet<String>) -> Vec<Reminder> {
    let mut out = Vec::new();
    for (dbid, db) in &ctx.dbs {
        let date_props: Vec<&Property> = db.props.iter().filter(|p| p.kind == PropKind::Date).collect();
        if date_props.is_empty() {
            continue;
        }
        for r in ctx.rows_of(dbid) {
            let cells = ctx.cells_of(r);
            for p in &date_props {
                let Some(Value::Object(o)) = cells.get(&p.id) else { continue };
                let Some(mins) = o.get("reminder").and_then(|x| x.as_i64()) else { continue };
                let Some(start) = o.get("start").and_then(|x| x.as_str()).and_then(parse_date) else { continue };
                let at = if start.has_time { start.start } else { start.start + 9 * 60 };
                let fire = at - mins;
                let key = format!("{}:{}:{}", r.id, p.id, fire);
                if fire <= ctx.now && fire > ctx.now - 24 * 60 && !shown.contains(&key) {
                    out.push(Reminder {
                        key,
                        row_id: r.id.clone(),
                        title: if r.title.is_empty() { "Untitled".into() } else { r.title.clone() },
                        database_id: db.id.clone(),
                        database_title: db.title.clone(),
                        when: value::format_iso(start.start, start.has_time),
                        message: String::new(),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.when.cmp(&b.when).then(a.key.cmp(&b.key)));
    out.truncate(20);
    out
}

static SHOWN_REMINDERS: std::sync::Mutex<Option<HashSet<String>>> = std::sync::Mutex::new(None);

/// The reminders that are due now and were not reported before in this run, and the messages of
/// automations. The UI shows each one inside Notaya (title only); turning reminders off is a setting
/// of the UI. Date automations whose time has come run first.
#[tauri::command]
pub fn due_reminders_now(state: State<'_, AppState>) -> Result<Vec<Reminder>> {
    due_reminders_now_person(&state)
}

fn due_reminders_now_person(state: &AppState) -> Result<Vec<Reminder>> {
    state.with_mut(|s| due_reminders_now_core(s, &Actor::Person))
}

pub(crate) fn due_reminders_now_core(s: &mut Session, _actor: &Actor) -> Result<Vec<Reminder>> {
    automation::tick_dates(s);
    let mut guard = SHOWN_REMINDERS.lock().unwrap_or_else(|e| e.into_inner());
    let shown = guard.get_or_insert_with(HashSet::new);
    let mut due = due_reminders(&ctx_of(s), shown);
    for d in &due {
        shown.insert(d.key.clone());
    }
    if shown.len() > 5000 {
        shown.clear();
    }
    let mut out = automation::take_notices();
    out.append(&mut due);
    Ok(out)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::database_io::export_lines;

    fn temp_vault() -> Session {
        let dir = std::env::temp_dir().join(format!("notaya-dbtest-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        Session::create_open(&dir.join("vault.nota")).unwrap()
    }

    fn make_db(s: &mut Session, title: &str) -> Database {
        let mut db = Database::new(new_id(), "sec".into(), title.into(), 1);
        db.props.push(Property { id: "n".into(), name: "Qty".into(), kind: PropKind::Number, ..Property::default() });
        db.props.push(Property { id: "f".into(), name: "Docs".into(), kind: PropKind::Files, ..Property::default() });
        db.sanitize().unwrap();
        s.put(Item::Database(db.clone())).unwrap();
        db
    }

    fn add_row(s: &mut Session, db: &mut Database, title: &str, cells: &str) -> String {
        let mut r = new_row(s, db, title.into()).unwrap();
        r.cells = cells.into();
        let id = r.id.clone();
        s.put(Item::Row(r)).unwrap();
        s.put(Item::Database(db.clone())).unwrap();
        id
    }

    #[test]
    fn deleting_rows_keeps_files_other_rows_use_and_removes_the_rest() {
        let mut s = temp_vault();
        let mut db = make_db(&mut s, "Files");
        let shared = s.put_file("application/octet-stream", b"shared").unwrap();
        let only = s.put_file("application/octet-stream", b"only").unwrap();
        let a = add_row(&mut s, &mut db, "A", &format!("{{\"f\":[{{\"id\":\"{shared}\",\"name\":\"s.pdf\"}},{{\"id\":\"{only}\",\"name\":\"o.pdf\"}}]}}"));
        let _b = add_row(&mut s, &mut db, "B", &format!("{{\"f\":[{{\"id\":\"{shared}\",\"name\":\"s.pdf\"}}]}}"));
        delete_rows(&mut s, &[a.clone()]).unwrap();
        assert!(s.get(&a).is_none());
        assert!(s.get_attachment(&shared).is_ok(), "row B still uses it");
        assert!(s.get_attachment(&only).is_err(), "nothing uses it any more");
    }

    #[test]
    fn rows_go_to_the_trash_and_come_back() {
        let mut s = temp_vault();
        let mut db = make_db(&mut s, "T");
        let a = add_row(&mut s, &mut db, "A", "{}");
        trash_rows(&mut s, &[a.clone()]).unwrap();
        assert!(row(&s, &a).unwrap().trashed_at.is_some());
        restore_rows(&mut s, &[a.clone()]).unwrap();
        assert!(row(&s, &a).unwrap().trashed_at.is_none());
        let count = meta_list(&s).into_iter().find(|m| m.id == db.id).unwrap().rows;
        assert_eq!(count, 1);
    }

    #[test]
    fn two_way_relations_follow_each_change() {
        let mut s = temp_vault();
        let mut a = make_db(&mut s, "A");
        let mut b = make_db(&mut s, "B");
        a.props.push(Property { id: "rel".into(), name: "To B".into(), kind: PropKind::Relation, relation_db: b.id.clone(), relation_back: "back".into(), ..Property::default() });
        b.props.push(Property { id: "back".into(), name: "To A".into(), kind: PropKind::Relation, relation_db: a.id.clone(), relation_back: "rel".into(), ..Property::default() });
        s.put(Item::Database(a.clone())).unwrap();
        s.put(Item::Database(b.clone())).unwrap();
        let ra = add_row(&mut s, &mut a, "ra", "{}");
        let rb = add_row(&mut s, &mut b, "rb", "{}");
        let mut new = Map::new();
        new.insert("rel".into(), json!([rb.clone()]));
        let changed = relation_sync_diff(&s, &ra, Some(&Map::new()), &new, &a).unwrap();
        assert_eq!(changed.len(), 1);
        s.put_many(changed).unwrap();
        assert_eq!(parse_cells(&row(&s, &rb).unwrap()).get("back").unwrap(), &json!([ra.clone()]));
        // Removing the link removes the other side.
        let changed = relation_sync_diff(&s, &ra, Some(&new), &Map::new(), &a).unwrap();
        s.put_many(changed).unwrap();
        assert!(parse_cells(&row(&s, &rb).unwrap()).get("back").is_none());
    }

    #[test]
    fn deleting_a_database_cleans_up_relations_to_it() {
        let mut s = temp_vault();
        let mut a = make_db(&mut s, "A");
        let b = make_db(&mut s, "B");
        a.props.push(Property { id: "rel".into(), name: "To B".into(), kind: PropKind::Relation, relation_db: b.id.clone(), ..Property::default() });
        a.props.push(Property { id: "roll".into(), name: "Sum".into(), kind: PropKind::Rollup, rollup_relation: "rel".into(), rollup_prop: "n".into(), rollup_calc: "sum".into(), ..Property::default() });
        s.put(Item::Database(a.clone())).unwrap();
        let ra = add_row(&mut s, &mut a, "ra", "{\"rel\":[\"x\"],\"n\":3}");
        delete_database(&mut s, &b.id).unwrap();
        assert!(s.get(&b.id).is_none());
        let a2 = database(&s, &a.id).unwrap();
        assert!(a2.prop("rel").is_none() && a2.prop("roll").is_none());
        let cells = parse_cells(&row(&s, &ra).unwrap());
        assert!(cells.get("rel").is_none());
        assert_eq!(cells.get("n"), Some(&json!(3)));
    }

    #[test]
    fn a_long_chain_of_rollups_ends_with_an_error_not_a_crash() {
        // Everything runs on a thread with a small stack: a crash by stack overflow would end the test run.
        let h = std::thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                let mut s = temp_vault();
                let mut a = make_db(&mut s, "Chain");
                a.props.push(Property { id: "next".into(), name: "Next".into(), kind: PropKind::Relation, relation_db: a.id.clone(), ..Property::default() });
                a.props.push(Property { id: "roll".into(), name: "Roll".into(), kind: PropKind::Rollup, rollup_relation: "next".into(), rollup_prop: "roll".into(), rollup_calc: "count".into(), ..Property::default() });
                s.put(Item::Database(a.clone())).unwrap();
                let ids: Vec<String> = (0..3000).map(|i| add_row(&mut s, &mut a, &format!("r{i}"), "{}")).collect();
                for (i, id) in ids.iter().enumerate() {
                    if let Some(n) = ids.get(i + 1) {
                        let mut r = row(&s, id).unwrap();
                        r.cells = format!("{{\"next\":[\"{n}\"]}}");
                        s.put(Item::Row(r)).unwrap();
                    }
                }
                let ctx = Ctx::new(s.items(), 0, 0);
                let db = database(&s, &a.id).unwrap();
                let first = row(&s, &ids[0]).unwrap();
                let roll = db.prop("roll").unwrap().clone();
                ctx.value(&db, &first, &roll).is_err()
            })
            .unwrap();
        assert!(h.join().expect("no stack overflow"), "a chain this long is refused with an error");
    }

    #[test]
    fn reminders_come_due_once_and_stay_quiet_for_old_or_future_dates() {
        let mut s = temp_vault();
        let mut db = Database::new(new_id(), "sec".into(), "Plans".into(), 1);
        db.props.push(Property { id: "d".into(), name: "When".into(), kind: PropKind::Date, ..Property::default() });
        s.put(Item::Database(db.clone())).unwrap();
        let now = value::from_parts(2026, 11, 3, 12, 0);
        let iso = |m: i64| value::format_iso(m, true);
        let cell = |start: i64, rem: i64| format!("{{\"d\":{{\"start\":\"{}\",\"reminder\":{rem}}}}}", iso(start));
        add_row(&mut s, &mut db, "due", &cell(now + 5, 10));
        add_row(&mut s, &mut db, "later", &cell(now + 120, 10));
        add_row(&mut s, &mut db, "old", &cell(now - 3 * 24 * 60, 10));
        add_row(&mut s, &mut db, "no reminder", &format!("{{\"d\":{{\"start\":\"{}\"}}}}", iso(now)));
        let ctx = Ctx::new(s.items(), now, 0);
        let mut shown = HashSet::new();
        let due = due_reminders(&ctx, &shown);
        assert_eq!(due.iter().map(|d| d.title.as_str()).collect::<Vec<_>>(), vec!["due"]);
        shown.insert(due[0].key.clone());
        assert!(due_reminders(&ctx, &shown).is_empty(), "reported once");
    }

    #[test]
    fn cells_are_checked() {
        let mut s = temp_vault();
        let db = make_db(&mut s, "C");
        let qty = db.prop("n").unwrap().clone();
        assert_eq!(clean_cell(&s, &db, &qty, &json!("4.5")).unwrap(), Some(json!(4.5)));
        assert!(clean_cell(&s, &db, &qty, &json!("abc")).is_err());
        assert_eq!(clean_cell(&s, &db, &qty, &json!("")).unwrap(), None);
        let files = db.prop("f").unwrap().clone();
        assert!(clean_cell(&s, &db, &files, &json!([{"id": "../../etc"}])).is_err(), "only vault ids");
        let ok = clean_cell(&s, &db, &files, &json!([{"id": "a".repeat(32), "name": "..\\x<y>.pdf", "mime": "text/html"}])).unwrap().unwrap();
        assert_eq!(ok[0]["name"], "..xy.pdf");
        let title = db.props[0].clone();
        assert!(clean_cell(&s, &db, &title, &json!("x")).is_err());
    }

    #[test]
    fn csv_export_follows_the_view_and_guards_formulas() {
        let mut s = temp_vault();
        let mut db = make_db(&mut s, "E");
        add_row(&mut s, &mut db, "=SUM(A1)", "{\"n\":-5}");
        add_row(&mut s, &mut db, "plain", "{\"n\":7,\"f\":[{\"id\":\"00000000000000000000000000000000\",\"name\":\"a.pdf\"}]}");
        let lines = export_lines(&s, &db.id, None, true).unwrap();
        assert_eq!(lines[0], vec!["Name", "Qty", "Docs"]);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1][0], "'=SUM(A1)", "a formula in a title is defused");
        assert_eq!(lines[1][1], "-5", "a real negative number is left alone");
        assert_eq!(lines[2][2], "a.pdf");
        // A view that hides Docs and filters.
        let mut v = View { id: "v1".into(), name: "Big".into(), ..View::default() };
        v.columns = vec![Column { prop: "n".into(), visible: true, width: 0 }, Column { prop: "f".into(), visible: false, width: 0 }];
        v.filter.conds.push(Cond { prop: "n".into(), op: "gt".into(), value: "0".into(), ..Cond::default() });
        let mut d = database(&s, &db.id).unwrap();
        d.views.push(v);
        s.put(Item::Database(d)).unwrap();
        let lines = export_lines(&s, &db.id, Some("v1"), true).unwrap();
        assert_eq!(lines[0], vec!["Name", "Qty"]);
        assert_eq!(lines.len(), 2, "only the row with Qty above 0");
        assert_eq!(lines[1], vec!["plain", "7"]);
    }
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;
    #[test]
    fn due_reminders_now_person_and_core_match() {
        compare(
            |state| due_reminders_now_person(state),
            |s, _paths| due_reminders_now_core(s, &Actor::Person),
        );
    }
    #[test]
    fn chart_data_person_and_core_match() {
        compare(
            |state| {
                chart_data_person(
                    state,
                    QueryRequest {
                        database_id: "10000000000000000000000000000007".into(),
                        view_id: "10000000000000000000000000000009".into(),
                        view: None,
                    },
                )
            },
            |s, _paths| {
                chart_data_core(
                    s,
                    &Actor::Person,
                    QueryRequest {
                        database_id: "10000000000000000000000000000007".into(),
                        view_id: "10000000000000000000000000000009".into(),
                        view: None,
                    },
                )
            },
        );
    }
    #[test]
    fn enable_dependencies_person_and_core_match() {
        compare(
            |state| enable_dependencies_person(state, "10000000000000000000000000000007".into()),
            |s, _paths| enable_dependencies_core(s, &Actor::Person, "10000000000000000000000000000007".into()),
        );
    }
    #[test]
    fn enable_sub_items_person_and_core_match() {
        compare(
            |state| enable_sub_items_person(state, "10000000000000000000000000000007".into()),
            |s, _paths| enable_sub_items_core(s, &Actor::Person, "10000000000000000000000000000007".into()),
        );
    }
    #[test]
    fn move_database_person_and_core_match() {
        compare(
            |state| move_database_person(state, "10000000000000000000000000000007".into(), "10000000000000000000000000000003".into()),
            |s, _paths| move_database_core(s, &Actor::Person, "10000000000000000000000000000007".into(), "10000000000000000000000000000003".into()),
        );
    }
    #[test]
    fn set_database_look_person_and_core_match() {
        compare(
            |state| {
                set_database_look_person(
                    state,
                    "10000000000000000000000000000007".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "x".into(),
                    true,
                    false,
                )
            },
            |s, _paths| {
                set_database_look_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000007".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "x".into(),
                    true,
                    false,
                )
            },
        );
    }
    #[test]
    fn row_titles_person_and_core_match() {
        compare(
            |state| row_titles_person(state, vec!["10000000000000000000000000000008".into()]),
            |s, _paths| row_titles_core(s, &Actor::Person, vec!["10000000000000000000000000000008".into()]),
        );
    }
    #[test]
    fn search_rows_person_and_core_match() {
        compare(
            |state| search_rows_person(state, "10000000000000000000000000000007".into(), "Row".into(), 100),
            |s, _paths| search_rows_core(s, &Actor::Person, "10000000000000000000000000000007".into(), "Row".into(), 100),
        );
    }
    #[test]
    fn reorder_rows_person_and_core_match() {
        compare(
            |state| reorder_rows_person(state, vec!["10000000000000000000000000000008".into()]),
            |s, _paths| reorder_rows_core(s, &Actor::Person, vec!["10000000000000000000000000000008".into()]),
        );
    }
    #[test]
    fn duplicate_row_person_and_core_match() {
        compare(
            |state| duplicate_row_person(state, "10000000000000000000000000000008".into()),
            |s, _paths| duplicate_row_core(s, &Actor::Person, "10000000000000000000000000000008".into()),
        );
    }
    #[test]
    fn press_button_person_and_core_match() {
        compare(
            |state| press_button_person(state, "10000000000000000000000000000008".into(), "button".into()),
            |s, _paths| press_button_core(s, &Actor::Person, "10000000000000000000000000000008".into(), "button".into()),
        );
    }
    #[test]
    fn set_cells_person_and_core_match() {
        compare(
            |state| {
                set_cells_person(
                    state,
                    "10000000000000000000000000000008".into(),
                    Some("Changed row".into()),
                    "{\"number\":9876.543210987}".into(),
                )
            },
            |s, _paths| {
                set_cells_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000008".into(),
                    Some("Changed row".into()),
                    "{\"number\":9876.543210987}".into(),
                )
            },
        );
    }
    #[test]
    fn create_row_person_and_core_match() {
        compare(
            |state| {
                create_row_person(
                    state,
                    "10000000000000000000000000000007".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "{\"number\":1234.567890123}".into(),
                    "-".into(),
                )
            },
            |s, _paths| {
                create_row_core(
                    s,
                    &Actor::Person,
                    "10000000000000000000000000000007".into(),
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    "{\"number\":1234.567890123}".into(),
                    "-".into(),
                )
            },
        );
    }
    #[test]
    fn get_row_person_and_core_match() {
        compare(
            |state| get_row_person(state, "10000000000000000000000000000008".into()),
            |s, _paths| get_row_core(s, &Actor::Person, "10000000000000000000000000000008".into()),
        );
    }
    #[test]
    fn query_view_person_and_core_match() {
        compare(
            |state| {
                query_view_person(
                    state,
                    QueryRequest {
                        database_id: "10000000000000000000000000000007".into(),
                        view_id: "10000000000000000000000000000009".into(),
                        view: None,
                    },
                )
            },
            |s, _paths| {
                query_view_core(
                    s,
                    &Actor::Person,
                    QueryRequest {
                        database_id: "10000000000000000000000000000007".into(),
                        view_id: "10000000000000000000000000000009".into(),
                        view: None,
                    },
                )
            },
        );
    }
    #[test]
    fn save_database_person_and_core_match() {
        compare(
            |state| save_database_person(state, crate::actions::tests::database_json()),
            |s, _paths| save_database_core(s, &Actor::Person, crate::actions::tests::database_json()),
        );
    }
    #[test]
    fn get_database_person_and_core_match() {
        compare(
            |state| get_database_person(state, "10000000000000000000000000000007".into()),
            |s, _paths| get_database_core(s, &Actor::Person, "10000000000000000000000000000007".into()),
        );
    }
    #[test]
    fn create_database_person_and_core_match() {
        compare(
            |state| {
                create_database_person(
                    state,
                    "10000000000000000000000000000003".into(),
                    String::new(),
                    true,
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    String::new(),
                )
            },
            |s, paths| {
                create_database_core(
                    s,
                    &Actor::Person,
                    paths,
                    "10000000000000000000000000000003".into(),
                    String::new(),
                    true,
                    "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ".into(),
                    String::new(),
                )
            },
        );
    }
}
