//! `particelle-schema` — YAML schema types and validation layer.
//!
//! Parses YAML configuration files into typed Rust structs.
//! Validates cross-field constraints and returns typed error lists.
//! This is the only crate that handles YAML parsing.

pub mod config;
pub mod validation;

pub use config::{
    ChannelConfig, CloudConfig, EngineConfig, HardwareConfig, JiRatioConfig, LayoutConfig,
    MidiBindingConfig, ParticelleConfig, RoutingConfig, SignalExprConfig, SignalOpConfig,
    TuningConfig, Vec3Config, WindowSpecConfig,
};
pub use validation::{validate, ValidationError};
