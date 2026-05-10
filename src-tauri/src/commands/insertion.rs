use tauri::Manager;

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::InsertionResult;
use crate::services::{HistoryService, InsertionService};

/// Insert `text` into the previously focused application via clipboard paste.
///
/// The main window is minimized first so the previous application can regain
/// focus before the paste key combination is simulated.  After the attempt,
/// the window is restored regardless of whether insertion succeeded.
///
/// Privacy: text is never logged, sent, or stored beyond the existing history entry.
#[tauri::command]
pub fn insert_text(
    text: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<InsertionResult, AppError> {
    // Minimize transtypro so the previously focused app regains focus.
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.minimize();
    }

    // Give the OS time to switch focus back to the target application.
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Perform clipboard write + paste simulation.
    let result = InsertionService::new(state.db.clone()).insert_text(text);

    // Restore the transtypro window so the user can see the result.
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }

    result
}

/// Mark an existing history entry as having been inserted into an external application.
#[tauri::command]
pub fn mark_history_inserted(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    HistoryService::new(state.db.clone()).mark_inserted(id)
}
