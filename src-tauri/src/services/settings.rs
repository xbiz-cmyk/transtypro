use std::sync::{Arc, Mutex};

use crate::db::repositories::SettingsRepository;
use crate::errors::AppError;
use crate::models::AppSettings;

pub struct SettingsService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl SettingsService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        SettingsRepository::new(&conn).get()
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<(), AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        SettingsRepository::new(&conn).upsert(&settings)
    }
}
