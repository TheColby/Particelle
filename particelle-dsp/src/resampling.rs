/// Resampling interface.
///
/// This module defines the `Resampler` trait and a placeholder implementation.
/// Phase 2 will integrate `rubato` for high-quality offline resampling.

/// Resampler trait: converts a planar f64 buffer from one sample rate to another.
pub trait Resampler: Send {
    /// Resample `input` (at `input_rate` Hz) to approximately `output_rate` Hz.
    /// Returns a new planar buffer.
    fn resample(
        &mut self,
        input: &[Vec<f64>],
        input_rate: f64,
        output_rate: f64,
    ) -> Vec<Vec<f64>>;
}

/// Placeholder resampler: returns input unchanged (identity when rates match).
/// TODO: Phase 2 — integrate `rubato` for production-quality FFT resampling.
pub struct IdentityResampler;

impl Resampler for IdentityResampler {
    fn resample(
        &mut self,
        input: &[Vec<f64>],
        _input_rate: f64,
        _output_rate: f64,
    ) -> Vec<Vec<f64>> {
        input.to_vec()
    }
}
