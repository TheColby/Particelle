use thiserror::Error;

/// Errors produced by the core engine layer.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Buffer size mismatch: expected {expected} frames, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },

    #[error("Channel count mismatch: expected {expected}, got {actual}")]
    ChannelCountMismatch { expected: usize, actual: usize },

    #[error("Invalid sample rate: {rate}")]
    InvalidSampleRate { rate: f64 },

    #[error("Invalid block size: {size}")]
    InvalidBlockSize { size: usize },

    #[error("Cloud not found: id={id}")]
    CloudNotFound { id: u64 },

    #[error("Particle pool exhausted: cloud_id={cloud_id}, max={max}")]
    ParticlePoolExhausted { cloud_id: u64, max: usize },
}
