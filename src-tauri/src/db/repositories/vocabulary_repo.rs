use crate::errors::AppError;
use crate::models::VocabularyEntry;
use rusqlite::{params, Connection};

pub struct VocabularyRepository<'a> {
    conn: &'a Connection,
}

impl<'a> VocabularyRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyEntry> {
        Ok(VocabularyEntry {
            id: row.get(0)?,
            term: row.get(1)?,
            replacement: row.get(2)?,
            category: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
        })
    }

    /// Returns all vocabulary entries ordered alphabetically by term.
    pub fn list(&self) -> Result<Vec<VocabularyEntry>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, term, replacement, category, enabled
                 FROM vocabulary ORDER BY term ASC",
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let entries = stmt
            .query_map([], Self::map_row)
            .map_err(|e| AppError::StorageError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(entries)
    }

    /// Returns a single entry by id, or `NotFound`.
    pub fn get(&self, id: &str) -> Result<VocabularyEntry, AppError> {
        self.conn
            .query_row(
                "SELECT id, term, replacement, category, enabled
                 FROM vocabulary WHERE id = ?1",
                params![id],
                Self::map_row,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::NotFound(format!("vocabulary entry '{id}'"))
                }
                other => AppError::StorageError(other.to_string()),
            })
    }

    /// Inserts a new vocabulary entry.
    pub fn create(&self, entry: &VocabularyEntry) -> Result<VocabularyEntry, AppError> {
        self.conn
            .execute(
                "INSERT INTO vocabulary (id, term, replacement, category, enabled)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    entry.id,
                    entry.term,
                    entry.replacement,
                    entry.category,
                    entry.enabled as i64,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        self.get(&entry.id)
    }

    /// Updates an existing vocabulary entry.  Returns `NotFound` if the id is absent.
    pub fn update(&self, entry: &VocabularyEntry) -> Result<VocabularyEntry, AppError> {
        let rows = self
            .conn
            .execute(
                "UPDATE vocabulary
                 SET term = ?1, replacement = ?2, category = ?3, enabled = ?4
                 WHERE id = ?5",
                params![
                    entry.term,
                    entry.replacement,
                    entry.category,
                    entry.enabled as i64,
                    entry.id,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "vocabulary entry '{}'",
                entry.id
            )));
        }
        self.get(&entry.id)
    }

    /// Deletes an entry by id.  Returns `NotFound` if the id is absent.
    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let rows = self
            .conn
            .execute("DELETE FROM vocabulary WHERE id = ?1", params![id])
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!("vocabulary entry '{id}'")));
        }
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

    fn entry(id: &str, term: &str) -> VocabularyEntry {
        VocabularyEntry {
            id: id.to_string(),
            term: term.to_string(),
            replacement: format!("{term}_replaced"),
            category: "test".to_string(),
            enabled: true,
        }
    }

    #[test]
    fn test_list_empty_on_fresh_db() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_list() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        repo.create(&entry("v1", "hello")).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].term, "hello");
    }

    #[test]
    fn test_update_entry() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        repo.create(&entry("v2", "alpha")).unwrap();
        let mut e = repo.get("v2").unwrap();
        e.term = "beta".to_string();
        let updated = repo.update(&e).unwrap();
        assert_eq!(updated.term, "beta");
    }

    #[test]
    fn test_update_not_found() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        let e = entry("missing", "x");
        let err = repo.update(&e).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_delete_entry() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        repo.create(&entry("v3", "foo")).unwrap();
        repo.delete("v3").unwrap();
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn test_delete_not_found() {
        let conn = setup();
        let repo = VocabularyRepository::new(&conn);
        let err = repo.delete("ghost").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
