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

// ─────────────────────── Windows focus helpers ───────────────────────────────

// Raw Win32 FFI — only compiled on Windows.
// HWND is stored as isize (pointer-sized integer) so it can cross thread
// boundaries inside Arc<Mutex<…>> without requiring unsafe Send impls.
// Only GetForegroundWindow / IsWindow / SetForegroundWindow are used; window
// title, process name, and contents are never read.
#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn GetForegroundWindow() -> isize;
    fn IsWindow(hwnd: isize) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
}

/// Capture the current foreground window handle as an opaque integer.
/// Returns 0 on non-Windows or if no foreground window exists.
/// The handle is used only to restore focus before insertion; no title or
/// contents are read.
pub fn capture_foreground_window() -> isize {
    #[cfg(target_os = "windows")]
    {
        unsafe { GetForegroundWindow() }
    }
    #[cfg(not(target_os = "windows"))]
    {
        0
    }
}

/// Restore focus to the window identified by `hwnd`.
/// No-op if `hwnd` is 0, the window no longer exists, or on non-Windows.
fn restore_foreground_window(hwnd: isize) {
    #[cfg(target_os = "windows")]
    {
        if hwnd == 0 {
            return;
        }
        unsafe {
            if IsWindow(hwnd) != 0 {
                SetForegroundWindow(hwnd);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
    }
}

// ─────────────────────── Status event helper ─────────────────────────────────

/// Emit a PTT status event explicitly to both the main window and the ptt-overlay window.
///
/// Uses emit_to by window label instead of the broadcast emit() so that each
/// window receives the event via a direct IPC path. This avoids the WebView2
/// background-throttling issue on Windows where hidden webviews may not receive
/// broadcast events reliably.
///
/// Message must contain only generic status strings — never user-dictated content.
pub fn emit_ptt_status(app: &tauri::AppHandle, phase: &str, message: &str) {
    let event = PttStatusEvent {
        phase: phase.to_string(),
        message: message.to_string(),
    };
    let _ = app.emit_to("main", "ptt-status", event.clone());
    let _ = app.emit_to("ptt-overlay", "ptt-status", event);
}

/// Hide the ptt-overlay window immediately from the backend.
/// No-op if the window does not exist.
pub fn hide_ptt_overlay(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("ptt-overlay") {
        let _ = overlay.hide();
    }
}

/// Hide the ptt-overlay window after `delay_ms` milliseconds.
/// Spawns a detached thread; does not block the caller.
pub fn hide_ptt_overlay_after(app: &tauri::AppHandle, delay_ms: u64) {
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        hide_ptt_overlay(&handle);
    });
}

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
    /// Opaque foreground window handle captured immediately before the PTT
    /// overlay is shown. Used only to restore focus before clipboard paste so
    /// the text lands in the correct app even if the overlay was dragged.
    /// Stored as isize (pointer-sized integer) to be Send + Sync.
    /// Window title, process name, and contents are never read.
    pub target_hwnd: Arc<Mutex<Option<isize>>>,
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
        emit_ptt_status(handle, "transcribing", "Transcribing…");

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
        // Strip whisper.cpp timestamp markers ([HH:MM:SS.mmm --> HH:MM:SS.mmm]) before
        // insertion. raw_text preserved in history for traceability; all user-visible text
        // uses the stripped version.
        let stripped_text = strip_whisper_timestamps(&raw_text);

        if self.is_cancelled(ptt, handle) {
            return;
        }

        // ── Step 3: Optional cleanup (non-fatal) ────────────────────────────
        // In insert_raw mode, skip cleanup entirely for lower latency.
        let final_text = if self.read_ptt_output_mode() == "insert_raw" {
            stripped_text.clone()
        } else {
            self.try_cleanup(&stripped_text, ptt, handle)
                .unwrap_or_else(|| stripped_text.clone())
        };

        if self.is_cancelled(ptt, handle) {
            return;
        }

        // ── Step 4: Insert ──────────────────────────────────────────────────
        ptt.set_phase(PttPhase::Inserting);
        emit_ptt_status(handle, "inserting", "Inserting…");

        // Restore focus to the original target window before pasting.
        // Dragging the overlay can shift the OS foreground away from the user's
        // app. We captured the HWND in ptt_start() before showing the overlay.
        // Only an opaque handle is used — no title or contents are read.
        let target_hwnd = ptt.target_hwnd.lock().ok().and_then(|g| *g).unwrap_or(0);
        restore_foreground_window(target_hwnd);
        if target_hwnd != 0 {
            // Brief pause so the OS can complete the focus transfer before
            // enigo fires SendInput (Ctrl+V) at the now-focused window.
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

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

        // Always store the final text (cleaned if cleanup ran, otherwise raw).
        // History UI displays cleaned_text as the primary field; an empty string
        // would show a blank entry for PTT dictations.
        let cleaned_text = final_text.clone();

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
        emit_ptt_status(handle, "done", "Done.");
        // Do NOT bring window to front on success — leave the user in their app.
        // Hide the overlay after the "Done." message has been visible briefly.
        hide_ptt_overlay_after(handle, 1500);
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Emit error, bring window to front, reset to Idle.
    fn abort(&self, ptt: &PttState, handle: &tauri::AppHandle, message: String) {
        ptt.set_phase(PttPhase::Idle);
        emit_ptt_status(handle, "error", &message);
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }

    /// Check cancel flag. If set, emit cancelled event, hide overlay, and return true.
    fn is_cancelled(&self, ptt: &PttState, handle: &tauri::AppHandle) -> bool {
        if ptt.cancel_flag.load(Ordering::SeqCst) {
            ptt.set_phase(PttPhase::Idle);
            emit_ptt_status(handle, "cancelled", "Cancelled.");
            hide_ptt_overlay(handle);
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
        emit_ptt_status(handle, "cleaning", "Cleaning text…");

        match CleanupService::new(self.db.clone()).cleanup(raw_text, &provider_id) {
            Ok(r) => Some(r.cleaned_text),
            Err(_) => None, // non-fatal
        }
    }

    fn read_ptt_output_mode(&self) -> String {
        self.db
            .lock()
            .ok()
            .and_then(|conn| SettingsRepository::new(&conn).get().ok())
            .map(|s| s.ptt_output_mode)
            .unwrap_or_else(|| "clean_before_insert".to_string())
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

// ────────────────────────── Timestamp stripping ──────────────────────────────

/// Strip whisper.cpp timestamp markers from raw transcription output.
///
/// Whisper.cpp emits lines in the form:
///   `[00:00:00.000 --> 00:00:07.940]  Hello, this is the text.`
///
/// This function removes the `[timestamp --> timestamp]` prefix from each such
/// line and joins the remaining text fragments with a space. Lines that do not
/// contain `-->` (including `[BLANK_AUDIO]` noise markers) are either kept as-is
/// (plain text) or skipped (bracket-only markers). Empty lines are always skipped.
///
/// If the input has no timestamp markers, it is returned unchanged (trimmed).
pub(crate) fn strip_whisper_timestamps(text: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.contains("-->") {
            // Timestamp line: extract the text that follows the closing `]`.
            if let Some(bracket_end) = trimmed.find(']') {
                let rest = trimmed[bracket_end + 1..].trim();
                if !rest.is_empty() {
                    parts.push(rest);
                }
            }
        } else if trimmed.starts_with('[') {
            // Non-timestamp bracket marker (e.g. [BLANK_AUDIO], [MUSIC]) — skip.
        } else if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
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
            target_hwnd: Arc::new(Mutex::new(None)),
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
    fn test_cancel_during_recording_discards_audio_and_goes_idle() {
        // Verify: when cancel is requested while in Recording phase,
        // calling cancel_recording on an empty audio view (no active stream)
        // returns an error which is safely ignored, and phase ends up Idle.
        let ptt = make_ptt_state();
        ptt.set_phase(PttPhase::Recording);
        ptt.cancel_flag.store(true, Ordering::SeqCst);

        // Simulate the cancel_ptt command logic.
        if ptt.is_phase(&PttPhase::Recording) {
            let audio_view = ptt.audio_state_view();
            // No active recording — expect an error; we must ignore it.
            let result = crate::services::audio::AudioService::cancel_recording(&audio_view);
            assert!(
                result.is_err(),
                "cancel_recording with no active stream should return an error"
            );
        }

        ptt.set_phase(PttPhase::Idle);
        assert!(
            ptt.is_phase(&PttPhase::Idle),
            "phase must be Idle after cancel"
        );
        assert!(
            ptt.cancel_flag.load(Ordering::SeqCst),
            "cancel flag remains set until pipeline thread sees it"
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

    #[test]
    fn test_strip_whisper_timestamps_single_line() {
        let input = "[00:00:00.000 --> 00:00:07.940]  Hello, this is a test.";
        assert_eq!(strip_whisper_timestamps(input), "Hello, this is a test.");
    }

    #[test]
    fn test_strip_whisper_timestamps_multiple_lines() {
        let input = "[00:00:00.000 --> 00:00:05.000]  First sentence.\n\
                     [00:00:05.000 --> 00:00:10.000]  Second sentence.";
        assert_eq!(
            strip_whisper_timestamps(input),
            "First sentence. Second sentence."
        );
    }

    #[test]
    fn test_strip_whisper_timestamps_plain_text_unchanged() {
        let input = "Hello, world!";
        assert_eq!(strip_whisper_timestamps(input), "Hello, world!");
    }

    #[test]
    fn test_strip_whisper_timestamps_blank_audio_skipped() {
        let input = "[BLANK_AUDIO]";
        assert_eq!(strip_whisper_timestamps(input), "");
    }

    #[test]
    fn test_strip_whisper_timestamps_mixed_blank_and_speech() {
        let input = "[00:00:00.000 --> 00:00:02.000]  [BLANK_AUDIO]\n\
                     [00:00:02.000 --> 00:00:06.000]  Actual words here.";
        assert_eq!(
            strip_whisper_timestamps(input),
            "[BLANK_AUDIO] Actual words here."
        );
    }

    #[test]
    fn test_strip_whisper_timestamps_empty_input() {
        assert_eq!(strip_whisper_timestamps(""), "");
    }
}
