use serde::{Deserialize, Serialize};

fn default_tukey_alpha() -> f64 {
    0.5
}

fn default_planck_epsilon() -> f64 {
    0.1
}

fn default_dpss_half_bandwidth() -> f64 {
    4.0
}

/// Specifies the type of mathematical window and any required parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowSpec {
    Rectangular,
    #[default]
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
    Gaussian {
        sigma: f64,
    },
    Tukey {
        #[serde(default = "default_tukey_alpha")]
        alpha: f64,
    },
    PlanckTaper {
        #[serde(default = "default_planck_epsilon")]
        epsilon: f64,
    },
    PlanckBessel {
        epsilon: f64,
        alpha: f64,
    },
    Kaiser {
        beta: f64,
    },
    DolphChebyshev {
        attenuation_db: f64,
    },
    Poisson {
        alpha: f64,
    },
    HannPoisson {
        alpha: f64,
    },
    Cauchy {
        alpha: f64,
    },
    Welch,
    Parzen,
    Dpss {
        #[serde(default = "default_dpss_half_bandwidth")]
        half_bandwidth: f64,
    },
    TukeyHarris {
        alpha: f64,
    },
    NuttallGaussian {
        sigma: f64,
    },
    RifeVincent1 {
        order: usize,
    },
    RifeVincent2 {
        order: usize,
    },
    RifeVincent3 {
        order: usize,
    },
    GeneralizedCosine {
        coeffs: Vec<f64>,
    },
    Symmetric {
        base: Box<WindowSpec>,
    },
    Asymmetric {
        left: Box<WindowSpec>,
        right: Box<WindowSpec>,
    },
    HalfLeft {
        base: Box<WindowSpec>,
    },
    HalfRight {
        base: Box<WindowSpec>,
    },
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
