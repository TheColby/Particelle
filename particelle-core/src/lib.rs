//! `particelle-core` — Engine nucleus.
//!
//! No I/O. No YAML. No hardware. No CLI.
//! This crate defines the fundamental audio model and grain engine types.

pub mod audio_block;
pub mod engine;
pub mod error;
pub mod grain;
pub mod layout;
pub mod pool;
pub mod spatializer;

pub use grain::{Cloud, Grain, GrainParams};
pub use pool::GrainPool;

/// Sample-accurate monotonic frame counter. All time is expressed in frames.
pub type FrameCount = u64;
