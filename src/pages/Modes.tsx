import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";

// MOCK: built-in dictation modes — replace with real backend data in Phase 2
const MOCK_MODES = [
  {
    id: 1,
    name: "Smart",
    description:
      "Automatically detects context and applies the best formatting.",
    is_active: true,
    is_builtin: true,
    system_prompt: "",
  },
  {
    id: 2,
    name: "Raw",
    description: "No cleanup — returns the transcript exactly as spoken.",
    is_active: false,
    is_builtin: true,
    system_prompt: "",
  },
  {
    id: 3,
    name: "Clean",
    description: "Fixes grammar and punctuation without changing meaning.",
    is_active: false,
    is_builtin: true,
    system_prompt: "",
  },
  {
    id: 4,
    name: "Email",
    description: "Formats text as a professional email body.",
    is_active: false,
    is_builtin: true,
    system_prompt: "",
  },
  {
    id: 5,
    name: "Developer",
    description:
      "Preserves commands, flags, code terms, and technical vocabulary.",
    is_active: false,
    is_builtin: true,
    system_prompt: "",
  },
];

export default function Modes() {
  // TODO: wire to backend — useModesStore when list_modes command is available
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
                    {mode.is_active && (
                      <Badge variant="success">Active</Badge>
                    )}
                    {mode.is_builtin && (
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
                  {!mode.is_builtin && (
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
