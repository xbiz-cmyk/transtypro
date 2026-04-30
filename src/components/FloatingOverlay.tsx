import { useUiStore } from "../stores/uiStore";

export default function FloatingOverlay() {
  const { overlayOpen, activeMode, toggleOverlay } = useUiStore();

  if (!overlayOpen) return null;

  return (
    <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50 pointer-events-auto">
      <div className="flex items-center gap-4 bg-(--color-surface-overlay) border border-(--color-border-default) rounded-2xl px-6 py-4 shadow-2xl min-w-72">
        {/* Pulse indicator — visual-only, not functional */}
        <span className="relative flex h-3 w-3">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-(--color-status-error) opacity-75" />
          <span className="relative inline-flex rounded-full h-3 w-3 bg-(--color-status-error)" />
        </span>

        {/* Mode label */}
        <div className="flex-1">
          <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-0.5">
            Active mode
          </p>
          <p className="text-sm font-semibold text-(--color-text-primary)">
            {activeMode}
          </p>
        </div>

        {/* Close / dismiss button */}
        <button
          onClick={toggleOverlay}
          className="text-(--color-text-muted) hover:text-(--color-text-primary) transition-colors p-1 rounded"
          title="Dismiss overlay"
        >
          ✕
        </button>
      </div>

      <p className="text-center text-xs text-(--color-text-muted) mt-2">
        Dictation not yet active — Phase 6
      </p>
    </div>
  );
}
