use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::db::repositories::VocabularyRepository;
use crate::errors::AppError;
use crate::models::VocabularyEntry;

pub struct VocabularyService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl VocabularyService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn list_vocabulary(&self) -> Result<Vec<VocabularyEntry>, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        VocabularyRepository::new(&conn).list()
    }

    /// Adds a new vocabulary entry.  Always generates a fresh UUID id.
    pub fn add_entry(&self, mut entry: VocabularyEntry) -> Result<VocabularyEntry, AppError> {
        entry.id = Uuid::new_v4().to_string();
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        VocabularyRepository::new(&conn).create(&entry)
    }

    pub fn update_entry(&self, entry: VocabularyEntry) -> Result<VocabularyEntry, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        VocabularyRepository::new(&conn).update(&entry)
    }

    pub fn delete_entry(&self, id: String) -> Result<(), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        VocabularyRepository::new(&conn).delete(&id)
    }
}
