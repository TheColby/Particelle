use thiserror::Error;
use crate::schema::CurveSchema;

/// Errors produced during curve compilation.
#[derive(Debug, Error)]
pub enum CurveError {
    #[error("Curve has no segments")]
    EmptySegments,
    #[error("Segment [{i}] has x_end ({x_end}) <= x ({x})")]
    InvalidSegmentRange { i: usize, x: f64, x_end: f64 },
    #[error("Segments are not sorted by x")]
    UnsortedSegments,
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A compiled curve ready for efficient evaluation.
///
/// After compilation, `eval` is a pure function with no allocation and
/// no JSON parsing. The curve is fully defined by the `CurveSchema` baked
/// into this struct. Compilation validates the schema and precomputes any
/// necessary coefficients.
///
/// `CompiledCurve` is `Send + Sync`; share it between threads via `Arc`.
pub struct CompiledCurve {
    schema: CurveSchema,
}

impl CompiledCurve {
    /// Parse a JSON string into a `CompiledCurve`.
    pub fn from_json(json: &str) -> Result<Self, CurveError> {
        let schema: CurveSchema = serde_json::from_str(json)?;
        Self::compile(schema)
    }

    /// Compile from a deserialized `CurveSchema`.
    pub fn compile(schema: CurveSchema) -> Result<Self, CurveError> {
        if schema.segments.is_empty() {
            return Err(CurveError::EmptySegments);
        }
        for (i, seg) in schema.segments.iter().enumerate() {
            if seg.x_end <= seg.x {
                return Err(CurveError::InvalidSegmentRange {
                    i,
                    x: seg.x,
                    x_end: seg.x_end,
                });
            }
        }
        // TODO: Phase 1 — precompute Hermite / spline coefficients per segment
        Ok(Self { schema })
    }

    /// Evaluate the curve at position `x`.
    ///
    /// This function must not allocate. Extrapolation is applied when `x` is
    /// outside the defined segment domain.
    pub fn eval(&self, x: f64) -> f64 {
        // TODO: Phase 1 — full shape dispatch and extrapolation
        // Placeholder: linear interpolation across the full segment list
        if let Some(first) = self.schema.segments.first() {
            if x <= first.x {
                return first.y;
            }
        }
        if let Some(last) = self.schema.segments.last() {
            if x >= last.x_end {
                return last.y_end;
            }
        }
        for seg in &self.schema.segments {
            if x >= seg.x && x <= seg.x_end {
                let t = (x - seg.x) / (seg.x_end - seg.x);
                return seg.y + t * (seg.y_end - seg.y);
            }
        }
        0.0
    }

    /// The x domain (min, max) of this curve.
    pub fn domain(&self) -> (f64, f64) {
        let first = self.schema.segments.first().unwrap();
        let last = self.schema.segments.last().unwrap();
        (first.x, last.x_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR_JSON: &str = r#"
    {
        "segments": [{ "x": 0.0, "y": 0.0, "x_end": 1.0, "y_end": 1.0, "shape": "linear" }],
        "extrapolation": { "left": "clamp", "right": "clamp" }
    }"#;

    #[test]
    fn linear_curve_midpoint() {
        let curve = CompiledCurve::from_json(LINEAR_JSON).unwrap();
        assert!((curve.eval(0.5) - 0.5).abs() < 1e-14);
    }

    #[test]
    fn linear_curve_endpoints() {
        let curve = CompiledCurve::from_json(LINEAR_JSON).unwrap();
        assert_eq!(curve.eval(0.0), 0.0);
        assert_eq!(curve.eval(1.0), 1.0);
    }

    #[test]
    fn empty_segments_errors() {
        let json = r#"{"segments":[],"extrapolation":{"left":"clamp","right":"clamp"}}"#;
        assert!(CompiledCurve::from_json(json).is_err());
    }
}
