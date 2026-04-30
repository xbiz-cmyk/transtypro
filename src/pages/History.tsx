import Card from "../components/ui/Card";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";
import Input from "../components/ui/Input";

// MOCK: history entries — replace with real backend data in Phase 2
const MOCK_ENTRIES = [
  {
    id: 1,
    created_at: 1745000000,
    mode: "Email",
    raw_text: "Hey can you send me the report by end of week",
    cleaned_text:
      "Hi, could you please send me the report by end of the week?",
    duration_secs: 4,
    cleanup_applied: true,
    provider_name: null,
  },
  {
    id: 2,
    created_at: 1745003600,
    mode: "Developer",
    raw_text: "git commit dash m feat add history page",
    cleaned_text: 'git commit -m "feat: add history page"',
    duration_secs: 3,
    cleanup_applied: true,
    provider_name: null,
  },
  {
    id: 3,
    created_at: 1745007200,
    mode: "Smart",
    raw_text: "schedule a meeting for tomorrow at 2pm",
    cleaned_text: "Schedule a meeting for tomorrow at 2 PM.",
    duration_secs: 2,
    cleanup_applied: false,
    provider_name: null,
  },
];

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

export default function History() {
  // TODO: wire to backend — useHistoryStore when list_history command is available
  const entries = MOCK_ENTRIES;

  return (
    <div id="history-page" className="p-8 max-w-4xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        History
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Your past dictation sessions.
      </p>

      {/* Filter bar */}
      <Card className="mb-5">
        <div className="flex items-center gap-3 flex-wrap">
          <Input
            id="history-search"
            placeholder="Search transcriptions..."
            className="flex-1 min-w-40"
          />

          {/* Date range placeholder */}
          <select
            id="history-date-filter"
            className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-secondary) focus:outline-none"
            defaultValue="all"
          >
            <option value="all">All time</option>
            <option value="today">Today</option>
            <option value="week">This week</option>
            <option value="month">This month</option>
          </select>

          {/* Mode filter placeholder */}
          <select
            id="history-mode-filter"
            className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-secondary) focus:outline-none"
            defaultValue="all"
          >
            <option value="all">All modes</option>
            <option value="smart">Smart</option>
            <option value="email">Email</option>
            <option value="developer">Developer</option>
          </select>
        </div>
      </Card>

      {/* Entry list */}
      {entries.length === 0 ? (
        <EmptyState
          icon="📋"
          heading="No history yet"
          subtext="Your dictation sessions will appear here after you record your first transcription."
        />
      ) : (
        <div className="space-y-3">
          {/* MOCK: rendered from MOCK_ENTRIES above */}
          {entries.map((entry) => (
            <Card key={entry.id}>
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <Badge variant="muted">{entry.mode}</Badge>
                    {entry.cleanup_applied && (
                      <Badge variant="success">Cleaned</Badge>
                    )}
                    <span className="text-xs text-(--color-text-muted)">
                      {formatDate(entry.created_at)}
                    </span>
                    <span className="text-xs text-(--color-text-muted)">
                      {entry.duration_secs}s
                    </span>
                  </div>
                  <p className="text-sm text-(--color-text-primary) truncate">
                    {entry.cleaned_text}
                  </p>
                  {entry.cleanup_applied && (
                    <p className="text-xs text-(--color-text-muted) mt-1 truncate">
                      Raw: {entry.raw_text}
                    </p>
                  )}
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <button
                    className="text-xs text-(--color-text-muted) hover:text-(--color-text-primary) transition-colors"
                    title="Copy to clipboard"
                  >
                    Copy
                  </button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
