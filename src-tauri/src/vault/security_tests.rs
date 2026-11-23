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
                crate::commands::notes::test_empty_trash_core(&mut s, &Actor::Person).unwrap();
            } else {
                crate::commands::notes::test_delete_page_core(&mut s, &Actor::Person, id.clone())
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
    let file = s
        .put_attachment("application/octet-stream", marker.as_bytes())
        .unwrap();
    // A separate live reference must survive the same cleanup.
    let kept_file = s.put_attachment("image/png", b"kept picture").unwrap();
    s.put(page(&format!("clatt://localhost/{kept_file}")))
        .unwrap();
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
    for failure in 0..3 {
        let (mut s, path, password) = fixture();
        let marker = Zeroizing::new(new_id());
        let file = s.put_attachment("image/png", marker.as_bytes()).unwrap();
        let item: Item=serde_json::from_value(serde_json::json!({"kind":"sticky","id":new_id(),"content":format!("clatt://localhost/{file} {}", &*marker),"text":&*marker,"color":"yellow","x":0,"y":0,"w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap();
        let id = item.id().to_string();
        s.put(item).unwrap();
        edit(&mut s, &id);
        crate::commands::stickies::delete_sticky_core(&mut s, &Actor::Person, id.clone()).unwrap();
        let before: Vec<_> = [
            Table::Records,
            Table::History,
            Table::Activity,
            Table::Anchor,
            Table::Attachments,
        ]
        .into_iter()
        .map(|t| s.store.all(t).unwrap())
        .collect();
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(if failure == 1 {
            // The deletion writes its journal/anchor, purges undo, re-links the
            // chain and then refreshes the final anchor in this SAME transaction.
            "CREATE TABLE anchor_writes(n INTEGER); INSERT INTO anchor_writes VALUES(0);
         CREATE TRIGGER fail_refresh BEFORE UPDATE ON journal_anchor WHEN NEW.id=zeroblob(16)
         BEGIN UPDATE anchor_writes SET n=n+1;
         SELECT CASE WHEN (SELECT n FROM anchor_writes)=2 THEN RAISE(ABORT,'fixture') END; END;"
        } else if failure == 0 {
            "CREATE TRIGGER fail_purge BEFORE DELETE ON journal_anchor WHEN substr(OLD.id,1,1)=x'03'
         BEGIN SELECT RAISE(ABORT,'fixture'); END;"
        } else {
            "CREATE TRIGGER fail_file_purge BEFORE DELETE ON attachments BEGIN SELECT RAISE(ABORT,'fixture'); END;"
        })
        .unwrap();
        drop(conn);
        let mut approval = crate::native_confirm::test_delete(&s, crate::native_confirm::Delete::One(id.clone()));
        let err = crate::commands::notes::delete_page_core(&mut s, &Actor::Person, id.clone(), &mut approval)
            .err()
            .expect("the purge must fail");
        assert!(
            !format!("{err:?}").contains(crate::native_confirm::CANCELLED),
            "the purge must be what fails, not the approval: {err:?}"
        );
        assert!(s.get(&id).is_some());
        for (table, rows) in [
            Table::Records,
            Table::History,
            Table::Activity,
            Table::Anchor,
            Table::Attachments,
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
            crate::commands::notes::test_empty_trash_core(&mut s, &Actor::Person).unwrap();
            assert!(s.get(&notebook).is_some());
        } else {
            crate::commands::notes::test_delete_page_core(&mut s, &Actor::Person, root.clone()).unwrap();
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
    crate::commands::stickies::delete_stickies_core(&mut s, &Actor::Person, ids.clone()).unwrap();
    crate::commands::notes::test_empty_trash_core(&mut s, &Actor::Person).unwrap();
    assert_absent(&s, &marker);
    for set in sets {
        assert!(s.activity_undo(&ai(), &set).is_err());
    }
    s.activity_verify().unwrap();
    drop(s);
    let s = Session::unlock(&path, &password, None).unwrap();
    assert_absent(&s, &marker);
    s.verify_attachments().unwrap();
}

#[test]
fn template_file_cleanup_failure_rolls_back_the_entire_purge() {
    let (mut s, path, _) = fixture();
    let marker = Zeroizing::new(new_id());
    let file = s
        .put_attachment("application/octet-stream", marker.as_bytes())
        .unwrap();
    let mut db = Database::new(new_id(), String::new(), "DB".into(), 1);
    db.templates.push(Template {
        id: new_id(),
        name: "Template".into(),
        content: marker.to_string(),
        cells: serde_json::json!({"file":file}).to_string(),
        ..Default::default()
    });
    let id = db.id.clone();
    s.put(Item::Database(db)).unwrap();
    edit(&mut s, &id);
    let before: Vec<_> = [
        Table::Records,
        Table::History,
        Table::Activity,
        Table::Anchor,
        Table::Attachments,
    ]
    .into_iter()
    .map(|table| s.store.all(table).unwrap())
    .collect();
    Connection::open(&path).unwrap().execute_batch(
        "CREATE TRIGGER fail_file_purge BEFORE DELETE ON attachments BEGIN SELECT RAISE(ABORT,'fixture'); END;"
    ).unwrap();
    let Item::Database(mut db) = s.get(&id).unwrap().clone() else {
        unreachable!()
    };
    db.templates.clear();
    assert!(crate::commands::database::save_database_core(
        &mut s,
        &Actor::Person,
        serde_json::to_string(&db).unwrap(),
    )
    .is_err());
    for (table, rows) in [
        Table::Records,
        Table::History,
        Table::Activity,
        Table::Anchor,
        Table::Attachments,
    ]
    .into_iter()
    .zip(before)
    {
        assert_eq!(s.store.all(table).unwrap(), rows);
    }
    assert!(matches!(s.get(&id), Some(Item::Database(db)) if db.templates.len() == 1));
    s.activity_verify().unwrap();
}

#[test]
fn template_purge_in_a_mixed_save_keeps_new_history_and_undo_files() {
    let (mut s, _, _) = fixture();
    let page_file = s
        .put_attachment("image/png", b"page before picture")
        .unwrap();
    let row_file = s
        .put_attachment("application/octet-stream", b"row before file")
        .unwrap();
    let mut db = Database::new(new_id(), String::new(), "DB".into(), 1);
    db.templates.push(Template {
        id: new_id(),
        name: "Template".into(),
        ..Default::default()
    });
    let db_id = db.id.clone();
    s.put(Item::Database(db)).unwrap();
    let Item::Page(mut p) = page("original") else {
        unreachable!()
    };
    let page_id = p.id.clone();
    s.put(Item::Page(p.clone())).unwrap();
    // These person edits are newer than the existing history snapshots.
    p.content = format!("clatt://localhost/{page_file}");
    p.text.clear();
    s.put(Item::Page(p.clone())).unwrap();
    let mut r = Row::new(new_id(), &db_id, "Row".into(), 1);
    s.put(Item::Row(r.clone())).unwrap();
    r.cells = serde_json::json!({"file":row_file}).to_string();
    s.put(Item::Row(r.clone())).unwrap();
    let Item::Database(mut db) = s.get(&db_id).unwrap().clone() else {
        unreachable!()
    };
    db.templates.clear();
    p.content = "after".into();
    r.cells = "{}".into();
    s.run_as(&ai(), |s| {
        s.put_many(vec![Item::Database(db), Item::Page(p), Item::Row(r)])
    })
    .unwrap();
    // The page picture is now only in this transaction's new history body;
    // the row file is only in its new indirect undo body. Both must survive.
    assert!(s.get_attachment(&page_file).is_ok());
    assert!(s.get_attachment(&row_file).is_ok());
    assert!(s
        .history_list(&page_id)
        .unwrap()
        .iter()
        .any(|meta| meta.before_ai
            && s.history_get(&page_id, &meta.version_id)
                .unwrap()
                .content
                .contains(&page_file)));
    assert_eq!(s.collect_garbage().unwrap(), 0);
    s.activity_verify().unwrap();
}


#[test]
fn purge_rolls_back_relation_cleanup_and_history_if_attachment_deletion_fails() {
    use crate::database::types::{PropKind, Property};
    let (mut s, path, password) = fixture();
    let doomed_id = new_id();
    let live_id = new_id();
    let live_row_id = new_id();
    let file = s.put_file("application/octet-stream", b"rollback fixture").unwrap();
    let mut doomed = Database::new(doomed_id.clone(), String::new(), "Doomed".into(), 1);
    doomed.trashed_at = Some(1);
    let doomed_row_id = new_id();
    let mut doomed_row = Row::new(doomed_row_id.clone(), &doomed_id, "Doomed row".into(), 1);
    doomed_row.cells = serde_json::json!({"file": [{"id": file}]}).to_string();
    let mut live = Database::new(live_id.clone(), String::new(), "Alive".into(), 1);
    live.props.push(Property {
        id: "rel".into(),
        name: "Related".into(),
        kind: PropKind::Relation,
        relation_db: doomed_id.clone(),
        ..Property::default()
    });
    let mut live_row = Row::new(live_row_id.clone(), &live_id, "Alive row".into(), 1);
    live_row.cells = serde_json::json!({"rel": [&doomed_row_id]}).to_string();
    s.put_many(vec![Item::Database(doomed), Item::Row(doomed_row), Item::Database(live), Item::Row(live_row)]).unwrap();
    s.drain_changes();

    let trigger = Connection::open(&path).unwrap();
    trigger.execute_batch(
        "CREATE TRIGGER fail_purge BEFORE DELETE ON attachments
         BEGIN SELECT RAISE(ABORT, 'synthetic attachment failure'); END;",
    ).unwrap();
    let tables = [Table::Records, Table::History, Table::Activity, Table::Anchor, Table::Attachments];
    let before = tables.map(|table| s.store.all(table).unwrap());
    assert!(crate::commands::notes::test_delete_page_core(&mut s, &Actor::Person, doomed_id.clone()).is_err());
    assert!(s.drain_changes().is_none());
    assert_eq!(tables.map(|table| s.store.all(table).unwrap()), before);
    assert!(s.history_list(&doomed_row_id).is_ok());
    assert!(s.get_attachment(&file).is_ok());
    assert!(matches!(s.get(&live_id), Some(Item::Database(db)) if db.prop("rel").is_some()));
    assert!(matches!(s.get(&live_row_id), Some(Item::Row(row)) if row.cells.contains(&doomed_row_id)));

    trigger.execute_batch("DROP TRIGGER fail_purge").unwrap();
    drop(trigger);
    crate::commands::notes::test_delete_page_core(&mut s, &Actor::Person, doomed_id).unwrap();
    drop(s);
    let s = Session::unlock(&path, &password, None).unwrap();
    assert!(s.get(&doomed_row_id).is_none());
    assert!(s.get_attachment(&file).is_err());
    assert!(matches!(s.get(&live_id), Some(Item::Database(db)) if db.prop("rel").is_none()));
    assert!(matches!(s.get(&live_row_id), Some(Item::Row(row)) if row.cells == "{}"));
    s.activity_verify().unwrap();
}


#[test]
#[ignore = "large vault benchmark and optional native fixture"]
fn wave1_purge_stress_and_fixture() {
    use crate::{
        commands::notes,
        database::types::{PropKind, Property},
        native_confirm::{self, Delete},
        portable::Paths,
    };
    use serde_json::json;
    use std::{fs, time::Instant};

    const ROWS: usize = 6500;
    const DBS: usize = 13;
    const PAGES: usize = 70;
    const TITLE: &str = "Wave 1 stress notebook";
    const KEEP_TITLE: &str = "Wave 1 surviving notebook";
    let temp = std::env::temp_dir().canonicalize().unwrap();
    let working = temp.join(format!("purge-stress-{}", new_id()));
    fs::create_dir(&working).unwrap();
    let seed_root = match std::env::var_os("NOTAYA_WAVE1_FIXTURE") {
        Some(path) => {
            let path = PathBuf::from(path);
            let parent = path.parent().unwrap().canonicalize().unwrap();
            assert!(path.is_absolute() && parent.starts_with(&temp), "fixture must be in scratch");
            let output = parent.join(path.file_name().unwrap());
            fs::create_dir(&output).expect("fixture output must be new");
            output
        }
        None => working.join("fixture"),
    };
    let seed = Paths::at(&seed_root);
    seed.ensure_dirs().unwrap();

    let seed_started = Instant::now();
    let mut s = Session::create_open(&seed.vault).unwrap();
    let mut settings = s.settings();
    settings.theme = "black".into();
    s.put(Item::Settings(settings)).unwrap();

    let now = now_ms();
    let doomed_id = new_id();
    let doomed_section = new_id();
    let keep_id = new_id();
    let keep_section = new_id();
    let mut records: Vec<Item> = Vec::new();
    for (id, section, name, order) in [(&doomed_id, &doomed_section, TITLE, 1), (&keep_id, &keep_section, KEEP_TITLE, 0)] {
        records.push(serde_json::from_value(json!({
            "kind": "notebook", "id": id, "name": name, "color": "#888888",
            "icon": "", "order": order, "created": now, "updated": now,
        })).unwrap());
        records.push(serde_json::from_value(json!({
            "kind": "section", "id": section, "notebookId": id, "name": "Fixture",
            "color": "#888888", "order": 0, "created": now, "updated": now,
        })).unwrap());
    }
    let keep_page_id = new_id();
    let make_page = |id: &str, section: &str, index: usize, parent: &str| {
        let title = format!("Synthetic page {index}");
        json!({
            "kind": "page", "id": id, "sectionId": section, "parent": parent,
            "title": title, "text": title, "order": index, "favorite": false,
            "content": json!({"type": "doc", "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": title}]}
            ]}).to_string(),
            "created": now, "updated": now,
        })
    };
    records.push(serde_json::from_value(make_page(&keep_page_id, &keep_section, 0, "")).unwrap());
    s.put_many(records).unwrap();
    let shared = s.put_file("application/octet-stream", b"shared fixture").unwrap();
    let exclusive = s.put_attachment("image/png", b"exclusive fixture").unwrap();
    let history_only = s.put_attachment("image/png", b"history fixture").unwrap();
    let mut survivor_db = Database::new(new_id(), keep_section.clone(), "Survivor table".into(), now);
    let survivor_db_id = survivor_db.id.clone();
    let survivor_row_id = new_id();
    let mut survivor_row = Row::new(survivor_row_id.clone(), &survivor_db_id, "Surviving row".into(), now);
    let mut doomed_files = vec![exclusive.clone(), history_only.clone()];
    let mut nested_rows = Vec::new();

    for db_index in 0..DBS {
        let db = Database::new(new_id(), doomed_section.clone(), format!("Synthetic table {db_index}"), now);
        if db_index == 0 {
            survivor_db.props.push(Property {
                id: "doomed-relation".into(), name: "Related table".into(), kind: PropKind::Relation,
                relation_db: db.id.clone(), ..Property::default()
            });
        }
        let mut records = Vec::with_capacity(ROWS / DBS + 1);
        for i in 0..ROWS / DBS {
            let mut row = Row::new(new_id(), &db.id, format!("Synthetic row {db_index}-{i}"), now);
            row.order = i as i64;
            if i % 53 == 0 {
                row.trashed_at = Some(now);
            }
            if i % 29 == 0 {
                let file = s.put_file("application/octet-stream", b"synthetic database attachment").unwrap();
                row.cells = json!({"file": [{"id": file}, {"id": shared}]}).to_string();
                doomed_files.push(file);
            }
            if i == 1 {
                nested_rows.push(row.id.clone());
                if db_index == 0 {
                    survivor_row.cells = json!({"file": [{"id": shared}], "doomed-relation": [row.id]}).to_string();
                }
            }
            records.push(Item::Row(row));
        }
        records.push(Item::Database(db));
        s.put_many(records).unwrap();
    }
    s.put_many(vec![Item::Database(survivor_db), Item::Row(survivor_row)]).unwrap();

    let first_page_id = new_id();
    let pages = (0..PAGES).map(|i| {
        let id = if i == 0 { first_page_id.clone() } else { new_id() };
        let parent = if i % 3 == 0 { nested_rows[i % DBS].as_str() } else { "" };
        let mut page = make_page(&id, &doomed_section, i, parent);
        if i % 7 == 1 {
            page["trashedAt"] = json!(now);
        }
        if i == 0 {
            page["cover"] = json!(format!("http://clatt.localhost/{history_only}"));
        }
        if i == 1 {
            page["cover"] = json!(format!("http://clatt.localhost/{exclusive}"));
        }
        serde_json::from_value::<Item>(page).unwrap()
    }).collect();
    s.put_many(pages).unwrap();

    let mut ai_edits = Vec::new();
    for id in &nested_rows {
        let Item::Row(mut row) = s.get(id).unwrap().clone() else { unreachable!() };
        row.text = "Synthetic AI change".into();
        ai_edits.push(Item::Row(row));
    }
    let Item::Page(mut first_page) = s.get(&first_page_id).unwrap().clone() else { unreachable!() };
    first_page.cover.clear();
    ai_edits.push(Item::Page(first_page));
    s.run_as(&ai(), |s| s.put_many(ai_edits)).unwrap();
    s.activity_verify().unwrap();

    let doomed = s.subtree_ids(&[doomed_id.clone()]);
    let total = ROWS + DBS + PAGES + 2;
    assert_eq!(doomed.len(), total);
    let counts = |s: &Session| json!({
        "records": s.store.ids(Table::Records).unwrap().len(),
        "history": s.store.ids(Table::History).unwrap().len(),
        "activity": s.store.ids(Table::Activity).unwrap().len(),
        "attachments": s.store.ids(Table::Attachments).unwrap().len(),
        "undoBodies": s.store.ids(Table::Anchor).unwrap().iter().filter(|row| row[0] == 3).count(),
    });
    let before = counts(&s);
    let expected_after = json!({
        "records": 6, "history": 4, "activity": before["activity"].as_u64().unwrap() + 2,
        "attachments": 1, "undoBodies": 0,
    });
    fs::write(seed_root.join("notaya-wave1-fixture.json"), serde_json::to_vec_pretty(&json!({
        "notebookId": doomed_id, "notebookTitle": TITLE, "totalItems": total,
        "survivorNotebookId": keep_id, "survivorNotebookTitle": KEEP_TITLE,
        "survivorPageId": keep_page_id, "survivingAttachmentId": shared,
        "before": before, "expectedAfter": expected_after,
    })).unwrap()).unwrap();
    drop(s);
    println!("Wave 1 seed: {total} doomed items, {PAGES} pages, {DBS} databases, {ROWS} rows, {} ms; persisted {before:?}",
        seed_started.elapsed().as_secs_f64() * 1000.0);


    for (mode, request) in [("delete-for-good", Delete::One(doomed_id.clone())), ("empty-bin", Delete::Empty)] {
        let paths = Paths::at(&working.join(mode));
        paths.ensure_dirs().unwrap();
        fs::copy(&seed.vault, &paths.vault).unwrap();
        let mut s = Session::unlock_open(&paths.vault).unwrap();
        let start = Instant::now();
        notes::delete_notebook_core(&mut s, &Actor::Person, &paths, doomed_id.clone()).unwrap();
        let trash_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert!(doomed_files.iter().all(|id| s.get_attachment(id).is_ok()));
        let start = Instant::now();
        let mut approval = native_confirm::test_delete(&s, request.clone());
        let approval_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        notes::delete_selected_core(&mut s, &Actor::Person, request, &mut approval).unwrap();
        let purge_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert!(doomed.iter().all(|id| s.get(id).is_none()));
        drop(s);

        let reopened = Session::unlock_open(&paths.vault).unwrap();
        assert!(doomed.iter().all(|id| reopened.get(id).is_none() && reopened.history_list(id).is_err()));
        assert_eq!(counts(&reopened), expected_after);
        assert!(doomed_files.iter().all(|id| reopened.get_attachment(id).is_err()));
        assert_eq!(reopened.history_list(&keep_page_id).unwrap().len(), 1);
        assert_eq!(reopened.get_attachment(&shared).unwrap().1, b"shared fixture");
        assert!(matches!(reopened.get(&survivor_db_id), Some(Item::Database(db)) if db.prop("doomed-relation").is_none()));
        assert!(matches!(reopened.get(&survivor_row_id), Some(Item::Row(row)) if !row.cells.contains("doomed-relation")));
        assert_eq!(reopened.activity_list(None, None, 1).unwrap()[0].counts["deleted"], total as u64);
        reopened.activity_verify().unwrap();
        println!("Wave 1 {mode}: trash {trash_ms:.3} ms, stage approval {approval_ms:.3} ms, purge {purge_ms:.3} ms; persistent cleanup verified");
    }
}
