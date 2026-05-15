import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getStatusSummary, ping } from "../lib/api";
import type { StatusSummary } from "../lib/types";
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
    <div id="home-page" className="relative p-8 max-w-[900px] overflow-hidden">

      {/* Aurora background glow — soft radial behind the right column */}
      <div
        className="pointer-events-none absolute right-0 top-0 w-[480px] h-[420px]"
        style={{
          background:
            "radial-gradient(ellipse 75% 65% at 80% 25%, oklch(0.55 0.16 250 / 0.09) 0%, transparent 70%)",
        }}
        aria-hidden="true"
      />

      <div className="relative grid grid-cols-[1fr_228px] gap-8 items-start">

        {/* ── Left column ──────────────────────────────────────────── */}
        <div>

          {/* ── Hero ─────────────────────────────────────────────── */}
          <div className="mb-10">
            <h1 className="text-[2.5rem] font-extrabold leading-[1.08] tracking-tight text-(--color-text-primary) mb-4">
              Hold a key.<br />
              Speak.<br />
              Watch it appear.
            </h1>

            <p className="text-sm text-(--color-text-secondary) leading-relaxed mb-7 max-w-sm">
              Fast local dictation that types into any desktop app.
              Your audio stays on this device unless you choose otherwise.
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

          {/* ── Status chips ─────────────────────────────────────── */}
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

          {/* ── Quick links ──────────────────────────────────────── */}
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

        {/* ── Right column — mic orb ───────────────────────────────── */}
        <div className="pt-4">
          <div
            className="rounded-(--radius-card) border p-5 flex flex-col items-center"
            style={{
              background: "oklch(0.18 0.01 250 / 0.55)",
              borderColor: "oklch(0.28 0.03 250 / 0.5)",
              boxShadow: "0 8px 32px oklch(0 0 0 / 0.25)",
            }}
          >
            <MicOrb />

            <div className="mt-4 text-center w-full">
              <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-(--color-text-muted) mb-3">
                Push-to-talk
              </p>

              {status?.active_mode && (
                <p className="text-xs text-(--color-text-muted) mb-2">
                  Mode:{" "}
                  <span className="text-(--color-text-secondary)">
                    {capitalize(status.active_mode)}
                  </span>
                </p>
              )}

              <p className="text-[11px] text-(--color-text-muted) leading-relaxed mb-4 max-w-[160px] mx-auto">
                Press your shortcut anywhere to speak.
              </p>

              <Link
                to="/settings"
                className="inline-flex items-center gap-1 text-xs text-(--color-brand-400) hover:text-(--color-brand-300) font-medium transition-colors duration-100"
              >
                Configure
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <path d="M9 18l6-6-6-6" />
                </svg>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────

function MicOrb() {
  return (
    <div className="relative flex flex-col items-center">
      {/* Orb + rings */}
      <div className="relative flex items-center justify-center w-40 h-40">
        {/* Ambient glow */}
        <div
          className="absolute w-28 h-28 rounded-full"
          style={{
            background:
              "radial-gradient(circle, oklch(0.55 0.16 250 / 0.22) 0%, transparent 70%)",
            filter: "blur(14px)",
          }}
          aria-hidden="true"
        />
        {/* Rings */}
        <div className="absolute w-36 h-36 rounded-full border border-(--color-brand-500)/10" aria-hidden="true" />
        <div className="absolute w-28 h-28 rounded-full border border-(--color-brand-500)/18" aria-hidden="true" />
        <div className="absolute w-20 h-20 rounded-full border border-(--color-brand-400)/28" aria-hidden="true" />
        {/* Main orb */}
        <div
          className="relative w-14 h-14 rounded-full flex items-center justify-center shadow-xl"
          style={{
            background:
              "linear-gradient(135deg, oklch(0.65 0.14 250), oklch(0.42 0.18 265))",
          }}
        >
          <DictationIcon size={22} className="text-white" />
        </div>
      </div>

      {/* Decorative waveform bars */}
      <div className="flex items-end gap-[3px] mt-1" aria-hidden="true">
        {[10, 16, 22, 16, 10].map((h, i) => (
          <div
            key={i}
            className="w-[5px] rounded-full bg-(--color-brand-400)/35"
            style={{ height: h }}
          />
        ))}
      </div>
    </div>
  );
}

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
