/// transtypro — Application error types.
///
/// All errors returned from Tauri commands and services should use `AppError`.
/// Do not use `todo!()`, `unimplemented!()`, or `panic!()` in callable code paths.
/// For features not yet implemented, use `AppError::FeatureNotImplemented`.
use serde::Serialize;

/// Typed application error for all backend operations.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A feature that is planned but not yet implemented in this phase.
    #[error("Feature not implemented: {0}")]
    FeatureNotImplemented(String),

    /// A generic internal error with a human-readable message.
    #[error("Internal error: {0}")]
    Internal(String),

    /// An I/O error from the filesystem or other OS operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input failed validation rules.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// A persistence or storage operation failed.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// An operation was blocked by the privacy policy.
    #[error("Privacy blocked: {0}")]
    PrivacyBlocked(String),

    /// An AI provider is unavailable or misconfigured.
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    /// A diagnostics check or report operation failed.
    #[error("Diagnostics error: {0}")]
    DiagnosticsError(String),

    /// An audio recording or device operation failed.
    #[error("Audio error: {0}")]
    AudioError(String),
}

/// Serialize AppError for Tauri's IPC layer.
/// Tauri requires command errors to implement `Serialize`.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
