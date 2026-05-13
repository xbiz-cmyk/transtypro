import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getStatusSummary, ping } from "../lib/api";
import type { StatusSummary } from "../lib/types";
import Logo from "../components/Logo";
import ErrorMessage from "../components/ui/ErrorMessage";
import {
  HistoryIcon,
  ModesIcon,
  PrivacyIcon,
  DiagnosticsIcon,
  DictationIcon,
} from "../components/icons";

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
    <div id="home-page" className="p-8 max-w-[900px]">
      <div className="grid grid-cols-[1fr_236px] gap-8 items-start">

        {/* ── Left column ──────────────────────────────────────── */}
        <div>

          {/* ── Hero ─────────────────────────────────────────── */}
          <div className="mb-10">
            {/* Brand wordmark accent */}
            <div className="flex items-center gap-2 mb-5">
              <Logo size={16} />
              <span className="text-[10px] font-bold uppercase tracking-[0.14em] text-(--color-brand-400)">
                transtypro
              </span>
            </div>

            <h1 className="text-[2.625rem] font-extrabold leading-[1.05] tracking-tight text-(--color-text-primary) mb-3">
              Speak.<br />
              Clean. Insert.
            </h1>

            <p className="text-sm text-(--color-text-secondary) leading-relaxed mb-7 max-w-xs">
              Fast local dictation for every desktop app.
            </p>

            <div className="flex items-center gap-2.5">
              <Link
                to="/dictation"
                id="home-start-dictating"
                className="inline-flex items-center gap-2 px-5 py-2.5 rounded-(--radius-btn) bg-(--color-brand-500) hover:bg-(--color-brand-400) text-white text-sm font-semibold transition-all duration-150 shadow-md hover:shadow-lg"
              >
                Start dictating
                <ArrowRightIcon />
              </Link>
              <Link
                to="/settings"
                className="inline-flex items-center px-4 py-2.5 rounded-(--radius-btn) text-(--color-text-secondary) hover:text-(--color-text-primary) text-sm font-medium border border-(--color-border-default) hover:border-(--color-brand-500)/40 hover:bg-(--color-surface-overlay) transition-all duration-150"
              >
                Configure push-to-talk
              </Link>
            </div>
          </div>

          {error && <ErrorMessage message={error} className="mb-6" />}

          {/* ── Status chips ───────────────────────────────── */}
          <div className="flex flex-wrap gap-2 mb-8">
            <StatusChip
              value={backendOk === null ? "Checking…" : backendOk ? "Connected" : "Offline"}
              state={backendOk === null ? "loading" : backendOk ? "ok" : "error"}
            />
            <StatusChip
              value={status === null ? "Checking…" : status.transcription_ready ? "Model ready" : "Model not set"}
              state={status === null ? "loading" : status.transcription_ready ? "ok" : "warn"}
            />
            <StatusChip
              value={isLocalOnly ? "Local only" : "Cloud enabled"}
              state={isLocalOnly ? "ok" : "warn"}
            />
            {status !== null && (
              <StatusChip
                value={`${status.history_count} saved`}
                state="info"
              />
            )}
            {status?.cleanup_provider && (
              <StatusChip
                value="Local cleanup"
                state="info"
              />
            )}
          </div>

          {/* ── Quick links ────────────────────────────────── */}
          <div className="grid grid-cols-2 gap-2">
            <QuickLink
              to="/history"
              label="History"
              subtitle="Review saved dictations"
              badge={
                status && status.history_count > 0
                  ? `${status.history_count} ${status.history_count === 1 ? "session" : "sessions"}`
                  : "No sessions yet"
              }
              icon={<HistoryIcon size={14} />}
            />
            <QuickLink
              to="/modes"
              label="Active mode"
              subtitle="Tune formatting behavior"
              badge={status?.active_mode ? capitalize(status.active_mode) : "Smart"}
              icon={<ModesIcon size={14} />}
            />
            <QuickLink
              to="/privacy"
              label="Privacy"
              subtitle="Local / cloud controls"
              badge={isLocalOnly ? "Local only" : "Cloud enabled"}
              icon={<PrivacyIcon size={14} />}
            />
            <QuickLink
              to="/diagnostics"
              label="Diagnostics"
              subtitle="Check system health"
              badge="Run checks"
              icon={<DiagnosticsIcon size={14} />}
            />
          </div>
        </div>

        {/* ── Right column — PTT panel ─────────────────────────── */}
        <div className="pt-1">
          <div className="rounded-(--radius-card) border border-(--color-border-subtle) bg-(--color-surface-raised) p-5">
            <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-(--color-text-muted) mb-4">
              Push-to-talk
            </p>

            <div className="flex items-center justify-center w-12 h-12 rounded-full bg-(--color-brand-500)/10 border border-(--color-brand-500)/20 mb-4">
              <DictationIcon size={20} className="text-(--color-brand-400)" />
            </div>

            <p className="text-[13px] font-semibold text-(--color-text-primary) mb-1.5 leading-tight">
              Ready to dictate
            </p>

            {status?.active_mode && (
              <p className="text-xs text-(--color-text-muted) mb-1">
                Mode:{" "}
                <span className="text-(--color-text-secondary)">
                  {capitalize(status.active_mode)}
                </span>
              </p>
            )}

            <p className="text-xs text-(--color-text-muted) leading-relaxed mb-5">
              Press your shortcut anywhere to speak. Release to insert.
            </p>

            <Link
              to="/settings"
              className="inline-flex items-center gap-1 text-xs text-(--color-brand-400) hover:text-(--color-brand-300) font-medium transition-colors duration-100"
            >
              Configure shortcut
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                <path d="M9 18l6-6-6-6" />
              </svg>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────

function StatusChip({
  value,
  state,
}: {
  value: string;
  state: "ok" | "error" | "warn" | "info" | "loading";
}) {
  const chipStyle = {
    ok:      "border-(--color-status-success)/25 bg-(--color-status-success)/8",
    error:   "border-(--color-status-error)/25 bg-(--color-status-error)/8",
    warn:    "border-(--color-status-warning)/25 bg-(--color-status-warning)/8",
    info:    "border-(--color-brand-400)/20 bg-(--color-brand-400)/8",
    loading: "border-(--color-border-default) bg-(--color-surface-raised)",
  }[state];

  const dotColor = {
    ok:      "bg-(--color-status-success)",
    error:   "bg-(--color-status-error)",
    warn:    "bg-(--color-status-warning)",
    info:    "bg-(--color-brand-400)",
    loading: "bg-(--color-text-muted) animate-pulse",
  }[state];

  return (
    <div
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-xs select-none ${chipStyle}`}
    >
      <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotColor}`} aria-hidden="true" />
      <span className="text-(--color-text-primary) font-medium">{value}</span>
    </div>
  );
}

function QuickLink({
  to,
  label,
  subtitle,
  badge,
  icon,
}: {
  to: string;
  label: string;
  subtitle: string;
  badge?: string;
  icon?: React.ReactElement;
}) {
  return (
    <Link
      to={to}
      className="group flex items-start gap-3 px-4 py-3.5 rounded-(--radius-card) bg-(--color-surface-raised) border border-(--color-border-subtle) hover:border-(--color-brand-500)/30 hover:bg-(--color-surface-overlay) transition-all duration-100"
    >
      {icon && (
        <span className="mt-0.5 text-(--color-text-muted) group-hover:text-(--color-brand-400) transition-colors duration-100 shrink-0">
          {icon}
        </span>
      )}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-(--color-text-secondary) group-hover:text-(--color-text-primary) transition-colors duration-100 truncate leading-none mb-1">
          {label}
        </p>
        <p className="text-xs text-(--color-text-muted) truncate leading-none mb-1.5">{subtitle}</p>
        {badge && (
          <p className="text-[11px] text-(--color-brand-400) font-medium truncate leading-none">{badge}</p>
        )}
      </div>
      <ChevronRightIcon />
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

function ChevronRightIcon() {
  return (
    <svg
      width="13"
      height="13"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-(--color-text-muted) group-hover:text-(--color-brand-400) transition-colors duration-100 shrink-0 mt-0.5"
      aria-hidden="true"
    >
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
