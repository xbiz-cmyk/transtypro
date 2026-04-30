import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getStatusSummary, ping } from "../lib/api";
import type { StatusSummary } from "../lib/types";
import Card, { CardHeader } from "../components/ui/Card";
import Badge from "../components/ui/Badge";
import ErrorMessage from "../components/ui/ErrorMessage";

export default function Home() {
  const [status, setStatus] = useState<StatusSummary | null>(null);
  const [backendOk, setBackendOk] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
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

  const isLocalOnly = status ? status.privacy_mode === "local-only" : true;

  return (
    <div id="home-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Welcome to transtypro
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Speak instead of type — local-first AI dictation for your desktop.
      </p>

      {error && <ErrorMessage message={error} className="mb-6" />}

      {/* Status cards row */}
      <div className="grid grid-cols-2 gap-4 mb-6">
        {/* Active mode card */}
        <Card>
          <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-2">
            Active mode
          </p>
          <p className="text-lg font-semibold text-(--color-text-primary)">
            {status?.active_mode ?? "Smart"}
          </p>
          <Link
            to="/modes"
            className="text-xs text-(--color-brand-300) hover:underline mt-1 block"
          >
            Change mode →
          </Link>
        </Card>

        {/* Privacy mode card */}
        <Card>
          <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-2">
            Privacy mode
          </p>
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full ${isLocalOnly ? "bg-(--color-status-success)" : "bg-(--color-status-warning)"}`}
            />
            <p className="text-lg font-semibold text-(--color-text-primary)">
              {isLocalOnly ? "Local only" : "Cloud enabled"}
            </p>
          </div>
          <Link
            to="/privacy"
            className="text-xs text-(--color-brand-300) hover:underline mt-1 block"
          >
            View privacy →
          </Link>
        </Card>

        {/* Last transcription card */}
        <Card>
          <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-2">
            Last transcription
          </p>
          <p className="text-sm text-(--color-text-muted) italic">
            {status && status.history_count > 0
              ? `${status.history_count} session(s) recorded`
              : "No sessions yet"}
          </p>
          <Link
            to="/history"
            className="text-xs text-(--color-brand-300) hover:underline mt-1 block"
          >
            View history →
          </Link>
        </Card>

        {/* Quick start card */}
        <Card>
          <p className="text-xs text-(--color-text-muted) uppercase tracking-wider mb-2">
            Quick start
          </p>
          <p className="text-sm text-(--color-text-secondary)">
            Go to{" "}
            <Link
              to="/dictation"
              className="text-(--color-brand-300) hover:underline"
            >
              Dictation
            </Link>{" "}
            to record your first session.
          </p>
          <p className="text-xs text-(--color-text-muted) mt-1">
            Recording available in Phase 3
          </p>
        </Card>
      </div>

      {/* System status */}
      <Card>
        <CardHeader>System status</CardHeader>
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
            state={
              backendOk === null ? "loading" : backendOk ? "ok" : "error"
            }
          />
          <StatusRow
            label="Transcription model"
            value={
              status?.transcription_ready ? "Ready" : "Not configured"
            }
            state={status?.transcription_ready ? "ok" : "info"}
          />
          <StatusRow
            label="Cleanup provider"
            value={status?.cleanup_provider ?? "None"}
            state="info"
          />
          <StatusRow
            label="History entries"
            value={String(status?.history_count ?? 0)}
            state="info"
          />
        </div>
      </Card>

      {/* Transcript model badge */}
      <div className="mt-4 flex items-center gap-2">
        {status?.transcription_ready ? (
          <Badge variant="success">Model ready</Badge>
        ) : (
          <Badge variant="muted">No model — configure in Models</Badge>
        )}
        {isLocalOnly && <Badge variant="success">Local only</Badge>}
      </div>
    </div>
  );
}

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
      <span className="text-sm text-(--color-text-primary) font-medium">
        {value}
      </span>
    </div>
  );
}
