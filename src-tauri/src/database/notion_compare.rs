//! Cases taken from a real Notion database ("Notaya comparison tests", built 2026-11-03 in the
//! owner's workspace). The same properties, formulas and rows are built here, and the results
//! are compared with what Notion computed. Cases only appear in `EXPECTED` once Notion's own
//! answer was observed.

#![cfg(test)]

use super::engine::Ctx;
use super::types::*;
use super::value::{self, Val};
use serde_json::{json, Value};

const FORMULAS: [(&str, &str); 20] = [
    ("Total", r#"prop("Qty") * prop("Price")"#),
    ("Label", r#"if(prop("Done"), "✔ ", "○ ") + prop("Name")"#),
    ("Size", r#"prop("Qty") > 5 ? "bulk" : "single""#),
    ("Upper", r#"upper(prop("Name"))"#),
    ("Len", r#"length(prop("Name"))"#),
    ("Tagcount", r#"length(prop("Tags"))"#),
    ("Joined", r#"join(prop("Tags").map(current), "/")"#),
    ("Rounded", r#"round(prop("Price") * 1.15 * 100) / 100"#),
    ("Mod", r#"prop("Qty") % 3"#),
    ("Squared", r#"pow(prop("Qty"), 2)"#),
    ("Sub", r#"substring(prop("Name"), 0, 3)"#),
    ("Has an", r#"contains(prop("Name"), "an")"#),
    ("Regex", r#"test(prop("Name"), "^[A-M]")"#),
    ("Year", r#"year(prop("Due"))"#),
    ("Plus10", r#"formatDate(dateAdd(prop("Due"), 10, "days"), "YYYY-MM-DD")"#),
    ("DaysTo", r#"dateBetween(prop("Due"), parseDate("2026-01-01"), "days")"#),
    ("Grade", r#"ifs(prop("Qty") >= 10, "A", prop("Qty") >= 5, "B", "C")"#),
    ("Replaced", r#"replaceAll(prop("Name"), "a", "A")"#),
    ("Abs", r#"abs(prop("Qty") - 10)"#),
    ("Empty", r#"empty(prop("Due"))"#),
];

/// name, qty, price, done, due, status, tags
type RowSpec = (&'static str, Option<f64>, Option<f64>, bool, Option<&'static str>, &'static str, &'static [&'static str]);
const ROWS: [RowSpec; 8] = [
    ("Banana split", Some(12.0), Some(2.5), true, Some("2026-03-15"), "Open", &["a", "b"]),
    ("apple pie", Some(3.0), Some(4.99), false, Some("2026-01-01"), "Doing", &["c"]),
    ("Zebra", Some(5.0), Some(0.1), false, Some("2026-12-31"), "Closed", &["a", "b", "c"]),
    ("مرحبا بالعالم", Some(7.0), Some(-3.25), true, Some("2025-02-28"), "Open", &[]),
    ("Mango", Some(0.0), Some(10.0), false, Some("2024-02-29"), "", &["b"]),
    ("Empty row", None, None, false, None, "", &[]),
    ("Time row", Some(10.0), Some(1.005), true, Some("2026-06-30T14:30"), "Doing", &[]),
    ("Banana", Some(2.5), Some(3.0), false, Some("2026-11-03"), "Open", &["a"]),
];

fn build() -> (Database, Vec<Row>) {
    let mut db = Database::new("db".into(), "sec".into(), "Cmp".into(), 1);
    db.props[0].name = "Name".into();
    let mut p = |id: &str, name: &str, kind: PropKind| db.props.push(Property { id: id.into(), name: name.into(), kind, ..Property::default() });
    p("qty", "Qty", PropKind::Number);
    p("price", "Price", PropKind::Number);
    p("done", "Done", PropKind::Checkbox);
    p("due", "Due", PropKind::Date);
    p("status", "Status", PropKind::Select);
    p("tags", "Tags", PropKind::MultiSelect);
    db.props.iter_mut().find(|x| x.id == "status").unwrap().options = ["Open", "Doing", "Closed"].iter().map(|n| SelectOption { id: n.to_lowercase(), name: n.to_string(), ..SelectOption::default() }).collect();
    db.props.iter_mut().find(|x| x.id == "tags").unwrap().options = ["a", "b", "c"].iter().map(|n| SelectOption { id: n.to_string(), name: n.to_string(), ..SelectOption::default() }).collect();
    for (name, f) in FORMULAS {
        db.props.push(Property { id: format!("f_{name}"), name: name.into(), kind: PropKind::Formula, formula: f.into(), ..Property::default() });
    }
    let rows = ROWS
        .iter()
        .enumerate()
        .map(|(i, (name, qty, price, done, due, status, tags))| {
            let mut r = Row::new(format!("r{i}"), "db", name.to_string(), 1);
            let mut c = serde_json::Map::new();
            if let Some(q) = qty {
                c.insert("qty".into(), json!(q));
            }
            if let Some(p) = price {
                c.insert("price".into(), json!(p));
            }
            if *done {
                c.insert("done".into(), json!(true));
            }
            if let Some(d) = due {
                c.insert("due".into(), json!({"start": d}));
            }
            if !status.is_empty() {
                c.insert("status".into(), json!(status.to_lowercase()));
            }
            if !tags.is_empty() {
                c.insert("tags".into(), json!(tags));
            }
            r.cells = Value::Object(c).to_string();
            r
        })
        .collect();
    (db, rows)
}

fn show(v: &Result<Val, String>) -> String {
    match v {
        Ok(Val::Null) => "∅".into(),
        Ok(Val::Str(s)) => format!("{s:?}"),
        Ok(other) => other.text(),
        Err(e) => format!("ERROR({e})"),
    }
}

#[test]
fn print_our_results() {
    let (db, rows) = build();
    let items: Vec<crate::model::Item> = std::iter::once(crate::model::Item::Database(db.clone())).chain(rows.iter().cloned().map(crate::model::Item::Row)).collect();
    let ctx = Ctx::new(items.iter(), value::from_parts(2026, 11, 3, 12, 0), 0);
    for (name, _) in FORMULAS {
        let p = db.props.iter().find(|p| p.name == name).unwrap();
        let line: Vec<String> = rows.iter().map(|r| show(&ctx.value(&db, r, p))).collect();
        println!("RESULT {name:9} | {}", line.join(" | "));
    }
}

fn ours(prop: &str, row: usize) -> Result<Val, String> {
    let (db, rows) = build();
    let items: Vec<crate::model::Item> = std::iter::once(crate::model::Item::Database(db.clone())).chain(rows.iter().cloned().map(crate::model::Item::Row)).collect();
    let ctx = Ctx::new(items.iter(), value::from_parts(2026, 11, 3, 12, 0), 0);
    let p = db.props.iter().find(|p| p.name == prop).unwrap();
    ctx.value(&db, &rows[row], p)
}

fn num(v: Result<Val, String>) -> f64 {
    match v {
        Ok(Val::Num(n)) => n,
        other => panic!("not a number: {other:?}"),
    }
}

/// Every line here was checked against Notion's own answer for the same row (2026-11-03: Notion's
/// filter on the formula result matched, or did not match, the value written here).
#[test]
fn agrees_with_what_notion_computed() {
    // Totals (rows 0, 1, 2, 7 and the float case in row 6).
    assert_eq!(num(ours("Total", 0)), 30.0);
    assert_eq!(num(ours("Total", 1)), 14.97);
    assert_eq!(num(ours("Total", 2)), 0.5);
    assert_eq!(num(ours("Total", 7)), 7.5);
    assert_eq!(num(ours("Total", 6)), 10.05, "1.005 * 10 shows as 10.05, not 10.049999999999999");
    // An empty number is 0 next to a number: it is not "bulk" and it is not grade A.
    assert_eq!(ours("Size", 5), Ok(Val::Str("single".into())));
    assert_eq!(ours("Grade", 5), Ok(Val::Str("C".into())));
    // Date functions of an empty date give an empty result, not an error.
    assert_eq!(ours("Year", 5), Ok(Val::Null));
    assert_eq!(ours("Plus10", 5), Ok(Val::Null));
    assert_eq!(ours("DaysTo", 5), Ok(Val::Null));
    // 2026-06-30 14:30 is 180 whole days after 2026-01-01.
    assert_eq!(num(ours("DaysTo", 6)), 180.0);
    assert_eq!(num(ours("DaysTo", 0)), 73.0);
    // Text: Arabic letters count as letters, replaceAll, join, concatenation.
    assert_eq!(ours("Sub", 3), Ok(Val::Str("مرح".into())));
    assert_eq!(ours("Replaced", 0), Ok(Val::Str("BAnAnA split".into())));
    assert_eq!(ours("Joined", 2), Ok(Val::Str("a/b/c".into())));
    assert_eq!(ours("Label", 7), Ok(Val::Str("○ Banana".into())));
    // Rounding, modulo and squares (compared in Notion with the same values).
    assert_eq!(num(ours("Rounded", 0)), 2.88);
    assert_eq!(num(ours("Mod", 7)), 2.5);
    assert_eq!(num(ours("Squared", 7)), 6.25);
    assert_eq!(ours("Plus10", 7), Ok(Val::Str("2026-11-13".into())));
}
