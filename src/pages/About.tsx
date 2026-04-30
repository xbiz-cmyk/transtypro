import Card, { CardHeader } from "../components/ui/Card";

export default function About() {
  return (
    <div id="about-page" className="p-8 max-w-2xl">
      {/* App identity */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-(--color-text-primary) mb-1">
          transtypro
        </h1>
        <p className="text-lg text-(--color-brand-300) font-medium mb-3">
          Speak instead of type.
        </p>
        <p className="text-sm text-(--color-text-muted)">
          Version 0.1.0-dev
        </p>
      </div>

      {/* Description */}
      <Card className="mb-5">
        <CardHeader>What is transtypro?</CardHeader>
        <div className="space-y-3 text-sm text-(--color-text-secondary)">
          <p>
            transtypro is a local-first AI dictation desktop app for Windows and
            macOS. Press a shortcut, speak naturally, and polished text appears
            in your active application.
          </p>
          <p>
            It is designed to feel like a better keyboard — not like a
            transcription tool. Privacy is visible by default: you always know
            whether audio or text leaves your computer.
          </p>
          <p>
            transtypro works offline once local models are installed. Cloud
            providers are optional and can be blocked entirely with local-only
            mode.
          </p>
        </div>
      </Card>

      {/* Local paths */}
      <Card className="mb-5">
        <CardHeader>Local paths</CardHeader>
        <div className="space-y-2">
          <PathRow
            label="Config directory"
            value="~/.config/transtypro"
            note="Phase 2"
          />
          <PathRow
            label="Data directory"
            value="~/.local/share/transtypro"
            note="Phase 2"
          />
          <PathRow
            label="Log directory"
            value="~/.local/share/transtypro/logs"
            note="Phase 2"
          />
        </div>
        <p className="text-xs text-(--color-text-muted) mt-3">
          Real paths will be available in Phase 2 when settings persistence is
          implemented.
        </p>
      </Card>

      {/* Credits */}
      <Card>
        <CardHeader>Built with</CardHeader>
        <div className="space-y-1 text-sm text-(--color-text-secondary)">
          <CreditRow label="Runtime" value="Tauri v2" />
          <CreditRow label="Backend" value="Rust" />
          <CreditRow label="Frontend" value="React 19 + TypeScript" />
          <CreditRow label="Styling" value="Tailwind CSS v4" />
          <CreditRow
            label="Transcription"
            value="whisper.cpp (local, Phase 4)"
          />
          <CreditRow
            label="AI cleanup"
            value="Ollama / OpenAI-compatible (Phase 5)"
          />
        </div>
      </Card>
    </div>
  );
}

function PathRow({
  label,
  value,
  note,
}: {
  label: string;
  value: string;
  note?: string;
}) {
  return (
    <div className="flex items-baseline justify-between gap-4">
      <span className="text-sm text-(--color-text-secondary) shrink-0">
        {label}
      </span>
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-xs font-mono text-(--color-text-muted) truncate">
          {value}
        </span>
        {note && (
          <span className="text-xs text-(--color-text-muted) shrink-0 opacity-60">
            ({note})
          </span>
        )}
      </div>
    </div>
  );
}

function CreditRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-(--color-text-muted)">{label}</span>
      <span className="text-sm text-(--color-text-primary)">{value}</span>
    </div>
  );
}
