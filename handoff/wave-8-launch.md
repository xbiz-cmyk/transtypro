# Wave 8 Launch Record — Privacy Enforcement, Diagnostics, Retention, and Polish

## Launch metadata

| Item | Value |
|---|---|
| Wave | 8 |
| Phase | Phase 8 — Privacy enforcement, diagnostics, retention, and polish |
| Base commit | `0fecbe7` — feat(shortcut): add global dictation overlay trigger |
| Branch | `phase/08-privacy-diagnostics` |
| Worktree | `C:\Users\User\Desktop\transtypro-privacy` |
| Launched | 2026-05-09 |
| Merge rule | No merge without the exact phrase: **ORCHESTRATOR APPROVED MERGE** |

---

## Phase 8 scope

**One sentence:** Replace all remaining static/mock placeholder data with real backend behavior, enforce the retention policy that has been stored but never executed, and polish two UX issues found during Phase 7 testing.

### Approved features

- Real `DiagnosticsService` — replaces the static two-item placeholder from Phase 2.
- Real diagnostics backend checks (database, filesystem, audio devices, whisper paths, providers, Ollama).
- Retention cleanup for old history entries (reads existing `settings.retention_days`).
- Retention cleanup for old temporary WAV files (reads existing `settings.audio_history_enabled`).
- Privacy page wired to real `get_privacy_status` command (replaces `MOCK_PRIVACY_SUMMARY`).
- Settings page wired to real `get_settings`/`update_settings` where practical.
- Manual retention cleanup action from Settings page.
- "Run diagnostics" button wired to real backend.
- Overlay indicator polish — red pulsing dot replaced with neutral/brand color.
- Better user-facing status and error messages.

### What Phase 8 is NOT

- No text insertion into active app (Phase 9).
- No clipboard paste simulation (Phase 9).
- No automatic active-app paste (Phase 9).
- No auto-start recording from shortcut press.
- No press-and-hold dictation mode.
- No packaging (Phase 10).
- No rewrite of audio recording (Phase 3 complete).
- No rewrite of Whisper transcription (Phase 4 complete).
- No rewrite of cleanup providers (Phase 5 complete).
- No rewrite of history pipeline (Phase 6 complete).
- No rewrite of shortcut logic except overlay indicator color (Phase 7 complete).
- No cloud storage.
- No account/login/sync.
- No changes to provider API key handling.
- No model downloads.
- No telemetry.
- No sending diagnostics data anywhere.
- **No custom shortcut rebinding — deferred to Phase 9.**

---

## Custom shortcut rebinding — deferred

Custom shortcut rebinding is **Phase 9**, not Phase 8.

Additionally: **Fn-only shortcuts are unsupported and unreliable.** Many keyboards handle the Fn key at the hardware or firmware level before the OS sees it. `tauri-plugin-global-shortcut` cannot reliably intercept Fn-only key combinations. Document this clearly in Settings if the topic comes up. The supported default shortcut is `CommandOrControl+Shift+Space` (Windows: `Ctrl+Shift+Space`, macOS: `Cmd+Shift+Space`).

The Settings page currently shows:
```
Shortcut rebinding coming in a future release.
```
Leave this text as-is. Do not implement rebinding. Do not change the displayed shortcut string.

---

## Architecture summary

### No new database migration

All required fields already exist:

| Field | Table | Migration | Status |
|---|---|---|---|
| `retention_days` | `settings` | 001 | ✓ exists |
| `audio_history_enabled` | `settings` | 001 | ✓ exists |
| `history.timestamp` | `history` | 001, ISO 8601 | ✓ exists |
| WAV file age | filesystem `mtime` | n/a | ✓ no DB needed |

Do not add a migration. The `schema_migrations` table has versions 1, 2, 3 applied. Leave it at 3.

### New backend services

#### `RetentionService` (new file: `src-tauri/src/services/retention.rs`)

```rust
pub struct RetentionService {
    db: Arc<Mutex<Connection>>,
    audio_dir: PathBuf,
}

pub struct RetentionResult {
    pub deleted_history_count: u32,
    pub deleted_wav_count: u32,
}

impl RetentionService {
    pub fn new(db: Arc<Mutex<Connection>>, audio_dir: PathBuf) -> Self
    pub fn apply_history_retention(&self) -> Result<u32, AppError>
    pub fn apply_audio_retention(&self) -> Result<u32, AppError>
    pub fn apply_all(&self) -> Result<RetentionResult, AppError>
}
```

Add `RetentionResult` to `src-tauri/src/models/mod.rs`.

#### `DiagnosticsService` (rewrite: `src-tauri/src/services/diagnostics.rs`)

Replace the static placeholder with a real service that takes DB and audio_dir:

```rust
pub struct DiagnosticsService {
    db: Arc<Mutex<Connection>>,
    audio_dir: PathBuf,
}

impl DiagnosticsService {
    pub fn new(db: Arc<Mutex<Connection>>, audio_dir: PathBuf) -> Self
    pub fn run_diagnostics(&self) -> Result<DiagnosticReport, AppError>
}
```

### New Tauri commands

#### `run_diagnostics` (replaces `run_diagnostics_placeholder`)

```rust
#[tauri::command]
pub fn run_diagnostics(
    state: State<'_, AppState>,
    audio_state: State<'_, AudioState>,
) -> Result<DiagnosticReport, AppError>
```

#### `apply_retention_policy` (new command)

```rust
#[tauri::command]
pub fn apply_retention_policy(
    state: State<'_, AppState>,
    audio_state: State<'_, AudioState>,
) -> Result<RetentionResult, AppError>
```

### Updated `lib.rs` invoke handler

- Remove `commands::diagnostics::run_diagnostics_placeholder`
- Add `commands::diagnostics::run_diagnostics`
- Add `commands::retention::apply_retention_policy`

### New method on `HistoryRepository`

Add to `src-tauri/src/db/repositories/history_repo.rs`:

```rust
/// Deletes history entries older than `days` days.
/// Returns the number of rows deleted.
/// If days == 0, returns 0 immediately (keep forever).
pub fn delete_older_than(&self, days: u32) -> Result<u32, AppError> {
    if days == 0 {
        return Ok(0);
    }
    let rows = self.conn.execute(
        "DELETE FROM history WHERE datetime(timestamp) < datetime('now', '-' || ?1 || ' days')",
        params![days],
    ).map_err(|e| AppError::StorageError(e.to_string()))?;
    Ok(rows as u32)
}
```

---

## Diagnostics checks

`run_diagnostics` executes all checks sequentially. Failures do not abort later checks. Status values: `"pass"`, `"fail"`, `"warn"`, `"skip"`.

| Check name | Pass condition | Fail/warn/skip | Source |
|---|---|---|---|
| `backend_alive` | Always pass | — | Always pass |
| `database_reachable` | `SELECT 1 FROM settings WHERE id=1` succeeds | `fail` on error | DB query |
| `migrations_current` | `schema_migrations` has version 3 | `warn` if missing | DB query |
| `microphone_available` | `cpal::default_host().input_devices()` returns ≥1 device | `warn` if 0 or error | cpal (existing dep) |
| `whisper_binary_configured` | `settings.whisper_binary_path.is_some()` | `warn` if None | Settings row |
| `whisper_binary_exists` | `Path::new(path).is_file()` | `fail` if missing | `std::fs`, skip if unconfigured |
| `whisper_model_configured` | `settings.whisper_model_path.is_some()` | `warn` if None | Settings row |
| `whisper_model_exists` | `Path::new(path).is_file()` | `fail` if missing | `std::fs`, skip if unconfigured |
| `providers_configured` | `SELECT COUNT(*) FROM providers WHERE enabled=1` > 0 | `warn` if 0 | DB query |
| `ollama_reachable` | `GET {base_url}/api/tags` returns 200 within 3s | `warn` on timeout/error | ureq (existing dep), skip if no Ollama provider |
| `shortcut_configured` | Shortcut string parses without error | `fail` if parse error | `tauri_plugin_global_shortcut::Shortcut::from_str` |
| `audio_dir_accessible` | `audio_dir` exists and is a directory | `warn` if missing | `std::fs` |
| `history_count` | Always pass; message shows count | — | `SELECT COUNT(*) FROM history` |
| `audio_dir_size` | Always pass; message shows file count + total bytes | — | `std::fs::read_dir` |

**Important diagnostics rules:**
- Never read API keys from the OS keychain for diagnostics.
- Never log secrets of any kind.
- `ollama_reachable` only calls `GET {base_url}/api/tags` — no auth, no data sent.
- `generated_at` is populated from `Utc::now().to_rfc3339()`.
- The diagnostics report is never sent anywhere — it is returned to the frontend in the Tauri IPC response only.

---

## Retention safety rules

These rules are mandatory and must be enforced in code, not just by convention.

### History retention rules

- Read `settings.retention_days` from DB.
- If `retention_days == 0`: return `Ok(0)` immediately — keep forever.
- If `retention_days > 0`: execute `DELETE FROM history WHERE datetime(timestamp) < datetime('now', '-N days')`.
- Return count of rows deleted.

### WAV file retention rules

**When `audio_history_enabled = false` (default):**
- Delete ALL regular `.wav` files in `audio_dir`.
- Rationale: audio history is disabled; any WAV is a leftover from a crash or cancel. Phase 3's `TranscriptionService` already deletes WAVs after transcription, but crashes/cancels may leave stragglers.

**When `audio_history_enabled = true` and `retention_days > 0`:**
- Delete `.wav` files in `audio_dir` with `mtime` older than `retention_days` days.
- Use `std::fs::metadata(path).modified()` for age check.

**When `audio_history_enabled = true` and `retention_days = 0`:**
- Return `Ok(0)` — keep all audio files forever.

**Safety rules enforced in code before every file deletion:**
1. `assert!(path.starts_with(&self.audio_dir))` — if false, skip file, log warning, never delete.
2. Only delete regular files — `metadata.is_file()` must be true; skip directories and symlinks.
3. Only delete files with extension `.wav` — skip any other file type.
4. Errors on individual files (permission denied, already deleted) are non-fatal — log and continue.
5. `audio_dir` is `data_dir.join("audio")` from Tauri setup — the app owns this directory.

### Startup auto-sweep

Automatic startup cleanup (calling `RetentionService::apply_all()` in `lib.rs` setup) is **only added if the implementation clearly satisfies all five safety rules above**. If there is any doubt about safety during implementation, **do not add startup cleanup**. The manual cleanup button in Settings is sufficient for Phase 8. The startup sweep is a nice-to-have, not a requirement.

---

## Frontend wiring plan

### `src/pages/Privacy.tsx`

- Remove `MOCK_PRIVACY_SUMMARY` constant.
- Add `useEffect` → call `getPrivacyStatus()` on mount.
- Add loading and error states.
- Replace hardcoded `summary` with real `PrivacySummary` from backend.
- Remove `"Retention enforcement — Phase 8"` placeholder text from Retention card.
- Keep `DATA_FLOW_ITEMS` as static content (describes architecture, not runtime state).

### `src/pages/Diagnostics.tsx`

- Remove `MOCK_CHECK_ITEMS` constant.
- Enable "Run diagnostics" button — calls `runDiagnostics()`.
- Show loading state during call.
- On success: display real `DiagnosticReport` checks and `generated_at`.
- On failure: show `ErrorMessage`.
- Add `"skip"` status → `"muted"` badge variant.
- Initial state (before first click): empty state — remove stale Phase 8 reference.
- "Export" button stays disabled with tooltip `"Diagnostics export — not yet implemented"`.

### `src/pages/Settings.tsx`

- Add `useEffect` → call `getSettings()` on mount, populate all state variables.
- Enable "Save settings" button → calls `updateSettings()`.
- Remove stale `"Theme switching — Phase 2"` helper text.
- Remove stale `"Real path available — Phase 2"` helper text.
- Replace hardcoded fake database path with `"Stored in your OS app data directory"`.
- Enable "Clear all history" button → calls `clearHistory()` with `window.confirm` guard.
- Remove stale `"Phase 2"` title attribute from Clear history button.
- Add "Storage cleanup" section with "Run retention cleanup" button → calls `applyRetentionPolicy()`.
- Show result after cleanup: `"Deleted X history entries and Y audio files."`.
- Language and default-mode selects may remain cosmetic local state (no direct AppSettings mapping).
- Do NOT implement shortcut rebinding. Leave shortcut display and "Shortcut rebinding coming in a future release." text unchanged.

### `src/components/FloatingOverlay.tsx`

- Change `bg-(--color-status-error)` to `bg-(--color-brand-300)` (or equivalent brand/info color) for both the ping ring and the solid dot.
- Rationale: the red pulsing indicator falsely communicates "recording active." The shortcut only navigates; no recording starts from the overlay.
- Keep `animate-ping` animation — it signals the shortcut was received.
- All other overlay behavior from Phase 7 is correct; do not change it.

### `src/lib/api.ts` — new wrappers

```ts
export async function getPrivacyStatus(): Promise<PrivacySummary> {
  return invoke<PrivacySummary>("get_privacy_status");
}

export async function runDiagnostics(): Promise<DiagnosticReport> {
  return invoke<DiagnosticReport>("run_diagnostics");
}

export async function applyRetentionPolicy(): Promise<RetentionResult> {
  return invoke<RetentionResult>("apply_retention_policy");
}
```

### `src/lib/types.ts` — new interface

```ts
export interface RetentionResult {
  deleted_history_count: number;
  deleted_wav_count: number;
}
```

---

## Local-first privacy rules

- Diagnostics data is **never sent anywhere**. It is returned as a Tauri IPC response to the local frontend only.
- No API keys are read, exposed, or logged by any Phase 8 code.
- The Ollama reachability check only calls `GET {base_url}/api/tags` — no credentials, no data sent.
- Retention sweep deletes files only; it does not read, copy, or transmit their contents.
- `get_privacy_status` is read-only — no mutations.
- `apply_retention_policy` deletes local data only — no network calls.
- If `local_only_mode = true`, no new network calls are introduced by Phase 8 code paths (diagnostics Ollama check is local network only; all other checks are filesystem/DB).

---

## How Phase 8 connects to prior phases

| Prior phase | Connection |
|---|---|
| Phase 2 (storage) | `retention_days` and `audio_history_enabled` fields finally read for enforcement. `HistoryRepository` gains `delete_older_than`. |
| Phase 3 (audio) | `AudioState.audio_dir` is the safe boundary for WAV cleanup. Audio recording logic untouched. |
| Phase 4 (transcription) | `whisper_binary_path` and `whisper_model_path` feed diagnostics checks. |
| Phase 5 (cleanup) | Provider list feeds `providers_configured` and `ollama_reachable` checks. `ureq` already a dep. |
| Phase 6 (history) | History entries swept by retention. |
| Phase 7 (shortcut) | `shortcut_configured` diagnostic check. Overlay color fixed per Phase 7 test observation. |

---

## Files the Phase 8 agent may edit

```
src-tauri/src/services/diagnostics.rs          ← rewrite with real checks
src-tauri/src/services/retention.rs            ← NEW
src-tauri/src/services/mod.rs                  ← export RetentionService
src-tauri/src/commands/diagnostics.rs          ← replace placeholder command
src-tauri/src/commands/retention.rs            ← NEW
src-tauri/src/db/repositories/history_repo.rs  ← add delete_older_than method
src-tauri/src/models/mod.rs                    ← add RetentionResult struct
src-tauri/src/lib.rs                           ← register new commands
src/lib/api.ts                                 ← add 3 new wrappers
src/lib/types.ts                               ← add RetentionResult interface
src/pages/Privacy.tsx                          ← replace mock with real backend
src/pages/Diagnostics.tsx                      ← replace mock with real backend
src/pages/Settings.tsx                         ← wire to backend settings
src/components/FloatingOverlay.tsx             ← change red indicator color
handoff/phase-08-privacy-diagnostics.md        ← required handoff file
docs/PROGRESS.md                               ← only after implementation succeeds
docs/TASK_BOARD.md                             ← only after implementation succeeds
```

---

## Files the Phase 8 agent must NOT edit

```
src-tauri/src/services/audio.rs                ← Phase 3 — do not touch
src-tauri/src/commands/audio.rs                ← Phase 3 — do not touch
src-tauri/src/services/transcription.rs        ← Phase 4 — do not touch
src-tauri/src/commands/transcription.rs        ← Phase 4 — do not touch
src-tauri/src/services/cleanup.rs              ← Phase 5 — do not touch
src-tauri/src/commands/cleanup.rs              ← Phase 5 — do not touch
src-tauri/src/services/providers.rs            ← Phase 5 — do not touch
src-tauri/src/commands/providers.rs            ← Phase 5 — do not touch
src-tauri/src/services/privacy.rs              ← already real; do not touch
src-tauri/src/commands/privacy.rs              ← already real; do not touch
src-tauri/src/services/history.rs              ← do not touch; retention is a new service
src-tauri/src/commands/history.rs              ← do not touch
src-tauri/src/commands/mod.rs                  ← do not touch (get_status_summary lives here)
src-tauri/src/db/migrations.rs                 ← no new migration; do not touch
src-tauri/src/db/repositories/settings_repo.rs ← do not touch
src-tauri/src/db/repositories/modes_repo.rs    ← do not touch
src-tauri/src/db/repositories/vocabulary_repo.rs ← do not touch
src-tauri/src/db/repositories/providers_repo.rs  ← do not touch
src-tauri/Cargo.toml                           ← no new dependencies; do not touch
src-tauri/Cargo.lock                           ← do not modify directly
src/stores/uiStore.ts                          ← do not touch
src/components/ShortcutHandler.tsx             ← do not touch
src/App.tsx                                    ← do not touch
src/pages/Dictation.tsx                        ← do not touch
src/pages/History.tsx                          ← do not touch
src/pages/Providers.tsx                        ← do not touch
src/pages/Models.tsx                           ← do not touch
src/pages/Home.tsx                             ← do not touch
docs/PHASES.md                                 ← orchestrator only
docs/PARALLEL_EXECUTION_PLAN.md                ← orchestrator only
docs/ARCHITECTURE.md                           ← orchestrator only
AGENTS.md                                      ← orchestrator only
CLAUDE.md                                      ← orchestrator only
SOUL.md                                        ← orchestrator only
```

---

## Implementation reminders for the Phase 8 agent

- Use existing `settings.retention_days`. Do not add a new settings field.
- Use existing `settings.audio_history_enabled`. Do not add a new settings field.
- Use existing `history.timestamp` (ISO 8601). Do not add a new column.
- Do not add a migration. Schema is complete at version 3.
- Do not change provider API key handling. Never read API keys from keyring for diagnostics.
- Do not send diagnostics data anywhere. IPC response only.
- Do not log secrets of any kind.
- Ollama diagnostics check: `GET {base_url}/api/tags` only. No auth required.
- Retention must never delete outside `app_audio_dir`. Enforce with `path.starts_with(&audio_dir)`.
- Retention must only delete regular `.wav` files. Check `metadata.is_file()` and extension.
- `retention_days = 0` means keep forever. Guard before any delete SQL or file delete.
- Overlay indicator must not be red — shortcut does not start recording.
- **Custom shortcut rebinding is Phase 9. Do not implement it.**
- **Fn-only shortcut is unsupported/unreliable. Document as such if the topic arises.**
- No new Cargo dependencies needed (`ureq`, `cpal`, `chrono` are already in Cargo.toml).
- Startup auto-sweep only if all five safety rules are clearly satisfied; otherwise manual-only is sufficient.

---

## Merge rule

No PR for `phase/08-privacy-diagnostics` may be merged without the orchestrator sending the exact phrase:

```
ORCHESTRATOR APPROVED MERGE
```

---

## Verification commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run lint
npm run build
pwsh scripts/quality-check.ps1
```

All 100 existing Rust tests must pass. Minimum 8 new tests targeting:
- `HistoryRepository::delete_older_than` (at least 2 tests)
- `RetentionService` (at least 3 tests)
- `DiagnosticsService` (at least 3 tests)

Target: 108+ tests total.

---

## Next step

Open a new agent session in `C:\Users\User\Desktop\transtypro-privacy`.

The agent reads this file before writing any code, then also reads:
- `handoff/phase-07-global-shortcut-overlay.md`
- `src-tauri/src/services/diagnostics.rs`
- `src-tauri/src/services/privacy.rs`
- `src-tauri/src/commands/diagnostics.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/db/repositories/history_repo.rs`
- `src-tauri/src/db/repositories/settings_repo.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/lib.rs`
- `src/pages/Privacy.tsx`
- `src/pages/Diagnostics.tsx`
- `src/pages/Settings.tsx`
- `src/components/FloatingOverlay.tsx`
- `src/lib/api.ts`
- `src/lib/types.ts`

The agent implements Phase 8 in this order:
1. Add `RetentionResult` to `models/mod.rs`
2. Add `delete_older_than` to `history_repo.rs`
3. Create `services/retention.rs`
4. Rewrite `services/diagnostics.rs`
5. Update `services/mod.rs`
6. Replace `commands/diagnostics.rs`
7. Create `commands/retention.rs`
8. Update `lib.rs` (register new commands; optionally startup sweep if clearly safe)
9. Run `cargo test` — all pass
10. Add API wrappers to `api.ts`
11. Add `RetentionResult` to `types.ts`
12. Wire `Privacy.tsx`
13. Wire `Diagnostics.tsx`
14. Wire `Settings.tsx`
15. Fix overlay indicator color in `FloatingOverlay.tsx`
16. Run all verification commands — all pass
17. Create `handoff/phase-08-privacy-diagnostics.md`
18. Open PR against `main`

Orchestrator reviews and merges with `ORCHESTRATOR APPROVED MERGE`.

---

## Agent prompt for the transtypro-privacy session

```
You are the transtypro Phase 8 agent.

Your working directory is: C:\Users\User\Desktop\transtypro-privacy
Your branch is: phase/08-privacy-diagnostics

Read these files before writing any code:
- handoff/wave-8-launch.md              ← this file; your full specification
- handoff/phase-07-global-shortcut-overlay.md
- src-tauri/src/services/diagnostics.rs
- src-tauri/src/services/privacy.rs
- src-tauri/src/commands/diagnostics.rs
- src-tauri/src/services/mod.rs
- src-tauri/src/db/repositories/history_repo.rs
- src-tauri/src/db/repositories/settings_repo.rs
- src-tauri/src/db/migrations.rs
- src-tauri/src/models/mod.rs
- src-tauri/src/lib.rs
- src/pages/Privacy.tsx
- src/pages/Diagnostics.tsx
- src/pages/Settings.tsx
- src/components/FloatingOverlay.tsx
- src/lib/api.ts
- src/lib/types.ts

Your task:
Implement Phase 8 — Privacy enforcement, diagnostics, retention, and polish.

Phase 8 scope (exactly):
1. Add RetentionResult to src-tauri/src/models/mod.rs
2. Add delete_older_than to src-tauri/src/db/repositories/history_repo.rs
3. Create src-tauri/src/services/retention.rs (RetentionService)
4. Rewrite src-tauri/src/services/diagnostics.rs (real checks, not static)
5. Update src-tauri/src/services/mod.rs (export RetentionService)
6. Replace src-tauri/src/commands/diagnostics.rs (run_diagnostics, not placeholder)
7. Create src-tauri/src/commands/retention.rs (apply_retention_policy)
8. Update src-tauri/src/lib.rs (register new commands, remove placeholder)
9. Add getPrivacyStatus, runDiagnostics, applyRetentionPolicy to src/lib/api.ts
10. Add RetentionResult to src/lib/types.ts
11. Wire src/pages/Privacy.tsx to real getPrivacyStatus()
12. Wire src/pages/Diagnostics.tsx to real runDiagnostics()
13. Wire src/pages/Settings.tsx to real getSettings()/updateSettings()
14. Fix src/components/FloatingOverlay.tsx overlay indicator color (not red)

Run all checks before creating PR:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test (all 100 existing + 8+ new must pass)
- npm run lint
- npm run build
- pwsh scripts/quality-check.ps1

Create handoff/phase-08-privacy-diagnostics.md with:
- what changed
- test results
- known limitations
- next recommended task

Then create a PR against main.

Constraints (read wave-8-launch.md for the full list):
- No new Cargo dependencies (ureq, cpal, chrono already available).
- No new DB migration.
- Do not touch Phase 3/4/5/6/7 service or command files.
- Do not implement shortcut rebinding.
- Do not send diagnostics data anywhere.
- Never read API keys from keyring for diagnostics.
- Retention must never delete outside audio_dir.
- Retention only deletes regular .wav files.
- retention_days = 0 means keep forever.
- Overlay indicator must not be red.
- All 100 existing Rust tests must pass.
```
