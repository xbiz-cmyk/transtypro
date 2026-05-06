# Wave 6 Launch Record — End-to-End Dictation Pipeline and History Persistence

## Launch metadata

| Item | Value |
|---|---|
| Wave | 6 |
| Phase | Phase 6 — End-to-end dictation pipeline and history persistence |
| Base commit | `2c203c3` — feat(providers): add cleanup provider support |
| Branch | `phase/06-dictation-pipeline` |
| Worktree | `C:\Users\User\Desktop\transtypro-pipeline` |
| Launched | 2026-05-06 |
| Merge rule | No merge without the exact phrase: **ORCHESTRATOR APPROVED MERGE** |

---

## Phase 6 scope

**One sentence:** Connect the outputs of Phases 3–5 to persistent history, and make the History page show real data.

### Approved features

- Save dictation result to history from the Dictation page ("Save as note" button).
- History page uses real backend data instead of mock data (`MOCK_ENTRIES`).
- Delete one history entry.
- Clear all history entries.
- Show raw text and cleaned/final text in history entries.
- Track mode used per entry.
- Keep Copy behavior working (copy cleaned text if available, otherwise raw text).
- Enable "Save as note" button (currently disabled).
- Add `create_history_entry` Tauri command (service method exists; command wrapper is missing).
- Wire `get_status_summary` to real SQLite (currently a static placeholder).
- Update Home page stale text where needed.

### What Phase 6 is not

- No global shortcuts (Phase 7).
- No text insertion into the active app (Phase 9).
- No clipboard paste simulation (Phase 9).
- No diagnostics overhaul (Phase 8).
- No audio recording rewrites (Phase 3 is complete).
- No Whisper transcription rewrites (Phase 4 is complete).
- No cleanup provider rewrites (Phase 5 is complete).
- No packaging (Phase 10).
- No cloud storage.
- No audio files stored in history — text only.
- No new provider features.
- No changes to provider API key handling.
- No changes to Whisper binary behavior.

---

## Architecture summary

### The data flow

```
Phase 3 output:  RecordingResult  { file_path, duration_ms, sample_rate }
                      ↓
Phase 4 output:  TranscriptionResult  { raw_text, duration_ms, model_path }
                      ↓
Phase 5 output:  CleanupResult  { cleaned_text, provider_id, ... }  ← optional
                      ↓
Phase 6 action:  User clicks "Save as note"
                      ↓
                 createHistoryEntry { rawText, cleanedText, modeUsed }
                      ↓
                 HistoryService::create_history_entry  (generates id + timestamp)
                      ↓
                 HistoryRepository::create  →  SQLite history table
                      ↓
History page:    listHistory()  →  real entries displayed
                      ↓
Home page:       get_status_summary()  →  real history_count, cleanup_provider, etc.
```

### New Tauri command: `create_history_entry`

Add to `src-tauri/src/commands/history.rs`:

```rust
#[tauri::command]
pub fn create_history_entry(
    raw_text: String,
    cleaned_text: String,
    mode_used: String,
    state: State<'_, AppState>,
) -> Result<HistoryEntry, AppError> {
    HistoryService::new(state.db.clone()).create_history_entry(HistoryEntry {
        id: String::new(),         // overwritten by service (Uuid::new_v4)
        raw_text,
        cleaned_text,
        mode_used,
        timestamp: String::new(),  // overwritten by service (Utc::now)
        was_inserted: false,       // always false until Phase 9
    })
}
```

Register in `src-tauri/src/lib.rs` invoke handler alongside the other history commands.

### Updated command: `get_status_summary`

Replace the static placeholder in `src-tauri/src/commands/mod.rs`.

Add `state: State<'_, AppState>` parameter. Read real values:

- `privacy_mode`: from `settings.local_only_mode`
- `transcription_ready`: `true` only when both `whisper_binary_path` and `whisper_model_path` are non-null
- `cleanup_provider`: first enabled cleanup provider name from `list_enabled_cleanup()`, or `None`
- `active_mode`: from `settings.active_mode`
- `history_count`: `HistoryRepository::list()?.len() as u32`

### New TypeScript wrappers (add to `src/lib/api.ts`)

The history Tauri commands exist but have no frontend wrappers. Add:

```ts
export async function listHistory(): Promise<HistoryEntry[]> {
  return invoke<HistoryEntry[]>("list_history");
}

export async function deleteHistoryEntry(id: string): Promise<void> {
  return invoke<void>("delete_history_entry", { id });
}

export async function clearHistory(): Promise<void> {
  return invoke<void>("clear_history");
}

export async function createHistoryEntry(params: {
  rawText: string;
  cleanedText: string;
  modeUsed: string;
}): Promise<HistoryEntry> {
  return invoke<HistoryEntry>("create_history_entry", params);
}
```

### Dictation page changes

1. Fetch `getSettings()` on mount to get `active_mode` for saving history entries.
2. Add `isSavingNote` and `noteSaved` state.
3. Add `handleSaveNote` handler:
   - `rawText` = `transcriptResult.raw_text`
   - `cleanedText` = `cleanupResult?.cleaned_text ?? transcriptResult.raw_text`
   - `modeUsed` = `activeMode` from settings (default `"Smart Mode"` on error)
   - Calls `createHistoryEntry(...)` then sets `noteSaved = true`
4. "Save as note" button: enabled when `transcriptResult` is set; shows `"Saving…"` / `"Saved ✓"` / `"Save as note"`.
5. "Insert" button: stays disabled, update title to `"Requires text insertion — Phase 9"`.
6. Reset `noteSaved` when a new recording starts.

### History page changes

Replace `MOCK_ENTRIES` with real backend:

1. State: `entries`, `isLoading`, `error`.
2. `useEffect` → `listHistory()` on mount.
3. Per-entry Delete button → `deleteHistoryEntry(id)` → filter from state.
4. Per-entry Copy button → `navigator.clipboard.writeText(cleaned_text)`.
5. Page-level "Clear all" button (danger, visible when entries exist) → `clearHistory()` → empty state.
6. Loading state (while fetching) + error state (`<ErrorMessage>`).
7. Filter bar remains cosmetic (no backend filtering in Phase 6).
8. Display: primary text = `cleaned_text`; show `raw_text` if different; show `mode_used` badge and timestamp.

### Home page changes

- Remove stale "Recording available in Phase 3" text — Phase 3 is done.
- All status cards already read from `StatusSummary` — no structural changes needed once `get_status_summary` is real.

---

## Database / schema decision

**No migration planned.**

The existing `history` table from migration 001 is sufficient:

```sql
CREATE TABLE history (
    id            TEXT PRIMARY KEY,
    raw_text      TEXT NOT NULL,
    cleaned_text  TEXT NOT NULL,
    mode_used     TEXT NOT NULL DEFAULT 'smart',
    timestamp     TEXT NOT NULL,
    was_inserted  INTEGER NOT NULL DEFAULT 0
);
```

Field assignment rules:

| Field | Phase 6 value |
|---|---|
| `raw_text` | `TranscriptionResult.raw_text` |
| `cleaned_text` | `CleanupResult.cleaned_text` if cleanup ran, otherwise same as `raw_text` |
| `mode_used` | `AppSettings.active_mode` fetched at Dictation page mount |
| `was_inserted` | Always `false` (0) — Phase 9 text insertion will set this |
| `id` | Generated by `HistoryService::create_history_entry` via `Uuid::new_v4()` |
| `timestamp` | Generated by `HistoryService::create_history_entry` via `Utc::now().to_rfc3339()` |

**`was_copied` — do not add.** No migration for this field in Phase 6. No downstream use case requires it yet.

**New migration — only if a compile blocker proves it unavoidable.** Stop and ask the orchestrator before adding any migration.

---

## Tauri camelCase argument reminder

Rust parameter names use `snake_case`. Tauri maps them to `camelCase` on the frontend. Always use camelCase in `invoke()` calls.

| Rust field | TypeScript / invoke key |
|---|---|
| `raw_text` | `rawText` |
| `cleaned_text` | `cleanedText` |
| `mode_used` | `modeUsed` |
| `was_inserted` | `wasInserted` |
| `provider_id` | `providerId` |
| `file_path` | `filePath` |

---

## Local-first privacy rules

- History stores text only. No audio file paths are persisted to the history table.
- The `file_path` from `RecordingResult` must never be stored in a history entry.
- The `model_path` from `TranscriptionResult` must never be stored in a history entry.
- No provider ID is stored in a history entry (the provider that cleaned the text is not recorded).
- No API keys are read, stored, or passed through any Phase 6 code path.
- `get_status_summary` is read-only — it reads settings and history count; no mutations.
- If `local_only_mode = true`, no new network calls are introduced by Phase 6 code.

---

## Audio files not stored in history

The `file_path` field of `RecordingResult` is a path to a temporary WAV file. It is used only for the `transcribe_audio` call. It must NOT be stored in any history entry field. Phase 4's `TranscriptionService` already handles WAV cleanup. Phase 6 must not persist WAV paths anywhere.

---

## `was_inserted` stays false until Phase 9

The `was_inserted` boolean in `HistoryEntry` is reserved for Phase 9 text insertion. All history entries created in Phase 6 must set `was_inserted = false`. Do not attempt to set it based on whether the user copied the text — copying is not the same as inserting.

---

## Files the Phase 6 agent may edit

```
src-tauri/src/commands/history.rs         ← add create_history_entry command
src-tauri/src/commands/mod.rs             ← wire get_status_summary to real DB
src-tauri/src/lib.rs                      ← register create_history_entry
src/lib/api.ts                            ← add history wrappers + createHistoryEntry
src/lib/types.ts                          ← only if a new type is strictly required
src/pages/Dictation.tsx                   ← enable Save as note
src/pages/History.tsx                     ← replace mock with real backend
src/pages/Home.tsx                        ← update stale text
handoff/phase-06-dictation-pipeline.md    ← required handoff file
docs/PROGRESS.md                          ← only after implementation succeeds
docs/TASK_BOARD.md                        ← only after implementation succeeds
```

---

## Files the Phase 6 agent must NOT edit

```
src-tauri/src/services/audio.rs           ← Phase 3 — do not touch
src-tauri/src/commands/audio.rs           ← Phase 3 — do not touch
src-tauri/src/services/transcription.rs   ← Phase 4 — do not touch
src-tauri/src/commands/transcription.rs   ← Phase 4 — do not touch
src-tauri/src/services/cleanup.rs         ← Phase 5 — do not touch
src-tauri/src/commands/cleanup.rs         ← Phase 5 — do not touch
src-tauri/src/services/providers.rs       ← Phase 5 — do not touch
src-tauri/src/commands/providers.rs       ← Phase 5 — do not touch
src-tauri/src/services/privacy.rs         ← do not touch
src-tauri/src/services/history.rs         ← already complete; do not edit unless
                                             a compile blocker requires it and
                                             the orchestrator approves first
src-tauri/src/db/**                       ← no schema changes; do not touch
src-tauri/Cargo.toml                      ← no new dependencies
src-tauri/Cargo.lock                      ← do not modify directly
src/pages/Providers.tsx                   ← do not touch
src/pages/Models.tsx                      ← do not touch
src/pages/Settings.tsx                    ← do not touch
src/pages/Privacy.tsx                     ← do not touch
src/pages/Diagnostics.tsx                 ← do not touch
docs/PHASES.md                            ← orchestrator only
docs/PARALLEL_EXECUTION_PLAN.md           ← orchestrator only
docs/ARCHITECTURE.md                      ← orchestrator only
AGENTS.md                                 ← orchestrator only
CLAUDE.md                                 ← orchestrator only
SOUL.md                                   ← orchestrator only
```

---

## Implementation reminders for the Phase 6 agent

- Use the existing `history` table. Do not add a migration.
- Do not add `was_copied` in Phase 6.
- `raw_text` must be `TranscriptionResult.raw_text`.
- `cleaned_text` must be `CleanupResult.cleaned_text` when cleanup ran; otherwise copy `raw_text`.
- `mode_used` must come from `AppSettings.active_mode`.
- `was_inserted` must stay `false` for all Phase 6 entries.
- "Save as note" calls `create_history_entry`.
- History page calls `list_history`, `delete_history_entry`, and `clear_history`.
- Copy copies `cleaned_text` if available, otherwise `raw_text`.
- "Insert" button must remain disabled.
- Audio WAV file paths must never be stored in history.
- Do not change any Phase 3, Phase 4, or Phase 5 working code.
- Do not add a new migration unless a compile blocker proves it unavoidable — stop and ask the orchestrator first.

---

## Verification commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run lint
npm run build
./scripts/quality-check.ps1
```

All 91 existing tests must pass. Add tests for:
- `create_history_entry` command (at least: creates entry, returns it with generated id + timestamp)
- `get_status_summary` with in-memory DB (at least: correct `history_count`, correct `transcription_ready`)

---

## Merge rule

No PR for `phase/06-dictation-pipeline` may be merged without the orchestrator sending the exact phrase:

```
ORCHESTRATOR APPROVED MERGE
```

---

## Next step

1. Open a new agent session in `C:\Users\User\Desktop\transtypro-pipeline`.
2. The agent reads this file and `handoff/phase-05-cleanup-providers.md`.
3. The agent implements Phase 6 in order:
   a. `create_history_entry` command + register in `lib.rs`
   b. Wire `get_status_summary` to real DB
   c. Add history wrappers + `createHistoryEntry` to `api.ts`
   d. Replace mock with real backend in `History.tsx`
   e. Enable "Save as note" in `Dictation.tsx`
   f. Update stale text in `Home.tsx`
   g. Run all checks
   h. Create `handoff/phase-06-dictation-pipeline.md`
   i. Create PR against `main`
4. Orchestrator reviews and merges with `ORCHESTRATOR APPROVED MERGE`.

---

## Agent prompt for the transtypro-pipeline session

```
You are the transtypro Phase 6 agent.

Your working directory is: C:\Users\User\Desktop\transtypro-pipeline
Your branch is: phase/06-dictation-pipeline

Read these files before writing any code:
- handoff/wave-6-launch.md  ← this file; your full specification
- handoff/phase-05-cleanup-providers.md
- src-tauri/src/commands/history.rs
- src-tauri/src/commands/mod.rs
- src-tauri/src/services/history.rs
- src-tauri/src/lib.rs
- src/pages/Dictation.tsx
- src/pages/History.tsx
- src/pages/Home.tsx
- src/lib/api.ts
- src/lib/types.ts

Your task:
Implement Phase 6 — End-to-end dictation pipeline and history persistence.

Phase 6 scope (exactly):
1. Add create_history_entry Tauri command to src-tauri/src/commands/history.rs
2. Register create_history_entry in src-tauri/src/lib.rs
3. Wire get_status_summary in src-tauri/src/commands/mod.rs to real SQLite
4. Add listHistory, deleteHistoryEntry, clearHistory, createHistoryEntry wrappers to src/lib/api.ts
5. Replace MOCK_ENTRIES in src/pages/History.tsx with real backend (list, delete, clear all, copy)
6. Enable Save as note in src/pages/Dictation.tsx
7. Update stale "Recording available in Phase 3" text in src/pages/Home.tsx

Run all checks before creating PR:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test
- npm run lint
- npm run build

Create handoff/phase-06-dictation-pipeline.md with:
- what changed
- test results
- known limitations
- next recommended task

Then create a PR against main.

Constraints (read wave-6-launch.md for the full list):
- Do not add a schema migration.
- Do not touch Phase 3/4/5 service or command files.
- Do not store audio file paths in history.
- was_inserted must be false for all Phase 6 entries.
- Insert button must stay disabled.
- No new Cargo dependencies.
```
