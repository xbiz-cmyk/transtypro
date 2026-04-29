# transtypro Agent Pack

This pack prepares Claude Code, Codex, and Antigravity agents to build **transtypro**.

transtypro is a local-first Windows and macOS AI dictation desktop app.

Main promise:

> Speak instead of type.

Use this pack before giving the full build prompt.

## How to use

1. Create an empty project folder.

```bash
mkdir transtypro
cd transtypro
git init
```

2. Copy all files from this pack into the project root.

3. Start Antigravity in this folder.

4. Open Claude Code inside the project folder.

5. Ask Claude Code:

```text
Read README-START-HERE.md, CLAUDE.md, AGENTS.md, SOUL.md, docs/ORCHESTRATION_PLAN.md, and docs/PHASES.md.

Do not build yet.

First validate the repository setup, create any missing baseline folders, and produce an implementation plan for Phase 0 only.
```

6. After Phase 0 is ready, give Claude the main build prompt from `prompts/MASTER_BUILD_PROMPT.md`.

## Important rule

Do not ask one agent to build everything at once.

Use phases. Use branches. Use review gates.

The safest pattern:

- One orchestrator agent plans and merges.
- Multiple implementation agents work on separate branches or worktrees.
- A reviewer agent checks each phase before merge.
- Codex is used as independent reviewer.
