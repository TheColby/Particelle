use crate::tuning::Tuning;

/// A single Just Intonation ratio: frequency = base_hz * (num / den).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JiRatio {
    pub degree: i32,
    pub num: u64,
    pub den: u64,
}

impl JiRatio {
    pub fn ratio(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

/// A Just Intonation scale defined by a table of rational ratios.
///
/// Degrees outside the defined table are wrapped by octave (ratio × 2^n).
#[derive(Debug, Clone)]
pub struct JiTable {
    /// Sorted by degree.
    pub ratios: Vec<JiRatio>,
    /// Number of degrees before the octave repeats.
    pub period: i32,
}

impl JiTable {
    pub fn new(ratios: Vec<JiRatio>) -> Self {
        let period = ratios.len() as i32;
        let mut sorted = ratios;
        sorted.sort_by_key(|r| r.degree);
        Self { ratios: sorted, period }
    }

    /// 5-limit 7-tone just major scale.
    pub fn just_major() -> Self {
        Self::new(vec![
            JiRatio { degree: 0, num: 1,  den: 1 },
            JiRatio { degree: 1, num: 9,  den: 8 },
            JiRatio { degree: 2, num: 5,  den: 4 },
            JiRatio { degree: 3, num: 4,  den: 3 },
            JiRatio { degree: 4, num: 3,  den: 2 },
            JiRatio { degree: 5, num: 5,  den: 3 },
            JiRatio { degree: 6, num: 15, den: 8 },
        ])
    }
}

impl Tuning for JiTable {
    fn degree_to_freq(&self, degree: i32, base_hz: f64) -> f64 {
        if self.ratios.is_empty() {
            return base_hz;
        }
        let period = self.period.max(1);
        let octave = degree.div_euclid(period);
        let d = degree.rem_euclid(period);
        let ratio = self.ratios
            .iter()
            .find(|r| r.degree == d)
            .map(|r| r.ratio())
            .unwrap_or(1.0);
        base_hz * ratio * 2.0f64.powi(octave)
    }

    fn freq_to_degree(&self, freq: f64, base_hz: f64) -> f64 {
        // Approximate: find closest ratio
        let ratio = freq / base_hz;
        let log2_ratio = ratio.log2();
        let octave = log2_ratio.floor() as i32;
        let frac = ratio / 2.0f64.powi(octave);
        let best = self.ratios.iter().min_by(|a, b| {
            let da = (a.ratio() - frac).abs();
            let db = (b.ratio() - frac).abs();
            da.partial_cmp(&db).unwrap()
        });
        best.map(|r| (octave * self.period + r.degree) as f64)
            .unwrap_or(0.0)
    }

    fn name(&self) -> &str {
        "JI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_fifth_ratio() {
        let ji = JiTable::just_major();
        let base = 440.0;
        let fifth = ji.degree_to_freq(4, base);
        let expected = base * 3.0 / 2.0;
        assert!((fifth - expected).abs() < 1e-10);
    }
}
