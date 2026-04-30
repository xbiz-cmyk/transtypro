use crate::errors::AppError;
use crate::models::{AppSettings, PrivacyDecision, PrivacyOperation};

#[derive(Default)]
pub struct PrivacyService;

impl PrivacyService {
    /// Returns a safe static default — real enforcement wired in Phase 8.
    pub fn get_privacy_status(&self) -> Result<AppSettings, AppError> {
        Ok(AppSettings {
            active_mode: "Smart Mode".to_string(),
            local_only_mode: false,
            theme: "dark".to_string(),
            retention_days: 30,
            audio_history_enabled: false,
            clipboard_restore_enabled: false,
        })
    }

    /// Contract stub — real enforcement wired in Phase 8.
    pub fn enforce_privacy_preview(
        &self,
        _op: PrivacyOperation,
    ) -> Result<PrivacyDecision, AppError> {
        Ok(PrivacyDecision {
            allowed: true,
            reason: "preview only".to_string(),
        })
    }
}
