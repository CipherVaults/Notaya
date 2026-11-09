#![allow(dead_code)]
// Development-only benchmark (EX-2026-11-09-65). Run from tools/db-bench:
//   cargo run --release -- 1000 10000 50000        (query timings at those row counts)
//   cargo run --release -- parse | search | load   (formula parse, search scan, loading micro-tests)
// It includes the REAL engine files from src-tauri/src/database/ and generates fake rows in memory.
use std::time::Instant;

pub mod error {
    pub type Result<T> = std::result::Result<T, Error>;
    #[derive(Debug)]
    pub enum Error { NotFound, Invalid(String), Other(String) }
    impl Error { pub fn invalid(m: impl Into<String>) -> Self { Error::Invalid(m.into()) } }
}
pub mod model {
    #[derive(Clone, serde::Serialize, serde::Deserialize, zeroize::Zeroize)]
    pub struct SourceTag { pub app: String, pub id: String }
    pub fn now_ms() -> i64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64 }
    pub fn new_id() -> String { use std::sync::atomic::*; static N: AtomicU64 = AtomicU64::new(1); format!("{:032x}", N.fetch_add(1, Ordering::Relaxed)) }
    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Item {
        Database(crate::database::types::Database),
        Row(crate::database::types::Row),
        Other(String),
    }
    impl Item { pub fn id(&self) -> &str { match self { Item::Database(d) => &d.id, Item::Row(r) => &r.id, Item::Other(s) => s } } }
}
pub mod database;
use database::{engine::Ctx, types::*};
use model::Item;

fn lcg(s: &mut u64) -> u64 { *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); *s >> 33 }

fn build(n: usize, with_formula: bool, with_rollup: bool, customers: usize) -> Vec<Item> {
    let mut seed = 42u64;
    let mut items = Vec::new();
    // customers db
    let mut cdb = Database::new("c0".into(), "s".into(), "Customers".into(), 0);
    cdb.props.push(Property { id: "tier".into(), name: "Score".into(), kind: PropKind::Number, ..Default::default() });
    let cdb_id = cdb.id.clone();
    items.push(Item::Database(cdb));
    for i in 0..customers {
        let mut r = Row::new(format!("cu{i:08}"), &cdb_id, format!("Customer {i}"), 0);
        r.cells = format!("{{\"tier\":{}}}", lcg(&mut seed) % 100);
        r.order = i as i64;
        items.push(Item::Row(r));
    }
    let mut db = Database::new("d0".into(), "s".into(), "Sales".into(), 0);
    let cats = ["Food","Rent","Travel","Tools","Fun","Health","Tax","Other"];
    db.props.push(Property { id: "amt".into(), name: "Amount".into(), kind: PropKind::Number, ..Default::default() });
    db.props.push(Property { id: "cat".into(), name: "Category".into(), kind: PropKind::Select,
        options: cats.iter().enumerate().map(|(i, c)| SelectOption { id: format!("o{i}"), name: c.to_string(), ..Default::default() }).collect(), ..Default::default() });
    db.props.push(Property { id: "day".into(), name: "Day".into(), kind: PropKind::Date, ..Default::default() });
    db.props.push(Property { id: "note".into(), name: "Note".into(), kind: PropKind::Text, ..Default::default() });
    db.props.push(Property { id: "cust".into(), name: "Customer".into(), kind: PropKind::Relation, relation_db: cdb_id.clone(), ..Default::default() });
    if with_formula {
        db.props.push(Property { id: "f1".into(), name: "Gross".into(), kind: PropKind::Formula,
            formula: "round(prop(\"Amount\") * 1.14 + if(prop(\"Category\") == \"Tax\", 10, 0), 2)".into(), ..Default::default() });
        db.props.push(Property { id: "f2".into(), name: "Label".into(), kind: PropKind::Formula,
            formula: "concat(prop(\"Category\"), \" - \", format(prop(\"Gross\")))".into(), ..Default::default() });
    }
    if with_rollup {
        db.props.push(Property { id: "ru".into(), name: "Cust score".into(), kind: PropKind::Rollup,
            rollup_relation: "cust".into(), rollup_prop: "tier".into(), rollup_calc: "sum".into(), ..Default::default() });
    }
    let dbid = db.id.clone();
    items.push(Item::Database(db));
    for i in 0..n {
        let mut r = Row::new(format!("ro{i:08}"), &dbid, format!("Row number {i}"), 1_700_000_000_000 + i as i64);
        let cu = lcg(&mut seed) as usize % customers.max(1);
        r.cells = format!(
            "{{\"amt\":{}.{:02},\"cat\":\"o{}\",\"day\":{{\"start\":\"2025-{:02}-{:02}\"}},\"note\":\"line {} some free text for the note column\",\"cust\":[\"cu{:08}\"]}}",
            lcg(&mut seed) % 5000, lcg(&mut seed) % 100, lcg(&mut seed) % 8, 1 + lcg(&mut seed) % 12, 1 + lcg(&mut seed) % 28, i, cu);
        r.order = i as i64;
        r.text = format!("Row number {i} plain text of the body for search");
        items.push(Item::Row(r));
    }
    items
}

fn ms(t: Instant) -> f64 { t.elapsed().as_secs_f64() * 1000.0 }

fn main() {
    if std::env::args().nth(1).as_deref() == Some("parse") { parse_micro(); return; }
    if std::env::args().nth(1).as_deref() == Some("search") { search_micro(); return; }
    if std::env::args().nth(1).as_deref() == Some("load") { load_micro(); return; }
    let sizes: Vec<usize> = std::env::args().skip(1).map(|a| a.parse().unwrap()).collect();
    for n in sizes {
        println!("==== {n} rows ====");
        for (label, formula, rollup) in [("plain (no formulas)", false, false), ("2 formulas", true, false), ("2 formulas + rollup", true, true)] {
            let items = build(n, formula, rollup, 2000);
            let db = items.iter().find_map(|i| if let Item::Database(d) = i { if d.id == "d0" { Some(d.clone()) } else { None } } else { None }).unwrap();
            let t = Instant::now();
            let ctx = Ctx::new(items.iter(), 0, 0);
            let t_ctx = ms(t);
            let dbref = ctx.dbs.get("d0").copied().unwrap();
            let mut v = View::default();
            v.columns = db.props.iter().map(|p| Column { prop: p.id.clone(), visible: true, width: 0 }).collect();
            let run = |label2: &str, view: &View| {
                let ctx = Ctx::new(items.iter(), 0, 0); // fresh caches, like the real per-call context
                let dbref = ctx.dbs.get("d0").copied().unwrap();
                let t = Instant::now();
                let r = ctx.run_view(dbref, view);
                let t_run = ms(t);
                let t = Instant::now();
                let s = serde_json::to_string(&r).unwrap();
                let t_ser = ms(t);
                println!("  [{label}] {label2:<34} compute {t_run:>8.1} ms | JSON {t_ser:>7.1} ms | {:>6.1} MB | rows out {}", s.len() as f64 / 1e6, r.rows.len());
            };
            run("no sort/filter", &v);
            let mut v2 = v.clone();
            v2.sorts = vec![Sort { prop: "amt".into(), descending: true }];
            run("sort by Amount", &v2);
            let mut v3 = v.clone();
            v3.filter.conds = vec![Cond { prop: "amt".into(), op: "gt".into(), value: "4000".into(), ..Default::default() }];
            run("filter Amount>4000", &v3);
            if formula {
                let mut v4 = v.clone();
                v4.sorts = vec![Sort { prop: "f1".into(), descending: true }];
                run("sort by formula", &v4);
                let mut v5 = v.clone();
                v5.filter.conds = vec![Cond { prop: "f1".into(), op: "gt".into(), value: "5000".into(), ..Default::default() }];
                run("filter on formula", &v5);
            }
            let mut v6 = v.clone();
            v6.group = Some(GroupBy { prop: "cat".into(), sort: "asc".into(), ..Default::default() });
            v6.calcs = vec![ColCalc { prop: "amt".into(), calc: "sum".into() }];
            run("group by Category + sum footer", &v6);
            // one cell edit: what set_cells does afterwards (row_result builds a whole Ctx)
            let t = Instant::now();
            let ctx2 = Ctx::new(items.iter(), 0, 0);
            let dbref2 = ctx2.dbs.get("d0").copied().unwrap();
            let row = ctx2.rows_of("d0")[n / 2];
            let ro = ctx2.row_out_pub(dbref2, row);
            let _ = serde_json::to_string(&ro).unwrap();
            println!("  [{label}] after one cell edit: row_result {:>7.1} ms (Ctx::new alone {:.1} ms)", ms(t), t_ctx);
            let _ = dbref;
        }
    }
}

#[allow(dead_code)]
pub fn parse_micro() {
    let f = "round(prop(\"Amount\") * 1.14 + if(prop(\"Category\") == \"Tax\", 10, 0), 2)";
    let t = Instant::now();
    let mut ok = 0;
    for _ in 0..100_000 { if database::formula::parse(f).is_ok() { ok += 1; } }
    println!("parse x100000 = {:.1} ms (ok {ok})", ms(t));
}

pub fn search_micro() {
    for n in [10_000usize, 100_000, 200_000] {
        let items = build(n, false, false, 10);
        let terms = ["number", "12"];
        let t = Instant::now();
        let mut hits = 0;
        for it in &items {
            if let Item::Row(r) = it {
                let title = r.title.to_lowercase();
                let hay = format!("{title} {}", r.text.to_lowercase());
                if terms.iter().all(|t| hay.contains(t)) { hits += 1; }
            }
        }
        println!("search-like scan {n} rows: {:.1} ms ({hits} hits)", ms(t));
    }
}

pub fn load_micro() {
    for n in [50_000usize, 200_000] {
        let items = build(n, false, false, 10);
        let blobs: Vec<String> = items.iter().map(|i| serde_json::to_string(i).unwrap()).collect();
        let bytes: usize = blobs.iter().map(|b| b.len()).sum();
        let t = Instant::now();
        let back: Vec<Item> = blobs.iter().map(|b| serde_json::from_str(b).unwrap()).collect();
        println!("parse {n} stored rows ({:.0} MB of JSON): {:.0} ms (decryption extra; ChaCha20 runs ~1 GB/s => ~{:.0} ms)", bytes as f64 / 1e6, ms(t), bytes as f64 / 1e9 * 1000.0);
        drop(back);
    }
}
