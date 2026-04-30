use crate::errors::AppError;
use crate::models::AppSettings;

#[derive(Default)]
pub struct SettingsService;

impl SettingsService {
    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        Err(AppError::FeatureNotImplemented(
            "settings storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn update_settings(&self, _settings: AppSettings) -> Result<(), AppError> {
        Err(AppError::FeatureNotImplemented(
            "settings storage starts in Phase 2 storage".to_string(),
        ))
    }
}
