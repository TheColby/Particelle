//! `particelle-schema` — YAML schema types and validation layer.
//!
//! Parses YAML configuration files into typed Rust structs.
//! Validates cross-field constraints and returns typed error lists.
//! This is the only crate that handles YAML parsing.

pub mod config;
pub mod validation;

pub use config::{
    ParticelleConfig, EngineConfig, LayoutConfig, ChannelConfig,
    HardwareConfig, TuningConfig, JiRatioConfig, CloudConfig,
    SignalExprConfig, SignalOpConfig, WindowSpecConfig, Vec3Config,
    RoutingConfig, MidiBindingConfig,
};
pub use validation::{validate, ValidationError};
