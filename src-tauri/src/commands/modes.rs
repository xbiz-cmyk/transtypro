use crate::models::DictationMode;
use crate::services::ModesService;

#[tauri::command]
pub fn list_modes() -> Result<Vec<DictationMode>, String> {
    ModesService.list_modes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_mode(id: String) -> Result<DictationMode, String> {
    ModesService.get_mode(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_mode(mode: DictationMode) -> Result<DictationMode, String> {
    ModesService.create_mode(mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_mode(mode: DictationMode) -> Result<DictationMode, String> {
    ModesService.update_mode(mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_mode(id: String) -> Result<(), String> {
    ModesService.delete_mode(id).map_err(|e| e.to_string())
}
