//! Dynamic and Perceptual Features
//!
//! Peak amplitude tracking, perceptual loudness approximation, log attack time.

use crate::envelope::EnvConfig;

/// Compute peak amplitude per window.
pub fn extract_peak_amplitude(config: &EnvConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        let peak = slice.iter().cloned().fold(0.0f64, |a, x| a.max(x.abs()));
        contour.push(peak);
        start += config.hop_size;
    }
    contour
}

/// Compute RMS-based loudness contour in dBFS.
/// Returns values in the range [-96, 0] dBFS (silence at -96, full scale at 0).
pub fn extract_loudness_dbfs(config: &EnvConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        let rms = (slice.iter().map(|&s| s * s).sum::<f64>() / slice.len() as f64).sqrt();
        let db = 20.0 * rms.max(1e-5).log10(); // cap at -100dBFS
        contour.push(db.clamp(-96.0, 0.0));
        start += config.hop_size;
    }
    contour
}

/// Compute crest factor contour (peak / RMS ratio).
/// Measures impulsiveness: high value = impulsive/percussive; low value = sustained.
pub fn extract_crest_factor(config: &EnvConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() || config.window_size == 0 || config.hop_size == 0 {
        return Vec::new();
    }

    let mut start = 0;
    let mut contour = Vec::with_capacity(audio.len() / config.hop_size + 1);

    while start + config.window_size <= audio.len() {
        let slice = &audio[start..start + config.window_size];
        let rms = (slice.iter().map(|&s| s * s).sum::<f64>() / slice.len() as f64).sqrt();
        let peak = slice.iter().cloned().fold(0.0f64, |a, x| a.max(x.abs()));
        let crest = if rms > f64::MIN_POSITIVE {
            peak / rms
        } else {
            0.0
        };
        contour.push(crest);
        start += config.hop_size;
    }
    contour
}

/// Estimate Log Attack Time (LAT) for the entire signal.
/// Returns a single scalar in log seconds; more negative = faster attack.
/// This is a global measurement so the returned vector will have one element.
pub fn estimate_log_attack_time(config: &EnvConfig, audio: &[f64]) -> Vec<f64> {
    if audio.is_empty() {
        return Vec::new();
    }

    // Find global RMS peak
    let global_rms_vec = super::envelope::extract_rms_envelope(config, audio);
    let max_rms = global_rms_vec.iter().cloned().fold(0.0f64, f64::max);

    if max_rms < f64::MIN_POSITIVE {
        return vec![0.0];
    }

    let onset_threshold = max_rms * 0.2; // 20% of peak
    let attack_threshold = max_rms * 0.9; // 90% of peak

    let mut onset_frame = 0;
    let mut attack_frame = 0;

    for (i, &rms) in global_rms_vec.iter().enumerate() {
        if rms >= onset_threshold && onset_frame == 0 {
            onset_frame = i;
        }
        if rms >= attack_threshold {
            attack_frame = i;
            break;
        }
    }

    if attack_frame <= onset_frame {
        return vec![-4.0]; // very fast: ~0.1ms
    }

    let attack_time_sec =
        (attack_frame - onset_frame) as f64 * config.hop_size as f64 / config.sample_rate;

    vec![attack_time_sec.max(1e-4).log10()]
}
