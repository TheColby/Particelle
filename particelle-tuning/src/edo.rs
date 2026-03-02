use crate::Tuning;

/// Equal Divisions of the Octave (EDO).
/// Represents tunings like 12-TET, 31-EDO, etc.
#[derive(Debug, Clone)]
pub struct EdoTuning {
    /// Number of equal steps in an octave.
    pub steps: u32,
    /// The base frequency for degree 0 (usually A4 = 440.0Hz).
    pub base_frequency: f64,
}

impl EdoTuning {
    pub fn new(steps: u32, base_frequency: f64) -> Self {
        Self {
            steps,
            base_frequency,
        }
    }

    /// Convenience constructor for standard 12-Tone Equal Temperament mapping.
    /// Degree 0 is A4 (440Hz). Degree -9 points to Middle C (~261.63Hz).
    pub fn twelve_tet(base_frequency: f64) -> Self {
        Self::new(12, base_frequency)
    }
}

impl Tuning for EdoTuning {
    fn frequency_for_degree(&self, degree: i32) -> f64 {
        // formula: f0 * 2^(degree / steps)
        let ratio = 2.0_f64.powf(degree as f64 / self.steps as f64);
        self.base_frequency * ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twelve_tet() {
        let tuning = EdoTuning::twelve_tet(440.0);
        
        // Degree 0 is A4 = 440
        assert!((tuning.frequency_for_degree(0) - 440.0).abs() < 1e-6);

        // Degree 12 is A5 = 880
        assert!((tuning.frequency_for_degree(12) - 880.0).abs() < 1e-6);

        // Degree -12 is A3 = 220
        assert!((tuning.frequency_for_degree(-12) - 220.0).abs() < 1e-6);

        // Degree -9 is Middle C (~261.625 Hz)
        assert!((tuning.frequency_for_degree(-9) - 261.625565).abs() < 1e-5);
    }

    #[test]
    fn test_31_edo() {
        let tuning = EdoTuning::new(31, 440.0);
        
        // Degree 31 is an octave up
        assert!((tuning.frequency_for_degree(31) - 880.0).abs() < 1e-6);
        
        // Degree 1 is a dieselis interval in 31-EDO
        let step_ratio = 2.0_f64.powf(1.0 / 31.0);
        assert!((tuning.frequency_for_degree(1) - (440.0 * step_ratio)).abs() < 1e-6);
    }
}
