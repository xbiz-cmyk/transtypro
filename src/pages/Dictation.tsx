import { useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import Textarea from "../components/ui/Textarea";
import Select from "../components/ui/Select";
import { useUiStore } from "../stores/uiStore";

// MOCK: dictation mode options — replace with real modes from backend in Phase 2
const MOCK_MODES = [
  "Smart",
  "Raw",
  "Clean",
  "Email",
  "Chat",
  "Notes",
  "Developer",
  "Terminal",
  "Translate",
  "Prompt",
];

export default function Dictation() {
  const { activeMode, setActiveMode } = useUiStore();
  const [resultText] = useState("");

  return (
    <div id="dictation-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Dictation
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Press the record button, speak naturally, and get polished text.
      </p>

      {/* Mode selector */}
      <Card className="mb-5">
        <CardHeader>Mode</CardHeader>
        <div className="flex items-center gap-3">
          <Select
            id="mode-selector"
            value={activeMode}
            onChange={(e) => setActiveMode(e.target.value)}
            className="flex-1"
          >
            {MOCK_MODES.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </Select>
          <Badge variant="muted">
            {/* MOCK: active mode badge */}
            Active: {activeMode}
          </Badge>
        </div>
      </Card>

      {/* Record button */}
      <Card className="mb-5 flex flex-col items-center py-10">
        <CardHeader>Recording</CardHeader>

        {/* Large record button — visual only, not functional */}
        <button
          id="record-button"
          disabled
          title="Recording will be available in Phase 3"
          className="w-24 h-24 rounded-full bg-(--color-surface-overlay) border-4 border-(--color-border-default) flex items-center justify-center cursor-not-allowed opacity-50 transition-colors"
        >
          <span className="text-4xl">🎙️</span>
        </button>

        <p className="text-xs text-(--color-text-muted) mt-4 text-center">
          Recording not yet available — Phase 3
        </p>

        {/* Waveform placeholder */}
        <div
          id="waveform-placeholder"
          className="w-full mt-6 h-16 bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn) flex items-center justify-center"
        >
          <span className="text-xs text-(--color-text-muted)">
            Waveform will appear here during recording
          </span>
        </div>
      </Card>

      {/* Result area */}
      <Card className="mb-5">
        <CardHeader>Result</CardHeader>
        <Textarea
          id="result-textarea"
          readOnly
          placeholder="Transcribed and cleaned text will appear here..."
          value={resultText}
          rows={4}
          className="w-full cursor-default"
        />
      </Card>

      {/* Action buttons */}
      <div className="flex items-center gap-3">
        <Button
          variant="secondary"
          disabled
          title="No text to copy yet"
        >
          Copy
        </Button>
        <Button
          variant="secondary"
          disabled
          title="Requires text insertion setup — Phase 6"
        >
          Insert
        </Button>
        <Button
          variant="ghost"
          disabled
          title="No text to save yet"
        >
          Save as note
        </Button>
      </div>
    </div>
  );
}
