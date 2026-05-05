import { useCallback, useEffect, useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";
import EmptyState from "../components/ui/EmptyState";
import ErrorMessage from "../components/ui/ErrorMessage";
import {
  createProvider,
  deleteProvider,
  listProviders,
  setProviderApiKey,
  testProviderConnection,
} from "../lib/api";
import type { AiProvider } from "../lib/types";

const PROVIDER_TYPE_LABELS: Record<string, string> = {
  ollama: "Ollama",
  openai_compatible: "OpenAI-compatible",
};

export default function Providers() {
  const [providers, setProviders] = useState<AiProvider[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Add-provider form state
  const [newName, setNewName] = useState("");
  const [newProviderType, setNewProviderType] = useState("ollama");
  const [newBaseUrl, setNewBaseUrl] = useState("http://localhost:11434");
  const [newModelName, setNewModelName] = useState("");
  const [newApiKey, setNewApiKey] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  // Per-provider action state
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});
  const [deletingId, setDeletingId] = useState<string | null>(null);

  // API key modal state
  const [apiKeyProviderId, setApiKeyProviderId] = useState<string | null>(null);
  const [apiKeyValue, setApiKeyValue] = useState("");
  const [isSavingKey, setIsSavingKey] = useState(false);
  const [keyError, setKeyError] = useState<string | null>(null);

  const loadProviders = useCallback(() => {
    setIsLoading(true);
    setError(null);
    listProviders()
      .then(setProviders)
      .catch((err: unknown) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, []);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

  const handleAdd = async () => {
    setFormError(null);
    if (!newName.trim()) { setFormError("Name is required."); return; }
    if (!newBaseUrl.trim()) { setFormError("Base URL is required."); return; }
    if (!newModelName.trim()) { setFormError("Model name is required."); return; }

    setIsSaving(true);
    try {
      const created = await createProvider({
        name: newName.trim(),
        providerType: newProviderType,
        baseUrl: newBaseUrl.trim(),
        model: newModelName.trim(),
        useForCleanup: true,
      });

      if (newProviderType === "openai_compatible" && newApiKey.trim()) {
        await setProviderApiKey(created.id, newApiKey.trim());
        // Reload so api_key_set badge reflects stored key
        loadProviders();
      } else {
        setProviders((prev) => [...prev, created]);
      }

      setNewName("");
      setNewBaseUrl("http://localhost:11434");
      setNewModelName("");
      setNewApiKey("");
    } catch (err: unknown) {
      setFormError(String(err));
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    setDeletingId(id);
    setError(null);
    try {
      await deleteProvider(id);
      setProviders((prev) => prev.filter((p) => p.id !== id));
      setTestResults((prev) => { const next = { ...prev }; delete next[id]; return next; });
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setDeletingId(null);
    }
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    setTestResults((prev) => ({ ...prev, [id]: "Testing…" }));
    try {
      const result = await testProviderConnection(id);
      setTestResults((prev) => ({ ...prev, [id]: result }));
    } catch (err: unknown) {
      setTestResults((prev) => ({ ...prev, [id]: `Error: ${String(err)}` }));
    } finally {
      setTestingId(null);
    }
  };

  const openApiKeyModal = (id: string) => {
    setApiKeyProviderId(id);
    setApiKeyValue("");
    setKeyError(null);
  };

  const handleSaveApiKey = async () => {
    if (!apiKeyProviderId) return;
    setKeyError(null);
    if (!apiKeyValue.trim()) { setKeyError("API key cannot be empty."); return; }
    setIsSavingKey(true);
    try {
      await setProviderApiKey(apiKeyProviderId, apiKeyValue.trim());
      setProviders((prev) =>
        prev.map((p) => p.id === apiKeyProviderId ? { ...p, api_key_set: true } : p)
      );
      setApiKeyProviderId(null);
      setApiKeyValue("");
    } catch (err: unknown) {
      setKeyError(String(err));
    } finally {
      setIsSavingKey(false);
    }
  };

  return (
    <div id="providers-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Providers
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        AI providers for text cleanup and formatting.
      </p>

      {error && (
        <div className="mb-5">
          <ErrorMessage message={error} />
        </div>
      )}

      {/* Provider list */}
      <Card className="mb-5">
        <CardHeader>Configured providers</CardHeader>

        {isLoading ? (
          <p className="text-sm text-(--color-text-muted)">Loading…</p>
        ) : providers.length === 0 ? (
          <EmptyState
            icon="☁️"
            heading="No providers configured"
            subtext="Add a provider below to enable text cleanup."
          />
        ) : (
          <div className="space-y-3">
            {providers.map((provider) => (
              <div
                key={provider.id}
                className="flex flex-col gap-2 p-3 bg-(--color-surface-base) rounded-(--radius-btn) border border-(--color-border-subtle)"
              >
                <div className="flex items-center justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-0.5 flex-wrap">
                      <span className="text-sm font-medium text-(--color-text-primary)">
                        {provider.name}
                      </span>
                      {provider.enabled && (
                        <Badge variant="success">Active</Badge>
                      )}
                      <Badge variant="muted">
                        {PROVIDER_TYPE_LABELS[provider.provider_type] ?? provider.provider_type}
                      </Badge>
                      {provider.api_key_set && (
                        <Badge variant="muted">API key set</Badge>
                      )}
                    </div>
                    <p className="text-xs text-(--color-text-muted)">
                      {provider.base_url} · {provider.model}
                    </p>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    {provider.provider_type === "openai_compatible" && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => openApiKeyModal(provider.id)}
                        disabled={testingId === provider.id || deletingId === provider.id}
                      >
                        {provider.api_key_set ? "Update key" : "Set key"}
                      </Button>
                    )}
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => handleTest(provider.id)}
                      disabled={testingId !== null || deletingId === provider.id}
                    >
                      {testingId === provider.id ? "Testing…" : "Test"}
                    </Button>
                    <Button
                      variant="danger"
                      size="sm"
                      onClick={() => handleDelete(provider.id)}
                      disabled={deletingId !== null || testingId === provider.id}
                    >
                      {deletingId === provider.id ? "Deleting…" : "Delete"}
                    </Button>
                  </div>
                </div>

                {testResults[provider.id] && (
                  <p className="text-xs text-(--color-text-muted) pl-1">
                    {testResults[provider.id]}
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Add provider form */}
      <Card>
        <CardHeader>Add provider</CardHeader>
        <div className="space-y-4">
          {formError && <ErrorMessage message={formError} />}

          <Input
            id="provider-name"
            label="Display name"
            placeholder="My Ollama"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
          />

          <Select
            id="provider-type"
            label="Provider type"
            value={newProviderType}
            onChange={(e) => {
              setNewProviderType(e.target.value);
              setNewBaseUrl(
                e.target.value === "ollama"
                  ? "http://localhost:11434"
                  : "https://api.openai.com/v1"
              );
            }}
          >
            <option value="ollama">Ollama (local)</option>
            <option value="openai_compatible">OpenAI-compatible</option>
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

          {newProviderType === "openai_compatible" && (
            <div className="flex flex-col gap-1">
              <label
                htmlFor="provider-api-key"
                className="text-sm font-medium text-(--color-text-secondary)"
              >
                API key
                <span className="ml-2 text-xs font-normal text-(--color-text-muted)">
                  (stored securely in OS keychain)
                </span>
              </label>
              <input
                id="provider-api-key"
                type="password"
                placeholder="sk-…"
                value={newApiKey}
                onChange={(e) => setNewApiKey(e.target.value)}
                autoComplete="off"
                className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) placeholder:text-(--color-text-muted) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
              />
            </div>
          )}

          <div className="flex items-center gap-3">
            <Button
              variant="primary"
              onClick={handleAdd}
              disabled={isSaving}
            >
              {isSaving ? "Saving…" : "Save provider"}
            </Button>
          </div>
        </div>
      </Card>

      {/* API key modal */}
      {apiKeyProviderId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-(--color-surface-raised) rounded-(--radius-card) p-6 w-full max-w-sm shadow-lg">
            <h2 className="text-lg font-semibold text-(--color-text-primary) mb-4">
              Set API key
            </h2>
            {keyError && <ErrorMessage message={keyError} />}
            <div className="mt-3 mb-4 flex flex-col gap-1">
              <label
                htmlFor="modal-api-key"
                className="text-sm font-medium text-(--color-text-secondary)"
              >
                API key
              </label>
              <input
                id="modal-api-key"
                type="password"
                placeholder="sk-…"
                value={apiKeyValue}
                onChange={(e) => setApiKeyValue(e.target.value)}
                autoComplete="off"
                className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) placeholder:text-(--color-text-muted) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
              />
              <p className="text-xs text-(--color-text-muted) mt-1">
                Stored in the OS keychain. Never logged or sent to cloud unless local-only mode is off.
              </p>
            </div>
            <div className="flex items-center gap-3">
              <Button
                variant="primary"
                onClick={handleSaveApiKey}
                disabled={isSavingKey}
              >
                {isSavingKey ? "Saving…" : "Save key"}
              </Button>
              <Button
                variant="ghost"
                onClick={() => setApiKeyProviderId(null)}
                disabled={isSavingKey}
              >
                Cancel
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
