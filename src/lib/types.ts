/**
 * transtypro — Shared TypeScript types.
 *
 * All types used across the frontend should be defined here.
 * Keep in sync with Rust models in src-tauri/src/models/.
 */

/** Application status summary returned by the backend. */
export interface StatusSummary {
  /** Current privacy mode: "local-only" or "cloud-enabled" */
  privacy_mode: string;
  /** Whether a transcription model is configured and ready */
  transcription_ready: boolean;
  /** Name of the active cleanup provider, if any */
  cleanup_provider: string | null;
  /** Current dictation mode name */
  active_mode: string;
  /** Total number of history entries */
  history_count: number;
}

/** Navigation item for the sidebar. */
export interface NavItem {
  /** Route path */
  path: string;
  /** Display label */
  label: string;
  /** Icon name or identifier */
  icon: string;
}

// ---------------------------------------------------------------------------
// AppSettings — mirrors Rust AppSettings struct (src-tauri/src/models/mod.rs)
// ---------------------------------------------------------------------------

export interface AppSettings {
  /** Currently active dictation mode name */
  active_mode: string;
  /** Whether local-only mode is enabled (blocks all cloud calls) */
  local_only_mode: boolean;
  /** UI theme: "dark" | "light" | "system" */
  theme: string;
  /** Number of days to retain history (0 = forever) */
  retention_days: number;
  /** Whether to persist audio recordings */
  audio_history_enabled: boolean;
  /** Whether to restore clipboard after dictation */
  clipboard_restore_enabled: boolean;
}

// ---------------------------------------------------------------------------
// HistoryEntry — mirrors Rust HistoryEntry struct
// ---------------------------------------------------------------------------

export interface HistoryEntry {
  /** UUID string */
  id: string;
  /** Raw transcript text */
  raw_text: string;
  /** Cleaned/formatted text */
  cleaned_text: string;
  /** Dictation mode used */
  mode_used: string;
  /** ISO-8601 timestamp string */
  timestamp: string;
  /** Whether the result was inserted into an external app */
  was_inserted: boolean;
}

// ---------------------------------------------------------------------------
// DictationMode — mirrors Rust DictationMode struct
// ---------------------------------------------------------------------------

export interface DictationMode {
  /** UUID string */
  id: string;
  name: string;
  description: string;
  /** System prompt template for cleanup */
  system_prompt: string;
  /** Whether this is the currently active mode */
  active: boolean;
  /** Whether this is a built-in system mode (non-deletable) */
  builtin: boolean;
}

// ---------------------------------------------------------------------------
// VocabularyEntry — mirrors Rust VocabularyEntry struct
// ---------------------------------------------------------------------------

export interface VocabularyEntry {
  /** UUID string */
  id: string;
  /** The spoken term */
  term: string;
  /** The replacement text to insert */
  replacement: string;
  /** Category label (e.g. "Technical", "Personal") */
  category: string;
  /** Whether this entry is active */
  enabled: boolean;
}

// ---------------------------------------------------------------------------
// AiProvider — mirrors Rust AiProvider struct
// ---------------------------------------------------------------------------

export interface AiProvider {
  /** UUID string */
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  /** Model name/identifier */
  model: string;
  /** Whether this provider is active */
  enabled: boolean;
  use_for_cleanup: boolean;
  use_for_transcription: boolean;
  /** Whether an API key is stored (key itself never sent to frontend) */
  api_key_set: boolean;
}

// ---------------------------------------------------------------------------
// ModelEntry — local whisper-compatible model (UI-only, no backend contract yet)
// ---------------------------------------------------------------------------

export interface ModelEntry {
  id: number;
  name: string;
  /** Filesystem path to the model file */
  path: string;
  /** File size in bytes */
  size_bytes: number;
  /** Model language capability (e.g. "multilingual", "en") */
  language: string;
  /** Whether this model is selected for transcription */
  is_active: boolean;
}

// ---------------------------------------------------------------------------
// DiagnosticCheck / DiagnosticReport — mirrors Rust structs
// ---------------------------------------------------------------------------

export interface DiagnosticCheck {
  name: string;
  /** Status string: "pass", "fail", "pending", etc. */
  status: string;
  message: string;
}

export interface DiagnosticReport {
  checks: DiagnosticCheck[];
  /** ISO-8601 timestamp string */
  generated_at: string;
}

// ---------------------------------------------------------------------------
// PrivacySummary — mirrors Rust PrivacySummary struct
// ---------------------------------------------------------------------------

export interface PrivacySummary {
  local_only_mode: boolean;
  audio_retention_days: number;
  history_retention_days: number;
  cloud_allowed: boolean;
  reason: string;
}

// ---------------------------------------------------------------------------
// PrivacyOperation / PrivacyDecision — mirrors Rust structs
// ---------------------------------------------------------------------------

export interface PrivacyOperation {
  operation_type: string;
  provider_id: string | null;
}

export interface PrivacyDecision {
  allowed: boolean;
  reason: string;
}
