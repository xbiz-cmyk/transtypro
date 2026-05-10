import { useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";
import { runDiagnostics } from "../lib/api";
import type { DiagnosticReport } from "../lib/types";

function statusBadgeVariant(
  status: string,
): "success" | "warning" | "danger" | "muted" {
  if (status === "pass") return "success";
  if (status === "fail") return "danger";
  if (status === "warn") return "warning";
  return "muted"; // "skip" and anything else
}

function statusLabel(status: string): string {
  if (status === "pass") return "Pass";
  if (status === "fail") return "Fail";
  if (status === "warn") return "Warn";
  if (status === "skip") return "Skip";
  return "Unknown";
}

function formatCheckName(name: string): string {
  return name
    .split("_")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export default function Diagnostics() {
  const [report, setReport] = useState<DiagnosticReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleRunDiagnostics() {
    setLoading(true);
    setError(null);
    try {
      const r = await runDiagnostics();
      setReport(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

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
            disabled={loading}
            onClick={handleRunDiagnostics}
          >
            {loading ? "Running…" : "Run diagnostics"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled
            title="Diagnostics export — not yet implemented"
          >
            Export
          </Button>
        </div>
      </div>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        System health checks and diagnostic information.
      </p>

      {/* Error state */}
      {error && (
        <Card className="mb-5">
          <p className="text-sm text-(--color-status-error)">
            Diagnostics failed: {error}
          </p>
        </Card>
      )}

      {/* Status checks — shown once a report is available */}
      {report && (
        <Card className="mb-5">
          <CardHeader>Status checks</CardHeader>
          <div className="space-y-2">
            {report.checks.map((check) => (
              <div
                key={check.name}
                className="flex items-center justify-between py-2 border-b border-(--color-border-subtle) last:border-0"
              >
                <div className="flex items-center gap-3">
                  <Badge variant={statusBadgeVariant(check.status)}>
                    {statusLabel(check.status)}
                  </Badge>
                  <span className="text-sm text-(--color-text-primary)">
                    {formatCheckName(check.name)}
                  </span>
                </div>
                {check.message && (
                  <span className="text-xs text-(--color-text-muted) max-w-xs text-right">
                    {check.message}
                  </span>
                )}
              </div>
            ))}
          </div>
          <p className="text-xs text-(--color-text-muted) mt-3">
            Generated at: {report.generated_at}
          </p>
        </Card>
      )}

      {/* Empty / results area */}
      {!report && !error && (
        <Card>
          <CardHeader>Results</CardHeader>
          <EmptyState
            icon="🔧"
            heading="No diagnostics run yet"
            subtext="Click Run diagnostics to check your system configuration."
          />
        </Card>
      )}
    </div>
  );
}
