# Phase 7 Handoff: Global Shortcut and Dictation Overlay Trigger

## Subagents used

| Agent | Role | Output |
|---|---|---|
| A — Backend | Researched Tauri v2 global shortcut plugin API, confirmed Cargo dep and capability permission | Rust implementation proposal |
| B — Frontend | Researched Router-safe listener approach, proposed ShortcutHandler component and uiStore changes | TypeScript implementation proposal |
| C — QA | Reviewed Phase 7 scope, confirmed forbidden features excluded, produced verification checklist | Manual QA checklist and automated verification plan |

**Only the main orchestrator committed.** Subagents were read-only.

---

## Files changed

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Added `tauri-plugin-global-shortcut = "2"` |
| `src-tauri/Cargo.lock` | Auto-updated by cargo (6 new crates: global-hotkey, tauri-plugin-global-shortcut, gethostname, x11rb, x11rb-protocol, xkeysym) |
| `src-tauri/src/lib.rs` | Added `tauri::Emitter` + `GlobalShortcutExt` imports; `.plugin(Builder::new().build())`; shortcut registration block in setup |
| `src-tauri/capabilities/default.json` | Added `"global-shortcut:allow-register"` permission |
| `src/stores/uiStore.ts` | Added `openOverlay()` and `closeOverlay()` actions; kept `toggleOverlay()` |
| `src/components/ShortcutHandler.tsx` | **NEW** — null-rendering component; listens for `"dictation-shortcut-pressed"`, calls `openOverlay()`, navigates to `/dictation` |
| `src/App.tsx` | Imported `ShortcutHandler`; mounted `<ShortcutHandler />` as first child inside `<BrowserRouter>` |
| `src/components/FloatingOverlay.tsx` | Imported `Link` from react-router-dom; changed dismiss button to use `closeOverlay()`; replaced stale "Dictation not yet active — Phase 6" with "Go to Dictation →" link |
| `src/pages/Settings.tsx` | Replaced "Shortcut configuration — Phase 7" placeholder text with "Shortcut rebinding coming in a future release." |

---

## Shortcut behavior

**Default shortcut:** `CommandOrControl+Shift+Space`  
- Windows: `Ctrl+Shift+Space`  
- macOS: `Cmd+Shift+Space`

**What happens on press:**
1. Rust handler receives `ShortcutState::Pressed`
2. `app_handle.emit("dictation-shortcut-pressed", ())` fires
3. `ShortcutHandler` (TypeScript, inside `<BrowserRouter>`) receives event
4. `openOverlay()` sets `overlayOpen = true`
5. `navigate("/dictation")` switches page
6. `FloatingOverlay` renders (overlayOpen is true)
7. Dictation page is shown

**What does NOT happen on press (by design):**
- Recording does not auto-start
- No text insertion or clipboard paste
- No active-app detection

---

## Canonical event name

```
dictation-shortcut-pressed
```

Used in:
- Rust: `app_handle.emit("dictation-shortcut-pressed", ())`
- TypeScript: `listen<null>("dictation-shortcut-pressed", handler)`

---

## Backend architecture

Plugin initialized in `lib.rs` builder chain:
```rust
.plugin(tauri_plugin_global_shortcut::Builder::new().build())
```

Shortcut registered in the `.setup()` closure:
```rust
let shortcut_handle = app.handle().clone();
match "CommandOrControl+Shift+Space".parse::<tauri_plugin_global_shortcut::Shortcut>() {
    Ok(shortcut) => {
        if let Err(e) = app.handle().global_shortcut().on_shortcut(
            shortcut,
            move |_app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    shortcut_handle.emit("dictation-shortcut-pressed", ()).ok();
                }
            },
        ) {
            eprintln!("[phase7] global shortcut registration failed: {e}");
        }
    }
    Err(e) => {
        eprintln!("[phase7] failed to parse shortcut string: {e}");
    }
}
```

**Note on plugin API:** `tauri-plugin-global-shortcut` v2.3.1 does not export a standalone `init()` function. The correct initialization is `Builder::new().build()`, confirmed by successful compilation.

---

## Frontend architecture

`ShortcutHandler` is a null-rendering React component mounted directly inside `<BrowserRouter>` so that `useNavigate()` is in scope:

```tsx
// src/App.tsx — inside BrowserRouter, before the layout div
<BrowserRouter>
  <ShortcutHandler />
  <div id="app-shell" ...>
```

The `useEffect` properly cleans up the Tauri event listener on unmount via the `unlisten` function returned by `listen()`.

`uiStore.ts` now exports three overlay controls:
- `toggleOverlay()` — kept for Sidebar toggle button
- `openOverlay()` — called by `ShortcutHandler` (opens idempotently)
- `closeOverlay()` — called by `FloatingOverlay` dismiss button and "Go to Dictation" link

---

## Registration failure behavior

If shortcut registration fails at runtime (e.g., OS denies the hotkey, another app holds it):
- The error is logged to stderr: `[phase7] global shortcut registration failed: <reason>`
- The app continues to start normally
- The shortcut simply has no effect
- No crash, no panic, no user-visible error dialog

This is the approved graceful-degradation behavior.

---

## Manual QA steps (Windows)

1. Run `npm run tauri dev`
2. App opens normally — no crash, no error dialog
3. Press `Ctrl+Shift+Space` while the app is in the background
4. **Expected:** App window comes to foreground, `FloatingOverlay` appears, page navigates to `/dictation`
5. Click the `✕` button on the overlay — **Expected:** overlay closes (uses `closeOverlay`, not `toggleOverlay`)
6. Click "Go to Dictation →" link on the overlay — **Expected:** overlay closes, `/dictation` page is shown
7. Press `Ctrl+Shift+Space` again — **Expected:** overlay re-opens, navigates to `/dictation`
8. Open Settings page — **Expected:** shortcut shows as `CommandOrControl+Shift+Space` with text "Shortcut rebinding coming in a future release." (no Phase 7 placeholder)
9. Verify microphone, record, transcribe, cleanup all still work as before

### macOS verification
Deferred — Windows-only QA performed for this phase. macOS testing should be done before main merge.

---

## Checks run and results

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS (0 warnings) |
| `cargo test` | PASS — 100/100 tests |
| `npm run lint` (tsc --noEmit) | PASS |
| `npm run build` | PASS (285 kB JS bundle) |
| `pwsh scripts/quality-check.ps1` | PASS — all checks passed |

All 100 existing Rust unit tests continue to pass. No new Rust unit tests were added for Phase 7 (the shortcut plugin is a Tauri runtime concern and cannot be unit-tested outside a running `AppHandle`).

---

## Known limitations

- No shortcut rebinding UI (Phase 8+)
- No press-and-hold dictation mode (future enhancement)
- No active app detection (Phase 11)
- Recording does not auto-start from shortcut (by design for Phase 7)
- Insert button remains disabled (`was_inserted` always false — Phase 9)
- DiagnosticsService still returns static report (Phase 8)
- `history_count` in `get_status_summary` still O(n) (Phase 8)
- Date-based history retention not enforced (Phase 8)
- macOS shortcut behavior not verified in this session

---

## Forbidden features — confirmed NOT added

- No text insertion into active app
- No clipboard paste simulation
- No auto-start recording from shortcut
- No press-and-hold mode
- No packaging changes
- No audio recording rewrite (Phase 3 untouched)
- No transcription rewrite (Phase 4 untouched)
- No cleanup providers rewrite (Phase 5 untouched)
- No history rewrite (Phase 6 untouched)
- No diagnostics overhaul (Phase 8)
- No cloud features added
- No audio files stored in history
- No account/login/sync
- No provider API key behavior changes
- No new database migrations

## Forbidden files — confirmed NOT touched

`src-tauri/src/services/audio.rs`, `src-tauri/src/commands/audio.rs`,
`src-tauri/src/services/transcription.rs`, `src-tauri/src/commands/transcription.rs`,
`src-tauri/src/services/cleanup.rs`, `src-tauri/src/commands/cleanup.rs`,
`src-tauri/src/services/providers.rs`, `src-tauri/src/commands/providers.rs`,
`src-tauri/src/services/history.rs`, `src-tauri/src/commands/history.rs`,
`src-tauri/src/db/**`, `src/pages/Providers.tsx`, `src/pages/Models.tsx`,
`src/pages/History.tsx`, `src/pages/Privacy.tsx`, `src/pages/Diagnostics.tsx`,
`docs/PHASES.md`, `AGENTS.md`, `CLAUDE.md`, `SOUL.md`

---

## Next recommended task

1. Orchestrator: review Phase 7 PR and send `ORCHESTRATOR APPROVED MERGE`
2. Launch Phase 8: Privacy enforcement, diagnostics overhaul, retention policy
3. Launch Phase 9 (after Phase 8): Text insertion into active app (`was_inserted = true`)
