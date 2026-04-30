import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";

// MOCK: diagnostic check items — replace with real backend data in Phase 8
// TODO: wire to backend when run_diagnostics command is registered in api.ts
const MOCK_CHECK_ITEMS = [
  { label: "Backend IPC", status: "ok" as const, detail: "ping → pong" },
  {
    label: "Microphone access",
    status: "unknown" as const,
    detail: "Phase 3",
  },
  {
    label: "Transcription model",
    status: "unknown" as const,
    detail: "Phase 4",
  },
  {
    label: "Cleanup provider",
    status: "unknown" as const,
    detail: "Phase 5",
  },
  { label: "SQLite database", status: "unknown" as const, detail: "Phase 2" },
  {
    label: "Text insertion",
    status: "unknown" as const,
    detail: "Phase 6",
  },
  {
    label: "Global shortcut",
    status: "unknown" as const,
    detail: "Phase 7",
  },
];

const statusBadgeVariant = {
  ok: "success" as const,
  warn: "warning" as const,
  error: "danger" as const,
  unknown: "muted" as const,
};

const statusLabel = {
  ok: "OK",
  warn: "Warn",
  error: "Error",
  unknown: "N/A",
};

export default function Diagnostics() {
  // TODO: wire to backend when run_diagnostics command is available in Phase 8
  const checks = MOCK_CHECK_ITEMS;

  return (
    <div id="diagnostics-page" className="p-8 max-w-3xl">
      <div className="flex items-center justify-between mb-1">
        <h1 className="text-2xl font-semibold text-(--color-text-primary)">
          Diagnostics
        </h1>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            disabled
            title="Run diagnostics — Phase 8"
          >
            Run diagnostics
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled
            title="Export diagnostics — not yet implemented"
          >
            Export
          </Button>
        </div>
      </div>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        System health checks and diagnostic information.
      </p>

      {/* Status checks */}
      <Card className="mb-5">
        <CardHeader>Status checks</CardHeader>
        <div className="space-y-2">
          {/* MOCK: rendered from MOCK_CHECK_ITEMS above */}
          {checks.map((check) => (
            <div
              key={check.label}
              className="flex items-center justify-between py-2 border-b border-(--color-border-subtle) last:border-0"
            >
              <div className="flex items-center gap-3">
                <Badge variant={statusBadgeVariant[check.status]}>
                  {statusLabel[check.status]}
                </Badge>
                <span className="text-sm text-(--color-text-primary)">
                  {check.label}
                </span>
              </div>
              {check.detail && (
                <span className="text-xs text-(--color-text-muted)">
                  {check.detail}
                </span>
              )}
            </div>
          ))}
        </div>
      </Card>

      {/* Results area */}
      <Card>
        <CardHeader>Results</CardHeader>
        <EmptyState
          icon="🔧"
          heading="No diagnostics run yet"
          subtext="Click Run diagnostics to check your system configuration. This feature will be fully implemented in Phase 8."
        />
      </Card>
    </div>
  );
}
