# Progress Log

Use this file to keep long-running agent work stable across sessions.

## Current phase

Phase 11 Product Polish — IMPLEMENTED + QA-fixed on `phase/11-product-polish`, PR open for review.
Phase 10.1 PTT Feedback Overlay Window — MERGED into main.
Phase 10 Push-to-Talk Pipeline — MERGED into main.
Phase 9 Text Insertion and Shortcut Rebinding — MERGED into main.
Phase 8 Privacy/Diagnostics/Retention — MERGED into main.
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

Phase 11 Product Polish: Visual polish, shortcut recorder, PTT speed setting, and overlay timing feedback.
- New: `src/components/Logo.tsx` — inline SVG logo: speech waveform bars + text cursor, `size` prop
- New: `src/components/icons.tsx` — 11 named SVG icon exports (HomeIcon…AboutIcon), consistent stroke style, no external packages
- Updated: `src/components/Sidebar.tsx` — emoji replaced with SVG icons via `<NavIcon>` lookup; Logo added to header; stale footer removed
- Updated: `src/pages/Home.tsx` — Logo added to heading; "Start dictating →" CTA button; singular/plural history count
- Updated: `src/pages/Settings.tsx` — shortcut recorder UI (Record/Use this/Cancel/Reset); ptt_output_mode selector (Best quality / Fast)
- Updated: `src/components/PttOverlay.tsx` — elapsed recording timer ("Listening… Xs"); "Live transcript preview" sub-label removed
- Updated: `src/lib/types.ts` — `ptt_output_mode: string` added to AppSettings interface
- Updated: `src-tauri/src/models/mod.rs` — `ptt_output_mode: String` added to AppSettings struct
- Updated: `src-tauri/src/db/migrations.rs` — migration 006 (`ptt_output_mode` column, default `clean_before_insert`) + 3 tests
- Updated: `src-tauri/src/db/repositories/settings_repo.rs` — SELECT/upsert/default for ptt_output_mode + 3 tests
- Updated: `src-tauri/src/services/ptt.rs` — `read_ptt_output_mode()` helper; `run_pipeline()` branches: `insert_raw` skips cleanup and emits no cleaning phase
- Updated: `src-tauri/src/services/diagnostics.rs` — expected migration version bumped 5 → 6 (1 line)
- Updated: `src-tauri/src/services/privacy.rs` — minimal forced touch: `ptt_output_mode` added to test struct literal
- QA fix: `src-tauri/src/lib.rs` — `read_shortcut_behavior`, `ptt_start`, `ptt_stop_and_run`, `ptt_toggle` made `pub(crate)`
- QA fix: `src-tauri/src/commands/shortcut.rs` — `update_shortcut` now registers behavior-aware handler (PTT toggle/hold/open_dictation) matching lib.rs setup(); previously hardcoded open_dictation
- QA fix: `src-tauri/src/services/ptt.rs` — `strip_whisper_timestamps()` helper strips `[HH:MM:SS --> HH:MM:SS]` markers from Whisper output before insertion; applied in both insert_raw and clean_before_insert paths; 6 new tests
- QA fix: `src/pages/Settings.tsx` — shortcut recorder "Use this" only returns to idle on backend success; stays in captured state for error retry
- 164 tests total (158 → 164, 6 new); all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (164/164), npm lint (tsc --noEmit), npm build (315.83 kB), quality-check.ps1
- Handoff: `handoff/phase-11-product-polish.md`

Phase 10.1 PTT Feedback Overlay Window: Secondary Tauri window (`ptt-overlay`) that appears while PTT is active and shows pipeline phase without stealing focus from the active app.
- New: `src/components/PttOverlay.tsx` — standalone overlay component: `ptt-status` listener, animated waveform bars (CSS-only, no audio data), phase labels (Listening… / Transcribing… / Cleaning… / Inserting… / Done. / error), Cancel button (recording/transcribing/cleaning), Dismiss button (error), auto-hide on done/idle/cancelled, default state `phase="recording"` for first-event safety
- Updated: `src-tauri/src/lib.rs` — `WebviewWindowBuilder` creates `ptt-overlay` at startup (hidden, always-on-top, decoration-free, non-focusable, skip-taskbar, 320×96 px, positioned bottom-center of primary monitor); `ptt_start()` calls `overlay.show()` before spawning the recording thread to guarantee listener registration before first event
- Updated: `src/App.tsx` — extracted `MainApp` component; `App` checks `IS_PTT_OVERLAY = getCurrentWindow().label === "ptt-overlay"` at module load time and renders `<PttOverlay />` or `<MainApp />` accordingly
- Updated: `src-tauri/capabilities/default.json` — `"ptt-overlay"` added to `windows` array (minimal capability change)
- No new Rust tests needed (changes are window wiring, not logic); 152 existing tests all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (152/152), npm lint (tsc --noEmit), npm build (307.85 kB), quality-check.ps1
- Focus safety: `.focused(false)` at build time; `show()` only, no `set_focus()`; active app retains focus throughout PTT pipeline
- First-event delivery: pre-created window hydrates at startup; `show()` before thread spawn; fallback default state `phase="recording"`
- Handoff: `handoff/phase-10.1-ptt-overlay-window.md`

Phase 10 Push-to-Talk Pipeline: PTT mode added. Pressing the global shortcut starts recording in the background (no focus steal); pressing again stops and runs the full pipeline: stop → transcribe → optional cleanup → insert → save history.
- New: `src-tauri/src/services/ptt.rs` — `PttPhase` enum, `PttState` (Tauri-managed with shared Arc audio fields), `PttPipelineService::run_pipeline()` + 8 unit tests
- New: `src-tauri/src/commands/ptt.rs` — `cancel_ptt` Tauri command
- New migration 005: `ALTER TABLE settings ADD COLUMN shortcut_behavior TEXT NOT NULL DEFAULT 'open_dictation'`
- Updated: `AppSettings` — `shortcut_behavior: String` field; new `PttStatusEvent { phase, message }` model
- Updated: `settings_repo` — persists/reads `shortcut_behavior` + 3 tests
- Updated: `lib.rs` — shared Arc fields for audio (AudioState + PttState share same Arcs); shortcut handler branches on `shortcut_behavior`; `cancel_ptt` registered
- Updated: `services/diagnostics.rs` — `check_migrations_current` now expects version 5; `check_shortcut_configured` reads from DB + 4 new tests
- Updated: `services/privacy.rs` — minimal compile-only fix: `shortcut_behavior` in test struct literal
- Updated: `src/stores/uiStore.ts` — `pttPhase`, `pttMessage`, `setPttStatus`
- Updated: `src/components/ShortcutHandler.tsx` — `ptt-status` listener; auto-hides overlay 2 s after done
- Updated: `src/components/FloatingOverlay.tsx` — PTT phase display with dynamic indicator color + Cancel button
- Updated: `src/pages/Settings.tsx` — shortcut behavior dropdown (hold option disabled on Windows)
- Updated: `src/lib/types.ts` — `AppSettings.shortcut_behavior`, `PttStatusEvent` interface
- Updated: `src/lib/api.ts` — `cancelPtt()` wrapper
- 19 new tests (151 total); all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (151/151), npm lint, npm build
- Windows caveat: `push_to_talk_hold` is not functional on Windows (`RegisterHotKey` fires press-only); `push_to_talk_toggle` is the working PTT mode on Windows
- Handoff: `handoff/phase-10-ptt-pipeline.md`

Phase 9 Text Insertion and Shortcut Rebinding: Insert button wired to active app via clipboard paste simulation; shortcut rebindable at runtime.
- New: `src-tauri/src/services/insertion.rs` — `InsertionService` (arboard clipboard write + enigo Ctrl+V/Cmd+V simulation, clipboard restore)
- New: `src-tauri/src/commands/insertion.rs` — `insert_text` (sync, window minimize/restore) + `mark_history_inserted`
- New: `src-tauri/src/commands/shortcut.rs` — `update_shortcut` with register-first strategy + 5 validation unit tests
- Updated: `Cargo.toml` — `arboard = "3"`, `enigo = "0.2"`
- Updated: migration 004 — `ALTER TABLE settings ADD COLUMN shortcut TEXT NOT NULL DEFAULT 'CommandOrControl+Shift+Space'`
- Updated: `AppSettings` — added `shortcut: String`; added `InsertionResult { success, method, message }` model
- Updated: `errors/mod.rs` — `InsertionError(String)` variant
- Updated: `settings_repo` — persists/reads shortcut column
- Updated: `history_repo` + `HistoryService` — `mark_inserted` sets `was_inserted = true`
- Updated: `lib.rs` — reads shortcut from DB before Arc<Mutex> wrap; registers 3 new commands
- Updated: `services/privacy.rs` — minimal forced touch: added `shortcut` field to test struct literal (required by struct completeness rule after adding field to `AppSettings`)
- Updated: `src/lib/types.ts` — `shortcut` on `AppSettings`, new `InsertionResult` interface
- Updated: `src/lib/api.ts` — `insertText`, `markHistoryInserted`, `updateShortcut`
- Updated: `src/pages/Dictation.tsx` — Insert button active after transcription; `handleInsert` with paste-fail fallback; `markHistoryInserted` called only if note was saved
- Updated: `src/pages/Settings.tsx` — editable shortcut input + Apply button with success/error feedback
- 15 new tests (132 total); all pass
- All checks pass: cargo fmt, cargo clippy -D warnings, cargo test (132/132), npm lint, npm build, quality-check.ps1
- Known limitation: DiagnosticsService check #11 (`shortcut_configured`) still hardcodes default string; fix recommended in Phase 10
- Handoff: `handoff/phase-09-text-insertion.md`

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

- Phase 11 Product Polish PR open against `main` — awaiting orchestrator review and merge

## Current known limitations

- No OS file picker for whisper binary or model path, or provider URLs (manual path entry only).
- No model download UI (out of scope).
- `push_to_talk_hold` is not functional on Windows (`RegisterHotKey` fires press-only); `push_to_talk_toggle` is the working PTT mode on Windows.
- No real audio level meter in overlay during PTT recording (animated bars are CSS-only decorative placeholders).
- `history_count` in `get_status_summary` uses `list_history().len()` — O(n); acceptable for now.
- No startup auto-sweep — retention is manual only.
- No search/filter backend for History page.
- No confirmation dialog before "Clear all" in History page.
- No provider enable/disable toggle in UI (update_provider command exists).
- Language selector in Settings is cosmetic local state only.
- Diagnostics export not implemented.
- No active app context capture (window title/process name) before insertion.
- Shortcut recorder does not support Fn-only shortcuts (hardware-level).
- Modifier-only shortcuts produce a warning but pass to backend for final validation.
- Elapsed timer shows whole seconds only (sub-second not needed for this UX).

## Next recommended task

1. Orchestrator: review and merge Phase 11 PR (`phase/11-product-polish`)
2. Phase 12: Packaging (Tauri bundler, installer, update manifest)
3. Optional: wire `get_recording_status` RMS level to overlay bar heights for real audio feedback
4. Optional: active app context capture (window title + PID) to show in overlay during PTT
