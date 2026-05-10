import { useCallback, useEffect, useRef, useState } from "react";
import Card, { CardHeader } from "../components/ui/Card";
import Button from "../components/ui/Button";
import Badge from "../components/ui/Badge";
import Textarea from "../components/ui/Textarea";
import Select from "../components/ui/Select";
import ErrorMessage from "../components/ui/ErrorMessage";
import {
  cancelRecording,
  cleanupText,
  createHistoryEntry,
  getRecordingStatus,
  getSettings,
  insertText,
  listEnabledCleanupProviders,
  listMicrophones,
  markHistoryInserted,
  startRecording,
  stopRecording,
  transcribeAudio,
} from "../lib/api";
import type {
  AiProvider,
  CleanupResult,
  MicrophoneInfo,
  RecordingResult,
  RecordingStatus,
  TranscriptionResult,
} from "../lib/types";

export default function Dictation() {
  const [micList, setMicList] = useState<MicrophoneInfo[]>([]);
  const [selectedMic, setSelectedMic] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [recordingStatus, setRecordingStatus] = useState<RecordingStatus | null>(null);
  const [lastResult, setLastResult] = useState<RecordingResult | null>(null);
  const [transcriptResult, setTranscriptResult] = useState<TranscriptionResult | null>(null);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Cleanup provider state
  const [cleanupProviders, setCleanupProviders] = useState<AiProvider[]>([]);
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
  const [cleanupResult, setCleanupResult] = useState<CleanupResult | null>(null);
  const [isCleaning, setIsCleaning] = useState(false);
  const [cleanupError, setCleanupError] = useState<string | null>(null);

  // Save as note state
  const [activeMode, setActiveMode] = useState<string>("Smart Mode");
  const [isSavingNote, setIsSavingNote] = useState(false);
  const [noteSaved, setNoteSaved] = useState(false);
  // entryId is set after a successful createHistoryEntry; used to mark as inserted.
  const [entryId, setEntryId] = useState<string | null>(null);

  // Insertion state
  const [isInserting, setIsInserting] = useState(false);
  const [inserted, setInserted] = useState(false);

  const loadCleanupProviders = useCallback(() => {
    listEnabledCleanupProviders()
      .then((providers) => {
        setCleanupProviders(providers);
        if (providers.length > 0) {
          setSelectedProviderId((prev) => prev ?? providers[0].id);
        }
      })
      .catch(() => {
        // Non-fatal — cleanup providers are optional
      });
  }, []);

  // Load microphone list, cleanup providers, and active mode on mount
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

    loadCleanupProviders();

    getSettings()
      .then((s) => setActiveMode(s.active_mode))
      .catch(() => {
        // Non-fatal — fall back to default mode name
      });
  }, [loadCleanupProviders]);

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
    setTranscriptResult(null);
    setCleanupResult(null);
    setCleanupError(null);
    setNoteSaved(false);
    setEntryId(null);
    setInserted(false);
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
    setTranscriptResult(null);
    setCleanupResult(null);
    setCleanupError(null);
    setNoteSaved(false);
    setEntryId(null);
    setInserted(false);
    setError(null);
  };

  const handleTranscribe = async () => {
    if (!lastResult) return;
    setIsTranscribing(true);
    setError(null);
    setCleanupResult(null);
    setCleanupError(null);
    try {
      const result = await transcribeAudio(lastResult.file_path);
      setTranscriptResult(result);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setIsTranscribing(false);
    }
  };

  const handleCleanup = async () => {
    if (!transcriptResult?.raw_text || !selectedProviderId) return;
    setIsCleaning(true);
    setCleanupError(null);
    setCleanupResult(null);
    try {
      const result = await cleanupText(transcriptResult.raw_text, selectedProviderId);
      setCleanupResult(result);
    } catch (err: unknown) {
      setCleanupError(String(err));
    } finally {
      setIsCleaning(false);
    }
  };

  const handleCopy = () => {
    const text = cleanupResult?.cleaned_text ?? transcriptResult?.raw_text;
    if (text) {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  };

  const handleSaveNote = async () => {
    if (!transcriptResult) return;
    setIsSavingNote(true);
    setError(null);
    try {
      const entry = await createHistoryEntry({
        rawText: transcriptResult.raw_text,
        cleanedText: cleanupResult?.cleaned_text ?? transcriptResult.raw_text,
        modeUsed: activeMode,
      });
      setEntryId(entry.id);
      setNoteSaved(true);
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setIsSavingNote(false);
    }
  };

  const handleInsert = async () => {
    if (!finalText) return;
    setIsInserting(true);
    setError(null);
    try {
      const result = await insertText(finalText);
      if (!result.success) {
        // Paste simulation failed — text is still in clipboard, show fallback message.
        setError(result.message);
      } else {
        // Mark history entry as inserted if a note was saved this session.
        if (entryId) {
          await markHistoryInserted(entryId).catch(() => {
            // Non-fatal — history marking failure should not hide insert success.
          });
        }
        setInserted(true);
      }
    } catch (err: unknown) {
      setError(String(err));
    } finally {
      setIsInserting(false);
    }
  };

  const levelPercent = Math.min((recordingStatus?.level_rms ?? 0) * 100, 100);
  const durationSec = lastResult
    ? (lastResult.duration_ms / 1000).toFixed(1)
    : null;

  const finalText = cleanupResult?.cleaned_text ?? transcriptResult?.raw_text ?? "";
  const displayText = finalText;

  const saveNoteLabel = isSavingNote
    ? "Saving…"
    : noteSaved
      ? "Saved ✓"
      : "Save as note";

  const insertLabel = inserted
    ? "Inserted ✓"
    : isInserting
      ? "Inserting…"
      : "Insert";

  return (
    <div id="dictation-page" className="p-8 max-w-3xl">
      <h1 className="text-2xl font-semibold text-(--color-text-primary) mb-1">
        Dictation
      </h1>
      <p className="text-sm text-(--color-text-secondary) mb-8">
        Press Record, speak naturally, then Stop. Press Transcribe to run local
        speech recognition using your configured whisper binary.
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
          placeholder="Transcribed text will appear here after you press Transcribe."
          value={displayText}
          rows={4}
          className="w-full cursor-default"
        />

        {cleanupResult && (
          <p className="text-xs text-(--color-text-muted) mt-1">
            Cleaned by {cleanupResult.provider_name} in {cleanupResult.duration_ms}ms
          </p>
        )}

        {cleanupError && (
          <div className="mt-2">
            <ErrorMessage message={cleanupError} />
          </div>
        )}

        {/* WAV file info and Transcribe button */}
        {lastResult && (
          <div className="mt-3 p-3 bg-(--color-surface-base) border border-(--color-border-subtle) rounded-(--radius-btn)">
            <p className="text-xs font-medium text-(--color-text-secondary) mb-1">
              Temporary WAV saved
            </p>
            <p className="text-xs text-(--color-text-muted) break-all mb-1">
              {lastResult.file_path}
            </p>
            <p className="text-xs text-(--color-text-muted) mb-3">
              {durationSec}s · {lastResult.sample_rate} Hz · mono 16-bit PCM
            </p>
            {isTranscribing ? (
              <p className="text-sm text-(--color-text-muted) italic">
                Transcribing…
              </p>
            ) : transcriptResult ? (
              <p className="text-xs text-(--color-text-muted)">
                Transcribed in {transcriptResult.duration_ms}ms
              </p>
            ) : (
              <Button
                id="transcribe-button"
                variant="primary"
                onClick={handleTranscribe}
              >
                Transcribe
              </Button>
            )}
          </div>
        )}
      </Card>

      {/* Cleanup provider picker + Clean button */}
      {cleanupProviders.length > 0 && transcriptResult && (
        <Card className="mb-5">
          <CardHeader>Clean text</CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <Select
                id="cleanup-provider-selector"
                value={selectedProviderId ?? ""}
                onChange={(e) => setSelectedProviderId(e.target.value)}
                disabled={isCleaning}
              >
                {cleanupProviders.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </Select>
            </div>
            <Button
              id="clean-button"
              variant="secondary"
              onClick={handleCleanup}
              disabled={isCleaning || !selectedProviderId}
            >
              {isCleaning ? "Cleaning…" : "Clean text"}
            </Button>
          </div>
        </Card>
      )}

      {/* Action buttons */}
      <div className="flex items-center gap-3">
        <Button
          variant="secondary"
          disabled={!displayText}
          title={displayText ? "Copy text to clipboard" : "No text to copy yet"}
          onClick={handleCopy}
        >
          Copy
        </Button>
        <Button
          id="insert-button"
          variant="primary"
          disabled={!finalText || isInserting || inserted}
          title={
            !finalText
              ? "Transcribe first to enable insertion"
              : inserted
                ? "Already inserted this session"
                : "Insert text into the previously focused application"
          }
          onClick={handleInsert}
        >
          {insertLabel}
        </Button>
        <Button
          variant="ghost"
          disabled={!transcriptResult || isSavingNote || noteSaved}
          title={transcriptResult ? "Save this session to history" : "Transcribe first to save"}
          onClick={handleSaveNote}
        >
          {saveNoteLabel}
        </Button>
      </div>
    </div>
  );
}
