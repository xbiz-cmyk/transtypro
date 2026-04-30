import { useEffect, useRef, useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import Textarea from "../components/ui/Textarea";
import Select from "../components/ui/Select";
import ErrorMessage from "../components/ui/ErrorMessage";
import {
  cancelRecording,
  getRecordingStatus,
  listMicrophones,
  startRecording,
  stopRecording,
} from "../lib/api";
import type { MicrophoneInfo, RecordingResult, RecordingStatus } from "../lib/types";

export default function Dictation() {
  const [micList, setMicList] = useState<MicrophoneInfo[]>([]);
  const [selectedMic, setSelectedMic] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [recordingStatus, setRecordingStatus] = useState<RecordingStatus | null>(null);
  const [lastResult, setLastResult] = useState<RecordingResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Load microphone list on mount
  useEffect(() => {
    listMicrophones()
      .then((mics) => {
        setMicList(mics);
        const defaultMic = mics.find((m) => m.is_default);
        setSelectedMic(defaultMic?.name ?? mics[0]?.name ?? null);
      })
      .catch((err: unknown) => {
        setError(String(err));
      });
  }, []);

  // Clear polling interval helper
  const clearStatusInterval = () => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  // Clean up on unmount — cancel any in-progress recording
  useEffect(() => {
    return () => {
      clearStatusInterval();
      if (isRecording) {
        cancelRecording().catch(() => {});
      }
    };
    // intentional: only run on unmount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleRecord = async () => {
    setError(null);
    setLastResult(null);
    setIsLoading(true);
    try {
      const status = await startRecording(selectedMic ?? undefined);
      setIsRecording(true);
      setRecordingStatus(status);
      // Poll for live RMS level every 200ms
      intervalRef.current = setInterval(async () => {
        try {
          const s = await getRecordingStatus();
          setRecordingStatus(s);
        } catch {
          // Polling errors are non-fatal — keep polling
        }
      }, 200);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleStop = async () => {
    clearStatusInterval();
    setIsLoading(true);
    try {
      const result = await stopRecording();
      setLastResult(result);
      setIsRecording(false);
      setRecordingStatus(null);
    } catch (err: unknown) {
      setError(String(err));
      setIsRecording(false);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = async () => {
    clearStatusInterval();
    try {
      await cancelRecording();
    } catch {
      // Ignore cancel errors — device may have already stopped
    }
    setIsRecording(false);
    setRecordingStatus(null);
    setLastResult(null);
    setError(null);
  };

  const levelPercent = Math.min((recordingStatus?.level_rms ?? 0) * 100, 100);
  const durationSec = lastResult
    ? (lastResult.duration_ms / 1000).toFixed(1)
    : null;

  return (
    <div id="dictation-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Dictation
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Press Record, speak naturally, and press Stop to save a temporary WAV
        file. Transcription is available in Phase 4.
      </p>

      {error && (
        <div className="mb-5">
          <ErrorMessage message={error} />
        </div>
      )}

      {/* Microphone selector */}
      <Card className="mb-5">
        <CardHeader>Microphone</CardHeader>
        {micList.length === 0 ? (
          <p className="text-sm text-(--color-text-muted)">
            Loading microphones…
          </p>
        ) : (
          <Select
            id="mic-selector"
            value={selectedMic ?? ""}
            onChange={(e) => setSelectedMic(e.target.value)}
            disabled={isRecording}
          >
            {micList.map((m) => (
              <option key={m.name} value={m.name}>
                {m.name}
                {m.is_default ? " (default)" : ""}
              </option>
            ))}
          </Select>
        )}
      </Card>

      {/* Recording controls */}
      <Card className="mb-5 flex flex-col items-center py-10">
        <CardHeader>Recording</CardHeader>

        {/* Record / Stop / Cancel buttons */}
        <div className="flex items-center gap-3 mb-6">
          {!isRecording ? (
            <Button
              id="record-button"
              variant="primary"
              disabled={isLoading || micList.length === 0}
              onClick={handleRecord}
            >
              🎙 Record
            </Button>
          ) : (
            <>
              <Button
                id="stop-button"
                variant="danger"
                disabled={isLoading}
                onClick={handleStop}
              >
                ⏹ Stop
              </Button>
              <Button
                id="cancel-button"
                variant="ghost"
                disabled={isLoading}
                onClick={handleCancel}
              >
                ✕ Cancel
              </Button>
            </>
          )}
        </div>

        {/* Recording status badge */}
        {isRecording && (
          <Badge variant="danger" className="mb-4">
            ● Recording
          </Badge>
        )}

        {/* Input level meter */}
        <div className="w-full px-2">
          <p className="text-xs text-(--color-text-muted) mb-1">Input level</p>
          <div
            role="progressbar"
            aria-valuenow={levelPercent}
            aria-valuemin={0}
            aria-valuemax={100}
            className="w-full h-4 bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn) overflow-hidden"
          >
            <div
              className="h-full bg-green-500 transition-all duration-100"
              style={{ width: `${levelPercent}%` }}
            />
          </div>
          {isRecording && recordingStatus && (
            <p className="text-xs text-(--color-text-muted) mt-1">
              {recordingStatus.sample_count.toLocaleString()} samples collected
            </p>
          )}
        </div>
      </Card>

      {/* Result area */}
      <Card className="mb-5">
        <CardHeader>Result</CardHeader>

        <Textarea
          id="result-textarea"
          readOnly
          placeholder="Transcribed text will appear here — transcription is Phase 4."
          value=""
          rows={4}
          className="w-full cursor-default"
        />

        {/* WAV file info after stop */}
        {lastResult && (
          <div className="mt-3 p-3 bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn)">
            <p className="text-xs font-medium text-(--color-text-secondary) mb-1">
              Temporary WAV saved
            </p>
            <p className="text-xs text-(--color-text-muted) break-all mb-1">
              {lastResult.file_path}
            </p>
            <p className="text-xs text-(--color-text-muted)">
              {durationSec}s · {lastResult.sample_rate} Hz · mono 16-bit PCM
            </p>
            <p className="text-xs text-(--color-text-muted) mt-1 italic">
              Transcription not yet available — coming in Phase 4.
            </p>
          </div>
        )}
      </Card>

      {/* Action buttons — disabled until transcription is wired (Phase 4+) */}
      <div className="flex items-center gap-3">
        <Button variant="secondary" disabled title="No text to copy yet">
          Copy
        </Button>
        <Button
          variant="secondary"
          disabled
          title="Requires text insertion setup — Phase 6"
        >
          Insert
        </Button>
        <Button variant="ghost" disabled title="No text to save yet">
          Save as note
        </Button>
      </div>
    </div>
  );
}
