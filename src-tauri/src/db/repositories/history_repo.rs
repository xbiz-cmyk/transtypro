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

    /// Marks a history entry as having been inserted into an external application.
    ///
    /// Returns `NotFound` if no row with the given id exists.
    pub fn mark_inserted(&self, id: &str) -> Result<(), AppError> {
        let updated = self
            .conn
            .execute(
                "UPDATE history SET was_inserted = 1 WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        if updated == 0 {
            return Err(AppError::NotFound(format!("history entry '{id}'")));
        }
        Ok(())
    }

    /// Deletes history entries older than `days` days using the `timestamp` column.
    ///
    /// Returns the number of rows deleted.
    /// If `days == 0` returns `Ok(0)` immediately — zero means keep forever.
    pub fn delete_older_than(&self, days: u32) -> Result<u32, AppError> {
        if days == 0 {
            return Ok(0);
        }
        let offset = format!("-{days} days");
        let rows = self
            .conn
            .execute(
                "DELETE FROM history WHERE datetime(timestamp) < datetime('now', ?1)",
                rusqlite::params![offset],
            )
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(rows as u32)
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

    #[test]
    fn test_mark_inserted_sets_flag() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        repo.create(&entry("10")).unwrap();
        assert!(!repo.get("10").unwrap().was_inserted);
        repo.mark_inserted("10").unwrap();
        assert!(
            repo.get("10").unwrap().was_inserted,
            "was_inserted must be true after mark_inserted"
        );
    }

    #[test]
    fn test_mark_inserted_not_found() {
        let conn = setup();
        let err = HistoryRepository::new(&conn)
            .mark_inserted("ghost-id")
            .unwrap_err();
        assert!(
            matches!(err, AppError::NotFound(_)),
            "unknown id must return NotFound"
        );
    }

    #[test]
    fn test_delete_older_than_zero_keeps_all() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        // Insert an entry with a very old timestamp.
        let mut e = entry("07");
        e.timestamp = "2000-01-01T00:00:00Z".to_string();
        repo.create(&e).unwrap();
        // days = 0 means keep forever → must delete nothing.
        let deleted = repo.delete_older_than(0).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn test_delete_older_than_removes_old_keeps_recent() {
        let conn = setup();
        let repo = HistoryRepository::new(&conn);
        // Old entry: year 2000 — definitely older than any sensible retention window.
        let mut old = entry("08");
        old.timestamp = "2000-06-15T12:00:00Z".to_string();
        repo.create(&old).unwrap();
        // Recent entry: far future — will never be deleted.
        let mut recent = entry("09");
        recent.timestamp = "2099-12-31T23:59:59Z".to_string();
        repo.create(&recent).unwrap();
        // Delete entries older than 30 days.
        let deleted = repo.delete_older_than(30).unwrap();
        assert_eq!(deleted, 1, "only the old entry should be deleted");
        let remaining = repo.list().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "09", "recent entry must survive");
    }
}
