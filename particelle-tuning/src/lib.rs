pub mod edo;
pub mod ji;
pub mod pipeline;
pub mod scala;

pub use edo::EdoTuning;
pub use ji::JiTuning;
pub use pipeline::PitchPipeline;
pub use scala::ScalaTuning;

/// Abstract tuning system capable of computing the frequency (in Hz) for a given scale degree.
pub trait Tuning: Send + Sync {
    /// Compute the frequency in Hz for the given `degree` integer.
    /// `degree` = 0 typically denotes the base frequency (e.g., A4 or C4).
    fn frequency_for_degree(&self, degree: i32) -> f64;
}
