//! Transactional edits: a refused operation leaves the input document unchanged.
use super::{ensure_ids, ids};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::collections::HashSet;

fn invalid() -> Error {
    Error::invalid("Invalid block document or operation.")
}
fn kind(node: &Value) -> &str {
    node["type"].as_str().unwrap_or("")
}
fn children(node: &Value) -> &[Value] {
    node["content"].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn block(node: &Value) -> bool {
    ids::addressable(node)
        && !matches!(
            kind(node),
            "listItem" | "taskItem" | "tableRow" | "tableCell" | "tableHeader" | "column"
        )
}

/// The editor's content expressions, including the importer invariants. Reject
/// malformed input rather than silently dropping or normalizing its text.
pub fn validate(doc: &Value) -> Result<()> {
    if kind(doc) != "doc" || doc.to_string().len() > crate::importers::doc::MAX_RENDER_BYTES {
        return Err(invalid());
    }
    check(doc, 0, &mut 0)?;
    check_tables(doc, &mut 0)
}

/// Check expanded table geometry with at most one bounded row of occupancy.
/// A rowspan reserves its columns in following rows; holes and overlaps are
/// refused instead of asking the editor to repair an ambiguous grid.
fn check_tables(node: &Value, page_cells: &mut usize) -> Result<()> {
    use crate::importers::doc::{MAX_PAGE_CELLS, MAX_TABLE_CELLS, MAX_TABLE_COLS};
    if kind(node) == "table" {
        let rows = children(node);
        if rows.len() > MAX_TABLE_CELLS {
            return Err(invalid());
        }
        let mut remaining = [0usize; MAX_TABLE_COLS];
        let mut width = None;
        for (r, row) in rows.iter().enumerate() {
            let mut col = 0;
            for cell in children(row) {
                while col < MAX_TABLE_COLS && remaining[col] > 0 {
                    col += 1;
                }
                let cols = cell["attrs"]["colspan"].as_u64().unwrap_or(1) as usize;
                let span = cell["attrs"]["rowspan"].as_u64().unwrap_or(1) as usize;
                if cols > MAX_TABLE_COLS - col || span > rows.len() - r {
                    return Err(invalid());
                }
                for occupied in &mut remaining[col..col + cols] {
                    if *occupied != 0 {
                        return Err(invalid());
                    }
                    *occupied = span;
                }
                col += cols;
            }
            let row_width = remaining.iter().rposition(|n| *n > 0).map_or(0, |i| i + 1);
            if row_width == 0
                || remaining[..row_width].contains(&0)
                || width.is_some_and(|w| w != row_width)
            {
                return Err(invalid());
            }
            if width.is_none() {
                let cells = row_width * rows.len();
                if cells > MAX_TABLE_CELLS || cells > MAX_PAGE_CELLS.saturating_sub(*page_cells) {
                    return Err(invalid());
                }
                *page_cells += cells;
                width = Some(row_width);
            }
            for n in &mut remaining {
                *n = n.saturating_sub(1);
            }
        }
    }
    for child in children(node) {
        check_tables(child, page_cells)?;
    }
    Ok(())
}

fn check(node: &Value, depth: usize, count: &mut usize) -> Result<()> {
    *count += 1;
    if depth > 64 || *count > 200_000 || !node.is_object() {
        return Err(invalid());
    }
    if !super::attrs::valid(node, false) || node.get("content").is_some_and(|c| !c.is_array()) {
        return Err(invalid());
    }
    let kids = children(node);
    let inline = |n: &Value| matches!(kind(n), "text" | "hardBreak" | "mathInline");
    let nonempty = !kids.is_empty();
    let shape = match kind(node) {
        "doc" => depth == 0 && nonempty && kids.iter().all(block),
        "paragraph" | "heading" => kids.iter().all(inline),
        "codeBlock" => kids.iter().all(|n| {
            kind(n) == "text"
                && n.get("marks")
                    .is_none_or(|m| m.as_array().is_some_and(Vec::is_empty))
        }),
        "bulletList" | "orderedList" => nonempty && kids.iter().all(|n| kind(n) == "listItem"),
        "taskList" => nonempty && kids.iter().all(|n| kind(n) == "taskItem"),
        "listItem" | "taskItem" => {
            nonempty && kind(&kids[0]) == "paragraph" && kids.iter().all(block)
        }
        "blockquote" | "callout" | "toggle" | "column" | "tableCell" | "tableHeader" => {
            nonempty && kids.iter().all(block)
        }
        "columns" => (2..=5).contains(&kids.len()) && kids.iter().all(|n| kind(n) == "column"),
        "table" => nonempty && kids.iter().all(|n| kind(n) == "tableRow"),
        "tableRow" => {
            nonempty
                && kids
                    .iter()
                    .all(|n| matches!(kind(n), "tableCell" | "tableHeader"))
        }
        "text" => kids.is_empty() && node["text"].as_str().is_some_and(|s| !s.is_empty()),
        "hardBreak" | "mathInline" | "horizontalRule" | "image" | "mathBlock" | "mermaidBlock"
        | "pageLink" | "databaseBlock" => kids.is_empty(),
        _ => false,
    };
    if !shape {
        return Err(invalid());
    }
    if let Some(marks) = node.get("marks") {
        let marks = marks.as_array().ok_or_else(invalid)?;
        if !inline(node) && !marks.is_empty() {
            return Err(invalid());
        }
        let mut seen = HashSet::new();
        for m in marks {
            if !matches!(
                kind(m),
                "bold"
                    | "italic"
                    | "underline"
                    | "strike"
                    | "code"
                    | "link"
                    | "highlight"
                    | "textStyle"
                    | "comment"
            ) || !seen.insert(kind(m))
                || !super::attrs::valid(m, true)
            {
                return Err(invalid());
            }
        }
        if seen.contains("code") && marks.len() > 1 {
            return Err(invalid());
        }
    }
    if let Some(dir) = node["attrs"].get("dir") {
        if !dir.is_null() && !matches!(dir.as_str(), Some("rtl" | "ltr" | "auto")) {
            return Err(invalid());
        }
    }
    if kind(node) == "heading"
        && node["attrs"]
            .get("level")
            .is_some_and(|v| !v.as_u64().is_some_and(|n| (1..=4).contains(&n)))
    {
        return Err(invalid());
    }
    if matches!(kind(node), "tableCell" | "tableHeader") {
        for key in ["colspan", "rowspan"] {
            if node["attrs"]
                .get(key)
                .is_some_and(|v| !v.as_u64().is_some_and(|n| n > 0 && n <= 20_000))
            {
                return Err(invalid());
            }
        }
    }
    for child in kids {
        check(child, depth + 1, count)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum Position<'a> {
    Start,
    End,
    Before(&'a str),
    After(&'a str),
}

fn path(node: &Value, id: &str) -> Result<Vec<usize>> {
    fn find(node: &Value, id: &str, here: &mut Vec<usize>) -> bool {
        if ids::addressable(node) && node["attrs"]["bid"].as_str() == Some(id) {
            return true;
        }
        for (i, child) in children(node).iter().enumerate() {
            here.push(i);
            if find(child, id, here) {
                return true;
            }
            here.pop();
        }
        false
    }
    let mut here = Vec::new();
    if !id.is_empty() && find(node, id, &mut here) {
        Ok(here)
    } else {
        Err(Error::NotFound)
    }
}

fn at_mut<'a>(mut doc: &'a mut Value, path: &[usize]) -> &'a mut Value {
    for &i in path {
        doc = &mut doc["content"][i];
    }
    doc
}

fn prepared(doc: &Value) -> Result<Value> {
    validate(doc)?;
    let mut out = doc.clone();
    ensure_ids(&mut out);
    Ok(out)
}

fn finish(original: &mut Value, mut next: Value) -> Result<()> {
    if children(&next).is_empty() {
        next["content"] = json!([{"type":"paragraph"}]);
    }
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
    let list = at_mut(doc, &parent)["content"]
        .as_array_mut()
        .ok_or_else(invalid)?;
    list.splice(index..index, nodes);
    Ok(())
}

pub fn insert(doc: &mut Value, pos: Position<'_>, mut nodes: Vec<Value>) -> Result<()> {
    if nodes.is_empty() {
        return Err(invalid());
    }
    for node in &nodes {
        check(node, 1, &mut 0)?;
    }
    let mut next = prepared(doc)?;
    ids::freshen(&mut nodes, &next);
    put(&mut next, pos, nodes)?;
    finish(doc, next)
}

/// Replace one addressed node; its identity survives a type/content change.
pub fn replace(doc: &mut Value, id: &str, node: Value) -> Result<()> {
    check(&node, 1, &mut 0)?;
    let mut next = prepared(doc)?;
    let p = path(&next, id)?;
    let mut nodes = vec![node];
    // Assign fragment identities against the old document before restoring the
    // addressed identity; existing unrelated ids are never changed.
    ids::freshen(&mut nodes, &next);
    if !ids::addressable(&nodes[0]) {
        return Err(invalid());
    }
    nodes[0]["attrs"]["bid"] = Value::String(id.to_owned());
    *at_mut(&mut next, &p) = nodes.remove(0);
    finish(doc, next)
}

fn selection(doc: &Value, selected: &[String]) -> Result<Vec<Vec<usize>>> {
    if selected.is_empty() {
        return Err(invalid());
    }
    let mut set = HashSet::new();
    let mut paths = Vec::new();
    for id in selected {
        if !set.insert(id.clone()) {
            return Err(invalid());
        }
        paths.push(path(doc, id)?);
    }
    paths.sort();
    if paths.windows(2).any(|p| p[1].starts_with(&p[0])) {
        return Err(invalid());
    }
    Ok(paths)
}

/// Cut in document order, independent of the caller's selection order.
fn cut(node: &mut Value, selected: &[Vec<usize>], out: &mut Vec<Value>) {
    fn walk(
        node: &mut Value,
        selected: &[Vec<usize>],
        here: &mut Vec<usize>,
        out: &mut Vec<Value>,
    ) {
        if let Some(kids) = node.get_mut("content").and_then(Value::as_array_mut) {
            let mut kept = Vec::with_capacity(kids.len());
            for (i, mut child) in std::mem::take(kids).into_iter().enumerate() {
                here.push(i);
                if selected.contains(here) {
                    out.push(child);
                } else {
                    walk(&mut child, selected, here, out);
                    kept.push(child);
                }
                here.pop();
            }
            *kids = kept;
        }
    }
    walk(node, selected, &mut Vec::new(), out);
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
pub fn move_between(
    source: &mut Value,
    target: &mut Value,
    selected: &[String],
    pos: Position<'_>,
) -> Result<()> {
    let mut from = prepared(source)?;
    let mut to = prepared(target)?;
    let set = selection(&from, selected)?;
    let mut nodes = Vec::new();
    cut(&mut from, &set, &mut nodes);
    // Preserve moved ids; reject a collision instead of changing a live identity.
    let mut dest = HashSet::new();
    ids::collect(&to, &mut dest);
    let mut moved = HashSet::new();
    for n in &nodes {
        ids::collect(n, &mut moved);
    }
    if moved.iter().any(|id| dest.contains(id)) {
        return Err(invalid());
    }
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
    if !matches!(direction, "ltr" | "rtl" | "auto") {
        return Err(invalid());
    }
    let mut next = prepared(doc)?;
    let p = path(&next, id)?;
    let node = at_mut(&mut next, &p);
    if !matches!(kind(node), "paragraph" | "heading") {
        return Err(invalid());
    }
    node["attrs"]["dir"] = if direction == "auto" {
        Value::Null
    } else {
        Value::String(direction.into())
    };
    finish(doc, next)
}

#[cfg(test)]
mod tests {
    use crate::blocks::{
        delete, ensure_ids, insert, move_between, move_blocks, replace, set_direction, validate,
        Position,
    };
    use serde_json::{json, Value};
    fn p(id: &str, text: &str) -> Value {
        json!({"type":"paragraph","attrs":{"bid":id},"content":[{"type":"text","text":text}]})
    }
    fn doc() -> Value {
        json!({"type":"doc","content":[p("1111111111111111","أ  "),p("2222222222222222","?Deutsch\u{200e}"),p("3333333333333333","e\u{301}")]})
    }

    #[test]
    fn insert_replace_delete_move_and_direction_preserve_text_and_ids() {
        let mut d = doc();
        insert(
            &mut d,
            Position::Before("1111111111111111"),
            vec![p("1111111111111111", "new")],
        )
        .unwrap();
        assert_ne!(d["content"][0]["attrs"]["bid"], "1111111111111111");
        assert_eq!(d["content"][1]["attrs"]["bid"], "1111111111111111");
        replace(&mut d, "2222222222222222", json!({"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"\u{200f}?German  "}]})).unwrap();
        set_direction(&mut d, "2222222222222222", "rtl").unwrap();
        move_blocks(
            &mut d,
            &["3333333333333333".into(), "2222222222222222".into()],
            Position::Start,
        )
        .unwrap();
        assert_eq!(d["content"][0]["attrs"]["bid"], "2222222222222222");
        assert_eq!(d["content"][0]["content"][0]["text"], "\u{200f}?German  ");
        delete(&mut d, &["1111111111111111".into()]).unwrap();
        assert!(!ensure_ids(&mut d));
        validate(&d).unwrap();
    }

    #[test]
    fn typed_attributes_reject_bad_nodes_and_marks_without_rewriting_text() {
        let mut d = doc();
        let before = d.clone();
        for node in [
            json!({"type":"mathBlock","attrs":{"latex":123}}),
            json!({"type":"mathInline","attrs":{"latex":null}}),
            json!({"type":"mermaidBlock","attrs":{"code":{},"showCode":false}}),
            json!({"type":"mermaidBlock","attrs":{"code":"secret","showCode":"true"}}),
            json!({"type":"mathBlock","attrs":{"latex":"x".repeat(20_001)}}),
            json!({"type":"mermaidBlock","attrs":{"code":"x".repeat(50_001)}}),
            json!({"type":"image","attrs":{"src":{},"width":"90"}}),
            json!({"type":"pageLink","attrs":{"pageId":123}}),
            json!({"type":"databaseBlock","attrs":{"databaseId":[]}}),
            json!({"type":"toggle","attrs":{"open":"yes"},"content":[{"type":"paragraph"}]}),
            json!({"type":"paragraph","content":[{"type":"text","text":"secret","marks":[{"type":"link","attrs":{"href":42}}]}]}),
            json!({"type":"paragraph","content":[{"type":"text","text":"secret","marks":[{"type":"textStyle","attrs":{"fontFamily":{}}}]}]}),
            json!({"type":"paragraph","content":[{"type":"text","text":"secret","marks":[{"type":"comment","attrs":{"text":false}}]}]}),
        ] {
            let error = insert(&mut d, Position::End, vec![node.clone()])
                .err()
                .unwrap();
            assert!(
                matches!(error, crate::error::Error::Invalid(ref msg) if msg == "Invalid block document or operation.")
            );
            assert_eq!(d, before);
            assert!(replace(&mut d, "1111111111111111", node).is_err());
            assert_eq!(d, before);
        }
        let exact = "\u{200f}\u{0639}\u{05d0} e\u{301}\u{200e}  \n";
        for (kind, attr) in [("mathBlock", "latex"), ("mermaidBlock", "code")] {
            insert(
                &mut d,
                Position::End,
                vec![json!({"type":kind,"attrs":{attr:exact}})],
            )
            .unwrap();
            assert_eq!(
                d["content"].as_array().unwrap().last().unwrap()["attrs"][attr]
                    .as_str()
                    .unwrap()
                    .as_bytes(),
                exact.as_bytes()
            );
        }
    }

    #[test]
    fn forged_inline_ids_are_refused_by_delete_and_both_moves_atomically() {
        for inline in ["text", "hardBreak", "mathInline"] {
            for bid in [json!("1111111111111111"), Value::Null] {
                let mut d = doc();
                let child = &mut d["content"][1]["content"][0];
                child["type"] = json!(inline);
                child["attrs"] = json!({"bid":bid});
                let before = d.clone();
                let mut target = doc();
                let target_before = target.clone();
                assert!(delete(&mut d, &["1111111111111111".into()]).is_err());
                assert_eq!(d, before);
                assert!(move_blocks(&mut d, &["1111111111111111".into()], Position::End).is_err());
                assert_eq!(d, before);
                assert!(move_between(
                    &mut d,
                    &mut target,
                    &["1111111111111111".into()],
                    Position::End
                )
                .is_err());
                assert_eq!(d, before);
                assert_eq!(target, target_before);
            }
        }
    }

    #[test]
    fn expanded_table_budgets_and_span_geometry_are_enforced() {
        fn cell(cols: usize, rows: usize) -> Value {
            json!({"type":"tableCell","attrs":{"colspan":cols,"rowspan":rows},"content":[{"type":"paragraph"}]})
        }
        fn table(width: usize, height: usize) -> Value {
            json!({"type":"table","content":(0..height).map(|_| json!({"type":"tableRow","content":[cell(width, 1)]})).collect::<Vec<_>>()})
        }
        let accepted = json!({"type":"doc","content":[table(100, 200)]});
        validate(&accepted).unwrap();
        validate(&json!({"type":"doc","content":vec![table(100,200);5]})).unwrap();
        let valid_span = json!({"type":"table","content":[
            {"type":"tableRow","content":[cell(99,2),cell(1,1)]},
            {"type":"tableRow","content":[cell(1,1)]}
        ]});
        validate(&json!({"type":"doc","content":[valid_span]})).unwrap();
        for bad in [
            json!({"type":"doc","content":[table(101,1)]}),
            json!({"type":"doc","content":[table(100,201)]}),
            json!({"type":"doc","content":vec![table(100,200);6]}),
            json!({"type":"doc","content":[{"type":"table","content":[{"type":"tableRow","content":[cell(1,2)]}]}]}),
            json!({"type":"doc","content":[{"type":"table","content":[{"type":"tableRow","content":[cell(2,1)]},{"type":"tableRow","content":[cell(1,1)]}]}]}),
            json!({"type":"doc","content":[{"type":"table","content":[{"type":"tableRow","content":[cell(1,1),cell(1,2)]},{"type":"tableRow","content":[cell(2,1)]}]}]}),
        ] {
            assert!(validate(&bad).is_err());
            let mut original = doc();
            let before = original.clone();
            assert!(insert(
                &mut original,
                Position::End,
                bad["content"].as_array().unwrap().clone()
            )
            .is_err());
            assert_eq!(original, before);
        }
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
        assert!(move_blocks(
            &mut d,
            &["1111111111111111".into()],
            Position::After("1111111111111111")
        )
        .is_err());
        assert_eq!(d, before);
        delete(
            &mut d,
            &[
                "1111111111111111".into(),
                "2222222222222222".into(),
                "3333333333333333".into(),
            ],
        )
        .unwrap();
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
        let item = d["content"][0]["content"][0]["attrs"]["bid"]
            .as_str()
            .unwrap()
            .to_owned();
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
