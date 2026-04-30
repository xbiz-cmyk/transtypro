# Wave 4 Launch: Phase 4 Local Transcription

## Base commit

`36cb66d` — feat(audio): add local microphone recording

All Wave 1, 2, and 3 work is merged into main. This handoff is committed on main before the Phase 4 branch diverges.

## Branch and worktree

| Item           | Value                                     |
| -------------- | ----------------------------------------- |
| Branch         | `phase/04-local-transcription`            |
| Worktree       | `C:\Users\User\Desktop\transtypro-stt`    |
| Base           | `main` at `36cb66d`                       |

## Phase 4 scope: Local transcription only

Accept the WAV file path produced by Phase 3 stopRecording(), invoke a user-configured whisper.cpp-compatible binary, capture stdout, and return the raw transcript text to the Dictation page.

### What Phase 4 MUST implement

- Migration 002: add `whisper_binary_path TEXT DEFAULT NULL` and `whisper_model_path TEXT DEFAULT NULL` to the settings table
- Update `AppSettings` struct and settings repository for the two new fields
- `TranscriptionResult` model: `{ raw_text: String, duration_ms: u64, model_path: String }`
- `TranscriptionError(String)` variant in `AppError`
- `TranscriptionService` with a `transcribe(file_path, binary_path, model_path)` static method
- `transcribe_audio` Tauri command
- Frontend: Transcribe button in Dictation.tsx (shown after stopRecording succeeds)
- Frontend: populate result textarea with `raw_text` after transcription
- Frontend: enable Copy button after text is available
- Frontend: Models.tsx — wire binary path and model path text inputs to `getSettings` / `updateSettings`
- `api.ts`: `transcribeAudio(filePath: string)` wrapper
- `types.ts`: `TranscriptionResult` interface and updated `AppSettings` with optional new fields
- `handoff/phase-04-local-transcription.md`: post-implementation handoff document

### What Phase 4 must NOT implement

- Do not bundle whisper.cpp
- Do not download models
- Do not implement cloud transcription
- Do not implement OpenAI API
- Do not implement AI provider cleanup
- Do not implement text insertion
- Do not implement global shortcuts
- Do not implement history entry creation
- Do not implement the full dictation pipeline
- Do not store API keys
- Do not touch providers (any file)
- Do not touch global shortcut code
- Do not touch history service unless absolutely required (prefer not to)
- Do not create transcription output files on disk

## Approved architecture

### Invocation

Use `std::process::Command` directly. Do not use shell execution (`Command::new("sh")` or similar).

Invoke the binary with arguments in the form:

```
<whisper_binary_path> -m <model_path> -f <wav_path>
```

Capture stdout. Do not assume `--output-txt` or any file-based output flag. The raw transcript text is taken directly from stdout of the process.

### Validation before execution (required)

Before calling the binary, validate all of the following. Return `AppError::TranscriptionError` if any check fails:

1. `binary_path` is non-empty
2. `binary_path` points to an existing file (`std::path::Path::new(binary_path).is_file()`)
3. `model_path` is non-empty
4. `model_path` points to an existing file
5. `wav_path` is non-empty
6. `wav_path` points to an existing file
7. `wav_path` is inside the app audio directory (`audio_dir`), checked by canonicalize + starts_with

### Privacy enforcement (required)

Call `PrivacyService::enforce_privacy_preview` with `operation_type = "local_transcription"` before spawning the process. If it returns a denial, return `AppError::PrivacyBlocked(reason)`.

### WAV cleanup (required)

After **successful** transcription: if `audio_history_enabled` is `false`, delete the WAV file (`std::fs::remove_file`). Ignore deletion errors.

After **failed** transcription: keep the WAV so the user can retry. Do not delete on failure.

### Settings migration 002

```sql
ALTER TABLE settings ADD COLUMN whisper_binary_path TEXT DEFAULT NULL;
ALTER TABLE settings ADD COLUMN whisper_model_path TEXT DEFAULT NULL;
```

Apply in the existing migration runner in `src-tauri/src/db/migrations.rs`.

### AppSettings additions (Rust)

```rust
pub whisper_binary_path: Option<String>,
pub whisper_model_path: Option<String>,
```

### AppSettings additions (TypeScript)

```typescript
whisper_binary_path?: string | null;
whisper_model_path?: string | null;
```

### TranscriptionResult (Rust)

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub duration_ms: u64,
    pub model_path: String,
}
```

### TranscriptionResult (TypeScript)

```typescript
export interface TranscriptionResult {
  raw_text: string;
  duration_ms: number;
  model_path: string;
}
```

### transcribe_audio Tauri command (proposed)

```rust
#[tauri::command]
pub fn transcribe_audio(
    file_path: String,
    state: tauri::State<'_, AppState>,
    audio_state: tauri::State<'_, AudioState>,
) -> Result<TranscriptionResult, AppError> {
    let settings = SettingsService::new(state.db.clone()).get_settings()?;
    let binary_path = settings.whisper_binary_path.as_deref().unwrap_or("");
    let model_path  = settings.whisper_model_path.as_deref().unwrap_or("");
    let result = TranscriptionService::transcribe(
        &file_path,
        binary_path,
        model_path,
        &audio_state.audio_dir,
    )?;
    if !settings.audio_history_enabled {
        let _ = std::fs::remove_file(&file_path);
    }
    Ok(result)
}
```

### Frontend flow

```
Phase 3: stopRecording() → RecordingResult { file_path: "/path/uuid.wav", ... }
Phase 4: transcribeAudio(lastResult.file_path) → TranscriptionResult { raw_text, ... }
```

Dictation.tsx state additions:
- `transcriptResult: TranscriptionResult | null`
- `isTranscribing: boolean`

After stop, show a **Transcribe** button. On click: call `transcribeAudio(lastResult.file_path)`, set `transcriptResult`, populate the textarea `value` with `transcriptResult.raw_text`, enable Copy button.

### api.ts

```typescript
export async function transcribeAudio(filePath: string): Promise<TranscriptionResult> {
  return invoke<TranscriptionResult>("transcribe_audio", { filePath });
}
```

Note: Tauri receives Rust `file_path` as camelCase `filePath` from the frontend. This is the standard Tauri v2 argument serialization rule. The agent must not use `file_path` as the JS key.

## Allowed files for Phase 4 agent

The Phase 4 agent may read any file. The Phase 4 agent may only write to:

```
src-tauri/src/services/transcription.rs         (new)
src-tauri/src/commands/transcription.rs         (new)
src-tauri/src/commands/mod.rs                   (append pub mod transcription)
src-tauri/src/services/mod.rs                   (append pub mod transcription)
src-tauri/src/lib.rs                            (register transcribe_audio command)
src-tauri/src/errors/mod.rs                     (append TranscriptionError(String))
src-tauri/src/models/mod.rs                     (append TranscriptionResult)
src-tauri/src/db/migrations.rs                  (add migration 002)
src-tauri/src/db/repositories/settings_repo.rs  (update for new fields)
src-tauri/src/services/settings.rs              (only if needed for AppSettings compatibility)
src-tauri/Cargo.toml                            (only if a new dependency is truly required)
src-tauri/Cargo.lock                            (only if Cargo.toml changes)
src/pages/Dictation.tsx                         (Transcribe button, textarea, Copy)
src/pages/Models.tsx                            (binary path + model path inputs wired to settings)
src/lib/api.ts                                  (transcribeAudio wrapper)
src/lib/types.ts                                (TranscriptionResult, updated AppSettings)
handoff/phase-04-local-transcription.md         (post-implementation handoff)
docs/PROGRESS.md                                (only after implementation succeeds)
docs/TASK_BOARD.md                              (only after implementation succeeds)
```

## Forbidden files for Phase 4 agent

The Phase 4 agent must not edit:

```
src-tauri/src/services/audio.rs
src-tauri/src/commands/audio.rs
src-tauri/src/services/providers.rs
src-tauri/src/commands/providers.rs
src-tauri/src/services/diagnostics.rs
src-tauri/src/commands/diagnostics.rs
src-tauri/src/services/history.rs
src-tauri/src/db/repositories/history_repo.rs
src/pages/Providers.tsx
src/pages/Diagnostics.tsx
src/pages/History.tsx
src/pages/Settings.tsx                          (unless absolutely required — ask orchestrator first)
docs/PHASES.md
docs/PARALLEL_EXECUTION_PLAN.md
AGENTS.md
CLAUDE.md
SOUL.md
```

## Local-only privacy rules

- `PrivacyService::enforce_privacy_preview` must be called before invoking the whisper binary.
- The `operation_type` must be `"local_transcription"`.
- If local-only mode is enabled and the operation is a local binary call, it should still be allowed (local calls are not cloud calls). The privacy check is a safeguard, not a blocker for local-only users.
- Do not send audio to any cloud endpoint in Phase 4.
- Do not call any HTTP API in Phase 4.
- Do not implement cloud fallback in Phase 4.

## Command execution safety rules

- Use `std::process::Command::new(binary_path)` — NOT `Command::new("sh")` or `Command::new("cmd")`.
- Do not pass arguments through a shell string. Use `.arg()` or `.args()` for each argument individually.
- The binary path comes from the app's own settings table, not from user-supplied runtime input. This limits the injection surface.
- Validate that the binary path is an existing file before passing it to Command::new. Return a typed AppError if not.
- Do not pass user-typed text as arguments to the binary. Only `file_path` and `model_path` (both from settings or our own WAV directory) are passed.
- Capture stdout with `.output()` or `.stdout(Stdio::piped())`. Do not inherit the parent's stdout.
- Check the process exit code. If non-zero, return `AppError::TranscriptionError` with stderr content included.

## Tauri camelCase argument reminder

Tauri v2 deserializes frontend arguments from camelCase to snake_case automatically on the Rust side.

When the Rust command signature has `file_path: String`, the frontend must pass `{ filePath: ... }`, not `{ file_path: ... }`.

Example:

```typescript
// Correct
invoke("transcribe_audio", { filePath: lastResult.file_path })

// Wrong — will silently pass null to Rust
invoke("transcribe_audio", { file_path: lastResult.file_path })
```

The same rule applies to `device_name` → `deviceName` (already wired in Phase 3 and confirmed working).

## Merge rule

No PR from this wave may be merged unless the orchestrator (user) provides the exact line:

```
ORCHESTRATOR APPROVED MERGE
```

The agent must open a PR and stop. The agent must not merge its own PR.

## Next step

After the Phase 4 branch and worktree are created and pushed, launch the Local Transcription agent inside `C:\Users\User\Desktop\transtypro-stt` with the prompt in the next section of this file.

## Agent prompt (copy-paste for transtypro-stt session)

```
You are the transtypro Local Transcription agent (Phase 4).

Your working directory is C:\Users\User\Desktop\transtypro-stt.
Your branch is phase/04-local-transcription.

Read handoff/wave-4-launch.md before writing any code.
Read handoff/phase-03-audio-recording.md before writing any code.
Read src-tauri/src/services/audio.rs before writing any code.
Read src-tauri/src/db/migrations.rs before writing any code.
Read src-tauri/src/db/repositories/settings_repo.rs before writing any code.
Read src-tauri/src/models/mod.rs before writing any code.
Read src-tauri/src/errors/mod.rs before writing any code.
Read src-tauri/src/lib.rs before writing any code.
Read src-tauri/src/services/mod.rs before writing any code.
Read src-tauri/src/commands/mod.rs before writing any code.
Read src/pages/Dictation.tsx before writing any code.
Read src/pages/Models.tsx before writing any code.
Read src/lib/api.ts before writing any code.
Read src/lib/types.ts before writing any code.

Phase 4 goal:
Wire local whisper.cpp-compatible transcription from the WAV file produced by Phase 3 stopRecording().

Implement in this order:
1. Migration 002: ALTER TABLE settings ADD COLUMN whisper_binary_path and whisper_model_path
2. Update AppSettings struct (Rust) and settings_repo.rs
3. Add TranscriptionError(String) to AppError
4. Add TranscriptionResult to models/mod.rs
5. Create src-tauri/src/services/transcription.rs with TranscriptionService::transcribe
6. Create src-tauri/src/commands/transcription.rs with transcribe_audio command
7. Register in mod.rs files and lib.rs
8. Update AppSettings in types.ts
9. Add TranscriptionResult to types.ts
10. Add transcribeAudio() to api.ts
11. Wire Dictation.tsx: Transcribe button, textarea, Copy button
12. Wire Models.tsx: binary path and model path inputs
13. Run cargo fmt, cargo clippy, cargo test, npm run build, npm run typecheck
14. Create handoff/phase-04-local-transcription.md
15. Commit all changes on phase/04-local-transcription
16. Open a PR to main — title: "feat(stt): add local transcription (phase 4)"
17. Stop. Do not merge.

All constraints from handoff/wave-4-launch.md apply. Read that file first.
```
