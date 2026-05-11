import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cancelPtt } from "../lib/api";
import type { PttStatusEvent } from "../lib/types";

const CANCELLABLE = new Set(["recording", "transcribing", "cleaning"]);

function WaveformBars() {
  const bars = [
    { height: "h-2", delay: "0ms" },
    { height: "h-4", delay: "120ms" },
    { height: "h-6", delay: "240ms" },
    { height: "h-4", delay: "120ms" },
    { height: "h-2", delay: "0ms" },
  ];
  return (
    <div className="flex items-center gap-0.5 shrink-0">
      {bars.map((b, i) => (
        <span
          key={i}
          className={`${b.height} w-1.5 rounded-full bg-(--color-brand-400) animate-bounce`}
          style={{ animationDelay: b.delay }}
        />
      ))}
    </div>
  );
}

function PhaseIndicator({ phase }: { phase: string }) {
  if (phase === "recording") return <WaveformBars />;
  return (
    <span
      className={`h-2.5 w-2.5 rounded-full shrink-0 ${
        phase === "error"
          ? "bg-(--color-status-error)"
          : phase === "done"
            ? "bg-(--color-status-success)"
            : "bg-(--color-brand-400) animate-pulse"
      }`}
    />
  );
}

export default function PttOverlay() {
  // Default to recording so the correct UI is visible even if the first
  // ptt-status event arrives before the listener is registered.
  const [phase, setPhase] = useState("recording");
  const [message, setMessage] = useState("Listening…");
  const autoHideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function clearAutoHideTimer() {
    if (autoHideTimerRef.current !== null) {
      clearTimeout(autoHideTimerRef.current);
      autoHideTimerRef.current = null;
    }
  }

  async function hideOverlay() {
    clearAutoHideTimer();
    try {
      await getCurrentWindow().hide();
    } catch {
      // Ignore — the window may already be hidden.
    }
    setPhase("recording");
    setMessage("Listening…");
  }

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    listen<PttStatusEvent>("ptt-status", (e) => {
      const { phase: p, message: m } = e.payload;
      setPhase(p);
      setMessage(m);
      clearAutoHideTimer();

      if (p === "done") {
        autoHideTimerRef.current = setTimeout(() => {
          void hideOverlay();
        }, 1500);
      } else if (p === "idle" || p === "cancelled") {
        void hideOverlay();
      }
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        // listen() failure is non-fatal; overlay stays visible with default state.
      });

    return () => {
      unlisten?.();
      clearAutoHideTimer();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleCancel() {
    try {
      await cancelPtt();
      // Hide locally on success — do not wait for the ptt-status event,
      // which may arrive late due to event routing on some platforms.
      await hideOverlay();
    } catch {
      setPhase("error");
      setMessage("Cancel failed. Open transtypro to stop.");
    }
  }

  const displayLabel = phase === "recording" ? "Listening…" : message;

  return (
    <div className="w-full h-screen flex items-center bg-(--color-surface-overlay) border border-(--color-border-default) rounded-2xl overflow-hidden select-none">
      {/*
        Drag region: the entire left/centre area is a drag handle.
        Tauri v2 excludes button elements from triggering drag, so Cancel
        and Dismiss remain clickable even though they are children of this div.
        cursor-move provides a visual hint that the overlay is draggable.
      */}
      <div
        data-tauri-drag-region
        className="flex-1 flex items-center gap-3 px-4 h-full min-w-0 cursor-move"
      >
        <PhaseIndicator phase={phase} />
        <div className="min-w-0">
          <p className="text-[10px] text-(--color-text-muted) uppercase tracking-wider leading-none mb-0.5">
            Push-to-talk
          </p>
          <p className="text-sm font-semibold text-(--color-text-primary) truncate">
            {displayLabel}
          </p>
          {phase === "recording" && (
            <p className="text-[10px] text-(--color-text-muted) truncate mt-0.5">
              Live transcript preview coming later
            </p>
          )}
        </div>
      </div>

      {/* Buttons: sibling of drag region, not inside it */}
      <div className="flex items-center shrink-0 pr-3">
        {CANCELLABLE.has(phase) && (
          <button
            onClick={handleCancel}
            className="text-xs text-(--color-text-muted) hover:text-(--color-status-error) border border-(--color-border-subtle) rounded px-2 py-1 transition-colors"
          >
            Cancel
          </button>
        )}
        {phase === "error" && (
          <button
            onClick={() => void hideOverlay()}
            className="text-(--color-text-muted) hover:text-(--color-text-primary) rounded p-1 transition-colors"
            title="Dismiss"
          >
            ✕
          </button>
        )}
      </div>
    </div>
  );
}
