# Phase 9 Handoff: Text Insertion and Shortcut Rebinding

## Phase goal

Enable transtypro to insert transcribed text into the previously focused application via clipboard paste simulation, and allow the user to change the global dictation shortcut at runtime without restarting the app.

## Acceptance criteria met

- [x] Insert button on Dictation page sends text to the active application via Ctrl+V / Cmd+V simulation
- [x] Paste failure leaves text in clipboard and shows a fallback message
- [x] Clipboard is optionally restored after insertion when the setting is enabled
- [x] History entry is marked as inserted (`was_inserted = true`) when a note was saved this session
- [x] `mark_history_inserted` is only called if a note was saved (entryId is non-null); Insert works independently of note-saving
- [x] Settings page shows an editable shortcut input with an Apply button
- [x] `update_shortcut` registers new shortcut before unregistering old (register-first strategy)
- [x] Invalid shortcut strings return a validation error; old shortcut remains active
- [x] Shortcut is persisted to the database and survives app restart
- [x] No dictated text, clipboard contents, or private data is logged

## Files changed

### New files

| File | Purpose |
|------|---------|
| `src-tauri/src/services/insertion.rs` | InsertionService: clipboard write + paste simulation |
| `src-tauri/src/commands/insertion.rs` | `insert_text` and `mark_history_inserted` Tauri commands |
| `src-tauri/src/commands/shortcut.rs` | `update_shortcut` Tauri command with validation |

### Modified files

| File | Change summary |
|------|---------------|
| `src-tauri/Cargo.toml` | Added `arboard = "3"` and `enigo = "0.2"` |
| `src-tauri/src/db/migrations.rs` | Added migration 004: `ALTER TABLE settings ADD COLUMN shortcut` |
| `src-tauri/src/models/mod.rs` | Added `shortcut: String` to `AppSettings`; added `InsertionResult` struct |
| `src-tauri/src/errors/mod.rs` | Added `InsertionError(String)` variant |
| `src-tauri/src/db/repositories/settings_repo.rs` | Updated SELECT/INSERT/UPDATE to include shortcut (column index 8) |
| `src-tauri/src/db/repositories/history_repo.rs` | Added `mark_inserted` method |
| `src-tauri/src/services/history.rs` | Added `mark_inserted` service method |
| `src-tauri/src/services/mod.rs` | Added `pub mod insertion` and `pub use insertion::InsertionService` |
| `src-tauri/src/commands/mod.rs` | Added `pub mod insertion` and `pub mod shortcut` only |
| `src-tauri/src/lib.rs` | Reads shortcut from DB at startup; registers 3 new commands |
| `src-tauri/src/services/privacy.rs` | Minimal forced touch: added `shortcut` field to test struct literal |
| `src/lib/types.ts` | Added `shortcut: string` to `AppSettings`; added `InsertionResult` interface |
| `src/lib/api.ts` | Added `insertText`, `markHistoryInserted`, `updateShortcut` wrappers |
| `src/pages/Dictation.tsx` | Enabled Insert button with full `handleInsert` implementation |
| `src/pages/Settings.tsx` | Replaced static shortcut display with editable input + Apply button |

## Implementation notes

### Insertion service

`InsertionService::insert_text`:
1. Reads `clipboard_restore_enabled` from settings
2. Saves current clipboard contents if restore is enabled (non-fatal on failure)
3. Writes new text to clipboard via `arboard`; returns `success=false` if write fails
4. Calls `simulate_paste()`: presses Meta+v (macOS) or Ctrl+v (other platforms) via `enigo`
5. Sleeps 150ms to allow the target app to process the paste event
6. Optionally restores saved clipboard contents (non-fatal)

### Paste simulation

`simulate_paste()` is a free function in `services/insertion.rs`:
- Uses `#[cfg(target_os = "macos")]` to select Meta vs Control modifier
- Always releases the modifier key even if the 'v' click fails
- Returns `bool` (true = paste simulated, false = failed)

### Insert command design

`insert_text` is a **sync** Tauri command (`pub fn`, not `async fn`) because it contains `std::thread::sleep` calls that would block async executors.

Window minimize/restore sequence:
1. Minimize main window → target app regains focus
2. Sleep 300ms → OS processes window focus switch
3. Clipboard write + paste simulation
4. Unminimize + show + set_focus → transtypro comes back

### Shortcut registration strategy (register-first)

`update_shortcut`:
1. Validate (non-empty, ≤100 chars, parseable by tauri-plugin-global-shortcut)
2. Early-exit if shortcut unchanged
3. Register new shortcut with full dictation handler via `on_shortcut()` — if this fails, return error; old shortcut still active
4. Unregister old shortcut (non-fatal warning if it fails)
5. Persist new shortcut to DB

`on_shortcut()` is used (not bare `register()`) because the shortcut plugin requires per-shortcut handlers to fire correctly in Tauri v2 when no global plugin handler is configured.

### Startup shortcut registration

In `lib.rs`, the shortcut is read from the raw `rusqlite::Connection` before it is wrapped in `Arc<Mutex<Connection>>` and passed to `AppState`. This avoids the need to lock the state during setup.

## Known limitations

### DiagnosticsService shortcut check (non-critical)

`services/diagnostics.rs` (check #11, "shortcut_configured") hardcodes the string `"CommandOrControl+Shift+Space"` as the expected shortcut. If the user rebinds the shortcut via the Settings page, the diagnostics check will still show "pass" for the old default string rather than validating the current value.

This is a pre-existing limitation introduced in Phase 8. Fixing it requires reading the shortcut from the database inside `DiagnosticsService::run()`. Recommended fix in Phase 10: pass the current shortcut string into the check, or have the check read from the settings repository.

### privacy.rs forced minimal touch

Adding `shortcut: String` to `AppSettings` made all struct literal initializers invalid across the codebase. `services/privacy.rs` is a generally forbidden file under Phase 9 scope, but it contains a test at line ~253 that constructs `AppSettings` without the new field.

The only change made to `privacy.rs` was adding:
```rust
shortcut: "CommandOrControl+Shift+Space".to_string(),
```
to the struct literal in the test. No logic was changed.

### Fn-only shortcuts unsupported

Fn-key-only shortcuts (e.g., F5, F12) are handled at the hardware/firmware level before the OS sees them, and are not supported by the global shortcut plugin. This is documented in the Settings UI.

## Privacy confirmation

- Dictated text is never logged at any point in the insertion pipeline
- Clipboard contents (both saved and written) are held only in memory for the duration of the `insert_text` call
- No insertion data is sent to any external service
- The `InsertionResult` returned to the frontend contains only a boolean success flag, method string, and human-readable message

## New Tauri commands registered

| Command | Sync/Async | Returns |
|---------|-----------|---------|
| `insert_text` | Sync | `InsertionResult` |
| `mark_history_inserted` | Sync | `()` |
| `update_shortcut` | Sync | `String` (accepted shortcut) |

## Test coverage added

| Location | Tests added |
|----------|-------------|
| `db/migrations.rs` | `test_migration_004_adds_shortcut_column`, `test_migration_004_default_value`, `test_migration_004_idempotent` |
| `db/repositories/settings_repo.rs` | `test_get_returns_default_shortcut`, `test_upsert_persists_custom_shortcut`, `test_upsert_preserves_whisper_paths_when_updating_shortcut` |
| `db/repositories/history_repo.rs` | `test_mark_inserted_sets_flag`, `test_mark_inserted_not_found` |
| `services/insertion.rs` | `test_insertion_result_success_serializes`, `test_insertion_result_failure_serializes` |
| `commands/shortcut.rs` | `test_shortcut_validation_empty`, `test_shortcut_validation_too_long`, `test_shortcut_validation_accepts_default`, `test_shortcut_validation_trims_whitespace`, `test_shortcut_validation_max_length_passes` |

Total: 132 tests passing (15 new tests added in Phase 9).

## QA checklist (manual)

- [ ] Launch app, open Dictation page — Insert button is disabled before transcription
- [ ] Record audio, transcribe — Insert button becomes active
- [ ] Click Insert — transtypro minimizes, text appears in focused text field, window restores
- [ ] If paste fails (no focusable app) — error message shown, text still in clipboard
- [ ] Click "Save as note" then "Insert" — history entry marked as inserted
- [ ] Click "Insert" without saving note — works, no error about markHistoryInserted
- [ ] Open Settings — shortcut input shows current value
- [ ] Type new shortcut (e.g., `CommandOrControl+Shift+D`), click Apply — success message shown
- [ ] Press new shortcut — dictation overlay opens
- [ ] Old shortcut no longer triggers overlay
- [ ] Restart app — new shortcut still registered
- [ ] Type invalid shortcut (e.g., `NotAKey`), click Apply — error message shown, old shortcut still works
- [ ] Enable "Restore clipboard" in settings, insert text — previous clipboard contents restored after insertion

## Recommended next tasks (Phase 10)

1. Fix DiagnosticsService check #11 to read shortcut from DB instead of hardcoding the default
2. Add active app context capture (window title, process name) before minimizing
3. Implement the floating overlay auto-pipeline (record → transcribe → insert in one shortcut press)
4. Add insertion method fallback options (direct keyboard simulation instead of clipboard)
