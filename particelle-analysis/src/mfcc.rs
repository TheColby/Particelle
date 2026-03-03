//! MFCC (Mel-Frequency Cepstral Coefficients)
//!
//! Extracts individual MFCC coefficients (1–13) over time.
//! These are the standard perceptual features used in speech and music recognition.

use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
use crate::spectral::SpectralConfig;

const N_MELS: usize = 26;
const N_MFCC: usize = 13;

/// Compute a mel filterbank from FFT power bins.
fn mel_filterbank(n_fft: usize, sample_rate: f64, n_mels: usize) -> Vec<Vec<f64>> {
    let hz_to_mel = |hz: f64| 2595.0 * (1.0 + hz / 700.0).log10();
    let mel_to_hz = |mel: f64| 700.0 * (10.0f64.powf(mel / 2595.0) - 1.0);
    
    let mel_low = hz_to_mel(0.0);
    let mel_high = hz_to_mel(sample_rate / 2.0);
    
    // Linearly spaced mel points
    let mel_points: Vec<f64> = (0..=n_mels + 1)
        .map(|i| mel_low + (mel_high - mel_low) * i as f64 / (n_mels + 1) as f64)
        .collect();
    
    // Convert back to Hz then to FFT bin indices
    let bin_points: Vec<usize> = mel_points
        .iter()
        .map(|&m| ((n_fft as f64 + 1.0) * mel_to_hz(m) / sample_rate).floor() as usize)
        .collect();
    
    // Build triangular filters
    let mut filterbank = vec![vec![0.0; n_fft / 2]; n_mels];
    for m in 0..n_mels {
        let f_left = bin_points[m];
        let f_center = bin_points[m + 1];
        let f_right = bin_points[m + 2];

        for k in f_left..f_center {
            if f_center > f_left && k < n_fft / 2 {
                filterbank[m][k] = (k - f_left) as f64 / (f_center - f_left) as f64;
            }
        }
        for k in f_center..f_right {
            if f_right > f_center && k < n_fft / 2 {
                filterbank[m][k] = (f_right - k) as f64 / (f_right - f_center) as f64;
            }
        }
    }
    filterbank
}

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

/// Extract an individual MFCC coefficient over time.
/// `coeff` must be in the range 0..13 (returns coefficient at 1-indexed position coeff+1).
pub fn extract_mfcc(config: &SpectralConfig, audio: &[f64], coeff: usize) -> Vec<f64> {
    if audio.is_empty() || coeff >= N_MFCC { return Vec::new(); }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(config.window_size);
    let mut complex_buf = vec![Complex { re: 0.0, im: 0.0 }; config.window_size];
    let nyquist_bins = config.window_size / 2;

    let filterbank = mel_filterbank(config.window_size, config.sample_rate, N_MELS);

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        windowed_power_spectrum(&audio[start..start + config.window_size], &fft, &mut complex_buf);

        // Apply Mel filterbank
        let mut mel_energies = vec![0.0f64; N_MELS];
        for m in 0..N_MELS {
            for k in 0..nyquist_bins {
                let power = complex_buf[k].norm_sqr();
                mel_energies[m] += filterbank[m][k] * power;
            }
            mel_energies[m] = mel_energies[m].max(f64::MIN_POSITIVE).ln();
        }

        // DCT-II to get cepstral coefficients
        let mut c = 0.0;
        for m in 0..N_MELS {
            c += mel_energies[m] 
                * (std::f64::consts::PI * coeff as f64 * (m as f64 + 0.5) / N_MELS as f64).cos();
        }

        contour.push(c);
        start += config.hop_size;
    }
    contour
}

/// Convenience: extract MFCC coefficient 1 (lowest, most important)
pub fn extract_mfcc1(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 1) }
pub fn extract_mfcc2(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 2) }
pub fn extract_mfcc3(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 3) }
pub fn extract_mfcc4(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 4) }
pub fn extract_mfcc5(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 5) }
pub fn extract_mfcc6(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 6) }
pub fn extract_mfcc7(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 7) }
pub fn extract_mfcc8(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 8) }
pub fn extract_mfcc9(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 9) }
pub fn extract_mfcc10(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 10) }
pub fn extract_mfcc11(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 11) }
pub fn extract_mfcc12(config: &SpectralConfig, audio: &[f64]) -> Vec<f64> { extract_mfcc(config, audio, 12) }
