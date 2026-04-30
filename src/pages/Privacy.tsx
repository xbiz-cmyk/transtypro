import Card, { CardHeader } from "../components/ui/Card";
import Badge from "../components/ui/Badge";

// MOCK: privacy summary — replace with real backend data in Phase 8
// TODO: wire to backend when get_privacy_status command is registered in api.ts
// Returns PrivacySummary (matches Rust PrivacyService::get_privacy_status)
const MOCK_PRIVACY_SUMMARY = {
  local_only_mode: true,
  audio_retention_days: 0,
  history_retention_days: 30,
  cloud_allowed: false,
  reason: "local-only mode enabled",
};

const DATA_FLOW_ITEMS = [
  {
    label: "Microphone audio",
    stays_local: true,
    note: "Processed on-device only",
  },
  {
    label: "Transcription",
    stays_local: true,
    note: "Local model via whisper.cpp",
  },
  {
    label: "Cleanup (AI formatting)",
    stays_local: null,
    note: "Depends on configured provider",
  },
  {
    label: "History & vocabulary",
    stays_local: true,
    note: "Stored in local SQLite database",
  },
  {
    label: "Settings",
    stays_local: true,
    note: "Stored in local config file",
  },
];

export default function Privacy() {
  // MOCK: using MOCK_PRIVACY_SUMMARY — replace with useSettingsStore / backend call in Phase 8
  const summary = MOCK_PRIVACY_SUMMARY;
  const isLocalOnly = summary.local_only_mode;

  return (
    <div id="privacy-page" className="p-8 max-w-2xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Privacy
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        transtypro is local-first. Here is exactly what happens to your data.
      </p>

      {/* Privacy status card */}
      <Card className="mb-5">
        <CardHeader>Current privacy status</CardHeader>
        <div className="flex items-center gap-3 mb-4">
          <span
            className={`w-3 h-3 rounded-full ${isLocalOnly ? "bg-(--color-status-success)" : "bg-(--color-status-warning)"}`}
          />
          <span className="text-base font-semibold text-(--color-text-primary)">
            {isLocalOnly ? "Local-only mode active" : "Cloud-enabled mode"}
          </span>
        </div>

        <div className="space-y-2">
          <StatusRow
            label="Cloud calls blocked"
            value={!summary.cloud_allowed ? "Yes" : "No"}
            ok={!summary.cloud_allowed}
          />
          <StatusRow
            label="Audio retention"
            value={
              summary.audio_retention_days === 0
                ? "Deleted after use"
                : `${summary.audio_retention_days} days`
            }
            ok={summary.audio_retention_days === 0}
          />
          <StatusRow
            label="History retention"
            value={
              summary.history_retention_days === 0
                ? "Forever"
                : `${summary.history_retention_days} days`
            }
            ok={true}
          />
          {summary.reason && (
            <p className="text-xs text-(--color-text-muted) pt-1">
              Reason: {summary.reason}
            </p>
          )}
        </div>
      </Card>

      {/* Privacy badges */}
      <Card className="mb-5">
        <CardHeader>Privacy guarantees</CardHeader>
        <div className="flex flex-wrap gap-3">
          <PrivacyBadge
            icon="🔒"
            label="Local Only"
            active={isLocalOnly}
          />
          <PrivacyBadge
            icon="🚫"
            label="No Cloud Calls"
            active={!summary.cloud_allowed}
          />
          <PrivacyBadge
            icon="🗑️"
            label="Audio Deleted After Use"
            active={summary.audio_retention_days === 0}
          />
        </div>
      </Card>

      {/* Data flow summary */}
      <Card className="mb-5">
        <CardHeader>Data flow summary</CardHeader>
        <div className="space-y-3">
          {DATA_FLOW_ITEMS.map((item) => (
            <div
              key={item.label}
              className="flex items-start justify-between gap-4"
            >
              <div>
                <p className="text-sm text-(--color-text-primary)">
                  {item.label}
                </p>
                <p className="text-xs text-(--color-text-muted)">{item.note}</p>
              </div>
              {item.stays_local === true ? (
                <Badge variant="success">Local</Badge>
              ) : item.stays_local === false ? (
                <Badge variant="warning">Cloud</Badge>
              ) : (
                <Badge variant="muted">Depends</Badge>
              )}
            </div>
          ))}
        </div>
      </Card>

      {/* Retention summary */}
      <Card>
        <CardHeader>Retention</CardHeader>
        <p className="text-sm text-(--color-text-secondary)">
          History entries are automatically deleted after{" "}
          <span className="font-medium text-(--color-text-primary)">
            {summary.history_retention_days === 0
              ? "never (kept forever)"
              : `${summary.history_retention_days} days`}
          </span>
          . Audio recordings are{" "}
          <span className="font-medium text-(--color-text-primary)">
            {summary.audio_retention_days === 0
              ? "deleted immediately after transcription"
              : `kept for ${summary.audio_retention_days} days`}
          </span>
          .
        </p>
        <p className="text-xs text-(--color-text-muted) mt-2">
          Retention enforcement — Phase 8
        </p>
      </Card>
    </div>
  );
}

function StatusRow({
  label,
  value,
  ok,
}: {
  label: string;
  value: string;
  ok: boolean;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-(--color-text-secondary)">{label}</span>
      <div className="flex items-center gap-2">
        <span
          className={`w-1.5 h-1.5 rounded-full ${ok ? "bg-(--color-status-success)" : "bg-(--color-status-warning)"}`}
        />
        <span className="text-sm text-(--color-text-primary) font-medium">
          {value}
        </span>
      </div>
    </div>
  );
}

function PrivacyBadge({
  icon,
  label,
  active,
}: {
  icon: string;
  label: string;
  active: boolean;
}) {
  return (
    <div
      className={`flex items-center gap-2 px-4 py-2.5 rounded-(--radius-card) border ${
        active
          ? "bg-(--color-status-success)/10 border-(--color-status-success)/30 text-(--color-status-success)"
          : "bg-(--color-surface-overlay) border-(--color-border-default) text-(--color-text-muted)"
      }`}
    >
      <span>{icon}</span>
      <span className="text-sm font-medium">{label}</span>
    </div>
  );
}
