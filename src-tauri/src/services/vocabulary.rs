use crate::errors::AppError;
use crate::models::VocabularyEntry;

#[derive(Default)]
pub struct VocabularyService;

impl VocabularyService {
    pub fn list_vocabulary(&self) -> Result<Vec<VocabularyEntry>, AppError> {
        Err(AppError::FeatureNotImplemented(
            "vocabulary storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn add_entry(&self, _entry: VocabularyEntry) -> Result<VocabularyEntry, AppError> {
        Err(AppError::FeatureNotImplemented(
            "vocabulary storage starts in Phase 2 storage".to_string(),
        ))
    }

    pub fn delete_entry(&self, _id: String) -> Result<(), AppError> {
        Err(AppError::FeatureNotImplemented(
            "vocabulary storage starts in Phase 2 storage".to_string(),
        ))
    }
}
