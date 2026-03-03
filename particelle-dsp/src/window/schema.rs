use serde::{Deserialize, Serialize};

/// Specifies the type of mathematical window and any required parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowSpec {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
    BlackmanNuttall,
    Nuttall,
    FlatTop,
    Bartlett,
    BartlettHann,
    Bohman,
    Cosine,
    Sine,
    Lanczos,
    Gaussian { sigma: f64 },
    Tukey { alpha: f64 },
    PlanckTaper { epsilon: f64 },
    PlanckBessel { epsilon: f64, alpha: f64 },
    Kaiser { beta: f64 },
    DolphChebyshev { attenuation_db: f64 },
    Poisson { alpha: f64 },
    HannPoisson { alpha: f64 },
    Cauchy { alpha: f64 },
    Welch,
    Parzen,
    TukeyHarris { alpha: f64 },
    NuttallGaussian { sigma: f64 },
    RifeVincent1 { order: usize },
    RifeVincent2 { order: usize },
    RifeVincent3 { order: usize },
    GeneralizedCosine { coeffs: Vec<f64> },
    Symmetric { base: Box<WindowSpec> },
    Asymmetric { left: Box<WindowSpec>, right: Box<WindowSpec> },
    HalfLeft { base: Box<WindowSpec> },
    HalfRight { base: Box<WindowSpec> },
}

impl Default for WindowSpec {
    fn default() -> Self {
        Self::Hann
    }
}

/// Strategy for normalizing the output window slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowNormalization {
    /// No scaling applied. Return raw mathematical values.
    None,
    /// Scale so the maximum peak is exactly 1.0 (Default).
    #[default]
    Peak,
    /// Scale so the Area Under Curve (Sum) is exactly 1.0. 
    /// Common in spectral analysis to preserve energy.
    Sum,
    /// Scale so the Root Mean Square is 1.0.
    Rms,
}

impl WindowSpec {
    /// Recursively resolve whether a window requires two passes to compute
    /// (e.g. Asymmetric where left and right parts are computed separately).
    pub fn is_composite(&self) -> bool {
        matches!(self, Self::Asymmetric { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_composite() {
        let asymmetric = WindowSpec::Asymmetric {
            left: Box::new(WindowSpec::Hann),
            right: Box::new(WindowSpec::Rectangular),
        };
        assert!(asymmetric.is_composite());

        let hann = WindowSpec::Hann;
        assert!(!hann.is_composite());

        let symmetric = WindowSpec::Symmetric {
            base: Box::new(WindowSpec::Hann),
        };
        assert!(!symmetric.is_composite());
    }
}
