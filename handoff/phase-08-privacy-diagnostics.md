# Phase 8 Handoff: Privacy Enforcement, Diagnostics, Retention, and Polish

## Summary

Phase 8 replaces all remaining static/mock placeholder data with real backend behavior:
- `DiagnosticsService` rewritten with 14 real system checks
- `RetentionService` created — enforces history and WAV file retention policy
- Privacy page wired to real `get_privacy_status` backend command
- Settings page wired to real `getSettings`/`updateSettings` with Save, Clear History, and Run Retention Cleanup buttons enabled
- Diagnostics page wired to real `runDiagnostics()` — "Run diagnostics" button enabled
- Floating overlay indicator changed from red (error color) to brand blue
- No new DB migration, no new Cargo dependencies, no forbidden files touched

---

## Backend Changes

### New model: `RetentionResult`

Added to `src-tauri/src/models/mod.rs`:
```rust
pub struct RetentionResult {
    pub deleted_history_count: u32,
    pub deleted_wav_count: u32,
}
```

### New method: `HistoryRepository::delete_older_than`

Added to `src-tauri/src/db/repositories/history_repo.rs`:
- `delete_older_than(days: u32) -> Result<u32, AppError>`
- Returns `Ok(0)` immediately when `days == 0` (keep forever)
- Deletes rows where `datetime(timestamp) < datetime('now', '-N days')`

### New service: `RetentionService`

Created `src-tauri/src/services/retention.rs`:

**`apply_history_retention()`**
- Reads `settings.retention_days` from DB
- Calls `HistoryRepository::delete_older_than(retention_days)`

**`apply_audio_retention()`**
- Reads `settings.audio_history_enabled` and `settings.retention_days`
- `audio_history_enabled = false` → delete ALL regular `.wav` files (crash/cancel leftovers)
- `audio_history_enabled = true`, `retention_days > 0` → delete `.wav` files older than N days
- `audio_history_enabled = true`, `retention_days = 0` → keep forever, return `Ok(0)`

**Safety rules enforced in code before every file deletion:**
1. `path.starts_with(&audio_dir)` — skip and warn if outside audio directory
2. `std::fs::symlink_metadata(&path)` is used (does NOT follow symlinks); if `file_type().is_symlink()` → skip and warn
3. `metadata.is_file()` — skip directories after symlink check
4. `extension == "wav"` (case-sensitive) — skip non-WAV files
5. Per-file errors are non-fatal — `eprintln!` and continue
6. `audio_dir` is the Tauri-managed `data_dir/audio` directory

Note: symlink tests are not included in the unit-test suite because Windows symlink creation requires elevated privileges and is unreliable in CI. The symlink guard is code-enforced via `symlink_metadata()` + `is_symlink()` check.

**`apply_all()`** — calls both methods and returns `RetentionResult`

No startup auto-sweep added. Manual cleanup button in Settings is sufficient for Phase 8.

### Rewritten service: `DiagnosticsService`

Rewrote `src-tauri/src/services/diagnostics.rs`. Now takes `db` and `audio_dir`. Runs 14 independent checks:

| # | Check | Pass condition | Fail/Warn/Skip |
|---|---|---|---|
| 1 | `backend_alive` | Always | Always pass |
| 2 | `database_reachable` | `SELECT 1 FROM settings WHERE id=1` | `fail` |
| 3 | `migrations_current` | `schema_migrations` has version 3 | `warn` |
| 4 | `microphone_available` | `cpal` input devices ≥ 1 | `warn` |
| 5 | `whisper_binary_configured` | `settings.whisper_binary_path.is_some()` | `warn` |
| 6 | `whisper_binary_exists` | File exists at path | `fail` / `skip` if unconfigured |
| 7 | `whisper_model_configured` | `settings.whisper_model_path.is_some()` | `warn` |
| 8 | `whisper_model_exists` | File exists at path | `fail` / `skip` if unconfigured |
| 9 | `providers_configured` | `COUNT(*) FROM providers WHERE enabled=1 > 0` | `warn` |
| 10 | `ollama_reachable` | `GET {base_url}/api/tags` returns 200 within 3s | `warn` / `skip` |
| 11 | `shortcut_configured` | `"CommandOrControl+Shift+Space".parse()` succeeds | `fail` |
| 12 | `audio_dir_accessible` | `audio_dir.is_dir()` | `warn` |
| 13 | `history_count` | Always | message shows count |
| 14 | `audio_dir_size` | Always | message shows file count + bytes |

**Privacy guarantees for diagnostics:**
- No API keys read from OS keychain
- No secrets logged anywhere
- `ollama_reachable` calls `GET /api/tags` only — no credentials, no data sent
- Report returned via Tauri IPC only — never sent externally
- No telemetry, no diagnostics export

### Updated: `services/mod.rs`

Added `pub mod retention` and `pub use retention::RetentionService`.

### Updated: `commands/diagnostics.rs`

Replaced `run_diagnostics_placeholder` with:
- `run_diagnostics(state, audio_state) -> Result<DiagnosticReport, AppError>`
- `apply_retention_policy(state, audio_state) -> Result<RetentionResult, AppError>`

Both commands are in `commands/diagnostics.rs` (not a separate `commands/retention.rs`) because `commands/mod.rs` is an orchestrator-only file.

### Updated: `lib.rs`

Removed: `commands::diagnostics::run_diagnostics_placeholder`
Added:
- `commands::diagnostics::run_diagnostics`
- `commands::diagnostics::apply_retention_policy`

---

## Frontend Changes

### `src/lib/api.ts`

Added 3 new wrappers:
```typescript
getPrivacyStatus()    // → PrivacySummary  (invoke "get_privacy_status")
runDiagnostics()      // → DiagnosticReport (invoke "run_diagnostics")
applyRetentionPolicy() // → RetentionResult (invoke "apply_retention_policy")
```

### `src/lib/types.ts`

Added:
```typescript
interface RetentionResult {
  deleted_history_count: number;
  deleted_wav_count: number;
}
```

### `src/pages/Privacy.tsx`

- Removed `MOCK_PRIVACY_SUMMARY`
- Added `useEffect` → `getPrivacyStatus()` on mount
- Shows loading state while fetching
- Shows error message on failure
- Displays real `PrivacySummary` data
- Removed `"Retention enforcement — Phase 8"` placeholder text
- Retention card now shows actionable text: "Use the Settings page to run manual retention cleanup."
- `DATA_FLOW_ITEMS` kept as-is (static architectural description)

### `src/pages/Diagnostics.tsx`

- Removed `MOCK_CHECK_ITEMS`
- "Run diagnostics" button is now enabled and calls `runDiagnostics()`
- Shows loading state (`"Running…"`) during call
- On success: renders real `DiagnosticReport.checks` and `generated_at` timestamp
- On failure: shows error message
- Added `"skip"` status support → `"muted"` badge with "Skip" label
- Initial state shows clean empty state (no Phase 8 references)
- "Export" button remains disabled with `title="Diagnostics export — not yet implemented"`
- Check names formatted as human-readable (snake_case → Title Case)

### `src/pages/Settings.tsx`

- Added `useEffect` → `getSettings()` on mount: hydrates all form fields
- **Save settings button** is now enabled → calls `updateSettings()` with all fields merged over loaded base settings (preserves `whisper_binary_path`, `whisper_model_path`)
- **Clear all history button** is now enabled → calls `clearHistory()` with `window.confirm` guard
- Removed stale `"Theme switching — Phase 2"` helper text
- Removed stale `"Real path available — Phase 2"` helper text from database path
- Database path display now shows `"Stored in your OS app data directory"`
- Default mode select values changed to lowercase IDs (`"smart"`, `"raw"`, etc.) to match `AppSettings.active_mode`
- Added **Storage cleanup section** with "Run retention cleanup" button → calls `applyRetentionPolicy()`
- Cleanup result: `"Deleted X history entries and Y audio files."`
- Shortcut display and "Shortcut rebinding coming in a future release." text: **unchanged**
- Added note: Fn-only shortcuts are unsupported — handled at hardware/firmware level

### `src/components/FloatingOverlay.tsx`

- Changed pulse indicator color from `bg-(--color-status-error)` (red) to `bg-(--color-brand-400)` (brand blue)
- Applied to both the ping ring and the solid dot
- `animate-ping` animation kept — signals shortcut was received
- Rationale: red falsely communicated "recording active"; the shortcut only navigates and does not start recording

---

## Settings Save Rule

When calling `updateSettings()`, the Settings page merges locally-edited fields over the loaded `AppSettings` base object via spread (`...loadedSettings`). This preserves `whisper_binary_path` and `whisper_model_path` even though they are not shown in the Settings UI. They are only editable on the Models page.

---

## Tests Added

**17 new tests — total now 117 (was 100)**

### `history_repo.rs` — 2 new tests
- `test_delete_older_than_zero_keeps_all` — `days=0` must delete nothing
- `test_delete_older_than_removes_old_keeps_recent` — old entry deleted, recent survives

### `services/retention.rs` — 6 new tests
- `test_history_retention_zero_days_keeps_all`
- `test_history_retention_deletes_old_keeps_recent`
- `test_audio_retention_deletes_wav_when_history_disabled`
- `test_audio_retention_skips_non_wav_files`
- `test_audio_retention_keeps_forever_when_history_enabled_and_zero_days`
- `test_apply_all_returns_combined_result`

### `services/diagnostics.rs` — 9 new tests
- `test_backend_alive_always_passes`
- `test_database_reachable_passes_with_valid_db`
- `test_migrations_current_passes_after_run`
- `test_providers_configured_warns_with_empty_db`
- `test_ollama_reachable_skips_when_no_ollama_provider`
- `test_shortcut_configured_passes`
- `test_whisper_checks_warn_when_unconfigured`
- `test_report_has_all_14_checks`
- `test_generated_at_is_non_empty`

---

## Checks Run and Results

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS (0 warnings) |
| `cargo test` | PASS — 117/117 tests |
| `npm run lint` (tsc --noEmit) | PASS |
| `npm run build` | PASS (288 kB JS bundle) |
| `pwsh scripts/quality-check.ps1` | PASS — all checks passed |

---

## Manual QA Checklist

- [ ] App opens without crash
- [ ] Privacy page loads real privacy status from backend
- [ ] Privacy page shows loading state briefly before data arrives
- [ ] Diagnostics page shows empty state before first run
- [ ] Diagnostics page "Run diagnostics" button is enabled
- [ ] Diagnostics page shows real check results after clicking Run
- [ ] All 14 checks appear in the diagnostics report
- [ ] Diagnostics page Export button remains disabled
- [ ] Settings page loads real settings from backend on open
- [ ] Settings page Save settings button is enabled
- [ ] Settings page Save correctly persists changes (verify with restart)
- [ ] Settings page Clear all history is enabled and asks for confirmation
- [ ] Settings page Run retention cleanup button runs and shows result
- [ ] Whisper model and binary paths survive a Settings save (not wiped)
- [ ] Floating overlay dot is brand blue, not red
- [ ] Global shortcut still triggers overlay after Phase 8 changes
- [ ] Microphone, record, transcribe, cleanup still work as before

---

## Known Limitations

- Language selector in Settings is cosmetic local state (no `AppSettings` field mapping)
- No startup auto-sweep — retention is manual only (Phase 9 can add startup sweep if needed)
- `history_count` in `get_status_summary` still uses O(n) `list_history().len()` (minor, acceptable)
- Text insertion remains disabled (`was_inserted` always `false`) — Phase 9
- No search/filter backend for History page
- macOS audio device enumeration untested (Phase 8 tested on Windows)
- Diagnostics export not implemented

---

## Deferred to Phase 9

- **Custom shortcut rebinding** — not implemented in Phase 8. Settings page shows:
  > "Shortcut rebinding coming in a future release."
- **Fn-only shortcut** — unsupported and unreliable. Many keyboards handle Fn at the hardware/firmware level before the OS can intercept it. `tauri-plugin-global-shortcut` cannot reliably capture Fn-only key combinations. Documented in the Settings UI.
- **Text insertion into active app** (`was_inserted = true`)
- **Clipboard paste simulation**

---

## Privacy Confirmation

- Diagnostics data is **never sent anywhere**. It is returned as a Tauri IPC response to the local frontend only.
- No API keys are read, exposed, or logged by any Phase 8 code path.
- The Ollama reachability check only calls `GET {base_url}/api/tags` — no credentials, no data.
- Retention sweep deletes local files only — no network calls.
- `get_privacy_status` is read-only.
- `apply_retention_policy` deletes local data only — no network calls.
- No telemetry added.
- No diagnostics export added.

---

## Files Changed

### New files
- `src-tauri/src/services/retention.rs` — RetentionService with history + WAV cleanup
- `handoff/phase-08-privacy-diagnostics.md` — this file

### Modified files
| File | Change |
|---|---|
| `src-tauri/src/models/mod.rs` | Added `RetentionResult` struct |
| `src-tauri/src/db/repositories/history_repo.rs` | Added `delete_older_than` method + 2 tests |
| `src-tauri/src/services/diagnostics.rs` | Full rewrite with 14 real checks + 9 tests |
| `src-tauri/src/services/mod.rs` | Added `pub mod retention` and `RetentionService` re-export |
| `src-tauri/src/commands/diagnostics.rs` | Replaced placeholder with real `run_diagnostics` + new `apply_retention_policy` |
| `src-tauri/src/lib.rs` | Removed placeholder, registered two new commands |
| `src/lib/api.ts` | Added 3 new wrappers: `getPrivacyStatus`, `runDiagnostics`, `applyRetentionPolicy` |
| `src/lib/types.ts` | Added `RetentionResult` interface |
| `src/pages/Privacy.tsx` | Removed mock, wired to real backend |
| `src/pages/Diagnostics.tsx` | Removed mock, wired to real backend, enabled Run button |
| `src/pages/Settings.tsx` | Wired to backend, enabled Save/Clear/Cleanup buttons |
| `src/components/FloatingOverlay.tsx` | Changed indicator from red to brand blue |

### Forbidden files — confirmed NOT touched
`src-tauri/src/services/audio.rs`, `src-tauri/src/commands/audio.rs`,
`src-tauri/src/services/transcription.rs`, `src-tauri/src/commands/transcription.rs`,
`src-tauri/src/services/cleanup.rs`, `src-tauri/src/commands/cleanup.rs`,
`src-tauri/src/services/providers.rs`, `src-tauri/src/commands/providers.rs`,
`src-tauri/src/services/privacy.rs`, `src-tauri/src/commands/privacy.rs`,
`src-tauri/src/services/history.rs`, `src-tauri/src/commands/history.rs`,
`src-tauri/src/commands/mod.rs`, `src-tauri/src/db/migrations.rs`,
`src-tauri/src/db/repositories/settings_repo.rs`, `src-tauri/src/db/repositories/modes_repo.rs`,
`src-tauri/src/db/repositories/vocabulary_repo.rs`, `src-tauri/src/db/repositories/providers_repo.rs`,
`src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`,
`src/stores/uiStore.ts`, `src/components/ShortcutHandler.tsx`, `src/App.tsx`,
`src/pages/Dictation.tsx`, `src/pages/History.tsx`, `src/pages/Providers.tsx`,
`src/pages/Models.tsx`, `src/pages/Home.tsx`,
`docs/PHASES.md`, `docs/PARALLEL_EXECUTION_PLAN.md`, `docs/ARCHITECTURE.md`,
`AGENTS.md`, `CLAUDE.md`, `SOUL.md`

---

## Next Recommended Task

1. Orchestrator: review Phase 8 PR and send `ORCHESTRATOR APPROVED MERGE`
2. Launch Phase 9: Text insertion into active app (`was_inserted = true`), custom shortcut rebinding
3. Address startup auto-sweep in Phase 9 after manual retention path is validated
