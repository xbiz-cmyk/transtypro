# Phase 4: Local Whisper Transcription — Handoff

## Branch

`phase/04-local-transcription`

## Goal

Wire `whisper.cpp`-compatible binary invocation to the Dictation page so that a recorded WAV file can be transcribed locally without any cloud call.

## What was built

### Migration 002

`src-tauri/src/db/migrations.rs` — added `MIGRATION_002` which runs two `ALTER TABLE` statements on the `settings` table to add `whisper_binary_path TEXT DEFAULT NULL` and `whisper_model_path TEXT DEFAULT NULL`. The existing idempotent runner (`schema_migrations` version table) ensures this runs exactly once.

### AppSettings extension

`src-tauri/src/models/mod.rs` — added `whisper_binary_path: Option<String>` and `whisper_model_path: Option<String>` to `AppSettings`. Added `TranscriptionResult { raw_text, duration_ms, model_path }`.

`src-tauri/src/db/repositories/settings_repo.rs` — extended `get()` to read the two new columns (default `None`) and `upsert()` to write them.

### Error variant

`src-tauri/src/errors/mod.rs` — added `TranscriptionError(String)` to `AppError`.

### TranscriptionService

`src-tauri/src/services/transcription.rs` — two public methods:

- `TranscriptionService::transcribe(wav_path, binary_path, model_path, audio_dir)`:
  - Validates: binary non-empty, binary is a file, model non-empty, model is a file, wav non-empty, wav is a file, wav inside `audio_dir` (canonicalize + starts_with path traversal guard).
  - Spawns: `Command::new(binary_path).arg("-m").arg(model_path).arg("-f").arg(wav_path)` — no shell, no string interpolation.
  - Checks: non-zero exit → error with stderr; empty stdout → `TranscriptionError("Local transcription produced no stdout…")`.
  - Returns `TranscriptionResult`.

- `TranscriptionService::cleanup_wav_if_needed(file_path, audio_history_enabled)`:
  - Deletes the WAV file only when `!audio_history_enabled`. Silently ignores delete errors.
  - On transcription failure the command never calls cleanup, so the file is kept for debugging.

### transcribe_audio command

`src-tauri/src/commands/transcription.rs` — single `#[tauri::command]`:

1. Load settings via `SettingsService`.
2. Call `PrivacyService::enforce_privacy_preview` with `operation_type = "local_transcription"`. Returns error if blocked (local-only mode does **not** block local transcription — see privacy service allow-list).
3. Invoke `TranscriptionService::transcribe`.
4. Call `TranscriptionService::cleanup_wav_if_needed` only on success.
5. Return `TranscriptionResult`.

Argument name: `file_path` (Tauri maps frontend camelCase `filePath` to snake_case `file_path`).

Registered in `src-tauri/src/lib.rs` `invoke_handler`.

### TypeScript / frontend

- `src/lib/types.ts` — `AppSettings` extended with `whisper_binary_path: string | null`, `whisper_model_path: string | null`; `TranscriptionResult` interface added.
- `src/lib/api.ts` — `transcribeAudio(filePath)` wrapper added; `getSettings` / `updateSettings` wrappers added (needed by Models page).
- `src/pages/Dictation.tsx` — Transcribe button shown after recording stops; transcript displayed in read-only textarea; Copy button enabled when transcript is non-empty; transcript reset on new record or cancel.
- `src/pages/Models.tsx` — new "Whisper configuration" card with binary path input, model path input, and Save button wired to `getSettings` / `updateSettings`.

## Path safety

The `audio_dir` is the canonical path to the app audio directory set up in `lib.rs`. Before invoking whisper, `TranscriptionService::transcribe` calls `std::fs::canonicalize` on the wav path and checks `starts_with(audio_dir)`. This prevents path traversal attacks where a crafted file path could point outside the audio directory.

## Privacy

`local_transcription` is allowed when `local_only_mode = true` (local call, no cloud). If the privacy service is ever extended to block local transcription it will surface as a clear error to the user.

## WAV cleanup policy

| `audio_history_enabled` | Transcription result | WAV file |
|---|---|---|
| false | success | deleted |
| false | failure | kept (for debugging) |
| true | success | kept |
| true | failure | kept |

## Tests

70 Rust unit tests pass (`cargo test`):

- `test_migration_002_adds_whisper_columns` — verifies columns exist after migration
- `test_run_migrations_is_idempotent` — running migrations twice does not error
- `test_get_defaults_whisper_paths_to_none` — fresh DB returns `None` for both paths
- `test_upsert_persists_whisper_binary_path` — round-trip binary path
- `test_upsert_persists_whisper_model_path` — round-trip model path
- `test_transcribe_fails_empty_binary_path`
- `test_transcribe_fails_nonexistent_binary`
- `test_transcribe_fails_empty_model_path`
- `test_transcribe_fails_nonexistent_model`
- `test_transcribe_fails_nonexistent_wav`
- `test_transcribe_fails_wav_outside_audio_dir`
- `test_cleanup_deletes_wav_when_history_disabled`
- `test_cleanup_keeps_wav_when_history_enabled`

## Verification checklist

- [x] `cargo fmt --check` — pass
- [x] `cargo clippy --all-targets --all-features -- -D warnings` — pass
- [x] `cargo test` — 70/70 pass
- [x] `npm run lint` (tsc --noEmit) — pass
- [x] `npm run build` — pass
- [x] `pwsh scripts/quality-check.ps1` — all checks pass

## Manual test instructions

1. Build and run the app: `npm run tauri dev`
2. Open Settings → Models page.
3. Enter the full path to a whisper.cpp-compatible binary in "Whisper binary path".
4. Enter the full path to a `.bin` model file in "Model file path".
5. Click Save. Confirm no error banner appears.
6. Navigate to Dictation page.
7. Select microphone, click Record, speak, click Stop.
8. The WAV info panel appears below the textarea.
9. Click Transcribe.
10. The textarea shows the transcript text.
11. The Copy button becomes enabled; clicking it copies the transcript to clipboard.
12. Click Record again — transcript clears.
13. Click Cancel — transcript clears.

## Known limitations

- No OS file picker for whisper binary or model path (Phase 5/6).
- No model download (out of scope for Phase 4).
- No cleanup providers — raw transcript only (Phase 5).
- No dictation pipeline — transcript is not saved to history (Phase 6).
- No text insertion — Insert button remains disabled (Phase 9).
- WAV deletion on success when `audio_history_enabled = false` is the only retention sweep; the broader retention policy (by date) is not yet enforced.

## Changed files

```
src-tauri/src/commands/mod.rs          — added pub mod transcription
src-tauri/src/commands/transcription.rs — new file
src-tauri/src/db/migrations.rs         — migration 002
src-tauri/src/db/repositories/settings_repo.rs — whisper columns
src-tauri/src/errors/mod.rs            — TranscriptionError variant
src-tauri/src/lib.rs                   — register transcribe_audio
src-tauri/src/models/mod.rs            — AppSettings + TranscriptionResult
src-tauri/src/services/mod.rs          — pub mod transcription
src-tauri/src/services/privacy.rs      — fix struct literal (whisper fields)
src-tauri/src/services/transcription.rs — new file
src/lib/api.ts                          — getSettings, updateSettings, transcribeAudio
src/lib/types.ts                        — AppSettings extended, TranscriptionResult added
src/pages/Dictation.tsx                — Transcribe button, transcript textarea, Copy
src/pages/Models.tsx                   — Whisper configuration card
```

## Next recommended tasks

1. Orchestrator: review and merge `phase/04-local-transcription` PR
2. Phase 5: Cleanup providers (Ollama, OpenAI-compatible) — cleans raw transcript before display
3. Phase 6: End-to-end dictation pipeline — saves transcript + cleaned text to history, enables Insert button
4. Phase 5/6 prep: add OS file picker for whisper binary and model path selection
