use crate::errors::AppError;
use crate::models::HistoryEntry;

#[derive(Default)]
pub struct HistoryService;

impl HistoryService {
    pub fn list_history(&self) -> Result<Vec<HistoryEntry>, AppError> {
        Err(AppError::FeatureNotImplemented(
            "history storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn get_entry(&self, _id: String) -> Result<HistoryEntry, AppError> {
        Err(AppError::FeatureNotImplemented(
            "history storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn delete_entry(&self, _id: String) -> Result<(), AppError> {
        Err(AppError::FeatureNotImplemented(
            "history storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn clear_history(&self) -> Result<(), AppError> {
        Err(AppError::FeatureNotImplemented(
            "history storage starts in Phase 2 storage".to_string(),
        ))
    }
}
