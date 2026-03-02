use serde::{Deserialize, Serialize};

/// All supported window function types.
///
/// Parameterized variants carry their shape parameters inline.
/// The `WindowCache` uses a JSON-serialized representation of this enum as
/// its cache key, avoiding the need to implement `Hash` over `Vec<f64>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowSpec {
    // --- Standard ---
    Rectangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
    Nuttall,
    FlatTop,
    Bartlett,
    Welch,
    Parzen,
    Sine,
    // --- Parameterized ---
    PowerSine      { power: f64 },
    Tukey          { alpha: f64 },
    PlanckTaper    { epsilon: f64 },
    Gaussian       { sigma: f64 },
    Kaiser         { beta: f64 },
    Kbd            { alpha: f64 },
    DolphChebyshev { attenuation_db: f64 },
    Dpss           { nw: f64 },
    Poisson        { alpha: f64 },
    Exponential    { tau: f64 },
    HannPoisson    { alpha: f64 },
    // --- Composite ---
    BartlettHann,
    HannSquared,
    HannCubed,
    RifeVincent    { order: u32 },
    GeneralizedBlackman { alpha: f64 },
    GeneralizedCosine   { coefficients: Vec<f64> },
    Trapezoid      { flat_top_ratio: f64 },
    HalfHann,
    HalfBlackman,
    AsymmetricTukey { left_alpha: f64, right_alpha: f64 },
    Lanczos,
    /// User-defined cosine sum: w[n] = Σ a_k · cos(2πkn/(N-1))
    CosineSum      { coefficients: Vec<f64> },
}

/// Normalization mode applied after window generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationMode {
    /// No normalization — raw window values.
    None,
    /// Normalize so the peak absolute value = 1.0.
    Peak,
    /// Normalize so the RMS of the window = 1.0.
    Rms,
    /// Normalize so the sum of the window = 1.0.
    Sum,
}
