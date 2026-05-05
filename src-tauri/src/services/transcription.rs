use std::path::Path;

use crate::errors::AppError;
use crate::models::TranscriptionResult;

pub struct TranscriptionService;

impl TranscriptionService {
    /// Invoke a whisper.cpp-compatible binary and return the raw transcript.
    ///
    /// Validates all paths and the WAV containment before spawning the process.
    /// Uses `std::process::Command` directly — no shell execution.
    /// On success, returns the trimmed stdout as `raw_text`.
    /// Returns `AppError::TranscriptionError` for any validation or process failure.
    pub fn transcribe(
        wav_path: &str,
        binary_path: &str,
        model_path: &str,
        audio_dir: &Path,
    ) -> Result<TranscriptionResult, AppError> {
        // 1. Binary path must be non-empty.
        if binary_path.is_empty() {
            return Err(AppError::TranscriptionError(
                "Whisper binary path is not configured. Set it in the Models page.".to_string(),
            ));
        }

        // 2. Binary file must exist.
        if !Path::new(binary_path).is_file() {
            return Err(AppError::TranscriptionError(format!(
                "Whisper binary not found at: {binary_path}"
            )));
        }

        // 3. Model path must be non-empty.
        if model_path.is_empty() {
            return Err(AppError::TranscriptionError(
                "Whisper model path is not configured. Set it in the Models page.".to_string(),
            ));
        }

        // 4. Model file must exist.
        if !Path::new(model_path).is_file() {
            return Err(AppError::TranscriptionError(format!(
                "Whisper model not found at: {model_path}"
            )));
        }

        // 5. WAV path must be non-empty.
        if wav_path.is_empty() {
            return Err(AppError::TranscriptionError(
                "WAV file path is empty.".to_string(),
            ));
        }

        // 6. WAV file must exist.
        if !Path::new(wav_path).is_file() {
            return Err(AppError::TranscriptionError(format!(
                "WAV file not found at: {wav_path}"
            )));
        }

        // 7. WAV must be inside the app audio directory (path traversal guard).
        let canonical_wav = Path::new(wav_path)
            .canonicalize()
            .map_err(|e| AppError::TranscriptionError(format!("Cannot resolve WAV path: {e}")))?;
        let canonical_audio_dir = audio_dir
            .canonicalize()
            .map_err(|e| AppError::TranscriptionError(format!("Cannot resolve audio dir: {e}")))?;
        if !canonical_wav.starts_with(&canonical_audio_dir) {
            return Err(AppError::TranscriptionError(
                "WAV file is not inside the app audio directory.".to_string(),
            ));
        }

        // 8. Invoke the binary — no shell, args passed individually.
        let started_at = std::time::Instant::now();
        let output = std::process::Command::new(binary_path)
            .arg("-m")
            .arg(model_path)
            .arg("-f")
            .arg(wav_path)
            .output()
            .map_err(|e| {
                AppError::TranscriptionError(format!("Failed to launch whisper binary: {e}"))
            })?;

        let duration_ms = started_at.elapsed().as_millis() as u64;

        // 9. Check exit code.
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let code = output.status.code().unwrap_or(-1);
            return Err(AppError::TranscriptionError(format!(
                "Whisper exited with code {code}: {stderr}"
            )));
        }

        // 10. Return transcript from stdout.
        let raw_text = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Per spec: if the binary succeeded but produced no stdout, surface a clear error.
        if raw_text.is_empty() {
            return Err(AppError::TranscriptionError(
                "Local transcription produced no stdout. \
                 Check whether your whisper binary prints text to stdout."
                    .to_string(),
            ));
        }

        Ok(TranscriptionResult {
            raw_text,
            duration_ms,
            model_path: model_path.to_string(),
        })
    }

    /// Delete the WAV file after transcription if audio history retention is disabled.
    ///
    /// Must be called only after **successful** transcription. If the transcription
    /// failed the WAV is kept so the user can retry. Deletion errors are silently
    /// ignored — a missing file does not surface as an error to the user.
    pub fn cleanup_wav_if_needed(file_path: &str, audio_history_enabled: bool) {
        if !audio_history_enabled {
            let _ = std::fs::remove_file(file_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Create a file at `std::env::temp_dir()/<name>` with the given content.
    fn make_temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content).unwrap();
        path
    }

    /// Create a directory at `std::env::temp_dir()/<name>`.
    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    // ── validation tests ──────────────────────────────────────────────────────

    #[test]
    fn test_transcribe_fails_empty_binary_path() {
        let result = TranscriptionService::transcribe(
            "/some/recording.wav",
            "",
            "/some/model.bin",
            Path::new("/tmp"),
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError, got: {err:?}"
        );
        assert!(
            err.to_string().to_lowercase().contains("binary"),
            "error should mention 'binary': {err}"
        );
    }

    #[test]
    fn test_transcribe_fails_nonexistent_binary() {
        let result = TranscriptionService::transcribe(
            "/some/recording.wav",
            "/nonexistent/path/whisper_phase4_bin",
            "/some/model.bin",
            Path::new("/tmp"),
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError, got: {err:?}"
        );
        assert!(
            err.to_string().to_lowercase().contains("not found")
                || err.to_string().to_lowercase().contains("binary"),
            "error should indicate binary not found: {err}"
        );
    }

    #[test]
    fn test_transcribe_fails_empty_model_path() {
        // Binary must exist to reach model validation.
        let binary = make_temp_file("ts_test_bin_emptymodel_p4", b"");
        let result = TranscriptionService::transcribe(
            "/some/recording.wav",
            binary.to_str().unwrap(),
            "",
            Path::new("/tmp"),
        );
        let _ = std::fs::remove_file(&binary);
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError"
        );
        assert!(
            err.to_string().to_lowercase().contains("model"),
            "error should mention 'model': {err}"
        );
    }

    #[test]
    fn test_transcribe_fails_nonexistent_model() {
        let binary = make_temp_file("ts_test_bin_nomodel_p4", b"");
        let result = TranscriptionService::transcribe(
            "/some/recording.wav",
            binary.to_str().unwrap(),
            "/nonexistent/model_phase4.bin",
            Path::new("/tmp"),
        );
        let _ = std::fs::remove_file(&binary);
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError"
        );
        assert!(
            err.to_string().to_lowercase().contains("not found")
                || err.to_string().to_lowercase().contains("model"),
            "error should indicate model not found: {err}"
        );
    }

    #[test]
    fn test_transcribe_fails_nonexistent_wav() {
        let binary = make_temp_file("ts_test_bin_nowav_p4", b"");
        let model = make_temp_file("ts_test_model_nowav_p4.bin", b"");
        let tmp = std::env::temp_dir();
        let result = TranscriptionService::transcribe(
            "/nonexistent/recording_phase4.wav",
            binary.to_str().unwrap(),
            model.to_str().unwrap(),
            tmp.as_path(),
        );
        let _ = std::fs::remove_file(&binary);
        let _ = std::fs::remove_file(&model);
        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError"
        );
        assert!(
            err.to_string().to_lowercase().contains("not found")
                || err.to_string().to_lowercase().contains("wav"),
            "error should indicate WAV not found: {err}"
        );
    }

    #[test]
    fn test_transcribe_fails_wav_outside_audio_dir() {
        // WAV lives in temp root; audio_dir is a subdirectory → WAV is outside audio_dir.
        let binary = make_temp_file("ts_test_bin_outside_p4", b"");
        let model = make_temp_file("ts_test_model_outside_p4.bin", b"");
        let wav = make_temp_file("ts_test_wav_outside_p4.wav", b"RIFF");
        let audio_dir = make_temp_dir("ts_test_audio_dir_p4");

        let result = TranscriptionService::transcribe(
            wav.to_str().unwrap(),
            binary.to_str().unwrap(),
            model.to_str().unwrap(),
            &audio_dir,
        );

        let _ = std::fs::remove_file(&binary);
        let _ = std::fs::remove_file(&model);
        let _ = std::fs::remove_file(&wav);
        let _ = std::fs::remove_dir(&audio_dir);

        let err = result.unwrap_err();
        assert!(
            matches!(err, AppError::TranscriptionError(_)),
            "expected TranscriptionError"
        );
        assert!(
            err.to_string().to_lowercase().contains("not inside")
                || err.to_string().to_lowercase().contains("audio dir"),
            "error should mention audio dir containment: {err}"
        );
    }

    // ── cleanup tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_cleanup_deletes_wav_when_history_disabled() {
        let wav = make_temp_file("ts_test_cleanup_delete_p4.wav", b"RIFF");
        assert!(wav.exists(), "temp WAV must exist before cleanup");
        TranscriptionService::cleanup_wav_if_needed(wav.to_str().unwrap(), false);
        assert!(
            !wav.exists(),
            "WAV should be deleted when audio_history_enabled=false"
        );
    }

    #[test]
    fn test_cleanup_keeps_wav_when_history_enabled() {
        let wav = make_temp_file("ts_test_cleanup_keep_p4.wav", b"RIFF");
        assert!(wav.exists(), "temp WAV must exist before cleanup");
        TranscriptionService::cleanup_wav_if_needed(wav.to_str().unwrap(), true);
        assert!(
            wav.exists(),
            "WAV should be kept when audio_history_enabled=true"
        );
        let _ = std::fs::remove_file(&wav);
    }
}
