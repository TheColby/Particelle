/// Interpolation primitives. All operate on f64.
///
/// These are stateless pure functions. No allocation occurs.

/// Linearly interpolate between `a` and `b` at normalized position `t` in [0, 1].
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Cubic Hermite interpolation.
///
/// `p0`, `p1`: values at x=0 and x=1.
/// `m0`, `m1`: tangents at x=0 and x=1.
/// `t`: position in [0, 1].
#[inline]
pub fn cubic_hermite(p0: f64, m0: f64, p1: f64, m1: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

/// Catmull-Rom spline interpolation through four evenly-spaced points.
///
/// `p0`..`p3` are values at x = -1, 0, 1, 2.
/// `t`: position in [0, 1] between p1 and p2.
#[inline]
pub fn catmull_rom(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let m1 = 0.5 * (p2 - p0);
    let m2 = 0.5 * (p3 - p1);
    cubic_hermite(p1, m1, p2, m2, t)
}

/// Monotone cubic interpolation (Fritsch-Carlson).
///
/// Guarantees no overshoot between `p1` and `p2` given neighbouring points.
#[inline]
pub fn monotone_cubic(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
    let h1 = p2 - p1;
    let h0 = p1 - p0;
    let h2 = p3 - p2;

    let delta1 = if h1 == 0.0 { 0.0 } else { h0 / h1 };
    let delta2 = if h1 == 0.0 { 0.0 } else { h2 / h1 };

    // Fritsch-Carlson tangent conditions
    let m1 = {
        let s = 0.5 * (h0 + h1);
        if s.abs() < 1e-15 { 0.0 } else { 0.5 * (delta1 + 1.0) * h0 / s }
    };
    let m2 = {
        let s = 0.5 * (h1 + h2);
        if s.abs() < 1e-15 { 0.0 } else { 0.5 * (1.0 + delta2) * h2 / s }
    };

    cubic_hermite(p1, m1, p2, m2, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints() {
        assert_eq!(lerp(0.0, 1.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 1.0, 1.0), 1.0);
        assert!((lerp(0.0, 1.0, 0.5) - 0.5).abs() < 1e-15);
    }

    #[test]
    fn cubic_hermite_endpoints() {
        assert!((cubic_hermite(0.0, 0.0, 1.0, 0.0, 0.0) - 0.0).abs() < 1e-14);
        assert!((cubic_hermite(0.0, 0.0, 1.0, 0.0, 1.0) - 1.0).abs() < 1e-14);
    }
}
