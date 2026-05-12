use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::db::repositories::SettingsRepository;
use crate::db::AppState;
use crate::errors::AppError;

/// Update the global dictation shortcut at runtime.
///
/// Strategy:
/// 1. Validate the new shortcut string (non-empty, ≤ 100 chars, parseable).
/// 2. If the new shortcut equals the current one, return success immediately.
/// 3. Register the new shortcut with the full dictation handler (register-first).
///    If registration fails, the old shortcut remains active and an error is returned.
/// 4. Unregister the old shortcut (non-fatal warning if it fails).
/// 5. Persist the new shortcut to the database.
///
/// Returns the accepted shortcut string on success.
#[tauri::command]
pub fn update_shortcut(
    shortcut: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, AppError> {
    let trimmed = shortcut.trim().to_string();

    // --- Validation ---
    if trimmed.is_empty() {
        return Err(AppError::ValidationError(
            "shortcut cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > 100 {
        return Err(AppError::ValidationError(
            "shortcut string is too long (max 100 characters)".to_string(),
        ));
    }

    // Parse early to catch invalid key names before touching registration.
    let new_parsed = trimmed
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| AppError::ValidationError(format!("invalid shortcut '{trimmed}': {e}")))?;

    // --- Read current shortcut ---
    let old_shortcut = {
        let conn = state
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        SettingsRepository::new(&conn).get()?.shortcut
    };

    // If unchanged, skip all OS calls.
    if trimmed == old_shortcut {
        return Ok(trimmed);
    }

    // --- Register new shortcut with the behavior-aware handler (register-first) ---
    // Must mirror the handler in lib.rs setup() exactly so that runtime shortcut
    // updates respect the shortcut_behavior setting (PTT toggle, open_dictation, etc.).
    let app_for_handler = app_handle.clone();
    app_handle
        .global_shortcut()
        .on_shortcut(new_parsed, move |app_handle, _shortcut, event| {
            let behavior = {
                let raw = crate::read_shortcut_behavior(app_handle);
                if cfg!(target_os = "windows") && raw == "push_to_talk_hold" {
                    "push_to_talk_toggle".to_string()
                } else {
                    raw
                }
            };

            match event.state() {
                tauri_plugin_global_shortcut::ShortcutState::Pressed => match behavior.as_str() {
                    "open_dictation" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app_handle.emit("dictation-shortcut-pressed", ());
                    }
                    "push_to_talk_hold" => {
                        crate::ptt_start(app_handle);
                    }
                    "push_to_talk_toggle" => {
                        crate::ptt_toggle(app_handle);
                    }
                    _ => {
                        eprintln!(
                            "[shortcut] unknown shortcut_behavior '{behavior}', \
                                 falling back to open_dictation"
                        );
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app_handle.emit("dictation-shortcut-pressed", ());
                    }
                },
                tauri_plugin_global_shortcut::ShortcutState::Released => {
                    if behavior == "push_to_talk_hold" {
                        crate::ptt_stop_and_run(app_handle);
                    }
                }
            }
        })
        .map_err(|e| {
            AppError::ValidationError(format!("could not register shortcut '{trimmed}': {e}"))
        })?;

    // --- Unregister old shortcut (non-fatal) ---
    if let Ok(old_parsed) = old_shortcut.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        if let Err(e) = app_for_handler.global_shortcut().unregister(old_parsed) {
            eprintln!("[shortcut] failed to unregister old shortcut '{old_shortcut}': {e}");
        }
    } else {
        eprintln!("[shortcut] could not parse old shortcut '{old_shortcut}' for unregistration");
    }

    // --- Persist new shortcut to DB ---
    {
        let conn = state
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        let mut settings = SettingsRepository::new(&conn).get()?;
        settings.shortcut = trimmed.clone();
        SettingsRepository::new(&conn).upsert(&settings)?;
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    // Validate the shortcut validation rules without requiring a live AppHandle.
    // Full registration is tested via manual QA (requires a running Tauri app).
    fn validate_shortcut(shortcut: &str) -> Result<String, String> {
        let trimmed = shortcut.trim().to_string();
        if trimmed.is_empty() {
            return Err("shortcut cannot be empty".to_string());
        }
        if trimmed.len() > 100 {
            return Err("shortcut string is too long (max 100 characters)".to_string());
        }
        Ok(trimmed)
    }

    #[test]
    fn test_shortcut_validation_empty() {
        assert!(validate_shortcut("").is_err());
        assert!(validate_shortcut("   ").is_err());
    }

    #[test]
    fn test_shortcut_validation_too_long() {
        let long = "A".repeat(101);
        assert!(validate_shortcut(&long).is_err());
    }

    #[test]
    fn test_shortcut_validation_accepts_default() {
        assert_eq!(
            validate_shortcut("CommandOrControl+Shift+Space").unwrap(),
            "CommandOrControl+Shift+Space"
        );
    }

    #[test]
    fn test_shortcut_validation_trims_whitespace() {
        assert_eq!(
            validate_shortcut("  CommandOrControl+Shift+D  ").unwrap(),
            "CommandOrControl+Shift+D"
        );
    }

    #[test]
    fn test_shortcut_validation_max_length_passes() {
        let exactly_100 = "A".repeat(100);
        assert!(validate_shortcut(&exactly_100).is_ok());
    }
}
