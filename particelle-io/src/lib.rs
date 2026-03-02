//! `particelle-io` — Audio file I/O and hardware audio backend.
//!
//! The `realtime` feature gates CPAL hardware support. Without it,
//! only offline file I/O is available.

pub mod file;
pub mod hardware;

pub use file::{AudioFileReader, AudioFileWriter, FileError};
pub use hardware::{HardwareConfig, HardwareError};
