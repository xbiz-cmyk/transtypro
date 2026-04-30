import { useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";
import EmptyState from "../components/ui/EmptyState";

// MOCK: provider list — replace with real backend data in Phase 5
// TODO: wire to backend — when list_providers command is available in Phase 5
const MOCK_PROVIDERS = [
  {
    id: "1",
    name: "Local Ollama",
    provider_type: "ollama",
    base_url: "http://localhost:11434",
    model: "llama3",
    enabled: true,
    use_for_cleanup: true,
    use_for_transcription: false,
    api_key_set: false,
  },
];

const PROVIDER_TYPE_LABELS: Record<string, string> = {
  ollama: "Ollama",
  "openai-compatible": "OpenAI-compatible",
  anthropic: "Anthropic",
};

export default function Providers() {
  const providers = MOCK_PROVIDERS;

  const [newProviderType, setNewProviderType] = useState("ollama");
  const [newBaseUrl, setNewBaseUrl] = useState("");
  const [newModelName, setNewModelName] = useState("");
  const [newApiKey, setNewApiKey] = useState("");

  return (
    <div id="providers-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Providers
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        AI providers for text cleanup and formatting.
      </p>

      {/* Provider list */}
      <Card className="mb-5">
        <CardHeader>Configured providers</CardHeader>

        {providers.length === 0 ? (
          <EmptyState
            icon="☁️"
            heading="No providers configured"
            subtext="Add a provider below to enable text cleanup."
          />
        ) : (
          <div className="space-y-3">
            {/* MOCK: rendered from MOCK_PROVIDERS above */}
            {providers.map((provider) => (
              <div
                key={provider.id}
                className="flex items-center justify-between gap-4 p-3 bg-(--color-surface-base) rounded-(--radius-btn) border border-(--color-border-subtle)"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-0.5">
                    <span className="text-sm font-medium text-(--color-text-primary)">
                      {provider.name}
                    </span>
                    {provider.enabled && (
                      <Badge variant="success">Active</Badge>
                    )}
                    <Badge variant="muted">
                      {PROVIDER_TYPE_LABELS[provider.provider_type]}
                    </Badge>
                  </div>
                  <p className="text-xs text-(--color-text-muted)">
                    {provider.base_url} · {provider.model}
                  </p>
                </div>
                <Button
                  variant="danger"
                  size="sm"
                  disabled
                  title="Delete providers — Phase 5"
                >
                  Delete
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Add provider form */}
      <Card>
        <CardHeader>Add provider</CardHeader>
        <div className="space-y-4">
          <Select
            id="provider-type"
            label="Provider type"
            value={newProviderType}
            onChange={(e) => setNewProviderType(e.target.value)}
          >
            <option value="ollama">Ollama (local)</option>
            <option value="openai-compatible">OpenAI-compatible</option>
            <option value="anthropic">Anthropic</option>
          </Select>

          <Input
            id="provider-base-url"
            label="Base URL"
            placeholder="http://localhost:11434"
            value={newBaseUrl}
            onChange={(e) => setNewBaseUrl(e.target.value)}
          />

          <Input
            id="provider-model-name"
            label="Model name"
            placeholder="llama3"
            value={newModelName}
            onChange={(e) => setNewModelName(e.target.value)}
          />

          {/* API key field — display only, masked, explicitly not saved */}
          <div className="flex flex-col gap-1">
            <label
              htmlFor="provider-api-key"
              className="text-sm font-medium text-(--color-text-secondary)"
            >
              API key
              <span className="ml-2 text-xs font-normal text-(--color-text-muted)">
                (display only — not yet saved)
              </span>
            </label>
            <input
              id="provider-api-key"
              type="password"
              placeholder="sk-..."
              value={newApiKey}
              onChange={(e) => setNewApiKey(e.target.value)}
              autoComplete="off"
              className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) placeholder:text-(--color-text-muted) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
            />
            <p className="text-xs text-(--color-status-warning)">
              API key storage is not yet implemented. Do not enter a real key.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <Button
              variant="primary"
              disabled
              title="Save provider — Phase 5"
            >
              Save provider
            </Button>
            <Button
              variant="ghost"
              disabled
              title="Test connection — test_provider_placeholder command"
            >
              Test connection
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
}
