use crate::errors::AppError;
use crate::models::AiProvider;
use crate::services::ProvidersService;

#[tauri::command]
pub fn list_providers() -> Result<Vec<AiProvider>, AppError> {
    ProvidersService.list_providers()
}

#[tauri::command]
pub fn get_provider(id: String) -> Result<AiProvider, AppError> {
    ProvidersService.get_provider(id)
}

#[tauri::command]
pub fn test_provider_placeholder(id: String) -> Result<String, AppError> {
    ProvidersService.test_provider_placeholder(id)
}
