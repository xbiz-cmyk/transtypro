use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::db::repositories::ProvidersRepository;
use crate::errors::AppError;
use crate::models::AiProvider;

pub struct ProvidersService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl ProvidersService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn list_providers(&self) -> Result<Vec<AiProvider>, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ProvidersRepository::new(&conn).list_all()
    }

    pub fn get_provider(&self, id: &str) -> Result<AiProvider, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ProvidersRepository::new(&conn).get_by_id(id)
    }

    pub fn create_provider(
        &self,
        name: &str,
        provider_type: &str,
        base_url: &str,
        model: &str,
        use_for_cleanup: bool,
    ) -> Result<AiProvider, AppError> {
        let provider = AiProvider {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            provider_type: provider_type.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            enabled: true,
            use_for_cleanup,
            use_for_transcription: false,
            api_key_set: false,
        };
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ProvidersRepository::new(&conn).insert(&provider)?;
        Ok(provider)
    }

    pub fn update_provider(
        &self,
        id: &str,
        name: &str,
        base_url: &str,
        model: &str,
        enabled: bool,
        use_for_cleanup: bool,
    ) -> Result<AiProvider, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        let repo = ProvidersRepository::new(&conn);
        let existing = repo.get_by_id(id)?;
        let updated = AiProvider {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            enabled,
            use_for_cleanup,
            provider_type: existing.provider_type,
            use_for_transcription: existing.use_for_transcription,
            api_key_set: existing.api_key_set,
        };
        repo.update(&updated)?;
        Ok(updated)
    }

    pub fn delete_provider(&self, id: &str) -> Result<(), AppError> {
        // Delete DB row first — surface NotFound immediately if provider doesn't exist.
        {
            let conn = self
                .db
                .lock()
                .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
            ProvidersRepository::new(&conn).delete(id)?;
        }
        // Remove keychain entry; ignore error if no key was stored.
        let _ = self.delete_api_key_from_keychain(id);
        Ok(())
    }

    /// Store an API key for a provider in the OS keychain.
    ///
    /// The key is NEVER stored in SQLite. Only `api_key_set = true` is written to the DB.
    /// If the keychain is unavailable, returns `ProviderError` — never falls back to plain text.
    ///
    /// Order of operations:
    ///   1. Verify provider exists (returns NotFound immediately if not).
    ///   2. Write key to OS keychain.
    ///   3. Set api_key_set = true in SQLite.
    ///   4. If step 3 fails, attempt to delete the just-written keychain entry before
    ///      returning the original DB error, preventing an orphan keychain entry.
    pub fn set_api_key(&self, id: &str, api_key: &str) -> Result<(), AppError> {
        // Step 1: confirm provider exists before touching keychain.
        {
            let conn = self
                .db
                .lock()
                .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
            ProvidersRepository::new(&conn).get_by_id(id)?;
        }

        // Step 2: write to OS keychain.
        let username = format!("provider:{id}");
        let entry = keyring::Entry::new("transtypro", &username)
            .map_err(|e| AppError::ProviderError(format!("Cannot access OS keychain: {e}")))?;
        entry.set_password(api_key).map_err(|e| {
            AppError::ProviderError(format!(
                "Cannot store API key: OS keychain unavailable. {e}"
            ))
        })?;

        // Step 3: update DB flag; on failure roll back keychain entry.
        let db_result = {
            let conn = self
                .db
                .lock()
                .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
            ProvidersRepository::new(&conn).set_api_key_flag(id, true)
        };
        if let Err(db_err) = db_result {
            // Best-effort rollback — ignore secondary error.
            let _ = entry.delete_credential();
            return Err(db_err);
        }
        Ok(())
    }

    /// Retrieve an API key from the OS keychain.
    ///
    /// Internal only — never exposed as a Tauri command.
    pub fn get_api_key(&self, id: &str) -> Result<String, AppError> {
        let username = format!("provider:{id}");
        let entry = keyring::Entry::new("transtypro", &username)
            .map_err(|e| AppError::ProviderError(format!("Cannot access OS keychain: {e}")))?;
        entry
            .get_password()
            .map_err(|e| AppError::ProviderError(format!("API key not found in keychain: {e}")))
    }

    fn delete_api_key_from_keychain(&self, id: &str) -> Result<(), AppError> {
        let username = format!("provider:{id}");
        let entry = keyring::Entry::new("transtypro", &username)
            .map_err(|e| AppError::ProviderError(format!("Cannot access OS keychain: {e}")))?;
        let _ = entry.delete_credential();
        Ok(())
    }

    /// Test the connection to a provider.
    ///
    /// Ollama: GET {base_url}/api/tags — returns success message on 200.
    /// OpenAI-compatible: GET {base_url}/models — 200 or 401 both count as reachable.
    pub fn test_connection(&self, id: &str) -> Result<String, AppError> {
        let provider = self.get_provider(id)?;
        match provider.provider_type.as_str() {
            "ollama" => {
                let url = format!("{}/api/tags", provider.base_url.trim_end_matches('/'));
                ureq::get(&url)
                    .call()
                    .map(|_| format!("Ollama reachable at {}", provider.base_url))
                    .map_err(|e| {
                        AppError::ProviderError(format!(
                            "Cannot reach Ollama at {}: {e}",
                            provider.base_url
                        ))
                    })
            }
            "openai_compatible" => {
                let url = format!("{}/models", provider.base_url.trim_end_matches('/'));
                let api_key = self.get_api_key(id).unwrap_or_default();
                match ureq::get(&url)
                    .set("Authorization", &format!("Bearer {api_key}"))
                    .call()
                {
                    Ok(_) => Ok("Provider reachable".to_string()),
                    // 401 = URL valid, key is wrong — host is reachable
                    Err(ureq::Error::Status(401, _)) => Ok("Provider reachable".to_string()),
                    Err(e) => Err(AppError::ProviderError(format!(
                        "Cannot reach provider at {}: {e}",
                        provider.base_url
                    ))),
                }
            }
            t => Err(AppError::ProviderError(format!(
                "Unknown provider type for connection test: {t}"
            ))),
        }
    }

    /// Compatibility placeholder — delegates to test_connection.
    pub fn test_provider_placeholder(&self, id: &str) -> Result<String, AppError> {
        self.test_connection(id)
    }

    pub fn list_enabled_cleanup_providers(&self) -> Result<Vec<AiProvider>, AppError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::StorageError("database lock is poisoned".into()))?;
        ProvidersRepository::new(&conn).list_enabled_cleanup()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn migrated_db() -> Arc<Mutex<rusqlite::Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    // ── set_api_key safety ────────────────────────────────────────────────────

    /// set_api_key must return NotFound for an unknown provider ID without
    /// ever reaching the OS keychain.  This is purely a DB-level check so it
    /// runs without real keychain access.
    #[test]
    fn test_set_api_key_missing_provider_returns_not_found() {
        let db = migrated_db();
        let svc = ProvidersService::new(db);
        let err = svc
            .set_api_key("nonexistent-provider-id", "sk-test")
            .unwrap_err();
        assert!(
            matches!(err, AppError::NotFound(_)),
            "expected NotFound for unknown provider ID, got {err:?}"
        );
    }
}
