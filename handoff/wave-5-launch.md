# Wave 5 Launch: Phase 5 Cleanup Providers

## Base commit

`ec0e72e` — fix(ui): show saved state on whisper settings button

All Wave 1–4 work and hotfixes are merged into main. This handoff is committed on
main before the Phase 5 branch diverges.

## Branch and worktree

| Item       | Value                                          |
| ---------- | ---------------------------------------------- |
| Branch     | `phase/05-cleanup-providers`                   |
| Worktree   | `C:\Users\User\Desktop\transtypro-providers`   |
| Base       | `main` at `ec0e72e`                            |

---

## Phase 5 scope: Cleanup Providers only

Accept raw transcript text from the Phase 4 Dictation page, send it to a
configured cleanup provider (Ollama local or OpenAI-compatible), and return
cleaned text. Store provider configuration in SQLite. Store API keys in the OS
keychain only. Wire the Providers page and extend the Dictation page.

### What Phase 5 MUST implement

- **Migration 003**: `CREATE TABLE IF NOT EXISTS providers (...)` — see schema below
- **ProvidersRepository**: CRUD for the providers table
- **ProvidersService**: list, create, update, delete, test connection, set/get API key via keyring
- **CleanupService**: HTTP calls to Ollama (`/api/generate`) and OpenAI-compatible (`/chat/completions`)
- **Tauri commands**: `list_providers`, `create_provider`, `update_provider`, `delete_provider`,
  `test_provider_connection`, `set_provider_api_key`, `list_enabled_cleanup_providers`, `cleanup_text`
- **New models**: `CleanupResult { cleaned_text, provider_id, provider_name, duration_ms }`
- **New error variants**: `ProviderError(String)`, `CleanupError(String)`
- **New Cargo deps**: `ureq = { version = "2", features = ["json"] }`, `keyring = "3"`
- **Providers.tsx**: replace MOCK_PROVIDERS with real `listProviders()` call; wire Save, Delete, Test
- **Dictation.tsx**: add provider picker and "Clean text" button after transcription
- **api.ts**: add wrappers for all new provider + cleanup commands
- **types.ts**: add `CleanupResult` interface
- **handoff/phase-05-cleanup-providers.md**: post-implementation handoff (required)

### What Phase 5 must NOT implement

- Text insertion (Phase 6)
- Global shortcuts (Phase 7)
- History entry creation (Phase 6)
- End-to-end dictation pipeline (Phase 6)
- Diagnostics (Phase 8)
- Model downloads
- Changes to Whisper transcription logic
- Changes to audio recording logic

---

## Approved architecture

### Migration 003

```sql
CREATE TABLE IF NOT EXISTS providers (
    id                    TEXT    PRIMARY KEY,
    name                  TEXT    NOT NULL,
    provider_type         TEXT    NOT NULL DEFAULT 'ollama',
    base_url              TEXT    NOT NULL DEFAULT '',
    model                 TEXT    NOT NULL DEFAULT '',
    enabled               INTEGER NOT NULL DEFAULT 1,
    use_for_cleanup       INTEGER NOT NULL DEFAULT 1,
    use_for_transcription INTEGER NOT NULL DEFAULT 0,
    api_key_set           INTEGER NOT NULL DEFAULT 0
);
```

No seed rows — providers are user-configured. `api_key_set` is a boolean flag only;
the actual key bytes live in the OS keychain, never in SQLite.

### New Cargo dependencies

```toml
ureq = { version = "2", features = ["json"] }
keyring = "3"
```

- `ureq`: simple blocking HTTP client, no async runtime required.
- `keyring`: OS credential storage — Windows Credential Manager on Windows,
  macOS Keychain on macOS. API keys never touch SQLite or frontend memory.

### API key storage rule (non-negotiable)

```rust
// Store — keyring service name: "transtypro", username: "provider:{uuid}"
let entry = keyring::Entry::new("transtypro", &format!("provider:{id}"))?;
entry.set_password(api_key)?;
// Then set api_key_set = true in DB

// Retrieve (internal only — NEVER returned to frontend)
let entry = keyring::Entry::new("transtypro", &format!("provider:{id}"))?;
let key = entry.get_password()?;

// Delete on provider delete
let entry = keyring::Entry::new("transtypro", &format!("provider:{id}"))?;
let _ = entry.delete_credential(); // ignore error if no key was stored
```

If keychain is unavailable, fail with a clear `AppError::ProviderError("Cannot store
API key: OS keychain unavailable. {detail}")`. **Never fall back to SQLite.**

### Privacy enforcement rule (non-negotiable)

Before ANY HTTP cleanup call, the command must call `PrivacyService::enforce_privacy_preview`:

```rust
let op = PrivacyOperation {
    operation_type: if is_ollama { "local_cleanup" } else { "cloud_cleanup" }.to_string(),
    provider_id: Some(provider_id.clone()),
};
let decision = PrivacyService::new(state.db.clone()).enforce_privacy_preview(op)?;
if !decision.allowed {
    return Err(AppError::PrivacyBlocked(decision.reason));
}
```

- `local_cleanup` is already in `ALLOWED_IN_LOCAL_MODE` in `privacy.rs` — allowed always.
- `cloud_cleanup` is already in `CLOUD_OPS` in `privacy.rs` — blocked when `local_only_mode = true`.

### HTTP call design

**Ollama** — `POST {base_url}/api/generate`:
```json
{ "model": "{model}", "prompt": "{system_prompt}\n\nText to clean:\n{raw_text}", "stream": false }
```
Parse response `.response`.

**OpenAI-compatible** — `POST {base_url}/chat/completions`:
```json
{
  "model": "{model}",
  "messages": [
    {"role": "system", "content": "{system_prompt}"},
    {"role": "user",   "content": "{raw_text}"}
  ]
}
```
Header: `Authorization: Bearer {key_from_keychain}`.
Parse response `.choices[0].message.content`.

**System prompt source**: load active mode's `system_prompt` field from settings.
If empty, use default: `"You are a text cleanup assistant. Fix grammar, punctuation,
and formatting. Return only the cleaned text with no explanation."`

### CleanupResult model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub cleaned_text: String,
    pub provider_id: String,
    pub provider_name: String,
    pub duration_ms: u64,
}
```

### Tauri commands list

**providers** (in `commands/providers.rs`):

| Rust command                   | Args (snake_case Rust)                                          | Returns          |
| ------------------------------ | --------------------------------------------------------------- | ---------------- |
| `list_providers`               | `state`                                                         | `Vec<AiProvider>`|
| `create_provider`              | `name, provider_type, base_url, model, use_for_cleanup, state`  | `AiProvider`     |
| `update_provider`              | `id, name, base_url, model, enabled, use_for_cleanup, state`    | `AiProvider`     |
| `delete_provider`              | `id, state`                                                     | `()`             |
| `test_provider_connection`     | `id, state`                                                     | `String`         |
| `set_provider_api_key`         | `id, api_key, state`                                            | `()`             |
| `list_enabled_cleanup_providers` | `state`                                                       | `Vec<AiProvider>`|

**cleanup** (in `commands/cleanup.rs`):

| Rust command   | Args (snake_case Rust)       | Returns         |
| -------------- | ---------------------------- | --------------- |
| `cleanup_text` | `raw_text, provider_id, state` | `CleanupResult` |

### test_provider_connection behaviour

- Ollama: `GET {base_url}/api/tags` — return "Ollama reachable at {base_url}" on 200,
  or error message if unreachable.
- OpenAI-compatible: `GET {base_url}/models` with `Authorization: Bearer {key}` —
  return "Provider reachable" on 200/401 (401 = URL valid, wrong key),
  or connection error if host unreachable.
- Never send user text in a test call.

### Frontend — Providers.tsx changes

On mount: call `listProviders()`. Display live data.

Add provider form:
- Type selector: `ollama` | `openai_compatible` (remove Anthropic option)
- Base URL input (default `http://localhost:11434` for Ollama)
- Model name input
- Name input
- API key input: shown only for `openai_compatible`, `type="password"`, never pre-filled
- "Save provider" → `createProvider(...)` → if `openai_compatible` and key entered,
  call `setProviderApiKey(id, key)` → clear form → refresh list
- "Delete" → `deleteProvider(id)` → refresh list
- "Test" → `testProviderConnection(id)` → show result inline

### Frontend — Dictation.tsx additions

New state:
```typescript
const [cleanupResult, setCleanupResult] = useState<CleanupResult | null>(null);
const [isCleaning, setIsCleaning] = useState(false);
const [enabledProviders, setEnabledProviders] = useState<AiProvider[]>([]);
const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
```

On mount: call `listEnabledCleanupProviders()`. Auto-select first provider.

After transcription succeeds: show "Clean text" button (hidden if no providers).
Result textarea shows `cleanupResult?.cleaned_text ?? transcriptResult?.raw_text ?? ""`.
Small label below: "Raw transcript" or "Cleaned by {provider_name}".

Reset `cleanupResult` on new record or cancel.

### Connection from Phase 4 to Phase 5

```
Phase 4 output: TranscriptionResult { raw_text: "...", ... }
                        ↓ user clicks "Clean text"
Phase 5 call:   cleanupText(rawText, providerId)
                        ↓
Phase 5 output: CleanupResult { cleaned_text: "...", provider_name: "...", ... }
                        ↓ textarea updates, Copy copies cleaned_text
```

---

## Allowed files for Phase 5 agent

The Phase 5 agent may read any file. It may only write to:

```
src-tauri/src/services/providers.rs              (replace stub)
src-tauri/src/services/cleanup.rs               (new)
src-tauri/src/commands/providers.rs             (replace stubs with real CRUD)
src-tauri/src/commands/cleanup.rs               (new)
src-tauri/src/commands/mod.rs                   (append pub mod cleanup)
src-tauri/src/services/mod.rs                   (append pub mod cleanup)
src-tauri/src/lib.rs                            (register new commands)
src-tauri/src/errors/mod.rs                     (append ProviderError, CleanupError)
src-tauri/src/models/mod.rs                     (append CleanupResult)
src-tauri/src/db/migrations.rs                  (add migration 003)
src-tauri/src/db/repositories/mod.rs            (append pub mod providers_repo)
src-tauri/src/db/repositories/providers_repo.rs (new)
src-tauri/Cargo.toml                            (add ureq, keyring)
src-tauri/Cargo.lock                            (auto-updated)
src/pages/Providers.tsx                         (wire real backend)
src/pages/Dictation.tsx                         (Clean text button + provider picker)
src/lib/api.ts                                  (new provider + cleanup wrappers)
src/lib/types.ts                                (add CleanupResult interface)
handoff/phase-05-cleanup-providers.md           (required post-implementation handoff)
docs/PROGRESS.md                               (only after all checks pass)
docs/TASK_BOARD.md                             (only after all checks pass)
```

---

## Forbidden files for Phase 5 agent

```
src-tauri/src/services/audio.rs
src-tauri/src/commands/audio.rs
src-tauri/src/services/transcription.rs
src-tauri/src/commands/transcription.rs
src-tauri/src/services/history.rs
src-tauri/src/db/repositories/history_repo.rs
src-tauri/src/services/diagnostics.rs
src-tauri/src/commands/diagnostics.rs
src-tauri/src/services/settings.rs             (unless absolutely required — ask orchestrator first)
src-tauri/src/services/privacy.rs              (unless absolutely required — ask orchestrator first)
src/pages/Diagnostics.tsx
src/pages/History.tsx
src/pages/Settings.tsx
src/pages/Models.tsx
docs/PHASES.md
docs/PARALLEL_EXECUTION_PLAN.md
AGENTS.md
CLAUDE.md
SOUL.md
```

---

## Local-first privacy rules

- `cleanup_text` MUST call `PrivacyService::enforce_privacy_preview` before any HTTP call.
- Ollama calls use `operation_type = "local_cleanup"` — allowed in local-only mode.
- OpenAI-compatible calls use `operation_type = "cloud_cleanup"` — BLOCKED in local-only mode.
- No HTTP call of any kind may be made without a passing privacy check.
- Do not implement cloud fallback — if blocked, return `AppError::PrivacyBlocked`.
- Do not call any external HTTP API in Phase 5 except via `CleanupService::cleanup` and
  `ProvidersService::test_connection`, both of which enforce privacy first.

## Cloud cleanup blocking rule

When `local_only_mode = true`:
- `"cloud_cleanup"` is in `CLOUD_OPS` in `privacy.rs` — it will be blocked automatically.
- The command must respect the `PrivacyDecision.allowed = false` result and return an error.
- The frontend must display this error clearly to the user.
- Under no circumstances should the HTTP call proceed after a denial.

## API key storage rule

1. API key values MUST be stored using `keyring = "3"` only.
2. API key values MUST NOT be stored in SQLite — ever.
3. API key values MUST NOT be returned to the frontend — ever.
4. The `api_key_set: bool` column in the `providers` table is the only key-related data in SQLite.
5. The `set_provider_api_key` Tauri command accepts the key from the frontend once, stores it
   in the OS keychain, sets `api_key_set = true` in the DB, and discards the in-memory value.
6. `get_api_key` is an internal service method only — no Tauri command exposes it.

---

## Tauri camelCase argument reminder

Tauri v2 deserialises frontend camelCase arguments to Rust snake_case automatically.

| Rust parameter      | Frontend must pass  |
| ------------------- | ------------------- |
| `provider_type`     | `providerType`      |
| `base_url`          | `baseUrl`           |
| `use_for_cleanup`   | `useForCleanup`     |
| `raw_text`          | `rawText`           |
| `provider_id`       | `providerId`        |
| `api_key`           | `apiKey`            |

Example:
```typescript
// ✅ Correct
invoke("cleanup_text", { rawText: transcript, providerId: selectedId })

// ❌ Wrong — will pass null to Rust
invoke("cleanup_text", { raw_text: transcript, provider_id: selectedId })
```

---

## Merge rule

No PR from this wave may be merged unless the orchestrator (user) provides the exact line:

```
ORCHESTRATOR APPROVED MERGE
```

The agent must open a PR and stop. The agent must not merge its own PR.

---

## Next step

After the Phase 5 branch and worktree are created and pushed, launch the
Cleanup Providers agent inside `C:\Users\User\Desktop\transtypro-providers`
with the prompt in the next section.

---

## Agent prompt (copy-paste for transtypro-providers session)

```
You are the transtypro Cleanup Providers agent (Phase 5).

Your working directory is C:\Users\User\Desktop\transtypro-providers.
Your branch is phase/05-cleanup-providers.

Read these files before writing any code:
- handoff/wave-5-launch.md
- handoff/phase-04-local-transcription.md
- src-tauri/src/services/providers.rs
- src-tauri/src/commands/providers.rs
- src-tauri/src/services/privacy.rs
- src-tauri/src/models/mod.rs
- src-tauri/src/errors/mod.rs
- src-tauri/src/lib.rs
- src-tauri/src/services/mod.rs
- src-tauri/src/commands/mod.rs
- src-tauri/src/db/migrations.rs
- src-tauri/src/db/repositories/mod.rs
- src-tauri/Cargo.toml
- src/pages/Providers.tsx
- src/pages/Dictation.tsx
- src/lib/api.ts
- src/lib/types.ts

Phase 5 goal:
Wire cleanup providers so that after Phase 4 transcription, the user can
optionally clean the raw transcript using a configured Ollama or
OpenAI-compatible provider.

Implement in this order:
1.  Migration 003: CREATE TABLE providers (see wave-5-launch.md for schema)
2.  ProvidersRepository (providers_repo.rs): CRUD, set_api_key_flag
3.  Register providers_repo in db/repositories/mod.rs
4.  ProvidersService (providers.rs): replace stub, add keyring-based key storage
5.  Add ProviderError and CleanupError to errors/mod.rs
6.  Add CleanupResult to models/mod.rs
7.  CleanupService (cleanup.rs): Ollama + OpenAI HTTP calls via ureq, privacy check
8.  commands/providers.rs: replace stubs with real CRUD + test + set_api_key commands
9.  commands/cleanup.rs: cleanup_text command
10. Register cleanup in commands/mod.rs and services/mod.rs
11. Register all new commands in lib.rs
12. Add ureq and keyring to Cargo.toml
13. Update types.ts: add CleanupResult interface
14. Update api.ts: add all new wrappers (listProviders, createProvider, updateProvider,
    deleteProvider, testProviderConnection, setProviderApiKey,
    listEnabledCleanupProviders, cleanupText)
15. Update Providers.tsx: replace MOCK_PROVIDERS with real backend calls
16. Update Dictation.tsx: add provider picker and Clean text button
17. Run: cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test
18. Run: npm run build && npm run lint
19. Run: pwsh scripts/quality-check.ps1
20. Create handoff/phase-05-cleanup-providers.md
21. Commit all changes on phase/05-cleanup-providers
22. Open PR to main — title: "feat(providers): add cleanup providers (phase 5)"
23. Stop. Do not merge.

All constraints from handoff/wave-5-launch.md apply. Read that file first.
```
