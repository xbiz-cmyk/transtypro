use crate::db::AppState;
use crate::errors::AppError;
use crate::models::CleanupResult;
use crate::services::CleanupService;
use tauri::State;

#[tauri::command]
pub fn cleanup_text(
    raw_text: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<CleanupResult, AppError> {
    CleanupService::new(state.db.clone()).cleanup(&raw_text, &provider_id)
}
