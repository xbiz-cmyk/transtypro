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
// AppSettings — mirrors Rust AppSettings struct
// ---------------------------------------------------------------------------

export interface AppSettings {
  /** UI theme: "dark" | "light" | "system" */
  theme: string;
  /** UI language / locale code */
  language: string;
  /** Default dictation mode name */
  default_mode: string;
  /** Global shortcut string (e.g. "CommandOrControl+Shift+Space") */
  shortcut: string;
  /** Privacy mode: "local-only" | "cloud-enabled" */
  privacy_mode: string;
  /** Number of days to retain history (0 = forever) */
  retention_days: number;
  /** Whether to persist audio recordings */
  audio_history_enabled: boolean;
  /** Whether to restore clipboard after dictation */
  clipboard_restore_enabled: boolean;
  /** Filesystem path to the SQLite database */
  db_path: string;
}

// ---------------------------------------------------------------------------
// HistoryEntry — mirrors Rust HistoryEntry struct
// ---------------------------------------------------------------------------

export interface HistoryEntry {
  id: number;
  /** Unix timestamp (seconds) */
  created_at: number;
  /** Dictation mode used */
  mode: string;
  /** Raw transcript text */
  raw_text: string;
  /** Cleaned/formatted text (may equal raw_text if no cleanup was applied) */
  cleaned_text: string;
  /** Duration in seconds */
  duration_secs: number;
  /** Whether cleanup was applied */
  cleanup_applied: boolean;
  /** Name of provider used, or null */
  provider_name: string | null;
}

// ---------------------------------------------------------------------------
// DictationMode — mirrors Rust DictationMode struct
// ---------------------------------------------------------------------------

export interface DictationMode {
  id: number;
  name: string;
  description: string;
  /** Whether this is the currently active mode */
  is_active: boolean;
  /** Whether this is a built-in system mode (non-deletable) */
  is_builtin: boolean;
  /** System prompt template for cleanup */
  system_prompt: string;
}

// ---------------------------------------------------------------------------
// VocabularyEntry — mirrors Rust VocabularyEntry struct
// ---------------------------------------------------------------------------

export interface VocabularyEntry {
  id: number;
  /** The spoken term */
  term: string;
  /** The replacement text to insert */
  replacement: string;
  /** Category label (e.g. "Technical", "Personal") */
  category: string;
}

// ---------------------------------------------------------------------------
// AiProvider — mirrors Rust AiProvider struct
// ---------------------------------------------------------------------------

export type ProviderType = "ollama" | "openai-compatible" | "anthropic";

export interface AiProvider {
  id: number;
  name: string;
  provider_type: ProviderType;
  base_url: string;
  model_name: string;
  /** Whether this is the active provider */
  is_active: boolean;
  /** API key is stored encrypted server-side; frontend only receives a masked hint */
  api_key_hint: string | null;
}

// ---------------------------------------------------------------------------
// ModelEntry — local whisper-compatible model
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
// DiagnosticReport — mirrors Rust DiagnosticReport struct
// ---------------------------------------------------------------------------

export interface DiagnosticItem {
  label: string;
  status: "ok" | "warn" | "error" | "unknown";
  detail: string | null;
}

export interface DiagnosticReport {
  generated_at: number;
  items: DiagnosticItem[];
}

// ---------------------------------------------------------------------------
// PrivacyStatus — frontend representation of current privacy state
// ---------------------------------------------------------------------------

export interface PrivacyStatus {
  mode: "local-only" | "cloud-enabled";
  audio_deleted_after_use: boolean;
  cloud_calls_blocked: boolean;
  retention_days: number;
}
