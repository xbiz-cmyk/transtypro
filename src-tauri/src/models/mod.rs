/// transtypro — Data models.
///
/// Shared Rust data structures used across commands, services, and repositories.
use serde::{Deserialize, Serialize};

/// Summary of the application status for the home page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSummary {
    /// Current privacy mode: "local-only" or "cloud-enabled"
    pub privacy_mode: String,
    /// Whether a transcription model is configured and ready
    pub transcription_ready: bool,
    /// Name of the active cleanup provider, if any
    pub cleanup_provider: Option<String>,
    /// Current dictation mode name
    pub active_mode: String,
    /// Total number of history entries
    pub history_count: u32,
}
