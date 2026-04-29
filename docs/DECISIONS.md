# Architecture Decisions

Record important decisions here.

## ADR format

### ADR-000: Title

Status: proposed | accepted | rejected

Context:
What problem are we solving?

Decision:
What did we choose?

Why:
Why is this better?

Consequences:
What tradeoffs does this create?

---

### ADR-001: SQLite Crate Selection

Status: deferred to Phase 2

Context:
transtypro needs local SQLite storage for settings, modes, vocabulary, and history.
Two main options: `rusqlite` (sync, simple) vs `sqlx` (async, migrations built-in).

Decision:
Deferred. No SQLite dependency added in Phase 0.
The decision will be made at the start of Phase 2 when persistence requirements are clearer.

Why:
Phase 0 only needs the skeleton. Adding SQLite now would create unused code and
complicate the first build without providing value.

Consequences:
- Phase 2 must start with this decision.
- Service interfaces do not depend on a specific SQLite crate.

---

### ADR-002: Use standard Tauri frontend layout at src/**

Status: accepted

Context:
`create-tauri-app` generated a standard Tauri v2 layout with React frontend in `src/**`.

Decision:
Keep the standard Tauri layout instead of moving frontend code into `frontend/src/**`.

Why:
This reduces custom scaffolding, keeps Tauri/Vite defaults simple, and avoids
unnecessary build path changes.

Consequences:
- All agent ownership docs must treat `src/**` as the frontend area.
- `AGENTS.md` and `docs/ORCHESTRATION_PLAN.md` have been updated to reflect this.
- Do not create a `frontend/` folder later unless the orchestrator explicitly approves it.
