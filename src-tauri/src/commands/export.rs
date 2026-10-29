//! "Export…": pages, a section or a notebook to Markdown, PDF or Word (see `crate::export`).
//! The UI sends only ids, the format and two switches; the save dialog is opened here, and the
//! file is built and written by the core. The UI never gives a path.

use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::limit;
use crate::error::{Error, Result};
use crate::export::{self, ExportSummary, Format, Options, Scope};
use crate::model::Item;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    /// "pages", "section" or "notebook".
    scope: String,
    /// The page ids (for "pages") or the one section or notebook id.
    ids: Vec<String>,
    /// "markdown", "pdf" or "docx".
    format: String,
    include_subpages: bool,
    #[serde(default)]
    include_trashed: bool,
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
        "section" | "notebook" if ids.len() == 1 => {
            let id = ids.into_iter().next().unwrap_or_default();
            Ok(if scope == "section" { Scope::Section(id) } else { Scope::Notebook(id) })
        }
        _ => Err(Error::invalid("Unknown export choice.")),
    }
}

/// Asks where to save, then writes the export. `None` if the save dialog was cancelled.
/// Runs on a worker thread: the dialog waits for the person, and a big notebook takes a moment.
#[tauri::command(async)]
pub fn export_notes(app: AppHandle, state: State<'_, AppState>, request: ExportRequest) -> Result<Option<ExportSummary>> {
    let format = Format::parse(&request.format)?;
    let scope = parse_scope(&request.scope, request.ids)?;
    let opts = Options { include_subpages: request.include_subpages, include_trashed: request.include_trashed };
    // Check that there is something to export before asking for a file name.
    let suggested = state.with(|s| {
        let items: Vec<&Item> = s.items().collect();
        export::collect(&items, &scope, &opts, &|_| None)?;
        export::suggested_name(&items, &scope)
    })?;
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Export")
        .set_file_name(format!("{suggested}.{}", format.ext()))
        .add_filter(format.label(), &[format.ext()])
        .blocking_save_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let dest = export::checked_destination(&chosen, format, &state.paths.data)?;
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
        assert!(parse_scope("pages", vec![]).is_err());
        assert!(parse_scope("everything", vec!["a".into()]).is_err());
        assert!(parse_scope("pages", vec!["x".repeat(65)]).is_err());
        let r: ExportRequest = serde_json::from_str(r#"{"scope":"pages","ids":["a"],"format":"pdf","includeSubpages":true}"#).unwrap();
        assert!(!r.include_trashed, "pages in the trash stay out unless asked for");
    }
}
