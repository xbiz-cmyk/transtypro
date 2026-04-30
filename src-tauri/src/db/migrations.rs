use crate::errors::AppError;

/// Migration 001 — initial schema.
///
/// All statements use IF NOT EXISTS / INSERT OR IGNORE so the migration is
/// safe to apply to an already-initialised database without corrupting data.
const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    id                        INTEGER PRIMARY KEY CHECK (id = 1),
    active_mode               TEXT    NOT NULL DEFAULT 'smart',
    local_only_mode           INTEGER NOT NULL DEFAULT 0,
    theme                     TEXT    NOT NULL DEFAULT 'dark',
    retention_days            INTEGER NOT NULL DEFAULT 30,
    audio_history_enabled     INTEGER NOT NULL DEFAULT 0,
    clipboard_restore_enabled INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO settings (
    id, active_mode, local_only_mode, theme,
    retention_days, audio_history_enabled, clipboard_restore_enabled
) VALUES (1, 'smart', 0, 'dark', 30, 0, 0);

CREATE TABLE IF NOT EXISTS modes (
    id            TEXT PRIMARY KEY,
    name          TEXT    NOT NULL,
    description   TEXT    NOT NULL DEFAULT '',
    system_prompt TEXT    NOT NULL DEFAULT '',
    active        INTEGER NOT NULL DEFAULT 0,
    builtin       INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO modes (id, name, description, system_prompt, active, builtin) VALUES
    ('smart',     'Smart Mode',     'Automatically adapts to context.',       '', 1, 1),
    ('raw',       'Raw Mode',       'Verbatim transcription, no cleanup.',    '', 0, 1),
    ('clean',     'Clean Mode',     'Fix grammar and punctuation.',           '', 0, 1),
    ('email',     'Email Mode',     'Format for professional email.',         '', 0, 1),
    ('chat',      'Chat Mode',      'Casual conversational tone.',            '', 0, 1),
    ('notes',     'Notes Mode',     'Structured notes format.',               '', 0, 1),
    ('developer', 'Developer Mode', 'Preserve code, flags, and acronyms.',   '', 0, 1),
    ('terminal',  'Terminal Mode',  'Output shell commands and scripts.',     '', 0, 1),
    ('translate', 'Translate Mode', 'Translate speech to another language.',  '', 0, 1),
    ('prompt',    'Prompt Mode',    'Format as AI prompt.',                   '', 0, 1);

CREATE TABLE IF NOT EXISTS vocabulary (
    id          TEXT PRIMARY KEY,
    term        TEXT    NOT NULL,
    replacement TEXT    NOT NULL,
    category    TEXT    NOT NULL DEFAULT '',
    enabled     INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS history (
    id           TEXT PRIMARY KEY,
    raw_text     TEXT    NOT NULL,
    cleaned_text TEXT    NOT NULL DEFAULT '',
    mode_used    TEXT    NOT NULL DEFAULT '',
    timestamp    TEXT    NOT NULL,
    was_inserted INTEGER NOT NULL DEFAULT 0
);
"#;

/// Migration 002 — add whisper binary and model path to settings.
///
/// The migration runner in `run_migrations` tracks applied versions in
/// `schema_migrations`, so each migration executes exactly once even if
/// `run_migrations` is called on every startup.
const MIGRATION_002: &str = r#"
ALTER TABLE settings ADD COLUMN whisper_binary_path TEXT DEFAULT NULL;
ALTER TABLE settings ADD COLUMN whisper_model_path TEXT DEFAULT NULL;
"#;

/// Ordered migration list: (version, sql).
const MIGRATIONS: &[(i64, &str)] = &[(1, MIGRATION_001), (2, MIGRATION_002)];

/// Run all pending migrations against the given connection.
///
/// Creates `schema_migrations` if absent, then applies each migration that
/// has not yet been recorded.  Safe to call on every startup.
pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| AppError::StorageError(format!("creating schema_migrations: {e}")))?;

    for (version, sql) in MIGRATIONS {
        let already_applied: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                rusqlite::params![version],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !already_applied {
            conn.execute_batch(sql)
                .map_err(|e| AppError::StorageError(format!("migration {version}: {e}")))?;

            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) \
                 VALUES (?1, datetime('now'))",
                rusqlite::params![version],
            )
            .map_err(|e| AppError::StorageError(format!("recording migration {version}: {e}")))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn migrated() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_migrations_run_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
    }

    #[test]
    fn test_migration_002_adds_whisper_columns() {
        let conn = migrated();
        conn.execute_batch("SELECT whisper_binary_path, whisper_model_path FROM settings LIMIT 1")
            .expect("migration 002 must have added whisper_binary_path and whisper_model_path");
    }

    #[test]
    fn test_run_migrations_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running a second time must not fail (columns already exist but
        // the migration runner skips already-applied versions).
        run_migrations(&conn).unwrap();
    }
}
