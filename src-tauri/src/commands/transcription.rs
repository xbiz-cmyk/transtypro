use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::{PrivacyOperation, TranscriptionResult};
use crate::services::{AudioState, PrivacyService, SettingsService, TranscriptionService};

/// Transcribe a WAV file using the locally configured whisper.cpp binary.
///
/// Reads binary and model paths from persisted settings.
/// Enforces the privacy gate before spawning the process.
/// Deletes the WAV after successful transcription if `audio_history_enabled` is false.
/// On failure, the WAV is kept so the user can retry.
///
/// **Frontend must pass the argument as camelCase:**
/// ```typescript
/// invoke("transcribe_audio", { filePath: "..." })
/// ```
/// Tauri v2 maps camelCase JS keys to snake_case Rust parameter names automatically.
#[tauri::command]
pub fn transcribe_audio(
    file_path: String,
    state: State<'_, AppState>,
    audio_state: State<'_, AudioState>,
) -> Result<TranscriptionResult, AppError> {
    let settings = SettingsService::new(state.db.clone()).get_settings()?;

    // Privacy gate — local_transcription is always allowed, but we enforce
    // the check for correctness and future-proofing.
    let decision =
        PrivacyService::new(state.db.clone()).enforce_privacy_preview(PrivacyOperation {
            operation_type: "local_transcription".to_string(),
            provider_id: None,
        })?;
    if !decision.allowed {
        return Err(AppError::TranscriptionError(format!(
            "Privacy check blocked transcription: {}",
            decision.reason
        )));
    }

    let binary_path = settings.whisper_binary_path.as_deref().unwrap_or("");
    let model_path = settings.whisper_model_path.as_deref().unwrap_or("");

    let result = TranscriptionService::transcribe(
        &file_path,
        binary_path,
        model_path,
        &audio_state.audio_dir,
    )?;

    // Clean up WAV only after successful transcription.
    // If transcription failed the error was returned above; the WAV is kept for retry.
    TranscriptionService::cleanup_wav_if_needed(&file_path, settings.audio_history_enabled);

    Ok(result)
}
