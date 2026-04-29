# AGENTS.md: Shared Instructions for AI Coding Agents

This file is used by Codex and other coding agents.

Claude Code reads `CLAUDE.md`, which imports this file.

## Project

Build `transtypro`, a cross-platform local-first AI dictation desktop app for Windows and macOS.

Stack:
- Tauri v2
- Rust backend
- React frontend
- TypeScript
- Tailwind CSS
- SQLite
- whisper.cpp-compatible local speech engine
- Optional Ollama-compatible local LLM cleanup
- Optional OpenAI-compatible cloud provider

## Non-negotiable rules

- Do not build a mock-only app.
- Do not leave dead buttons.
- Do not hardcode secrets.
- Do not log API keys.
- Do not send data to cloud when local-only mode is enabled.
- Do not implement destructive file operations without explicit review.
- Do not edit unrelated files.
- Do not mix many large changes in one commit.
- Do not let two agents edit the same file area at the same time.
- Do not use placeholder URLs for model downloads as real production URLs.
- Do not claim a feature works unless it was tested or clearly marked as partial.

## Repository discipline

- Keep commits small and meaningful.
- Use feature branches or worktrees for parallel work.
- Each phase must have:
  - implementation notes
  - tests or manual verification
  - known limitations
  - changed files list
- Update docs when behavior changes.

## Preferred commands

Use these commands when available:

```bash
npm install
npm run dev
npm run build
npm run lint
npm run test
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

If the project uses `pnpm` instead of `npm`, update this file and package scripts.

## Rust rules

- Use typed errors.
- Keep OS-specific logic behind service interfaces.
- Never panic for normal user errors.
- Prefer `Result<T, AppError>`.
- Do not put all logic in `main.rs`.
- Keep Tauri commands thin.
- Business logic belongs in services.
- Database logic belongs in repositories.
- Add tests for pure logic.

## TypeScript rules

- Use strict TypeScript.
- Avoid `any`.
- Keep API types in `src/lib/types.ts`.
- Keep Tauri command wrappers in `src/lib/api.ts`.
- Keep UI components small and reusable.
- Do not duplicate business rules in every page.

## UI rules

- Dark theme default.
- Light theme optional.
- Clear empty states.
- Clear loading states.
- Clear error states.
- Sidebar navigation.
- Floating overlay for dictation.
- Use simple language in user-facing messages.
- Make privacy state visible.

## Security and privacy rules

- API keys must be stored encrypted or through the OS keychain when implemented.
- Mask API keys in UI.
- Local-only mode must block all cloud transcription and cleanup calls.
- Temporary audio must be deleted unless user enables audio history.
- Clipboard must be restored when setting is enabled.
- Diagnostics must never export secrets or full private history.

## Testing rules

Before claiming a phase complete, run available checks.

Minimum checks:
- frontend build
- Rust format
- Rust tests
- TypeScript type check
- manual test notes

## File ownership for parallel agents

Use these boundaries:

- Frontend agent:
  - `src/**` (standard Tauri v2 layout, see ADR-002)
  - UI docs only
  - Do not create a `frontend/` folder unless the orchestrator explicitly approves it.

- Backend agent:
  - `src-tauri/src/services/**`
  - `src-tauri/src/commands/**`
  - `src-tauri/src/models/**`

- Database/privacy agent:
  - `src-tauri/src/db/**`
  - privacy services
  - history/vocabulary repositories

- Audio/STT agent:
  - audio capture
  - transcription service
  - model service
  - whisper sidecar integration

- QA/docs agent:
  - tests
  - docs
  - scripts
  - diagnostics

If a task crosses boundaries, create a plan first and ask the orchestrator to split it.
