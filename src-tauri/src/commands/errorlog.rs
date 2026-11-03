//! The error log: what the UI may report, and the two things the Settings screen does with it.
//! None of these needs the vault to be open, because the log works before the unlock too.

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::error::{Error, Result};
use crate::logging::{self, FrontendReport, LogInfo};
use crate::state::AppState;

/// A problem the UI noticed. Every field is cleaned again in `logging` before anything is written.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendError {
    pub kind: String,
    pub message: String,
    pub source: Option<String>,
    pub line: Option<f64>,
}

/// `async` so that a write never runs on the thread that draws the windows.
#[tauri::command]
pub async fn log_frontend_error(report: FrontendError) {
    let line = report.line.filter(|l| l.is_finite() && *l >= 0.0).map(|l| l.min(u32::MAX as f64) as u32);
    logging::frontend_error(&FrontendReport {
        kind: &report.kind,
        message: &report.message,
        source: report.source.as_deref().unwrap_or(""),
        line,
    });
}

/// Where the log is, how big it is and when the last entry was written.
#[tauri::command]
pub fn log_info(state: State<'_, AppState>) -> LogInfo {
    logging::info(&state.paths)
}

/// Opens the logs folder in Explorer. The path is never taken from the UI.
#[tauri::command]
pub fn open_log_folder(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let folder = logging::folder(&state.paths);
    // A person may open it before anything was logged.
    std::fs::create_dir_all(&folder)?;
    app.opener().open_path(folder.display().to_string(), None::<&str>).map_err(|e| Error::Other(e.to_string()))
}
