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
    let mut s = vault::Session::create_open(&paths.vault).unwrap();
    commands::notes::create_notebook_core(&mut s, &Actor::Person, "Area 1 throwaway".into(), String::new(), String::new()).unwrap();
    let section = s.items().find_map(|i| if let Item::Section(x) = i { Some(x.id.clone()) } else { None }).unwrap();
    let first = commands::notes::create_page_core(&mut s, &Actor::Person, section.clone(), "Delete probe".into()).unwrap().id.clone();
    let second = commands::notes::create_page_core(&mut s, &Actor::Person, section, "Empty probe".into()).unwrap().id.clone();
    commands::notes::trash_page_core(&mut s, &Actor::Person, first.clone()).unwrap();
    commands::notes::delete_page_core(&mut s, &Actor::Person, first.clone()).unwrap();
    assert!(s.get(&first).is_none());
    commands::notes::trash_page_core(&mut s, &Actor::Person, second.clone()).unwrap();
    commands::notes::empty_trash_core(&mut s, &Actor::Person).unwrap();
    assert!(s.get(&second).is_none());
    drop(s);
    let reopened = vault::Session::unlock_open(&paths.vault).unwrap();
    assert!(reopened.get(&first).is_none() && reopened.get(&second).is_none());
    println!("REPRODUCED: delete_page_core and empty_trash_core permanently removed throwaway pages without confirmation; persisted after reopen");
    drop(reopened);

    // A simulated renderer listener answers the actual production broker using
    // only the values it emits. No live dialog is created or clicked.
    let broker = ai::confirm::Confirmations::default();
    let allowed = broker.request("confirm", serde_json::json!({"action":"bin.empty"}), Confirm::Strong,
        |event| {
            assert!(broker.answer("confirm", event["id"].as_str().unwrap(), true, event["word"].as_str()));
            assert!(!broker.answer("confirm", event["id"].as_str().unwrap(), true, event["word"].as_str()));
        }, || false);
    assert!(allowed);
    assert!(broker.request("pairing", serde_json::json!({}), Confirm::Ask,
        |event| { assert!(broker.answer("pairing", event["id"].as_str().unwrap(), true, None)); }, || false));
    println!("REPRODUCED: action and pairing broker accepted programmatic replies from emitted fields; duplicate reply rejected");
    println!("Evidence data preserved at {}", root.display());
}
