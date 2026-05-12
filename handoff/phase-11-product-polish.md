# Phase 11 Handoff: Product Polish, Shortcut Recorder, and PTT Speed Settings

## Phase goal

Replace prototype-level styling and UX with a polished product:
- SVG icons and a logo component in the sidebar (no more emoji)
- A shortcut recorder so users can press keys instead of typing Tauri shortcut strings
- A `ptt_output_mode` setting that lets users skip AI cleanup for faster PTT dictation
- An elapsed timer in the PTT overlay so users understand pipeline duration

## Branch

`phase/11-product-polish`

## Commit

(populated after merge)

---

## Files changed

| File | Change type | Description |
|---|---|---|
| `src/components/Logo.tsx` | New | Inline SVG logo: speech waveform bars + text cursor |
| `src/components/icons.tsx` | New | 11 named SVG icon exports for sidebar navigation |
| `src/components/Sidebar.tsx` | Edit | Replace emoji with SVG icons; add Logo to header; remove stale footer |
| `src/components/PttOverlay.tsx` | Edit | Elapsed recording timer; remove "Live transcript preview" sub-label |
| `src/pages/Home.tsx` | Edit | Add Logo to heading; improve Quick Start with a proper CTA button; improve history count |
| `src/pages/Settings.tsx` | Edit | Add shortcut recorder UI; add ptt_output_mode selector |
| `src/lib/types.ts` | Edit | Add `ptt_output_mode: string` to AppSettings interface |
| `src-tauri/src/models/mod.rs` | Edit | Add `ptt_output_mode: String` to AppSettings struct |
| `src-tauri/src/db/migrations.rs` | Edit | Add migration 006 + 3 new tests |
| `src-tauri/src/db/repositories/settings_repo.rs` | Edit | Update SELECT, default, upsert for ptt_output_mode; add 3 new tests |
| `src-tauri/src/services/ptt.rs` | Edit | Add `read_ptt_output_mode()` helper; branch in `run_pipeline()` |
| `src-tauri/src/services/diagnostics.rs` | Edit (1 line) | Bump expected migration version from 5 → 6 |
| `src-tauri/src/services/privacy.rs` | Edit (1 line) | Minimal forced touch: add `ptt_output_mode` to test struct literal for struct completeness |

---

## Migration 006 details

```sql
ALTER TABLE settings ADD COLUMN ptt_output_mode TEXT NOT NULL DEFAULT 'clean_before_insert';
```

Supported values:
- `clean_before_insert` — default, existing behavior: attempt AI cleanup if a provider is configured
- `insert_raw` — skip cleanup entirely, insert raw transcript immediately for lower latency

The migration is idempotent (tracked by version in `schema_migrations`). Applied at app startup before any service runs. No data loss on existing databases.

---

## ptt_output_mode behavior

### `clean_before_insert` (default)

PTT pipeline behavior is unchanged from Phase 10:
- Recording → Transcribing → Cleaning (if provider configured) → Inserting → Done

### `insert_raw`

In `run_pipeline()`, before the cleanup step, the mode is read from the DB:
```rust
let final_text = if self.read_ptt_output_mode() == "insert_raw" {
    raw_text.clone()  // skip cleanup
} else {
    self.try_cleanup(&raw_text, ptt, handle).unwrap_or_else(|| raw_text.clone())
};
```

When `insert_raw`:
- No `"cleaning"` phase is emitted — overlay jumps from Transcribing to Inserting
- `try_cleanup()` is never called
- `cleaned_text` saved in history equals `raw_text`
- Pipeline is faster (no LLM call)

The `try_cleanup()` helper is not modified.

---

## Visual polish summary

### Logo.tsx

Inline SVG: three vertical rect bars of increasing height (6, 12, 16 px) representing a speech waveform, plus a text cursor (vertical bar with top and bottom serifs). Uses `var(--color-brand-400)` for bars, `var(--color-text-primary)` for cursor. `size` prop (default 22). No external assets.

### icons.tsx

Eleven named SVG icon exports: `HomeIcon`, `DictationIcon`, `HistoryIcon`, `ModesIcon`, `VocabularyIcon`, `ModelsIcon`, `ProvidersIcon`, `PrivacyIcon`, `DiagnosticsIcon`, `SettingsIcon`, `AboutIcon`.

All use: `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, `strokeWidth={1.75}`, `strokeLinecap="round"`, `strokeLinejoin="round"`. Colors inherit from the nav item's Tailwind class — active/hover states work automatically. No external icon packages.

### Sidebar.tsx

- Emoji icon strings replaced with icon key strings (`"home"`, `"dictation"`, etc.)
- `<NavIcon name={item.icon} />` helper maps key → icon component
- `<Logo size={22} />` added to brand header, left of wordmark
- Stale "Phase 1 — UI shell" footer removed

### Home.tsx

- Logo added inline with page heading
- Quick Start card upgraded from text link to proper `<Link>` styled as a button ("Start dictating →")
- History count uses singular/plural ("1 session" vs "2 sessions")

---

## Shortcut recorder behavior

State machine: `idle → recording → captured → idle`

### Flow

1. User clicks **Record shortcut** — transitions to `recording` state
2. A `keydown` listener attaches to `window`
3. User presses a key combination (e.g. Ctrl+Shift+D)
4. Listener builds the Tauri shortcut string:
   - `ctrlKey || metaKey` → `CommandOrControl`
   - `shiftKey` → `Shift`
   - `altKey` → `Alt`
   - Key mapping: `" "` → `"Space"`, `"Enter"` → `"Return"`, single letters → `.toUpperCase()`, others as-is
5. Transitions to `captured` state; displays `CommandOrControl+Shift+D`
6. User clicks **Use this** → calls `handleApplyShortcut(capturedCombo)` → calls existing `updateShortcut` backend command
7. On success: shortcut applied, state returns to `idle`
8. User clicks **Cancel** → state returns to `idle`, no backend call

### Additional controls

- **Reset to default** button: calls `handleApplyShortcut("CommandOrControl+Shift+Space")`
- **Advanced: enter shortcut manually** (collapsed `<details>`): existing text input + Apply button, unchanged

### Validation

- Modifier-only keypresses (`Control`, `Alt`, `Shift`, `Meta`) do not capture
- Single-key shortcut (no modifier): warning shown — "Single-key shortcut — add a modifier to avoid conflicts with typing." Backend call still made if user proceeds.
- Backend errors shown as-is (handles OS registration failures, parse failures)

### Privacy note

The keydown listener reads only `e.key` and modifier booleans. It never logs keys. It is removed via `useEffect` cleanup as soon as `recorderState` leaves `"recording"`. `e.preventDefault()` suppresses keys in the Settings page during recording only.

---

## Overlay timer behavior

### Change from Phase 10.1

The `"Live transcript preview coming later"` sub-label has been removed.

### Elapsed timer

- `elapsedSec` state + `elapsedIntervalRef` ref added
- When the `"recording"` phase event arrives: `clearElapsedTimer()` then start a new 1s interval that increments `elapsedSec`
- When any non-recording phase event arrives (transcribing, cleaning, inserting, done, error, cancelled): `clearElapsedTimer()` — timer stops and resets to 0
- `hideOverlay()` also calls `clearElapsedTimer()`
- `useEffect` cleanup calls both `clearAutoHideTimer()` and `clearElapsedTimer()`

### Display

```
Listening… 0s   (at start)
Listening… 1s   (after 1 second)
Listening… 4s   (after 4 seconds)
Transcribing audio…  (from backend message, timer stopped)
Cleaning text…       (from backend message)
Inserting…           (from backend message)
Done.                (from backend message, overlay hides after 1.5s)
```

Phase label comes from the backend `message` field as before. Only the recording phase has the elapsed suffix.

---

## Tests and checks run

### Rust tests

| Module | New tests added |
|---|---|
| `db/migrations.rs` | `test_migration_006_adds_ptt_output_mode_column`, `test_migration_006_default_value`, `test_migration_006_idempotent` |
| `db/repositories/settings_repo.rs` | `test_settings_repo_ptt_output_mode_default`, `test_settings_repo_ptt_output_mode_round_trip`, `test_settings_repo_ptt_output_mode_preserves_other_fields` |

**Total: 158 tests, 0 failed** (up from 152)

### Verification commands

| Command | Result |
|---|---|
| `cargo fmt` | ✅ Pass |
| `cargo fmt --check` | ✅ Pass (0 differences) |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ Pass (0 warnings) |
| `cargo test` | ✅ 158/158 pass |
| `npm run lint` (`tsc --noEmit`) | ✅ 0 errors |
| `npm run build` | ✅ Pass (315.80 kB JS) |
| `pwsh scripts/quality-check.ps1` | ✅ All checks passed |

---

## Manual QA plan / expected results

This section documents the expected behavior. Live QA results should be confirmed against a real Tauri dev build with Whisper binary + model configured.

### A. Visual polish

1. Open app. Sidebar shows SVG icons — no emoji anywhere.
2. Logo (speech bars + text cursor) visible in sidebar header, left of "transtypro".
3. No "Phase 1 — UI shell" text in sidebar footer.
4. Navigate all 11 pages. No broken navigation.
5. Home page: Logo appears next to heading. "Start dictating →" button navigates to `/dictation`.
6. History count displays "1 session" (singular) or "N sessions" (plural) correctly.

### B. Shortcut recorder

7. Open Settings → Dictation. "Record shortcut" and "Reset to default" buttons visible.
8. Click "Record shortcut". Animated pulse indicator + "Press your shortcut…" text appears.
9. Press Ctrl+Shift+D. `CommandOrControl+Shift+D` is displayed. "Use this" and "Cancel" appear.
10. Click "Use this". "Shortcut applied." success message. Shortcut now active globally.
11. Click "Reset to default". "Shortcut applied." shown. Shortcut is back to `CommandOrControl+Shift+Space`.
12. Click "Record shortcut", then "Cancel". No backend call. Previous shortcut unchanged.
13. Advanced text input visible inside collapsed `<details>` element.
14. Press Ctrl-only during recording — no capture (modifier-only). Then Ctrl+D — captures `CommandOrControl+D`.
15. Press single key (e.g. "D") — captures `D` with warning about single-key conflict.

### C. PTT speed setting

16. Settings → Dictation: "PTT output mode" selector visible with two options.
17. Select "Fast — insert raw transcript immediately". Save. "Settings saved." shown.
18. Open Notepad. Press PTT shortcut. Speak. Press shortcut again.
19. Overlay: "Listening… Xs" → "Transcribing audio…" → "Inserting…" → "Done." — no "Cleaning text…" phase.
20. Spoken text inserts into Notepad as raw transcript.
21. Select "Best quality — clean before insert". Save. Repeat PTT with cleanup provider configured.
22. Overlay: "Listening… Xs" → "Transcribing audio…" → "Cleaning text…" → "Inserting…" → "Done."
23. Cleaned text inserts into Notepad.
24. History entries: both modes show correct `raw_text` and `cleaned_text`.

### D. Overlay timing

25. Press PTT shortcut. Overlay shows "Listening… 0s".
26. Wait 3 seconds without speaking. Shows "Listening… 3s".
27. Stop recording. Elapsed timer stops and disappears.
28. "Done." appears. Overlay auto-hides after ~1.5s.

### E. Regressions

29. `open_dictation` mode: main window opens, FloatingOverlay appears, PTT overlay does not appear.
30. Cancel during PTT: overlay hides immediately, no text inserted.
31. Error path (no Whisper binary): overlay shows error with Dismiss button.
32. Drag overlay while recording → text still inserts into Notepad.
33. All Phase 10.1 QA items remain passing.

---

## Confirmed: no forbidden behaviors

- No forbidden files touched (all edits within approved file list, plus one-line forced touch in `privacy.rs` for struct completeness — same pattern as Phase 9 and 10, documented here)
- No telemetry added
- No transcript text, clipboard contents, or audio data logged or sent
- No cloud storage, no model downloads, no account/login/sync
- No active-app content reading, no screen capture, no OCR
- No unsafe shell commands
- `AudioService`, `TranscriptionService`, `CleanupService`, `InsertionService`, `HistoryService`, `PrivacyService` are all unchanged
- No real live transcription, no live insertion while speaking, no partial transcript preview
- No provider API key behavior changes
- No packaging changes
- `ptt-status` events still contain only phase keywords and generic status strings — no user content
- ShortcutRecorder keydown listener removed via `useEffect` cleanup; keys never logged
- `read_ptt_output_mode()` reads only a setting string from DB — no user content

---

## Known limitations

- Elapsed timer starts from 0s on every recording start — if user stops and restarts quickly, it resets correctly.
- Shortcut recorder does not support Fn-only shortcuts (hardware-level, OS never sees them).
- Modifier-only shortcuts (Ctrl-only, Alt-only) produce a warning but are not blocked by the recorder — the backend `update_shortcut` validation provides the final gate.
- Position of PTT overlay resets to bottom-center on app restart (unchanged from Phase 10.1).
- Elapsed timer shows only whole seconds — sub-second accuracy not needed for this UX purpose.

---

## Next recommended task

- Phase 12: Packaging (Tauri bundler, installer, update manifest)
- Optional: wire `get_recording_status` RMS to overlay bar heights for real audio feedback
- Optional: position overlay on the monitor containing the active app
