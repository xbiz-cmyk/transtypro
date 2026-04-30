# Phase 1 Handoff — Frontend UI Agent

## Branch

`phase/01-ui-shell`

## Summary

Phase 1 builds the complete UI shell for transtypro. All 11 pages are navigable
via the sidebar. No backend commands are called beyond those already registered
in `api.ts` (ping, get_app_version, get_status_summary). All other data is mock.
Reusable components, Zustand stores, and TypeScript type mirrors are in place for
future backend wiring.

---

## Files created

### UI primitive components (`src/components/ui/`)

| File | Description |
|---|---|
| `Card.tsx` | Standard content container with border, padding, dark bg; exports `CardHeader` |
| `Button.tsx` | Variants: primary, secondary, ghost, danger; sizes: sm, md, lg; disabled state |
| `Input.tsx` | Text input with label, helper text, error state |
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
| `Dictation.tsx` | Record button (inactive), mode selector, waveform placeholder, result textarea, Copy/Insert/Save buttons (all disabled) |
| `History.tsx` | Filter bar (search, date range, mode), 3 mock entries with badges, empty state |
| `Modes.tsx` | 5 mock built-in modes with badges, Add/Edit/Delete placeholders |
| `Vocabulary.tsx` | 4 mock entries in term/replacement/category table, Add/Delete placeholders |
| `Models.tsx` | 1 mock installed model, add-model form with file path input and Browse button |
| `Providers.tsx` | 1 mock provider, add-provider form with type selector/URL/model/masked API key field |
| `Settings.tsx` | Grouped cards: General, Dictation, Privacy, Storage with toggles and selectors |
| `Privacy.tsx` | Status card, privacy badges (Local Only, No Cloud Calls, Audio Deleted After Use), data flow table |
| `Diagnostics.tsx` | 7-item status check list, Run diagnostics + Export buttons (disabled), results empty state |
| `About.tsx` | App name, version, tagline, description, local paths placeholder, credits |

---

## Files modified

| File | Change |
|---|---|
| `src/App.tsx` | Added routes for all 11 pages; renders FloatingOverlay |
| `src/components/Sidebar.tsx` | All nav items enabled as NavLink; split into main + bottom groups; About added; footer updated to Phase 1 |
| `src/pages/Home.tsx` | Replaced single status card with 2×2 card grid (active mode, privacy mode, last transcription, quick-start); kept system status section |
| `src/lib/types.ts` | Added: AppSettings, HistoryEntry, DictationMode, VocabularyEntry, AiProvider, ModelEntry, DiagnosticItem, DiagnosticReport, PrivacyStatus; updated NavItem (removed `enabled` field) |
| `package.json` | Added zustand dependency |
| `package-lock.json` | Updated lockfile |

---

## Mock data locations

All mock data is labeled with `// MOCK:` comments.

| File | Location | Description |
|---|---|---|
| `src/pages/History.tsx` | Lines 7–36 | `MOCK_ENTRIES` — 3 sample history entries |
| `src/pages/Modes.tsx` | Lines 5–34 | `MOCK_MODES` — 5 built-in dictation modes |
| `src/pages/Vocabulary.tsx` | Lines 5–25 | `MOCK_VOCABULARY` — 4 vocabulary entries |
| `src/pages/Models.tsx` | Lines 6–15 | `MOCK_MODELS` — 1 installed model |
| `src/pages/Providers.tsx` | Lines 5–16 | `MOCK_PROVIDERS` — 1 configured provider |
| `src/pages/Privacy.tsx` | Lines 5–11 | `MOCK_PRIVACY_STATUS` — privacy state values |
| `src/pages/Diagnostics.tsx` | Lines 5–22 | `MOCK_CHECK_ITEMS` — 7 diagnostic check items |

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

## Commands run and results

| Command | Result |
|---|---|
| `npm install zustand` | ✅ zustand@5 added, 91 packages total |
| `npm run lint` (tsc --noEmit) | ✅ 0 errors after 3 fixes |
| `npm run build` (tsc + vite build) | ✅ Built in 1.89s, 274 kB JS, 24.9 kB CSS |
| `git status --short` | ✅ Clean after all commits |

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

---

## What the backend agent needs to provide

For the frontend to wire real data, the backend (phase/02-backend-contracts) must register:

| Tauri command | Return type | Used by |
|---|---|---|
| `list_modes` | `DictationMode[]` | Modes page, Dictation page mode selector |
| `get_mode` | `DictationMode` | Modes page |
| `create_mode` | `DictationMode` | Modes page Add |
| `update_mode` | `DictationMode` | Modes page Edit |
| `delete_mode` | `()` | Modes page Delete |
| `list_vocabulary` | `VocabularyEntry[]` | Vocabulary page |
| `create_vocabulary_entry` | `VocabularyEntry` | Vocabulary page Add |
| `delete_vocabulary_entry` | `()` | Vocabulary page Delete |
| `list_history` | `HistoryEntry[]` | History page |
| `get_settings` | `AppSettings` | Settings page, Privacy page |
| `update_settings` | `AppSettings` | Settings page Save |
| `list_providers` | `AiProvider[]` | Providers page |
| `create_provider` | `AiProvider` | Providers page Add |
| `delete_provider` | `()` | Providers page Delete |
| `list_models` | `ModelEntry[]` | Models page |
| `add_model` | `ModelEntry` | Models page Add |
| `remove_model` | `()` | Models page Remove |
| `get_privacy_status` | `PrivacyStatus` | Privacy page |
| `run_diagnostics` | `DiagnosticReport` | Diagnostics page |

All return types already exist in `src/lib/types.ts`.

---

## Privacy impact

None. No data leaves the app. No storage written. No cloud calls made.
