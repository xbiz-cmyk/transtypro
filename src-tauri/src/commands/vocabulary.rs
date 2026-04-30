use crate::models::VocabularyEntry;
use crate::services::VocabularyService;

#[tauri::command]
pub fn list_vocabulary() -> Result<Vec<VocabularyEntry>, String> {
    VocabularyService
        .list_vocabulary()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_vocabulary_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, String> {
    VocabularyService
        .add_entry(entry)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_vocabulary_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, String> {
    VocabularyService
        .update_entry(entry)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_vocabulary_entry(id: String) -> Result<(), String> {
    VocabularyService
        .delete_entry(id)
        .map_err(|e| e.to_string())
}
