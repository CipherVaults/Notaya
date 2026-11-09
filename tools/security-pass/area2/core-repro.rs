// Source-linked standalone proof. No app, window, dialog, pipe, or network.
use std::path::{Path, PathBuf};
use actions::Actor;
use model::Item;

fn fixture(root: &Path, name: &str) -> (portable::Paths, zeroize::Zeroizing<String>, String) {
    let paths = portable::Paths::at(&root.join(name));
    paths.ensure_dirs().unwrap();
    let password = zeroize::Zeroizing::new(model::new_id());
    let mut s = vault::Session::create(&paths.vault, &password, None,
        crypto::KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap();
    let id = model::new_id();
    let page: Item = serde_json::from_value(serde_json::json!({
        "kind":"page", "id":id, "sectionId":"", "title":"Throwaway proof",
        "content":"before", "text":"before", "order":0, "favorite":false,
        "trashedAt":null, "created":1, "updated":1
    })).unwrap();
    s.put(page).unwrap();
    let mut page = s.get(&id).unwrap().clone();
    if let Item::Page(p) = &mut page { p.content = "after".into(); p.text = "after".into(); }
    s.put(page).unwrap();
    s.snapshot_page(&id).unwrap();
    s.activity_verify().unwrap();
    s.verify_attachments().unwrap();
    assert!(s.history_list(&id).unwrap().len() >= 2);
    (paths, password, id)
}

fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("explicit throwaway root"));
    assert!(root.starts_with(PathBuf::from(r"D:\Notaya Tests\v1-6-S")));
    assert!(!root.exists(), "fresh proof folder required");
    std::fs::create_dir_all(&root).unwrap();
    logging::init(&portable::Paths::at(&root));

    // Attacker performs disk edits only while the session is absent. No password
    // or data key is used by any mutation below; credentials remain inside fixture.
    for sql in ["DELETE FROM activity", "DROP TABLE activity"] {
        let name = if sql.starts_with("DELETE") { "activity-delete-all" } else { "activity-drop-table" };
        let (paths, password, id) = fixture(&root, name);
        let records_before = vault::store::Store::open(&paths.vault).unwrap().all(vault::store::Table::Records).unwrap();
        rusqlite::Connection::open(&paths.vault).unwrap().execute_batch(sql).unwrap();
        let mut s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
        assert!(matches!(s.get(&id), Some(Item::Page(p)) if p.text == "after"));
        assert!(s.activity_list(None, None, 500).unwrap().is_empty());
        s.activity_verify().expect("bug: erased activity passes integrity check");
        s.verify_attachments().expect("bug: full restore verifier also accepts erased activity");
        assert_eq!(records_before, vault::store::Store::open(&paths.vault).unwrap().all(vault::store::Table::Records).unwrap());
        s.record_ai_reads(&Actor::Ai { pairing_id:model::new_id(), label:"Proof".into() }, 1).unwrap();
        s.activity_verify().unwrap();
        assert_eq!(s.activity_list(None, None, 500).unwrap().len(), 1);
        println!("REPRODUCED: {name}; unchanged records, empty log verified, new chain accepted");
        drop(s);
        let target = portable::Paths::at(&root.join(format!("{name}-restore")));
        restore::verify_and_stage(&target, &paths.vault, &password, None).unwrap();
        println!("REPRODUCED: {name}; restore staging accepted the erased prior log");
    }

    let (paths, password, id) = fixture(&root, "history-remove-pair");
    let s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    let versions = s.history_list(&id).unwrap();
    let victim = versions.iter().find(|v| s.history_get(&id, &v.version_id).unwrap().text == "before").unwrap().version_id.clone();
    let count = versions.len();
    let mut meta_id = vault::parse_id(&victim).unwrap();
    let body_id = { let mut b = meta_id; b[0] = 1; b };
    assert_eq!(meta_id[0], 0);
    drop(s);
    let conn = rusqlite::Connection::open(&paths.vault).unwrap();
    conn.execute("DELETE FROM history WHERE id=?1 OR id=?2", rusqlite::params![&meta_id[..], &body_id[..]]).unwrap();
    drop(conn);
    let s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    assert_eq!(s.history_list(&id).unwrap().len(), count - 1);
    assert!(matches!(s.history_get(&id, &victim), Err(error::Error::NotFound)));
    s.activity_verify().unwrap();
    s.verify_attachments().expect("bug: deleted snapshot pair passes journal verification");
    println!("REPRODUCED: deleting one metadata/body pair hides a history version; full verifier accepts");
    drop(s);
    restore::verify_and_stage(&portable::Paths::at(&root.join("history-restore")), &paths.vault, &password, None).unwrap();
    println!("REPRODUCED: restore staging accepted the missing history pair");

    // Controls: isolated missing activity entry and missing history body fail.
    let (paths, password, _) = fixture(&root, "activity-control");
    rusqlite::Connection::open(&paths.vault).unwrap().execute_batch(
        "DELETE FROM activity WHERE id=(SELECT id FROM activity WHERE id != zeroblob(16) LIMIT 1)").unwrap();
    let s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    assert!(s.activity_verify().is_err());
    println!("CONTROL: deleting an individual activity row with head retained is detected");
    drop(s);
    let (paths, password, id) = fixture(&root, "history-control");
    let s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    meta_id = vault::parse_id(&s.history_list(&id).unwrap()[0].version_id).unwrap();
    meta_id[0] = 1;
    drop(s);
    rusqlite::Connection::open(&paths.vault).unwrap().execute("DELETE FROM history WHERE id=?1", rusqlite::params![&meta_id[..]]).unwrap();
    let s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    assert!(s.verify_attachments().is_err());
    println!("CONTROL: deleting a history body with metadata retained is detected");
    drop(s);

    // Permanently deleted non-page text must not survive in current undo rows.
    let (paths, password, _) = fixture(&root, "sticky-purge");
    let mut s = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    let sticky_id = model::new_id();
    let marker = zeroize::Zeroizing::new(format!("purgeprobe{}", model::new_id()));
    s.put(serde_json::from_value(serde_json::json!({"kind":"sticky", "id":sticky_id,
        "content":&*marker, "text":&*marker, "color":"yellow", "x":0,"y":0,
        "w":320,"h":320,"onTop":false,"open":false,"created":1,"updated":1})).unwrap()).unwrap();
    let mut edited = s.get(&sticky_id).unwrap().clone();
    if let Item::Sticky(p) = &mut edited { p.content = "replacement".into(); p.text = p.content.clone(); }
    s.run_as(&Actor::Ai { pairing_id:model::new_id(), label:"Proof".into() }, |s| s.put(edited)).unwrap();
    commands::stickies::delete_sticky_core(&mut s, &Actor::Person, sticky_id.clone()).unwrap();
    commands::notes::delete_page_core(&mut s, &Actor::Person, sticky_id.clone()).unwrap();
    assert!(s.get(&sticky_id).is_none());
    s.activity_verify().unwrap();
    drop(s);
    let store = vault::store::Store::open(&paths.vault).unwrap();
    let header = store.read_header().unwrap();
    let dek = header.unlock(&password, None).unwrap();
    let mut retained = false;
    for (row, sealed) in store.all(vault::store::Table::Activity).unwrap() {
        // The production format's public AAD layout, no keys/data printed.
        let mut aad = header.format.row_binding().to_vec();
        aad.extend_from_slice(&header.format.version().to_le_bytes());
        aad.extend_from_slice(&header.vault_id);
        aad.extend_from_slice(vault::store::Table::Activity.tag());
        aad.extend_from_slice(&row);
        let plain = crypto::open(&dek, &aad, &sealed).unwrap();
        retained |= plain.windows(marker.len()).any(|w| w == marker.as_bytes());
    }
    assert!(retained, "bug: permanently deleted sticky text remains in activity undo");
    println!("REPRODUCED: permanently deleted sticky body remains decryptable in current activity undo");

    let key = crypto::random_key();
    let a = crypto::seal(&key, b"record-one", b"throwaway");
    let b = crypto::seal(&key, b"record-one", b"throwaway");
    assert_ne!(&a[..24], &b[..24]);
    assert!(crypto::open(&key, b"record-two", &a).is_err());
    assert!(crypto::open(&crypto::random_key(), b"record-one", &a).is_err());
    assert!(crypto::open(&key, b"record-one", &a[..20]).is_err());
    println!("CONTROL: fresh nonces; wrong AAD/key and truncated ciphertext rejected");
    println!("Evidence preserved at {}", root.display());
}
