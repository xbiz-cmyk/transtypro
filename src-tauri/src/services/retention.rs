use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::db::repositories::{HistoryRepository, SettingsRepository};
use crate::errors::AppError;
use crate::models::RetentionResult;

pub struct RetentionService {
    db: Arc<Mutex<rusqlite::Connection>>,
    audio_dir: PathBuf,
}

impl RetentionService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>, audio_dir: PathBuf) -> Self {
        Self { db, audio_dir }
    }

    /// Delete history entries that exceed the configured retention window.
    ///
    /// Returns the count of deleted rows.
    /// If `settings.retention_days == 0`, returns `Ok(0)` immediately (keep forever).
    pub fn apply_history_retention(&self) -> Result<u32, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        let settings = SettingsRepository::new(&conn).get()?;
        HistoryRepository::new(&conn).delete_older_than(settings.retention_days)
    }

    /// Delete stale WAV files from the audio directory according to settings.
    ///
    /// Rules:
    /// - `audio_history_enabled = false` → delete ALL regular `.wav` files (crash/cancel leftovers).
    /// - `audio_history_enabled = true`, `retention_days > 0` → delete `.wav` files older than N days.
    /// - `audio_history_enabled = true`, `retention_days = 0` → keep forever, return `Ok(0)`.
    ///
    /// Safety checks enforced before every file deletion:
    /// 1. `path.starts_with(&audio_dir)` — skip and warn if not satisfied.
    /// 2. `metadata.is_file()` — skip directories and symlinks.
    /// 3. Extension must be `"wav"` — skip other file types.
    /// 4. Per-file errors are non-fatal — log with `eprintln!` and continue.
    pub fn apply_audio_retention(&self) -> Result<u32, AppError> {
        let (audio_history_enabled, retention_days) = {
            let conn = self
                .db
                .lock()
                .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
            let s = SettingsRepository::new(&conn).get()?;
            (s.audio_history_enabled, s.retention_days)
        };

        // Keep forever when audio history is on and retention_days = 0.
        if audio_history_enabled && retention_days == 0 {
            return Ok(0);
        }

        // Compute cutoff time when audio history is on and retention_days > 0.
        let cutoff: Option<SystemTime> = if audio_history_enabled {
            Some(SystemTime::now() - Duration::from_secs(u64::from(retention_days) * 86_400))
        } else {
            None // audio_history_enabled = false → delete all .wav files
        };

        let entries = match std::fs::read_dir(&self.audio_dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "[retention] cannot read audio dir {}: {e}",
                    self.audio_dir.display()
                );
                return Ok(0);
            }
        };

        let mut deleted: u32 = 0;

        for entry in entries.flatten() {
            let path = entry.path();

            // Safety rule 1: path must be inside audio_dir.
            if !path.starts_with(&self.audio_dir) {
                eprintln!(
                    "[retention] skipping path outside audio_dir: {}",
                    path.display()
                );
                continue;
            }

            // Safety rule 2: must be a regular file (not a directory or symlink).
            let metadata = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "[retention] cannot read metadata for {}: {e}",
                        path.display()
                    );
                    continue;
                }
            };
            if !metadata.is_file() {
                continue;
            }

            // Safety rule 3: must have a `.wav` extension (case-sensitive).
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "wav" {
                continue;
            }

            // If a cutoff is set, only delete files older than the cutoff.
            if let Some(cutoff_time) = cutoff {
                let modified = match metadata.modified() {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("[retention] cannot read mtime for {}: {e}", path.display());
                        continue;
                    }
                };
                if modified > cutoff_time {
                    continue; // file is recent enough, keep it
                }
            }

            // Safety rule 4: per-file errors are non-fatal.
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("[retention] failed to delete {}: {e}", path.display());
            } else {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Run both history and audio retention cleanup.
    pub fn apply_all(&self) -> Result<RetentionResult, AppError> {
        let deleted_history_count = self.apply_history_retention()?;
        let deleted_wav_count = self.apply_audio_retention()?;
        Ok(RetentionResult {
            deleted_history_count,
            deleted_wav_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::repositories::SettingsRepository;
    use rusqlite::Connection;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn insert_history(db: &Arc<Mutex<Connection>>, id: &str, timestamp: &str) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO history (id, raw_text, cleaned_text, mode_used, timestamp, was_inserted)
             VALUES (?1, 'text', 'text', 'smart', ?2, 0)",
            rusqlite::params![id, timestamp],
        )
        .unwrap();
    }

    fn unique_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tt_retention_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_history_retention_zero_days_keeps_all() {
        let db = setup_db();
        // Override retention_days to 0 (keep forever).
        {
            let conn = db.lock().unwrap();
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.retention_days = 0;
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        insert_history(&db, "h1", "2000-01-01T00:00:00Z");

        let dir = unique_dir();
        let svc = RetentionService::new(db.clone(), dir.clone());
        let deleted = svc.apply_history_retention().unwrap();
        assert_eq!(deleted, 0, "retention_days=0 must delete nothing");

        let conn = db.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "entry must still exist");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_history_retention_deletes_old_keeps_recent() {
        let db = setup_db();
        // Default retention_days = 30 from migrations.
        insert_history(&db, "old", "2000-06-15T12:00:00Z");
        insert_history(&db, "new", "2099-12-31T23:59:59Z");

        let dir = unique_dir();
        let svc = RetentionService::new(db.clone(), dir.clone());
        let deleted = svc.apply_history_retention().unwrap();
        assert_eq!(deleted, 1, "only the old entry should be deleted");

        let conn = db.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "the recent entry must survive");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_audio_retention_deletes_wav_when_history_disabled() {
        let db = setup_db();
        // audio_history_enabled defaults to false — should delete all .wav files.
        let dir = unique_dir();
        let wav = dir.join("recording.wav");
        std::fs::write(&wav, b"RIFF").unwrap();

        let svc = RetentionService::new(db, dir.clone());
        let deleted = svc.apply_audio_retention().unwrap();
        assert_eq!(deleted, 1);
        assert!(!wav.exists(), "wav file must be deleted");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_audio_retention_skips_non_wav_files() {
        let db = setup_db();
        // audio_history_enabled = false, but file is .txt — must be skipped.
        let dir = unique_dir();
        let txt = dir.join("notes.txt");
        std::fs::write(&txt, b"hello").unwrap();

        let svc = RetentionService::new(db, dir.clone());
        let deleted = svc.apply_audio_retention().unwrap();
        assert_eq!(deleted, 0, "non-wav file must not be deleted");
        assert!(txt.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_audio_retention_keeps_forever_when_history_enabled_and_zero_days() {
        let db = setup_db();
        {
            let conn = db.lock().unwrap();
            let mut s = SettingsRepository::new(&conn).get().unwrap();
            s.audio_history_enabled = true;
            s.retention_days = 0;
            SettingsRepository::new(&conn).upsert(&s).unwrap();
        }
        let dir = unique_dir();
        let wav = dir.join("keep.wav");
        std::fs::write(&wav, b"RIFF").unwrap();

        let svc = RetentionService::new(db, dir.clone());
        let deleted = svc.apply_audio_retention().unwrap();
        assert_eq!(
            deleted, 0,
            "retention_days=0 with audio history on must keep files"
        );
        assert!(wav.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_apply_all_returns_combined_result() {
        let db = setup_db();
        // retention_days = 30 (default); audio_history_enabled = false (default).
        insert_history(&db, "old", "2000-01-01T00:00:00Z");

        let dir = unique_dir();
        let wav = dir.join("stale.wav");
        std::fs::write(&wav, b"RIFF").unwrap();

        let svc = RetentionService::new(db, dir.clone());
        let result = svc.apply_all().unwrap();
        assert_eq!(result.deleted_history_count, 1);
        assert_eq!(result.deleted_wav_count, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
