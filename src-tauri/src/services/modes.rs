use crate::errors::AppError;
use crate::models::DictationMode;

#[derive(Default)]
pub struct ModesService;

impl ModesService {
    pub fn list_modes(&self) -> Result<Vec<DictationMode>, AppError> {
        Err(AppError::FeatureNotImplemented(
            "modes storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn get_mode(&self, _id: String) -> Result<DictationMode, AppError> {
        Err(AppError::FeatureNotImplemented(
            "modes storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn create_mode(&self, _mode: DictationMode) -> Result<DictationMode, AppError> {
        Err(AppError::FeatureNotImplemented(
            "modes storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn update_mode(&self, _mode: DictationMode) -> Result<DictationMode, AppError> {
        Err(AppError::FeatureNotImplemented(
            "modes storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn delete_mode(&self, _id: String) -> Result<(), AppError> {
        Err(AppError::FeatureNotImplemented(
            "modes storage starts in Phase 2 storage".to_string(),
        ))
    }
}
