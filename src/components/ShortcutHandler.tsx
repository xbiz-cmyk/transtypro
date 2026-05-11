import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "../stores/uiStore";
import type { PttStatusEvent } from "../lib/types";

/** PTT phases that keep the overlay visible. */
const ACTIVE_PTT_PHASES = new Set([
  "recording",
  "transcribing",
  "cleaning",
  "inserting",
  "error",
]);

export default function ShortcutHandler() {
  const navigate = useNavigate();
  const openOverlay = useUiStore((s) => s.openOverlay);
  const closeOverlay = useUiStore((s) => s.closeOverlay);
  const setPttStatus = useUiStore((s) => s.setPttStatus);

  // Keep the auto-hide timer ref so we can clear it if another event arrives.
  const doneTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const cleanups: Array<() => void> = [];

    // ── dictation-shortcut-pressed ─────────────────────────────────────────
    // Fires in open_dictation mode only. Unchanged from Phase 7.
    listen<null>("dictation-shortcut-pressed", () => {
      openOverlay();
      navigate("/dictation");
    }).then((fn) => cleanups.push(fn));

    // ── ptt-status ─────────────────────────────────────────────────────────
    // Fires from the PTT pipeline in push_to_talk_hold / push_to_talk_toggle mode.
    listen<PttStatusEvent>("ptt-status", (e) => {
      const { phase, message } = e.payload;

      // Clear any pending done-timer so rapid phase changes don't close early.
      if (doneTimerRef.current !== null) {
        clearTimeout(doneTimerRef.current);
        doneTimerRef.current = null;
      }

      setPttStatus(phase, message);

      if (ACTIVE_PTT_PHASES.has(phase)) {
        // Open the overlay so the user can see the status if the window is visible.
        // If the window is minimized the overlay state change is invisible — that is correct.
        openOverlay();
      }

      if (phase === "done") {
        openOverlay();
        // Auto-hide overlay after 2 seconds.
        doneTimerRef.current = setTimeout(() => {
          setPttStatus("idle", "");
          closeOverlay();
          doneTimerRef.current = null;
        }, 2000);
      }

      if (phase === "idle" || phase === "cancelled") {
        setPttStatus("idle", "");
        closeOverlay();
      }
    }).then((fn) => cleanups.push(fn));

    return () => {
      cleanups.forEach((fn) => fn());
      if (doneTimerRef.current !== null) {
        clearTimeout(doneTimerRef.current);
      }
    };
  }, [navigate, openOverlay, closeOverlay, setPttStatus]);

  return null;
}
