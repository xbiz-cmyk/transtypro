# Multi-Agent Launch Plan — Handoff

## Current repo state

- **Branch:** `main`
- **HEAD:** `ad0678d` — Merge pull request #1 from xbiz-cmyk/phase/00-bootstrap
- **Status:** Clean, working tree clean
- **Local branches:** `main` only (`phase/00-bootstrap` deleted after merge)
- **Remote branches:** `origin/main`, `origin/phase/00-bootstrap` (kept for reference)

### What exists

- Tauri v2 + React 19 + TypeScript + Vite 7 skeleton
- Tailwind CSS v4 dark theme
- Rust backend: `commands/`, `services/`, `errors/`, `models/`, `db/`, `utils/`
- Frontend: `App.tsx`, `Sidebar.tsx`, `StatusBar.tsx`, `Home.tsx`
- IPC verified via `ping` command
- All checks pass

### What does NOT exist yet

- Pages beyond Home
- SQLite database
- Audio recording
- Transcription
- Cleanup providers
- Real settings
- Global shortcut
- Text insertion

---

## Branch strategy

### Create on launch (Wave 1)

```bash
git checkout main
git checkout -b phase/01-ui-shell           # Frontend UI agent
git push -u origin phase/01-ui-shell

git checkout main
git checkout -b phase/02-backend-contracts  # Backend contracts agent
git push -u origin phase/02-backend-contracts
```

### Optional (Wave 1)

```bash
git checkout main
git checkout -b phase/qa-setup              # QA agent
git push -u origin phase/qa-setup
```

### Create on launch (Wave 2) — after phase/02-backend-contracts is merged or reviewed

```bash
git checkout main
git checkout -b phase/02-storage-settings   # Database/privacy agent
git push -u origin phase/02-storage-settings
```

### Do NOT create yet

- `phase/03-audio-recording`
- `phase/04-local-transcription`
- `phase/05-cleanup-providers`
- `phase/06-dictation-pipeline`
- `phase/07-shortcuts-context`
- `phase/08-privacy-diagnostics`
- `phase/09-voice-inbox`
- `phase/10-packaging`

---

## Which agents to start first

### Wave 1 — Safe to launch in parallel

| Agent | Branch | First task | Estimated scope |
|---|---|---|---|
| Frontend UI agent | `phase/01-ui-shell` | Create all 10 pages, floating overlay component, reusable components, stores, enable sidebar nav | Medium — UI only, no backend deps |
| Backend contracts agent | `phase/02-backend-contracts` | Define all Tauri command interfaces, service traits, error variants, model structs | Medium — Rust only, no DB deps |

**Why these two are safe in parallel:**
- Frontend agent only touches `src/**`
- Backend contracts agent only touches `src-tauri/src/**` (excluding `db/`)
- Zero file overlap
- No shared dependencies other than types (coordinated via handoff)

### Optional Wave 1

| Agent | Branch | First task |
|---|---|---|
| QA agent | `phase/qa-setup` | Create quality-check script, update QA checklist, verify existing checks |

---

## Which agents must wait

| Agent | Waits for | Reason |
|---|---|---|
| Database/privacy agent | `phase/02-backend-contracts` merged or reviewed | Needs service interfaces defined before writing repositories |
| Audio/STT agent | Phase 2 merged into main | Needs SQLite and settings for model management |
| Cleanup provider agent | Phase 2 merged + Phase 3 complete | Needs audio pipeline and privacy service |

---

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Frontend agent adds types to `src/lib/types.ts` that conflict with Backend agent's model additions | Medium | Types.ts is Frontend-owned. Backend defines models in Rust only. Frontend syncs types from Rust models at merge time. |
| Backend agent needs to update `src/lib/api.ts` with new command wrappers | Medium | Backend agent documents needed API additions in handoff. Frontend agent or orchestrator adds them. |
| SQLite crate decision (ADR-001) blocks Database agent | Low | Decision made at Phase 2 start before DB agent launches. Backend agent proposes, orchestrator approves. |
| Worktree conflicts if multiple agents modify `package-lock.json` or `Cargo.lock` | Low | Only one agent per lock file. Frontend owns `package-lock.json`, Backend owns `Cargo.lock`. |
| Agent implements Phase 3+ features too early | Medium | Each agent task explicitly lists what must NOT be implemented. Orchestrator reviews. |

---

## Next approval needed

The orchestrator (this agent) requests approval to:

1. **Create Wave 1 branches** `phase/01-ui-shell` and `phase/02-backend-contracts` from the latest `origin/main`
2. **Optionally create** `phase/qa-setup` for the QA agent (Wave 1)
3. **Push Wave 1 branches** to `origin`
4. **Launch Wave 1 agents** with their first tasks as defined in `docs/PARALLEL_EXECUTION_PLAN.md`
5. **Create Wave 2 branch** `phase/02-storage-settings` from the latest `origin/main` after `phase/02-backend-contracts` is merged or reviewed
6. **Launch Database/privacy agent** on `phase/02-storage-settings`

No implementation will begin until the user approves this launch plan.
