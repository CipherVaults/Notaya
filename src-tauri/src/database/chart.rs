//! Chart data for a chart view (bar, line, donut, number): the rows of the view are grouped along
//! the x axis (by option, date bucket or number bucket), optionally split into series, and each
//! cell of the chart is a calculation over the rows in it. The UI draws the numbers (hand-written
//! SVG); nothing here needs a charting library.

use serde::Serialize;

use super::engine::Ctx;
use super::types::*;
use super::value::*;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
    pub text: String,
    pub color: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChartSeries {
    pub name: String,
    pub color: String,
    pub points: Vec<ChartPoint>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    pub labels: Vec<String>,
    pub series: Vec<ChartSeries>,
    pub total: Option<String>,
    pub rows: usize,
}

const PALETTE: [&str; 9] = ["blue", "orange", "green", "purple", "pink", "yellow", "red", "brown", "gray"];

fn pick_color(color: &str, i: usize) -> String {
    if color.is_empty() {
        PALETTE[i % PALETTE.len()].into()
    } else {
        color.into()
    }
}

fn y_value(ctx: &Ctx, db: &Database, rows: &[&Row], cfg: &ChartConfig) -> (f64, String) {
    let calc_name = if cfg.y_calc.is_empty() { "countAll" } else { cfg.y_calc.as_str() };
    let prop = db.prop(&cfg.y_prop);
    let vals: Vec<Val> = match prop {
        Some(p) => rows.iter().map(|r| ctx.value(db, r, p).unwrap_or(Val::Null)).collect(),
        None => rows.iter().map(|_| Val::Bool(true)).collect(),
    };
    let res = calc(if prop.is_none() && !matches!(calc_name, "countAll" | "count") { "countAll" } else { calc_name }, &vals);
    let n = match &res {
        Val::Num(n) => *n,
        _ => 0.0,
    };
    let text = match (&res, prop) {
        (Val::Num(n), Some(p)) if matches!(calc_name, "sum" | "avg" | "median" | "min" | "max" | "range") => format_number(*n, &p.number_format),
        _ => calc_display(calc_name, &res),
    };
    (n, text)
}

pub fn chart_data(ctx: &Ctx, db: &Database, view: &View) -> ChartData {
    let cfg = &view.chart;
    let all = ctx.rows_of(&db.id);
    let rows: Vec<&Row> = all.iter().copied().filter(|r| view.filter.is_empty() || ctx.matches(db, r, &view.filter)).collect();
    if cfg.kind == "number" || db.prop(&cfg.x_prop).is_none() {
        let (n, text) = y_value(ctx, db, &rows, cfg);
        return ChartData {
            labels: vec![],
            series: vec![ChartSeries { name: cfg.title.clone(), color: pick_color(&cfg.color, 0), points: vec![ChartPoint { label: String::new(), value: n, text: text.clone(), color: pick_color(&cfg.color, 0) }] }],
            total: Some(text),
            rows: rows.len(),
        };
    }
    let xg = GroupBy { prop: cfg.x_prop.clone(), sort: if cfg.x_sort.is_empty() { "manual".into() } else { cfg.x_sort.clone() }, date_by: cfg.x_by.clone(), number_step: cfg.x_step, hide_empty: cfg.hide_empty, ..GroupBy::default() };
    let groups = ctx.build_groups(db, &rows, &xg);
    let labels: Vec<String> = groups.iter().map(|g| g.label.clone()).collect();
    let by_id: std::collections::HashMap<&str, &Row> = rows.iter().map(|r| (r.id.as_str(), *r)).collect();
    let members = |ids: &[String]| -> Vec<&Row> { ids.iter().filter_map(|id| by_id.get(id.as_str()).copied()).collect() };

    // Series: one per value of the second grouping, or one for everything.
    let series = if let Some(gp) = db.prop(&cfg.group_prop) {
        let sg = GroupBy { prop: gp.id.clone(), sort: "manual".into(), ..GroupBy::default() };
        let subs = ctx.build_groups(db, &rows, &sg);
        subs.iter()
            .enumerate()
            .map(|(si, sub)| {
                let color = pick_color(&sub.color, si);
                let sub_ids: std::collections::HashSet<&String> = sub.rows.iter().collect();
                ChartSeries {
                    name: sub.label.clone(),
                    color: color.clone(),
                    points: groups
                        .iter()
                        .map(|g| {
                            let m: Vec<String> = g.rows.iter().filter(|id| sub_ids.contains(id)).cloned().collect();
                            let (n, text) = y_value(ctx, db, &members(&m), cfg);
                            ChartPoint { label: g.label.clone(), value: n, text, color: color.clone() }
                        })
                        .collect(),
                }
            })
            .collect()
    } else {
        vec![ChartSeries {
            name: if cfg.title.is_empty() { db.prop(&cfg.y_prop).map_or("Count".to_string(), |p| p.name.clone()) } else { cfg.title.clone() },
            color: pick_color(&cfg.color, 0),
            points: groups
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    let (n, text) = y_value(ctx, db, &members(&g.rows), cfg);
                    // A donut colors each slice; a bar chart uses one color unless the axis has option colors.
                    let color = if cfg.kind == "donut" || !g.color.is_empty() { pick_color(&g.color, i) } else { pick_color(&cfg.color, 0) };
                    ChartPoint { label: g.label.clone(), value: n, text, color }
                })
                .collect(),
        }]
    };
    let total = if cfg.kind == "donut" {
        let (_, text) = y_value(ctx, db, &rows, cfg);
        Some(text)
    } else {
        None
    };
    ChartData { labels, series, total, rows: rows.len() }
}
