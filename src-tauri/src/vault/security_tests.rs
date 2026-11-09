//! Offline SQL attacks and credentialed current-file scans on throwaway vaults.
use super::*;
use crate::{
    database::types::{Database, Row, Template},
    model::{new_id, now_ms},
};
use rusqlite::{params, Connection};
use std::path::PathBuf;

fn fixture() -> (Session, PathBuf, Zeroizing<String>) {
    let root =
        PathBuf::from(std::env::var_os("NOTAYA_HOME").expect("throwaway NOTAYA_HOME required"));
    let dir = root.join(format!("sf3-{}", new_id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("vault.nota");
    let password = Zeroizing::new(new_id());
    let session = Session::create(
        &path,
        &password,
        None,
        KdfParams {
            m_kib: 64,
            t: 1,
            p: 1,
        },
    )
    .unwrap();
    (session, path, password)
}
fn ai() -> Actor {
    Actor::Ai {
        pairing_id: "test".into(),
        label: "Test AI".into(),
    }
}
fn page(marker: &str) -> Item {
    serde_json::from_value(serde_json::json!({"kind":"page", "id":new_id(), "sectionId":"", "title":"Page",
        "content":marker,"text":marker,"order":0,"favorite":false,"trashedAt":null,"created":1,"updated":1})).unwrap()
}
fn edit(s: &mut Session, id: &str) -> String {
    let mut item = s.get(id).unwrap().clone();
    match &mut item {
        Item::Page(p) => {
            p.content = "after".into();
            p.text = p.content.clone();
        }
        Item::Row(p) => {
            p.content = "after".into();
            p.text = p.content.clone();
            p.cells = "{}".into();
        }
        Item::Sticky(p) => {
            p.content = "after".into();
            p.text = p.content.clone();
        }
        Item::Database(p) => {
            p.templates[0].content = "after".into();
            p.templates[0].cells = "{}".into();
        }
        _ => unreachable!(),
    }
    s.run_as(&ai(), |s| s.put(item)).unwrap();
    s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0]
        .change_set
        .clone()
}
fn assert_absent(s: &Session, marker: &str) {
    for table in [
        Table::Records,
        Table::Attachments,
        Table::Activity,
        Table::History,
        Table::Anchor,
    ] {
        for (id, bytes) in s.store.all(table).unwrap() {
            let plain =
                crypto::open(&s.dek, &aad(s.format, &s.vault_id, table, &id), &bytes).unwrap();
            assert!(
                !plain.windows(marker.len()).any(|w| w == marker.as_bytes()),
                "deleted content retained in {table:?}"
            );
        }
    }
}
fn rejected_by_restore(path: &Path, password: &str) {
    let target =
        crate::portable::Paths::at(&path.parent().unwrap().join(format!("restore-{}", new_id())));
    assert!(crate::restore::verify_and_stage(&target, path, password, None).is_err());
}

#[test]
fn offline_activity_erasure_truncation_reorder_replacement_and_middle_delete_fail() {
    for mode in 0..7 {
        let (mut s, path, password) = fixture();
        for _ in 0..4 {
            s.record_ai_reads(&ai(), 1).unwrap();
        }
        let rows = s.store.ids(Table::Activity).unwrap();
        let entries: Vec<_> = rows.into_iter().filter(|id| *id != [0; 16]).collect();
        let old_head = s.store.get(Table::Activity, &[0; 16]).unwrap().unwrap();
        s.record_ai_reads(&ai(), 2).unwrap();
        let tail = s
            .store
            .ids(Table::Activity)
            .unwrap()
            .into_iter()
            .find(|id| *id != [0; 16] && !entries.contains(id))
            .unwrap();
        let records = s.store.all(Table::Records).unwrap();
        drop(s);
        let conn = Connection::open(&path).unwrap();
        match mode {
            0 => {
                conn.execute_batch("DELETE FROM activity").unwrap();
            }
            1 => {
                conn.execute_batch("DROP TABLE activity").unwrap();
            }
            2 => {
                conn.execute("DELETE FROM activity WHERE id=?1", params![&tail[..]])
                    .unwrap();
            }
            3 => {
                conn.execute("DELETE FROM activity WHERE id=?1", params![&entries[1][..]])
                    .unwrap();
            }
            4 => {
                let a: Vec<u8> = conn
                    .query_row(
                        "SELECT data FROM activity WHERE id=?1",
                        params![&entries[0][..]],
                        |r| r.get(0),
                    )
                    .unwrap();
                let b: Vec<u8> = conn
                    .query_row(
                        "SELECT data FROM activity WHERE id=?1",
                        params![&entries[1][..]],
                        |r| r.get(0),
                    )
                    .unwrap();
                conn.execute(
                    "UPDATE activity SET data=?2 WHERE id=?1",
                    params![&entries[0][..], b],
                )
                .unwrap();
                conn.execute(
                    "UPDATE activity SET data=?2 WHERE id=?1",
                    params![&entries[1][..], a],
                )
                .unwrap();
            }
            5 => {
                conn.execute(
                    "UPDATE activity SET data=x'01' WHERE id=?1",
                    params![&entries[1][..]],
                )
                .unwrap();
            }
            _ => {
                conn.execute("DELETE FROM activity WHERE id=?1", params![&tail[..]])
                    .unwrap();
                conn.execute(
                    "UPDATE activity SET data=?1 WHERE id=zeroblob(16)",
                    params![old_head],
                )
                .unwrap();
            }
        }
        drop(conn);
        match Session::unlock(&path, &password, None) {
            Ok(s) => {
                assert!(s.activity_verify().is_err());
                assert!(s.verify_attachments().is_err());
            }
            Err(_) => {}
        }
        assert_eq!(
            Store::open(&path).unwrap().all(Table::Records).unwrap(),
            records
        );
        rejected_by_restore(&path, &password);
    }
}

#[test]
fn offline_history_pair_page_table_half_deletion_and_swap_fail() {
    for mode in 0..7 {
        let (mut s, path, password) = fixture();
        let item = page("before");
        let id = item.id().to_string();
        s.put(item).unwrap();
        edit(&mut s, &id);
        let versions = s.history_list(&id).unwrap();
        let a = parse_id(&versions[0].version_id).unwrap();
        let b = history::body_id(&versions[0].version_id).unwrap();
        let other = parse_id(&versions[1].version_id).unwrap();
        let records = s.store.all(Table::Records).unwrap();
        drop(s);
        let conn = Connection::open(&path).unwrap();
        match mode {
            0 => {
                conn.execute(
                    "DELETE FROM history WHERE id=?1 OR id=?2",
                    params![&a[..], &b[..]],
                )
                .unwrap();
            }
            1 => {
                for m in &versions {
                    conn.execute(
                        "DELETE FROM history WHERE id=?1 OR id=?2",
                        params![
                            &parse_id(&m.version_id).unwrap()[..],
                            &history::body_id(&m.version_id).unwrap()[..]
                        ],
                    )
                    .unwrap();
                }
            }
            2 => {
                conn.execute_batch("DROP TABLE history").unwrap();
            }
            3 => {
                conn.execute("DELETE FROM history WHERE id=?1", params![&a[..]])
                    .unwrap();
            }
            4 => {
                conn.execute("DELETE FROM history WHERE id=?1", params![&b[..]])
                    .unwrap();
            }
            5 => {
                for (one, two) in [
                    (a, other),
                    (b, history::body_id(&versions[1].version_id).unwrap()),
                ] {
                    let x: Vec<u8> = conn
                        .query_row(
                            "SELECT data FROM history WHERE id=?1",
                            params![&one[..]],
                            |r| r.get(0),
                        )
                        .unwrap();
                    let y: Vec<u8> = conn
                        .query_row(
                            "SELECT data FROM history WHERE id=?1",
                            params![&two[..]],
                            |r| r.get(0),
                        )
                        .unwrap();
                    conn.execute(
                        "UPDATE history SET data=?2 WHERE id=?1",
                        params![&one[..], y],
                    )
                    .unwrap();
                    conn.execute(
                        "UPDATE history SET data=?2 WHERE id=?1",
                        params![&two[..], x],
                    )
                    .unwrap();
                }
            }
            _ => {
                conn.execute_batch("DELETE FROM history").unwrap();
            }
        }
        drop(conn);
        assert!(Session::unlock(&path, &password, None).is_err());
        assert_eq!(
            Store::open(&path).unwrap().all(Table::Records).unwrap(),
            records
        );
        rejected_by_restore(&path, &password);
    }
}

#[test]
fn anchorless_legacy_migrates_but_deleted_missing_table_or_mismatched_anchor_fails() {
    for mode in 0..6 {
        let (mut s, path, password) = fixture();
        if mode == 0 || mode == 5 {
            if mode == 0 {
                let item = page("legacy history");
                let id = item.id().to_string();
                s.put(item).unwrap();
                let mut item = s.get(&id).unwrap().clone();
                if let Item::Page(p) = &mut item {
                    p.text = "after".into();
                }
                s.put(item).unwrap();
                s.snapshot_page(&id).unwrap();
            }
            let mut settings = s.settings();
            settings.journals_anchored = false;
            let item = Item::Settings(settings);
            let (id, bytes) = s.seal_item(&item).unwrap();
            s.store.put(Table::Records, &id, &bytes).unwrap();
        }
        if mode == 0 {
            Connection::open(&path)
                .unwrap()
                .execute_batch("DELETE FROM journal_anchor")
                .unwrap();
        } else if mode == 5 {
            Connection::open(&path)
                .unwrap()
                .execute_batch(
                    "DROP TABLE activity; DROP TABLE history; DROP TABLE journal_anchor;",
                )
                .unwrap();
        } else if mode == 3 {
            let mut wrong = s.anchor.clone();
            wrong.activity_rows += 1;
            s.store
                .put(
                    Table::Anchor,
                    &anchor::ID,
                    &s.seal_json(Table::Anchor, &anchor::ID, &wrong).unwrap(),
                )
                .unwrap();
        } else if mode == 2 {
            Connection::open(&path)
                .unwrap()
                .execute_batch("DROP TABLE journal_anchor")
                .unwrap();
        } else if mode == 4 {
            s.store.put(Table::Anchor, &anchor::ID, b"damaged").unwrap();
        } else {
            s.store.delete(Table::Anchor, &anchor::ID).unwrap();
        }
        drop(s);
        if mode == 0 || mode == 5 {
            let mut s = Session::unlock(&path, &password, None).unwrap();
            assert!(s.settings().journals_anchored);
            s.activity_verify().unwrap();
            s.clear_activity(&Actor::Person, "CLEAR").unwrap();
            s.activity_verify().unwrap();
            drop(s);
            Session::unlock(&path, &password, None)
                .unwrap()
                .activity_verify()
                .unwrap();
        } else {
            assert!(Session::unlock(&path, &password, None).is_err());
            rejected_by_restore(&path, &password);
        }
    }
}

#[test]
fn failed_anchor_write_rolls_back_data_journals_and_memory() {
    let (mut s, path, password) = fixture();
    let item = page("before");
    let id = item.id().to_string();
    s.put(item).unwrap();
    let before: Vec<_> = [
        Table::Records,
        Table::History,
        Table::Activity,
        Table::Anchor,
    ]
    .into_iter()
    .map(|t| s.store.all(t).unwrap())
    .collect();
    let rev = s.rev(&id);
    Connection::open(&path).unwrap().execute_batch("CREATE TRIGGER fail_anchor BEFORE INSERT ON journal_anchor WHEN NEW.id=zeroblob(16) BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    let mut item = s.get(&id).unwrap().clone();
    if let Item::Page(p) = &mut item {
        p.text = "aborted".into();
    }
    assert!(s.run_as(&ai(), |s| s.put(item)).is_err());
    assert_eq!(s.rev(&id), rev);
    for (table, rows) in [
        Table::Records,
        Table::History,
        Table::Activity,
        Table::Anchor,
    ]
    .into_iter()
    .zip(before)
    {
        assert_eq!(s.store.all(table).unwrap(), rows);
    }
    // The cached anchor still matches disk.
    s.activity_verify().unwrap();
    drop(s);
    Session::unlock(&path, &password, None)
        .unwrap()
        .activity_verify()
        .unwrap();
}

#[test]
fn permanent_delete_and_empty_bin_purge_all_item_content_and_refuse_undo() {
    for empty in [false, true] {
        for kind in 0..4 {
            let (mut s, path, password) = fixture();
            let marker = Zeroizing::new(format!("purge{}", new_id()));
            let item = match kind {
            0 => page(&marker),
            1 => serde_json::from_value(serde_json::json!({"kind":"sticky","id":new_id(),"content":&*marker,"text":&*marker,"color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap(),
            2 => {
                let db = Database::new(new_id(), String::new(), "DB".into(), 1); let db_id = db.id.clone(); s.put(Item::Database(db)).unwrap();
                let mut row = Row::new(new_id(), &db_id, "Row".into(), 1);
                row.content = marker.to_string(); row.text = marker.to_string(); row.cells = serde_json::json!({"cell":&*marker}).to_string(); Item::Row(row)
            }
            _ => {
                let mut db = Database::new(new_id(), String::new(), "DB".into(), 1);
                db.templates.push(Template { id: new_id(), name: "Template".into(), content: marker.to_string(), cells: serde_json::json!({"cell":&*marker}).to_string(), ..Default::default() }); Item::Database(db)
            }
        };
            let id = item.id().to_string();
            s.put(item).unwrap();
            let set = edit(&mut s, &id);
            if kind == 1 {
                crate::commands::stickies::delete_sticky_core(&mut s, &Actor::Person, id.clone())
                    .unwrap();
            } else {
                crate::commands::notes::trash_page_core(&mut s, &Actor::Person, id.clone())
                    .unwrap();
            }
            // Ordinary binning preserves the indirect before-content.
            assert!([Table::Anchor, Table::History].into_iter().any(|table| s
                .store
                .all(table)
                .unwrap()
                .iter()
                .any(|(row, bytes)| {
                    let plain =
                        crypto::open(&s.dek, &aad(s.format, &s.vault_id, table, row), bytes)
                            .unwrap();
                    plain.windows(marker.len()).any(|w| w == marker.as_bytes())
                })));
            if empty {
                crate::commands::notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
            } else {
                crate::commands::notes::delete_page_core(&mut s, &Actor::Person, id.clone())
                    .unwrap();
            }
            assert!(s.get(&id).is_none());
            assert_absent(&s, &marker);
            assert!(s
                .activity_list(None, None, 500)
                .unwrap()
                .iter()
                .any(|e| e.items.iter().any(|i| i.id == id)));
            assert!(s
                .activity_undo(&Actor::Person, &set)
                .err()
                .unwrap()
                .to_string()
                .contains("permanently deleted"));
            s.activity_verify().unwrap();
            drop(s);
            let s = Session::unlock(&path, &password, None).unwrap();
            assert_absent(&s, &marker);
            s.verify_attachments().unwrap();
        }
    }
}

#[test]
fn template_removal_purges_old_template_bodies_but_keeps_database() {
    let (mut s, path, password) = fixture();
    let marker = Zeroizing::new(format!("template{}", new_id()));
    let file = s.put_attachment("application/octet-stream", marker.as_bytes()).unwrap();
    // A separate live reference must survive the same cleanup.
    let kept_file = s.put_attachment("image/png", b"kept picture").unwrap();
    s.put(page(&format!("clatt://localhost/{kept_file}"))).unwrap();
    let mut db = Database::new(new_id(), String::new(), "DB".into(), 1);
    db.templates.push(Template {
        id: new_id(),
        name: "Template".into(),
        content: marker.to_string(),
        cells: serde_json::json!({"cell":&*marker,"file":file}).to_string(),
        ..Default::default()
    });
    let id = db.id.clone();
    s.put(Item::Database(db)).unwrap();
    let mut settings = s.settings();
    settings.replaced_edits.push(crate::model::ReplacedEdit {
        id: id.clone(),
        rev: 1,
        by: "ai:Test AI".into(),
        status: "replaced".into(),
        previous: serde_json::to_string(s.get(&id).unwrap()).unwrap(),
    });
    s.put(Item::Settings(settings)).unwrap();
    let set = edit(&mut s, &id);
    let Item::Database(mut db) = s.get(&id).unwrap().clone() else {
        unreachable!()
    };
    db.templates.clear();
    crate::commands::database::save_database_core(
        &mut s,
        &Actor::Person,
        serde_json::to_string(&db).unwrap(),
    )
    .unwrap();
    assert_absent(&s, &marker);
    assert!(s.get_attachment(&file).is_err());
    assert!(s.get_attachment(&kept_file).is_ok());
    assert!(s.get(&id).is_some());
    assert!(s.activity_undo(&Actor::Person, &set).is_err());
    s.activity_verify().unwrap();
    drop(s);
    let s = Session::unlock(&path, &password, None).unwrap();
    assert_absent(&s, &marker);
}

#[test]
fn thinning_moves_membership_and_records_an_authenticated_act() {
    let (mut s, path, password) = fixture();
    let item = page("before");
    let id = item.id().to_string();
    s.put(item).unwrap();
    for _ in 0..13 {
        edit(&mut s, &id);
    }
    assert!(s.thin_history(now_ms() + 366 * history::DAY).unwrap() > 0);
    assert_eq!(s.history_list(&id).unwrap().len(), 20);
    assert!(s
        .activity_list(None, None, 100)
        .unwrap()
        .iter()
        .any(|e| e.action == "history.thinned"));
    s.activity_verify().unwrap();
    drop(s);
    let mut s = Session::unlock(&path, &password, None).unwrap();
    s.verify_attachments().unwrap();
    s.clear_activity(&Actor::Person, "CLEAR").unwrap();
    s.activity_verify().unwrap();
}

#[test]
fn failed_purge_rolls_back_deleted_records_and_relinked_activity() {
    for fail_final_anchor in [false, true] {
        let (mut s, path, password) = fixture();
        let marker = Zeroizing::new(new_id());
        let item: Item=serde_json::from_value(serde_json::json!({"kind":"sticky","id":new_id(),"content":&*marker,"text":&*marker,"color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap();
        let id = item.id().to_string();
        s.put(item).unwrap();
        edit(&mut s, &id);
        crate::commands::stickies::delete_sticky_core(&mut s, &Actor::Person, id.clone()).unwrap();
        let before: Vec<_> = [
            Table::Records,
            Table::History,
            Table::Activity,
            Table::Anchor,
        ]
        .into_iter()
        .map(|t| s.store.all(t).unwrap())
        .collect();
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(if fail_final_anchor {
            // The deletion writes its journal/anchor, purges undo, re-links the
            // chain and then refreshes the final anchor in this SAME transaction.
            "CREATE TABLE anchor_writes(n INTEGER); INSERT INTO anchor_writes VALUES(0);
         CREATE TRIGGER fail_refresh BEFORE UPDATE ON journal_anchor WHEN NEW.id=zeroblob(16)
         BEGIN UPDATE anchor_writes SET n=n+1;
         SELECT CASE WHEN (SELECT n FROM anchor_writes)=2 THEN RAISE(ABORT,'fixture') END; END;"
        } else {
            "CREATE TRIGGER fail_purge BEFORE DELETE ON journal_anchor WHEN substr(OLD.id,1,1)=x'03'
         BEGIN SELECT RAISE(ABORT,'fixture'); END;"
        })
        .unwrap();
        drop(conn);
        assert!(
            crate::commands::notes::delete_page_core(&mut s, &Actor::Person, id.clone()).is_err()
        );
        assert!(s.get(&id).is_some());
        for (table, rows) in [
            Table::Records,
            Table::History,
            Table::Activity,
            Table::Anchor,
        ]
        .into_iter()
        .zip(before)
        {
            assert_eq!(s.store.all(table).unwrap(), rows);
        }
        s.activity_verify().unwrap();
        drop(s);
        Session::unlock(&path, &password, None)
            .unwrap()
            .activity_verify()
            .unwrap();
    }
}

#[test]
fn g3_style_old_reader_ignores_added_table_and_settings_latch() {
    let (s, path, password) = fixture();
    assert_eq!(s.store.read_header().unwrap().format, Format::Current);
    assert_eq!(s.store.read_header().unwrap().format.version(), 2);
    drop(s);
    // 1.5.1's read path selects only records/attachments and serde ignores
    // unknown settings fields. This checks that pattern, not the old exe.
    let store = Store::open(&path).unwrap();
    let header = store.read_header().unwrap();
    let dek = header.unlock(&password, None).unwrap();
    #[derive(serde::Deserialize)]
    struct OldSettings {
        kind: String,
        theme: String,
    }
    for (id, bytes) in store.all(Table::Records).unwrap() {
        let plain = crypto::open(
            &dek,
            &aad(header.format, &header.vault_id, Table::Records, &id),
            &bytes,
        )
        .unwrap();
        let old: OldSettings = serde_json::from_slice(crypto::unpad(&plain).unwrap()).unwrap();
        assert!(old.kind == "settings" && old.theme == "sunrise");
    }
    assert!(store.ids(Table::Attachments).unwrap().is_empty());
}

#[test]
fn ordinary_ai_sticky_binning_remains_undoable() {
    let (mut s, _, _) = fixture();
    let marker = Zeroizing::new(new_id());
    let id = new_id();
    s.put(serde_json::from_value(serde_json::json!({"kind":"sticky","id":id,"content":&*marker,"text":&*marker,"color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap()).unwrap();
    crate::commands::stickies::delete_sticky_core(&mut s, &ai(), id.clone()).unwrap();
    let set = s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0]
        .change_set
        .clone();
    assert_eq!(s.activity_undo(&ai(), &set).unwrap().restored, 1);
    assert!(!s.in_bin(&id));
    assert!(matches!(s.get(&id),Some(Item::Sticky(p)) if p.text==*marker));
    s.activity_verify().unwrap();
}

#[test]
fn missing_indirect_undo_body_fails_verification_and_undo() {
    let (mut s, _, _) = fixture();
    let item = page("before");
    let id = item.id().to_string();
    s.put(item).unwrap();
    let set = edit(&mut s, &id);
    let body = s
        .store
        .ids(Table::Anchor)
        .unwrap()
        .into_iter()
        .find(|id| id[0] == 3)
        .unwrap();
    s.store.delete(Table::Anchor, &body).unwrap();
    assert!(s.activity_verify().is_err());
    assert!(s.activity_undo(&ai(), &set).is_err());
}

#[test]
fn container_delete_and_bin_cleanup_purge_descendants_and_unreferenced_files() {
    // Exercise the real notebook/section -> database -> row -> subpage paths,
    // including files used only by a previous cell/body. No window or dialog.
    for section_only in [false, true] {
        let (mut s, path, password) = fixture();
        let marker = Zeroizing::new(format!("cascade{}", new_id()));
        let picture = s.put_attachment("image/png", marker.as_bytes()).unwrap();
        let notebook = new_id();
        let section = new_id();
        s.put(
            serde_json::from_value(serde_json::json!({
                "kind":"notebook", "id":notebook, "name":"Book", "color":"",
                "icon":"", "order":0, "created":1, "updated":1
            }))
            .unwrap(),
        )
        .unwrap();
        s.put(
            serde_json::from_value(serde_json::json!({
                "kind":"section", "id":section, "notebookId":notebook, "name":"Section",
                "color":"", "order":0, "created":1, "updated":1
            }))
            .unwrap(),
        )
        .unwrap();
        let mut db = Database::new(new_id(), section.clone(), "DB".into(), 1);
        db.templates.push(Template {
            id: new_id(),
            name: "Template".into(),
            content: marker.to_string(),
            cells: "{}".into(),
            ..Default::default()
        });
        let db_id = db.id.clone();
        s.put(Item::Database(db)).unwrap();
        let mut row = Row::new(new_id(), &db_id, "Row".into(), 1);
        row.content = format!("clatt://localhost/{picture} {}", &*marker);
        row.text = marker.to_string();
        row.cells = serde_json::json!({"cell":&*marker}).to_string();
        let row_id = row.id.clone();
        s.put(Item::Row(row)).unwrap();
        let Item::Page(mut child) = page(&marker) else {
            unreachable!()
        };
        child.parent = row_id.clone();
        child.section_id = section.clone();
        let child_id = child.id.clone();
        s.put(Item::Page(child)).unwrap();
        let sets = [
            edit(&mut s, &db_id),
            edit(&mut s, &row_id),
            edit(&mut s, &child_id),
        ];
        let mut settings = s.settings();
        settings.replaced_edits.push(crate::model::ReplacedEdit {
            id: row_id.clone(),
            rev: 1,
            by: "ai:Test AI".into(),
            status: "replaced".into(),
            previous: marker.to_string(),
        });
        s.put(Item::Settings(settings)).unwrap();
        let root = if section_only {
            section.clone()
        } else {
            notebook.clone()
        };
        let mut item = s.get(&root).unwrap().clone();
        match &mut item {
            Item::Notebook(n) => n.trashed_at = Some(now_ms()),
            Item::Section(n) => n.trashed_at = Some(now_ms()),
            _ => unreachable!(),
        }
        s.put(item).unwrap();
        assert!(s.get_attachment(&picture).is_ok(), "binning retains files");
        if section_only {
            crate::commands::notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
            assert!(s.get(&notebook).is_some());
        } else {
            crate::commands::notes::delete_page_core(&mut s, &Actor::Person, root.clone()).unwrap();
        }
        for id in [&root, &db_id, &row_id, &child_id] {
            assert!(s.get(id).is_none());
        }
        assert!(s.get_attachment(&picture).is_err());
        assert_absent(&s, &marker);
        for set in sets {
            assert!(s.activity_undo(&Actor::Person, &set).is_err());
        }
        s.activity_verify().unwrap();
        drop(s);
        let s = Session::unlock(&path, &password, None).unwrap();
        assert_absent(&s, &marker);
        s.verify_attachments().unwrap();
    }
}

#[test]
fn bulk_sticky_bin_preserves_undo_until_empty_bin_purges_it() {
    let (mut s, path, password) = fixture();
    let marker = Zeroizing::new(format!("bulk{}", new_id()));
    let mut ids = vec![];
    let mut sets = vec![];
    for _ in 0..2 {
        let id = new_id();
        s.put(
            serde_json::from_value(serde_json::json!({
                "kind":"sticky", "id":id, "content":&*marker, "text":&*marker,
                "color":"yellow", "x":0,"y":0,"w":320,"h":320,
                "onTop":false,"open":false,"created":1,"updated":1
            }))
            .unwrap(),
        )
        .unwrap();
        sets.push(edit(&mut s, &id));
        ids.push(id);
    }
    crate::commands::stickies::delete_stickies_core(&mut s, &ai(), ids.clone()).unwrap();
    let bin_set = s.activity_list(Some("ai:Test AI"), None, 1).unwrap()[0]
        .change_set
        .clone();
    assert_eq!(s.activity_undo(&ai(), &bin_set).unwrap().restored, 2);
    assert!(ids.iter().all(|id| !s.in_bin(id)));
    crate::commands::stickies::delete_stickies_core(&mut s, &Acto