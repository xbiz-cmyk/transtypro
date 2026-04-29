/// transtypro — Tauri command wrappers.
///
/// Commands are thin wrappers that delegate to services.
/// They must not contain business logic.
use crate::errors::AppError;

/// Verify frontend-backend IPC communication.
#[tauri::command]
pub fn ping() -> Result<String, AppError> {
    Ok("pong".to_string())
}

/// Return the current application version.
#[tauri::command]
pub fn get_app_version() -> Result<String, AppError> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Placeholder: get application status summary for the home page.
#[tauri::command]
pub fn get_status_summary() -> Result<crate::models::StatusSummary, AppError> {
    Ok(crate::models::StatusSummary {
        privacy_mode: "local-only".to_string(),
        transcription_ready: false,
        cleanup_provider: None,
        active_mode: "Smart Mode".to_string(),
        history_count: 0,
    })
}
