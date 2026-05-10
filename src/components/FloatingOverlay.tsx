import { Link } from "react-router-dom";
import { useUiStore } from "../stores/uiStore";
import { cancelPtt } from "../lib/api";

/** PTT phases where the Cancel button is shown. */
const CANCELLABLE_PHASES = new Set(["recording", "transcribing", "cleaning"]);

/** Indicator color class per PTT phase. */
function indicatorColor(phase: string): string {
  if (phase === "recording") return "bg-red-500";
  if (phase === "error") return "bg-red-500";
  if (phase === "done") return "bg-green-500";
  return "bg-(--color-brand-400)";
}

/** Whether the indicator should pulse. */
function shouldPulse(phase: string): boolean {
  return phase !== "done" && phase !== "idle";
}

export default function FloatingOverlay() {
  const { overlayOpen, activeMode, closeOverlay, pttPhase, pttMessage } =
    useUiStore();

  if (!overlayOpen) return null;

  const isPttActive = pttPhase !== "idle";

  async function handleCancel() {
    try {
      await cancelPtt();
    } catch {
      // Overlay will close via the ptt-status "cancelled" event.
    }
  }

  return (
    <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50 pointer-events-auto">
      <div className="flex items-center gap-4 bg-(--color-surface-overlay) border border-(--color-border-default) rounded-2xl px-6 py-4 shadow-2xl min-w-72">
        {/* Pulse indicator */}
        <span className="relative flex h-3 w-3 shrink-0">
          {shouldPulse(isPttActive ? pttPhase : "active") && (
            <span
              className={`animate-ping absolute inline-flex h-full w-full rounded-full ${
                isPttActive
                  ? indicatorColor(pttPhase)
                  : "bg-(--color-brand-400)"
              } opacity-75`}
            />
          )}
          <span
            className={`relative inline-flex rounded-full h-3 w-3 ${
              isPttActive
                ? indicatorColor(pttPhase)
                : "bg-(--color-brand-400)"
            }`}
          />
        </span>

        {/* Status label */}
        <div className="flex-1 min-w-0">
          {isPttActive ? (
            <>
              <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-0.5">
                Push-to-talk
              </p>
              <p className="text-sm font-semibold text-(--color-text-primary) truncate">
                {pttPhase === "error" ? pttMessage : pttMessage || pttPhase}
              </p>
            </>
          ) : (
            <>
              <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-0.5">
                Active mode
              </p>
              <p className="text-sm font-semibold text-(--color-text-primary)">
                {activeMode}
              </p>
            </>
          )}
        </div>

        {/* Cancel button — only shown during cancellable PTT phases */}
        {isPttActive && CANCELLABLE_PHASES.has(pttPhase) && (
          <button
            onClick={handleCancel}
            className="text-xs text-(--color-text-muted) hover:text-(--color-status-error) transition-colors px-2 py-1 rounded border border-(--color-border-subtle) shrink-0"
            title="Cancel dictation"
          >
            Cancel
          </button>
        )}

        {/* Dismiss button — shown in open_dictation mode or on error */}
        {(!isPttActive || pttPhase === "error") && (
          <button
            onClick={closeOverlay}
            className="text-(--color-text-muted) hover:text-(--color-text-primary) transition-colors p-1 rounded shrink-0"
            title="Dismiss overlay"
          >
            ✕
          </button>
        )}
      </div>

      {/* Go to Dictation link — only in open_dictation mode (no active PTT) */}
      {!isPttActive && (
        <Link
          to="/dictation"
          onClick={closeOverlay}
          className="block text-center text-xs text-(--color-accent) hover:underline mt-2"
        >
          Go to Dictation →
        </Link>
      )}
    </div>
  );
}
