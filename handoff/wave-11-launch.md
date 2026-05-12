# Wave 11 Launch Record — Product Polish, Shortcut Recorder, and PTT Speed Settings

## Current main commit

`b014691` — fix(ptt): add feedback overlay window

## Phase name

Phase 11: Product polish, shortcut recorder, and PTT speed settings

## Why Phase 11 is needed

The app works functionally end-to-end, but the experience still feels like a prototype. The UI uses emoji icons in the sidebar. The shortcut setting requires users to type raw Tauri shortcut strings manually. Push-to-talk always runs cleanup before insertion, which adds latency; there is no user control over this. The overlay feedback has no sense of elapsed time.

Phase 11 addresses all four gaps without touching any core pipeline services.

## Branch

`phase/11-product-polish`

## Worktree path

`C:\Users\User\Desktop\transtypro-polish`

## Subagent recommendation

**Single agent.** Do not split into subagents.

Streams B (shortcut recorder) and C (PTT speed setting) both require editing `src/pages/Settings.tsx`. Splitting them would guarantee a merge conflict. Stream A (visual polish) is independent but small enough not to justify a separate agent's cold-start cost. Total scope is manageable sequentially.

---

## Approved scope

### A. Visual polish and branding

**Goal:** Replace prototype-level styling with a polished desktop app look.

**Logo component** — `src/components/Logo.tsx` (new):
- Inline SVG: speech-wave bars (three vertical rects of increasing height) + a text cursor bar to the right
- Conveys "speech → text"
- `size` prop, no external deps, uses `var(--color-brand-400)` and `var(--color-text-primary)` CSS vars

**Icons file** — `src/components/icons.tsx` (new):
- Single file, named exports: `HomeIcon`, `DictationIcon`, `HistoryIcon`, `ModesIcon`, `VocabularyIcon`, `ModelsIcon`, `ProvidersIcon`, `PrivacyIcon`, `DiagnosticsIcon`, `SettingsIcon`, `AboutIcon`
- All icons: `size = 16` prop, `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, `strokeWidth={1.75}`
- Consistent visual weight across all nav items
- No external icon packages unless implementation proves absolutely impossible without one

**Sidebar** — `src/components/Sidebar.tsx` (edit):
- Replace emoji icon strings with SVG icon components from `icons.tsx`
- The `NavItem.icon` field (`string`) changes from emoji values to identifier keys (e.g., `"home"`, `"dictation"`)
- Add a `<NavIcon name={item.icon} />` lookup helper that maps key → icon component
- Add `<Logo size={22} />` in the brand header alongside "transtypro" wordmark
- Fix sidebar footer: remove "Phase 1 — UI shell" text

**Home page** — `src/pages/Home.tsx` (edit):
- Improve dashboard layout and spacing
- Add Logo inline with the page heading or as a header row
- Improve status rows and card labels
- "Quick start" card: add a proper "Start dictating →" button navigating to `/dictation`
- Improve history count display

No data model changes, no new API calls.

---

### B. Shortcut recorder

**Goal:** User presses keys instead of typing Tauri shortcut strings manually.

**Implementation — inline `ShortcutRecorder` component in `src/pages/Settings.tsx`:**

State machine: `idle → recording → captured → idle`

While `recording`, attach a `keydown` listener to `window`:
- Skip pure modifier keypresses (`Control`, `Alt`, `Shift`, `Meta`)
- On a non-modifier key: build shortcut string:
  - `(ctrlKey || metaKey)` → prepend `CommandOrControl`
  - `shiftKey` → prepend `Shift`
  - `altKey` → prepend `Alt`
  - Key name mapping: `" "` → `"Space"`, `"Enter"` → `"Return"`, single letters → `.toUpperCase()`, everything else as-is
  - Join with `+`
- Transition to `captured` state, display the combo
- Call `e.preventDefault()` during recording to suppress the key in the page

While `captured`:
- Show "Use this" → calls existing `handleApplyShortcut(combo)`
- Show "Cancel" → returns to `idle` without calling backend
- Combo displayed in `font-mono` label

Additional controls:
- "Reset to default" button → calls `handleApplyShortcut("CommandOrControl+Shift+Space")`
- Keep existing manual text input as an advanced fallback (can be collapsed or placed below the recorder)

Validation shown to user before calling backend:
- If combo has no modifier + single normal key: warn "Single-key shortcuts can conflict with typing. Add a modifier."
- Backend `updateShortcut` errors shown as-is (handles parse failures, OS registration failures)

**No backend changes for Stream B.** Existing `update_shortcut` command is reused unchanged.

---

### C. PTT speed setting

**Goal:** Let the user choose speed vs quality for PTT output.

#### Migration 006 — `src-tauri/src/db/migrations.rs`

```sql
ALTER TABLE settings ADD COLUMN ptt_output_mode TEXT NOT NULL DEFAULT 'clean_before_insert';
```

Add to `MIGRATIONS` array: `(6, MIGRATION_006)`.

New tests:
- `test_migration_006_adds_ptt_output_mode_column`
- `test_migration_006_default_value` (assert `"clean_before_insert"`)
- `test_migration_006_idempotent`

#### Rust model — `src-tauri/src/models/mod.rs`

Add to `AppSettings`:
```rust
/// PTT output mode. "clean_before_insert" (default) | "insert_raw"
pub ptt_output_mode: String,
```

#### Settings repository — `src-tauri/src/db/repositories/settings_repo.rs`

- Add `ptt_output_mode` as column 10 in SELECT (after `shortcut_behavior`)
- Add `ptt_output_mode: "clean_before_insert".to_string()` to `QueryReturnedNoRows` default
- Add `?11` / `ptt_output_mode = excluded.ptt_output_mode` to upsert

New tests:
- `test_settings_repo_ptt_output_mode_default`
- `test_settings_repo_ptt_output_mode_round_trip` (save `"insert_raw"`, read back)
- `test_settings_repo_ptt_output_mode_preserves_other_fields`

#### PTT pipeline service — `src-tauri/src/services/ptt.rs`

Add helper:
```rust
fn read_ptt_output_mode(&self) -> String {
    self.db.lock().ok()
        .and_then(|conn| SettingsRepository::new(&conn).get().ok())
        .map(|s| s.ptt_output_mode)
        .unwrap_or_else(|| "clean_before_insert".to_string())
}
```

In `run_pipeline()`, before the cleanup step, branch on mode:

```rust
let final_text = if self.read_ptt_output_mode() == "insert_raw" {
    // Fast mode: skip cleanup, insert raw immediately
    raw_text.clone()
} else {
    // Quality mode: attempt cleanup (existing behavior, non-fatal)
    self.try_cleanup(&raw_text, ptt, handle)
        .unwrap_or_else(|| raw_text.clone())
};
```

When `insert_raw`: no `Cleaning` phase emitted. Pipeline goes directly to `Inserting`. `cleaned_text` saved in history equals `raw_text`. The `try_cleanup()` method is untouched.

#### Diagnostics service — `src-tauri/src/services/diagnostics.rs`

**Minimal change only:** Update `check_migrations_current` to expect schema version 6 instead of 5.

No diagnostics rewrite. No other changes to this file.

#### Frontend — `src/lib/types.ts`

Add to `AppSettings` interface:
```ts
/** PTT output mode: "clean_before_insert" | "insert_raw" */
ptt_output_mode: string;
```

#### Frontend — `src/pages/Settings.tsx`

1. Add `const [pttOutputMode, setPttOutputMode] = useState("clean_before_insert");`
2. In load `useEffect`: `setPttOutputMode(s.ptt_output_mode ?? "clean_before_insert");`
3. In `handleSave` base object and override: include `ptt_output_mode: pttOutputMode`
4. Add in the Dictation card (after shortcut behavior selector):

```tsx
<Select
  id="ptt-output-mode-selector"
  label="PTT output mode"
  value={pttOutputMode}
  onChange={(e) => setPttOutputMode(e.target.value)}
>
  <option value="clean_before_insert">Best quality — clean before insert (slower)</option>
  <option value="insert_raw">Fast — insert raw transcript immediately</option>
</Select>
<p className="text-xs text-(--color-text-muted)">
  Fast mode skips AI cleanup and inserts the raw transcript immediately after transcription.
</p>
```

No new API wrapper needed — `updateSettings()` already serializes the full `AppSettings` object.

---

### D. Overlay timing feedback

**Goal:** Make PTT delay feel understandable.

**Implementation — `src/components/PttOverlay.tsx` (edit):**

Add a local elapsed timer using `useRef<ReturnType<typeof setInterval>>`:
- Start a `setInterval` (1 s tick) when phase transitions to `"recording"`
- Increment a `elapsedSec` state counter each tick
- Display `Xs` next to the phase label (e.g., `"Listening… 3s"`)
- Clear and reset the interval when phase reaches `"done"`, `"idle"`, or `"cancelled"`

Improve status message labels where needed:
- Phase `"transcribing"` → display: "Transcribing audio…"
- Phase `"cleaning"` → display: "Cleaning text…"
- Phase `"inserting"` → display: "Inserting…"
- Phase `"done"` → display: "Done."

The `message` field from the backend `ptt-status` event is the primary display string. If the message already matches these labels (because `ptt.rs` emits them), the frontend just shows it. The elapsed timer is an additive local display only — no backend changes needed.

**No backend changes needed for Stream D.**

---

## DB ownership correction (correction 1)

The agent may edit only these DB files:
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/settings_repo.rs`

All other DB files are forbidden:
- `src-tauri/src/db/connection.rs`
- `src-tauri/src/db/repositories/history_repo.rs`
- `src-tauri/src/db/repositories/modes_repo.rs`
- `src-tauri/src/db/repositories/providers_repo.rs`
- `src-tauri/src/db/repositories/vocabulary_repo.rs`

## Diagnostics correction (correction 2)

`src-tauri/src/services/diagnostics.rs` may be edited **only** to update the `check_migrations_current` function's expected version from 5 to 6. No other diagnostics changes. No diagnostics rewrite.

---

## Allowed files for Phase 11 agent

The agent may create or edit only:

- `src/components/Logo.tsx` (new)
- `src/components/icons.tsx` (new)
- `src/components/Sidebar.tsx`
- `src/components/PttOverlay.tsx`
- `src/pages/Home.tsx`
- `src/pages/Settings.tsx`
- `src/lib/types.ts`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/repositories/settings_repo.rs`
- `src-tauri/src/services/ptt.rs`
- `src-tauri/src/services/diagnostics.rs` — migration version check only
- `handoff/phase-11-product-polish.md` (new)
- `docs/PROGRESS.md` — only after implementation succeeds
- `docs/TASK_BOARD.md` — only after implementation succeeds

## May inspect but should avoid editing unless strictly required

- `src/App.tsx`
- `src/components/FloatingOverlay.tsx`
- `src/components/ShortcutHandler.tsx`
- `src/components/ui/*.tsx`
- `src/lib/api.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/shortcut.rs`
- `src-tauri/src/commands/ptt.rs`

## Forbidden files

Do not edit under any circumstances:

- `src-tauri/src/services/audio.rs`
- `src-tauri/src/commands/audio.rs`
- `src-tauri/src/services/transcription.rs`
- `src-tauri/src/commands/transcription.rs`
- `src-tauri/src/services/cleanup.rs`
- `src-tauri/src/commands/cleanup.rs`
- `src-tauri/src/services/insertion.rs`
- `src-tauri/src/commands/insertion.rs`
- `src-tauri/src/services/history.rs`
- `src-tauri/src/commands/history.rs`
- `src-tauri/src/services/providers.rs`
- `src-tauri/src/commands/providers.rs`
- `src-tauri/src/services/privacy.rs`
- `src-tauri/src/commands/privacy.rs`
- `src-tauri/src/services/retention.rs`
- `src-tauri/src/commands/diagnostics.rs`
- `src-tauri/src/db/connection.rs`
- `src-tauri/src/db/repositories/history_repo.rs`
- `src-tauri/src/db/repositories/modes_repo.rs`
- `src-tauri/src/db/repositories/providers_repo.rs`
- `src-tauri/src/db/repositories/vocabulary_repo.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/errors/mod.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/commands/mod.rs`
- `src/pages/Dictation.tsx`
- `src/pages/History.tsx`
- `src/pages/Modes.tsx`
- `src/pages/Vocabulary.tsx`
- `src/pages/Models.tsx`
- `src/pages/Providers.tsx`
- `src/pages/Privacy.tsx`
- `src/pages/Diagnostics.tsx`
- `src/pages/About.tsx`
- `docs/PHASES.md`
- `docs/ARCHITECTURE.md`
- `docs/PARALLEL_EXECUTION_PLAN.md`
- `AGENTS.md`
- `CLAUDE.md`
- `SOUL.md`

---

## Privacy and safety rules

- Do not implement real live transcription.
- Do not implement live insertion while speaking.
- Do not implement partial transcript preview.
- Do not read active app contents.
- Do not inspect or capture screen contents.
- Do not use OCR.
- Do not add telemetry.
- Do not send audio, text, clipboard contents, or diagnostics anywhere.
- Do not rewrite AudioService, TranscriptionService, CleanupService, InsertionService, HistoryService.
- Do not rewrite privacy logic.
- Do not change provider API key behavior.
- Do not implement packaging.
- Do not add model downloads.
- Do not add cloud storage.
- Do not add account/login/sync.
- Do not add external icon packages unless implementation proves absolutely impossible without one.
- `ptt-status` messages must never contain transcript text, clipboard contents, or user-dictated content (unchanged rule from Phase 10).
- `ShortcutRecorder` keydown listener must be detached (via `useEffect` cleanup) as soon as recording ends. Keys must not be logged.

---

## Testing plan

### Rust tests (currently 152 — target ≥ 160 after Phase 11)

New tests in `migrations.rs`:
- `test_migration_006_adds_ptt_output_mode_column`
- `test_migration_006_default_value` — assert value is `"clean_before_insert"`
- `test_migration_006_idempotent`

New tests in `settings_repo.rs`:
- `test_settings_repo_ptt_output_mode_default`
- `test_settings_repo_ptt_output_mode_round_trip`
- `test_settings_repo_ptt_output_mode_preserves_other_fields`

Update in `diagnostics.rs` tests: `check_migrations_current` assertion changes from version 5 → 6.

No new tests needed for `ptt.rs` (mode-branch reads from DB and calls existing methods; phase transition logic already covered).

### TypeScript

- `npm run lint` (`tsc --noEmit`) — 0 errors
- `npm run build` — must pass; bundle size should remain comparable

---

## Manual QA plan

Run against a real Tauri dev build with Whisper binary and model configured.

**A. Visual polish**
1. Open app. Sidebar shows SVG icons — no emoji anywhere.
2. Logo visible in sidebar header.
3. Sidebar footer "Phase 1 — UI shell" text is gone.
4. Navigate all 11 pages. No broken navigation.
5. Icon visual consistency: all icons same stroke weight and color.
6. Home page: status rows readable. Badges display correctly. "Start dictating →" link works.

**B. Shortcut recorder**
7. Open Settings → Dictation. "Record" button visible.
8. Click "Record". Display transitions to "Press your shortcut…" (animated indicator).
9. Press Ctrl+Shift+D (Windows) / Cmd+Shift+D (macOS). Display shows "CommandOrControl+Shift+D". "Use this" and "Cancel" buttons appear.
10. Click "Use this". Success message shown. Press the shortcut globally — it triggers correctly.
11. Click "Reset to default". Shortcut returns to `CommandOrControl+Shift+Space`.
12. Click "Record", then "Cancel". No backend call made. Previous shortcut unchanged.
13. Advanced text input still accessible for power users.

**C. PTT speed setting**
14. Settings → Dictation: "PTT output mode" selector visible.
15. Select "Fast — insert raw transcript immediately". Save. Confirm success message.
16. Open Notepad. Press PTT shortcut. Speak. Press shortcut again.
17. Overlay: "Listening…" → "Transcribing audio…" → "Inserting…" → "Done." (no "Cleaning text…" phase).
18. Spoken text inserts into Notepad as raw transcript.
19. Switch to "Best quality". Save. Repeat PTT.
20. Overlay: "Listening…" → "Transcribing audio…" → "Cleaning text…" → "Inserting…" → "Done."
21. Cleaned text inserts into Notepad (takes longer).
22. History entries: both modes store correct `raw_text` and `cleaned_text`.

**D. Overlay timing feedback**
23. Press PTT shortcut. Elapsed timer counts up: "Listening… 0s", "Listening… 1s", "Listening… 2s"…
24. Stop recording. Timer stops counting and disappears (or shows final summary) after Done.

**E. Regressions**
25. `open_dictation` mode: press shortcut, main window opens/focuses, FloatingOverlay appears. PTT overlay does not appear.
26. Cancel during PTT: overlay hides, no text inserted.
27. Error path: remove Whisper binary, trigger PTT. Error in overlay and main window. No crash.
28. Drag overlay while recording → text still inserts into Notepad (focus restore regression).
29. All existing Phase 10.1 QA checklist items pass.

---

## Verification commands

Run in `C:\Users\User\Desktop\transtypro-polish` after implementation:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run lint
npm run build
pwsh scripts/quality-check.ps1
```

Pass criteria:
- `cargo test`: all tests pass, count ≥ 160
- `cargo clippy`: 0 warnings
- `npm run lint`: 0 TypeScript errors
- `npm run build`: success

---

## What is intentionally NOT in Phase 11

- `insert_raw_then_clean_history` PTT mode — deferred (requires async history update after insertion)
- Real audio level meter in PttOverlay (waveform bars remain decorative CSS-only)
- Active app context capture (window title / process name)
- OS file picker for whisper binary/model path
- Language selector wired to real translation backend
- Packaging / distribution (Phase 12+)
- Model download UI

---

## Merge rule

No PR may be merged unless the orchestrator provides the exact line:

```
ORCHESTRATOR APPROVED MERGE
```

## Next step

Launch the Phase 11 implementation agent in the worktree at:

`C:\Users\User\Desktop\transtypro-polish`

Branch: `phase/11-product-polish`

Implementation order recommended:
1. C backend (migration 006, model, settings_repo, ptt.rs, diagnostics version bump)
2. C frontend (`types.ts`, `Settings.tsx` ptt_output_mode section)
3. A (Logo.tsx, icons.tsx, Sidebar.tsx, Home.tsx)
4. B (Settings.tsx shortcut recorder — same file as step 2, continue editing)
5. D (PttOverlay.tsx elapsed timer)
6. Run all verification commands
7. Create `handoff/phase-11-product-polish.md`
8. Open PR against `main`
