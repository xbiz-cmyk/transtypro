# Multi-Agent Orchestration Plan for transtypro

## Goal

Build transtypro faster by using several coding agents without creating chaos.

The key rule:

> Parallelize by ownership area, not by random features.

## Recommended tools

- Antigravity: agent manager and workspace orchestration.
- Claude Code: main builder and specialist agents.
- Codex: independent reviewer, refactor assistant, and test writer.

## Safety setup

Use review-first settings for agent tools.

Recommended:
- Terminal execution: request review for risky commands.
- Review policy: request review or review-driven development.
- Browser JavaScript: request review unless doing simple docs browsing.
- Avoid fully autonomous destructive terminal actions.
- Keep git initialized from the start.

## Branch model

Use this branch structure:

```text
main
phase/00-bootstrap
phase/01-ui-shell
phase/02-storage-settings
phase/03-audio-stt
phase/04-cleanup-providers
phase/05-dictation-pipeline
phase/06-privacy-diagnostics
phase/07-packaging
```

For parallel development, use git worktrees:

```bash
git worktree add ../transtypro-ui phase/01-ui-shell
git worktree add ../transtypro-backend phase/02-storage-settings
git worktree add ../transtypro-audio phase/03-audio-stt
git worktree add ../transtypro-qa phase/06-privacy-diagnostics
```

Only do this after Phase 0 creates the baseline project.

## Agent roles

### 1. Orchestrator agent

Owns:
- architecture
- task splitting
- merge order
- review gates
- docs
- conflict resolution

Does not:
- rewrite everything alone
- allow overlapping edits
- merge untested work

### 2. Frontend UI agent

Owns:
- React layout
- pages
- components
- stores
- UI state
- empty/loading/error states

Avoids:
- Rust service implementation
- database schema changes

### 3. Backend Rust agent

Owns:
- Tauri commands
- Rust services
- typed errors
- command wiring
- provider abstraction

Avoids:
- detailed frontend styling

### 4. Audio/STT agent

Owns:
- microphone capture
- WAV temp file
- whisper.cpp sidecar path
- transcription options
- model manager

Avoids:
- provider billing
- UI page redesign

### 5. Privacy/storage agent

Owns:
- SQLite migrations
- repositories
- settings
- history
- vocabulary
- privacy enforcement

Avoids:
- audio engine internals

### 6. QA/reviewer agent

Owns:
- tests
- lint/type checks
- manual QA
- diagnostics
- bug reports
- release checklist

Avoids:
- adding features without task approval

### 7. Docs/release agent

Owns:
- README
- build docs
- privacy docs
- installer notes
- release checklist

Avoids:
- production code unless doc build requires it

## Parallel work map

Do not parallelize Phase 0.

After Phase 0:

```text
Frontend UI agent       -> UI shell and static pages
Privacy/storage agent   -> SQLite, settings, modes, vocabulary, history
Backend Rust agent      -> Tauri command interfaces and typed errors
QA agent                -> test setup and manual QA checklist
```

After service contracts stabilize:

```text
Audio/STT agent         -> recording and transcription
Backend Rust agent      -> cleanup providers and pipeline
Frontend UI agent       -> connect pages to real commands
Privacy/storage agent   -> local-only enforcement and retention
```

Final phase:

```text
QA agent                -> full verification
Docs/release agent      -> packaging docs
Orchestrator            -> final merge and release candidate
```

## Review gate for every merge

Before merging a branch, require:

- changed files list
- explanation
- tests run
- manual QA notes
- privacy impact
- known limitations
- next steps

## Conflict avoidance

Each agent must write a `handoff/<agent-name>.md` file with:

- branch name
- touched files
- completed work
- commands run
- open questions
- next safe task

## How to use Codex

Use Codex as an independent reviewer:

```text
Read AGENTS.md, SOUL.md, docs/ARCHITECTURE.md, and the diff.

Review this branch for:
- privacy violations
- unhandled errors
- fake implementations
- broken tests
- unclear UI messages
- unsafe commands
- cross-platform issues

Return blocking issues first, then improvements.
```

Use Codex for focused tasks:
- write tests
- refactor one module
- review security
- check TypeScript strictness
- check Rust error handling

Do not let Codex and Claude edit the same files at the same time.
