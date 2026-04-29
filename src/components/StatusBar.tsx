/**
 * transtypro — Status bar component.
 *
 * Top bar showing privacy mode indicator and connection status.
 * Provides at-a-glance trust information per SOUL.md.
 */

interface StatusBarProps {
  privacyMode: string;
  version: string;
}

export default function StatusBar({ privacyMode, version }: StatusBarProps) {
  const isLocalOnly = privacyMode === "local-only";

  return (
    <header
      id="status-bar"
      className="h-11 bg-(--color-surface-raised) border-b border-(--color-border-default) flex items-center justify-between px-5"
    >
      {/* Privacy indicator */}
      <div className="flex items-center gap-2">
        <span
          id="privacy-indicator"
          className={`inline-block w-2 h-2 rounded-full ${
            isLocalOnly ? "bg-(--color-status-success)" : "bg-(--color-status-warning)"
          }`}
          title={isLocalOnly ? "Local-only mode — no data leaves this device" : "Cloud-enabled mode"}
        />
        <span className="text-xs font-medium text-(--color-text-secondary)">
          {isLocalOnly ? "Local only" : "Cloud enabled"}
        </span>
      </div>

      {/* Version */}
      <span className="text-xs text-(--color-text-muted)">
        v{version}
      </span>
    </header>
  );
}
