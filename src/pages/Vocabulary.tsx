import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";

// MOCK: vocabulary entries — replace with real backend data in Phase 2
const MOCK_VOCABULARY = [
  {
    id: 1,
    term: "git push origin main",
    replacement: "git push origin main",
    category: "Technical",
  },
  {
    id: 2,
    term: "PR",
    replacement: "pull request",
    category: "Technical",
  },
  {
    id: 3,
    term: "gonna",
    replacement: "going to",
    category: "Grammar",
  },
  {
    id: 4,
    term: "TTP",
    replacement: "transtypro",
    category: "Personal",
  },
];

export default function Vocabulary() {
  // TODO: wire to backend — when list_vocabulary command is available in Phase 2
  const entries = MOCK_VOCABULARY;

  return (
    <div id="vocabulary-page" className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-1">
        <h1 className="text-2xl font-semibold text-(--color-text-primary)">
          Vocabulary
        </h1>
        <Button
          variant="primary"
          size="sm"
          disabled
          title="Add vocabulary — Phase 2"
        >
          + Add entry
        </Button>
      </div>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Custom terms and replacements applied during transcription.
      </p>

      {entries.length === 0 ? (
        <EmptyState
          icon="📖"
          heading="No vocabulary entries"
          subtext="Add custom terms and their replacements to improve transcription accuracy."
        />
      ) : (
        <Card>
          <CardHeader>Entries</CardHeader>
          {/* Table header */}
          <div className="grid grid-cols-[1fr_1fr_auto_auto] gap-4 px-2 pb-2 border-b border-(--color-border-subtle) text-xs font-medium text-(--color-text-muted) uppercase tracking-wider">
            <span>Term</span>
            <span>Replacement</span>
            <span>Category</span>
            <span />
          </div>

          {/* MOCK: rendered from MOCK_VOCABULARY above */}
          <div className="divide-y divide-(--color-border-subtle)">
            {entries.map((entry) => (
              <div
                key={entry.id}
                className="grid grid-cols-[1fr_1fr_auto_auto] gap-4 px-2 py-3 items-center"
              >
                <span className="text-sm text-(--color-text-primary) font-mono">
                  {entry.term}
                </span>
                <span className="text-sm text-(--color-text-secondary)">
                  {entry.replacement}
                </span>
                <Badge variant="muted">{entry.category}</Badge>
                <Button
                  variant="danger"
                  size="sm"
                  disabled
                  title="Delete vocabulary — Phase 2"
                >
                  Delete
                </Button>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}
