//! Chroma / Pitch Class Features
//!
//! Extract energy per pitch class (C, C#, D, … B) aggregated into 12-bin chroma vectors.
//! Each chroma vector is then reduced to a single scalar (most active pitch class index,
//! or chroma energy of a specific pitch class) for parameter mapping.

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

/// Compute the 12-bin normalized chroma energy from a power spectrum.
fn chroma_vector(complex_buf: &[Complex<f64>], nyquist_bins: usize, bin_hz: f64) -> [f64; 12] {
    let mut chroma = [0.0f64; 12];

    for i in 1..nyquist_bins {
        let freq_hz = i as f64 * bin_hz;
        if freq_hz < 27.5 || freq_hz > 4186.0 { continue; } // Piano keyboard range
        
        // MIDI note number then pitch class (0 = C, 11 = B)
        let midi = 69.0 + 12.0 * (freq_hz / 440.0).log2();
        let pitch_class = midi.round() as i64 % 12;
        let pitch_class = (pitch_class + 12) as usize % 12;

        chroma[pitch_class] += complex_buf[i].norm_sqr();
    }

    // L1-normalize
    let total: f64 = chroma.iter().sum();
    if total > f64::MIN_POSITIVE {
        for c in chroma.iter_mut() { *c /= total; }
    }
    chroma
}

/// Extract the active pitch class index over time (0=C, 1=C#, …, 11=B).
/// Useful for tracking the key/tonality of the signal frame-by-frame.
pub fn extract_chroma_active_class(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
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
        let chroma = chroma_vector(&complex_buf, nyquist_bins, bin_hz);
        let strongest = chroma.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i as f64)
            .unwrap_or(0.0);
        contour.push(strongest);
        start += config.hop_size;
    }
    contour
}

/// Extract chroma vector energy for a specific pitch class over time.
/// `pitch_class`: 0=C, 1=C#, 2=D, 3=D#, 4=E, 5=F, 6=F#, 7=G, 8=G#, 9=A, 10=A#, 11=B
pub fn extract_chroma_energy(config: &SpectralConfig, audio: &[f64], pitch_class: usize) -> Vec<f64> {
    let pc = pitch_class.min(11);
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
        let chroma = chroma_vector(&complex_buf, nyquist_bins, bin_hz);
        contour.push(chroma[pc]);
        start += config.hop_size;
    }
    contour
}

/// Overall chroma "strength": variance across pitch classes (high = dominant pitch class).
pub fn extract_chroma_strength(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> {
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
        let chroma = chroma_vector(&complex_buf, nyquist_bins, bin_hz);
        let mean = chroma.iter().sum::<f64>() / 12.0;
        let var = chroma.iter().map(|c| (c - mean) * (c - mean)).sum::<f64>() / 12.0;
        contour.push(var.sqrt() * 12.0); // scale up for useability
        start += config.hop_size;
    }
    contour
}
