import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";

// MOCK: built-in dictation modes — replace with real backend data in Phase 2
// TODO: wire to backend — useModesStore when list_modes command is available
const MOCK_MODES = [
  {
    id: "mode-001",
    name: "Smart",
    description:
      "Automatically detects context and applies the best formatting.",
    system_prompt: "",
    active: true,
    builtin: true,
  },
  {
    id: "mode-002",
    name: "Raw",
    description: "No cleanup — returns the transcript exactly as spoken.",
    system_prompt: "",
    active: false,
    builtin: true,
  },
  {
    id: "mode-003",
    name: "Clean",
    description: "Fixes grammar and punctuation without changing meaning.",
    system_prompt: "",
    active: false,
    builtin: true,
  },
  {
    id: "mode-004",
    name: "Email",
    description: "Formats text as a professional email body.",
    system_prompt: "",
    active: false,
    builtin: true,
  },
  {
    id: "mode-005",
    name: "Developer",
    description:
      "Preserves commands, flags, code terms, and technical vocabulary.",
    system_prompt: "",
    active: false,
    builtin: true,
  },
];

export default function Modes() {
  const modes = MOCK_MODES;

  return (
    <div id="modes-page" className="p-8 max-w-3xl">
      <div className="flex items-center justify-between mb-1">
        <h1 className="text-2xl font-semibold text-(--color-text-primary)">
          Modes
        </h1>
        {/* Add mode placeholder */}
        <Button
          variant="primary"
          size="sm"
          disabled
          title="Custom modes — Phase 2"
        >
          + Add mode
        </Button>
      </div>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Dictation modes control how your speech is transcribed and formatted.
      </p>

      {modes.length === 0 ? (
        <EmptyState
          icon="⚡"
          heading="No modes configured"
          subtext="Modes will appear here once the backend is connected."
        />
      ) : (
        <div className="space-y-3">
          {/* MOCK: rendered from MOCK_MODES above */}
          {modes.map((mode) => (
            <Card key={mode.id}>
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-medium text-(--color-text-primary)">
                      {mode.name}
                    </span>
                    {mode.active && (
                      <Badge variant="success">Active</Badge>
                    )}
                    {mode.builtin && (
                      <Badge variant="muted">Built-in</Badge>
                    )}
                  </div>
                  <p className="text-sm text-(--color-text-muted)">
                    {mode.description}
                  </p>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    disabled
                    title="Edit modes — Phase 2"
                  >
                    Edit
                  </Button>
                  {!mode.builtin && (
                    <Button
                      variant="danger"
                      size="sm"
                      disabled
                      title="Delete modes — Phase 2"
                    >
                      Delete
                    </Button>
                  )}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
