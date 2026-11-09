//! A closed action match, independent of Tauri's command dispatcher.
use super::{pairing::Pairing, rules};
use crate::{
    actions::{registry::ACTIONS, ActionSpec, Actor, Confirm, Level as Required},
    blocks,
    commands::{self, calculator, database as db, notes, stickies},
    database::{engine::Ctx, saves, types::*},
    error::{Error, Result},
    model::{Item, SourceTag},
    portable::Paths,
    state::AppState,
    vault::Session,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use crate::native_confirm::{self, Approval, Delete};
use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::Ordering,
};

fn invalid() -> Error {
    Error::invalid("invalid")
}
fn code(code: &'static str) -> Error {
    Error::invalid(code)
}
pub fn error_code(e: &Error) -> &'static str {
    match e {
        Error::Locked => "locked",
        Error::NotFound => "not_found",
        Error::RateLimited(_) => "rate_limited",
        Error::Invalid(s) => {
            for c in [
                "off",
                "stopped",
                "not_paired",
                "pairing_refused",
                "level_too_low",
                "not_found",
                "person_editing",
                "claimed_by_other_ai",
                "item_changed",
                "not_confirmed",
                "cancelled",
                "too_big",
                "rate_limited",
            ] {
                if s == c || s.starts_with(&format!("{c}:")) {
                    return c;
                }
            }
            if s.starts_with("revision_conflict:") {
                "item_changed"
            } else {
                "invalid"
            }
        }
        _ => "invalid",
    }
}
fn value<T: Serialize>(v: T) -> Result<Value> {
    serde_json::to_value(v).map_err(|_| invalid())
}
fn get<T: DeserializeOwned>(p: &Value, key: &str) -> Result<T> {
    serde_json::from_value(p.get(key).cloned().ok_or_else(invalid)?).map_err(|_| invalid())
}
fn text(p: &Value, key: &str) -> String {
    p[key].as_str().unwrap_or("").into()
}
fn boolean(p: &Value, key: &str) -> bool {
    p[key].as_bool().unwrap_or(false)
}
fn kept(p: &Value, key: &str, old: &str) -> String {
    p[key].as_str().unwrap_or(old).into()
}
fn ids(p: &Value, key: &str) -> Result<Vec<String>> {
    get(p, key)
}
pub fn actor(p: &Pairing) -> Actor {
    Actor::Ai {
        pairing_id: p.id.clone(),
        label: p.name.clone(),
    }
}

fn schema(s: &Value, v: &Value) -> bool {
    if let Some(choices) = s["oneOf"].as_array() {
        return choices.iter().filter(|c| schema(c, v)).count() == 1;
    }
    if let Some(choices) = s["enum"].as_array() {
        if !choices.contains(v) {
            return false;
        }
    }
    let valid = match s["type"].as_str() {
        Some("string") => v.is_string(),
        Some("boolean") => v.is_boolean(),
        Some("integer") => v.as_i64().is_some() || v.as_u64().is_some(),
        Some("number") => v.is_number(),
        Some("array") => v
            .as_array()
            .is_some_and(|a| a.len() <= 10_000 && a.iter().all(|x| schema(&s["items"], x))),
        Some("object") => v.as_object().is_some_and(|o| {
            s["required"].as_array().is_none_or(|r| {
                r.iter()
                    .all(|k| k.as_str().is_some_and(|k| o.contains_key(k)))
            }) && o.iter().all(|(k, v)| match s["properties"].get(k) {
                Some(s) => schema(s, v),
                None => s["additionalProperties"] != false,
            })
        }),
        _ => true,
    };
    valid
        && s["minimum"]
            .as_f64()
            .is_none_or(|n| v.as_f64().is_some_and(|x| x >= n))
        && s["maximum"]
            .as_f64()
            .is_none_or(|n| v.as_f64().is_some_and(|x| x <= n))
}
pub fn hidden(s: &Session, id: &str) -> bool {
    s.settings()
        .ai
        .hidden_notebooks
        .iter()
        .any(|root| s.is_within(id, root))
}
fn visible(s: &Session, i: &Item) -> bool {
    !matches!(i, Item::Settings(_)) && !hidden(s, i.id())
}
fn live(s: &Session, i: &Item) -> bool {
    visible(s, i) && !s.in_bin(i.id())
}
fn require(s: &Session, id: &str, bin: bool) -> Result<()> {
    if s.get(id).is_none_or(|i| !visible(s, i)) || (!bin && s.in_bin(id)) {
        Err(Error::NotFound)
    } else {
        Ok(())
    }
}
fn source(i: &Item) -> Option<&SourceTag> {
    match i {
        Item::Notebook(x) => x.source.as_ref(),
        Item::Section(x) => x.source.as_ref(),
        Item::Page(x) => x.source.as_ref(),
        Item::Database(x) => x.source.as_ref(),
        Item::Row(x) => x.source.as_ref(),
        _ => None,
    }
}
fn set_source(s: &mut Session, id: &str, tag: &Value) -> Result<()> {
    if tag.is_null() {
        return Ok(());
    }
    let tag: SourceTag = serde_json::from_value(tag.clone()).map_err(|_| invalid())?;
    if tag.app.is_empty() || tag.app.len() > 64 || tag.id.is_empty() || tag.id.len() > 256 {
        return Err(invalid());
    }
    let mut item = s.get(id).cloned().ok_or(Error::NotFound)?;
    match &mut item {
        Item::Notebook(x) => x.source = Some(tag),
        Item::Section(x) => x.source = Some(tag),
        Item::Page(x) => x.source = Some(tag),
        Item::Database(x) => x.source = Some(tag),
        Item::Row(x) => x.source = Some(tag),
        _ => return Err(invalid()),
    }
    s.put(item)
}

fn validate_source(tag: &Value) -> Result<()> {
    if tag.is_null() {
        return Ok(());
    }
    let tag: SourceTag = serde_json::from_value(tag.clone()).map_err(|_| invalid())?;
    if tag.app.is_empty() || tag.app.len() > 64 || tag.id.is_empty() || tag.id.len() > 256 {
        return Err(invalid());
    }
    Ok(())
}

pub struct Prepared {
    pub spec: &'static ActionSpec,
    pub params: Value,
    pub targets: Vec<String>,
    pub revs: BTreeMap<String, u64>,
    pub confirm: Confirm,
    pub count: usize,
}
pub fn prepare(s: &Session, pairing: &Pairing, name: &str, mut params: Value) -> Result<Prepared> {
    let a = s.settings().ai;
    // Caller has already established the unlocked session. Order is security policy.
    if !a.enabled {
        return Err(code("off"));
    }
    if a.paused {
        return Err(code("stopped"));
    }
    let p = a
        .pairings
        .iter()
        .find(|p| p.id == pairing.id && p.token_hash == pairing.token_hash)
        .ok_or_else(|| code("not_paired"))?;
    let spec = ACTIONS
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(invalid)?;
    // Unsupported execution must also fail closed for previews and discovery.
    if !rules::allowed(p.level, spec) {
        return Err(code("level_too_low"));
    }
    if name == "batch.run" { return Err(invalid()); }
    let expect = params
        .as_object_mut()
        .ok_or_else(invalid)?
        .remove("expect_rev");
    let expected = expect.map(|v| v.as_u64().ok_or_else(invalid)).transpose()?;
    if !schema(
        &serde_json::from_str::<Value>(spec.params).map_err(|_| invalid())?,
        &params,
    ) {
        return Err(invalid());
    }
    validate_source(&params["source"])?;
    let mut targets = HashSet::new();
    let subobject = name.starts_with("db.property.")
        || name.starts_with("db.view.")
        || name.starts_with("db.template.")
        || name.starts_with("db.automation.");
    for key in [
        "id",
        "pageId",
        "rowId",
        "databaseId",
        "sectionId",
        "notebookId",
        "parentId",
        "targetPageId",
        "parent",
    ] {
        if key == "id" && subobject {
            continue;
        }
        if let Some(id) = params[key].as_str().filter(|id| !id.is_empty()) {
            targets.insert(id.to_string());
        }
    }
    // ids on schema operations are property/view ids, not vault items.
    if !subobject {
        if let Some(list) = params["ids"].as_array() {
            for id in list {
                targets.insert(id.as_str().ok_or_else(invalid)?.to_string());
            }
        }
    }
    if name == "bin.empty" {
        for i in s.items().filter(|i| visible(s, i) && s.in_bin(i.id())) {
            targets.insert(i.id().into());
        }
    }
    let bin = name.starts_with("bin.");
    for id in &targets {
        require(s, id, bin)?;
    }
    if bin && name != "bin.list" && targets.iter().any(|id| !s.in_bin(id)) {
        return Err(Error::NotFound);
    }
    if name == "sticky.close_all" {
        for i in s
            .items()
            .filter(|i| matches!(i,Item::Sticky(x) if x.open && x.trashed_at.is_none()))
        {
            targets.insert(i.id().into());
        }
    }
    if name == "activity.undo" {
        let (restore, _) = s.activity_undo_plan(&actor(p), &text(&params, "changeSetId"))?;
        targets.extend(s.ai_write_targets(&actor(p), &restore)?);
        // Discover every affected item before checking hidden notebooks, locks and leases.
        for e in s
            .activity_list(None, None, 100_000)?
            .iter()
            .filter(|e| e.change_set == text(&params, "changeSetId"))
        {
            for item in &e.items {
                if s.get(&item.id).is_some() {
                    targets.insert(item.id.clone());
                }
            }
        }
    }
    let revision_targets = targets.clone();
    let writes = spec.level != Required::Look
        && !matches!(
            name,
            "ui.open"
                | "ui.highlight"
                | "ui.notify"
                | "ui.wait_for_person"
                | "db.template.list"
                | "db.automation.list"
        );
    if writes {
        // Container operations and moves affect descendants too.
        let roots: Vec<_> = targets.iter().cloned().collect();
        if name.ends_with(".trash") || name.ends_with(".move") || name.starts_with("bin.") {
            for i in s.items() {
                if roots.iter().any(|r| s.is_within(i.id(), r)) {
                    targets.insert(i.id().into());
                }
            }
        }
        // Schema changes can rewrite every row, and back relations can rewrite a
        // second database. Include those items in the permission/confirm footprint.
        if name.starts_with("db.") || name.starts_with("bin.") {
            let mut roots: Vec<_> = targets.iter().cloned().collect();
            let mut scanned = HashSet::new();
            while let Some(root) = roots.pop() {
                let db_id = match s.get(&root) {
                    Some(Item::Database(d)) => Some(d.id.as_str()),
                    Some(Item::Row(r)) => Some(r.db.as_str()),
                    _ => None,
                };
                if let Some(db_id) = db_id {
                    if !scanned.insert(db_id.to_string()) {
                        continue;
                    }
                    targets.insert(db_id.to_string());
                    if let Some(Item::Database(d)) = s.get(db_id) {
                        for prop in &d.props {
                            if !prop.relation_db.is_empty()
                                && targets.insert(prop.relation_db.clone())
                            {
                                roots.push(prop.relation_db.clone());
                            }
                        }
                        let definitions = value(d)?;
                        fn databases(v: &Value, out: &mut Vec<String>) {
                            match v {
                                Value::Object(o) => {
                                    for (key, v) in o {
                                        if key == "database" {
                                            if let Some(id) = v.as_str().filter(|id| !id.is_empty())
                                            {
                                                out.push(id.into());
                                            }
                                        } else {
                                            databases(v, out);
                                        }
                                    }
                                }
                                Value::Array(a) => {
                                    for v in a {
                                        databases(v, out);
                                    }
                                }
                                _ => {}
                            }
                        }
                        let mut other = vec![];
                        databases(&definitions, &mut other);
                        for id in other {
                            if targets.insert(id.clone()) {
                                roots.push(id);
                            }
                        }
                    }
                    for i in s.items() {
                        if matches!(i,Item::Row(r) if r.db==db_id) {
                            targets.insert(i.id().into());
                        }
                    }
                }
            }
            fn referenced(s: &Session, v: &Value, targets: &mut HashSet<String>) {
                match v {
                    Value::String(id) if s.get(id).is_some() => {
                        targets.insert(id.clone());
                    }
                    Value::Array(a) => {
                        for v in a {
                            referenced(s, v, targets);
                        }
                    }
                    Value::Object(o) => {
                        for v in o.values() {
                            referenced(s, v, targets);
                        }
                    }
                    _ => {}
                }
            }
            for key in ["cells", "patch", "value", "template", "automation"] {
                referenced(s, &params[key], &mut targets);
            }
        }
        if targets.iter().any(|id| hidden(s, id)) {
            return Err(Error::NotFound);
        }
        let mut denied = None;
        for id in &targets {
            // Undo's planner already checks changed roots and their descendants.
            // Its footprint also contains destination ancestors, whose unrelated
            // descendants are not themselves being restored.
            let check = if name == "activity.undo" {
                s.coordination.check(&actor(p), |other| other == id)
            } else { s.check_write(&actor(p), id, None) };
            if let Err(error) = check {
                if error_code(&error) == "person_editing" {
                    denied = Some(error);
                    break;
                }
                denied = Some(error);
            }
        }
        if let Some(error) = denied {
            return Err(error);
        }
        for id in &targets {
            let mut cursor = Some(id.as_str());
            let mut seen = HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    return Err(invalid());
                }
                if matches!(s.get(id),Some(Item::Database(d)) if d.locked) {
                    return Err(invalid());
                }
                cursor = s.get(id).and_then(Session::parent_of);
            }
        }
        if let Some(rev) = expected {
            for id in &revision_targets {
                if s.rev(id) != rev {
                    return Err(code("item_changed"));
                }
            }
        }
    }
    // References in relations and automation definitions cannot point into hidden notebooks.
    fn references(s: &Session, v: &Value) -> bool {
        match v {
            Value::String(id) => s.get(id).is_some() && hidden(s, id),
            Value::Array(a) => a.iter().any(|v| references(s, v)),
            Value::Object(o) => o.values().any(|v| references(s, v)),
            _ => false,
        }
    }
    for key in ["cells", "patch", "value", "template", "automation"] {
        if references(s, &params[key]) {
            return Err(Error::NotFound);
        }
    }
    let mut count = targets.len().max(1);
    for key in ["blockIds", "ids"] {
        count = count.max(params[key].as_array().map_or(0, Vec::len));
    }
    if matches!(
        name,
        "page.write" | "db.row.write_page" | "page.append" | "page.insert"
    ) {
        images(s, &doc(&params)?)?;
    }
    let mut confirm = rules::confirmation(spec, count, a.big_change_confirm, a.big_change_items);
    // Buttons and triggered automations inherit the confirmation needed by their
    // effects. A delete still asks while the optional big-change switch is off.
    if writes && (name.starts_with("db.row.") || name == "db.button.press") {
        for id in &targets {
            if let Some(Item::Database(d)) = s.get(id) {
                if d.automations
                    .iter()
                    .filter(|a| a.enabled)
                    .any(|a| a.actions.iter().any(|a| a.kind == "trash"))
                    || d.props
                        .iter()
                        .any(|p| p.actions.iter().any(|a| a.kind == "trash"))
                {
                    confirm = rules::strongest(confirm, Confirm::Ask);
                }
            }
        }
    }
    let mut targets: Vec<_> = targets.into_iter().collect();
    targets.sort();
    let revs = targets.iter().map(|id| (id.clone(), s.rev(id))).collect();
    Ok(Prepared {
        spec,
        params,
        targets,
        revs,
        confirm,
        count,
    })
}

/// Strip hidden references from structured metadata, never rewrite note strings.
fn filter(s: &Session, mut v: Value) -> Value {
    match &mut v {
        Value::Array(a) => {
            a.retain(|v| {
                v.get("id")
                    .and_then(Value::as_str)
                    .is_none_or(|id| !hidden(s, id))
                    && v.as_str().is_none_or(|id| !hidden(s, id))
            });
            for v in a {
                *v = filter(s, v.take());
            }
        }
        Value::Object(o) => {
            for (key, v) in o.iter_mut() {
                if matches!(
                    key.as_str(),
                    "content" | "text" | "title" | "name" | "source" | "code" | "latex"
                ) {
                    continue;
                }
                if key == "cells" {
                    if let Some(raw) = v.as_str() {
                        if let Ok(cells) = serde_json::from_str::<Value>(raw) {
                            *v = json!(filter(s, cells).to_string());
                            continue;
                        }
                    }
                }
                if v.as_str().is_some_and(|id| hidden(s, id)) {
                    *v = Value::Null;
                } else {
                    *v = filter(s, v.take());
                }
            }
        }
        _ => {}
    }
    v
}
fn tree(s: &Session, a: &Actor) -> Result<Value> {
    Ok(filter(s, value(notes::list_notes_core(s, a)?)?))
}
fn context(s: &Session) -> Ctx<'_> {
    let (now, tz) = crate::database::clock::local_now();
    Ctx::new(s.items().filter(|i| live(s, i)), now, tz)
}
fn look_settings(s: &Session) -> Value {
    let x = s.settings();
    json!({"theme":x.theme,"accent":x.accent,"font":x.font,"fontSize":x.font_size,
        "editorWidth":x.editor_width,"density":x.density,"backdrop":x.backdrop,"stickyDefaultColor":x.sticky_default_color})
}
fn markdown(s: &Session, page: &crate::model::Page) -> Result<String> {
    use crate::export::{ExportDoc, Part};
    let doc = ExportDoc {
        title: page.title.clone(),
        missing_images: 0,
        parts: vec![Part::Page {
            level: 1,
            title: page.title.clone(),
            blocks: crate::export::doc::parse(&page.content, &|id| match s.get(id) {
                Some(Item::Page(p)) if !hidden(s, id) => Some(p.title.clone()),
                _ => None,
            }),
        }],
        images: Default::default(),
    };
    Ok(crate::export::markdown::render(&doc, "pictures").text)
}
fn doc(p: &Value) -> Result<Value> {
    let c = &p["content"];
    if p["format"] == "blocks" || c.is_object() || c.is_array() {
        let v = if c.is_array() {
            json!({"type":"doc","content":c})
        } else if let Some(s) = c.as_str() {
            serde_json::from_str(s).map_err(|_| invalid())?
        } else {
            c.clone()
        };
        blocks::validate(&v)?;
        return Ok(v);
    }
    use crate::importers::{self, source::Vfs, Control, Images};
    let vfs = Vfs::from_files(&[]).map_err(|_| invalid())?;
    let ctl = Control::new(std::time::Duration::from_secs(10), Default::default());
    let mut images = Images::default();
    let mut problems = vec![];
    let mut ctx = importers::Ctx {
        vfs: &vfs,
        ctl: &ctl,
        dir: String::new(),
        images: &mut images,
        problems: &mut problems,
        name: String::new(),
    };
    let parsed = importers::markdown::parse(c.as_str().ok_or_else(invalid)?, &mut ctx);
    let rendered = importers::doc::render_limited(
        &parsed.blocks,
        &|_| importers::doc::ImageOut::Text(String::new()),
        &|_| None,
        8 * 1024 * 1024,
    )
    .map_err(|_| code("too_big"))?;
    let v = serde_json::from_str(&rendered.content).map_err(|_| invalid())?;
    blocks::validate(&v)?;
    Ok(v)
}

fn images(s: &Session, doc: &Value) -> Result<()> {
    if let Some(marks) = doc["marks"].as_array() {
        for mark in marks.iter().filter(|m| m["type"] == "link") {
            let href = mark["attrs"]["href"].as_str().ok_or_else(invalid)?;
            let lower = href.to_ascii_lowercase();
            let internal = href.strip_prefix("notaya:page/").is_some_and(|rest| {
                let id = rest.split('#').next().unwrap_or("");
                !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            });
            if href.chars().any(char::is_control) || href.trim() != href
                || !(internal || ["https://", "http://", "mailto:"].iter().any(|scheme| lower.starts_with(scheme))) {
                return Err(invalid());
            }
        }
    }
    if doc["type"] == "image" {
        let src = doc["attrs"]["src"].as_str().ok_or_else(invalid)?;
        let id = crate::export::doc::attachment_id(src).ok_or_else(invalid)?;
        let (mime, _) = s.get_attachment(&id)?;
        if !matches!(
            mime.as_str(),
            "image/png" | "image/jpeg" | "image/gif" | "image/webp"
        ) {
            return Err(invalid());
        }
    }
    if let Some(children) = doc["content"].as_array() {
        for child in children {
            images(s, child)?;
        }
    }
    Ok(())
}

fn formula_test(q: &Value) -> Result<Value> {
    use crate::database::{engine::val_json, formula, value::Val};
    fn val(v: &Value) -> Val {
        match v {
            Value::Number(n) => Val::Num(n.as_f64().unwrap_or(0.)),
            Value::Bool(b) => Val::Bool(*b),
            Value::String(s) => Val::Str(s.clone()),
            Value::Array(a) => Val::List(a.iter().map(val).collect()),
            _ => Val::Null,
        }
    }
    struct Env<'a>(&'a Value);
    impl formula::Env for Env<'_> {
        fn prop(&self, name: &str) -> std::result::Result<Val, String> {
            Ok(val(&self.0[name]))
        }
        fn row_id(&self) -> String {
            String::new()
        }
        fn now(&self) -> i64 {
            crate::database::clock::local_now().0
        }
    }
    let expression = formula::parse(&text(q, "formula")).map_err(|_| invalid())?;
    let rows = q["rows"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![json!({})]);
    if rows.len() > 200 {
        return Err(code("too_big"));
    }
    rows.iter()
        .map(|r| {
            formula::eval(&expression, &Env(r))
                .map(|v| val_json(&v))
                .map_err(|_| invalid())
        })
        .collect::<Result<Vec<_>>>()
        .and_then(value)
}

fn definition(s: &mut Session, a: &Actor, name: &str, q: &Value) -> Result<Value> {
    let mut d = db::database(s, &text(q, "databaseId"))?;
    let id = text(q, "id");
    match name {
        "db.template.list" => return value(&d.templates),
        "db.automation.list" => return value(&d.automations),
        "db.template.create" | "db.template.update" => {
            let mut t: Template = get(q, "template")?;
            // Template page bodies go through the same block validator as page.write.
            let content = blocks::parse(&t.content)?;
            images(s, &content)?;
            if name.ends_with("create") {
                t.id = crate::model::new_id();
                d.templates.push(t);
            } else {
                let old = d
                    .templates
                    .iter_mut()
                    .find(|old| old.id == t.id)
                    .ok_or(Error::NotFound)?;
                *old = t;
            }
        }
        "db.template.trash" => {
            d.templates.retain(|t| t.id != id);
        }
        "db.automation.create" | "db.automation.update" => {
            let mut v: Automation = get(q, "automation")?;
            if name.ends_with("create") {
                v.id = crate::model::new_id();
                d.automations.push(v);
            } else {
                let old = d
                    .automations
                    .iter_mut()
                    .find(|old| old.id == v.id)
                    .ok_or(Error::NotFound)?;
                *old = v;
            }
        }
        "db.automation.pause" => {
            d.automations
                .iter_mut()
                .find(|v| v.id == id)
                .ok_or(Error::NotFound)?
                .enabled = false;
        }
        "db.automation.trash" => {
            d.automations.retain(|v| v.id != id);
        }
        _ => return Err(invalid()),
    }
    value(saves::save_database_core(
        s,
        a,
        serde_json::to_string(&d).map_err(|_| invalid())?,
    )?)
}

pub fn execute(s: &mut Session, p: &Pairing, r: &Prepared, paths: &Paths) -> Result<Value> {
    execute_approved(s, p, r, paths, None)
}
fn native_delete(s: &Session, r: &Prepared) -> Option<Delete> {
    match r.spec.name {
        "bin.delete_forever" => Some(Delete::One(text(&r.params, "id"))),
        "bin.empty" => Some(Delete::Selected(r.targets.iter().filter(|id| s.in_bin(id)).cloned().collect())),
        _ => None,
    }
}
fn execute_approved(s: &mut Session, p: &Pairing, r: &Prepared, paths: &Paths, approval: Option<&mut Approval>) -> Result<Value> {
    if r.spec.level == Required::Look { return execute_inner(s, p, r, paths, approval); }
    // VACUUM INTO cannot run inside a transaction. Make the existing safety
    // copy first, under the same session lock, before any action writes.
    let backup = matches!(r.spec.name,"notebook.trash" | "section.trash")
        || (r.spec.name=="db.create" && !s.items().any(|i| matches!(i,Item::Database(_))));
    if backup { crate::backup::make_safety_backup(paths,s).map_err(|e|crate::backup::safety_failure(&e))?; }
    s.atomic_action(backup, |s| execute_inner(s, p, r, paths, approval))
}

fn execute_inner(s: &mut Session, p: &Pairing, r: &Prepared, paths: &Paths, approval: Option<&mut Approval>) -> Result<Value> {
    let a = actor(p);
    let q = &r.params;
    let id = text(q, "id");
    let name = r.spec.name;
    let before: HashSet<_> = s.items().map(|i| i.id().to_string()).collect();
    let answer=s.run_action(&a,name,|s| {
        macro_rules! out {($e:expr)=>{value($e?)}}
        match name {
            "ai.hello"|"about.get"=>Ok(json!({"version":env!("CARGO_PKG_VERSION"),"locked":false})),
            "ai.permissions.get"=>Ok(json!({"level":p.level,"actions":ACTIONS.iter().filter(|a|a.name!="batch.run" && rules::allowed(p.level,a)).map(|a|a.name).collect::<Vec<_>>()})),
            "ai.limits.get"=>Ok(json!({"requestBytes":8*1024*1024,"responseBytes":32*1024*1024,"batchActions":0,"rowsPerPage":200})),
            "settings.get"=>Ok(look_settings(s)),
            "ui.where"=>{let place=s.settings().last_place;value(place.filter(|p|s.get(&p.id).is_some_and(|i|live(s,i))))},
            "ui.open"|"ui.highlight"|"ui.notify"|"ui.wait_for_person"|"calc.open"=>Ok(json!({"action":name,"params":q})),
            "db.formula.test"=>formula_test(q),
            "calc.evaluate"=>{let expr=text(q,"expression");let mut result=formula_test(&json!({"formula":expr}))?;
                Ok(json!({"value":result.as_array_mut().ok_or_else(invalid)?.remove(0)}))},
            "tree.list"=>tree(s,&a),
            "notebook.list"|"section.list"|"db.list"|"sticky.list"|"page.recent"|"page.favorites"=> {
                let mut items:Vec<_>=s.items().filter(|i|live(s,i)).filter(|i|match name {
                    "notebook.list"=>matches!(i,Item::Notebook(_)),"section.list"=>matches!(i,Item::Section(_)),
                    "db.list"=>matches!(i,Item::Database(_)),"sticky.list"=>matches!(i,Item::Sticky(_)),
                    "page.favorites"=>matches!(i,Item::Page(p) if p.favorite),_=>matches!(i,Item::Page(_))}).collect();
                items.sort_by_key(|i|i.id());value(items)
            }
            "notebook.get"|"section.get"|"db.get"|"sticky.get"=>value(s.get(&id).ok_or(Error::NotFound)?),
            "page.get"|"page.outline"|"block.list"=>{
                // Legacy ID migration is a write: only an Edit/Full pairing with
                // current lock/lease/hidden authorization may persist it. Other
                // reads preserve legacy content and revisions exactly.
                let may_migrate = p.level != rules::Level::Look && s.get(&id).is_some_and(|item|
                    s.ai_write_targets(&a,std::slice::from_ref(item)).is_ok());
                let page=if may_migrate {blocks::read(s,&a,&id)?} else {notes::get_page_core(s,&a,id.clone())?};
                let content=blocks::parse(&page.content)?;
                if name=="page.outline" {fn headings(v:&Value,out:&mut Vec<Value>){if v["type"]=="heading"{out.push(v.clone());}if let Some(c)=v["content"].as_array(){for v in c{headings(v,out);}}}let mut h=vec![];headings(&content,&mut h);return Ok(json!(h));}
                if name=="block.list"{return Ok(content["content"].clone());}
                Ok(json!({"id":page.id,"title":page.title,"format":q["format"].as_str().unwrap_or("markdown"),
                    "content":if q["format"]=="blocks"{content}else{json!(markdown(s,&page)?)},"rev":s.rev(&id)}))
            }
            "db.query"=>{
                let d=db::database(s,&text(q,"databaseId"))?;
                let view=q["viewId"].as_str().and_then(|id|d.view(id)).or_else(||d.views.first()).ok_or_else(invalid)?;
                let mut result=context(s).run_view(&d,view);
                let offset=q["offset"].as_u64().unwrap_or(0) as usize;let limit=q["limit"].as_u64().unwrap_or(200) as usize;
                result.rows=result.rows.into_iter().skip(offset).take(limit).collect();value(result)
            }
            "db.row.get"=>{let row=db::row(s,&id)?;let d=db::database(s,&row.db)?;value(context(s).row_out_pub(&d,&row))}
            "db.row_titles"=>out!(db::row_titles_core(s,&a,ids(q,"ids")?)),
            "db.chart.data"=>{let d=db::database(s,&text(q,"databaseId"))?;let v=d.view(&text(q,"viewId")).ok_or(Error::NotFound)?;value(crate::database::chart::chart_data(&context(s),&d,v))}
            "db.reminders.due"=>value(db::due_reminders(&context(s),&Default::default())),
            "db.file.list"=>{let row=db::row(s,&text(q,"rowId"))?;let cells:Value=serde_json::from_str(&row.cells).map_err(|_|invalid())?;
                Ok(json!(cells[&text(q,"propId")].as_array().map(|files| files.iter().map(|f|json!({"name":f["name"],"size":f["size"]})).collect::<Vec<_>>()).unwrap_or_default()))}
            "search.all"=>{Ok(filter(s,value(commands::tools::search_core(s,&a,text(q,"query"))?)?))}
            "calc.history"=>out!(calculator::calc_history_core(s,&a)),
            "backup.list"=>out!(commands::backup::list_backups_core(paths,&a)),
            "bin.list"=>{let items=notes::list_bin_core(s,&a)?;Ok(filter(s,value(items)?))}
            "find.by_source"=>{let tag:SourceTag=get(q,"source")?;value(s.items().filter(|i|live(s,i)&&source(i)==Some(&tag)).collect::<Vec<_>>())}
            "history.list"=>out!(s.history_list(&id)),"history.get"=>out!(s.history_get(&id,&text(q,"versionId"))),
            "history.restore"=>out!(s.history_restore(&a,&id,&text(q,"versionId"))),
            "activity.verify"=>out!(s.activity_verify()),
            "activity.list"=>{let entries=s.activity_list(q["who"].as_str(),q["before"].as_u64(),q["limit"].as_u64().unwrap_or(100) as usize)?;
                value(entries.into_iter().filter(|e|e.items.iter().all(|i|s.get(&i.id).is_some()&&!hidden(s,&i.id))).collect::<Vec<_>>())}
            "activity.undo"=>out!(s.activity_undo(&a,&text(q,"changeSetId"))),
            "ai.claim"=>out!(s.claim(&a,&ids(q,"ids")?)),
            "ai.release"=>{s.coordination.release(&a,Some(&ids(q,"ids")?));Ok(Value::Null)}
            "notebook.create"=>out!(notes::create_notebook_core(s,&a,text(q,"name"),text(q,"color"),text(q,"icon"))),
            "notebook.update"=>{let n=commands::notebook(s,&id)?;out!(notes::update_notebook_core(s,&a,id.clone(),kept(q,"name",&n.name),kept(q,"color",&n.color),kept(q,"icon",&n.icon)))},
            "section.create"=>out!(notes::create_section_core(s,&a,text(q,"notebookId"),text(q,"name"),text(q,"color"))),
            "section.update"=>{let n=commands::section(s,&id)?;out!(notes::update_section_core(s,&a,id.clone(),kept(q,"name",&n.name),kept(q,"color",&n.color)))},
            "section.move"=>{let mut section=commands::section(s,&id)?;commands::notebook(s,&text(q,"notebookId"))?;section.notebook_id=text(q,"notebookId");out!(s.put(Item::Section(section)))}
            "notebook.reorder"|"section.reorder"|"page.reorder"|"db.reorder"=>out!(notes::reorder_core(s,&a,ids(q,"ids")?)),
            "notebook.trash"=>out!(notes::delete_notebook_core(s,&a,paths,id.clone())),
            "section.trash"=>out!(notes::delete_section_core(s,&a,paths,id.clone())),
            "sticky.trash"=>out!(stickies::delete_sticky_core(s,&a,id.clone())),
            "page.create"=>out!(notes::create_page_core(s,&a,text(q,"sectionId"),text(q,"title"))),
            "page.create_sub"=>out!(notes::create_subpage_core(s,&a,text(q,"parentId"),text(q,"title"))),
            "page.write"|"db.row.write_page"=>{let old=notes::get_page_core(s,&a,id.clone())?;let content=doc(q)?;images(s,&content)?;out!(blocks::write(s,&a,&id,old.title.clone(),content))}
            "page.rename"=>{let old=notes::get_page_core(s,&a,id.clone())?;out!(notes::save_page_core(s,&a,id.clone(),text(q,"title"),old.content.clone(),old.text.clone()))}
            "page.set_look"=>{let page=notes::get_page_core(s,&a,id.clone())?;out!(notes::set_page_look_core(s,&a,id.clone(),kept(q,"icon",&page.icon),kept(q,"cover",&page.cover)))},
            "page.favorite"=>out!(notes::set_page_favorite_core(s,&a,id.clone(),boolean(q,"favorite"))),
            "page.move"=>{
                let parent=text(q,"parentId");let section=if parent.is_empty(){text(q,"sectionId")}else{notes::get_page_core(s,&a,parent.clone())?.section_id.clone()};
                if !parent.is_empty() && s.is_within(&parent,&id) {return Err(invalid());}
                notes::move_page_core(s,&a,id.clone(),section)?;
                let mut page=commands::page(s,&id)?;page.parent=parent;out!(s.put(Item::Page(page)))
            }
            "page.trash"|"db.trash"|"db.row.trash"=>out!(notes::trash_page_core(s,&a,id.clone())),
            "bin.restore"=>out!(notes::restore_bin_item_core(s,&a,id.clone(),false)),
            "bin.delete_forever"=>out!(notes::delete_page_core(s,&a,id.clone(),approval.ok_or_else(native_confirm::cancelled)?)),
            "bin.empty"=>{
                // The authorization footprint also includes live relation and
                // automation destinations. Only items already in the bin are
                // deletion roots; the shared core keeps the permanent-delete guard.
                let request=native_delete(s,r).ok_or_else(invalid)?;
                notes::delete_selected_core(s,&a,request,approval.ok_or_else(native_confirm::cancelled)?)?;
                Ok(Value::Null)
            }
            "db.create"=>out!(db::create_database_core(s,&a,paths,text(q,"sectionId"),text(q,"parent"),boolean(q,"inline"),text(q,"title"),text(q,"starterKind"))),
            "db.rename"|"db.set_look"=>{let d=db::database(s,&id)?;out!(db::set_database_look_core(s,&a,id.clone(),q["title"].as_str().unwrap_or(&d.title).into(),q["icon"].as_str().unwrap_or(&d.icon).into(),q["favorite"].as_bool().unwrap_or(d.favorite),d.locked))}
            "db.move"=>out!(db::move_database_core(s,&a,id.clone(),text(q,"sectionId"))),
            "db.property.add"=>out!(saves::add_property(s,&a,&text(q,"databaseId"),get(q,"value")?)),
            "db.property.update"=>out!(saves::update_property(s,&a,&text(q,"databaseId"),&id,&q["value"])),
            "db.property.reorder"=>out!(saves::reorder_properties(s,&a,&text(q,"databaseId"),&ids(q,"ids")?)),
            "db.property.trash"=>out!(saves::remove_property(s,&a,&text(q,"databaseId"),&id)),
            "db.property.hide"=>out!(saves::hide_property(s,&a,&text(q,"databaseId"),&id,&text(&q["value"],"viewId"),boolean(&q["value"],"hidden"))),
            "db.property.change_kind"=>{
                let d=db::database(s,&text(q,"databaseId"))?;
                let old=d.prop(&id).ok_or(Error::NotFound)?;
                if old.kind==PropKind::Title{return Err(invalid());}
                let mut prop=old.clone();prop.kind=get(q,"kind")?;
                // Validate the replacement before any destructive cleanup.
                let mut check=d.clone();*check.props.iter_mut().find(|p|p.id==id).ok_or(Error::NotFound)?=prop.clone();check.sanitize()?;
                saves::remove_property(s,&a,&d.id,&id)?;
                out!(saves::add_property(s,&a,&d.id,prop))
            }
            "db.view.add"=>out!(saves::add_view(s,&a,&text(q,"databaseId"),get(q,"value")?)),
            "db.view.update"=>out!(saves::update_view(s,&a,&text(q,"databaseId"),&id,&q["value"])),
            "db.view.rename"=>out!(saves::rename_view(s,&a,&text(q,"databaseId"),&id,text(&q["value"],"name"))),
            "db.view.reorder"=>out!(saves::reorder_views(s,&a,&text(q,"databaseId"),&ids(q,"ids")?)),
            "db.view.trash"=>out!(saves::remove_view(s,&a,&text(q,"databaseId"),&id)),
            "db.row.create"=>out!(db::create_row_core(s,&a,text(q,"databaseId"),text(q,"title"),q.get("cells").unwrap_or(&json!({})).to_string(),text(q,"templateId"))),
            "db.row.set"=>out!(db::set_cells_core(s,&a,text(q,"rowId"),q["title"].as_str().map(String::from),q.get("patch").unwrap_or(&json!({})).to_string())),
            "db.row.duplicate"=>out!(db::duplicate_row_core(s,&a,text(q,"rowId"))),
            "db.row.reorder"=>out!(db::reorder_rows_core(s,&a,ids(q,"ids")?)),
            "db.sub_items.enable"=>out!(db::enable_sub_items_core(s,&a,id.clone())),
            "db.dependencies.enable"=>out!(db::enable_dependencies_core(s,&a,id.clone())),
            "db.button.press"=>out!(db::press_button_core(s,&a,text(q,"rowId"),text(q,"propId"))),
            "db.file.remove"=>out!(commands::database_io::remove_file_core(s,&a,text(q,"rowId"),text(q,"propId"),text(q,"attachmentId"))),
            "db.template.list"|"db.template.create"|"db.template.update"|"db.template.trash"|
            "db.automation.list"|"db.automation.create"|"db.automation.update"|"db.automation.pause"|"db.automation.trash"=>definition(s,&a,name,q),
            "sticky.create"=>{let (sticky,_)=stickies::create_sticky_core(s,&a,boolean(q,"open"))?;value(sticky)},
            "sticky.write"=>{let content=blocks::parse(&text(q,"content"))?;images(s,&content)?;out!(stickies::save_sticky_core(s,&a,id.clone(),text(q,"content"),blocks::plain_text(&content)))},
            "sticky.style"=>{let st=commands::sticky(s,&id)?;out!(stickies::set_sticky_style_core(s,&a,id.clone(),q["color"].as_str().unwrap_or(&st.color).into(),q["onTop"].as_bool().unwrap_or(st.on_top)))},
            "sticky.place_on_desktop"=>out!(stickies::set_sticky_geometry_core(s,&a,id.clone(),get(q,"x")?,get(q,"y")?,get(q,"w")?,get(q,"h")?)),
            "sticky.open"=>out!(stickies::open_sticky_core(s,&a,id.clone())),"sticky.close"=>out!(stickies::close_sticky_core(s,&a,id.clone())),
            "sticky.close_all"=>out!(stickies::hide_stickies_core(s,&a)),
            "calc.add"=>out!(calculator::add_calc_entry_core(s,&a,text(q,"expr"),text(q,"result"))),
            "calc.clear"=>out!(calculator::clear_calc_history_core(s,&a)),
            "settings.set"=>{let mut settings=value(s.settings())?;let o=settings.as_object_mut().ok_or_else(invalid)?;
                for (k,v) in q.as_object().ok_or_else(invalid)? {if !look_settings(s).as_object().unwrap().contains_key(k){return Err(invalid());}o.insert(k.clone(),v.clone());}
                commands::vault::save_settings_core(s,&a,serde_json::from_value(settings).map_err(|_|invalid())?)?;Ok(look_settings(s))}
            // Block operations are implemented by C4's schema-checked pure functions.
            "page.append"|"page.insert"|"page.replace_block"|"page.delete_blocks"|"page.move_blocks"|
            "block.insert"|"block.update"|"block.turn_into"|"block.set_direction"|"picture.add"=>block_edit(s,&a,name,q),
            _=>Err(invalid()),
        }
    })?;
    if !q["source"].is_null() && name != "find.by_source" {
        let root = if !id.is_empty() {
            Some(id)
        } else if name == "db.row.set" {
            Some(text(q, "rowId"))
        } else {
            s.items()
                .find(|i| {
                    !before.contains(i.id())
                        && match name {
                            "notebook.create" => matches!(i, Item::Notebook(_)),
                            "section.create" => matches!(i, Item::Section(_)),
                            "db.create" => matches!(i, Item::Database(_)),
                            "db.row.create" => matches!(i, Item::Row(_)),
                            _ => matches!(i, Item::Page(_)),
                        }
                })
                .map(|i| i.id().into())
        };
        if let Some(root) = root {
            set_source(s, &root, &q["source"])?;
        }
    }
    Ok(filter(s, answer))
}

fn block_edit(s: &mut Session, a: &Actor, name: &str, q: &Value) -> Result<Value> {
    let id = q["pageId"]
        .as_str()
        .or(q["id"].as_str())
        .ok_or_else(invalid)?;
    let page = blocks::read(s, a, id)?;
    let mut content = blocks::parse(&page.content)?;
    let block = text(q, "blockId");
    let selected = if q["blockIds"].is_array() {
        ids(q, "blockIds")?
    } else {
        vec![block.clone()]
    };
    let pos = if let Some(id) = q["before"].as_str() {
        blocks::Position::Before(id)
    } else if let Some(id) = q["after"].as_str() {
        blocks::Position::After(id)
    } else {
        blocks::Position::End
    };
    let nodes = || -> Result<Vec<Value>> {
        let v = if q["content"].get("type").is_some_and(|t| t != "doc") {
            json!({"type":"doc","content":[q["content"].clone()]})
        } else {
            doc(q)?
        };
        blocks::validate(&v)?;
        images(s, &v)?;
        Ok(v["content"].as_array().ok_or_else(invalid)?.clone())
    };
    match name {
        "page.append" | "page.insert" | "block.insert" => {
            blocks::insert(&mut content, pos, nodes()?)?
        }
        "page.replace_block" | "block.update" => {
            let nodes = nodes()?;
            if nodes.len() != 1 {
                return Err(invalid());
            }
            blocks::replace(
                &mut content,
                selected.first().ok_or_else(invalid)?,
                nodes.into_iter().next().ok_or_else(invalid)?,
            )?;
        }
        "page.delete_blocks" => blocks::delete(&mut content, &selected)?,
        "page.move_blocks" => {
            let target = text(q, "targetPageId");
            if target.is_empty() || target == id {
                blocks::move_blocks(&mut content, &selected, pos)?;
            } else {
                let other = blocks::read(s, a, &target)?;
                let mut to = blocks::parse(&other.content)?;
                blocks::move_between(&mut content, &mut to, &selected, pos)?;
                blocks::write(s, a, &target, other.title.clone(), to)?;
            }
        }
        "block.set_direction" => {
            blocks::set_direction(&mut content, &block, &text(q, "direction"))?
        }
        "block.turn_into" => {
            fn change(v: &mut Value, id: &str, kind: &str) -> bool {
                if v["attrs"]["bid"] == id {
                    v["type"] = json!(kind);
                    return true;
                }
                if let Some(c) = v["content"].as_array_mut() {
                    for v in c {
                        if change(v, id, kind) {
                            return true;
                        }
                    }
                }
                false
            }
            if !change(&mut content, &block, &text(q, "kind")) {
                return Err(Error::NotFound);
            }
        }
        "picture.add" => {
            let aid = text(q, "attachmentId");
            let (mime, _) = s.get_attachment(&aid)?;
            if !matches!(
                mime.as_str(),
                "image/png" | "image/jpeg" | "image/gif" | "image/webp"
            ) {
                return Err(invalid());
            }
            blocks::insert(
                &mut content,
                pos,
                vec![
                    json!({"type":"image","attrs":{"src":format!("http://clatt.localhost/{aid}")}}),
                ],
            )?;
        }
        _ => return Err(invalid()),
    }
    value(blocks::write(s, a, id, page.title.clone(), content)?)
}

pub fn call(
    state: &AppState,
    p: &Pairing,
    name: &str,
    params: Value,
    preview: bool,
    cancelled: impl Fn() -> bool,
    notify: impl Fn(),
) -> Result<Value> {
    let epoch = state.ai.epoch.load(Ordering::SeqCst);
    let mut r = state.with(|s| prepare(s, p, name, params))?;
    if name == "ui.wait_for_person" {
        r.confirm = Confirm::Ask;
    }
    if preview {
        return Ok(
            json!({"preview":true,"action":name,"items":r.targets,"count":r.count,"confirmation":format!("{:?}",r.confirm)}),
        );
    }
    if r.confirm != Confirm::None {
        notify();
        let details=state.with(|s|Ok(json!({"client":p.name,"action":name,"items":r.targets.iter().filter_map(|id|s.get(id)).map(|i|match i{
            Item::Page(p)=>json!({"id":p.id,"title":p.title}),Item::Notebook(n)=>json!({"id":n.id,"title":n.name}),Item::Section(n)=>json!({"id":n.id,"title":n.name}),_=>json!({"id":i.id()})}).collect::<Vec<_>>()})))?;
        if !state.ai.confirms.request(
            "confirm",
            details,
            r.confirm,
            |v| state.ai.emit("ai-confirm", v),
            || cancelled() || state.ai.epoch.load(Ordering::SeqCst) != epoch,
        ) {
            return Err(code("not_confirmed"));
        }
    }
    // Rare permanent deletes get a second, native gate after the existing
    // in-window Strong confirmation. Pipe workers already run off the event loop.
    let target = state.with(|s| native_delete(s, &r).map(|request| native_confirm::delete_target(s, epoch, request)).transpose())?;
    let mut approval = target.map(|target| state.ai.native.ask(target)).transpose()?;
    let answer = state.with_actor(&actor(p), |s| {
        if cancelled() || state.ai.epoch.load(Ordering::SeqCst) != epoch {
            return Err(code("stopped"));
        }
        let again = prepare(s, p, name, r.params.clone())?;
        if again.revs != r.revs {
            return Err(code("item_changed"));
        }
        let current = s
            .settings()
            .ai
            .pairings
            .into_iter()
            .find(|current| current.id == p.id)
            .ok_or_else(|| code("not_paired"))?;
        let answer = execute_approved(s, &current, &again, &state.paths, approval.as_mut())?;
        if again.spec.level == Required::Look {
            state.ai.read(&current);
        }
        Ok(answer)
    })?;
    state.ai.emit(
        "ai-activity",
        json!({"pairingId":p.id,"name":p.name,"at":crate::model::now_ms(),"items":r.targets}),
    );
    if matches!(
        name,
        "ui.open"
            | "ui.highlight"
            | "ui.notify"
            | "calc.open"
            | "sticky.open"
            | "sticky.close"
            | "sticky.close_all"
            | "sticky.style"
            | "sticky.place_on_desktop"
    ) {
        state
            .ai
            .emit("ai-ui", json!({"action":name,"params":r.params}));
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (AppState, Pairing, String, String) {
        let root = std::path::PathBuf::from(
            std::env::var_os("NOTAYA_HOME").expect("throwaway NOTAYA_HOME required"),
        )
        .join(crate::model::new_id());
        let paths = Paths::at(&root);
        paths.ensure_dirs().unwrap();
        let mut s = Session::create_open(&paths.vault).unwrap();
        notes::create_notebook_core(
            &mut s,
            &Actor::Person,
            "Visible".into(),
            String::new(),
            String::new(),
        )
        .unwrap();
        let section = s
            .items()
            .find_map(|i| {
                if let Item::Section(x) = i {
                    Some(x.id.clone())
                } else {
                    None
                }
            })
            .unwrap();
        let page =
            notes::create_page_core(&mut s, &Actor::Person, section.clone(), "old".into()).unwrap();
        let (mut p, _) = super::super::pairing::create("Codex".into());
        p.level = rules::Level::Full;
        let mut settings = s.settings();
        settings.ai.enabled = true;
        settings.ai.pairings.push(p.clone());
        s.put(Item::Settings(settings)).unwrap();
        let state = AppState::new(paths);
        state.set_session(s);
        (state, p, section, page.id.clone())
    }

    fn snapshot(s: &Session) -> Value {
        let items: BTreeMap<_, _> = s.items().map(|i| (i.id(), value(i).unwrap())).collect();
        json!({"items": items, "revs": s.revs, "activity":s.activity_list(None,None,100_000).unwrap()})
    }
    fn make_db(s: &mut Session, paths: &Paths, section: &str) -> String {
        let created = value(db::create_database_core(s, &Actor::Person, paths, section.into(),
            String::new(), false, "Test database".into(), String::new()).unwrap()).unwrap();
        created["database"]["id"].as_str().unwrap().into()
    }
    fn make_row(s: &mut Session, id: &str) -> String {
        let created = value(db::create_row_core(s, &Actor::Person, id.into(), "Test row".into(), "{}".into(), String::new()).unwrap()).unwrap();
        created["row"]["id"].as_str().unwrap().into()
    }

    #[test]
    fn formula_requests_are_bounded_in_disposable_process() {
        const CHILD: &str = "NOTAYA_C5D_FORMULA_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "ai::dispatch::tests::formula_requests_are_bounded_in_disposable_process", "--nocapture"])
                .env(CHILD, "1").output().unwrap();
            assert!(output.status.success(), "child failed: {} {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            return;
        }
        let (state, p, _, _) = fixture();
        state.with_mut(|s| {
            let mut settings = s.settings(); settings.ai.pairings[0].level = rules::Level::Look;
            s.put(Item::Settings(settings))
        }).unwrap();
        let attacks = [format!("{}1", "+".repeat(100_000)), format!("{}1", "-".repeat(1000)),
            format!("{}1", "+".repeat(1000)), format!("{}1", "1^".repeat(1000)),
            format!("{}1 ?", "1+".repeat(1000)), format!("1{} ?", ".abs()".repeat(500)),
            format!("{}1", "1+".repeat(5000)), format!("[{}1]", "1,".repeat(3000)),
            format!("\"{}\"", "x".repeat(65536)), " ".repeat(65537)];
        for _ in 0..32 {
            for expression in &attacks {
                for (action, params) in [("calc.evaluate", json!({"expression":expression})),
                    ("db.formula.test", json!({"formula":expression}))] {
                    let error = call(&state, &p, action, params, false, ||false, ||{}).unwrap_err();
                    assert_eq!(error_code(&error), "invalid");
                }
            }
        }
        assert_eq!(call(&state,&p,"calc.evaluate",json!({"expression":"-2^2+10"}),false,||false,||{}).unwrap()["value"].as_f64(), Some(6.0));
    }

    #[test]
    fn failed_property_conversion_rolls_back_cells_journals_revisions_and_disk() {
        let (state, p, section, _) = fixture();
        state.with_actor(&actor(&p), |s| {
            let id = make_db(s, &state.paths, &section);
            saves::add_property(s,&Actor::Person,&id,Property{id:"text".into(),name:"Text".into(),kind:PropKind::Text,..Default::default()})?;
            let row = make_row(s, &id);
            db::set_cells_core(s,&Actor::Person,row.clone(),None,json!({"text":"keep exactly"}).to_string())?;
            s.drain_changes();
            let before = snapshot(s);
            let disk_before = value(Session::unlock_open(&state.paths.vault)?.items().collect::<Vec<_>>())?;
            let r = prepare(s,&p,"db.property.change_kind",json!({"databaseId":id,"id":"text","kind":"relation"}))?;
            assert!(execute(s,&p,&r,&state.paths).is_err());
            assert_eq!(snapshot(s),before);
            assert!(s.drain_changes().is_none());
            let reopened = Session::unlock_open(&state.paths.vault)?;
            let disk_after = value(reopened.items().collect::<Vec<_>>())?;
            fn ordered(v: Value) -> BTreeMap<String,Value> { v.as_array().unwrap().iter().map(|i|(i["id"].as_str().unwrap_or("settings").into(),i.clone())).collect() }
            assert_eq!(ordered(disk_after),ordered(disk_before));
            assert_eq!(serde_json::from_str::<Value>(&db::row(s,&row)?.cells).unwrap()["text"], "keep exactly");
            Ok(())
        }).unwrap();
    }

    #[test]
    fn row_and_triggered_automation_failure_roll_back_together() {
        let (state,p,section,_) = fixture();
        state.with_actor(&actor(&p), |s| {
            let id = make_db(s,&state.paths,&section);
            let row = make_row(s,&id);
            let mut d = db::database(s,&id)?;
            d.automations.push(Automation{id:"bad".into(),name:"Bad".into(),enabled:true,
                trigger:Trigger{kind:"propChanged".into(),..Default::default()},
                actions:vec![Action{kind:"setProp".into(),prop:"missing".into(),value:"x".into(),..Default::default()}],..Default::default()});
            s.run_as(&Actor::Person,|s|s.put(Item::Database(d)))?;
            s.drain_changes(); let before=snapshot(s);
            let r=prepare(s,&p,"db.row.set",json!({"rowId":row,"title":"must roll back"}))?;
            assert!(execute(s,&p,&r,&state.paths).is_err());
            assert_eq!(snapshot(s),before); assert!(s.drain_changes().is_none());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn bin_empty_late_failure_rolls_back_earlier_deletes() {
        let (state,p,section,page)=fixture();
        state.with_actor(&actor(&p),|s| {
            let second=notes::create_page_core(s,&Actor::Person,section,"Second".into())?.id.clone();
            notes::trash_page_core(s,&Actor::Person,page.clone())?;
            notes::trash_page_core(s,&Actor::Person,second.clone())?;
            let mut r=prepare(s,&p,"bin.empty",json!({}))?;
            r.targets=vec![page.clone(),second.clone()];
            // Simulate a late shared-core failure after the first deletion.
            s.coordination.editing(&second,"main",true);
            s.drain_changes(); let before=snapshot(s);
            let mut approval=native_confirm::test_delete(s,native_delete(s,&r).unwrap());
            assert!(execute_approved(s,&p,&r,&state.paths,Some(&mut approval)).is_err());
            assert_eq!(snapshot(s),before); assert!(s.get(&page).is_some());
            assert!(s.drain_changes().is_none());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn bin_empty_deletes_only_binned_items_not_live_relation_targets() {
        let (state,p,section,_) = fixture();
        state.with_actor(&actor(&p), |s| {
            let binned = make_db(s,&state.paths,&section);
            let live = make_db(s,&state.paths,&section);
            let live_row = make_row(s,&live);
            saves::add_property(s,&Actor::Person,&binned,Property {
                id:"relation".into(), name:"Related".into(), kind:PropKind::Relation,
                relation_db:live.clone(), ..Default::default()
            })?;
            notes::trash_page_core(s,&Actor::Person,binned.clone())?;
            let r = prepare(s,&p,"bin.empty",json!({}))?;
            assert!(r.targets.contains(&live) && r.targets.contains(&live_row),
                "live relation destinations still require authorization");
            let mut approval=native_confirm::test_delete(s,native_delete(s,&r).unwrap());
            execute_approved(s,&p,&r,&state.paths,Some(&mut approval))?;
            assert!(s.get(&binned).is_none());
            assert!(s.get(&live).is_some() && s.get(&live_row).is_some());
            Ok(())
        }).unwrap();
    }

    #[test]
    fn atomic_action_panic_restores_cache_leases_and_journals_and_can_retry() {
        let (state,p,_,page)=fixture();
        state.with_actor(&actor(&p),|s| {
            s.drain_changes();let before=snapshot(s);
            let panicked=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _:Result<()> = s.atomic_action(false,|s| {
                    s.run_action(&actor(&p),"page.rename",|s|{
                        let mut item=commands::page(s,&page)?;item.title="lost".into();s.put(Item::Page(item))
                    })?;
                    panic!("injected action panic")
                });
            }));
            assert!(panicked.is_err());assert_eq!(snapshot(s),before);assert!(s.drain_changes().is_none());
            let other=Actor::Ai{pairing_id:"other".into(),label:"Other".into()};
            s.check_write(&other,&page,None)?; // No implicit claim from the aborted write.
            let r=prepare(s,&p,"page.rename",json!({"id":page,"title":"retry"}))?;
            execute(s,&p,&r,&state.paths)?;assert_eq!(commands::page(s,&page)?.title,"retry");
            Ok(())
        }).unwrap();
    }

    #[test]
    fn transaction_keeps_required_safety_copies_for_database_and_container_actions() {
        let (state,p,section,_) = fixture();
        call(&state,&p,"db.create",json!({"sectionId":section,"title":"First"}),false,||false,||{}).unwrap();
        let backups=std::fs::read_dir(&state.paths.backups).unwrap().count();assert!(backups>0);
        state.with_actor(&actor(&p),|s|{
            let r=prepare(s,&p,"section.trash",json!({"id":section}))?;
            execute(s,&p,&r,&state.paths)?;
            assert!(s.in_bin(&section));Ok(())
        }).unwrap();
    }

    #[test]
    fn undo_rechecks_hidden_locked_and_leased_restored_destinations() {
        for protected in ["hidden", "locked", "lease"] {
            let (state,p,section,page)=fixture();
            let set=state.with_mut(|s| {
                let d=make_db(s,&state.paths,&section); let row=make_row(s,&d);
                let mut child=commands::page(s,&page)?; child.parent=row.clone();
                s.put(Item::Page(child))?;
                s.run_action(&actor(&p),"page.move",|s| {
                    let mut child=commands::page(s,&page)?;child.parent.clear();
                    s.put(Item::Page(child))
                })?;
                let set=s.activity_list(None,None,100)?.into_iter().find(|e| e.action=="page.move").unwrap().change_set.clone();
                // The old destination is absent from the moved page's current ancestry.
                let (restore,_) = s.activity_undo_plan(&actor(&p),&set)?;
                assert!(s.ai_write_targets(&actor(&p),&restore)?.contains(&d));
                match protected {
                    "hidden"=> { // Move the destination (only) under another, now-hidden notebook.
                        notes::create_notebook_core(s,&Actor::Person,"Hidden".into(),String::new(),String::new())?;
                        let nb=s.items().find_map(|i|match i {Item::Notebook(n) if n.name=="Hidden"=>Some(n.id.clone()),_=>None}).unwrap();
                        let destination=s.items().find_map(|i|match i {Item::Section(x) if x.notebook_id==nb=>Some(x.id.clone()),_=>None}).unwrap();
                        let mut db=db::database(s,&d)?;db.section_id=destination;s.put(Item::Database(db))?;
                        let mut settings=s.settings();settings.ai.hidden_notebooks.push(nb);s.put(Item::Settings(settings))?;
                    },
                    "locked"=>{let mut db=db::database(s,&d)?;db.locked=true;s.put(Item::Database(db))?;},
                    _=>s.coordination.editing(&row,"main",true),
                }
                assert!(s.activity_undo(&actor(&p),&set).is_err());
                assert!(commands::page(s,&page)?.parent.is_empty());
                Ok(set)
            }).unwrap();
            assert!(call(&state,&p,"activity.undo",json!({"changeSetId":set}),true,||false,||{}).is_err());
        }
    }

    #[test]
    fn undo_rechecks_destination_after_window_confirmation() {
        let (state,p,section,page)=fixture();
        let (destination,set)=state.with_mut(|s| {
            let mut settings=s.settings();settings.ai.big_change_items=0;s.put(Item::Settings(settings))?;
            let d=make_db(s,&state.paths,&section);let row=make_row(s,&d);
            let mut child=commands::page(s,&page)?;child.parent=row;s.put(Item::Page(child))?;
            s.run_action(&actor(&p),"page.move",|s|{let mut child=commands::page(s,&page)?;child.parent.clear();s.put(Item::Page(child))})?;
            let set=s.activity_list(None,None,100)?.into_iter().find(|e|e.action=="page.move").unwrap().change_set.clone();
            Ok((d,set))
        }).unwrap();
        let state=std::sync::Arc::new(state);let weak=std::sync::Arc::downgrade(&state);
        state.ai.bind(move |event,v|{if event=="ai-confirm" {
            let state=weak.upgrade().unwrap();state.with_mut(|s|{let mut d=db::database(s,&destination)?;d.locked=true;s.put(Item::Database(d))}).unwrap();
            assert!(state.ai.confirms.answer("confirm",v["id"].as_str().unwrap(),true,v["word"].as_str()));
        }});
        assert!(call(&state,&p,"activity.undo",json!({"changeSetId":set}),false,||false,||{}).is_err());
        assert!(state.with(|s|Ok(commands::page(s,&page)?.parent.is_empty())).unwrap());
    }

    #[test]
    fn sibling_leases_and_claims_do_not_block_unrelated_writes_or_undo() {
        for protection in ["lease", "claim"] {
            let (state,p,section,page)=fixture();
            let sibling=state.with_mut(|s|Ok(notes::create_page_core(s,&Actor::Person,section.clone(),"Sibling".into())?.id.clone())).unwrap();
            let set=state.with_mut(|s| {
                s.run_action(&actor(&p),"page.rename",|s| {
                    let mut item=commands::page(s,&page)?;item.title="AI edit".into();s.put(Item::Page(item))
                })?;
                let set=s.activity_list(None,None,100)?.into_iter().find(|e|e.action=="page.rename").unwrap().change_set.clone();
                Ok(set)
            }).unwrap();
            state.with_mut(|s| {
                if protection=="lease" { s.coordination.editing(&sibling,"main",true); }
                else { s.claim(&Actor::Ai{pairing_id:"other".into(),label:"Other".into()},std::slice::from_ref(&sibling))?; }
                Ok(())
            }).unwrap();
            call(&state,&p,"activity.undo",json!({"changeSetId":set}),true,||false,||{}).unwrap();
            state.with_actor(&actor(&p),|s| {
                let r=prepare(s,&p,"activity.undo",json!({"changeSetId":set}))?;
                execute(s,&p,&r,&state.paths)?;
                assert_eq!(commands::page(s,&page)?.title,"old");
                Ok(())
            }).unwrap();
            call(&state,&p,"page.rename",json!({"id":page,"title":"Unrelated edit"}),false,||false,||{}).unwrap();
            assert!(call(&state,&p,"page.rename",json!({"id":sibling,"title":"Blocked"}),false,||false,||{}).is_err());
            assert!(call(&state,&p,"section.trash",json!({"id":section}),true,||false,||{}).is_err());
        }
    }

    #[test]
    fn authorized_edit_reads_expose_persistent_legacy_ids_for_targeted_edits() {
        let (state,p,_,page)=fixture();
        state.with_mut(|s| {
            let mut item=commands::page(s,&page)?;
            item.content=json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"exact legacy"}]}]}).to_string();
            s.put(Item::Page(item))?;
            let mut settings=s.settings();settings.ai.pairings[0].level=rules::Level::Look;
            s.put(Item::Settings(settings))
        }).unwrap();
        let before=state.with(|s|Ok(snapshot(s))).unwrap();
        for action in ["page.get","page.outline","block.list"] {
            call(&state,&p,action,json!({"id":page}),false,||false,||{}).unwrap();
            assert_eq!(state.with(|s|Ok(snapshot(s))).unwrap(),before);
        }
        state.with_mut(|s| {let mut settings=s.settings();settings.ai.pairings[0].level=rules::Level::Full;s.put(Item::Settings(settings))}).unwrap();
        let first=call(&state,&p,"block.list",json!({"id":page}),false,||false,||{}).unwrap();
        let bid=first[0]["attrs"]["bid"].as_str().unwrap();
        assert_eq!(bid.len(),16);
        let second=call(&state,&p,"block.list",json!({"id":page}),false,||false,||{}).unwrap();
        assert_eq!(first,second);
        call(&state,&p,"block.set_direction",json!({"pageId":page,"blockId":bid,"direction":"rtl"}),false,||false,||{}).unwrap();
        let updated=call(&state,&p,"block.list",json!({"id":page}),false,||false,||{}).unwrap();
        assert_eq!(updated[0]["attrs"]["bid"],bid);
        assert_eq!(updated[0]["attrs"]["dir"],"rtl");
        assert_eq!(updated[0]["content"][0]["text"],"exact legacy");
    }

    #[test]
    fn look_reads_and_previews_leave_locked_legacy_rows_unchanged() {
        let (state,p,section,_) = fixture();
        let row=state.with_mut(|s|{
            let id=make_db(s,&state.paths,&section);let row=make_row(s,&id);
            let mut r=db::row(s,&row)?;r.content=json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"legacy exact"}]}]}).to_string();s.put(Item::Row(r))?;
            let mut d=db::database(s,&id)?;d.locked=true;s.put(Item::Database(d))?;
            let mut settings=s.settings();settings.ai.pairings[0].level=rules::Level::Look;s.put(Item::Settings(settings))?;
            Ok(row)
        }).unwrap();
        for level in [rules::Level::Look,rules::Level::Edit,rules::Level::Full] {
            state.with_mut(|s| {let mut settings=s.settings();settings.ai.pairings[0].level=level;s.put(Item::Settings(settings))}).unwrap();
            let before=state.with(|s|Ok(snapshot(s))).unwrap();
            for action in ["page.get","page.outline","block.list"] {
                for preview in [true,false] {
                    call(&state,&p,action,json!({"id":row}),preview,||false,||{}).unwrap();
                    assert_eq!(state.with(|s|Ok(snapshot(s))).unwrap(),before);
                }
            }
        }
    }

    #[test]
    fn unsupported_batches_fail_closed_for_execution_and_preview() {
        let (state,p,_,page)=fixture();
        for params in [json!({"steps":[]}),json!({"steps":[{"action":"ai_set_enabled","params":{}}]}),
            json!({"steps":vec![json!({"action":"page.rename","params":{"id":page,"title":"bad"}});501]})] {
            for preview in [false,true] {assert!(call(&state,&p,"batch.run",params.clone(),preview,||false,||{}).is_err());}
        }
        let permissions=call(&state,&p,"ai.permissions.get",json!({}),false,||false,||{}).unwrap();
        assert!(!permissions.to_string().contains("batch.run"));
    }

    #[test]
    fn unsafe_block_links_are_rejected_in_pages_templates_and_stickies() {
        let (state,p,section,page)=fixture();
        state.with_mut(|s|{
            let id=make_db(s,&state.paths,&section);
            let (sticky,_)=stickies::create_sticky_core(s,&Actor::Person,false)?;
            for href in ["javascript:alert(1)","file:///D:/private","data:text/html,test"," https://example.com","https:\n//example.com"] {
                let body=json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"exact text","marks":[{"type":"link","attrs":{"href":href}}]}]}]});
                for (action,params) in [("page.write",json!({"id":page,"format":"blocks","content":body})),
                    ("db.template.create",json!({"databaseId":id,"template":{"id":"t","name":"Template","content":body.to_string()}})),
                    ("sticky.write",json!({"id":sticky.id,"content":body.to_string()}))] {
                    // Preparation rejects page content early; the shared execution
                    // validator also protects templates and stickies.
                    match prepare(s,&p,action,params) {Err(_)=>{},Ok(r)=>assert!(execute(s,&p,&r,&state.paths).is_err())}
                }
            }
            for href in ["https://example.com/a?b=1","mailto:a@example.com","notaya:page/abc_12#heading"] {
                let body=json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"exact text","marks":[{"type":"link","attrs":{"href":href}}]}]}]});
                images(s,&body)?;assert_eq!(body["content"][0]["content"][0]["marks"][0]["attrs"]["href"],href);
            }
            Ok(())
        }).unwrap();
    }
    #[test]
    fn live_pairing_levels_off_hidden_leases_and_revisions_are_ordered() {
        let (state, p, _, page) = fixture();
        let q = json!({"id":page,"title":"new"});
        state
            .with_mut(|s| {
                let mut settings = s.settings();
                let mut other = p.clone();
                other.id = crate::model::new_id();
                other.level = rules::Level::Look;
                settings.ai.pairings.push(other.clone());
                s.put(Item::Settings(settings))?;
                assert_eq!(
                    error_code(&prepare(s, &other, "page.rename", q.clone()).err().unwrap()),
                    "level_too_low"
                );
                assert!(prepare(s, &p, "page.rename", q.clone()).is_ok());
                s.coordination.editing(&page, "main", true);
                assert_eq!(
                    error_code(&prepare(s, &p, "page.rename", q.clone()).err().unwrap()),
                    "person_editing"
                );
                s.coordination.forget_window("main");
                let mut stale = q.clone();
                stale["expect_rev"] = json!(s.rev(&page) + 1);
                assert_eq!(
                    error_code(&prepare(s, &p, "page.rename", stale).err().unwrap()),
                    "item_changed"
                );
                let mut settings = s.settings();
                let nb = s
                    .items()
                    .find_map(|i| {
                        if let Item::Notebook(x) = i {
                            Some(x.id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap();
                settings.ai.hidden_notebooks.push(nb);
                s.put(Item::Settings(settings))?;
                assert_eq!(
                    error_code(&prepare(s, &p, "page.rename", q.clone()).err().unwrap()),
                    "not_found"
                );
                Ok(())
            })
            .unwrap();
        for forbidden in [
            "ai_confirm_answer",
            "ai_set_level",
            "lock_vault",
            "unlock_vault",
            "activity.clear",
            "clipboard_read",
        ] {
            assert!(state
                .with(|s| prepare(s, &p, forbidden, json!({})).map(|_| ()))
                .is_err());
        }
    }
    #[test]
    fn exact_blocks_source_tags_preview_and_settings_cannot_change_ai() {
        let (state, p, _, page) = fixture();
        let title = "\u{200f}عنوان\u{200e}  ";
        let line="\u{061c}مرحبا\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}\u{2066}\u{2067}\u{2068}\u{2069}  ";
        let code_text = "\tgraph TD\n\tA[مرحبا] --> B  \n";
        let doc = json!({"type":"doc","content":[{"type":"paragraph","attrs":{"dir":"rtl"},"content":[{"type":"text","text":line}]},
            {"type":"paragraph","attrs":{"dir":"ltr"},"content":[{"type":"text","text":"?Wie geht es dir?  e\u{301}"}]},
            {"type":"codeBlock","content":[{"type":"text","text":code_text}]}]});
        call(
            &state,
            &p,
            "page.rename",
            json!({"id":page,"title":title}),
            false,
            || false,
            || {},
        )
        .unwrap();
        let q = json!({"id":page,"format":"blocks","content":doc,"source":{"app":"notion","id":"original-id"}});
        let before = state.with(|s| Ok(s.rev(&page))).unwrap();
        call(&state, &p, "page.write", q.clone(), true, || false, || {}).unwrap();
        assert_eq!(state.with(|s| Ok(s.rev(&page))).unwrap(), before);
        call(&state, &p, "page.write", q, false, || false, || {}).unwrap();
        let read = call(
            &state,
            &p,
            "page.get",
            json!({"id":page,"format":"blocks"}),
            false,
            || false,
            || {},
        )
        .unwrap();
        assert_eq!(read["title"].as_str().unwrap().as_bytes(), title.as_bytes());
        assert_eq!(
            read["content"]["content"][0]["content"][0]["text"]
                .as_str()
                .unwrap()
                .as_bytes(),
            line.as_bytes()
        );
        assert_eq!(
            read["content"]["content"][2]["content"][0]["text"]
                .as_str()
                .unwrap()
                .as_bytes(),
            code_text.as_bytes()
        );
        assert_eq!(read["content"]["content"][0]["attrs"]["dir"], "rtl");
        let found = call(
            &state,
            &p,
            "find.by_source",
            json!({"source":{"app":"notion","id":"original-id"}}),
            false,
            || false,
            || {},
        )
        .unwrap();
        assert_eq!(found[0]["id"], page);
        let settings = call(
            &state,
            &p,
            "settings.get",
            json!({}),
            false,
            || false,
            || {},
        )
        .unwrap();
        for key in [
            "ai",
            "idleLockMinutes",
            "screenCaptureProtection",
            "replacedEdits",
        ] {
            assert!(settings.get(key).is_none());
        }
        for patch in [
            json!({"ai":{"enabled":false}}),
            json!({"screenCaptureProtection":false}),
        ] {
            assert!(call(&state, &p, "settings.set", patch, false, || false, || {}).is_err());
        }
        state
            .with_mut(|s| {
                let mut incoming = s.settings();
                incoming.ai = Default::default();
                commands::vault::save_settings_core(s, &Actor::Person, incoming)?;
                assert!(s.settings().ai.enabled);
                Ok(())
            })
            .unwrap();
    }
    #[test]
    fn malicious_errors_and_structured_input_never_echo_note_text() {
        let (state, p, _, page) = fixture();
        let err = call(
            &state,
            &p,
            "page.write",
            json!({"id":page,"format":"blocks","content":{"type":"script","text":"SECRET NOTE"}}),
            false,
            || false,
            || {},
        )
        .unwrap_err();
        assert_eq!(error_code(&err), "invalid");
        assert!(!error_code(&err).contains("SECRET"));
        assert!(!schema(
            &json!({"type":"object","properties":{},"additionalProperties":false}),
            &json!({"ai_confirm_answer":true})
        ));
    }

    #[test]
    fn every_action_checks_each_live_pairing_level_before_parameters() {
        let (state, full, _, _) = fixture();
        state
            .with_mut(|s| {
                let mut settings = s.settings();
                let mut look = full.clone();
                look.id = crate::model::new_id();
                look.level = rules::Level::Look;
                let mut edit = full.clone();
                edit.id = crate::model::new_id();
                edit.level = rules::Level::Edit;
                settings.ai.pairings.extend([look.clone(), edit.clone()]);
                s.put(Item::Settings(settings))?;
                for pairing in [&look, &edit, &full] {
                    for spec in ACTIONS {
                        let result = prepare(s, pairing, spec.name, json!({}));
                        let denied = result
                            .as_ref()
                            .err()
                            .is_some_and(|e| error_code(e) == "level_too_low");
                        assert_eq!(
                            denied,
                            !rules::allowed(pairing.level, spec),
                            "{} {:?}",
                            spec.name,
                            pairing.level
                        );
                    }
                }
                let mut settings = s.settings();
                settings.ai.enabled = false;
                s.put(Item::Settings(settings))?;
                assert_eq!(
                    error_code(&prepare(s, &look, "never", Value::Null).err().unwrap()),
                    "off"
                );
                let mut settings = s.settings();
                settings.ai.enabled = true;
                settings.ai.paused = true;
                s.put(Item::Settings(settings))?;
                assert_eq!(
                    error_code(&prepare(s, &look, "never", Value::Null).err().unwrap()),
                    "stopped"
                );
                let mut settings = s.settings();
                settings.ai.paused = false;
                settings.ai.pairings.clear();
                s.put(Item::Settings(settings))?;
                assert_eq!(
                    error_code(&prepare(s, &look, "never", Value::Null).err().unwrap()),
                    "not_paired"
                );
                Ok(())
            })
            .unwrap();
        state.take_session();
        assert_eq!(
            error_code(
                &call(&state, &full, "ai.hello", json!({}), false, || false, || {}).unwrap_err()
            ),
            "locked"
        );
    }

    #[test]
    fn deletes_always_require_window_confirmation() {
        let (state, p, _, page) = fixture();
        state
            .with_mut(|s| {
                for enabled in [true, false] {
                    let mut settings = s.settings();
                    settings.ai.big_change_confirm = enabled;
                    s.put(Item::Settings(settings))?;
                    assert_eq!(
                        prepare(s, &p, "page.trash", json!({"id":page}))?.confirm,
                        Confirm::Ask
                    );
                }
                Ok(())
            })
            .unwrap();
        let state = std::sync::Arc::new(state);
        let weak = std::sync::Arc::downgrade(&state);
        state.ai.bind(move |event, v| {
            if event == "ai-confirm" {
                let state = weak.upgrade().unwrap();
                assert!(!state.ai.confirms.answer(
                    "pairing",
                    v["id"].as_str().unwrap(),
                    true,
                    None
                ));
                assert!(state.ai.confirms.answer(
                    "confirm",
                    v["id"].as_str().unwrap(),
                    true,
                    v["word"].as_str()
                ));
            }
        });
        call(
            &state,
            &p,
            "page.trash",
            json!({"id":page}),
            false,
            || false,
            || {},
        )
        .unwrap();
        assert!(state.with(|s| Ok(s.in_bin(&page))).unwrap());
        state.ai.native.inject(native_confirm::TestApprover(true));
        call(
            &state,
            &p,
            "bin.delete_forever",
            json!({"id":page}),
            false,
            || false,
            || {},
        )
        .unwrap();
        assert!(state.with(|s| Ok(s.get(&page).is_none())).unwrap());
    }

    #[test]
    fn database_query_uses_screen_values_without_rounding_and_locked_database_is_read_only() {
        let (state, p, section, _) = fixture();
        let db = call(
            &state,
            &p,
            "db.create",
            json!({"sectionId":section,"title":"Finance"}),
            false,
            || false,
            || {},
        )
        .unwrap();
        let id = db["database"]["id"].as_str().unwrap().to_string();
        state
            .with_mut(|s| {
                for (pid, name, format) in [
                    ("amount", "Amount", "dollar"),
                    ("percent", "Percent", "percent"),
                ] {
                    saves::add_property(
                        s,
                        &Actor::Person,
                        &id,
                        Property {
                            id: pid.into(),
                            name: name.into(),
                            kind: PropKind::Number,
                            number_format: format.into(),
                            ..Default::default()
                        },
                    )?;
                }
                saves::add_property(
                    s,
                    &Actor::Person,
                    &id,
                    Property {
                        id: "formula".into(),
                        name: "Formula".into(),
                        kind: PropKind::Formula,
                        formula: "prop(\"Amount\") * prop(\"Percent\")".into(),
                        ..Default::default()
                    },
                )?;
                db::create_row_core(
                    s,
                    &Actor::Person,
                    id.clone(),
                    "row".into(),
                    json!({"amount":12345.678901,"percent":0.123456789}).to_string(),
                    String::new(),
                )?;
                Ok(())
            })
            .unwrap();
        let actual = call(
            &state,
            &p,
            "db.query",
            json!({"databaseId":id}),
            false,
            || false,
            || {},
        )
        .unwrap();
        let expected = state
            .with(|s| {
                let d = db::database(s, &id)?;
                value(db::ctx_of(s).run_view(&d, &d.views[0]))
            })
            .unwrap();
        assert_eq!(actual, expected);
        state
            .with_mut(|s| {
                let mut d = db::database(s, &id)?;
                d.locked = true;
                s.put(Item::Database(d))
            })
            .unwrap();
        assert!(call(
            &state,
            &p,
            "db.get",
            json!({"id":id}),
            false,
            || false,
            || {}
        )
        .is_ok());
        assert!(call(
            &state,
            &p,
            "db.rename",
            json!({"id":id,"title":"changed"}),
            false,
            || false,
            || {}
        )
        .is_err());
    }

    #[test]
    fn lock_flushes_one_read_summary_and_wipes_runtime_counts() {
        let (state, p, _, page) = fixture();
        for _ in 0..2 {
            call(
                &state,
                &p,
                "page.get",
                json!({"id":page,"format":"blocks"}),
                false,
                || false,
                || {},
            )
            .unwrap();
        }
        let s = state.take_session().unwrap();
        let entries = s.activity_list(None, None, 100).unwrap();
        let summaries: Vec<_> = entries.iter().filter(|e| e.action == "ai.reads").collect();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].counts["reads"], 2);
        assert!(summaries[0].items.is_empty());
        state.set_session(s);
        let s = state.take_session().unwrap();
        assert_eq!(
            s.activity_list(None, None, 100)
                .unwrap()
                .iter()
                .filter(|e| e.action == "ai.reads")
                .count(),
            1
        );
    }
}
