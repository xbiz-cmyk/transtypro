use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::{DiagnosticReport, RetentionResult};
use crate::services::{AudioState, DiagnosticsService, RetentionService};

/// Run all diagnostic checks and return the full report.
///
/// Diagnostics data is returned via Tauri IPC only.
/// It is never sent externally, logged to a file, or exported.
/// No API keys are read from the OS keychain.
#[tauri::command]
pub fn run_diagnostics(
    state: State<'_, AppState>,
    audio_state: State<'_, AudioState>,
) -> Result<DiagnosticReport, AppError> {
    DiagnosticsService::new(state.db.clone(), audio_state.audio_dir.clone()).run_diagnostics()
}

/// Apply the configured retention policy: delete old history entries and
/// stale WAV files from the audio directory.
///
/// Returns counts of deleted records and files.
/// This is a local-only operation — no network calls are made.
#[tauri::command]
pub fn apply_retention_policy(
    state: State<'_, AppState>,
    audio_state: State<'_, AudioState>,
) -> Result<RetentionResult, AppError> {
    RetentionService::new(state.db.clone(), audio_state.audio_dir.clone()).apply_all()
}
