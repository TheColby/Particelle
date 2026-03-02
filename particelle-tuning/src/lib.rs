//! `particelle-tuning` — Microtonal tuning systems and pitch pipeline.
//!
//! Supports EDO (arbitrary divisions of the octave), fixed Just Intonation,
//! and Scala format (.scl + .kbm). All pitch arithmetic is f64.

pub mod edo;
pub mod ji;
pub mod pipeline;
pub mod scala;
pub mod tuning;

pub use edo::Edo;
pub use ji::{JiTable, JiRatio};
pub use pipeline::PitchPipeline;
pub use scala::{ScalaScale, KbmMapping};
pub use tuning::Tuning;
