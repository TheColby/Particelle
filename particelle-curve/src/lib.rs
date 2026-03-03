//! `particelle-curve` — JSON curve schema, compiled evaluators, and
//! control-rate reconstruction methods.
//!
//! This crate has no dependency on other `particelle-*` crates.
//! It can be compiled and tested in isolation.

pub mod evaluator;
pub mod reconstruction;
pub mod schema;

pub use evaluator::CompiledCurve;
pub use reconstruction::ReconstructionMethod;
pub use schema::{CurveSchema, EaseDir, Extrapolation, ExtrapolationMode, Segment, SegmentShape};
