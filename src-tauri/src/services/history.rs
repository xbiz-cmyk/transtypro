use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::db::repositories::HistoryRepository;
use crate::errors::AppError;
use crate::models::HistoryEntry;

pub struct HistoryService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl HistoryService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn list_history(&self) -> Result<Vec<HistoryEntry>, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        HistoryRepository::new(&conn).list()
    }

    pub fn get_entry(&self, id: String) -> Result<HistoryEntry, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        HistoryRepository::new(&conn).get(&id)
    }

    pub fn delete_entry(&self, id: String) -> Result<(), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        HistoryRepository::new(&conn).delete(&id)
    }

    pub fn clear_history(&self) -> Result<(), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        HistoryRepository::new(&conn).clear()
    }

    /// Creates a new history entry.  Generates a UUID id and an ISO 8601 timestamp.
    /// Called internally by the dictation pipeline (Phase 6); no Tauri command exposed yet.
    pub fn create_history_entry(&self, mut entry: HistoryEntry) -> Result<HistoryEntry, AppError> {
        entry.id = Uuid::new_v4().to_string();
        entry.timestamp = Utc::now().to_rfc3339();
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        HistoryRepository::new(&conn).create(&entry)
    }
}
