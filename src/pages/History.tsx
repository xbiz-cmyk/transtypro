import { useEffect, useState } from "react";
import Card from "../components/ui/Card";
import Badge from "../components/ui/Badge";
import Button from "../components/ui/Button";
import EmptyState from "../components/ui/EmptyState";
import ErrorMessage from "../components/ui/ErrorMessage";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";
import { clearHistory, deleteHistoryEntry, listHistory } from "../lib/api";
import type { HistoryEntry } from "../lib/types";

function formatDate(isoString: string): string {
  return new Date(isoString).toLocaleString();
}

export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [deleteErrors, setDeleteErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    listHistory()
      .then((data) => setEntries(data))
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, []);

  const handleDelete = async (id: string) => {
    try {
      await deleteHistoryEntry(id);
      setEntries((prev) => prev.filter((e) => e.id !== id));
      setDeleteErrors((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
    } catch (err: unknown) {
      setDeleteErrors((prev) => ({ ...prev, [id]: String(err) }));
    }
  };

  const handleClearAll = async () => {
    try {
      await clearHistory();
      setEntries([]);
      setDeleteErrors({});
    } catch (err: unknown) {
      setError(String(err));
    }
  };

  const handleCopy = (entry: HistoryEntry) => {
    const text = entry.cleaned_text || entry.raw_text;
    navigator.clipboard.writeText(text).catch(() => {});
  };

  return (
    <div id="history-page" className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-1">
        <h1 className="text-2xl font-semibold text-(--color-text-primary)">
          History
        </h1>
        {entries.length > 0 && (
          <Button variant="danger" onClick={handleClearAll}>
            Clear all
          </Button>
        )}
      </div>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Your past dictation sessions.
      </p>

      {error && (
        <div className="mb-5">
          <ErrorMessage message={error} />
        </div>
      )}

      {/* Filter bar — cosmetic only in Phase 6 */}
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
      {isLoading ? (
        <p className="text-sm text-(--color-text-muted) italic">
          Loading history…
        </p>
      ) : entries.length === 0 ? (
        <EmptyState
          icon="📋"
          heading="No history yet"
          subtext="Your dictation sessions will appear here after you record your first transcription."
        />
      ) : (
        <div className="space-y-3">
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
                  {deleteErrors[entry.id] && (
                    <p className="text-xs text-(--color-status-error) mt-1">
                      {deleteErrors[entry.id]}
                    </p>
                  )}
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <button
                    className="text-xs text-(--color-text-muted) hover:text-(--color-text-primary) transition-colors"
                    title="Copy to clipboard"
                    onClick={() => handleCopy(entry)}
                  >
                    Copy
                  </button>
                  <button
                    className="text-xs text-(--color-status-error) hover:opacity-75 transition-opacity"
                    title="Delete entry"
                    onClick={() => handleDelete(entry.id)}
                  >
                    Delete
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
