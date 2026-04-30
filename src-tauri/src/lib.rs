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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            commands::vocabulary::delete_vocabulary_entry,
            // Phase 2 — history
            commands::history::list_history,
            commands::history::get_history_entry,
            commands::history::delete_history_entry,
            commands::history::clear_history,
            // Phase 2 — privacy
            commands::privacy::get_privacy_status,
            commands::privacy::enforce_privacy_preview,
            // Phase 2 — providers
            commands::providers::list_providers,
            commands::providers::get_provider,
            commands::providers::test_provider_placeholder,
            // Phase 2 — diagnostics
            commands::diagnostics::run_diagnostics_placeholder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
