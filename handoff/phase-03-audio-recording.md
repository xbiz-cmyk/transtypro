# Phase 03 Handoff — Audio Recording Agent

## Branch

`phase/03-audio-recording`

---

## Summary

Phase 3 adds real microphone recording to transtypro. A user can navigate to the
Dictation page, select a microphone, press **Record**, speak, and press **Stop** to
receive a temporary WAV file. Press **Cancel** to discard without writing a file.
A live RMS level meter updates at 200 ms intervals while recording.

No transcription, no text insertion, no cloud calls, no settings schema changes.

---

## Architecture

### cpal::Stream threading

`cpal::Stream` is `!Send` on WASAPI (Windows). It cannot be stored in an
`Arc<Mutex<>>` or moved between threads. The solution:

1. `start_recording` spawns a dedicated OS thread that **owns the stream**.
2. The audio callback on that thread pushes `f32` samples into a shared
   `Arc<Mutex<Vec<f32>>>` buffer.
3. The audio thread reports setup success/failure through a
   `std::sync::mpsc::channel` before `start_recording` returns.
4. `stop_recording` / `cancel_recording` set an `Arc<AtomicBool>` stop flag,
   call `thread.unpark()`, and join the thread. The stream drops when the thread exits.

### AudioState (managed by Tauri, separate from AppState)

```rust
pub struct AudioState {
    pub audio_dir: PathBuf,                         // app_data_dir/audio/
    pub recording: Arc<Mutex<Option<RecordingHandle>>>,
    pub samples:   Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Arc<Mutex<u32>>,
    pub channels:    Arc<Mutex<u16>>,
}
```

`RecordingHandle` stores the stop flag, thread join handle, mic name, and
`started_at` — but NOT the `cpal::Stream` (that lives on the audio thread).

### WAV output format

- **Mono 16-bit PCM** regardless of capture channel count.
- If the device has > 1 channel, samples are mixed down by averaging channel
  frames before writing.
- Sample rate: native device rate (not resampled in Phase 3).
- File path: `%APPDATA%/transtypro/audio/<uuid>.wav`.
- Cancel discards the buffer and writes no file.
- Maximum recording: 5 minutes. Samples beyond the limit are silently dropped
  (buffer is capped at `5 min × sample_rate × channels` total samples).

---

## Commands Added

| Tauri command | Rust signature | Purpose |
|---|---|---|
| `list_microphones` | `() → Result<Vec<MicrophoneInfo>, AppError>` | Enumerate input devices |
| `start_recording` | `(device_name: Option<String>) → Result<RecordingStatus, AppError>` | Begin recording |
| `stop_recording` | `() → Result<RecordingResult, AppError>` | Write WAV, return path |
| `cancel_recording` | `() → Result<RecordingStatus, AppError>` | Discard buffer, no file |
| `get_recording_status` | `() → Result<RecordingStatus, AppError>` | Live RMS level |

> **Frontend argument naming:** Tauri maps Rust `snake_case` command parameter names to `camelCase` in JavaScript by default. The `start_recording` command's `device_name: Option<String>` argument must therefore be passed as `deviceName` from the frontend (`invoke("start_recording", { deviceName: ... })`). No `#[tauri::command(rename_all = "snake_case")]` attribute is used.

---

## Models Added

### Rust (`src-tauri/src/models/mod.rs`)

```rust
pub struct MicrophoneInfo {
    pub name: String,
    pub is_default: bool,
}

pub struct RecordingStatus {
    pub is_recording: bool,
    pub selected_microphone: Option<String>,
    pub level_rms: f32,        // range 0.0–1.0
    pub sample_count: u64,     // total raw samples in buffer
}

pub struct RecordingResult {
    pub file_path: String,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,         // always 1 (mono output)
}
```

### TypeScript (`src/lib/types.ts`)

```typescript
interface MicrophoneInfo  { name: string; is_default: boolean; }
interface RecordingStatus { is_recording: boolean; selected_microphone: string | null;
                            level_rms: number; sample_count: number; }
interface RecordingResult { file_path: string; duration_ms: number;
                            sample_rate: number; channels: number; }
```

---

## Error Variant Added

`AppError::AudioError(String)` — returned for all audio device, stream, WAV, and
lock-related errors. No `unwrap()`, `panic!()`, `todo!()`, or
`unimplemented!()` in any callable path.

---

## Frontend Dictation Page Behavior

1. **On mount** — `listMicrophones()` populates the mic selector; auto-selects the
   default device.
2. **Record button** — enabled when idle; calls `startRecording(selectedMic)`;
   starts a 200 ms `setInterval` polling `getRecordingStatus()` to drive the level
   meter.
3. **Stop button** — shown while recording; calls `stopRecording()`; clears the
   interval; displays the WAV path and duration below the result textarea.
4. **Cancel button** — shown while recording; calls `cancelRecording()`; clears the
   interval; resets state to idle with no file shown.
5. **Level meter** — a `<div role="progressbar">` whose width scales with
   `level_rms × 100 %`. Green fill, 100 ms CSS transition.
6. **WAV info** — displayed after stop: file path, duration, sample rate, "mono 16-bit PCM".
   Clearly labelled "Transcription not yet available — coming in Phase 4."
7. **Copy / Insert / Save** — remain disabled (no transcribed text yet).
8. **Interval cleanup** — `useEffect` return function clears the interval on unmount;
   also cleared explicitly on Stop and Cancel.

---

## Files Created

| File | Purpose |
|---|---|
| `src-tauri/src/services/audio.rs` | `AudioState`, `RecordingHandle`, `AudioService`, pure helpers, audio thread, WAV writer |
| `src-tauri/src/commands/audio.rs` | 5 thin Tauri command wrappers |

## Files Modified

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Added `cpal = "0.15"`, `hound = "3.5"` |
| `src-tauri/Cargo.lock` | Updated automatically |
| `src-tauri/src/errors/mod.rs` | Added `AudioError(String)` variant |
| `src-tauri/src/models/mod.rs` | Added `MicrophoneInfo`, `RecordingStatus`, `RecordingResult` |
| `src-tauri/src/services/mod.rs` | Added `pub mod audio`; re-exported `AudioService`, `AudioState` |
| `src-tauri/src/commands/mod.rs` | Added `pub mod audio` |
| `src-tauri/src/lib.rs` | Created `audio_dir`; managed `AudioState`; registered 5 audio commands |
| `src/lib/types.ts` | Added 3 audio interfaces |
| `src/lib/api.ts` | Added 5 audio command wrappers |
| `src/pages/Dictation.tsx` | Wired mic selector, Record/Stop/Cancel, level meter, WAV result panel |

---

## Tests Added

20 pure-function unit tests in `src-tauri/src/services/audio.rs` `#[cfg(test)]`:

| Test | What it verifies |
|---|---|
| `rms_empty_slice_is_zero` | No divide-by-zero on empty input |
| `rms_full_scale_is_one` | ±1.0 samples → RMS 1.0 |
| `rms_half_scale_is_half` | ±0.5 samples → RMS 0.5 |
| `i16_to_f32_zero_maps_to_zero` | i16 0 → 0.0 |
| `i16_to_f32_max_maps_near_one` | i16::MAX → ~1.0 |
| `i16_to_f32_min_maps_near_neg_one` | i16::MIN → ~-1.0 |
| `u16_to_f32_max_maps_near_one` | u16::MAX → ~1.0 |
| `u16_to_f32_midpoint_maps_near_zero` | u16 32768 → ~0.0 |
| `f32_to_i16_zero_is_zero` | 0.0 → 0 |
| `f32_to_i16_one_maps_to_max` | 1.0 → i16::MAX |
| `f32_to_i16_clamp_high` | 2.0 clamped → i16::MAX |
| `f32_to_i16_clamp_low` | -2.0 clamped → i16::MIN |
| `mix_to_mono_already_mono_returns_unchanged` | 1-channel input passes through |
| `mix_to_mono_stereo_averages_channels` | [0.5,-0.5] → 0.0; [1.0,0.0] → 0.5 |
| `duration_ms_zero_samples_is_zero` | 0 samples → 0 ms |
| `duration_ms_one_second_at_44100` | 44100 samples at 44100 Hz → 1000 ms |
| `duration_ms_half_second_at_16000` | 8000 samples at 16000 Hz → 500 ms |
| `wav_path_ends_with_wav_extension` | Path ends with `.wav` |
| `wav_path_is_inside_given_dir` | Path starts with given dir |
| `wav_paths_are_unique` | Two calls return different paths (UUID-based) |

TDD cycle followed: stubs returning `42`/`42.0` were written first; 18 of 20 tests
failed in the red phase (2 passed immediately because stub returns dir/fixed.wav which
still ends with `.wav` and is in dir); all 20 pass after implementation.

**Total tests: 56 passed, 0 failed** (36 Phase 2 + 20 Phase 3).

---

## Checks Run

| Check | Result |
|---|---|
| `cargo fmt` | ✅ pass |
| `cargo fmt --check` | ✅ pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ pass — 0 warnings |
| `cargo test` | ✅ pass — 56 passed, 0 failed |
| `npm install` | ✅ pass |
| `npm run build` | ✅ pass — 71 modules, 0 errors |
| `npm run lint` (tsc --noEmit) | ✅ pass — 0 errors |
| `pwsh scripts/quality-check.ps1` | ✅ all checks passed |

---

## Privacy Impact

- No data leaves the device.
- No cloud calls of any kind.
- WAV files are written to the OS app data directory (`%APPDATA%/transtypro/audio/`).
- `cancel_recording` never writes a file.
- No API keys are stored, logged, or handled.
- Audio retention sweep is NOT implemented in Phase 3 — WAV files accumulate until
  Phase 8 implements the retention policy against `audio_history_enabled` and
  `retention_days`.
- No microphone preference is persisted to the database in Phase 3.

---

## Known Limitations

- No transcription (Phase 4). WAV files are saved but not processed.
- No audio retention sweep (Phase 8). Temporary WAV files are not deleted.
- No microphone preference persistence. The selected mic is UI state only.
- Level meter shows `level_rms` polled at 200 ms, which is sufficient for UI
  feedback but not suitable for precise metering.
- Capture uses the device's native channel count (often mono for microphones).
  If the device reports stereo, samples are mixed to mono before WAV write.
  `RecordingResult.channels` always reports `1`.
- The 5-minute recording limit silently drops additional samples; it does not
  interrupt the UI session. The recorded WAV will be shorter than the elapsed time.
- Sample formats beyond F32/I16/U16 (e.g. I32, F64) return
  `AppError::AudioError("unsupported sample format: …")`. This covers all common
  Windows/macOS input devices.

---

## What Phase 4 Should Do Next

1. Read `RecordingResult.file_path` (from `stop_recording`) as the WAV input.
2. Accept a configured model path and whisper-compatible binary path from settings.
3. Execute the whisper binary with the WAV path; capture stdout as raw transcript.
4. Return the transcript as `TranscriptionResult { raw_text, duration_ms }`.
5. Wire the Dictation page to call `transcribe_audio(file_path)` after `stop_recording`.
6. Enable the Copy button once transcript is available.
7. Use `AppSettings.audio_history_enabled` to decide whether to delete the WAV after
   transcription (if false, delete; if true, keep until Phase 8 retention sweep).
8. Handle missing model and missing binary with clear, actionable error messages.
