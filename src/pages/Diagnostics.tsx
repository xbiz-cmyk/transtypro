import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";

// MOCK: diagnostic check items — replace with real backend data in Phase 8
// TODO: wire to backend when run_diagnostics_placeholder command is registered in api.ts
const MOCK_CHECK_ITEMS = [
  { name: "Backend IPC", status: "pass", message: "ping → pong" },
  { name: "Microphone access", status: "pending", message: "Phase 3" },
  { name: "Transcription model", status: "pending", message: "Phase 4" },
  { name: "Cleanup provider", status: "pending", message: "Phase 5" },
  { name: "SQLite database", status: "pending", message: "Phase 2" },
  { name: "Text insertion", status: "pending", message: "Phase 6" },
  { name: "Global shortcut", status: "pending", message: "Phase 7" },
];

function statusBadgeVariant(
  status: string,
): "success" | "warning" | "danger" | "muted" {
  if (status === "pass") return "success";
  if (status === "fail") return "danger";
  if (status === "warn") return "warning";
  return "muted";
}

function statusLabel(status: string): string {
  if (status === "pass") return "Pass";
  if (status === "fail") return "Fail";
  if (status === "warn") return "Warn";
  return "Pending";
}

export default function Diagnostics() {
  // TODO: wire to backend when run_diagnostics_placeholder command is available in Phase 8
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
            title="Run diagnostics — run_diagnostics_placeholder command — Phase 8"
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
              key={check.name}
              className="flex items-center justify-between py-2 border-b border-(--color-border-subtle) last:border-0"
            >
              <div className="flex items-center gap-3">
                <Badge variant={statusBadgeVariant(check.status)}>
                  {statusLabel(check.status)}
                </Badge>
                <span className="text-sm text-(--color-text-primary)">
                  {check.name}
                </span>
              </div>
              {check.message && (
                <span className="text-xs text-(--color-text-muted)">
                  {check.message}
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
