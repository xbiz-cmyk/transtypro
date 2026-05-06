# Phase 6: Dictation Pipeline — Handoff

## Branch

`phase/06-dictation-pipeline`

## Goal

Connect the outputs of Phases 3–5 (audio recording → transcription → cleanup) to persistent
SQLite history, replace the mock History page with real backend data, enable "Save as note"
on the Dictation page, and make `get_status_summary` return live system state.

---

## What was built

### Backend (Rust)

#### New command: `create_history_entry`

Added to `src-tauri/src/commands/history.rs`.

Signature:
```rust
pub fn create_history_entry(
    raw_text: String,
    cleaned_text: String,
    mode_used: String,
    state: State<'_, AppState>,
) -> Result<HistoryEntry, AppError>
```

Delegates to `HistoryService::create_history_entry`, which generates a `Uuid::new_v4()` id and
`Utc::now().to_rfc3339()` timestamp. `was_inserted` is always `false` until Phase 9 text insertion.
No audio WAV path is stored anywhere in the history record.

Registered in `src-tauri/src/lib.rs` invoke handler under `// Phase 6 — history creation`.

Tauri camelCase mapping:
| Rust param | Frontend invoke key |
|---|---|
| `raw_text` | `rawText` |
| `cleaned_text` | `cleanedText` |
| `mode_used` | `modeUsed` |

#### Updated command: `get_status_summary`

`src-tauri/src/commands/mod.rs` — replaced the fully static placeholder.

The logic is extracted into a testable helper `build_status_summary(db: Arc<Mutex<Connection>>)`
that does not require a Tauri State wrapper. The Tauri command calls it with `state.db.clone()`.

| Field | Source |
|---|---|
| `privacy_mode` | `settings.local_only_mode` → `"local-only"` or `"cloud-enabled"` |
| `transcription_ready` | `whisper_binary_path.is_some() && whisper_model_path.is_some()` |
| `cleanup_provider` | First enabled cleanup provider name, or `None` |
| `active_mode` | `settings.active_mode` |
| `history_count` | `history.len() as u32` (full list; O(n) acceptable for Phase 6) |

Three sequential lock acquisitions on `Arc<Mutex<Connection>>` — safe, no deadlock risk.

### Frontend (TypeScript + React)

#### `src/lib/api.ts` — 4 new wrappers

```ts
listHistory()                                          → HistoryEntry[]
deleteHistoryEntry(id: string)                         → void
clearHistory()                                         → void
createHistoryEntry({ rawText, cleanedText, modeUsed }) → HistoryEntry
```

`HistoryEntry` type already existed in `src/lib/types.ts` — no type changes needed.

#### `src/pages/History.tsx` — real backend

- `MOCK_ENTRIES` array removed entirely.
- `useEffect` calls `listHistory()` on mount; shows loading state while fetching.
- `ErrorMessage` shown on fetch failure.
- Per-entry **Copy** button: `navigator.clipboard.writeText(entry.cleaned_text || entry.raw_text)`.
- Per-entry **Delete** button: calls `deleteHistoryEntry(id)`; filters entry from state on success; shows inline error on failure.
- Page-level **Clear all** button (visible only when entries exist): calls `clearHistory()`; empties state.
- Raw text shown as secondary line when `raw_text !== cleaned_text`.
- Filter bar remains cosmetic (no backend filtering).

#### `src/pages/Dictation.tsx` — Save as note enabled

- `getSettings()` called on mount to read `active_mode` (fallback: `"Smart Mode"` on error).
- New state: `activeMode`, `isSavingNote`, `noteSaved`.
- `handleSaveNote`:
  - `rawText` = `transcriptResult.raw_text`
  - `cleanedText` = `cleanupResult?.cleaned_text ?? transcriptResult.raw_text`
  - `modeUsed` = `activeMode`
  - Calls `createHistoryEntry(...)` then sets `noteSaved = true`.
- **Save as note** button:
  - Disabled until `transcriptResult` is set.
  - Shows `"Saving…"` / `"Saved ✓"` / `"Save as note"`.
  - Auto-disables after first save (no double-save).
- `noteSaved` reset when `handleRecord` or `handleCancel` is called.
- **Insert** button title updated to `"Requires text insertion — Phase 9"`.
- WAV `file_path` never stored in history entries.

#### `src/pages/Home.tsx`

Removed stale `"Recording available in Phase 3"` placeholder text from the Quick start card.
No structural changes.

---

## Tests

### New Rust tests: 9 added (91 → 100 total)

**`src-tauri/src/commands/history.rs`** — 4 tests (service-level, in-memory SQLite):

| Test | Verifies |
|---|---|
| `test_create_generates_id_and_timestamp` | `id` and `timestamp` are non-empty; fields stored correctly |
| `test_create_entry_appears_in_list` | Entry is retrievable via `list_history()` after creation |
| `test_create_raw_equals_cleaned_when_no_cleanup` | `raw_text == cleaned_text` case works correctly |
| `test_created_entries_have_unique_ids` | Two sequential entries get distinct UUIDs |

**`src-tauri/src/commands/mod.rs`** — 4 tests (helper function `build_status_summary`, in-memory SQLite):

| Test | Verifies |
|---|---|
| `test_status_summary_defaults` | Fresh DB (cloud-enabled default) yields correct field values |
| `test_status_summary_history_count` | `history_count` increments after `create_history_entry` |
| `test_status_summary_transcription_ready_requires_both_paths` | `true` only when both whisper paths are set |
| `test_status_summary_privacy_mode_reflects_settings` | Reflects `local_only_mode` toggle correctly |

**Why Tauri State injection tests were not added:**

`tauri::State<'_, T>` cannot be constructed outside a running Tauri application — it is an opaque
handle created by the Tauri runtime. Rather than wrapping the logic in `build_status_summary`
(a testable function) and duplicating it in the Tauri command wrapper, the command wrapper is
kept as a one-liner delegate. All observable logic is covered by the helper tests above.

The same reasoning applies to the `create_history_entry` command, which is a one-liner delegate
to `HistoryService::create_history_entry`. The service and its repository are covered by the
4 tests added here plus the 7 pre-existing `history_repo` tests.

---

## Files changed

### Modified

| File | Change |
|---|---|
| `src-tauri/src/commands/history.rs` | Added `create_history_entry` command + 4 service-level tests |
| `src-tauri/src/commands/mod.rs` | Replaced static `get_status_summary` with live DB-backed version + `build_status_summary` helper + 4 tests; added `use crate::db::AppState` import |
| `src-tauri/src/lib.rs` | Registered `create_history_entry` in invoke handler |
| `src/lib/api.ts` | Added `listHistory`, `deleteHistoryEntry`, `clearHistory`, `createHistoryEntry` wrappers; added `HistoryEntry` to imports |
| `src/pages/Dictation.tsx` | Added `getSettings`/`createHistoryEntry` calls; enabled Save as note; added `activeMode`, `isSavingNote`, `noteSaved` state; updated Insert title |
| `src/pages/History.tsx` | Removed `MOCK_ENTRIES`; replaced with real `listHistory`/`deleteHistoryEntry`/`clearHistory` calls; loading/error/delete-error states; Clear all button |
| `src/pages/Home.tsx` | Removed stale "Recording available in Phase 3" text |

### No new files

No schema migration. No new Cargo dependencies. No new npm dependencies.

---

## Forbidden files — confirmed not touched

- `src-tauri/src/services/audio.rs` ✓
- `src-tauri/src/commands/audio.rs` ✓
- `src-tauri/src/services/transcription.rs` ✓
- `src-tauri/src/commands/transcription.rs` ✓
- `src-tauri/src/services/cleanup.rs` ✓
- `src-tauri/src/commands/cleanup.rs` ✓
- `src-tauri/src/services/providers.rs` ✓
- `src-tauri/src/commands/providers.rs` ✓
- `src-tauri/src/services/privacy.rs` ✓
- `src-tauri/src/services/history.rs` ✓
- `src-tauri/src/db/**` ✓
- `src-tauri/Cargo.toml` ✓
- `src/pages/Providers.tsx` ✓
- `src/pages/Models.tsx` ✓
- `src/pages/Settings.tsx` ✓
- `src/pages/Privacy.tsx` ✓
- `src/pages/Diagnostics.tsx` ✓

---

## Verification results

```
cargo fmt --check          PASS
cargo clippy -D warnings   PASS (0 warnings)
cargo test                 PASS (100/100)
npm run lint               PASS (tsc --noEmit, 0 errors)
npm run build              PASS (71 modules, 283 kB JS bundle)
pwsh quality-check.ps1     All checks passed
```

---

## Manual QA checklist

| Step | Expected result |
|---|---|
| Open app → Home page | Backend connected; status values load from DB (not hardcoded) |
| Configure whisper binary + model path in Models | Home shows "Transcription model: Ready" |
| Enable a cleanup provider in Providers | Home shows provider name in Cleanup provider |
| Go to Dictation, record audio, stop | WAV file saved; Transcribe button appears |
| Click Transcribe | Transcript text appears in Result area |
| Click Save as note | Button shows "Saving…" then "Saved ✓"; button disables |
| Go to History | Entry appears with mode badge, timestamp, and cleaned text |
| Click Copy on entry | Clipboard receives cleaned text |
| Click Delete on entry | Entry removed from list |
| Dictate another session, save, then click Clear all | All entries removed |
| Check Home page history_count | Shows correct count after saves/deletes |
| Click Record again on Dictation page | "Saved ✓" resets to "Save as note" for the new session |
| Quick start card on Home | No "Recording available in Phase 3" text |

---

## Privacy impact

- History stores text only. No audio WAV file paths stored.
- `model_path` from `TranscriptionResult` not stored.
- No provider ID stored in history entries.
- No API keys read, stored, or passed through Phase 6 code.
- `get_status_summary` is read-only — no mutations.
- No new network calls introduced. Phase 5 privacy enforcement unchanged.
- `was_inserted` is always `false` for Phase 6 entries — Phase 9 will update this.

---

## Known limitations

- No global shortcut (Phase 7).
- No text insertion into active app (Phase 9) — Insert button stays disabled.
- No clipboard paste simulation (Phase 9).
- `history_count` in `get_status_summary` uses `list_history().len()` — O(n). Phase 8 can optimize with a `COUNT(*)` query.
- Date-based history retention policy stored in settings but not enforced (Phase 8).
- WAV retention sweep still limited to transcription-time cleanup (Phase 8).
- No search/filter backend for History page (Phase 8/9).
- No confirmation dialog before "Clear all" in History page.
- `DiagnosticsService` still returns a static report (Phase 8).

---

## Next recommended tasks

1. **Phase 7: Global shortcut and context** — press-and-hold or toggle shortcut; active app detection; auto-start dictation pipeline from shortcut.
2. **Phase 9: Text insertion** — clipboard-based paste into active app; `was_inserted = true` on success.
3. **Phase 8: Privacy enforcement + diagnostics + retention** — enforce `retention_days` for history; delete old WAV files; real diagnostics report; local-only enforcement audit.
