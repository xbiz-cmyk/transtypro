# Phase 10 Handoff: Push-to-Talk Dictation Pipeline

## Phase goal

Add a push-to-talk (PTT) mode that lets the user press the configured global shortcut once to start recording in the background (without stealing focus from the target app), press it again to stop, then automatically runs the pipeline: stop recording → transcribe → optional cleanup → insert into the active application → save to history.

## Shortcut behavior model

Three values are stored in `settings.shortcut_behavior`:

| Value | Behavior |
|---|---|
| `open_dictation` | Opens the Dictation page (Phase 7/9 default) |
| `push_to_talk_toggle` | Press once to start, press again to stop and insert |
| `push_to_talk_hold` | Hold to record, release to insert (**not functional on Windows** — see below) |

### Windows caveat: Released events do not fire

`tauri-plugin-global-shortcut` on Windows uses `RegisterHotKey` / `WM_HOTKEY` under the hood. This API fires on key-down only — `ShortcutState::Released` never arrives on Windows. Therefore `push_to_talk_hold` is present in the DB schema and Rust code but is permanently disabled in the Settings UI dropdown. Users on Windows should use `push_to_talk_toggle`.

The `Released` branch in `lib.rs` is kept for correctness on macOS/Linux, where the event may fire.

## Files changed

### Rust backend

| File | Change |
|---|---|
| `src-tauri/src/db/migrations.rs` | Migration 005: `ALTER TABLE settings ADD COLUMN shortcut_behavior TEXT NOT NULL DEFAULT 'open_dictation'` + 3 tests |
| `src-tauri/src/models/mod.rs` | `AppSettings.shortcut_behavior: String` field; `PttStatusEvent { phase, message }` struct |
| `src-tauri/src/db/repositories/settings_repo.rs` | SELECT/upsert updated for col 9 (`shortcut_behavior`) + 3 tests |
| `src-tauri/src/services/ptt.rs` | **New file.** `PttPhase` enum, `PttState` (Tauri-managed), `PttPipelineService::run_pipeline()` + 8 unit tests |
| `src-tauri/src/services/mod.rs` | `pub mod ptt;` + re-exports |
| `src-tauri/src/commands/ptt.rs` | **New file.** `cancel_ptt` Tauri command |
| `src-tauri/src/commands/mod.rs` | `pub mod ptt;` |
| `src-tauri/src/lib.rs` | Shared Arc fields for audio; manages `PttState`; shortcut handler branches on `shortcut_behavior`; `cancel_ptt` registered |
| `src-tauri/src/services/diagnostics.rs` | `check_migrations_current` uses version 5; `check_shortcut_configured` reads shortcut from DB + 4 new tests |
| `src-tauri/src/services/privacy.rs` | Minimal compile-only fix: `shortcut_behavior` field added to test struct literal |

### Frontend

| File | Change |
|---|---|
| `src/stores/uiStore.ts` | `pttPhase`, `pttMessage`, `setPttStatus` added |
| `src/components/ShortcutHandler.tsx` | `ptt-status` listener: opens overlay on active phases, auto-hides 2 s after "done", closes on idle/cancelled |
| `src/components/FloatingOverlay.tsx` | PTT status display with dynamic indicator color + Cancel button (cancellable phases only) |
| `src/pages/Settings.tsx` | `shortcutBehavior` state + Select dropdown; `push_to_talk_hold` option disabled with Windows note |
| `src/lib/types.ts` | `AppSettings.shortcut_behavior: string`; `PttStatusEvent` interface |
| `src/lib/api.ts` | `cancelPtt()` wrapper |

## Architecture decisions

### Shared Arc fields pattern

`AudioState` and `PttState` both reference the same audio recording Arcs. They are constructed in `lib.rs` before either state is managed:

```rust
let recording_arc = Arc::new(Mutex::new(None));
// ...
app.manage(AudioState { recording: recording_arc.clone(), ... });
app.manage(PttState   { recording: recording_arc,        ... });
```

`PttState::audio_state_view()` reconstructs a temporary `AudioState` from these shared Arcs so `AudioService` methods can be called from the pipeline thread without touching the managed `AudioState` type.

### PTT pipeline thread

`ptt_stop_and_run()` in `lib.rs` spawns a `std::thread::spawn` to run `PttPipelineService::run_pipeline()`. The shortcut callback returns immediately; the pipeline runs on its own thread. The pipeline emits `ptt-status` events at each phase transition via `tauri::Emitter`.

### Cleanup failure is non-fatal

If the optional cleanup step fails (network error, disabled provider, privacy block), the pipeline falls back to `raw_text` and continues to insert. Only `stop_recording` and `transcribe` failures abort the pipeline.

### InsertionService called directly

`InsertionService::insert_text()` is called directly from the pipeline — NOT via the `insert_text` Tauri command. The Tauri command minimizes and restores the window to give focus back to the target app, which is wrong for PTT (the target app already has focus and must not be disturbed).

### `unsafe impl Send/Sync for PttState`

`PttState` contains `RecordingHandle` which wraps `cpal::Stream`, which is not `Send`. All fields are behind `Arc<Mutex<…>>`, so cross-thread access is safe. The `unsafe impl` is required to satisfy Tauri's `T: Send + Sync` bound on managed state.

## Test results

- **Rust**: 151 tests pass (`cargo test`)
- **Clippy**: clean (`cargo clippy --all-targets --all-features -- -D warnings`)
- **Format**: clean (`cargo fmt --check`)
- **TypeScript**: 0 errors (`tsc --noEmit`)
- **Frontend build**: success (`npm run build`, 291 kB JS bundle)

## Known limitations

- `push_to_talk_hold` is not functional on Windows (Windows `RegisterHotKey` fires press-only).
- No audio level meter in the overlay during PTT recording (RMS level polling would require a separate timer or interval event).
- PTT pipeline does not capture active app context (window title / process name) before inserting — same limitation as Phase 9 insertion.
- Startup retention sweep is not automatic — manual only via Settings.

## Next recommended task

- Orchestrator: review and merge Phase 9 PR (`phase/09-text-insertion`) before or alongside this PR.
- Phase 11: Packaging (Tauri bundler, installer, update manifest).
- Optional: active app context capture (window title + PID) to show in overlay during PTT.
