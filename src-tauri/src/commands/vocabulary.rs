use tauri::State;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::VocabularyEntry;
use crate::services::VocabularyService;

#[tauri::command]
pub fn list_vocabulary(state: State<'_, AppState>) -> Result<Vec<VocabularyEntry>, AppError> {
    VocabularyService::new(state.db.clone()).list_vocabulary()
}

#[tauri::command]
pub fn add_vocabulary_entry(
    entry: VocabularyEntry,
    state: State<'_, AppState>,
) -> Result<VocabularyEntry, AppError> {
    VocabularyService::new(state.db.clone()).add_entry(entry)
}

#[tauri::command]
pub fn update_vocabulary_entry(
    entry: VocabularyEntry,
    state: State<'_, AppState>,
) -> Result<VocabularyEntry, AppError> {
    VocabularyService::new(state.db.clone()).update_entry(entry)
}

#[tauri::command]
pub fn delete_vocabulary_entry(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    VocabularyService::new(state.db.clone()).delete_entry(id)
}
