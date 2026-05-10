# Progress Log

Use this file to keep long-running agent work stable across sessions.

## Current phase

Phase 8 Privacy/Diagnostics/Retention — IMPLEMENTED on `phase/08-privacy-diagnostics`, PR open for review.
Phase 7 Global Shortcut — MERGED into main.
Phase 6 Dictation Pipeline — MERGED into main.
Phase 5 Cleanup Providers — MERGED into main.
Phase 4 Local Transcription — MERGED into main.
Phase 3 Audio Recording — MERGED into main.
Phase 2 Storage (Wave 2) — MERGED into main.
Phase 0 — Bootstrap — MERGED (PR #1, commit `ad0678d`).
Phase 2 Backend Contracts — MERGED into main.
Phase 1 UI Shell — MERGED into main.

## Last completed work

Phase 8 Privacy/Diagnostics/Retention: Real backend wired to Privacy, Diagnostics, and Settings pages.
- New: `src-tauri/src/services/retention.rs` — `RetentionService` with history + WAV cleanup (4 safety rules enforced before every deletion)
- New: `RetentionResult { deleted_history_count, deleted_wav_count }` model
- Updated: `src-tauri/src/db/repositories/history_repo.rs` — `delete_older_than(days)` method
- Updated: `src-tauri/src/services/diagnostics.rs` — full rewrite with 14 real checks (backend_alive, DB reachable, migrations, microphone, whisper binary/model, providers, ollama, shortcut, audio dir, history count, audio dir size)
- Updated: `src-tauri/src/services/mod.rs` — added `pub mod retention` + `RetentionService` re-export
- Updated: `src-tauri/src/commands/diagnostics.rs` — replaced placeholder; added `run_diagnostics` + `apply_retention_policy`
- Updated: `src-tauri/src/lib.rs` — registered both new commands
- Updated: `src/lib/api.ts` — 3 new wrappers: `getPrivacyStatus`, `runDiagnostics`, `applyRetentionPolicy`
- Updated: `src/lib/types.ts` — `RetentionResult` interface
- Updated: `src/pages/Privacy.tsx` — removed mock, wired to `get_privacy_status`
- Updated: `src/pages/Diagnostics.tsx` — removed mock, Run button enabled, wired to `run_diagnostics`
- Updated: `src/pages/Settings.tsx` — wired to backend; Save/Clear/Cleanup buttons all enabled
- Updated: `src/components/FloatingOverlay.tsx` — pulse indicator changed from red to brand blue
- 17 new tests (117 total: 2 history_repo + 6 retention + 9 diagnostics); all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (117/117), npm lint, npm build, quality-check.ps1
- Handoff: `handoff/phase-08-privacy-diagnostics.md`

Phase 7 Global Shortcut: System-wide `CommandOrControl+Shift+Space` shortcut that opens the floating overlay and navigates to /dictation.
- New: `tauri-plugin-global-shortcut = "2"` Cargo dependency
- New: `global-shortcut:allow-register` capability permission
- Updated: `lib.rs` — plugin init (`Builder::new().build()`) + shortcut registration in setup; graceful failure via `eprintln!` (not panic)
- Updated: `src/stores/uiStore.ts` — added `openOverlay()` and `closeOverlay()` actions
- New: `src/components/ShortcutHandler.tsx` — null-rendering component inside BrowserRouter; listens for `"dictation-shortcut-pressed"`, calls `openOverlay()`, navigates to `/dictation`
- Updated: `src/App.tsx` — `<ShortcutHandler />` mounted as first child inside `<BrowserRouter>`
- Updated: `src/components/FloatingOverlay.tsx` — dismiss uses `closeOverlay()`; "Go to Dictation →" link replaces stale Phase 6 text
- Updated: `src/pages/Settings.tsx` — removed "Phase 7" placeholder text
- All 100 existing Rust tests pass; cargo fmt, clippy, npm lint, npm build, quality-check all pass
- Handoff: `handoff/phase-07-global-shortcut-overlay.md`

Phase 6 Dictation Pipeline: End-to-end history persistence and live status summary.
- New: `create_history_entry` Tauri command — saves dictation result to SQLite history
- Updated: `get_status_summary` — reads real DB values (privacy mode, transcription ready, cleanup provider, active mode, history count)
- New: `build_status_summary` helper — extracted for testability, bypasses Tauri State
- Updated: `src/lib/api.ts` — 4 new history wrappers: `listHistory`, `deleteHistoryEntry`, `clearHistory`, `createHistoryEntry`
- Updated: `src/pages/History.tsx` — removed `MOCK_ENTRIES`; real backend with loading/error/delete/clear states
- Updated: `src/pages/Dictation.tsx` — Save as note enabled; fetches `active_mode` from settings
- Updated: `src/pages/Home.tsx` — removed stale "Recording available in Phase 3" text
- 9 new unit tests (100 total); all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (100/100), npm lint, npm build, quality-check.ps1
- Handoff: `handoff/phase-06-dictation-pipeline.md`

Phase 5 Cleanup Providers: SQLite-backed provider CRUD + OS keychain key storage + Ollama/OpenAI HTTP cleanup + privacy enforcement.
- New: migration 003 — `providers` table
- New: `ProvidersRepository` — full CRUD + `api_key_set` flag + `list_enabled_cleanup`
- New: `ProvidersService` — delegates to repo; API keys stored in OS keychain via `keyring = "3"`
- New: `CleanupService` — disabled-provider guard, privacy enforcement, system prompt from active mode, Ollama + OpenAI-compatible HTTP via `ureq = "2"`
- New: `commands/cleanup.rs` — `cleanup_text` Tauri command
- Updated: `commands/providers.rs` — 9 real commands (`get_provider` is real SQLite; `test_provider_placeholder` delegates to `test_connection`)
- Updated: `lib.rs` — all Phase 5 commands registered
- Updated: `src/lib/types.ts` — `CleanupResult` interface
- Updated: `src/lib/api.ts` — 8 new API wrappers
- Updated: `src/pages/Providers.tsx` — full rewrite with real backend (list, add, delete, test, set-api-key modal, `api_key_set` badge)
- Updated: `src/pages/Dictation.tsx` — cleanup provider picker + "Clean text" button; textarea shows cleaned text when available
- 91 unit tests; all pass (21 new tests added in Phase 5)
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (91 pass), npm build, lint, quality-check.ps1
- Handoff: `handoff/phase-05-cleanup-providers.md`

Phase 4 Local Transcription: Local whisper.cpp binary invocation wired to the Dictation page.
- New: `src-tauri/src/services/transcription.rs` — `TranscriptionService::transcribe` (path validation, Command::new, WAV cleanup) + 8 unit tests
- New: `src-tauri/src/commands/transcription.rs` — `transcribe_audio` Tauri command with privacy check
- New: `TranscriptionResult { raw_text, duration_ms, model_path }` model
- New: `AppError::TranscriptionError(String)` variant
- New migration 002: `ALTER TABLE settings ADD COLUMN whisper_binary_path / whisper_model_path`
- Updated: `AppSettings` — `whisper_binary_path: Option<String>`, `whisper_model_path: Option<String>`
- Updated: `settings_repo` — reads/writes two new columns
- Updated: `Dictation.tsx` — Transcribe button, transcript textarea, Copy button
- Updated: `Models.tsx` — Whisper configuration card (binary path + model path + Save)
- Updated: `src/lib/api.ts` + `src/lib/types.ts` — `transcribeAudio`, `getSettings`, `updateSettings` wrappers and types
- 70 unit tests; all pass (13 new tests added in Phase 4)
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (70 pass), npm build, lint, quality-check.ps1
- Handoff: `handoff/phase-04-local-transcription.md`

Phase 3 Audio Recording: Real microphone recording to temporary WAV file.
- New: `cpal = "0.15"`, `hound = "3.5"` dependencies
- New: `src-tauri/src/services/audio.rs` — `AudioState`, `RecordingHandle`, `AudioService`,
  pure helpers (rms, i16/u16/f32 conversions, mix_to_mono, duration_ms, build_wav_path),
  dedicated audio thread, WAV writer
- New: `src-tauri/src/commands/audio.rs` — 5 Tauri commands:
  `list_microphones`, `start_recording`, `stop_recording`, `cancel_recording`,
  `get_recording_status`
- New: `AppError::AudioError(String)` variant
- New: `MicrophoneInfo`, `RecordingStatus`, `RecordingResult` Rust models + TS interfaces
- Updated: `lib.rs` — `AudioState` managed separately from `AppState`; 5 commands registered
- Updated: `Dictation.tsx` — mic selector, Record/Stop/Cancel buttons, RMS level meter,
  WAV result panel; Copy/Insert/Save remain disabled
- Updated: `src/lib/api.ts` + `src/lib/types.ts` — 5 wrappers + 3 interfaces
- 20 pure-function unit tests (TDD: red phase confirmed 18 failures, green phase all 56 pass)
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (56 pass), npm build, lint,
  quality-check.ps1
- Handoff: `handoff/phase-03-audio-recording.md`

Phase 2 Storage (Wave 2): SQLite persistence layer wired to all storage-backed services.
- New: `rusqlite` (bundled), `uuid`, `chrono` dependencies
- New: `db/connection.rs` — `AppState { db: Arc<Mutex<Connection>> }`
- New: `db/migrations.rs` — idempotent migration runner; migration 001 creates 4 tables + seeds 10 built-in modes and 1 settings row
- New: `db/repositories/` — `SettingsRepository`, `ModesRepository`, `VocabularyRepository`, `HistoryRepository`
- Updated: 5 services (`settings`, `modes`, `vocabulary`, `history`, `privacy`) — real SQLite calls
- Updated: 5 command modules — added `tauri::State<'_, AppState>` wiring
- Updated: `lib.rs` — `.setup()` hook opens DB, runs migrations, manages `AppState`
- Added: `HistoryService::create_history_entry` (service method only; no Tauri command yet)
- Privacy: `enforce_privacy_preview` fails closed on unknown operations when local-only mode is on
- 36 unit tests; all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test, npm run build, quality-check.ps1
- Handoff: `handoff/phase-02-storage-settings.md`

Phase 2 Backend Contracts: All Tauri command interfaces, service contracts, data models,
and AppError variants defined on `phase/02-backend-contracts`.
- 10 new models (AppSettings, DictationMode, VocabularyEntry, HistoryEntry, AiProvider,
  DiagnosticCheck, DiagnosticReport, PrivacySummary, PrivacyOperation, PrivacyDecision)
- 6 new AppError variants
- 7 service structs (SettingsService, ModesService, VocabularyService, HistoryService,
  PrivacyService, ProvidersService, DiagnosticsService)
- 21 new Phase 2 commands registered (24 total including Phase 0)
- All commands return Result<T, AppError> — AppError implements Serialize for Tauri IPC
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test, npm run build
- Handoff: `handoff/phase-02-backend-contracts.md`

Phase 0: Project skeleton created and merged into `main`.
- Tauri v2 + React 19 + TypeScript + Vite 7
- Tailwind CSS v4 with dark-theme-first design tokens
- Rust backend module structure (commands, services, errors, models, db, utils)
- Typed `AppError` with `FeatureNotImplemented` variant (no panic/todo)
- Frontend: Sidebar, StatusBar, Home page with system status
- `ping` command verifies frontend-backend IPC
- react-router-dom v7 routing
- All checks pass: `cargo fmt`, `cargo clippy`, `cargo test`, `tsc --noEmit`, `npm run build`
- ADR-001 (SQLite deferred) and ADR-002 (frontend layout) recorded

## Current orchestrator status

- Phase 8 Privacy/Diagnostics/Retention PR open against `main` — awaiting orchestrator review and merge

## Current known limitations

- No OS file picker for whisper binary or model path, or provider URLs (manual path entry only).
- No model download UI (out of scope).
- Global shortcut registered but no shortcut rebinding UI (deferred to Phase 9).
- No text insertion (Phase 9) — Insert button remains disabled; `was_inserted` always `false`.
- No clipboard paste simulation (Phase 9).
- `history_count` in `get_status_summary` uses `list_history().len()` — O(n); acceptable for now.
- No startup auto-sweep — retention is manual only (Phase 9 can add startup sweep).
- No search/filter backend for History page.
- No confirmation dialog before "Clear all" in History page.
- No provider enable/disable toggle in UI (update_provider command exists).
- Language selector in Settings is cosmetic local state only.
- Diagnostics export not implemented.

## Next recommended task

1. Orchestrator: review and merge Phase 8 PR (`phase/08-privacy-diagnostics`)
2. Launch Phase 9: Text insertion into active app (`was_inserted = true`), custom shortcut rebinding
3. Phase 9: Consider adding startup auto-sweep after manual retention path is validated
