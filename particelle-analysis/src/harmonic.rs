//! Harmonic Features
//!
//! Utilities for quantifying the harmonic content of signals.
//! Includes Harmonic Ratio (HNR), Inharmonicity, and Tristimulus.

use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
use crate::spectral::SpectralConfig;

/// Compute a power spectrum from a windowed block (Hann window).
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

/// Extract Harmonic Ratio contour over time.
/// Quantifies how tonal a signal is (HNR — ratio of harmonic energy to total energy).
/// Uses a basic autocorrelation approach normalized to [0, 1].
pub fn extract_harmonic_ratio(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);
    
    let min_period = (config.sample_rate / 2000.0) as usize; // max 2kHz
    let max_period = (config.sample_rate / 40.0) as usize;   // min 40Hz

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        
        let mut r0 = 0.0;
        for &s in slice {
            r0 += s * s;
        }

        if r0 < f64::MIN_POSITIVE {
            contour.push(0.0);
            start += config.hop_size;
            continue;
        }

        let mut max_r = 0.0;
        let max_lag = max_period.min(config.window_size - 1);

        for lag in min_period..max_lag {
            let mut r = 0.0;
            let n = config.window_size - lag;
            for i in 0..n {
                r += slice[i] * slice[i + lag];
            }
            if r > max_r {
                max_r = r;
            }
        }

        contour.push((max_r / r0).clamp(0.0, 1.0));
        start += config.hop_size;
    }

    contour
}

/// Extract Inharmonicity contour over time.
/// Measures how much the partials deviate from a perfect harmonic series.
/// Higher = more inharmonic (bells, stretched piano strings, percussion).
pub fn extract_inharmonicity(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        windowed_power_spectrum(slice, &fft, &mut complex_buf);

        // Find loudest peak = assume f0
        let mut f0_bin = 1;
        let mut max_mag = 0.0;
        for i in 1..nyquist_bins {
            let m = complex_buf[i].norm();
            if m > max_mag {
                max_mag = m;
                f0_bin = i;
            }
        }

        if f0_bin == 0 {
            contour.push(0.0);
            start += config.hop_size;
            continue;
        }

        let f0 = f0_bin as f64 * bin_hz;
        
        // Measure how far the first 8 local peaks deviate from ideal harmonics
        let mut total_dev = 0.0;
        let mut count = 0;

        for n in 1..=8u32 {
            let ideal_hz = f0 * n as f64;
            let ideal_bin = (ideal_hz / bin_hz) as usize;

            if ideal_bin >= nyquist_bins {
                break;
            }

            // Find peak within ±5% of expected
            let search_range = ((ideal_bin as f64 * 0.05) as usize).max(1);
            let low = ideal_bin.saturating_sub(search_range);
            let high = (ideal_bin + search_range).min(nyquist_bins - 1);

            let mut peak_bin = ideal_bin;
            let mut peak_mag = 0.0;
            for b in low..=high {
                let m = complex_buf[b].norm();
                if m > peak_mag {
                    peak_mag = m;
                    peak_bin = b;
                }
            }
            
            let actual_hz = peak_bin as f64 * bin_hz;
            let deviation = (actual_hz - ideal_hz).abs() / ideal_hz;
            total_dev += deviation;
            count += 1;
        }

        let inharmonicity = if count > 0 { total_dev / count as f64 } else { 0.0 };
        contour.push(inharmonicity.clamp(0.0, 1.0));
        start += config.hop_size;
    }

    contour
}

/// Extract Tristimulus-1 contour: ratio of first harmonic energy to total energy.
/// Measures the "purity" — high value means fundamental dominates.
pub fn extract_tristimulus1(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;
    let bin_hz = config.sample_rate / config.window_size as f64;

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        windowed_power_spectrum(slice, &fft, &mut complex_buf);

        // Identify f0 as loudest bin
        let mut f0_bin = 1;
        let mut max_mag = 0.0;
        for i in 1..nyquist_bins {
            let m = complex_buf[i].norm();
            if m > max_mag { max_mag = m; f0_bin = i; }
        }
        let f0_hz = f0_bin as f64 * bin_hz;
        
        let mut total = 0.0;
        let mut h1_energy = 0.0;
        let search = 2usize;
        
        for b in 0..nyquist_bins {
            let m = complex_buf[b].norm();
            let power = m * m;
            total += power;
            
            // Bins near f0 (harmonic 1)
            let freq = b as f64 * bin_hz;
            if (freq - f0_hz).abs() < bin_hz * search as f64 {
                h1_energy += power;
            }
        }

        let t1 = if total > f64::MIN_POSITIVE { h1_energy / total } else { 0.0 };
        contour.push(t1.clamp(0.0, 1.0));
        start += config.hop_size;
    }

    contour
}
