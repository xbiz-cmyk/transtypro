use crate::models::AiProvider;
use crate::services::ProvidersService;

#[tauri::command]
pub fn list_providers() -> Result<Vec<AiProvider>, String> {
    ProvidersService.list_providers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_provider(id: String) -> Result<AiProvider, String> {
    ProvidersService.get_provider(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_provider_placeholder(id: String) -> Result<String, String> {
    ProvidersService
        .test_provider_placeholder(id)
        .map_err(|e| e.to_string())
}
