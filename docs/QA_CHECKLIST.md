# QA Checklist

## Development checks

- [ ] Frontend installs successfully.
- [ ] Frontend dev server starts.
- [ ] Tauri dev app starts.
- [ ] Rust compiles.
- [ ] TypeScript type check passes.
- [ ] Rust format passes.
- [ ] Rust tests pass.
- [ ] Frontend build passes.

## Manual app checks

- [ ] App opens.
- [ ] Sidebar navigation works.
- [ ] Onboarding opens.
- [ ] Home page shows current status.
- [ ] Settings persist after restart.
- [ ] Microphone list loads.
- [ ] Microphone recording creates WAV.
- [ ] Local model path can be selected.
- [ ] Missing model error is useful.
- [ ] Whisper transcription works with a sample WAV.
- [ ] Cleanup disabled mode works.
- [ ] Ollama provider test works when Ollama is running.
- [ ] OpenAI-compatible provider test handles invalid key safely.
- [ ] Local-only mode blocks cloud calls.
- [ ] Text inserts into TextEdit or Notepad.
- [ ] Text inserts into browser input.
- [ ] Clipboard restore works.
- [ ] History saves when enabled.
- [ ] History does not save in no-history mode.
- [ ] Diagnostics report excludes secrets.

## Privacy checks

- [ ] API keys are masked in UI.
- [ ] API keys are not logged.
- [ ] Diagnostics do not include secrets.
- [ ] Local-only mode blocks transcription network calls.
- [ ] Local-only mode blocks cleanup network calls.
- [ ] Temporary audio is deleted when audio history is disabled.
