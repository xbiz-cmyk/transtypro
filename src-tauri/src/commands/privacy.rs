use crate::errors::AppError;
use crate::models::{PrivacyDecision, PrivacyOperation, PrivacySummary};
use crate::services::PrivacyService;

#[tauri::command]
pub fn get_privacy_status() -> Result<PrivacySummary, AppError> {
    PrivacyService.get_privacy_status()
}

#[tauri::command]
pub fn enforce_privacy_preview(op: PrivacyOperation) -> Result<PrivacyDecision, AppError> {
    PrivacyService.enforce_privacy_preview(op)
}
