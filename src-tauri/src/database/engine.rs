//! Works out what a database view shows: stored and computed values (formulas, rollups, created
//! and edited times, IDs), filters, sorts, groups and column calculations. It runs in the Rust
//! core; the UI receives finished rows. Nothing here touches the vault or the disk.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use serde::Serialize;
use serde_json::{json, Map, Value};
use zeroize::Zeroize;

use super::formula;
use super::types::*;
use super::value::*;
use crate::model::Item;

/// Everything a query needs: the databases and their (not trashed) rows, the clock and the
/// caches. Built from the vault's items for one call and then dropped.
const MAX_VALUE_DEPTH: usize = 32;

pub struct Ctx<'a> {
    pub dbs: HashMap<&'a str, &'a Database>,
    pub rows: HashMap<&'a str, Vec<&'a Row>>,
    pub by_id: HashMap<&'a str, &'a Row>,
    /// "Now" as local minutes since 1970 and the local offset from UTC in minutes.
    pub now: i64,
    pub tz: i64,
    cells: RefCell<HashMap<String, Rc<Map<String, Value>>>>,
    memo: RefCell<HashMap<(String, String), Result<Val, String>>>,
    /// Parsed formulas (including parse errors) live only for this query, keyed by exact text.
    formulas: RefCell<HashMap<String, Result<Rc<formula::Expr>, String>>>,
    busy: RefCell<HashSet<(String, String)>>,
    /// How many formula or rollup values are being worked out inside one another right now.
    depth: std::cell::Cell<usize>,
}

impl<'a> Ctx<'a> {
    pub fn new(items: impl Iterator<Item = &'a Item>, now: i64, tz: i64) -> Ctx<'a> {
        let mut dbs = HashMap::new();
        let mut rows: HashMap<&str, Vec<&Row>> = HashMap::new();
        let mut by_id = HashMap::new();
        for it in items {
            match it {
                Item::Database(d) if d.trashed_at.is_none() => {
                    dbs.insert(d.id.as_str(), d);
                }
                Item::Row(r) if r.trashed_at.is_none() => {
                    rows.entry(r.db.as_str()).or_default().push(r);
                    by_id.insert(r.id.as_str(), r);
                }
                _ => {}
            }
        }
        for v in rows.values_mut() {
            v.sort_by_key(|r| (r.order, r.created));
        }
        Ctx { dbs, rows, by_id, now, tz, cells: RefCell::default(), memo: RefCell::default(), formulas: RefCell::default(), busy: RefCell::default(), depth: std::cell::Cell::new(0) }
    }

    pub fn rows_of(&self, db: &str) -> &[&'a Row] {
        self.rows.get(db).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn cells_of(&self, row: &Row) -> Rc<Map<String, Value>> {
        if let Some(c) = self.cells.borrow().get(&row.id) {
            return c.clone();
        }
        let m: Map<String, Value> = serde_json::from_str::<Value>(&row.cells).ok().and_then(|v| if let Value::Object(m) = v { Some(m) } else { None }).unwrap_or_default();
        let rc = Rc::new(m);
        self.cells.borrow_mut().insert(row.id.clone(), rc.clone());
        rc
    }

    fn ms_to_local_minutes(&self, ms: i64) -> i64 {
        ms.div_euclid(60_000) + self.tz
    }

    /// The ids stored in a relation cell that still point at a live row.
    pub fn relation_ids(&self, row: &Row, prop: &Property) -> Vec<String> {
        let cells = self.cells_of(row);
        match cells.get(&prop.id) {
            Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str()).filter(|id| self.by_id.contains_key(id)).map(String::from).collect(),
            _ => Vec::new(),
        }
    }

    /// The value of a property for a row, as the engine compares and shows it: select options by
    /// name, relations as the titles of the related rows, dates parsed.
    pub fn value(&self, db: &Database, row: &Row, prop: &Property) -> Result<Val, String> {
        match prop.kind {
            PropKind::Formula | PropKind::Rollup => {
                let key = (row.id.clone(), prop.id.clone());
                if let Some(v) = self.memo.borrow().get(&key) {
                    return v.clone();
                }
                if !self.busy.borrow_mut().insert(key.clone()) {
                    return Err("Circular reference".into());
                }
                // A long chain (row 1 rolls up row 2, which rolls up row 3, ...) is not a cycle, but each
                // link costs stack; stop it before the stack runs out.
                if self.depth.get() >= MAX_VALUE_DEPTH {
                    self.busy.borrow_mut().remove(&key);
                    return Err("Too many levels of formulas and rollups".into());
                }
                self.depth.set(self.depth.get() + 1);
                let r = if prop.kind == PropKind::Formula { self.formula_value(db, row, prop) } else { self.rollup_value(db, row, prop) };
                self.depth.set(self.depth.get() - 1);
                self.busy.borrow_mut().remove(&key);
                self.memo.borrow_mut().insert(key, r.clone());
                r
            }
            _ => Ok(self.stored(db, row, prop)),
        }
    }

    fn stored(&self, _db: &Database, row: &Row, prop: &Property) -> Val {
        let cells = self.cells_of(row);
        let raw = cells.get(&prop.id);
        let str_of = |v: Option<&Value>| -> Val {
            match v {
                Some(Value::String(s)) if !s.is_empty() => Val::Str(s.clone()),
                _ => Val::Null,
            }
        };
        match prop.kind {
            PropKind::Title => {
                if row.title.is_empty() {
                    Val::Null
                } else {
                    Val::Str(row.title.clone())
                }
            }
            PropKind::Text | PropKind::Url | PropKind::Email | PropKind::Phone => str_of(raw),
            PropKind::Number => match raw {
                Some(Value::Number(n)) => n.as_f64().map_or(Val::Null, Val::Num),
                _ => Val::Null,
            },
            PropKind::Select | PropKind::Status => match raw {
                Some(Value::String(id)) => prop.option(id).map_or(Val::Null, |o| Val::Str(o.name.clone())),
                _ => Val::Null,
            },
            PropKind::MultiSelect => match raw {
                Some(Value::Array(a)) => Val::List(a.iter().filter_map(|x| x.as_str()).filter_map(|id| prop.option(id)).map(|o| Val::Str(o.name.clone())).collect()),
                _ => Val::List(vec![]),
            },
            PropKind::Date => match raw {
                Some(Value::Object(o)) => match o.get("start").and_then(|s| s.as_str()).and_then(parse_date) {
                    Some(mut d) => {
                        d.end = o.get("end").and_then(|s| s.as_str()).and_then(parse_date).map(|e| e.start);
                        if d.end.is_some() && d.end.unwrap() < d.start {
                            d.end = None;
                        }
                        Val::Date(d)
                    }
                    None => Val::Null,
                },
                _ => Val::Null,
            },
            PropKind::Person => match raw {
                Some(Value::Array(a)) => Val::List(a.iter().filter_map(|x| x.as_str()).map(|s| Val::Str(s.to_string())).collect()),
                _ => Val::List(vec![]),
            },
            PropKind::Files => match raw {
                Some(Value::Array(a)) => Val::List(a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str())).map(|s| Val::Str(s.to_string())).collect()),
                _ => Val::List(vec![]),
            },
            PropKind::Checkbox => Val::Bool(matches!(raw, Some(Value::Bool(true)))),
            PropKind::Relation => Val::List(
                self.relation_ids(row, prop)
                    .iter()
                    .filter_map(|id| self.by_id.get(id.as_str()))
                    .map(|r| Val::Str(if r.title.is_empty() { "Untitled".into() } else { r.title.clone() }))
                    .collect(),
            ),
            PropKind::Place => match raw {
                Some(Value::Object(o)) => str_of(o.get("address")),
                _ => Val::Null,
            },
            PropKind::Id => {
                if row.number > 0 {
                    Val::Str(if prop.id_prefix.is_empty() { row.number.to_string() } else { format!("{}-{}", prop.id_prefix, row.number) })
                } else {
                    Val::Null
                }
            }
            PropKind::CreatedTime => Val::Date(DateVal { start: self.ms_to_local_minutes(row.created), end: None, has_time: true }),
            PropKind::LastEditedTime => Val::Date(DateVal { start: self.ms_to_local_minutes(row.updated), end: None, has_time: true }),
            PropKind::CreatedBy => str_of(Some(&Value::String(row.created_by.clone()))),
            PropKind::LastEditedBy => str_of(Some(&Value::String(row.edited_by.clone()))),
            PropKind::Formula | PropKind::Rollup | PropKind::Button => Val::Null,
        }
    }

    fn formula_value(&self, db: &Database, row: &Row, prop: &Property) -> Result<Val, String> {
        struct Env<'c, 'a> {
            ctx: &'c Ctx<'a>,
            db: &'c Database,
            row: &'c Row,
        }
        impl<'c, 'a> formula::Env for Env<'c, 'a> {
            fn prop(&self, name: &str) -> Result<Val, String> {
                let p = self.db.prop_by_name(name).ok_or_else(|| format!("No property named {name}"))?;
                self.ctx.value(self.db, self.row, p)
            }
            fn row_id(&self) -> String {
                self.row.id.clone()
            }
            fn now(&self) -> i64 {
                self.ctx.now
            }
        }
        // Release the cache borrow before evaluation: prop() can recursively evaluate a formula.
        let parsed = self.formulas.borrow_mut().entry(prop.formula.clone()).or_insert_with(|| formula::parse(&prop.formula).map(Rc::new)).clone()?;
        formula::eval(&parsed, &Env { ctx: self, db, row })
    }

    fn rollup_value(&self, db: &Database, row: &Row, prop: &Property) -> Result<Val, String> {
        let rel = db.prop(&prop.rollup_relation).filter(|p| p.kind == PropKind::Relation).ok_or("Pick a relation to roll up")?;
        let tdb = self.dbs.get(rel.relation_db.as_str()).ok_or("The related database is gone")?;
        let tp = tdb.prop(&prop.rollup_prop).ok_or("Pick a property to roll up")?;
        let mut vals = Vec::new();
        for id in self.relation_ids(row, rel) {
            if let Some(r) = self.by_id.get(id.as_str()) {
                vals.push(self.value(tdb, r, tp)?);
            }
        }
        let calc_name = if prop.rollup_calc.is_empty() { "showOriginal" } else { prop.rollup_calc.as_str() };
        Ok(calc(calc_name, &vals))
    }

    // ---- filters ----

    pub fn matches(&self, db: &Database, row: &Row, f: &Filter) -> bool {
        let mut results = f.conds.iter().filter_map(|c| self.cond(db, row, c)).chain(f.groups.iter().filter(|g| !g.is_empty()).map(|g| self.matches(db, row, g)));
        if f.op == "or" {
            let mut any = false;
            let mut seen = false;
            for r in results {
                seen = true;
                any |= r;
            }
            !seen || any
        } else {
            results.all(|r| r)
        }
    }

    /// None when the condition names a property that no longer exists (it is ignored).
    fn cond(&self, db: &Database, row: &Row, c: &Cond) -> Option<bool> {
        let prop = db.prop(&c.prop)?;
        let v = self.value(db, row, prop).unwrap_or(Val::Null);
        Some(self.test(prop, &v, c, row))
    }

    fn test(&self, prop: &Property, v: &Val, c: &Cond, row: &Row) -> bool {
        match c.op.as_str() {
            "isEmpty" => return v.is_empty(),
            "isNotEmpty" => return !v.is_empty(),
            _ => {}
        }
        let wanted_names = || -> Vec<String> {
            let mut ids: Vec<&String> = c.values.iter().collect();
            if !c.value.is_empty() {
                ids.push(&c.value);
            }
            ids.iter().map(|id| prop.option(id).map(|o| o.name.clone()).unwrap_or_else(|| (*id).clone())).collect()
        };
        match prop.kind {
            PropKind::Select | PropKind::Status => {
                let names = wanted_names();
                let hit = names.iter().any(|n| matches!(v, Val::Str(s) if s.to_lowercase() == n.to_lowercase()));
                match c.op.as_str() {
                    "is" | "in" | "contains" => hit,
                    "isNot" | "notIn" | "notContains" => !hit,
                    _ => true,
                }
            }
            PropKind::MultiSelect | PropKind::Person | PropKind::Files => {
                let names = if prop.kind == PropKind::MultiSelect { wanted_names() } else { vec![c.value.clone()] };
                let items = if let Val::List(l) = v { l.clone() } else { vec![] };
                let hit = names.iter().any(|n| items.iter().any(|i| i.text().to_lowercase() == n.to_lowercase() || (prop.kind != PropKind::MultiSelect && i.text().to_lowercase().contains(&n.to_lowercase()))));
                match c.op.as_str() {
                    "contains" | "is" | "in" => hit,
                    "notContains" | "isNot" | "notIn" => !hit,
                    _ => true,
                }
            }
            PropKind::Relation => {
                let ids = self.relation_ids(row, prop);
                let mut wanted: Vec<&String> = c.values.iter().collect();
                if !c.value.is_empty() {
                    wanted.push(&c.value);
                }
                let hit = wanted.iter().any(|w| ids.contains(*w));
                match c.op.as_str() {
                    "contains" | "is" => hit,
                    "notContains" | "isNot" => !hit,
                    _ => true,
                }
            }
            PropKind::Checkbox => {
                let want = c.value == "true";
                match c.op.as_str() {
                    "is" => v.as_bool() == want,
                    "isNot" => v.as_bool() != want,
                    _ => true,
                }
            }
            _ => match v {
                Val::Num(n) => self.num_test(*n, c),
                Val::Date(d) => self.date_test(d, c),
                Val::Bool(b) => match c.op.as_str() {
                    "is" => *b == (c.value == "true"),
                    "isNot" => *b != (c.value == "true"),
                    _ => true,
                },
                Val::List(l) => {
                    let w = c.value.to_lowercase();
                    let hit = l.iter().any(|i| i.text().to_lowercase().contains(&w));
                    match c.op.as_str() {
                        "contains" | "is" => hit,
                        "notContains" | "isNot" => !hit,
                        _ => true,
                    }
                }
                Val::Str(_) | Val::Null => {
                    // A number or date property that is empty fails every comparison but "is not".
                    if matches!(prop.kind, PropKind::Number) {
                        return matches!(c.op.as_str(), "neq" | "notEquals");
                    }
                    if matches!(prop.kind, PropKind::Date | PropKind::CreatedTime | PropKind::LastEditedTime) {
                        return false;
                    }
                    let t = v.text().to_lowercase();
                    let w = c.value.to_lowercase();
                    match c.op.as_str() {
                        "equals" | "is" => t == w,
                        "notEquals" | "isNot" => t != w,
                        "contains" => t.contains(&w),
                        "notContains" => !t.contains(&w),
                        "startsWith" => t.starts_with(&w),
                        "endsWith" => t.ends_with(&w),
                        _ => true,
                    }
                }
            },
        }
    }

    fn num_test(&self, n: f64, c: &Cond) -> bool {
        let w = match c.value.trim().parse::<f64>() {
            Ok(w) => w,
            Err(_) => return true,
        };
        match c.op.as_str() {
            "eq" | "equals" | "is" => tidy(n) == tidy(w),
            "neq" | "notEquals" | "isNot" => tidy(n) != tidy(w),
            "gt" => n > w,
            "gte" => n >= w,
            "lt" => n < w,
            "lte" => n <= w,
            _ => true,
        }
    }

    /// The [lo, hi) minute range a date condition compares against.
    pub fn date_target(&self, c: &Cond) -> Option<(i64, i64)> {
        let today = start_of_day(self.now);
        let day = MINUTES_PER_DAY;
        Some(match c.rel.as_str() {
            "" | "exact" => {
                let d = parse_date(&c.value)?;
                if d.has_time {
                    (d.start, d.start + 1)
                } else {
                    (d.start, d.start + day)
                }
            }
            "today" => (today, today + day),
            "tomorrow" => (today + day, today + 2 * day),
            "yesterday" => (today - day, today),
            "oneWeekAgo" => (today - 7 * day, today - 6 * day),
            "oneWeekFromNow" => (today + 7 * day, today + 8 * day),
            "oneMonthAgo" => {
                let t = add_months(today, -1);
                (t, t + day)
            }
            "oneMonthFromNow" => {
                let t = add_months(today, 1);
                (t, t + day)
            }
            "thisWeek" => {
                let s = start_of_week(today);
                (s, s + 7 * day)
            }
            "pastWeek" => (today - 6 * day, today + day),
            "nextWeek" => (today + day, today + 8 * day),
            "pastMonth" => (today - 29 * day, today + day),
            "nextMonth" => (today + day, today + 31 * day),
            "thisMonth" => {
                let s = start_of_month(today);
                (s, add_months(s, 1))
            }
            "thisYear" => {
                let s = start_of_year(today);
                (s, add_months(s, 12))
            }
            "pastYear" => (add_months(today, -12) + day, today + day),
            "nextYear" => (today + day, add_months(today, 12) + day),
            _ => return None,
        })
    }

    fn date_test(&self, d: &DateVal, c: &Cond) -> bool {
        let (lo, hi) = match c.op.as_str() {
            "between" => {
                let a = parse_date(&c.value);
                let b = parse_date(&c.value2);
                match (a, b) {
                    (Some(a), Some(b)) => (a.start.min(b.start), start_of_day(a.start.max(b.start)) + MINUTES_PER_DAY),
                    _ => return true,
                }
            }
            _ => match self.date_target(c) {
                Some(t) => t,
                None => return true,
            },
        };
        let end = d.end.unwrap_or(d.start);
        let overlaps = d.start < hi && end >= lo;
        // A date without a time is the whole day: compare by days.
        match c.op.as_str() {
            "is" | "between" => overlaps,
            "before" => d.start < lo,
            "after" => end >= hi,
            "onOrBefore" => d.start < hi,
            "onOrAfter" => end >= lo,
            "isNot" => !overlaps,
            _ => true,
        }
    }

    // ---- sorting and grouping ----

    fn sort_key(&self, db: &Database, row: &Row, prop: &Property) -> Val {
        let v = self.value(db, row, prop).unwrap_or(Val::Null);
        match prop.kind {
            PropKind::Select | PropKind::Status => match &v {
                Val::Str(n) => Val::Num(prop.options.iter().position(|o| &o.name == n).map_or(f64::MAX, |i| i as f64)),
                _ => Val::Null,
            },
            PropKind::MultiSelect => match &v {
                Val::List(l) if !l.is_empty() => Val::Num(l.iter().filter_map(|x| prop.options.iter().position(|o| o.name == x.text())).min().map_or(f64::MAX, |i| i as f64)),
                _ => Val::Null,
            },
            PropKind::Id => Val::Num(row.number as f64),
            _ => v,
        }
    }

    pub fn sorted<'r>(&self, db: &Database, rows: Vec<&'r Row>, sorts: &[Sort]) -> Vec<&'r Row> {
        if sorts.is_empty() {
            return rows; // already in manual order
        }
        let props: Vec<(&Property, bool)> = sorts.iter().filter_map(|s| db.prop(&s.prop).map(|p| (p, s.descending))).collect();
        let keys: Vec<Vec<Val>> = rows.iter().map(|r| props.iter().map(|(p, _)| self.sort_key(db, r, p)).collect()).collect();
        let mut positions: Vec<usize> = (0..rows.len()).collect();
        // Stable sort preserves the incoming manual order for ties, including missing sort props.
        // The same key generation and comparison (especially empty-last in BOTH directions)
        // apply; comparisons now index positions instead of hashing row ids repeatedly.
        positions.sort_by(|&a, &b| {
            let (ka, kb) = (&keys[a], &keys[b]);
            for (i, (_, desc)) in props.iter().enumerate() {
                let (x, y) = (&ka[i], &kb[i]);
                let o = match (x, y) {
                    (Val::Null, Val::Null) => std::cmp::Ordering::Equal,
                    (Val::Null, _) => std::cmp::Ordering::Greater, // empty last in both directions
                    (_, Val::Null) => std::cmp::Ordering::Less,
                    _ => {
                        let o = compare(x, y);
                        if *desc {
                            o.reverse()
                        } else {
                            o
                        }
                    }
                };
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
            }
            std::cmp::Ordering::Equal
        });
        positions.into_iter().map(|i| rows[i]).collect()
    }

    fn group_keys(&self, db: &Database, row: &Row, prop: &Property, g: &GroupBy) -> Vec<(String, String)> {
        // (key, label)
        let none = || vec![(String::new(), format!("No {}", prop.name))];
        let v = self.value(db, row, prop).unwrap_or(Val::Null);
        match prop.kind {
            PropKind::Select | PropKind::Status => match &v {
                Val::Str(n) => {
                    let o = prop.options.iter().find(|o| &o.name == n);
                    if prop.kind == PropKind::Status && g.date_by == "group" {
                        let grp = o.map(|o| o.group.as_str()).unwrap_or("todo");
                        return vec![(grp.into(), match grp { "inProgress" => "In progress", "complete" => "Complete", _ => "To-do" }.into())];
                    }
                    o.map(|o| vec![(o.id.clone(), o.name.clone())]).unwrap_or_else(none)
                }
                _ => none(),
            },
            PropKind::MultiSelect => match &v {
                Val::List(l) if !l.is_empty() => l.iter().filter_map(|x| prop.options.iter().find(|o| o.name == x.text())).map(|o| (o.id.clone(), o.name.clone())).collect(),
                _ => none(),
            },
            PropKind::Checkbox => vec![if v.as_bool() { ("true".into(), "Checked".into()) } else { ("false".into(), "Unchecked".into()) }],
            _ => match &v {
                Val::Null => none(),
                Val::Date(d) => {
                    let (lo, label) = match g.date_by.as_str() {
                        "week" => {
                            let s = start_of_week(d.start);
                            (s, format!("Week of {}", format_pattern(s, "MMM D, YYYY")))
                        }
                        "month" => {
                            let s = start_of_month(d.start);
                            (s, format_pattern(s, "MMMM YYYY"))
                        }
                        "year" => {
                            let s = start_of_year(d.start);
                            (s, format_pattern(s, "YYYY"))
                        }
                        "relative" => {
                            let today = start_of_day(self.now);
                            let dd = start_of_day(d.start);
                            let rank = if dd == today {
                                (0, "Today")
                            } else if dd == today + MINUTES_PER_DAY {
                                (1, "Tomorrow")
                            } else if dd == today - MINUTES_PER_DAY {
                                (-1, "Yesterday")
                            } else if dd >= start_of_week(today) && dd < start_of_week(today) + 7 * MINUTES_PER_DAY {
                                (2, "This week")
                            } else if dd >= start_of_week(today) + 7 * MINUTES_PER_DAY && dd < start_of_week(today) + 14 * MINUTES_PER_DAY {
                                (3, "Next week")
                            } else if dd < today {
                                (-2, "Earlier")
                            } else {
                                (4, "Later")
                            };
                            return vec![(format!("{:+03}", rank.0), rank.1.into())];
                        }
                        _ => {
                            let s = start_of_day(d.start);
                            (s, format_pattern(s, "MMM D, YYYY"))
                        }
                    };
                    vec![(format!("{:020}", lo + (1i64 << 40)), label)]
                }
                Val::Num(n) => {
                    if g.number_step > 0.0 {
                        let lo = (n / g.number_step).floor() * g.number_step;
                        vec![(format!("{:020.6}", lo + 1e12), format!("{} – {}", num_str(lo), num_str(lo + g.number_step)))]
                    } else {
                        vec![(format!("{:020.6}", n + 1e12), num_str(*n))]
                    }
                }
                Val::List(l) if !l.is_empty() => l.iter().map(|x| (x.text().to_lowercase(), x.text())).collect(),
                other => {
                    let t = other.text();
                    if t.is_empty() {
                        none()
                    } else {
                        vec![(t.to_lowercase(), t)]
                    }
                }
            },
        }
    }

    pub fn build_groups(&self, db: &Database, rows: &[&Row], g: &GroupBy) -> Vec<GroupOut> {
        let prop = match db.prop(&g.prop) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let mut order: Vec<String> = Vec::new();
        let mut map: HashMap<String, GroupOut> = HashMap::new();
        // Select-like groups exist even when empty (board columns), in option order.
        let seed: Vec<(String, String, String)> = match prop.kind {
            PropKind::Select | PropKind::MultiSelect => prop.options.iter().map(|o| (o.id.clone(), o.name.clone(), o.color.clone())).collect(),
            PropKind::Status if g.date_by == "group" => vec![("todo".into(), "To-do".into(), "gray".into()), ("inProgress".into(), "In progress".into(), "blue".into()), ("complete".into(), "Complete".into(), "green".into())],
            PropKind::Status => prop.options.iter().map(|o| (o.id.clone(), o.name.clone(), o.color.clone())).collect(),
            PropKind::Checkbox => vec![("false".into(), "Unchecked".into(), String::new()), ("true".into(), "Checked".into(), String::new())],
            _ => Vec::new(),
        };
        let seeded = !seed.is_empty();
        for (k, label, color) in seed {
            order.push(k.clone());
            map.insert(k.clone(), GroupOut { key: k, label, color, count: 0, rows: Vec::new(), subs: Vec::new(), calcs: Map::new() });
        }
        if seeded && matches!(prop.kind, PropKind::Select | PropKind::Status | PropKind::MultiSelect) {
            order.insert(0, String::new());
            map.insert(String::new(), GroupOut { key: String::new(), label: format!("No {}", prop.name), color: String::new(), count: 0, rows: Vec::new(), subs: Vec::new(), calcs: Map::new() });
        }
        for r in rows {
            for (k, label) in self.group_keys(db, r, prop, g) {
                let color = prop.option(&k).map(|o| o.color.clone()).unwrap_or_default();
                let e = map.entry(k.clone()).or_insert_with(|| {
                    order.push(k.clone());
                    GroupOut { key: k.clone(), label, color, count: 0, rows: Vec::new(), subs: Vec::new(), calcs: Map::new() }
                });
                e.rows.push(r.id.clone());
                e.count += 1;
            }
        }
        let manual = g.sort.is_empty() || g.sort == "manual";
        let mut out: Vec<GroupOut> = order.into_iter().filter_map(|k| map.remove(&k)).collect();
        let is_optioned = seeded;
        if !manual || !is_optioned {
            // Alphabetical (or chronological / numeric by key), "No value" last.
            out.sort_by(|a, b| {
                if a.key.is_empty() != b.key.is_empty() {
                    return if a.key.is_empty() { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less };
                }
                let o = if matches!(prop.kind, PropKind::Date | PropKind::Number | PropKind::CreatedTime | PropKind::LastEditedTime | PropKind::Formula | PropKind::Rollup) { a.key.cmp(&b.key) } else { natural_cmp(&a.label, &b.label) };
                if g.sort == "desc" {
                    o.reverse()
                } else {
                    o
                }
            });
        }
        if g.hide_empty {
            out.retain(|x| x.count > 0);
        }
        out.retain(|x| !g.hidden.contains(&x.key));
        out
    }

    pub fn row_out_pub(&self, db: &Database, row: &Row) -> RowOut {
        self.row_out(db, row)
    }

    fn row_out(&self, db: &Database, row: &Row) -> RowOut {
        let cells = self.cells_of(row);
        let mut values = Map::new();
        for p in &db.props {
            if p.kind == PropKind::Title {
                continue;
            }
            let stored_json = cells.get(&p.id);
            let v = match p.kind {
                PropKind::Formula | PropKind::Rollup | PropKind::CreatedTime | PropKind::LastEditedTime | PropKind::CreatedBy | PropKind::LastEditedBy | PropKind::Id => {
                    Some(computed_json(p, self.value(db, row, p)))
                }
                _ => stored_json.cloned(),
            };
            if let Some(v) = v {
                values.insert(p.id.clone(), v);
            }
        }
        RowOut { id: row.id.clone(), title: row.title.clone(), icon: row.icon.clone(), number: row.number, created: row.created, updated: row.updated, favorite: row.favorite, values }
    }

    /// Runs a view: filter, sort, group, calculations.
    pub fn run_view(&self, db: &Database, view: &View) -> ViewResult {
        self.run_view_with_limit(db, view, MAX_ROWS_PER_DATABASE)
    }

    // The frozen large fixture includes truncated groups and footers. It checks
    // the engine with the historical limit at the same point in the pipeline.
    #[cfg(test)]
    pub(crate) fn run_view_with_old_limit(&self, db: &Database, view: &View) -> ViewResult {
        self.run_view_with_limit(db, view, 50_000)
    }

    fn run_view_with_limit(&self, db: &Database, view: &View, limit: usize) -> ViewResult {
        let all = self.rows_of(&db.id);
        let filtered: Vec<&Row> = all.iter().copied().filter(|r| view.filter.is_empty() || self.matches(db, r, &view.filter)).collect();
        let mut sorted = self.sorted(db, filtered, &view.sorts);
        let total = all.len();
        let truncated = sorted.len() > limit;
        sorted.truncate(limit);
        let rows: Vec<RowOut> = sorted.iter().map(|r| self.row_out(db, r)).collect();
        let calcs = self.footer(db, &view.calcs, &sorted);
        let groups = view.group.as_ref().map(|g| {
            let mut gs = self.build_groups(db, &sorted, g);
            for grp in &mut gs {
                let members: Vec<&Row> = grp.rows.iter().filter_map(|id| self.by_id.get(id.as_str()).copied()).collect();
                grp.calcs = self.footer(db, &view.calcs, &members);
                if let Some(sg) = &view.sub_group {
                    grp.subs = self.build_groups(db, &members, sg);
                    for s in &mut grp.subs {
                        let m2: Vec<&Row> = s.rows.iter().filter_map(|id| self.by_id.get(id.as_str()).copied()).collect();
                        s.calcs = self.footer(db, &view.calcs, &m2);
                    }
                }
            }
            gs
        });
        ViewResult { rows, total, filtered: sorted.len(), truncated, groups, calcs }
    }

    pub fn footer(&self, db: &Database, calcs: &[ColCalc], rows: &[&Row]) -> Map<String, Value> {
        let mut out = Map::new();
        for c in calcs {
            if c.calc.is_empty() || c.calc == "none" {
                continue;
            }
            let p = match db.prop(&c.prop) {
                Some(p) => p,
                None => continue,
            };
            let vals: Vec<Val> = rows.iter().map(|r| self.value(db, r, p).unwrap_or(Val::Null)).collect();
            let res = calc(&c.calc, &vals);
            let shown = match (&res, c.calc.as_str()) {
                (Val::Num(n), "sum" | "avg" | "median" | "min" | "max" | "range") => format_number(*n, &p.number_format),
                (Val::Date(d), _) => format_date(d, &p.date_format),
                _ => calc_display(&c.calc, &res),
            };
            out.insert(c.prop.clone(), Value::String(shown));
        }
        out
    }
}

pub fn val_json(v: &Val) -> Value {
    match v {
        Val::Null => Value::Null,
        Val::Bool(b) => json!(b),
        Val::Num(n) => json!(n),
        Val::Str(s) => json!(s),
        Val::Date(d) => json!({"start": format_iso(d.start, d.has_time), "end": d.end.map(|e| format_iso(e, d.has_time)), "hasTime": d.has_time}),
        Val::List(l) => Value::Array(l.iter().map(val_json).collect()),
    }
}

/// How a computed property reaches the UI: its type, JSON value and the text to show.
pub fn computed_json(p: &Property, v: Result<Val, String>) -> Value {
    match v {
        Err(e) => json!({"t": "error", "v": Value::Null, "text": format!("⚠ {e}")}),
        Ok(v) => {
            let text = match &v {
                Val::Num(n) => format_number(*n, &p.number_format),
                Val::Date(d) => format_date(d, &p.date_format),
                other => other.text(),
            };
            json!({"t": v.type_name(), "v": val_json(&v), "text": text})
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RowOut {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub number: i64,
    pub created: i64,
    pub updated: i64,
    pub favorite: bool,
    /// property id -> stored JSON for stored kinds; {t, v, text} for computed kinds.
    pub values: Map<String, Value>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupOut {
    pub key: String,
    pub label: String,
    pub color: String,
    pub count: usize,
    pub rows: Vec<String>,
    pub subs: Vec<GroupOut>,
    pub calcs: Map<String, Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewResult {
    pub rows: Vec<RowOut>,
    /// Rows in the database / rows that passed the filter / the cut-off warning.
    pub total: usize,
    pub filtered: usize,
    pub truncated: bool,
    pub groups: Option<Vec<GroupOut>>,
    pub calcs: Map<String, Value>,
}

pub(crate) fn zeroize_json_value(v: &mut Value) {
    match v {
        Value::Null => {}
        Value::Bool(_) | Value::Number(_) => {
            *v = Value::Null;
        }
        Value::String(s) => {
            s.zeroize();
            *v = Value::Null;
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                zeroize_json_value(item);
            }
            items.clear();
            *v = Value::Null;
        }
        Value::Object(map) => {
            zeroize_json_map(map);
            *v = Value::Null;
        }
    }
}

pub(crate) fn zeroize_json_map(map: &mut Map<String, Value>) {
    for (mut k, mut v) in std::mem::take(map) {
        k.zeroize();
        zeroize_json_value(&mut v);
    }
}

impl Zeroize for RowOut {
    fn zeroize(&mut self) {
        self.id.zeroize();
        self.title.zeroize();
        self.icon.zeroize();
        self.number.zeroize();
        self.created.zeroize();
        self.updated.zeroize();
        self.favorite.zeroize();
        zeroize_json_map(&mut self.values);
    }
}

impl Drop for RowOut {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl Zeroize for GroupOut {
    fn zeroize(&mut self) {
        self.key.zeroize();
        self.label.zeroize();
        self.color.zeroize();
        self.count.zeroize();
        self.rows.zeroize();
        self.subs.zeroize();
        zeroize_json_map(&mut self.calcs);
    }
}

impl Drop for GroupOut {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl Zeroize for ViewResult {
    fn zeroize(&mut self) {
        self.rows.zeroize();
        self.total.zeroize();
        self.filtered.zeroize();
        self.truncated.zeroize();
        if let Some(groups) = &mut self.groups {
            groups.zeroize();
        }
        self.groups = None;
        zeroize_json_map(&mut self.calcs);
    }
}

impl Drop for ViewResult {
    fn drop(&mut self) {
        self.zeroize();
    }
}
