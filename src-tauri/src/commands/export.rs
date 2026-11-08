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
        let items: Vec<&Item> = s.items().collect();
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
            let items: Vec<&Item> = s.items().collect();
            export::collect_zip(&items, &scope, &opts, format)
        })?;
        return export::write_zip(&dest, format, zdoc, &|id| state.with(|s| s.get_attachment(id)).ok(), ZipLimits::default()).map(Some);
    }
    // The pages and pictures are copied out while the vault is held; the file is built and
    // written afterwards, so a large export does not block saving.
    let doc = state.with(|s| {
        let items: Vec<&Item> = s.items().collect();
        export::collect(&items, &scope, &opts, &|id| s.get_attachment(id).ok())
    })?;
    export::write(&dest, format, &doc, &state.paths.data).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

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
