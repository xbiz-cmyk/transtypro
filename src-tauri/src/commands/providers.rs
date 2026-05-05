use crate::db::AppState;
use crate::errors::AppError;
use crate::models::AiProvider;
use crate::services::ProvidersService;
use tauri::State;

#[tauri::command]
pub fn list_providers(state: State<'_, AppState>) -> Result<Vec<AiProvider>, AppError> {
    ProvidersService::new(state.db.clone()).list_providers()
}

#[tauri::command]
pub fn get_provider(id: String, state: State<'_, AppState>) -> Result<AiProvider, AppError> {
    ProvidersService::new(state.db.clone()).get_provider(&id)
}

#[tauri::command]
pub fn create_provider(
    name: String,
    provider_type: String,
    base_url: String,
    model: String,
    use_for_cleanup: bool,
    state: State<'_, AppState>,
) -> Result<AiProvider, AppError> {
    ProvidersService::new(state.db.clone()).create_provider(
        &name,
        &provider_type,
        &base_url,
        &model,
        use_for_cleanup,
    )
}

#[tauri::command]
pub fn update_provider(
    id: String,
    name: String,
    base_url: String,
    model: String,
    enabled: bool,
    use_for_cleanup: bool,
    state: State<'_, AppState>,
) -> Result<AiProvider, AppError> {
    ProvidersService::new(state.db.clone()).update_provider(
        &id,
        &name,
        &base_url,
        &model,
        enabled,
        use_for_cleanup,
    )
}

#[tauri::command]
pub fn delete_provider(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    ProvidersService::new(state.db.clone()).delete_provider(&id)
}

#[tauri::command]
pub fn test_provider_connection(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    ProvidersService::new(state.db.clone()).test_connection(&id)
}

#[tauri::command]
pub fn set_provider_api_key(
    id: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    ProvidersService::new(state.db.clone()).set_api_key(&id, &api_key)
}

#[tauri::command]
pub fn list_enabled_cleanup_providers(
    state: State<'_, AppState>,
) -> Result<Vec<AiProvider>, AppError> {
    ProvidersService::new(state.db.clone()).list_enabled_cleanup_providers()
}

/// Compatibility command — delegates to test_connection.
#[tauri::command]
pub fn test_provider_placeholder(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    ProvidersService::new(state.db.clone()).test_provider_placeholder(&id)
}
