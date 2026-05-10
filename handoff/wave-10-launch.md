# Wave 10 Launch: Push-to-Talk Auto Dictation Pipeline

**Wave:** 10
**Phase:** 10 — Push-to-Talk Auto Dictation Pipeline
**Branch:** `phase/10-ptt-pipeline`
**Worktree:** `C:\Users\User\Desktop\transtypro-ptt`
**Base commit:** `4f4a4be` — feat(insertion): add text insertion and shortcut rebinding
**Launched:** 2026-05-10
**Agent:** transtypro-rust-backend + transtypro-frontend-ui (single agent, single PR)

---

## Phase 10 Scope

Push-to-talk auto dictation pipeline. When the user holds or presses the configured global shortcut, transtypro starts recording. When the shortcut is released (or pressed again in toggle mode), transtypro stops recording, transcribes the audio locally, optionally cleans the text via the configured provider, inserts the final text into the previously active application, and saves a history entry.

No live transcription. No live insertion. No partial transcript preview. Final insert only, after pipeline completes.

---

## Approved Architecture Summary

### Core pipeline (backend-orchestrated)

A new `PttPipelineService` in `services/ptt.rs` chains existing services in sequence:

```
AudioService::stop_recording()
  → TranscriptionService::transcribe()
  → CleanupService::cleanup() [optional, skipped if no provider or local-only blocks it]
  → InsertionService::insert_text() [called directly — NOT via Tauri command]
  → HistoryService::create_history_entry()
  → HistoryService::mark_inserted() [only if insert succeeded]
```

Each step emits a `ptt-status` Tauri event to the frontend. The pipeline runs on a dedicated spawned thread so it never blocks the shortcut plugin's callback thread.

### Tauri-managed state

A new `PttState` struct is added and managed via `app.manage()` in `lib.rs`:

```rust
pub struct PttState {
    pub phase: Arc<Mutex<PttPhase>>,
    pub cancel_flag: Arc<AtomicBool>,
}

pub enum PttPhase {
    Idle,
    Recording,
    Transcribing,
    Cleaning,
    Inserting,
    Done,
    Error(String),
}
```

### New Tauri event

`"ptt-status"` carries `PttStatusEvent { phase: String, message: String }`.

Phase values: `"recording"`, `"transcribing"`, `"cleaning"`, `"inserting"`, `"done"`, `"error"`, `"idle"`, `"cancelled"`.

**The `message` field must NEVER contain transcript text, clipboard contents, or any user-dictated content.** Status strings only: "Recording…", "Transcription failed: binary not found", "Done."

### New Tauri command

`cancel_ptt()` — sets `cancel_flag`, resets phase to Idle, emits `{ phase: "idle", message: "Cancelled." }`.

---

## Desired Push-and-Hold Behavior

**Push-and-hold is the desired product behavior.**

The user presses and holds the configured shortcut → recording starts immediately.
The user releases the shortcut → recording stops, pipeline runs, text is inserted.

This is `push_to_talk_hold` mode.

---

## Released-Event Verification Requirement

**The Phase 10 agent must verify whether `ShortcutState::Released` events fire reliably on Windows before implementing the hold behavior.**

The `tauri-plugin-global-shortcut` plugin exposes both `ShortcutState::Pressed` and `ShortcutState::Released`. On Windows, global shortcuts registered via `RegisterHotKey` only fire a press message (`WM_HOTKEY`). If the plugin uses `WH_KEYBOARD_LL` hooks internally, Released events will work. This must be tested by the agent, not assumed.

### Verification procedure

Add a temporary `eprintln!` branch in the shortcut handler for the `Released` state, run the app in dev mode, press and release the shortcut, and observe whether the Released branch fires.

### If Released events work reliably

Implement `push_to_talk_hold`:

- `ShortcutState::Pressed` → start recording (do not steal focus)
- `ShortcutState::Released` → stop recording, spawn pipeline thread

### If Released events do NOT work reliably on Windows

Implement `push_to_talk_toggle` as the fallback:

- `ShortcutState::Pressed` (phase is `Idle`) → start recording
- `ShortcutState::Pressed` (phase is `Recording`) → stop recording, spawn pipeline thread

Document in `handoff/phase-10-ptt-pipeline.md` which mode was implemented and why.

If only one PTT mode is implemented, still document why the other was deferred. Both values (`push_to_talk_hold` and `push_to_talk_toggle`) must exist in `settings.shortcut_behavior` even if only one is currently functional, so the Setting UI and DB are forward-compatible.

---

## Toggle Fallback Rule

If Released events are confirmed non-functional on Windows, the agent implements toggle mode (`push_to_talk_toggle`) and the Settings selector shows both options. The `push_to_talk_hold` option is shown but disabled or annotated as "Not available on this platform" if Released events are known to not work. The DB column stores whichever value the user selected; the runtime handler checks and falls back gracefully.

---

## Shortcut Behavior Values

Migration 005 adds:

```sql
ALTER TABLE settings ADD COLUMN shortcut_behavior TEXT NOT NULL DEFAULT 'open_dictation'
```

Valid values stored in DB:

| Value | Behavior |
|-------|---------|
| `open_dictation` | Pressed → unminimize + show + focus transtypro + emit `dictation-shortcut-pressed` + navigate to Dictation (current Phase 7 behavior, unchanged) |
| `push_to_talk_hold` | Pressed → start recording (no focus steal); Released → stop + run pipeline |
| `push_to_talk_toggle` | Pressed (Idle) → start recording; Pressed (Recording) → stop + run pipeline |

The Settings page shows a selector with all three options. If the agent determines that `push_to_talk_hold` is unavailable on Windows, annotate it in the UI.

---

## Migration 005 Decision

File: `src-tauri/src/db/migrations.rs`

```rust
const MIGRATION_005: &str =
    "ALTER TABLE settings ADD COLUMN shortcut_behavior TEXT NOT NULL DEFAULT 'open_dictation'";
```

Add to the `MIGRATIONS` array as entry `(5, MIGRATION_005)`.

Update `SettingsRepository` (`settings_repo.rs`) to include `shortcut_behavior` in:
- The `SELECT` query (column index 9, after `shortcut`)
- The `INSERT OR REPLACE` / `ON CONFLICT DO UPDATE` upsert
- The `AppSettings` struct default value (`"open_dictation".to_string()`)

Update `AppSettings` in `models/mod.rs` to add:

```rust
pub shortcut_behavior: String,
```

Update `AppSettings` in `src/lib/types.ts` to add:

```ts
shortcut_behavior: string;
```

---

## Diagnostics Shortcut Fix Requirement

File: `src-tauri/src/services/diagnostics.rs`

**Problem:** `check_shortcut_configured()` currently hardcodes `"CommandOrControl+Shift+Space"` as the expected shortcut string. If the user rebinds the shortcut, the check incorrectly validates the old default.

**Fix:** Read the configured shortcut from the DB via `SettingsRepository`. Attempt to parse the string using `tauri_plugin_global_shortcut::Shortcut`. If parse succeeds → `"pass"` with message showing the key string. If parse fails or string is empty → `"warn"`.

The shortcut string (key name, e.g., `"CommandOrControl+Shift+D"`) is safe to include in diagnostic output — it is not a secret.

---

## Diagnostics Migration Version Fix Requirement

File: `src-tauri/src/services/diagnostics.rs`

**Problem:** `check_migrations_current()` hardcodes `WHERE version = 3`. Phase 9 added migration 004. Phase 10 adds migration 005. The check now incorrectly passes even when migrations 4 and 5 are missing.

**Fix:** Query `SELECT MAX(version) FROM schema_migrations` and compare against the constant `5` (the highest migration version after Phase 10). If `MAX(version) >= 5` → `"pass"`. Otherwise → `"warn"` with a message showing current vs expected version.

---

## PTT Status Event Requirement

The backend must emit `"ptt-status"` events at each pipeline phase transition. The frontend must listen for these events in `ShortcutHandler.tsx` and update `uiStore` state. The `FloatingOverlay` must display the current PTT phase if the window is already visible.

Event schema:

```ts
export interface PttStatusEvent {
  phase: "recording" | "transcribing" | "cleaning" | "inserting" | "done" | "error" | "idle" | "cancelled";
  message: string;
}
```

Phase display in overlay:

| Phase | Indicator | Text |
|-------|-----------|------|
| `recording` | Red pulse | "Recording…" |
| `transcribing` | Brand blue pulse | "Transcribing…" |
| `cleaning` | Brand blue pulse | "Cleaning text…" |
| `inserting` | Brand blue pulse | "Inserting…" |
| `done` | Green (no pulse, 2s then hide) | "Done." |
| `error` | Red (stays until dismissed) | Error message |
| `idle` / `cancelled` | (overlay hides) | — |

A Cancel button must be visible in the overlay when phase is `recording`, `transcribing`, or `cleaning`. It calls `cancelPtt()`.

After `done`, the overlay auto-hides after 2 seconds via `setTimeout` in `ShortcutHandler.tsx`.

---

## Focus Safety Rules

**In `open_dictation` mode (unchanged from Phase 7):**
- On shortcut `Pressed`: unminimize window, show window, set_focus, emit `"dictation-shortcut-pressed"`
- Frontend navigates to Dictation page

**In `push_to_talk_hold` or `push_to_talk_toggle` mode:**
- On shortcut `Pressed` (start): do NOT call `window.unminimize()`, `window.show()`, or `window.set_focus()`
- Do NOT emit `"dictation-shortcut-pressed"`
- Only emit `"ptt-status"` with `{ phase: "recording", message: "Recording…" }`
- Start audio recording via `AudioService::start_recording()`
- The target application retains focus throughout recording

**Insertion in PTT mode:**
- `PttPipelineService` calls `InsertionService::insert_text(final_text)` **directly** — NOT via the `insert_text` Tauri command
- The Tauri command (`commands/insertion.rs`) minimizes and restores the transtypro window; this is wrong for PTT (window is already not focused, and the target app already has focus)
- By calling the service directly, the PTT pipeline skips the minimize/restore wrapper
- The 300ms sleep in the Tauri command is unnecessary when PTT is used; the service's 150ms post-paste sleep is still applied by `InsertionService::insert_text()`

**After PTT pipeline completes:**
- If error: bring transtypro to front so user sees the error (`window.unminimize()` + `window.show()` + `window.set_focus()` in the pipeline's error path)
- If success: do NOT bring transtypro to front; leave the user in their active application

---

## Active App Title Capture: Optional / Deferred Rule

Active app title capture is **optional for Phase 10**. Only implement it if the implementation is clearly safe and small (fewer than ~20 lines of platform-specific Rust, no new crate dependency, no unsafe FFI beyond `extern "system"` declarations for `GetForegroundWindow` and `GetWindowTextW` on Windows).

**Rules if implemented:**
- Capture window title ONLY (title bar string, e.g., "Document1 - Notepad")
- Do NOT read window contents
- Do NOT capture the screen
- Do NOT use OCR
- Do NOT store the title in the history entry (no DB change)
- Do NOT log the title (no `eprintln!` of the title)
- Use ONLY for temporary UI display: overlay message "Recording… (Notepad)"
- Discard immediately after overlay message is sent

**If there is any uncertainty about safety, OS behavior, or implementation complexity, defer to Phase 11.** The pipeline must work correctly without it. Title capture is a display nicety only.

---

## Clipboard and Insertion Safety Rules

- `PttPipelineService` calls `InsertionService::insert_text(final_text)` directly (bypassing the Tauri command wrapper)
- `final_text` is NEVER logged at any point in the PTT pipeline
- Clipboard save/restore behavior is controlled entirely by `InsertionService` via the `clipboard_restore_enabled` setting
- `PttStatusEvent.message` must NEVER contain transcript text, clipboard contents, or any user content
- No audio, text, clipboard contents, or diagnostics are sent anywhere external
- The `PttStatusEvent` is delivered to the local frontend webview only

---

## Local-First Privacy Rules

- All pipeline steps run locally unless a cloud cleanup provider is configured AND `local_only_mode = false`
- `CleanupService` already enforces privacy via `PrivacyService::enforce_privacy_preview()` before any HTTP call
- PTT uses `CleanupService` as-is; privacy enforcement is automatic
- If `local_only_mode = true` and the cleanup provider is cloud-based, cleanup is silently skipped and the raw transcript is used
- No new network calls are introduced in Phase 10
- No telemetry is added

---

## Live Transcription: Out of Scope

Live transcription (streaming whisper output while recording is in progress) is explicitly out of scope for Phase 10. Do not implement it. Do not add partial transcript preview. The final transcript is produced after recording stops and the full WAV file is transcribed.

---

## Live Insertion: Out of Scope

Live insertion (inserting partial transcript text into the active app while still recording) is explicitly out of scope for Phase 10. Do not implement it.

---

## Allowed Files for the Phase 10 Agent

The Phase 10 agent working in `phase/10-ptt-pipeline` may ONLY create or edit these files:

**New files to create:**

- `src-tauri/src/services/ptt.rs`
- `src-tauri/src/commands/ptt.rs`
- `handoff/phase-10-ptt-pipeline.md`

**Existing files to modify:**

- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/mod.rs` — **MINIMAL EDIT ONLY**: add `pub mod ptt;` — no other changes
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/settings_repo.rs`
- `src-tauri/src/services/diagnostics.rs`
- `src/stores/uiStore.ts`
- `src/components/ShortcutHandler.tsx`
- `src/components/FloatingOverlay.tsx`
- `src/pages/Settings.tsx`
- `src/lib/types.ts`
- `src/lib/api.ts`
- `docs/PROGRESS.md` — only after implementation succeeds and all checks pass
- `docs/TASK_BOARD.md` — only after implementation succeeds and all checks pass

---

## Forbidden Files for the Phase 10 Agent

The following files must NOT be touched under any circumstances:

**Backend — do not touch:**
- `src-tauri/src/services/audio.rs`
- `src-tauri/src/commands/audio.rs`
- `src-tauri/src/services/transcription.rs`
- `src-tauri/src/commands/transcription.rs`
- `src-tauri/src/services/cleanup.rs`
- `src-tauri/src/commands/cleanup.rs`
- `src-tauri/src/services/insertion.rs`
- `src-tauri/src/commands/insertion.rs`
- `src-tauri/src/commands/shortcut.rs`
- `src-tauri/src/services/history.rs`
- `src-tauri/src/commands/history.rs`
- `src-tauri/src/services/providers.rs`
- `src-tauri/src/commands/providers.rs`
- `src-tauri/src/services/privacy.rs`
- `src-tauri/src/commands/privacy.rs`
- `src-tauri/src/services/retention.rs`
- `src-tauri/src/commands/diagnostics.rs`
- `src-tauri/src/db/repositories/history_repo.rs`
- `src-tauri/src/db/repositories/modes_repo.rs`
- `src-tauri/src/db/repositories/vocabulary_repo.rs`
- `src-tauri/src/db/repositories/providers_repo.rs`
- `src-tauri/src/db/connection.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`

**Frontend — do not touch:**
- `src/pages/Dictation.tsx`
- `src/pages/History.tsx`
- `src/pages/Privacy.tsx`
- `src/pages/Diagnostics.tsx`
- `src/pages/Providers.tsx`
- `src/pages/Models.tsx`
- `src/pages/Home.tsx`
- `src/App.tsx`

**Docs and config — do not touch:**
- `docs/PHASES.md`
- `docs/PARALLEL_EXECUTION_PLAN.md`
- `docs/ARCHITECTURE.md`
- `AGENTS.md`
- `CLAUDE.md`
- `SOUL.md`

---

## Implementation Reminders for the Phase 10 Agent

- Call `AudioService` existing methods — do not modify `AudioService` internals
- Call `TranscriptionService` existing methods — do not modify `TranscriptionService` internals
- Call `CleanupService` existing methods — do not modify `CleanupService` internals
- Call `InsertionService::insert_text()` directly from `PttPipelineService` — NOT via the `insert_text` Tauri command (which adds window minimize/restore that is wrong for PTT)
- Call `HistoryService` existing methods — do not modify `HistoryService` internals
- Do NOT add Cargo dependencies unless absolutely required (no new crates for PTT)
- Do NOT add telemetry
- Do NOT log dictated text at any level (`eprintln!`, `println!`, or otherwise)
- Do NOT log clipboard contents
- Do NOT send diagnostics anywhere
- Do NOT read active app contents
- Do NOT capture the screen
- Do NOT use OCR
- Do NOT implement live transcription
- Do NOT implement live insertion
- Do NOT auto-start recording in `open_dictation` mode
- In push-to-talk mode, avoid `window.unminimize()`, `window.show()`, `window.set_focus()` on shortcut press
- Verify Released events first; implement `push_to_talk_hold` if they work, `push_to_talk_toggle` if not
- Fix `check_shortcut_configured` to read from DB
- Fix `check_migrations_current` to check for version 5
- Double-recording guard: if `AudioState.recording` is Some when PTT press arrives, emit error event and return without starting
- Only one PTT pipeline may run at a time; ignore shortcut press if phase is not Idle or Recording (in toggle) / not Idle (in hold)
- Cleanup failure is non-fatal: skip cleanup, use raw transcript, continue to insert
- History is saved even if insertion fails; `was_inserted = false` in that case

---

## Testing Requirements

**All 133 existing Rust tests must pass.**

New tests to add (minimum 15 new; target 148+ total):

| Test | Location | What it verifies |
|------|----------|-----------------|
| `test_migration_005_adds_shortcut_behavior_column` | `db/migrations.rs` | Column exists after migration |
| `test_migration_005_default_value` | `db/migrations.rs` | Default is `'open_dictation'` |
| `test_migration_005_idempotent` | `db/migrations.rs` | Re-running is safe |
| `test_settings_repo_shortcut_behavior_default` | `settings_repo.rs` | Fresh DB returns `'open_dictation'` |
| `test_settings_repo_shortcut_behavior_round_trip` | `settings_repo.rs` | Save `'push_to_talk_toggle'`, read back |
| `test_ptt_status_event_serde_recording` | `models/mod.rs` | `PttStatusEvent` serializes correctly |
| `test_ptt_status_event_serde_error` | `models/mod.rs` | Error phase serializes correctly |
| `test_ptt_phase_idle_allows_start` | `services/ptt.rs` | Idle phase allows pipeline start |
| `test_ptt_phase_recording_blocks_double_start` | `services/ptt.rs` | Second press during recording is a no-op |
| `test_cancel_flag_aborts_pipeline` | `services/ptt.rs` | Cancel flag causes early exit |
| `test_diagnostics_shortcut_reads_from_db` | `services/diagnostics.rs` | Custom shortcut appears in diag output |
| `test_diagnostics_shortcut_default_passes` | `services/diagnostics.rs` | Default shortcut is parseable → pass |
| `test_diagnostics_migrations_version_5_passes` | `services/diagnostics.rs` | Max version 5 → pass |
| `test_diagnostics_migrations_low_version_warns` | `services/diagnostics.rs` | Max version < 5 → warn |
| `test_pipeline_skips_cleanup_when_no_provider` | `services/ptt.rs` | No provider → uses raw text, continues |

Frontend: TypeScript build (`npm run build`) and lint (`npm run lint`) must pass with zero errors.

---

## Verification Commands

Run in `transtypro-ptt` worktree before PR:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run build
npm run lint
```

PowerShell:
```powershell
.\scripts\quality-check.ps1
```

Expected: all exit 0. `cargo test` shows 148+ tests passing.

---

## Merge Rule

**No PR may be merged unless the orchestrator provides the exact phrase:**

```
ORCHESTRATOR APPROVED MERGE
```

---

## Next Step

The orchestrator will launch the Phase 10 implementation agent in the `transtypro-ptt` worktree (`C:\Users\User\Desktop\transtypro-ptt`) on branch `phase/10-ptt-pipeline`.

The agent must:
1. Read this file and all referenced source files before writing any code
2. Verify Released event behavior first (before committing to hold vs toggle)
3. Implement the pipeline in small steps with tests
4. Run all verification commands before PR
5. Create `handoff/phase-10-ptt-pipeline.md` documenting all decisions

Implementation must not begin until the worktree is confirmed at `4f4a4be`.
