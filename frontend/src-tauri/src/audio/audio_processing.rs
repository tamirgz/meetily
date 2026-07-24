use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};
use nnnoiseless::DenoiseState;
use realfft::num_complex::{Complex32, ComplexFloat};
use realfft::RealFftPlanner;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::path::PathBuf;

use super::encode::encode_single_audio; // Correct path to encode module

/// Sanitize a filename to be safe for filesystem use
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Create a meeting folder with timestamp and return the path
/// Creates structure: base_path/MeetingName_YYYY-MM-DD_HH-MM/
///                    ├── .checkpoints/  (for incremental saves, optional)
///
/// # Arguments
/// * `base_path` - Base directory for meetings
/// * `meeting_name` - Name of the meeting
/// * `create_checkpoints_dir` - Whether to create .checkpoints/ subdirectory (only needed when auto_save is true)
pub fn create_meeting_folder(
    base_path: &PathBuf,
    meeting_name: &str,
    create_checkpoints_dir: bool,
) -> Result<PathBuf> {
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let sanitized_name = sanitize_filename(meeting_name);
    let folder_name = format!("{}_{}", sanitized_name, timestamp);
    let meeting_folder = base_path.join(folder_name);

    // Create main meeting folder
    std::fs::create_dir_all(&meeting_folder)?;

    // Only create .checkpoints subdirectory if requested (when auto_save is true)
    if create_checkpoints_dir {
        let checkpoints_dir = meeting_folder.join(".checkpoints");
        std::fs::create_dir_all(&checkpoints_dir)?;
        log::info!(
            "Created meeting folder with checkpoints: {}",
            meeting_folder.display()
        );
    } else {
        log::info!(
            "Created meeting folder without checkpoints: {}",
            meeting_folder.display()
        );
    }

    Ok(meeting_folder)
}

pub fn normalize_v2(audio: &[f32]) -> Vec<f32> {
    let rms = (audio.iter().map(|&x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
    let peak = audio
        .iter()
        .fold(0.0f32, |max, &sample| max.max(sample.abs()));

    // Return the original audio if it's completely silent
    if rms == 0.0 || peak == 0.0 {
        return audio.to_vec();
    }

    // Increase target RMS for better voice volume while keeping peak in check
    let target_rms = 0.9; // Increased from 0.6
    let target_peak = 0.95; // Slightly reduced to prevent clipping

    let rms_scaling = target_rms / rms;
    let peak_scaling = target_peak / peak;

    // Apply a minimum scaling factor to boost very quiet audio
    let min_scaling = 1.5; // Minimum boost for quiet audio
    let scaling_factor = (rms_scaling.min(peak_scaling)).max(min_scaling);

    // Apply scaling with soft clipping to prevent harsh distortion
    audio
        .iter()
        .map(|&sample| {
            let scaled = sample * scaling_factor;
            // Soft clip at ±0.95 to prevent harsh distortion
            if scaled > 0.95 {
                0.95 + (scaled - 0.95) * 0.05
            } else if scaled < -0.95 {
                -0.95 + (scaled + 0.95) * 0.05
            } else {
                scaled
            }
        })
        .collect()
}

/// True peak limiter with lookahead buffer (prevents clipping)
struct TruePeakLimiter {
    lookahead_samples: usize,
    buffer: Vec<f32>,
    gain_reduction: Vec<f32>,
    current_position: usize,
}

impl TruePeakLimiter {
    fn new(sample_rate: u32) -> Self {
        const LIMITER_LOOKAHEAD_MS: usize = 10;
        let lookahead_samples = ((sample_rate as usize * LIMITER_LOOKAHEAD_MS) / 1000).max(1);

        Self {
            lookahead_samples,
            buffer: vec![0.0; lookahead_samples],
            gain_reduction: vec![1.0; lookahead_samples],
            current_position: 0,
        }
    }

    fn process(&mut self, sample: f32, true_peak_limit: f32) -> f32 {
        self.buffer[self.current_position] = sample;

        let sample_abs = sample.abs();
        if sample_abs > true_peak_limit {
            let reduction = true_peak_limit / sample_abs;
            self.gain_reduction[self.current_position] = reduction;
        } else {
            self.gain_reduction[self.current_position] = 1.0;
        }

        let output_position = (self.current_position + 1) % self.lookahead_samples;
        let output_sample = self.buffer[output_position] * self.gain_reduction[output_position];

        self.current_position = output_position;
        output_sample
    }
}

/// Professional loudness normalizer using EBU R128 standard
/// This is a STATEFUL normalizer that tracks cumulative loudness over time
///
/// EBU R128 is the broadcast industry standard for loudness normalization:
/// - Target: -23 LUFS (Loudness Units relative to Full Scale)
/// - Used by: Netflix, YouTube, Spotify, all professional broadcast
/// - Perceptually accurate (not just simple RMS)
///
pub struct LoudnessNormalizer {
    ebur128: ebur128::EbuR128,
    limiter: TruePeakLimiter,
    gain_linear: f32,
    loudness_buffer: Vec<f32>,
    true_peak_limit: f32,
}

impl LoudnessNormalizer {
    /// Create a new EBU R128 loudness normalizer
    ///
    /// # Arguments
    /// * `channels` - Number of audio channels (1 for mono, 2 for stereo)
    /// * `sample_rate` - Sample rate in Hz (e.g., 48000)
    pub fn new(channels: u32, sample_rate: u32) -> Result<Self> {
        const TRUE_PEAK_LIMIT: f64 = -1.0;
        const ANALYZE_CHUNK_SIZE: usize = 512;

        let ebur128 = ebur128::EbuR128::new(
            channels,
            sample_rate,
            ebur128::Mode::I | ebur128::Mode::TRUE_PEAK,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create EBU R128 normalizer: {}", e))?;

        let true_peak_limit = 10_f32.powf(TRUE_PEAK_LIMIT as f32 / 20.0);

        Ok(Self {
            ebur128,
            limiter: TruePeakLimiter::new(sample_rate),
            gain_linear: 1.0,
            loudness_buffer: Vec::with_capacity(ANALYZE_CHUNK_SIZE),
            true_peak_limit,
        })
    }

    /// Normalize loudness using EBU R128 standard with true peak limiting
    ///
    /// This maintains cumulative loudness measurements across all processed audio,
    /// resulting in consistent normalization that sounds natural.
    ///
    /// Target: -23 LUFS (professional broadcast standard for speech/dialog)
    /// Applies sample-by-sample with 10ms lookahead limiter to prevent clipping
    pub fn normalize_loudness(&mut self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }

        const TARGET_LUFS: f64 = -23.0;
        const ANALYZE_CHUNK_SIZE: usize = 512;

        let mut normalized_samples = Vec::with_capacity(samples.len());

        for &sample in samples {
            // Accumulate samples for loudness analysis
            self.loudness_buffer.push(sample);

            // Analyze loudness every 512 samples
            if self.loudness_buffer.len() >= ANALYZE_CHUNK_SIZE {
                if let Err(e) = self.ebur128.add_frames_f32(&self.loudness_buffer) {
                    warn!("Failed to add frames to EBU R128: {}", e);
                } else {
                    // Update gain based on cumulative loudness
                    if let Ok(current_lufs) = self.ebur128.loudness_global() {
                        if current_lufs.is_finite() && current_lufs < 0.0 {
                            let gain_db = TARGET_LUFS - current_lufs;
                            self.gain_linear = 10_f32.powf(gain_db as f32 / 20.0);
                        }
                    }
                }
                self.loudness_buffer.clear();
            }

            // Apply gain and true peak limiting
            let amplified = sample * self.gain_linear;
            let limited = self.limiter.process(amplified, self.true_peak_limit);

            normalized_samples.push(limited);
        }

        normalized_samples
    }
}

enum NoiseSuppressionBackend {
    #[cfg(target_os = "macos")]
    DeepFilterNet {
        input: std::sync::mpsc::SyncSender<Vec<f32>>,
        output: std::sync::mpsc::Receiver<Vec<f32>>,
        frame_buffer: Vec<f32>,
        frame_size: usize,
    },
    RnNoise {
        denoiser: DenoiseState<'static>,
        frame_buffer: Vec<f32>,
        frame_size: usize,
    },
}

/// Stateful neural microphone enhancement.
///
/// macOS uses the official full-band DeepFilterNet3 model at 48 kHz. RNNoise
/// remains a small cross-platform fallback so a model initialization error
/// never prevents a recording from starting.
pub struct NoiseSuppressionProcessor {
    backend: NoiseSuppressionBackend,
}

impl NoiseSuppressionProcessor {
    /// Create a new noise suppression processor
    ///
    /// # Arguments
    /// * `sample_rate` - Must be 48000 Hz (RNNoise requirement)
    pub fn new(sample_rate: u32) -> Result<Self> {
        if sample_rate != 48000 {
            return Err(anyhow::anyhow!(
                "Noise suppression requires 48kHz sample rate, got {}Hz",
                sample_rate
            ));
        }

        #[cfg(target_os = "macos")]
        {
            use df::tract::{DfParams, DfTract, RuntimeParams};
            use ndarray_df::{Array2, Axis};
            use std::sync::mpsc;
            use std::time::Duration;

            // Tract's execution state is intentionally !Send. Own it on one
            // inference thread instead of running a neural network inside
            // Core Audio's real-time callback.
            let (input_tx, input_rx) = mpsc::sync_channel::<Vec<f32>>(64);
            let (output_tx, output_rx) = mpsc::sync_channel::<Vec<f32>>(64);
            let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<usize>>(1);

            let thread_result = std::thread::Builder::new()
                .name("meetily-deepfilternet".to_string())
                .spawn(move || {
                    // Limit maximum attenuation so quiet consonants and
                    // code-switched words are not erased with the noise.
                    let runtime = RuntimeParams::default().with_atten_lim(35.0);
                    let mut model = match DfTract::new(DfParams::default(), &runtime) {
                        Ok(model) => model,
                        Err(error) => {
                            let _ = ready_tx.send(Err(anyhow::anyhow!(error)));
                            return;
                        }
                    };
                    let frame_size = model.hop_size;
                    if ready_tx.send(Ok(frame_size)).is_err() {
                        return;
                    }

                    while let Ok(frame) = input_rx.recv() {
                        let noisy = match Array2::from_shape_vec((1, frame_size), frame) {
                            Ok(frame) => frame,
                            Err(error) => {
                                warn!("DeepFilterNet input frame error: {error}");
                                continue;
                            }
                        };
                        let mut enhanced = Array2::<f32>::zeros((1, frame_size));
                        let result = match model.process(noisy.view(), enhanced.view_mut()) {
                            Ok(_) => enhanced.index_axis(Axis(0), 0).iter().copied().collect(),
                            Err(error) => {
                                warn!("DeepFilterNet frame failed: {error}");
                                noisy.index_axis(Axis(0), 0).iter().copied().collect()
                            }
                        };
                        if output_tx.send(result).is_err() {
                            break;
                        }
                    }
                });

            if let Err(error) = thread_result {
                warn!("Could not start DeepFilterNet worker ({error}); falling back to RNNoise");
            } else {
                match ready_rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(frame_size)) => {
                        info!(
                            "✅ DeepFilterNet3 microphone enhancement initialized ({}-sample frames @ 48kHz)",
                            frame_size
                        );
                        return Ok(Self {
                            backend: NoiseSuppressionBackend::DeepFilterNet {
                                input: input_tx,
                                output: output_rx,
                                frame_buffer: Vec::with_capacity(frame_size * 2),
                                frame_size,
                            },
                        });
                    }
                    Ok(Err(error)) => {
                        #[cfg(test)]
                        eprintln!("DeepFilterNet3 initialization error: {error:#}");
                        warn!(
                            "DeepFilterNet3 initialization failed ({error}); falling back to RNNoise"
                        );
                    }
                    Err(error) => warn!(
                        "DeepFilterNet3 initialization timed out ({error}); falling back to RNNoise"
                    ),
                }
            }
        }

        const FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;
        info!(
            "Initializing RNNoise fallback (frame size: {} samples @ 48kHz)",
            FRAME_SIZE
        );
        Ok(Self {
            backend: NoiseSuppressionBackend::RnNoise {
                denoiser: *DenoiseState::new(),
                frame_buffer: Vec::with_capacity(FRAME_SIZE * 2),
                frame_size: FRAME_SIZE,
            },
        })
    }

    /// Apply noise suppression to audio samples
    ///
    /// Processes audio in 480-sample frames (10ms at 48kHz).
    /// Buffers partial frames for next call.
    ///
    /// CRITICAL FIX: Always returns same length as input to prevent latency accumulation
    ///
    /// # Arguments
    /// * `samples` - Input audio samples at 48kHz
    ///
    /// # Returns
    /// Noise-suppressed audio samples (SAME LENGTH as input)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }

        match &mut self.backend {
            #[cfg(target_os = "macos")]
            NoiseSuppressionBackend::DeepFilterNet {
                input,
                output: enhanced_output,
                frame_buffer,
                frame_size,
            } => {
                use std::sync::mpsc::TrySendError;

                frame_buffer.extend_from_slice(samples);
                let mut output = Vec::with_capacity(samples.len());
                while frame_buffer.len() >= *frame_size {
                    let frame: Vec<f32> = frame_buffer.drain(..*frame_size).collect();
                    match input.try_send(frame) {
                        Ok(()) => {}
                        Err(TrySendError::Full(frame)) => {
                            // Capture must never block or drop audio because
                            // enhancement temporarily falls behind.
                            warn!("DeepFilterNet queue full; passing one frame through");
                            output.extend(frame);
                        }
                        Err(TrySendError::Disconnected(frame)) => output.extend(frame),
                    }
                }
                while let Ok(frame) = enhanced_output.try_recv() {
                    output.extend(frame);
                }
                output
            }
            NoiseSuppressionBackend::RnNoise {
                denoiser,
                frame_buffer,
                frame_size,
            } => {
                frame_buffer.extend_from_slice(samples);
                let mut output = Vec::with_capacity(samples.len());
                while frame_buffer.len() >= *frame_size {
                    let frame: Vec<f32> = frame_buffer.drain(..*frame_size).collect();
                    let mut denoised_frame = vec![0.0f32; *frame_size];
                    let _ = denoiser.process_frame(&mut denoised_frame, &frame);
                    output.extend(denoised_frame);
                }
                output
            }
        }
    }

    /// Get the number of buffered samples waiting for processing
    pub fn buffered_samples(&self) -> usize {
        match &self.backend {
            #[cfg(target_os = "macos")]
            NoiseSuppressionBackend::DeepFilterNet { frame_buffer, .. } => frame_buffer.len(),
            NoiseSuppressionBackend::RnNoise { frame_buffer, .. } => frame_buffer.len(),
        }
    }

    /// Flush any remaining buffered samples
    /// Call this at the end of recording to process partial frames
    pub fn flush(&mut self) -> Vec<f32> {
        let (remaining, frame_size) = match &self.backend {
            #[cfg(target_os = "macos")]
            NoiseSuppressionBackend::DeepFilterNet {
                frame_buffer,
                frame_size,
                ..
            } => (frame_buffer.len(), *frame_size),
            NoiseSuppressionBackend::RnNoise {
                frame_buffer,
                frame_size,
                ..
            } => (frame_buffer.len(), *frame_size),
        };
        if remaining == 0 {
            return Vec::new();
        }

        let padding = vec![0.0; frame_size - remaining];
        let mut output = self.process(&padding);
        output.truncate(remaining);
        output
    }
}

/// High-pass filter to remove low-frequency rumble and noise
/// Removes frequencies below cutoff_hz (typically 80-100 Hz for speech)
pub struct HighPassFilter {
    #[allow(dead_code)]
    sample_rate: f32,
    #[allow(dead_code)]
    cutoff_hz: f32,
    // First-order IIR filter coefficients
    alpha: f32,
    prev_input: f32,
    prev_output: f32,
}

impl HighPassFilter {
    /// Create a new high-pass filter
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `cutoff_hz` - Cutoff frequency in Hz (typical: 80-100 Hz for speech)
    pub fn new(sample_rate: u32, cutoff_hz: f32) -> Self {
        let sample_rate_f = sample_rate as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate_f;
        let alpha = rc / (rc + dt);

        info!(
            "Initializing high-pass filter: cutoff={}Hz @ {}Hz",
            cutoff_hz, sample_rate
        );

        Self {
            sample_rate: sample_rate_f,
            cutoff_hz,
            alpha,
            prev_input: 0.0,
            prev_output: 0.0,
        }
    }

    /// Apply high-pass filter to audio samples
    /// Uses first-order IIR (Infinite Impulse Response) filter
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());

        for &sample in samples {
            // First-order high-pass IIR filter formula:
            // y[n] = alpha * (y[n-1] + x[n] - x[n-1])
            let filtered = self.alpha * (self.prev_output + sample - self.prev_input);

            self.prev_input = sample;
            self.prev_output = filtered;

            output.push(filtered);
        }

        output
    }

    /// Reset filter state (call when starting new recording)
    pub fn reset(&mut self) {
        self.prev_input = 0.0;
        self.prev_output = 0.0;
    }
}

pub fn spectral_subtraction(audio: &[f32], d: f32) -> Result<Vec<f32>> {
    let mut real_planner = RealFftPlanner::<f32>::new();
    let window_size = 1600; // 16k sample rate - 100ms

    // CRITICAL FIX: Handle cases where audio is longer than window size
    if audio.is_empty() {
        return Ok(Vec::new());
    }

    // If audio is longer than window size, truncate to prevent overflow
    let processed_audio = if audio.len() > window_size {
        warn!(
            "Audio length {} exceeds window size {}, truncating",
            audio.len(),
            window_size
        );
        &audio[..window_size]
    } else {
        audio
    };

    let r2c = real_planner.plan_fft_forward(window_size);
    let mut y = r2c.make_output_vec();

    // Safe padding: only pad if audio is shorter than window size
    let mut padded_audio = processed_audio.to_vec();
    if processed_audio.len() < window_size {
        let padding_needed = window_size - processed_audio.len();
        padded_audio.extend(vec![0.0f32; padding_needed]);
    }

    let mut indata = padded_audio;
    r2c.process(&mut indata, &mut y)?;

    let mut processed_audio = y
        .iter()
        .map(|&x| {
            let magnitude_y = x.abs().powf(2.0);

            let div = 1.0 - (d / magnitude_y);

            let gain = {
                if div > 0.0 {
                    f32::sqrt(div)
                } else {
                    0.0f32
                }
            };

            x * gain
        })
        .collect::<Vec<Complex32>>();

    let c2r = real_planner.plan_fft_inverse(window_size);

    let mut outdata = c2r.make_output_vec();

    c2r.process(&mut processed_audio, &mut outdata)?;

    Ok(outdata)
}

// not an average of non-speech segments, but I don't know how much pause time we
// get. for now, we will just assume the noise is constant (kinda defeats the purpose)
// but oh well
pub fn average_noise_spectrum(audio: &[f32]) -> f32 {
    let mut total_sum = 0.0f32;

    for sample in audio {
        let magnitude = sample.abs();

        total_sum += magnitude.powf(2.0);
    }

    total_sum / audio.len() as f32
}

pub fn audio_to_mono(audio: &[f32], channels: u16) -> Vec<f32> {
    let mut mono_samples = Vec::with_capacity(audio.len() / channels as usize);

    // For microphone arrays (> 2 channels), only use first 2 channels
    // Many microphone arrays have auxiliary channels for beam-forming/noise cancellation
    // that can contain anti-phase signals. Averaging all channels can cause destructive
    // interference resulting in near-zero output.
    let effective_channels = if channels > 2 { 2 } else { channels };

    // Iterate over the audio slice in chunks, each containing `channels` samples
    for chunk in audio.chunks(channels as usize) {
        // Sum only the first effective_channels (typically 1-2 for mic arrays)
        let sum: f32 = chunk.iter().take(effective_channels as usize).sum();

        // Calculate the average mono sample using effective channel count
        let mono_sample = sum / effective_channels as f32;

        // Store the computed mono sample
        mono_samples.push(mono_sample);
    }

    mono_samples
}

/// High-quality audio resampling with adaptive parameters based on sample rate ratio
///
/// This function automatically selects the best resampling parameters based on:
/// - Sample rate ratio (upsampling vs downsampling)
/// - Quality requirements (integer ratios get optimized paths)
/// - Anti-aliasing needs
///
/// Supports all common sample rates: 8kHz, 16kHz, 24kHz, 44.1kHz, 48kHz, etc.
pub fn resample(input: &[f32], from_sample_rate: u32, to_sample_rate: u32) -> Result<Vec<f32>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Fast path: No resampling needed
    if from_sample_rate == to_sample_rate {
        return Ok(input.to_vec());
    }

    let ratio = to_sample_rate as f64 / from_sample_rate as f64;

    // Adaptive parameters based on sample rate ratio
    let (sinc_len, interpolation_type, oversampling) = if ratio >= 2.0 {
        // Large upsampling (e.g., 8kHz → 16kHz, 16kHz → 48kHz, 24kHz → 48kHz)
        // Needs high quality to avoid artifacts
        debug!(
            "High-quality upsampling: {}Hz → {}Hz (ratio: {:.2}x)",
            from_sample_rate, to_sample_rate, ratio
        );
        (
            512,                          // Longer sinc for smoother interpolation
            SincInterpolationType::Cubic, // Cubic for best quality
            512,                          // Higher oversampling
        )
    } else if ratio >= 1.5 {
        // Moderate upsampling (e.g., 32kHz → 48kHz)
        debug!(
            "Moderate upsampling: {}Hz → {}Hz (ratio: {:.2}x)",
            from_sample_rate, to_sample_rate, ratio
        );
        (384, SincInterpolationType::Cubic, 384)
    } else if ratio > 1.0 {
        // Small upsampling (e.g., 44.1kHz → 48kHz)
        debug!(
            "Small upsampling: {}Hz → {}Hz (ratio: {:.2}x)",
            from_sample_rate, to_sample_rate, ratio
        );
        (256, SincInterpolationType::Linear, 256)
    } else if ratio <= 0.5 {
        // Large downsampling (e.g., 48kHz → 16kHz, 48kHz → 8kHz)
        // Needs strong anti-aliasing
        debug!(
            "Anti-aliased downsampling: {}Hz → {}Hz (ratio: {:.2}x)",
            from_sample_rate, to_sample_rate, ratio
        );
        (
            512,                          // Longer sinc for anti-aliasing
            SincInterpolationType::Cubic, // Cubic for quality
            512,
        )
    } else {
        // Moderate downsampling (e.g., 48kHz → 24kHz, 48kHz → 32kHz)
        debug!(
            "Moderate downsampling: {}Hz → {}Hz (ratio: {:.2}x)",
            from_sample_rate, to_sample_rate, ratio
        );
        (384, SincInterpolationType::Linear, 384)
    };

    let params = SincInterpolationParameters {
        sinc_len,
        f_cutoff: 0.95, // Preserve most of the frequency content
        interpolation: interpolation_type,
        oversampling_factor: oversampling,
        window: WindowFunction::BlackmanHarris2, // Best window for audio
    };

    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        2.0, // Maximum relative deviation
        params,
        input.len(),
        1, // Mono
    )?;

    let waves_in = vec![input.to_vec()];
    let waves_out = resampler.process(&waves_in, None)?;

    debug!(
        "Resampling complete: {} samples → {} samples",
        input.len(),
        waves_out[0].len()
    );

    Ok(waves_out.into_iter().next().unwrap())
}

// Alias for compatibility with existing code
pub fn resample_audio(input: &[f32], from_sample_rate: u32, to_sample_rate: u32) -> Vec<f32> {
    match resample(input, from_sample_rate, to_sample_rate) {
        Ok(result) => result,
        Err(e) => {
            debug!("Resampling failed: {}, returning original audio", e);
            input.to_vec()
        }
    }
}

/// Fast resampling optimized for transcription preprocessing
///
pub fn write_audio_to_file(
    audio: &[f32],
    sample_rate: u32,
    output_path: &PathBuf,
    device: &str,
    skip_encoding: bool,
) -> Result<String> {
    write_audio_to_file_with_meeting_name(
        audio,
        sample_rate,
        output_path,
        device,
        skip_encoding,
        None,
    )
}

pub fn write_audio_to_file_with_meeting_name(
    audio: &[f32],
    sample_rate: u32,
    output_path: &PathBuf,
    device: &str,
    skip_encoding: bool,
    meeting_name: Option<&str>,
) -> Result<String> {
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let sanitized_device_name = device.replace(['/', '\\'], "_");

    // Create meeting folder if meeting name is provided
    let final_output_path = if let Some(name) = meeting_name {
        let sanitized_meeting_name = sanitize_filename(name);
        let meeting_folder = output_path.join(&sanitized_meeting_name);

        // Create the meeting folder if it doesn't exist
        if !meeting_folder.exists() {
            std::fs::create_dir_all(&meeting_folder)?;
        }

        meeting_folder
    } else {
        output_path.clone()
    };

    let file_path = final_output_path
        .join(format!("{}_{}.mp4", sanitized_device_name, timestamp))
        .to_str()
        .expect("Failed to create valid path")
        .to_string();
    let file_path_clone = file_path.clone();
    // Run FFmpeg in a separate task
    if !skip_encoding {
        encode_single_audio(
            bytemuck::cast_slice(audio),
            sample_rate,
            1,
            &file_path.into(),
        )?;
    }
    Ok(file_path_clone)
}

/// Write transcript text to a file alongside the recording (legacy plain text format)
pub fn write_transcript_to_file(
    transcript_text: &str,
    output_path: &PathBuf,
    meeting_name: Option<&str>,
) -> Result<String> {
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Create meeting folder if meeting name is provided (same logic as audio)
    let final_output_path = if let Some(name) = meeting_name {
        let sanitized_meeting_name = sanitize_filename(name);
        let meeting_folder = output_path.join(&sanitized_meeting_name);

        // Create the meeting folder if it doesn't exist
        if !meeting_folder.exists() {
            std::fs::create_dir_all(&meeting_folder)?;
        }

        meeting_folder
    } else {
        output_path.clone()
    };

    let file_path = final_output_path.join(format!("transcript_{}.txt", timestamp));

    // Write transcript to file
    std::fs::write(&file_path, transcript_text)?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Write structured transcript with timestamps to JSON file
pub fn write_transcript_json_to_file(
    segments: &[super::recording_saver::TranscriptSegment],
    output_path: &PathBuf,
    meeting_name: Option<&str>,
    audio_filename: &str,
    recording_duration: f64,
) -> Result<String> {
    use serde_json::json;

    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Create meeting folder if meeting name is provided
    let final_output_path = if let Some(name) = meeting_name {
        let sanitized_meeting_name = sanitize_filename(name);
        let meeting_folder = output_path.join(&sanitized_meeting_name);

        if !meeting_folder.exists() {
            std::fs::create_dir_all(&meeting_folder)?;
        }

        meeting_folder
    } else {
        output_path.clone()
    };

    let file_path = final_output_path.join(format!("transcript_{}.json", timestamp));

    // Create structured JSON transcript
    let transcript_json = json!({
        "version": "1.0",
        "recording_duration": recording_duration,
        "audio_file": audio_filename,
        "sample_rate": 48000,
        "created_at": Utc::now().to_rfc3339(),
        "meeting_name": meeting_name,
        "segments": segments,
    });

    // Write JSON to file with pretty formatting
    let json_string = serde_json::to_string_pretty(&transcript_json)?;
    std::fs::write(&file_path, json_string)?;

    Ok(file_path.to_string_lossy().to_string())
}

#[cfg(all(test, target_os = "macos"))]
mod neural_enhancement_tests {
    use super::{NoiseSuppressionBackend, NoiseSuppressionProcessor};
    use std::time::{Duration, Instant};

    #[test]
    fn deepfilternet_initializes_and_returns_finite_audio() {
        let mut processor =
            NoiseSuppressionProcessor::new(48_000).expect("noise suppressor should initialize");
        assert!(
            matches!(
                &processor.backend,
                NoiseSuppressionBackend::DeepFilterNet { .. }
            ),
            "macOS should use DeepFilterNet3 rather than the RNNoise fallback"
        );

        let input_frame = vec![0.01f32; 480];
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut enhanced = Vec::new();
        while enhanced.is_empty() && Instant::now() < deadline {
            enhanced.extend(processor.process(&input_frame));
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(
            !enhanced.is_empty(),
            "DeepFilterNet3 did not return an enhanced frame"
        );
        assert!(
            enhanced.iter().all(|sample| sample.is_finite()),
            "DeepFilterNet3 returned non-finite audio"
        );
    }
}
