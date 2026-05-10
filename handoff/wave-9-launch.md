# Wave 9 Launch: Text Insertion + Custom Shortcut Rebinding

**Wave:** 9
**Phase:** 09 — Text Insertion + Custom Shortcut Rebinding
**Branch:** `phase/09-text-insertion`
**Worktree:** `C:\Users\User\Desktop\transtypro-insertion`
**Base commit:** `be49369` — feat(privacy): add diagnostics and retention cleanup
**Launched:** 2026-05-09
**Agent:** transtypro-rust-backend + transtypro-frontend-ui

---

## Goal

Enable the Insert button in the Dictation page to actually insert transcribed text into the previously focused application via clipboard paste. Add custom shortcut rebinding UI and backend so the user can change `CommandOrControl+Shift+Space` to any key combination.

---

## New Cargo Dependencies

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
arboard = "3"
enigo = "0.2"
```

- `arboard`: Cross-platform clipboard read/write (backup, write text, restore).
- `enigo`: Cross-platform keyboard simulation (Ctrl+V / Cmd+V paste).

---

## Database Migration 004

File: `src-tauri/src/db/migrations.rs`

Add a fourth migration entry to the migrations list:

```rust
(
    "004",
    "ALTER TABLE settings ADD COLUMN shortcut TEXT NOT NULL DEFAULT 'CommandOrControl+Shift+Space'",
),
```

This is an `ALTER TABLE` statement. SQLite supports this safely. The default fills in for all existing rows.

---

## Model Changes

File: `src-tauri/src/models/mod.rs`

### 1. Add `shortcut` field to `AppSettings`

```rust
pub struct AppSettings {
    pub active_mode: String,
    pub local_only_mode: bool,
    pub theme: String,
    pub retention_days: u32,
    pub audio_history_enabled: bool,
    pub clipboard_restore_enabled: bool,
    pub whisper_binary_path: Option<String>,
    pub whisper_model_path: Option<String>,
    pub shortcut: String,   // NEW
}
```

Default (when constructing manually): `"CommandOrControl+Shift+Space".to_string()`

### 2. Add `InsertionResult`

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InsertionResult {
    pub success: bool,
    pub method: String,
    pub message: String,
}
```

Also add `InsertionResult` to the `pub use` block in `models/mod.rs`.

---

## Error Changes

File: `src-tauri/src/errors/mod.rs`

Add one new variant:

```rust
#[error("insertion error: {0}")]
InsertionError(String),
```

---

## New Service: InsertionService

File: `src-tauri/src/services/insertion.rs`

### Design

1. Save the current clipboard contents (`arboard::Clipboard::get_text().ok()` → `Option<String>`).
2. Write the dictated text to the clipboard (`arboard::Clipboard::set_text(text)`).
3. Simulate Ctrl+V (Windows/Linux) or Cmd+V (macOS) using `enigo`.
4. If `clipboard_restore_enabled` is true AND clipboard was saved (`Some`), restore original after a 200ms delay.
5. Return `InsertionResult { success: bool, method: "clipboard_paste".into(), message }`.

### Window focus

The command layer (`commands/insertion.rs`) calls `app_handle.get_webview_window("main").map(|w| w.minimize())` then `std::thread::sleep(Duration::from_millis(300))` before calling `InsertionService::insert_text()`. This gives the previously focused application time to regain focus before the paste is simulated.

### Clipboard safety rules

- NEVER log the clipboard text or the dictated text.
- NEVER send clipboard content anywhere.
- On `get_text()` error (empty clipboard, image content, etc.) → treat as `None`, proceed without restore attempt.
- On `set_text()` failure → return `InsertionResult { success: false, method: "clipboard_paste".into(), message: error.to_string() }`.
- On paste simulation failure → still attempt clipboard restore, then return `InsertionResult { success: false, ... }`.

### Platform key mapping

```rust
#[cfg(target_os = "macos")]
let key = enigo::Key::Meta;
#[cfg(not(target_os = "macos"))]
let key = enigo::Key::Control;
```

Simulate: hold modifier → press V → release V → release modifier.

### Module signature

```rust
pub struct InsertionService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl InsertionService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self { ... }
    pub fn insert_text(&self, text: String) -> Result<InsertionResult, AppError> { ... }
}
```

---

## New Commands

### `commands/insertion.rs`

```rust
#[tauri::command]
pub async fn insert_text(
    text: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<InsertionResult, AppError> {
    // 1. Minimize the transtypro window so previous app regains focus.
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.minimize();
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    // 2. Call service.
    let db = state.db.clone();
    let svc = InsertionService::new(db);
    svc.insert_text(text)
}

#[tauri::command]
pub async fn mark_history_inserted(
    id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    let svc = HistoryService::new(db);
    svc.mark_inserted(id)
}
```

### `commands/shortcut.rs`

```rust
#[tauri::command]
pub async fn update_shortcut(
    shortcut: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, AppError> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // 1. Validate: non-empty, reasonable length.
    let trimmed = shortcut.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::ValidationError("shortcut cannot be empty".into()));
    }
    if trimmed.len() > 100 {
        return Err(AppError::ValidationError("shortcut string too long".into()));
    }

    // 2. Read the current shortcut from DB.
    let old_shortcut = {
        let conn = state.db.lock().map_err(|_| AppError::StorageError("lock poisoned".into()))?;
        crate::db::repositories::SettingsRepository::new(&conn).get()?.shortcut
    };

    // 3. Register-first strategy: register new → if fails, return error (old still active).
    app_handle
        .global_shortcut()
        .register(trimmed.as_str())
        .map_err(|e| AppError::ValidationError(format!("could not register shortcut '{trimmed}': {e}")))?;

    // 4. Unregister old (non-fatal if it fails).
    if let Err(e) = app_handle.global_shortcut().unregister(old_shortcut.as_str()) {
        eprintln!("[shortcut] failed to unregister old shortcut '{old_shortcut}': {e}");
    }

    // 5. Persist new shortcut to DB.
    {
        let conn = state.db.lock().map_err(|_| AppError::StorageError("lock poisoned".into()))?;
        let mut s = crate::db::repositories::SettingsRepository::new(&conn).get()?;
        s.shortcut = trimmed.clone();
        crate::db::repositories::SettingsRepository::new(&conn).upsert(&s)?;
    }

    Ok(trimmed)
}
```

### `commands/mod.rs` — MINIMAL EDIT ONLY

The ONLY allowed change to `commands/mod.rs` is adding two `pub mod` declarations:

```rust
pub mod insertion;
pub mod shortcut;
```

No other changes. Do not restructure or rename anything else in this file.

---

## Services Module

File: `src-tauri/src/services/mod.rs`

Add:

```rust
pub mod insertion;
```

---

## Settings Repository Changes

File: `src-tauri/src/db/repositories/settings_repo.rs`

- Add `shortcut` to the `SELECT` query column list.
- Add `shortcut` to the `INSERT OR REPLACE` statement.
- Map `shortcut` column in the `query_row` closure.
- Default for manual construction: `"CommandOrControl+Shift+Space".to_string()`.

---

## History Repository Changes

File: `src-tauri/src/db/repositories/history_repo.rs`

Add one new method:

```rust
pub fn mark_inserted(&self, id: &str) -> Result<(), AppError> {
    let updated = self.conn.execute(
        "UPDATE history SET was_inserted = 1 WHERE id = ?1",
        rusqlite::params![id],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("history entry '{id}' not found")));
    }
    Ok(())
}
```

---

## History Service Changes

File: `src-tauri/src/services/history.rs`

Add:

```rust
pub fn mark_inserted(&self, id: String) -> Result<(), AppError> {
    let conn = self.db.lock().map_err(|_| AppError::StorageError("lock poisoned".into()))?;
    HistoryRepository::new(&conn).mark_inserted(&id)
}
```

---

## `lib.rs` Changes

File: `src-tauri/src/lib.rs`

### 1. Read shortcut from DB on startup

In the `setup` closure, after getting a DB connection, read `settings.shortcut` and use it instead of the hardcoded `"CommandOrControl+Shift+Space"` string:

```rust
let shortcut_str = {
    let conn = db.lock().expect("db lock poisoned at startup");
    crate::db::repositories::SettingsRepository::new(&conn)
        .get()
        .map(|s| s.shortcut)
        .unwrap_or_else(|_| "CommandOrControl+Shift+Space".to_string())
};
```

Then use `shortcut_str.as_str()` in the `global_shortcut().register(...)` call.

### 2. Register new commands

In the `.invoke_handler(tauri::generate_handler![...])` call, add:

```rust
commands::insertion::insert_text,
commands::insertion::mark_history_inserted,
commands::shortcut::update_shortcut,
```

---

## Frontend Type Changes

File: `src/lib/types.ts`

### 1. Add `shortcut` to `AppSettings`

```ts
export interface AppSettings {
  active_mode: string;
  local_only_mode: boolean;
  theme: string;
  retention_days: number;
  audio_history_enabled: boolean;
  clipboard_restore_enabled: boolean;
  whisper_binary_path: string | null;
  whisper_model_path: string | null;
  shortcut: string;   // NEW
}
```

### 2. Add `InsertionResult`

```ts
export interface InsertionResult {
  success: boolean;
  method: string;
  message: string;
}
```

---

## Frontend API Changes

File: `src/lib/api.ts`

Add three new functions:

```ts
/** Insert text into the previously active application via clipboard paste. */
export async function insertText(text: string): Promise<InsertionResult> {
  return invoke<InsertionResult>("insert_text", { text });
}

/** Mark a history entry as having been inserted into an external app. */
export async function markHistoryInserted(id: string): Promise<void> {
  return invoke<void>("mark_history_inserted", { id });
}

/** Update the global dictation shortcut. Returns the accepted shortcut string. */
export async function updateShortcut(shortcut: string): Promise<string> {
  return invoke<string>("update_shortcut", { shortcut });
}
```

Also add `InsertionResult` to the import from `"./types"`.

---

## Dictation Page Changes

File: `src/pages/Dictation.tsx`

### State additions

```tsx
const [entryId, setEntryId] = useState<string | null>(null);
const [inserting, setInserting] = useState(false);
const [inserted, setInserted] = useState(false);
```

Reset `entryId`, `inserting`, `inserted` whenever a new recording starts.

### After `createHistoryEntry` succeeds

Capture the returned entry ID:

```tsx
const entry = await createHistoryEntry({ rawText, cleanedText, modeUsed: activeMode });
setEntryId(entry.id);
```

### `handleInsert` function

```tsx
async function handleInsert() {
  if (!finalText || !entryId) return;
  setInserting(true);
  try {
    await insertText(finalText);
    await markHistoryInserted(entryId);
    setInserted(true);
  } catch (e) {
    // Show error in existing error state.
    setError(String(e));
  } finally {
    setInserting(false);
  }
}
```

### Insert button

Change from disabled placeholder to an active button:

```tsx
<Button
  variant="primary"
  disabled={!finalText || inserting || inserted}
  onClick={handleInsert}
>
  {inserted ? "Inserted ✓" : inserting ? "Inserting…" : "Insert"}
</Button>
```

The button is disabled when: no text available, currently inserting, or already inserted (one-shot per session).

---

## Settings Page Changes

File: `src/pages/Settings.tsx`

### New state

```tsx
const [shortcut, setShortcut] = useState("CommandOrControl+Shift+Space");
const [shortcutSaving, setShortcutSaving] = useState(false);
const [shortcutMessage, setShortcutMessage] = useState<string | null>(null);
const [shortcutError, setShortcutError] = useState<string | null>(null);
```

### Hydrate from backend

In the `useEffect` that calls `getSettings()`, add:

```tsx
setShortcut(s.shortcut);
```

### Include in save

In `handleSave`, include `shortcut` in the merged object passed to `updateSettings`:

```tsx
await updateSettings({
  ...base,
  active_mode: defaultMode,
  local_only_mode: localOnly,
  theme,
  retention_days: Math.max(0, parseInt(retentionDays, 10) || 0),
  audio_history_enabled: audioHistory,
  clipboard_restore_enabled: clipboardRestore,
  shortcut,   // NEW
});
```

### `handleApplyShortcut`

```tsx
async function handleApplyShortcut() {
  setShortcutSaving(true);
  setShortcutMessage(null);
  setShortcutError(null);
  try {
    const accepted = await updateShortcut(shortcut);
    setShortcut(accepted);
    setShortcutMessage(`Shortcut updated to: ${accepted}`);
  } catch (e) {
    setShortcutError(String(e));
  } finally {
    setShortcutSaving(false);
  }
}
```

### Replace static shortcut display with editable input

In the Dictation card, replace the static `<div id="shortcut-display">` block with:

```tsx
<div className="flex flex-col gap-1">
  <label htmlFor="shortcut-input" className="text-sm font-medium text-(--color-text-secondary)">
    Global shortcut
  </label>
  <div className="flex gap-2">
    <Input
      id="shortcut-input"
      value={shortcut}
      onChange={(e) => setShortcut(e.target.value)}
    />
    <Button
      variant="secondary"
      size="sm"
      disabled={shortcutSaving}
      onClick={handleApplyShortcut}
    >
      {shortcutSaving ? "Applying…" : "Apply shortcut"}
    </Button>
  </div>
  {shortcutMessage && (
    <p className="text-xs text-(--color-status-success)">{shortcutMessage}</p>
  )}
  {shortcutError && (
    <p className="text-xs text-(--color-status-error)">{shortcutError}</p>
  )}
  <p className="text-xs text-(--color-text-muted)">
    Note: Fn-only shortcuts are unsupported — they are handled at the
    hardware/firmware level before the OS sees them.
  </p>
</div>
```

---

## Allowed Files

The agent working in `phase/09-text-insertion` may ONLY edit these files:

**Rust backend:**
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/errors/mod.rs`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/services/insertion.rs` (NEW)
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/commands/insertion.rs` (NEW)
- `src-tauri/src/commands/shortcut.rs` (NEW)
- `src-tauri/src/commands/mod.rs` — MINIMAL EDIT ONLY (`pub mod insertion;` and `pub mod shortcut;`)
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/settings_repo.rs`
- `src-tauri/src/db/repositories/history_repo.rs`
- `src-tauri/src/services/history.rs`
- `src-tauri/src/lib.rs`

**Frontend:**
- `src/lib/api.ts`
- `src/lib/types.ts`
- `src/pages/Dictation.tsx`
- `src/pages/Settings.tsx`

**Docs:**
- `handoff/phase-09-text-insertion.md`
- `docs/PROGRESS.md`
- `docs/TASK_BOARD.md`

---

## Forbidden Files

Do NOT touch:

- Any audio, transcription, cleanup, privacy, or diagnostics service/command files.
- `src-tauri/src/services/retention.rs`
- `src-tauri/src/commands/history.rs` (no changes — separate `mark_history_inserted` goes in `commands/insertion.rs`)
- Any modes, vocabulary, or providers repositories.
- `src-tauri/src/db/connection.rs`
- `src/stores/uiStore.ts`
- `src/components/ShortcutHandler.tsx`
- `src/components/FloatingOverlay.tsx`
- `src/App.tsx`
- `src/pages/History.tsx`
- `src/pages/Privacy.tsx`
- `src/pages/Diagnostics.tsx`
- `src/pages/Providers.tsx`
- `src/pages/Models.tsx`
- `src/pages/Home.tsx`
- `docs/PHASES.md`
- `docs/PARALLEL_EXECUTION_PLAN.md`
- `docs/ARCHITECTURE.md`
- `AGENTS.md`, `CLAUDE.md`, `SOUL.md`

---

## Testing Targets

**All 117 existing Rust tests must continue to pass.**

New tests to add (target: 127+ total, minimum 10 new):

| Test | File | What |
|------|------|------|
| `test_mark_inserted_sets_flag` | `history_repo.rs` | `mark_inserted` sets `was_inserted=1` |
| `test_mark_inserted_not_found` | `history_repo.rs` | `mark_inserted` returns `NotFound` for unknown ID |
| `test_migration_004_adds_shortcut_column` | `migrations.rs` | Column exists after migration |
| `test_migration_004_default_value` | `migrations.rs` | Default is `'CommandOrControl+Shift+Space'` |
| `test_settings_repo_shortcut_round_trip` | `settings_repo.rs` | Save + reload `shortcut` field |
| `test_settings_repo_shortcut_default` | `settings_repo.rs` | Fresh DB returns default shortcut |
| `test_shortcut_validation_empty` | `commands/shortcut.rs` | Empty string → `ValidationError` |
| `test_shortcut_validation_too_long` | `commands/shortcut.rs` | 101-char string → `ValidationError` |
| `test_insertion_result_serde_success` | `models/mod.rs` | `InsertionResult` serializes correctly |
| `test_insertion_result_serde_failure` | `models/mod.rs` | `InsertionResult { success: false }` serializes correctly |

Frontend: TypeScript build (`npm run build`) must pass with no type errors. The `shortcut` field must be present in `AppSettings` and `InsertionResult` must be importable from `types.ts`.

---

## Merge Gate

**No PR may be merged unless the orchestrator provides the exact phrase:**

```
ORCHESTRATOR APPROVED MERGE
```

---

## Known Limitations

- Text insertion works via clipboard paste — if the target app's focused field does not accept `Ctrl+V` / `Cmd+V`, insertion will silently fail. Phase 11 will add active-app detection and accessibility API fallbacks.
- The 300ms minimize delay may not be enough on very slow systems. Future phases may expose this as a configurable setting.
- Fn-only shortcuts (`F1`–`F24` alone) are unsupported — the OS does not expose them to user-space applications.
- Shortcut validation is string-length only. Invalid key names are caught at registration time and returned as a `ValidationError`.
- Global shortcut conflicts with other apps (e.g., browser extensions using `Ctrl+Shift+Space`) will cause registration failure. The error message will surface in the Settings UI.

---

## Next Phase

Phase 10: Active App Detection
- Detect which application was focused before the shortcut.
- Use platform accessibility APIs (Windows: `GetForegroundWindow`/`UIAutomation`, macOS: `NSWorkspace.shared.frontmostApplication`).
- Use detected app context to auto-select mode (e.g., terminal → Developer mode).
- Replace the 300ms minimize hack with proper focus handoff.
