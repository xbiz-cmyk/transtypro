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

## Files changed

| File | Change |
|---|---|
| `src-tauri/capabilities/default.json` | Added `"ptt-overlay"` to `windows` array |
| `src-tauri/src/lib.rs` | Created `ptt-overlay` `WebviewWindowBuilder` in `setup()`; added `overlay.show()` call in `ptt_start()` before spawning the recording thread |
| `src/App.tsx` | Extracted `MainApp` component; `App` checks `IS_PTT_OVERLAY` constant and renders `<PttOverlay />` if true, else `<MainApp />` |
| `src/components/PttOverlay.tsx` | **New file.** Standalone overlay component: `ptt-status` listener, animated waveform bars, phase labels, Cancel/Dismiss buttons, window hide on terminal phases |

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
| `cargo test` | ✅ 152/152 pass |
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
