//! WP-19 investigation. All regression inputs below are invented.
//! The optional local probe reads source/target paths only from environment variables.

use super::*;
use crate::importers::zip;
use serde_json::json;
use std::collections::{BTreeMap, HashSet};

const HUB: &str = "11111111111111111111111111111111";
const TABLE: &str = "22222222222222222222222222222222";
const CARD: &str = "33333333333333333333333333333333";

fn fixture(files: Vec<(String, Vec<u8>)>) -> Vfs {
    let refs: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
    Vfs::from_zip_bytes(zip::build(&refs, false)).unwrap()
}

fn table_paths(p: &plan::Plan) -> Vec<String> {
    p.skipped.iter().filter(|s| s.item.to_ascii_lowercase().ends_with(".csv") && s.reason.contains("database"))
        .map(|s| s.item.clone()).collect()
}

fn finish(vfs: &Vfs, p: &mut plan::Plan, s: &Session) -> Vec<Item> {
    let ctl = Control::standard();
    let (tables, _, _) = notion_db::read_tables(vfs, &table_paths(p));
    let rows = notion_db::prepare_pages(&tables, &mut p.pages);
    let (mut items, _) = p.items(&ctl, &[], 1, 0).unwrap();
    let sec = items.iter().find_map(|i| if let Item::Section(x) = i { Some(x.id.clone()) } else { None }).unwrap();
    let made = notion_db::import_databases(&tables, &rows, &items, &sec, s, 1, &|_| Ok(())).unwrap();
    super::drop_row_pages(&mut items, &made.removed_pages);
    items.extend(made.items);
    items
}

fn empty_session() -> Session {
    // Synthetic tests only: no export paths or real vaults are involved.
    let root = std::env::temp_dir().join(format!("notaya-n1-invented-{}", crate::model::new_id()));
    std::fs::create_dir_all(&root).unwrap();
    Session::create_open(&root.join("vault.nota")).unwrap()
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_csv_only_source_reaches_database_import() {
    let v = fixture(vec![(format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\n".to_vec())]);
    assert!(plan::build(&v, &Control::standard(), None, 1).is_ok(), "CSV-only sources need a container plan");
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_database_discovery_does_not_depend_on_capped_skip_report() {
    let mut files: Vec<_> = (0..500).map(|i| (format!("a{i:03}.bin"), vec![0])).collect();
    files.push(("Home.md".into(), b"# Home\n\nBody\n".to_vec()));
    files.push((format!("zTable {TABLE}_all.csv"), b"Name\nCard\n".to_vec()));
    let v = fixture(files);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p, &empty_session());
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Database(_))).count(), 1);
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_database_without_row_documents_stays_in_source_section() {
    let v = fixture(vec![
        ("Section A/Home.md".into(), b"# Home A\n\nBody\n".to_vec()),
        ("Section B/Home.md".into(), b"# Home B\n\nBody\n".to_vec()),
        (format!("Section B/Table {TABLE}_all.csv"), b"Name\nCard\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p, &empty_session());
    let wanted = items.iter().find_map(|i| if let Item::Section(s) = i { (s.name == "Section B").then_some(&s.id) } else { None }).unwrap();
    assert!(items.iter().any(|i| matches!(i, Item::Database(d) if &d.section_id == wanted)));
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_title_only_source_home_survives_row_cleanup() {
    let v = fixture(vec![
        (format!("Home {HUB}.md"), b"# Home\n".to_vec()),
        (format!("Home {HUB}/Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\n".to_vec()),
        (format!("Home {HUB}/Table {TABLE}/Card {CARD}.md"), b"# Card\n\nStatus: Done\n\nBody\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home_id = p.pages.iter().find(|p| p.title == "Home").unwrap().id.clone();
    let items = finish(&v, &mut p, &empty_session());
    assert!(items.iter().any(|i| matches!(i, Item::Page(p) if p.id == home_id)));
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_linked_row_has_one_body_and_links_target_the_row() {
    let v = fixture(vec![
        ("Home.md".into(), format!("# Home\n\n[Card](Card%20{CARD}.md)\n").into_bytes()),
        (format!("Card {CARD}.md"), b"# Card\n\nStatus: Done\n\nBody\n".to_vec()),
        (format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p, &empty_session());
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Row(_))).count(), 1);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Page(p) if p.title == "Card")).count(), 0);
    let row = items.iter().find_map(|i| if let Item::Row(r) = i { Some(r) } else { None }).unwrap();
    let home = items.iter().find_map(|i| if let Item::Page(p) = i { (p.title == "Home").then_some(p) } else { None }).unwrap();
    assert!(row.text.contains("Body"));
    assert!(home.content.contains(&row.id), "navigation should address the canonical row");
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_flat_home_links_define_containment() {
    // A proposed arrangement policy, not proof that arbitrary navigation means ownership.
    let v = fixture(vec![
        (format!("Home {HUB}.md"), format!("# Home\n\n[Child](Child%20{CARD}.md)\n").into_bytes()),
        (format!("Child {CARD}.md"), b"# Child\n\nBody\n".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home = p.pages.iter().position(|p| p.title == "Home").unwrap();
    assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, Some(home));
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_folder_without_id_keeps_its_source_home_as_parent() {
    let v = fixture(vec![
        (format!("Home {HUB}.md"), b"# Home\n\nBody\n".to_vec()),
        (format!("Home/Child {CARD}.md"), b"# Child\n\nBody\n".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home = p.pages.iter().position(|p| p.title == "Home").unwrap();
    assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, Some(home));
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_source_home_in_html_is_not_discarded_because_csv_shares_id() {
    // A proposed dashboard-preservation policy. Real HTML database views intentionally
    // share the CSV id; a separate home page is not established by that id alone.
    let v = fixture(vec![
        (format!("Home {HUB}.html"), b"<html><title>Home</title><body><p>Dashboard body</p></body></html>".to_vec()),
        (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        (format!("Card {CARD}.html"), b"<html><title>Card</title><body><p>Body</p></body></html>".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    assert!(p.pages.iter().any(|p| p.title == "Home"));
}

#[test]
#[ignore = "fails until WP-19 fix lanes N2/N3 land"]
fn n1_same_named_csv_views_with_different_ids_report_ambiguity() {
    // A proposed warning, never permission to merge distinct ids by their display titles.
    let a = format!("Table {TABLE}.csv");
    let b = format!("Table {HUB}_all.csv");
    let v = fixture(vec![
        (a.clone(), b"Name,Status\nCard,Done\n".to_vec()),
        (b.clone(), b"Name,Status,Detail\nCard,Done,Body\n".to_vec()),
    ]);
    let (_, _, notes) = notion_db::read_tables(&v, &[a, b]);
    // Reporting ambiguity avoids silently merging two genuinely distinct databases.
    assert!(!notes.is_empty());
}

fn error_kind(e: &str) -> &'static str {
    if e.contains("more than 5000 pages") { "page_limit" }
    else if e.contains("No pages were found") { "no_documents" }
    else if e.contains("too long") { "time_limit" }
    else if e.contains("larger than 2 GB") { "zip_limit" }
    else { "other_error_suppressed" }
}

/// Manual investigation only. No real paths, titles, cells, reasons or bodies are printed.
/// Uses the real planner, image checks, table builder, renderer and cleanup, without
/// saving imported content. The only vault on disk is an empty throwaway session.
#[test]
#[ignore = "manual count-only probe; source and throwaway home must be supplied through environment"]
fn n1_private_count_probe() {
    if std::env::var_os("N1_SOURCE").is_none() { return; }
    // Run this test alone; even an unexpected importer panic must not reveal a private value.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        let source = PathBuf::from(std::env::var_os("N1_SOURCE").unwrap());
        let home = PathBuf::from(std::env::var_os("NOTAYA_HOME").expect("home required"));
        let root = std::fs::canonicalize(PathBuf::from(std::env::var_os("N1_TEST_ROOT").expect("test root required"))).expect("test root unavailable");
        assert!(!root.starts_with(std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")).unwrap()));
        assert!(std::fs::canonicalize(&source).expect("source unavailable").starts_with(&root));
        assert_eq!(std::fs::canonicalize(home.parent().expect("home parent required")).unwrap(), root);
        assert!(home.file_name().is_some_and(|n| n.to_string_lossy().starts_with("home-")));
        let mode = std::env::var("N1_MODE").unwrap_or_else(|_| "folder".into());
        let start = std::time::Instant::now();
        let ctl = Control::standard();
        let vf = if mode == "zip" {
            let mut paths = vec![source];
            if let Some(second) = std::env::var_os("N1_SOURCE_2") { paths.push(PathBuf::from(second)); }
            Vfs::from_files(&paths)
        } else { Vfs::from_folder(&source) };
        let v = match vf { Ok(v) => v, Err(e) => { println!("N1_COUNTS {}", json!({"error":error_kind(&e),"phase":"vfs","ms":start.elapsed().as_millis()})); return } };
        let mut extensions = BTreeMap::<String, usize>::new();
        for p in v.paths() { let ext = p.rsplit('.').next().unwrap_or("").to_ascii_lowercase(); let key = if ["md","html","csv","png","jpg","jpeg","webp","pdf","docx","txt","svg"].contains(&ext.as_str()) {ext} else {"other".into()}; *extensions.entry(key).or_default() += 1; }
        let mut p = match plan::build(&v, &ctl, Some("Investigation"), 1) { Ok(p) => p, Err(e) => { println!("N1_COUNTS {}", json!({"error":error_kind(&e),"phase":"plan","vfs_extensions":extensions,"vfs_refused":v.refused.len(),"ms":start.elapsed().as_millis()})); return } };
        let mut skipped = BTreeMap::<&str, usize>::new();
        for x in &p.skipped { let key = if x.reason.contains("database") {"csv"} else if x.reason.contains("picture") {"image"} else if x.reason.contains("kind of file") {"unsupported"} else if x.reason.contains("links that Notion") {"index"} else {"other"}; *skipped.entry(key).or_default() += 1; }
        let planned = p.pages.len();
        let generated = p.pages.iter().filter(|p| p.key.starts_with("dir:")).count();
        let docs: HashSet<_> = p.pages.iter().filter(|p| !p.key.starts_with("dir:")).map(|p| p.id.clone()).collect();
        let source_count = docs.len();
        let top: HashSet<_> = p.pages.iter().filter(|p| p.parent.is_none() && !p.key.starts_with("dir:")).map(|p| p.id.clone()).collect();
        let urls = p.store_images(&v, &ctl, &mut |_,_| Ok(crate::model::new_id())).expect("image stage failed");
        let (tables, _, notes) = notion_db::read_tables(&v, &table_paths(&p));
        let csv_rows: usize = tables.iter().map(|t| t.table.rows.len()).sum();
        let row_pages = notion_db::prepare_pages(&tables, &mut p.pages);
        let row_ids: HashSet<_> = row_pages.iter().map(|r| r.page_id.clone()).collect();
        let source_row_ids: HashSet<_> = row_ids.intersection(&docs).cloned().collect();
        let (mut items, summary) = p.items(&ctl, &urls, 1, 0).expect("render failed");
        assert!(home.is_absolute() && !home.starts_with(env!("CARGO_MANIFEST_DIR")));
        std::fs::create_dir(&home).expect("home must be new");
        let session = Session::create_open(&home.join("empty-probe.nota")).expect("empty vault unavailable");
        let sec = items.iter().find_map(|i| if let Item::Section(s) = i { Some(s.id.clone()) } else { None }).expect("section unavailable");
        let made = notion_db::import_databases(&tables, &row_pages, &items, &sec, &session, 1, &|_| Ok(())).expect("database stage failed");
        let moved = made.removed_pages.len();
        super::drop_row_pages(&mut items, &made.removed_pages);
        let source_docs_removed = docs.iter().filter(|id| !made.removed_pages.contains(*id) && !items.iter().any(|i| i.id() == *id)).count();
        let top_removed = top.iter().filter(|id| !made.removed_pages.contains(*id) && !items.iter().any(|i| i.id() == *id)).count();
        items.extend(made.items);
        let pages: Vec<_> = items.iter().filter_map(|i| if let Item::Page(p) = i {Some(p)} else {None}).collect();
        let retained_row_pages = pages.iter().filter(|p| row_ids.contains(&p.id)).count();
        let paired_titles = pages.iter().filter(|p| items.iter().any(|i| matches!(i, Item::Row(r) if r.title.trim().to_lowercase() == p.title.trim().to_lowercase()))).count();
        let source_paired_titles = pages.iter().filter(|p| docs.contains(&p.id) && items.iter().any(|i| matches!(i, Item::Row(r) if r.title.trim().to_lowercase() == p.title.trim().to_lowercase()))).count();
        let mut duplicates = BTreeMap::<String, HashSet<String>>::new();
        for item in &items { if let Item::Row(r) = item { duplicates.entry(r.title.trim().to_lowercase()).or_default().insert(r.db.clone()); } }
        let mut d = json!({"planned_pages":planned,"generated_folder_pages":generated,"notebooks":items.iter().filter(|i| matches!(i,Item::Notebook(_))).count(),"sections":summary.sections,"pages":pages.len(),"databases":items.iter().filter(|i| matches!(i,Item::Database(_))).count(),"rows":items.iter().filter(|i| matches!(i,Item::Row(_))).count(),"root_pages":pages.iter().filter(|p| p.parent.is_empty()).count(),"loose_section_pages":pages.iter().filter(|pg| p.sections.iter().position(|s| s==plan::LOOSE_SECTION).is_some_and(|s| pg.section_id==items.iter().filter_map(|i| if let Item::Section(s)=i{Some(&s.id)}else{None}).nth(s).cloned().unwrap_or_default())).count(),"row_candidates":row_pages.len(),"row_pages_moved":moved,"row_pages_retained":retained_row_pages,"page_row_title_pairs":paired_titles,"source_docs_pruned":source_docs_removed,"root_source_docs_pruned":top_removed,"tables_read":tables.len(),"csv_rows":csv_rows,"table_notes":notes.len(),"images":summary.images,"skip_before":skipped,"skip_after_images":summary.skipped.len(),"vfs_refused":v.refused.len(),"vfs_extensions":extensions,"ms":start.elapsed().as_millis()});
        d["row_titles_in_multiple_databases"] = json!(duplicates.values().filter(|s| s.len() > 1).count());
        d["retained_row_roots"] = json!(pages.iter().filter(|p| row_ids.contains(&p.id) && p.parent.is_empty()).count());
        d["source_documents_planned"] = json!(source_count);
        d["source_row_candidates"] = json!(source_row_ids.len());
        d["generated_row_candidates"] = json!(row_ids.len() - source_row_ids.len());
        d["retained_source_row_pages"] = json!(pages.iter().filter(|p| source_row_ids.contains(&p.id)).count());
        d["source_page_row_title_pairs"] = json!(source_paired_titles);
        d["source_documents_survived"] = json!(pages.iter().filter(|p| docs.contains(&p.id)).count());
        d["final_skipped_count"] = json!(summary.skipped.iter().filter(|x| !tables.iter().any(|t| t.paths.contains(&x.item))).count());
        d["columns_min"] = json!(tables.iter().map(|t| t.table.header.len()).min().unwrap_or(0));
        d["columns_max"] = json!(tables.iter().map(|t| t.table.header.len()).max().unwrap_or(0));
        let shallow_depth = v.paths().filter(|p| p.ends_with(".md") || p.ends_with(".html")).map(|p| p.matches('/').count()).min();
        let shallow: HashSet<_> = v.paths().filter(|p| (p.ends_with(".md") || p.ends_with(".html")) && Some(p.matches('/').count()) == shallow_depth).cloned().collect();
        d["shallow_source_documents"] = json!(shallow.len());
        d["shallow_source_documents_planned"] = json!(p.pages.iter().filter(|p| shallow.contains(&p.key)).count());
        d["shallow_source_documents_survived"] = json!(p.pages.iter().filter(|p| shallow.contains(&p.key) && pages.iter().any(|q| q.id == p.id)).count());
        println!("N1_COUNTS {d}");
    });
    std::panic::set_hook(previous);
    assert!(result.is_ok(), "count-only probe stopped; details suppressed");
}
