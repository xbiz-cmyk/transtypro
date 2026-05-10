/// transtypro — Tauri command wrappers.
///
/// Commands are thin wrappers that delegate to services.
/// They must not contain business logic.
use std::sync::{Arc, Mutex};

use crate::db::AppState;
use crate::errors::AppError;
use crate::models::StatusSummary;
use crate::services::{HistoryService, ProvidersService, SettingsService};

pub mod audio;
pub mod cleanup;
pub mod diagnostics;
pub mod history;
pub mod insertion;
pub mod modes;
pub mod privacy;
pub mod providers;
pub mod settings;
pub mod shortcut;
pub mod transcription;
pub mod vocabulary;

/// Verify frontend-backend IPC communication.
#[tauri::command]
pub fn ping() -> Result<String, AppError> {
    Ok("pong".to_string())
}

/// Return the current application version.
#[tauri::command]
pub fn get_app_version() -> Result<String, AppError> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Build a StatusSummary from a live database connection.
///
/// Extracted into a helper so it can be unit-tested with an in-memory DB
/// without requiring a Tauri State wrapper.
pub(crate) fn build_status_summary(
    db: Arc<Mutex<rusqlite::Connection>>,
) -> Result<StatusSummary, AppError> {
    let app_settings = SettingsService::new(db.clone()).get_settings()?;
    let history = HistoryService::new(db.clone()).list_history()?;
    let cleanup_providers = ProvidersService::new(db).list_enabled_cleanup_providers()?;

    Ok(StatusSummary {
        privacy_mode: if app_settings.local_only_mode {
            "local-only".to_string()
        } else {
            "cloud-enabled".to_string()
        },
        transcription_ready: app_settings.whisper_binary_path.is_some()
            && app_settings.whisper_model_path.is_some(),
        cleanup_provider: cleanup_providers.into_iter().next().map(|p| p.name),
        active_mode: app_settings.active_mode,
        history_count: history.len() as u32,
    })
}

/// Return a live application status summary for the home page.
#[tauri::command]
pub fn get_status_summary(state: tauri::State<'_, AppState>) -> Result<StatusSummary, AppError> {
    build_status_summary(state.db.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::models::HistoryEntry;
    use crate::services::HistoryService;
    use rusqlite::Connection;

    fn setup() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_status_summary_defaults() {
        let db = setup();
        let summary = build_status_summary(db).unwrap();
        // Migration 001 seeds local_only_mode = 0 (false) → "cloud-enabled"
        assert_eq!(summary.privacy_mode, "cloud-enabled");
        assert!(!summary.transcription_ready);
        assert!(summary.cleanup_provider.is_none());
        assert_eq!(summary.history_count, 0);
    }

    #[test]
    fn test_status_summary_history_count() {
        let db = setup();
        let svc = HistoryService::new(db.clone());
        svc.create_history_entry(HistoryEntry {
            id: String::new(),
            raw_text: "a".to_string(),
            cleaned_text: "a".to_string(),
            mode_used: "smart".to_string(),
            timestamp: String::new(),
            was_inserted: false,
        })
        .unwrap();
        let summary = build_status_summary(db).unwrap();
        assert_eq!(summary.history_count, 1);
    }

    #[test]
    fn test_status_summary_transcription_ready_requires_both_paths() {
        use crate::services::SettingsService;
        let db = setup();
        // Only binary path set — not ready
        let mut s = SettingsService::new(db.clone()).get_settings().unwrap();
        s.whisper_binary_path = Some("/usr/bin/whisper".to_string());
        s.whisper_model_path = None;
        SettingsService::new(db.clone()).update_settings(s).unwrap();
        assert!(
            !build_status_summary(db.clone())
                .unwrap()
                .transcription_ready
        );

        // Both paths set — ready
        let mut s2 = SettingsService::new(db.clone()).get_settings().unwrap();
        s2.whisper_model_path = Some("/models/ggml-base.bin".to_string());
        SettingsService::new(db.clone())
            .update_settings(s2)
            .unwrap();
        assert!(build_status_summary(db).unwrap().transcription_ready);
    }

    #[test]
    fn test_status_summary_privacy_mode_reflects_settings() {
        use crate::services::SettingsService;
        let db = setup();
        let mut s = SettingsService::new(db.clone()).get_settings().unwrap();
        s.local_only_mode = false;
        SettingsService::new(db.clone()).update_settings(s).unwrap();
        assert_eq!(
            build_status_summary(db).unwrap().privacy_mode,
            "cloud-enabled"
        );
    }
}
