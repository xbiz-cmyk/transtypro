# Wave 10.1 Launch Record — PTT Feedback Overlay Window

## Current main commit

`8274875` — feat(ptt): add push-to-talk dictation pipeline

## Phase scope

**Phase 10.1: PTT feedback overlay window**

Show a small, always-on-top, non-focusable status overlay while push-to-talk is active. The overlay is a separate Tauri window so it remains visible even when the main transtypro window is minimized or behind the active app.

## Issue found during manual testing

Push-to-talk works correctly (shortcut fires → recording → transcription → insertion). However, when the user presses the PTT shortcut while working in another app, they get no visible feedback. The main transtypro window intentionally does not steal focus in PTT mode, so the `FloatingOverlay` inside the main window is invisible. The user cannot tell whether recording started, whether transcription is running, or whether an error occurred.

## Branch

`phase/10.1-ptt-overlay-window`

## Worktree path

`C:\Users\User\Desktop\transtypro-ptt-overlay`

## Approved architecture

**Option B — separate Tauri secondary window** named `ptt-overlay`.

Create the window at app startup (hidden). Show it at `ptt_start()` before emitting the first `ptt-status: recording` event. Hide it when the phase reaches `done`, `idle`, or `cancelled`.

### Window properties

```rust
tauri::WebviewWindowBuilder::new(
    app.handle(),
    "ptt-overlay",
    tauri::WebviewUrl::App("index.html".into()),
)
.title("")
.inner_size(320.0, 96.0)
.always_on_top(true)
.skip_taskbar(true)
.decorations(false)
.visible(false)
.focused(false)
.resizable(false)
.minimizable(false)
.maximizable(false)
.closable(false)
.build()?
```

Position: bottom-center of the primary monitor, 16 px above the taskbar. Position is set after `build()` using monitor geometry.

### App.tsx label detection (minimal change)

```tsx
import { getCurrentWindow } from "@tauri-apps/api/window";
const IS_PTT_OVERLAY = getCurrentWindow().label === "ptt-overlay";

export default function App() {
  if (IS_PTT_OVERLAY) return <PttOverlay />;
  // existing render unchanged
}
```

### PttOverlay.tsx (new component)

- Listens to `ptt-status` Tauri events.
- Shows phase-appropriate status message and animated bars during recording.
- Hides window (`getCurrentWindow().hide()`) when phase is `done`, `idle`, or `cancelled`.
- Provides a Cancel button for `recording`, `transcribing`, `cleaning` phases — calls `cancelPtt()`.
- Does not navigate or render any other app UI.
- Does not display transcript text at any time.

### Animated waveform bars (simulated — no audio data)

Five CSS-animated bars using `animate-bounce` with fixed heights and staggered delays. These are cosmetic only. They do not reflect real audio levels. No connection to AudioService.

## Focus safety rules

- The `ptt-overlay` window must never call `set_focus()`, `set_always_on_top()` after creation (already set at build), or any focus-granting API.
- `show()` must be used without `set_focus()`.
- The active app must remain focused when the overlay appears.
- Insertion via clipboard paste (InsertionService) runs ~10+ seconds after focus is last touched; focus must be undisturbed at that point.
- Test verification: open Notepad, trigger PTT, confirm text pastes into Notepad (not transtypro).

## First-event delivery (Correction 2)

**Required**: the `ptt-overlay` window must be loaded and listening before the first `ptt-status` event arrives.

Implementation requirement: in `lib.rs`, `ptt_start()` must call `window.show()` **before** emitting `ptt-status: recording`. The sequence must be:

1. Acquire lock, transition Idle → Recording.
2. Show `ptt-overlay` window (synchronous call, no await).
3. Spawn audio recording thread.
4. Thread emits `ptt-status: { phase: "recording", message: "Listening…" }`.

Because the window is pre-created at startup and merely shown here (not loaded on demand), the webview is already hydrated and the `ptt-status` listener registered when step 4 fires.

If the first event is still missed due to React hydration latency, `PttOverlay` must start with a safe default state of `phase: "recording"` and rely on subsequent events for transitions.

## Capability file correction (Correction 1)

If Tauri's capability system blocks the `ptt-overlay` window from:
- receiving `ptt-status` events
- calling `hide()` on itself
- calling the `cancel_ptt` Tauri command

then the agent must add `ptt-overlay` to the allowed-windows list in `src-tauri/capabilities/default.json` — making only the minimal change required. Do not broaden permissions for other windows or add capabilities beyond what `ptt-overlay` needs.

## Simulated live transcript placeholder rules

- During the `recording` phase: show animated bars + "Listening…" or "Capturing speech…".
- Do not display any words the user is speaking.
- Do not display a partial transcript.
- Do not display fake recognized words.
- The animated bars are purely decorative CSS.
- The phase label ("Transcribing…", "Cleaning…", "Inserting…") is the only dynamic content derived from events.
- A note in the UI or handoff may say "Live transcript preview coming later" — this is a future feature placeholder only, not implemented here.

## Local-first privacy rules

- `ptt-status` messages must never contain transcript text, clipboard contents, or user-dictated content.
- `PttOverlay` must not log, display, or transmit any audio data.
- The overlay window must not have access to backend services beyond `cancel_ptt`.
- No audio, text, clipboard contents, or diagnostics are sent anywhere by the overlay.

## Allowed files for Phase 10.1 agent

The agent may create or edit these files only:

- `src-tauri/src/lib.rs` — create ptt-overlay window at startup; show window in ptt_start before emitting first event
- `src-tauri/capabilities/default.json` — only if required to allow ptt-overlay to receive events or call cancel_ptt; minimal change only
- `src/App.tsx` — add label check, render PttOverlay if ptt-overlay window
- `src/components/PttOverlay.tsx` — new component (create)
- `handoff/phase-10.1-ptt-overlay-window.md` — implementation handoff (create after implementation)
- `docs/PROGRESS.md` — update only after implementation succeeds
- `docs/TASK_BOARD.md` — update only after implementation succeeds

## Files the agent may inspect but should avoid editing

Edit only if absolutely required by the task:

- `src-tauri/src/services/ptt.rs`
- `src-tauri/src/commands/ptt.rs`
- `src/components/FloatingOverlay.tsx`
- `src/components/ShortcutHandler.tsx`
- `src/stores/uiStore.ts`
- `src/lib/api.ts`
- `src/lib/types.ts`

## Forbidden files for Phase 10.1 agent

Do not edit:

- `src-tauri/src/services/audio.rs`
- `src-tauri/src/commands/audio.rs`
- `src-tauri/src/services/transcription.rs`
- `src-tauri/src/commands/transcription.rs`
- `src-tauri/src/services/cleanup.rs`
- `src-tauri/src/commands/cleanup.rs`
- `src-tauri/src/services/insertion.rs`
- `src-tauri/src/commands/insertion.rs`
- `src-tauri/src/services/history.rs`
- `src-tauri/src/commands/history.rs`
- `src-tauri/src/services/providers.rs`
- `src-tauri/src/commands/providers.rs`
- `src-tauri/src/services/privacy.rs`
- `src-tauri/src/commands/privacy.rs`
- `src-tauri/src/services/retention.rs`
- `src-tauri/src/services/diagnostics.rs`
- `src-tauri/src/commands/diagnostics.rs`
- `src-tauri/src/db/**`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/errors/mod.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/commands/mod.rs`
- `src/pages/**`
- `docs/PHASES.md`
- `docs/PARALLEL_EXECUTION_PLAN.md`
- `docs/ARCHITECTURE.md`
- `AGENTS.md`
- `CLAUDE.md`
- `SOUL.md`

## Implementation reminders for Phase 10.1 agent

- Create ptt-overlay window at startup, hidden by default.
- Show the window in `ptt_start()` **before** emitting the first `ptt-status` event.
- The window must be small (320×96 px) and always on top.
- It must not call `set_focus()` or any focus-granting API.
- It must not steal focus from the active app.
- It must not break clipboard paste insertion.
- Recording phase shows animated CSS bars + "Listening…" — no audio data, no real transcript.
- Transcribing / cleaning / inserting / done / error phases show a plain status label.
- Do not display fake transcript text.
- Do not implement real live transcription.
- Do not implement live insertion.
- Do not send or log user-dictated content.
- If capabilities block the overlay, make only the minimal `default.json` change.
- Test with Notepad: open Notepad, press PTT shortcut, speak, confirm text inserts into Notepad.
- Keep existing `FloatingOverlay` in the main window working.
- Keep existing PTT pipeline working.
- Keep existing manual Dictation flow working.
- Run `cargo test` (152 tests must pass), `cargo clippy`, `npm run build`, `npm run lint`.

## Merge rule

No PR may be merged unless the orchestrator provides the exact line:

```
ORCHESTRATOR APPROVED MERGE
```

## Next step

Launch the Phase 10.1 implementation agent in the worktree at:

`C:\Users\User\Desktop\transtypro-ptt-overlay`

Branch: `phase/10.1-ptt-overlay-window`
