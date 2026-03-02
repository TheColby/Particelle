use serde::{Deserialize, Serialize};

/// A complete curve definition, deserialized from JSON.
///
/// Curves are deserialized once from JSON files before rendering begins.
/// Inside the audio loop, only `CompiledCurve::eval` is called — no parsing,
/// no allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveSchema {
    pub segments: Vec<Segment>,
    pub extrapolation: Extrapolation,
    #[serde(default)]
    pub events: Vec<DiscreteEvent>,
}

/// A single curve segment from x to x_end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub x: f64,
    pub y: f64,
    pub x_end: f64,
    pub y_end: f64,
    pub shape: SegmentShape,
}

/// Interpolation shape for a curve segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentShape {
    Hold,
    Linear,
    Smoothstep,
    Smootherstep,
    Sine,
    Cosine,
    RaisedCosine,
    EaseQuad(EaseDir),
    EaseCubic(EaseDir),
    EaseQuart(EaseDir),
    EaseQuint(EaseDir),
    Exp { k: f64 },
    Log { k: f64 },
    Power { p: f64 },
    CatmullRom,
    CubicHermite,
    MonotoneCubic,
}

/// Direction for ease family segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EaseDir {
    In,
    Out,
    InOut,
}

/// Extrapolation rules for values outside the curve's defined x range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrapolation {
    pub left: ExtrapolationMode,
    pub right: ExtrapolationMode,
}

impl Default for Extrapolation {
    fn default() -> Self {
        Self {
            left: ExtrapolationMode::Clamp,
            right: ExtrapolationMode::Clamp,
        }
    }
}

/// How to extrapolate beyond the curve's defined domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtrapolationMode {
    /// Hold the boundary value.
    Clamp,
    /// Repeat the curve from the beginning.
    Repeat,
    /// Mirror-repeat the curve.
    Mirror,
    /// Extrapolate linearly from the boundary tangent.
    Linear,
    /// Return 0.0 outside the defined domain.
    Zero,
}

/// A discrete event embedded in a curve (e.g., trigger, label marker).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscreteEvent {
    pub x: f64,
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
}
