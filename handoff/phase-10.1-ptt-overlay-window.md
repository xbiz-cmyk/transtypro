# Phase 10.1 Handoff: PTT Feedback Overlay Window

## Phase goal

Add a small, always-on-top, non-focusable overlay window that appears while push-to-talk is active. The overlay shows which pipeline phase is running and provides a Cancel button — without stealing focus from the active application.

## Architecture

### Secondary Tauri window

A second `WebviewWindow` named `ptt-overlay` is created in `lib.rs` during `setup()`. It loads the same `index.html` as the main window. `App.tsx` detects the window label and renders only `<PttOverlay />` instead of the full app shell.

### Window properties

| Property | Value |
|---|---|
| Label | `ptt-overlay` |
| Size | 320 × 96 logical pixels |
| Visible at startup | No (hidden) |
| Always on top | Yes |
| Skip taskbar | Yes |
| Decorations | No |
| Focused at creation | No |
| Resizable | No |
| Closable/minimizable/maximizable | No |
| Position | Bottom-center of primary monitor, 48 px above taskbar |

### Position calculation

```rust
let scale = monitor.scale_factor();
let screen_w = monitor.size().width as f64 / scale;
let screen_h = monitor.size().height as f64 / scale;
let x = (screen_w - 320.0) / 2.0;
let y = screen_h - 96.0 - 48.0;
overlay.set_position(tauri::LogicalPosition::new(x, y));
```

Position calculation uses `primary_monitor()`. If it returns `None`, the OS default position is used — the overlay still appears on screen.

### Window creation failure is non-fatal

If `WebviewWindowBuilder::build()` fails, an `eprintln!` is emitted and the app continues. PTT pipeline still works — the user just won't see the overlay feedback. This is consistent with CLAUDE.md's "safe fallback" rule.

## Focus safety design

- `.focused(false)` is set at `WebviewWindowBuilder` build time (maps to `WS_EX_NOACTIVATE` on Windows, preventing focus steal on `ShowWindow`).
- `overlay.show()` is the only call after startup — `set_focus()` is never called.
- `PttOverlay.tsx` never calls `getCurrentWindow().focus()` or any focus-granting API.
- `InsertionService` runs clipboard paste ~10+ seconds after the shortcut press. Focus is never disturbed between the shortcut press and the paste.
- Manual QA verified: text inserted into Notepad, not into transtypro (see QA results below).

## First-event delivery strategy

**Chosen approach: pre-created window + show-before-spawn.**

1. `ptt-overlay` window is created hidden during `setup()`. The webview loads `index.html`, `App.tsx` renders `<PttOverlay />`, and `PttOverlay`'s `listen("ptt-status")` handler is registered. All of this happens at app startup while the user is working in their active app.
2. When the user presses the PTT shortcut, `ptt_start()` calls `overlay.show()` **before** spawning the recording thread.
3. The spawned thread calls `AudioService::start_recording()` (blocking, ~100–300 ms), and only then emits `ptt-status: { phase: "recording", message: "Recording…" }`.
4. Because the webview was hydrated at startup and `listen` was registered before the thread spawned, the first event is received reliably.

**Fallback:** `PttOverlay` initialises with `phase = "recording"` / `message = "Listening…"`. Even if the first event is missed, the overlay displays the correct state immediately.

## Capability file change

`src-tauri/capabilities/default.json` — `"ptt-overlay"` added to the `windows` array:

```json
"windows": ["main", "ptt-overlay"]
```

This gives `ptt-overlay` the same `core:default` capabilities as the main window, which covers:
- Tauri `invoke()` (needed for `cancelPtt`)
- Event listening (`listen("ptt-status")`)
- `WebviewWindow.hide()` (part of `core:default`)

The `global-shortcut:allow-register` permission is technically included but unused by the overlay frontend — the overlay never registers any shortcuts.

## Simulated listening placeholder (not real live transcription)

During the `recording` phase, the overlay shows:
- Five CSS `animate-bounce` bars with staggered `animation-delay` values. These are purely decorative CSS elements with no connection to `AudioService`, microphone levels, or any audio data.
- Primary label: **"Listening…"**
- Secondary line: **"Live transcript preview coming later"**

No words the user speaks are displayed. No partial transcript is shown. No fake recognized text is generated. The bars do not reflect audio levels. This is a future feature placeholder only.

## Fix applied after initial QA (PR #17 review)

### Root cause

Manual QA found: overlay appeared but stayed stuck on default "Listening…" state; Cancel did not hide it.

**Root cause**: On Windows, WebView2 throttles JavaScript execution in hidden/background windows. The ptt-overlay window is created hidden at startup; its `listen("ptt-status")` handler may not be registered before the first events fire. Additionally, `AppHandle::emit("ptt-status", ...)` uses `EventTarget::Any` (global broadcast), which can be subject to this throttling for hidden webviews. Using `AppHandle::emit_to(window_label, "ptt-status", ...)` targets each window's IPC channel directly, bypassing the broadcast path.

A secondary issue: `handleCancel()` depended on receiving a `ptt-status: cancelled` event to hide the overlay. If that event was missed (same throttling issue), the overlay remained visible after Cancel.

### Fixes applied

**Fix 1 — Explicit emit_to for all ptt-status events**

Added `pub fn emit_ptt_status(app, phase, message)` to `src-tauri/src/services/ptt.rs`. This helper calls `app.emit_to("main", ...)` and `app.emit_to("ptt-overlay", ...)` separately, targeting each window's IPC channel directly. All 6 emit sites in `ptt.rs`, 1 in `commands/ptt.rs`, and 2 in `lib.rs` were replaced with this helper.

**Fix 2 — Pre-spawn recording emit in ptt_start()**

In `lib.rs` `ptt_start()`, `emit_ptt_status(app, "recording", "Recording…")` is now called between `overlay.show()` and `std::thread::spawn`. This guarantees the overlay receives a real event in the window immediately after it is shown, before the audio thread starts.

**Fix 3 — Cancel hides locally on success**

`handleCancel()` now calls `await hideOverlay()` after a successful `cancelPtt()` call, without waiting for a `ptt-status` event. If `cancelPtt()` fails, it sets an error state in the overlay.

**Fix 4 — Timer managed via useRef**

The auto-hide timer is stored in `autoHideTimerRef`. Existing timers are cleared before a new one is set. Cleanup on unmount cancels any pending timer.

**Fix 5 — Draggable overlay**

The overlay body area uses `data-tauri-drag-region` on the content div (left/centre). Tauri v2 excludes interactive elements (buttons) from triggering drag, so Cancel and Dismiss remain clickable. `cursor-move` provides a visual hint. The overlay starts at bottom-centre; the user can drag it anywhere during the session (position resets on restart).

## Second fix round applied after second QA (PR #17 review — round 2)

### Root cause

After fix round 1, manual QA found three remaining failures:
1. Overlay did not disappear after "Done."
2. Cancel did not hide the overlay.
3. Drag did not move the overlay.

**Root cause for hide failures**: `getCurrentWindow().hide()` requires the `core:window:allow-hide` capability. This permission is NOT included in `core:default`. Without it, `hide()` silently fails (promise rejects, caught and swallowed). The backend never hid the window either.

**Root cause for drag failure**: `data-tauri-drag-region` is a WebView2 HTML attribute that relies on Tauri intercepting `WM_NCHITTEST`. In this project's configuration it did not trigger window dragging. The correct approach for programmatic drag is `getCurrentWindow().startDragging()`, which requires the `core:window:allow-start-dragging` capability.

**Root cause for Cancel not stopping recording**: `cancel_ptt` set the cancel flag and emitted a "cancelled" event but never called `overlay.hide()` from the backend. The JS `hideOverlay()` call also failed silently due to the missing capability.

### Fixes applied (round 2)

**Fix 1 — Add missing capabilities**

`src-tauri/capabilities/default.json`: added `core:window:allow-hide` and `core:window:allow-start-dragging` to the permissions array. These are scoped to both `main` and `ptt-overlay` windows (the `windows` array already contains both).

**Fix 2 — Backend hide for Done phase**

Added `pub fn hide_ptt_overlay(app)` and `pub fn hide_ptt_overlay_after(app, delay_ms)` helpers to `src-tauri/src/services/ptt.rs`. `run_pipeline()` calls `hide_ptt_overlay_after(handle, 1500)` after emitting "done". The `is_cancelled()` helper calls `hide_ptt_overlay(handle)` immediately. This ensures the overlay disappears even if the JS `hide()` call is slow or races.

**Fix 3 — Backend hide in cancel_ptt command**

`src-tauri/src/commands/ptt.rs`: after `emit_ptt_status(..., "cancelled", ...)`, calls `hide_ptt_overlay(&app_handle)` directly. The overlay is hidden at the Rust level before the command returns.

**Fix 4 — Replace data-tauri-drag-region with startDragging()**

`src/components/PttOverlay.tsx`: removed `data-tauri-drag-region` attribute. Added `handleDragStart(e)` function that calls `getCurrentWindow().startDragging()` on left-pointer-down. Content area div uses `onPointerDown={handleDragStart}`. Cancel and Dismiss buttons are in a sibling div outside the drag area and are not affected.

## Files changed

| File | Change |
|---|---|
| `src-tauri/capabilities/default.json` | Added `"ptt-overlay"` to `windows` array; added `core:window:allow-hide` and `core:window:allow-start-dragging` permissions |
| `src-tauri/src/lib.rs` | Created `ptt-overlay` `WebviewWindowBuilder` in `setup()`; `overlay.show()` + `emit_ptt_status` pre-spawn emit in `ptt_start()`; thread emits replaced with helper |
| `src-tauri/src/services/ptt.rs` | Added `emit_ptt_status`, `hide_ptt_overlay`, `hide_ptt_overlay_after` helpers; replaced all `handle.emit()` calls; `run_pipeline()` calls `hide_ptt_overlay_after(1500)` after done; `is_cancelled()` calls `hide_ptt_overlay` |
| `src-tauri/src/commands/ptt.rs` | Replaced `app_handle.emit()` with `emit_ptt_status`; added `hide_ptt_overlay` call after cancel; removed unused imports |
| `src/App.tsx` | Extracted `MainApp` component; `App` checks `IS_PTT_OVERLAY` constant and renders `<PttOverlay />` if true, else `<MainApp />` |
| `src/components/PttOverlay.tsx` | Standalone overlay component: `ptt-status` listener, animated waveform bars, phase labels; Cancel hides locally after success; timer managed via `useRef`; `onPointerDown` drag via `startDragging()`; buttons outside drag area |

## Privacy and safety guarantees

- `ptt-status` events contain only `phase` (a keyword string) and `message` (a generic status string such as "Transcribing…"). Transcript text, clipboard contents, and audio data are never included — this is enforced by existing `ptt.rs` code, unchanged.
- `PttOverlay` renders only `phase` and `message`. It has no access to history, audio buffers, clipboard, or transcript text.
- The only Tauri command called by the overlay is `cancel_ptt`.
- No logging, no telemetry, no network calls, no screen reading, no OCR.

## Checks run

| Check | Result |
|---|---|
| `cargo fmt --check` | ✅ Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ Pass (0 warnings) |
| `cargo test` | ✅ 152/152 pass (round 2) |
| `npm run lint` (`tsc --noEmit`) | ✅ 0 errors |
| `npm run build` | ✅ Pass (307.85 kB JS) |
| `pwsh scripts/quality-check.ps1` | ✅ All checks passed |

## Manual QA steps and results

The following tests must be verified with a real Tauri build and a configured Whisper binary + model. This handoff documents the expected behavior; live QA results should be filled in by the reviewer.

### Golden path — Notepad focus test

1. Start the app.
2. Open Notepad. Type a few characters. Keep Notepad focused.
3. Ensure Settings → Shortcut behavior = `push_to_talk_toggle`.
4. Ensure Whisper binary and model are configured in Settings → Models.
5. Press the PTT shortcut.
6. **Expected**: `ptt-overlay` appears at bottom-center. Animated bars visible. Label "Listening…". Sub-label "Live transcript preview coming later". **Notepad retains focus** — cursor still active in Notepad.
7. Speak a short sentence.
8. Press the PTT shortcut again.
9. **Expected**: overlay transitions "Transcribing…" → (if cleanup provider configured) "Cleaning…" → "Inserting…" → "Done." → overlay disappears after 1.5 s.
10. **Expected**: spoken text appears in Notepad (not in transtypro). Focus was never stolen.

### Cancel path

1. Open Notepad, keep it focused.
2. Press PTT shortcut. Overlay appears with animated bars.
3. Press the **Cancel** button in the overlay.
4. **Expected**: overlay disappears immediately. No transcription runs. No text inserted into any app.

### Error path

1. Remove the Whisper binary path in Settings → Models.
2. Press PTT shortcut, speak, press shortcut again.
3. **Expected**: overlay shows error message in red indicator + text. Dismiss (✕) button visible.
4. Press Dismiss.
5. **Expected**: overlay hides. Main transtypro window was already brought to front by existing error handler in `ptt.rs`.

### Drag overlay to new position

1. When the overlay is visible (recording phase), drag it from its default bottom-centre position to another location on screen.
2. **Expected**: overlay moves to the dragged position.
3. Press PTT shortcut again (stop → run pipeline).
4. **Expected**: overlay remains at the moved position during the session, transitions phase labels, then disappears after Done.
5. **Expected**: Cancel button still clickable (not a drag trigger).
6. **Expected**: final text still inserts into Notepad.
7. Note: position resets to bottom-centre on next app launch.

### open_dictation regression

1. Change Shortcut behavior to `open_dictation`.
2. Press PTT shortcut.
3. **Expected**: main transtypro window opens or comes to front; main-window `FloatingOverlay` appears inside the main window. `ptt-overlay` remains hidden — this mode does not use the overlay.

### Main window PTT display regression

1. Change Shortcut behavior back to `push_to_talk_toggle`.
2. Bring the main transtypro window to the foreground.
3. Press PTT shortcut.
4. **Expected**: BOTH the `ptt-overlay` window (bottom-center) AND the main window's `FloatingOverlay` (via `ShortcutHandler` / `uiStore`) show the PTT status simultaneously. Both receive events from `handle.emit("ptt-status", …)` which broadcasts to all windows.

## Known limitations

- On Windows, `focused(false)` maps to `WS_EX_NOACTIVATE`. Most focus-granting paths are blocked; OS-level edge cases on some Windows builds are theoretically possible but not observed in testing.
- The animated bars do not reflect real audio levels. They are decorative only.
- Window position is calculated from `primary_monitor()`. Multi-monitor configurations where the active app is on a secondary monitor will show the overlay on the primary monitor — acceptable for Phase 10.1.
- The overlay uses the same CSS design tokens as the main window (loaded from `index.html` → `index.css`). If a future light-theme feature is added, the overlay will need to respect it.

## Confirmed safe behaviors

- No forbidden files were touched.
- No telemetry added.
- No transcript text, clipboard contents, or audio data logged or sent.
- No cloud storage, no model downloads, no account/login/sync.
- No active-app content reading, no screen capture, no OCR.
- No unsafe shell commands.
- `AudioService`, `TranscriptionService`, `CleanupService`, `InsertionService`, `PttPipelineService`, `HistoryService`, privacy logic, and provider behavior are all unchanged.
- No real live transcription implemented.
- No live insertion while speaking.
- No partial transcript preview.
- No packaging changes.

## Next recommended task

- Phase 11: Packaging (Tauri bundler, installer, update manifest).
- Optional: audio level meter in overlay (connect `get_recording_status` RMS to bar heights during recording phase).
- Optional: position overlay on the monitor containing the active app (requires active-app context capture from Phase 11+).
