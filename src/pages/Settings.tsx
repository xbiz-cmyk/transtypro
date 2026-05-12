import { useState, useEffect } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Input from "../components/ui/Input";
import Select from "../components/ui/Select";
import Toggle from "../components/ui/Toggle";
import {
  getSettings,
  updateSettings,
  updateShortcut,
  clearHistory,
  applyRetentionPolicy,
} from "../lib/api";
import type { AppSettings, RetentionResult } from "../lib/types";

export default function Settings() {
  // Base settings loaded from backend — preserved on save for fields not shown here.
  const [loadedSettings, setLoadedSettings] = useState<AppSettings | null>(null);

  // Editable fields — hydrated from backend on mount.
  const [theme, setTheme] = useState("dark");
  const [language, setLanguage] = useState("en");
  const [defaultMode, setDefaultMode] = useState("smart");
  const [localOnly, setLocalOnly] = useState(false);
  const [retentionDays, setRetentionDays] = useState("30");
  const [audioHistory, setAudioHistory] = useState(false);
  const [clipboardRestore, setClipboardRestore] = useState(false);

  // Settings load / save state.
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Shortcut rebinding state.
  const [shortcut, setShortcut] = useState("CommandOrControl+Shift+Space");
  const [shortcutSaving, setShortcutSaving] = useState(false);
  const [shortcutMessage, setShortcutMessage] = useState<string | null>(null);
  const [shortcutError, setShortcutError] = useState<string | null>(null);

  // Shortcut recorder state machine: idle → recording → captured → idle
  const [recorderState, setRecorderState] = useState<"idle" | "recording" | "captured">("idle");
  const [capturedCombo, setCapturedCombo] = useState<string | null>(null);
  const [recorderWarning, setRecorderWarning] = useState<string | null>(null);

  // Shortcut behavior state.
  const [shortcutBehavior, setShortcutBehavior] = useState("open_dictation");

  // PTT output mode state.
  const [pttOutputMode, setPttOutputMode] = useState("clean_before_insert");

  // Retention cleanup state.
  const [cleanupRunning, setCleanupRunning] = useState(false);
  const [cleanupResult, setCleanupResult] = useState<RetentionResult | null>(null);
  const [cleanupError, setCleanupError] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setLoadedSettings(s);
        setTheme(s.theme);
        setDefaultMode(s.active_mode);
        setLocalOnly(s.local_only_mode);
        setRetentionDays(String(s.retention_days));
        setAudioHistory(s.audio_history_enabled);
        setClipboardRestore(s.clipboard_restore_enabled);
        setShortcut(s.shortcut);
        setShortcutBehavior(s.shortcut_behavior ?? "open_dictation");
        setPttOutputMode(s.ptt_output_mode ?? "clean_before_insert");
      })
      .catch(() => {
        // Non-fatal: form stays at defaults; user can still save.
      })
      .finally(() => setLoading(false));
  }, []);

  async function handleSave() {
    setSaving(true);
    setSaveMessage(null);
    setSaveError(null);
    try {
      // Merge locally-edited fields over the loaded base settings so that
      // whisper_binary_path, whisper_model_path, and other unseen fields are preserved.
      const base: AppSettings = loadedSettings ?? {
        active_mode: "smart",
        local_only_mode: false,
        theme: "dark",
        retention_days: 30,
        audio_history_enabled: false,
        clipboard_restore_enabled: false,
        whisper_binary_path: null,
        whisper_model_path: null,
        shortcut: "CommandOrControl+Shift+Space",
        shortcut_behavior: "open_dictation",
        ptt_output_mode: "clean_before_insert",
      };
      await updateSettings({
        ...base,
        active_mode: defaultMode,
        local_only_mode: localOnly,
        theme,
        retention_days: Math.max(0, parseInt(retentionDays, 10) || 0),
        audio_history_enabled: audioHistory,
        clipboard_restore_enabled: clipboardRestore,
        shortcut,
        shortcut_behavior: shortcutBehavior,
        ptt_output_mode: pttOutputMode,
      });
      setSaveMessage("Settings saved.");
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function handleClearHistory() {
    if (!window.confirm("Delete all history entries? This cannot be undone.")) {
      return;
    }
    try {
      await clearHistory();
      setSaveMessage("History cleared.");
    } catch (e) {
      setSaveError(String(e));
    }
  }

  async function handleRetentionCleanup() {
    setCleanupRunning(true);
    setCleanupResult(null);
    setCleanupError(null);
    try {
      const result = await applyRetentionPolicy();
      setCleanupResult(result);
    } catch (e) {
      setCleanupError(String(e));
    } finally {
      setCleanupRunning(false);
    }
  }

  // Keydown listener for shortcut recorder — attached only while recording.
  // Reads only e.key and modifier booleans. Never logs keys.
  useEffect(() => {
    if (recorderState !== "recording") return;

    function onKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      const key = e.key;
      if (["Control", "Alt", "Shift", "Meta"].includes(key)) return;

      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");
      const mapped =
        key === " " ? "Space"
        : key === "Enter" ? "Return"
        : key.length === 1 ? key.toUpperCase()
        : key;
      parts.push(mapped);

      const combo = parts.join("+");
      setCapturedCombo(combo);
      setRecorderState("captured");
      setRecorderWarning(
        parts.length === 1
          ? "Single-key shortcut — add a modifier to avoid conflicts with typing."
          : null
      );
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [recorderState]);

  async function handleApplyShortcut(combo?: string) {
    const target = combo ?? shortcut;
    setShortcutSaving(true);
    setShortcutMessage(null);
    setShortcutError(null);
    try {
      const accepted = await updateShortcut(target);
      setShortcut(accepted);
      setShortcutMessage("Shortcut applied.");
    } catch (e) {
      setShortcutError(String(e));
    } finally {
      setShortcutSaving(false);
    }
  }

  return (
    <div id="settings-page" className="p-8 max-w-2xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Settings
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Configure transtypro to match your workflow.
      </p>

      {loading && (
        <p className="text-sm text-(--color-text-muted) mb-4">Loading settings…</p>
      )}

      {/* General */}
      <Card className="mb-5">
        <CardHeader>General</CardHeader>
        <div className="space-y-4">
          <Select
            id="theme-selector"
            label="Theme"
            value={theme}
            onChange={(e) => setTheme(e.target.value)}
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
            <option value="smart">Smart</option>
            <option value="raw">Raw</option>
            <option value="clean">Clean</option>
            <option value="email">Email</option>
            <option value="developer">Developer</option>
          </Select>

          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-(--color-text-secondary)">
              Global shortcut
            </label>

            {/* Shortcut recorder */}
            <div className="flex flex-col gap-2">
              {recorderState === "idle" && (
                <div className="flex items-center gap-2">
                  <Button
                    id="record-shortcut-button"
                    variant="secondary"
                    size="sm"
                    disabled={shortcutSaving}
                    onClick={() => {
                      setRecorderState("recording");
                      setCapturedCombo(null);
                      setRecorderWarning(null);
                      setShortcutMessage(null);
                      setShortcutError(null);
                    }}
                  >
                    Record shortcut
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={shortcutSaving}
                    onClick={() => void handleApplyShortcut("CommandOrControl+Shift+Space")}
                  >
                    Reset to default
                  </Button>
                </div>
              )}

              {recorderState === "recording" && (
                <div className="flex items-center gap-3 px-3 py-2 rounded-(--radius-btn) border border-(--color-border-default) bg-(--color-surface-base)">
                  <span className="w-2 h-2 rounded-full bg-(--color-brand-400) animate-pulse shrink-0" />
                  <span className="text-sm text-(--color-text-muted) flex-1">Press your shortcut…</span>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => setRecorderState("idle")}
                  >
                    Cancel
                  </Button>
                </div>
              )}

              {recorderState === "captured" && capturedCombo && (
                <div className="flex flex-col gap-2 px-3 py-2 rounded-(--radius-btn) border border-(--color-border-default) bg-(--color-surface-base)">
                  <span className="font-mono text-sm text-(--color-text-primary)">{capturedCombo}</span>
                  {recorderWarning && (
                    <p className="text-xs text-(--color-status-warning)">{recorderWarning}</p>
                  )}
                  <div className="flex gap-2">
                    <Button
                      id="use-shortcut-button"
                      variant="primary"
                      size="sm"
                      disabled={shortcutSaving}
                      onClick={() => {
                        void handleApplyShortcut(capturedCombo);
                        setRecorderState("idle");
                      }}
                    >
                      {shortcutSaving ? "Applying…" : "Use this"}
                    </Button>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setRecorderState("idle")}
                    >
                      Cancel
                    </Button>
                  </div>
                </div>
              )}
            </div>

            {/* Advanced: manual text input */}
            <details className="mt-1">
              <summary className="text-xs text-(--color-text-muted) cursor-pointer select-none hover:text-(--color-text-secondary)">
                Advanced: enter shortcut manually
              </summary>
              <div className="flex items-center gap-2 mt-2">
                <Input
                  id="shortcut-input"
                  value={shortcut}
                  onChange={(e) => {
                    setShortcut(e.target.value);
                    setShortcutMessage(null);
                    setShortcutError(null);
                  }}
                  placeholder="e.g. CommandOrControl+Shift+Space"
                  className="flex-1 font-mono"
                  disabled={shortcutSaving}
                />
                <Button
                  id="apply-shortcut-button"
                  variant="secondary"
                  size="sm"
                  disabled={shortcutSaving || !shortcut.trim()}
                  onClick={() => void handleApplyShortcut()}
                >
                  {shortcutSaving ? "Applying…" : "Apply"}
                </Button>
              </div>
            </details>

            {shortcutMessage && (
              <p className="text-xs text-(--color-status-success)">{shortcutMessage}</p>
            )}
            {shortcutError && (
              <p className="text-xs text-(--color-status-error)">{shortcutError}</p>
            )}
            <p className="text-xs text-(--color-text-muted)">
              Note: Fn-only shortcuts are unsupported — they are handled at the
              hardware/firmware level before the OS sees them. Modifier-only shortcuts
              (Ctrl-only, Alt-only) are also unreliable.
            </p>
          </div>

          <Select
            id="shortcut-behavior-selector"
            label="Shortcut behavior"
            value={shortcutBehavior}
            onChange={(e) => setShortcutBehavior(e.target.value)}
          >
            <option value="open_dictation">Open Dictation page (default)</option>
            <option value="push_to_talk_toggle">
              Push-to-talk — press once to start, press again to stop and insert
            </option>
            <option value="push_to_talk_hold" disabled>
              Push-to-talk — hold to record, release to insert (not available on Windows)
            </option>
          </Select>
          <p className="text-xs text-(--color-text-muted)">
            Push-to-talk records in the background and inserts text into the
            active application without switching to transtypro.
          </p>

          <Select
            id="ptt-output-mode-selector"
            label="PTT output mode"
            value={pttOutputMode}
            onChange={(e) => setPttOutputMode(e.target.value)}
          >
            <option value="clean_before_insert">Best quality — clean before insert (slower)</option>
            <option value="insert_raw">Fast — insert raw transcript immediately</option>
          </Select>
          <p className="text-xs text-(--color-text-muted)">
            Fast mode skips AI cleanup and inserts the raw transcript immediately after transcription.
          </p>
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
              Stored in your OS app data directory
            </div>
          </div>

          <Button
            variant="danger"
            size="sm"
            onClick={handleClearHistory}
          >
            Clear all history
          </Button>

          {/* Retention cleanup */}
          <div className="border-t border-(--color-border-subtle) pt-4">
            <p className="text-sm font-medium text-(--color-text-secondary) mb-2">
              Storage cleanup
            </p>
            <p className="text-xs text-(--color-text-muted) mb-3">
              Delete history entries and audio files that exceed the configured
              retention period.
            </p>
            <Button
              variant="secondary"
              size="sm"
              disabled={cleanupRunning}
              onClick={handleRetentionCleanup}
            >
              {cleanupRunning ? "Running…" : "Run retention cleanup"}
            </Button>
            {cleanupResult && (
              <p className="text-xs text-(--color-status-success) mt-2">
                Deleted {cleanupResult.deleted_history_count} history{" "}
                {cleanupResult.deleted_history_count === 1 ? "entry" : "entries"} and{" "}
                {cleanupResult.deleted_wav_count} audio{" "}
                {cleanupResult.deleted_wav_count === 1 ? "file" : "files"}.
              </p>
            )}
            {cleanupError && (
              <p className="text-xs text-(--color-status-error) mt-2">
                Cleanup failed: {cleanupError}
              </p>
            )}
          </div>
        </div>
      </Card>

      {/* Feedback messages */}
      {saveMessage && (
        <p className="text-sm text-(--color-status-success) mb-4">{saveMessage}</p>
      )}
      {saveError && (
        <p className="text-sm text-(--color-status-error) mb-4">
          Save failed: {saveError}
        </p>
      )}

      {/* Save */}
      <div className="flex justify-end">
        <Button
          variant="primary"
          disabled={saving}
          onClick={handleSave}
        >
          {saving ? "Saving…" : "Save settings"}
        </Button>
      </div>
    </div>
  );
}
