//! `particelle-dsp` — DSP primitives: windows, interpolation, smoothing, resampling.
//!
//! No I/O. No YAML. Window math is stubbed; all types and registry interfaces
//! are fully defined.

pub mod window;
pub mod interpolation;
pub mod smoothing;
pub mod resampling;
