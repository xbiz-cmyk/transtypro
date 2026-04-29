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
            commands::ping,
            commands::get_app_version,
            commands::get_status_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
