//! `particelle-schema` — YAML schema types and validation layer.
//!
//! Parses YAML configuration files into typed Rust structs.
//! Validates cross-field constraints and returns typed error lists.
//! This is the only crate that handles YAML parsing.

pub mod compat;
pub mod config;
pub mod validation;

pub use compat::{
    normalize_yaml_value, parse_yaml_compat, parse_yaml_compat_with_report, CompatParseOutput,
    MigrationNote, MigrationReport,
};
pub use config::{
    ChannelConfig, CloudConfig, EngineConfig, HardwareConfig, JiRatioConfig, LayoutConfig,
    MidiBindingConfig, ParticelleConfig, RoutingConfig, SignalExprConfig, SignalOpConfig,
    TuningConfig, Vec3Config, WindowSpecConfig, CURRENT_SCHEMA_VERSION,
};
pub use validation::{validate, ValidationError};
