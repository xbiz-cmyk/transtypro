# Wave 3 Launch — Handoff

## Base commit

Wave 3 branch created from `main` at commit `0cfa263` — "feat(storage): add sqlite persistence layer".

---

## Wave 3 scope

**Phase 3 — Audio Recording only.**

One agent, one branch, one worktree.

The goal is: a user presses Record on the Dictation page, speaks, presses Stop, and gets
back a path to a temporary WAV file. No transcription. No text insertion. No shortcuts.

### Deliverables

- List available microphones via `cpal`
- Select microphone (default or named device)
- Start recording to an in-memory buffer
- Stop recording → write temporary WAV to `%APPDATA%/transtypro/audio/<uuid>.wav`
- Cancel recording → discard buffer, no file written
- Expose RMS input level over IPC for a level meter in the UI
- Wire the Dictation page Record/Stop/Cancel buttons and mic selector
- Remove `// TODO: wire to backend` for recording-related calls only
- All other Dictation page buttons (Insert, Save, Copy) remain disabled

### Explicitly out of scope for Phase 3

- Diagnostics — `DiagnosticsService` and `src/pages/Diagnostics.tsx` are NOT touched
- Microphone preference in settings — do not change `AppSettings`, do not change the database schema, do not touch `settings_repo.rs` or `migrations.rs`
- Audio retention policy enforcement — `audio_history_enabled` field is already in settings; do not implement the sweep yet
- Local transcription — `src-tauri/src/services/transcription.rs` must NOT be created
- Whisper sidecar integration — Phase 4
- Text insertion — Phase 6
- Global shortcut — Phase 7
- Cleanup providers / provider calls — Phase 5
- Model downloads — Phase 4
- Cloud API calls of any kind
- History entries from dictation — Phase 6

---

## Branch and worktree

| Item | Value |
|---|---|
| Branch | `phase/03-audio-recording` |
| Worktree path | `C:\Users\User\Desktop\transtypro-audio` |
| Base commit | `0cfa263` |
| Status | Ready |

---

## Audio agent ownership boundaries

### Files the Audio agent may create or edit

```
src-tauri/src/services/audio.rs          new — AudioService
src-tauri/src/commands/audio.rs          new — list_microphones, start_recording,
                                               stop_recording, cancel_recording,
                                               get_recording_status
src-tauri/src/commands/mod.rs            append: pub mod audio
src-tauri/src/services/mod.rs            append: pub mod audio + re-export
src-tauri/src/lib.rs                     register audio commands; add AudioState
                                               as a second managed state
src-tauri/Cargo.toml                     add cpal, hound dependencies
src-tauri/Cargo.lock                     updated by cargo automatically
src-tauri/src/models/mod.rs              append MicrophoneInfo, RecordingStatus structs
src-tauri/src/errors/mod.rs              append AudioError(String) variant
src/pages/Dictation.tsx                  wire Record/Stop/Cancel, mic Select,
                                               level meter — nothing else
src/lib/api.ts                           add listMicrophones, startRecording,
                                               stopRecording, cancelRecording,
                                               getRecordingStatus wrappers
src/lib/types.ts                         add MicrophoneInfo, RecordingStatus interfaces
handoff/phase-03-audio-recording.md      create — full handoff
```

### Files the Audio agent must NOT touch

```
src-tauri/src/db/**                      storage is complete — no new migrations
src-tauri/src/services/settings.rs       owned by storage layer
src-tauri/src/services/modes.rs          owned by storage layer
src-tauri/src/services/vocabulary.rs     owned by storage layer
src-tauri/src/services/history.rs        dictation pipeline wires this in Phase 6
src-tauri/src/services/privacy.rs        privacy enforcement is complete
src-tauri/src/services/providers.rs      Phase 5
src-tauri/src/services/diagnostics.rs   Phase 8
src-tauri/src/commands/settings.rs
src-tauri/src/commands/modes.rs
src-tauri/src/commands/vocabulary.rs
src-tauri/src/commands/history.rs
src-tauri/src/commands/privacy.rs
src-tauri/src/commands/providers.rs
src-tauri/src/commands/diagnostics.rs
src/pages/Home.tsx
src/pages/History.tsx
src/pages/Modes.tsx
src/pages/Vocabulary.tsx
src/pages/Models.tsx
src/pages/Providers.tsx
src/pages/Settings.tsx                   do not add microphone preference here
src/pages/Privacy.tsx
src/pages/Diagnostics.tsx               Phase 8
src/pages/About.tsx
src/components/**                        reuse existing components — do not add new ones
src/stores/**                            no new stores needed
src/App.tsx                              no routing changes
docs/PHASES.md
docs/PARALLEL_EXECUTION_PLAN.md
AGENTS.md
CLAUDE.md
SOUL.md
handoff/**                               except own handoff file
scripts/**
tests/**
```

---

## Architectural notes for the Audio agent

### cpal stream is !Send on Windows

`cpal::Stream` cannot be stored directly in an `Arc<Mutex<>>` and shared across threads.
The correct pattern is:

1. Spawn an audio thread that owns the `Stream`.
2. Collect audio samples into a `Vec<f32>` buffer on the audio callback.
3. Send samples to the main thread via `std::sync::mpsc` channel.
4. Store the channel sender and the collected buffer in `AudioState`.

### AudioState (separate from AppState)

Add a second managed state type — do not put audio state inside `AppState` (which holds
the DB connection).

```rust
// In src-tauri/src/lib.rs or a new utils/audio_state.rs
pub struct AudioState {
    pub recording: Arc<Mutex<Option<RecordingHandle>>>,
    pub samples: Arc<Mutex<Vec<f32>>>,
}
```

Register in lib.rs `.setup()` hook alongside `AppState`.

### Temporary WAV files

Write to `app_data_dir()/audio/<uuid>.wav`.
Use `hound::WavWriter` to encode f32 samples at 16 kHz, mono, 16-bit PCM.
On `cancel_recording`, discard samples and do not write any file.
Do not implement retention sweep — just write the file and return the path.

### Tauri commands to implement

```rust
list_microphones() -> Result<Vec<MicrophoneInfo>, AppError>
start_recording(device_name: Option<String>) -> Result<(), AppError>
stop_recording() -> Result<String, AppError>     // returns path to WAV
cancel_recording() -> Result<(), AppError>
get_recording_status() -> Result<RecordingStatus, AppError>
```

---

## Checks before handoff

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run build
npm run lint
git status --short
git diff --stat
pwsh scripts/quality-check.ps1
```

All six quality-check gates must show PASS before the PR is opened.

---

## Lock file rule

- `Cargo.lock` — this agent may update (adding cpal, hound deps).
- `package-lock.json` — must not be touched.

---

## Merge rule

No PR may be merged unless the user provides the exact line:

ORCHESTRATOR APPROVED MERGE

---

## Next step

The user opens Claude Code in `C:\Users\User\Desktop\transtypro-audio` on branch
`phase/03-audio-recording` and pastes the Audio Recording agent prompt provided by
the orchestrator.

After Phase 3 is merged, Wave 4 planning begins:
- Phase 4 Local Transcription (depends on Phase 3 WAV output)
- Phase 5 Cleanup Providers (can be planned in parallel with Phase 4, but must branch
  from main AFTER Phase 3 is merged to avoid Cargo.toml and lib.rs conflicts)
