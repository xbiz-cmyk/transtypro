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

/// Application-level settings stored per user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub active_mode: String,
    pub local_only_mode: bool,
    pub theme: String,
    pub retention_days: u32,
    pub audio_history_enabled: bool,
    pub clipboard_restore_enabled: bool,
}

/// A named dictation mode (Smart, Raw, Clean, Email, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationMode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub active: bool,
    /// True if this mode ships with the app and cannot be deleted.
    pub builtin: bool,
}

/// A vocabulary substitution rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntry {
    pub id: String,
    pub term: String,
    pub replacement: String,
    pub category: String,
    pub enabled: bool,
}

/// A single transcription history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub mode_used: String,
    /// ISO 8601 timestamp string — no chrono dependency required yet.
    pub timestamp: String,
    pub was_inserted: bool,
}

/// Configuration for an AI cleanup or transcription provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    /// "openai_compatible" | "ollama" | "local"
    pub provider_type: String,
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
    pub use_for_cleanup: bool,
    pub use_for_transcription: bool,
    /// True if an API key has been stored; the key itself is never held here.
    pub api_key_set: bool,
}

/// Result of a single diagnostic check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    /// "pass" | "fail" | "pending"
    pub status: String,
    pub message: String,
}

/// Full diagnostics report returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub checks: Vec<DiagnosticCheck>,
    /// ISO 8601 timestamp string.
    pub generated_at: String,
}

/// Current privacy state returned to the frontend status bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySummary {
    pub local_only_mode: bool,
    /// Days to retain audio files; 0 means delete immediately.
    pub audio_retention_days: u32,
    pub history_retention_days: u32,
    pub cloud_allowed: bool,
    pub reason: String,
}

/// Describes a privacy-relevant operation to evaluate before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyOperation {
    /// "transcribe" | "cleanup" | "export" | "cloud_call"
    pub operation_type: String,
    pub provider_id: Option<String>,
}

/// Result of a privacy enforcement check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyDecision {
    pub allowed: bool,
    pub reason: String,
}
