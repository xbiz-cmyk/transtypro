use crate::errors::AppError;
use crate::models::DiagnosticReport;
use crate::services::DiagnosticsService;

#[tauri::command]
pub fn run_diagnostics_placeholder() -> Result<DiagnosticReport, AppError> {
    DiagnosticsService.run_diagnostics_placeholder()
}
