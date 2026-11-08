//! "Export…": pages, sections or notebooks to HTML, Markdown, PDF or Word, as one document or as
//! separate files in a ZIP (see `crate::export`). The UI sends only ids, the format and a few
//! switches; the save dialog is opened here, and the file is built and written by the core. The
//! UI never gives a path.

use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::limit;
use crate::error::{Error, Result};
use crate::export::{self, ExportSummary, Format, Options, Scope, ZipLimits};
use crate::model::Item;
use crate::state::AppState;
use crate::vault::Session;

/// Filter the full snapshot before selection, link lookup and inline database
/// resolution, so inherited bin membership also excludes rows and pictures.
fn export_items(s: &Session, include_trashed: bool) -> Vec<&Item> {
    s.items().filter(|item| include_trashed || !s.in_bin(item.id())).collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    /// "pages", "section", "sections", "notebook" or "notebooks".
    scope: String,
    /// The page ids (for "pages"), the section ids or the notebook ids.
    ids: Vec<String>,
    /// "html", "markdown", "pdf" or "docx".
    format: String,
    include_subpages: bool,
    #[serde(default)]
    include_trashed: bool,
    /// "single" (one document, the default) or "zip" (one file per page in a ZIP).
    #[serde(default)]
    output: Option<String>,
}

fn parse_scope(scope: &str, ids: Vec<String>) -> Result<Scope> {
    if ids.is_empty() || ids.len() > 10_000 {
        return Err(Error::invalid("Choose what to export."));
    }
    for id in &ids {
        limit("Id", id, 64)?;
    }
    match scope {
        "pages" => Ok(Scope::Pages(ids)),
        "sections" | "notebooks" if ids.len() <= export::MAX_CHOSEN => Ok(if scope == "sections" { Scope::Sections(ids) } else { Scope::Notebooks(ids) }),
        "section" | "notebook" if ids.len() == 1 => {
            let id = ids.into_iter().next().unwrap_or_default();
            Ok(if scope == "section" { Scope::Section(id) } else { Scope::Notebook(id) })
        }
        _ => Err(Error::invalid("Unknown export choice.")),
    }
}

/// `true` for a ZIP of separate files.
fn parse_output(output: Option<&str>) -> Result<bool> {
    match output {
        None | Some("single") => Ok(false),
        Some("zip") => Ok(true),
        Some(_) => Err(Error::invalid("Unknown export choice.")),
    }
}

/// Asks where to save, then writes the export. `None` if the save dialog was cancelled.
/// Runs on a worker thread: the dialog waits for the person, and a big notebook takes a moment.
#[tauri::command(async)]
pub fn export_notes(app: AppHandle, state: State<'_, AppState>, request: ExportRequest) -> Result<Option<ExportSummary>> {
    export_notes_person(app, state, request)
}

fn export_notes_person(app: AppHandle, state: State<'_, AppState>, request: ExportRequest) -> Result<Option<ExportSummary>> {
    let format = Format::parse(&request.format)?;
    let zip = parse_output(request.output.as_deref())?;
    let scope = parse_scope(&request.scope, request.ids)?;
    let mut opts = Options { include_subpages: request.include_subpages, include_trashed: request.include_trashed, ..Options::default() };
    if format == Format::Html && !zip {
        opts.max_image_bytes = export::html::MAX_PICTURE_BYTES;
    }
    // Check that there is something to export before asking for a file name.
    let suggested = state.with(|s| {
        let items = export_items(s, opts.include_trashed);
        if zip {
            export::collect_zip(&items, &scope, &opts, format)?;
        } else {
            export::collect(&items, &scope, &opts, &|_| None)?;
        }
        export::suggested_name(&items, &scope)
    })?;
    let (ext, label) = if zip { ("zip", "ZIP archive") } else { (format.ext(), format.label()) };
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Export")
        .set_file_name(format!("{suggested}.{ext}"))
        .add_filter(label, &[ext])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let dest = export::checked_destination(&chosen, ext, &state.paths.data)?;
    if zip {
        // The layout is made while the vault is held; each page's pictures are read (one short
        // lock each) while the ZIP is built, so a large export neither blocks saving nor holds
        // every picture in memory at once.
        let zdoc = state.with(|s| {
            let items = export_items(s, opts.include_trashed);
            export::collect_zip(&items, &scope, &opts, format)
        })?;
        return export::write_zip(&dest, format, zdoc, &|id| state.with(|s| s.get_attachment(id)).ok(), ZipLimits::default()).map(Some);
    }
    // The pages and pictures are copied out while the vault is held; the file is built and
    // written afterwards, so a large export does not block saving.
    let doc = state.with(|s| {
        let items = export_items(s, opts.include_trashed);
        export::collect(&items, &scope, &opts, &|id| s.get_attachment(id).ok())
    })?;
    export::write(&dest, format, &doc, &state.paths.data).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_export_excludes_inherited_bin_pages_pictures_and_inline_databases() {
        use crate::actions::Actor;
        use crate::model::new_id;
        use serde_json::json;
        let (mut s, paths) = crate::changes::tests::fixture();
        let nb = s.items().find(|i| matches!(i, Item::Notebook(_))).unwrap().id().to_string();
        let sec = s.items().find(|i| matches!(i, Item::Section(_))).unwrap().id().to_string();
        let Item::Page(mut hidden) = s.items().find(|i| matches!(i, Item::Page(_))).unwrap().clone() else { unreachable!() };
        let hidden_id = hidden.id.clone();
        let db = s.items().find(|i| matches!(i, Item::Database(_))).unwrap().id().to_string();
        let picture = s.put_attachment("image/png", b"throwaway picture").unwrap();
        hidden.content = json!({"type":"doc","content":[
            {"type":"image","attrs":{"src":format!("http://clatt.localhost/{picture}")}},
            {"type":"databaseBlock","attrs":{"databaseId":db}}
        ]}).to_string();
        s.put(Item::Page(hidden.clone())).unwrap();
        let Item::Section(mut live_section) = s.get(&sec).unwrap().clone() else { unreachable!() };
        live_section.id = new_id();
        let live_sec = live_section.id.clone();
        s.put(Item::Section(live_section)).unwrap();
        let mut live_page = hidden;
        live_page.id = new_id(); live_page.section_id = live_sec;
        live_page.title = "live export page".into();
        // A live page points to a database that will be hidden by its section.
        live_page.content = json!({"type":"doc","content":[
            {"type":"databaseBlock","attrs":{"databaseId":db}}
        ]}).to_string();
        s.put(Item::Page(live_page)).unwrap();
        super::super::notes::delete_section_core(&mut s, &Actor::Person, &paths, sec).unwrap();
        let items = export_items(&s, false);
        assert!(!items.iter().any(|i| i.id() == hidden_id || i.id() == db || matches!(i, Item::Row(_))));
        let scope = Scope::Notebook(nb.clone());
        let doc = export::collect(&items, &scope, &Options::default(), &|_| panic!("hidden pictures must not be read")).unwrap();
        let html = export::html::render(&doc).unwrap();
        assert!(html.contains("live export page"));
        assert!(!html.contains("exact page") && !html.contains("exact row"));
        assert!(doc.images.is_empty());
        assert!(!doc.parts.iter().any(|part| matches!(part, export::Part::Page { blocks, .. }
            if blocks.iter().any(|b| matches!(b, export::doc::Block::Database { table: Some(_), .. })))));
        let zip = export::collect_zip(&items, &scope, &Options::default(), Format::Html).unwrap();
        assert_eq!(zip.files.len(), 1);
        assert!(export_items(&s, true).iter().any(|i| i.id() == hidden_id));
        super::super::notes::delete_notebook_core(&mut s, &Actor::Person, &paths, nb).unwrap();
        let items = export_items(&s, false);
        assert!(export::collect(&items, &scope, &Options::default(), &|_| panic!("binned notebook pictures must not be read")).is_err());
        assert!(export::collect_zip(&items, &scope, &Options::default(), Format::Html).is_err());
    }

    #[test]
    fn requests_are_checked() {
        assert_eq!(parse_scope("pages", vec!["a".into(), "b".into()]).unwrap(), Scope::Pages(vec!["a".into(), "b".into()]));
        assert_eq!(parse_scope("notebook", vec!["n".into()]).unwrap(), Scope::Notebook("n".into()));
        assert!(parse_scope("section", vec!["a".into(), "b".into()]).is_err());
        assert_eq!(parse_scope("sections", vec!["a".into(), "b".into()]).unwrap(), Scope::Sections(vec!["a".into(), "b".into()]));
        assert_eq!(parse_scope("notebooks", vec!["a".into()]).unwrap(), Scope::Notebooks(vec!["a".into()]));
        assert!(parse_scope("notebooks", (0..1001).map(|i| i.to_string()).collect()).is_err(), "counts are capped");
        assert!(parse_scope("sections", vec![]).is_err());
        assert!(parse_scope("pages", vec![]).is_err());
        assert!(parse_scope("everything", vec!["a".into()]).is_err());
        assert!(parse_scope("pages", vec!["x".repeat(65)]).is_err());
        let r: ExportRequest = serde_json::from_str(r#"{"scope":"pages","ids":["a"],"format":"pdf","includeSubpages":true}"#).unwrap();
        assert!(!r.include_trashed, "pages in the trash stay out unless asked for");
        assert!(!parse_output(r.output.as_deref()).unwrap(), "one document unless a ZIP is asked for");
        assert!(parse_output(Some("zip")).unwrap());
        assert!(parse_output(Some("tar")).is_err());
        assert!(Format::parse("html").is_ok());
    }
}
