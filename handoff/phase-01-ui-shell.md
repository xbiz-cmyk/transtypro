# Phase 1 Handoff — Frontend UI Agent

## Branch

`phase/01-ui-shell`

## Summary

Phase 1 builds the complete UI shell for transtypro. All 11 pages are navigable
via the sidebar. No backend commands are called beyond those already registered
in `api.ts` (ping, get_app_version, get_status_summary). All other data is mock.
Reusable components, Zustand stores, and TypeScript type mirrors are in place for
future backend wiring. Types are aligned with the Rust models defined in PR #5
(phase/02-backend-contracts).

No changes to `src-tauri/**`. No changes to `docs/PROGRESS.md` or `docs/TASK_BOARD.md`.

---

## Files created

### UI primitive components (`src/components/ui/`)

| File | Description |
|---|---|
| `Card.tsx` | Standard content container with border, padding, dark bg; exports `CardHeader` |
| `Button.tsx` | Variants: primary, secondary, ghost, danger; sizes: sm, md, lg; disabled state |
| `Input.tsx` | Text input with label, helper text, error state |
| `Textarea.tsx` | Multiline text input with label, helper text, error state, disabled state |
| `Select.tsx` | Styled select element with label, helper text, error state, disabled state |
| `Toggle.tsx` | Toggle switch (role="switch") with label, description, disabled state |
| `Badge.tsx` | Variants: default, success, warning, danger, muted |
| `Modal.tsx` | Backdrop + panel shell; header, body, footer; not wired to state |
| `EmptyState.tsx` | Icon, heading, subtext, optional action button |
| `LoadingSpinner.tsx` | Centered animated spinner; sizes: sm, md, lg |
| `ErrorMessage.tsx` | Error display with icon and message |

### Feature component

| File | Description |
|---|---|
| `src/components/FloatingOverlay.tsx` | Fixed-position overlay; visible/hidden driven by uiStore.overlayOpen; shows mode name and pulse indicator |

### Stores (`src/stores/`)

| File | State shape |
|---|---|
| `uiStore.ts` | `overlayOpen: boolean`, `toggleOverlay()`, `activeMode: string`, `setActiveMode(mode)` |
| `settingsStore.ts` | `settings: Partial<AppSettings>`, `isLoading: boolean`, `error: string \| null` |
| `historyStore.ts` | `entries: HistoryEntry[]`, `isLoading: boolean`, `error: string \| null` |
| `modesStore.ts` | `modes: DictationMode[]`, `isLoading: boolean`, `error: string \| null` |

### Pages (`src/pages/`)

| File | Description |
|---|---|
| `Dictation.tsx` | Record button (inactive), mode Select, waveform placeholder, result Textarea (readOnly), Copy/Insert/Save buttons (all disabled) |
| `History.tsx` | Filter bar (Input search, Select date range, Select mode), 3 mock entries with badges, empty state |
| `Modes.tsx` | 5 mock built-in modes with badges, Add/Edit/Delete placeholders |
| `Vocabulary.tsx` | 4 mock entries in term/replacement/category table, Add/Delete placeholders |
| `Models.tsx` | 1 mock installed model, add-model form with file path input and Browse button |
| `Providers.tsx` | 1 mock provider, add-provider form with Select type/URL/model/masked API key |
| `Settings.tsx` | Grouped cards: General (Select), Dictation (Select), Privacy (Toggle, Input), Storage |
| `Privacy.tsx` | Status card using PrivacySummary fields, privacy badges, data flow table, retention summary |
| `Diagnostics.tsx` | 7-item DiagnosticCheck list, Run diagnostics + Export buttons (disabled), results empty state |
| `About.tsx` | App name, version, tagline, description, local paths placeholder, credits |

---

## Files modified

| File | Change |
|---|---|
| `src/App.tsx` | Added routes for all 11 pages; renders FloatingOverlay |
| `src/components/Sidebar.tsx` | All nav items enabled as NavLink; split into main + bottom groups; About added; footer updated to Phase 1 |
| `src/pages/Home.tsx` | Replaced single status card with 2×2 card grid (active mode, privacy mode, last transcription, quick-start); kept system status section |
| `src/lib/types.ts` | Types aligned with PR #5 backend contracts (see below); added Textarea/Select/Toggle; removed PrivacyStatus (replaced by PrivacySummary) |
| `package.json` | Added zustand dependency |
| `package-lock.json` | Updated lockfile |

---

## Type alignment with PR #5 backend contracts

All types in `src/lib/types.ts` now match the Rust structs in `src-tauri/src/models/mod.rs`.

| Frontend type | Changes from initial version |
|---|---|
| `AppSettings` | Fields: `active_mode`, `local_only_mode`, `theme`, `retention_days`, `audio_history_enabled`, `clipboard_restore_enabled`. Removed: `language`, `default_mode`, `shortcut`, `privacy_mode`, `db_path` |
| `DictationMode` | `id: string` (was number); `active` (was `is_active`); `builtin` (was `is_builtin`) |
| `VocabularyEntry` | `id: string` (was number); added `enabled: boolean` |
| `HistoryEntry` | `id: string` (was number); `mode_used` (was `mode`); `timestamp: string` (was `created_at: number`); `was_inserted` (was `cleanup_applied`); removed `duration_secs`, `provider_name` |
| `AiProvider` | `id: string` (was number); `model` (was `model_name`); `enabled` (was `is_active`); added `use_for_cleanup`, `use_for_transcription`, `api_key_set`; removed `api_key_hint`, `ProviderType` union |
| `DiagnosticItem` | Renamed to `DiagnosticCheck`; `name` (was `label`); `message` (was `detail`); `status: string` (was union) |
| `DiagnosticReport` | `checks: DiagnosticCheck[]` (was `items: DiagnosticItem[]`); `generated_at: string` (was number) |
| `PrivacyStatus` | Replaced by `PrivacySummary`: `local_only_mode`, `audio_retention_days`, `history_retention_days`, `cloud_allowed`, `reason` |
| New: `PrivacyOperation` | `{ operation_type: string, provider_id: string \| null }` |
| New: `PrivacyDecision` | `{ allowed: boolean, reason: string }` |

---

## Mock data locations

All mock data is labeled with `// MOCK:` comments.

| File | Variable | Description |
|---|---|---|
| `src/pages/History.tsx` | `MOCK_ENTRIES` | 3 sample history entries (aligned to HistoryEntry fields) |
| `src/pages/Modes.tsx` | `MOCK_MODES` | 5 built-in dictation modes (aligned to DictationMode fields) |
| `src/pages/Vocabulary.tsx` | `MOCK_VOCABULARY` | 4 vocabulary entries (aligned to VocabularyEntry fields) |
| `src/pages/Models.tsx` | `MOCK_MODELS` | 1 installed model |
| `src/pages/Providers.tsx` | `MOCK_PROVIDERS` | 1 configured provider (aligned to AiProvider fields) |
| `src/pages/Privacy.tsx` | `MOCK_PRIVACY_SUMMARY` | Privacy state (PrivacySummary fields) |
| `src/pages/Diagnostics.tsx` | `MOCK_CHECK_ITEMS` | 7 diagnostic check items (DiagnosticCheck fields) |

---

## Tauri commands called

Only commands already registered in `src/lib/api.ts` are called:

| Command | Called from | Purpose |
|---|---|---|
| `ping` | `Home.tsx` | Backend IPC health check |
| `get_app_version` | `App.tsx` | Display version in status bar |
| `get_status_summary` | `App.tsx`, `Home.tsx` | Privacy mode and status summary |

All other pages use mock data. Every location that should call a backend command
has a `// TODO: wire to backend` comment.

---

## Registered Tauri commands the frontend will wire (from PR #5)

These commands are registered in the backend (PR #5). The frontend will call them in Phase 2+:

| Tauri command | Frontend return type | Used by |
|---|---|---|
| `list_modes` | `DictationMode[]` | Modes page, Dictation page mode selector |
| `get_mode` | `DictationMode` | Modes page |
| `create_mode` | `DictationMode` | Modes page Add |
| `update_mode` | `DictationMode` | Modes page Edit |
| `delete_mode` | `void` | Modes page Delete |
| `list_vocabulary` | `VocabularyEntry[]` | Vocabulary page |
| `add_vocabulary_entry` | `VocabularyEntry` | Vocabulary page Add |
| `update_vocabulary_entry` | `VocabularyEntry` | Vocabulary page Edit |
| `delete_vocabulary_entry` | `void` | Vocabulary page Delete |
| `list_history` | `HistoryEntry[]` | History page |
| `get_history_entry` | `HistoryEntry` | History page detail |
| `delete_history_entry` | `void` | History page delete |
| `clear_history` | `void` | Settings page |
| `get_settings` | `AppSettings` | Settings page, Privacy page |
| `update_settings` | `void` | Settings page Save |
| `list_providers` | `AiProvider[]` | Providers page |
| `get_provider` | `AiProvider` | Providers page |
| `test_provider_placeholder` | `string` | Providers page Test connection button |
| `get_privacy_status` | `PrivacySummary` | Privacy page |
| `enforce_privacy_preview` | `PrivacyDecision` | Privacy enforcement check |
| `run_diagnostics_placeholder` | `DiagnosticReport` | Diagnostics page Run button |

Note: no `create_provider`, `delete_provider`, `list_models`, `add_model`, or `remove_model`
commands were registered in PR #5. Those pages remain fully mock.

---

## Commands run and results

| Command | Result |
|---|---|
| `npm install zustand` | ✅ zustand@5 added |
| `npm run lint` (tsc --noEmit) | ✅ 0 errors |
| `npm run build` (tsc + vite build) | ✅ Pass |
| `pwsh scripts/quality-check.ps1` | ✅ Frontend checks pass |

---

## Known limitations

- No real Tauri invoke() calls beyond ping, get_app_version, get_status_summary
- No SQLite persistence (Phase 2)
- Record button on Dictation page is disabled (Phase 3)
- Text insertion button is disabled (Phase 6)
- Shortcuts are display-only (Phase 7)
- Privacy enforcement is mock (Phase 8)
- Diagnostics Run button is disabled (Phase 8)
- FloatingOverlay can be toggled via uiStore but is not triggered by any shortcut yet (Phase 7)
- Providers API key field displays only and stores nothing
- About page paths are placeholder strings, not real system paths
- No create_provider or delete_provider commands exist yet — Providers page Add/Delete remain disabled
- Models page (add_model, remove_model) has no backend contract yet

---

## Privacy impact

None. No data leaves the app. No storage written. No cloud calls made.
