use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

pub fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// Root configuration. This is the single source of truth for a Particelle patch.
///
/// Deserialized from YAML. All engine behavior is declared here.
/// No parameter is hidden or configurable any other way.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticelleConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
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
    #[serde(default)]
    pub analysis: Vec<AnalysisConfig>,
}

/// Core engine configuration. Fixed at initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct HardwareConfig {
    pub device_name: Option<String>,
    #[serde(default)]
    pub midi_input: Option<String>,
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
#[serde(deny_unknown_fields)]
pub struct LayoutConfig {
    pub channels: Vec<ChannelConfig>,
}

/// A single output channel with spatial metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
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
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct JiRatioConfig {
    pub degree: u32,
    pub num: u64,
    pub den: u64,
}

/// A cloud configuration block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudConfig {
    pub id: String,
    /// Source audio: a file path or `"input"` for duplex.
    pub source: String,
    pub density: SignalExprConfig,
    pub duration: SignalExprConfig,
    pub position: SignalExprConfig,
    pub amplitude: SignalExprConfig,
    pub window: WindowSpecConfig,
    #[serde(default = "default_listener_pos")]
    pub listener_pos: Vec3Config,
    pub width: SignalExprConfig,
    #[serde(default = "default_directivity")]
    pub directivity: SignalExprConfig,
    #[serde(default = "default_orientation")]
    pub orientation_azimuth: SignalExprConfig,
    #[serde(default = "default_orientation")]
    pub orientation_elevation: SignalExprConfig,
    #[serde(default)]
    pub max_particles: Option<usize>,
    /// When true (default), grain read-heads wrap around when they pass the end
    /// of the source buffer so the cloud plays continuously regardless of render duration.
    #[serde(default = "default_loop_source")]
    pub loop_source: bool,
    /// Rate multiplier, sampled at grain creation. Negative values reverse playback.
    #[serde(default = "default_playback_rate")]
    pub playback_rate: SignalExprConfig,
    /// Optional absolute tuning: rate *= frequency(degree, reference_hz) / source_hz.
    #[serde(default)]
    pub pitch: Option<PitchConfig>,
}

fn default_loop_source() -> bool {
    true
}
fn default_directivity() -> SignalExprConfig {
    SignalExprConfig::Const(1.0)
}
fn default_orientation() -> SignalExprConfig {
    SignalExprConfig::Const(0.0)
}
fn default_listener_pos() -> Vec3Config {
    Vec3Config {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Vec3Config {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// An offline audio feature analysis block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisConfig {
    pub id: String,
    pub source: String,
    pub extractor: String,
}

/// MIDI/control routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RoutingConfig {
    #[serde(default)]
    pub midi_bindings: Vec<MidiBindingConfig>,
}

/// A single MIDI-to-parameter binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MidiBindingConfig {
    /// Source, e.g. `"midi.cc.1"`, `"mpe.pressure"`.
    pub source: String,
    /// Target parameter path, e.g. `"cloud.shimmer.density"`.
    pub target: String,
    #[serde(default)]
    pub transform: Option<SignalExprConfig>,
}

/// Degree zero is reference_hz (440 by default); source_hz is the recorded
/// source's fundamental, not its sample rate. Scala KBM sets the absolute base
/// from its reference key; degree still denotes a scale degree, not a MIDI key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchConfig {
    pub degree: i32,
    #[serde(default = "default_reference_hz")]
    pub reference_hz: f64,
    pub source_hz: f64,
}
fn default_reference_hz() -> f64 {
    440.0
}
fn default_playback_rate() -> SignalExprConfig {
    SignalExprConfig::Const(1.0)
}
