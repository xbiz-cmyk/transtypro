use crate::errors::AppError;
use crate::models::{PrivacyDecision, PrivacyOperation, PrivacySummary};

#[derive(Default)]
pub struct PrivacyService;

impl PrivacyService {
    /// Returns a safe static default — real enforcement wired in Phase 8.
    pub fn get_privacy_status(&self) -> Result<PrivacySummary, AppError> {
        Ok(PrivacySummary {
            local_only_mode: false,
            audio_retention_days: 0,
            history_retention_days: 30,
            cloud_allowed: true,
            reason: "static placeholder until storage is wired".to_string(),
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
