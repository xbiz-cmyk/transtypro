# Phase 5: Cleanup Providers — Handoff

## What was built

Full SQLite-backed AI provider management + text cleanup pipeline for transtypro Phase 5.

## Commits on this branch

```
7c71727 style: apply cargo fmt
3f0e0e0 feat(dictation): add cleanup provider picker and Clean text button
687cd8d feat(providers): wire Providers page to real backend
51da596 feat(providers): add TypeScript types and API wrappers for Phase 5
a4a8faa feat(backend): implement ProvidersService, CleanupService, and wire all commands
539af11 feat(deps): add ureq and keyring dependencies
370e2bd feat(models): add ProviderError, CleanupError, CleanupResult
ae12b72 feat(db): add providers repository with CRUD and api_key_set flag
32f735a feat(db): add migration 003 — providers table
```

## Architecture

### Backend (Rust)

**Migration 003** — new `providers` table with columns:
`id, name, provider_type, base_url, model, enabled, use_for_cleanup, use_for_transcription, api_key_set`

**ProvidersRepository** (`src-tauri/src/db/repositories/providers_repo.rs`)  
CRUD operations: `list_all`, `get_by_id`, `insert`, `update`, `delete`, `set_api_key_flag`, `list_enabled_cleanup`.

**ProvidersService** (`src-tauri/src/services/providers.rs`)  
Delegates to `ProvidersRepository` for all CRUD. API keys stored in OS keychain (`keyring = "3"`) under service `"transtypro"`, username `"provider:{id}"`. The key value is never returned to the frontend. Only the `api_key_set` boolean flag lives in SQLite.

`set_api_key` safety order:
1. Verify provider exists in DB (returns `NotFound` immediately if not — no keychain write occurs).
2. Write key to OS keychain.
3. Update `api_key_set = true` in SQLite.
4. If step 3 fails, attempt to delete the just-written keychain entry before returning the original DB error (best-effort rollback — prevents orphan keychain entries).

The `test_set_api_key_missing_provider_returns_not_found` test verifies step 1 using in-memory SQLite with no OS keychain dependency. Rollback behavior in step 4 is not unit-tested because it requires simulating a DB failure after a real keychain write; this is verified manually by confirming `set_api_key_flag` is called after `set_password` and the rollback path (`entry.delete_credential()`) is reachable on error.

**CleanupService** (`src-tauri/src/services/cleanup.rs`)  
- Fetches provider from DB, rejects disabled providers with `CleanupError`
- Enforces privacy via `PrivacyService::enforce_privacy_preview`:
  - Ollama → `"local_cleanup"` (always allowed)
  - OpenAI-compatible → `"cloud_cleanup"` (blocked when `local_only_mode = true`)
- Loads active mode's `system_prompt` from DB; falls back to hardcoded default
- Makes HTTP call via `ureq` (blocking, no async runtime):
  - Ollama: `POST {base_url}/api/generate` — parses `.response`
  - OpenAI-compatible: `POST {base_url}/chat/completions` with Bearer auth — parses `.choices[0].message.content`

**New Tauri commands** (all registered in `lib.rs`):
- `list_providers`, `get_provider`, `create_provider`, `update_provider`, `delete_provider`
- `test_provider_connection`, `set_provider_api_key`
- `list_enabled_cleanup_providers`, `test_provider_placeholder`
- `cleanup_text`

### Frontend (TypeScript + React)

**`src/lib/types.ts`** — Added `CleanupResult` interface  
**`src/lib/api.ts`** — Added 8 API wrapper functions for all new commands  
**`src/pages/Providers.tsx`** — Full rewrite: real backend calls for list/add/delete/test/set-api-key; `api_key_set` badge; API key modal for OpenAI-compatible; no Anthropic option  
**`src/pages/Dictation.tsx`** — Added cleanup provider picker and "Clean text" button; shows cleaned text in result textarea after cleanup

## Tests

91 Rust unit tests — all pass.

New tests added this phase:
- `db::migrations` — 2 tests for migration 003 (table exists, empty on fresh DB)
- `db::repositories::providers_repo` — 12 tests covering full CRUD, api_key_set flag, not-found errors
- `services::cleanup` — 7 tests: prompt construction (default + active mode), Ollama prompt builder, disabled provider rejection, unknown type error, cloud blocked in local-only, Ollama allowed in local-only

## Verification results

```
cargo fmt --check       PASS
cargo clippy -D warnings  PASS
cargo test              PASS (91/91)
npm run lint            PASS (tsc --noEmit)
npm run build           PASS (282 kB JS bundle)
quality-check.ps1       All checks passed
```

## Known limitations

- No OS file picker for provider form fields (manual URL entry only).
- `test_provider_connection` makes a real network call — will fail if Ollama/OpenAI endpoint is not reachable at test time.
- `test_provider_placeholder` delegates to `test_provider_connection` (kept for backward compatibility).
- Cleanup text is not persisted to history (history pipeline is Phase 6).
- `get_status_summary` still returns a static `cleanup_provider: None` — will need real lookup in Phase 6.
- No UI for enabling/disabling providers after creation (update_provider command exists but no form).
- Dictation page cleanup section only shows when at least one enabled cleanup provider exists.

## Files changed

### New files
- `src-tauri/src/db/repositories/providers_repo.rs`
- `src-tauri/src/services/providers.rs`
- `src-tauri/src/services/cleanup.rs`
- `src-tauri/src/commands/cleanup.rs`
- `handoff/phase-05-cleanup-providers.md`

### Modified files
- `src-tauri/Cargo.toml` — added `ureq`, `keyring`
- `src-tauri/src/db/migrations.rs` — migration 003
- `src-tauri/src/db/repositories/mod.rs` — pub mod + pub use for providers_repo
- `src-tauri/src/errors/mod.rs` — ProviderError, CleanupError variants
- `src-tauri/src/models/mod.rs` — CleanupResult struct
- `src-tauri/src/services/mod.rs` — pub mod cleanup, pub use CleanupService
- `src-tauri/src/commands/providers.rs` — full rewrite (real SQLite)
- `src-tauri/src/commands/mod.rs` — pub mod cleanup
- `src-tauri/src/lib.rs` — all Phase 5 commands registered
- `src/lib/types.ts` — CleanupResult interface
- `src/lib/api.ts` — 8 new wrappers
- `src/pages/Providers.tsx` — full rewrite
- `src/pages/Dictation.tsx` — cleanup section added

## Next recommended tasks

1. **Merge this PR** — no blockers
2. **Phase 6: Dictation pipeline** — wire transcript + cleanup into `HistoryService::create_history_entry`; update `get_status_summary` to return active cleanup provider name
3. **OS file picker** — replace manual path input with `tauri-plugin-dialog` for provider form if needed
4. **Provider enable/disable UI** — add toggle on provider card to call `update_provider`
