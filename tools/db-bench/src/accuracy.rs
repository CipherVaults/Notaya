//! Independent P7 scale oracle. The inputs use exact integers and a deliberately
//! simple property schema. The oracle does its own sorting, grouping, formula,
//! relation, rollup, selection and footer arithmetic, with no engine calls.
use serde_json::json;

use crate::database::{engine::{Ctx, GroupOut, RowOut}, types::*, windows::{QueryCache, RowRange}};
use crate::model::Item;

fn property(id: &str, name: &str, kind: PropKind) -> Property {
    Property { id: id.into(), name: name.into(), kind, ..Default::default() }
}

fn fixture(n: usize) -> (Vec<Item>, Database, View) {
    let mut db = Database::new("test".into(), "section".into(), "Synthetic".into(), 0);
    let mut related = Database::new("related".into(), "section".into(), "Synthetic related".into(), 0);
    related.props.push(property("weight", "Weight", PropKind::Number));
    db.props.extend([
        property("amount", "Amount", PropKind::Number),
        property("checked", "Checked", PropKind::Checkbox),
        Property {
            options: ["Blue", "Amber"].into_iter().enumerate().map(|(i, name)| SelectOption {
                id: i.to_string(), name: name.into(), color: name.to_lowercase(), group: String::new(),
            }).collect(),
            ..property("category", "Category", PropKind::Select)
        },
        Property { formula: "prop(\"Amount\") * 2".into(), ..property("double", "Double", PropKind::Formula) },
        Property { relation_db: related.id.clone(), ..property("link", "Link", PropKind::Relation) },
        Property {
            rollup_relation: "link".into(), rollup_prop: "weight".into(), rollup_calc: "sum".into(),
            ..property("rollup", "Rollup", PropKind::Rollup)
        },
    ]);
    let mut items = vec![Item::Database(db.clone()), Item::Database(related.clone())];
    for i in 0..2 {
        let mut row = Row::new(format!("target-{i}"), &related.id, format!("Target {i}"), 0);
        row.cells = json!({"weight": i + 7}).to_string();
        items.push(Item::Row(row));
    }
    for i in 0..n {
        let mut row = Row::new(format!("r{i}"), &db.id, format!("Row {i}"), i as i64);
        row.order = i as i64;
        row.cells = json!({"amount": i % 13, "checked": i % 3 == 0, "category": (i % 2).to_string(),
            "link": [format!("target-{}", i % 2)]}).to_string();
        items.push(Item::Row(row));
    }
    let view = View {
        sorts: vec![Sort { prop: "double".into(), descending: true }],
        group: Some(GroupBy { prop: "category".into(), hide_empty: true, ..Default::default() }),
        sub_group: Some(GroupBy { prop: "checked".into(), ..Default::default() }),
        calcs: vec![ColCalc { prop: "amount".into(), calc: "sum".into() }],
        ..View::default()
    };
    (items, db, view)
}

fn reference_order(n: usize, filtered: bool) -> Vec<usize> {
    let mut order = Vec::new();
    // Descending integer buckets and ascending creation order within each tie.
    for amount in (0..13).rev() {
        order.extend((amount..n).step_by(13).filter(|i| !filtered || i % 3 == 0));
    }
    order
}

fn row_ids(indices: &[usize]) -> Vec<String> {
    indices.iter().map(|i| format!("r{i}")).collect()
}

fn footer(indices: &[usize]) -> serde_json::Map<String, serde_json::Value> {
    let sum: usize = indices.iter().map(|i| i % 13).sum();
    json!({"amount": sum.to_string()}).as_object().unwrap().clone()
}

fn reference_groups(order: &[usize]) -> Vec<GroupOut> {
    ["Blue", "Amber"].into_iter().enumerate().map(|(category, name)| {
        let members: Vec<_> = order.iter().copied().filter(|i| i % 2 == category).collect();
        let subs = [false, true].into_iter().map(|checked| {
            let sub: Vec<_> = members.iter().copied().filter(|i| (i % 3 == 0) == checked).collect();
            GroupOut {
                key: checked.to_string(), label: if checked { "Checked" } else { "Unchecked" }.into(),
                color: String::new(), count: sub.len(), rows: row_ids(&sub), subs: vec![], calcs: footer(&sub),
            }
        }).collect();
        GroupOut {
            key: category.to_string(), label: name.into(), color: name.to_lowercase(),
            count: members.len(), rows: row_ids(&members), subs, calcs: footer(&members),
        }
    }).collect()
}

fn reference_row(i: usize) -> RowOut {
    let computed = |n: usize| json!({"t": "number", "v": n as f64, "text": n.to_string()});
    RowOut {
        id: format!("r{i}"), title: format!("Row {i}"), icon: String::new(), number: 0,
        created: i as i64, updated: i as i64, favorite: false,
        number_text: [("amount".into(), (i % 13).to_string())].into(),
        number_input: [("amount".into(), (i % 13).to_string())].into(),
        values: json!({
            "amount": i % 13,
            "category": (i % 2).to_string(),
            "checked": i % 3 == 0,
            "double": computed((i % 13) * 2),
            "link": [format!("target-{}", i % 2)],
            "rollup": computed(i % 2 + 7),
        }).as_object().unwrap().clone(),
    }
}

fn check_at_scale(n: usize, filtered: bool) {
    let (items, db, mut view) = fixture(n);
    if filtered {
        view.filter.conds.push(Cond { prop: "checked".into(), op: "is".into(), value: "true".into(), ..Default::default() });
    }
    let order = reference_order(n, filtered);
    let mut cache = QueryCache::default();
    let snapshot = cache.query(items.iter(), &db, "table", view.clone(), (0, 0)).metadata(&db, "");
    assert_eq!(snapshot.total, n);
    assert_eq!(snapshot.filtered, order.len());
    assert!(!snapshot.truncated);
    assert_eq!(snapshot.row_ids, row_ids(&order));
    assert_eq!(snapshot.calcs, footer(&order));
    assert_eq!(serde_json::to_vec(&snapshot.groups).unwrap(), serde_json::to_vec(&Some(reference_groups(&order))).unwrap());

    let hit = cache.query(items.iter(), &db, "table", view.clone(), (0, 0));
    assert_eq!(hit.metadata(&db, "").generation, snapshot.generation);
    if n <= 50_000 {
        hit.check_shadow(items.iter(), &db);
        let ctx = Ctx::new(items.iter(), 0, 0);
        let historical = ctx.run_view_with_old_limit(&db, &view);
        assert_eq!(historical.filtered, snapshot.filtered);
        assert_eq!(serde_json::to_vec(&historical.groups).unwrap(), serde_json::to_vec(&snapshot.groups).unwrap());
        assert_eq!(historical.calcs, snapshot.calcs);
        for (page, rows) in historical.rows.chunks(256).enumerate() {
            let actual = hit.rows(&[RowRange { start: page * 256, count: rows.len() }]);
            assert!(serde_json::to_vec(&actual).unwrap() == serde_json::to_vec(rows).unwrap(), "old-window mismatch at {n} rows, page {page}");
        }
    }
    for (page, expected) in order.chunks(256).enumerate() {
        let entry = cache.get(&db.id, "table", &snapshot.generation).expect("generation survived scrolling");
        let actual = entry.rows(&[RowRange { start: page * 256, count: 256 }]);
        let expected: Vec<_> = expected.iter().copied().map(reference_row).collect();
        assert!(serde_json::to_vec(&actual).unwrap() == serde_json::to_vec(&expected).unwrap(), "uncapped-window mismatch at {n} rows, page {page}");
    }
    let entry = cache.get(&db.id, "table", &snapshot.generation).unwrap();
    assert!(entry.rows(&[RowRange { start: order.len(), count: 1 }]).is_empty());
    cache.invalidate();
    assert!(cache.get(&db.id, "table", &snapshot.generation).is_none());
    println!("{n} rows ({filtered:?} filtered): checked all {} ordered values, groups, totals and generation", order.len());
}

#[test]
fn p7_legacy_and_uncapped_reference() {
    for n in [1_000, 50_000, 50_001, 100_000, 200_000] {
        check_at_scale(n, false);
    }
    check_at_scale(50_000, true);
}
