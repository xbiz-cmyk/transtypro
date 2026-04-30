use crate::models::AppSettings;
use crate::services::SettingsService;

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    SettingsService.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(settings: AppSettings) -> Result<(), String> {
    SettingsService
        .update_settings(settings)
        .map_err(|e| e.to_string())
}
