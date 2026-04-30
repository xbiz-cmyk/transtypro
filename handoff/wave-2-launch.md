# Wave 2 Launch — Handoff

## Base commit

Wave 2 branch created from `main` at commit `84d3a55` — "feat(ui): build phase 1 UI shell".

---

## Worktree and branch

| Agent | Branch | Worktree path | Status |
|---|---|---|---|
| Database/privacy agent | `phase/02-storage-settings` | `C:/Users/User/Desktop/transtypro-storage` | Ready |

Branch pushed to `origin`.

---

## Agent ownership boundaries

### Database/privacy agent — `transtypro-storage` / `phase/02-storage-settings`

May create or edit:
```
src-tauri/src/db/mod.rs
src-tauri/src/db/connection.rs
src-tauri/src/db/migrations.rs
src-tauri/src/db/repositories/settings.rs
src-tauri/src/db/repositories/modes.rs
src-tauri/src/db/repositories/vocabulary.rs
src-tauri/src/db/repositories/history.rs
src-tauri/src/db/repositories/mod.rs
src-tauri/src/services/settings.rs      (replace FeatureNotImplemented stubs with real impl)
src-tauri/src/services/modes.rs         (replace FeatureNotImplemented stubs with real impl)
src-tauri/src/services/vocabulary.rs    (replace FeatureNotImplemented stubs with real impl)
src-tauri/src/services/history.rs       (replace FeatureNotImplemented stubs with real impl)
src-tauri/src/services/privacy.rs       (replace FeatureNotImplemented stubs with real impl)
src-tauri/Cargo.toml                    (add rusqlite, serde_json deps)
```

Must NOT touch:
```
src/**                               (any frontend code)
src-tauri/src/commands/**            (owned by Backend contracts agent — do not modify)
src-tauri/src/models/**              (owned by Backend contracts agent — do not modify)
src-tauri/src/errors/mod.rs          (owned by Backend contracts agent — coordinate if new variants needed)
src-tauri/src/services/diagnostics.rs
src-tauri/src/services/providers.rs
docs/**
handoff/**                           (except adding own handoff file)
scripts/**
tests/**
```

### Coordination rule

If the storage agent needs a new model field or error variant, it must:
1. Document the change in its handoff file.
2. Ask the orchestrator to schedule the change.
3. Never edit `src-tauri/src/models/**` or `src-tauri/src/errors/mod.rs` directly.

---

## What must NOT start yet

| Feature | Phase | Status |
|---|---|---|
| Microphone recording | Phase 3 | Do not start |
| Whisper / local transcription | Phase 4 | Do not start |
| Cleanup providers (Ollama, OpenAI) | Phase 5 | Do not start |
| End-to-end dictation pipeline | Phase 6 | Do not start |
| Global shortcut | Phase 7 | Do not start |
| Active app detection | Phase 7 | Do not start |
| Text insertion / clipboard | Phase 6 | Do not start |
| Real provider API calls | Phase 5 | Do not start |
| Diagnostics export | Phase 8 | Do not start |
| Voice Inbox | Phase 9 | Do not start |
| Packaging / signing | Phase 10 | Do not start |
| Audio/STT agent | Wave 3 | Must wait for Phase 2 merge |

---

## What the storage agent should implement

1. SQLite connection module (`src-tauri/src/db/connection.rs`)
   - Open/create the database in the OS app data directory
   - Return a connection pool or single connection wrapped in `Arc<Mutex<Connection>>`

2. Migration runner (`src-tauri/src/db/migrations.rs`)
   - Run schema migrations on startup
   - Tables: `settings`, `modes`, `vocabulary`, `history`

3. Repositories (one file per domain):
   - `settings` — get/upsert app settings row
   - `modes` — CRUD for dictation modes
   - `vocabulary` — CRUD for vocabulary entries
   - `history` — insert, list (with pagination), delete, purge by retention policy

4. Wire services:
   - Replace `FeatureNotImplemented` stubs in `settings.rs`, `modes.rs`, `vocabulary.rs`, `history.rs`, `privacy.rs` with real repository calls
   - Inject repository via service constructor or via `AppState`

5. Privacy enforcement in `privacy.rs`:
   - `enforce_operation` checks `local_only_mode` flag from settings
   - Returns `PrivacyBlocked` when a cloud operation is attempted in local-only mode

---

## Checks before handoff

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git status --short
git diff --stat
```

---

## Lock file rule

- `Cargo.lock` — only this agent may modify (for adding rusqlite and related deps).
- `package-lock.json` — must not be touched.

---

## Merge order (for reference — do not merge until orchestrator approves)

```
1. phase/02-storage-settings → main   (this agent)
```

No PR may be merged unless the user provides the exact line:

ORCHESTRATOR APPROVED MERGE
