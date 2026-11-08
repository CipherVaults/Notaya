//! Rust-only block editing for action callers. This module grants no permission:
//! the dispatcher must enforce pairing, level, confirmation, leases and revision.
mod ids;
mod ops;

pub use ids::ensure_ids;
pub use ops::{
    delete, insert, move_between, move_blocks, replace, set_direction, validate, Position,
};

use crate::{
    actions::Actor,
    commands::notes,
    error::{Error, Result},
    model::{Item, Page},
    vault::Session,
};
use serde_json::{json, Value};

/// Empty legacy pages have the editor's ordinary empty paragraph.
pub fn parse(content: &str) -> Result<Value> {
    let doc = if content.is_empty() {
        json!({"type":"doc","content":[{"type":"paragraph"}]})
    } else {
        serde_json::from_str(content).map_err(|_| Error::invalid("Invalid block document."))?
    };
    validate(&doc)?;
    Ok(doc)
}

/// Read for an action, persisting newly assigned identities so two reads agree.
/// Identity assignment preserves timestamps, title and the search text.
pub fn read(s: &mut Session, actor: &Actor, id: &str) -> Result<Page> {
    let mut page = notes::get_page_core(s, actor, id.to_owned())?;
    let mut doc = parse(&page.content)?;
    if ensure_ids(&mut doc) {
        page.content = doc.to_string();
        let item = match s.get(id) {
            Some(Item::Page(_)) => Item::Page(page.clone()),
            Some(Item::Row(row)) => {
                let mut row = row.clone();
                row.content = page.content.clone();
                Item::Row(row)
            }
            _ => return Err(Error::NotFound),
        };
        s.put(item)?;
    }
    Ok(page)
}

/// Exact blocks-format write. JSON serialization may change escaping/spacing;
/// every stored string (including the title) keeps its original UTF-8 bytes.
pub fn write(
    s: &mut Session,
    actor: &Actor,
    id: &str,
    title: String,
    mut doc: Value,
) -> Result<Page> {
    validate(&doc)?;
    ensure_ids(&mut doc);
    let text = plain_text(&doc);
    notes::save_page_core(s, actor, id.to_owned(), title, doc.to_string(), text)?;
    notes::get_page_core(s, actor, id.to_owned())
}

/// Search copy, without any of the export/Markdown whitespace cleanup.
pub fn plain_text(node: &Value) -> String {
    match node["type"].as_str().unwrap_or("") {
        "text" => node["text"].as_str().unwrap_or("").to_owned(),
        "hardBreak" => "\n".into(),
        "mermaidBlock" => node["attrs"]["code"].as_str().unwrap_or("").to_owned(),
        "mathInline" | "mathBlock" => node["attrs"]["latex"].as_str().unwrap_or("").to_owned(),
        _ => {
            let sep = if matches!(
                node["type"].as_str(),
                Some("paragraph" | "heading" | "codeBlock")
            ) {
                ""
            } else {
                "\n"
            };
            node["content"]
                .as_array()
                .map(|children| {
                    children
                        .iter()
                        .map(plain_text)
                        .collect::<Vec<_>>()
                        .join(sep)
                })
                .unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_arabic_and_direction_marks_survive_write_and_two_reads() {
        let root = std::env::var_os("NOTAYA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(crate::model::new_id());
        std::fs::create_dir_all(&root).unwrap();
        let mut session = Session::create_open(&root.join("vault.nota")).unwrap();
        let title = "\u{200f}عنوان\u{200e}  ";
        let line = "\u{061c}مرحبا\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}\u{2066}\u{2067}\u{2068}\u{2069}\u{200e}\u{200f}  ";
        let german = "?Wie geht es dir?  e\u{301}";
        let code = "\tgraph TD\n\tA[مرحبا] --> B  \n\n";
        let page: Page = serde_json::from_value(json!({"id":"11111111111111111111111111111111","sectionId":"s","title":"old","content":"","text":"","order":0,"favorite":false,"created":1,"updated":1,"trashedAt":null})).unwrap();
        session.put(Item::Page(page)).unwrap();
        let doc = json!({"type":"doc","content":[
            {"type":"paragraph","attrs":{"dir":"rtl"},"content":[{"type":"text","text":line}]},
            {"type":"paragraph","attrs":{"dir":"ltr"},"content":[{"type":"text","text":german}]},
            {"type":"codeBlock","attrs":{"language":"mermaid"},"content":[{"type":"text","text":code}]}
        ]});
        // A legacy page's first action read assigns and persists ids as well.
        let mut legacy = notes::get_page_core(
            &session,
            &Actor::Person,
            "11111111111111111111111111111111".into(),
        )
        .unwrap();
        legacy.content = doc.to_string();
        session.put(Item::Page(legacy)).unwrap();
        let first = read(
            &mut session,
            &Actor::Person,
            "11111111111111111111111111111111",
        )
        .unwrap();
        let second = read(
            &mut session,
            &Actor::Person,
            "11111111111111111111111111111111",
        )
        .unwrap();
        assert_eq!(first.content, second.content);
        let mut assigned: Value = serde_json::from_str(&first.content).unwrap();
        assert!(!ensure_ids(&mut assigned));
        assert_eq!(first.updated, 1);
        let page = write(
            &mut session,
            &Actor::Person,
            "11111111111111111111111111111111",
            title.into(),
            doc,
        )
        .unwrap();
        let first = read(
            &mut session,
            &Actor::Person,
            "11111111111111111111111111111111",
        )
        .unwrap();
        let second = read(
            &mut session,
            &Actor::Person,
            "11111111111111111111111111111111",
        )
        .unwrap();
        assert_eq!(first.content, second.content);
        assert_eq!(page.title.as_bytes(), title.as_bytes());
        let got: Value = serde_json::from_str(&second.content).unwrap();
        for (i, expected) in [line, german, code].iter().enumerate() {
            assert_eq!(
                got["content"][i]["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .as_bytes(),
                expected.as_bytes()
            );
        }
        assert_eq!(got["content"][0]["attrs"]["dir"], "rtl");
        assert_eq!(got["content"][1]["attrs"]["dir"], "ltr");
        drop(session);
        let mut session = Session::unlock_open(&root.join("vault.nota")).unwrap();
        assert_eq!(
            read(
                &mut session,
                &Actor::Person,
                "11111111111111111111111111111111"
            )
            .unwrap()
            .content,
            second.content
        );
        drop(session);
        std::fs::remove_dir_all(root).unwrap();
    }
}
