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

fn scope_of(ids: &Ids, name: &str) -> Scope {
    match name {
        "one page" => Scope::Pages(vec![ids.j1.clone()]),
        "section" => Scope::Section(ids.sec_j.clone()),
        "notebook" => Scope::Notebook(ids.nb_a.clone()),
        "two sections" => Scope::Sections(vec![ids.sec_j.clone(), ids.sec_i.clone()]),
        _ => Scope::Notebooks(vec![ids.nb_a.clone(), ids.nb_b.clone()]),
    }
}

#[test]
#[ignore = "work in progress: the matrix run did not finish within 10 minutes under Wine, cause not found yet (see the resume note)"]
fn every_format_scope_and_output_comes_back_in() {
    let root = temp();
    let src = new_vault(&root.join("src-data"));
    let (ids, cases) = source(&src);
    let mut failures: Vec<String> = Vec::new();
    let mut table = vec![format!("{:<8} {:<14} {:<6} {:>5} {:>8} {:>6}  notes", "format", "scope", "output", "pages", "sections", "images")];
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
                let summary = match import_from(&dst, &[dest.clone()], false, &root.join("src-data"), 1_700_000_000_000) {
                    Ok(s) => s,
                    Err(e) => {
                        failures.push(format!("{label}: import failed: {e}"));
                        continue;
                    }
                };
                let pages: Vec<Page> = dst.with(|s| Ok(s.items().filter_map(|i| if let Item::Page(p) = i { Some(p.clone()) } else { None }).collect())).unwrap();
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
                    "{:<8} {:<14} {:<6} {:>5} {:>8} {:>6}  {}",
                    format.ext(),
                    scope_name,
                    out,
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
