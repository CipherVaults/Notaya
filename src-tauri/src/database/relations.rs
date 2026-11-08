//! Reciprocal relation validation and row synchronization.
use super::types::{Database, PropKind, Property, Row};
use crate::{
    commands::database::{parse_cells, write_cells},
    error::{Error, Result},
    model::Item,
    vault::Session,
};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

fn invalid() -> Error {
    Error::invalid("Invalid database relation.")
}

pub(crate) fn reciprocal(source: &Database, p: &Property, target: &Database) -> bool {
    p.kind == PropKind::Relation
        && p.relation_db == target.id
        && target.prop(&p.relation_back).is_some_and(|back| {
            back.kind == PropKind::Relation
                && back.relation_db == source.id
                && back.relation_back == p.id
        })
}

/// `+` is accepted only while preparing a new, controlled back property.
pub(crate) fn validate(s: &Session, db: &Database, creating: bool) -> Result<()> {
    validate_with(s, db, creating, &HashMap::new())
}

pub(crate) fn validate_with(
    s: &Session,
    db: &Database,
    creating: bool,
    pending: &HashMap<String, Database>,
) -> Result<()> {
    for p in db.props.iter().filter(|p| p.kind == PropKind::Relation) {
        let target = if p.relation_db == db.id {
            db
        } else {
            match pending.get(&p.relation_db) {
                Some(d) => d,
                None => match s.get(&p.relation_db) {
                    Some(Item::Database(d)) => d,
                    _ => return Err(invalid()),
                },
            }
        };
        if !p.relation_back.is_empty()
            && !(creating && p.relation_back == "+")
            && !reciprocal(db, p, target)
        {
            return Err(invalid());
        }
    }
    Ok(())
}

/// For every relation property that has another side, makes the other rows list this row (or not)
/// to match `new` (compared with `old`). Returns the changed rows.
pub(crate) fn relation_sync_diff(
    s: &Session,
    row_id: &str,
    old: Option<&Map<String, Value>>,
    new: &Map<String, Value>,
    db: &Database,
) -> Result<Vec<Item>> {
    validate(s, db, false)?;
    let mut out: HashMap<String, Row> = HashMap::new();
    for p in db
        .props
        .iter()
        .filter(|p| p.kind == PropKind::Relation && !p.relation_back.is_empty())
    {
        let ids = |m: &Map<String, Value>| -> Vec<String> {
            m.get(&p.id)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };
        let before = old.map(&ids).unwrap_or_default();
        let after = ids(new);
        for target in before
            .iter()
            .filter(|x| !after.contains(x))
            .chain(after.iter().filter(|x| !before.contains(x)))
        {
            let adding = after.contains(target);
            let mut t = match out.remove(target) {
                Some(t) => t,
                None => match s.get(target) {
                    Some(Item::Row(t)) => t.clone(),
                    _ => continue,
                },
            };
            if t.db != p.relation_db {
                return Err(invalid());
            }
            let mut tm = parse_cells(&t);
            let mut list: Vec<String> = tm
                .get(&p.relation_back)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            if adding {
                if !list.contains(&row_id.to_string()) {
                    list.push(row_id.to_string());
                }
                // A "one row only" other side keeps just the newest link.
                if let Some(tdb) = match s.get(&t.db) {
                    Some(Item::Database(d)) => Some(d),
                    _ => None,
                } {
                    if tdb
                        .prop(&p.relation_back)
                        .map_or(false, |q| q.relation_limit_one)
                        && list.len() > 1
                    {
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
