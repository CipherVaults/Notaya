//! Stable editor block identities. No text or other attribute is rewritten.
use std::collections::HashSet;

use serde_json::{json, Value};

pub fn valid_id(id: &str) -> bool {
    id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(super) fn addressable(node: &Value) -> bool {
    matches!(node["type"].as_str(), Some(
        "paragraph" | "heading" | "blockquote" | "bulletList" | "orderedList" |
        "taskList" | "listItem" | "taskItem" | "codeBlock" | "horizontalRule" |
        "image" | "table" | "tableRow" | "tableCell" | "tableHeader" | "callout" |
        "toggle" | "columns" | "column" | "mathBlock" | "mermaidBlock" | "pageLink" |
        "databaseBlock"
    ))
}

pub(super) fn collect(node: &Value, ids: &mut HashSet<String>) {
    if addressable(node) {
        if let Some(id) = node["attrs"]["bid"].as_str().filter(|id| valid_id(id)) {
            ids.insert(id.to_owned());
        }
    }
    if let Some(children) = node["content"].as_array() {
        for child in children { collect(child, ids); }
    }
}

fn assign(node: &mut Value, seen: &mut HashSet<String>, reserved: &mut HashSet<String>) -> bool {
    let mut changed = false;
    if addressable(node) {
        let id = node["attrs"]["bid"].as_str().filter(|id| valid_id(id)).map(str::to_owned);
        if !id.as_ref().is_some_and(|id| seen.insert(id.clone())) {
            let id = loop {
                let candidate = hex::encode(crate::crypto::random_bytes::<8>());
                if reserved.insert(candidate.clone()) { break candidate; }
            };
            seen.insert(id.clone());
            if !node["attrs"].is_object() { node["attrs"] = json!({}); }
            node["attrs"]["bid"] = Value::String(id);
            changed = true;
        }
    }
    if let Some(children) = node["content"].as_array_mut() {
        for child in children { changed |= assign(child, seen, reserved); }
    }
    changed
}

/// Assign missing/invalid/duplicate ids once. Persist the changed document before
/// returning it from an action read; repeated calls on that document are no-ops.
/// Callers accepting untrusted JSON must first use `blocks::validate`.
pub fn ensure_ids(doc: &mut Value) -> bool {
    let mut reserved = HashSet::new();
    collect(doc, &mut reserved);
    assign(doc, &mut HashSet::new(), &mut reserved)
}

/// New fragments cannot steal an identity already held by the destination.
pub(super) fn freshen(nodes: &mut [Value], destination: &Value) {
    let mut seen = HashSet::new();
    collect(destination, &mut seen);
    let mut reserved = seen.clone();
    for node in nodes.iter() { collect(node, &mut reserved); }
    for node in nodes { assign(node, &mut seen, &mut reserved); }
}
