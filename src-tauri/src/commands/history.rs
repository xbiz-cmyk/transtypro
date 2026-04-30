use crate::errors::AppError;
use crate::models::HistoryEntry;
use crate::services::HistoryService;

#[tauri::command]
pub fn list_history() -> Result<Vec<HistoryEntry>, AppError> {
    HistoryService.list_history()
}

#[tauri::command]
pub fn get_history_entry(id: String) -> Result<HistoryEntry, AppError> {
    HistoryService.get_entry(id)
}

#[tauri::command]
pub fn delete_history_entry(id: String) -> Result<(), AppError> {
    HistoryService.delete_entry(id)
}

#[tauri::command]
pub fn clear_history() -> Result<(), AppError> {
    HistoryService.clear_history()
}
