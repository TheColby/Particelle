/// The core tuning interface.
///
/// Converts between integer scale degrees and frequencies in Hz.
/// All arithmetic is f64. No rounding occurs in this layer.
///
/// `base_hz` is the reference pitch for degree 0 (e.g., 261.63 for C4 at A4=440).
pub trait Tuning: Send + Sync {
    /// Convert a scale degree to a frequency in Hz.
    fn degree_to_freq(&self, degree: i32, base_hz: f64) -> f64;

    /// Convert a frequency in Hz to the nearest scale degree (may be fractional).
    fn freq_to_degree(&self, freq: f64, base_hz: f64) -> f64;

    /// Human-readable name for this tuning system.
    fn name(&self) -> &str;
}
