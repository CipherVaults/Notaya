#![allow(dead_code)]

// Reuse db-bench's stand-ins and its production-source includes without changing the bench.
#[allow(unused_variables)]
#[path = "../db-bench/src/main.rs"]
mod bench;
pub use bench::{database, error, model};
use database::{engine, formula, types, value};
#[path = "../../src-tauri/src/database/chart.rs"]
mod chart;

use engine::Ctx;
use model::Item;
use serde::Deserialize;
use serde_json::{Value, json};
use std::io;
use types::{ChartConfig, ColCalc, Database, GroupBy, PropKind, Property, Row, Sort, View};
use value::{Number, Val};

#[derive(Deserialize)]
struct Case {
    id: String,
    target: String,
    currency_scale: Option<u32>,
    expression: Option<String>,
    input: Option<String>,
    rows: Option<Vec<String>>,
    operation: Option<String>,
    row_source: Option<String>,
    work_repeat: Option<usize>,
    sort_expressions: Option<[String; 2]>,
}

struct NoProperties;
impl formula::Env for NoProperties {
    fn prop(&self, name: &str) -> Result<Val, String> {
        Err(format!("Unexpected property: {name}"))
    }
    fn row_id(&self) -> String {
        "synthetic".into()
    }
    fn now(&self) -> i64 {
        0
    }
}

fn case_db(id: &str) -> Database {
    Database::new(id.into(), "synthetic".into(), id.into(), 0)
}

fn property(id: &str, kind: PropKind) -> Property {
    Property {
        id: id.into(),
        name: id.into(),
        kind,
        ..Default::default()
    }
}

fn make_row(db: &str, ix: usize, cells: String) -> Row {
    let id = format!("{db}-{ix}");
    let mut row = Row::new(id.clone(), db, id, 0);
    row.order = ix as i64;
    row.cells = cells;
    row
}

fn add_rollup(db: &mut Database, related_db: &str, operation: &str, format: &str) {
    db.props.extend([
        Property {
            relation_db: related_db.into(),
            ..property("links", PropKind::Relation)
        },
        Property {
            rollup_relation: "links".into(),
            rollup_prop: "amount".into(),
            rollup_calc: operation.into(),
            number_format: format.into(),
            ..property("amount", PropKind::Rollup)
        },
    ]);
}

fn read_computed(ctx: &Ctx, db: &Database, row: &Row) -> Value {
    let raw = match ctx.value(db, row, db.prop("amount").unwrap()) {
        Ok(Val::Num(n)) => n.try_as_f64().ok().map(|value| value.to_string()),
        _ => None,
    };
    let computed = ctx.row_out_pub(db, row).values.remove("amount").unwrap();
    json!({ "raw": raw, "computed": computed })
}

fn exact_meter(case: &Case) -> Value {
    let value = Number::decimal(case.input.as_deref().unwrap()).unwrap();
    let divisor = Number::integer(1);
    let mut charges = Vec::new();
    let error = "meter rejected work";
    let mut reject = |cost| {
        charges.push(cost);
        Err(error.into())
    };
    let rejected = [
        value.significant_checked(&mut reject).as_ref().err().is_some_and(|err| err == error),
        value.float_checked(&mut reject).as_ref().err().is_some_and(|err| err == error),
        value.rem(&divisor, &mut reject).as_ref().err().is_some_and(|err| err == error),
    ];
    json!({ "charges": charges, "rejected": rejected })
}

fn view_sort(case: &Case) -> Value {
    let [first, second] = case.sort_expressions.as_ref().unwrap();
    let mut db = case_db("sort");
    db.props.extend([
        property("input", PropKind::Number),
        Property {
            formula: format!(r#"if(prop("input") == 0, {first}, {second})"#),
            ..property("amount", PropKind::Formula)
        },
    ]);
    let items = [
        Item::Database(db),
        Item::Row(make_row("sort", 0, r#"{"input":0}"#.into())),
        Item::Row(make_row("sort", 1, r#"{"input":1}"#.into())),
    ];
    let view = View {
        sorts: vec![Sort { prop: "amount".into(), descending: false }],
        ..Default::default()
    };
    let ctx = Ctx::new(items.iter(), 0, 0);
    let order: Vec<usize> = ctx.run_view(ctx.dbs["sort"], &view).rows.iter()
        .map(|row| row.id.strip_prefix("sort-").unwrap().parse().unwrap())
        .collect();
    json!({ "order": order })
}

fn aggregate(case: &Case, format: &str) -> Value {
    let rows = case.rows.as_ref().unwrap();
    let operation = case.operation.as_deref().unwrap();
    let mut items = Vec::new();
    let mut db = case_db("inputs");
    for id in ["group", "subgroup"] {
        db.props.push(property(id, PropKind::Text));
    }

    if case.row_source.as_deref() == Some("rollup") {
        // Each input is a rollup over its own leaf. Text reaches the production toNumber()
        // path, since JSON numbers cannot represent NaN or Infinity.
        let mut leaf = case_db("leaf");
        leaf.props.push(property("source", PropKind::Text));
        leaf.props.push(Property {
            formula: "toNumber(prop(\"source\"))".into(),
            ..property("amount", PropKind::Formula)
        });
        items.push(Item::Database(leaf));
        add_rollup(&mut db, "leaf", "sum", format);
        for (ix, text) in rows.iter().enumerate() {
            items.push(Item::Row(make_row(
                "leaf",
                ix,
                json!({ "source": text }).to_string(),
            )));
            let cells = json!({ "group": "G", "subgroup": "S", "links": [format!("leaf-{ix}")] });
            items.push(Item::Row(make_row("inputs", ix, cells.to_string())));
        }
    } else {
        db.props.push(Property {
            number_format: format.into(),
            ..property("amount", PropKind::Number)
        });
        for (ix, text) in rows.iter().enumerate() {
            let group = if case.target == "rust.donut-slice" && ix == 1 { "H" } else { "G" };
            let cells = format!(r#"{{"group":"{group}","subgroup":"S","amount":{text}}}"#);
            items.push(Item::Row(make_row("inputs", ix, cells)));
        }
        if case.target == "rust.donut-slice" {
            items.push(Item::Row(make_row("inputs", rows.len(), r#"{"group":"G","subgroup":"T","amount":0}"#.into())));
        }
    }
    items.push(Item::Database(db));

    if case.target == "rust.rollup" {
        let mut top = case_db("total");
        add_rollup(&mut top, "inputs", operation, format);
        let links: Vec<_> = (0..rows.len()).map(|ix| format!("inputs-{ix}")).collect();
        items.push(Item::Database(top));
        items.push(Item::Row(make_row(
            "total",
            0,
            json!({ "links": links }).to_string(),
        )));
        let ctx = Ctx::new(items.iter(), 0, 0);
        return read_computed(&ctx, ctx.dbs["total"], ctx.rows_of("total")[0]);
    }

    let ctx = Ctx::new(items.iter(), 0, 0);
    let db = ctx.dbs["inputs"];
    let mut view = View {
        calcs: vec![ColCalc {
            prop: "amount".into(),
            calc: operation.into(),
        }],
        group: Some(GroupBy {
            prop: "group".into(),
            ..Default::default()
        }),
        sub_group: Some(GroupBy {
            prop: "subgroup".into(),
            ..Default::default()
        }),
        chart: ChartConfig {
            kind: "number".into(),
            y_prop: "amount".into(),
            y_calc: operation.into(),
            ..Default::default()
        },
        ..Default::default()
    };
    if case.target == "rust.donut-slice" {
        view.chart.kind = "donut".into();
        view.chart.x_prop = "group".into();
        view.chart.group_prop = "subgroup".into();
        let chart = chart::chart_data(&ctx, db, &view).unwrap();
        let series = chart.series.iter().find(|s| s.name == "S").unwrap();
        let slice = series.donut_slice.as_ref().unwrap();
        return json!({ "raw": slice.value.to_string(), "text": slice.text, "total": chart.total,
            "points": series.points.iter().map(|p| &p.text).collect::<Vec<_>>() });
    }
    if case.target == "rust.chart-total" {
        let chart = match chart::chart_data(&ctx, db, &view) {
            Ok(chart) => chart,
            Err(error) => return json!({ "error": error }),
        };
        let point = &chart.series[0].points[0];
        return json!({ "raw": point.value.to_string(), "text": chart.total });
    }

    let result = ctx.run_view(db, &view);
    let calcs = match case.target.as_str() {
        "rust.view-footer" => result.calcs,
        "rust.group-footer" | "rust.subgroup-footer" => {
            let [group] = result.groups.unwrap().try_into().ok().unwrap();
            let group: engine::GroupOut = group;
            assert_eq!(group.rows.len(), rows.len(), "group membership");
            if case.target == "rust.group-footer" {
                group.calcs
            } else {
                let [subgroup]: [engine::GroupOut; 1] = group.subs.try_into().ok().unwrap();
                assert_eq!(subgroup.rows.len(), rows.len(), "subgroup membership");
                subgroup.calcs
            }
        }
        _ => panic!("Unknown target {}", case.target),
    };
    json!({ "text": calcs.get("amount").expect("missing footer") })
}

fn main() {
    let cases: Vec<Case> = serde_json::from_reader(io::stdin().lock()).unwrap();
    let results: Vec<_> = cases.into_iter().map(|case| {
        let format = if case.currency_scale.is_some() { "dollar" } else { "number" };
        let mut result = match case.target.as_str() {
            "rust.formula" | "rust.formula-budget" => {
                let generated;
                let expression = if let Some(repeat) = case.work_repeat {
                    let list = vec!["0"; repeat].join(",");
                    generated = format!("lets(a,(2/7)^2400,b,(3/5)^2400,map([{list}],a*b))");
                    &generated
                } else {
                    case.expression.as_deref().unwrap()
                };
                let observed = formula::run(expression, &NoProperties).and_then(|value| {
                    match value {
                        Val::Num(n) => {
                            let raw = match case.currency_scale {
                                Some(places) => n.fixed(places)?,
                                None => n.try_as_f64()?.to_string(),
                            };
                            let text = value::format_number(&n, format)?;
                            Ok(json!({ "raw": raw, "text": text }))
                        }
                        other => Ok(json!({ "returned_type": other.type_name(), "value": engine::val_json(&other)? })),
                    }
                });
                observed.unwrap_or_else(|error| json!({ "error": error }))
            }
            "rust.computed" => {
                let mut db = case_db("formulas");
                db.props.push(Property {
                    formula: case.expression.as_ref().unwrap().clone(),
                    number_format: format.into(),
                    ..property("amount", PropKind::Formula)
                });
                let row = make_row("formulas", 0, "{}".into());
                let items = [Item::Database(db), Item::Row(row)];
                let ctx = Ctx::new(items.iter(), 0, 0);
                read_computed(&ctx, ctx.dbs["formulas"], ctx.rows_of("formulas")[0])
            }
            "rust.view-sort" => view_sort(&case),
            "rust.exact-meter" => exact_meter(&case),
            _ => aggregate(&case, format),
        };
        result["id"] = json!(case.id);
        result["target"] = json!(case.target);
        result
    }).collect();
    serde_json::to_writer(io::stdout().lock(), &results).unwrap();
}
