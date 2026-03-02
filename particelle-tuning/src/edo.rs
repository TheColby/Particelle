use crate::tuning::Tuning;

/// Equal Division of the Octave (EDO) tuning.
///
/// Any positive number of steps per octave is supported.
/// 12-EDO = standard equal temperament.
/// 31-EDO, 53-EDO, etc. are natively supported.
#[derive(Debug, Clone)]
pub struct Edo {
    /// Number of equal steps per octave.
    pub steps: u32,
}

impl Edo {
    pub fn new(steps: u32) -> Self {
        assert!(steps > 0, "EDO steps must be > 0");
        Self { steps }
    }

    /// Standard 12-tone equal temperament.
    pub fn twelve_tet() -> Self {
        Self { steps: 12 }
    }
}

impl Tuning for Edo {
    fn degree_to_freq(&self, degree: i32, base_hz: f64) -> f64 {
        base_hz * 2.0f64.powf(degree as f64 / self.steps as f64)
    }

    fn freq_to_degree(&self, freq: f64, base_hz: f64) -> f64 {
        (freq / base_hz).log2() * self.steps as f64
    }

    fn name(&self) -> &str {
        "EDO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_edo_a4_440() {
        let edo = Edo::twelve_tet();
        // A4 is 9 semitones above C4 (261.6255...)
        let c4 = 440.0 / 2.0f64.powf(9.0 / 12.0);
        let a4 = edo.degree_to_freq(9, c4);
        assert!((a4 - 440.0).abs() < 1e-10, "12-EDO A4 must be 440.0 Hz, got {}", a4);
    }

    #[test]
    fn octave_is_double() {
        let edo = Edo::new(31);
        let base = 220.0;
        let octave = edo.degree_to_freq(31, base);
        assert!((octave - 440.0).abs() < 1e-10);
    }

    #[test]
    fn roundtrip_degree_freq() {
        let edo = Edo::twelve_tet();
        let base = 261.625_565_300_598_6;
        for degree in -12i32..=24 {
            let freq = edo.degree_to_freq(degree, base);
            let back = edo.freq_to_degree(freq, base);
            assert!((back - degree as f64).abs() < 1e-9);
        }
    }
}
