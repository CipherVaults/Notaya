//! Works out what a database view shows: stored and computed values (formulas, rollups, created
//! and edited times, IDs), filters, sorts, groups and column calculations. It runs in the Rust
//! core; the UI receives finished rows. Nothing here touches the vault or the disk.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use serde::Serialize;
use serde_json::{json, value::RawValue, Map, Value};
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
    cells: RefCell<HashMap<String, Rc<RowCells>>>,
    memo: RefCell<HashMap<(String, String), Result<Val, String>>>,
    /// Parsed formulas (including parse errors) live only for this query, keyed by exact text.
    formulas: RefCell<HashMap<String, Result<Rc<formula::Expr>, String>>>,
    busy: RefCell<HashSet<(String, String)>>,
    /// How many formula or rollup values are being worked out inside one another right now.
    depth: std::cell::Cell<usize>,
}

struct RowCells {
    values: Rc<Map<String, Value>>,
    numbers: HashMap<String, String>,
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
        self.row_cells(row).values.clone()
    }

    fn row_cells(&self, row: &Row) -> Rc<RowCells> {
        if let Some(cells) = self.cells.borrow().get(&row.id) {
            return cells.clone();
        }
        // Read number tokens before serde_json converts them to binary64. Keep the regular
        // JSON values for other cell types and consumers of the existing row payload.
        let raw = serde_json::from_str::<BTreeMap<String, &RawValue>>(&row.cells).unwrap_or_default();
        let mut values = Map::new();
        let mut numbers = HashMap::new();
        for (id, raw) in raw {
            let text = raw.get();
            if text.starts_with(|c: char| c == '-' || c.is_ascii_digit()) {
                numbers.insert(id.clone(), text.to_owned());
            }
            if let Ok(value) = serde_json::from_str(text) {
                values.insert(id, value);
            }
        }
        let cells = Rc::new(RowCells { values: Rc::new(values), numbers });
        self.cells.borrow_mut().insert(row.id.clone(), cells.clone());
        cells
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
            PropKind::Number => match self.row_cells(row).numbers.get(&prop.id) {
                Some(n) => Number::decimal(n).map(Val::Num),
                _ => Ok(Val::Null),
            },
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
            PropKind::Number | PropKind::Formula | PropKind::Rollup | PropKind::Button => Val::Null,
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
        calc(calc_name, &vals)
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
                Val::Num(n) => self.num_test(n, c),
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

    fn num_test(&self, n: &Number, c: &Cond) -> bool {
        let test = match c.op.as_str() {
            "eq" | "equals" | "is" => std::cmp::Ordering::is_eq,
            "neq" | "notEquals" | "isNot" => std::cmp::Ordering::is_ne,
            "gt" => std::cmp::Ordering::is_gt,
            "gte" => std::cmp::Ordering::is_ge,
            "lt" => std::cmp::Ordering::is_lt,
            "lte" => std::cmp::Ordering::is_le,
            _ => return true,
        };
        let Ok(w) = Number::decimal(&c.value) else { return true };
        // This filter API returns a boolean. An unresolved comparison cannot match.
        with_budget(|charge| n.cmp_checked(&w, charge)).is_ok_and(test)
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
                Val::Str(n) => Val::integer(prop.options.iter().position(|o| &o.name == n).unwrap_or(usize::MAX)),
                _ => Val::Null,
            },
            PropKind::MultiSelect => match &v {
                Val::List(l) if !l.is_empty() => Val::integer(l.iter().filter_map(|x| prop.options.iter().position(|o| o.name == x.text())).min().unwrap_or(usize::MAX)),
                _ => Val::Null,
            },
            PropKind::Id => Val::integer(row.number),
            _ => v,
        }
    }

    pub fn sorted<'r>(&self, db: &Database, rows: Vec<&'r Row>, sorts: &[Sort]) -> Vec<&'r Row> {
        if sorts.is_empty() {
            return rows; // already in manual order
        }
        let props: Vec<(&Property, bool)> = sorts.iter().filter_map(|s| db.prop(&s.prop).map(|p| (p, s.descending))).collect();
        let mut keys: Vec<Vec<Val>> = rows.iter().map(|r| {
            props.iter().map(|(p, _)| self.sort_key(db, r, p)).collect()
        }).collect();
        refine_sort_keys(&mut keys, props.len());
        let mut positions: Vec<usize> = (0..rows.len()).collect();
        // Stable sort preserves the incoming manual order for ties, including missing sort props.
        // The same key generation and comparison (especially empty-last in BOTH directions)
        // apply; comparisons now index positions instead of hashing row ids repeatedly.
        positions.sort_by(|&a, &b| {
            let (ka, kb) = (&keys[a], &keys[b]);
            for (i, (_, desc)) in props.iter().enumerate() {
                let (x, y) = (&ka[i], &kb[i]);
                let mut o = compare_sort_val(x, y);
                if *desc && !matches!(x, Val::Null) && !matches!(y, Val::Null) {
                    o = o.reverse();
                }
                if o != Ordering::Equal {
                    return o;
                }
            }
            Ordering::Equal
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
                    let n = match n.try_as_f64() {
                        Ok(value) => value,
                        Err(error) => return vec![("~numeric-error".into(), format!("\u{26a0} {error}"))],
                    };
                    if g.number_step > 0.0 {
                        let lo = (n / g.number_step).floor() * g.number_step;
                        vec![(format!("{:020.6}", lo + 1e12), format!("{} – {}", num_str(lo), num_str(lo + g.number_step)))]
                    } else {
                        vec![(format!("{:020.6}", n + 1e12), num_str(n))]
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
        let cells = self.row_cells(row);
        let mut values = Map::new();
        let mut number_text = BTreeMap::new();
        let mut number_input = BTreeMap::new();
        for p in &db.props {
            if p.kind == PropKind::Title {
                continue;
            }
            let stored_json = cells.values.get(&p.id);
            let v = match p.kind {
                PropKind::Formula | PropKind::Rollup | PropKind::CreatedTime | PropKind::LastEditedTime | PropKind::CreatedBy | PropKind::LastEditedBy | PropKind::Id => {
                    Some(computed_json(p, self.value(db, row, p)))
                }
                PropKind::Number => match cells.numbers.get(&p.id) {
                    Some(input) => match Number::decimal(input) {
                        Ok(n) => {
                            number_input.insert(p.id.clone(), input.clone());
                            match format_number(&n, &p.number_format) {
                                Ok(text) => {
                                    number_text.insert(p.id.clone(), text);
                                    Some(match stored_json {
                                        Some(value) => value.clone(),
                                        None => match n.try_as_f64() {
                                            Ok(value) => json!(value),
                                            Err(error) => computed_json(p, Err(error)),
                                        },
                                    })
                                }
                                Err(e) => Some(computed_json(p, Err(e))),
                            }
                        }
                        Err(e) => Some(computed_json(p, Err(e))),
                    },
                    None => stored_json.cloned(),
                },
                _ => stored_json.cloned(),
            };
            if let Some(v) = v {
                values.insert(p.id.clone(), v);
            }
        }
        RowOut { id: row.id.clone(), title: row.title.clone(), icon: row.icon.clone(), number: row.number, created: row.created, updated: row.updated, favorite: row.favorite, values, number_text, number_input }
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
            let shown = self.calculate(db, rows, p, &c.calc).and_then(|res| match res {
                Val::Date(d) => Ok(format_date(&d, &p.date_format)),
                res => calc_property_display(&c.calc, &res, p),
            }).unwrap_or_else(|e| format!("⚠ {e}"));
            out.insert(c.prop.clone(), Value::String(shown));
        }
        out
    }

    pub(crate) fn calculate(&self, db: &Database, rows: &[&Row], prop: &Property, name: &str) -> Result<Val, String> {
        if matches!(name, "countAll" | "count") {
            return Ok(Val::integer(rows.len()));
        }
        let vals = rows.iter().map(|r| self.value(db, r, prop)).collect::<Result<Vec<_>, _>>()?;
        calc(name, &vals)
    }
}

fn compare_sort_val(x: &Val, y: &Val) -> Ordering {
    match (x, y) {
        (Val::Null, Val::Null) => Ordering::Equal,
        (Val::Null, _) => Ordering::Greater,
        (_, Val::Null) => Ordering::Less,
        _ => compare(x, y),
    }
}

fn refine_sort_keys(keys: &mut [Vec<Val>], columns: usize) {
    let mut tie_groups: Vec<Vec<usize>> = if keys.len() > 1 {
        vec![(0..keys.len()).collect()]
    } else {
        Vec::new()
    };
    for col in 0..columns {
        if tie_groups.is_empty() {
            break;
        }
        // Each row gets a fresh budget. All rows in this column share certified witnesses.
        let mut remaining = vec![MAX_SORTS * Number::MAX_ARITHMETIC_WORK; keys.len()];
        let mut column_memo: Vec<(Number, bool)> = Vec::new();
        for group in &tie_groups {
            refine_val_group(keys, col, &[], group, &mut remaining, &mut column_memo);
        }
        let mut next_groups = Vec::new();
        for group in tie_groups {
            extend_sort_tie_groups(group, |a, b| compare_sort_val(&keys[a][col], &keys[b][col]), &mut next_groups);
        }
        tie_groups = next_groups;
    }
}

fn extend_sort_tie_groups(
    mut group: Vec<usize>,
    compare_rows: impl Fn(usize, usize) -> Ordering,
    ties: &mut Vec<Vec<usize>>,
) {
    // Stable sorting preserves ascending row indices within each equal run.
    group.sort_by(|&a, &b| compare_rows(a, b));
    ties.extend(
        group.chunk_by(|&a, &b| compare_rows(a, b) == Ordering::Equal)
            .filter(|run| run.len() > 1)
            .map(<[usize]>::to_vec),
    );
}

fn sort_key_at<'a>(keys: &'a [Vec<Val>], row: usize, col: usize, path: &[usize]) -> &'a Val {
    let mut key = &keys[row][col];
    for &pos in path {
        let Val::List(list) = key else { unreachable!("sort path must traverse lists") };
        key = &list[pos];
    }
    key
}

const MAX_SORT_COLUMN_MEMO: usize = 256;
type SortNumber<'a> = (usize, &'a mut Number, bool, usize, usize, Option<Number>);

fn strictly_tighter(a: &Number, b: &Number) -> bool {
    (a.lower() > b.lower() || a.upper() < b.upper())
        && a.lower() >= b.lower()
        && a.upper() <= b.upper()
}

fn refine_val_group(
    keys: &mut [Vec<Val>],
    col: usize,
    path: &[usize],
    group: &[usize],
    remaining: &mut [usize],
    column_memo: &mut Vec<(Number, bool)>,
) {
    let mut numbers = Vec::new();
    let mut list_rows = Vec::new();
    let mut max_len = 0;
    let mut rows = keys.iter_mut();
    let mut next_row = 0;
    for &row in group {
        // Tie groups keep row indices in ascending order, so each borrow is disjoint.
        let mut key = &mut rows.nth(row - next_row).expect("sort row in bounds")[col];
        next_row = row + 1;
        for &pos in path {
            let Val::List(list) = key else { unreachable!("sort path must traverse lists") };
            key = &mut list[pos];
        }
        match key {
            Val::Num(n) => {
                // Exact points can overlap an interval and need no replay.
                let refinable = !n.is_exact();
                let initial = remaining[row].min(Number::MAX_ARITHMETIC_WORK);
                numbers.push((row, n, refinable, initial, initial, None));
            }
            Val::List(list) => {
                list_rows.push(row);
                max_len = max_len.max(list.len());
            }
            _ => {}
        }
    }
    if numbers.len() > 1 && numbers.iter().any(|entry| entry.2) {
        for (_, n, refinable, _, _, before) in &mut numbers {
            if !*refinable {
                continue;
            }
            if let Some((cached, terminal)) = column_memo.iter().find(|(cached, _)| cached.same_expr(n)) {
                **n = cached.clone();
                *refinable = !terminal;
            }
            *before = Some((**n).clone());
        }
        for target_bits in [448, 896, 1792, 3584, 4096] {
            if !refine_overlapping_sort_numbers(&mut numbers, target_bits) {
                break;
            }
        }
        for (row, n, refinable, initial, local_remaining, before) in &numbers {
            if initial == local_remaining {
                continue;
            }
            remaining[*row] -= initial - local_remaining;
            // A later row may have more work available or start from tighter bounds.
            let tightened = before.as_ref().is_some_and(|before| strictly_tighter(n, before));
            let terminal = !refinable && *initial == Number::MAX_ARITHMETIC_WORK && !tightened;
            if let Some(slot) = column_memo.iter_mut().find(|(cached, _)| cached.same_expr(n)) {
                if strictly_tighter(n, &slot.0) {
                    *slot = ((**n).clone(), false);
                } else if n.lower() == slot.0.lower() && n.upper() == slot.0.upper() {
                    slot.1 |= terminal;
                }
            } else if column_memo.len() < MAX_SORT_COLUMN_MEMO {
                column_memo.push(((**n).clone(), terminal));
            }
        }
        canonicalize_sort_numbers(&mut numbers, column_memo);
    }

    if list_rows.len() > 1 {
        let mut list_groups = vec![list_rows];
        let mut child_path = path.to_vec();
        child_path.push(0);
        for pos in 0..max_len {
            list_groups.retain_mut(|group| {
                group.retain(|&row| matches!(sort_key_at(keys, row, col, path), Val::List(list) if list.len() > pos));
                group.len() > 1
            });
            if list_groups.is_empty() {
                break;
            }
            child_path[path.len()] = pos;
            let mut next_groups = Vec::new();
            for group in list_groups {
                refine_val_group(keys, col, &child_path, &group, remaining, column_memo);
                extend_sort_tie_groups(
                    group,
                    |a, b| compare(sort_key_at(keys, a, col, &child_path), sort_key_at(keys, b, col, &child_path)),
                    &mut next_groups,
                );
            }
            list_groups = next_groups;
        }
    }
}

fn overlapping_sort_cluster_len(entries: &[SortNumber<'_>]) -> usize {
    let mut max_upper = entries[0].1.upper();
    for (index, entry) in entries.iter().enumerate().skip(1) {
        let next = &entry.1;
        if next.lower() > max_upper {
            return index;
        }
        max_upper = max_upper.max(next.upper());
    }
    entries.len()
}

fn canonicalize_sort_numbers(mut entries: &mut [SortNumber<'_>], column_memo: &[(Number, bool)]) {
    entries.sort_unstable_by(|a, b| a.1.lower().cmp(b.1.lower()));
    while !entries.is_empty() {
        let end = overlapping_sort_cluster_len(entries);
        let (cluster, rest) = entries.split_at_mut(end);
        entries = rest;
        if cluster.len() < 2 {
            continue;
        }

        // Copies of one witness always overlap. Give each class one sort key even
        // when its rows could afford different tiers or the column memo is full.
        let mut assigned = vec![false; cluster.len()];
        let mut peers = Vec::new();
        for index in 0..cluster.len() {
            let n = &*cluster[index].1;
            if assigned[index] || n.is_exact() || !n.same_expr(n) {
                continue;
            }
            let mut best = n;
            if let Some((cached, _)) = column_memo.iter().find(|(cached, _)| {
                cached.same_expr(n) && strictly_tighter(cached, n)
            }) {
                best = cached;
            }
            for (other, entry) in cluster.iter().enumerate().skip(index + 1) {
                if !assigned[other] && n.same_expr(entry.1) {
                    assigned[other] = true;
                    peers.push(other);
                    if strictly_tighter(entry.1, best) {
                        best = entry.1;
                    }
                }
            }
            if !peers.is_empty() || strictly_tighter(best, n) {
                let best = best.clone();
                *cluster[index].1 = best.clone();
                for other in peers.drain(..) {
                    *cluster[other].1 = best.clone();
                }
            }
        }
    }
}

fn refine_overlapping_sort_numbers(mut entries: &mut [SortNumber<'_>], target_bits: u32) -> bool {
    entries.sort_unstable_by(|a, b| a.1.lower().cmp(b.1.lower()));
    let mut active = false;
    while !entries.is_empty() {
        let end = overlapping_sort_cluster_len(entries);
        let (cluster, rest) = entries.split_at_mut(end);
        entries = rest;
        if cluster.len() > 1 && cluster.iter().any(|entry| entry.2) {
            let first = &cluster[0].1;
            // Identical witnesses cannot separate. An overlapping exact
            // point still needs comparison against the interval members.
            if !cluster[1..].iter().all(|entry| first.same_expr(entry.1)) {
                for (_, n, refinable, _, local_remaining, _) in cluster {
                    if *refinable {
                        *refinable = n.refine_for_sort(target_bits, local_remaining);
                        active |= *refinable;
                    }
                }
            }
        }
    }
    active
}

pub fn val_json(v: &Val) -> Result<Value, String> {
    super::value::with_budget(|charge| val_json_checked(v, charge))
}

fn val_json_checked(v: &Val, charge: &mut Charge<'_>) -> Result<Value, String> {
    Ok(match v {
        Val::Null => Value::Null,
        Val::Bool(b) => json!(b),
        Val::Num(n) => json!(n.float_checked(charge)?),
        Val::Str(s) => json!(s),
        Val::Date(d) => json!({"start": format_iso(d.start, d.has_time), "end": d.end.map(|e| format_iso(e, d.has_time)), "hasTime": d.has_time}),
        Val::List(l) => Value::Array(l.iter().map(|v| val_json_checked(v, charge)).collect::<Result<_, _>>()?),
    })
}

/// How a computed property reaches the UI: its type, JSON value and the text to show.
pub fn computed_json(p: &Property, v: Result<Val, String>) -> Value {
    let projected = v.and_then(|v| {
        let text = match &v {
            Val::Num(n) => {
                if let Some((amount, text)) = currency_amount(n, &p.number_format)? {
                    let amount = amount.parse::<f64>().map_err(|_| "Numeric overflow")?;
                    if !amount.is_finite() {
                        return Err("Numeric overflow".into());
                    }
                    return Ok(json!({"t": "number", "v": amount, "text": text}));
                }
                format_number(n, &p.number_format)?
            }
            Val::Date(d) => format_date(d, &p.date_format),
            other => other.try_text()?,
        };
        Ok(json!({"t": v.type_name(), "v": val_json(&v)?, "text": text}))
    });
    projected.unwrap_or_else(|e| json!({"t": "error", "v": Value::Null, "text": format!("\u{26a0} {e}")}))
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
    /// Display and editor text derived from the original decimal in the stored number cell.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub number_text: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub number_input: BTreeMap<String, String>,
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
        for map in [&mut self.number_text, &mut self.number_input] {
            for (mut id, mut text) in std::mem::take(map) {
                id.zeroize();
                text.zeroize();
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_filters_require_a_certified_comparison() {
        let ctx = Ctx::new(std::iter::empty(), 0, 0);
        let mut charge = |_| Ok(());
        let center = Number::integer(1);
        let delta = Number::decimal("1e-100").unwrap();
        let root = center.add(&delta).unwrap().sqrt(&mut charge).unwrap();
        let mut condition = Cond { op: "gt".into(), value: "1".into(), ..Default::default() };
        assert!(ctx.num_test(&root, &condition));
        condition.op = "neq".into();
        condition.value = "0".into();
        assert!(!ctx.num_test(&root.sub(&root).unwrap(), &condition));
    }

    #[test]
    fn table_view_sorts_overlapping_formula_numbers_in_certified_order() {
        let mut db = Database::new("db".into(), "synthetic".into(), "Sort".into(), 0);
        db.props.extend([
            Property { id: "input".into(), name: "Input".into(), kind: PropKind::Number, ..Default::default() },
            Property {
                id: "amount".into(),
                kind: PropKind::Formula,
                formula: r#"if(prop("Input") == 0, sqrt(2), sqrt(2)*sqrt(2)*sqrt(2)/2 + 1e-100)"#.into(),
                ..Default::default()
            },
        ]);
        let mut larger = Row::new("larger".into(), "db", "Larger".into(), 0);
        larger.cells = r#"{"input":1}"#.into();
        let mut smaller = Row::new("smaller".into(), "db", "Smaller".into(), 0);
        smaller.order = 1;
        smaller.cells = r#"{"input":0}"#.into();
        let items = [Item::Database(db), Item::Row(larger), Item::Row(smaller)];
        let ctx = Ctx::new(items.iter(), 0, 0);
        let db = ctx.dbs["db"];
        assert_eq!(ctx.rows_of("db")[0].id, "larger");
        let view = View { sorts: vec![Sort { prop: "amount".into(), descending: false }], ..Default::default() };
        let result = ctx.run_view(db, &view);
        let order: Vec<_> = result.rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(order, ["smaller", "larger"]);
    }

    #[test]
    fn table_view_sorts_overlapping_formula_numbers_in_lists() {
        let root = CLOSE_SORT_ROOT;
        assert_formula_view_sort_permutations(&[&format!("[{root}]")]);
        assert_formula_view_sort_permutations(&[&format!(r#"["prefix", [true, [{root}, "suffix"]]]"#)]);
    }

    const CLOSE_SORT_ROOT: &str = r#"sqrt(1+(prop("Input")+1)*1e-100)"#;
    const DEEP_SORT_ROOT: &str = r#"sqrt(1+(prop("Input")+1)*1e-800)"#;
    const DISTINCT_EQUAL_SORT_PREFIX: &str = r#"if(prop("Input") == 1, sqrt(8)/2, sqrt(2))"#;
    const HEAVY_EQUAL_SORT_PREFIX: &str = r#"if(prop("Input") == 1, sqrt(4+4e-800)/2, sqrt(1+1e-800))"#;
    const REPEATED_SORT_PREFIX: &str = "p,p,p,p,p,p,p,p,p,p";
    const SHARED_SORT_ROOT: &str = "sqrt(1+1e-800)";

    #[test]
    fn table_view_sorts_deep_formula_numbers_with_three_columns() {
        assert_formula_view_sort_permutations(&[DEEP_SORT_ROOT; 3]);
    }

    #[test]
    fn table_view_sorts_deep_formula_numbers_with_ten_columns() {
        assert_formula_view_sort_permutations(&[DEEP_SORT_ROOT; 10]);
    }

    #[test]
    fn table_view_sorts_deep_formula_numbers_in_three_element_lists() {
        assert_formula_view_sort_permutations(&[&format!(
            "[{DEEP_SORT_ROOT}, {DEEP_SORT_ROOT}, {DEEP_SORT_ROOT}]"
        )]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_heavy_distinct_equal_prefixes() {
        assert_formula_view_sort_permutations(&[&format!(
            "[{HEAVY_EQUAL_SORT_PREFIX}, [{CLOSE_SORT_ROOT}]]"
        )]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_heavy_equal_prefix_at_400() {
        let suffix = r#"sqrt(1+(prop("Input")+1)*1e-400)"#;
        assert_formula_view_sort_permutations(&[&format!(
            "[{HEAVY_EQUAL_SORT_PREFIX}, [{suffix}]]"
        )]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_heavy_equal_prefix_at_800() {
        assert_formula_view_sort_permutations(&[&format!(
            "[{HEAVY_EQUAL_SORT_PREFIX}, [{DEEP_SORT_ROOT}]]"
        )]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_equal_sqrt_prefix() {
        assert_formula_view_sort_permutations(&[&format!("[sqrt(2), [{CLOSE_SORT_ROOT}]]")]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_equal_exp_prefix() {
        assert_formula_view_sort_permutations(&[&format!("[exp(1), [{CLOSE_SORT_ROOT}]]")]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_distinct_equal_prefixes() {
        assert_formula_view_sort_permutations(&[&format!(
            "[{DISTINCT_EQUAL_SORT_PREFIX}, [{CLOSE_SORT_ROOT}]]"
        )]);
    }

    #[test]
    fn table_view_sorts_numeric_suffix_in_second_column() {
        assert_formula_view_sort_permutations(&["sqrt(2)", CLOSE_SORT_ROOT]);
        assert_formula_view_sort_permutations(&[DISTINCT_EQUAL_SORT_PREFIX, CLOSE_SORT_ROOT]);
    }

    #[test]
    fn table_view_sorts_second_column_after_repeated_heavy_equal_witnesses() {
        let prefix = format!("let(p, {HEAVY_EQUAL_SORT_PREFIX}, [{REPEATED_SORT_PREFIX}])");
        assert_formula_view_sort_permutations(&[&prefix, DEEP_SORT_ROOT]);
    }

    #[test]
    fn table_view_sorts_second_column_after_ten_distinct_heavy_equal_witnesses() {
        let witnesses = (1..=10).map(|count| format!("p{}", "+0".repeat(count))).collect::<Vec<_>>();
        let prefix = format!("let(p, {HEAVY_EQUAL_SORT_PREFIX}, [{}])", witnesses.join(", "));
        assert_formula_view_sort_permutations(&[&prefix, DEEP_SORT_ROOT]);
    }

    #[test]
    fn table_view_sorts_nested_suffix_after_repeated_heavy_equal_witnesses() {
        let list = format!("let(p, {HEAVY_EQUAL_SORT_PREFIX}, [{REPEATED_SORT_PREFIX}, [{DEEP_SORT_ROOT}]])");
        assert_formula_view_sort_permutations(&[&list]);
    }

    #[test]
    fn table_view_reuses_a_shared_witness_from_an_earlier_list_position() {
        let input = r#"prop("Input")"#;
        for suffix in [input, DEEP_SORT_ROOT] {
            let list = format!("[{HEAVY_EQUAL_SORT_PREFIX}, {SHARED_SORT_ROOT}, [{suffix}]]");
            assert_formula_view_sort_permutations(&[&list]);
        }
        let equal = format!("[{HEAVY_EQUAL_SORT_PREFIX}, {SHARED_SORT_ROOT}]");
        assert_formula_view_sort_cases(&[&equal], true);
    }

    fn asymmetric_budget_sort_list(suffix: &str) -> String {
        let prefixes: Vec<_> = (1..=10)
            .map(|count| format!("if(input == 0, q{}, h)", "+0".repeat(count)))
            .collect();
        let prefixes = prefixes.join(", ");
        format!(
            r#"lets(input, prop("Input"), q, {SHARED_SORT_ROOT}, h, sqrt(4+4e-800)/2,
                [{prefixes}, if(input == 2, sqrt(1+2e-800), q), [{suffix}]])"#
        )
    }

    #[test]
    fn table_view_canonicalizes_a_shared_witness_after_one_row_exhausts_its_budget() {
        for suffix in ["input", "sqrt(2+input)"] {
            assert_formula_view_sort_permutations(&[&asymmetric_budget_sort_list(suffix)]);
        }
    }

    #[test]
    fn sort_refinement_canonicalizes_shared_witnesses_even_with_a_full_memo() {
        let items = formula_sort_items(&[&asymmetric_budget_sort_list("input")], [0, 1, 2]);
        let ctx = Ctx::new(items.iter(), 0, 0);
        let db = ctx.dbs["db"];
        let prop = db.prop("col0").unwrap();
        let keys: Vec<_> = ctx.rows_of("db").iter()
            .map(|row| {
                let key = ctx.value(db, row, prop).unwrap();
                assert!(matches!(&key, Val::List(values) if values.len() == 12));
                vec![key]
            })
            .collect();
        let group = [0, 1, 2];
        // Supply the budgets at the shared leaf directly so the test exercises
        // canonicalization even when replay at the earlier prefixes changes.
        let column_budget = MAX_SORTS * Number::MAX_ARITHMETIC_WORK;
        let Val::Num(coarse) = sort_key_at(&keys, 0, 0, &[10]) else {
            panic!("expected shared root");
        };

        // The third row's nearby root triggers replay. Check that row A gets the
        // same bounds as row B even when the shared witness cannot be cached.
        for full_memo in [false, true] {
            let mut keys = keys.clone();
            let mut remaining = [0, column_budget, column_budget];
            let mut column_memo = Vec::new();
            if full_memo {
                column_memo.extend((column_memo.len()..MAX_SORT_COLUMN_MEMO).map(|index| {
                    (coarse.add(&Number::integer(index + 1)).unwrap(), false)
                }));
            }
            refine_val_group(&mut keys, 0, &[10], &group, &mut remaining, &mut column_memo);
            let a = sort_key_at(&keys, 0, 0, &[10]);
            let b = sort_key_at(&keys, 1, 0, &[10]);
            let (Val::Num(a), Val::Num(b)) = (a, b) else { panic!("expected shared roots") };
            assert!(a.same_expr(b));
            assert!(strictly_tighter(b, coarse));
            assert_eq!(a.cmp(b), Ordering::Equal);
            assert_eq!(remaining[0], 0);
            assert!(remaining[1] < column_budget);
            assert_eq!(column_memo.iter().any(|(n, _)| n.same_expr(a)), !full_memo);
            assert!(column_memo.len() <= MAX_SORT_COLUMN_MEMO);
        }
    }

    #[test]
    fn budget_starved_group_does_not_block_later_group_memo() {
        let root = |offset| {
            Number::integer(1).add(&Number::decimal(offset).unwrap()).unwrap()
                .sqrt(&mut |_| Ok(())).unwrap()
        };
        let shared = root("1e-800");
        let near = root("2e-800");
        let mut keys = [
            vec![Val::Num(shared.clone())],
            vec![Val::Num(near.clone())],
            vec![Val::Num(shared.clone())],
            vec![Val::Num(near)],
        ];
        let column_budget = MAX_SORTS * Number::MAX_ARITHMETIC_WORK;
        let mut remaining = [1, column_budget, column_budget, column_budget];
        let mut column_memo = Vec::new();

        refine_val_group(&mut keys, 0, &[], &[0, 1], &mut remaining, &mut column_memo);
        assert_eq!(remaining[0], 0);
        assert_eq!(
            column_memo.iter().find(|(n, _)| n.same_expr(&shared)),
            Some(&(shared.clone(), false)),
        );

        refine_val_group(&mut keys, 0, &[], &[2, 3], &mut remaining, &mut column_memo);
        assert!(remaining[2] < column_budget);
        assert_eq!(compare(&keys[2][0], &keys[3][0]), Ordering::Less);
        let (cached, terminal) = column_memo.iter().find(|(n, _)| n.same_expr(&shared)).unwrap();
        assert!(strictly_tighter(cached, &shared));
        assert!(!terminal);
    }

    #[test]
    fn a_full_budget_that_tightens_a_witness_does_not_mark_it_terminal() {
        let shared = Number::integer(1).add(&Number::decimal("1e-800").unwrap()).unwrap()
            .sqrt(&mut |_| Ok(())).unwrap();
        let equal = shared.add(&Number::integer(0)).unwrap();
        assert!(!shared.same_expr(&equal));
        let mut keys = [vec![Val::Num(shared.clone())], vec![Val::Num(equal)]];
        let leaf_budget = Number::MAX_ARITHMETIC_WORK;
        let mut remaining = [leaf_budget; 2];
        let mut column_memo = Vec::new();

        refine_val_group(&mut keys, 0, &[], &[0, 1], &mut remaining, &mut column_memo);
        assert_eq!(remaining[0], 0);
        let (cached, terminal) = column_memo.iter().find(|(n, _)| n.same_expr(&shared)).unwrap();
        assert!(strictly_tighter(cached, &shared));
        assert!(!terminal);
    }

    #[test]
    fn table_view_refines_a_shared_witness_after_a_budget_starved_group() {
        let prefixes = (1..=9).map(|count| {
            format!("if(input == 3, q{}, q{})", "+0".repeat(count), "-0".repeat(count))
        }).collect::<Vec<_>>().join(", ");
        // The first group's nested list finishes before the second group uses
        // the memo. F splits from D and E after the first prefix spends part
        // of their budgets. The next nine prefixes leave D short of work at Q.
        let formula = format!(
            r#"lets(input, prop("Input"), q, {SHARED_SORT_ROOT},
                if(input < 3, [1, {DEEP_SORT_ROOT}],
                    [0, [if(input == 5, sqrt(1+4e-800), sqrt(1+3e-800)),
                        {prefixes}, if(input == 3, q, 1)]]))"#
        );
        for manual in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            let mut items = formula_sort_items(&[&formula], manual);
            for (index, id) in [(3, "D"), (4, "E"), (5, "F")] {
                let mut row = Row::new(id.into(), "db", id.into(), 0);
                row.order = index;
                row.cells = format!(r#"{{"input":{index}}}"#);
                items.push(Item::Row(row));
            }
            let ctx = Ctx::new(items.iter(), 0, 0);
            for (descending, expected) in [(false, ["E", "D", "F", "A", "B", "C"]), (true, ["C", "B", "A", "F", "D", "E"])] {
                let view = View { sorts: vec![Sort { prop: "col0".into(), descending }], ..Default::default() };
                let result = ctx.run_view(ctx.dbs["db"], &view);
                let actual: Vec<_> = result.rows.iter().map(|row| row.id.as_str()).collect();
                assert_eq!(actual, expected, "manual order {manual:?}, descending {descending}");
            }
        }
    }

    #[test]
    fn sort_refinement_resumes_partially_refined_witness_at_a_later_list_position() {
        let list = format!(
            r#"[if(prop("Input") == 2, sqrt(1+1e-100), {SHARED_SORT_ROOT}),
                if(prop("Input") == 1, sqrt(1+2e-800), {SHARED_SORT_ROOT})]"#
        );
        assert_formula_view_sort_permutations(&[&list]);

        let root = |offset| {
            Number::integer(1).add(&Number::decimal(offset).unwrap()).unwrap()
                .sqrt(&mut |_| Ok(())).unwrap()
        };
        let first = root("1e-800");
        let far = root("1e-100");
        let near = root("2e-800");
        let mut keys = [
            vec![Val::List(vec![Val::Num(first.clone()), Val::Num(first.clone())])],
            vec![Val::List(vec![Val::Num(first.clone()), Val::Num(near.clone())])],
            vec![Val::List(vec![Val::Num(far), Val::Num(first.clone())])],
        ];
        let leaf_budget = Number::MAX_ARITHMETIC_WORK;
        let mut remaining = [MAX_SORTS * leaf_budget; 3];
        let mut column_memo = Vec::new();
        let mut expected = first;
        let mut expected_remaining = leaf_budget;
        assert!(expected.refine_for_sort(448, &mut expected_remaining));
        assert!(expected_remaining < leaf_budget);

        refine_val_group(&mut keys, 0, &[0], &[0, 1, 2], &mut remaining, &mut column_memo);
        assert_eq!(
            column_memo.iter().find(|(n, _)| n.same_expr(&expected)),
            Some(&(expected.clone(), false)),
        );
        assert_eq!(sort_key_at(&keys, 0, 0, &[0]), sort_key_at(&keys, 1, 0, &[0]));
        assert_eq!(compare(sort_key_at(&keys, 1, 0, &[0]), sort_key_at(&keys, 2, 0, &[0])), Ordering::Less);
        let before_suffix = remaining[0];
        assert_eq!(MAX_SORTS * leaf_budget - before_suffix, leaf_budget - expected_remaining);

        expected_remaining = leaf_budget;
        let mut near = near;
        let mut near_remaining = leaf_budget;
        assert!(near.refine_for_sort(448, &mut near_remaining));
        for bits in [896, 1792] {
            assert!(expected.refine_for_sort(bits, &mut expected_remaining));
            assert!(near.refine_for_sort(bits, &mut near_remaining));
        }
        assert!(expected.upper() >= near.lower());
        assert!(expected.refine_for_sort(3584, &mut expected_remaining));
        assert!(near.refine_for_sort(3584, &mut near_remaining));
        assert!(expected.upper() < near.lower());

        // Measure the work charged to the two rows that tied at position 0.
        refine_val_group(&mut keys, 0, &[1], &[0, 1], &mut remaining, &mut column_memo);
        assert_eq!(
            column_memo.iter().find(|(n, _)| n.same_expr(&expected)),
            Some(&(expected.clone(), false)),
        );
        assert_eq!(sort_key_at(&keys, 0, 0, &[1]), &Val::Num(expected));
        assert_eq!(sort_key_at(&keys, 1, 0, &[1]), &Val::Num(near));
        assert_eq!(before_suffix - remaining[0], leaf_budget - expected_remaining);
    }

    #[test]
    fn sort_refinement_spends_no_work_on_identical_dags() {
        let make_key = || {
            let mut root = Number::integer(2).sqrt(&mut |_| Ok(())).unwrap();
            for _ in 0..40 {
                root = root.add(&root).unwrap();
            }
            vec![Val::Num(root)]
        };
        // Build the DAGs separately so equality has to visit their shared children.
        let mut keys = [make_key(), make_key()];
        let budget = [Number::MAX_ARITHMETIC_WORK; 2];
        let mut remaining = budget;
        let mut column_memo = Vec::new();
        refine_val_group(&mut keys, 0, &[], &[0, 1], &mut remaining, &mut column_memo);
        assert_eq!(remaining, budget);
        assert!(column_memo.is_empty());
    }

    const SORT_ROW_IDS: [&str; 3] = ["A", "B", "C"];

    fn formula_sort_items(formulas: &[&str], order: [usize; 3]) -> Vec<Item> {
        let mut db = Database::new("db".into(), "synthetic".into(), "Sort".into(), 0);
        db.props.push(Property {
            id: "input".into(),
            name: "Input".into(),
            kind: PropKind::Number,
            ..Default::default()
        });
        db.props.extend(formulas.iter().enumerate().map(|(column, formula)| Property {
            id: format!("col{column}"),
            kind: PropKind::Formula,
            formula: (*formula).into(),
            ..Default::default()
        }));
        let mut items = vec![Item::Database(db)];
        for (position, index) in order.into_iter().enumerate() {
            let id = SORT_ROW_IDS[index];
            let mut row = Row::new(id.into(), "db", id.into(), 0);
            row.order = position as i64;
            row.cells = format!(r#"{{"input":{index}}}"#);
            items.push(Item::Row(row));
        }
        items
    }

    fn assert_formula_view_sort_permutations(formulas: &[&str]) {
        assert_formula_view_sort_cases(formulas, false);
    }

    fn assert_formula_view_sort_cases(formulas: &[&str], preserve_manual_order: bool) {
        let permutations = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
        for order in permutations {
            let items = formula_sort_items(formulas, order);
            let ctx = Ctx::new(items.iter(), 0, 0);
            let db = ctx.dbs["db"];
            let rows = ctx.rows_of("db");
            let manual = order.map(|index| SORT_ROW_IDS[index]);
            assert_eq!(rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(), manual);
            for row in rows {
                for prop in &db.props[1..] {
                    let key = ctx.value(db, row, prop);
                    assert!(matches!(key, Ok(Val::Num(_) | Val::List(_))), "{} on {}: {key:?}", prop.formula, row.id);
                }
            }
            for (descending, sorted) in [(false, SORT_ROW_IDS), (true, ["C", "B", "A"])] {
                let expected = if preserve_manual_order { manual } else { sorted };
                let view = View {
                    sorts: (0..formulas.len()).map(|column| Sort {
                        prop: format!("col{column}"),
                        descending,
                    }).collect(),
                    ..Default::default()
                };
                let result = ctx.run_view(db, &view);
                let actual: Vec<_> = result.rows.iter().map(|row| row.id.as_str()).collect();
                assert_eq!(actual, expected, "{formulas:?}, manual order {manual:?}, descending {descending}");
            }
        }
    }

    #[test]
    fn table_view_refines_shared_witnesses_and_orders_dropped_witnesses_consistently() {
        let a = "sqrt(2)";
        let c = "sqrt(2)*sqrt(2)*sqrt(2)/2 + 1e-100";
        let midpoint = format!("({a}+{c})/2");
        let shared = format!("lets(t,{midpoint},u,t+t,v,u+u,w,v+v,x,w+w,y,x+x,z,y+y,z/64)");
        // Each addition has a fresh exact leaf and a fresh operation node.
        let dropped = format!("{midpoint}{}-64", "+1".repeat(64));
        let permutations = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
        let ids = ["A", "B", "C"];
        let view = View { sorts: vec![Sort { prop: "amount".into(), descending: false }], ..Default::default() };
        for (b, has_witness) in [(shared, true), (dropped, false)] {
            let mut expected = has_witness.then(|| ids.map(str::to_string).to_vec());
            for order in permutations {
                let mut db = Database::new("db".into(), "synthetic".into(), "Sort".into(), 0);
                db.props.extend([
                    Property { id: "input".into(), name: "Input".into(), kind: PropKind::Number, ..Default::default() },
                    Property {
                        id: "amount".into(),
                        kind: PropKind::Formula,
                        formula: format!(r#"if(prop("Input") == 0, {a}, if(prop("Input") == 1, {b}, {c}))"#),
                        ..Default::default()
                    },
                ]);
                let mut items = vec![Item::Database(db)];
                for (position, index) in order.into_iter().enumerate() {
                    let id = ids[index];
                    let mut row = Row::new(id.into(), "db", id.into(), 0);
                    row.order = position as i64;
                    row.cells = format!(r#"{{"input":{index}}}"#);
                    items.push(Item::Row(row));
                }
                let ctx = Ctx::new(items.iter(), 0, 0);
                let db = ctx.dbs["db"];
                let amount = db.prop("amount").unwrap();
                let get_number = |id| match ctx.value(db, ctx.by_id[id], amount).unwrap() {
                    Val::Num(n) => n,
                    value => panic!("unexpected sort key {value:?}"),
                };
                let cmp = with_budget(|charge| get_number("A").cmp_checked(&get_number("B"), charge));
                if has_witness {
                    assert_eq!(cmp, Ok(std::cmp::Ordering::Less));
                } else {
                    assert_eq!(cmp.unwrap_err(), "The formula exceeds its arithmetic work budget");
                }
                let order: Vec<_> = ctx.run_view(db, &view).rows.iter().map(|row| row.id.clone()).collect();
                let position = |id| order.iter().position(|row| row == id).unwrap();
                assert!(position("A") < position("C"), "{order:?}");
                assert_eq!(&order, expected.get_or_insert_with(|| order.clone()));
            }
        }
    }

    #[test]
    fn computed_json_preserves_transcendental_cancellation_and_golden_exp() {
        for (expression, format, expected, raw) in [
            ("ln(sqrt(1+1e-100))", "number", "5e-101", 5e-101),
            ("exp(sqrt(1e-200+1e-250))-1", "number", "1e-100", 1e-100),
            ("exp(1)", "number", "2.71828182846", std::f64::consts::E),
            ("exp(-746)", "number", "1.03828480952e-324", 0.0),
            ("exp(-750)*1e308", "number", "1.90168496348e-18", 1.9016849634750064e-18),
            ("exp(-750)/1e-325", "number", "0.190168496348", 0.19016849634750063),
            ("ln(exp(-750))", "number", "-750", -750.0),
            ("exp(-746)*1e308*1e15", "dollar", "$0.1", 0.1),
        ] {
            let mut db = Database::new("db".into(), "synthetic".into(), "Computed".into(), 0);
            db.props.push(Property {
                id: "amount".into(),
                kind: PropKind::Formula,
                formula: expression.into(),
                number_format: format.into(),
                ..Default::default()
            });
            let row = Row::new("row".into(), "db", "Row".into(), 0);
            let items = [Item::Database(db), Item::Row(row)];
            let ctx = Ctx::new(items.iter(), 0, 0);
            let out = ctx.row_out_pub(ctx.dbs["db"], ctx.by_id["row"]);
            assert_eq!(out.values["amount"], json!({"t": "number", "text": expected, "v": raw}), "{expression}");
        }
    }

    #[test]
    fn stored_currency_display_uses_the_decimal_token() {
        let mut db = Database::new("db".into(), "synthetic".into(), "Test".into(), 0);
        db.props.push(Property {
            id: "amount".into(),
            name: "Amount".into(),
            kind: PropKind::Number,
            number_format: "dollar".into(),
            ..Default::default()
        });
        let mut below = Row::new("a".into(), "db", "Below".into(), 0);
        below.cells = r#"{"amount":999999999999.98499}"#.into();
        let mut overflow = Row::new("b".into(), "db", "Overflow".into(), 0);
        overflow.cells = r#"{"amount":1e309}"#.into();
        let items = [Item::Database(db), Item::Row(below), Item::Row(overflow)];
        let ctx = Ctx::new(items.iter(), 0, 0);
        let db = ctx.dbs["db"];
        let below = ctx.row_out_pub(db, ctx.by_id["a"]);
        assert!(below.values["amount"].is_number());
        assert_eq!(below.number_text["amount"], "$999,999,999,999.98");
        let overflow = ctx.row_out_pub(db, ctx.by_id["b"]);
        assert_eq!(overflow.values["amount"]["t"], "error");
        assert!(overflow.values["amount"]["text"].as_str().unwrap().contains("overflow"));
    }
}
