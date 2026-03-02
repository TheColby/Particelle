use std::sync::Arc;
use crate::tuning::Tuning;

/// Full pitch pipeline: degree → tuning → bend → curve offset → mod → final Hz.
///
/// All arithmetic is f64. No rounding occurs until the caller converts to
/// a playback ratio.
pub struct PitchPipeline {
    pub tuning: Arc<dyn Tuning>,
    /// Reference pitch for degree 0 (e.g., 261.625... for C4).
    pub base_hz: f64,
    /// MPE pitchbend range in semitones (e.g., 48.0 for ±48 semitone MPE range).
    pub pitchbend_range_semitones: f64,
}

impl PitchPipeline {
    pub fn new(tuning: Arc<dyn Tuning>, base_hz: f64, pitchbend_range_semitones: f64) -> Self {
        Self { tuning, base_hz, pitchbend_range_semitones }
    }

    /// Resolve a complete pitch to Hz.
    ///
    /// - `degree`: scale degree (integer)
    /// - `pitchbend`: normalized pitchbend in [-1, 1] (MPE)
    /// - `curve_offset_cents`: additive cents from curve modulation
    /// - `mod_hz`: additive Hz from LFO/env modulation
    pub fn resolve(
        &self,
        degree: i32,
        pitchbend: f64,
        curve_offset_cents: f64,
        mod_hz: f64,
    ) -> f64 {
        let tuning_hz = self.tuning.degree_to_freq(degree, self.base_hz);
        let bend_octaves = pitchbend * self.pitchbend_range_semitones / 12.0;
        let bent_hz = tuning_hz * 2.0f64.powf(bend_octaves);
        let cents_hz = bent_hz * (2.0f64.powf(curve_offset_cents / 1200.0) - 1.0);
        bent_hz + cents_hz + mod_hz
    }

    /// Compute playback ratio relative to a source sample rate.
    ///
    /// `source_pitch_hz`: the pitch at which the source material was recorded.
    pub fn playback_ratio(
        &self,
        degree: i32,
        pitchbend: f64,
        curve_offset_cents: f64,
        mod_hz: f64,
        source_pitch_hz: f64,
    ) -> f64 {
        let target_hz = self.resolve(degree, pitchbend, curve_offset_cents, mod_hz);
        target_hz / source_pitch_hz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edo::Edo;

    fn default_pipeline() -> PitchPipeline {
        let edo = Arc::new(Edo::twelve_tet());
        // C4 at A4=440
        let c4 = 440.0 / 2.0f64.powf(9.0 / 12.0);
        PitchPipeline::new(edo, c4, 48.0)
    }

    #[test]
    fn a4_at_degree_9() {
        let p = default_pipeline();
        let hz = p.resolve(9, 0.0, 0.0, 0.0);
        assert!((hz - 440.0).abs() < 1e-9, "A4 must be 440 Hz, got {}", hz);
    }

    #[test]
    fn zero_bend_is_identity() {
        let p = default_pipeline();
        let hz_no_bend = p.resolve(0, 0.0, 0.0, 0.0);
        let hz_zero    = p.resolve(0, 0.0, 0.0, 0.0);
        assert_eq!(hz_no_bend, hz_zero);
    }

    #[test]
    fn full_bend_up_48_semitones() {
        let p = default_pipeline();
        let base = p.resolve(0, 0.0, 0.0, 0.0);
        let bent = p.resolve(0, 1.0, 0.0, 0.0);
        let expected = base * 2.0f64.powf(48.0 / 12.0);
        assert!((bent - expected).abs() < 1e-8);
    }
}
