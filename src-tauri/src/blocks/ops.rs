//! Transactional edits: a refused operation leaves the input document unchanged.
use std::collections::HashSet;
use serde_json::{json, Value};
use crate::error::{Error, Result};
use super::{ensure_ids, ids};

fn invalid() -> Error { Error::invalid("Invalid block document or operation.") }
fn kind(node: &Value) -> &str { node["type"].as_str().unwrap_or("") }
fn children(node: &Value) -> &[Value] { node["content"].as_array().map(Vec::as_slice).unwrap_or(&[]) }
fn block(node: &Value) -> bool {
    ids::addressable(node) && !matches!(kind(node), "listItem" | "taskItem" | "tableRow" | "tableCell" | "tableHeader" | "column")
}

/// The editor's content expressions, including the importer invariants. Reject
/// malformed input rather than silently dropping or normalizing its text.
pub fn validate(doc: &Value) -> Result<()> {
    if kind(doc) != "doc" || doc.to_string().len() > crate::importers::doc::MAX_RENDER_BYTES { return Err(invalid()); }
    check(doc, 0, &mut 0)
}

fn check(node: &Value, depth: usize, count: &mut usize) -> Result<()> {
    *count += 1;
    if depth > 64 || *count > 200_000 || !node.is_object() { return Err(invalid()); }
    if node.get("attrs").is_some_and(|a| !a.is_object()) || node.get("content").is_some_and(|c| !c.is_array()) { return Err(invalid()); }
    let kids = children(node);
    let inline = |n: &Value| matches!(kind(n), "text" | "hardBreak" | "mathInline");
    let nonempty = !kids.is_empty();
    let shape = match kind(node) {
        "doc" => depth == 0 && nonempty && kids.iter().all(block),
        "paragraph" | "heading" => kids.iter().all(inline),
        "codeBlock" => kids.iter().all(|n| kind(n) == "text" && n.get("marks").is_none_or(|m| m.as_array().is_some_and(Vec::is_empty))),
        "bulletList" | "orderedList" => nonempty && kids.iter().all(|n| kind(n) == "listItem"),
        "taskList" => nonempty && kids.iter().all(|n| kind(n) == "taskItem"),
        "listItem" | "taskItem" => nonempty && kind(&kids[0]) == "paragraph" && kids.iter().all(block),
        "blockquote" | "callout" | "toggle" | "column" | "tableCell" | "tableHeader" => nonempty && kids.iter().all(block),
        "columns" => (2..=5).contains(&kids.len()) && kids.iter().all(|n| kind(n) == "column"),
        "table" => nonempty && kids.iter().all(|n| kind(n) == "tableRow"),
        "tableRow" => nonempty && kids.iter().all(|n| matches!(kind(n), "tableCell" | "tableHeader")),
        "text" => kids.is_empty() && node["text"].as_str().is_some_and(|s| !s.is_empty()),
        "hardBreak" | "mathInline" | "horizontalRule" | "image" | "mathBlock" | "mermaidBlock" | "pageLink" | "databaseBlock" => kids.is_empty(),
        _ => false,
    };
    if !shape { return Err(invalid()); }
    if let Some(marks) = node.get("marks") {
        let marks = marks.as_array().ok_or_else(invalid)?;
        if !inline(node) && !marks.is_empty() { return Err(invalid()); }
        let mut seen = HashSet::new();
        for m in marks {
            if !matches!(kind(m), "bold" | "italic" | "underline" | "strike" | "code" | "link" | "highlight" | "textStyle" | "comment")
                || !seen.insert(kind(m)) || m.get("attrs").is_some_and(|a| !a.is_object()) { return Err(invalid()); }
        }
        if seen.contains("code") && marks.len() > 1 { return Err(invalid()); }
    }
    if let Some(dir) = node["attrs"].get("dir") {
        if !dir.is_null() && !matches!(dir.as_str(), Some("rtl" | "ltr" | "auto")) { return Err(invalid()); }
    }
    if kind(node) == "heading" && node["attrs"].get("level").is_some_and(|v| !v.as_u64().is_some_and(|n| (1..=4).contains(&n))) { return Err(invalid()); }
    if matches!(kind(node), "tableCell" | "tableHeader") {
        for key in ["colspan", "rowspan"] {
            if node["attrs"].get(key).is_some_and(|v| !v.as_u64().is_some_and(|n| n > 0 && n <= 20_000)) { return Err(invalid()); }
        }
    }
    for child in kids { check(child, depth + 1, count)?; }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum Position<'a> { Start, End, Before(&'a str), After(&'a str) }

fn path(node: &Value, id: &str) -> Result<Vec<usize>> {
    fn find(node: &Value, id: &str, here: &mut Vec<usize>) -> bool {
        if ids::addressable(node) && node["attrs"]["bid"].as_str() == Some(id) { return true; }
        for (i, child) in children(node).iter().enumerate() {
            here.push(i);
            if find(child, id, here) { return true; }
            here.pop();
        }
        false
    }
    let mut here = Vec::new();
    if !id.is_empty() && find(node, id, &mut here) { Ok(here) } else { Err(Error::NotFound) }
}

fn at_mut<'a>(mut doc: &'a mut Value, path: &[usize]) -> &'a mut Value {
    for &i in path { doc = &mut doc["content"][i]; }
    doc
}

fn prepared(doc: &Value) -> Result<Value> {
    validate(doc)?;
    let mut out = doc.clone();
    ensure_ids(&mut out);
    Ok(out)
}

fn finish(original: &mut Value, mut next: Value) -> Result<()> {
    if children(&next).is_empty() { next["content"] = json!([{"type":"paragraph"}]); }
    validate(&next)?;
    ensure_ids(&mut next);
    *original = next;
    Ok(())
}

fn put(doc: &mut Value, pos: Position<'_>, nodes: Vec<Value>) -> Result<()> {
    let (parent, index) = match pos {
        Position::Start => (vec![], 0),
        Position::End => (vec![], children(doc).len()),
        Position::Before(id) | Position::After(id) => {
            let mut p = path(doc, id)?;
            let i = p.pop().ok_or_else(invalid)?;
            (p, i + usize::from(matches!(pos, Position::After(_))))
        }
    };
    let list = at_mut(doc, &parent)["content"].as_array_mut().ok_or_else(invalid)?;
    list.splice(index..index, nodes);
    Ok(())
}

pub fn insert(doc: &mut Value, pos: Position<'_>, mut nodes: Vec<Value>) -> Result<()> {
    if nodes.is_empty() { return Err(invalid()); }
    let mut next = prepared(doc)?;
    ids::freshen(&mut nodes, &next);
    put(&mut next, pos, nodes)?;
    finish(doc, next)
}

/// Replace one addressed node; its identity survives a type/content change.
pub fn replace(doc: &mut Value, id: &str, node: Value) -> Result<()> {
    let mut next = prepared(doc)?;
    let p = path(&next, id)?;
    let mut nodes = vec![node];
    // Assign fragment identities against the old document before restoring the
    // addressed identity; existing unrelated ids are never changed.
    ids::freshen(&mut nodes, &next);
    if !ids::addressable(&nodes[0]) { return Err(invalid()); }
    nodes[0]["attrs"]["bid"] = Value::String(id.to_owned());
    *at_mut(&mut next, &p) = nodes.remove(0);
    finish(doc, next)
}

fn selection(doc: &Value, selected: &[String]) -> Result<HashSet<String>> {
    if selected.is_empty() { return Err(invalid()); }
    let mut set = HashSet::new();
    let mut paths = Vec::new();
    for id in selected {
        if !set.insert(id.clone()) { return Err(invalid()); }
        paths.push(path(doc, id)?);
    }
    paths.sort();
    if paths.windows(2).any(|p| p[1].starts_with(&p[0])) { return Err(invalid()); }
    Ok(set)
}

/// Cut in document order, independent of the caller's selection order.
fn cut(node: &mut Value, selected: &HashSet<String>, out: &mut Vec<Value>) {
    if let Some(kids) = node["content"].as_array_mut() {
        let mut i = 0;
        while i < kids.len() {
            if kids[i]["attrs"]["bid"].as_str().is_some_and(|id| selected.contains(id)) {
                out.push(kids.remove(i));
            } else { cut(&mut kids[i], selected, out); i += 1; }
        }
    }
}

pub fn delete(doc: &mut Value, selected: &[String]) -> Result<()> {
    let mut next = prepared(doc)?;
    let set = selection(&next, selected)?;
    cut(&mut next, &set, &mut Vec::new());
    finish(doc, next)
}

pub fn move_blocks(doc: &mut Value, selected: &[String], pos: Position<'_>) -> Result<()> {
    let mut next = prepared(doc)?;
    let set = selection(&next, selected)?;
    let mut nodes = Vec::new();
    cut(&mut next, &set, &mut nodes);
    put(&mut next, pos, nodes)?;
    finish(doc, next)
}

/// Two-page move is atomic in memory. The action caller must save both pages in
/// one Session::put_many transaction after checking both permissions/revisions.
pub fn move_between(source: &mut Value, target: &mut Value, selected: &[String], pos: Position<'_>) -> Result<()> {
    let mut from = prepared(source)?;
    let mut to = prepared(target)?;
    let set = selection(&from, selected)?;
    let mut nodes = Vec::new();
    cut(&mut from, &set, &mut nodes);
    // Preserve moved ids; reject a collision instead of changing a live identity.
    let mut dest = HashSet::new();
    ids::collect(&to, &mut dest);
    let mut moved = HashSet::new();
    for n in &nodes { ids::collect(n, &mut moved); }
    if moved.iter().any(|id| dest.contains(id)) { return Err(invalid()); }
    put(&mut to, pos, nodes)?;
    let mut new_source = source.clone();
    finish(&mut new_source, from)?;
    let mut new_target = target.clone();
    finish(&mut new_target, to)?;
    *source = new_source;
    *target = new_target;
    Ok(())
}

pub fn set_direction(doc: &mut Value, id: &str, direction: &str) -> Result<()> {
    if !matches!(direction, "ltr" | "rtl" | "auto") { return Err(invalid()); }
    let mut next = prepared(doc)?;
    let p = path(&next, id)?;
    let node = at_mut(&mut next, &p);
    if !matches!(kind(node), "paragraph" | "heading") { return Err(invalid()); }
    node["attrs"]["dir"] = if direction == "auto" { Value::Null } else { Value::String(direction.into()) };
    finish(doc, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(id: &str, text: &str) -> Value { json!({"type":"paragraph","attrs":{"bid":id},"content":[{"type":"text","text":text}]}) }
    fn doc() -> Value { json!({"type":"doc","content":[p("1111111111111111","أ  "),p("2222222222222222","?Deutsch\u{200e}"),p("3333333333333333","e\u{301}")]}) }

    #[test]
    fn insert_replace_delete_move_and_direction_preserve_text_and_ids() {
        let mut d = doc();
        insert(&mut d, Position::Before("1111111111111111"), vec![p("1111111111111111", "new")]).unwrap();
        assert_ne!(d["content"][0]["attrs"]["bid"], "1111111111111111");
        assert_eq!(d["content"][1]["attrs"]["bid"], "1111111111111111");
        replace(&mut d, "2222222222222222", json!({"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"\u{200f}?German  "}]})).unwrap();
        set_direction(&mut d, "2222222222222222", "rtl").unwrap();
        move_blocks(&mut d, &["3333333333333333".into(), "2222222222222222".into()], Position::Start).unwrap();
        assert_eq!(d["content"][0]["attrs"]["bid"], "2222222222222222");
        assert_eq!(d["content"][0]["content"][0]["text"], "\u{200f}?German  ");
        delete(&mut d, &["1111111111111111".into()]).unwrap();
        assert!(!ensure_ids(&mut d));
        validate(&d).unwrap();
    }

    #[test]
    fn failed_operations_are_atomic_and_schema_is_enforced() {
        let mut d = doc();
        let before = d.clone();
        assert!(delete(&mut d, &["1111111111111111".into(), "missing".into()]).is_err());
        assert_eq!(d, before);
        for node in [
            json!({"type":"bulletList","content":[]}),
            json!({"type":"image","content":[p("", "bad")]}),
            json!({"type":"paragraph","content":[{"type":"text","text":""}]}),
            json!({"type":"codeBlock","content":[{"type":"text","text":"x","marks":[{"type":"bold"}]}]}),
            json!({"type":"listItem","content":[{"type":"codeBlock"}]}),
        ] {
            assert!(insert(&mut d, Position::End, vec![node]).is_err());
            assert_eq!(d, before);
        }
        assert!(move_blocks(&mut d, &["1111111111111111".into()], Position::After("1111111111111111")).is_err());
        assert_eq!(d, before);
        delete(&mut d, &["1111111111111111".into(),"2222222222222222".into(),"3333333333333333".into()]).unwrap();
        assert_eq!(d["content"][0]["type"], "paragraph");
    }

    #[test]
    fn nested_ids_container_rules_and_cross_page_moves() {
        let mut d = json!({"type":"doc","content":[{"type":"bulletList","content":[
            {"type":"listItem","content":[{"type":"paragraph"}]},
            {"type":"listItem","content":[{"type":"paragraph"}]}
        ]},{"type":"table","content":[{"type":"tableRow","content":[{"type":"tableCell","content":[{"type":"paragraph"}]}]}]}]});
        assert!(ensure_ids(&mut d));
        assert!(!ensure_ids(&mut d));
        let item = d["content"][0]["content"][0]["attrs"]["bid"].as_str().unwrap().to_owned();
        let list = d["content"][0]["attrs"]["bid"].as_str().unwrap().to_owned();
        let before = d.clone();
        assert!(delete(&mut d, &[list.clone(), item.clone()]).is_err());
        assert_eq!(d, before);
        // Moving a list item out of a list would violate doc's block+ schema.
        assert!(move_blocks(&mut d, &[item], Position::End).is_err());
        let mut target = doc();
        move_between(&mut d, &mut target, &[list.clone()], Position::End).unwrap();
        assert_eq!(target["content"][3]["attrs"]["bid"], list);
        validate(&d).unwrap();
        let before = target.clone();
        let mut collision = target.clone();
        assert!(move_between(&mut target, &mut collision, &[list], Position::End).is_err());
        assert_eq!(target, before);
        assert_eq!(collision, before);
    }
}
