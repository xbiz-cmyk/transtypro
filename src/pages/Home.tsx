import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getStatusSummary, ping } from "../lib/api";
import type { StatusSummary } from "../lib/types";
import Logo from "../components/Logo";
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
    <div id="home-page" className="p-8 max-w-xl">
      {/* ── Hero ─────────────────────────────────────────────────── */}
      <div className="mb-10">
        <Logo size={38} className="mb-5" />
        <h1 className="text-4xl font-bold text-(--color-text-primary) tracking-tight leading-tight mb-3">
          Speak. Clean. Insert.
        </h1>
        <p className="text-base text-(--color-text-secondary) mb-7 leading-relaxed">
          Fast local dictation for every desktop app.
        </p>
        <div className="flex items-center gap-3">
          <Link
            to="/dictation"
            id="home-start-dictating"
            className="inline-flex items-center gap-2 px-5 py-2.5 rounded-(--radius-btn) bg-(--color-brand-500) hover:bg-(--color-brand-400) text-white text-sm font-semibold transition-colors duration-150 shadow-sm"
          >
            Start dictating
            <ArrowRightIcon />
          </Link>
          <Link
            to="/settings"
            className="inline-flex items-center gap-2 px-4 py-2.5 rounded-(--radius-btn) bg-transparent hover:bg-(--color-surface-raised) text-(--color-text-secondary) hover:text-(--color-text-primary) border border-(--color-border-default) text-sm font-medium transition-colors duration-150"
          >
            Configure push-to-talk
          </Link>
        </div>
      </div>

      {error && <ErrorMessage message={error} className="mb-6" />}

      {/* ── Status chips ─────────────────────────────────────────── */}
      <div className="flex flex-wrap gap-2 mb-8">
        <StatusChip
          label="Backend"
          value={backendOk === null ? "…" : backendOk ? "Connected" : "Offline"}
          state={backendOk === null ? "loading" : backendOk ? "ok" : "error"}
        />
        <StatusChip
          label="Model"
          value={status ? (status.transcription_ready ? "Ready" : "Not configured") : "…"}
          state={status === null ? "loading" : status.transcription_ready ? "ok" : "warn"}
        />
        <StatusChip
          label="Privacy"
          value={isLocalOnly ? "Local only" : "Cloud enabled"}
          state={isLocalOnly ? "ok" : "warn"}
        />
        {status !== null && (
          <StatusChip
            label="Sessions"
            value={String(status.history_count)}
            state="info"
          />
        )}
        {status?.cleanup_provider && (
          <StatusChip
            label="Cleanup"
            value={status.cleanup_provider}
            state="info"
          />
        )}
      </div>

      {/* ── Quick links ──────────────────────────────────────────── */}
      <div className="grid grid-cols-2 gap-2.5">
        <QuickLink
          to="/history"
          label="History"
          description={
            status && status.history_count > 0
              ? `${status.history_count} ${status.history_count === 1 ? "session" : "sessions"} recorded`
              : "No sessions yet"
          }
        />
        <QuickLink
          to="/modes"
          label="Active mode"
          description={
            status?.active_mode
              ? `${capitalize(status.active_mode)} mode`
              : "Smart mode"
          }
        />
        <QuickLink
          to="/privacy"
          label="Privacy"
          description={isLocalOnly ? "Local only — no cloud calls" : "Cloud enabled"}
        />
        <QuickLink
          to="/diagnostics"
          label="Diagnostics"
          description="Check system health"
        />
      </div>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────

function StatusChip({
  label,
  value,
  state,
}: {
  label: string;
  value: string;
  state: "ok" | "error" | "warn" | "info" | "loading";
}) {
  const dotColor = {
    ok:      "bg-(--color-status-success)",
    error:   "bg-(--color-status-error)",
    warn:    "bg-(--color-status-warning)",
    info:    "bg-(--color-brand-400)",
    loading: "bg-(--color-text-muted) animate-pulse",
  }[state];

  return (
    <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-(--color-surface-raised) border border-(--color-border-default) text-xs select-none">
      <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotColor}`} aria-hidden="true" />
      <span className="text-(--color-text-muted)">{label}</span>
      <span className="text-(--color-text-primary) font-medium">{value}</span>
    </div>
  );
}

function QuickLink({
  to,
  label,
  description,
}: {
  to: string;
  label: string;
  description: string;
}) {
  return (
    <Link
      to={to}
      className="group flex flex-col gap-1 p-4 rounded-(--radius-card) bg-(--color-surface-raised) border border-(--color-border-default) hover:border-(--color-brand-500)/25 hover:bg-(--color-surface-overlay) transition-colors duration-150"
    >
      <span className="text-sm font-medium text-(--color-text-secondary) group-hover:text-(--color-brand-300) transition-colors duration-100">
        {label} <span className="opacity-50">→</span>
      </span>
      <span className="text-xs text-(--color-text-muted) leading-snug">{description}</span>
    </Link>
  );
}

function ArrowRightIcon() {
  return (
    <svg
      width="13"
      height="13"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M5 12h14M12 5l7 7-7 7" />
    </svg>
  );
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
