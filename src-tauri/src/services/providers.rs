use crate::errors::AppError;
use crate::models::AiProvider;

#[derive(Default)]
pub struct ProvidersService;

impl ProvidersService {
    pub fn list_providers(&self) -> Result<Vec<AiProvider>, AppError> {
        Err(AppError::FeatureNotImplemented(
            "provider storage starts in Phase 5".to_string(),
        ))
    }

    pub fn get_provider(&self, _id: String) -> Result<AiProvider, AppError> {
        Err(AppError::FeatureNotImplemented(
            "provider storage starts in Phase 5".to_string(),
        ))
    }

    /// Safe stub — returns a fixed string, never touches a real provider.
    pub fn test_provider_placeholder(&self, _id: String) -> Result<String, AppError> {
        Ok("provider test not yet implemented".to_string())
    }
}
