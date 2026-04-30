use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::AppSettings;
use crate::services::SettingsService;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    SettingsService::new(state.db.clone()).get_settings()
}

#[tauri::command]
pub fn update_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), AppError> {
    SettingsService::new(state.db.clone()).update_settings(settings)
}
