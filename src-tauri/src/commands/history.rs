use crate::models::HistoryEntry;
use crate::services::HistoryService;

#[tauri::command]
pub fn list_history() -> Result<Vec<HistoryEntry>, String> {
    HistoryService.list_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history_entry(id: String) -> Result<HistoryEntry, String> {
    HistoryService.get_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_history_entry(id: String) -> Result<(), String> {
    HistoryService.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history() -> Result<(), String> {
    HistoryService.clear_history().map_err(|e| e.to_string())
}
