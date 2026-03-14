//! Temporal Features
//!
//! Provides utilities for extracting time-domain information like zero crossing rate.

/// Configuration for temporal feature extractors.
#[derive(Debug, Clone, Copy)]
pub struct TemporalConfig {
    pub window_size: usize,
    pub hop_size: usize,
    pub sample_rate: f64,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            window_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
        }
    }
}

/// Calculate the Zero Crossing Rate (ZCR) of the signal over time.
/// Returns the normalized crossing rate (0.0 to 1.0) per window.
pub fn extract_zero_crossing_rate(config: &TemporalConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        let mut crossings = 0;
        for i in 1..slice.len() {
            if (slice[i - 1] < 0.0 && slice[i] >= 0.0) || (slice[i - 1] >= 0.0 && slice[i] < 0.0) {
                crossings += 1;
            }
        }
        contour.push(crossings as f64 / slice.len() as f64);
        start += config.hop_size;
    }

    contour
}
