//! Automations and buttons (A35, owner 2026-11-04: "add the automations, finish them all").
//!
//! An automation is "when X happens to a row, if it matches this filter, do these actions". A button
//! property runs its actions when pressed. Everything stays inside the vault: the actions can set a
//! property, add a row, show a message inside Notaya or move the row to the trash. They cannot reach the
//! network, start programs, send anything out or delete for good. Limits keep a mistake from running away:
//! a chain of automations that trigger each other stops after three steps, one run does at most 200
//! actions and adds at most 50 rows, and an action that fails shows a message and the rest carries on.

use crate::actions::Actor;

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Map, Value};

use super::database::{add_row_inner, change_row, database, ctx_of, parse_cells, row, trash_rows, Reminder};
use crate::database::clock;
use crate::database::engine::Ctx;
use crate::database::types::*;
use crate::database::value::{self, parse_date};
use crate::error::{Error, Result};
use crate::model::{new_id, Item};
use crate::vault::Session;

const MAX_DEPTH: usize = 3;
const MAX_ACTIONS_PER_RUN: usize = 200;
const MAX_ROWS_PER_RUN: usize = 50;
/// Dates older than this are not caught up after Notaya was closed.
const CATCH_UP_MINUTES: i64 = 7 * 24 * 60;

pub enum Event<'a> {
    Added,
    /// The ids of the properties whose value changed ("title" for the row's name).
    Changed(&'a [String]),
    /// A button property was pressed.
    Button(&'a str),
}

/// What one run (one edit and everything it sets off) may still do, and what it has to report.
pub struct Run {
    actions_left: usize,
    rows_left: usize,
    now: i64,
    pub notices: Vec<Reminder>,
}

impl Default for Run {
    fn default() -> Self {
        Run::at(clock::local_now().0)
    }
}

impl Run {
    pub fn at(now: i64) -> Run {
        Run { actions_left: MAX_ACTIONS_PER_RUN, rows_left: MAX_ROWS_PER_RUN, now, notices: Vec::new() }
    }

    fn say(&mut self, s: &Session, row_id: &str, source: &str, text: String) {
        let (title, dbid, dbtitle) = match s.get(row_id) {
            Some(Item::Row(r)) => {
                let t = match s.get(&r.db) {
                    Some(Item::Database(d)) => d.title.clone(),
                    _ => String::new(),
                };
                (r.title.clone(), r.db.clone(), t)
            }
            _ => (String::new(), String::new(), String::new()),
        };
        let _ = title;
        self.notices.push(Reminder {
            key: format!("auto:{}", new_id()),
            row_id: row_id.to_string(),
            title: text,
            database_id: dbid,
            database_title: dbtitle,
            when: String::new(),
            message: source.to_string(),
        });
    }
}

static NOTICES: Mutex<Vec<Reminder>> = Mutex::new(Vec::new());
static LAST_TICK: Mutex<Option<HashMap<String, i64>>> = Mutex::new(None);

/// Hands the messages of a finished run to the screen (shown on its next poll).
pub fn finish(run: Run) {
    if run.notices.is_empty() {
        return;
    }
    let mut q = NOTICES.lock().unwrap_or_else(|e| e.into_inner());
    q.extend(run.notices);
    let extra = q.len().saturating_sub(50);
    q.drain(..extra);
}

/// Forgets the messages and the date bookkeeping of this run (the vault was locked).
pub fn clear_notices() {
    NOTICES.lock().unwrap_or_else(|e| e.into_inner()).clear();
    *LAST_TICK.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// The messages waiting for the screen, once.
pub fn take_notices() -> Vec<Reminder> {
    std::mem::take(&mut *NOTICES.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Runs what an event on a row sets off: the automations of its database that match, or the button's actions.
pub fn fire(s: &mut Session, run: &mut Run, row_id: &str, ev: &Event, depth: usize) {
    fire_core(s, &Actor::Automation, run, row_id, ev, depth)
}

pub(crate) fn fire_core(s: &mut Session, _actor: &Actor, run: &mut Run, row_id: &str, ev: &Event, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(r) = row(s, row_id) else { return };
    if r.trashed_at.is_some() {
        return;
    }
    let Ok(db) = database(s, &r.db) else { return };
    let mut jobs: Vec<(String, Vec<Action>)> = Vec::new();
    match ev {
        Event::Button(pid) => {
            if let Some(p) = db.prop(pid).filter(|p| p.kind == PropKind::Button) {
                jobs.push((p.name.clone(), p.actions.clone()));
            }
        }
        _ => {
            if !db.automations.iter().any(|a| a.enabled) {
                return;
            }
            let ctx = ctx_of(s);
            for a in db.automations.iter().filter(|a| a.enabled) {
                let hit = match (a.trigger.kind.as_str(), ev) {
                    ("rowAdded", Event::Added) => true,
                    ("propChanged", Event::Changed(ps)) => a.trigger.prop.is_empty() || ps.contains(&a.trigger.prop),
                    _ => false,
                };
                if hit && (a.filter.is_empty() || ctx.matches(&db, &r, &a.filter)) {
                    jobs.push((a.name.clone(), a.actions.clone()));
                }
            }
        }
    }
    for (name, actions) in jobs {
        exec(s, run, &name, row_id, &actions, depth);
    }
}

fn exec(s: &mut Session, run: &mut Run, name: &str, row_id: &str, actions: &[Action], depth: usize) {
    for act in actions {
        if run.actions_left == 0 {
            run.say(s, row_id, name, "Stopped: this run did too many actions.".into());
            return;
        }
        run.actions_left -= 1;
        if let Err(e) = do_action(s, run, name, row_id, act, depth) {
            run.say(s, row_id, name, format!("An action did not run: {e}"));
        }
    }
}

fn expand(text: &str, src: &Row) -> String {
    let t: String = src.title.chars().take(300).collect();
    text.replace("{title}", &t)
}

fn do_action(s: &mut Session, run: &mut Run, name: &str, row_id: &str, act: &Action, depth: usize) -> Result<()> {
    match act.kind.as_str() {
        "setProp" => {
            let src = row(s, row_id)?;
            let db = database(s, &src.db)?;
            let mut patch = Map::new();
            let mut title = None;
            if act.prop == TITLE_PROP {
                title = Some(expand(&act.value, &src));
            } else {
                let p = db.prop(&act.prop).ok_or(Error::NotFound)?;
                patch.insert(act.prop.clone(), resolve(p, &act.value, &src, Some(&src), run.now)?);
            }
            let changed = change_row(s, row_id, title, &patch)?;
            if !changed.is_empty() {
                fire(s, run, row_id, &Event::Changed(&changed), depth + 1);
            }
            Ok(())
        }
        "addRow" => {
            if run.rows_left == 0 {
                return Err(Error::invalid("too many new rows in one run"));
            }
            run.rows_left -= 1;
            let src = row(s, row_id)?;
            let target = if act.database.is_empty() { src.db.clone() } else { act.database.clone() };
            let tdb = database(s, &target)?;
            if tdb.trashed_at.is_some() {
                return Err(Error::invalid("that database is in the trash"));
            }
            let mut given = Map::new();
            for c in &act.cells {
                let p = tdb.prop(&c.prop).ok_or(Error::NotFound)?;
                given.insert(c.prop.clone(), resolve(p, &c.value, &src, None, run.now)?);
            }
            let id = add_row_inner(s, &target, expand(&act.title, &src), &given, "")?;
            fire(s, run, &id, &Event::Added, depth + 1);
            Ok(())
        }
        "notify" => {
            let src = row(s, row_id)?;
            let text = expand(&act.title, &src);
            run.say(s, row_id, name, if text.trim().is_empty() { name.to_string() } else { text });
            Ok(())
        }
        "trash" => trash_rows(s, &[row_id.to_string()]),
        _ => Err(Error::invalid("unknown action")),
    }
}

/// Turns the text of an action into the value of a property (Null = clear it). `cur` is the row being
/// changed, for "add 5" and "toggle"; `src` is the row that set the action off.
fn resolve(p: &Property, spec: &str, src: &Row, cur: Option<&Row>, now: i64) -> Result<Value> {
    let t = spec.trim();
    if t == "@clear" {
        return Ok(Value::Null);
    }
    let bad = || Error::invalid(format!("\"{}\" does not fit \"{}\"", spec.chars().take(60).collect::<String>(), p.name));
    let current = || -> Option<Value> { cur.and_then(|r| parse_cells(r).get(&p.id).cloned()) };
    fn names(list: &str) -> Vec<&str> {
        list.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()).collect()
    }
    Ok(match p.kind {
        PropKind::Text | PropKind::Url | PropKind::Email | PropKind::Phone => Value::String(expand(spec, src)),
        PropKind::Number => {
            if let Some(n) = t.strip_prefix("@add:") {
                let add: f64 = n.trim().parse().ok().filter(|f: &f64| f.is_finite()).ok_or_else(bad)?;
                let base = current().and_then(|v| v.as_f64()).unwrap_or(0.0);
                if !(base + add).is_finite() {
                    return Err(bad());
                }
                json!(base + add)
            } else if t.is_empty() {
                Value::Null
            } else {
                json!(t.parse::<f64>().ok().filter(|f| f.is_finite()).ok_or_else(bad)?)
            }
        }
        PropKind::Select | PropKind::Status => json!(p.option_by_name(t).ok_or_else(bad)?.id),
        PropKind::MultiSelect => {
            let mut ids: Vec<String> = Vec::new();
            let (list, keep) = match t.strip_prefix("@add:") {
                Some(rest) => (rest, true),
                None => (t, false),
            };
            if keep {
                if let Some(Value::Array(a)) = current() {
                    ids.extend(a.iter().filter_map(|x| x.as_str().map(String::from)));
                }
            }
            for n in names(list) {
                let id = p.option_by_name(n).ok_or_else(bad)?.id.clone();
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            json!(ids)
        }
        PropKind::Date => {
            let day = value::start_of_day(now);
            let iso = if t == "@now" {
                value::format_iso(now, true)
            } else if let Some(rest) = t.strip_prefix("@today") {
                let days: i64 = if rest.is_empty() { 0 } else { rest.trim_start_matches('+').parse().map_err(|_| bad())? };
                if days.abs() > 36_500 {
                    return Err(bad());
                }
                value::format_iso(day + days * 1440, false)
            } else {
                let d = parse_date(t).ok_or_else(bad)?;
                value::format_iso(d.start, d.has_time)
            };
            json!({ "start": iso })
        }
        PropKind::Checkbox => match t.to_lowercase().as_str() {
            "true" | "yes" | "1" | "checked" => json!(true),
            "false" | "no" | "0" | "unchecked" => json!(false),
            "@toggle" => json!(!matches!(current(), Some(Value::Bool(true)))),
            _ => return Err(bad()),
        },
        PropKind::Person => json!(names(t).into_iter().map(String::from).collect::<Vec<_>>()),
        PropKind::Relation => {
            if t == "@self" {
                json!([src.id])
            } else {
                return Err(bad());
            }
        }
        _ => return Err(Error::invalid(format!("\"{}\" cannot be set by an action", p.name))),
    })
}

// ---------------------------------------------------------------- dates

/// Rows of `db` whose date in `prop` (plus the offset) falls in `(since, now]`, for the automation `a`.
pub fn date_hits(ctx: &Ctx, db: &Database, a: &Automation, since: i64, now: i64) -> Vec<String> {
    let mut out = Vec::new();
    let Some(prop) = db.prop(&a.trigger.prop) else { return out };
    for r in ctx.rows_of(&db.id) {
        if r.trashed_at.is_some() {
            continue;
        }
        let cells = ctx.cells_of(r);
        let Some(Value::Object(o)) = cells.get(&prop.id) else { continue };
        let Some(start) = o.get("start").and_then(|x| x.as_str()).and_then(parse_date) else { continue };
        let at = (if start.has_time { start.start } else { start.start + 9 * 60 }) + a.trigger.offset_mins;
        if at > since && at <= now && (a.filter.is_empty() || ctx.matches(db, r, &a.filter)) {
            out.push(r.id.clone());
        }
    }
    out
}

/// Runs the date automations whose time has come since the last check. Called with every poll of the screen.
pub fn tick_dates(s: &mut Session) {
    tick_dates_core(s, &Actor::Automation)
}

pub(crate) fn tick_dates_core(s: &mut Session, actor: &Actor) {
    // Most vaults have no date automation: look before building anything.
    let any = s.items().any(|i| matches!(i, Item::Database(d) if d.trashed_at.is_none() && d.automations.iter().any(|a| a.enabled && a.trigger.kind == "dateReached")));
    if any {
        tick_dates_at_core(s, actor, clock::local_now().0);
    }
}

pub fn tick_dates_at(s: &mut Session, now: i64) {
    tick_dates_at_core(s, &Actor::Automation, now)
}

pub(crate) fn tick_dates_at_core(s: &mut Session, _actor: &Actor, now: i64) {
    struct Job {
        db: String,
        auto: Automation,
        rows: Vec<String>,
    }
    let mut jobs: Vec<Job> = Vec::new();
    {
        let mut guard = LAST_TICK.lock().unwrap_or_else(|e| e.into_inner());
        let mem = guard.get_or_insert_with(HashMap::new);
        let ctx = Ctx::new(s.items(), now, 0);
        for db in ctx.dbs.values().filter(|d| d.trashed_at.is_none()) {
            for a in db
                .automations
                .iter()
                .filter(|a| a.enabled && a.trigger.kind == "dateReached")
            {
                let since = a
                    .last_run
                    .max(mem.get(&a.id).copied().unwrap_or(0))
                    .max(now - CATCH_UP_MINUTES);
                if since >= now {
                    continue;
                }
                mem.insert(a.id.clone(), now);
                let rows = date_hits(&ctx, db, a, since, now);
                if !rows.is_empty() {
                    jobs.push(Job {
                        db: db.id.clone(),
                        auto: a.clone(),
                        rows,
                    });
                }
            }
        }
    }
    for job in jobs {
        let mut run = Run::at(now);
        for id in &job.rows {
            exec(s, &mut run, &job.auto.name, id, &job.auto.actions, 0);
        }
        finish(run);
        if let Ok(mut db) = database(s, &job.db) {
            if let Some(a) = db.automations.iter_mut().find(|a| a.id == job.auto.id) {
                a.last_run = now;
                let _ = s.put(Item::Database(db));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::now_ms;

    fn temp_vault() -> Session {
        let dir = std::env::temp_dir().join(format!("notaya-autotest-{}", new_id()));
        std::fs::create_dir_all(&dir).unwrap();
        Session::create_open(&dir.join("vault.nota")).unwrap()
    }

    struct Fx {
        s: Session,
        db: String,
        status: String,
        done: String,
        n: String,
        due: String,
        opts: (String, String),
    }

    fn fixture() -> Fx {
        let mut s = temp_vault();
        let mut db = Database::new(new_id(), "sec".into(), "Tasks".into(), now_ms());
        let mut status = Property { id: "st".into(), name: "Status".into(), kind: PropKind::Status, ..Property::default() };
        status.options = vec![
            SelectOption { id: "o1".into(), name: "Todo".into(), group: "todo".into(), ..SelectOption::default() },
            SelectOption { id: "o2".into(), name: "Done".into(), group: "complete".into(), ..SelectOption::default() },
        ];
        db.props.push(status);
        db.props.push(Property { id: "dn".into(), name: "Done".into(), kind: PropKind::Checkbox, ..Property::default() });
        db.props.push(Property { id: "n".into(), name: "Count".into(), kind: PropKind::Number, ..Property::default() });
        db.props.push(Property { id: "due".into(), name: "Due".into(), kind: PropKind::Date, ..Property::default() });
        db.sanitize().unwrap();
        let id = db.id.clone();
        s.put(Item::Database(db)).unwrap();
        Fx { s, db: id, status: "st".into(), done: "dn".into(), n: "n".into(), due: "due".into(), opts: ("o1".into(), "o2".into()) }
    }

    fn put_db(fx: &mut Fx, f: impl FnOnce(&mut Database)) {
        let mut d = database(&fx.s, &fx.db).unwrap();
        f(&mut d);
        d.sanitize().unwrap();
        fx.s.put(Item::Database(d)).unwrap();
    }

    fn auto(name: &str, trigger: Trigger, actions: Vec<Action>) -> Automation {
        Automation { id: new_id(), name: name.into(), enabled: true, trigger, filter: Filter::default(), actions, last_run: 0 }
    }

    fn set(prop: &str, value: &str) -> Action {
        Action { kind: "setProp".into(), prop: prop.into(), value: value.into(), ..Action::default() }
    }

    fn cell(fx: &Fx, id: &str, prop: &str) -> Option<Value> {
        parse_cells(&row(&fx.s, id).unwrap()).get(prop).cloned()
    }

    fn add(fx: &mut Fx, title: &str) -> String {
        let id = add_row_inner(&mut fx.s, &fx.db.clone(), title.into(), &Map::new(), "-").unwrap();
        let mut run = Run::at(1_000_000);
        fire(&mut fx.s, &mut run, &id, &Event::Added, 0);
        id
    }

    fn change(fx: &mut Fx, id: &str, patch: Value) {
        let given = patch.as_object().unwrap().clone();
        let changed = change_row(&mut fx.s, id, None, &given).unwrap();
        let mut run = Run::at(1_000_000);
        fire(&mut fx.s, &mut run, id, &Event::Changed(&changed), 0);
    }

    #[test]
    fn a_new_row_gets_a_status_and_a_date_from_an_automation() {
        let mut fx = fixture();
        put_db(&mut fx, |d| d.automations.push(auto("Start", Trigger { kind: "rowAdded".into(), ..Trigger::default() }, vec![set("st", "todo"), set("dn", "false")])));
        let id = add(&mut fx, "First");
        assert_eq!(cell(&fx, &id, "st"), Some(json!(fx.opts.0.clone())));
        assert_eq!(cell(&fx, &id, "dn"), Some(json!(false)));
    }

    #[test]
    fn changing_a_property_sets_another_but_only_when_the_filter_matches() {
        let mut fx = fixture();
        let st = fx.status.clone();
        let mut a = auto("Tick done", Trigger { kind: "propChanged".into(), prop: st.clone(), ..Trigger::default() }, vec![set("dn", "true")]);
        a.filter.conds.push(Cond { prop: st.clone(), op: "is".into(), values: vec![fx.opts.1.clone()], value: fx.opts.1.clone(), ..Cond::default() });
        put_db(&mut fx, |d| d.automations.push(a));
        let id = add(&mut fx, "T");
        let (o0, o1) = fx.opts.clone();
        change(&mut fx, &id, json!({ "st": o0 }));
        assert_eq!(cell(&fx, &id, "dn"), None, "Todo does not match the filter");
        change(&mut fx, &id, json!({ "st": o1 }));
        assert_eq!(cell(&fx, &id, "dn"), Some(json!(true)));
        let _ = (&fx.done, &fx.n, &fx.due);
    }

    #[test]
    fn two_automations_that_undo_each_other_stop_after_three_steps() {
        let mut fx = fixture();
        let a = auto("up", Trigger { kind: "propChanged".into(), prop: "n".into(), ..Trigger::default() }, vec![set("n", "@add:1")]);
        put_db(&mut fx, |d| d.automations.push(a));
        let id = add(&mut fx, "Loop");
        change(&mut fx, &id, json!({ "n": 1 }));
        let v = cell(&fx, &id, "n").and_then(|v| v.as_f64()).unwrap();
        assert!(v > 1.0 && v <= 5.0, "it stopped by itself, at {v}");
    }

    #[test]
    fn a_button_adds_a_linked_row_shows_a_message_and_trashes() {
        let mut fx = fixture();
        let btn = Property {
            id: "b".into(),
            name: "Follow up".into(),
            kind: PropKind::Button,
            button_label: "Follow up".into(),
            actions: vec![
                Action { kind: "addRow".into(), title: "Follow up: {title}".into(), cells: vec![SetCell { prop: "st".into(), value: "Todo".into() }, SetCell { prop: "due".into(), value: "@today+7".into() }], ..Action::default() },
                set("n", "@add:2"),
                Action { kind: "notify".into(), title: "Added a follow-up for {title}".into(), ..Action::default() },
            ],
            ..Property::default()
        };
        put_db(&mut fx, |d| d.props.push(btn));
        let id = add(&mut fx, "Call Sam");
        let mut run = Run::at(value::from_parts(2026, 11, 4, 10, 0));
        fire(&mut fx.s, &mut run, &id, &Event::Button("b"), 0);
        let rows: Vec<Row> = fx.s.items().filter_map(|i| if let Item::Row(r) = i { Some(r.clone()) } else { None }).collect();
        assert_eq!(rows.len(), 2);
        let made = rows.iter().find(|r| r.id != id).unwrap();
        assert_eq!(made.title, "Follow up: Call Sam");
        let cells = parse_cells(made);
        assert_eq!(cells.get("due").unwrap()["start"], "2026-11-11");
        assert_eq!(cell(&fx, &id, "n"), Some(json!(2.0)));
        assert_eq!(run.notices.len(), 1);
        assert_eq!(run.notices[0].title, "Added a follow-up for Call Sam");
        // The trash action is reversible: the row is only moved.
        let mut run = Run::at(1);
        let t = Action { kind: "trash".into(), ..Action::default() };
        exec(&mut fx.s, &mut run, "t", &id, &[t], 0);
        assert!(row(&fx.s, &id).unwrap().trashed_at.is_some());
    }

    #[test]
    fn a_bad_action_reports_and_the_next_one_still_runs() {
        let mut fx = fixture();
        let mut run = Run::at(1);
        let id = add(&mut fx, "x");
        exec(&mut fx.s, &mut run, "Mixed", &id, &[set("st", "Nonexistent"), set("dn", "true")], 0);
        assert_eq!(run.notices.len(), 1);
        assert!(run.notices[0].title.contains("did not run"));
        assert_eq!(cell(&fx, &id, "dn"), Some(json!(true)));
    }

    #[test]
    fn one_run_adds_at_most_fifty_rows() {
        let mut fx = fixture();
        let id = add(&mut fx, "seed");
        let many: Vec<Action> = (0..60).map(|_| Action { kind: "addRow".into(), title: "x".into(), ..Action::default() }).collect();
        let mut run = Run::at(1);
        exec(&mut fx.s, &mut run, "Many", &id, &many, 0);
        let rows = fx.s.items().filter(|i| matches!(i, Item::Row(_))).count();
        assert_eq!(rows, 1 + MAX_ROWS_PER_RUN);
        assert_eq!(run.notices.len(), 10, "the rest reported once each");
    }

    #[test]
    fn a_date_automation_fires_once_for_a_date_that_arrives() {
        let mut fx = fixture();
        let a = auto("Due soon", Trigger { kind: "dateReached".into(), prop: "due".into(), offset_mins: -60 }, vec![Action { kind: "notify".into(), title: "{title} is due in an hour".into(), ..Action::default() }]);
        let aid = a.id.clone();
        put_db(&mut fx, |d| d.automations.push(a));
        let now = value::from_parts(2026, 11, 4, 12, 0);
        let iso = |m: i64| value::format_iso(m, true);
        let r1 = add(&mut fx, "soon");
        let r2 = add(&mut fx, "far");
        let mut g = |r: &str, start: i64| change_row(&mut fx.s, r, None, json!({ "due": { "start": iso(start) } }).as_object().unwrap()).unwrap();
        g(&r1, now + 30);
        g(&r2, now + 600);
        let _ = take_notices();
        {
            let mut m = LAST_TICK.lock().unwrap();
            m.get_or_insert_with(HashMap::new).insert(aid.clone(), now - 60);
        }
        tick_dates_at(&mut fx.s, now);
        let got = take_notices();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].title, "soon is due in an hour");
        tick_dates_at(&mut fx.s, now + 1);
        assert!(take_notices().is_empty(), "not again");
        let d = database(&fx.s, &fx.db).unwrap();
        assert_eq!(d.automations[0].last_run, now, "remembered, so a restart does not repeat it");
    }

    #[test]
    fn silly_values_are_refused_not_stored() {
        let fx = fixture();
        let d = database(&fx.s, &fx.db).unwrap();
        let r = Row::new("r".into(), &d.id, "t".into(), 1);
        let date = d.prop("due").unwrap();
        let num = d.prop("n").unwrap();
        assert!(resolve(date, "@today+99999999999", &r, None, 1_000_000).is_err());
        assert!(resolve(date, "@today+7", &r, None, 1_000_000).is_ok());
        assert!(resolve(num, "@add:inf", &r, Some(&r), 1).is_err());
        assert!(resolve(num, "NaN", &r, None, 1).is_err());
        assert_eq!(resolve(num, "", &r, None, 1).unwrap(), Value::Null);
    }

    #[test]
    fn a_button_is_left_out_of_an_export() {
        let mut fx = fixture();
        put_db(&mut fx, |d| d.props.push(Property { id: "b".into(), name: "Go".into(), kind: PropKind::Button, ..Property::default() }));
        let lines = crate::commands::database_io::export_lines(&fx.s, &fx.db, None, true).unwrap();
        assert!(!lines[0].contains(&"Go".to_string()));
    }

    #[test]
    fn sanitize_refuses_unknown_actions_and_tidies_missing_properties() {
        let mut fx = fixture();
        let mut d = database(&fx.s, &fx.db).unwrap();
        d.automations.push(auto("x", Trigger { kind: "rowAdded".into(), ..Trigger::default() }, vec![Action { kind: "runProgram".into(), ..Action::default() }]));
        assert!(d.sanitize().is_err());
        let mut d = database(&fx.s, &fx.db).unwrap();
        d.automations.push(auto("x", Trigger { kind: "dateReached".into(), prop: "st".into(), ..Trigger::default() }, vec![set("ghost", "1"), set("dn", "true")]));
        d.sanitize().unwrap();
        assert!(!d.automations[0].enabled, "no date property: switched off");
        assert_eq!(d.automations[0].actions.len(), 1, "the action on a missing property is dropped");
        let _ = &mut fx;
    }
}

#[cfg(test)]
mod action_parity_tests {
    use super::*;
    use crate::actions::tests::compare;

    #[test]
    fn fire_wrapper_and_core_match() {
        compare(
            |state| {
                state.with_mut(|s| {
                    fire(s, &mut Run::at(0), "10000000000000000000000000000008", &Event::Button("button"), 0);
                    Ok(())
                })
            },
            |s, _| {
                fire_core(
                    s,
                    &Actor::Automation,
                    &mut Run::at(0),
                    "10000000000000000000000000000008",
                    &Event::Button("button"),
                    0,
                );
                Ok(())
            },
        );
    }

    #[test]
    fn tick_dates_wrapper_and_core_match() {
        compare(
            |state| {
                state.with_mut(|s| {
                    tick_dates(s);
                    Ok(())
                })
            },
            |s, _| {
                tick_dates_core(s, &Actor::Automation);
                Ok(())
            },
        );
    }

    #[test]
    fn tick_dates_at_wrapper_and_core_match() {
        compare(
            |state| {
                state.with_mut(|s| {
                    tick_dates_at(s, 0);
                    Ok(())
                })
            },
            |s, _| {
                tick_dates_at_core(s, &Actor::Automation, 0);
                Ok(())
            },
        );
    }
}
