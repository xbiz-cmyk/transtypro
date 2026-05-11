import { useEffect, useState } from "react";
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
  // Default to recording state so the correct UI is visible even if the
  // first ptt-status event arrives before this component's listener fires.
  const [phase, setPhase] = useState("recording");
  const [message, setMessage] = useState("Listening…");

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    listen<PttStatusEvent>("ptt-status", (e) => {
      const { phase: p, message: m } = e.payload;
      setPhase(p);
      setMessage(m);

      if (p === "done" || p === "idle" || p === "cancelled") {
        const delay = p === "done" ? 1500 : 0;
        setTimeout(() => {
          void getCurrentWindow().hide();
          // Reset to safe default so next activation starts correctly.
          setPhase("recording");
          setMessage("Listening…");
        }, delay);
      }
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        // If listen fails the overlay is still shown; PTT pipeline continues.
      });

    return () => {
      unlisten?.();
    };
  }, []);

  async function handleCancel() {
    try {
      await cancelPtt();
    } catch {
      // The ptt-status "cancelled" event will trigger the hide.
    }
  }

  async function handleDismiss() {
    try {
      await getCurrentWindow().hide();
    } catch {
      // ignore
    }
    setPhase("recording");
    setMessage("Listening…");
  }

  const displayLabel =
    phase === "recording" ? "Listening…" : message;

  return (
    <div className="w-full h-screen flex items-center gap-3 px-4 bg-(--color-surface-overlay) border border-(--color-border-default) rounded-2xl overflow-hidden select-none">
      <PhaseIndicator phase={phase} />

      <div className="flex-1 min-w-0">
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

      {CANCELLABLE.has(phase) && (
        <button
          onClick={handleCancel}
          className="text-xs text-(--color-text-muted) hover:text-(--color-status-error) border border-(--color-border-subtle) rounded px-2 py-1 shrink-0 transition-colors"
        >
          Cancel
        </button>
      )}

      {phase === "error" && (
        <button
          onClick={handleDismiss}
          className="text-(--color-text-muted) hover:text-(--color-text-primary) rounded p-1 shrink-0 transition-colors"
          title="Dismiss"
        >
          ✕
        </button>
      )}
    </div>
  );
}
