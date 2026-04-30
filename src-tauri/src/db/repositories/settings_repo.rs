use crate::errors::AppError;
use crate::models::AppSettings;
use rusqlite::Connection;

pub struct SettingsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Returns the persisted settings row, or safe defaults if no row exists.
    pub fn get(&self) -> Result<AppSettings, AppError> {
        let result = self.conn.query_row(
            "SELECT active_mode, local_only_mode, theme, retention_days,
                    audio_history_enabled, clipboard_restore_enabled,
                    whisper_binary_path, whisper_model_path
             FROM settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    active_mode: row.get(0)?,
                    local_only_mode: row.get::<_, i64>(1)? != 0,
                    theme: row.get(2)?,
                    retention_days: row.get::<_, i64>(3)? as u32,
                    audio_history_enabled: row.get::<_, i64>(4)? != 0,
                    clipboard_restore_enabled: row.get::<_, i64>(5)? != 0,
                    whisper_binary_path: row.get(6)?,
                    whisper_model_path: row.get(7)?,
                })
            },
        );

        match result {
            Ok(settings) => Ok(settings),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings {
                active_mode: "smart".to_string(),
                local_only_mode: false,
                theme: "dark".to_string(),
                retention_days: 30,
                audio_history_enabled: false,
                clipboard_restore_enabled: false,
                whisper_binary_path: None,
                whisper_model_path: None,
            }),
            Err(e) => Err(AppError::StorageError(e.to_string())),
        }
    }

    /// Upserts the single settings row.  Atomic: replaces all fields at once.
    pub fn upsert(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.conn
            .execute(
                "INSERT INTO settings (
                    id, active_mode, local_only_mode, theme,
                    retention_days, audio_history_enabled, clipboard_restore_enabled,
                    whisper_binary_path, whisper_model_path
                 ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    active_mode               = excluded.active_mode,
                    local_only_mode           = excluded.local_only_mode,
                    theme                     = excluded.theme,
                    retention_days            = excluded.retention_days,
                    audio_history_enabled     = excluded.audio_history_enabled,
                    clipboard_restore_enabled = excluded.clipboard_restore_enabled,
                    whisper_binary_path       = excluded.whisper_binary_path,
                    whisper_model_path        = excluded.whisper_model_path",
                rusqlite::params![
                    settings.active_mode,
                    settings.local_only_mode as i64,
                    settings.theme,
                    settings.retention_days as i64,
                    settings.audio_history_enabled as i64,
                    settings.clipboard_restore_enabled as i64,
                    settings.whisper_binary_path,
                    settings.whisper_model_path,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_get_returns_defaults() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let s = repo.get().unwrap();
        assert_eq!(s.theme, "dark");
        assert_eq!(s.active_mode, "smart");
        assert!(!s.local_only_mode);
        assert_eq!(s.retention_days, 30);
        assert!(!s.audio_history_enabled);
        assert!(!s.clipboard_restore_enabled);
    }

    #[test]
    fn test_upsert_persists_change() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let mut s = repo.get().unwrap();
        s.theme = "light".to_string();
        s.local_only_mode = true;
        s.retention_days = 7;
        repo.upsert(&s).unwrap();
        let s2 = repo.get().unwrap();
        assert_eq!(s2.theme, "light");
        assert!(s2.local_only_mode);
        assert_eq!(s2.retention_days, 7);
    }

    #[test]
    fn test_upsert_idempotent() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let s = repo.get().unwrap();
        repo.upsert(&s).unwrap();
        repo.upsert(&s).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_defaults_whisper_paths_to_none() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let s = repo.get().unwrap();
        assert!(
            s.whisper_binary_path.is_none(),
            "whisper_binary_path should default to None"
        );
        assert!(
            s.whisper_model_path.is_none(),
            "whisper_model_path should default to None"
        );
    }

    #[test]
    fn test_upsert_persists_whisper_binary_path() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let mut s = repo.get().unwrap();
        s.whisper_binary_path = Some("/usr/local/bin/whisper".to_string());
        repo.upsert(&s).unwrap();
        let s2 = repo.get().unwrap();
        assert_eq!(
            s2.whisper_binary_path.as_deref(),
            Some("/usr/local/bin/whisper")
        );
    }

    #[test]
    fn test_upsert_persists_whisper_model_path() {
        let conn = setup();
        let repo = SettingsRepository::new(&conn);
        let mut s = repo.get().unwrap();
        s.whisper_model_path = Some("/models/ggml-base.en.bin".to_string());
        repo.upsert(&s).unwrap();
        let s2 = repo.get().unwrap();
        assert_eq!(
            s2.whisper_model_path.as_deref(),
            Some("/models/ggml-base.en.bin")
        );
    }
}
