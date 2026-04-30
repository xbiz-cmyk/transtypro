use crate::models::{PrivacyDecision, PrivacyOperation, PrivacySummary};
use crate::services::PrivacyService;

#[tauri::command]
pub fn get_privacy_status() -> Result<PrivacySummary, String> {
    PrivacyService
        .get_privacy_status()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn enforce_privacy_preview(op: PrivacyOperation) -> Result<PrivacyDecision, String> {
    PrivacyService
        .enforce_privacy_preview(op)
        .map_err(|e| e.to_string())
}
