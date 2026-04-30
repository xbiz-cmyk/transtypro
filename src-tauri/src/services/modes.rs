use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::db::repositories::ModesRepository;
use crate::errors::AppError;
use crate::models::DictationMode;

pub struct ModesService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl ModesService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn list_modes(&self) -> Result<Vec<DictationMode>, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ModesRepository::new(&conn).list()
    }

    pub fn get_mode(&self, id: String) -> Result<DictationMode, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ModesRepository::new(&conn).get(&id)
    }

    /// Creates a new custom mode.  Generates a fresh UUID id; forces builtin = false.
    pub fn create_mode(&self, mut mode: DictationMode) -> Result<DictationMode, AppError> {
        mode.id = Uuid::new_v4().to_string();
        mode.builtin = false;
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ModesRepository::new(&conn).create(&mode)
    }

    pub fn update_mode(&self, mode: DictationMode) -> Result<DictationMode, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ModesRepository::new(&conn).update(&mode)
    }

    pub fn delete_mode(&self, id: String) -> Result<(), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ModesRepository::new(&conn).delete(&id)
    }
}
