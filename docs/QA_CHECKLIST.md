# QA Checklist

## Development checks

- [ ] Frontend installs successfully.
- [ ] Frontend dev server starts.
- [ ] Tauri dev app starts.
- [ ] Rust compiles.
- [ ] TypeScript type check passes.
- [ ] Rust format passes.
- [ ] Rust tests pass.
- [ ] Frontend build passes.

## Manual app checks

- [ ] App opens.
- [ ] Sidebar navigation works.
- [ ] Onboarding opens.
- [ ] Home page shows current status.
- [ ] Settings persist after restart.
- [ ] Microphone list loads.
- [ ] Microphone recording creates WAV.
- [ ] Local model path can be selected.
- [ ] Missing model error is useful.
- [ ] Whisper transcription works with a sample WAV.
- [ ] Cleanup disabled mode works.
- [ ] Ollama provider test works when Ollama is running.
- [ ] OpenAI-compatible provider test handles invalid key safely.
- [ ] Local-only mode blocks cloud calls.
- [ ] Text inserts into TextEdit or Notepad.
- [ ] Text inserts into browser input.
- [ ] Clipboard restore works.
- [ ] History saves when enabled.
- [ ] History does not save in no-history mode.
- [ ] Diagnostics report excludes secrets.

## Privacy checks

- [ ] API keys are masked in UI.
- [ ] API keys are not logged.
- [ ] Diagnostics do not include secrets.
- [ ] Local-only mode blocks transcription network calls.
- [ ] Local-only mode blocks cleanup network calls.
- [ ] Temporary audio is deleted when audio history is disabled.

---

## Phase 1 — UI shell

These checks apply to the `phase/01-ui-shell` branch before merging.

### Page existence and navigation

- [ ] Home page exists and renders.
- [ ] Dictation page exists and renders.
- [ ] History page exists and renders.
- [ ] Modes page exists and renders.
- [ ] Vocabulary page exists and renders.
- [ ] Models page exists and renders.
- [ ] Providers page exists and renders.
- [ ] Settings page exists and renders.
- [ ] Privacy page exists and renders.
- [ ] Diagnostics page exists and renders.
- [ ] About page exists and renders.
- [ ] All 11 pages are reachable via sidebar navigation.
- [ ] No sidebar items are dead (every item navigates to a real page).

### Overlay

- [ ] FloatingOverlay component renders without runtime errors.
- [ ] FloatingOverlay is positioned correctly (floating, not inline).
- [ ] FloatingOverlay does not trigger any Tauri invoke() calls.

### Stores

- [ ] All Zustand or context stores initialize without errors.
- [ ] Stores do not call real Tauri invoke() for unimplemented commands.
- [ ] Store initialization does not throw on empty/default state.

### Empty / loading / error states

- [ ] History page shows an empty state when no data is present.
- [ ] Modes page shows an empty state when no data is present.
- [ ] Vocabulary page shows an empty state when no data is present.
- [ ] Providers page shows an empty state when no data is present.
- [ ] Models page shows an empty state when no data is present.
- [ ] All data pages have a visible loading state.
- [ ] All data pages have a visible error state.

### Mock data discipline

- [ ] All mock data is clearly labeled with `// MOCK:` comments in source.
- [ ] No unlabeled hardcoded data that could be mistaken for real storage.

### Build and lint

- [ ] `npm run build` passes with zero errors.
- [ ] `npm run lint` passes (`tsc --noEmit` — configured in package.json).
- [ ] TypeScript type check passes (no `any` without justification).

### Tauri command discipline

- [ ] No real `invoke()` calls for commands that are not yet registered in `lib.rs`.
- [ ] Mock data and placeholder responses used instead of real commands.

---

## Phase 2 — Backend contracts

These checks apply to the `phase/02-backend-contracts` branch before merging.

### Tauri command registration

All of the following commands must be registered in `src-tauri/src/lib.rs`:

**Settings:**
- [ ] `get_settings`
- [ ] `update_settings`

**Modes:**
- [ ] `list_modes`
- [ ] `get_mode`
- [ ] `create_mode`
- [ ] `update_mode`
- [ ] `delete_mode`

**Vocabulary:**
- [ ] `list_vocabulary`
- [ ] `add_vocabulary_entry`
- [ ] `delete_vocabulary_entry`

**History:**
- [ ] `list_history`
- [ ] `get_history_entry`
- [ ] `delete_history_entry`
- [ ] `clear_history`

**Privacy:**
- [ ] `get_privacy_status`
- [ ] `enforce_privacy_preview`

**Providers:**
- [ ] `list_providers`
- [ ] `get_provider`
- [ ] `test_provider_placeholder`

**Diagnostics:**
- [ ] `run_diagnostics_placeholder`

### Error handling

- [ ] All service methods return `Result<T, AppError>`.
- [ ] `AppError` has a `FeatureNotImplemented` variant (or equivalent).
- [ ] All storage-dependent operations return `FeatureNotImplemented` (not panic, not empty vec silently).
- [ ] No `panic!()` in any callable command or service path.
- [ ] No `todo!()` in any callable command or service path.
- [ ] No `unimplemented!()` in any callable command or service path.

### Storage discipline

- [ ] No real SQLite database is opened or created.
- [ ] No real provider API calls are made.
- [ ] No API keys are stored, logged, or exposed.

### Rust quality checks

- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test` passes.

### Frontend build compatibility

- [ ] `npm run build` still passes after any shared type changes.

---

## Phase 2 — Storage (Wave 2 reference)

These checks apply to the `phase/02-storage-settings` branch before merging.
The Database/privacy agent must complete all items below.

### Migrations

- [ ] SQLite migrations run cleanly on a fresh database.
- [ ] Migrations are idempotent (re-running does not corrupt data).

### Settings

- [ ] Settings persist across app restarts.
- [ ] Default settings are applied on first run.
- [ ] Settings update is atomic (no partial writes).

### Modes

- [ ] Modes can be created and appear in the list.
- [ ] Modes can be edited and changes persist.
- [ ] Modes can be deleted and no longer appear.

### Vocabulary

- [ ] Vocabulary entries persist after restart.
- [ ] Duplicate entries are handled gracefully.
- [ ] Entries can be deleted.

### History

- [ ] History entries are created after dictation (once dictation is wired).
- [ ] History entries are listed correctly.
- [ ] History is not created when privacy/no-history mode is on.
- [ ] Individual history entries can be deleted.
- [ ] `clear_history` removes all entries.

### Privacy enforcement

- [ ] Local-only mode blocks all cloud transcription calls at the service level.
- [ ] Local-only mode blocks all cloud cleanup calls at the service level.
- [ ] Privacy status is readable via `get_privacy_status`.

### Retention policy

- [ ] Retention policy deletes entries older than the configured threshold.
- [ ] Retention policy does not delete entries that are within the threshold.

### Repository tests

- [ ] `cargo test` covers settings repository (create, read, update).
- [ ] `cargo test` covers modes repository (CRUD).
- [ ] `cargo test` covers vocabulary repository (add, list, delete).
- [ ] `cargo test` covers history repository (add, list, delete, clear).
- [ ] `cargo test` covers privacy enforcement logic.

---

## General checks (apply to every agent)

These checks must pass before any branch is submitted for orchestrator review.

### Correctness

- [ ] No dead buttons in UI (every button either does something or is visibly disabled).
- [ ] No fake feature claims (partial features clearly labeled as partial).
- [ ] No placeholder URLs used as real model download URLs.

### Security and privacy

- [ ] No secrets or API keys logged anywhere.
- [ ] No secrets or API keys exposed in UI without masking.
- [ ] No cloud calls in local-only mode.

### Ownership boundaries

- [ ] Backend agents did not edit any files under `src/**`.
- [ ] Frontend agents did not edit any files under `src-tauri/**`.
- [ ] QA agent did not edit `src/**` or `src-tauri/src/**`.

### Handoff completeness

- [ ] Handoff file created in `handoff/`.
- [ ] Changed files listed in handoff.
- [ ] Test or check results included in handoff.
- [ ] Known limitations documented.
- [ ] Next recommended step documented.
