use crate::errors::AppError;
use crate::models::VocabularyEntry;
use crate::services::VocabularyService;

#[tauri::command]
pub fn list_vocabulary() -> Result<Vec<VocabularyEntry>, AppError> {
    VocabularyService.list_vocabulary()
}

#[tauri::command]
pub fn add_vocabulary_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, AppError> {
    VocabularyService.add_entry(entry)
}

#[tauri::command]
pub fn update_vocabulary_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, AppError> {
    VocabularyService.update_entry(entry)
}

#[tauri::command]
pub fn delete_vocabulary_entry(id: String) -> Result<(), AppError> {
    VocabularyService.delete_entry(id)
}
