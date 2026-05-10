/**
 * transtypro — Tauri command wrappers.
 *
 * All frontend-to-backend communication goes through these wrappers.
 * Never call `invoke()` directly from components.
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  AiProvider,
  AppSettings,
  CleanupResult,
  DiagnosticReport,
  HistoryEntry,
  MicrophoneInfo,
  PrivacySummary,
  RecordingResult,
  RecordingStatus,
  RetentionResult,
  StatusSummary,
  TranscriptionResult,
} from "./types";

/** Verify frontend-backend IPC communication. */
export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

/** Get the current application version. */
export async function getAppVersion(): Promise<string> {
  return invoke<string>("get_app_version");
}

/** Get the application status summary for the home page. */
export async function getStatusSummary(): Promise<StatusSummary> {
  return invoke<StatusSummary>("get_status_summary");
}

// ---------------------------------------------------------------------------
// Phase 2: Settings (wrappers added in Phase 4 when first needed by Models page)
// ---------------------------------------------------------------------------

/** Return the persisted application settings. */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

/** Persist updated application settings. */
export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke<void>("update_settings", { settings });
}

// ---------------------------------------------------------------------------
// Phase 6: History
// ---------------------------------------------------------------------------

/** Return all history entries, newest first. */
export async function listHistory(): Promise<HistoryEntry[]> {
  return invoke<HistoryEntry[]>("list_history");
}

/** Delete a single history entry by ID. */
export async function deleteHistoryEntry(id: string): Promise<void> {
  return invoke<void>("delete_history_entry", { id });
}

/** Delete all history entries. */
export async function clearHistory(): Promise<void> {
  return invoke<void>("clear_history");
}

/** Save a dictation result to history and return the created entry. */
export async function createHistoryEntry(params: {
  rawText: string;
  cleanedText: string;
  modeUsed: string;
}): Promise<HistoryEntry> {
  return invoke<HistoryEntry>("create_history_entry", params);
}

// ---------------------------------------------------------------------------
// Phase 3: Audio recording
// ---------------------------------------------------------------------------

/** Return all available microphone input devices. */
export async function listMicrophones(): Promise<MicrophoneInfo[]> {
  return invoke<MicrophoneInfo[]>("list_microphones");
}

/** Begin recording from the named device (or the system default if omitted). */
export async function startRecording(deviceName?: string): Promise<RecordingStatus> {
  return invoke<RecordingStatus>("start_recording", {
    deviceName: deviceName ?? null,
  });
}

/** Stop recording, write a temporary WAV file, and return its metadata. */
export async function stopRecording(): Promise<RecordingResult> {
  return invoke<RecordingResult>("stop_recording");
}

/** Abort an active recording without writing a file. */
export async function cancelRecording(): Promise<RecordingStatus> {
  return invoke<RecordingStatus>("cancel_recording");
}

/** Return the current recording state with a live RMS level reading. */
export async function getRecordingStatus(): Promise<RecordingStatus> {
  return invoke<RecordingStatus>("get_recording_status");
}

// ---------------------------------------------------------------------------
// Phase 4: Local transcription
// ---------------------------------------------------------------------------

/** Transcribe the WAV file at the given path using the configured local binary. */
export async function transcribeAudio(filePath: string): Promise<TranscriptionResult> {
  return invoke<TranscriptionResult>("transcribe_audio", { filePath });
}

// ---------------------------------------------------------------------------
// Phase 5: Providers
// ---------------------------------------------------------------------------

/** List all configured AI providers. */
export async function listProviders(): Promise<AiProvider[]> {
  return invoke<AiProvider[]>("list_providers");
}

/** Create a new AI provider. Returns the created provider including its generated ID. */
export async function createProvider(params: {
  name: string;
  providerType: string;
  baseUrl: string;
  model: string;
  useForCleanup: boolean;
}): Promise<AiProvider> {
  return invoke<AiProvider>("create_provider", params);
}

/** Update an existing AI provider's mutable fields. */
export async function updateProvider(params: {
  id: string;
  name: string;
  baseUrl: string;
  model: string;
  enabled: boolean;
  useForCleanup: boolean;
}): Promise<AiProvider> {
  return invoke<AiProvider>("update_provider", params);
}

/** Delete a provider by ID. Also removes its OS keychain entry. */
export async function deleteProvider(id: string): Promise<void> {
  return invoke<void>("delete_provider", { id });
}

/** Test the connection to a provider. Returns a human-readable status string. */
export async function testProviderConnection(id: string): Promise<string> {
  return invoke<string>("test_provider_connection", { id });
}

/**
 * Store an API key for a provider in the OS keychain.
 * The key value is NOT returned to the frontend after this call.
 */
export async function setProviderApiKey(id: string, apiKey: string): Promise<void> {
  return invoke<void>("set_provider_api_key", { id, apiKey });
}

/** List all enabled providers configured for text cleanup. */
export async function listEnabledCleanupProviders(): Promise<AiProvider[]> {
  return invoke<AiProvider[]>("list_enabled_cleanup_providers");
}

// ---------------------------------------------------------------------------
// Phase 8: Privacy status, diagnostics, retention
// ---------------------------------------------------------------------------

/** Return the current privacy status derived from persisted settings. */
export async function getPrivacyStatus(): Promise<PrivacySummary> {
  return invoke<PrivacySummary>("get_privacy_status");
}

/** Run all diagnostic checks and return the full report. */
export async function runDiagnostics(): Promise<DiagnosticReport> {
  return invoke<DiagnosticReport>("run_diagnostics");
}

/** Apply the configured retention policy and return deleted counts. */
export async function applyRetentionPolicy(): Promise<RetentionResult> {
  return invoke<RetentionResult>("apply_retention_policy");
}

// ---------------------------------------------------------------------------
// Phase 5: Cleanup
// ---------------------------------------------------------------------------

/** Send raw transcript text to a cleanup provider and return the cleaned result. */
export async function cleanupText(
  rawText: string,
  providerId: string,
): Promise<CleanupResult> {
  return invoke<CleanupResult>("cleanup_text", { rawText, providerId });
}
