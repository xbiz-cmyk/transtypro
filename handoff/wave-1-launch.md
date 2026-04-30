# Wave 1 Launch — Handoff

## Base commit

All Wave 1 branches created from `main` at commit `bec4e8d` — "docs: add multi-agent execution plan".

---

## Worktrees and branches

| Agent | Branch | Worktree path | Status |
|---|---|---|---|
| Frontend UI agent | `phase/01-ui-shell` | `C:/Users/User/Desktop/transtypro-ui` | Ready |
| Backend contracts agent | `phase/02-backend-contracts` | `C:/Users/User/Desktop/transtypro-backend` | Ready |
| QA setup agent | `phase/qa-setup` | `C:/Users/User/Desktop/transtypro-qa` | Ready |

All branches pushed to `origin`.

---

## Agent ownership boundaries

### Frontend UI agent — `transtypro-ui` / `phase/01-ui-shell`

May create or edit:
```
src/pages/**
src/components/**
src/stores/**
src/App.tsx
src/index.css
src/lib/types.ts   (UI-specific types only)
```

Must NOT touch:
```
src-tauri/**
src/lib/api.ts     (only add stubs for commands that already exist in backend)
docs/**            (except UI-specific docs if needed)
```

### Backend contracts agent — `transtypro-backend` / `phase/02-backend-contracts`

May create or edit:
```
src-tauri/src/commands/**
src-tauri/src/services/**
src-tauri/src/models/**
src-tauri/src/errors/mod.rs
src-tauri/src/lib.rs
src-tauri/Cargo.toml
```

Must NOT touch:
```
src/**
src-tauri/src/db/**    (reserved for Database/privacy agent in Wave 2)
```

### QA setup agent — `transtypro-qa` / `phase/qa-setup`

May create or edit:
```
tests/**
scripts/**
docs/QA_CHECKLIST.md
```

Must NOT touch:
```
src/**
src-tauri/src/**
```

---

## Wave 2 — Storage agent (does NOT start yet)

The Database/privacy agent on `phase/02-storage-settings` must wait until
`phase/02-backend-contracts` is merged or reviewed before starting.

Branch `phase/02-storage-settings` has not been created yet.

---

## Shared file protocol

If any agent needs to modify a file outside its ownership:
1. Document the change needed in its handoff file.
2. Ask the orchestrator to schedule the change.
3. Never edit the file directly.

Lock file rules:
- `package-lock.json` — Frontend agent only.
- `Cargo.lock` — Backend/Database agent only.

---

## Checks each agent must run before handoff

### Frontend UI agent
```bash
npm run build
npm run lint
git status --short
git diff --stat
```

### Backend contracts agent
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run build
git status --short
git diff --stat
```

### QA setup agent
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

## Merge order (for reference — do not merge until orchestrator approves)

```
1. phase/01-ui-shell          → main
2. phase/02-backend-contracts → main
3. phase/02-storage-settings  → main   (Wave 2 — not started)
4. phase/qa-setup             → main   (if applicable)
```

No PR may be merged unless the user provides the exact line:

ORCHESTRATOR APPROVED MERGE
