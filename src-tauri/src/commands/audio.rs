/// transtypro — Audio recording Tauri commands (Phase 3).
///
/// Thin wrappers that delegate to AudioService. No business logic here.
use crate::errors::AppError;
use crate::models::{MicrophoneInfo, RecordingResult, RecordingStatus};
use crate::services::{AudioService, AudioState};

/// Return all available microphone input devices.
#[tauri::command]
pub fn list_microphones(
    _audio_state: tauri::State<'_, AudioState>,
) -> Result<Vec<MicrophoneInfo>, AppError> {
    AudioService::list_microphones()
}

/// Begin recording from the named device (or the system default if None).
#[tauri::command]
pub fn start_recording(
    device_name: Option<String>,
    audio_state: tauri::State<'_, AudioState>,
) -> Result<RecordingStatus, AppError> {
    AudioService::start_recording(device_name, &audio_state)
}

/// Stop recording, write a temporary WAV file, and return its path.
#[tauri::command]
pub fn stop_recording(
    audio_state: tauri::State<'_, AudioState>,
) -> Result<RecordingResult, AppError> {
    AudioService::stop_recording(&audio_state)
}

/// Abort an active recording without writing a file.
#[tauri::command]
pub fn cancel_recording(
    audio_state: tauri::State<'_, AudioState>,
) -> Result<RecordingStatus, AppError> {
    AudioService::cancel_recording(&audio_state)
}

/// Return the current recording state with a live RMS level reading.
#[tauri::command]
pub fn get_recording_status(
    audio_state: tauri::State<'_, AudioState>,
) -> Result<RecordingStatus, AppError> {
    AudioService::get_recording_status(&audio_state)
}
