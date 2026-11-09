// Standalone proof body. run-proofs.py adds path modules from the reviewed checkout.
// Not part of Notaya's build. No Tauri app/window/pipe is started.
use std::path::PathBuf;
use actions::{Actor, Confirm};
use model::Item;

fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("explicit throwaway root"));
    assert!(root.starts_with(PathBuf::from(r"D:\Notaya Tests\v1-6-S")));
    assert!(!root.exists(), "fresh data folder required; preserve previous evidence");
    std::fs::create_dir_all(&root).unwrap();
    let paths = portable::Paths::at(&root);
    paths.ensure_dirs().unwrap();
    logging::init(&paths); // pin all logging to the throwaway folder before using any core
    let locked = state::AppState::new(paths.clone());
    assert!(matches!(locked.with(|_| Ok(())), Err(error::Error::Locked)));
    println!("CONTROL: the shared AppState read gate refused a locked session");
    let password = zeroize::Zeroizing::new(model::new_id());
    let mut s = vault::Session::create(&paths.vault, &password, None, crypto::KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap();
    commands::notes::create_notebook_core(&mut s, &Actor::Person, "Area 1 throwaway".into(), String::new(), String::new()).unwrap();
    let section = s.items().find_map(|i| if let Item::Section(x) = i { Some(x.id.clone()) } else { None }).unwrap();
    let first = commands::notes::create_page_core(&mut s, &Actor::Person, section.clone(), "Delete probe".into()).unwrap().id.clone();
    let second = commands::notes::create_page_core(&mut s, &Actor::Person, section.clone(), "Empty probe".into()).unwrap().id.clone();
    let marker = format!("areaoneplaintext{}", model::new_id());
    let kept = commands::notes::create_page_core(&mut s, &Actor::Person, section, marker.clone()).unwrap().id.clone();
    let displayed = commands::notes::get_page_core(&s, &Actor::Person, kept.clone()).unwrap().title.clone();
    commands::notes::trash_page_core(&mut s, &Actor::Person, first.clone()).unwrap();
    commands::notes::delete_page_core(&mut s, &Actor::Person, first.clone()).unwrap();
    assert!(s.get(&first).is_none());
    commands::notes::trash_page_core(&mut s, &Actor::Person, second.clone()).unwrap();
    commands::notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
    assert!(s.get(&second).is_none());
    drop(s);
    let reopened = vault::Session::unlock(&paths.vault, &password, None).unwrap();
    assert!(reopened.get(&first).is_none() && reopened.get(&second).is_none());
    assert!(reopened.get(&kept).is_some());
    println!("REPRODUCED: delete_page_core and empty_trash_core permanently removed throwaway pages without confirmation; persisted after reopen");
    drop(reopened);

    // A simulated renderer listener answers the actual production broker using
    // only the values it emits. No live dialog is created or clicked.
    let broker = ai::confirm::Confirmations::default();
    let allowed = broker.request("confirm", serde_json::json!({"action":"bin.empty"}), Confirm::Strong,
        |event| {
            assert!(!broker.answer("pairing", event["id"].as_str().unwrap(), true, event["word"].as_str()));
            assert!(!broker.answer("confirm", event["id"].as_str().unwrap(), true, Some("WRONG")));
            assert!(broker.answer("confirm", event["id"].as_str().unwrap(), true, event["word"].as_str()));
            assert!(!broker.answer("confirm", event["id"].as_str().unwrap(), true, event["word"].as_str()));
        }, || false);
    assert!(allowed);
    assert!(broker.request("pairing", serde_json::json!({}), Confirm::Ask,
        |event| { assert!(broker.answer("pairing", event["id"].as_str().unwrap(), true, None)); }, || false));
    println!("REPRODUCED: action and pairing broker accepted programmatic replies from emitted fields; duplicate reply rejected");

    logging::frontend_error(&logging::FrontendReport {
        kind: "error", message: &displayed, source: "index.js", line: Some(1),
    });
    let log = std::fs::read_to_string(logging::folder(&paths).join("notaya-log.txt")).unwrap();
    assert!(log.contains(&marker));
    println!("REPRODUCED: arbitrary unquoted renderer text survived sanitization into the plaintext error log (generated marker only)");
    println!("Evidence data preserved at {}", root.display());
}
