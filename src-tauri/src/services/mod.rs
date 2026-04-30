/// transtypro — Service interfaces.
///
/// Services contain business logic. Tauri commands delegate to services.
/// Phase 0 stubs return `FeatureNotImplemented`.
/// Phase 2 adds service structs for all core domains.
/// Phase 2 storage and beyond wire real implementations.
use crate::errors::AppError;

pub mod diagnostics;
pub mod history;
pub mod modes;
pub mod privacy;
pub mod providers;
pub mod settings;
pub mod vocabulary;

pub use diagnostics::DiagnosticsService;
pub use history::HistoryService;
pub use modes::ModesService;
pub use privacy::PrivacyService;
pub use providers::ProvidersService;
pub use settings::SettingsService;
pub use vocabulary::VocabularyService;

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
