# Phase 0 Handoff — Main Agent

## Branch

`phase/00-bootstrap`

## Summary

Phase 0 creates the project skeleton: Tauri v2 desktop app with React + TypeScript frontend,
Rust backend, Tailwind CSS v4 dark theme, and verified frontend-backend IPC.

## Touched files

### New files

- `package.json` — npm project config
- `tsconfig.json` — TypeScript config
- `tsconfig.node.json` — TypeScript config for Vite
- `vite.config.ts` — Vite + React + Tailwind v4 config
- `index.html` — app entry point
- `src/main.tsx` — React entry
- `src/App.tsx` — app shell with router + sidebar + status bar
- `src/index.css` — Tailwind v4 with dark-theme design tokens
- `src/vite-env.d.ts` — Vite type reference
- `src/lib/types.ts` — shared TypeScript types
- `src/lib/api.ts` — Tauri command wrappers
- `src/components/Sidebar.tsx` — sidebar navigation
- `src/components/StatusBar.tsx` — top status bar
- `src/pages/Home.tsx` — home page with status summary
- `src-tauri/Cargo.toml` — Rust project config
- `src-tauri/build.rs` — Tauri build script
- `src-tauri/tauri.conf.json` — Tauri app config
- `src-tauri/src/main.rs` — Rust main entry
- `src-tauri/src/lib.rs` — module declarations + Tauri builder
- `src-tauri/src/errors/mod.rs` — typed AppError enum
- `src-tauri/src/commands/mod.rs` — Tauri command wrappers
- `src-tauri/src/services/mod.rs` — service interface stubs
- `src-tauri/src/db/mod.rs` — database placeholder
- `src-tauri/src/models/mod.rs` — shared data structs
- `src-tauri/src/utils/mod.rs` — utilities placeholder
- `src-tauri/capabilities/` — Tauri capabilities (from scaffold)
- `src-tauri/icons/` — app icons (from scaffold)
- `public/` — static assets (from scaffold)

### Modified files

- `AGENTS.md` — updated file ownership paths from `frontend/src/**` to `src/**` (ADR-002)
- `docs/ORCHESTRATION_PLAN.md` — added `src/**` ownership note to Frontend UI agent
- `docs/PROGRESS.md` — Phase 0 completed
- `docs/TASK_BOARD.md` — Phase 0 moved to Done
- `docs/DECISIONS.md` — ADR-001 (SQLite deferred), ADR-002 (frontend layout)

### Preserved files (not touched)

- `CLAUDE.md`
- `SOUL.md`
- `README-START-HERE.md`
- `docs/PHASES.md`
- `docs/SAFE_COMMANDS.md`
- `docs/QA_CHECKLIST.md`
- `docs/ARCHITECTURE.md`
- `prompts/**`
- `handoff/README.md`
- `.claude/**`
- `.gitignore`
- `.gitattributes`

## Commands run

| Command | Result |
|---|---|
| `winget install Rustlang.Rustup` | ✅ Installed rustc 1.95.0 |
| `rustup default stable` | ✅ stable-x86_64-pc-windows-msvc |
| `npx -y create-tauri-app@latest _scaffold_temp -m npm -t react-ts --tauri-version 2 -y` | ✅ Scaffold in temp dir |
| `npm install` | ✅ 90 packages |
| `npm run build` | ✅ tsc + vite build |
| `npm run lint` (tsc --noEmit) | ✅ No errors |
| `cargo fmt --check` | ✅ No formatting issues |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ No warnings |
| `cargo test` | ✅ 0 passed, 0 failed (compiles + runs) |

## Commands that failed

None.

## Known limitations

- Only the Home page is implemented; other pages are "Coming soon" in sidebar.
- Status summary returns static defaults (no persistence yet).
- No unit tests yet — `cargo test` runs but finds 0 test cases.
- `npm run test` prints a placeholder message (no test runner configured).
- Tailwind `@theme` custom properties use `oklch()` which may not render on very old WebView2 versions.

## Open questions

- None blocking for Phase 0.

## Next safe task

Phase 1: UI shell — create all pages, full sidebar navigation, floating overlay component,
reusable UI components, stores with mock data clearly labeled.
