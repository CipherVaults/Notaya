//! Compiled action catalogue. Metadata is not permission to execute an action:
//! the AI dispatcher must apply its session, permission and confirmation checks.

mod actor;
pub(crate) mod registry;

pub use actor::Actor;

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Look,
    Edit,
    Full,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    None,
    Ask,
    AskIfBig,
    Strong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undo {
    History,
    Bin,
    Previous,
    View,
    None,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ActionUi {
    pub label: &'static str,
    pub icon: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionSpec {
    pub name: &'static str,
    pub level: Level,
    pub confirm: Confirm,
    pub undo: Undo,
    /// JSON Schema for the action-specific parameters. Dispatcher controls such
    /// as preview and expect_rev are supplied separately by the dispatcher.
    pub params: &'static str,
    pub ui: ActionUi,
    /// Existing commands whose vault logic is reused. These are provenance,
    /// not an arbitrary command-invocation route. In particular, mixed commands
    /// (settings and database look/schema) still need the dispatcher's field gates.
    pub commands: &'static [&'static str],
}

#[derive(Debug, Serialize)]
pub struct ListedAction {
    pub name: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
}

/// UI catalogue only; no schemas, permissions, pairing data or vault contents.
#[tauri::command]
pub fn list_actions() -> Vec<ListedAction> {
    registry::ACTIONS
        .iter()
        .map(|a| ListedAction {
            name: a.name,
            label: a.ui.label,
            icon: a.ui.icon,
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::crypto::KdfParams;
    use crate::database::types::{Database, PropKind, Property, Row};
    use crate::error::Result;
    use crate::model::{CalcEntry, Item, Notebook, Page, Section, Settings, Sticky};
    use crate::portable::Paths;
    use crate::state::AppState;
    use crate::vault::Session;
    use serde_json::{Value, json};
    use std::collections::{BTreeMap, HashSet};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn registry_names_policy_and_schemas_are_complete() {
        let mut names = HashSet::new();
        for a in registry::ACTIONS {
            assert!(names.insert(a.name), "duplicate {}", a.name);
            let parts: Vec<_> = a.name.split('.').collect();
            assert!(
                parts.len() >= 2
                    && parts.iter().all(|p| !p.is_empty()
                        && p.bytes()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'))
            );
            assert_ne!(a.level, Level::Never);
            // Typed, mandatory fields; serialization checks the complete policy.
            assert!(!format!("{:?} {:?} {:?}", a.level, a.confirm, a.undo).is_empty());
            assert!(!a.ui.label.is_empty() && !a.ui.icon.is_empty());
            let schema: Value = serde_json::from_str(a.params).unwrap();
            assert_eq!(schema["type"], "object", "{}", a.name);
            assert_eq!(schema["additionalProperties"], false, "{}", a.name);
            for required in schema["required"].as_array().unwrap() {
                assert!(schema["properties"].get(required.as_str().unwrap()).is_some());
            }
        }
        for (name, level, confirm) in [
            ("page.get", Level::Look, Confirm::None),
            ("page.write", Level::Edit, Confirm::AskIfBig),
            ("page.trash", Level::Edit, Confirm::Ask),
            ("bin.empty", Level::Full, Confirm::Strong),
            ("settings.set", Level::Full, Confirm::None),
        ] {
            let a = registry::ACTIONS.iter().find(|a| a.name == name).unwrap();
            assert_eq!((a.level, a.confirm), (level, confirm));
        }
    }

    #[test]
    fn no_action_references_a_never_command() {
        // WP-06g, including commands to be added by the AI-screen lane. Mixed
        // read/settings commands are permitted only through C5's filtered actions.
        let never = [
            "start_notes",
            "enable_security",
            "disable_security",
            "unlock_vault",
            "lock_vault",
            "change_master",
            "pick_keyfile",
            "create_keyfile",
            "restore_vault",
            "start_over",
            "import_portable_notes",
            "pick_backup_file",
            "backup_now",
            "set_backdrop",
            "quit_app",
            "open_data_folder",
            "open_log_folder",
            "open_url",
            "password_strength",
            "get_shell_prefs",
            "set_shell_prefs",
            "open_file",
            "save_file_as",
            "attach_file",
            "import_csv",
            "export_csv",
            "export_database",
            "export_notes",
            "import_notes",
            "import_cancel",
            "window_ready",
            "flush_done",
            "log_frontend_error",
            "log_info",
            "ai_set_enabled",
            "ai_set_level",
            "ai_remove_pairing",
            "ai_set_hidden",
            "ai_set_big_change",
            "ai_pause",
            "ai_resume",
            "ai_confirm_answer",
            "ai_pair_answer",
            "clear_activity_log",
            "lock_database",
            "unlock_database",
            "lock_page",
            "unlock_page",
            "clipboard_read",
            "clipboard_write",
            "set_auto_lock",
            "set_screenshot_protection",
        ];
        // Enumerate the actual IPC surface, not a hand-maintained list of commands.
        let lib = include_str!("../lib.rs");
        let handler = lib
            .split("tauri::generate_handler![")
            .nth(1)
            .unwrap()
            .split("])")
            .next()
            .unwrap();
        let commands: HashSet<_> = handler
            .split(',')
            .filter_map(|c| c.trim().rsplit("::").next())
            .collect();
        for a in registry::ACTIONS {
            for command in a.commands {
                assert!(commands.contains(command), "unknown command {command}");
                assert!(!never.contains(command), "{} references {command}", a.name);
            }
        }
        // Raw image data and legacy permanent notebook/sticky deletion are not
        // the AI's picture-reference and recycle-bin operations.
        for command in [
            "add_attachment",
            "delete_notebook",
            "delete_section",
            "delete_sticky",
            "delete_stickies",
        ] {
            assert!(!registry::ACTIONS.iter().any(|a| a.commands.contains(&command)));
        }
    }

    #[test]
    fn ui_catalogue_contains_only_names_labels_and_icons() {
        let listed = list_actions();
        assert_eq!(listed.len(), registry::ACTIONS.len());
        for (entry, spec) in listed.iter().zip(registry::ACTIONS) {
            let v = serde_json::to_value(entry).unwrap();
            assert_eq!(v.as_object().unwrap().len(), 3);
            assert_eq!(
                v,
                json!({"name": spec.name, "label": spec.ui.label, "icon": spec.ui.icon})
            );
        }
    }

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const FIXTURE_IDS: [&str; 9] = [
        "10000000000000000000000000000001", // notebook
        "10000000000000000000000000000002", // section
        "10000000000000000000000000000003", // second section
        "10000000000000000000000000000004", // page
        "10000000000000000000000000000005", // trashed page
        "10000000000000000000000000000006", // sticky
        "10000000000000000000000000000007", // database
        "10000000000000000000000000000008", // row
        "10000000000000000000000000000009", // view
    ];

    /// Two independent vaults with the same fixed records. No UI, no default
    /// Paths::resolve, no production data. Files are kept as throwaway evidence.
    pub(crate) fn compare<T: Serialize>(
        person: impl FnOnce(&AppState) -> Result<T>,
        core: impl FnOnce(&mut Session, &Paths) -> Result<T>,
    ) {
        let root = std::env::var_os("NOTAYA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("notaya-action-parity"))
            .join(format!(
                "action-parity-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        let left = Paths::at(&root.join("person"));
        let right = Paths::at(&root.join("core"));
        let seed = seed_items();
        let mut sessions = Vec::new();
        for paths in [&left, &right] {
            std::fs::create_dir_all(&paths.data).unwrap();
            let mut s = Session::create(&paths.vault, "throwaway", None, KdfParams { m_kib: 64, t: 1, p: 1 }).unwrap();
            s.put_many(seed.clone()).unwrap();
            sessions.push(s);
        }
        let mut right_session = sessions.pop().unwrap();
        let state = AppState::new(left);
        state.set_session(sessions.pop().unwrap());
        let a = person(&state)
            .map(|v| serde_json::to_value(v).unwrap())
            .map_err(|e| e.to_string());
        let b = core(&mut right_session, &right)
            .map(|v| serde_json::to_value(v).unwrap())
            .map_err(|e| e.to_string());
        assert!(a.is_ok(), "fixture must exercise success: {a:?}");
        // Only fresh random identifiers and wall-clock timestamps may differ.
        // Text, numbers, field shapes, existing ids and errors compare exactly.
        let mut left_ids = BTreeMap::new();
        let mut right_ids = BTreeMap::new();
        assert_eq!(
            canonical(a.unwrap(), &mut left_ids, ""),
            canonical(b.unwrap(), &mut right_ids, "")
        );
        let saved_a = state.with(|s| Ok(snapshot(s))).unwrap();
        let saved_b = snapshot(&right_session);
        assert_eq!(
            canonical(saved_a, &mut left_ids, ""),
            canonical(saved_b, &mut right_ids, "")
        );
    }

    fn snapshot(s: &Session) -> Value {
        let mut items: Vec<_> = s
            .items()
            .map(|i| (i.id().to_string(), serde_json::to_value(i).unwrap()))
            .collect();
        // Existing record order is stable; each fixture creates at most one item
        // of a given kind (a notebook also creates its first section).
        items.sort_by_key(|(id, v)| {
            let fresh = id.len() == 32 && !FIXTURE_IDS.contains(&id.as_str()) && id != crate::model::SETTINGS_ID;
            (
                v["kind"].as_str().unwrap().to_string(),
                fresh,
                if fresh { String::new() } else { id.clone() },
            )
        });
        Value::Array(items.into_iter().map(|(_, v)| v).collect())
    }

    fn canonical(v: Value, ids: &mut BTreeMap<String, String>, key: &str) -> Value {
        match v {
            Value::String(s)
                if s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()) && s != crate::model::SETTINGS_ID && !FIXTURE_IDS.contains(&s.as_str()) =>
            {
                let next = format!("fresh-{}", ids.len());
                Value::String(ids.entry(s).or_insert(next).clone())
            }
            Value::Number(n)
                if ["created", "updated", "trashedAt"].contains(&key) && n.as_i64().is_some_and(|n| n > 1_000_000) =>
            {
                json!("clock")
            }
            Value::Array(a) => Value::Array(a.into_iter().map(|v| canonical(v, ids, key)).collect()),
            Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| {
                        let v = canonical(v, ids, &k);
                        (k, v)
                    })
                    .collect(),
            ),
            v => v,
        }
    }

    pub(crate) fn database_json() -> String {
        let db = seed_items()
            .into_iter()
            .find_map(|i| if let Item::Database(d) = i { Some(d) } else { None })
            .unwrap();
        serde_json::to_string(&db).unwrap()
    }

    fn seed_items() -> Vec<Item> {
        let n = Notebook {
            source: None,
            id: "10000000000000000000000000000001".into(),
            name: "Notebook".into(),
            color: String::new(),
            icon: String::new(),
            order: 0,
            created: 1,
            updated: 1,
        trashed_at: None, };
        let section = Section {
            source: None,
            id: "10000000000000000000000000000002".into(),
            notebook_id: "10000000000000000000000000000001".into(),
            name: "Section".into(),
            color: String::new(),
            order: 0,
            created: 1,
            updated: 1,
        trashed_at: None, };
        let mut second = section.clone();
        second.id = "10000000000000000000000000000003".into();
        second.name = "Other".into();
        second.order = 1;
        let page: Page = serde_json::from_value(json!({"id":"10000000000000000000000000000004", "sectionId":"10000000000000000000000000000002", "title":"\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629} ? Deutsch  ", "content":"{\"type\":\"doc\",\"content\":[]}", "text":"\u{200f}exact\ttext  ", "order":0, "favorite":false, "parent":"", "trashedAt":null, "created":1, "updated":1})).unwrap();
        let mut trashed = page.clone();
        trashed.id = "10000000000000000000000000000005".into();
        trashed.trashed_at = Some(2);
        trashed.order = 1;
        let sticky: Sticky = serde_json::from_value(json!({"id":"10000000000000000000000000000006", "content":"exact", "text":"exact", "color":"#ffeeaa", "x":140.0, "y":140.0, "w":320.0, "h":320.0, "onTop":true, "open":true, "created":1, "updated":1})).unwrap();
        let mut db = Database::new("10000000000000000000000000000007".into(), "10000000000000000000000000000002".into(), "Finance".into(), 1);
        db.views[0].id = "10000000000000000000000000000009".into();
        db.props.push(Property {
            id: "number".into(),
            name: "Amount".into(),
            kind: PropKind::Number,
            ..Property::default()
        });
        db.props.push(Property {
            id: "button".into(),
            name: "Button".into(),
            kind: PropKind::Button,
            ..Property::default()
        });
        let mut row = Row::new("10000000000000000000000000000008".into(), "10000000000000000000000000000007", "Row".into(), 1);
        row.cells = r#"{"number":1234.567890123}"#.into();
        let mut settings = Settings::default();
        settings.calc_history = vec![CalcEntry {
            expr: "2+2".into(),
            result: "4".into(),
        }];
        settings.search_history = vec!["a search".into(), "older".into()];
        vec![
            Item::Settings(settings),
            Item::Notebook(n),
            Item::Section(section),
            Item::Section(second),
            Item::Page(page),
            Item::Page(trashed),
            Item::Sticky(sticky),
            Item::Database(db),
            Item::Row(row),
        ]
    }
}
