import { useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";
import Toggle from "../components/ui/Toggle";

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
          <Select
            id="theme-selector"
            label="Theme"
            value={theme}
            onChange={(e) => setTheme(e.target.value)}
            helperText="Theme switching — Phase 2"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </Select>

          <Select
            id="language-selector"
            label="Language"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
          >
            <option value="en">English</option>
            <option value="fr">Français</option>
            <option value="de">Deutsch</option>
            <option value="es">Español</option>
          </Select>
        </div>
      </Card>

      {/* Dictation */}
      <Card className="mb-5">
        <CardHeader>Dictation</CardHeader>
        <div className="space-y-4">
          <Select
            id="default-mode-selector"
            label="Default mode"
            value={defaultMode}
            onChange={(e) => setDefaultMode(e.target.value)}
          >
            <option value="Smart">Smart</option>
            <option value="Raw">Raw</option>
            <option value="Clean">Clean</option>
            <option value="Email">Email</option>
            <option value="Developer">Developer</option>
          </Select>

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
          <Toggle
            id="local-only-toggle"
            label="Local-only mode"
            description="Block all cloud transcription and cleanup calls"
            checked={localOnly}
            onChange={setLocalOnly}
          />

          <Input
            id="retention-days"
            label="History retention (days)"
            type="number"
            min="0"
            max="365"
            value={retentionDays}
            onChange={(e) => setRetentionDays(e.target.value)}
            helperText="Set to 0 to keep history forever."
          />

          <Toggle
            id="audio-history-toggle"
            label="Save audio recordings"
            description="Keep the raw audio file after transcription"
            checked={audioHistory}
            onChange={setAudioHistory}
          />

          <Toggle
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
