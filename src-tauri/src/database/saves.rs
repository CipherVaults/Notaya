//! Small database saves for action callers. Reuse the existing core transaction
//! for validation, row cleanup, back relations and ID numbering. Permission,
//! confirmation, claims and revision checks belong to the dispatcher.
use super::types::{
    Column, Database, PropKind, Property, Row, View, MAX_PROPERTIES, MAX_VIEWS, TITLE_PROP,
};
use crate::{
    actions::Actor,
    commands::database::{database, row, parse_cells, write_cells},
    error::{Error, Result},
    model::{new_id, now_ms, Item},
    vault::Session,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use super::{clock, relations};

fn invalid() -> Error {
    Error::invalid("Invalid database small save.")
}
fn load(s: &Session, id: &str) -> Result<Database> {
    let db = database(s, id)?;
    if db.locked || db.trashed_at.is_some() {
        return Err(invalid());
    }
    Ok(db)
}
fn save(s: &mut Session, actor: &Actor, db: Database) -> Result<Database> {
    let json = serde_json::to_string(&db).map_err(|_| invalid())?;
    // Legacy validation can mention a property name. Actions must not expose
    // note content in errors; storage faults keep their numeric fault codes.
    save_database_core(s, actor, json).map_err(|e| match e {
        Error::Invalid(_) | Error::Other(_) => invalid(),
        other => other,
    })
}

/// A shallow, typed-field patch. No identity/type changes through update; arrays
/// (options, filters, etc.) replace the corresponding field as a whole.
fn patch<T: Serialize + DeserializeOwned>(old: &T, patch: &Value, protected: &[&str]) -> Result<T> {
    let mut value = serde_json::to_value(old).map_err(|_| invalid())?;
    let incoming = patch.as_object().ok_or_else(invalid)?;
    let object = value.as_object_mut().ok_or_else(invalid)?;
    for (key, val) in incoming {
        let current = object.get(key).ok_or_else(invalid)?;
        if protected.contains(&key.as_str()) && val != current {
            return Err(invalid());
        }
        object.insert(key.clone(), val.clone());
    }
    serde_json::from_value(value).map_err(|_| invalid())
}

pub fn add_property(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    mut property: Property,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    if property.id.is_empty() {
        property.id = new_id();
    }
    if db.props.len() >= MAX_PROPERTIES
        || db.prop(&property.id).is_some()
        || property.kind == PropKind::Title
        || property.name.trim().is_empty()
    {
        return Err(invalid());
    }
    db.props.push(property);
    save(s, actor, db)
}

pub fn update_property(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
    changes: &Value,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    let p = db
        .props
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or(Error::NotFound)?;
    *p = patch(p, changes, &["id", "kind"])?;
    if p.name.trim().is_empty() {
        return Err(invalid());
    }
    save(s, actor, db)
}

fn reorder<T: Clone>(
    items: &mut Vec<T>,
    ids: &[String],
    get_id: impl Fn(&T) -> &str,
) -> Result<()> {
    if ids.len() != items.len() {
        return Err(invalid());
    }
    let mut seen = HashSet::new();
    let mut next = Vec::with_capacity(items.len());
    for id in ids {
        if !seen.insert(id) {
            return Err(invalid());
        }
        next.push(
            items
                .iter()
                .find(|p| get_id(p) == id)
                .ok_or(Error::NotFound)?
                .clone(),
        );
    }
    *items = next;
    Ok(())
}

pub fn reorder_properties(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    ids: &[String],
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    if ids.first().map(String::as_str) != Some(TITLE_PROP) {
        return Err(invalid());
    }
    reorder(&mut db.props, ids, |p| &p.id)?;
    save(s, actor, db)
}

/// Visibility is per saved view, never a property deletion.
pub fn hide_property(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
    view_id: &str,
    hidden: bool,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    if db.prop(id).is_none() {
        return Err(Error::NotFound);
    }
    let view = db
        .views
        .iter_mut()
        .find(|v| v.id == view_id)
        .ok_or(Error::NotFound)?;
    if let Some(c) = view.columns.iter_mut().find(|c| c.prop == id) {
        c.visible = !hidden;
    } else {
        view.columns.push(Column {
            prop: id.into(),
            visible: !hidden,
            width: 0,
        });
    }
    save(s, actor, db)
}

/// Caller must obtain the property's destructive-action confirmation first.
pub fn remove_property(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    let index = db
        .props
        .iter()
        .position(|p| p.id == id)
        .ok_or(Error::NotFound)?;
    if db.props[index].kind == PropKind::Title {
        return Err(invalid());
    }
    db.props.remove(index);
    save(s, actor, db)
}

pub fn add_view(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    mut view: View,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    if view.id.is_empty() {
        view.id = new_id();
    }
    if db.views.len() >= MAX_VIEWS || db.view(&view.id).is_some() || view.name.trim().is_empty() {
        return Err(invalid());
    }
    db.views.push(view);
    save(s, actor, db)
}

pub fn update_view(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
    changes: &Value,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    let view = db
        .views
        .iter_mut()
        .find(|v| v.id == id)
        .ok_or(Error::NotFound)?;
    *view = patch(view, changes, &["id"])?;
    if changes.get("name").is_some() && view.name.trim().is_empty() {
        return Err(invalid());
    }
    save(s, actor, db)
}

pub fn rename_view(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
    name: String,
) -> Result<Database> {
    if name.trim().is_empty() {
        return Err(invalid());
    }
    update_view(s, actor, database_id, id, &serde_json::json!({"name":name}))
}

pub fn reorder_views(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    ids: &[String],
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    reorder(&mut db.views, ids, |v| &v.id)?;
    save(s, actor, db)
}

pub fn remove_view(
    s: &mut Session,
    actor: &Actor,
    database_id: &str,
    id: &str,
) -> Result<Database> {
    let mut db = load(s, database_id)?;
    let index = db
        .views
        .iter()
        .position(|v| v.id == id)
        .ok_or(Error::NotFound)?;
    if db.views.len() == 1 {
        return Err(invalid());
    }
    db.views.remove(index);
    save(s, actor, db)
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn with_session(test: impl FnOnce(&mut Session)) {
        let root = std::env::var_os("NOTAYA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(new_id());
        std::fs::create_dir_all(&root).unwrap();
        let mut session = Session::create_open(&root.join("vault.nota")).unwrap();
        session
            .put(Item::Database(Database::new(
                "dddddddddddddddddddddddddddddddd".into(),
                "s".into(),
                "\u{200f}عربي  ".into(),
                1,
            )))
            .unwrap();
        test(&mut session);
        drop(session);
        std::fs::remove_dir_all(root).unwrap();
    }
    fn prop(id: &str) -> Property {
        Property {
            id: id.into(),
            name: "  Property  ".into(),
            ..Property::default()
        }
    }
    fn snapshot(s: &Session) -> String {
        serde_json::to_string(&database(s, "dddddddddddddddddddddddddddddddd").unwrap()).unwrap()
    }

    #[test]
    fn properties_and_views_save_one_piece_and_keep_exact_names() {
        with_session(|s| {
            let actor = Actor::Person;
            let db =
                add_property(s, &actor, "dddddddddddddddddddddddddddddddd", prop("p")).unwrap();
            assert_eq!(db.prop("p").unwrap().name.as_bytes(), b"  Property  ");
            assert_eq!(db.title, "\u{200f}عربي  ");
            let name = "\u{200e}?Deutsch\u{200f}  e\u{301}";
            let db = update_property(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                "p",
                &json!({"name":name}),
            )
            .unwrap();
            assert_eq!(db.prop("p").unwrap().name.as_bytes(), name.as_bytes());
            let first = db.views[0].id.clone();
            add_view(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                View {
                    id: "v2".into(),
                    name: name.into(),
                    ..View::default()
                },
            )
            .unwrap();
            let db = hide_property(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                "p",
                &first,
                true,
            )
            .unwrap();
            assert!(
                !db.view(&first)
                    .unwrap()
                    .columns
                    .iter()
                    .find(|c| c.prop == "p")
                    .unwrap()
                    .visible
            );
            assert!(
                db.view("v2")
                    .unwrap()
                    .columns
                    .iter()
                    .find(|c| c.prop == "p")
                    .unwrap()
                    .visible
            );
            rename_view(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                "v2",
                name.into(),
            )
            .unwrap();
            update_view(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                "v2",
                &json!({"wrap":true}),
            )
            .unwrap();
            let db = reorder_views(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                &["v2".into(), first.clone()],
            )
            .unwrap();
            assert_eq!(db.views[0].name.as_bytes(), name.as_bytes());
            assert!(db.views[0].wrap);
            add_property(s, &actor, "dddddddddddddddddddddddddddddddd", prop("q")).unwrap();
            let db = reorder_properties(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                &["title".into(), "q".into(), "p".into()],
            )
            .unwrap();
            assert_eq!(db.props[1].id, "q");
            let mut row = Row::new(
                "ffffffffffffffffffffffffffffffff".into(),
                "dddddddddddddddddddddddddddddddd",
                "\u{200f}صف  ".into(),
                1,
            );
            row.cells = json!({"p":"\u{200e}?German  ","q":"kept"}).to_string();
            s.put(Item::Row(row)).unwrap();
            remove_property(s, &actor, "dddddddddddddddddddddddddddddddd", "p").unwrap();
            let row =
                crate::commands::database::row(s, "ffffffffffffffffffffffffffffffff").unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&row.cells).unwrap(),
                json!({"q":"kept"})
            );
            assert_eq!(row.title, "\u{200f}صف  ");
            remove_view(s, &actor, "dddddddddddddddddddddddddddddddd", &first).unwrap();
            let before = snapshot(s);
            assert!(remove_view(s, &actor, "dddddddddddddddddddddddddddddddd", "v2").is_err());
            assert_eq!(snapshot(s), before);
        });
    }

    #[test]
    fn blank_small_save_names_are_rejected_atomically_with_content_free_errors() {
        with_session(|s| {
            let actor = Actor::Person;
            let id = "dddddddddddddddddddddddddddddddd";
            let db = add_property(s, &actor, id, prop("p")).unwrap();
            let view = db.views[0].id.clone();
            let before = snapshot(s);
            for name in ["", "  ", "\t\r\n", "\u{2003}\u{00a0}"] {
                let property = Property {
                    name: name.into(),
                    ..prop("q")
                };
                let error = add_property(s, &actor, id, property).err().unwrap();
                assert!(
                    matches!(&error, Error::Invalid(message) if message == "Invalid database small save.")
                );
                assert_eq!(error.fault_code(), None);
                assert_eq!(snapshot(s), before);

                let error = update_property(s, &actor, id, "p", &json!({"name":name}))
                    .err()
                    .unwrap();
                assert!(
                    matches!(&error, Error::Invalid(message) if message == "Invalid database small save.")
                );
                assert_eq!(error.fault_code(), None);
                assert_eq!(snapshot(s), before);

                let error = add_view(s, &actor, id, View {
                    id: "blank".into(), name: name.into(), ..View::default()
                }).err().unwrap();
                assert!(matches!(&error, Error::Invalid(message) if message == "Invalid database small save."));
                assert_eq!(snapshot(s), before);
                let error = update_view(s, &actor, id, &view, &json!({"name":name, "wrap":true})).err().unwrap();
                assert!(matches!(&error, Error::Invalid(message) if message == "Invalid database small save."));
                assert_eq!(snapshot(s), before);

                let error = rename_view(s, &actor, id, &view, name.into())
                    .err()
                    .unwrap();
                assert!(
                    matches!(&error, Error::Invalid(message) if message == "Invalid database small save.")
                );
                assert_eq!(error.fault_code(), None);
                assert_eq!(snapshot(s), before);
            }
        });
    }

    #[test]
    fn invalid_updates_are_atomic_bounded_and_content_free() {
        with_session(|s| {
            let actor = Actor::Person;
            add_property(s, &actor, "dddddddddddddddddddddddddddddddd", prop("p")).unwrap();
            let before = snapshot(s);
            for changes in [
                json!({"kind":"number"}),
                json!({"id":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}),
                json!({"unknown":"secret"}),
                json!({"name":"secret".repeat(121)}),
            ] {
                let error =
                    update_property(s, &actor, "dddddddddddddddddddddddddddddddd", "p", &changes)
                        .err()
                        .unwrap();
                assert!(!error.to_string().contains("secret"));
                assert_eq!(snapshot(s), before);
            }
            assert!(
                add_property(s, &actor, "dddddddddddddddddddddddddddddddd", prop("p")).is_err()
            );
            assert!(
                remove_property(s, &actor, "dddddddddddddddddddddddddddddddd", "title").is_err()
            );
            assert!(reorder_properties(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                &["title".into(), "title".into()]
            )
            .is_err());
            assert!(hide_property(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                "p",
                "missing",
                true
            )
            .is_err());
            assert_eq!(snapshot(s), before);
            let mut bad = prop("relation");
            bad.kind = PropKind::Relation;
            bad.name = "private name".into();
            bad.relation_db = "missing".into();
            assert!(
                !add_property(s, &actor, "dddddddddddddddddddddddddddddddd", bad)
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("private name")
            );
            assert_eq!(snapshot(s), before);
            let mut db = database(s, "dddddddddddddddddddddddddddddddd").unwrap();
            db.locked = true;
            s.put(Item::Database(db)).unwrap();
            assert!(
                add_property(s, &actor, "dddddddddddddddddddddddddddddddd", prop("q")).is_err()
            );
        });
    }

    #[test]
    fn relation_back_properties_and_id_numbering_use_existing_transaction() {
        with_session(|s| {
            s.put(Item::Database(Database::new(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
                "s".into(),
                "Other".into(),
                1,
            )))
            .unwrap();
            s.put(Item::Row(Row::new(
                "ffffffffffffffffffffffffffffffff".into(),
                "dddddddddddddddddddddddddddddddd",
                "Row".into(),
                1,
            )))
            .unwrap();
            let actor = Actor::Person;
            let db = add_property(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                Property {
                    id: "rel".into(),
                    kind: PropKind::Relation,
                    relation_db: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
                    relation_back: "+".into(),
                    ..prop("rel")
                },
            )
            .unwrap();
            let back = db.prop("rel").unwrap().relation_back.clone();
            assert!(database(s, "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
                .unwrap()
                .prop(&back)
                .is_some());
            remove_property(s, &actor, "dddddddddddddddddddddddddddddddd", "rel").unwrap();
            assert!(database(s, "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
                .unwrap()
                .prop(&back)
                .is_none());
            add_property(
                s,
                &actor,
                "dddddddddddddddddddddddddddddddd",
                Property {
                    kind: PropKind::Id,
                    ..prop("number")
                },
            )
            .unwrap();
            assert_eq!(
                crate::commands::database::row(s, "ffffffffffffffffffffffffffffffff")
                    .unwrap()
                    .number,
                1
            );
        });
    }
}
