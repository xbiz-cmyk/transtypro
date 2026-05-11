/// transtypro — PTT Tauri commands (Phase 10).
use std::sync::atomic::Ordering;

use tauri::Emitter;

use crate::errors::AppError;
use crate::models::PttStatusEvent;
use crate::services::audio::AudioService;
use crate::services::ptt::{PttPhase, PttState};

/// Cancel an active PTT pipeline.
///
/// If recording is in progress, stops and discards the audio before emitting
/// the cancelled event. Ignores "not currently recording" errors from
/// AudioService (race condition between cancel and pipeline completion is safe).
/// Safe to call even when no pipeline is running.
#[tauri::command]
pub fn cancel_ptt(
    ptt_state: tauri::State<'_, PttState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    ptt_state.cancel_flag.store(true, Ordering::SeqCst);

    // If audio is being captured, stop and discard it now.
    // Use audio_state_view() so we don't touch the managed AudioState type.
    if ptt_state.is_phase(&PttPhase::Recording) {
        let audio_view = ptt_state.audio_state_view();
        // Ignore errors — if recording already stopped due to a race, that is fine.
        let _ = AudioService::cancel_recording(&audio_view);
    }

    ptt_state.set_phase(PttPhase::Idle);
    let _ = app_handle.emit(
        "ptt-status",
        PttStatusEvent {
            phase: "cancelled".to_string(),
            message: "Cancelled.".to_string(),
        },
    );
    Ok(())
}
