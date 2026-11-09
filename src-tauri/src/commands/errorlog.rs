//! The error log: what the UI may report, and the two things the Settings screen does with it.
//! None of these needs the vault to be open, because the log works before the unlock too.

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::error::{Error, Result};
use crate::logging::{self, FrontendKind, FrontendReport, LogInfo};
use crate::state::AppState;

/// Only a closed class and bounded location reach the log. Legacy `message` fields are ignored
/// by serde, so even an old caller cannot forward free text to the logging sink.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendError {
    pub kind: FrontendKind,
    pub source: Option<String>,
    pub line: Option<f64>,
}

/// `async` so that a write never runs on the thread that draws the windows.
#[tauri::command]
pub async fn log_frontend_error(report: FrontendError) {
    logging::frontend_error(&report.for_log());
}

impl FrontendError {
    pub(crate) fn for_log(&self) -> FrontendReport<'_> {
        let line = self.line.filter(|l| l.is_finite() && *l >= 0.0).map(|l| l.min(u32::MAX as f64) as u32);
        FrontendReport { kind: self.kind, source: self.source.as_deref().unwrap_or(""), line }
    }
}

/// Where the log is, how big it is and when the last entry was written.
#[tauri::command]
pub fn log_info(state: State<'_, AppState>) -> LogInfo {
    logging::info(&state.paths)
}

/// Opens the logs folder in Explorer. The path is never taken from the UI.
#[tauri::command]
pub fn open_log_folder(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    open_log_folder_person(app, state)
}

fn open_log_folder_person(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let folder = logging::folder(&state.paths);
    // A person may open it before anything was logged.
    std::fs::create_dir_all(&folder)?;
    app.opener().open_path(folder.display().to_string(), None::<&str>).map_err(|e| Error::Other(e.to_string()))
}
