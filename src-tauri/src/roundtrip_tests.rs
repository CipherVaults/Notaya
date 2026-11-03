//! Export, then import, for every format, scope and output, on throwaway data (the owner's matrix,
//! 2026-10-29): what goes out must come back in. Run with `--nocapture` to see the table.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::commands::import_notes::{import_from, VaultAccess};
use crate::error::Result;
use crate::export::{self, Format, Options, Scope, ZipLimits};
use crate::model::{new_id, Item, Notebook, Page, Section};
use crate::vault::Session;

/// A real 3x3 PNG (red, green, blue columns).
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 3, 0, 0, 0, 3, 8, 2, 0, 0, 0, 217, 74, 34, 232, 0, 0, 0, 16, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 192, 0, 193, 88,
    88, 0, 134, 151, 8, 248, 175, 105, 88, 207, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

struct Vault(Mutex<Session>);

impl VaultAccess for Vault {
    fn with<T>(&self, f: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
        f(&self.0.lock().unwrap())
    }
    fn with_mut<T>(&self, f: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        f(&mut self.0.lock().unwrap())
    }
}

fn temp() -> PathBuf {
    let d = std::env::temp_dir().join(format!("notaya-roundtrip-{}", new_id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn new_vault(dir: &Path) -> Vault {
    std::fs::create_dir_all(dir).unwrap();
    Vault(Mutex::new(Session::create_open(&dir.join("vault.nota")).unwrap()))
}

fn page(id: &str, section: &str, parent: &str, title: &str, order: i64, image: Option<&str>) -> Item {
    let mut content = vec![
        format!(r#"{{"type":"heading","attrs":{{"level":2}},"content":[{{"type":"text","text":"Heading of {title}"}}]}}"#),
        format!(
            r#"{{"type":"paragraph","content":[{{"type":"text","text":"MARK {title} with "}},{{"type":"text","text":"bold words","marks":[{{"type":"bold"}}]}},{{"type":"text","text":" and "}},{{"type":"text","text":"marked text","marks":[{{"type":"highlight","attrs":{{"color":"rgba(120, 119, 116, 0.28)"}}}}]}}]}}"#
        ),
        format!(r#"{{"type":"bulletList","content":[{{"type":"listItem","content":[{{"type":"paragraph","content":[{{"type":"text","text":"ITEM {title}"}}]}}]}}]}}"#),
    ];
    if let Some(att) = image {
        content.push(format!(r#"{{"type":"image","attrs":{{"src":"http://clatt.localhost/{att}"}}}}"#));
    }
    Item::Page(Page {
        id: id.into(),
        section_id: section.into(),
        title: title.into(),
        content: format!(r#"{{"type":"doc","content":[{}]}}"#, content.join(",")),
        text: String::new(),
        order,
        favorite: false,
        icon: String::new(),
        cover: String::new(),
        parent: parent.into(),
        trashed_at: None,
        created: order,
        updated: order,
    })
}

/// "Alpha" (sections Journal and Ideas) and "Beta" (section Work). Journal One has a picture and a sub-page.
struct Ids {
    nb_a: String,
    nb_b: String,
    sec_j: String,
    sec_i: String,
    j1: String,
}

fn source(v: &Vault) -> (Ids, Vec<(&'static str, Vec<&'static str>)>) {
    let att = v.with_mut(|s| s.put_attachment("image/png", PNG)).unwrap();
    let nb = |id: &str, name: &str, order| Item::Notebook(Notebook { id: id.into(), name: name.into(), color: String::new(), icon: String::new(), order, created: 0, updated: 0 });
    let sec = |id: &str, nbid: &str, name: &str, order| {
        Item::Section(Section { id: id.into(), notebook_id: nbid.into(), name: name.into(), color: String::new(), order, created: 0, updated: 0 })
    };
    let id = |_: &str| new_id();
    let (nb_a, nb_b, sec_j, sec_i, sec_w, j1, jc, j2, i1, w1) = (id("nbA"), id("nbB"), id("secJ"), id("secI"), id("secW"), id("j1"), id("jc"), id("j2"), id("i1"), id("w1"));
    let items = vec![
        nb(&nb_a, "Alpha", 0),
        nb(&nb_b, "Beta", 1),
        sec(&sec_j, &nb_a, "Journal", 0),
        sec(&sec_i, &nb_a, "Ideas", 1),
        sec(&sec_w, &nb_b, "Work", 0),
        page(&j1, &sec_j, "", "Journal One", 0, Some(&att)),
        page(&jc, &sec_j, &j1, "Journal Child", 0, None),
        page(&j2, &sec_j, "", "Journal Two", 1, None),
        page(&i1, &sec_i, "", "Idea One", 0, None),
        page(&w1, &sec_w, "", "Work One", 0, None),
    ];
    let ids = Ids { nb_a, nb_b, sec_j, sec_i, j1 };
    v.with_mut(|s| s.put_many(items)).unwrap_or_else(|e| panic!("put_many: {e}"));
    // (scope name, expected titles)
    let cases = vec![
        ("one page", vec!["Journal One", "Journal Child"]),
        ("section", vec!["Journal One", "Journal Child", "Journal Two"]),
        ("notebook", vec!["Journal One", "Journal Child", "Journal Two", "Idea One"]),
        ("two sections", vec!["Journal One", "Journal Child", "Journal Two", "Idea One"]),
        ("two notebooks", vec!["Journal One", "Journal Child", "Journal Two", "Idea One", "Work One"]),
    ];
    (ids, cases)
}

/// The notebooks and sections a ZIP export of each case must come back as (one page and its
/// sub-page: loose pages; a single notebook keeps its name).
fn expected_shape(name: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    match name {
        "one page" => (vec!["*"], vec!["Imported pages"]),
        "section" => (vec!["*"], vec!["Journal"]),
        "notebook" => (vec!["Alpha"], vec!["Journal", "Ideas"]),
        "two sections" => (vec!["*"], vec!["Journal", "Ideas"]),
        _ => (vec!["Alpha", "Beta"], vec!["Journal", "Ideas", "Work"]),
    }
}

/// The names of the notebooks and sections in a vault.
fn shape_of(v: &Vault) -> (Vec<String>, Vec<String>) {
    v.with(|s| {
        let mut nbs = Vec::new();
        let mut secs = Vec::new();
        for i in s.items() {
            match i {
                Item::Notebook(n) => nbs.push(n.name.clone()),
                Item::Section(x) => secs.push(x.name.clone()),
                _ => {}
            }
        }
        Ok((nbs, secs))
    })
    .unwrap()
}

fn scope_of(ids: &Ids, name: &str) -> Scope {
    match name {
        "one page" => Scope::Pages(vec![ids.j1.clone()]),
        "section" => Scope::Section(ids.sec_j.clone()),
        "notebook" => Scope::Notebook(ids.nb_a.clone()),
        "two sections" => Scope::Sections(vec![ids.sec_j.clone(), ids.sec_i.clone()]),
        _ => Scope::Notebooks(vec![ids.nb_a.clone(), ids.nb_b.clone()]),
    }
}

/// A page with every palette text color and highlight, a code block, a task list, a captioned
/// picture and a shelf: exported to Markdown, HTML and Word and imported again, each comes back
/// as it was (the owner's round-trip losses of 2026-11-03).
#[test]
fn colors_code_tasks_captions_and_shelves_come_back() {
    use crate::importers::palette::{HIGHLIGHTS, TEXT_COLORS};
    use serde_json::{json, Value};
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let att = src.with_mut(|s| s.put_attachment("image/png", PNG)).unwrap();
    let mut colored = Vec::new();
    for (i, (_, c)) in TEXT_COLORS.iter().enumerate() {
        colored.push(json!({ "type": "text", "text": format!("tc{i}"), "marks": [{ "type": "textStyle", "attrs": { "color": c, "fontFamily": null, "fontSize": null } }] }));
        colored.push(json!({ "type": "text", "text": " " }));
    }
    for (i, (_, h)) in HIGHLIGHTS.iter().enumerate() {
        colored.push(json!({ "type": "text", "text": format!("hl{i}"), "marks": [{ "type": "highlight", "attrs": { "color": h } }] }));
        colored.push(json!({ "type": "text", "text": " " }));
    }
    colored.push(json!({ "type": "text", "text": "both", "marks": [{ "type": "highlight", "attrs": { "color": HIGHLIGHTS[9].1 } }, { "type": "textStyle", "attrs": { "color": TEXT_COLORS[1].1 } }] }));
    let code = "fn main() {\n    let x = 1;\n}";
    let content = json!({ "type": "doc", "content": [
        { "type": "paragraph", "content": colored },
        { "type": "codeBlock", "attrs": { "language": "rust" }, "content": [{ "type": "text", "text": code }] },
        { "type": "taskList", "content": [
            { "type": "taskItem", "attrs": { "checked": true }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Done task" }] }] },
            { "type": "taskItem", "attrs": { "checked": false }, "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Open task" }] }] }
        ] },
        { "type": "image", "attrs": { "src": format!("http://clatt.localhost/{att}"), "caption": "A caption" } },
        { "type": "toggle", "attrs": { "open": true, "shelf": true }, "content": [
            { "type": "paragraph", "content": [{ "type": "text", "text": "Shelf title" }] },
            { "type": "paragraph", "content": [{ "type": "text", "text": "Inside the shelf" }] }
        ] }
    ] });
    let (nb, sec, pid) = (new_id(), new_id(), new_id());
    let items = vec![
        Item::Notebook(Notebook { id: nb.clone(), name: "Colors".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0 }),
        Item::Section(Section { id: sec.clone(), notebook_id: nb, name: "Main".into(), color: String::new(), order: 0, created: 0, updated: 0 }),
        Item::Page(Page {
            id: pid.clone(),
            section_id: sec,
            title: "Round trip".into(),
            content: content.to_string(),
            text: String::new(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            created: 0,
            updated: 0,
        }),
    ];
    src.with_mut(|s| s.put_many(items)).unwrap();
    let picture = src.with(|s| s.get_attachment(&att)).unwrap();
    let attachment = |id: &str| (id == att).then(|| picture.clone());

    fn find<'v>(v: &'v Value, pred: &dyn Fn(&Value) -> bool) -> Option<&'v Value> {
        if pred(v) {
            return Some(v);
        }
        v["content"].as_array()?.iter().find_map(|c| find(c, pred))
    }
    let mut failures: Vec<String> = Vec::new();
    for format in [Format::Markdown, Format::Html, Format::Docx] {
        let label = format.label();
        let dir = root.join(format!("out-{}", format.ext()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("page").with_extension(format.ext());
        let exported = src.with(|s| {
            let items: Vec<&Item> = s.items().collect();
            let doc = export::collect(&items, &Scope::Pages(vec![pid.clone()]), &Options::default(), &attachment)?;
            export::write(&dest, format, &doc, &root.join("src-data")).map(|_| ())
        });
        if let Err(e) = exported {
            failures.push(format!("{label}: export failed: {e}"));
            continue;
        }
        let dst = new_vault(&root.join(format!("dst-{}", format.ext())));
        if let Err(e) = import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000, &crate::importers::Control::standard()) {
            failures.push(format!("{label}: import failed: {e}"));
            continue;
        }
        let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
        let Some(v) = pages.iter().map(|p| serde_json::from_str::<Value>(&p.content).unwrap()).find(|v| v.to_string().contains("tc0")) else {
            failures.push(format!("{label}: the page did not come back"));
            continue;
        };
        let marks_of = |word: &str| -> (Option<String>, Option<String>) {
            let Some(node) = find(&v, &|n| n["type"] == "text" && n["text"] == word) else { return (None, Some("missing".into())) };
            let mut out = (None, None);
            for m in node["marks"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
                if m["type"] == "textStyle" {
                    out.0 = m["attrs"]["color"].as_str().map(str::to_string);
                }
                if m["type"] == "highlight" {
                    out.1 = Some(m["attrs"]["color"].as_str().unwrap_or("").to_string());
                }
            }
            out
        };
        for (i, (name, c)) in TEXT_COLORS.iter().enumerate() {
            let got = marks_of(&format!("tc{i}"));
            if got.0.as_deref() != Some(*c) {
                failures.push(format!("{label}: text color {name} came back as {got:?}"));
            }
        }
        for (i, (name, h)) in HIGHLIGHTS.iter().enumerate() {
            let got = marks_of(&format!("hl{i}"));
            if got.1.as_deref() != Some(*h) {
                failures.push(format!("{label}: highlight {name} came back as {got:?}"));
            }
        }
        if marks_of("both") != (Some(TEXT_COLORS[1].1.into()), Some(HIGHLIGHTS[9].1.into())) {
            failures.push(format!("{label}: color and highlight together came back as {:?}", marks_of("both")));
        }
        match find(&v, &|n| n["type"] == "codeBlock") {
            Some(c) if c["attrs"]["language"] == "rust" && c["content"][0]["text"] == code => {}
            other => failures.push(format!("{label}: the code block came back as {other:?}")),
        }
        match find(&v, &|n| n["type"] == "taskList") {
            Some(t) if t["content"][0]["attrs"]["checked"] == true
                && t["content"][1]["attrs"]["checked"] == false
                && t["content"][0].to_string().contains("Done task")
                && t["content"][1].to_string().contains("Open task") => {}
            other => failures.push(format!("{label}: the task list came back as {other:?}")),
        }
        match find(&v, &|n| n["type"] == "image") {
            Some(i) if i["attrs"]["caption"] == "A caption" => {}
            other => failures.push(format!("{label}: the picture came back as {other:?}")),
        }
        if v.to_string().matches("A caption").count() != 1 {
            failures.push(format!("{label}: the caption is also kept as text"));
        }
        match find(&v, &|n| n["type"] == "toggle") {
            Some(t) if t["attrs"]["shelf"] == true && t.to_string().contains("Shelf title") && t.to_string().contains("Inside the shelf") => {}
            other => failures.push(format!("{label}: the shelf came back as {other:?}")),
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// A page mixing Arabic, Hebrew and English (and one English block forced right-to-left) is exported
/// to Markdown, HTML and Word and imported again: every block runs the way it did (the owner's
/// "the Arabic stays right and the English left, with the export as well", 2026-11-03). Markdown is plain
/// text, so it keeps the direction the text itself gives; a block forced the other way is the one thing it
/// cannot carry (nor can Word carry "left-to-right" for Arabic text).
#[test]
fn arabic_hebrew_and_english_keep_their_direction() {
    use crate::export::doc::{first_strong, Dir};
    use serde_json::{json, Value};
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let t = |s: &str| json!({ "type": "text", "text": s });
    let p = |s: &str| json!({ "type": "paragraph", "content": [t(s)] });
    let item = |s: &str| json!({ "type": "listItem", "content": [p(s)] });
    let cell = |s: &str, header: bool| json!({ "type": if header { "tableHeader" } else { "tableCell" }, "content": [p(s)] });
    let content = json!({ "type": "doc", "content": [
        { "type": "heading", "attrs": { "level": 2 }, "content": [t("عنوان عربي")] },
        p("هذه فقرة عربية تبدأ من اليمين"),
        p("This paragraph is English."),
        p("זוהי פסקה בעברית"),
        { "type": "paragraph", "attrs": { "dir": "rtl" }, "content": [t("English forced right to left")] },
        { "type": "paragraph", "attrs": { "dir": "ltr" }, "content": [t("مرحبا فرض اليسار")] },
        { "type": "bulletList", "content": [item("بند عربي"), item("English bullet")] },
        { "type": "taskList", "content": [{ "type": "taskItem", "attrs": { "checked": true }, "content": [p("مهمة عربية")] }] },
        { "type": "blockquote", "content": [p("اقتباس عربي")] },
        { "type": "table", "content": [
            { "type": "tableRow", "content": [cell("الاسم", true), cell("القيمة", true)] },
            { "type": "tableRow", "content": [cell("أحمد", false), cell("42", false)] }
        ] }
    ] });
    let (nb, sec, pid) = (new_id(), new_id(), new_id());
    let items = vec![
        Item::Notebook(Notebook { id: nb.clone(), name: "Directions".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0 }),
        Item::Section(Section { id: sec.clone(), notebook_id: nb, name: "Main".into(), color: String::new(), order: 0, created: 0, updated: 0 }),
        Item::Page(Page {
            id: pid.clone(),
            section_id: sec,
            title: "Directions".into(),
            content: content.to_string(),
            text: String::new(),
            order: 0,
            favorite: false,
            icon: String::new(),
            cover: String::new(),
            parent: String::new(),
            trashed_at: None,
            created: 0,
            updated: 0,
        }),
    ];
    src.with_mut(|s| s.put_many(items)).unwrap();

    /// Every paragraph and heading of an editor document: (text, the direction the editor shows it in).
    fn shown(v: &Value, out: &mut Vec<(String, Dir)>) {
        if v["type"] == "paragraph" || v["type"] == "heading" {
            let text = v["content"].as_array().map(|c| c.iter().filter_map(|n| n["text"].as_str()).collect::<String>()).unwrap_or_default();
            let dir = match v["attrs"]["dir"].as_str() {
                Some("rtl") => Dir::Rtl,
                Some("ltr") => Dir::Ltr,
                _ => first_strong(&text).unwrap_or(Dir::Ltr),
            };
            out.push((text, dir));
        }
        for c in v["content"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            shown(c, out);
        }
    }
    let mut before = Vec::new();
    shown(&content, &mut before);
    assert_eq!(before.iter().filter(|(_, d)| *d == Dir::Rtl).count(), 10, "{before:?}");

    let mut failures: Vec<String> = Vec::new();
    for format in [Format::Markdown, Format::Html, Format::Docx] {
        let label = format.label();
        let dir = root.join(format!("out-{}", format.ext()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("page").with_extension(format.ext());
        let exported = src.with(|s| {
            let items: Vec<&Item> = s.items().collect();
            let doc = export::collect(&items, &Scope::Pages(vec![pid.clone()]), &Options::default(), &|_: &str| None)?;
            export::write(&dest, format, &doc, &root.join("src-data")).map(|_| ())
        });
        if let Err(e) = exported {
            failures.push(format!("{label}: export failed: {e}"));
            continue;
        }
        let dst = new_vault(&root.join(format!("dst-{}", format.ext())));
        if let Err(e) = import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000, &crate::importers::Control::standard()) {
            failures.push(format!("{label}: import failed: {e}"));
            continue;
        }
        let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
        let Some(v) = pages.iter().map(|p| serde_json::from_str::<Value>(&p.content).unwrap()).find(|v| v.to_string().contains("English bullet")) else {
            failures.push(format!("{label}: the page did not come back"));
            continue;
        };
        let mut after = Vec::new();
        shown(&v, &mut after);
        for (text, want) in &before {
            // Markdown cannot carry a direction that the text does not give (its two forced blocks), and Word
            // cannot carry "left-to-right" for Arabic text (a paragraph without bidi is how every Word
            // file says it, so Notaya reads that by the text).
            let lost = match format {
                Format::Markdown => text == "English forced right to left" || text == "مرحبا فرض اليسار",
                Format::Docx => text == "مرحبا فرض اليسار",
                _ => false,
            };
            let want = if lost { first_strong(text).unwrap_or(Dir::Ltr) } else { *want };
            match after.iter().find(|(t, _)| t == text) {
                Some((_, got)) if *got == want => {}
                Some((_, got)) => failures.push(format!("{label}: {text:?} came back {got:?}, was {want:?}")),
                None => failures.push(format!("{label}: {text:?} is missing from {after:?}")),
            }
        }
        // The Arabic table's first column is still the Arabic one.
        let table = v["content"].as_array().and_then(|c| c.iter().find(|n| n["type"] == "table"));
        match table {
            Some(t) if t["content"][0]["content"][0]["content"][0]["content"][0]["text"] == "الاسم" => {}
            other => failures.push(format!("{label}: the Arabic table came back as {other:?}")),
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

#[test]
fn every_format_scope_and_output_comes_back_in() {
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let (ids, cases) = source(&src);
    let mut failures: Vec<String> = Vec::new();
    let mut table = vec![format!("{:<8} {:<14} {:<6} {:>9} {:>5} {:>8} {:>6}  notes", "format", "scope", "output", "notebooks", "pages", "sections", "images")];
    for format in [Format::Markdown, Format::Html, Format::Docx, Format::Pdf] {
        for (scope_name, expected) in &cases {
            for zip in [false, true] {
                let out = if zip { "zip" } else { "single" };
                let label = format!("{} / {scope_name} / {out}", format.label());
                let dir = root.join(format!("out-{}-{}-{}", format.ext(), scope_name.replace(' ', "_"), out));
                std::fs::create_dir_all(&dir).unwrap();
                let dest = dir.join(if zip { "export.zip" } else { "export" }).with_extension(if zip { "zip" } else { format.ext() });
                let scope = scope_of(&ids, scope_name);
                let opts = Options { include_subpages: true, ..Options::default() };
                // The pictures are read before the vault is held (its lock is not re-entrant).
                let picture_ids: Vec<String> = src
                    .with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.content.clone()) } else { None }).collect::<Vec<_>>()))
                    .unwrap()
                    .iter()
                    .filter_map(|c| c.split("http://clatt.localhost/").nth(1).map(|r| r.split('"').next().unwrap_or("").to_string()))
                    .collect();
                let pictures: Vec<(String, (String, Vec<u8>))> = picture_ids.into_iter().map(|id| { let pic = src.with(|s| s.get_attachment(&id)).unwrap(); (id, pic) }).collect();
                let attachment = |id: &str| pictures.iter().find(|(i, _)| i == id).map(|(_, p)| p.clone());
                let exported = src.with(|s| {
                    let items: Vec<&Item> = s.items().collect();
                    if zip {
                        let zdoc = export::collect_zip(&items, &scope, &opts, format)?;
                        export::write_zip(&dest, format, zdoc, &attachment, ZipLimits::default()).map(|_| ())
                    } else {
                        let doc = export::collect(&items, &scope, &opts, &attachment)?;
                        export::write(&dest, format, &doc, &root.join("src-data")).map(|_| ())
                    }
                });
                if let Err(e) = exported {
                    failures.push(format!("{label}: export failed: {e}"));
                    continue;
                }
                let dst = new_vault(&root.join(format!("dst-{}-{}-{}", format.ext(), scope_name.replace(' ', "_"), out)));
                let summary = match import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000, &crate::importers::Control::standard()) {
                    Ok(s) => s,
                    Err(e) => {
                        failures.push(format!("{label}: import failed: {e}"));
                        continue;
                    }
                };
                let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
                // The structure: a ZIP comes back as the notebooks, sections and pages that went out
                // (Notaya's marker says how it is laid out); one combined file is one page.
                let (notebooks, sections) = shape_of(&dst);
                let (want_nbs, want_secs): (Vec<&str>, Vec<&str>) = if zip { expected_shape(scope_name) } else { (vec!["*"], vec!["Imported pages"]) };
                // "*": one notebook named after the file ("export" is too general, so a date).
                let mut got_nbs: Vec<&str> = notebooks.iter().map(|n| if want_nbs == ["*"] { "*" } else { n.as_str() }).collect();
                let mut got_secs: Vec<&str> = sections.iter().map(String::as_str).collect();
                got_nbs.sort_unstable();
                got_secs.sort_unstable();
                let (mut want_nbs, mut want_secs) = (want_nbs, want_secs);
                want_nbs.sort_unstable();
                want_secs.sort_unstable();
                if (got_nbs.len(), got_secs.len(), pages.len()) != (want_nbs.len(), want_secs.len(), if zip { expected.len() } else { 1 }) {
                    failures.push(format!(
                        "{label}: {} notebooks, {} sections, {} pages came back for {} notebooks, {} sections, {} pages",
                        got_nbs.len(),
                        got_secs.len(),
                        pages.len(),
                        want_nbs.len(),
                        want_secs.len(),
                        if zip { expected.len() } else { 1 }
                    ));
                }
                if got_secs != want_secs || got_nbs != want_nbs {
                    failures.push(format!("{label}: notebooks {got_nbs:?} and sections {got_secs:?}, expected {want_nbs:?} and {want_secs:?}"));
                }
                if summary.notebooks.len() != got_nbs.len() || summary.notebook_id != summary.notebooks[0].id {
                    failures.push(format!("{label}: the summary names {} notebooks", summary.notebooks.len()));
                }
                let text: String = pages.iter().map(|p| format!("{}\n{}\n", p.title, p.text)).collect();
                let json: String = pages.iter().map(|p| p.content.clone()).collect();
                let mut notes = Vec::new();
                // Every exported page's text is back in the import.
                for title in expected {
                    if !text.contains(&format!("MARK {title}")) {
                        failures.push(format!("{label}: the text of '{title}' did not come back"));
                    }
                    if !text.contains(&format!("ITEM {title}")) {
                        failures.push(format!("{label}: the list item of '{title}' did not come back"));
                    }
                }
                // One file per page in a ZIP; one combined document otherwise.
                if zip {
                    let real = pages.iter().filter(|p| !p.text.trim().is_empty()).count();
                    if real < expected.len() {
                        failures.push(format!("{label}: {real} pages with text for {} exported pages", expected.len()));
                    }
                } else if summary.pages != 1 {
                    failures.push(format!("{label}: a combined file should import as one page, got {}", summary.pages));
                }
                // Formatting and pictures where the format carries them.
                if format != Format::Pdf && !json.contains("\"bold\"") {
                    failures.push(format!("{label}: bold did not come back"));
                }
                if matches!(format, Format::Html | Format::Docx) && !json.contains("highlight") {
                    failures.push(format!("{label}: the highlight did not come back"));
                }
                let has_picture = expected.contains(&"Journal One");
                let wants_images = has_picture && format != Format::Pdf;
                if wants_images && summary.images == 0 {
                    failures.push(format!("{label}: the picture did not come back"));
                }
                if format == Format::Pdf {
                    notes.push("text only");
                }
                table.push(format!(
                    "{:<8} {:<14} {:<6} {:>9} {:>5} {:>8} {:>6}  {}",
                    format.ext(),
                    scope_name,
                    out,
                    summary.notebooks.len(),
                    summary.pages,
                    summary.sections,
                    summary.images,
                    notes.join(", ")
                ));
            }
        }
    }
    println!("\n{}\n", table.join("\n"));
    let _ = std::fs::remove_dir_all(&root);
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// A page with an inline link to each of `inline` and a "Link to page" row to each of `rows`
/// (page ids; the link text is `LINK <name>` with the name from `name_of`).
fn linking(id: &str, section: &str, parent: &str, title: &str, order: i64, inline: &[&str], rows: &[&str], name_of: &dyn Fn(&str) -> &'static str) -> Item {
    let mut content = vec![format!(r#"{{"type":"paragraph","content":[{{"type":"text","text":"MARK {title}"}}]}}"#)];
    for t in inline {
        let name = name_of(t);
        content.push(format!(
            r#"{{"type":"paragraph","content":[{{"type":"text","text":"see "}},{{"type":"text","text":"LINK {name}","marks":[{{"type":"link","attrs":{{"href":"notaya:page/{t}#h0abc"}}}}]}},{{"type":"text","text":" here"}}]}}"#
        ));
    }
    for t in rows {
        content.push(format!(r#"{{"type":"pageLink","attrs":{{"pageId":"{t}"}}}}"#));
    }
    let mut p = page(id, section, parent, title, order, None);
    if let Item::Page(p) = &mut p {
        p.content = format!(r#"{{"type":"doc","content":[{}]}}"#, content.join(","));
    }
    p
}

/// Links between pages (in text, and "Link to page" rows) go out as links between the files of
/// a ZIP and come back as links between the new pages, in every format that has links. Links
/// to pages that were not exported come back as their text; nothing dead or `notaya:` is left.
#[test]
fn links_between_exported_pages_come_back_as_links_between_the_imported_pages() {
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let nb = |id: &str, name: &str, order| Item::Notebook(Notebook { id: id.into(), name: name.into(), color: String::new(), icon: String::new(), order, created: 0, updated: 0 });
    let sec = |id: &str, nbid: &str, name: &str, order| Item::Section(Section { id: id.into(), notebook_id: nbid.into(), name: name.into(), color: String::new(), order, created: 0, updated: 0 });
    let (n1, n2, s1, s2, pa, pb, pc, pt) = (new_id(), new_id(), new_id(), new_id(), new_id(), new_id(), new_id(), new_id());
    let ids = [(pa.clone(), "pa"), (pb.clone(), "pb"), (pc.clone(), "pc"), (pt.clone(), "ptrash")];
    let name_of = |id: &str| ids.iter().find(|(i, _)| i == id).map_or("?", |(_, n)| *n);
    let mut trashed = page(&pt, &s1, "", "Thrown away", 9, None);
    if let Item::Page(p) = &mut trashed {
        p.trashed_at = Some(5);
    }
    let items = vec![
        nb(&n1, "My notes", 0),
        nb(&n2, "Other", 1),
        sec(&s1, &n1, "Day 1", 0),
        sec(&s2, &n2, "Café & co", 0),
        linking(&pa, &s1, "", "Plan #1 (draft)", 0, &[&pb, &pc, &pt], &[&pb, &pc, &pt], &name_of),
        linking(&pb, &s1, &pa, "Sub 100%", 1, &[&pa, &pc], &[], &name_of),
        linking(&pc, &s2, "", "Ünïcode", 0, &[&pb], &[&pa], &name_of),
        trashed,
    ];
    src.with_mut(|s| s.put_many(items)).unwrap();
    let mut failures: Vec<String> = Vec::new();
    for format in [Format::Markdown, Format::Html, Format::Docx, Format::Pdf] {
        for zip in [true, false] {
            let label = format!("{} / {}", format.label(), if zip { "zip" } else { "single" });
            let dir = root.join(format!("links-{}-{zip}", format.ext()));
            std::fs::create_dir_all(&dir).unwrap();
            let dest = dir.join(if zip { "links.zip".to_string() } else { format!("links.{}", format.ext()) });
            let scope = Scope::Notebooks(vec![n1.clone(), n2.clone()]);
            let opts = Options { include_subpages: true, ..Options::default() };
            src.with(|s| {
                let items: Vec<&Item> = s.items().collect();
                if zip {
                    let zdoc = export::collect_zip(&items, &scope, &opts, format)?;
                    export::write_zip(&dest, format, zdoc, &|_| None, ZipLimits::default()).map(|_| ())
                } else {
                    let doc = export::collect(&items, &scope, &opts, &|_| None)?;
                    export::write(&dest, format, &doc, &root.join("src-data")).map(|_| ())
                }
            })
            .unwrap_or_else(|e| panic!("{label}: export: {e}"));
            let dst = new_vault(&root.join(format!("links-dst-{}-{zip}", format.ext())));
            let summary = import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000, &crate::importers::Control::standard())
                .unwrap_or_else(|e| panic!("{label}: import: {e}"));
            let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
            let all: String = pages.iter().map(|p| p.content.clone()).collect();
            for bad in ["notaya-src", &format!("notaya:page/{pa}"), &format!("notaya:page/{pb}"), &format!("notaya:page/{pc}"), &format!("notaya:page/{pt}"), "notaya_page", "Thrown away"] {
                if all.contains(bad) {
                    failures.push(format!("{label}: {bad:?} was left in the imported pages"));
                }
            }
            if !zip || format == Format::Pdf {
                // One page (single file), or text only (PDF): the text of every link is there.
                let text: String = pages.iter().map(|p| p.text.clone()).collect();
                for t in ["LINK pb", "LINK pc", "LINK ptrash", "Sub 100%"] {
                    if !text.contains(t) {
                        failures.push(format!("{label}: the text {t:?} did not come back"));
                    }
                }
                continue;
            }
            if summary.notebooks.len() != 2 {
                failures.push(format!("{label}: {} notebooks", summary.notebooks.len()));
            }
            let by_title = |t: &str| pages.iter().find(|p| p.title == t);
            let (Some(a), Some(b), Some(c)) = (by_title("Plan #1 (draft)"), by_title("Sub 100%"), by_title("Ünïcode")) else {
                failures.push(format!("{label}: pages came back as {:?}", pages.iter().map(|p| &p.title).collect::<Vec<_>>()));
                continue;
            };
            if b.parent != a.id {
                failures.push(format!("{label}: the sub-page lost its page"));
            }
            let link = |to: &Page| format!("\"href\":\"notaya:page/{}\"", to.id);
            let row = |to: &Page| format!("{{\"attrs\":{{\"pageId\":\"{}\"}},\"type\":\"pageLink\"}}", to.id);
            for (from, want) in [
                (a, vec![link(b), link(c), row(b), row(c)]),
                (b, vec![link(a), link(c)]),
                (c, vec![link(b), row(a)]),
            ] {
                for w in want {
                    if !from.content.contains(&w) {
                        failures.push(format!("{label}: '{}' lacks {w}: {}", from.title, from.content));
                    }
                }
            }
            // The links to the trashed page are text: the inline one and the row ("Page not found").
            if !a.text.contains("see LINK ptrash here") || a.content.matches("pageLink").count() != 2 {
                failures.push(format!("{label}: the link to a page outside the export: {}", a.content));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// A "Link to page" row inside a shelf goes out in every format and comes back as a row inside a
/// shelf (found in the real app: Markdown kept the shelf body as plain text).
#[test]
fn page_link_rows_inside_a_shelf_come_back_as_rows_inside_a_shelf() {
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let nb = Item::Notebook(Notebook { id: new_id(), name: "Book".into(), color: String::new(), icon: String::new(), order: 0, created: 0, updated: 0 });
    let Item::Notebook(n) = &nb else { unreachable!() };
    let section = Item::Section(Section { id: new_id(), notebook_id: n.id.clone(), name: "Sec".into(), color: String::new(), order: 0, created: 0, updated: 0 });
    let Item::Section(sec) = &section else { unreachable!() };
    let (pa, pb) = (new_id(), new_id());
    let mut a = page(&pa, &sec.id, "", "Holder", 0, None);
    if let Item::Page(p) = &mut a {
        p.content = format!(
            r#"{{"type":"doc","content":[{{"type":"toggle","attrs":{{"open":true,"shelf":true}},"content":[{{"type":"paragraph","content":[{{"type":"text","text":"Shelf SHELFTITLE"}}]}},{{"type":"pageLink","attrs":{{"pageId":"{pb}"}}}}]}}]}}"#
        );
    }
    let b = page(&pb, &sec.id, "", "Target", 1, None);
    src.with_mut(|s| s.put_many(vec![nb.clone(), section.clone(), a, b])).unwrap();
    let mut failures: Vec<String> = Vec::new();
    for format in [Format::Markdown, Format::Html, Format::Docx] {
        let label = format.label().to_string();
        let dir = root.join(format!("shelf-{}", format.ext()));
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("shelf.zip");
        let scope = Scope::Notebook(n.id.clone());
        let opts = Options { include_subpages: true, ..Options::default() };
        src.with(|s| {
            let items: Vec<&Item> = s.items().collect();
            let zdoc = export::collect_zip(&items, &scope, &opts, format)?;
            export::write_zip(&dest, format, zdoc, &|_| None, ZipLimits::default()).map(|_| ())
        })
        .unwrap_or_else(|e| panic!("{label}: export: {e}"));
        let dst = new_vault(&root.join(format!("shelf-dst-{}", format.ext())));
        import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000, &crate::importers::Control::standard())
            .unwrap_or_else(|e| panic!("{label}: import: {e}"));
        let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
        let (Some(a), Some(b)) = (pages.iter().find(|p| p.title == "Holder"), pages.iter().find(|p| p.title == "Target")) else {
            failures.push(format!("{label}: pages came back as {:?}", pages.iter().map(|p| &p.title).collect::<Vec<_>>()));
            continue;
        };
        let doc: serde_json::Value = serde_json::from_str(&a.content).unwrap();
        let shelf = &doc["content"][0];
        let ok_shelf = shelf["type"] == "toggle" && shelf["attrs"]["shelf"] == true;
        let row = shelf["content"].as_array().and_then(|c| c.iter().find(|n| n["type"] == "pageLink"));
        if !ok_shelf || row.map(|r| r["attrs"]["pageId"] == b.id.as_str()) != Some(true) {
            failures.push(format!("{label}: the row inside the shelf: {}", a.content));
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}
