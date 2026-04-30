import Card from "../components/ui/Card";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";

// MOCK: history entries — replace with real backend data in Phase 2
// TODO: wire to backend — useHistoryStore when list_history command is available
const MOCK_ENTRIES = [
  {
    id: "entry-001",
    raw_text: "Hey can you send me the report by end of week",
    cleaned_text:
      "Hi, could you please send me the report by end of the week?",
    mode_used: "Email",
    timestamp: "2026-04-29T10:00:00Z",
    was_inserted: false,
  },
  {
    id: "entry-002",
    raw_text: "git commit dash m feat add history page",
    cleaned_text: 'git commit -m "feat: add history page"',
    mode_used: "Developer",
    timestamp: "2026-04-29T11:00:00Z",
    was_inserted: true,
  },
  {
    id: "entry-003",
    raw_text: "schedule a meeting for tomorrow at 2pm",
    cleaned_text: "Schedule a meeting for tomorrow at 2 PM.",
    mode_used: "Smart",
    timestamp: "2026-04-29T12:00:00Z",
    was_inserted: false,
  },
];

function formatDate(isoString: string): string {
  return new Date(isoString).toLocaleString();
}

export default function History() {
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

          <Select id="history-date-filter" defaultValue="all">
            <option value="all">All time</option>
            <option value="today">Today</option>
            <option value="week">This week</option>
            <option value="month">This month</option>
          </Select>

          <Select id="history-mode-filter" defaultValue="all">
            <option value="all">All modes</option>
            <option value="smart">Smart</option>
            <option value="email">Email</option>
            <option value="developer">Developer</option>
          </Select>
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
                    <Badge variant="muted">{entry.mode_used}</Badge>
                    {entry.was_inserted && (
                      <Badge variant="success">Inserted</Badge>
                    )}
                    <span className="text-xs text-(--color-text-muted)">
                      {formatDate(entry.timestamp)}
                    </span>
                  </div>
                  <p className="text-sm text-(--color-text-primary) truncate">
                    {entry.cleaned_text}
                  </p>
                  {entry.raw_text !== entry.cleaned_text && (
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
