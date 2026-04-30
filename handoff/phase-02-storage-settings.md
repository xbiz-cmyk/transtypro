# Phase 02 Handoff — Storage and Settings (Wave 2)

## Branch

`phase/02-storage-settings`

---

## Summary

Phase 2 storage wires real SQLite persistence to all service stubs that previously returned
`AppError::FeatureNotImplemented`.  Adds a bundled SQLite database, idempotent migrations,
four domain repositories with unit tests, and upgrades the privacy service to derive its
state from persisted settings.

The dictation pipeline (Phase 6) will call `HistoryService::create_history_entry` internally.
No new Tauri command is exposed for history creation yet.

---

## Database path strategy

The database is opened in the OS app data directory resolved by Tauri's `PathResolver`:

| OS | Path |
|---|---|
| Windows | `%APPDATA%\transtypro\transtypro.sqlite` |
| macOS | `~/Library/Application Support/transtypro/transtypro.sqlite` |

The directory is created with `std::fs::create_dir_all` on first run.

---

## Migrations

| Version | Contents |
|---|---|
| 001 | `settings`, `modes`, `vocabulary`, `history` tables; seeds 1 settings row + 10 built-in modes |

Strategy:
- `schema_migrations` table tracks applied versions.
- Each migration is applied once; subsequent runs skip it.
- All SQL uses `CREATE TABLE IF NOT EXISTS` and `INSERT OR IGNORE` — fully idempotent.

---

## Database schema

```sql
schema_migrations (version INTEGER PK, applied_at TEXT)

settings (
    id INTEGER PK CHECK(id=1),
    active_mode TEXT DEFAULT 'smart',
    local_only_mode INTEGER DEFAULT 0,
    theme TEXT DEFAULT 'dark',
    retention_days INTEGER DEFAULT 30,
    audio_history_enabled INTEGER DEFAULT 0,
    clipboard_restore_enabled INTEGER DEFAULT 0
)

modes (
    id TEXT PK, name TEXT, description TEXT, system_prompt TEXT,
    active INTEGER DEFAULT 0, builtin INTEGER DEFAULT 0
)
-- Seeded: smart, raw, clean, email, chat, notes, developer, terminal, translate, prompt

vocabulary (
    id TEXT PK, term TEXT, replacement TEXT,
    category TEXT DEFAULT '', enabled INTEGER DEFAULT 1
)

history (
    id TEXT PK, raw_text TEXT, cleaned_text TEXT DEFAULT '',
    mode_used TEXT DEFAULT '', timestamp TEXT, was_inserted INTEGER DEFAULT 0
)
```

---

## Rust app state

`AppState { db: Arc<Mutex<rusqlite::Connection>> }` defined in `src-tauri/src/db/connection.rs`
and managed by Tauri via `.setup()` hook in `lib.rs`.

```
lib.rs setup hook
  → app.path().app_data_dir()
  → create db directory
  → rusqlite::Connection::open(db_path)
  → db::run_migrations(&conn)
  → app.manage(AppState { db: Arc::new(Mutex::new(conn)) })
```

Commands accept `state: tauri::State<'_, AppState>` and construct services per-call
via `ServiceName::new(state.db.clone())`.  The `Arc::clone` is cheap; the `Mutex` is
held only during the repository call.

---

## Files created

| File | Purpose |
|---|---|
| `src-tauri/src/db/connection.rs` | `AppState` struct |
| `src-tauri/src/db/migrations.rs` | migration runner + inline SQL |
| `src-tauri/src/db/repositories/mod.rs` | re-exports |
| `src-tauri/src/db/repositories/settings_repo.rs` | single-row settings get/upsert |
| `src-tauri/src/db/repositories/modes_repo.rs` | modes CRUD with built-in guard |
| `src-tauri/src/db/repositories/vocabulary_repo.rs` | vocabulary CRUD |
| `src-tauri/src/db/repositories/history_repo.rs` | history create/list/get/delete/clear |

## Files modified

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Added `rusqlite` (bundled), `uuid` (v4), `chrono` (serde) |
| `src-tauri/src/db/mod.rs` | Declared submodules; re-exported `AppState`, `run_migrations` |
| `src-tauri/src/lib.rs` | Added `.setup()` hook; import `tauri::Manager` |
| `src-tauri/src/services/settings.rs` | Real repo call; removed `#[derive(Default)]` |
| `src-tauri/src/services/modes.rs` | Real repo call; UUID for new modes |
| `src-tauri/src/services/vocabulary.rs` | Real repo call; UUID for new entries |
| `src-tauri/src/services/history.rs` | Real repo call; added `create_history_entry` |
| `src-tauri/src/services/privacy.rs` | Reads settings; fail-closed privacy enforcement |
| `src-tauri/src/commands/settings.rs` | Added `tauri::State<'_, AppState>` parameter |
| `src-tauri/src/commands/modes.rs` | Added `tauri::State<'_, AppState>` parameter |
| `src-tauri/src/commands/vocabulary.rs` | Added `tauri::State<'_, AppState>` parameter |
| `src-tauri/src/commands/history.rs` | Added `tauri::State<'_, AppState>` parameter |
| `src-tauri/src/commands/privacy.rs` | Added `tauri::State<'_, AppState>` parameter |

---

## Commands now backed by SQLite

| Command | Behaviour |
|---|---|
| `get_settings` | Returns persisted settings row; defaults on first run |
| `update_settings` | Upserts settings row atomically |
| `list_modes` | Returns all mode rows (10 built-in + custom) |
| `get_mode` | Returns one row by id; `NotFound` if absent |
| `create_mode` | Inserts new custom mode with UUID id; `builtin` forced to false |
| `update_mode` | Updates custom mode; `ValidationError` for built-in |
| `delete_mode` | Deletes custom mode; `ValidationError` for built-in |
| `list_vocabulary` | Returns all vocabulary rows ordered by term |
| `add_vocabulary_entry` | Inserts row; always generates UUID id (ignores client id) |
| `update_vocabulary_entry` | Updates row; `NotFound` if absent |
| `delete_vocabulary_entry` | Deletes row; `NotFound` if absent |
| `list_history` | Returns all rows ordered by timestamp DESC |
| `get_history_entry` | Returns one row by id; `NotFound` if absent |
| `delete_history_entry` | Deletes one row; `NotFound` if absent |
| `clear_history` | Deletes all history rows |
| `get_privacy_status` | Derives `PrivacySummary` from live settings row |
| `enforce_privacy_preview` | Checks `local_only_mode`; fails closed on unknown ops |

---

## Commands still placeholders

| Command | Reason |
|---|---|
| `list_providers` | Phase 5 — cloud provider config not implemented |
| `get_provider` | Phase 5 |
| `test_provider_placeholder` | Phase 5 |
| `run_diagnostics_placeholder` | Phase 8 |

---

## Privacy enforcement behaviour

`enforce_privacy_preview` when `local_only_mode = true`:
- **Always blocked**: `cloud_transcription`, `cloud_cleanup`, `provider_test`, `provider_call`, `openai`, `anthropic`, `remote_model`
- **Always allowed**: `local_transcription`, `local_cleanup`, `read_settings`, `write_settings`, `read_history`, `write_history`, `read_vocabulary`, `write_vocabulary`, `read_modes`, `write_modes`
- **Unknown operations**: blocked by default (fail closed) with reason `"unknown operation 'X' blocked by default in local-only mode"`

When `local_only_mode = false`, all operations are allowed.

---

## Tests added

36 unit tests, all in `#[cfg(test)]` modules using in-memory SQLite:

| Module | Tests |
|---|---|
| `settings_repo` | default values, upsert persistence, idempotent upsert |
| `modes_repo` | 10 built-ins on fresh DB, create custom, force builtin=false, get not found, update custom, update built-in blocked, delete built-in blocked, delete custom |
| `vocabulary_repo` | empty on fresh DB, add+list, update, update not found, delete, delete not found |
| `history_repo` | empty on fresh DB, create+list, get by id, get not found, delete, delete not found, clear |
| `privacy service` | status cloud_allowed default, status blocked in local mode, audio retention 0 when disabled, audio retention uses settings, enforce allows cloud when off, enforce blocks cloud_transcription, blocks openai, blocks provider_test, allows local_transcription, blocks unknown op in local mode, allows unknown op when not local, history retention from settings |

---

## Checks run

| Check | Result |
|---|---|
| `cargo fmt` | ✅ pass |
| `cargo fmt --check` | ✅ pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ pass — 0 warnings |
| `cargo test` | ✅ pass — 36 passed, 0 failed |
| `npm run build` | ✅ pass |
| `npm run lint` | ✅ pass |
| `pwsh scripts/quality-check.ps1` | ✅ all 6 checks passed |

---

## Privacy impact

- All data is stored locally in the OS app data directory.
- No cloud calls of any kind in this phase.
- No API keys are stored, logged, or handled.
- `local_only_mode` blocks cloud operations at the service level when enabled.
- `enforce_privacy_preview` fails closed: unknown operations are blocked in local-only mode.
- History entries are only created by the dictation pipeline (Phase 6) — no direct UI path yet.

---

## Known limitations

- `list_history` returns an empty list until Phase 6 wires `HistoryService::create_history_entry`.
- `audio_retention_days` policy is stored but not enforced (retention sweep is Phase 8).
- `retention_days` for history is stored but no automatic purge runs yet (Phase 8).
- Timestamps use `chrono::Utc::now().to_rfc3339()` — correct ISO 8601 format.
- `HistoryEntry.timestamp` in history table is a TEXT string; sorting is lexicographic (works correctly for ISO 8601).
- Built-in mode `system_prompt` fields are empty strings — real prompts are designed in Phase 5.
- No `add_history_entry` Tauri command is exposed; the dictation pipeline calls `HistoryService::create_history_entry` internally.

---

## What the next phase should do

**Wave 3 — Audio/STT agent (`phase/03-audio-recording`)**:
- List available microphones
- Start/stop recording to a temporary WAV file
- Input level meter
- Temporary file cleanup according to `audio_history_enabled` and `retention_days` settings
- Call `SettingsService::get_settings()` to read the configured microphone and retention policy

**Wave 3 — Providers agent (`phase/05-cleanup-providers`)**:
- Implement `ProvidersService` with real SQLite storage for provider configs
- Add a `providers` table in a new migration (migration 002)
- Implement masked API key handling (OS keychain or encrypted column)
- Wire `enforce_privacy_preview("cloud_cleanup")` before any provider call
