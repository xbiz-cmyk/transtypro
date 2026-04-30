# Progress Log

Use this file to keep long-running agent work stable across sessions.

## Current phase

Phase 2 Storage (Wave 2) — IMPLEMENTED on `phase/02-storage-settings`, PR open for review.
Phase 0 — Bootstrap — MERGED (PR #1, commit `ad0678d`).
Phase 2 Backend Contracts — MERGED into main.
Phase 1 UI Shell — MERGED into main.

## Last completed work

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

- Phase 2 Storage PR open against `main` — awaiting orchestrator review and merge
- Wave 3 (Audio/STT, Providers) should start after Phase 2 storage is merged

## Current known limitations

- No audio recording (Phase 3).
- No transcription (Phase 4).
- No cleanup providers (Phase 5) — `ProvidersService` still returns `FeatureNotImplemented`.
- No dictation pipeline (Phase 6) — `HistoryService::create_history_entry` is ready but not called yet.
- No global shortcut (Phase 7).
- `DiagnosticsService` still returns a static report (Phase 8).
- History list is empty until the dictation pipeline creates entries.
- Retention policy is stored but not enforced (Phase 8).

## Next recommended task

1. Orchestrator: review and merge `phase/02-storage-settings` PR
2. Launch Wave 3:
   - Audio/STT agent → `phase/03-audio-recording`
   - (Optionally, in parallel) Providers backend → `phase/05-cleanup-providers`
3. After Phase 3 and Phase 5: start Phase 6 dictation pipeline
