//! Spectral Features
//!
//! Provides utilities for extracting frequency-domain information such as
//! Spectral Flatness (Wiener Entropy) and Spectral Centroid over sliding windows.

use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

/// Configuration for Spectral Feature extractors.
#[derive(Debug, Clone, Copy)]
pub struct SpectralConfig {
    /// FFT window size in samples (typically a power of 2, e.g. 2048)
    pub window_size: usize,
    /// Hop size in samples
    pub hop_size: usize,
    /// Audio sample rate
    pub sample_rate: f64,
}

impl Default for SpectralConfig {
    fn default() -> Self {
        Self {
            window_size: 2048,
            hop_size: 512,
            sample_rate: 48000.0,
        }
    }
}

/// Helper to run a hann window and FFT on a slice, yielding power spectrum
fn compute_power_spectrum(
    audio_slice: &[f64],
    fft: &Arc<dyn rustfft::Fft<f64>>,
    window_buf: &mut [f64],
    complex_buf: &mut [Complex<f64>],
) {
    // Apply Hann window
    let l = window_buf.len() as f64;
    for i in 0..window_buf.len() {
        let wr = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / l).cos());
        window_buf[i] = audio_slice[i] * wr;
        complex_buf[i] = Complex {
            re: window_buf[i],
            im: 0.0,
        };
    }

    fft.process(complex_buf);
}

/// Extract Spectral Flatness (Wiener Entropy) contour from an audio buffer.
/// A high value (approaching 1.0) means white noise. A low value (approaching 0.0) means purely tonal.
pub fn extract_spectral_flatness(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);

    let mut window_buf = vec![0.0; config.window_size];
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        compute_power_spectrum(slice, &fft, &mut window_buf, &mut complex_buf);

        // Power spectrum
        let mut sum_power = 0.0;
        let mut sum_log_power = 0.0;
        // Only iterate up to Nyquist
        let nyquist_bins = config.window_size / 2;
        let mut valid_bins = 0;

        for bin in complex_buf.iter().take(nyquist_bins) {
            let magnitude = bin.norm();
            let power = magnitude * magnitude;
            // Prevent log(0)
            let padded_power = power.max(f64::MIN_POSITIVE);

            sum_power += padded_power;
            sum_log_power += padded_power.ln();
            valid_bins += 1;
        }

        let arithmetic_mean = sum_power / valid_bins as f64;
        let geometric_mean = (sum_log_power / valid_bins as f64).exp();

        let flatness = if arithmetic_mean > f64::MIN_POSITIVE {
            geometric_mean / arithmetic_mean
        } else {
            0.0
        };

        contour.push(flatness);
        start += config.hop_size;
    }

    contour
}

/// Extract Spectral Centroid (brightness) contour from an audio buffer.
/// Returns the "center of mass" of the spectrum in Hz.
pub fn extract_spectral_centroid(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);

    let mut window_buf = vec![0.0; config.window_size];
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    let nyquist_bins = config.window_size / 2;
    let bin_resolution = config.sample_rate / config.window_size as f64;

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        compute_power_spectrum(slice, &fft, &mut window_buf, &mut complex_buf);

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, bin) in complex_buf.iter().enumerate().take(nyquist_bins) {
            let magnitude = bin.norm();
            let freq_hz = i as f64 * bin_resolution;

            numerator += freq_hz * magnitude;
            denominator += magnitude;
        }

        let centroid = if denominator > f64::MIN_POSITIVE {
            numerator / denominator
        } else {
            0.0
        };

        contour.push(centroid);
        start += config.hop_size;
    }

    contour
}

/// Extract Spectral Rolloff contour from an audio buffer.
/// Returns the frequency below which 85% of the spectral energy lies.
pub fn extract_spectral_rolloff(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);

    let mut window_buf = vec![0.0; config.window_size];
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    let nyquist_bins = config.window_size / 2;
    let bin_resolution = config.sample_rate / config.window_size as f64;

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        compute_power_spectrum(slice, &fft, &mut window_buf, &mut complex_buf);

        let mut total_power = 0.0;
        let mut power_spectrum = Vec::with_capacity(nyquist_bins);

        for bin in complex_buf.iter().take(nyquist_bins) {
            let magnitude = bin.norm();
            let power = magnitude * magnitude;
            power_spectrum.push(power);
            total_power += power;
        }

        let threshold = total_power * 0.85;
        let mut cumulative_power = 0.0;
        let mut rolloff_bin = 0;

        for (i, &power) in power_spectrum.iter().enumerate() {
            cumulative_power += power;
            if cumulative_power >= threshold {
                rolloff_bin = i;
                break;
            }
        }

        contour.push(rolloff_bin as f64 * bin_resolution);
        start += config.hop_size;
    }

    contour
}

/// Extract Spectral Crest contour from an audio buffer.
/// Ratio of maximum power to average power in the spectrum.
pub fn extract_spectral_crest(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);

    let mut window_buf = vec![0.0; config.window_size];
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    let nyquist_bins = config.window_size / 2;

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        compute_power_spectrum(slice, &fft, &mut window_buf, &mut complex_buf);

        let mut max_power = 0.0f64;
        let mut sum_power = 0.0f64;

        for bin in complex_buf.iter().take(nyquist_bins) {
            let magnitude = bin.norm();
            let power = magnitude * magnitude;
            if power > max_power {
                max_power = power;
            }
            sum_power += power;
        }

        let arithmetic_mean = sum_power / (nyquist_bins as f64);
        let crest = if arithmetic_mean > f64::MIN_POSITIVE {
            max_power / arithmetic_mean
        } else {
            0.0
        };

        contour.push(crest);
        start += config.hop_size;
    }

    contour
}

/// Extract Spectral Flux contour from an audio buffer.
/// The L2 norm of the difference between successive magnitude spectra.
pub fn extract_spectral_flux(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);

    let mut window_buf = vec![0.0; config.window_size];
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    let nyquist_bins = config.window_size / 2;
    let mut prev_magnitude = vec![0.0; nyquist_bins];

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        compute_power_spectrum(slice, &fft, &mut window_buf, &mut complex_buf);

        let mut flux = 0.0;

        for i in 0..nyquist_bins {
            let magnitude = complex_buf[i].norm();
            let diff = magnitude - prev_magnitude[i];

            // Half-wave rectification for flux (only positive changes)
            if diff > 0.0 {
                flux += diff * diff;
            }

            prev_magnitude[i] = magnitude;
        }

        contour.push(flux.sqrt());
        start += config.hop_size;
    }

    contour
}
