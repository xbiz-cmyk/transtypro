# Phase 10 Handoff: Push-to-Talk Dictation Pipeline

## Phase goal

Add a push-to-talk (PTT) mode that lets the user press the configured global shortcut once to start recording in the background (without stealing focus from the target app), press it again to stop, then automatically runs the pipeline: stop recording → transcribe → optional cleanup → insert into the active application → save to history.

## Shortcut behavior model

Three values are stored in `settings.shortcut_behavior`:

| Value | Behavior |
|---|---|
| `open_dictation` | Opens the Dictation page (Phase 7/9 default) |
| `push_to_talk_toggle` | Press once to start, press again to stop and insert |
| `push_to_talk_hold` | Hold to record, release to insert (stored for cross-platform compatibility; see Windows note) |

### Windows: push_to_talk_hold maps to push_to_talk_toggle at runtime

`tauri-plugin-global-shortcut` on Windows uses `RegisterHotKey` / `WM_HOTKEY` under the hood. This API fires on key-down only — `ShortcutState::Released` never arrives on Windows. If `push_to_talk_hold` were handled naively on Windows, pressing the shortcut would start recording with no way to stop it.

**Fix applied:** the shortcut handler in `lib.rs` normalises the behavior at runtime using `cfg!()`:

```rust
let behavior = {
    let raw = read_shortcut_behavior(app_handle);
    if cfg!(target_os = "windows") && raw == "push_to_talk_hold" {
        "push_to_talk_toggle".to_string()
    } else {
        raw
    }
};
```

The `push_to_talk_hold` value is intentionally kept in the DB schema and Settings UI (disabled option) for cross-platform compatibility: a config file created on macOS and copied to Windows will degrade gracefully to toggle mode rather than getting stuck.

The `Released` branch in `lib.rs` is kept for correctness on macOS/Linux, where the event may fire.

## Cancellation during recording

`cancel_ptt` (Tauri command) calls `AudioService::cancel_recording` when the PTT phase is `Recording` before resetting the phase:

```rust
if ptt_state.is_phase(&PttPhase::Recording) {
    let audio_view = ptt_state.audio_state_view();
    let _ = AudioService::cancel_recording(&audio_view); // errors ignored (race-safe)
}
ptt_state.set_phase(PttPhase::Idle);
```

If a race condition occurs (pipeline already stopped the recording), `cancel_recording` returns an error which is ignored — no audio is lost and no WAV is written.

Manual QA required: full live test (start PTT recording → press Cancel in overlay → verify recording stops and no WAV is written and no transcription runs).

## History cleaned_text

`cleaned_text` in the PTT history entry is always set to `final_text` (the cleaned text if cleanup ran, otherwise the raw text). It is never blank when raw text exists. This ensures the History UI always has text to display for PTT entries.

## Files changed

### Rust backend

| File | Change |
|---|---|
| `src-tauri/src/db/migrations.rs` | Migration 005: `ALTER TABLE settings ADD COLUMN shortcut_behavior TEXT NOT NULL DEFAULT 'open_dictation'` + 3 tests |
| `src-tauri/src/models/mod.rs` | `AppSettings.shortcut_behavior: String` field; `PttStatusEvent { phase, message }` struct |
| `src-tauri/src/db/repositories/settings_repo.rs` | SELECT/upsert updated for col 9 (`shortcut_behavior`) + 3 tests |
| `src-tauri/src/services/ptt.rs` | **New file.** `PttPhase` enum, `PttState` (Tauri-managed), `PttPipelineService::run_pipeline()` + 9 unit tests |
| `src-tauri/src/services/mod.rs` | `pub mod ptt;` + re-exports |
| `src-tauri/src/commands/ptt.rs` | **New file.** `cancel_ptt` Tauri command (stops active recording before resetting phase) |
| `src-tauri/src/commands/mod.rs` | `pub mod ptt;` |
| `src-tauri/src/lib.rs` | Shared Arc fields for audio; manages `PttState`; shortcut handler normalises `push_to_talk_hold` to `push_to_talk_toggle` on Windows; `cancel_ptt` registered |
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

`PttState::audio_state_view()` reconstructs a temporary `AudioState` from these shared Arcs so `AudioService` methods can be called from the pipeline thread or from `cancel_ptt` without touching the managed `AudioState` type.

### PTT pipeline thread

`ptt_stop_and_run()` in `lib.rs` spawns a `std::thread::spawn` to run `PttPipelineService::run_pipeline()`. The shortcut callback returns immediately; the pipeline runs on its own thread. The pipeline emits `ptt-status` events at each phase transition via `tauri::Emitter`.

### Cleanup failure is non-fatal

If the optional cleanup step fails (network error, disabled provider, privacy block), the pipeline falls back to `raw_text` and continues to insert. Only `stop_recording` and `transcribe` failures abort the pipeline.

### InsertionService called directly

`InsertionService::insert_text()` is called directly from the pipeline — NOT via the `insert_text` Tauri command. The Tauri command minimizes and restores the window to give focus back to the target app, which is wrong for PTT (the target app already has focus and must not be disturbed).

### `unsafe impl Send/Sync for PttState`

`PttState` contains `RecordingHandle` which wraps `cpal::Stream`, which is not `Send`. All fields are behind `Arc<Mutex<…>>`, so cross-thread access is safe. The `unsafe impl` is required to satisfy Tauri's `T: Send + Sync` bound on managed state.

## Test results

- **Rust**: 152 tests pass (`cargo test`)
- **Clippy**: clean (`cargo clippy --all-targets --all-features -- -D warnings`)
- **Format**: clean (`cargo fmt --check`)
- **TypeScript**: 0 errors (`tsc --noEmit`)
- **Frontend build**: success (`npm run build`, 291 kB JS bundle)
- **quality-check.ps1**: all checks passed

## Known limitations

- `push_to_talk_hold` Released events do not fire on Windows; runtime maps it to toggle mode (see above).
- No audio level meter in the overlay during PTT recording.
- PTT pipeline does not capture active app context (window title / process name) before inserting — same limitation as Phase 9 insertion.
- Startup retention sweep is not automatic — manual only via Settings.
- Cancel during recording: pure phase logic is unit-tested; full live test (start → cancel → verify no WAV / no transcription) requires manual QA.

## Next recommended task

- Phase 11: Packaging (Tauri bundler, installer, update manifest).
- Optional: active app context capture (window title + PID) to show in overlay during PTT.
