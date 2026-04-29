# transtypro Architecture

## High-level flow

```text
Global shortcut
  -> Audio recording
  -> Temporary WAV
  -> Speech-to-text
  -> Vocabulary replacement
  -> Optional AI cleanup
  -> Text insertion
  -> History save
```

## Frontend

React + TypeScript.

Main responsibilities:
- pages
- settings forms
- model manager UI
- dictation controls
- overlay state
- history and voice inbox
- privacy status
- diagnostics

## Backend

Rust inside Tauri.

Main responsibilities:
- microphone recording
- filesystem paths
- SQLite
- model management
- STT process execution
- provider API calls
- privacy enforcement
- clipboard and paste operations
- active app context detection
- diagnostics

## Service boundaries

```text
commands/     thin Tauri command wrappers
services/     business logic
db/           migrations and repositories
models/       Rust data structs
errors/       typed errors
utils/        shared helpers
```

## Data storage

Use OS app data directory:

- macOS: `~/Library/Application Support/transtypro`
- Windows: `%APPDATA%/transtypro`

Subfolders:
- `models/`
- `audio/`
- `logs/`
- `exports/`

Database:
- `transtypro.sqlite`

## Privacy enforcement

All cloud-capable services must call privacy enforcement before any network request.

Required pattern:

```rust
privacy_service.enforce_operation(Operation::CloudCleanup)?;
```

If local-only mode is enabled, this must return an error before the request is made.

## Provider abstraction

Provider types:
- none
- local_ollama
- openai_compatible
- custom_http

## Text insertion strategy

Default:
1. Save existing clipboard.
2. Put generated text on clipboard.
3. Simulate paste shortcut.
4. Restore previous clipboard if enabled.

Fallback:
- leave final text copied
- show notification

## OS-specific interfaces

Keep OS-specific code behind traits or modules:

- active app detection
- global shortcut behavior
- clipboard/paste
- startup registration
- permissions

## Build output

The app must have:
- development mode
- production build
- package config
- clear docs
