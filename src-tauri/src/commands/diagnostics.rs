use crate::models::DiagnosticReport;
use crate::services::DiagnosticsService;

#[tauri::command]
pub fn run_diagnostics_placeholder() -> Result<DiagnosticReport, String> {
    DiagnosticsService
        .run_diagnostics_placeholder()
        .map_err(|e| e.to_string())
}
