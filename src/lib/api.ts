/**
 * transtypro — Tauri command wrappers.
 *
 * All frontend-to-backend communication goes through these wrappers.
 * Never call `invoke()` directly from components.
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  MicrophoneInfo,
  RecordingResult,
  RecordingStatus,
  StatusSummary,
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
