//! Envelope and RMS extracting
//!
//! Provides utilities for extracting the amplitude contour of an audio signal
//! via sliding window Root Mean Square (RMS) or simple peak tracking.

/// Configuration for the RMS Envelope follower.
#[derive(Debug, Clone, Copy)]
pub struct EnvConfig {
    /// Window size in samples over which to calculate the RMS.
    pub window_size: usize,
    /// Hop size in samples between consecutive RMS calculations.
    pub hop_size: usize,
    /// Audio sample rate (Hz). Used for time-based calculations.
    pub sample_rate: f64,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            window_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
        }
    }
}

/// Calculate the RMS envelope of an entire audio buffer.
/// Returns a vector of RMS values, spaced by `config.hop_size` samples.
pub fn extract_rms_envelope(config: &EnvConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut envelope = Vec::new();
    let mut start = 0;

    // Reserve approximate capacity
    envelope.reserve(audio.len() / config.hop_size + 1);

    while start < audio.len() {
        let end = (start + config.window_size).min(audio.len());
        let window = &audio[start..end];

        let mut sum_sq = 0.0;
        for &sample in window {
            sum_sq += sample * sample;
        }

        // Root mean square
        let rms = (sum_sq / window.len() as f64).sqrt();
        envelope.push(rms);

        start += config.hop_size;
    }

    envelope
}
