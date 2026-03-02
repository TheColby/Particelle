use serde::{Deserialize, Serialize};

/// Control-rate to audio-rate reconstruction methods.
///
/// A curve produces values at control rate (e.g., every 64 or 256 samples).
/// The reconstruction method defines how those sparse values are densified
/// to the audio sample rate before entering the DSP graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum ReconstructionMethod {
    /// Zero-order hold: hold control value until the next update.
    Zoh,
    /// Linear interpolation between control points.
    Linear,
    /// Cubic polynomial interpolation.
    Cubic,
    /// Monotone cubic (Fritsch-Carlson): cubic with no overshoot.
    MonotoneCubic,
    /// Windowed sinc interpolation. `taps` must be even and > 0.
    Sinc { taps: usize },
    /// Single-pole IIR low-pass. `coefficient` in [0, 1).
    OnePole { coefficient: f64 },
    /// Two-pole IIR. `a1`, `a2` are the feedback coefficients.
    TwoPole { a1: f64, a2: f64 },
    /// Slew limiter: bounds the per-sample rate of change.
    SlewLimiter { max_rate: f64 },
    /// Minimum-phase BLEP step for click-free discontinuities.
    MinblepStep,
}

impl Default for ReconstructionMethod {
    fn default() -> Self {
        Self::Linear
    }
}
