//! Small database saves for action callers. Reuse the existing core transaction
//! for validation, row cleanup, back relations and ID numbering. Permission,
//! confirmation, claims and revision checks belong to the dispatcher.
use super::types::{
    Column, Database, PropKind, Property, Row, View, MAX_PROPERTIES, MAX_VIEWS, TITLE_PROP,
};
use super::{clock, relations};
use crate::{
    actions::Actor,
    commands::database::{database, parse_cells, row, write_cells},
    error::{Error, Result},
    model::{new_id, now_ms, Item},
    vault::Session,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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

pub(crate) fn save_database_core(
    s: &mut Session,
    _actor: &Actor,
    database_json: String,
) -> Result<Database> {
    s.run_as(_actor, |s| {
        if database_json.len() > 30_000_000 {
            return Err(Error::invalid("This database definition is too large."));
        }
        let mut incoming: Database = serde_json::from_str(&database_json)
            .map_err(|_| Error::invalid("That database definition is not valid."))?;
        let old = database(s, &incoming.id)?;
        if matches!(_actor, Actor::Ai { .. }) && s.in_bin(&incoming.id) {
            return Err(Error::invalid(
                "This database is in the bin. Restore it before editing.",
            ));
        }
        if old.locked && incoming.locked {
            return Err(Error::invalid(
                "This database is locked. Unlock it to change its properties.",
            ));
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
            a.last_run = old
                .automations
                .iter()
                .find(|o| o.id == a.id)
                .map_or(now_min, |o| o.last_run);
        }
        incoming.updated = now_ms();
        incoming.sanitize()?;
        // Existing malformed back references must never be used for cleanup, even
        // when the incoming save removes the offending property.
        relations::validate(s, &old, false)?;
        relations::validate(s, &incoming, true)?;
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
        let mut changed_databases: HashMap<String, Database> = HashMap::new();
        let mut incoming_extra: Vec<(String, Property)> = Vec::new(); // (target db id, new back property)
        for p in &mut incoming.props {
            match p.kind {
                PropKind::Relation => {
                    let target_exists = p.relation_db == incoming.id
                        || matches!(s.get(&p.relation_db), Some(Item::Database(_)));
                    if !target_exists {
                        return Err(Error::invalid(format!(
                            "The relation \"{}\" needs a database to link to.",
                            p.name
                        )));
                    }
                    if p.relation_back == "+" {
                        let back = Property {
                            id: new_id(),
                            name: format!(
                                "Related to {} ({})",
                                if incoming.title.is_empty() {
                                    "Untitled"
                                } else {
                                    &incoming.title
                                },
                                p.name
                            ),
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
                let mut tdb = match changed_databases.remove(&target) {
                    Some(db) => db,
                    None => database(s, &target)?,
                };
                if tdb.props.len() >= MAX_PROPERTIES {
                    return Err(Error::invalid(
                        "The other database has too many properties.",
                    ));
                }
                let col = Column {
                    prop: back.id.clone(),
                    visible: true,
                    width: 0,
                };
                for v in &mut tdb.views {
                    v.columns.push(col.clone());
                }
                tdb.props.push(back);
                tdb.updated = now_ms();
                changed_databases.insert(target, tdb);
            }
        }
        for b in self_extra {
            let col = Column {
                prop: b.id.clone(),
                visible: true,
                width: 0,
            };
            for v in &mut incoming.views {
                v.columns.push(col.clone());
            }
            incoming.props.push(b);
        }
        // Removed properties: cut them out of rows and out of properties that point at them.
        let removed: Vec<Property> = old
            .props
            .iter()
            .filter(|p| incoming.prop(&p.id).is_none())
            .cloned()
            .collect();
        let rows_here: Vec<String> = s
            .items()
            .filter_map(|i| {
                if let Item::Row(r) = i {
                    if r.db == incoming.id {
                        Some(r.id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
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
            for p in removed
                .iter()
                .filter(|p| p.kind == PropKind::Relation && !p.relation_back.is_empty())
            {
                if p.relation_db == incoming.id {
                    if !relations::reciprocal(&old, p, &old) {
                        return Err(invalid());
                    }
                    if let Some(i) = incoming.props.iter().position(|q| q.id == p.relation_back) {
                        incoming.props.remove(i);
                    }
                } else {
                    let mut tdb = match changed_databases.remove(&p.relation_db) {
                        Some(db) => db,
                        None => database(s, &p.relation_db)?,
                    };
                    if !relations::reciprocal(&old, p, &tdb) {
                        return Err(invalid());
                    }
                    if tdb.props.iter().any(|q| q.id == p.relation_back) {
                        tdb.props.retain(|q| q.id != p.relation_back);
                        tdb.props.retain(|q| {
                            !(q.kind == PropKind::Rollup && q.rollup_relation == p.relation_back)
                        });
                        for v in &mut tdb.views {
                            v.columns
                                .retain(|c| tdb.props.iter().any(|q| q.id == c.prop));
                        }
                        changed_databases.insert(p.relation_db.clone(), tdb);
                    }
                }
            }
            let gone: HashSet<String> = removed.iter().map(|p| p.id.clone()).collect();
            incoming
                .props
                .retain(|q| !(q.kind == PropKind::Rollup && gone.contains(&q.rollup_relation)));
            let keep: HashSet<String> = incoming.props.iter().map(|q| q.id.clone()).collect();
            for v in &mut incoming.views {
                v.columns.retain(|c| keep.contains(&c.prop));
            }
        }
        // A new ID property numbers the rows that exist, oldest first.
        let new_id_prop = incoming.props.iter().any(|p| p.kind == PropKind::Id)
            && !old.props.iter().any(|p| p.kind == PropKind::Id);
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
        changed_databases.insert(incoming.id.clone(), incoming.clone());
        // Validate the complete prospective graph before the one atomic write.
        for db in changed_databases.values_mut() {
            db.sanitize()?;
        }
        for db in changed_databases.values() {
            relations::validate_with(s, db, false, &changed_databases)?;
        }
        // Retargeting an existing relation must not leave its former reciprocal
        // property pointing at a different pair in the prospective graph.
        for p in old
            .props
            .iter()
            .filter(|p| p.kind == PropKind::Relation && !p.relation_back.is_empty())
        {
            if !changed_databases.contains_key(&p.relation_db) {
                let target = database(s, &p.relation_db)?;
                relations::validate_with(s, &target, false, &changed_databases)?;
            }
        }
        changed_items.extend(changed_databases.into_values().map(Item::Database));
        s.put_many(changed_items)?;
        Ok(incoming)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TEST_SECTION: &str = "11111111111111111111111111111111";

    fn with_session(test: impl FnOnce(&mut Session)) {
        let root = std::env::var_os("NOTAYA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(new_id());
        std::fs::create_dir_all(&root).unwrap();
        let mut session = Session::create_open(&root.join("vault.nota")).unwrap();
        session.put(Item::Section(serde_json::from_value(json!({"id": TEST_SECTION,"notebookId":"","name":"Test section","color":"","order":0,"created":1,"updated":1})).unwrap())).unwrap();
        session
            .put(Item::Database(Database::new(
                "dddddddddddddddddddddddddddddddd".into(),
                TEST_SECTION.into(),
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
    fn moved_database_save_rejects_ai_in_bin_but_allows_pending_person_save() {
        for bin_container in [false, true] {
            with_session(|s| {
                let id = "dddddddddddddddddddddddddddddddd";
                let mut db = database(s, id).unwrap();
                if bin_container {
                    let section_id = new_id();
                    s.put(Item::Section(crate::model::Section {
                        source: None,
                        id: section_id.clone(),
                        notebook_id: new_id(),
                        name: "private container".into(),
                        color: String::new(),
                        order: 0,
                        created: 1,
                        updated: 1,
                        trashed_at: Some(1),
                    }))
                    .unwrap();
                    db.section_id = section_id;
                } else {
                    db.trashed_at = Some(1);
                }
                s.put(Item::Database(db.clone())).unwrap();
                assert!(s.in_bin(id));
                s.drain_changes();
                let before = snapshot(s);
                let rev = s.rev(id);
                db.title = "private pending edit".into();
                // A save must not let the caller clear the bin flag either.
                db.trashed_at = None;
                let json = serde_json::to_string(&db).unwrap();
                let ai = Actor::Ai {
                    pairing_id: "test-pairing".into(),
                    label: "Codex".into(),
                };
                let error = crate::commands::database::save_database_core(s, &ai, json.clone())
                    .err()
                    .unwrap();
                assert!(matches!(error, Error::Invalid(ref message)
                    if message == "This database is in the bin. Restore it before editing."));
                assert_eq!(snapshot(s), before);
                assert_eq!(s.rev(id), rev);
                assert!(s.drain_changes().is_none());

                let saved =
                    crate::commands::database::save_database_core(s, &Actor::Person, json).unwrap();
                assert_eq!(saved.title, "private pending edit");
                assert_eq!(saved.trashed_at, if bin_container { None } else { Some(1) });
                assert!(s.in_bin(id));
                let changes = s.drain_changes().unwrap();
                assert_eq!(changes.by, "person");
                assert_eq!(changes.ids, vec![id]);
                assert_eq!(changes.revs[id], rev + 1);
            });
        }
    }

    #[test]
    fn moved_database_save_attributes_the_whole_batch_and_restores_outer_actor() {
        with_session(|s| {
            let id = "dddddddddddddddddddddddddddddddd";
            let target = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            let row_id = "ffffffffffffffffffffffffffffffff";
            let mut db = add_property(s, &Actor::Person, id, prop("old")).unwrap();
            s.put(Item::Database(Database::new(
                target.into(),
                TEST_SECTION.into(),
                "target".into(),
                1,
            )))
            .unwrap();
            let mut row = Row::new(row_id.into(), id, "private row".into(), 1);
            row.cells = json!({"old":"private cell"}).to_string();
            s.put(Item::Row(row)).unwrap();
            s.drain_changes();
            let revs = [id, target, row_id].map(|id| (id, s.rev(id)));
            db.props.retain(|p| p.id != "old");
            db.props.push(Property {
                kind: PropKind::Relation,
                relation_db: target.into(),
                relation_back: "+".into(),
                ..prop("rel")
            });
            let ai = Actor::Ai {
                pairing_id: "test-pairing".into(),
                label: "Codex".into(),
            };
            s.run_as(&Actor::Automation, |s| {
                let saved = crate::commands::database::save_database_core(
                    s,
                    &ai,
                    serde_json::to_string(&db).unwrap(),
                )?;
                let changes = s.drain_changes().unwrap();
                assert_eq!(changes.by, "ai");
                assert_eq!(changes.ids, vec![id, target, row_id]);
                assert_eq!(changes.kinds, vec!["database", "database", "row"]);
                assert!(changes.tree);
                for (id, rev) in revs {
                    assert_eq!(changes.revs[id], rev + 1);
                }
                assert!(s.drain_changes().is_none());
                assert!(parse_cells(&crate::commands::database::row(s, row_id)?).is_empty());
                assert!(database(s, target)?
                    .prop(&saved.prop("rel").unwrap().relation_back)
                    .is_some());

                s.put(Item::Database(saved))?;
                assert_eq!(s.drain_changes().unwrap().by, "automation");
                Ok(())
            })
            .unwrap();
            let saved = database(s, id).unwrap();
            s.put(Item::Database(saved)).unwrap();
            assert_eq!(s.drain_changes().unwrap().by, "person");
        });
    }

    #[test]
    fn small_database_save_attributes_ai_and_failed_save_restores_outer_actor() {
        with_session(|s| {
            let id = "dddddddddddddddddddddddddddddddd";
            let ai = Actor::Ai {
                pairing_id: "test-pairing".into(),
                label: "Codex".into(),
            };
            s.drain_changes();
            let rev = s.rev(id);
            add_property(s, &ai, id, prop("p")).unwrap();
            let changes = s.drain_changes().unwrap();
            assert_eq!(changes.by, "ai");
            assert_eq!(changes.ids, vec![id]);
            assert_eq!(changes.revs[id], rev + 1);
            s.run_as(&Actor::Automation, |s| {
                assert!(save_database_core(s, &ai, "invalid json".into()).is_err());
                assert!(s.drain_changes().is_none());
                s.put(Item::Database(database(s, id)?))?;
                assert_eq!(s.drain_changes().unwrap().by, "automation");
                Ok(())
            })
            .unwrap();
        });
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

                let error = add_view(
                    s,
                    &actor,
                    id,
                    View {
                        id: "blank".into(),
                        name: name.into(),
                        ..View::default()
                    },
                )
                .err()
                .unwrap();
                assert!(
                    matches!(&error, Error::Invalid(message) if message == "Invalid database small save.")
                );
                assert_eq!(snapshot(s), before);
                let error = update_view(s, &actor, id, &view, &json!({"name":name, "wrap":true}))
                    .err()
                    .unwrap();
                assert!(
                    matches!(&error, Error::Invalid(message) if message == "Invalid database small save.")
                );
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
    fn hostile_and_oversized_identifiers_are_rejected_atomically() {
        with_session(|s| {
            let id = "dddddddddddddddddddddddddddddddd";
            let before = snapshot(s);
            for bad in [
                "__proto__".into(),
                "constructor".into(),
                "prototype".into(),
                "x".repeat(super::super::types::MAX_ID_BYTES + 1),
            ] {
                for property in [
                    prop(&bad),
                    Property {
                        kind: PropKind::Select,
                        options: vec![super::super::types::SelectOption {
                            id: bad.clone(),
                            name: "Option".into(),
                            ..Default::default()
                        }],
                        ..prop("select")
                    },
                ] {
                    let error = add_property(s, &Actor::Person, id, property).err().unwrap();
                    assert!(
                        matches!(error, Error::Invalid(ref msg) if msg == "Invalid database small save.")
                    );
                    assert_eq!(snapshot(s), before);
                }
                assert!(add_view(
                    s,
                    &Actor::Person,
                    id,
                    View {
                        id: bad,
                        ..Default::default()
                    }
                )
                .is_err());
                assert_eq!(snapshot(s), before);
            }
            // Short legacy ids and a name containing direction marks are exact.
            let name = "\u{200f}\u{0639}\u{05d0} e\u{301}  ";
            let db = add_property(
                s,
                &Actor::Person,
                id,
                Property {
                    name: name.into(),
                    ..prop("old-id")
                },
            )
            .unwrap();
            assert_eq!(db.prop("old-id").unwrap().name.as_bytes(), name.as_bytes());
            let error = update_property(
                s,
                &Actor::Person,
                id,
                "old-id",
                &json!({"options":[{"id":"__proto__"}]}),
            )
            .err()
            .unwrap();
            assert!(matches!(error, Error::Invalid(_)));
        });
    }

    #[test]
    fn hostile_back_references_cannot_change_schema_or_rows() {
        with_session(|s| {
            let a = "dddddddddddddddddddddddddddddddd";
            let b = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            let row_id = "ffffffffffffffffffffffffffffffff";
            for (back_id, back_kind, back_db, back_prop) in [
                ("title", PropKind::Title, "", ""),
                ("ordinary", PropKind::Text, "", ""),
                ("mismatch", PropKind::Relation, a, "different"),
                ("wrong_db", PropKind::Relation, b, "rel"),
            ] {
                let original = Database::new(a.into(), TEST_SECTION.into(), "Source".into(), 1);
                s.put(Item::Database(original.clone())).unwrap();
                let mut target = Database::new(b.into(), TEST_SECTION.into(), "Target".into(), 1);
                if back_kind != PropKind::Title {
                    target.props.push(Property {
                        kind: back_kind,
                        relation_db: back_db.into(),
                        relation_back: back_prop.into(),
                        ..prop(back_id)
                    });
                }
                s.put(Item::Database(target.clone())).unwrap();
                let mut target_row = Row::new(row_id.into(), b, "Exact title".into(), 1);
                target_row.cells = json!({back_id:"untouched"}).to_string();
                s.put(Item::Row(target_row.clone())).unwrap();
                let rel = Property {
                    kind: PropKind::Relation,
                    relation_db: b.into(),
                    relation_back: back_id.into(),
                    ..prop("rel")
                };
                let error = add_property(s, &Actor::Person, a, rel.clone())
                    .err()
                    .unwrap();
                assert!(
                    matches!(error, Error::Invalid(ref msg) if msg == "Invalid database small save.")
                );
                assert_eq!(snapshot(s), serde_json::to_string(&original).unwrap());

                // Simulate a legacy malformed record; removal and row sync
                // must re-check it rather than trusting its stored metadata.
                let mut malformed = original.clone();
                malformed.props.push(rel);
                s.put(Item::Database(malformed.clone())).unwrap();
                let before = snapshot(s);
                assert!(remove_property(s, &Actor::Person, a, "rel").is_err());
                assert_eq!(snapshot(s), before);
                assert!(relations::relation_sync_diff(
                    s,
                    "source-row",
                    None,
                    &serde_json::from_value(json!({"rel":[row_id]})).unwrap(),
                    &malformed
                )
                .is_err());
                assert_eq!(
                    serde_json::to_value(database(s, b).unwrap()).unwrap(),
                    serde_json::to_value(&target).unwrap()
                );
                assert_eq!(row(s, row_id).unwrap().cells, target_row.cells);
                assert_eq!(row(s, row_id).unwrap().title, target_row.title);
            }
        });
    }

    #[test]
    fn multiple_back_relations_share_one_validated_target_save() {
        with_session(|s| {
            let a = "dddddddddddddddddddddddddddddddd";
            let b = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            s.put(Item::Database(Database::new(
                b.into(),
                TEST_SECTION.into(),
                "Target".into(),
                1,
            )))
            .unwrap();
            let mut db = database(s, a).unwrap();
            for id in ["rel1", "rel2"] {
                db.props.push(Property {
                    kind: PropKind::Relation,
                    relation_db: b.into(),
                    relation_back: "+".into(),
                    ..prop(id)
                });
            }
            let saved = save(s, &Actor::Person, db).unwrap();
            let target = database(s, b).unwrap();
            for p in saved.props.iter().filter(|p| p.kind == PropKind::Relation) {
                assert!(relations::reciprocal(&saved, p, &target));
            }
            let c = "cccccccccccccccccccccccccccccccc";
            let mut other = Database::new(c.into(), TEST_SECTION.into(), "Other target".into(), 1);
            other.props.push(Property {
                kind: PropKind::Relation,
                relation_db: a.into(),
                relation_back: "rel1".into(),
                ..prop("back")
            });
            s.put(Item::Database(other)).unwrap();
            let before = snapshot(s);
            assert!(update_property(
                s,
                &Actor::Person,
                a,
                "rel1",
                &json!({"relationDb":c,"relationBack":"back"})
            )
            .is_err());
            assert_eq!(snapshot(s), before);
            let mut full = target.clone();
            full.props[1].name = "x".repeat(121);
            s.put(Item::Database(full)).unwrap();
            let before = snapshot(s);
            assert!(remove_property(s, &Actor::Person, a, "rel2").is_err());
            assert_eq!(snapshot(s), before);
        });
    }

    #[test]
    fn relation_back_properties_and_id_numbering_use_existing_transaction() {
        with_session(|s| {
            s.put(Item::Database(Database::new(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
                TEST_SECTION.into(),
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
