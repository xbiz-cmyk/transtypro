# Wave 7 Launch Record — Global Shortcut and Dictation Overlay Trigger

## Launch metadata

| Item | Value |
|---|---|
| Wave | 7 |
| Phase | Phase 7 — Global shortcut and dictation overlay trigger |
| Base commit | `8288b93` — feat(history): add dictation save pipeline |
| Branch | `phase/07-global-shortcut-overlay` |
| Worktree | `C:\Users\User\Desktop\transtypro-shortcut` |
| Launched | 2026-05-05 |
| Merge rule | No merge without the exact phrase: **ORCHESTRATOR APPROVED MERGE** |

---

## Phase 7 scope

**One sentence:** Register a system-wide keyboard shortcut (`CommandOrControl+Shift+Space`) that opens the floating overlay and navigates to the Dictation page without any click.

### Approved features

- Register `CommandOrControl+Shift+Space` as a global shortcut via `tauri-plugin-global-shortcut`.
- When shortcut fires, emit Tauri event `"dictation-shortcut-pressed"` to the frontend.
- Frontend `ShortcutHandler` (null-rendering component inside `<BrowserRouter>`) listens for the event, calls `openOverlay()`, and navigates to `/dictation`.
- `uiStore` gains `openOverlay()` and `closeOverlay()` actions (currently only `toggleOverlay()` exists).
- `FloatingOverlay.tsx` gains a "Go to Dictation →" link and removes the stale `"Dictation not yet active — Phase 6"` text.
- `Settings.tsx` removes the `"Shortcut configuration — Phase 7"` placeholder text (shortcut is now real).
- `capabilities/default.json` gains `"global-shortcut:allow-register"` permission.

### What Phase 7 is not

- No recording triggered automatically from the shortcut (Phase 8+ behavior).
- No hold-to-dictate / release-to-stop shortcut (future enhancement).
- No shortcut configuration UI (key rebinding deferred).
- No active app detection or window tracking (Phase 11).
- No text insertion (Phase 9).
- No clipboard paste simulation (Phase 9).
- No diagnostics overhaul (Phase 8).
- No packaging (Phase 10).
- No changes to audio recording logic (Phase 3 is complete).
- No changes to transcription logic (Phase 4 is complete).
- No changes to cleanup providers (Phase 5 is complete).
- No changes to history persistence (Phase 6 is complete).

---

## Architecture summary

### Event flow

```
User presses CommandOrControl+Shift+Space
        ↓
tauri-plugin-global-shortcut handler (Rust, lib.rs setup block)
        ↓
app_handle.emit("dictation-shortcut-pressed", ())
        ↓
ShortcutHandler component (TypeScript, inside BrowserRouter)
  → listen("dictation-shortcut-pressed", handler)
  → openOverlay()         ← uiStore action
  → navigate("/dictation") ← react-router-dom
        ↓
FloatingOverlay appears (overlayOpen = true)
Dictation page is shown
```

### Canonical event name

```
"dictation-shortcut-pressed"
```

This exact string must be used in both:
- Rust: `app_handle.emit("dictation-shortcut-pressed", ())`
- TypeScript: `listen("dictation-shortcut-pressed", handler)`

Any mismatch silently breaks the feature. Do not change the name without updating both sides.

### Default shortcut

```
CommandOrControl+Shift+Space
```

On Windows this resolves to `Ctrl+Shift+Space`. On macOS this resolves to `Cmd+Shift+Space`.

---

## Subagent strategy

Phase 7 uses three subagents. **Only the main orchestrator agent commits.** Subagents propose code and report — they do not write files.

| Agent | Role | Scope |
|---|---|---|
| A — Backend | Rust implementation | `src-tauri/**` only |
| B — Frontend | TypeScript/React implementation | `src/**` only |
| C — QA | Review + verification | Read-only |

### Agent A (Backend) — what to propose

1. Add `tauri-plugin-global-shortcut = "2"` to `src-tauri/Cargo.toml` under `[dependencies]`.
2. In `src-tauri/src/lib.rs`:
   - Add `.plugin(tauri_plugin_global_shortcut::init())` to the builder chain (after the opener plugin).
   - In the `.setup()` closure, obtain `app.handle().clone()`, then register the shortcut:
     ```rust
     use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
     let handle = app.handle().clone();
     app.handle().global_shortcut().on_shortcut(
         Shortcut::new(None, tauri_plugin_global_shortcut::Code::Space)
             .with_mods(&[
                 tauri_plugin_global_shortcut::Modifiers::CONTROL,
                 tauri_plugin_global_shortcut::Modifiers::SHIFT,
             ]),
         move |_app, _shortcut, event| {
             if event.state() == ShortcutState::Pressed {
                 handle.emit("dictation-shortcut-pressed", ()).ok();
             }
         },
     )?;
     ```
3. Add `"global-shortcut:allow-register"` to `src-tauri/capabilities/default.json` permissions array.
4. Verify: `cargo add tauri-plugin-global-shortcut@2` then `cargo build` succeeds.

**Note on API shape**: The `tauri-plugin-global-shortcut` v2 API changes between minor versions. If the `Shortcut::new(...).with_mods(...)` builder does not compile, try:
```rust
let shortcut: Shortcut = "CommandOrControl+Shift+Space".parse().unwrap();
app.handle().global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
    if event.state() == ShortcutState::Pressed {
        handle.emit("dictation-shortcut-pressed", ()).ok();
    }
})?;
```
Use whichever compiles. The string-parse form is more portable. Report which form compiled.

### Agent B (Frontend) — what to propose

1. Add `@tauri-apps/plugin-global-shortcut` npm package (version `^2`).
2. Create `src/components/ShortcutHandler.tsx`:
   ```tsx
   import { useEffect } from "react";
   import { listen } from "@tauri-apps/api/event";
   import { useNavigate } from "react-router-dom";
   import { useUiStore } from "../stores/uiStore";

   export default function ShortcutHandler() {
     const navigate = useNavigate();
     const openOverlay = useUiStore((s) => s.openOverlay);

     useEffect(() => {
       let unlisten: (() => void) | undefined;
       listen<null>("dictation-shortcut-pressed", () => {
         openOverlay();
         navigate("/dictation");
       }).then((fn) => { unlisten = fn; });
       return () => { unlisten?.(); };
     }, [navigate, openOverlay]);

     return null;
   }
   ```
3. In `src/stores/uiStore.ts`:
   - Add `openOverlay: () => void` and `closeOverlay: () => void` to the `UiState` interface.
   - Implement them: `openOverlay: () => set({ overlayOpen: true })`, `closeOverlay: () => set({ overlayOpen: false })`.
   - Keep `toggleOverlay` (used by Sidebar).
4. In `src/App.tsx`:
   - Import `ShortcutHandler`.
   - Place `<ShortcutHandler />` as first child inside `<BrowserRouter>`, before Sidebar.
5. In `src/components/FloatingOverlay.tsx`:
   - Import `Link` from `react-router-dom` and `useUiStore`.
   - Replace stale `"Dictation not yet active — Phase 6"` text with a `<Link to="/dictation">` button that also calls `closeOverlay()` before navigating.
   - Use `closeOverlay()` (not `toggleOverlay()`) on the dismiss/close button.
6. In `src/pages/Settings.tsx`:
   - Find the `"Shortcut configuration — Phase 7"` helper text and remove it (the shortcut is now real, no helper text needed).
   - Update the displayed shortcut string to show `Ctrl+Shift+Space` (Windows) / `⌘⇧Space` (macOS). Use a simple conditional or just `CommandOrControl+Shift+Space` text.

**Note**: `@tauri-apps/plugin-global-shortcut` is only needed on the Rust side; the frontend listens via the generic Tauri event system (`@tauri-apps/api/event`). You do NOT need to import anything from `@tauri-apps/plugin-global-shortcut` in TypeScript — just use `listen` from `@tauri-apps/api/event`. The npm package may not even be necessary.

### Agent C (QA) — what to verify

Review both proposed implementations for:
1. Event name consistency: Rust `emit` string === TypeScript `listen` string.
2. `ShortcutHandler` placed inside `<BrowserRouter>` (required for `useNavigate()`).
3. `unlisten` cleanup in `useEffect` return.
4. `openOverlay` defined in uiStore before `ShortcutHandler` uses it.
5. No `toggleOverlay` calls in new code (use `openOverlay`/`closeOverlay` specifically).
6. `capabilities/default.json` has `"global-shortcut:allow-register"`.
7. `Cargo.toml` has `tauri-plugin-global-shortcut = "2"`.
8. No Phase 3/4/5/6 service or command files touched.
9. All existing 100 cargo tests still pass.
10. `cargo fmt --check` and `cargo clippy -D warnings` clean.
11. `npm run lint` clean (tsc --noEmit).
12. `npm run build` produces a bundle.

---

## Files the Phase 7 agent may edit

```
src-tauri/Cargo.toml                          ← add tauri-plugin-global-shortcut = "2"
src-tauri/src/lib.rs                          ← add plugin init + shortcut registration
src-tauri/capabilities/default.json           ← add global-shortcut:allow-register
src/components/ShortcutHandler.tsx            ← NEW: null component, listen + navigate
src/stores/uiStore.ts                         ← add openOverlay + closeOverlay
src/App.tsx                                   ← add <ShortcutHandler /> inside BrowserRouter
src/components/FloatingOverlay.tsx            ← remove stale text, add dictation link
src/pages/Settings.tsx                        ← remove "Phase 7" placeholder text
handoff/phase-07-global-shortcut.md           ← required handoff file
docs/PROGRESS.md                              ← only after implementation succeeds
docs/TASK_BOARD.md                            ← only after implementation succeeds
```

---

## Files the Phase 7 agent must NOT edit

```
src-tauri/src/services/audio.rs               ← Phase 3 — do not touch
src-tauri/src/commands/audio.rs               ← Phase 3 — do not touch
src-tauri/src/services/transcription.rs       ← Phase 4 — do not touch
src-tauri/src/commands/transcription.rs       ← Phase 4 — do not touch
src-tauri/src/services/cleanup.rs             ← Phase 5 — do not touch
src-tauri/src/commands/cleanup.rs             ← Phase 5 — do not touch
src-tauri/src/services/providers.rs           ← Phase 5 — do not touch
src-tauri/src/commands/providers.rs           ← Phase 5 — do not touch
src-tauri/src/services/privacy.rs             ← do not touch
src-tauri/src/services/history.rs             ← do not touch
src-tauri/src/commands/history.rs             ← do not touch
src-tauri/src/commands/mod.rs                 ← do not touch
src-tauri/src/db/**                           ← no schema changes
src-tauri/Cargo.lock                          ← do not modify directly (cargo resolves)
src/pages/Providers.tsx                       ← do not touch
src/pages/Models.tsx                          ← do not touch
src/pages/History.tsx                         ← do not touch
src/pages/Dictation.tsx                       ← do not touch
src/pages/Privacy.tsx                         ← do not touch
src/pages/Diagnostics.tsx                     ← do not touch
docs/PHASES.md                                ← orchestrator only
docs/PARALLEL_EXECUTION_PLAN.md               ← orchestrator only
docs/ARCHITECTURE.md                          ← orchestrator only
AGENTS.md                                     ← orchestrator only
CLAUDE.md                                     ← orchestrator only
SOUL.md                                       ← orchestrator only
```

---

## Implementation reminders for the Phase 7 agent

- Use `"dictation-shortcut-pressed"` as the event name everywhere. No variation.
- Default shortcut: `CommandOrControl+Shift+Space`. Do not change it without orchestrator approval.
- `ShortcutHandler` MUST be a child of `<BrowserRouter>` so `useNavigate()` works.
- `ShortcutHandler` returns `null` — it is a logic-only component.
- `openOverlay()` and `closeOverlay()` must be added to uiStore before `ShortcutHandler` uses them.
- `FloatingOverlay` should use `closeOverlay()` on dismiss, not `toggleOverlay()`.
- Do NOT add `@tauri-apps/plugin-global-shortcut` to npm imports — the frontend uses `@tauri-apps/api/event` only.
- Do NOT store the shortcut key in SQLite — settings storage for shortcuts is Phase 8+.
- Do NOT auto-start recording when the shortcut fires — navigation to Dictation page only.
- Do NOT add a new migration.
- Do NOT change Phase 3/4/5/6 service or command files.
- Minimum new tests: at least one Rust test for shortcut registration (if the plugin is testable) or document why it cannot be unit-tested.
- All 100 existing Rust tests must continue to pass.

---

## Verification commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
npm run lint
npm run build
pwsh quality-check.ps1
```

All 100 existing tests must pass after Phase 7. Add tests for any new testable Rust logic.

---

## Known limitations going into Phase 7

- No recording auto-trigger from shortcut.
- No hold-to-dictate behavior.
- No shortcut rebinding UI.
- Insert button still disabled (Phase 9).
- `was_inserted` still always `false` (Phase 9).
- DiagnosticsService still returns static report (Phase 8).
- `history_count` in `get_status_summary` still O(n) (Phase 8).
- Date-based history retention not enforced (Phase 8).
- No active app detection (Phase 11).

---

## Merge rule

No PR for `phase/07-global-shortcut-overlay` may be merged without the orchestrator sending the exact phrase:

```
ORCHESTRATOR APPROVED MERGE
```

---

## Next step after Wave 7

1. Open a new agent session in `C:\Users\User\Desktop\transtypro-shortcut`.
2. The agent reads this file and the relevant prior handoffs.
3. The agent implements Phase 7 in order:
   a. Add `tauri-plugin-global-shortcut = "2"` to Cargo.toml
   b. Register plugin + shortcut in `lib.rs`
   c. Add `global-shortcut:allow-register` to capabilities
   d. Add `openOverlay` + `closeOverlay` to uiStore
   e. Create `ShortcutHandler.tsx`
   f. Wire `ShortcutHandler` into `App.tsx`
   g. Update `FloatingOverlay.tsx`
   h. Update `Settings.tsx`
   i. Run all checks
   j. Create `handoff/phase-07-global-shortcut.md`
   k. Create PR against `main`
4. Orchestrator reviews and merges with `ORCHESTRATOR APPROVED MERGE`.

---

## Agent prompt for the transtypro-shortcut session

```
You are the transtypro Phase 7 agent.

Your working directory is: C:\Users\User\Desktop\transtypro-shortcut
Your branch is: phase/07-global-shortcut-overlay

Read these files before writing any code:
- handoff/wave-7-launch.md              ← this file; your full specification
- handoff/phase-06-dictation-pipeline.md
- src-tauri/Cargo.toml
- src-tauri/src/lib.rs
- src-tauri/capabilities/default.json
- src/stores/uiStore.ts
- src/App.tsx
- src/components/FloatingOverlay.tsx
- src/pages/Settings.tsx

Your task:
Implement Phase 7 — Global shortcut and dictation overlay trigger.

Phase 7 scope (exactly):
1. Add tauri-plugin-global-shortcut = "2" to src-tauri/Cargo.toml
2. Register the plugin and CommandOrControl+Shift+Space shortcut in src-tauri/src/lib.rs
   - On shortcut press: app_handle.emit("dictation-shortcut-pressed", ()).ok()
3. Add "global-shortcut:allow-register" to src-tauri/capabilities/default.json
4. Add openOverlay() and closeOverlay() to src/stores/uiStore.ts
5. Create src/components/ShortcutHandler.tsx (null component, listen + openOverlay + navigate)
6. Wire <ShortcutHandler /> as first child inside <BrowserRouter> in src/App.tsx
7. Update src/components/FloatingOverlay.tsx:
   - Remove "Dictation not yet active — Phase 6" text
   - Add "Go to Dictation →" link that closes overlay and navigates
   - Use closeOverlay() on dismiss button
8. Update src/pages/Settings.tsx:
   - Remove "Shortcut configuration — Phase 7" placeholder text
9. Run all checks
10. Create handoff/phase-07-global-shortcut.md
11. Create PR against main

Run all checks before creating PR:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test (all 100 must pass)
- npm run lint
- npm run build
- pwsh quality-check.ps1

Create handoff/phase-07-global-shortcut.md with:
- what changed
- test results
- known limitations
- next recommended task

Then create a PR against main.

Constraints (read wave-7-launch.md for the full list):
- Event name must be exactly "dictation-shortcut-pressed" in both Rust and TypeScript.
- ShortcutHandler MUST be inside <BrowserRouter> in App.tsx.
- Do NOT import @tauri-apps/plugin-global-shortcut in TypeScript — use @tauri-apps/api/event only.
- Do NOT auto-start recording when shortcut fires.
- Do NOT touch Phase 3/4/5/6 service or command files.
- Do NOT add a schema migration.
- All 100 existing Rust tests must pass.
```
