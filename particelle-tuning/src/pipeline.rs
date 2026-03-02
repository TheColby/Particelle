use crate::Tuning;

/// The unified pitch pipeline.
///
/// Converts a scale degree through the full chain:
/// `degree → tuning Hz → MPE pitchbend → curve offset → modulation → final Hz`
///
/// The result can be divided by a base frequency to obtain the playback ratio
/// for grain readers.
pub struct PitchPipeline {
    tuning: Box<dyn Tuning>,
}

impl PitchPipeline {
    pub fn new(tuning: Box<dyn Tuning>) -> Self {
        Self { tuning }
    }

    /// Compute the final frequency in Hz for a given degree with modifiers.
    ///
    /// - `degree`: Integer scale degree (0 = base note).
    /// - `pitchbend_semitones`: MPE pitch bend expressed in semitones (can be fractional).
    /// - `curve_offset_hz`: Additive offset from a control curve, in Hz.
    /// - `modulation_hz`: Additive modulation offset, in Hz.
    pub fn compute_hz(
        &self,
        degree: i32,
        pitchbend_semitones: f64,
        curve_offset_hz: f64,
        modulation_hz: f64,
    ) -> f64 {
        let base_hz = self.tuning.frequency_for_degree(degree);

        // Apply pitchbend: multiply by 2^(semitones/12)
        let bent_hz = base_hz * 2.0_f64.powf(pitchbend_semitones / 12.0);

        // Additive offsets
        let final_hz = bent_hz + curve_offset_hz + modulation_hz;

        // Never return <= 0 Hz
        final_hz.max(f64::EPSILON)
    }

    /// Compute the playback ratio relative to a reference frequency.
    ///
    /// `reference_hz` is the base sample rate frequency (e.g., the frequency at which
    /// the source audio was recorded).
    pub fn compute_ratio(
        &self,
        degree: i32,
        pitchbend_semitones: f64,
        curve_offset_hz: f64,
        modulation_hz: f64,
        reference_hz: f64,
    ) -> f64 {
        let hz = self.compute_hz(degree, pitchbend_semitones, curve_offset_hz, modulation_hz);
        hz / reference_hz
    }

    /// Access the underlying tuning system.
    pub fn tuning(&self) -> &dyn Tuning {
        self.tuning.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edo::EdoTuning;

    #[test]
    fn test_no_modifiers() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        let hz = pipeline.compute_hz(0, 0.0, 0.0, 0.0);
        assert!((hz - 440.0).abs() < 1e-6);
    }

    #[test]
    fn test_pitchbend_up_octave() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        // 12 semitones bend up = one octave
        let hz = pipeline.compute_hz(0, 12.0, 0.0, 0.0);
        assert!((hz - 880.0).abs() < 1e-6);
    }

    #[test]
    fn test_pitchbend_down() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        // -12 semitones = one octave down
        let hz = pipeline.compute_hz(0, -12.0, 0.0, 0.0);
        assert!((hz - 220.0).abs() < 1e-6);
    }

    #[test]
    fn test_additive_offsets() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        let hz = pipeline.compute_hz(0, 0.0, 10.0, 5.0);
        assert!((hz - 455.0).abs() < 1e-6);
    }

    #[test]
    fn test_never_negative() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        let hz = pipeline.compute_hz(0, 0.0, -1000.0, 0.0);
        assert!(hz > 0.0);
    }

    #[test]
    fn test_playback_ratio() {
        let pipeline = PitchPipeline::new(Box::new(EdoTuning::twelve_tet(440.0)));
        // Degree 0 at 440Hz, reference 440Hz → ratio 1.0
        let ratio = pipeline.compute_ratio(0, 0.0, 0.0, 0.0, 440.0);
        assert!((ratio - 1.0).abs() < 1e-6);

        // Degree 12 → 880Hz / 440Hz = 2.0
        let ratio = pipeline.compute_ratio(12, 0.0, 0.0, 0.0, 440.0);
        assert!((ratio - 2.0).abs() < 1e-6);
    }
}
