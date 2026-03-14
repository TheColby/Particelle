use thiserror::Error;

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

#[derive(Debug, Clone)]
struct CompiledSegment {
    seg: crate::schema::Segment,
    /// Tangents at x and x_end. Only used for Hermite-style interpolation.
    m0: f64,
    m1: f64,
}

/// A compiled curve ready for efficient evaluation.
#[derive(Debug)]
pub struct CompiledCurve {
    segments: Vec<CompiledSegment>,
    extrapolation: crate::schema::Extrapolation,
}

impl CompiledCurve {
    /// Parse a JSON string into a `CompiledCurve`.
    pub fn from_json(json: &str) -> Result<Self, CurveError> {
        let schema: crate::schema::CurveSchema = serde_json::from_str(json)?;
        Self::compile(schema)
    }

    /// Compile from a deserialized `CurveSchema`.
    pub fn compile(schema: crate::schema::CurveSchema) -> Result<Self, CurveError> {
        if schema.segments.is_empty() {
            return Err(CurveError::EmptySegments);
        }

        let mut segments = Vec::with_capacity(schema.segments.len());
        for (i, seg) in schema.segments.iter().enumerate() {
            if seg.x_end <= seg.x {
                return Err(CurveError::InvalidSegmentRange {
                    i,
                    x: seg.x,
                    x_end: seg.x_end,
                });
            }
            if i > 0 && seg.x < schema.segments[i - 1].x_end {
                return Err(CurveError::UnsortedSegments);
            }
            segments.push(CompiledSegment {
                seg: seg.clone(),
                m0: 0.0,
                m1: 0.0,
            });
        }

        // Monotone Cubic Spline (Fritsch-Carlson) tangent calculation
        // We only apply this to segments that explicitly request a spline-type shape
        // for simplicity, but we'll computesecant slopes (dk) for all.
        let mut d = Vec::with_capacity(segments.len());
        for seg in &segments {
            d.push((seg.seg.y_end - seg.seg.y) / (seg.seg.x_end - seg.seg.x));
        }

        let n = segments.len();
        let mut m = vec![0.0; n + 1];

        if n > 0 {
            m[0] = d[0];
            m[n] = d[n - 1];
            for i in 1..n {
                // Average of secants
                m[i] = (d[i - 1] + d[i]) / 2.0;

                // Adjust for monotonicity
                if d[i - 1] * d[i] <= 0.0 {
                    m[i] = 0.0;
                } else {
                    let _h_prev = segments[i - 1].seg.x_end - segments[i - 1].seg.x;
                    let _h_curr = segments[i].seg.x_end - segments[i].seg.x;
                    let alpha = m[i] / d[i - 1];
                    let beta = m[i] / d[i];
                    if alpha * alpha + beta * beta > 9.0 {
                        let tau = 3.0 / (alpha * alpha + beta * beta).sqrt();
                        m[i] *= tau;
                    }
                }
            }

            for i in 0..n {
                segments[i].m0 = m[i];
                segments[i].m1 = m[i + 1];
            }
        }

        Ok(Self {
            segments,
            extrapolation: schema.extrapolation,
        })
    }

    /// Evaluate the curve at position `x`.
    pub fn eval(&self, mut x: f64) -> f64 {
        let (x_min, x_max) = self.domain();
        let range = x_max - x_min;

        if x < x_min {
            match self.extrapolation.left {
                crate::schema::ExtrapolationMode::Clamp => {
                    return self.segments.first().unwrap().seg.y
                }
                crate::schema::ExtrapolationMode::Zero => return 0.0,
                crate::schema::ExtrapolationMode::Linear => {
                    let first = self.segments.first().unwrap();
                    let t = (x - first.seg.x) / (first.seg.x_end - first.seg.x);
                    return first.seg.y + t * (first.seg.y_end - first.seg.y); // Use linear slope of first segment
                }
                crate::schema::ExtrapolationMode::Repeat => {
                    x = x_max - (x_min - x) % range;
                    if x == x_min {
                        x = x_max;
                    }
                }
                crate::schema::ExtrapolationMode::Mirror => {
                    let dist = x_min - x;
                    let iteration = (dist / range).floor() as i64;
                    let rem = dist % range;
                    x = if iteration % 2 == 0 {
                        x_min + rem
                    } else {
                        x_max - rem
                    };
                }
            }
        } else if x > x_max {
            match self.extrapolation.right {
                crate::schema::ExtrapolationMode::Clamp => {
                    return self.segments.last().unwrap().seg.y_end
                }
                crate::schema::ExtrapolationMode::Zero => return 0.0,
                crate::schema::ExtrapolationMode::Linear => {
                    let last = self.segments.last().unwrap();
                    let t = (x - last.seg.x) / (last.seg.x_end - last.seg.x);
                    return last.seg.y + t * (last.seg.y_end - last.seg.y);
                }
                crate::schema::ExtrapolationMode::Repeat => {
                    x = x_min + (x - x_max) % range;
                }
                crate::schema::ExtrapolationMode::Mirror => {
                    let dist = x - x_max;
                    let iteration = (dist / range).floor() as i64;
                    let rem = dist % range;
                    x = if iteration % 2 == 0 {
                        x_max - rem
                    } else {
                        x_min + rem
                    };
                }
            }
        }

        // Find binary search or linear scan for segment
        // Binary search is better for many segments
        let mut low = 0;
        let mut high = self.segments.len() - 1;
        let mut idx = 0;

        while low <= high {
            let mid = (low + high) / 2;
            if x < self.segments[mid].seg.x {
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            } else if x > self.segments[mid].seg.x_end {
                low = mid + 1;
            } else {
                idx = mid;
                break;
            }
        }

        let cseg = &self.segments[idx];
        let seg = &cseg.seg;
        let mut t = (x - seg.x) / (seg.x_end - seg.x);
        t = t.clamp(0.0, 1.0);

        use crate::schema::SegmentShape;
        match &seg.shape {
            SegmentShape::Hold => seg.y,
            SegmentShape::Linear => seg.y + t * (seg.y_end - seg.y),
            SegmentShape::Smoothstep => {
                let s = t * t * (3.0 - 2.0 * t);
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::Smootherstep => {
                let s = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::Sine => {
                let s = (t * std::f64::consts::PI / 2.0).sin();
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::Cosine => {
                let s = 1.0 - (t * std::f64::consts::PI / 2.0).cos();
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::RaisedCosine => {
                let s = 0.5 * (1.0 - (t * std::f64::consts::PI).cos());
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::EaseQuad(dir) => seg.y + self.ease(t, 2.0, dir) * (seg.y_end - seg.y),
            SegmentShape::EaseCubic(dir) => seg.y + self.ease(t, 3.0, dir) * (seg.y_end - seg.y),
            SegmentShape::EaseQuart(dir) => seg.y + self.ease(t, 4.0, dir) * (seg.y_end - seg.y),
            SegmentShape::EaseQuint(dir) => seg.y + self.ease(t, 5.0, dir) * (seg.y_end - seg.y),
            SegmentShape::Exp { k } => {
                if *k == 0.0 {
                    seg.y + t * (seg.y_end - seg.y)
                } else {
                    let _s = (k.exp() * t).exp() / k.exp(); // Placeholder, usually exp is (exp(kt)-1)/(exp(k)-1)
                                                            // Let's use standard exp curve form:
                    let _s = (k.powf(t) - 1.0) / (*k - 1.0); // If k is the base
                                                             // Re-evaluating based on common audio exp:
                    let s = ((*k * t).exp() - 1.0) / (k.exp() - 1.0);
                    seg.y + s * (seg.y_end - seg.y)
                }
            }
            SegmentShape::Log { k } => {
                let s = ((*k * t + 1.0).ln()) / (k + 1.0).ln();
                seg.y + s * (seg.y_end - seg.y)
            }
            SegmentShape::Power { p } => seg.y + t.powf(*p) * (seg.y_end - seg.y),
            SegmentShape::MonotoneCubic | SegmentShape::CubicHermite | SegmentShape::CatmullRom => {
                // Cubic Hermite Spline: h00*y0 + h10*m0 + h01*y1 + h11*m1
                let h = seg.x_end - seg.x;
                let t2 = t * t;
                let t3 = t2 * t;
                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                let h10 = t3 - 2.0 * t2 + t;
                let h01 = -2.0 * t3 + 3.0 * t2;
                let h11 = t3 - t2;
                h00 * seg.y + h10 * h * cseg.m0 + h01 * seg.y_end + h11 * h * cseg.m1
            }
        }
    }

    fn ease(&self, t: f64, n: f64, dir: &crate::schema::EaseDir) -> f64 {
        use crate::schema::EaseDir;
        match dir {
            EaseDir::In => t.powf(n),
            EaseDir::Out => 1.0 - (1.0 - t).powf(n),
            EaseDir::InOut => {
                if t < 0.5 {
                    0.5 * (2.0 * t).powf(n)
                } else {
                    1.0 - 0.5 * (2.0 * (1.0 - t)).powf(n)
                }
            }
        }
    }

    /// The x domain (min, max) of this curve.
    pub fn domain(&self) -> (f64, f64) {
        let first = self.segments.first().unwrap();
        let last = self.segments.last().unwrap();
        (first.seg.x, last.seg.x_end)
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
    fn extrapolation_clamp() {
        let curve = CompiledCurve::from_json(LINEAR_JSON).unwrap();
        assert_eq!(curve.eval(-1.0), 0.0);
        assert_eq!(curve.eval(2.0), 1.0);
    }

    #[test]
    fn extrapolation_zero() {
        let json = r#"{
            "segments": [{ "x": 0.0, "y": 1.0, "x_end": 1.0, "y_end": 1.0, "shape": "linear" }],
            "extrapolation": { "left": "zero", "right": "zero" }
        }"#;
        let curve = CompiledCurve::from_json(json).unwrap();
        assert_eq!(curve.eval(-0.1), 0.0);
        assert_eq!(curve.eval(1.1), 0.0);
        assert_eq!(curve.eval(0.5), 1.0);
    }

    #[test]
    fn shape_smoothstep() {
        let json = r#"{
            "segments": [{ "x": 0.0, "y": 0.0, "x_end": 1.0, "y_end": 1.0, "shape": "smoothstep" }],
            "extrapolation": { "left": "clamp", "right": "clamp" }
        }"#;
        let curve = CompiledCurve::from_json(json).unwrap();
        assert!((curve.eval(0.5) - 0.5).abs() < 1e-14);
        assert!(curve.eval(0.2) < 0.2); // Smoothstep accelerates
    }

    #[test]
    fn shape_monotone_cubic() {
        let json = r#"{
            "segments": [
                { "x": 0.0, "y": 0.0, "x_end": 1.0, "y_end": 1.0, "shape": "monotone_cubic" },
                { "x": 1.0, "y": 1.0, "x_end": 2.0, "y_end": 0.0, "shape": "monotone_cubic" }
            ],
            "extrapolation": { "left": "clamp", "right": "clamp" }
        }"#;
        let curve = CompiledCurve::from_json(json).unwrap();
        assert_eq!(curve.eval(1.0), 1.0);
        assert!(curve.eval(0.5) > 0.0);
        assert!(curve.eval(1.5) > 0.0);
    }
}
