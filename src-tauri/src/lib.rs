/// transtypro — Main library entry point.
///
/// Module declarations and Tauri app builder.
/// Keep this file thin: business logic belongs in services,
/// database logic in db, and command wiring in commands.
pub mod commands;
pub mod db;
pub mod errors;
pub mod models;
pub mod services;
pub mod utils;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::services::ptt::{
    capture_foreground_window, emit_ptt_status, PttPhase, PttPipelineService, PttState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            // Phase 2: SQLite database
            let db_path = data_dir.join("transtypro.sqlite");
            let conn = rusqlite::Connection::open(&db_path)?;
            db::run_migrations(&conn)?;

            // Phase 9: Read configured shortcut from DB before wrapping in Arc<Mutex>.
            let shortcut_str = db::repositories::SettingsRepository::new(&conn)
                .get()
                .map(|s| s.shortcut)
                .unwrap_or_else(|_| "CommandOrControl+Shift+Space".to_string());

            app.manage(db::AppState {
                db: Arc::new(Mutex::new(conn)),
            });

            // Phase 3: Audio recording state (separate from DB state).
            // Phase 10: Create the Arc fields first so PttState can share them.
            let audio_dir = data_dir.join("audio");
            std::fs::create_dir_all(&audio_dir)?;

            let recording_arc: Arc<Mutex<Option<services::audio::RecordingHandle>>> =
                Arc::new(Mutex::new(None));
            let samples_arc: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let sample_rate_arc: Arc<Mutex<u32>> = Arc::new(Mutex::new(44100));
            let channels_arc: Arc<Mutex<u16>> = Arc::new(Mutex::new(1));

            app.manage(services::AudioState {
                audio_dir: audio_dir.clone(),
                recording: recording_arc.clone(),
                samples: samples_arc.clone(),
                sample_rate: sample_rate_arc.clone(),
                channels: channels_arc.clone(),
            });

            // Phase 10: PTT state — shares audio Arcs so the pipeline thread can
            // reconstruct an AudioState view without touching managed AudioState type.
            app.manage(PttState {
                phase: Arc::new(Mutex::new(PttPhase::Idle)),
                cancel_flag: Arc::new(AtomicBool::new(false)),
                audio_dir: audio_dir.clone(),
                recording: recording_arc,
                samples: samples_arc,
                sample_rate: sample_rate_arc,
                channels: channels_arc,
                target_hwnd: Arc::new(Mutex::new(None)),
            });

            // Phase 7 / Phase 9 / Phase 10: Register the configured global shortcut.
            // Phase 9: shortcut string is read from DB.
            // Phase 10: shortcut handler branches on shortcut_behavior setting.
            match shortcut_str.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                Ok(shortcut) => {
                    if let Err(e) = app.handle().global_shortcut().on_shortcut(
                        shortcut,
                        move |app_handle, _shortcut, event| {
                            let behavior = {
                                let raw = read_shortcut_behavior(app_handle);
                                // On Windows, RegisterHotKey never fires Released, so
                                // push_to_talk_hold would start recording with no way to stop.
                                // Map it to toggle mode at runtime so the shortcut stays safe.
                                if cfg!(target_os = "windows") && raw == "push_to_talk_hold" {
                                    "push_to_talk_toggle".to_string()
                                } else {
                                    raw
                                }
                            };

                            match event.state() {
                                tauri_plugin_global_shortcut::ShortcutState::Pressed => {
                                    match behavior.as_str() {
                                        "open_dictation" => {
                                            // Phase 7/9 behavior — unchanged.
                                            if let Some(window) =
                                                app_handle.get_webview_window("main")
                                            {
                                                let _ = window.unminimize();
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                            }
                                            let _ = app_handle
                                                .emit("dictation-shortcut-pressed", ());
                                        }
                                        "push_to_talk_hold" => {
                                            // Hold mode: Pressed starts recording.
                                            // Released is required to stop — see handoff for
                                            // Windows caveat. If Released never fires,
                                            // the user can also use toggle mode instead.
                                            ptt_start(app_handle);
                                        }
                                        "push_to_talk_toggle" => {
                                            // Toggle mode: Pressed toggles start/stop.
                                            ptt_toggle(app_handle);
                                        }
                                        _ => {
                                            eprintln!(
                                                "[shortcut] unknown shortcut_behavior '{behavior}', \
                                                 falling back to open_dictation"
                                            );
                                            if let Some(window) =
                                                app_handle.get_webview_window("main")
                                            {
                                                let _ = window.unminimize();
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                            }
                                            let _ = app_handle
                                                .emit("dictation-shortcut-pressed", ());
                                        }
                                    }
                                }
                                tauri_plugin_global_shortcut::ShortcutState::Released => {
                                    // On Windows with RegisterHotKey, Released events do NOT
                                    // fire. This branch is present for platforms where they do.
                                    if behavior == "push_to_talk_hold" {
                                        ptt_stop_and_run(app_handle);
                                    }
                                }
                            }
                        },
                    ) {
                        eprintln!("[shortcut] global shortcut registration failed: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("[shortcut] failed to parse shortcut string '{shortcut_str}': {e}");
                }
            }

            // Phase 10.1: Create the PTT feedback overlay window (hidden at startup).
            // Pre-loading the webview ensures PttOverlay's ptt-status listener is
            // already registered before the first PTT recording starts.
            // Window creation failure is non-fatal — PTT still works, just without overlay.
            match tauri::WebviewWindowBuilder::new(
                app.handle(),
                "ptt-overlay",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("")
            .inner_size(320.0, 96.0)
            .always_on_top(true)
            .skip_taskbar(true)
            .decorations(false)
            .visible(false)
            .focused(false)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .closable(false)
            .build()
            {
                Ok(overlay) => {
                    // Position near bottom-center of the primary monitor.
                    if let Ok(Some(monitor)) = overlay.primary_monitor() {
                        let scale = monitor.scale_factor();
                        let screen_w = monitor.size().width as f64 / scale;
                        let screen_h = monitor.size().height as f64 / scale;
                        let x = (screen_w - 320.0) / 2.0;
                        let y = screen_h - 96.0 - 48.0;
                        let _ = overlay.set_position(tauri::LogicalPosition::new(x, y));
                    }
                }
                Err(e) => {
                    eprintln!("[ptt-overlay] failed to create overlay window: {e}");
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Phase 0 — core
            commands::ping,
            commands::get_app_version,
            commands::get_status_summary,
            // Phase 2 — settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            // Phase 2 — modes
            commands::modes::list_modes,
            commands::modes::get_mode,
            commands::modes::create_mode,
            commands::modes::update_mode,
            commands::modes::delete_mode,
            // Phase 2 — vocabulary
            commands::vocabulary::list_vocabulary,
            commands::vocabulary::add_vocabulary_entry,
            commands::vocabulary::update_vocabulary_entry,
            commands::vocabulary::delete_vocabulary_entry,
            // Phase 2 — history
            commands::history::list_history,
            commands::history::get_history_entry,
            commands::history::delete_history_entry,
            commands::history::clear_history,
            // Phase 6 — history creation
            commands::history::create_history_entry,
            // Phase 2 — privacy
            commands::privacy::get_privacy_status,
            commands::privacy::enforce_privacy_preview,
            // Phase 5 — providers (real SQLite-backed)
            commands::providers::list_providers,
            commands::providers::get_provider,
            commands::providers::create_provider,
            commands::providers::update_provider,
            commands::providers::delete_provider,
            commands::providers::test_provider_connection,
            commands::providers::set_provider_api_key,
            commands::providers::list_enabled_cleanup_providers,
            commands::providers::test_provider_placeholder,
            // Phase 5 — cleanup
            commands::cleanup::cleanup_text,
            // Phase 8 — diagnostics (real checks) and retention cleanup
            commands::diagnostics::run_diagnostics,
            commands::diagnostics::apply_retention_policy,
            // Phase 3 — audio recording
            commands::audio::list_microphones,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::audio::cancel_recording,
            commands::audio::get_recording_status,
            // Phase 4 — local transcription
            commands::transcription::transcribe_audio,
            // Phase 9 — text insertion and shortcut rebinding
            commands::insertion::insert_text,
            commands::insertion::mark_history_inserted,
            commands::shortcut::update_shortcut,
            // Phase 10 — PTT pipeline control
            commands::ptt::cancel_ptt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ─────────────────────── PTT shortcut handler helpers ────────────────────────

/// Read shortcut_behavior from the DB. Returns "open_dictation" on any error.
pub(crate) fn read_shortcut_behavior(app: &tauri::AppHandle) -> String {
    app.state::<db::AppState>()
        .db
        .lock()
        .ok()
        .and_then(|conn| db::repositories::SettingsRepository::new(&conn).get().ok())
        .map(|s| s.shortcut_behavior)
        .unwrap_or_else(|| "open_dictation".to_string())
}

/// Start PTT recording. CAS guard: only transitions from Idle → Recording.
/// Spawns a thread because AudioService::start_recording is blocking.
pub(crate) fn ptt_start(app: &tauri::AppHandle) {
    let ptt = app.state::<PttState>();

    // CAS: only start if currently Idle.
    let should_start = {
        let mut guard = ptt.phase.lock().unwrap();
        if matches!(*guard, PttPhase::Idle) {
            *guard = PttPhase::Recording;
            ptt.cancel_flag.store(false, Ordering::SeqCst);
            true
        } else {
            false
        }
    };

    if !should_start {
        return;
    }

    // Capture the foreground window BEFORE showing the overlay.
    // This preserves the user's active target so we can restore focus
    // immediately before clipboard paste — even if the overlay was dragged.
    // Only an opaque handle is stored; no title or contents are read.
    let hwnd = capture_foreground_window();
    if let Ok(mut guard) = ptt.target_hwnd.lock() {
        *guard = if hwnd != 0 { Some(hwnd) } else { None };
    }

    // Show ptt-overlay before spawning the recording thread.
    // Never call set_focus() here — the active app must keep focus.
    if let Some(overlay) = app.get_webview_window("ptt-overlay") {
        let _ = overlay.show();
    }

    // Emit recording status directly to both windows immediately after show().
    // WebView2 may throttle JS in hidden windows; emit_to by label targets each
    // window's IPC directly, ensuring the event arrives even if the webview just
    // resumed from a background/throttled state.
    emit_ptt_status(app, "recording", "Recording…");

    let handle = app.clone();
    std::thread::spawn(move || {
        let ptt = handle.state::<PttState>();
        let audio_view = ptt.audio_state_view();
        match services::AudioService::start_recording(None, &audio_view) {
            Ok(_) => {
                // Re-emit recording status from the thread after start_recording
                // completes. The pre-spawn emit already fired; this confirms state.
                emit_ptt_status(&handle, "recording", "Recording…");
            }
            Err(e) => {
                ptt.set_phase(PttPhase::Idle);
                emit_ptt_status(&handle, "error", &format!("Could not start recording: {e}"));
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    });
}

/// Stop recording and spawn the full PTT pipeline.
/// Only acts if the current phase is Recording.
pub(crate) fn ptt_stop_and_run(app: &tauri::AppHandle) {
    let ptt = app.state::<PttState>();

    let is_recording = {
        let guard = ptt.phase.lock().unwrap();
        matches!(*guard, PttPhase::Recording)
    };

    if !is_recording {
        return;
    }

    let handle = app.clone();
    std::thread::spawn(move || {
        let ptt = handle.state::<PttState>();
        let db = handle.state::<db::AppState>();
        let svc = PttPipelineService::new(db.db.clone());
        svc.run_pipeline(&ptt, &handle);
    });
}

/// Toggle PTT: first press starts recording, second press stops and runs pipeline.
/// Ignores presses while the pipeline is in any other phase (transcribing etc.).
pub(crate) fn ptt_toggle(app: &tauri::AppHandle) {
    let ptt = app.state::<PttState>();

    let (is_idle, is_recording) = {
        let guard = ptt.phase.lock().unwrap();
        (
            matches!(*guard, PttPhase::Idle),
            matches!(*guard, PttPhase::Recording),
        )
    };

    if is_idle {
        ptt_start(app);
    } else if is_recording {
        ptt_stop_and_run(app);
    }
    // else: pipeline in progress — ignore the press
}
