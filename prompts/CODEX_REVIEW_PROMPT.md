# Codex independent review prompt

Read:
- AGENTS.md
- SOUL.md
- docs/ARCHITECTURE.md
- docs/PHASES.md
- docs/QA_CHECKLIST.md

Then review the current branch/diff.

Focus on:
- privacy violations
- local-only mode bypasses
- secrets handling
- fake implementations
- incomplete UI wiring
- unhandled errors
- missing tests
- cross-platform risks
- large or risky changes
- unclear docs

Return:

1. Blocking issues
2. Non-blocking issues
3. Security/privacy concerns
4. Tests that should be added
5. Recommended next action
