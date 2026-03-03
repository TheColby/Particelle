//! `particelle-dsp` — DSP primitives: windows, interpolation, smoothing, resampling.
//!
//! No I/O. No YAML. Window math is stubbed; all types and registry interfaces
//! are fully defined.

pub mod interpolation;
pub mod resampling;
pub mod smoothing;
pub mod window;
