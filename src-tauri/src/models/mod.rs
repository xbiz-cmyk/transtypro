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

// ─────────────────────────── Phase 3: Audio ──────────────────────────────────

/// Information about an available microphone input device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneInfo {
    pub name: String,
    pub is_default: bool,
}

/// Live recording state returned by get_recording_status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub selected_microphone: Option<String>,
    /// RMS level of recent samples, range 0.0–1.0.
    pub level_rms: f32,
    /// Total samples collected in the buffer (all channels combined).
    pub sample_count: u64,
}

/// Result returned after a successful stop_recording call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResult {
    /// Absolute path to the written WAV file.
    pub file_path: String,
    pub duration_ms: u64,
    pub sample_rate: u32,
    /// Always 1 — WAV output is mono regardless of capture channels.
    pub channels: u16,
}
