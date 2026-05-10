/// transtypro — PTT Tauri commands (Phase 10).
use std::sync::atomic::Ordering;

use tauri::Emitter;

use crate::errors::AppError;
use crate::models::PttStatusEvent;
use crate::services::ptt::{PttPhase, PttState};

/// Cancel an active PTT pipeline.
///
/// Sets the cancel flag so the pipeline thread exits at the next checkpoint,
/// resets the phase to Idle, and emits a "cancelled" ptt-status event.
/// Safe to call even when no pipeline is running.
#[tauri::command]
pub fn cancel_ptt(
    ptt_state: tauri::State<'_, PttState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    ptt_state.cancel_flag.store(true, Ordering::SeqCst);
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
