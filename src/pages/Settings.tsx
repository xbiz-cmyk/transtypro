import { useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Input from "../components/ui/Input";

export default function Settings() {
  // TODO: wire to useSettingsStore when get_settings command is available in Phase 2
  const [theme, setTheme] = useState("dark");
  const [language, setLanguage] = useState("en");
  const [defaultMode, setDefaultMode] = useState("Smart");
  const [localOnly, setLocalOnly] = useState(true);
  const [retentionDays, setRetentionDays] = useState("30");
  const [audioHistory, setAudioHistory] = useState(false);
  const [clipboardRestore, setClipboardRestore] = useState(false);

  return (
    <div id="settings-page" className="p-8 max-w-2xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Settings
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Configure transtypro to match your workflow.
      </p>

      {/* General */}
      <Card className="mb-5">
        <CardHeader>General</CardHeader>
        <div className="space-y-4">
          <div className="flex flex-col gap-1">
            <label
              htmlFor="theme-selector"
              className="text-sm font-medium text-(--color-text-secondary)"
            >
              Theme
            </label>
            <select
              id="theme-selector"
              value={theme}
              onChange={(e) => setTheme(e.target.value)}
              className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="system">System</option>
            </select>
            <p className="text-xs text-(--color-text-muted)">
              Theme switching — Phase 2
            </p>
          </div>

          <div className="flex flex-col gap-1">
            <label
              htmlFor="language-selector"
              className="text-sm font-medium text-(--color-text-secondary)"
            >
              Language
            </label>
            <select
              id="language-selector"
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
            >
              <option value="en">English</option>
              <option value="fr">Français</option>
              <option value="de">Deutsch</option>
              <option value="es">Español</option>
            </select>
          </div>
        </div>
      </Card>

      {/* Dictation */}
      <Card className="mb-5">
        <CardHeader>Dictation</CardHeader>
        <div className="space-y-4">
          <div className="flex flex-col gap-1">
            <label
              htmlFor="default-mode-selector"
              className="text-sm font-medium text-(--color-text-secondary)"
            >
              Default mode
            </label>
            <select
              id="default-mode-selector"
              value={defaultMode}
              onChange={(e) => setDefaultMode(e.target.value)}
              className="bg-(--color-surface-base) border border-(--color-border-default) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-primary) focus:outline-none focus:ring-2 focus:ring-(--color-brand-500)/50"
            >
              <option value="Smart">Smart</option>
              <option value="Raw">Raw</option>
              <option value="Clean">Clean</option>
              <option value="Email">Email</option>
              <option value="Developer">Developer</option>
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-(--color-text-secondary)">
              Global shortcut
            </label>
            <div
              id="shortcut-display"
              className="bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-muted) font-mono"
            >
              CommandOrControl+Shift+Space
            </div>
            <p className="text-xs text-(--color-text-muted)">
              Shortcut configuration — Phase 7
            </p>
          </div>
        </div>
      </Card>

      {/* Privacy */}
      <Card className="mb-5">
        <CardHeader>Privacy</CardHeader>
        <div className="space-y-4">
          <ToggleRow
            id="local-only-toggle"
            label="Local-only mode"
            description="Block all cloud transcription and cleanup calls"
            checked={localOnly}
            onChange={setLocalOnly}
          />

          <div className="flex flex-col gap-1">
            <label
              htmlFor="retention-days"
              className="text-sm font-medium text-(--color-text-secondary)"
            >
              History retention (days)
            </label>
            <Input
              id="retention-days"
              type="number"
              min="0"
              max="365"
              value={retentionDays}
              onChange={(e) => setRetentionDays(e.target.value)}
              helperText="Set to 0 to keep history forever."
            />
          </div>

          <ToggleRow
            id="audio-history-toggle"
            label="Save audio recordings"
            description="Keep the raw audio file after transcription"
            checked={audioHistory}
            onChange={setAudioHistory}
          />

          <ToggleRow
            id="clipboard-restore-toggle"
            label="Restore clipboard after dictation"
            description="Restore original clipboard contents when text is inserted"
            checked={clipboardRestore}
            onChange={setClipboardRestore}
          />
        </div>
      </Card>

      {/* Storage */}
      <Card className="mb-5">
        <CardHeader>Storage</CardHeader>
        <div className="space-y-4">
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-(--color-text-secondary)">
              Database path
            </label>
            <div
              id="db-path-display"
              className="bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn) px-3 py-2 text-sm text-(--color-text-muted) font-mono truncate"
            >
              ~/.local/share/transtypro/transtypro.db
            </div>
            <p className="text-xs text-(--color-text-muted)">
              Real path available — Phase 2
            </p>
          </div>

          <Button
            variant="danger"
            size="sm"
            disabled
            title="Clear history — Phase 2"
          >
            Clear all history
          </Button>
        </div>
      </Card>

      {/* Save */}
      <div className="flex justify-end">
        <Button
          variant="primary"
          disabled
          title="Settings persistence — Phase 2"
        >
          Save settings
        </Button>
      </div>
    </div>
  );
}

function ToggleRow({
  id,
  label,
  description,
  checked,
  onChange,
}: {
  id: string;
  label: string;
  description: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <label
          htmlFor={id}
          className="text-sm font-medium text-(--color-text-primary) cursor-pointer"
        >
          {label}
        </label>
        <p className="text-xs text-(--color-text-muted)">{description}</p>
      </div>
      <button
        id={id}
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none ${
          checked ? "bg-(--color-brand-500)" : "bg-(--color-surface-overlay)"
        }`}
      >
        <span
          className={`pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow transform transition-transform duration-200 ${
            checked ? "translate-x-5" : "translate-x-0"
          }`}
        />
      </button>
    </div>
  );
}
