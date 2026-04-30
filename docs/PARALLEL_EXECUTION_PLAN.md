# Parallel Execution Plan

Post-Phase 0 multi-agent launch plan for transtypro.

Phase 0 (bootstrap) is merged into `main`. The project skeleton is ready.
This document defines how to run Phase 1 and Phase 2 in parallel without conflicts.

---

## 1. Recommended Agent List

### Wave 1 — Start immediately after approval

| Agent | Role | Phase | Branch |
|---|---|---|---|
| **Frontend UI agent** | Build all pages, components, stores, overlay | Phase 1 | `phase/01-ui-shell` |
| **Backend Rust agent** | Tauri commands, service interfaces, typed errors | Phase 2 (backend contracts) | `phase/02-storage-settings` |

### Wave 2 — Start after Wave 1 contracts stabilize

| Agent | Role | Phase | Branch |
|---|---|---|---|
| **Database/privacy agent** | SQLite migrations, repositories, settings | Phase 2 (storage) | `phase/02-storage-settings` (same branch, after backend agent) |
| **QA agent** | Test harness, manual QA checklist, type checks | Cross-phase | `phase/qa-setup` |

### Wave 3 — Start after Phase 2 is merged

| Agent | Role | Phase | Branch |
|---|---|---|---|
| **Audio/STT agent** | Microphone recording, whisper sidecar | Phase 3–4 | `phase/03-audio-stt` |
| **Backend Rust agent** | Cleanup providers, pipeline | Phase 5–6 | `phase/04-cleanup-providers` |

### Not started yet

| Agent | Waits for |
|---|---|
| Docs/release agent | Phase 8+ |
| Audio/STT agent | Phase 2 merge |
| Privacy enforcement | Phase 2 merge + Phase 5 |

---

## 2. Branch Names

```text
main                        ← current baseline (Phase 0 merged)
phase/01-ui-shell           ← Frontend UI agent
phase/02-storage-settings   ← Backend Rust agent → then Database/privacy agent
phase/qa-setup              ← QA agent (optional, lightweight)
```

Future branches (do not create yet):
```text
phase/03-audio-stt
phase/04-cleanup-providers
phase/05-dictation-pipeline
phase/06-privacy-diagnostics
phase/07-packaging
```

---

## 3. Ownership Boundaries

### Frontend UI agent — `phase/01-ui-shell`

Owns (may create/edit):
```
src/pages/**                 ← all page components
src/components/**            ← all reusable UI components
src/stores/**                ← state management
src/App.tsx                  ← router and layout
src/index.css                ← design tokens and global styles
src/lib/types.ts             ← add UI-specific types only
```

Must NOT touch:
```
src-tauri/**                 ← any Rust code
src/lib/api.ts               ← only add stubs for commands that exist
docs/**                      ← except UI-specific docs
```

### Backend Rust agent — `phase/02-storage-settings`

Owns (may create/edit):
```
src-tauri/src/commands/**    ← Tauri command wrappers
src-tauri/src/services/**    ← business logic services
src-tauri/src/models/**      ← Rust data structs
src-tauri/src/errors/mod.rs  ← add error variants
src-tauri/src/lib.rs         ← register new commands
src-tauri/Cargo.toml         ← add dependencies
```

Must NOT touch:
```
src/**                       ← any frontend code
src-tauri/src/db/**          ← reserved for Database agent
```

### Database/privacy agent — `phase/02-storage-settings` (after backend)

Owns (may create/edit):
```
src-tauri/src/db/**          ← migrations, repositories, connection
```

Must coordinate with Backend agent on:
```
src-tauri/src/services/**    ← service implementations that call repositories
src-tauri/src/models/**      ← shared data structs
src-tauri/Cargo.toml         ← SQLite dependency
```

### QA agent — `phase/qa-setup`

Owns (may create/edit):
```
tests/**                     ← test files (if created)
scripts/**                   ← QA scripts
docs/QA_CHECKLIST.md         ← update checklist
```

Must NOT touch:
```
src/**                       ← frontend code
src-tauri/src/**             ← backend code
```

---

## 4. Merge Order

```
1. phase/01-ui-shell          → main    (Frontend UI agent)
2. phase/02-storage-settings  → main    (Backend + Database agents)
3. phase/qa-setup             → main    (QA agent, if applicable)
```

Rules:
- Phase 1 and Phase 2 can be developed in parallel.
- Phase 1 merges first because it has no backend dependencies.
- Phase 2 merges second because it adds Rust dependencies and database schema.
- If Phase 2 is ready before Phase 1, it may merge first only if there are no file conflicts.
- The orchestrator reviews every merge.

---

## 5. Conflict Prevention Rules

1. **No two agents edit the same file at the same time.**
   - `src/lib/api.ts` is owned by the Frontend agent in Phase 1.
   - `src/lib/types.ts` may be edited by both agents but on different lines. If both need to edit, coordinate through the orchestrator.

2. **Shared file protocol:**
   - If an agent needs to modify a file outside its ownership, it must:
     a. Document the needed change in its handoff file.
     b. Ask the orchestrator to schedule the change.
     c. Never edit the file directly.

3. **Each agent branches from `main` at the same commit (`ad0678d`).**

4. **Each agent runs all checks before handoff:**
   - Frontend: `npm run build`, `npm run lint`
   - Backend: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`
   - Both: `git status --short`, `git diff --stat`

5. **Lock files:**
   - `package-lock.json` — only the Frontend agent may modify (when adding npm deps).
   - `Cargo.lock` — only the Backend/Database agent may modify (when adding Rust deps).

6. **No agent may modify:**
   - `CLAUDE.md`, `AGENTS.md`, `SOUL.md` — orchestrator only.
   - `docs/ORCHESTRATION_PLAN.md`, `docs/PHASES.md` — orchestrator only.
   - `.gitignore`, `.gitattributes` — orchestrator only.

---

## 6. First Task for Each Agent

### Frontend UI agent — Phase 1

**Goal:** Build the full UI shell with all pages navigable.

Create:
- All pages from PHASES.md Phase 1: Home (update), Dictation, History, Modes, Vocabulary, Models, Providers, Settings, Privacy, Diagnostics
- Floating overlay component (positioned, not functional)
- Reusable card, button, input, badge, modal components
- State stores (if using Zustand or React context)
- Enable all sidebar navigation items
- Empty/loading/error states for every page
- Mock data clearly labeled with `// MOCK:` comments

Do NOT:
- Implement real backend calls (use mock data)
- Add real Tauri commands
- Touch Rust code
- Implement actual recording, transcription, or text insertion

### Backend Rust agent — Phase 2 (contracts)

**Goal:** Define all Tauri command interfaces and service contracts.

Create:
- Settings service interface and commands (`get_settings`, `update_settings`)
- Modes service interface and commands (`list_modes`, `get_mode`, `create_mode`, `update_mode`, `delete_mode`)
- Vocabulary service interface and commands
- History service interface and commands
- Privacy service interface with `enforce_operation` pattern
- Add new error variants to `AppError`
- Register all new commands in `lib.rs`
- Do NOT implement SQLite persistence yet — return `FeatureNotImplemented` for data-dependent operations until Database agent wires the repositories

Do NOT:
- Implement SQLite schema or migrations
- Touch frontend code
- Implement audio or transcription services

### QA agent — Setup

**Goal:** Establish testing infrastructure.

Create:
- Update `docs/QA_CHECKLIST.md` with Phase 1 and Phase 2 items
- Create `scripts/quality-check.ps1` (Windows) with all safe commands
- Verify all existing checks still pass on `main`

Do NOT:
- Add features
- Edit source code
- Touch ownership boundaries

---

## 7. Review Gates

### Before any branch is merged, the orchestrator requires:

| Item | Required |
|---|---|
| Changed files list | ✅ |
| What changed and why | ✅ |
| Tests/checks run and results | ✅ |
| Manual QA notes | ✅ |
| Privacy impact statement | ✅ |
| Known limitations | ✅ |
| Next recommended step | ✅ |
| Handoff file created | ✅ |
| No `todo!()` / `panic!()` in callable paths | ✅ |
| No dead buttons | ✅ |
| No fake feature claims | ✅ |

### Review process:
1. Agent creates PR or reports completion.
2. Orchestrator reviews the handoff file and diff.
3. Orchestrator runs checks on the branch.
4. Orchestrator may request Codex review for security/quality.
5. Orchestrator merges or requests changes.

---

## 8. Commands Each Agent Must Run Before Handoff

### Frontend UI agent
```bash
npm run build
npm run lint
git status --short
git diff --stat
```

### Backend Rust agent
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run build        # ensure frontend still builds with any shared changes
git status --short
git diff --stat
```

### Database/privacy agent
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git status --short
git diff --stat
```

### QA agent
```bash
npm run build
npm run lint
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git status --short
git diff --stat
```

---

## 9. What Must NOT Be Implemented Yet

| Feature | Phase | Status |
|---|---|---|
| Microphone recording | Phase 3 | ❌ Do not start |
| Whisper transcription | Phase 4 | ❌ Do not start |
| Cleanup providers (Ollama, OpenAI) | Phase 5 | ❌ Do not start |
| End-to-end dictation pipeline | Phase 6 | ❌ Do not start |
| Global shortcut | Phase 7 | ❌ Do not start |
| Active app detection | Phase 7 | ❌ Do not start |
| Text insertion / clipboard | Phase 6 | ❌ Do not start |
| Privacy enforcement logic | Phase 8 | ❌ Do not start |
| Diagnostics export | Phase 8 | ❌ Do not start |
| Voice Inbox | Phase 9 | ❌ Do not start |
| Packaging / signing | Phase 10 | ❌ Do not start |
| SQLite database | Phase 2 (DB agent) | ❌ Wait for Backend contracts |
| Real model downloads | Phase 4 | ❌ Do not use placeholder URLs |
| Cloud API calls | Phase 5 | ❌ Do not start |
