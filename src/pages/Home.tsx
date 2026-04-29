/**
 * transtypro — Home page.
 *
 * Shows application status summary and getting-started information.
 * This is the landing page the user sees after opening the app.
 */
import { useEffect, useState } from "react";
import { getStatusSummary, ping } from "../lib/api";
import type { StatusSummary } from "../lib/types";

export default function Home() {
  const [status, setStatus] = useState<StatusSummary | null>(null);
  const [backendOk, setBackendOk] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Verify IPC and load status
    async function init() {
      try {
        const pong = await ping();
        setBackendOk(pong === "pong");
      } catch {
        setBackendOk(false);
      }

      try {
        const summary = await getStatusSummary();
        setStatus(summary);
      } catch (err) {
        setError(String(err));
      }
    }
    init();
  }, []);

  return (
    <div id="home-page" className="p-8 max-w-3xl">
      {/* Page heading */}
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Welcome to transtypro
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Speak instead of type — local-first AI dictation for your desktop.
      </p>

      {/* Backend connection check */}
      <div
        id="connection-status"
        className="bg-(--color-surface-raised) border border-(--color-border-default) rounded-(--radius-card) p-5 mb-6"
      >
        <h2 className="text-sm font-medium text-(--color-text-secondary) uppercase tracking-wider mb-3">
          System Status
        </h2>
        <div className="space-y-3">
          <StatusRow
            label="Backend connection"
            value={
              backendOk === null
                ? "Checking..."
                : backendOk
                  ? "Connected"
                  : "Not connected"
            }
            state={backendOk === null ? "loading" : backendOk ? "ok" : "error"}
          />
          {status && (
            <>
              <StatusRow
                label="Privacy mode"
                value={status.privacy_mode === "local-only" ? "Local only" : "Cloud enabled"}
                state="ok"
              />
              <StatusRow
                label="Transcription model"
                value={status.transcription_ready ? "Ready" : "Not configured"}
                state={status.transcription_ready ? "ok" : "info"}
              />
              <StatusRow
                label="Cleanup provider"
                value={status.cleanup_provider ?? "None"}
                state="info"
              />
              <StatusRow
                label="Active mode"
                value={status.active_mode}
                state="ok"
              />
              <StatusRow
                label="History entries"
                value={String(status.history_count)}
                state="info"
              />
            </>
          )}
        </div>
      </div>

      {/* Error display */}
      {error && (
        <div className="bg-(--color-status-error)/10 border border-(--color-status-error)/30 rounded-(--radius-card) p-4 mb-6">
          <p className="text-sm text-(--color-status-error)">{error}</p>
        </div>
      )}

      {/* Getting started */}
      <div className="bg-(--color-surface-raised) border border-(--color-border-default) rounded-(--radius-card) p-5">
        <h2 className="text-sm font-medium text-(--color-text-secondary) uppercase tracking-wider mb-3">
          Getting Started
        </h2>
        <div className="space-y-2 text-sm text-(--color-text-secondary)">
          <p>
            transtypro is being built phase by phase. This is{" "}
            <span className="font-medium text-(--color-brand-300)">Phase 0</span> — the
            project skeleton is ready and the frontend-backend connection is verified.
          </p>
          <p>Upcoming phases will add:</p>
          <ul className="list-disc list-inside space-y-1 ml-2 text-(--color-text-muted)">
            <li>UI shell with all pages (Phase 1)</li>
            <li>Settings and local storage (Phase 2)</li>
            <li>Audio recording (Phase 3)</li>
            <li>Local transcription (Phase 4)</li>
            <li>AI text cleanup (Phase 5)</li>
            <li>End-to-end dictation (Phase 6)</li>
          </ul>
        </div>
      </div>
    </div>
  );
}

/** Individual status row within a status card. */
function StatusRow({
  label,
  value,
  state,
}: {
  label: string;
  value: string;
  state: "ok" | "error" | "info" | "loading";
}) {
  const dotColor = {
    ok: "bg-(--color-status-success)",
    error: "bg-(--color-status-error)",
    info: "bg-(--color-status-info)",
    loading: "bg-(--color-status-warning) animate-pulse",
  }[state];

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        <span className={`w-1.5 h-1.5 rounded-full ${dotColor}`} />
        <span className="text-sm text-(--color-text-secondary)">{label}</span>
      </div>
      <span className="text-sm text-(--color-text-primary) font-medium">{value}</span>
    </div>
  );
}
