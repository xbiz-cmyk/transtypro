use crate::errors::AppError;
use crate::models::DictationMode;
use rusqlite::{params, Connection};

pub struct ModesRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ModesRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DictationMode> {
        Ok(DictationMode {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            system_prompt: row.get(3)?,
            active: row.get::<_, i64>(4)? != 0,
            builtin: row.get::<_, i64>(5)? != 0,
        })
    }

    /// Lists all modes: built-in first, then custom, alphabetically within each group.
    pub fn list(&self) -> Result<Vec<DictationMode>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, description, system_prompt, active, builtin
                 FROM modes
                 ORDER BY builtin DESC, name ASC",
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let modes = stmt
            .query_map([], Self::map_row)
            .map_err(|e| AppError::StorageError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(modes)
    }

    /// Returns a single mode by id, or `NotFound`.
    pub fn get(&self, id: &str) -> Result<DictationMode, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, description, system_prompt, active, builtin
                 FROM modes WHERE id = ?1",
                params![id],
                Self::map_row,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("mode '{id}'")),
                other => AppError::StorageError(other.to_string()),
            })
    }

    /// Creates a new custom mode.  Forces `builtin = false` regardless of input.
    pub fn create(&self, mode: &DictationMode) -> Result<DictationMode, AppError> {
        self.conn
            .execute(
                "INSERT INTO modes (id, name, description, system_prompt, active, builtin)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                params![
                    mode.id,
                    mode.name,
                    mode.description,
                    mode.system_prompt,
                    mode.active as i64,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        self.get(&mode.id)
    }

    /// Updates a custom mode.  Returns `ValidationError` for built-in modes.
    pub fn update(&self, mode: &DictationMode) -> Result<DictationMode, AppError> {
        let existing = self.get(&mode.id)?;
        if existing.builtin {
            return Err(AppError::ValidationError(format!(
                "built-in mode '{}' cannot be modified",
                mode.id
            )));
        }
        self.conn
            .execute(
                "UPDATE modes
                 SET name = ?1, description = ?2, system_prompt = ?3, active = ?4
                 WHERE id = ?5",
                params![
                    mode.name,
                    mode.description,
                    mode.system_prompt,
                    mode.active as i64,
                    mode.id,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        self.get(&mode.id)
    }

    /// Deletes a custom mode.  Returns `ValidationError` for built-in modes.
    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let existing = self.get(id)?;
        if existing.builtin {
            return Err(AppError::ValidationError(format!(
                "built-in mode '{id}' cannot be deleted"
            )));
        }
        self.conn
            .execute("DELETE FROM modes WHERE id = ?1", params![id])
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

    fn custom_mode(id: &str) -> DictationMode {
        DictationMode {
            id: id.to_string(),
            name: format!("Custom {id}"),
            description: "test mode".to_string(),
            system_prompt: "".to_string(),
            active: false,
            builtin: false,
        }
    }

    #[test]
    fn test_list_returns_10_builtins() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let modes = repo.list().unwrap();
        assert_eq!(modes.len(), 10);
        assert!(modes.iter().all(|m| m.builtin));
    }

    #[test]
    fn test_create_custom_mode() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let m = custom_mode("custom-1");
        let created = repo.create(&m).unwrap();
        assert_eq!(created.id, "custom-1");
        assert!(!created.builtin);
        assert_eq!(repo.list().unwrap().len(), 11);
    }

    #[test]
    fn test_create_forces_builtin_false() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let mut m = custom_mode("custom-2");
        m.builtin = true; // caller tries to set builtin; should be ignored
        let created = repo.create(&m).unwrap();
        assert!(!created.builtin);
    }

    #[test]
    fn test_get_not_found() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let err = repo.get("nonexistent").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_update_custom_mode() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        repo.create(&custom_mode("custom-upd")).unwrap();
        let mut updated = repo.get("custom-upd").unwrap();
        updated.name = "Updated Name".to_string();
        let result = repo.update(&updated).unwrap();
        assert_eq!(result.name, "Updated Name");
    }

    #[test]
    fn test_update_builtin_mode_blocked() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let mut smart = repo.get("smart").unwrap();
        smart.name = "Hacked".to_string();
        let err = repo.update(&smart).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_delete_builtin_blocked() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        let err = repo.delete("smart").unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_delete_custom_succeeds() {
        let conn = setup();
        let repo = ModesRepository::new(&conn);
        repo.create(&custom_mode("custom-del")).unwrap();
        repo.delete("custom-del").unwrap();
        assert_eq!(repo.list().unwrap().len(), 10);
    }
}
