use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::{PrivacyDecision, PrivacyOperation, PrivacySummary};
use crate::services::PrivacyService;

#[tauri::command]
pub fn get_privacy_status(state: State<'_, AppState>) -> Result<PrivacySummary, AppError> {
    PrivacyService::new(state.db.clone()).get_privacy_status()
}

#[tauri::command]
pub fn enforce_privacy_preview(
    op: PrivacyOperation,
    state: State<'_, AppState>,
) -> Result<PrivacyDecision, AppError> {
    PrivacyService::new(state.db.clone()).enforce_privacy_preview(op)
}
