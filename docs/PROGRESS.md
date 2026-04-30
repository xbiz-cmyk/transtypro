# Progress Log

Use this file to keep long-running agent work stable across sessions.

## Current phase

Phase 3 Audio Recording — IMPLEMENTED on `phase/03-audio-recording`, PR open for review.
Phase 2 Storage (Wave 2) — MERGED into main.
Phase 0 — Bootstrap — MERGED (PR #1, commit `ad0678d`).
Phase 2 Backend Contracts — MERGED into main.
Phase 1 UI Shell — MERGED into main.

## Last completed work

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

- Phase 3 Audio Recording PR open against `main` — awaiting orchestrator review and merge

## Current known limitations

- No transcription (Phase 4). WAV files are saved but not processed.
- No audio retention sweep (Phase 8). Temporary WAV files accumulate.
- No microphone preference persistence (Phase 3 by design).
- No cleanup providers (Phase 5) — `ProvidersService` still returns `FeatureNotImplemented`.
- No dictation pipeline (Phase 6) — `HistoryService::create_history_entry` is ready but not called.
- No global shortcut (Phase 7).
- `DiagnosticsService` still returns a static report (Phase 8).
- History list is empty until the dictation pipeline creates entries.
- Retention policy is stored but not enforced (Phase 8).

## Next recommended task

1. Orchestrator: review and merge `phase/03-audio-recording` PR
2. Launch Phase 4: Local Transcription agent → `phase/04-local-transcription`
3. Optionally in parallel: Phase 5 Cleanup Providers → `phase/05-cleanup-providers`
4. After Phase 4 and Phase 5: start Phase 6 dictation pipeline
