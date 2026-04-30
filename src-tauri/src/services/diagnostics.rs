use crate::errors::AppError;
use crate::models::{DiagnosticCheck, DiagnosticReport};

#[derive(Default)]
pub struct DiagnosticsService;

impl DiagnosticsService {
    /// Returns a static diagnostic report — real checks wired in Phase 8.
    pub fn run_diagnostics_placeholder(&self) -> Result<DiagnosticReport, AppError> {
        Ok(DiagnosticReport {
            checks: vec![
                DiagnosticCheck {
                    name: "backend_alive".to_string(),
                    status: "pass".to_string(),
                    message: "Rust backend is running".to_string(),
                },
                DiagnosticCheck {
                    name: "storage".to_string(),
                    status: "pending".to_string(),
                    message: "SQLite not yet wired".to_string(),
                },
            ],
            generated_at: "2026-04-30T00:00:00Z".to_string(),
        })
    }
}
