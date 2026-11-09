//! Frozen old-engine oracle, captured BEFORE P2/P3 at 5809969 (WP-16).
//! Deterministic generated inputs: no vault, clock, random ids, network or private formulas.
//! Ordinary tests only READ the oracle and compare the complete serialized ViewResult bytes.
//! Capture is explicit/ignored, refuses to replace a nonempty oracle, and is not a blessing tool.
use super::{engine::Ctx, types::*};
use crate::model::Item;
use serde_json::{json, Value};
use std::io::Write;

const NOW: i64 = 29_903_280; // fixed local clock, including relative date functions/filters
const CALCS: &[&str] = &["countAll", "count", "countValues", "countNotEmpty", "countEmpty", "percentEmpty", "percentNotEmpty", "countUnique", "showUnique", "showOriginal", "sum", "avg", "median", "min", "max", "range", "earliest", "latest", "dateRange", "checked", "unchecked", "percentChecked", "percentUnchecked"];
const KINDS: &[PropKind] = &[PropKind::Title, PropKind::Text, PropKind::Number, PropKind::Select, PropKind::MultiSelect, PropKind::Status, PropKind::Date, PropKind::Person, PropKind::Files, PropKind::Checkbox, PropKind::Url, PropKind::Email, PropKind::Phone, PropKind::Formula, PropKind::Relation, PropKind::Rollup, PropKind::CreatedTime, PropKind::CreatedBy, PropKind::LastEditedTime, PropKind::LastEditedBy, PropKind::Id, PropKind::Place, PropKind::Button];
const OPS: &[&str] = &["isEmpty", "isNotEmpty", "equals", "is", "notEquals", "isNot", "contains", "notContains", "startsWith", "endsWith", "in", "notIn", "eq", "neq", "gt", "gte", "lt", "lte", "between", "before", "after", "onOrBefore", "onOrAfter", "unknown"];

fn property(id: &str, kind: PropKind) -> Property {
    Property { id: id.into(), name: id.into(), kind, ..Default::default() }
}

fn corpus() -> Vec<Item> {
    let mut db = Database::new("gold".into(), "s".into(), "Synthetic".into(), 0);
    db.props = KINDS.iter().map(|k| {
        let id = serde_json::to_value(k).unwrap().as_str().unwrap().to_owned();
        let mut p = property(&id, *k);
        p.options = [("b", "مرحبا 10", "complete"), ("a", "שלום 2", "todo"), ("c", "Alpha", "inProgress")].iter().map(|(id, name, group)| SelectOption { id: (*id).into(), name: (*name).into(), group: (*group).into(), color: "blue".into() }).collect();
        p.formula = "prop(\"number\") * 1.1".into();
        p.relation_db = "gold".into();
        p.rollup_relation = "relation".into();
        p.rollup_prop = "number".into();
        p.rollup_calc = "sum".into();
        p.id_prefix = "ID".into();
        p
    }).collect();
    let mut items = vec![Item::Database(db)];
    for i in 0..10 {
        let mut r = Row::new(format!("r{i}"), "gold", ["مرحبا 10", "שלום 2", "Alpha 2", "alpha 2", "", "Alpha 10"][i % 6].into(), 1_700_000_000_000 + i as i64 % 3);
        r.order = (9 - i) as i64 / 2; // manual ties, intentionally different from id order
        r.number = i as i64;
        r.updated = r.created + 120_000;
        r.created_by = "Synthetic author".into();
        r.edited_by = if i % 2 == 0 { "שלום" } else { "" }.into();
        r.favorite = i % 2 == 0;
        r.cells = if i == 0 { "{}".into() } else if i == 9 { "malformed json".into() } else {
            json!({"text": (["مرحبا 10", "שלום 2", "Alpha", ""][i%4]), "number": if i%3==0 {Value::Null} else {json!((i%3) as f64 - 1.1)}, "select": (["a","b","c","gone"][i%4]), "status": (["b","a","c"][i%3]), "multiSelect": if i%2==0 {json!(["a","b"])} else {json!([])}, "date": {"start": if i%2==0 {"2026-11-09"} else {"2024-02-29T12:30"}, "end":"2026-11-11"}, "person":["مرحبا", "שלום"], "files":[{"name":"file 2.txt", "id":"fake"}], "checkbox":i%2==0, "url":"https://example.invalid", "email":"synthetic@example.invalid", "phone":"123", "relation":["r0", "r2", "missing"], "place":{"address":"مرحبا שלום"}}).to_string()
        };
        if i == 8 { r.trashed_at = Some(1); }
        items.push(Item::Row(r));
    }
    items
}

fn emit(items: &[Item], db: &str, name: &str, view: View, out: &mut Vec<u8>) {
    // Fresh query context per view, as in query_view_core. Key/row/field order is NOT normalized.
    let ctx = Ctx::new(items.iter(), NOW, 120);
    out.extend(serde_json::to_vec(name).unwrap());
    out.push(b'\n');
    serde_json::to_writer(&mut *out, &ctx.run_view(ctx.dbs[db], &view)).unwrap();
    out.push(b'\n');
}

fn views_bytes() -> Vec<u8> {
    let items = corpus();
    let db = match &items[0] { Item::Database(d) => d, _ => unreachable!() };
    let mut out = Vec::new();
    emit(&items, "gold", "manual", View::default(), &mut out);
    for p in &db.props {
        for op in OPS {
            let date = matches!(p.kind, PropKind::Date | PropKind::CreatedTime | PropKind::LastEditedTime);
            let c = Cond { prop: p.id.clone(), op: (*op).into(), value: if date {"2026-11-09"} else if p.kind == PropKind::Checkbox {"true"} else if p.kind == PropKind::Number {"0"} else {"a"}.into(), value2: "2026-11-11".into(), values: vec!["b".into(), "r2".into()], ..Default::default() };
            emit(&items, "gold", &format!("filter/{}/{op}", p.id), View { filter: Filter { conds: vec![c], ..Default::default() }, ..Default::default() }, &mut out);
        }
        for desc in [false, true] {
            emit(&items, "gold", &format!("sort/{}/{desc}", p.id), View { sorts: vec![Sort { prop: p.id.clone(), descending: desc }], ..Default::default() }, &mut out);
        }
        for sort in ["manual", "asc", "desc"] {
            emit(&items, "gold", &format!("group/{}/{sort}", p.id), View { group: Some(GroupBy { prop: p.id.clone(), sort: sort.into(), ..Default::default() }), sub_group: Some(GroupBy { prop: "checkbox".into(), ..Default::default() }), calcs: vec![ColCalc { prop: "number".into(), calc: "sum".into() }], ..Default::default() }, &mut out);
        }
        for calc in CALCS {
            emit(&items, "gold", &format!("footer/{}/{calc}", p.id), View { calcs: vec![ColCalc { prop: p.id.clone(), calc: (*calc).into() }], ..Default::default() }, &mut out);
        }
    }
    for rel in ["today", "tomorrow", "yesterday", "oneWeekAgo", "oneWeekFromNow", "oneMonthAgo", "oneMonthFromNow", "thisWeek", "pastWeek", "nextWeek", "pastMonth", "nextMonth", "thisMonth", "thisYear", "pastYear", "nextYear"] {
        emit(&items, "gold", rel, View { filter: Filter { conds: vec![Cond { prop: "date".into(), op: "is".into(), rel: rel.into(), ..Default::default() }], ..Default::default() }, ..Default::default() }, &mut out);
    }
    emit(&items, "gold", "nested-or-and-missing-sort", View { filter: Filter { op: "or".into(), conds: vec![Cond {prop:"deleted".into(), op:"is".into(), ..Default::default()}], groups: vec![Filter {conds:vec![Cond {prop:"number".into(), op:"gt".into(), value:"0".into(), ..Default::default()}], ..Default::default()}] }, sorts: vec![Sort { prop:"deleted".into(), descending:false }, Sort { prop:"number".into(), descending:true }, Sort {prop:"title".into(), descending:false}], ..Default::default() }, &mut out);
    for by in ["day", "week", "month", "year", "relative", "group"] {
        emit(&items, "gold", by, View { group: Some(GroupBy {prop: if by=="group" {"status"} else {"date"}.into(), date_by:by.into(), sort:"desc".into(), hide_empty:true, hidden:vec!["b".into()], ..Default::default()}), ..Default::default() }, &mut out);
    }
    out
}

fn formulas_bytes() -> Vec<u8> {
    let formulas: Vec<String> = serde_json::from_str(include_str!("formulas.json")).unwrap();
    let mut items = corpus();
    if let Item::Database(db) = &mut items[0] {
        db.props.push(Property { formula:"prop(\"cycle\")".into(), ..property("cycle", PropKind::Formula) });
        for (i, f) in formulas.iter().enumerate() {
            db.props.push(Property {formula:f.clone(), ..property(&format!("f{i}"), PropKind::Formula)});
        }
        // Same text in two properties proves shared ASTs still use the current row/env.
        db.props.push(Property {formula:"prop(\"number\")".into(), ..property("duplicate_text", PropKind::Formula)});
        for (i, calc) in CALCS.iter().enumerate() {
            for target in ["number", "date", "checkbox", "text", "formula"] {
                db.props.push(Property {rollup_relation:"relation".into(), rollup_prop:target.into(), rollup_calc:(*calc).into(), ..property(&format!("ru{i}_{target}"), PropKind::Rollup)});
            }
        }
    }
    let mut out = Vec::new();
    emit(&items, "gold", "functions-errors-rollups", View::default(), &mut out);
    // Cross-database relation, missing db/property, rollup cycles and depth-32 limit.
    let mut chain = Database::new("chain".into(), "s".into(), "Chain".into(), 0);
    for i in 0..35 { chain.props.push(Property {formula: if i==34 {"1".into()} else {format!("prop(\"depth{}\")", i+1)}, ..property(&format!("depth{i}"), PropKind::Formula)}); }
    chain.props.push(Property {relation_db:"gold".into(), ..property("link", PropKind::Relation)});
    for (id, rel, prop) in [("cross", "link", "formula"), ("bad_prop", "link", "missing"), ("bad_rel", "missing", "number"), ("roll_cycle", "self", "roll_cycle")] {
        chain.props.push(Property {rollup_relation:rel.into(), rollup_prop:prop.into(), rollup_calc:"sum".into(), ..property(id, PropKind::Rollup)});
    }
    chain.props.push(Property {relation_db:"chain".into(), ..property("self", PropKind::Relation)});
    let mut row = Row::new("chain-row".into(), "chain", "Chain".into(), 0);
    row.cells = json!({"link":["r1","r2"], "self":["chain-row"]}).to_string();
    items.push(Item::Database(chain));
    items.push(Item::Row(row));
    emit(&items, "chain", "depth-cross-relations-cycles", View::default(), &mut out);
    out
}

fn large_bytes() -> Vec<u8> {
    let mut db = Database::new("large".into(), "s".into(), "Generated".into(), 0);
    db.props.push(property("n", PropKind::Number));
    db.props.push(Property {formula:"prop(\"n\") * 1.1".into(), ..property("f", PropKind::Formula)});
    let mut items = vec![Item::Database(db)];
    let mut seed = 42u64;
    for i in 0..50_017 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let mut row = Row::new(format!("l{i}"), "large", format!("صف {i} שלום"), 0);
        row.order = (50_017-i) as i64 / 3;
        row.cells = if i%17==0 {"{}".into()} else {json!({"n": (seed>>33)%29}).to_string()};
        items.push(Item::Row(row));
    }
    let mut out = Vec::new();
    emit(&items, "large", "truncated-formula-sort-ties-empty-last", View {sorts:vec![Sort {prop:"f".into(), descending:true}, Sort {prop:"n".into(), descending:false}], group:Some(GroupBy {prop:"n".into(), number_step:10.0, ..Default::default()}), calcs:vec![ColCalc {prop:"f".into(), calc:"sum".into()}], ..Default::default()}, &mut out);
    emit(&items, "large", "filtered-formula", View { filter: Filter {conds:vec![Cond {prop:"f".into(), op:"gt".into(), value:"29".into(), ..Default::default()}], ..Default::default()}, ..Default::default()}, &mut out);
    out
}

fn check(name: &str, expected: &[u8], actual: Vec<u8>) {
    let mismatch = expected.iter().zip(&actual).position(|(a,b)| a!=b);
    assert!(mismatch.is_none() && expected.len()==actual.len(), "{name}: byte mismatch at {:?}; old length {}, new length {}. Do not regenerate the old oracle.", mismatch.or(Some(expected.len().min(actual.len()))), expected.len(), actual.len());
}

#[test]
fn golden_views_byte_for_byte() { check("views", include_bytes!("views.jsonl"), views_bytes()); }
#[test]
fn golden_formulas_rollups_byte_for_byte() { check("formulas", include_bytes!("formulas.jsonl"), formulas_bytes()); }
#[test]
fn golden_large_byte_for_byte() { check("large", include_bytes!("large.jsonl"), large_bytes()); }

#[test]
#[ignore = "one-time OLD-engine capture only; refuses to replace existing answers"]
fn capture_old_engine_golden() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = if root.ends_with("db-bench") {root.join("../../src-tauri/src/database/golden")} else {root.join("src/database/golden")};
    for (name, bytes) in [("views.jsonl", views_bytes()), ("formulas.jsonl", formulas_bytes()), ("large.jsonl", large_bytes())] {
        let path = dir.join(name);
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0, "old oracle already captured");
        std::fs::OpenOptions::new().write(true).open(path).unwrap().write_all(&bytes).unwrap();
        println!("captured {name}: {} bytes", bytes.len());
    }
}
