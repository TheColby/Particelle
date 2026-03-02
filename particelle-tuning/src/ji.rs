use crate::Tuning;

/// A single rational ratio entry for Just Intonation.
#[derive(Debug, Clone)]
pub struct JiRatio {
    /// Scale degree within one octave (0-based).
    pub degree: u32,
    /// Numerator of the frequency ratio.
    pub num: u64,
    /// Denominator of the frequency ratio.
    pub den: u64,
}

impl JiRatio {
    pub fn ratio(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

/// Just Intonation tuning defined by a set of rational ratios.
///
/// Degree 0 maps to `base_frequency`. Degrees outside the defined octave
/// are folded: the octave number determines how many factors of 2.0 are applied.
#[derive(Debug, Clone)]
pub struct JiTuning {
    /// Sorted ratios for one octave. The first entry should have ratio 1/1 at degree 0.
    ratios: Vec<JiRatio>,
    /// Number of scale degrees per octave.
    scale_size: u32,
    /// Base frequency for degree 0 (e.g., 261.625 Hz for C4).
    pub base_frequency: f64,
}

impl JiTuning {
    pub fn new(ratios: Vec<JiRatio>, base_frequency: f64) -> Self {
        let scale_size = ratios.len() as u32;
        let mut sorted = ratios;
        sorted.sort_by_key(|r| r.degree);
        Self {
            ratios: sorted,
            scale_size,
            base_frequency,
        }
    }
}

impl Tuning for JiTuning {
    fn frequency_for_degree(&self, degree: i32) -> f64 {
        if self.ratios.is_empty() {
            return self.base_frequency;
        }

        let size = self.scale_size as i32;

        // Euclidean mod to get the degree within one octave
        let octave_degree = ((degree % size) + size) % size;
        // How many octaves above or below degree 0
        let octave = if degree >= 0 {
            degree / size
        } else {
            (degree - size + 1) / size
        };

        // Find the ratio for this degree within the octave
        let ratio = self.ratios
            .iter()
            .find(|r| r.degree == octave_degree as u32)
            .map(|r| r.ratio())
            .unwrap_or(1.0);

        // Apply octave transposition
        let octave_factor = 2.0_f64.powi(octave);

        self.base_frequency * ratio * octave_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pythagorean_scale() -> JiTuning {
        // A simple 7-note JI scale
        JiTuning::new(
            vec![
                JiRatio { degree: 0, num: 1, den: 1 },   // Unison
                JiRatio { degree: 1, num: 9, den: 8 },   // Major second
                JiRatio { degree: 2, num: 5, den: 4 },   // Major third
                JiRatio { degree: 3, num: 4, den: 3 },   // Perfect fourth
                JiRatio { degree: 4, num: 3, den: 2 },   // Perfect fifth
                JiRatio { degree: 5, num: 5, den: 3 },   // Major sixth
                JiRatio { degree: 6, num: 15, den: 8 },  // Major seventh
            ],
            261.625, // C4
        )
    }

    #[test]
    fn test_unison() {
        let t = pythagorean_scale();
        assert!((t.frequency_for_degree(0) - 261.625).abs() < 1e-6);
    }

    #[test]
    fn test_perfect_fifth() {
        let t = pythagorean_scale();
        // 3/2 * 261.625 = 392.4375
        let expected = 261.625 * 3.0 / 2.0;
        assert!((t.frequency_for_degree(4) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_octave_up() {
        let t = pythagorean_scale();
        // Degree 7 wraps to octave 1, degree 0 → 2 * base
        let expected = 261.625 * 2.0;
        assert!((t.frequency_for_degree(7) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_octave_down() {
        let t = pythagorean_scale();
        // Degree -7 wraps to octave -1, degree 0 → base / 2
        let expected = 261.625 / 2.0;
        assert!((t.frequency_for_degree(-7) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_negative_degree_ratio() {
        let t = pythagorean_scale();
        // Degree -3 → octave -1, degree 4 (perfect fifth) → (3/2) * base / 2
        let expected = 261.625 * (3.0 / 2.0) / 2.0;
        assert!((t.frequency_for_degree(-3) - expected).abs() < 1e-6);
    }
}
