use crate::errors::AppError;
use crate::models::AppSettings;
use crate::services::SettingsService;

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, AppError> {
    SettingsService.get_settings()
}

#[tauri::command]
pub fn update_settings(settings: AppSettings) -> Result<(), AppError> {
    SettingsService.update_settings(settings)
}
