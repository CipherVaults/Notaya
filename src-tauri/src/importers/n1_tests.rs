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

fn finish(vfs: &Vfs, p: &mut plan::Plan) -> Vec<Item> {
    let ctl = Control::standard();
    let (tables, _, _) = notion_db::read_tables(vfs, p, &ctl).unwrap();
    let rows = notion_db::prepare_pages(&tables, p, &ctl).unwrap();
    let mut made = notion_db::import_databases(&tables, &rows, p, 1, &ctl).unwrap();
    let (mut items, _) = p.items(&ctl, &[], 1, 0, &made.row_targets).unwrap();
    let gone = made.move_bodies(&mut items, &ctl).unwrap();
    super::drop_row_pages(&mut items, &gone, &p.generated_page_ids());
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
fn n1_csv_only_source_reaches_database_import() {
    let v = fixture(vec![(format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\n".to_vec())]);
    assert!(plan::build(&v, &Control::standard(), None, 1).is_ok(), "CSV-only sources need a container plan");
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Notebook(_))).count(), 1);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Section(_))).count(), 1);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Database(_))).count(), 1);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Row(_))).count(), 1);
    assert!(!items.iter().any(|i| matches!(i, Item::Page(_))));
}

#[test]
fn n1_database_discovery_does_not_depend_on_capped_skip_report() {
    let mut files: Vec<_> = (0..500).map(|i| (format!("a{i:03}.bin"), vec![0])).collect();
    files.push(("Home.md".into(), b"# Home\n\nBody\n".to_vec()));
    files.push((format!("zTable {TABLE}_all.csv"), b"Name\nCard\n".to_vec()));
    let v = fixture(files);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Database(_))).count(), 1);
}

#[test]
fn n1_database_without_row_documents_stays_in_source_section() {
    let v = fixture(vec![
        ("Section A/Home.md".into(), b"# Home A\n\nBody\n".to_vec()),
        ("Section B/Home.md".into(), b"# Home B\n\nBody\n".to_vec()),
        (format!("Section B/Table {TABLE}_all.csv"), b"Name\nCard\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    let wanted = items.iter().find_map(|i| if let Item::Section(s) = i { (s.name == "Section B").then_some(&s.id) } else { None }).unwrap();
    assert!(items.iter().any(|i| matches!(i, Item::Database(d) if &d.section_id == wanted)));
}

#[test]
fn n2_csv_only_section_is_kept_beside_an_unrelated_document_section() {
    let v = fixture(vec![
        ("Section A/Home.md".into(), b"# Home\n\nBody\n".to_vec()),
        (format!("Section B/Tables/Table {TABLE}_all.csv"), b"Name\nCard\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    let section = items.iter().find_map(|i| if let Item::Section(s) = i { (s.name == "Section B").then_some(&s.id) } else { None }).unwrap();
    assert!(items.iter().any(|i| matches!(i, Item::Database(d) if &d.section_id == section)));
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Page(_))).count(), 1, "CSV folders create no empty placeholders");
}

#[test]
fn n1_title_only_source_home_survives_row_cleanup() {
    let v = fixture(vec![
        (format!("Home {HUB}.md"), b"# Home\n".to_vec()),
        (format!("Home {HUB}/Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\n".to_vec()),
        (format!("Home {HUB}/Table {TABLE}/Card {CARD}.md"), b"# Card\n\nStatus: Done\n\nBody\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home_id = p.pages.iter().find(|p| p.title == "Home").unwrap().id.clone();
    let generated = p.generated_page_ids();
    assert!(!generated.contains(&home_id));
    let items = finish(&v, &mut p);
    assert!(items.iter().any(|i| matches!(i, Item::Page(p) if p.id == home_id)));
    assert!(items.iter().any(|i| matches!(i, Item::Database(db) if db.parent == home_id)));
    assert!(!items.iter().any(|i| generated.contains(i.id())), "empty generated row containers are still cleaned up");
}

#[test]
fn combined_zip_keeps_one_database_and_uses_markdown_csv_values() {
    let root = std::env::temp_dir().join(format!("notaya-combined-{}", crate::model::new_id()));
    std::fs::create_dir_all(&root).unwrap();
    let html = root.join("one.zip");
    let markdown = root.join("two.zip");
    let html_csv = format!("Export A/Collection {TABLE}_all.csv");
    let md_csv = format!("Export B/Collection {TABLE}_all.csv");
    let html_home = format!("Export A/Overview {HUB}.html");
    let md_home = format!("Export B/Overview {HUB}.md");
    std::fs::write(&html, zip::build(&[
        (&html_home, b"<h1>Overview</h1><p>Rich body</p>"),
        (&html_csv, b"Name\nHTML value\n"),
    ], true)).unwrap();
    std::fs::write(&markdown, zip::build(&[
        (&md_home, b"# Overview\n\nPlain body"),
        (&md_csv, b"Name\nCSV value\n"),
    ], true)).unwrap();
    for paths in [[html.clone(), markdown.clone()], [markdown, html]] {
        let v = Vfs::from_files(&paths).unwrap();
        let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
        let items = finish(&v, &mut p);
        assert_eq!(items.iter().filter(|item| matches!(item, Item::Database(_))).count(), 1);
        assert_eq!(items.iter().filter(|item| matches!(item, Item::Row(_))).count(), 1);
        assert!(items.iter().any(|item| matches!(item, Item::Row(row) if row.title == "CSV value")));
        assert!(items.iter().any(|item| matches!(item, Item::Page(page) if page.text.contains("Rich body"))));
    }

    let html = root.join("one.zip");
    let markdown = root.join("two.zip");
    std::fs::write(&markdown, zip::build(&[
        (&md_home, b"# Overview"),
        (&md_csv, b"Name\n\"unclosed"),
    ], true)).unwrap();
    let v = Vfs::from_files(&[html, markdown.clone()]).unwrap();
    let ctl = Control::standard();
    let plan = plan::build(&v, &ctl, None, 1).unwrap();
    let (tables, _, notes) = notion_db::read_tables(&v, &plan, &ctl).unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].table.rows, [["HTML value"]]);
    assert!(notes.iter().any(|note| note.contains("HTML export's CSV was used")));

    let csv_only = root.join("three.zip");
    let other_csv = format!("Export C/Collection {TABLE}_all.csv");
    std::fs::write(&csv_only, zip::build(&[(&other_csv, b"Name\nCSV-only value\n")], true)).unwrap();
    std::fs::write(&markdown, zip::build(&[
        (&md_home, b"# Overview"),
        (&md_csv, b"Name\nCSV value\n"),
    ], true)).unwrap();
    let v = Vfs::from_files(&[csv_only, markdown]).unwrap();
    let plan = plan::build(&v, &ctl, None, 1).unwrap();
    let (tables, _, _) = notion_db::read_tables(&v, &plan, &ctl).unwrap();
    assert_eq!(tables.len(), 2, "a CSV-only archive does not establish an HTML counterpart");
}

#[test]
fn idless_csv_copies_pair_only_when_logical_path_and_bytes_agree() {
    let root = std::env::temp_dir().join(format!("notaya-idless-csv-{}", crate::model::new_id()));
    std::fs::create_dir_all(&root).unwrap();
    let html = root.join("one.zip");
    let markdown = root.join("two.zip");
    std::fs::write(&html, zip::build(&[
        (&format!("Export A {TABLE}/Overview {HUB}.html"), b"<h1>Overview</h1><p>Body</p>"),
        (&format!("Export A {TABLE}/Items.csv"), b"Name\nSame\n"),
        (&format!("Export A {TABLE}/Different.csv"), b"Name\nHTML only\n"),
    ], true)).unwrap();
    std::fs::write(&markdown, zip::build(&[
        (&format!("Export B {CARD}/Overview {HUB}.md"), b"# Overview\n\nBody"),
        (&format!("Export B {CARD}/Items.csv"), b"Name\nSame\n"),
        (&format!("Export B {CARD}/Different.csv"), b"Name\nMarkdown only\n"),
    ], true)).unwrap();
    let v = Vfs::from_files(&[html, markdown]).unwrap();
    let ctl = Control::standard();
    let plan = plan::build(&v, &ctl, None, 1).unwrap();
    let (tables, _, _) = notion_db::read_tables(&v, &plan, &ctl).unwrap();
    assert_eq!(tables.iter().filter(|t| t.title == "Items" && t.paths.len() == 2).count(), 1);
    assert_eq!(tables.iter().filter(|t| t.title == "Different" && t.paths.len() == 1).count(), 2);
}

#[test]
fn ambiguous_rows_keep_the_source_body_and_unicode_matches_canonical_spelling() {
    let v = fixture(vec![
        (format!("Collection {TABLE}_all.csv"), "Name\nRepeat\nRepeat\nCaf\u{e9} \u{645}\u{631}\u{62d}\u{628}\u{627}\n".as_bytes().to_vec()),
        (format!("Collection {TABLE}/Repeat {CARD}.md"), b"# Repeat\n\nStandalone body".to_vec()),
        (format!("Collection {TABLE}/International {HUB}.md"), "# Cafe\u{301} \u{645}\u{631}\u{62d}\u{628}\u{627}\n\nInternational body".as_bytes().to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    assert!(items.iter().any(|item| matches!(item, Item::Page(page) if page.title == "Repeat" && page.text.contains("Standalone body"))));
    assert_eq!(items.iter().filter(|item| matches!(item, Item::Row(row) if row.title == "Repeat" && row.text.is_empty())).count(), 2);
    assert_eq!(items.iter().filter(|item| matches!(item, Item::Row(row) if row.text.contains("International body"))).count(), 1);
}

#[test]
fn n1_linked_row_has_one_body_and_links_target_the_row() {
    check_canonical_row_import(6);
}

#[test]
fn n3_canonical_row_navigation_works_at_3001_and_the_import_limit() {
    check_canonical_row_import(3_001);
    check_canonical_row_import(crate::importers::MAX_PAGES);
}

#[test]
fn n3_same_named_databases_fold_only_rows_with_a_unique_owner() {
    let v = fixture(vec![
        (format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Ready\nOnly A,Ready\n".to_vec()),
        (format!("Table {HUB}_all.csv"), b"Name,Status\nCard,Ready\nOnly B,Ready\n".to_vec()),
        (format!("Table {TABLE}/Card {CARD}.md"), b"# Card\n\nStatus: Ready\n\nBody for A".to_vec()),
        (format!("Table {TABLE}/Card {CARD}/A child.md"), b"# A child\n\nA child body".to_vec()),
        (format!("Table {HUB}/Card.md"), b"# Card\n\nStatus: Ready\n\nBody for B".to_vec()),
        (format!("Table {HUB}/Card/B child.md"), b"# B child\n\nB child body".to_vec()),
        ("Table/Card.md".into(), b"# Card\n\nStatus: Ready\n\nUnresolved folder body".to_vec()),
        ("Table/Card/Unresolved child.md".into(), b"# Unresolved child\n\nUnresolved child body".to_vec()),
        (format!("Table {CARD}/Card.md"), b"# Card\n\nStatus: Ready\n\nUnknown database ID body".to_vec()),
        ("Card.md".into(), b"# Card\n\nStatus: Ready\n\nUnresolved flat body".to_vec()),
        ("Home.md".into(), format!("# Home\n\n\u{2192} [A](Table%20{TABLE}/Card%20{CARD}.md)\n\n\u{2192} [B](Table%20{HUB}/Card.md)\n\n\u{2192} [Unresolved](Table/Card.md)").into_bytes()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    let database = |marker| items.iter().find_map(|item| match item {
        Item::Row(r) if r.title == marker => Some(&r.db),
        _ => None,
    }).unwrap();
    let row = |db| items.iter().find_map(|item| match item {
        Item::Row(r) if r.title == "Card" && &r.db == db => Some(r),
        _ => None,
    }).unwrap();
    let page = |text: &str| items.iter().find_map(|item| match item {
        Item::Page(p) if p.text.contains(text) => Some(p),
        _ => None,
    }).unwrap();
    let a = row(database("Only A"));
    let b = row(database("Only B"));
    assert_ne!(a.db, b.db);
    assert!(a.text.contains("Body for A") && !a.text.contains("Body for B"));
    assert!(b.text.contains("Body for B") && !b.text.contains("Body for A"));
    assert_eq!(page("A child body").parent, a.id);
    assert_eq!(page("B child body").parent, b.id);
    let unresolved = page("Unresolved folder body");
    assert_eq!(page("Unresolved child body").parent, unresolved.id);
    assert_eq!(page("Unresolved flat body").title, "Card");
    assert_eq!(page("Unknown database ID body").title, "Card");
    let home = items.iter().find_map(|item| match item {
        Item::Page(p) if p.title == "Home" => Some(p),
        _ => None,
    }).unwrap();
    for id in [&a.id, &b.id, &unresolved.id] {
        assert!(home.content.contains(&format!("\"pageId\":\"{id}\"")));
    }
}

#[test]
fn n3_flat_row_stays_ambiguous_when_same_named_databases_order_columns_differently() {
    let v = fixture(vec![
        (format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Ready\n".to_vec()),
        (format!("Table {HUB}_all.csv"), b"Status,Name\nReady,Card\nReady,Unique\n".to_vec()),
        ("Card.md".into(), b"# Card\n\nStatus: Ready\n\nAmbiguous body".to_vec()),
        ("Card/Child.md".into(), b"# Child\n\nChild body".to_vec()),
        ("Unique.md".into(), b"# Unique\n\nStatus: Ready\n\nUnique body".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    let cards: Vec<_> = items.iter().filter_map(|item| match item {
        Item::Row(r) if r.title == "Card" => Some(r),
        _ => None,
    }).collect();
    assert_eq!(cards.len(), 2);
    assert_ne!(cards[0].db, cards[1].db);
    assert!(cards.iter().all(|row| !row.text.contains("Ambiguous body")));

    let card = items.iter().find_map(|item| match item {
        Item::Page(p) if p.title == "Card" => Some(p),
        _ => None,
    }).expect("the ambiguous flat page stays a page");
    assert!(card.text.contains("Ambiguous body"));
    assert!(items.iter().any(|item| matches!(item, Item::Page(p) if p.title == "Child" && p.parent == card.id)));
    assert!(items.iter().any(|item| matches!(item, Item::Row(r) if r.title == "Unique" && r.text.contains("Unique body"))));
    assert!(!items.iter().any(|item| matches!(item, Item::Page(p) if p.title == "Unique")));
}

fn check_canonical_row_import(documents: usize) {
    let mut files = vec![
        ("Home.md".into(), format!("# Home\n\n[Card](Card%20{CARD}.md)\n\nOpen [the HTML copy](Card%20{CARD}.html).\n\n<details>\n<summary>Nested link</summary>\n\n\u{2192} [Card](Card%20{CARD}.md)\n\n</details>").into_bytes()),
        (format!("Card {CARD}.md"), b"# Card\n\nStatus: Done\n\nBody".to_vec()),
        (format!("Card {CARD}.html"), format!("<title>Card</title><p>Status: Done</p><p>Body! See <a href=\"Card%20{CARD}.md\">this card</a>.</p>").into_bytes()),
        (format!("Card {CARD}/Child.md"), format!("# Child\n\nStatus: A note\n\nBack to [Card](../Card%20{CARD}.md).").into_bytes()),
        (format!("Card {CARD}/Child/Grandchild.md"), b"# Grandchild\n\nGrandchild body".to_vec()),
        ("Sibling.md".into(), b"# Sibling\n\nStatus: Ready\n\nSibling body".to_vec()),
        ("Sibling/Sibling child.md".into(), b"# Sibling child\n\nIndependent child".to_vec()),
        (format!("Table {TABLE}_all.csv"), b"Name,Status\nCard,Done\nSibling,Ready\nChild,Ready\n".to_vec()),
    ];
    for n in 6..documents {
        files.push((format!("Card {CARD}/Child/Descendant {n}.md"), b"Another note".to_vec()));
    }
    let v = fixture(files);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    assert_eq!(p.pages.len(), documents, "the HTML twin is one source document");
    let source_id = p.pages.iter().find(|page| page.title == "Card").unwrap().id.clone();
    let items = finish(&v, &mut p);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Row(_))).count(), 3);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Page(_))).count(), documents - 2);
    assert!(items.iter().all(|i| i.id() != source_id));
    let page = |title| items.iter().find_map(|item| match item {
        Item::Page(p) if p.title == title => Some(p),
        _ => None,
    }).unwrap();
    let row = |title| items.iter().find_map(|item| match item {
        Item::Row(r) if r.title == title => Some(r),
        _ => None,
    }).unwrap();
    let card = row("Card");
    assert!(card.text.contains("Body!") && !card.text.contains("Status:"));
    let home = page("Home");
    let href = format!("\"href\":\"notaya:page/{}\"", card.id);
    assert!(home.content.contains(&format!("\"pageId\":\"{}\"", card.id)));
    assert!(home.content.contains(&href), "the HTML alias points to the same canonical row");
    let home_doc: serde_json::Value = serde_json::from_str(&home.content).unwrap();
    let toggle = home_doc["content"].as_array().unwrap().iter().find(|b| b["type"] == "toggle").unwrap();
    assert!(toggle["content"].as_array().unwrap().iter().any(|b| b["type"] == "pageLink" && b["attrs"]["pageId"] == card.id));
    assert!(card.content.contains(&href), "links inside a moved body use canonical IDs too");
    let child = page("Child");
    let grandchild = page("Grandchild");
    assert_eq!(child.parent, card.id);
    assert!(child.text.contains("Status: A note") && !row("Child").text.contains("A note"));
    assert_eq!(grandchild.parent, child.id);
    assert!(child.content.contains(&href));
    assert!(items.iter().all(|item| match item {
        Item::Page(p) => !p.content.contains(&child.id),
        Item::Row(r) => !r.content.contains(&child.id),
        _ => true,
    }), "the folder-owned child has no authored inbound link");
    let sibling = row("Sibling");
    let sibling_child = page("Sibling child");
    assert_eq!(sibling_child.parent, sibling.id);
    assert!(sibling.text.contains("Sibling body"));
    let card_id = card.id.clone();
    let child_id = child.id.clone();
    let sibling_id = sibling.id.clone();
    let sibling_child_id = sibling_child.id.clone();
    let home_id = home.id.clone();
    let expected: HashSet<_> = std::iter::once(card_id.clone()).chain(items.iter().filter_map(|item| match item {
        Item::Page(p) if p.parent == card.id || p.parent == child.id => Some(p.id.clone()),
        _ => None,
    })).collect();
    assert_eq!(expected.len(), documents - 3);
    let mut session = empty_session();
    session.put_many(items).unwrap();
    let target = |session: &Session, id: &str| {
        serde_json::to_value(crate::commands::notes::page_link_target(session, id)).unwrap()
    };
    assert_eq!(target(&session, &card_id)["isRow"], true);
    assert_eq!(target(&session, &card_id)["title"], "Card");
    assert_eq!(target(&session, &child_id)["isRow"], false);
    assert_eq!(target(&session, &child_id)["title"], "Child");
    let tree = |s: &Session| {
        serde_json::to_value(crate::commands::notes::list_notes_core(s, &crate::actions::Actor::Person).unwrap()).unwrap()
    };
    let metadata = tree(&session);
    assert_eq!(metadata["pages"].as_array().unwrap()[0]["id"], home_id);
    assert_eq!(metadata["pages"].as_array().unwrap().len(), 1);
    let row_pages = metadata["rowPages"].as_array().unwrap();
    assert_eq!(row_pages.len(), documents - 3);
    let direct: Vec<_> = row_pages.iter().filter(|p| p["parent"] == card_id).collect();
    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0]["id"], child_id);
    crate::commands::database::trash_rows(&mut session, std::slice::from_ref(&card_id)).unwrap();
    assert!(target(&session, &card_id).is_null() && target(&session, &child_id).is_null());
    assert_eq!(target(&session, &sibling_id)["title"], "Sibling");
    let after = tree(&session);
    assert_eq!(after["rowPages"].as_array().unwrap().len(), 1);
    assert_eq!(after["rowPages"][0]["id"], sibling_child_id);
    let trashed = session.items().filter_map(|item| match item {
        Item::Page(p) if p.trashed_at.is_some() => Some(p.id.clone()),
        Item::Row(r) if r.trashed_at.is_some() => Some(r.id.clone()),
        _ => None,
    }).collect::<HashSet<_>>();
    assert_eq!(trashed, expected);
    assert_eq!(session.items().filter(|item| matches!(item, Item::Row(_) | Item::Page(_))).count(), documents + 1);
}

#[test]
fn n1_flat_home_links_define_containment() {
    // Q66 did not approve containment inferred from arbitrary navigation. Keep this as
    // a negative policy probe: a link alone must not make its target a child page.
    let v = fixture(vec![
        (format!("Home {HUB}.md"), format!("# Home\n\n[Child](Child%20{CARD}.md)\n").into_bytes()),
        (format!("Child {CARD}.md"), b"# Child\n\nBody\n".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, None);
}

#[test]
fn n1_folder_without_id_keeps_its_source_home_as_parent() {
    let v = fixture(vec![
        (format!("Home {HUB}.md"), b"# Home\n\nBody\n".to_vec()),
        (format!("Home/Child {CARD}.md"), b"# Child\n\nBody\n".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home = p.pages.iter().position(|p| p.title == "Home").unwrap();
    assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, Some(home));
    assert_eq!(p.sections.len(), 1, "children do not get a second section");
    assert!(!p.pages.iter().any(|p| p.generated));
}

#[test]
fn n2_ambiguous_clean_names_are_reported_without_parenting() {
    for files in [
        vec![
            (format!("Home {HUB}.md"), b"# Home\n".to_vec()),
            (format!("Home {TABLE}.md"), b"# Home\n".to_vec()),
            ("Home/Child.md".into(), b"# Child\n\nBody\n".to_vec()),
        ],
        vec![
            (format!("Home {HUB}.md"), b"# Home\n".to_vec()),
            ("Home/Child.md".into(), b"# Child\n\nBody\n".to_vec()),
            (format!("Home {TABLE}/Other.md"), b"# Other\n\nBody\n".to_vec()),
        ],
    ] {
        let p = plan::build(&fixture(files), &Control::standard(), None, 1).unwrap();
        assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, None);
        assert_eq!(p.notes.iter().filter(|n| n.contains("ambiguous")).count(), 1);
    }
}

#[test]
fn n2_clean_name_matches_are_local_to_the_same_parent() {
    let v = fixture(vec![
        (format!("Area A/Home {HUB}.md"), b"# Home\n".to_vec()),
        ("Area B/Home/Child.md".into(), b"# Child\n\nBody\n".to_vec()),
    ]);
    let p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let home = p.pages.iter().position(|p| p.title == "Home" && !p.generated).unwrap();
    assert_ne!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, Some(home));
}

#[test]
fn n1_source_home_in_html_is_not_discarded_because_csv_shares_id() {
    // Mandatory A53 item 5: preserve useful body outside the CSV-backed view table.
    let v = fixture(vec![
        (format!("Home #1 {HUB}.html"), b"<html><title>Home</title><body><p>Status: Draft</p></body></html>".to_vec()),
        (format!("Home #1 {HUB}.csv"), b"Name,Status\nHome,Draft\n".to_vec()),
        (format!("Card {CARD}.html"), b"<html><title>Card</title><body><p>Body</p></body></html>".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    let home = items.iter().find_map(|i| match i {
        Item::Page(p) if p.title == "Home" && p.text == "Status: Draft" => Some(p),
        _ => None,
    }).expect("HTML home and its text survive");
    assert!(items.iter().any(|i| matches!(i, Item::Database(db) if db.parent == home.id && db.section_id == home.section_id && db.source.as_ref().is_some_and(|s| s.id == HUB))));
    assert!(items.iter().any(|i| matches!(i, Item::Row(row) if row.title == "Home" && row.text.is_empty())));
}

#[test]
fn n1_same_named_csv_views_with_different_ids_report_ambiguity() {
    // Mandatory A53 item 6: warn, never merge distinct ids by their display titles.
    let a = format!("Table {TABLE}.csv");
    let b = format!("Table {HUB}_all.csv");
    let v = fixture(vec![
        (a.clone(), b"Name,Status\nCard,Done\n".to_vec()),
        (b.clone(), b"Name,Status,Detail\nCard,Done,Body\n".to_vec()),
    ]);
    let ctl = Control::standard();
    let p = plan::build(&v, &ctl, None, 1).unwrap();
    let (_, _, notes) = notion_db::read_tables(&v, &p, &ctl).unwrap();
    // Reporting ambiguity avoids silently merging two genuinely distinct databases.
    assert!(!notes.is_empty());
}

#[test]
fn n2_database_wrappers_keep_only_useful_sanitized_content() {
    for (body, wanted) in [
        ("<h1>Home</h1><table class=\"collection-content\"><tr><td>View row</td></tr></table>", false),
        ("<h1>Home</h1><table><tr><td>View row</td></tr></table>", false),
        ("<h1>Home</h1><p>Dashboard body</p><table class=\"collection-content\"><tr><td>View row</td></tr></table><script>bad()</script>", true),
        ("<h1>Home</h1><table class=\"simple-table\"><tr><td>Useful table</td></tr></table>", true),
    ] {
        let v = fixture(vec![
            (format!("Home {HUB}.html"), format!("<html><title>Home</title><body>{body}</body></html>").into_bytes()),
            (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        ]);
        let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
        assert_eq!(p.pages.len(), usize::from(wanted));
        let items = finish(&v, &mut p);
        for page in items.iter().filter_map(|i| if let Item::Page(p) = i { Some(p) } else { None }) {
            assert!(!page.text.contains("View row") && !page.text.contains("bad()"));
            assert!(page.text.contains("Dashboard body") || page.text.contains("Useful table"));
        }
        assert_eq!(items.iter().filter(|i| matches!(i, Item::Database(_))).count(), 1);
    }
}

#[test]
fn n2_html_markdown_csv_wrapper_is_one_page_with_navigation() {
    let v = fixture(vec![
        (format!("Home {HUB}.html"), format!("<html><title>Home</title><body><p><a href=\"Card%20{CARD}.md\">Card</a></p><table class=\"collection-content\"><tr><td>View row</td></tr></table></body></html>").into_bytes()),
        (format!("Home {HUB}.md"), b"# Home\n\nMarkdown body\n".to_vec()),
        (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        (format!("Card {CARD}.md"), b"# Card\n\nCard body\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    let items = finish(&v, &mut p);
    assert_eq!(items.iter().filter(|i| matches!(i, Item::Page(p) if p.title == "Home")).count(), 1);
    let card = items.iter().find_map(|i| if let Item::Page(p) = i { (p.title == "Card").then_some(p) } else { None }).unwrap();
    let home = items.iter().find_map(|i| if let Item::Page(p) = i { (p.title == "Home").then_some(p) } else { None }).unwrap();
    assert!(home.content.contains(&card.id), "wrapper navigation survives");
    assert!(!home.text.contains("View row"));
}

fn structured_wrapper(body: &str) -> Vec<u8> {
    format!("<html><head><title>Home</title></head><body><article class=\"page sans\"><header><h1 class=\"page-title\">Home</h1></header><div class=\"page-body\">{body}</div></article></body></html>").into_bytes()
}

#[test]
fn n2c_empty_html_source_home_keeps_children_while_empty_view_creates_nothing() {
    for child_folder in [None, Some("Home".to_string()), Some(format!("Home {HUB}"))] {
        let home_path = format!("Home {HUB}.html");
        let mut files = vec![
            (home_path.clone(), structured_wrapper("")),
            (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        ];
        if let Some(folder) = &child_folder {
            files.push((format!("{folder}/Child.md"), b"# Child\n\nChild body\n".to_vec()));
        }
        let v = fixture(files);
        let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
        assert_eq!(p.sections.len(), 1);
        let home_id = if child_folder.is_some() {
            let home = p.pages.iter().position(|p| p.key == home_path).expect("a title-only source home must survive wrapper classification");
            assert!(!p.pages[home].generated);
            assert_eq!(p.pages.iter().find(|p| p.title == "Child").unwrap().parent, Some(home));
            Some(p.pages[home].id.clone())
        } else {
            assert!(p.pages.is_empty(), "an empty view without source-home structure creates no page");
            None
        };
        let items = finish(&v, &mut p);
        assert_eq!(items.iter().filter(|i| matches!(i, Item::Database(_))).count(), 1);
        assert_eq!(items.iter().filter(|i| matches!(i, Item::Row(_))).count(), 1);
        assert_eq!(items.iter().filter(|i| matches!(i, Item::Section(_))).count(), 1);
        let pages: Vec<_> = items.iter().filter_map(|i| if let Item::Page(p) = i { Some(p) } else { None }).collect();
        if let Some(id) = home_id {
            assert_eq!(pages.len(), 2);
            let home = pages.iter().find(|p| p.id == id).expect("source home survives cleanup");
            assert!(home.text.is_empty());
            let child = pages.iter().find(|p| p.title == "Child").unwrap();
            assert_eq!(child.parent, id);
            assert_eq!(child.section_id, home.section_id);
        } else {
            assert!(pages.is_empty());
        }
    }
}

#[test]
fn n2c_long_plain_wrapper_falls_back_to_a_complete_page() {
    struct InventedVault(Mutex<Session>);
    impl VaultAccess for InventedVault {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> { f(&self.0.lock().unwrap()) }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> { f(&mut self.0.lock().unwrap()) }
    }
    for (count, character) in [(2_000, 'x'), (2_001, 'x'), (2_000, '\u{03bb}'), (2_001, '\u{03bb}')] {
        let text: String = std::iter::repeat(character).take(count).collect();
        let files = [
            (format!("Home {HUB}.html"), structured_wrapper(&format!("<p>{text}</p>"))),
            (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        ];
        let root = std::env::temp_dir().join(format!("notaya-n2c-header-{}", crate::model::new_id()));
        let data = root.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let source = root.join("invented.zip");
        let refs: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
        std::fs::write(&source, zip::build(&refs, false)).unwrap();
        let path = data.join("vault.nota");
        let vault = InventedVault(Mutex::new(Session::create_open(&path).unwrap()));
        let summary = super::import_from(&vault, &[source], false, &data, 1, &Control::standard())
            .expect("a long wrapper must keep its page without aborting the import");
        drop(vault);
        let reopened = Session::unlock_open(&path).unwrap();
        let db = reopened.items().find_map(|i| if let Item::Database(d) = i { Some(d) } else { None }).unwrap();
        assert_eq!(reopened.items().filter(|i| matches!(i, Item::Row(_))).count(), 1);
        let pages: Vec<_> = reopened.items().filter_map(|i| if let Item::Page(p) = i { Some(p) } else { None }).collect();
        if count == 2_000 {
            assert_eq!(db.description, text);
            assert!(pages.is_empty());
            assert_eq!(summary.pages, 0);
        } else {
            assert!(db.description.is_empty());
            assert_eq!(pages.len(), 1);
            assert_eq!(pages[0].text, text);
            assert_eq!(summary.pages, 1);
        }
    }
}

#[test]
fn n2c_ambiguous_exact_stems_are_reported_without_parenting() {
    struct InventedVault(Mutex<Session>);
    impl VaultAccess for InventedVault {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> { f(&self.0.lock().unwrap()) }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> { f(&mut self.0.lock().unwrap()) }
    }
    let mut files = Vec::new();
    for (name, id) in [("Home", HUB), ("Board", TABLE)] {
        files.push((format!("{name} {id}.md"), format!("# {name}\n\nBody\n").into_bytes()));
        files.push((format!("{name} {id}/{name} child.md"), format!("# {name} child\n\nBody\n").into_bytes()));
        files.push((format!("{} {id}/{name} other.md", name.to_uppercase()), format!("# {name} other\n\nBody\n").into_bytes()));
    }
    // A unique exact match still wins even when another folder shares its clean name.
    files.push((format!("Solo {HUB}.md"), b"# Solo\n\nBody\n".to_vec()));
    files.push((format!("Solo {HUB}/Unique child.md"), b"# Unique child\n\nBody\n".to_vec()));
    files.push((format!("Solo {TABLE}/Unmatched.md"), b"# Unmatched\n\nBody\n".to_vec()));
    let root = std::env::temp_dir().join(format!("notaya-n2c-ambiguity-{}", crate::model::new_id()));
    let data = root.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let source = root.join("invented.zip");
    let refs: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
    std::fs::write(&source, zip::build(&refs, false)).unwrap();
    let path = data.join("vault.nota");
    let vault = InventedVault(Mutex::new(Session::create_open(&path).unwrap()));
    let summary = super::import_from(&vault, &[source], false, &data, 1, &Control::standard()).unwrap();
    let warnings: Vec<_> = summary.notes.iter().filter(|n| n.contains("ambiguous")).collect();
    assert_eq!(warnings.len(), 2, "one summary warning per ambiguous name");
    for name in ["Home", "Board"] {
        let expected = format!("The folder match for \"{name}\" is ambiguous in its source folder; those pages were left in their original arrangement.");
        assert_eq!(warnings.iter().filter(|n| n.as_str() == expected).count(), 1);
    }
    drop(vault);
    let reopened = Session::unlock_open(&path).unwrap();
    let pages: Vec<_> = reopened.items().filter_map(|i| if let Item::Page(p) = i { Some(p) } else { None }).collect();
    assert_eq!(pages.len(), 9, "all source pages survive");
    assert_eq!(summary.pages, 9);
    for name in ["Home", "Board"] {
        for title in [name.to_string(), format!("{name} child"), format!("{name} other")] {
            assert!(pages.iter().find(|p| p.title == title).unwrap().parent.is_empty(), "no ambiguous folder is guessed");
        }
    }
    let solo = pages.iter().find(|p| p.title == "Solo").unwrap();
    assert_eq!(pages.iter().find(|p| p.title == "Unique child").unwrap().parent, solo.id);
    assert!(pages.iter().find(|p| p.title == "Unmatched").unwrap().parent.is_empty());
}

#[test]
fn n2_plain_wrapper_text_is_saved_in_the_database_description_without_a_duplicate_page() {
    let v = fixture(vec![
        (format!("Home {HUB}.html"), structured_wrapper("<p>Read these invented instructions first.</p><script>bad()</script>")),
        (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
    ]);
    let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
    assert!(p.pages.is_empty());
    assert_eq!(p.database_headers.len(), 1);
    let root = std::env::temp_dir().join(format!("notaya-n2-header-{}", crate::model::new_id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("vault.nota");
    let mut session = Session::create_open(&path).unwrap();
    let items = finish(&v, &mut p);
    let db = items.iter().find_map(|i| if let Item::Database(d) = i { Some(d) } else { None }).unwrap();
    assert_eq!(db.description, "Read these invented instructions first.");
    assert!(!items.iter().any(|i| matches!(i, Item::Page(_))));
    let db_id = db.id.clone();
    session.put_many(items).unwrap();
    // Read back from the saved vault, not only the transient importer result.
    drop(session);
    let reopened = Session::unlock_open(&path).unwrap();
    assert!(reopened.items().any(|i| matches!(i, Item::Database(d) if d.id == db_id && d.description == "Read these invented instructions first.")));
}

#[test]
fn n2_wrapper_navigation_formatting_children_and_multiline_text_stay_on_pages() {
    for (body, extra, has_folder) in [
        ("<p><strong>Read this first.</strong></p>", None, false),
        ("<p>First line<br>Second line</p>", None, false),
        ("<p>First paragraph</p><p>Second paragraph</p>", None, false),
        ("<p>Home instructions</p>", None, true),
        ("<p>Home instructions</p>", Some(format!("[Home](Home%20{HUB}.html)")), false),
        ("<p><a href=\"https://example.com\">Read this first.</a></p>", None, false),
    ] {
        let mut files = vec![
            (format!("Home {HUB}.html"), structured_wrapper(body)),
            (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        ];
        if has_folder { files.push((format!("Home {HUB}/Child.md"), b"# Child\n\nChild body".to_vec())); }
        if let Some(link) = extra { files.push(("Guide.md".into(), format!("# Guide\n\n{link}").into_bytes())); }
        let v = fixture(files);
        let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
        let items = finish(&v, &mut p);
        let home = items.iter().find_map(|i| if let Item::Page(p) = i { (p.title == "Home").then_some(p) } else { None }).expect("a page preserves the richer content or incoming navigation");
        assert!(!home.text.is_empty());
        assert!(items.iter().any(|i| matches!(i, Item::Database(d) if d.description.is_empty())));
        if has_folder { assert!(items.iter().any(|i| matches!(i, Item::Page(p) if p.title == "Child" && p.parent == home.id))); }
    }
}

#[test]
fn n2_a_failed_or_ambiguous_database_keeps_its_plain_wrapper_page() {
    for csvs in [
        vec![(format!("Home {HUB}.csv"), b"".to_vec())],
        vec![(format!("A/Home {HUB}_all.csv"), b"Name\nCard\n".to_vec()), (format!("B/Home {HUB}_all.csv"), b"Name\nOther\n".to_vec())],
    ] {
        let mut files = vec![(format!("Home {HUB}.html"), structured_wrapper("<p>Preserve this invented header.</p>"))];
        files.extend(csvs);
        let v = fixture(files);
        let mut p = plan::build(&v, &Control::standard(), None, 1).unwrap();
        let items = finish(&v, &mut p);
        assert!(items.iter().any(|i| matches!(i, Item::Page(p) if p.title == "Home" && p.text == "Preserve this invented header.")));
        assert!(items.iter().all(|i| !matches!(i, Item::Database(d) if !d.description.is_empty())));
    }
}

#[test]
fn n2_conflicting_csv_names_warn_once_and_never_merge_by_name() {
    let paths = vec![format!("Table {TABLE}.csv"), format!("Table {TABLE}_all.csv"), format!("Table {HUB}.csv"), format!("Table {CARD}.csv")];
    let v = fixture(paths.iter().map(|p| (p.clone(), b"Name\nCard\n".to_vec())).collect());
    let ctl = Control::standard();
    let p = plan::build(&v, &ctl, None, 1).unwrap();
    let (tables, covered, notes) = notion_db::read_tables(&v, &p, &ctl).unwrap();
    assert_eq!(tables.len(), 3, "only the same-id plain/all pair is combined");
    assert_eq!(covered.len(), 4);
    assert_eq!(notes.iter().filter(|n| n.contains("ambiguous")).count(), 1);
}

fn persist_fixture(files: Vec<(String, Vec<u8>)>) -> (plan::ImportSummary, Vec<Item>) {
    struct InventedVault(Mutex<Session>);
    impl VaultAccess for InventedVault {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> { f(&self.0.lock().unwrap()) }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> { f(&mut self.0.lock().unwrap()) }
    }
    let root = std::env::temp_dir().join(format!("notaya-n2-invented-{}", crate::model::new_id()));
    let data = root.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let source = root.join("invented.zip");
    let refs: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
    std::fs::write(&source, zip::build(&refs, false)).unwrap();
    let vault = InventedVault(Mutex::new(Session::create_open(&data.join("vault.nota")).unwrap()));
    let summary = super::import_from(&vault, &[source], false, &data, 1, &Control::standard()).unwrap();
    let items = vault.with(|s| Ok(s.items().cloned().collect())).unwrap();
    (summary, items)
}

#[test]
fn n2_persisted_summary_counts_csv_beyond_the_skip_cap_correctly() {
    let mut files: Vec<_> = (0..500).map(|i| (format!("a{i:03}.bin"), vec![0])).collect();
    files.push(("Home.md".into(), b"# Home\n\nBody\n".to_vec()));
    files.push((format!("zTable {TABLE}_all.csv"), b"Name\nCard\n".to_vec()));
    let (summary, items) = persist_fixture(files);
    assert_eq!(summary.skipped.len(), 500);
    assert_eq!(summary.pages, items.iter().filter(|i| matches!(i, Item::Page(_))).count());
    assert!(summary.notes.iter().any(|n| n == "Imported 1 databases and 1 rows."));
    assert!(!summary.notes.iter().any(|n| n.contains("more items were not imported") || n.contains("more database view files")));
    assert!(!summary.skipped.iter().any(|s| s.item.ends_with(".csv")));
}

#[test]
fn n2_failed_csv_beyond_the_skip_cap_is_still_reported() {
    let mut files: Vec<_> = (0..500).map(|i| (format!("a{i:03}.bin"), vec![0])).collect();
    files.push(("Home.md".into(), b"# Home\n\nBody\n".to_vec()));
    files.push((format!("zTable {TABLE}.csv"), b"Name\n\"unclosed".to_vec()));
    let (summary, items) = persist_fixture(files);
    assert!(!items.iter().any(|i| matches!(i, Item::Database(_))));
    assert_eq!(summary.skipped.len(), 500);
    assert!(summary.notes.iter().any(|n| n.contains("1 more database view files were not imported")));
    assert!(summary.notes.iter().any(|n| n.contains("was not imported") && n.contains("quoted value")));
}

fn error_kind(e: &str) -> &'static str {
    if e.contains(&format!("more than {} pages", crate::importers::MAX_PAGES)) { "page_limit" }
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
        let (tables, _, notes) = notion_db::read_tables(&v, &p, &ctl).expect("CSV stage failed");
        let shallow_depth = v.paths().filter(|p| p.ends_with(".md") || p.ends_with(".html")).map(|p| p.matches('/').count()).min();
        let shallow: HashSet<_> = v.paths().filter(|p| (p.ends_with(".md") || p.ends_with(".html")) && Some(p.matches('/').count()) == shallow_depth).cloned().collect();
        let vfs_refused = v.refused.len();
        drop(v);
        let csv_rows: usize = tables.iter().map(|t| t.table.rows.len()).sum();
        let row_pages = notion_db::prepare_pages(&tables, &p, &ctl).expect("row matching failed");
        let row_ids: HashSet<_> = row_pages.iter().map(|r| p.pages[r.page].id.clone()).collect();
        let source_row_ids: HashSet<_> = row_ids.intersection(&docs).cloned().collect();
        assert!(home.is_absolute() && !home.starts_with(env!("CARGO_MANIFEST_DIR")));
        std::fs::create_dir(&home).expect("home must be new");
        let _session = Session::create_open(&home.join("empty-probe.nota")).expect("empty vault unavailable");
        let mut made = notion_db::import_databases(&tables, &row_pages, &mut p, 1, &ctl).expect("database stage failed");
        let (mut items, summary) = p.items(&ctl, &urls, 1, 0, &made.row_targets).expect("render failed");
        let removed_pages = made.move_bodies(&mut items, &ctl).expect("move failed");
        let moved = removed_pages.len();
        super::drop_row_pages(&mut items, &removed_pages, &p.generated_page_ids());
        let source_docs_removed = docs.iter().filter(|id| !removed_pages.contains(*id) && !items.iter().any(|i| i.id() == *id)).count();
        let top_removed = top.iter().filter(|id| !removed_pages.contains(*id) && !items.iter().any(|i| i.id() == *id)).count();
        items.extend(made.items);
        let pages: Vec<_> = items.iter().filter_map(|i| if let Item::Page(p) = i {Some(p)} else {None}).collect();
        let retained_row_pages = pages.iter().filter(|p| row_ids.contains(&p.id)).count();
        let paired_titles = pages.iter().filter(|p| items.iter().any(|i| matches!(i, Item::Row(r) if r.title.trim().to_lowercase() == p.title.trim().to_lowercase()))).count();
        let source_paired_titles = pages.iter().filter(|p| docs.contains(&p.id) && items.iter().any(|i| matches!(i, Item::Row(r) if r.title.trim().to_lowercase() == p.title.trim().to_lowercase()))).count();
        let mut duplicates = BTreeMap::<String, HashSet<String>>::new();
        for item in &items { if let Item::Row(r) = item { duplicates.entry(r.title.trim().to_lowercase()).or_default().insert(r.db.clone()); } }
        let mut d = json!({"planned_pages":planned,"generated_folder_pages":generated,"notebooks":items.iter().filter(|i| matches!(i,Item::Notebook(_))).count(),"sections":summary.sections,"pages":pages.len(),"databases":items.iter().filter(|i| matches!(i,Item::Database(_))).count(),"rows":items.iter().filter(|i| matches!(i,Item::Row(_))).count(),"root_pages":pages.iter().filter(|p| p.parent.is_empty()).count(),"loose_section_pages":pages.iter().filter(|pg| p.sections.iter().position(|s| s==plan::LOOSE_SECTION).is_some_and(|s| pg.section_id==items.iter().filter_map(|i| if let Item::Section(s)=i{Some(&s.id)}else{None}).nth(s).cloned().unwrap_or_default())).count(),"row_candidates":row_pages.len(),"row_pages_moved":moved,"row_pages_retained":retained_row_pages,"page_row_title_pairs":paired_titles,"source_docs_pruned":source_docs_removed,"root_source_docs_pruned":top_removed,"tables_read":tables.len(),"csv_rows":csv_rows,"table_notes":notes.len(),"images":summary.images,"skip_before":skipped,"skip_after_images":summary.skipped.len(),"vfs_refused":vfs_refused,"vfs_extensions":extensions,"ms":start.elapsed().as_millis()});
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
        d["shallow_source_documents"] = json!(shallow.len());
        d["shallow_source_documents_planned"] = json!(p.pages.iter().filter(|p| shallow.contains(&p.key)).count());
        d["shallow_source_documents_survived"] = json!(p.pages.iter().filter(|p| shallow.contains(&p.key) && pages.iter().any(|q| q.id == p.id)).count());
        println!("N1_COUNTS {d}");
    });
    std::panic::set_hook(previous);
    assert!(result.is_ok(), "count-only probe stopped; details suppressed");
}

#[test]
#[ignore = "requires an admitted fresh NOTAYA_N2_M2_QUALIFICATION_ROOT; run this exact test only"]
fn n2q_empty_source_home_survives_import_reopen_with_children() {
    use std::collections::BTreeSet;
    use std::io::Write;
    use std::path::{Component, PathBuf};

    struct InventedVault(Mutex<Session>);
    impl VaultAccess for InventedVault {
        fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
            f(&self.0.lock().unwrap())
        }
        fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
            f(&mut self.0.lock().unwrap())
        }
    }
    fn imported_ids(s: &Session) -> BTreeSet<String> {
        s.items().filter(|i| matches!(i,
            Item::Notebook(_) | Item::Section(_) | Item::Page(_) |
            Item::Database(_) | Item::Row(_)
        )).map(|i| i.id().to_owned()).collect()
    }

    // The separately reviewed launcher admits this exact fresh run root and its
    // ancestry. Never fall back to a user's home or a private-count probe path.
    let root = PathBuf::from(std::env::var_os("NOTAYA_N2_M2_QUALIFICATION_ROOT")
        .expect("an explicit fresh qualification root is required"));
    assert!(root.is_absolute(), "qualification root must be absolute");
    assert!(root.components().all(|c| match c {
        Component::Prefix(_) | Component::RootDir => true,
        Component::Normal(p) => p.to_str().is_some_and(|s| !s.contains(':')),
        _ => false,
    }), "qualification root must have only ordinary components");
    assert!(root.file_name().and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("n2-m2-")), "unexpected qualification root leaf");
    #[cfg(windows)]
    assert!(matches!(root.components().next(), Some(Component::Prefix(p))
        if matches!(p.kind(), std::path::Prefix::Disk(_))), "UNC/device roots are refused");
    let ancestors: Vec<_> = root.ancestors().skip(1).collect();
    assert!(ancestors.len() <= 32, "qualification ancestry must be bounded");
    for ancestor in ancestors.into_iter().rev() {
        let meta = std::fs::symlink_metadata(ancestor).expect("qualification parent unavailable");
        assert!(meta.is_dir() && !meta.file_type().is_symlink(), "ordinary parents required");
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            assert_eq!(meta.file_attributes() & (0x400 | 0x4000), 0,
                "reparse/encrypted qualification parents are refused");
        }
    }
    std::fs::create_dir(&root).expect("qualification root must be fresh; never overwrite");

    for (label, child_folder) in [
        ("empty-view", None),
        ("idless-home", Some("Home".to_owned())),
        ("exact-home", Some(format!("Home {HUB}"))),
    ] {
        let case = root.join(label);
        std::fs::create_dir(&case).unwrap();
        let data = case.join("data");
        std::fs::create_dir(&data).unwrap();
        let mut files = vec![
            (format!("Home {HUB}.html"), structured_wrapper("")),
            (format!("Home {HUB}.csv"), b"Name\nCard\n".to_vec()),
        ];
        if let Some(folder) = &child_folder {
            files.push((format!("{folder}/Child.md"), b"# Child\n\nChild body\n".to_vec()));
        }
        let source = case.join("invented.zip");
        let refs: Vec<_> = files.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
        let mut output = std::fs::OpenOptions::new().write(true).create_new(true)
            .open(&source).unwrap();
        output.write_all(&zip::build(&refs, false)).unwrap();
        drop(output);
        let path = data.join("vault.nota");
        let vault = InventedVault(Mutex::new(Session::create_open(&path).unwrap()));
        let summary = super::import_from(&vault, &[source], false, &data, 1, &Control::standard())
            .unwrap_or_else(|e| panic!("{label}: real import failed: {e}"));
        let (before_ids, home_id, child_id) = vault.with(|s| {
            let page_id = |title: &str| s.items().find_map(|i| match i {
                Item::Page(p) if p.title == title => Some(p.id.clone()),
                _ => None,
            });
            Ok((imported_ids(s), page_id("Home"), page_id("Child")))
        }).unwrap();
        drop(vault);

        let reopened = Session::unlock_open(&path)
            .unwrap_or_else(|e| panic!("{label}: saved vault did not reopen: {e}"));
        assert_eq!(imported_ids(&reopened), before_ids, "{label}: imported IDs changed");
        let notebooks: Vec<_> = reopened.items().filter_map(|i| if let Item::Notebook(n) = i { Some(n) } else { None }).collect();
        let sections: Vec<_> = reopened.items().filter_map(|i| if let Item::Section(s) = i { Some(s) } else { None }).collect();
        let databases: Vec<_> = reopened.items().filter_map(|i| if let Item::Database(d) = i { Some(d) } else { None }).collect();
        let rows: Vec<_> = reopened.items().filter_map(|i| if let Item::Row(r) = i { Some(r) } else { None }).collect();
        let pages: Vec<_> = reopened.items().filter_map(|i| if let Item::Page(p) = i { Some(p) } else { None }).collect();
        assert_eq!((notebooks.len(), sections.len(), databases.len(), rows.len()), (1, 1, 1, 1), "{label}");
        let database = databases[0];
        assert_eq!(sections[0].notebook_id, notebooks[0].id, "{label}");
        assert_eq!(database.section_id, sections[0].id, "{label}");
        assert_eq!(rows[0].db, database.id, "{label}");
        assert_eq!(rows[0].title, "Card", "{label}");
        let expected_pages = if child_folder.is_some() { 2 } else { 0 };
        assert_eq!(summary.pages, expected_pages, "{label}: summary pages");
        assert_eq!(pages.len(), expected_pages, "{label}: saved pages");
        let person = crate::actions::Actor::Person;
        let tree = serde_json::to_value(crate::commands::notes::list_notes_core(&reopened, &person).unwrap()).unwrap();
        let tree_pages = tree["pages"].as_array().expect("notes tree pages array");
        let tree_ids: BTreeSet<String> = tree_pages.iter().map(|p| p["id"].as_str().unwrap().to_owned()).collect();
        let page_ids: BTreeSet<String> = pages.iter().map(|p| p.id.clone()).collect();
        assert_eq!(tree_ids, page_ids, "{label}: notes tree omitted or duplicated a page");
        assert_eq!(tree_pages.len(), expected_pages, "{label}: duplicate tree metadata");
        assert_eq!(tree["databases"].as_array().unwrap().len(), 1, "{label}");
        if child_folder.is_some() {
            let home = pages.iter().find(|p| Some(p.id.as_str()) == home_id.as_deref()).expect("saved home ID missing");
            let child = pages.iter().find(|p| Some(p.id.as_str()) == child_id.as_deref()).expect("saved child ID missing");
            assert_eq!(home.title, "Home", "{label}");
            assert!(home.text.is_empty() && home.parent.is_empty(), "{label}: genuine empty root home changed");
            assert_eq!(child.title, "Child", "{label}");
            assert!(child.text.contains("Child body"), "{label}: child body missing");
            assert_eq!(child.parent, home.id, "{label}: saved child lost its home");
            assert_eq!(child.section_id, home.section_id, "{label}");
            assert_eq!(home.section_id, database.section_id, "{label}");
        } else {
            assert!(home_id.is_none() && child_id.is_none(), "empty view created a page");
        }
        for stored in &pages {
            let opened = crate::commands::notes::get_page_core(&reopened, &person, stored.id.clone()).unwrap();
            assert_eq!(opened.id, stored.id, "{label}: get-page ID");
            assert_eq!(opened.parent, stored.parent, "{label}: get-page parent");
            assert_eq!(opened.section_id, stored.section_id, "{label}: get-page section");
            assert_eq!(opened.content, stored.content, "{label}: get-page body");
            let meta = tree_pages.iter().find(|p| p["id"].as_str() == Some(stored.id.as_str())).unwrap();
            assert_eq!(meta["parent"].as_str(), Some(stored.parent.as_str()), "{label}: tree parent");
            assert_eq!(meta["sectionId"].as_str(), Some(stored.section_id.as_str()), "{label}: tree section");
        }
        println!("N2_M2_CASE {label} passed");
        // Retain the ZIP/vault evidence on success and on failure. No cleanup here.
    }
}
