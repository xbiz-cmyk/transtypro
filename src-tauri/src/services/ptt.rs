/// transtypro — Push-to-talk pipeline service (Phase 10).
///
/// PttState holds the shared PTT lifecycle state managed by Tauri.
/// PttPipelineService orchestrates: stop recording → transcribe → optional cleanup
///   → insert → save history.
///
/// Privacy rules enforced here:
/// - No transcript text is ever logged at any level.
/// - No clipboard contents are logged.
/// - PttStatusEvent.message contains only generic status strings.
/// - No network calls beyond existing CleanupService behavior.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::{HistoryEntry, InsertionResult, PttStatusEvent};
use crate::services::audio::{AudioService, AudioState, RecordingHandle};
use crate::services::{
    CleanupService, HistoryService, InsertionService, ProvidersService, TranscriptionService,
};

// ─────────────────────────────── PTT phase ───────────────────────────────────

/// Current phase of the PTT pipeline lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum PttPhase {
    Idle,
    Recording,
    Transcribing,
    Cleaning,
    Inserting,
    Done,
    Error(String),
    Cancelled,
}

// ─────────────────────────────── PTT state ───────────────────────────────────

/// Tauri-managed PTT state. Holds the pipeline phase, cancel flag, and
/// clones of the audio Arc fields so the pipeline thread can reconstruct
/// a temporary AudioState view without modifying the managed AudioState type.
pub struct PttState {
    pub phase: Arc<Mutex<PttPhase>>,
    pub cancel_flag: Arc<AtomicBool>,
    /// Same audio directory used by AudioService — needed by TranscriptionService
    /// for path containment validation.
    pub audio_dir: PathBuf,
    /// Arcs shared with the managed AudioState so both refer to the same data.
    pub recording: Arc<Mutex<Option<RecordingHandle>>>,
    pub samples: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Arc<Mutex<u32>>,
    pub channels: Arc<Mutex<u16>>,
}

impl PttState {
    /// Construct a temporary AudioState that shares the same Arc data as the
    /// managed AudioState. Use only for PTT pipeline AudioService calls.
    pub fn audio_state_view(&self) -> AudioState {
        AudioState {
            audio_dir: self.audio_dir.clone(),
            recording: self.recording.clone(),
            samples: self.samples.clone(),
            sample_rate: self.sample_rate.clone(),
            channels: self.channels.clone(),
        }
    }

    /// Return true if the current phase equals `expected`.
    pub fn is_phase(&self, expected: &PttPhase) -> bool {
        self.phase.lock().map(|g| *g == *expected).unwrap_or(false)
    }

    /// Atomically set phase to `next` and return the previous phase.
    pub fn set_phase(&self, next: PttPhase) -> PttPhase {
        let mut guard = self.phase.lock().unwrap();
        let old = guard.clone();
        *guard = next;
        old
    }
}

// PttState is Tauri-managed state; it must be Send + Sync.
// All fields are Arc<Mutex<…>>/AtomicBool/PathBuf which are inherently Send+Sync.
unsafe impl Send for PttState {}
unsafe impl Sync for PttState {}

// ─────────────────────────── Pipeline service ────────────────────────────────

pub struct PttPipelineService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl PttPipelineService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    /// Run the full PTT pipeline on the calling thread.
    ///
    /// MUST be called from a dedicated spawned thread — never from the
    /// shortcut callback or the Tauri async executor.
    ///
    /// Pipeline:
    ///   1. stop_recording  → WAV path
    ///   2. transcribe      → raw_text
    ///   3. optional cleanup → final_text (falls back to raw_text on any error)
    ///   4. insert          → InsertionResult
    ///   5. save history    → HistoryEntry
    ///   6. mark_inserted if insertion succeeded
    pub fn run_pipeline(&self, ptt: &PttState, handle: &tauri::AppHandle) {
        // ── Step 1: Stop recording ──────────────────────────────────────────
        let audio_view = ptt.audio_state_view();
        let recording_result = match AudioService::stop_recording(&audio_view) {
            Ok(r) => r,
            Err(e) => {
                self.abort(ptt, handle, format!("Could not stop recording: {e}"));
                return;
            }
        };

        if self.is_cancelled(ptt, handle) {
            return;
        }

        let wav_path = recording_result.file_path.clone();

        // ── Step 2: Transcribe ──────────────────────────────────────────────
        ptt.set_phase(PttPhase::Transcribing);
        let _ = handle.emit(
            "ptt-status",
            PttStatusEvent {
                phase: "transcribing".to_string(),
                message: "Transcribing…".to_string(),
            },
        );

        let (binary_path, model_path) = match self.read_whisper_paths() {
            Ok(t) => t,
            Err(e) => {
                self.abort(ptt, handle, format!("{e}"));
                return;
            }
        };

        let transcription_result = match TranscriptionService::transcribe(
            &wav_path,
            &binary_path,
            &model_path,
            &ptt.audio_dir,
        ) {
            Ok(r) => r,
            Err(e) => {
                self.abort(ptt, handle, format!("Transcription failed: {e}"));
                return;
            }
        };

        let raw_text = transcription_result.raw_text.clone();

        if self.is_cancelled(ptt, handle) {
            return;
        }

        // ── Step 3: Optional cleanup (non-fatal) ────────────────────────────
        let final_text = self
            .try_cleanup(&raw_text, ptt, handle)
            .unwrap_or_else(|| raw_text.clone());

        if self.is_cancelled(ptt, handle) {
            return;
        }

        // ── Step 4: Insert ──────────────────────────────────────────────────
        ptt.set_phase(PttPhase::Inserting);
        let _ = handle.emit(
            "ptt-status",
            PttStatusEvent {
                phase: "inserting".to_string(),
                message: "Inserting…".to_string(),
            },
        );

        // Call InsertionService directly — NOT via the insert_text Tauri command.
        // The Tauri command minimizes/restores the window which is wrong for PTT
        // (the target app already has focus; we must not disturb it).
        let insertion_result: InsertionResult = InsertionService::new(self.db.clone())
            .insert_text(final_text.clone())
            .unwrap_or_else(|e| InsertionResult {
                success: false,
                method: "clipboard_only".to_string(),
                message: format!("Insertion error: {e}"),
            });

        // ── Step 5: Save history (non-fatal) ────────────────────────────────
        let active_mode = self
            .read_active_mode()
            .unwrap_or_else(|_| "smart".to_string());

        let cleaned_text = if final_text != raw_text {
            final_text.clone()
        } else {
            String::new()
        };

        let entry = HistoryEntry {
            id: String::new(),
            raw_text: raw_text.clone(),
            cleaned_text,
            mode_used: active_mode,
            timestamp: String::new(),
            was_inserted: false,
        };

        let saved_entry = HistoryService::new(self.db.clone())
            .create_history_entry(entry)
            .ok();

        // ── Step 6: Mark inserted if paste succeeded ────────────────────────
        if insertion_result.success {
            if let Some(ref e) = saved_entry {
                let _ = HistoryService::new(self.db.clone()).mark_inserted(e.id.clone());
            }
        }

        // ── Done ────────────────────────────────────────────────────────────
        ptt.set_phase(PttPhase::Idle);
        let _ = handle.emit(
            "ptt-status",
            PttStatusEvent {
                phase: "done".to_string(),
                message: "Done.".to_string(),
            },
        );
        // Do NOT bring window to front on success — leave the user in their app.
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Emit error, bring window to front, reset to Idle.
    fn abort(&self, ptt: &PttState, handle: &tauri::AppHandle, message: String) {
        ptt.set_phase(PttPhase::Idle);
        let _ = handle.emit(
            "ptt-status",
            PttStatusEvent {
                phase: "error".to_string(),
                message,
            },
        );
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }

    /// Check cancel flag. If set, emit cancelled event and return true.
    fn is_cancelled(&self, ptt: &PttState, handle: &tauri::AppHandle) -> bool {
        if ptt.cancel_flag.load(Ordering::SeqCst) {
            ptt.set_phase(PttPhase::Idle);
            let _ = handle.emit(
                "ptt-status",
                PttStatusEvent {
                    phase: "cancelled".to_string(),
                    message: "Cancelled.".to_string(),
                },
            );
            true
        } else {
            false
        }
    }

    /// Attempt AI cleanup. Returns Some(cleaned_text) or None (non-fatal skip).
    fn try_cleanup(
        &self,
        raw_text: &str,
        ptt: &PttState,
        handle: &tauri::AppHandle,
    ) -> Option<String> {
        let provider_id = match self.find_cleanup_provider_id() {
            Ok(Some(id)) => id,
            _ => return None,
        };

        ptt.set_phase(PttPhase::Cleaning);
        let _ = handle.emit(
            "ptt-status",
            PttStatusEvent {
                phase: "cleaning".to_string(),
                message: "Cleaning text…".to_string(),
            },
        );

        match CleanupService::new(self.db.clone()).cleanup(raw_text, &provider_id) {
            Ok(r) => Some(r.cleaned_text),
            Err(_) => None, // non-fatal
        }
    }

    fn find_cleanup_provider_id(&self) -> Result<Option<String>, AppError> {
        let providers = ProvidersService::new(self.db.clone()).list_enabled_cleanup_providers()?;
        Ok(providers.into_iter().next().map(|p| p.id))
    }

    fn read_whisper_paths(&self) -> Result<(String, String), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("db lock poisoned".into()))?;
        let settings = SettingsRepository::new(&conn).get()?;
        let binary = settings.whisper_binary_path.ok_or_else(|| {
            AppError::TranscriptionError(
                "Whisper binary path is not configured. Set it in the Models page.".into(),
            )
        })?;
        let model = settings.whisper_model_path.ok_or_else(|| {
            AppError::TranscriptionError(
                "Whisper model path is not configured. Set it in the Models page.".into(),
            )
        })?;
        Ok((binary, model))
    }

    fn read_active_mode(&self) -> Result<String, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("db lock poisoned".into()))?;
        Ok(SettingsRepository::new(&conn).get()?.active_mode)
    }
}

// ─────────────────────────────── Tests ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_ptt_state() -> PttState {
        PttState {
            phase: Arc::new(Mutex::new(PttPhase::Idle)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            audio_dir: PathBuf::from("/tmp/test_audio"),
            recording: Arc::new(Mutex::new(None)),
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(44100)),
            channels: Arc::new(Mutex::new(1)),
        }
    }

    #[test]
    fn test_ptt_state_starts_idle() {
        let ptt = make_ptt_state();
        assert!(ptt.is_phase(&PttPhase::Idle));
    }

    #[test]
    fn test_cancel_flag_is_clear_on_new_ptt_state() {
        let ptt = make_ptt_state();
        assert!(!ptt.cancel_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_ptt_set_phase_transitions() {
        let ptt = make_ptt_state();
        let old = ptt.set_phase(PttPhase::Recording);
        assert_eq!(old, PttPhase::Idle);
        assert!(ptt.is_phase(&PttPhase::Recording));
    }

    #[test]
    fn test_ptt_is_phase_correct() {
        let ptt = make_ptt_state();
        assert!(ptt.is_phase(&PttPhase::Idle));
        assert!(!ptt.is_phase(&PttPhase::Recording));
        ptt.set_phase(PttPhase::Recording);
        assert!(ptt.is_phase(&PttPhase::Recording));
        assert!(!ptt.is_phase(&PttPhase::Idle));
    }

    #[test]
    fn test_ptt_phase_idle_allows_start() {
        let ptt = make_ptt_state();
        // Simulate the CAS guard used in the shortcut handler.
        let should_start = {
            let mut guard = ptt.phase.lock().unwrap();
            if matches!(*guard, PttPhase::Idle) {
                *guard = PttPhase::Recording;
                true
            } else {
                false
            }
        };
        assert!(should_start, "Idle phase should allow PTT start");
        assert!(ptt.is_phase(&PttPhase::Recording));
    }

    #[test]
    fn test_ptt_phase_recording_blocks_double_start() {
        let ptt = make_ptt_state();
        ptt.set_phase(PttPhase::Recording);
        // Simulate second press while already recording.
        let would_start = {
            let guard = ptt.phase.lock().unwrap();
            matches!(*guard, PttPhase::Idle)
        };
        assert!(
            !would_start,
            "Recording phase must block a second PTT start"
        );
    }

    #[test]
    fn test_ptt_status_event_serde_recording() {
        let event = PttStatusEvent {
            phase: "recording".to_string(),
            message: "Recording…".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"phase\":\"recording\""));
        assert!(json.contains("\"message\""));
    }

    #[test]
    fn test_ptt_status_event_serde_error() {
        let event = PttStatusEvent {
            phase: "error".to_string(),
            message: "Transcription failed: binary not found".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"phase\":\"error\""));
    }
}
