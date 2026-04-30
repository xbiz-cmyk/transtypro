/// transtypro — Audio recording service (Phase 3).
///
/// Architecture:
///   cpal::Stream is !Send on WASAPI (Windows). It must be created on and live
///   on a single OS thread. A dedicated audio thread owns the stream; it pushes
///   f32 samples into a shared Arc<Mutex<Vec<f32>>> buffer.  The Tauri command
///   thread signals the audio thread via an Arc<AtomicBool> stop flag and joins
///   it before writing the WAV.
///
/// WAV output is always mono 16-bit PCM. Multi-channel capture is mixed down by
/// averaging channels before writing.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::errors::AppError;
use crate::models::{MicrophoneInfo, RecordingResult, RecordingStatus};

// uuid is used by build_wav_path
use uuid::Uuid;

// ─────────────────────────────── State types ─────────────────────────────────

/// Metadata for an active recording session. Does NOT hold the cpal::Stream;
/// that lives on the audio thread.
pub struct RecordingHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub thread_handle: std::thread::JoinHandle<()>,
    pub selected_microphone: String,
    pub started_at: Instant,
}

/// Tauri-managed audio state, separate from AppState (SQLite).
pub struct AudioState {
    /// Directory where temporary WAV files are written.
    pub audio_dir: PathBuf,
    /// Active recording handle, or None when idle.
    pub recording: Arc<Mutex<Option<RecordingHandle>>>,
    /// Shared sample buffer between audio thread callback and Tauri commands.
    pub samples: Arc<Mutex<Vec<f32>>>,
    /// Native device sample rate, set when recording starts.
    pub sample_rate: Arc<Mutex<u32>>,
    /// Native device channel count, set when recording starts.
    pub channels: Arc<Mutex<u16>>,
}

// ─────────────────────── Pure helper functions (testable) ────────────────────
//
// These are free functions with no side effects. They are stubbed here so the
// test suite can be written and confirmed to FAIL before real implementations
// are added (TDD red phase). Replace the stub bodies with real logic in the
// green phase below.

/// Compute root-mean-square of a slice of f32 samples.
/// Returns 0.0 for an empty slice.
pub(crate) fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Convert a signed 16-bit integer sample to f32 in the range [-1.0, 1.0].
pub(crate) fn i16_to_f32(s: i16) -> f32 {
    s as f32 / i16::MAX as f32
}

/// Convert an unsigned 16-bit integer sample to f32 in the range [-1.0, 1.0].
pub(crate) fn u16_to_f32(s: u16) -> f32 {
    (s as f32 / u16::MAX as f32) * 2.0 - 1.0
}

/// Clamp f32 to [-1.0, 1.0] and convert to i16 (for WAV writing).
/// Maps +1.0 → i16::MAX, -1.0 → i16::MIN.
pub(crate) fn f32_to_i16(s: f32) -> i16 {
    let clamped = s.clamp(-1.0, 1.0);
    // Scale by 32768 then clamp to i16 range; avoids the asymmetry of using
    // only i16::MAX (32767) as the multiplier.
    let scaled = (clamped * 32768.0_f32) as i32;
    scaled.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// Mix a multi-channel interleaved sample buffer down to mono by averaging.
/// Returns a copy of `samples` unchanged if `channels == 1`.
pub(crate) fn mix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    let ch = channels as usize;
    if ch <= 1 {
        return samples.to_vec();
    }
    samples
        .chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

/// Compute recording duration in milliseconds from mono frame count and rate.
/// `sample_count` is the number of mono frames (not raw interleaved samples).
pub(crate) fn duration_ms(sample_count: usize, sample_rate: u32) -> u64 {
    if sample_rate == 0 {
        return 0;
    }
    (sample_count as u64 * 1000) / sample_rate as u64
}

/// Generate a unique WAV file path inside `dir` using a UUID filename.
pub(crate) fn build_wav_path(dir: &Path) -> PathBuf {
    dir.join(format!("{}.wav", Uuid::new_v4()))
}

// ─────────────────────────────── Service ─────────────────────────────────────

pub struct AudioService;

impl AudioService {
    /// Return all available input devices.
    pub fn list_microphones() -> Result<Vec<MicrophoneInfo>, AppError> {
        let host = cpal::default_host();
        let default_name = host.default_input_device().and_then(|d| d.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| AppError::AudioError(format!("failed to enumerate devices: {e}")))?;

        let mut result = Vec::new();
        for device in devices {
            let name = device
                .name()
                .unwrap_or_else(|_| "Unknown Device".to_string());
            let is_default = default_name.as_deref() == Some(name.as_str());
            result.push(MicrophoneInfo { name, is_default });
        }
        Ok(result)
    }

    /// Begin recording from the named device (or the system default if None).
    ///
    /// Spawns a dedicated audio thread that owns the cpal::Stream. Samples are
    /// pushed into `state.samples` via the audio callback. Returns immediately
    /// with RecordingStatus once the stream is confirmed running, or an error
    /// if setup fails.
    pub fn start_recording(
        device_name: Option<String>,
        state: &AudioState,
    ) -> Result<RecordingStatus, AppError> {
        // Guard: reject concurrent recordings
        {
            let guard = state
                .recording
                .lock()
                .map_err(|_| AppError::AudioError("state lock poisoned".to_string()))?;
            if guard.is_some() {
                return Err(AppError::AudioError(
                    "already recording — call cancel_recording first".to_string(),
                ));
            }
        }

        // Clear the sample buffer and reset metadata before starting
        {
            let mut buf = state
                .samples
                .lock()
                .map_err(|_| AppError::AudioError("samples lock poisoned".to_string()))?;
            buf.clear();
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        let samples_shared = state.samples.clone();
        let sample_rate_shared = state.sample_rate.clone();
        let channels_shared = state.channels.clone();
        let stop_for_thread = stop_flag.clone();
        let device_name_for_thread = device_name.clone();

        // Channel for the audio thread to report setup success or failure
        let (setup_tx, setup_rx) = std::sync::mpsc::channel::<Result<(), String>>();

        let thread_handle = std::thread::spawn(move || {
            audio_thread(
                stop_for_thread,
                samples_shared,
                sample_rate_shared,
                channels_shared,
                device_name_for_thread,
                setup_tx,
            );
        });

        // Wait up to 5 seconds for the audio thread to confirm the stream is running
        match setup_rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(Ok(())) => {}
            Ok(Err(msg)) => {
                // Thread reported a setup error — join it and surface the error
                let _ = thread_handle.join();
                return Err(AppError::AudioError(msg));
            }
            Err(_) => {
                // Timeout — signal stop and join
                stop_flag.store(true, Ordering::Relaxed);
                let _ = thread_handle.join();
                return Err(AppError::AudioError(
                    "audio device setup timed out".to_string(),
                ));
            }
        }

        let mic_name = device_name.unwrap_or_else(|| "default".to_string());

        let handle = RecordingHandle {
            stop_flag,
            thread_handle,
            selected_microphone: mic_name.clone(),
            started_at: Instant::now(),
        };

        {
            let mut guard = state
                .recording
                .lock()
                .map_err(|_| AppError::AudioError("state lock poisoned".to_string()))?;
            *guard = Some(handle);
        }

        Ok(RecordingStatus {
            is_recording: true,
            selected_microphone: Some(mic_name),
            level_rms: 0.0,
            sample_count: 0,
        })
    }

    /// Stop recording, write a temporary WAV file, and return its path.
    /// Returns an error if not currently recording.
    pub fn stop_recording(state: &AudioState) -> Result<RecordingResult, AppError> {
        let handle = {
            let mut guard = state
                .recording
                .lock()
                .map_err(|_| AppError::AudioError("state lock poisoned".to_string()))?;
            guard
                .take()
                .ok_or_else(|| AppError::AudioError("not currently recording".to_string()))?
        };

        // Signal and join the audio thread
        let thread_ref = handle.thread_handle.thread().clone();
        handle.stop_flag.store(true, Ordering::Relaxed);
        thread_ref.unpark();
        handle
            .thread_handle
            .join()
            .map_err(|_| AppError::AudioError("audio thread panicked".to_string()))?;

        // Collect raw interleaved samples
        let raw_samples = {
            let mut buf = state
                .samples
                .lock()
                .map_err(|_| AppError::AudioError("samples lock poisoned".to_string()))?;
            std::mem::take(&mut *buf)
        };

        if raw_samples.is_empty() {
            return Err(AppError::AudioError(
                "no audio was captured — check microphone permissions".to_string(),
            ));
        }

        let sample_rate = *state
            .sample_rate
            .lock()
            .map_err(|_| AppError::AudioError("sample_rate lock poisoned".to_string()))?;

        let native_channels = *state
            .channels
            .lock()
            .map_err(|_| AppError::AudioError("channels lock poisoned".to_string()))?;

        // Mix down to mono
        let mono_samples = mix_to_mono(&raw_samples, native_channels);
        let frame_count = mono_samples.len();
        let dur_ms = duration_ms(frame_count, sample_rate);

        // Write WAV
        std::fs::create_dir_all(&state.audio_dir)
            .map_err(|e| AppError::AudioError(format!("cannot create audio directory: {e}")))?;
        let wav_path = build_wav_path(&state.audio_dir);
        write_wav(&wav_path, &mono_samples, sample_rate)?;

        let file_path = wav_path.to_string_lossy().to_string();
        Ok(RecordingResult {
            file_path,
            duration_ms: dur_ms,
            sample_rate,
            channels: 1,
        })
    }

    /// Abort an active recording without writing a file.
    pub fn cancel_recording(state: &AudioState) -> Result<RecordingStatus, AppError> {
        let handle = {
            let mut guard = state
                .recording
                .lock()
                .map_err(|_| AppError::AudioError("state lock poisoned".to_string()))?;
            guard
                .take()
                .ok_or_else(|| AppError::AudioError("not currently recording".to_string()))?
        };

        let thread_ref = handle.thread_handle.thread().clone();
        handle.stop_flag.store(true, Ordering::Relaxed);
        thread_ref.unpark();
        handle
            .thread_handle
            .join()
            .map_err(|_| AppError::AudioError("audio thread panicked".to_string()))?;

        // Discard samples — no WAV written
        if let Ok(mut buf) = state.samples.lock() {
            buf.clear();
        }

        Ok(RecordingStatus {
            is_recording: false,
            selected_microphone: None,
            level_rms: 0.0,
            sample_count: 0,
        })
    }

    /// Return the current recording state including a live RMS level reading.
    pub fn get_recording_status(state: &AudioState) -> Result<RecordingStatus, AppError> {
        let guard = state
            .recording
            .lock()
            .map_err(|_| AppError::AudioError("state lock poisoned".to_string()))?;

        if guard.is_none() {
            return Ok(RecordingStatus {
                is_recording: false,
                selected_microphone: None,
                level_rms: 0.0,
                sample_count: 0,
            });
        }

        let mic_name = guard.as_ref().map(|h| h.selected_microphone.clone());
        drop(guard);

        let (sample_count, level) = {
            let buf = state
                .samples
                .lock()
                .map_err(|_| AppError::AudioError("samples lock poisoned".to_string()))?;
            let count = buf.len() as u64;
            let window = 2048.min(buf.len());
            let start = buf.len().saturating_sub(window);
            let level = rms(&buf[start..]);
            (count, level)
        };

        Ok(RecordingStatus {
            is_recording: true,
            selected_microphone: mic_name,
            level_rms: level,
            sample_count,
        })
    }
}

// ─────────────────────────── Audio thread ────────────────────────────────────

/// Owns the cpal::Stream for the duration of a recording session.
/// Reports setup success/failure through `setup_tx`, then parks in a loop
/// until `stop_flag` is set.
fn audio_thread(
    stop_flag: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate_out: Arc<Mutex<u32>>,
    channels_out: Arc<Mutex<u16>>,
    device_name: Option<String>,
    setup_tx: std::sync::mpsc::Sender<Result<(), String>>,
) {
    let host = cpal::default_host();

    // Resolve device
    let device = if let Some(ref name) = device_name {
        match host.input_devices() {
            Ok(mut devs) => devs.find(|d| d.name().map(|n| n == *name).unwrap_or(false)),
            Err(_) => None,
        }
        .or_else(|| host.default_input_device())
    } else {
        host.default_input_device()
    };

    let device = match device {
        Some(d) => d,
        None => {
            let _ = setup_tx.send(Err("no input device available".to_string()));
            return;
        }
    };

    let supported_config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            let _ = setup_tx.send(Err(format!("cannot get device config: {e}")));
            return;
        }
    };

    let sample_rate = supported_config.sample_rate().0;
    let native_channels = supported_config.channels();
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();

    // 5-minute hard cap expressed in total (interleaved) samples
    let max_samples = (5u64 * 60 * sample_rate as u64 * native_channels as u64) as usize;

    // Write device config into shared state before signalling ready
    if let Ok(mut sr) = sample_rate_out.lock() {
        *sr = sample_rate;
    }
    if let Ok(mut ch) = channels_out.lock() {
        *ch = native_channels;
    }

    // Build the stream for whichever sample format the device reports
    let err_fn = |_err: cpal::StreamError| {};

    let stream_result = match sample_format {
        cpal::SampleFormat::F32 => {
            let s_clone = samples.clone();
            let stop_clone = stop_flag.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if stop_clone.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Ok(mut buf) = s_clone.lock() {
                        let room = max_samples.saturating_sub(buf.len());
                        let n = data.len().min(room);
                        buf.extend_from_slice(&data[..n]);
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let s_clone = samples.clone();
            let stop_clone = stop_flag.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if stop_clone.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Ok(mut buf) = s_clone.lock() {
                        let room = max_samples.saturating_sub(buf.len());
                        let n = data.len().min(room);
                        buf.extend(data[..n].iter().map(|&s| i16_to_f32(s)));
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let s_clone = samples.clone();
            let stop_clone = stop_flag.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    if stop_clone.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Ok(mut buf) = s_clone.lock() {
                        let room = max_samples.saturating_sub(buf.len());
                        let n = data.len().min(room);
                        buf.extend(data[..n].iter().map(|&s| u16_to_f32(s)));
                    }
                },
                err_fn,
                None,
            )
        }
        fmt => {
            let _ = setup_tx.send(Err(format!("unsupported sample format: {fmt:?}")));
            return;
        }
    };

    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            let _ = setup_tx.send(Err(format!("failed to build audio stream: {e}")));
            return;
        }
    };

    if let Err(e) = stream.play() {
        let _ = setup_tx.send(Err(format!("failed to start audio stream: {e}")));
        return;
    }

    // Notify caller that setup succeeded
    let _ = setup_tx.send(Ok(()));

    // Keep the stream alive until stop is signalled
    while !stop_flag.load(Ordering::Relaxed) {
        std::thread::park_timeout(std::time::Duration::from_millis(10));
    }
    // `stream` drops here, stopping the recording
}

// ─────────────────────────── WAV writer ──────────────────────────────────────

fn write_wav(path: &Path, mono_samples: &[f32], sample_rate: u32) -> Result<(), AppError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| AppError::AudioError(format!("cannot create WAV file: {e}")))?;

    for &s in mono_samples {
        writer
            .write_sample(f32_to_i16(s))
            .map_err(|e| AppError::AudioError(format!("WAV write error: {e}")))?;
    }

    writer
        .finalize()
        .map_err(|e| AppError::AudioError(format!("WAV finalize error: {e}")))
}

// ──────────────────────────────── Tests ──────────────────────────────────────
//
// All tests are pure-function unit tests. No real microphone or filesystem is
// required. Each helper is tested against its stub (which returns 42/42.0) to
// establish the TDD red state, then retested after real implementation.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── rms ──────────────────────────────────────────────────────────────────

    #[test]
    fn rms_empty_slice_is_zero() {
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn rms_full_scale_is_one() {
        let samples = vec![1.0f32, -1.0, 1.0, -1.0];
        let result = rms(&samples);
        assert!((result - 1.0f32).abs() < 1e-6, "expected 1.0, got {result}");
    }

    #[test]
    fn rms_half_scale_is_half() {
        let samples = vec![0.5f32, -0.5, 0.5, -0.5];
        let result = rms(&samples);
        assert!((result - 0.5f32).abs() < 1e-6, "expected 0.5, got {result}");
    }

    // ── i16_to_f32 ───────────────────────────────────────────────────────────

    #[test]
    fn i16_to_f32_zero_maps_to_zero() {
        assert_eq!(i16_to_f32(0), 0.0f32);
    }

    #[test]
    fn i16_to_f32_max_maps_near_one() {
        let v = i16_to_f32(i16::MAX);
        assert!(
            (v - 1.0f32).abs() < 1e-4,
            "i16::MAX should map near 1.0, got {v}"
        );
    }

    #[test]
    fn i16_to_f32_min_maps_near_neg_one() {
        let v = i16_to_f32(i16::MIN);
        assert!(
            (v + 1.0f32).abs() < 1e-3,
            "i16::MIN should map near -1.0, got {v}"
        );
    }

    // ── u16_to_f32 ───────────────────────────────────────────────────────────

    #[test]
    fn u16_to_f32_max_maps_near_one() {
        let v = u16_to_f32(u16::MAX);
        assert!(
            (v - 1.0f32).abs() < 1e-4,
            "u16::MAX should map near 1.0, got {v}"
        );
    }

    #[test]
    fn u16_to_f32_midpoint_maps_near_zero() {
        // Midpoint of u16 is 32768 which maps to 0.0
        let v = u16_to_f32(32768u16);
        assert!(v.abs() < 1e-4, "u16 midpoint should map near 0.0, got {v}");
    }

    // ── f32_to_i16 ───────────────────────────────────────────────────────────

    #[test]
    fn f32_to_i16_zero_is_zero() {
        assert_eq!(f32_to_i16(0.0), 0i16);
    }

    #[test]
    fn f32_to_i16_one_maps_to_max() {
        assert_eq!(f32_to_i16(1.0), i16::MAX);
    }

    #[test]
    fn f32_to_i16_clamp_high() {
        assert_eq!(f32_to_i16(2.0), i16::MAX);
    }

    #[test]
    fn f32_to_i16_clamp_low() {
        assert_eq!(f32_to_i16(-2.0), i16::MIN);
    }

    // ── mix_to_mono ──────────────────────────────────────────────────────────

    #[test]
    fn mix_to_mono_already_mono_returns_unchanged() {
        let input = vec![0.1f32, 0.5, -0.3];
        let result = mix_to_mono(&input, 1);
        assert_eq!(result, input);
    }

    #[test]
    fn mix_to_mono_stereo_averages_channels() {
        // Two frames: [0.5, -0.5] → 0.0,  [1.0, 0.0] → 0.5
        let input = vec![0.5f32, -0.5, 1.0, 0.0];
        let result = mix_to_mono(&input, 2);
        assert_eq!(result.len(), 2, "stereo → mono should halve frame count");
        assert!(
            result[0].abs() < 1e-6,
            "frame 0 average should be 0.0, got {}",
            result[0]
        );
        assert!(
            (result[1] - 0.5f32).abs() < 1e-6,
            "frame 1 average should be 0.5, got {}",
            result[1]
        );
    }

    // ── duration_ms ──────────────────────────────────────────────────────────

    #[test]
    fn duration_ms_zero_samples_is_zero() {
        assert_eq!(duration_ms(0, 44100), 0);
    }

    #[test]
    fn duration_ms_one_second_at_44100() {
        assert_eq!(duration_ms(44100, 44100), 1000);
    }

    #[test]
    fn duration_ms_half_second_at_16000() {
        assert_eq!(duration_ms(8000, 16000), 500);
    }

    // ── build_wav_path ───────────────────────────────────────────────────────

    #[test]
    fn wav_path_ends_with_wav_extension() {
        let dir = PathBuf::from("/tmp");
        let path = build_wav_path(&dir);
        assert!(
            path.extension().and_then(|e| e.to_str()) == Some("wav"),
            "path should end with .wav, got {path:?}"
        );
    }

    #[test]
    fn wav_path_is_inside_given_dir() {
        let dir = PathBuf::from("/tmp/audio");
        let path = build_wav_path(&dir);
        assert!(
            path.starts_with(&dir),
            "path {path:?} should be inside {dir:?}"
        );
    }

    #[test]
    fn wav_paths_are_unique() {
        let dir = PathBuf::from("/tmp/audio");
        let p1 = build_wav_path(&dir);
        let p2 = build_wav_path(&dir);
        assert_ne!(p1, p2, "consecutive paths should be unique (UUID-based)");
    }
}
