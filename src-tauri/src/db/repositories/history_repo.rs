use crate::errors::AppError;
use crate::models::HistoryEntry;
use rusqlite::{params, Connection};

pub struct HistoryRepository<'a> {
    conn: &'a Connection,
}

impl<'a> HistoryRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
        Ok(HistoryEntry {
            id: row.get(0)?,
            raw_text: row.get(1)?,
            cleaned_text: row.get(2)?,
            mode_used: row.get(3)?,
            timestamp: row.get(4)?,
            was_inserted: row.get::<_, i64>(5)? != 0,
        })
    }

    /// Inserts a new history record.
    pub fn create(&self, entry: &HistoryEntry) -> Result<HistoryEntry, AppError> {
        self.conn
            .execute(
                "INSERT INTO history (id, raw_text, cleaned_text, mode_used, timestamp, was_inserted)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    entry.id,
                    entry.raw_text,
                    entry.cleaned_text,
                    entry.mode_used,
                    entry.timestamp,
                    entry.was_inserted as i64,
                ],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        self.get(&entry.id)
    }

    /// Returns all history entries ordered newest-first.
    pub fn list(&self) -> Result<Vec<HistoryEntry>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, raw_text, cleaned_text, mode_used, timestamp, was_inserted
                 FROM history ORDER BY timestamp DESC",
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let entries = stmt
            .query_map([], Self::map_row)
            .map_err(|e| AppError::StorageError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(entries)
    }

    /// Returns a single history entry by id, or `NotFound`.
    pub fn get(&self, id: &str) -> Result<HistoryEntry, AppError> {
        self.conn
            .query_row(
                "SELECT id, raw_text, cleaned_text, mode_used, timestamp, was_inserted
                 FROM history WHERE id = ?1",
                params![id],
                Self::map_row,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::NotFound(format!("history entry '{id}'"))
                }
                other => AppError::StorageError(other.to_string()),
            })
    }

    /// Deletes a single history entry by id.  Returns `NotFound` if absent.
    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let rows = self
            .conn
            .execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!("history entry '{id}'")));
        }
        Ok(())
    }

    /// Deletes all history entries.
    pub fn clear(&self) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM history", [])
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

    fn entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            raw_text: format!("raw text for {id}"),
            cleaned_text: format!("cleaned text for {id}"),
            mode_used: "smart".to_string(),
            timestamp: format!("2026-04-{id}T00:00:00Z"),
            was_inserted: false,
        }
    }

    #[test]
    fn test_list_empty_on_fresh_db() {
        let conn = setup();
        assert!(HistoryRepository::new(&conn).list().unwrap().is_empty());
    }

    #[test]
    fn test_create_and_list() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        repo.create(&entry("01")).unwrap();
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].raw_text, "raw text for 01");
    }

    #[test]
    fn test_get_by_id() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        repo.create(&entry("02")).unwrap();
        let e = repo.get("02").unwrap();
        assert_eq!(e.id, "02");
        assert_eq!(e.mode_used, "smart");
    }

    #[test]
    fn test_get_not_found() {
        let conn = setup();
        let err = HistoryRepository::new(&conn).get("ghost").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_delete_entry() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        repo.create(&entry("03")).unwrap();
        repo.delete("03").unwrap();
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn test_delete_not_found() {
        let conn = setup();
        let err = HistoryRepository::new(&conn).delete("ghost").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_clear_history() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        repo.create(&entry("04")).unwrap();
        repo.create(&entry("05")).unwrap();
        repo.create(&entry("06")).unwrap();
        repo.clear().unwrap();
        assert!(repo.list().unwrap().is_empty());
    }
}
