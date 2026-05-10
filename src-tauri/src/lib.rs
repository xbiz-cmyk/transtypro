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

use std::sync::{Arc, Mutex};

use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

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
            app.manage(db::AppState {
                db: Arc::new(Mutex::new(conn)),
            });

            // Phase 3: Audio recording state (separate from DB state)
            let audio_dir = data_dir.join("audio");
            std::fs::create_dir_all(&audio_dir)?;
            app.manage(services::AudioState {
                audio_dir,
                recording: Arc::new(Mutex::new(None)),
                samples: Arc::new(Mutex::new(Vec::new())),
                sample_rate: Arc::new(Mutex::new(44100)),
                channels: Arc::new(Mutex::new(1)),
            });

            // Phase 7: Register global shortcut CommandOrControl+Shift+Space
            match "CommandOrControl+Shift+Space".parse::<tauri_plugin_global_shortcut::Shortcut>() {
                Ok(shortcut) => {
                    if let Err(e) = app.handle().global_shortcut().on_shortcut(
                        shortcut,
                        move |app_handle, _shortcut, event| {
                            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed
                            {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.unminimize();
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                                let _ = app_handle.emit("dictation-shortcut-pressed", ());
                            }
                        },
                    ) {
                        eprintln!("[phase7] global shortcut registration failed: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("[phase7] failed to parse shortcut string: {e}");
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
