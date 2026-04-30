use crate::errors::AppError;
use crate::models::DictationMode;
use crate::services::ModesService;

#[tauri::command]
pub fn list_modes() -> Result<Vec<DictationMode>, AppError> {
    ModesService.list_modes()
}

#[tauri::command]
pub fn get_mode(id: String) -> Result<DictationMode, AppError> {
    ModesService.get_mode(id)
}

#[tauri::command]
pub fn create_mode(mode: DictationMode) -> Result<DictationMode, AppError> {
    ModesService.create_mode(mode)
}

#[tauri::command]
pub fn update_mode(mode: DictationMode) -> Result<DictationMode, AppError> {
    ModesService.update_mode(mode)
}

#[tauri::command]
pub fn delete_mode(id: String) -> Result<(), AppError> {
    ModesService.delete_mode(id)
}
