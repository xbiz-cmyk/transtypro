use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::DictationMode;
use crate::services::ModesService;

#[tauri::command]
pub fn list_modes(state: State<'_, AppState>) -> Result<Vec<DictationMode>, AppError> {
    ModesService::new(state.db.clone()).list_modes()
}

#[tauri::command]
pub fn get_mode(id: String, state: State<'_, AppState>) -> Result<DictationMode, AppError> {
    ModesService::new(state.db.clone()).get_mode(id)
}

#[tauri::command]
pub fn create_mode(
    mode: DictationMode,
    state: State<'_, AppState>,
) -> Result<DictationMode, AppError> {
    ModesService::new(state.db.clone()).create_mode(mode)
}

#[tauri::command]
pub fn update_mode(
    mode: DictationMode,
    state: State<'_, AppState>,
) -> Result<DictationMode, AppError> {
    ModesService::new(state.db.clone()).update_mode(mode)
}

#[tauri::command]
pub fn delete_mode(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    ModesService::new(state.db.clone()).delete_mode(id)
}
