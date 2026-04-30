use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::HistoryEntry;
use crate::services::HistoryService;

#[tauri::command]
pub fn list_history(state: State<'_, AppState>) -> Result<Vec<HistoryEntry>, AppError> {
    HistoryService::new(state.db.clone()).list_history()
}

#[tauri::command]
pub fn get_history_entry(id: String, state: State<'_, AppState>) -> Result<HistoryEntry, AppError> {
    HistoryService::new(state.db.clone()).get_entry(id)
}

#[tauri::command]
pub fn delete_history_entry(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    HistoryService::new(state.db.clone()).delete_entry(id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), AppError> {
    HistoryService::new(state.db.clone()).clear_history()
}
