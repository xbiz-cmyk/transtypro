/// transtypro — Service interfaces.
///
/// Services contain business logic. Tauri commands delegate to services.
/// Each service will be implemented in its own submodule in later phases.
///
/// Phase 0: Only interface stubs that return `FeatureNotImplemented` errors.
/// Phase 2+: Real implementations.
use crate::errors::AppError;

/// Placeholder for the transcription service (Phase 3-4).
pub fn transcribe_audio(_audio_path: &str) -> Result<String, AppError> {
    Err(AppError::FeatureNotImplemented(
        "audio transcription starts in Phase 4".to_string(),
    ))
}

/// Placeholder for the cleanup service (Phase 5).
pub fn cleanup_text(_raw_text: &str, _mode: &str) -> Result<String, AppError> {
    Err(AppError::FeatureNotImplemented(
        "text cleanup starts in Phase 5".to_string(),
    ))
}

/// Placeholder for the text insertion service (Phase 6).
pub fn insert_text(_text: &str) -> Result<(), AppError> {
    Err(AppError::FeatureNotImplemented(
        "text insertion starts in Phase 6".to_string(),
    ))
}
