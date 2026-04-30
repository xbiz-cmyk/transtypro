# Phase 02 Handoff — Backend Contracts Agent

## Branch

`phase/02-backend-contracts`

---

## Summary

Phase 2 (Backend Contracts) defines all Tauri command interfaces, service contract
structs, data models, and typed error variants that the Frontend UI agent and the
Database/Privacy agent depend on.

Every data-dependent service method returns `AppError::FeatureNotImplemented` until the
Database agent wires real SQLite repositories in `phase/02-storage-settings`.

Four operations return safe static defaults rather than errors:
- `PrivacyService::get_privacy_status` — returns static `PrivacySummary` with `local_only_mode: false`
- `PrivacyService::enforce_privacy_preview` — returns `PrivacyDecision { allowed: true, reason: "preview only" }`
- `DiagnosticsService::run_diagnostics_placeholder` — returns a static two-check report
- `ProvidersService::test_provider_placeholder` — returns `Ok("provider test not yet implemented")`

---

## Files Created or Modified

### New files

| File | Description |
|---|---|
| `src-tauri/src/services/settings.rs` | SettingsService struct with get/update stubs |
| `src-tauri/src/services/modes.rs` | ModesService struct with CRUD stubs |
| `src-tauri/src/services/vocabulary.rs` | VocabularyService struct with list/add/delete stubs |
| `src-tauri/src/services/history.rs` | HistoryService struct with list/get/delete/clear stubs |
| `src-tauri/src/services/privacy.rs` | PrivacyService struct with static-default implementations |
| `src-tauri/src/services/providers.rs` | ProvidersService struct with list/get stubs + test placeholder |
| `src-tauri/src/services/diagnostics.rs` | DiagnosticsService struct with static report stub |
| `src-tauri/src/commands/settings.rs` | get_settings, update_settings commands |
| `src-tauri/src/commands/modes.rs` | list_modes, get_mode, create_mode, update_mode, delete_mode commands |
| `src-tauri/src/commands/vocabulary.rs` | list_vocabulary, add_vocabulary_entry, update_vocabulary_entry, delete_vocabulary_entry commands |
| `src-tauri/src/commands/history.rs` | list_history, get_history_entry, delete_history_entry, clear_history commands |
| `src-tauri/src/commands/privacy.rs` | get_privacy_status, enforce_privacy_preview commands |
| `src-tauri/src/commands/providers.rs` | list_providers, get_provider, test_provider_placeholder commands |
| `src-tauri/src/commands/diagnostics.rs` | run_diagnostics_placeholder command |

### Modified files

| File | Change |
|---|---|
| `src-tauri/src/models/mod.rs` | Added 10 new structs (AppSettings, DictationMode, VocabularyEntry, HistoryEntry, AiProvider, DiagnosticCheck, DiagnosticReport, PrivacyOperation, PrivacyDecision, PrivacySummary) |
| `src-tauri/src/errors/mod.rs` | Added 6 new error variants (NotFound, ValidationError, StorageError, PrivacyBlocked, ProviderUnavailable, DiagnosticsError) |
| `src-tauri/src/services/mod.rs` | Added submodule declarations and re-exports; preserved Phase 0 stubs |
| `src-tauri/src/commands/mod.rs` | Added submodule declarations; preserved ping, get_app_version, get_status_summary |
| `src-tauri/src/lib.rs` | Registered all 24 Tauri commands |

---

## Complete List of Registered Tauri Commands

Exact command name strings as registered via `tauri::generate_handler!`:

```
ping
get_app_version
get_status_summary
get_settings
update_settings
list_modes
get_mode
create_mode
update_mode
delete_mode
list_vocabulary
add_vocabulary_entry
update_vocabulary_entry
delete_vocabulary_entry
list_history
get_history_entry
delete_history_entry
clear_history
get_privacy_status
enforce_privacy_preview
list_providers
get_provider
test_provider_placeholder
run_diagnostics_placeholder
```

---

## Complete List of Service Methods Defined

### SettingsService (`services/settings.rs`)
- `get_settings() -> Result<AppSettings, AppError>` — returns FeatureNotImplemented
- `update_settings(settings: AppSettings) -> Result<(), AppError>` — returns FeatureNotImplemented

### ModesService (`services/modes.rs`)
- `list_modes() -> Result<Vec<DictationMode>, AppError>` — returns FeatureNotImplemented
- `get_mode(id: String) -> Result<DictationMode, AppError>` — returns FeatureNotImplemented
- `create_mode(mode: DictationMode) -> Result<DictationMode, AppError>` — returns FeatureNotImplemented
- `update_mode(mode: DictationMode) -> Result<DictationMode, AppError>` — returns FeatureNotImplemented
- `delete_mode(id: String) -> Result<(), AppError>` — returns FeatureNotImplemented

### VocabularyService (`services/vocabulary.rs`)
- `list_vocabulary() -> Result<Vec<VocabularyEntry>, AppError>` — returns FeatureNotImplemented
- `add_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, AppError>` — returns FeatureNotImplemented
- `update_entry(entry: VocabularyEntry) -> Result<VocabularyEntry, AppError>` — returns FeatureNotImplemented
- `delete_entry(id: String) -> Result<(), AppError>` — returns FeatureNotImplemented

### HistoryService (`services/history.rs`)
- `list_history() -> Result<Vec<HistoryEntry>, AppError>` — returns FeatureNotImplemented
- `get_entry(id: String) -> Result<HistoryEntry, AppError>` — returns FeatureNotImplemented
- `delete_entry(id: String) -> Result<(), AppError>` — returns FeatureNotImplemented
- `clear_history() -> Result<(), AppError>` — returns FeatureNotImplemented

### PrivacyService (`services/privacy.rs`)
- `get_privacy_status() -> Result<PrivacySummary, AppError>` — returns static default PrivacySummary
- `enforce_privacy_preview(op: PrivacyOperation) -> Result<PrivacyDecision, AppError>` — returns `{ allowed: true, reason: "preview only" }`

### ProvidersService (`services/providers.rs`)
- `list_providers() -> Result<Vec<AiProvider>, AppError>` — returns FeatureNotImplemented
- `get_provider(id: String) -> Result<AiProvider, AppError>` — returns FeatureNotImplemented
- `test_provider_placeholder(id: String) -> Result<String, AppError>` — returns `Ok("provider test not yet implemented")`

### DiagnosticsService (`services/diagnostics.rs`)
- `run_diagnostics_placeholder() -> Result<DiagnosticReport, AppError>` — returns static report with `backend_alive: pass` and `storage: pending`

---

## Complete List of Models Defined

### Existing (Phase 0)
- `StatusSummary { privacy_mode: String, transcription_ready: bool, cleanup_provider: Option<String>, active_mode: String, history_count: u32 }`

### New (Phase 2)
- `AppSettings { active_mode: String, local_only_mode: bool, theme: String, retention_days: u32, audio_history_enabled: bool, clipboard_restore_enabled: bool }`
- `DictationMode { id: String, name: String, description: String, system_prompt: String, active: bool, builtin: bool }`
- `VocabularyEntry { id: String, term: String, replacement: String, category: String, enabled: bool }`
- `HistoryEntry { id: String, raw_text: String, cleaned_text: String, mode_used: String, timestamp: String, was_inserted: bool }`
- `AiProvider { id: String, name: String, provider_type: String, base_url: String, model: String, enabled: bool, use_for_cleanup: bool, use_for_transcription: bool, api_key_set: bool }`
- `DiagnosticCheck { name: String, status: String, message: String }`
- `DiagnosticReport { checks: Vec<DiagnosticCheck>, generated_at: String }`
- `PrivacySummary { local_only_mode: bool, audio_retention_days: u32, history_retention_days: u32, cloud_allowed: bool, reason: String }`
- `PrivacyOperation { operation_type: String, provider_id: Option<String> }`
- `PrivacyDecision { allowed: bool, reason: String }`

All models derive `Serialize, Deserialize, Debug, Clone`.

---

## Complete List of AppError Variants

### Existing (Phase 0)
- `FeatureNotImplemented(String)`
- `Internal(String)`
- `Io(#[from] std::io::Error)`
- `Serialization(#[from] serde_json::Error)`

### New (Phase 2)
- `NotFound(String)`
- `ValidationError(String)`
- `StorageError(String)`
- `PrivacyBlocked(String)`
- `ProviderUnavailable(String)`
- `DiagnosticsError(String)`

---

## Error handling convention

All new Phase 2 Tauri commands return `Result<T, AppError>` directly.
`AppError` implements `Serialize` (serializes as its `Display` string) so Tauri's IPC layer
can transmit it to the frontend without a `.map_err(|e| e.to_string())` conversion.
The frontend receives errors as plain strings over IPC — no TypeScript changes needed.

## Commands Run and Results

| Command | Result |
|---|---|
| `cargo fmt` | ✅ pass — no formatting issues |
| `cargo fmt --check` | ✅ pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ pass — 0 warnings |
| `cargo test` | ✅ pass — 0 tests (no unit tests added this phase) |
| `npm install` | ✅ pass — 90 packages |
| `npm run build` | ✅ pass — frontend compiles with no errors |
| `pwsh scripts/quality-check.ps1` | ✅ pass — all checks |

---

## Known Limitations

- All data operations return `FeatureNotImplemented` until the Database agent wires SQLite.
- `generated_at` in `DiagnosticReport` is a static string `"2026-04-30T00:00:00Z"` — will be replaced when chrono or similar is added.
- No unit tests in this phase — services have no logic to test.
- `api_key_set` on `AiProvider` is always `false` in all stubs.
- `PrivacyService::enforce_privacy_preview` always returns `allowed: true` — real enforcement is Phase 8.

---

## What the Database Agent Must Do Next

The Database agent (`phase/02-storage-settings`) must:

1. Add `rusqlite` or `sqlx` to `Cargo.toml`.
2. Create `src-tauri/src/db/mod.rs` with connection pool and migrations.
3. Create repository files under `src-tauri/src/db/`:
   - `settings_repo.rs`
   - `modes_repo.rs`
   - `vocabulary_repo.rs`
   - `history_repo.rs`
4. Update each service in `src-tauri/src/services/` to accept a repository (or connection) and return real data instead of `FeatureNotImplemented`.
5. Wire the database connection through the Tauri app state (via `tauri::State`).
6. Update the `invoke_handler` registration if any command signatures change (e.g., adding `State<DbPool>`).
7. Add migrations and verify settings persist after app restart.

Coordinate with the Backend Contracts agent if model fields need to change — do not change model field names without a PR comment.

---

## What the Frontend Agent Must Add to `src/lib/api.ts`

The frontend agent should add `invoke()` wrappers for all new commands.
Exact TypeScript call signatures for every command:

```typescript
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  DictationMode,
  VocabularyEntry,
  HistoryEntry,
  AiProvider,
  DiagnosticReport,
  PrivacySummary,
  PrivacyOperation,
  PrivacyDecision,
} from "./types";

// Settings
export const getSettings = () =>
  invoke<AppSettings>("get_settings");

export const updateSettings = (settings: AppSettings) =>
  invoke<void>("update_settings", { settings });

// Modes
export const listModes = () =>
  invoke<DictationMode[]>("list_modes");

export const getMode = (id: string) =>
  invoke<DictationMode>("get_mode", { id });

export const createMode = (mode: DictationMode) =>
  invoke<DictationMode>("create_mode", { mode });

export const updateMode = (mode: DictationMode) =>
  invoke<DictationMode>("update_mode", { mode });

export const deleteMode = (id: string) =>
  invoke<void>("delete_mode", { id });

// Vocabulary
export const listVocabulary = () =>
  invoke<VocabularyEntry[]>("list_vocabulary");

export const addVocabularyEntry = (entry: VocabularyEntry) =>
  invoke<VocabularyEntry>("add_vocabulary_entry", { entry });

export const updateVocabularyEntry = (entry: VocabularyEntry) =>
  invoke<VocabularyEntry>("update_vocabulary_entry", { entry });

export const deleteVocabularyEntry = (id: string) =>
  invoke<void>("delete_vocabulary_entry", { id });

// History
export const listHistory = () =>
  invoke<HistoryEntry[]>("list_history");

export const getHistoryEntry = (id: string) =>
  invoke<HistoryEntry>("get_history_entry", { id });

export const deleteHistoryEntry = (id: string) =>
  invoke<void>("delete_history_entry", { id });

export const clearHistory = () =>
  invoke<void>("clear_history");

// Privacy
export const getPrivacyStatus = () =>
  invoke<PrivacySummary>("get_privacy_status");

export const enforcePrivacyPreview = (op: PrivacyOperation) =>
  invoke<PrivacyDecision>("enforce_privacy_preview", { op });

// Providers
export const listProviders = () =>
  invoke<AiProvider[]>("list_providers");

export const getProvider = (id: string) =>
  invoke<AiProvider>("get_provider", { id });

export const testProviderPlaceholder = (id: string) =>
  invoke<string>("test_provider_placeholder", { id });

// Diagnostics
export const runDiagnosticsPlaceholder = () =>
  invoke<DiagnosticReport>("run_diagnostics_placeholder");
```

TypeScript types to add to `src/lib/types.ts`:

```typescript
export interface AppSettings {
  active_mode: string;
  local_only_mode: boolean;
  theme: string;
  retention_days: number;
  audio_history_enabled: boolean;
  clipboard_restore_enabled: boolean;
}

export interface DictationMode {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  active: boolean;
  builtin: boolean;
}

export interface VocabularyEntry {
  id: string;
  term: string;
  replacement: string;
  category: string;
  enabled: boolean;
}

export interface HistoryEntry {
  id: string;
  raw_text: string;
  cleaned_text: string;
  mode_used: string;
  timestamp: string;
  was_inserted: boolean;
}

export interface AiProvider {
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  model: string;
  enabled: boolean;
  use_for_cleanup: boolean;
  use_for_transcription: boolean;
  api_key_set: boolean;
}

export interface DiagnosticCheck {
  name: string;
  status: string;
  message: string;
}

export interface DiagnosticReport {
  checks: DiagnosticCheck[];
  generated_at: string;
}

export interface PrivacySummary {
  local_only_mode: boolean;
  audio_retention_days: number;
  history_retention_days: number;
  cloud_allowed: boolean;
  reason: string;
}

export interface PrivacyOperation {
  operation_type: string;
  provider_id: string | null;
}

export interface PrivacyDecision {
  allowed: boolean;
  reason: string;
}
```

---

## Privacy Impact

- No data leaves the app.
- No cloud calls of any kind.
- No real storage — all data is discarded after each call.
- API keys are never stored, logged, or handled. The `api_key_set: bool` field is a boolean flag only.
- `PrivacyService::enforce_privacy_preview` is a contract stub; real enforcement is Phase 8.

---

## What Is Intentionally Not Implemented

- No SQLite schema or migrations (Database agent, `phase/02-storage-settings`)
- No real data persistence (all data methods return `FeatureNotImplemented`)
- No real provider API calls
- No audio, transcription, shortcut, or text insertion
- No frontend changes
- No API key storage of any kind
- No `chrono` dependency (timestamps are static strings for now)
