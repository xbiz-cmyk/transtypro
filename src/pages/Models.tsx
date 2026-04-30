import { useEffect, useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import EmptyState from "../components/ui/EmptyState";
import Input from "../components/ui/Input";
import ErrorMessage from "../components/ui/ErrorMessage";
import { getSettings, updateSettings } from "../lib/api";
import type { AppSettings } from "../lib/types";

// MOCK: installed models — replace with real backend data in Phase 4
const MOCK_MODELS = [
  {
    id: 1,
    name: "ggml-base.en.bin",
    path: "/path/to/models/ggml-base.en.bin",
    size_bytes: 147964211,
    language: "en",
    is_active: true,
  },
];

function formatBytes(bytes: number): string {
  const mb = bytes / 1024 / 1024;
  return `${mb.toFixed(1)} MB`;
}

export default function Models() {
  // TODO: wire to backend — when list_models command is available in Phase 4
  const models = MOCK_MODELS;

  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [binaryPath, setBinaryPath] = useState("");
  const [modelPath, setModelPath] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [settingsError, setSettingsError] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setSettings(s);
        setBinaryPath(s.whisper_binary_path ?? "");
        setModelPath(s.whisper_model_path ?? "");
      })
      .catch((err: unknown) => setSettingsError(String(err)));
  }, []);

  const handleSave = async () => {
    if (!settings) return;
    setIsSaving(true);
    setSettingsError(null);
    try {
      await updateSettings({
        ...settings,
        whisper_binary_path: binaryPath.trim() || null,
        whisper_model_path: modelPath.trim() || null,
      });
      setSettings((prev) =>
        prev
          ? {
              ...prev,
              whisper_binary_path: binaryPath.trim() || null,
              whisper_model_path: modelPath.trim() || null,
            }
          : prev
      );
    } catch (err: unknown) {
      setSettingsError(String(err));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div id="models-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Models
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Local whisper-compatible speech recognition models.
      </p>

      {/* Installed models */}
      <Card className="mb-5">
        <CardHeader>Installed models</CardHeader>

        {models.length === 0 ? (
          <EmptyState
            icon="🧠"
            heading="No models installed"
            subtext="Add a model file path below to configure local transcription."
          />
        ) : (
          <div className="space-y-3">
            {/* MOCK: rendered from MOCK_MODELS above */}
            {models.map((model) => (
              <div
                key={model.id}
                className="flex items-center justify-between gap-4 p-3 bg-(--color-surface-base) rounded-(--radius-btn) border border-(--color-border-subtle)"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-0.5">
                    <span className="text-sm font-medium text-(--color-text-primary)">
                      {model.name}
                    </span>
                    {model.is_active && (
                      <Badge variant="success">Active</Badge>
                    )}
                    <Badge variant="muted">{model.language}</Badge>
                  </div>
                  <p className="text-xs text-(--color-text-muted) truncate">
                    {model.path}
                  </p>
                  <p className="text-xs text-(--color-text-muted)">
                    {formatBytes(model.size_bytes)}
                  </p>
                </div>
                <Button
                  variant="danger"
                  size="sm"
                  disabled
                  title="Remove models — Phase 4"
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>

      {/* Whisper configuration */}
      <Card className="mb-5">
        <CardHeader>Whisper configuration</CardHeader>

        {settingsError && (
          <div className="mb-4">
            <ErrorMessage message={settingsError} />
          </div>
        )}

        <div className="space-y-4">
          <Input
            id="whisper-binary-path"
            label="Whisper binary path"
            placeholder="/usr/local/bin/whisper-cpp"
            helperText="Full path to the whisper.cpp-compatible binary."
            value={binaryPath}
            onChange={(e) => setBinaryPath(e.target.value)}
            disabled={settings === null || isSaving}
          />
          <Input
            id="whisper-model-path"
            label="Model file path"
            placeholder="/path/to/ggml-base.en.bin"
            helperText="Full path to the whisper.cpp-compatible model file."
            value={modelPath}
            onChange={(e) => setModelPath(e.target.value)}
            disabled={settings === null || isSaving}
          />
          <Button
            id="whisper-save-button"
            variant="primary"
            disabled={settings === null || isSaving}
            onClick={handleSave}
          >
            {isSaving ? "Saving…" : "Save"}
          </Button>
        </div>
      </Card>

      {/* Add model section */}
      <Card>
        <CardHeader>Add model</CardHeader>
        <div className="space-y-4">
          <Input
            id="model-path-input"
            label="Model file path"
            placeholder="/path/to/ggml-model.bin"
            helperText="Enter the full path to a whisper.cpp-compatible model file."
            disabled
          />
          <div className="flex items-center gap-3">
            <Button
              variant="secondary"
              disabled
              title="Browse for model file — Phase 4"
            >
              Browse...
            </Button>
            <Button
              variant="primary"
              disabled
              title="Add model — Phase 4"
            >
              Add model
            </Button>
          </div>

          {/* Model metadata placeholder */}
          <div
            id="model-metadata-placeholder"
            className="p-3 bg-(--color-surface-base) rounded-(--radius-btn) border border-(--color-border-subtle)"
          >
            <p className="text-xs text-(--color-text-muted)">
              Model metadata will appear here after you select a file.
            </p>
          </div>
        </div>
      </Card>
    </div>
  );
}
