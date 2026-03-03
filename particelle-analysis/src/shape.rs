//! Spectral Shape Features
//!
//! High-order statistical moments and information-theoretic measures
//! of the spectral magnitude distribution.

use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
use crate::spectral::SpectralConfig;

fn windowed_power_spectrum(
    audio_slice: &[f64], 
    fft: &Arc<dyn rustfft::Fft<f64>>, 
    complex_buf: &mut [Complex<f64>]
) {
    let l = audio_slice.len() as f64;
    for i in 0..audio_slice.len() {
        let wr = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / l).cos());
        complex_buf[i] = Complex { re: audio_slice[i] * wr, im: 0.0 };
    }
    fft.process(complex_buf);
}

/// Extract Spectral Spread over time (second central moment of spectrum).
/// The standard deviation of frequencies around the centroid.
pub fn extract_spectral_spread(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() { return Vec::new(); }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;
    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);
        let mut total_mag = 0.0;
        let mut centroid = 0.0;
        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            centroid += i as f64 * bin_hz * m;
            total_mag += m;
        }
        if total_mag > f64::MIN_POSITIVE { centroid /= total_mag; }

        let mut variance = 0.0;
        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            let diff = (i as f64 * bin_hz) - centroid;
            variance += diff * diff * m;
        }
        if total_mag > f64::MIN_POSITIVE { variance /= total_mag; }

        contour.push(variance.sqrt());
        start += config.hop_size;
    }
    contour
}

/// Extract Spectral Skewness over time (third normalised central moment).
/// Positive = spectrum skewed toward higher frequencies.
pub fn extract_spectral_skewness(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() { return Vec::new(); }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;
    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);
        let mut total_mag = 0.0;
        let mut centroid = 0.0;
        let mut m2 = 0.0;
        let mut m3 = 0.0;

        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            total_mag += m;
            centroid += i as f64 * bin_hz * m;
        }
        if total_mag > f64::MIN_POSITIVE { centroid /= total_mag; }

        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            let d = (i as f64 * bin_hz) - centroid;
            m2 += d * d * m;
            m3 += d * d * d * m;
        }
        if total_mag > f64::MIN_POSITIVE { m2 /= total_mag; m3 /= total_mag; }
        let sigma = m2.sqrt();
        let skewness = if sigma > f64::MIN_POSITIVE { m3 / (sigma * sigma * sigma) } else { 0.0 };
        contour.push(skewness);
        start += config.hop_size;
    }
    contour
}

/// Extract Spectral Kurtosis over time (fourth normalised central moment).
/// Measures the "tailedness" of the spectral distribution.
pub fn extract_spectral_kurtosis(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() { return Vec::new(); }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;
    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);
        let mut total_mag = 0.0;
        let mut centroid = 0.0;
        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            total_mag += m;
            centroid += i as f64 * bin_hz * m;
        }
        if total_mag > f64::MIN_POSITIVE { centroid /= total_mag; }

        let mut m2 = 0.0;
        let mut m4 = 0.0;
        for i in 0..nyquist_bins {
            let m = complex_buf[i].norm();
            let d = (i as f64 * bin_hz) - centroid;
            m2 += d * d * m;
            m4 += d * d * d * d * m;
        }
        if total_mag > f64::MIN_POSITIVE { m2 /= total_mag; m4 /= total_mag; }
        let kurtosis = if m2 > f64::MIN_POSITIVE { m4 / (m2 * m2) } else { 0.0 };
        contour.push(kurtosis);
        start += config.hop_size;
    }
    contour
}

/// Extract Spectral Entropy over time.
/// Information entropy of the normalised magnitude distribution. 
/// High entropy = complex/noisy; Low entropy = sparse/tonal.
pub fn extract_spectral_entropy(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() { return Vec::new(); }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);
        let mut total_mag = 0.0;
        for i in 0..nyquist_bins { total_mag += complex_buf[i].norm(); }

        let mut entropy = 0.0;
        if total_mag > f64::MIN_POSITIVE {
            for i in 0..nyquist_bins {
                let p = complex_buf[i].norm() / total_mag;
                if p > f64::MIN_POSITIVE {
                    entropy -= p * p.ln();
                }
            }
            entropy /= (nyquist_bins as f64).ln(); // normalise to [0,1]
        }
        contour.push(entropy.clamp(0.0, 1.0));
        start += config.hop_size;
    }
    contour
}

/// Extract Spectral Contrast over time.
/// Average difference between spectral peaks and valleys across 6 sub-bands.
pub fn extract_spectral_contrast(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() { return Vec::new(); }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;
    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    // Sub-bands based on approximate musical octave boundaries
    let band_edges_hz: [f64; 7] = [200.0, 400.0, 800.0, 1600.0, 3200.0, 6400.0, config.sample_rate / 2.0];

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);
        let magnitudes: Vec<f64> = (0..nyquist_bins).map(|i| complex_buf[i].norm()).collect();
        
        let mut total_contrast = 0.0;
        let mut n_bands = 0;
        let mut low_hz = 0.0;

        for &high_hz in &band_edges_hz {
            let low_bin = (low_hz / bin_hz).floor() as usize;
            let high_bin = (high_hz / bin_hz).ceil().min(nyquist_bins as f64) as usize;

            if high_bin <= low_bin + 2 {
                low_hz = high_hz;
                continue;
            }

            let band = &magnitudes[low_bin..high_bin];
            let mut sorted = band.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let n = sorted.len();
            let top_n = (n / 5).max(1);
            let peak: f64 = sorted[n - top_n..].iter().sum::<f64>() / top_n as f64;
            let valley: f64 = sorted[..top_n].iter().sum::<f64>() / top_n as f64;

            if valley > f64::MIN_POSITIVE {
                total_contrast += (peak / valley).ln();
            }
            n_bands += 1;
            low_hz = high_hz;
        }

        contour.push(if n_bands > 0 { total_contrast / n_bands as f64 } else { 0.0 });
        start += config.hop_size;
    }
    contour
}
