use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration. This is the single source of truth for a Particelle patch.
///
/// Deserialized from YAML. All engine behavior is declared here.
/// No parameter is hidden or configurable any other way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticelleConfig {
    pub engine: EngineConfig,
    #[serde(default)]
    pub hardware: Option<HardwareConfig>,
    pub layout: LayoutConfig,
    #[serde(default)]
    pub tuning: TuningConfig,
    #[serde(default)]
    pub clouds: Vec<CloudConfig>,
    #[serde(default)]
    pub routing: RoutingConfig,
}

/// Core engine configuration. Fixed at initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub sample_rate: f64,
    pub block_size: usize,
    #[serde(default = "default_max_particles")]
    pub max_particles_per_cloud: usize,
}

fn default_max_particles() -> usize {
    4096
}

/// Hardware audio device configuration (realtime mode only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub device_name: Option<String>,
    #[serde(default = "default_latency_ms")]
    pub latency_ms: f64,
    #[serde(default)]
    pub duplex: bool,
}

fn default_latency_ms() -> f64 {
    10.0
}

/// Declarative multichannel output layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub channels: Vec<ChannelConfig>,
}

/// A single output channel with spatial metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChannelConfig {
    Spherical {
        name: String,
        azimuth_deg: f64,
        #[serde(default)]
        elevation_deg: f64,
    },
    Cartesian {
        name: String,
        x: f64,
        y: f64,
        z: f64,
    },
}

/// Tuning system selection and configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum TuningConfig {
    /// 12-tone equal temperament (default).
    #[default]
    TwelveTet,
    /// Arbitrary equal divisions of the octave.
    Edo { steps: u32 },
    /// Fixed Just Intonation via rational ratios.
    Ji { ratios: Vec<JiRatioConfig> },
    /// Scala .scl and optional .kbm files.
    Scala {
        scl_path: String,
        kbm_path: Option<String>,
    },
}

/// A single JI ratio entry for TuningConfig::Ji.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiRatioConfig {
    pub degree: u32,
    pub num: u64,
    pub den: u64,
}

/// A cloud configuration block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub id: String,
    /// Source audio: a file path or `"input"` for duplex.
    pub source: String,
    pub density: SignalExprConfig,
    pub duration: SignalExprConfig,
    pub position: SignalExprConfig,
    pub amplitude: SignalExprConfig,
    pub window: WindowSpecConfig,
    pub listener_pos: Vec3Config,
    pub width: SignalExprConfig,
    #[serde(default)]
    pub max_particles: Option<usize>,
}

/// An inline signal expression: a constant, a named reference, or an expression tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalExprConfig {
    Const(f64),
    Ref(String), // e.g. "$midi_cc1" or "curves/density.json"
    Expr(SignalOpConfig),
}

/// An expression node with an operator and arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOpConfig {
    pub op: String,
    pub args: Vec<SignalExprConfig>,
}

/// A window specification in YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSpecConfig {
    #[serde(rename = "type")]
    pub kind: String,
    /// Flattened extra parameters (e.g., `{ beta: 8.6 }` for Kaiser).
    #[serde(flatten, default)]
    pub params: HashMap<String, serde_json::Value>,
}

/// 3D listener-space position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec3Config {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// MIDI/control routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default)]
    pub midi_bindings: Vec<MidiBindingConfig>,
}

/// A single MIDI-to-parameter binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiBindingConfig {
    /// Source, e.g. `"midi.cc.1"`, `"mpe.pressure"`.
    pub source: String,
    /// Target parameter path, e.g. `"cloud.shimmer.density"`.
    pub target: String,
    #[serde(default)]
    pub transform: Option<SignalExprConfig>,
}
