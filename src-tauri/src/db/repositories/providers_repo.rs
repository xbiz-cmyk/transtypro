use crate::errors::AppError;
use crate::models::AiProvider;
use rusqlite::params;

pub struct ProvidersRepository<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> ProvidersRepository<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn list_all(&self) -> Result<Vec<AiProvider>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, provider_type, base_url, model, \
                 enabled, use_for_cleanup, use_for_transcription, api_key_set \
                 FROM providers ORDER BY name",
            )
            .map_err(|e| AppError::StorageError(format!("prepare list_all: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(AiProvider {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider_type: row.get(2)?,
                    base_url: row.get(3)?,
                    model: row.get(4)?,
                    enabled: row.get::<_, i64>(5)? != 0,
                    use_for_cleanup: row.get::<_, i64>(6)? != 0,
                    use_for_transcription: row.get::<_, i64>(7)? != 0,
                    api_key_set: row.get::<_, i64>(8)? != 0,
                })
            })
            .map_err(|e| AppError::StorageError(format!("list_all: {e}")))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::StorageError(format!("collect list_all: {e}")))
    }

    pub fn get_by_id(&self, id: &str) -> Result<AiProvider, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, provider_type, base_url, model, \
                 enabled, use_for_cleanup, use_for_transcription, api_key_set \
                 FROM providers WHERE id = ?1",
                params![id],
                |row| {
                    Ok(AiProvider {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        provider_type: row.get(2)?,
                        base_url: row.get(3)?,
                        model: row.get(4)?,
                        enabled: row.get::<_, i64>(5)? != 0,
                        use_for_cleanup: row.get::<_, i64>(6)? != 0,
                        use_for_transcription: row.get::<_, i64>(7)? != 0,
                        api_key_set: row.get::<_, i64>(8)? != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::NotFound(format!("provider '{id}' not found"))
                }
                _ => AppError::StorageError(format!("get_by_id: {e}")),
            })
    }

    pub fn insert(&self, provider: &AiProvider) -> Result<(), AppError> {
        self.conn
            .execute(
                "INSERT INTO providers \
                 (id, name, provider_type, base_url, model, \
                  enabled, use_for_cleanup, use_for_transcription, api_key_set) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    provider.id,
                    provider.name,
                    provider.provider_type,
                    provider.base_url,
                    provider.model,
                    provider.enabled as i64,
                    provider.use_for_cleanup as i64,
                    provider.use_for_transcription as i64,
                    provider.api_key_set as i64,
                ],
            )
            .map(|_| ())
            .map_err(|e| AppError::StorageError(format!("insert provider: {e}")))
    }

    pub fn update(&self, provider: &AiProvider) -> Result<(), AppError> {
        let affected = self
            .conn
            .execute(
                "UPDATE providers SET \
                 name = ?1, base_url = ?2, model = ?3, \
                 enabled = ?4, use_for_cleanup = ?5, use_for_transcription = ?6 \
                 WHERE id = ?7",
                params![
                    provider.name,
                    provider.base_url,
                    provider.model,
                    provider.enabled as i64,
                    provider.use_for_cleanup as i64,
                    provider.use_for_transcription as i64,
                    provider.id,
                ],
            )
            .map_err(|e| AppError::StorageError(format!("update provider: {e}")))?;

        if affected == 0 {
            return Err(AppError::NotFound(format!(
                "provider '{}' not found",
                provider.id
            )));
        }
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let affected = self
            .conn
            .execute("DELETE FROM providers WHERE id = ?1", params![id])
            .map_err(|e| AppError::StorageError(format!("delete provider: {e}")))?;

        if affected == 0 {
            return Err(AppError::NotFound(format!("provider '{id}' not found")));
        }
        Ok(())
    }

    pub fn set_api_key_flag(&self, id: &str, set: bool) -> Result<(), AppError> {
        let affected = self
            .conn
            .execute(
                "UPDATE providers SET api_key_set = ?1 WHERE id = ?2",
                params![set as i64, id],
            )
            .map_err(|e| AppError::StorageError(format!("set_api_key_flag: {e}")))?;

        if affected == 0 {
            return Err(AppError::NotFound(format!("provider '{id}' not found")));
        }
        Ok(())
    }

    pub fn list_enabled_cleanup(&self) -> Result<Vec<AiProvider>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, provider_type, base_url, model, \
                 enabled, use_for_cleanup, use_for_transcription, api_key_set \
                 FROM providers WHERE enabled = 1 AND use_for_cleanup = 1 \
                 ORDER BY name",
            )
            .map_err(|e| AppError::StorageError(format!("prepare list_enabled_cleanup: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(AiProvider {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider_type: row.get(2)?,
                    base_url: row.get(3)?,
                    model: row.get(4)?,
                    enabled: row.get::<_, i64>(5)? != 0,
                    use_for_cleanup: row.get::<_, i64>(6)? != 0,
                    use_for_transcription: row.get::<_, i64>(7)? != 0,
                    api_key_set: row.get::<_, i64>(8)? != 0,
                })
            })
            .map_err(|e| AppError::StorageError(format!("list_enabled_cleanup: {e}")))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::StorageError(format!("collect list_enabled_cleanup: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn migrated() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn make_provider(id: &str, name: &str, provider_type: &str) -> AiProvider {
        AiProvider {
            id: id.to_string(),
            name: name.to_string(),
            provider_type: provider_type.to_string(),
            base_url: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
            enabled: true,
            use_for_cleanup: true,
            use_for_transcription: false,
            api_key_set: false,
        }
    }

    #[test]
    fn test_providers_empty_on_fresh_db() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let providers = repo.list_all().unwrap();
        assert!(providers.is_empty());
    }

    #[test]
    fn test_insert_and_list_provider() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("abc", "My Ollama", "ollama");
        repo.insert(&p).unwrap();
        let list = repo.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "My Ollama");
        assert_eq!(list[0].provider_type, "ollama");
        assert!(!list[0].api_key_set);
    }

    #[test]
    fn test_get_by_id_returns_provider() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("xyz", "OpenAI", "openai_compatible");
        repo.insert(&p).unwrap();
        let got = repo.get_by_id("xyz").unwrap();
        assert_eq!(got.id, "xyz");
        assert_eq!(got.provider_type, "openai_compatible");
    }

    #[test]
    fn test_get_by_id_not_found() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let err = repo.get_by_id("nonexistent").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_update_provider_changes_fields() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let mut p = make_provider("u1", "Old Name", "ollama");
        repo.insert(&p).unwrap();
        p.name = "New Name".to_string();
        p.model = "mistral".to_string();
        repo.update(&p).unwrap();
        let updated = repo.get_by_id("u1").unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.model, "mistral");
    }

    #[test]
    fn test_update_not_found_returns_error() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("missing", "X", "ollama");
        let err = repo.update(&p).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_delete_provider_removes_row() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("del1", "To Delete", "ollama");
        repo.insert(&p).unwrap();
        repo.delete("del1").unwrap();
        let list = repo.list_all().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_delete_not_found_returns_error() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let err = repo.delete("nothing").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_api_key_set_defaults_to_false() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("k0", "Test", "ollama");
        repo.insert(&p).unwrap();
        assert!(!repo.get_by_id("k0").unwrap().api_key_set);
    }

    #[test]
    fn test_set_api_key_flag_to_true() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let p = make_provider("k1", "OpenAI", "openai_compatible");
        repo.insert(&p).unwrap();
        repo.set_api_key_flag("k1", true).unwrap();
        assert!(repo.get_by_id("k1").unwrap().api_key_set);
    }

    #[test]
    fn test_set_api_key_flag_not_found_returns_error() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        let err = repo.set_api_key_flag("ghost", true).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_list_enabled_cleanup_filters_correctly() {
        let conn = migrated();
        let repo = ProvidersRepository::new(&conn);
        // Active cleanup provider — should appear
        repo.insert(&make_provider("a", "Active", "ollama")).unwrap();
        // Disabled provider — should NOT appear
        let mut p2 = make_provider("b", "Disabled", "ollama");
        p2.enabled = false;
        repo.insert(&p2).unwrap();
        // Not for cleanup — should NOT appear
        let mut p3 = make_provider("c", "NotCleanup", "ollama");
        p3.use_for_cleanup = false;
        repo.insert(&p3).unwrap();
        let result = repo.list_enabled_cleanup().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "a");
    }
}
