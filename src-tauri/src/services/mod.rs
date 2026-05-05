/// transtypro — Service interfaces.
///
/// Services contain business logic. Tauri commands delegate to services.
/// Phase 0 stubs return `FeatureNotImplemented`.
/// Phase 2 adds service structs for all core domains.
/// Phase 2 storage and beyond wire real implementations.
/// Phase 3 adds AudioService and AudioState for microphone recording.
/// Phase 4 adds TranscriptionService for local whisper.cpp execution.
use crate::errors::AppError;

pub mod audio;
pub mod diagnostics;
pub mod history;
pub mod modes;
pub mod privacy;
pub mod providers;
pub mod settings;
pub mod transcription;
pub mod vocabulary;

pub use audio::{AudioService, AudioState};
pub use diagnostics::DiagnosticsService;
pub use history::HistoryService;
pub use modes::ModesService;
pub use privacy::PrivacyService;
pub use providers::ProvidersService;
pub use settings::SettingsService;
pub use transcription::TranscriptionService;
pub use vocabulary::VocabularyService;

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
